use super::*;

/// Runs `browser/tests/ui/PAGE` on the test server in the window's own
/// process when `mode` is empty, else with `DWELLER_PROCESS=mode`; saves
/// the PNG and gives the `--state` text followed by stderr.
fn run_site(base: &str, page: &str, mode: &str, png: &Path, events: &str) -> String {
    let mut command = dweller_command();
    if mode.is_empty() {
        command.env_remove("DWELLER_PROCESS");
    } else {
        command.env("DWELLER_PROCESS", mode);
    }
    let out = command
        .current_dir(root())
        .arg(format!("{base}/ui/{page}"))
        .args(["--font", "fixed", "--width", "400", "--height", "400"])
        .args(["--frames", "1000", "--until-idle", "--state", "--save"])
        .arg(png)
        .args(["--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{page} {events}: {}\n{}", out.status, text(&out.stderr));
    assert!(out.stderr.is_empty(), "{page} {events}: {}", text(&out.stderr));
    text(&out.stdout)
}

fn line<'a>(state: &'a str, head: &str) -> &'a str {
    state.lines().find(|l| l.starts_with(head)).unwrap_or("")
}

fn kids(state: &str) -> Vec<&str> {
    state.lines().filter(|l| l.starts_with("renderer-child ")).collect()
}

fn pid(line: &str) -> &str {
    line.split(' ').nth(1).unwrap_or("")
}

/// Q2.15a: the process of a frame is chosen for the URL after redirects.
/// A frame whose `src` is of the parent's site and redirects to another
/// site (`?hop`) runs in a renderer locked to that site, from the response
/// the browser kept (the parent's renderer gets no byte of it), and the
/// window shows the pixels of the in-process window. A redirect to a
/// document that refuses framing (`?hop-deny`) starts no renderer. A
/// frame that left for another site by itself (`?away`) keeps its place
/// when its parent reports it again.
#[test]
fn window_renderer_oopif_site_redirect() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let golden = scratch().join("oopif_site_redirect_in_process.png");
    let shot = scratch().join("oopif_site_redirect.png");

    run_site(&srv.base, "oopif.html?hop", "", &golden, "loaded");
    let state = run_site(&srv.base, "oopif.html?hop", "site", &shot, "renderer:loaded renderer:child frame");
    let children = kids(&state);
    assert_eq!(children.len(), 1, "the redirected frame has a renderer of its own\n{state}");
    assert!(children[0].contains(" http://localhost frame 200 100 at 22 40 "), "{state}");
    assert!(children[0].ends_with(" title Child localhost?hop"), "{state}");
    assert_ne!(pid(children[0]), pid(line(&state, "renderer ")), "{state}");
    assert!(state.contains("\nrenderer-site http://127.0.0.1\n"), "{state}");
    assert!(std::fs::read(&shot).unwrap() == std::fs::read(&golden).unwrap(), "?hop: {} differs from the in-process window {}", shot.display(), golden.display());

    let state = run_site(&srv.base, "oopif.html?hop", "tab", &shot, "renderer:loaded renderer:frame frame");
    assert!(kids(&state).is_empty(), "{state}");

    let state = run_site(&srv.base, "oopif.html?hop-deny", "site", &shot, "renderer:loaded renderer:frame frame");
    assert!(kids(&state).is_empty(), "a refused document starts no renderer\n{state}");
    assert!(line(&state, "renderer ").ends_with(" title Parent"), "{state}");

    let grown = "renderer:loaded renderer:child click:60,52 renderer:input renderer:child frame";
    let state = run_site(&srv.base, "oopif.html?away", "site", &shot, grown);
    let children = kids(&state);
    assert!(line(&state, "renderer ").ends_with(" title Parent"), "{state}");
    assert!(children.len() == 1 && children[0].contains(" http://127.0.0.1 frame 260 130 at 22 40 "), "{state}");
}

/// Q2.15b: a `blob:` URL has the site of its inner origin. A page of
/// `127.0.0.1` navigates to a `blob:http://localhost...` URL: the honest
/// renderer shows the error page (no such blob), and a renderer that
/// commits a document at that URL (`DWELLER_TEST_EVIL=blob`) is killed,
/// as for any commit outside its site lock.
#[test]
fn window_renderer_oopif_site_blob() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let run = |evil: &str, events: &str| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "site")
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_EVIL", evil)
            .arg(format!("{}/ui/blob-away.html", srv.base))
            .args(["--font", "fixed", "--frames", "300", "--until-idle", "--state", "--events", events])
            .output()
            .unwrap();
        (text(&out.stdout), text(&out.stderr))
    };

    let (state, err) = run("", "renderer:loaded frame frame frame");
    assert!(err.is_empty(), "{err}");
    assert!(line(&state, "url ").starts_with("url blob:http://localhost:"), "{state}");
    assert!(line(&state, "renderer ").ends_with(" title Cannot load page"), "{state}");
    assert!(state.contains("\nrenderer-site http://127.0.0.1\n"), "{state}");

    let (state, err) = run("blob", "renderer:gone");
    let renderer = line(&state, "renderer ");
    assert!(renderer.ends_with(" gone commit outside site lock"), "{state}");
    assert_eq!(err, format!("error: bad message: commit outside site lock {}\n", pid(renderer)));
    assert!(line(&state, "url ").ends_with("/ui/blob-away.html"), "the forged commit is not shown\n{state}");
}
