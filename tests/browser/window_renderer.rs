use super::*;

/// Q1.8: with `DWELLER_PROCESS=tab` the renderer whose page runs out of
/// memory shows the same page and tells the browser (`OutOfMemory`), which
/// keeps it: the renderer line has the page's title, not `gone`.
#[test]
fn window_renderer_out_of_memory() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "tab")
        .env("DWELLER_JS_HEAP_LIMIT", "64000000")
        .args([
            "browser/tests/ui/out-of-memory.html",
            "--font", "fixed", "--frames", "300", "--state", "--events", "renderer:loaded frame frame frame",
        ])
        .output()
        .unwrap();
    let state = text(&out.stdout);
    assert!(out.status.success() && out.stderr.is_empty(), "{}\n{}", out.status, text(&out.stderr));
    let line = state
        .lines()
        .find(|l| l.starts_with("renderer "))
        .unwrap_or_else(|| panic!("no renderer line in\n{state}"));
    let r: Vec<&str> = line.splitn(3, ' ').collect();
    assert_eq!(r[2], "title Out of memory", "{state}");
}

/// Q2.2a: with `DWELLER_PROCESS=tab` each tab has a renderer process of
/// its own (`dweller --renderer FD`, the binary IPC of
/// docs/process-model.md §2). The tab's title comes from the renderer,
/// whose pid is not the window's; a renderer that is killed is noted for
/// its tab (`gone crashed`); one that sends a malformed message is
/// killed (`bad message:` on stderr). Without the flag no renderer runs.
#[test]
fn window_renderer_lifecycle() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let run = |events: &str, envs: &[(&str, &str)]| -> (u32, String, String) {
        let child = dweller_command()
            .current_dir(root())
            .envs(envs.iter().copied())
            .args(["browser/tests/ui/form.html", "--frames", "300", "--state", "--events", events])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let pid = child.id();
        let out = child.wait_with_output().unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        (pid, text(&out.stdout), text(&out.stderr))
    };
    let renderer = |state: &str| -> Vec<String> {
        let line = state
            .lines()
            .find(|l| l.starts_with("renderer "))
            .unwrap_or_else(|| panic!("no renderer line in\n{state}"));
        line.splitn(4, ' ').map(String::from).collect()
    };
    let tab = [("DWELLER_PROCESS", "tab")];

    let (window, state, err) = run("renderer:loaded", &tab);
    let r = renderer(&state);
    assert!(err.is_empty(), "{err}");
    assert_ne!(r[1], window.to_string(), "the renderer is another process");
    assert_eq!(&r[2..], ["title", "Form"], "{state}");

    let (_, state, err) = run("renderer:loaded renderer:kill renderer:gone", &tab);
    assert!(err.is_empty(), "{err}");
    assert_eq!(&renderer(&state)[2..], ["gone", "crashed"], "{state}");

    let evil = [("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "malformed")];
    let (_, state, err) = run("renderer:gone", &evil);
    let r = renderer(&state);
    assert_eq!(&r[2..], ["gone", "UnknownType"], "{state}");
    assert_eq!(err, format!("error: bad message: UnknownType {}\n", r[1]));

    let (_, state, err) = run("", &[]);
    assert!(err.is_empty(), "{err}");
    assert!(!state.lines().any(|l| l.starts_with("renderer ")), "{state}");
}

/// Q2.4a: with `DWELLER_PROCESS=tab` the renderer's loads go through the
/// browser's net, which runs CORS and ORB for the initiator it knows: a
/// two-origin page shows the same text as in the window's own process.
/// A renderer that names a document the browser never gave it as a
/// request's initiator is killed (`bad message: bad fetch`).
/// Q2.4d: so is one that claims another top-level site, a referrer of
/// another origin or the TRACE method; the `Cookie`, `Referer`, `Origin`
/// and `Sec-*` headers it adds are dropped (the page shows the headers
/// net sent, as without them).
#[test]
fn window_renderer_net() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let tab = [("DWELLER_PROCESS", "tab")];
    for page in ["net/cors.html", "net/crossorigin.html"] {
        let href = format!("{}/{page}", srv.base);
        let page_only = |got: String| -> String {
            got.lines()
                .filter(|l| !l.starts_with("renderer"))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let own = page_only(login_window_with(&href, "frame", None, &[]));
        let renderer = page_only(login_window_with(&href, "frame", None, &tab));
        assert!(own.contains("fetch same origin: basic status 200 body true") || own.contains("crossorigin"), "{page}: {own}");
        assert_eq!(renderer, own, "{page}");
    }
    let headers = format!("{}/net/request_headers.html", srv.base);
    let plain = login_window_with(&headers, "frame", None, &tab);
    let evil = login_window_with(&headers, "frame", None, &[("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "headers")]);
    assert!(plain.contains(&format!("Referer: {headers}")) && !evil.contains("evil"), "{evil}");
    let page_only = |got: &str| got.lines().filter(|l| !l.starts_with("renderer")).collect::<Vec<_>>().join("\n");
    assert_eq!(page_only(&evil), page_only(&plain));
    for evil in ["initiator", "top", "referrer", "trace"] {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "tab")
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_EVIL", evil)
            .args([
                &format!("{}/net/cors.html", srv.base),
                "--font", "fixed", "--frames", "300", "--state", "--events", "renderer:gone",
            ])
            .output()
            .unwrap();
        let state = text(&out.stdout);
        let line = state
            .lines()
            .find(|l| l.starts_with("renderer "))
            .unwrap_or_else(|| panic!("{evil}: no renderer line in\n{state}"));
        let r: Vec<&str> = line.splitn(4, ' ').collect();
        assert_eq!(&r[2..], ["gone", "bad fetch"], "{evil}: {state}");
        assert_eq!(text(&out.stderr), format!("error: bad message: bad fetch {}\n", r[1]), "{evil}");
    }
}

/// Q2.4f: a navigation the browser starts for a renderer (a link click,
/// a form submission, `browser/tests/net/samesite_links.html`) carries
/// the page that began it, so net decides SameSite as in the window's
/// own process: from `127.0.0.1` to `localhost` a link sends Lax but not
/// Strict cookies and a form POST neither; from `localhost` a link sends
/// both, and so does a typed URL.
#[test]
fn window_renderer_samesite() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let site = srv.base.clone();
    let other = site.replace("127.0.0.1", "localhost");
    let dir = profile("renderer_samesite");
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROFILE", &dir)
        .args(["dump", "--text", &format!("{other}/samesite/set")])
        .output()
        .unwrap();
    assert_eq!(text(&out.stdout), "set\n", "{}", text(&out.stderr));
    let show = format!("{other}/samesite/show");
    let tab = [("DWELLER_PROCESS", "tab")];
    for envs in [&[][..], &tab[..]] {
        let (wait, then) = if envs.is_empty() { ("loaded ", " loaded") } else { ("", " renderer:input renderer:loaded") };
        for (what, from, click, want) in [
            ("cross-site link", &site, "click:80,65", "GET cookies: lax=1; dflt=1"),
            ("cross-site post", &site, "click:60,115", "POST cookies:"),
            ("same-site link", &other, "click:80,65", "GET cookies: strict=1; lax=1; dflt=1"),
        ] {
            let got = login_window_with(&format!("{from}/net/samesite_links.html?{show}"), &format!("{wait}{click}{then}"), Some(&dir), envs);
            assert!(got.lines().any(|l| l.trim_end().ends_with(&format!("Show{want}"))), "{what} {envs:?}: want {want:?}, got:\n{got}");
        }
        let typed = login_window_with(&show, "frame", Some(&dir), envs);
        assert!(typed.lines().any(|l| l == "ShowGET cookies: strict=1; lax=1; dflt=1"), "typed {envs:?}: {typed}");
    }
}

/// Q2.4b: with `DWELLER_PROCESS=tab` `document.cookie`, `localStorage`
/// and `sessionStorage` live in the browser, and pages see what they see
/// in the window's own process: an `HttpOnly` cookie set by the server
/// is not in `document.cookie` (`login/details.html`), a script stores
/// neither an `HttpOnly` nor a `Secure` cookie on http:, the popups of
/// `storage/tabs.html` share localStorage and copy sessionStorage as in
/// `storage_between_tabs`, and a write in one tab reaches another tab of
/// the origin as a `storage` event (another renderer under the flag) but
/// not a tab of another origin. A renderer that asks for the cookies or
/// the storage of a document it was not given is killed.
#[test]
fn window_renderer_storage() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let tab = [("DWELLER_PROCESS", "tab")];
    let write = format!("{base}/storage/write.html");
    let other = format!("{}?other", write.replace("127.0.0.1", "localhost"));
    let port = base.rsplit(':').next().unwrap();
    for envs in [&[][..], &tab[..]] {
        let has = |got: &str, want: &[&str], not: &[&str], what: &str| {
            assert!(
                want.iter().all(|w| got.contains(w)) && !not.iter().any(|n| got.contains(n)),
                "{what} {envs:?}: want {want:?}, not {not:?}, got:\n{got}"
            );
        };
        let details = login_window_with(&format!("{base}/login/details.html"), "frame frame frame", None, envs);
        has(&details, &["fetch 200 cookie tab=1", "xhr 200 cookie tab=1"], &["cookie sid", "; sid="], "HttpOnly");
        let script = login_window_with(&format!("{base}/storage/script-cookies.html"), "frame", None, envs);
        has(&script, &["script cookies: [js=1]"], &[], "script cookies");
        let tabs = login_window_with(&format!("{base}/storage/tabs.html"), "frame frame frame key:cmd+1 frame frame", None, envs);
        has(
            &tabs,
            &["copy=from the opener (old null, local, from child.html?copy) | fresh=null (old null, local, from child.html?fresh) | own session child: null | own length 1"],
            &[],
            "storage between popups",
        );
        let visits = format!("{base}/storage/visits.html");
        let again = login_window_with(&visits, &format!("frame {}", go_to(&visits)), None, envs);
        has(&again, &["visits 2"], &[], "sessionStorage across a navigation");
        let (wait, then) = if envs.is_empty() { (" loaded", " loaded frame") } else { ("", "") };
        let events = format!(
            "frame key:cmd+t {} key:return{wait} key:cmd+t {} key:return{wait} key:cmd+1{then} frame frame",
            typed(&write),
            typed(&other)
        );
        let heard = login_window_with(&format!("{base}/storage/listen.html"), &events, None, envs);
        has(
            &heard,
            &[&format!("heard written=127.0.0.1:{port} (old null, from write.html)")],
            &["written?other"],
            "storage events between tabs",
        );
    }
    for (evil, page, why) in [("cookie", "storage/script-cookies.html", "bad cookie"), ("storage", "storage/write.html", "bad storage")] {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "tab")
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_EVIL", evil)
            .args([&format!("{base}/{page}"), "--font", "fixed", "--frames", "300", "--state", "--events", "renderer:gone"])
            .output()
            .unwrap();
        let state = text(&out.stdout);
        let line = state
            .lines()
            .find(|l| l.starts_with("renderer "))
            .unwrap_or_else(|| panic!("{evil}: no renderer line in\n{state}"));
        let r: Vec<&str> = line.splitn(4, ' ').collect();
        assert_eq!(&r[2..], ["gone", why], "{evil}: {state}");
        assert_eq!(text(&out.stderr), format!("error: bad message: {why} {}\n", r[1]), "{evil}");
    }
}

/// Q2.4c: with `DWELLER_PROCESS=tab` and a profile, no renderer (the
/// window's, and its agent's Space's, which shows a page that uses
/// `document.cookie` and `localStorage`, served over http) holds a file
/// of the profile open,
/// or a regular file but the binary and system libraries (macOS:
/// `lsof`; Linux: `/proc/PID/fd`).
#[test]
fn window_renderer_no_profile_files() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let dir = profile("q24c-profile");
    let dir_text = dir.canonicalize().unwrap().to_str().unwrap().to_string();
    let sock = format!("target/sockets/q24c-{}.sock", std::process::id());
    let envs = [("DWELLER_PROCESS", "tab"), ("DWELLER_PROFILE", dir.to_str().unwrap())];
    let window = spaces_window_env(&sock, &[], &envs);
    let mut conn = AgentConn::open(&sock);
    let page = format!("{}/storage/script-cookies.html", srv.base);
    let (got, _) = conn.ask(&format!("{{\"id\":1,\"cmd\":\"navigate\",\"url\":{}}}", serde_like(&page)));
    assert!(got.contains("\"ok\":true"), "{got}");
    let used = "(localStorage.setItem('q24c', 'kept'), localStorage.getItem('q24c') + ' ' + document.cookie)";
    let (got, _) = conn.ask(&format!("{{\"id\":2,\"cmd\":\"eval\",\"expression\":{}}}", serde_like(used)));
    assert!(got.contains("kept js=1"), "{got}");
    let children = |pid: &str| -> Vec<String> {
        let out = Command::new("pgrep").args(["-P", pid]).output().unwrap();
        text(&out.stdout).lines().map(str::to_string).collect()
    };
    let spaces = children(&window.id().to_string());
    let listed: Vec<(String, Vec<(String, bool, String)>)> = spaces
        .iter()
        .chain(spaces.iter().flat_map(|p| children(p)).collect::<Vec<_>>().iter())
        .filter(|pid| {
            let args = Command::new("ps").args(["-o", "args=", "-p", pid]).output().unwrap();
            text(&args.stdout).contains(" --renderer 3 ")
        })
        .map(|pid| (pid.clone(), open_files(pid)))
        .collect();
    let lock = format!("{dir_text}/lock");
    let space_files: Vec<(String, Vec<(String, bool, String)>)> = spaces.iter().filter(|pid| !listed.iter().any(|(r, _)| r == *pid)).map(|pid| (pid.clone(), open_files(pid))).collect();
    drop(conn);
    let out = window.wait_with_output().unwrap();
    assert!(out.status.success(), "the window: {}\n{}", out.status, text(&out.stderr));
    assert!(!space_files.is_empty(), "want the agent's Space: {spaces:?}");
    for (pid, files) in &space_files {
        assert!(files.iter().all(|(_, _, name)| *name != lock), "Space {pid} inherited the profile's lock: {files:?}");
    }
    let exe = dweller().canonicalize().unwrap();
    let system = ["/System/", "/usr/", "/Library/Fonts/", "/dev/"];
    assert!(listed.len() >= 2, "want the window's renderer and its Space's: {listed:?}");
    for (pid, files) in &listed {
        assert!(!files.is_empty(), "no open files listed for {pid}");
        for (fd, regular, name) in files {
            assert!(!name.starts_with(&dir_text), "renderer {pid} holds {name} (fd {fd}) of the profile");
            let allowed = !regular || fd == "txt" || Path::new(name) == exe || system.iter().any(|s| name.starts_with(s));
            assert!(allowed, "renderer {pid} holds the regular file {name} (fd {fd}): {files:?}");
        }
    }
}

/// Q2.5: with `DWELLER_PROCESS=tab` a renderer that dies (SIGKILL) shows
/// the browser's sad tab ("This page crashed"); the other tab's renderer
/// goes on; Reload (the sad tab's button, or cmd+R) starts a fresh
/// renderer that loads the tab's page again.
#[test]
fn window_renderer_sad_tab() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let run = |events: &str| -> String {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "tab")
            .args(["browser/tests/ui/form.html", "--frames", "300", "--state", "--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        assert!(out.stderr.is_empty(), "{events}: {}", text(&out.stderr));
        text(&out.stdout)
    };
    let renderer = |state: &str| -> Vec<String> {
        let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or_else(|| panic!("no renderer line in\n{state}"));
        line.splitn(4, ' ').map(String::from).collect()
    };

    let killed = run("renderer:loaded renderer:kill renderer:gone frame");
    assert_eq!(&renderer(&killed)[2..], ["gone", "crashed"], "{killed}");
    assert!(killed.contains("\nsad-tab This page crashed\n"), "{killed}");

    let tabs = format!(
        "renderer:loaded key:cmd+t {} key:return renderer:loaded key:cmd+1 renderer:kill renderer:gone frame key:cmd+2 renderer:loaded renderer:frame frame",
        typed("browser/tests/ui/stuck.html")
    );
    let other = run(&tabs);
    assert_eq!(&renderer(&other)[2..], ["title", "Stuck"], "{other}");
    assert!(other.contains("tabs 2, showing 2") && !other.contains("sad-tab"), "{other}");

    for reload in ["click:512,198", "key:cmd+r"] {
        let again = run(&format!("renderer:loaded renderer:kill renderer:gone frame {reload} frame renderer:loaded"));
        assert_eq!(&renderer(&again)[2..], ["title", "Form"], "{reload}: {again}");
        assert!(!again.contains("sad-tab"), "{reload}: {again}");
    }
}

/// Q2.5: a renderer that does not answer the browser's ping for
/// `DWELLER_HANG_MS` (a `while (1) {}` in its page) shows the "not
/// responding" bar; `stuck:stop` kills it, the sad tab says "This page
/// was stopped", and the window reaps it while it runs (no zombie).
/// Without an answer a headless run waits, and says so on stderr.
#[test]
fn window_renderer_hang() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let start = |events: &str| {
        dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "tab")
            .env("DWELLER_HANG_MS", "1000")
            .args(["browser/tests/ui/renderer_hang.html", "--frames", "600", "--state", "--events", events])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap()
    };
    let pause = || std::thread::sleep(std::time::Duration::from_millis(20));

    let mut child = start("renderer:hung stuck:stop renderer:gone");
    let window = child.id().to_string();
    let mut pid = String::new();
    while pid.is_empty() && child.try_wait().unwrap().is_none() {
        let out = Command::new("pgrep").args(["-P", &window]).output().unwrap();
        pid = text(&out.stdout).lines().next().unwrap_or("").to_string();
        pause();
    }
    let mut reaped = false;
    while !reaped && child.try_wait().unwrap().is_none() {
        let ps = Command::new("ps").args(["-o", "stat=", "-p", &pid]).output().unwrap();
        reaped = text(&ps.stdout).trim().is_empty();
        pause();
    }
    let out = child.wait_with_output().unwrap();
    let state = text(&out.stdout);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    assert!(out.stderr.is_empty(), "{}", text(&out.stderr));
    assert!(!pid.is_empty() && reaped, "renderer {pid:?} was not reaped while the window ran\n{state}");
    assert!(state.contains(&format!("renderer {pid} gone stopped\n")), "{state}");
    assert!(state.contains("\nsad-tab This page was stopped\n") && state.contains("\nunresponsive 1\n"), "{state}");

    let out = start("renderer:hung").wait_with_output().unwrap();
    let state = text(&out.stdout);
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(err.starts_with("stuck: renderer ") && err.contains(" not answering, waiting\n"), "{err}");
    assert!(!state.contains(" gone ") && state.contains("\nunresponsive "), "{state}");
}

/// The open files of process `pid`: (fd, is a regular file, path).
fn open_files(pid: &str) -> Vec<(String, bool, String)> {
    if cfg!(target_os = "macos") {
        let out = Command::new("lsof").args(["-n", "-P", "-F", "ftn", "-p", pid]).output().unwrap();
        let mut files: Vec<(String, bool, String)> = Vec::new();
        for line in text(&out.stdout).lines() {
            let (tag, rest) = line.split_at(1.min(line.len()));
            match tag {
                "f" => files.push((rest.to_string(), false, String::new())),
                "t" => files.last_mut().unwrap().1 = rest == "REG",
                "n" => files.last_mut().unwrap().2 = rest.to_string(),
                _ => {}
            }
        }
        return files;
    }
    let dir = PathBuf::from(format!("/proc/{pid}/fd"));
    std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| {
            let path = e.unwrap().path();
            let name = std::fs::read_link(&path).unwrap_or_default().to_string_lossy().to_string();
            let regular = std::fs::metadata(&path).map(|m| m.is_file()).unwrap_or(false);
            (path.file_name().unwrap().to_string_lossy().to_string(), regular, name)
        })
        .collect()
}

/// Q2.2b: with `DWELLER_PROCESS=tab` the tab's pixels come from its
/// renderer: the browser gives it two shared-memory buffers (`resize`
/// with 2 fds), it paints the page into one and sends `frame`, and the
/// window shows that buffer. The window's PNG is the one of the same
/// page painted in the window's own process. A frame of a size the
/// browser did not give kills the renderer (`bad message: wrong size`).
#[test]
fn window_renderer_frames() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let run_page = |page: &str, png: &Path, events: &str, envs: &[(&str, &str)]| -> (String, String) {
        let out = dweller_command()
            .current_dir(root())
            .envs(envs.iter().copied())
            .args([page, "--frames", "300", "--state", "--save"])
            .arg(png)
            .args(["--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        (text(&out.stdout), text(&out.stderr))
    };
    let run = |png: &Path, events: &str, envs: &[(&str, &str)]| run_page("browser/tests/ui/form.html", png, events, envs);
    let golden = scratch().join("renderer_frames_in_process.png");
    let tab = scratch().join("renderer_frames_tab.png");

    let (state, err) = run(&golden, "loaded", &[]);
    assert!(err.is_empty(), "{err}");
    assert!(!state.lines().any(|l| l.starts_with("renderer")), "{state}");

    let (state, err) = run(&tab, "renderer:loaded renderer:frame", &[("DWELLER_PROCESS", "tab")]);
    assert!(err.is_empty(), "{err}");
    assert!(state.contains("\nrenderer-frame "), "{state}");
    assert!(
        std::fs::read(&tab).unwrap() == std::fs::read(&golden).unwrap(),
        "the renderer's frame {} differs from the in-process window {}",
        tab.display(),
        golden.display()
    );

    let evil = [("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "frame")];
    let (state, err) = run(&tab, "renderer:gone", &evil);
    let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or("");
    let pid = line.split(' ').nth(1).unwrap_or("");
    assert!(line.ends_with(" gone wrong size"), "{state}");
    assert_eq!(err, format!("error: bad message: wrong size {pid}\n"));
    assert!(!state.lines().any(|l| l.starts_with("renderer-frame")), "{state}");

    let page = "browser/tests/ui/renderer_damage.html";
    let (_, err) = run_page(page, &golden, "loaded", &[]);
    assert!(err.is_empty(), "{err}");
    let (state, err) = run_page(page, &tab, "renderer:loaded renderer:frame", &[("DWELLER_PROCESS", "tab")]);
    assert!(err.is_empty(), "{err}");
    let line = state.lines().find(|l| l.starts_with("renderer-frame ")).unwrap_or("");
    assert!(line.ends_with(" damage 59 29 101 51"), "{state}");
    assert!(
        std::fs::read(&tab).unwrap() == std::fs::read(&golden).unwrap(),
        "the renderer's frame {} differs from the in-process window {}",
        tab.display(),
        golden.display()
    );
}

/// Q2.8a, Q2.8b: a renderer enters its sandbox (macOS: seatbelt; Linux:
/// Landlock and seccomp-bpf) before it reads any page. With
/// `DWELLER_TEST_SANDBOX_PROBE=1` it then tries to open the profile
/// (`$HOME/.dweller`), to write in /tmp, to connect to 127.0.0.1 and to
/// fork, and prints each errno: all are EPERM (1), but Landlock refuses
/// the profile with EACCES (13); and the page still matches the
/// in-process window. With `DWELLER_NO_SANDBOX=1` it warns and the same
/// probes succeed.
#[test]
fn window_renderer_sandbox() {
    if !has_sdl() || !cfg!(any(target_os = "macos", target_os = "linux")) {
        println!("skipped: macOS or Linux with SDL2 only");
        return;
    }
    let home = scratch().join("sandbox_home");
    let profile = home.join(".dweller");
    std::fs::create_dir_all(&profile).unwrap();
    let base = [("HOME", home.to_str().unwrap()), ("DWELLER_PROFILE", profile.to_str().unwrap())];
    let run = |png: &Path, events: &str, envs: &[(&str, &str)]| -> (String, String) {
        let out = dweller_command()
            .current_dir(root())
            .envs(base.iter().copied())
            .envs(envs.iter().copied())
            .args(["browser/tests/ui/form.html", "--frames", "300", "--state", "--save"])
            .arg(png)
            .args(["--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        (text(&out.stdout), text(&out.stderr))
    };
    let golden = scratch().join("renderer_sandbox_in_process.png");
    let tab = scratch().join("renderer_sandbox_tab.png");
    let probe = [("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_SANDBOX_PROBE", "1")];

    let (_, err) = run(&golden, "loaded", &[]);
    assert!(err.is_empty(), "{err}");

    let (state, err) = run(&tab, "renderer:loaded renderer:frame", &probe);
    let denied = ["sandbox-probe home=1 tmp=1 tcp=1 fork=1\n", "sandbox-probe home=13 tmp=1 tcp=1 fork=1\n"];
    assert!(denied.contains(&err.as_str()), "{err}");
    assert!(state.contains("\nrenderer-frame "), "{state}");
    assert!(
        std::fs::read(&tab).unwrap() == std::fs::read(&golden).unwrap(),
        "the sandboxed renderer's frame {} differs from the in-process window {}",
        tab.display(),
        golden.display()
    );

    let open = [probe[0], probe[1], probe[2], ("DWELLER_NO_SANDBOX", "1")];
    let (state, err) = run(&tab, "renderer:loaded renderer:frame", &open);
    assert!(err.starts_with("warning: DWELLER_NO_SANDBOX=1, the renderer runs without a sandbox\n"), "{err}");
    assert!(err.contains("sandbox-probe home=0 tmp=0 tcp=") && err.ends_with(" fork=0\n") && !err.contains("tcp=1 "), "{err}");
    assert!(state.contains("\nrenderer-frame "), "{state}");
}

#[test]
fn window_renderer_input() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let run_page = |page: &str, png: &Path, events: &str, envs: &[(&str, &str)]| -> (String, String) {
        let out = dweller_command()
            .current_dir(root())
            .envs(envs.iter().copied())
            .args([page, "--font", "fixed", "--width", "400", "--height", "340"])
            .args(["--frames", "1000", "--until-idle", "--state", "--save"])
            .arg(png)
            .args(["--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        (text(&out.stdout), text(&out.stderr))
    };
    let page = "browser/tests/ui/renderer_input.html";
    let run = |png: &Path, events: &str, envs: &[(&str, &str)]| run_page(page, png, events, envs);
    let tab = [("DWELLER_PROCESS", "tab"), ("DWELLER_STEADY_CARET", "1")];
    let scroll = |state: &str| state.lines().find(|l| l.starts_with("scroll ")).unwrap_or("").to_string();
    let still = scratch().join("renderer_input_still.png");
    let golden = scratch().join("renderer_input_in_process.png");
    let shot = scratch().join("renderer_input_tab.png");
    let scroller = "browser/tests/ui/scroller.html";
    let cases = [
        ("click", page, "click:30,60", 0),
        ("typing", page, "click:30,112 text:hello", 0),
        ("wheel", page, "wheel:-3", 0),
        ("inner scroller", scroller, "move:50,100 wheel:-1", 0),
    ];
    for (what, page, events, caret) in cases {
        let (_, err) = run_page(page, &still, "renderer:loaded renderer:frame", &tab);
        assert!(err.is_empty(), "{err}");
        let (in_process, err) = run_page(page, &golden, &format!("loaded {events}"), &[]);
        assert!(err.is_empty(), "{err}");
        let (state, err) = run_page(page, &shot, &format!("renderer:loaded renderer:frame {events} renderer:input"), &tab);
        assert!(err.is_empty(), "{err}");
        assert!(state.contains("\nrenderer-frame "), "{state}");
        assert_eq!(scroll(&state), "scroll 0", "{what}: the window scrolled its own copy\n{state}");
        assert!(std::fs::read(&shot).unwrap() != std::fs::read(&still).unwrap(), "{what}: no change in {}", shot.display());
        assert!(!state.contains("value hello"), "{what}: the window typed into its own copy\n{state}");
        let (w, _, a) = png_rgba(&std::fs::read(&shot).unwrap());
        let (_, _, b) = png_rgba(&std::fs::read(&golden).unwrap());
        let columns: std::collections::BTreeSet<usize> =
            (0..a.len() / 4).filter(|i| a[i * 4..i * 4 + 4] != b[i * 4..i * 4 + 4]).map(|i| i % w).collect();
        assert!(
            columns.len() <= caret,
            "{what}: the renderer's frame {} differs from the in-process window {} in columns {columns:?}\n{in_process}",
            shot.display(),
            golden.display()
        );
    }

    let evil = [("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "ack")];
    let (state, err) = run(&shot, "renderer:loaded renderer:frame click:30,60 renderer:gone", &evil);
    let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or("");
    let pid = line.split(' ').nth(1).unwrap_or("");
    assert!(line.ends_with(" gone unexpected input_ack"), "{state}");
    assert_eq!(err, format!("error: bad message: unexpected input_ack {pid}\n"));
}

/// Q2.3d: with `DWELLER_PROCESS=tab` the renderer owns the focus, the
/// caret and the selection (`browser/tests/ui/renderer_focus.html`):
/// Tab moves the focus ring across three fields, a drag and Shift+Left
/// select text, each the same page area as the in-process window; the
/// caret blinks with damage only its rect and hides when the window
/// loses the focus; the toolbar shows the title and URL the renderer
/// reports (pushState), and a reported URL of another origin than the
/// browser navigated to kills the renderer.
#[test]
fn window_renderer_focus() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let run = |png: &Path, events: &str, envs: &[(&str, &str)]| -> (String, String) {
        let out = dweller_command()
            .current_dir(root())
            .envs(envs.iter().copied())
            .args(["browser/tests/ui/renderer_focus.html", "--font", "fixed", "--width", "400", "--height", "340"])
            .args(["--frames", "1000", "--until-idle", "--state", "--save"])
            .arg(png)
            .args(["--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        (text(&out.stdout), text(&out.stderr))
    };
    let page_area = |png: &Path| {
        let (w, _, rgba) = png_rgba(&std::fs::read(png).unwrap());
        (w, rgba[40 * w * 4..].to_vec())
    };
    let tab = [("DWELLER_PROCESS", "tab"), ("DWELLER_STEADY_CARET", "1")];
    let still = scratch().join("renderer_focus_still.png");
    let golden = scratch().join("renderer_focus_in_process.png");
    let shot = scratch().join("renderer_focus_tab.png");
    let cases = [
        ("tab", "key:tab key:tab key:tab"),
        ("shift tab", "key:tab key:tab key:tab key:shift+tab"),
        ("drag", "click:20,62 move:60,62 move:90,62 release:90,62"),
        ("shift left", "click:90,102 key:shift+left key:shift+left key:shift+left"),
    ];
    for (what, events) in cases {
        let (_, err) = run(&still, "renderer:loaded renderer:frame", &tab);
        assert!(err.is_empty(), "{err}");
        let (_, err) = run(&golden, &format!("loaded {events}"), &[]);
        assert!(err.is_empty(), "{what}: {err}");
        let (state, err) = run(&shot, &format!("renderer:loaded renderer:frame {events} renderer:input"), &tab);
        assert!(err.is_empty(), "{what}: {err}");
        assert!(page_area(&shot) != page_area(&still), "{what}: no change in {}", shot.display());
        assert!(
            page_area(&shot) == page_area(&golden),
            "{what}: the renderer's frame {} differs from the in-process window {}\n{state}",
            shot.display(),
            golden.display()
        );
    }

    let (_, err) = run(&golden, "loaded click:90,102", &[]);
    assert!(err.is_empty(), "{err}");
    let (state, err) = run(&shot, "renderer:loaded renderer:frame click:90,102 renderer:input window:blur renderer:next", &tab);
    assert!(err.is_empty(), "{err}");
    let (w, a) = page_area(&shot);
    let (_, b) = page_area(&golden);
    let columns: std::collections::BTreeSet<usize> =
        (0..a.len() / 4).filter(|i| a[i * 4..i * 4 + 4] != b[i * 4..i * 4 + 4]).map(|i| i % w).collect();
    assert!((1..=2).contains(&columns.len()), "window blur: the caret did not hide, columns {columns:?}\n{state}");

    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "tab")
        .args(["browser/tests/ui/renderer_focus.html", "--font", "fixed", "--width", "400", "--height", "340"])
        .args(["--frames", "1000", "--until-idle", "--state", "--save"])
        .arg(&shot)
        .args(["--events", "renderer:loaded renderer:frame click:90,102 renderer:input renderer:next renderer:next"])
        .output()
        .unwrap();
    assert!(out.status.success(), "blink: {}\n{}", out.status, text(&out.stderr));
    let (state, err) = (text(&out.stdout), text(&out.stderr));
    assert!(err.is_empty(), "{err}");
    let line = state.lines().find(|l| l.starts_with("renderer-frame ")).unwrap_or("");
    let rect: Vec<i64> = line.split(' ').skip(4).filter_map(|v| v.parse().ok()).collect();
    assert!(rect.len() == 4 && rect[2] - rect[0] <= 3 && rect[3] - rect[1] <= 30, "blink: the last frame's damage is not the caret\n{state}");

    let (state, err) = run(&shot, "renderer:loaded renderer:frame click:60,185 renderer:input", &tab);
    assert!(err.is_empty(), "{err}");
    let pushed = format!("file://{}/browser/tests/ui/renderer_focus.html?pushed", root().display());
    assert!(state.contains(&format!("\naddress {pushed}\n")), "{state}");
    assert!(state.contains(" title Pushed\n"), "{state}");
    assert!(state.starts_with(&format!("url {pushed}\n")) && state.contains("\nhistory 2 of 2\n"), "{state}");

    let evil = [("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "url")];
    let (state, err) = run(&shot, "renderer:gone", &evil);
    let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or("");
    let pid = line.split(' ').nth(1).unwrap_or("");
    assert!(line.ends_with(" gone bad url"), "{state}");
    assert_eq!(err, format!("error: bad message: bad url {pid}\n"));
}

/// Q2.6a: with `DWELLER_PROCESS=tab` the browser owns the tab's session
/// history (`browser/tests/ui/renderer_history.html`): `location.href`,
/// `history.back()`, `form.submit()` and `location.reload()` from the
/// page's scripts are navigations the browser makes, `location.hash` and
/// pushState are entries it records, a back or forward to an entry of the
/// shown document fires popstate (and hashchange) without a load, a
/// top-level `data:` navigation from the page is refused, and a
/// pushState reported with another origin kills the renderer.
#[test]
fn window_renderer_history() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let run = |events: &str, envs: &[(&str, &str)]| -> (String, String) {
        let out = dweller_command()
            .current_dir(root())
            .envs(envs.iter().copied())
            .args(["browser/tests/ui/renderer_history.html", "--font", "fixed", "--width", "400", "--height", "400"])
            .args(["--frames", "1000", "--until-idle", "--state", "--text", "--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        (text(&out.stdout), text(&out.stderr))
    };
    let page = format!("file://{}/browser/tests/ui/renderer_history.html", root().display());
    let tab = [("DWELLER_PROCESS", "tab")];
    let to = "renderer:loaded click:50,62 renderer:input renderer:loaded";
    let cases: Vec<(String, Vec<String>, &str)> = vec![
        (to.into(), vec![format!("\naddress {page}?to=1\nhistory 2 of 2\n"), "length 2".into()], "location.href"),
        (
            format!("{to} click:50,106 renderer:input renderer:loaded"),
            vec![format!("\naddress {page}\nhistory 1 of 2\n"), "events load load load".into()],
            "history.back to another document",
        ),
        (
            "renderer:loaded click:50,150 renderer:input renderer:loaded".into(),
            vec![format!("\naddress {page}?q=sent\nhistory 2 of 2\n")],
            "form.submit",
        ),
        (
            "renderer:loaded click:50,194 renderer:input key:cmd+[ renderer:loaded".into(),
            vec![format!("\naddress {page}\nhistory 1 of 2\n"), "events load popstate hashchange popstate hashchange".into()],
            "location.hash, then back in the document",
        ),
        (
            "renderer:loaded click:50,238 renderer:input key:cmd+[ renderer:loaded key:cmd+] renderer:loaded".into(),
            vec![format!("\naddress {page}?pushed\nhistory 2 of 2\n"), "length 2 state {\"n\":1} events load popstate popstate".into()],
            "pushState, back and forward in the document",
        ),
        (
            "renderer:loaded click:50,326 renderer:input renderer:loaded".into(),
            vec![format!("\naddress {page}\nhistory 1 of 1\n"), "events load reload load".into()],
            "location.reload",
        ),
    ];
    for (events, wants, what) in &cases {
        let (state, err) = run(events, &tab);
        assert!(err.is_empty(), "{what}: {err}");
        for want in wants {
            assert!(state.contains(want.as_str()), "{what}: expected {want:?} in:\n{state}");
        }
    }

    let (state, err) = run("renderer:loaded click:50,282 renderer:input", &tab);
    assert!(state.contains(&format!("\naddress {page}\nhistory 1 of 1\n")), "{state}");
    assert_eq!(err, "error: Not allowed to navigate top frame to data URL: data:text/html,hi\n");

    let evil = [("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "pushstate")];
    let (state, err) = run("renderer:loaded click:50,238 renderer:gone", &evil);
    let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or("");
    let pid = line.split(' ').nth(1).unwrap_or("");
    assert!(line.ends_with(" gone bad url"), "{state}");
    assert_eq!(err, format!("error: bad message: bad url {pid}\n"));
}

/// Q2.6b: with `DWELLER_PROCESS=tab` a cross-site navigation
/// (`127.0.0.1` and `localhost` are two sites) commits in a new renderer
/// and retires the old one; a same-site one stays in its renderer. The
/// way back to a site is a new renderer again (no process is kept), and
/// back and forward across sites swap too.
#[test]
fn window_renderer_sites() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/ui/renderer_sites.html", srv.base);
    let other = page.replace("127.0.0.1", "localhost");
    let run = |events: &str| -> String {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "tab")
            .args([page.as_str(), "--font", "fixed", "--width", "400", "--height", "400"])
            .args(["--frames", "1000", "--until-idle", "--state", "--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        assert!(out.stderr.is_empty(), "{events}: {}", text(&out.stderr));
        text(&out.stdout)
    };
    let pids = |state: &str| -> Vec<String> {
        let line = state.lines().find(|l| l.starts_with("renderer-pids ")).unwrap_or("");
        line.split(' ').skip(1).map(String::from).collect()
    };
    let distinct = |p: &[String]| p.iter().collect::<std::collections::HashSet<_>>().len() == p.len();

    let state = run("renderer:loaded click:50,106 renderer:input renderer:loaded");
    assert!(state.contains(&format!("\naddress {page}?same\nhistory 2 of 2\n")), "{state}");
    assert!(state.contains(" title 127.0.0.1?same\n") && state.contains("\nrenderer-site http://127.0.0.1\n"), "{state}");
    assert_eq!(pids(&state).len(), 1, "same site keeps the renderer:\n{state}");

    let there = "renderer:loaded click:50,62 renderer:input renderer:loaded click:50,62 renderer:input renderer:loaded";
    let cases = [
        (there.to_string(), page.as_str(), "3 of 3", "127.0.0.1", 3),
        (format!("{there} key:cmd+[ renderer:loaded"), other.as_str(), "2 of 3", "localhost", 4),
        (format!("{there} key:cmd+[ renderer:loaded key:cmd+] renderer:loaded"), page.as_str(), "3 of 3", "127.0.0.1", 5),
    ];
    for (events, address, at, host, count) in &cases {
        let state = run(events);
        let p = pids(&state);
        assert!(state.contains(&format!("\naddress {address}\nhistory {at}\n")), "{events}:\n{state}");
        assert!(state.contains(&format!("\nrenderer {} title {host}\n", p.last().unwrap())), "{events}:\n{state}");
        assert!(state.contains(&format!("\nrenderer-site http://{host}\n")), "{events}:\n{state}");
        assert!(p.len() == *count && distinct(&p), "{events}: renderers {p:?}\n{state}");
    }
}

/// Q3.10: with `DWELLER_PROCESS=tab`, a same-site navigation to a document
/// whose Cross-Origin-Opener-Policy does not match the current one
/// (`/coop/same-origin/...`) commits in a new renderer; the next one with
/// the same policy and origin stays in it.
#[test]
fn window_renderer_coop() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/ui/coop_nav.html", srv.base);
    let run = |events: &str| -> String {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "tab")
            .args([page.as_str(), "--font", "fixed", "--width", "400", "--height", "400"])
            .args(["--frames", "1000", "--until-idle", "--state", "--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        assert!(out.stderr.is_empty(), "{events}: {}", text(&out.stderr));
        text(&out.stdout)
    };
    let pids = |state: &str| -> usize {
        let line = state.lines().find(|l| l.starts_with("renderer-pids ")).unwrap_or("");
        line.split(' ').skip(1).count()
    };

    let state = run("renderer:loaded click:50,62 renderer:input renderer:loaded");
    assert!(state.contains(" title /coop/same-origin/ui/coop_nav.html\n"), "{state}");
    assert_eq!(pids(&state), 2, "a COOP mismatch swaps the renderer:\n{state}");

    let state = run("renderer:loaded click:50,62 renderer:input renderer:loaded click:50,62 renderer:input renderer:loaded");
    assert!(state.contains(" title /coop/same-origin/ui/coop_nav.html?again\n"), "{state}");
    assert_eq!(pids(&state), 2, "the same policy and origin keep the renderer:\n{state}");
}

/// Q2.7: with `DWELLER_PROCESS=tab` and `DWELLER_TEST=1`, the renderer
/// of the second tab acts compromised (`DWELLER_TEST_EVIL`) and sends one
/// forged message about another site: a cross-site frame's cookies
/// (`site` on `site-lock.html?cookie`) or storage (`?storage`), a
/// fetch whose initiator the browser never recorded, a pushState to
/// another site, the commit of a cross-site navigation without its
/// response. The browser kills that renderer (`bad message: REASON PID`)
/// and the window and the first tab's renderer live. Without the hook the
/// renderer does not ask for the cross-site frame's cookies or storage,
/// and nothing is killed.
#[test]
fn window_renderer_site_lock() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let first = format!("{}/ui/renderer_sites.html", srv.base);
    let evil = format!("{}/ui/site-lock.html", srv.base);
    let run = |hook: &str, second: &str, then: &str| -> (String, String) {
        let mut cmd = dweller_command();
        cmd.current_dir(root()).env("DWELLER_PROCESS", "tab").env("DWELLER_TEST", "1");
        if !hook.is_empty() {
            cmd.env("DWELLER_TEST_EVIL", hook);
        }
        let events = format!("renderer:loaded key:cmd+t {} key:return renderer:loaded {then} key:cmd+1 renderer:frame frame", typed(second));
        let out = cmd
            .args([first.as_str(), "--font", "fixed", "--width", "400", "--height", "400"])
            .args(["--frames", "1000", "--until-idle", "--state", "--events", &events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{hook} {second}: {}\n{}", out.status, text(&out.stderr));
        (text(&out.stdout), text(&out.stderr))
    };
    let first_lives = |state: &str, killed: &str, case: &str| {
        let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or("");
        let r: Vec<&str> = line.splitn(4, ' ').collect();
        assert!(r.len() == 4 && r[1] != killed && r[2..] == ["title", "127.0.0.1"], "{case}: the first tab's renderer\n{state}");
        assert!(state.contains("tabs 2, showing 1"), "{case}: {state}");
    };

    for search in ["?cookie", "?storage"] {
        let (state, err) = run("", &format!("{evil}{search}"), "frame frame frame");
        assert!(err.is_empty(), "{search} without the hook: {err}");
        first_lives(&state, "", search);
    }

    let cases = [
        ("site", format!("{evil}?cookie"), "renderer:gone", "cookie outside site lock"),
        ("site", format!("{evil}?storage"), "renderer:gone", "storage outside site lock"),
        ("initiator", evil.clone(), "renderer:gone", "bad fetch"),
        ("pushstate", evil.clone(), "renderer:gone", "bad url"),
        ("commit", evil.clone(), "click:50,88 renderer:gone", "commit outside site lock"),
    ];
    for (hook, second, then, why) in &cases {
        let (state, err) = run(hook, second, then);
        let pid = err.strip_prefix(&format!("error: bad message: {why} ")).and_then(|p| p.strip_suffix('\n'));
        let pid = pid.unwrap_or_else(|| panic!("{hook} {second}: stderr {err:?}\n{state}"));
        assert!(pid.parse::<u32>().is_ok(), "{hook}: {err}");
        first_lives(&state, pid, hook);
    }
}

/// Q2.3c: with `DWELLER_PROCESS=tab` the renderer runs the default
/// actions (`browser/tests/ui/renderer_actions.html`): a link click and
/// a form POST are asked of the browser, which navigates the renderer,
/// and refuses a `file:` link of an http page; a `select` choice comes
/// back from the window's dropdown; Space presses a button; keys scroll
/// the page. Each gives the same PNG as the in-process window.
#[test]
fn window_renderer_input_actions() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let run = |page: &str, png: &Path, events: &str, envs: &[(&str, &str)]| -> (String, String) {
        let out = dweller_command()
            .current_dir(root())
            .envs(envs.iter().copied())
            .args([page, "--font", "fixed", "--width", "400", "--height", "340"])
            .args(["--frames", "1000", "--until-idle", "--state", "--save"])
            .arg(png)
            .args(["--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        (text(&out.stdout), text(&out.stderr))
    };
    let secret = format!("file://{}/browser/tests/no-file/secret.html", root().display());
    let page = format!("{}/ui/renderer_actions.html?{secret}", srv.base);
    let page_area = |png: &Path| {
        let (w, _, rgba) = png_rgba(&std::fs::read(png).unwrap());
        rgba[40 * w * 4..].to_vec()
    };
    let tab = [("DWELLER_PROCESS", "tab")];
    let still = scratch().join("renderer_actions_still.png");
    let golden = scratch().join("renderer_actions_in_process.png");
    let shot = scratch().join("renderer_actions_tab.png");
    let cases = [
        ("link", page.as_str(), "click:80,65 renderer:input renderer:loaded"),
        ("form post", page.as_str(), "click:260,165 renderer:input renderer:loaded"),
        ("select", page.as_str(), "click:60,215 release:60,215 renderer:input click:60,260 release:60,260"),
        ("space", page.as_str(), "click:60,115 key:space keyup:space"),
        ("key scroll", "browser/tests/ui/renderer_input.html", "click:390,330 key:pagedown key:down"),
    ];
    for (what, page, events) in cases {
        let in_process_events: Vec<&str> = events.split(' ').filter(|w| !w.starts_with("renderer:")).collect();
        let (_, err) = run(page, &still, "renderer:loaded renderer:frame", &tab);
        assert!(err.is_empty(), "{err}");
        let (_, err) = run(page, &golden, &format!("loaded {} loaded move:390,330", in_process_events.join(" ")), &[]);
        assert!(err.is_empty(), "{what}: {err}");
        let (state, err) = run(page, &shot, &format!("renderer:loaded renderer:frame {events} move:390,330 renderer:input"), &tab);
        assert!(err.is_empty(), "{what}: {err}");
        assert!(std::fs::read(&shot).unwrap() != std::fs::read(&still).unwrap(), "{what}: no change in {}", shot.display());
        assert!(
            page_area(&shot) == page_area(&golden),
            "{what}: the renderer's frame {} differs from the in-process window {}\n{state}",
            shot.display(),
            golden.display()
        );
    }

    run(&page, &still, "renderer:loaded renderer:frame", &tab);
    let (_, err) = run(&page, &shot, "renderer:loaded renderer:frame click:275,65 renderer:input move:390,330 renderer:input", &tab);
    assert_eq!(err, format!("error: navigation refused: {secret}\n"));
    assert!(page_area(&shot) == page_area(&still), "file: link: the page changed in {}", shot.display());
}

/// Q2.3e: with `DWELLER_PROCESS=tab` the window keeps no copy of the
/// page (`browser/tests/ui/renderer_status.html`): its image is fetched
/// once, by the renderer. The renderer reports the hover link (status
/// line) and the cursor shape, a task past 5 s ("The page is busy" until
/// it ends), Tab past the last element (the address bar takes the focus),
/// and the window's blur fires `blur` in the page. A cross-origin
/// redirect commits in the renderer without a second fetch, and a chain
/// of 25 redirects ends on an error page (20 is the limit).
#[test]
fn window_renderer_status() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let window = |page: &str, events: &str| -> String {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "tab")
            .args([page, "--font", "fixed", "--width", "400", "--height", "340"])
            .args(["--frames", "2000", "--until-idle", "--state", "--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        assert!(out.stderr.is_empty(), "{events}: {}", text(&out.stderr));
        text(&out.stdout)
    };
    let hits = |path: &str| {
        let log = text(&run(&["dump", "--text", &format!("{}/log", srv.base)]).stdout);
        log.matches(&format!("GET {path} ")).count() + log.matches(&format!("GET {path}\n")).count()
    };
    let page = format!("{}/ui/renderer_status.html", srv.base);

    let state = window(&page, "renderer:loaded renderer:frame move:30,60 renderer:input");
    assert!(state.contains(&format!("\nstatus {}/ui/target.html\n", srv.base)), "{state}");
    assert!(state.contains("\nrenderer-cursor pointer\n"), "{state}");
    assert_eq!(hits("/log/renderer-status-nostore.png"), 1, "the window loaded its own copy of the page");

    let state = window(&page, "renderer:loaded renderer:frame move:30,60 renderer:input move:30,102 renderer:input");
    assert!(state.contains("\nstatus \n") && state.contains("\nrenderer-cursor text\n"), "{state}");

    let state = window(&page, "renderer:loaded renderer:frame window:blur frame frame move:300,300 renderer:input");
    assert!(state.contains(" title Blurred\n"), "{state}");

    let state = window(&page, "renderer:loaded renderer:frame key:tab key:tab key:tab renderer:input");
    assert!(state.contains("\nfocus page\n"), "{state}");
    let state = window(&page, "renderer:loaded renderer:frame key:tab key:tab key:tab key:tab renderer:input frame frame");
    assert!(state.contains("\nfocus address\n"), "{state}");

    let state = window(&page, "renderer:loaded renderer:frame click:30,145 renderer:busy");
    assert!(state.contains("\nstatus The page is busy\n"), "{state}");
    let state = window(&page, "renderer:loaded renderer:frame click:30,145 renderer:input frame");
    assert!(state.contains(" title Ran\n") && state.contains("\nstatus \n"), "{state}");

    let port = srv.base.rsplit(':').next().unwrap();
    let landed = format!("http://localhost:{port}/log/renderer-hop-nostore.html");
    let state = window(&format!("{}/samesite/go?to={landed}", srv.base), "renderer:loaded renderer:frame");
    assert!(state.contains(&format!("\naddress {landed}\n")) && state.contains(" title Logged\n"), "{state}");
    assert_eq!(hits("/log/renderer-hop-nostore.html"), 1, "the cross-origin redirect was fetched twice");

    let state = window(&format!("{}/hops/20", srv.base), "renderer:loaded renderer:frame");
    assert!(state.contains(" title Hops\n"), "{state}");
    let state = window(&format!("{}/hops/25", srv.base), "renderer:loaded renderer:frame");
    assert!(!state.contains(" title Hops\n") && state.contains("\nrenderer-frame "), "{state}");
}

/// Q2.3b: with `DWELLER_PROCESS=tab` a window's agent commands are
/// answered by the tab's renderer, byte for byte as by the window
/// itself; a renderer that answers a request id it was not sent is
/// killed.
#[test]
fn window_renderer_agent() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let requests = [
        "{\"id\":1,\"cmd\":\"navigate\",\"url\":\"browser/tests/agent/site/todo.html\"}",
        "{\"id\":2,\"cmd\":\"type\",\"ref\":\"e2\",\"text\":\"Buy milk\",\"submit\":true}",
        "{\"id\":3,\"cmd\":\"snapshot\"}",
        "{\"id\":4,\"cmd\":\"eval\",\"expression\":\"document.title\"}",
        "{\"id\":5,\"cmd\":\"click\",\"ref\":\"e4\"}",
        "{\"id\":6,\"cmd\":\"read\"}",
        "{\"id\":7,\"cmd\":\"tabs\"}",
    ];
    let session = |sock: &str, env: &[(&str, &str)]| -> (Vec<String>, String) {
        let window = spaces_window_env(sock, &[], env);
        let mut conn = AgentConn::open(sock);
        let got: Vec<String> = requests.iter().map(|r| conn.ask(r).0).collect();
        drop(conn);
        let out = window.wait_with_output().unwrap();
        (got, text(&out.stderr))
    };
    let (want, _) = session(&format!("target/sockets/q23b-in-{}.sock", std::process::id()), &[]);
    let (got, err) = session(&format!("target/sockets/q23b-tab-{}.sock", std::process::id()), &[("DWELLER_PROCESS", "tab")]);
    assert!(want[0].contains("\"ok\":true"), "{}", want[0]);
    assert_eq!(got, want, "{err}");
    let (bad, err) = session(
        &format!("target/sockets/q23b-evil-{}.sock", std::process::id()),
        &[("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "agent")],
    );
    assert!(err.contains("bad message: bad agent_reply "), "{err}");
    assert!(bad[0].contains("the tab's renderer is gone"), "{}", bad[0]);
}

/// Q2.7b: with `DWELLER_PROCESS=tab` an agent's navigation to another
/// site (`127.0.0.1` and `localhost`) commits in a new renderer, as the
/// user's do: one process of the window's tree ends and another starts at
/// each swap, and the agent's next command is answered by the new page.
#[test]
fn window_renderer_agent_sites() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    fn tree(pid: &str) -> Vec<String> {
        let out = Command::new("pgrep").args(["-P", pid]).output().unwrap();
        let kids: Vec<String> = text(&out.stdout).lines().map(str::to_string).collect();
        kids.iter().flat_map(|k| tree(k)).chain(kids.clone()).collect()
    }
    let page = format!("{}/ui/renderer_sites.html", srv.base);
    let other = page.replace("127.0.0.1", "localhost");
    let sock = format!("target/sockets/q27b-{}.sock", std::process::id());
    let window = spaces_window_env(&sock, &[], &[("DWELLER_PROCESS", "tab")]);
    let mut conn = AgentConn::open(&sock);
    let mut trees: Vec<std::collections::HashSet<String>> = Vec::new();
    for (n, (to, host)) in [(&page, "127.0.0.1"), (&other, "localhost"), (&page, "127.0.0.1")].into_iter().enumerate() {
        let went = conn.ask(&format!("{{\"id\":{n},\"cmd\":\"navigate\",\"url\":\"{to}\"}}")).0;
        assert!(went.starts_with(&format!("{{\"id\":{n},\"ok\":true,")) && went.contains(&format!("\"url\":\"{to}\"")), "{went}");
        let title = conn.ask("{\"id\":9,\"cmd\":\"eval\",\"expression\":\"document.title\"}").0;
        assert!(title.contains(&format!("\"result\":\"{host}\"")), "{to}: {title}");
        trees.push(tree(&window.id().to_string()).into_iter().collect());
    }
    drop(conn);
    let out = window.wait_with_output().unwrap();
    for w in trees.windows(2) {
        assert!(w[1].difference(&w[0]).count() == 1 && w[0].difference(&w[1]).count() == 1, "no swap: {trees:?}");
    }
    assert!(!text(&out.stderr).contains("bad message"), "{}", text(&out.stderr));
}

/// Q2.7c: with `DWELLER_PROCESS=tab` a ref of a page the tab left is
/// refused, as in a window without the flag, whether the navigation
/// stayed in the renderer or swapped it; an `eval` whose script navigates
/// to another site gets an answer.
#[test]
fn window_renderer_agent_refs() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/ui/renderer_sites.html", srv.base);
    let other = page.replace("127.0.0.1", "localhost");
    let same = format!("{page}?same");
    let sock = format!("target/sockets/q27c-{}.sock", std::process::id());
    let window = spaces_window_env(&sock, &[], &[("DWELLER_PROCESS", "tab")]);
    let mut conn = AgentConn::open(&sock);
    let mut n = 0;
    let mut ask = |conn: &mut AgentConn, body: String| {
        n += 1;
        conn.ask(&format!("{{\"id\":{n},{body}}}")).0
    };
    let mut snapped = |conn: &mut AgentConn, to: &str| {
        let went = ask(conn, format!("\"cmd\":\"navigate\",\"url\":\"{page}\""));
        assert!(went.contains("\"ok\":true"), "{went}");
        let old = ref_named(&ask(conn, "\"cmd\":\"snapshot\"".into()), "same site");
        let went = ask(conn, format!("\"cmd\":\"navigate\",\"url\":\"{to}\""));
        assert!(went.contains("\"ok\":true"), "{went}");
        let clicked = ask(conn, format!("\"cmd\":\"click\",\"ref\":\"{old}\""));
        assert!(clicked.contains(&format!("\"ok\":false,\"error\":\"unknown ref {old}:")), "{to}: {clicked}");
        ask(conn, "\"cmd\":\"eval\",\"expression\":\"document.title\"".into())
    };
    let title = snapped(&mut conn, &same);
    assert!(title.contains("\"result\":\"127.0.0.1?same\""), "same site: {title}");
    let title = snapped(&mut conn, &other);
    assert!(title.contains("\"result\":\"localhost\""), "other site: {title}");
    drop(snapped);
    let went = ask(&mut conn, format!("\"cmd\":\"navigate\",\"url\":\"{page}\""));
    assert!(went.contains("\"ok\":true"), "{went}");
    let old = ref_named(&ask(&mut conn, "\"cmd\":\"snapshot\"".into()), "same site");
    let evaled = conn.ask(&format!("{{\"id\":99,\"cmd\":\"eval\",\"expression\":\"location.href = '{other}'\"}}")).0;
    assert!(evaled.starts_with("{\"id\":99,"), "{evaled}");
    let mut title = String::new();
    for _ in 0..50 {
        title = ask(&mut conn, "\"cmd\":\"eval\",\"expression\":\"document.title\"".into());
        if title.contains("\"result\":\"localhost\"") {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    assert!(title.contains("\"result\":\"localhost\""), "the eval did not navigate: {title}");
    let clicked = ask(&mut conn, format!("\"cmd\":\"click\",\"ref\":\"{old}\""));
    assert!(clicked.contains(&format!("unknown ref {old}:")), "after the eval: {clicked}");
    drop(conn);
    let out = window.wait_with_output().unwrap();
    assert!(!text(&out.stderr).contains("bad message"), "{}", text(&out.stderr));
}
