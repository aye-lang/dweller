use super::*;

/// The History API in the window (`browser/tests/agent/site/spa.html`):
/// buttons that call pushState change the address without loading, and
/// back moves in the document (popstate shows item 2 again).
#[test]
fn window_history_api() {
    window_history_api_in(&[]);
}

/// Q2.6a: the same with `DWELLER_PROCESS=tab`: the browser records the
/// renderer's pushState entries, and back is a traversal in the
/// renderer's document.
#[test]
fn window_history_api_in_renderer() {
    window_history_api_in(&[("DWELLER_PROCESS", "tab")]);
}

fn window_history_api_in(envs: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let (frames, events) = if envs.is_empty() {
        ("5", "click:30,133 click:140,133 key:cmd+[")
    } else {
        ("1000", "renderer:loaded click:30,133 renderer:input click:140,133 renderer:input key:cmd+[ renderer:loaded")
    };
    let out = dweller_command()
        .current_dir(root())
        .envs(envs.iter().copied())
        .args(["browser/tests/agent/site/spa.html", "--font", "fixed", "--frames", frames, "--state", "--text", "--events", events])
        .args(if envs.is_empty() { &[][..] } else { &["--until-idle"][..] })
        .output()
        .unwrap();
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let page = format!(
        "file://{}/browser/tests/agent/site/spa.html?item=2",
        root().display()
    );
    for want in [
        format!("url {page}\n"),
        format!("\naddress {page}\n"),
        "\nhistory 2 of 3\n".into(),
        "\nItem 2\n".into(),
        "\nhistory.length 3\n".into(),
    ] {
        assert!(got.contains(&want), "expected {want:?} in:\n{got}");
    }
}

/// Q1.9: what `SESSION_EVENTS` makes in an empty window: a tab with two
/// entries, back on the first, and a second tab, shown.
const SESSION_EVENTS: &str = "key:cmd+l text:browser/tests/basic.html key:return loaded key:cmd+l text:browser/tests/article.html key:return loaded key:cmd+[ loaded key:cmd+t text:browser/tests/attributes.html key:return loaded";

fn session_run(profile: &Path, restore: &str, events: &str) -> (String, String) {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROFILE", profile)
        .env("DWELLER_RESTORE", restore)
        .args(["--width", "800", "--height", "600", "--frames", "300", "--until-idle", "--state", "--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
    (text(&out.stdout), text(&out.stderr))
}

fn shows(got: &str, page: &str, history: &str, tabs: &str) -> bool {
    got.contains(&format!("/browser/tests/{page}\nscroll ")) && got.contains(&format!("\nhistory {history}\n")) && got.contains(&format!("\ntabs {tabs}\n"))
}

// The window started again on `profile` has the tabs of `SESSION_EVENTS`:
// the second one shown; the first, loaded when shown, on its first entry
// with the second ahead of it.
fn session_restored(profile: &Path) {
    let (got, err) = session_run(profile, "1", "frame");
    assert!(shows(&got, "attributes.html", "1 of 1", "2, showing 2") && err.is_empty(), "{got}\n{err}");
    let (got, err) = session_run(profile, "1", "key:cmd+1 loaded");
    assert!(shows(&got, "basic.html", "1 of 2", "2, showing 1") && err.is_empty(), "{got}\n{err}");
    let (got, err) = session_run(profile, "1", "key:cmd+] loaded");
    assert!(shows(&got, "article.html", "2 of 2", "2, showing 1") && err.is_empty(), "{got}\n{err}");
}

/// Q1.9: the window killed (SIGKILL) once its session was saved (at most
/// a second after the last change) comes back with its tabs.
#[test]
fn window_session_restore_after_kill() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = profile("session-kill");
    let mut window = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROFILE", &dir)
        .env("DWELLER_RESTORE", "1")
        .args(["--width", "800", "--height", "600", "--frames", "16384", "--events", SESSION_EVENTS])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    let start = std::time::Instant::now();
    while !std::fs::read_to_string(dir.join("session")).is_ok_and(|s| s.starts_with("dweller-session 1\n1\n") && s.ends_with("/attributes.html\nend\n")) {
        if start.elapsed() > std::time::Duration::from_secs(30) {
            let _ = window.kill();
            panic!("no session saved: {:?}", std::fs::read_to_string(dir.join("session")));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let status = Command::new("kill").args(["-9", &window.id().to_string()]).status().unwrap();
    assert!(status.success());
    window.wait().unwrap();
    session_restored(&dir);
}

/// Q1.9: after a clean quit the tabs come back too. DWELLER_RESTORE=0
/// starts fresh; a torn session file starts a new tab, and says so.
#[test]
fn window_session_restore_after_quit() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = profile("session-quit");
    let (_, err) = session_run(&dir, "1", SESSION_EVENTS);
    assert!(err.is_empty(), "{err}");
    session_restored(&dir);
    let (got, _) = session_run(&dir, "0", "frame");
    assert!(got.starts_with("url \n") && !got.contains("\ntabs "), "{got}");
    let saved = std::fs::read(dir.join("session")).unwrap();
    std::fs::write(dir.join("session"), &saved[..saved.len() / 2]).unwrap();
    let (got, err) = session_run(&dir, "1", "frame");
    assert!(got.starts_with("url \n") && !got.contains("\ntabs "), "{got}");
    assert_eq!(err, "error: the saved session is unreadable: starting with a new tab\n");
}

/// P4.46: back, forward and reload load in steps
/// (`browser/tests/history/`: a sheet delayed 0.3 s and a script delayed
/// 0.6 s that makes the page tall, neither cached). The page is painted
/// before its script runs, the address bar takes typing during the load,
/// no loop iteration waits for the network, and the scroll offset each
/// entry was left at comes back once the script made the page tall enough.
#[test]
fn window_history_steps() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let a = format!("{}/history/a.html", srv.base);
    let b = format!("{}/history/b.html", srv.base);
    let run = |page: &str, events: &str| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_TIMING", "1")
            .args([
                page, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "600",
                "--until-idle", "--state", "--text", "--events", events,
            ])
            .output()
            .unwrap();
        let err = text(&out.stderr);
        assert!(out.status.success(), "{events}: {}\n{err}", out.status);
        let loops = iterations(&err);
        assert_no_net_wait(&loops, page);
        (text(&out.stdout), loops)
    };
    let scroll = |got: &str| -> String {
        got.lines()
            .find(|l| l.starts_with("scroll "))
            .unwrap_or("")
            .to_string()
    };
    let (got, _) = run(&a, "loaded wheel:-10 frame");
    let at_a = scroll(&got);
    let (got, _) = run(&b, "loaded wheel:-5 frame");
    let at_b = scroll(&got);
    assert!(at_a != "scroll 0" && at_b != "scroll 0" && at_a != at_b, "{at_a}, {at_b}");

    let to_b = "loaded wheel:-10 frame click:20,60 painted loaded wheel:-5 frame";
    let (got, loops) = run(&a, &format!("{to_b} key:cmd+[ key:cmd+l text:abc painted loaded"));
    for want in [
        format!("url {a}\n"),
        format!("\n{at_a}\n"),
        "\nfocus address\naddress abc\nhistory 1 of 2\n".into(),
        "\nran /history/a.html\n".into(),
    ] {
        assert!(got.contains(&want), "back: expected {want:?} in:\n{got}");
    }
    let last = |name: &str, label: &str| {
        loops
            .iter()
            .rposition(|it| it.has(name, label))
            .unwrap_or_else(|| panic!("no {name} record of {label:?}"))
    };
    let back = last("ui_navigate", "");
    let painted = last("ui_first_paint", &a);
    let script = last("script", "/history/nostore.js");
    assert!(
        back < painted && painted < script,
        "back in iteration {back}, painted in {painted}, the script ran in {script}"
    );

    let (got, _) = run(&a, &format!("{to_b} key:cmd+[ painted loaded key:cmd+] painted loaded"));
    for want in [format!("url {b}\n"), format!("\n{at_b}\n"), "\nhistory 2 of 2\n".into()] {
        assert!(got.contains(&want), "forward: expected {want:?} in:\n{got}");
    }

    let (got, _) = run(&a, "loaded wheel:-10 frame key:cmd+r painted loaded");
    for want in [format!("url {a}\n"), format!("\n{at_a}\n"), "\nhistory 1 of 1\n".into()] {
        assert!(got.contains(&want), "reload: expected {want:?} in:\n{got}");
    }
}
