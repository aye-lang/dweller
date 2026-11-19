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
        .args(["--frames", "1000", "--until-idle", "--state", "--processes", "--save"])
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
            .args(["--font", "fixed", "--frames", "300", "--until-idle", "--state", "--processes", "--events", events])
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

fn run_state(page: &str, mode: &str, events: &str, evil: &str) -> (String, String) {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", mode)
        .env("DWELLER_TEST", "1")
        .env("DWELLER_TEST_EVIL", evil)
        .arg(page)
        .args(["--font", "fixed", "--width", "400", "--height", "340"])
        .args(["--frames", "600", "--until-idle", "--state", "--processes", "--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{page} {events}: {}\n{}", out.status, text(&out.stderr));
    (text(&out.stdout), text(&out.stderr))
}

/// Q2.15c: the initiator's sandbox flags in tab and site mode. The
/// renderer runs "allowed by sandboxing to navigate" for a person's
/// click on a `target=_top` link or form of a sandboxed frame, as the
/// in-process window does (`window_sandbox_targets`). And the browser
/// holds a sandboxed cross-site frame's renderer to the flags it sent
/// it: a POST navigation without `allow-forms`, and a cookie or storage
/// request without `allow-same-origin`, kill that renderer only.
#[test]
fn window_renderer_oopif_site_sandbox() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let targets = [("click:10,60", "sandbox_targets.html"), ("click:10,100", "other.html"), ("click:10,140", "sandbox_targets.html"), ("click:10,180", "other.html?")];
    for mode in ["tab", "site"] {
        for (click, want) in targets {
            let events = format!("renderer:loaded {click} renderer:input frame frame frame");
            let (state, err) = run_state("browser/tests/ui/sandbox_targets.html", mode, &events, "");
            assert!(err.is_empty(), "{mode} {click}: {err}");
            assert!(line(&state, "url ").ends_with(&format!("/browser/tests/ui/{want}")), "{mode} {click}: wanted {want}\n{state}");
        }
    }

    let page = format!("{}/ui/oopif-sandbox.html", srv.base);
    let (state, err) = run_state(&page, "site", "renderer:loaded renderer:child frame", "");
    assert!(err.is_empty(), "{err}");
    let children = kids(&state);
    assert_eq!(children.len(), 1, "{state}");
    assert!(children[0].ends_with(" title Sandboxed cookie SecurityError storage SecurityError"), "the form is not submitted, the origin is opaque\n{state}");

    let forged = [("sandbox-form", "form in a sandboxed frame"), ("sandbox-cookie", "cookie in a sandboxed origin"), ("sandbox-storage", "storage in a sandboxed origin")];
    for (evil, why) in forged {
        let (state, err) = run_state(&page, "site", "renderer:loaded renderer:child-gone frame", evil);
        let children = kids(&state);
        assert!(children.len() == 1 && children[0].ends_with(&format!(" http://localhost gone {why}")), "{evil}:\n{state}");
        assert_eq!(err, format!("error: bad message: {why} {}\n", pid(children[0])), "{evil}");
        assert!(line(&state, "renderer ").ends_with(" title Parent"), "{evil}: the parent lives\n{state}");
    }
}

fn far_pixels(a: &Path, b: &Path, by: u8) -> usize {
    let (w, h, got) = png_rgba(&std::fs::read(a).unwrap());
    let (gw, gh, want) = png_rgba(&std::fs::read(b).unwrap());
    assert_eq!((w, h), (gw, gh));
    got.chunks(4).zip(want.chunks(4)).filter(|(p, q)| p.iter().zip(q.iter()).any(|(x, y)| x.abs_diff(*y) > by)).count()
}

/// Q2.15d: the window shows a cross-site frame as its parent's paint
/// places it: clipped by an ancestor's `overflow: hidden`, moved and
/// clipped by an ancestor scroll container's scroll offset, moved by an
/// ancestor's `transform: translate()` (all byte-equal to the in-process
/// window), and blended at an ancestor's `opacity` (each channel within 1
/// of the in-process window, which rounds once where the two processes
/// round twice).
#[test]
fn window_renderer_oopif_site_effects() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let golden = scratch().join("oopif_site_effects_in_process.png");
    let shot = scratch().join("oopif_site_effects.png");
    let cases = [("?hidden", "at 20 40 ", 0u8), ("?scrolled", "at 20 -5 ", 0), ("?moved", "at 50 60 ", 0), ("?faded", "at 20 40 ", 1)];
    for (query, at, by) in cases {
        let page = format!("oopif-effects.html{query}");
        run_site(&srv.base, &page, "", &golden, "loaded frame frame");
        let state = run_site(&srv.base, &page, "site", &shot, "renderer:loaded renderer:child frame");
        let children = kids(&state);
        assert!(children.len() == 1 && children[0].contains(" frame 200 100 ") && children[0].contains(at), "{query}:\n{state}");
        assert_eq!(far_pixels(&shot, &golden, by), 0, "{query}: {} differs from the in-process window {}", shot.display(), golden.display());
    }
}

fn title_of(line: &str) -> &str {
    line.split_once(" title ").map(|(_, t)| t).unwrap_or("")
}

/// Q2.15e: the hit-test slow path. Where the parent paints something
/// above a cross-site frame, or the frame has `pointer-events: none`,
/// the browser asks the parent's renderer which frame is at the point
/// before it sends the mouse event: a click on the cover goes to the
/// parent, a click beside it (and one through a `pointer-events: none`
/// cover) to the child at child-local coordinates, and a click on a
/// `pointer-events: none` frame to the parent. A reply nobody asked for
/// kills its renderer.
#[test]
fn window_renderer_oopif_site_hit_test() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let on_cover = "click:52,106 release:52,106 renderer:input";
    let beside = "click:172,136 release:172,136 renderer:input";
    let cases = [
        ("?cover", on_cover, "P up", "C"),
        ("?cover", beside, "P", "C down 150,60,click,up 150,60"),
        ("?through", on_cover, "P", "C down 30,30,click,up 30,30"),
        ("?none", beside, "P up", "C"),
    ];
    for (query, clicks, parent, child) in cases {
        let page = format!("{}/ui/oopif-input.html{query}", srv.base);
        let events = format!("renderer:loaded renderer:child {clicks} frame");
        let (state, err) = run_state(&page, "site", &events, "");
        assert!(err.is_empty(), "{query} {clicks}: {err}");
        let children = kids(&state);
        assert_eq!(children.len(), 1, "{query}:\n{state}");
        assert_eq!(title_of(line(&state, "renderer ")), parent, "{query} {clicks}: the parent\n{state}");
        assert_eq!(title_of(children[0]), child, "{query} {clicks}: the child\n{state}");
    }

    let page = format!("{}/ui/oopif-input.html", srv.base);
    let (state, err) = run_state(&page, "site", "renderer:gone", "hit-reply");
    let renderer = line(&state, "renderer ");
    assert!(renderer.ends_with(" gone unexpected hit_test_reply"), "{state}");
    assert_eq!(err, format!("error: bad message: unexpected hit_test_reply {}\n", pid(renderer)));
}

/// Q2.15f: `location.replace()` to another document replaces the current
/// session history entry. `replace.html` goes to `?second` (a new
/// entry), which replaces itself by `?third`: two entries, and Back
/// (`cmd+[`) shows the first page, not `?second`. A
/// replace that crosses sites (`?go` goes to `?far`, which replaces
/// itself by a page of another site: a renderer swap with
/// `DWELLER_PROCESS=site`) keeps the count too. The same in the window's
/// own process.
#[test]
fn window_renderer_oopif_site_replace() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let shot = scratch().join("oopif_site_replace.png");
    let far = srv.base.replace("127.0.0.1", "localhost");
    for mode in ["", "tab", "site"] {
        let (wait, back, landed) = if mode.is_empty() {
            ("loaded frame frame", "key:cmd+[ painted loaded frame", "loaded frame frame")
        } else {
            ("renderer:loaded renderer:title:?third_2 frame", "key:cmd+[ renderer:loaded renderer:title:Replace_2 frame", "renderer:loaded renderer:title:?landed_2 frame")
        };

        let state = run_site(&srv.base, "replace.html", mode, &shot, wait);
        assert_eq!(line(&state, "url "), format!("url {}/ui/replace.html?third", srv.base), "{mode}:\n{state}");
        assert_eq!(line(&state, "history "), "history 2 of 2", "{mode}:\n{state}");

        let state = run_site(&srv.base, "replace.html", mode, &shot, &format!("{wait} {back}"));
        assert_eq!(line(&state, "url "), format!("url {}/ui/replace.html", srv.base), "{mode}: Back skips the replaced entry\n{state}");
        assert_eq!(line(&state, "history "), "history 1 of 2", "{mode}:\n{state}");

        let state = run_site(&srv.base, "replace.html?go", mode, &shot, landed);
        assert_eq!(line(&state, "url "), format!("url {far}/ui/replace.html?landed"), "{mode}:\n{state}");
        assert_eq!(line(&state, "history "), "history 2 of 2", "{mode}: the entry of ?far was replaced\n{state}");
    }
}

/// Q2.15g: on Linux without Landlock, seccomp refuses every open after
/// the renderer entered its sandbox, and the fonts opened late (fallback
/// faces for CJK, Arabic, Hebrew and symbols, bold italic serif, the
/// canvas's own system faces) were missing. The renderer now maps the
/// font files before it enters. With Landlock turned off for the test
/// (the probe's refusals are all EPERM, none is Landlock's EACCES) the
/// page with `DWELLER_PROCESS=site` is byte-equal to the in-process
/// window's, as it is with Landlock.
#[test]
fn window_renderer_oopif_site_late_fonts() {
    if !has_sdl() || !cfg!(target_os = "linux") {
        println!("skipped: Linux with SDL2 only");
        return;
    }
    let home = scratch().join("late_fonts_home");
    let profile = home.join(".dweller");
    std::fs::create_dir_all(&profile).unwrap();
    let run = |png: &Path, events: &str, envs: &[(&str, &str)]| -> String {
        let out = dweller_command()
            .current_dir(root())
            .env_remove("DWELLER_PROCESS")
            .env("HOME", &home)
            .env("DWELLER_PROFILE", &profile)
            .envs(envs.iter().copied())
            .args(["browser/tests/ui/late-fonts.html", "--width", "400", "--height", "300"])
            .args(["--frames", "600", "--until-idle", "--save"])
            .arg(png)
            .args(["--events", events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
        text(&out.stderr)
    };
    let golden = scratch().join("late_fonts_in_process.png");
    let shot = scratch().join("late_fonts_site.png");
    let site = [("DWELLER_PROCESS", "site"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_SANDBOX_PROBE", "1")];
    let seccomp_only = [site[0], site[1], site[2], ("DWELLER_TEST_NO_LANDLOCK", "1")];

    let err = run(&golden, "loaded frame", &[]);
    assert!(err.is_empty(), "{err}");

    let err = run(&shot, "renderer:loaded renderer:frame frame", &seccomp_only);
    assert_eq!(err, "sandbox-probe home=1 tmp=1 tcp=1 fork=1\n", "seccomp alone refuses with EPERM");
    assert!(std::fs::read(&shot).unwrap() == std::fs::read(&golden).unwrap(), "without Landlock: {} differs from {}", shot.display(), golden.display());

    let err = run(&shot, "renderer:loaded renderer:frame frame", &site);
    assert!(err.starts_with("sandbox-probe home="), "{err}");
    assert!(std::fs::read(&shot).unwrap() == std::fs::read(&golden).unwrap(), "with Landlock: {} differs from {}", shot.display(), golden.display());
}
