use super::*;

fn typed(url: &str) -> String {
    url.as_bytes()
        .chunks(30)
        .map(|c| format!("text:{}", String::from_utf8_lossy(c)))
        .collect::<Vec<_>>()
        .join(" ")
}

fn visit(first: &str, then: &[String]) -> String {
    let mut events = String::from("loaded");
    for url in then {
        events.push_str(&format!(" key:cmd+l {} key:return loaded", typed(url)));
    }
    let out = dweller_command()
        .current_dir(root())
        .args([first, "--frames", "600", "--until-idle", "--state", "--text", "--events", &events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    text(&out.stdout)
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
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let a = format!("{}/net/cache-a.html", srv.base);
    let b = format!("{}/net/cache-b.html", srv.base);
    let page = visit(&a, &[b.clone()]);
    for class in ["fresh", "etag", "nocache", "plain"] {
        let want = format!("{class} rgb(1, 2, 3)");
        assert!(page.contains(&want), "expected {want:?} in:\n{page}");
    }
    let log = visit(&a, &[b, format!("{}/log", srv.base)]);
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
