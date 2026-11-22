use super::*;

fn dump_scheme(scheme: &str) -> String {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_COLOR_SCHEME", scheme)
        .args(["dump", "--text", "--js", "browser/tests/color-scheme/color_scheme.html"])
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{}\n{}", out.status, text(&out.stderr));
    text(&out.stdout)
}

/// Q5.2: `browser/tests/color-scheme/color_scheme.html` reports what its script
/// sees of the color scheme: `prefers-color-scheme` by the process's
/// preference (`DWELLER_COLOR_SCHEME`), `light-dark()`, the system
/// colors, the links and the controls by each element's used scheme
/// (`color-scheme`, the page's `<meta name="color-scheme">`). The light
/// run prints `color_scheme.txt`, the dark one `color_scheme_dark.txt`;
/// the script's report of the dark one also with `DWELLER_PROCESS=site`,
/// where the renderer takes the preference from its environment.
#[test]
fn color_scheme_preference() {
    let mut failures = Vec::new();
    for (scheme, golden) in [("light", "color_scheme.txt"), ("dark", "color_scheme_dark.txt")] {
        check_golden(
            &root().join("browser/tests/color-scheme").join(golden),
            &dump_scheme(scheme),
            "color_scheme_preference",
            &mut failures,
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let want = std::fs::read_to_string(root().join("browser/tests/color-scheme/color_scheme_dark.txt")).unwrap();
    let href = format!("file://{}", root().join("browser/tests/color-scheme/color_scheme.html").display());
    let site = login_window_with(
        &href,
        "frame",
        None,
        &[("DWELLER_PROCESS", "site"), ("DWELLER_COLOR_SCHEME", "dark")],
    );
    let report = &want[want.find("prefers dark").unwrap()..];
    assert!(site.contains(report.trim()), "{site}");
}

fn shot_scheme(scheme: &str) -> (usize, usize, Vec<u8>) {
    let png = scratch().join(format!("color_scheme_{scheme}.png"));
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_COLOR_SCHEME", scheme)
        .args(["screenshot", "browser/tests/color-scheme/color_scheme_canvas.html", "--font", "fixed", "--width", "100", "-o"])
        .arg(&png)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    png_rgba(&std::fs::read(&png).unwrap())
}

/// Q5.2: with a dark used scheme the canvas is Chromium's dark Canvas
/// (#121212), and an unchecked check box, an empty `progress` and the
/// empty part of a range's track are the dark Field (#3b3b3b); with the
/// light preference the first three are white and the track #c4c4c4.
#[test]
fn color_scheme_canvas_and_checkbox() {
    for (scheme, canvas, field, track) in [("light", [255u8, 255, 255], [255u8, 255, 255], [196u8, 196, 196]), ("dark", [18, 18, 18], [59, 59, 59], [59, 59, 59])] {
        let (w, _, px) = shot_scheme(scheme);
        let at = |x: usize, y: usize| px[(y * w + x) * 4..(y * w + x) * 4 + 3].to_vec();
        assert_eq!(at(2, 2), canvas, "{scheme}: the canvas");
        assert_eq!(at(26, 26), field, "{scheme}: inside the check box");
        assert_eq!(at(70, 26), field, "{scheme}: inside the progress bar");
        assert_eq!(at(80, 59), track, "{scheme}: the range's track");
    }
}

/// Q5.2, slice 5: `prefers-color-scheme` inside a frame is the used
/// color scheme of its `iframe` element, already when the frame's load
/// scripts run (`browser/tests/net/color-scheme/frame.html`): an element
/// with `color-scheme: dark` gives a dark frame whatever the person
/// prefers; `light`, or an embedder that did not opt in, gives a light
/// one. Each frame reports its media query, a rule under it and its
/// root's color.
#[test]
fn color_scheme_frames_follow_their_element() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let want = "dark: prefers dark true rule rgb(0, 0, 2) root rgb(255, 255, 255)\n\
                light: prefers dark false rule rgb(0, 0, 1) root rgb(0, 0, 0)\n\
                normal: prefers dark false rule rgb(0, 0, 1) root rgb(0, 0, 0)";
    for scheme in ["light", "dark"] {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_COLOR_SCHEME", scheme)
            .args(["dump", "--text", "--js", &format!("{}/net/color-scheme/frame.html", srv.base)])
            .output()
            .unwrap();
        assert!(out.status.success() && out.stderr.is_empty(), "{}\n{}", out.status, text(&out.stderr));
        let got = text(&out.stdout);
        assert!(got.contains(want), "{scheme}: {got}");
    }
}

/// Q5.2, slice 5: a frame's canvas
/// (`browser/tests/color-scheme/frame_canvas.html`, four frames on a
/// green page). A frame without a background is transparent when its
/// used color scheme is its `iframe` element's; a dark frame in a light
/// element gets the dark canvas, and a light frame in an element with
/// `color-scheme: dark` a white one; a frame's own translucent
/// background is blended over the page.
#[test]
fn color_scheme_frame_canvas() {
    let png = scratch().join("color_scheme_frame_canvas.png");
    let out = dweller_command()
        .current_dir(root())
        .args(["screenshot", "browser/tests/color-scheme/frame_canvas.html", "--font", "fixed", "--width", "300", "-o"])
        .arg(&png)
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let (w, _, px) = png_rgba(&std::fs::read(&png).unwrap());
    let at = |x: usize| px[(25 * w + x) * 4..(25 * w + x) * 4 + 3].to_vec();
    assert_eq!(at(25), [0, 128, 0], "the same scheme: transparent");
    assert_eq!(at(85), [18, 18, 18], "a dark frame in a light element");
    assert_eq!(at(145), [0, 64, 128], "a translucent background over the page");
    assert_eq!(at(205), [255, 255, 255], "a light frame in a dark element");
    assert_eq!(at(270), [0, 128, 0], "the page");
}

/// Q5.2, slice 5: a frame's `matchMedia('(prefers-color-scheme: dark)')`
/// fires `change` (to its `onchange` and its listeners, a
/// `MediaQueryListEvent`) when its `iframe` element's used scheme turns
/// dark (`browser/tests/net/color-scheme/frame_change.html`), in one
/// process and with `DWELLER_PROCESS=site`.
#[test]
fn color_scheme_frame_media_change() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    for mode in ["single", "site"] {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", mode)
            .args([&format!("{}/net/color-scheme/frame_change.html", srv.base), "--font", "fixed", "--width", "400", "--height", "200"])
            .args(["--frames", "400", "--until-idle", "--text", "--events", "renderer:title:done frame"])
            .output()
            .unwrap();
        assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {}\n{}", out.status, text(&out.stderr));
        let got = text(&out.stdout);
        let want = "frame: start false; onchange true; change true (prefers-color-scheme: dark) true;";
        assert!(got.contains(want), "{mode}: {got}");
    }
}

/// Q5.2, slice 4: the caret of a text field is the field's `color`
/// (`caret-color: auto`): light in a dark field, black in a light one
/// (`browser/tests/color-scheme/caret.html`, focused by a click), in one process and with
/// `DWELLER_PROCESS=site`.
#[test]
fn color_scheme_caret() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    for mode in ["single", "site"] {
        for (scheme, light) in [("light", false), ("dark", true)] {
            let png = scratch().join(format!("color_scheme_caret_{mode}_{scheme}.png"));
            let out = dweller_command()
                .current_dir(root())
                .env("DWELLER_PROCESS", mode)
                .env("DWELLER_COLOR_SCHEME", scheme)
                .env("DWELLER_STEADY_CARET", "1")
                .args(["browser/tests/color-scheme/caret.html", "--font", "fixed", "--width", "120", "--height", "120"])
                .args(["--frames", "200", "--until-idle", "--save"])
                .arg(&png)
                .args(["--events", "painted click:50,60 frame frame"])
                .output()
                .unwrap();
            assert!(out.status.success(), "{mode} {scheme}: {}\n{}", out.status, text(&out.stderr));
            let (w, h, px) = png_rgba(&std::fs::read(&png).unwrap());
            let field = if light { 59u8 } else { 255 };
            let caret = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|&(x, y)| {
                let p = &px[(y * w + x) * 4..(y * w + x) * 4 + 3];
                let row_field = (0..w).filter(|&k| px[(y * w + k) * 4] == field && px[(y * w + k) * 4 + 1] == field).count() > 40;
                row_field && if light { p.iter().all(|&c| c > 200) } else { p.iter().all(|&c| c < 50) }
            }).count();
            assert!(caret >= 8, "{mode} {scheme}: {caret} caret pixels");
        }
    }
}
