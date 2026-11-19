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
/// (#121212) and an unchecked check box is filled with the dark Field
/// (#3b3b3b); with the light preference both are white.
#[test]
fn color_scheme_canvas_and_checkbox() {
    for (scheme, canvas, field) in [("light", [255u8, 255, 255], [255u8, 255, 255]), ("dark", [18, 18, 18], [59, 59, 59])] {
        let (w, _, px) = shot_scheme(scheme);
        let at = |x: usize, y: usize| px[(y * w + x) * 4..(y * w + x) * 4 + 3].to_vec();
        assert_eq!(at(2, 2), canvas, "{scheme}: the canvas");
        assert_eq!(at(26, 26), field, "{scheme}: inside the check box");
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
