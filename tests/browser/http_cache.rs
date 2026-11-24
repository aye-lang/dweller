use super::*;

fn typed(url: &str) -> String {
    url.as_bytes()
        .chunks(30)
        .map(|c| format!("text:{}", String::from_utf8_lossy(c)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn visit(first: &str, then: &[String], site: bool) -> String {
    visit_with(first, then, site, &[])
}

fn visit_with(first: &str, then: &[String], site: bool, env: &[(&str, &str)]) -> String {
    let loaded = if site { "renderer:loaded" } else { "loaded" };
    let mut events = String::from(loaded);
    for url in then {
        events.push_str(&format!(" key:cmd+l {} key:return {loaded}", typed(url)));
    }
    let mut command = dweller_command();
    if site {
        command.env("DWELLER_PROCESS", "site");
    }
    let out = command
        .envs(env.iter().copied())
        .current_dir(root())
        .args([first, "--frames", "600", "--until-idle", "--state", "--text", "--events", &events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    text(&out.stdout)
}

fn revalidates(site: bool) {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let a = format!("{}/net/cache-a.html", srv.base);
    let b = format!("{}/net/cache-b.html", srv.base);
    let page = visit(&a, &[b.clone()], site);
    for class in ["fresh", "etag", "nocache", "plain"] {
        let want = format!("{class} rgb(1, 2, 3)");
        assert!(page.contains(&want), "expected {want:?} in:\n{page}");
    }
    let log = visit(&a, &[b, format!("{}/log", srv.base)], site);
    let count = |line: &str| log.lines().filter(|l| l.trim() == line).count();
    let got = [
        count("cached fresh 200"),
        count("cached fresh 304"),
        count("cached etag 200"),
        count("cached etag 304"),
        count("cached nocache 200"),
        count("cached nocache 304"),
        count("cached plain 200"),
    ];
    assert_eq!(got, [2, 0, 2, 2, 2, 2, 4], "{log}");
}

/// Q4.12a: the cache follows the response's cache headers between page
/// loads. `/cached/MODE/s.css` is used by `net/cache-a.html` and then by
/// `net/cache-b.html`: a fresh sheet (`max-age=3600`) is asked for once; a
/// stale one with an ETag (`max-age=0`, or `no-cache`) is asked for again
/// with `If-None-Match`, and the 304's body is the kept one (the second
/// page still has its colour); one without cache headers is fetched
/// again. Before, every sheet was fetched once and reused for the whole
/// session, whatever its headers said.
#[test]
fn http_cache_revalidates_between_pages() {
    revalidates(false);
}

/// The same with a renderer per site: the cache is the browser
/// process's, and a renderer's navigation moves its load epoch.
#[test]
fn http_cache_revalidates_in_renderers() {
    revalidates(true);
}

fn logged(log: &str, line: &str) -> usize {
    log.lines().filter(|l| l.trim() == line).count()
}

/// Q4.12b: the cache is on disk in the profile (`<profile>/cache`, one
/// file per response that can be used again: fresh for a time, or with a
/// validator). A second process with the same profile asks nothing for
/// the fresh sheet and revalidates the two with an ETag (304, the body
/// from the file); the sheet without cache headers is not stored and is
/// fetched again. With `DWELLER_DISK_CACHE_BYTES=1` the next start
/// removes the files, and a third process fetches all four.
#[test]
fn http_cache_survives_restart() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let profile = scratch().join("http-cache-profile");
    let _ = std::fs::remove_dir_all(&profile);
    let dir = profile.to_str().unwrap();
    let a = format!("{}/net/cache-a.html", srv.base);
    let b = format!("{}/net/cache-b.html", srv.base);
    let log = format!("{}/log", srv.base);
    let env = [("DWELLER_PROFILE", dir), ("DWELLER_RESTORE", "0")];
    visit_with(&a, &[], false, &env);
    let files = std::fs::read_dir(profile.join("cache")).unwrap().count();
    assert_eq!(files, 3, "the fresh sheet and the two with an ETag");
    let second = visit_with(&b, &[log.clone()], false, &env);
    let got = [
        logged(&second, "cached fresh 200"),
        logged(&second, "cached etag 200"),
        logged(&second, "cached etag 304"),
        logged(&second, "cached nocache 304"),
        logged(&second, "cached plain 200"),
    ];
    assert_eq!(got, [1, 1, 1, 1, 2], "{second}");
    let small = [("DWELLER_PROFILE", dir), ("DWELLER_RESTORE", "0"), ("DWELLER_DISK_CACHE_BYTES", "1")];
    let third = visit_with(&b, &[log], false, &small);
    assert_eq!(logged(&third, "cached fresh 200"), 2, "{third}");
    assert_eq!(logged(&third, "cached etag 200"), 2, "{third}");
}

fn request_order(site: bool) -> String {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return String::new();
    };
    let page = format!("{}/net/priority.html", srv.base);
    let env = [("DWELLER_HOST_CONNECTIONS", "2")];
    visit_with(&page, &[format!("{}/order", srv.base)], site, &env)
}

/// Q4.12c: `net/priority.html` starts eight images from a script, then
/// adds a style sheet and has a script element, on a host with two
/// connections (HTTP/1.1). Images wait in the loader while the host's
/// connections are taken, so the sheet is asked for right after the two
/// images in flight, not after all eight (libcurl's own queue is first
/// in, first out). The same with a renderer per site. (The page is left
/// for `/order` before the last images are asked for.) Requests that go
/// out together, on the host's two connections, reach the server in
/// either order: `i1` and `i2` once `late.js` is done, then the sheet and
/// `i3` once both are done. So each such pair is compared as a set.
#[test]
fn request_priority_sheet_before_waiting_images() {
    for site in [false, true] {
        let got = request_order(site);
        if got.is_empty() {
            return;
        }
        let names: Vec<&str> = got.lines().last().unwrap_or("").split(' ').collect();
        let pair = |at: usize| {
            let mut p = names.get(at..at + 2).map(|p| p.to_vec()).unwrap_or_default();
            p.sort();
            p
        };
        let ok = names.first() == Some(&"late.js") && pair(1) == ["i1.png", "i2.png"] && pair(3) == ["i3.png", "late.css"];
        assert!(ok, "site {site}: expected late.js, then i1.png and i2.png, then late.css and i3.png, in:\n{got}");
    }
}

/// Q4.30: the `crossorigin` sheets and module script of
/// `net/early_cors.html` (each answered after 0.4 s) are asked for
/// together as the page is parsed, each with the request its fetch
/// makes, and the first layout (its background image) comes before the
/// scripts run (`ran.txt`). Before, the early loads left `crossorigin`
/// out: the scripts ran before the first paint, then its style pass
/// fetched the sheets one by one (github's first paint: 1 s of sheets).
/// In one process and with a renderer per site.
#[test]
fn early_loads_crossorigin() {
    for site in [false, true] {
        let Some(srv) = server() else {
            println!("skipped: cannot start python3 for the test server");
            return;
        };
        let page = format!("{}/net/early_cors.html", srv.base);
        let env: &[(&str, &str)] = if site { &[] } else { &[IN_PROCESS] };
        let got = visit_with(&page, &[format!("{}/order-at", srv.base)], site, env);
        let at = |name: &str| -> u64 {
            got.split_whitespace()
                .find_map(|w| w.strip_prefix(&format!("{name}:")))
                .and_then(|ms| ms.parse().ok())
                .unwrap_or_else(|| panic!("site {site}: no {name} in:\n{got}"))
        };
        let early = [at("slow-a.css"), at("slow-b.css"), at("slow-m.js")];
        let spread = early.iter().max().unwrap() - early.iter().min().unwrap();
        assert!(spread < 300, "site {site}: the sheets and the module were asked for {spread} ms apart:\n{got}");
        assert!(at("laid-out.png") < at("ran.txt"), "site {site}: the scripts ran before the first layout:\n{got}");
    }
}
