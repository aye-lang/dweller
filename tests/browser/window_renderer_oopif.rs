use super::*;

/// Runs `browser/tests/ui/oopif.html` (a `localhost` iframe and a same-site
/// one in a `127.0.0.1` page) with `query`, in the window's own process
/// when `mode` is empty, else with `DWELLER_PROCESS=mode`; saves the PNG
/// and gives the `--state` text.
fn run_oopif(base: &str, query: &str, mode: &str, png: &Path, events: &str, envs: &[(&str, &str)]) -> String {
    let mut command = dweller_command();
    if !mode.is_empty() {
        command.env("DWELLER_PROCESS", mode);
    }
    let out = command
        .current_dir(root())
        .envs(envs.iter().copied())
        .arg(format!("{base}/ui/oopif.html{query}"))
        .args(["--font", "fixed", "--width", "400", "--height", "400"])
        .args(["--frames", "1000", "--until-idle", "--state", "--save"])
        .arg(png)
        .args(["--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{query} {events}: {}\n{}", out.status, text(&out.stderr));
    if envs.is_empty() {
        assert!(out.stderr.is_empty(), "{query} {events}: {}", text(&out.stderr));
    }
    text(&out.stdout) + &text(&out.stderr)
}

fn state_line<'a>(state: &'a str, head: &str) -> &'a str {
    state.lines().find(|l| l.starts_with(head)).unwrap_or("")
}

fn children(state: &str) -> Vec<&str> {
    state.lines().filter(|l| l.starts_with("renderer-child ")).collect()
}

fn same_png(a: &Path, b: &Path, what: &str) {
    assert!(
        std::fs::read(a).unwrap() == std::fs::read(b).unwrap(),
        "{what}: the composited window {} differs from the in-process window {}",
        a.display(),
        b.display()
    );
}

/// Q2.9: with `DWELLER_PROCESS=site` the cross-site iframe is laid out as a
/// placeholder in its parent's renderer, runs in a renderer of its own,
/// and the window composites its frame at the iframe's content box: the
/// PNG is byte-equal to the window that runs both documents in its own
/// process, also after a wheel scroll of the parent that clips the child
/// (`?scroll`), after the iframe grows (the child gets buffers of the new
/// size), and after the iframe is removed (the child renderer ends).
#[test]
fn window_renderer_oopif_pixels() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let golden = scratch().join("oopif_in_process.png");
    let shot = scratch().join("oopif_site.png");
    let grow = "click:60,52";
    let cases = [
        ("", String::new(), "renderer:child".to_string(), "frame 200 100 at 22 40 "),
        ("?scroll", "move:300,300 wheel:-2 frame frame".to_string(), "renderer:child move:300,300 wheel:-2 renderer:input renderer:child".to_string(), "frame 200 100 at 22 -40 "),
        ("", format!("{grow} frame frame"), format!("renderer:child {grow} renderer:input renderer:child"), "frame 260 130 at 22 40 "),
        ("?remove", format!("{grow} frame frame"), format!("renderer:child {grow} renderer:input frame frame"), ""),
    ];
    for (query, in_process, site, child) in &cases {
        run_oopif(&srv.base, query, "", &golden, &format!("loaded {in_process}"), &[]);
        let state = run_oopif(&srv.base, query, "site", &shot, &format!("renderer:loaded {site} frame"), &[]);
        let kids = children(&state);
        let child: &str = child;
        if child.is_empty() {
            assert!(kids.is_empty(), "{query} {site}: the removed frame's renderer stays\n{state}");
        } else {
            assert_eq!(kids.len(), 1, "{query} {site}: one cross-site child, the same-site frame stays\n{state}");
            assert!(kids[0].contains(" http://localhost ") && kids[0].contains(child), "{query} {site}:\n{state}");
            assert!(kids[0].ends_with(" title Child localhost"), "{query} {site}:\n{state}");
        }
        same_png(&shot, &golden, &format!("{query} {site}"));
    }

    run_oopif(&srv.base, "?deny", "", &golden, "loaded", &[]);
    let state = run_oopif(&srv.base, "?deny", "site", &shot, "renderer:loaded renderer:child frame", &[]);
    assert!(children(&state)[0].ends_with(" title Cannot load page"), "X-Frame-Options: DENY is checked in the browser\n{state}");
    same_png(&shot, &golden, "?deny");
}

/// Q2.9: the cross-site frame's renderer is another process than its
/// parent's, locked to the frame's site; the frame's own navigations stay
/// in it (`?nav`, same site) or move it to a renderer of the new site
/// (`?away`); with `DWELLER_PROCESS=tab` the frame stays in the tab's
/// renderer; and a parent that names a same-site frame as remote is
/// killed (`bad subframes`).
#[test]
fn window_renderer_oopif_process() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let shot = scratch().join("oopif_process.png");
    let pid = |line: &str| line.split(' ').nth(1).unwrap_or("").to_string();

    let state = run_oopif(&srv.base, "", "site", &shot, "renderer:loaded renderer:child frame", &[]);
    let parent = pid(state_line(&state, "renderer "));
    let child = pid(children(&state)[0]);
    assert!(!parent.is_empty() && !child.is_empty() && parent != child, "{state}");
    assert!(state.contains("\nrenderer-site http://127.0.0.1\n") && state.contains(&format!("\nrenderer-pids {parent}\n")), "{state}");

    let state = run_oopif(&srv.base, "?nav", "site", &shot, "renderer:loaded renderer:child frame", &[]);
    let kids = children(&state);
    assert!(kids.len() == 1 && kids[0].contains(" http://localhost frame ") && kids[0].ends_with(" title Child localhost?navigated"), "{state}");

    let state = run_oopif(&srv.base, "?away", "site", &shot, "renderer:loaded renderer:child frame", &[]);
    let kids = children(&state);
    assert!(kids.len() == 1 && kids[0].contains(" http://127.0.0.1 frame ") && kids[0].ends_with(" title Child 127.0.0.1?arrived"), "{state}");
    assert_ne!(pid(kids[0]), pid(state_line(&state, "renderer ")), "{state}");

    let state = run_oopif(&srv.base, "", "tab", &shot, "renderer:loaded renderer:frame frame", &[]);
    assert!(children(&state).is_empty(), "{state}");

    let evil = [("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "subframes")];
    let state = run_oopif(&srv.base, "", "site", &shot, "renderer:gone", &evil);
    let line = state_line(&state, "renderer ");
    assert!(line.ends_with(" gone bad subframes"), "{state}");
    assert!(state.contains(&format!("error: bad message: bad subframes {}\n", pid(line))), "{state}");
}

/// Q2.9: a killed cross-site frame's renderer shows a sad frame in that
/// frame's rect only: the parent's renderer lives, and every pixel outside
/// the frame's content box is the in-process window's.
#[test]
fn window_renderer_oopif_crash() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let golden = scratch().join("oopif_crash_in_process.png");
    let shot = scratch().join("oopif_crash_site.png");

    run_oopif(&srv.base, "", "", &golden, "loaded", &[]);
    let state = run_oopif(&srv.base, "", "site", &shot, "renderer:loaded renderer:child renderer:kill-child renderer:child-gone frame frame", &[]);
    assert!(state_line(&state, "renderer ").ends_with(" title Parent"), "{state}");
    let kids = children(&state);
    assert!(kids.len() == 1 && kids[0].ends_with(" http://localhost gone crashed"), "{state}");

    let (w, h, a) = png_rgba(&std::fs::read(&shot).unwrap());
    let (_, _, b) = png_rgba(&std::fs::read(&golden).unwrap());
    let inside = |x: usize, y: usize| (22..222).contains(&x) && (80..180).contains(&y);
    for y in 0..h {
        for x in 0..w {
            let i = (y * w + x) * 4;
            if inside(x, y) {
                assert_eq!(&a[i..i + 4], &[232, 234, 237, 255], "sad frame at {x},{y}");
            } else {
                assert_eq!(&a[i..i + 4], &b[i..i + 4], "outside the frame at {x},{y}");
            }
        }
    }
}
