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
