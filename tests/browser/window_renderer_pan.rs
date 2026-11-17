use super::*;

fn pan_run(base: &str, query: &str, mode: &str, png: &Path, events: &str) -> (String, String) {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", mode)
        .env("DWELLER_TIMING", "1")
        .arg(format!("{base}/ui/pan_busy.html{query}"))
        .args(["--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--state", "--processes", "--save"])
        .arg(png)
        .args(["--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    (text(&out.stdout), text(&out.stderr))
}

/// Q4.25e: with `DWELLER_PROCESS=tab` the window scrolls the renderer's
/// last frame at once while the renderer is busy: a click starts a script
/// of 20 s in `browser/tests/ui/pan_busy.html`, the wheel that follows is
/// not acked before the window ends, and the window's pixels are already
/// those of the page scrolled by the wheel (the in-process window after
/// the same wheel) except the strip the old frame does not hold.
#[test]
fn window_renderer_pan_while_busy() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let dir = scratch();
    let (own, tab) = (dir.join("pan-own.png"), dir.join("pan-tab.png"));
    pan_run(&srv.base, "?10", IN_PROCESS.1, &own, "wheel:-3 frame frame");
    let events = "renderer:loaded click:200,100 release:200,100 renderer:title:busy wheel:-3 frame frame";
    let (state, err) = pan_run(&srv.base, "?20000", "tab", &tab, events);
    assert!(state.contains("title Pan busy"), "the script ended before the window:\n{state}");
    assert!(err.contains("DWELLER_TIMING\tui_pan\t"), "the window never drew a shifted frame");
    assert!(!err.lines().any(|l| l.contains("\tinput_to_frame\t") && l.ends_with("wheel")), "the wheel was acked");
    assert!(err.contains("DWELLER_TIMING\tinput_to_pan\t"), "no input_to_pan record");
    let (w, h, a) = png_rgba(&std::fs::read(&tab).unwrap());
    let (w2, h2, b) = png_rgba(&std::fs::read(&own).unwrap());
    assert_eq!((w, h), (w2, h2));
    let rows = h - 120;
    assert!(a[..rows * w * 4] == b[..rows * w * 4], "the rows above the strip differ: {} and {}", tab.display(), own.display());
    assert!(a != b, "nothing was left blank: the renderer's frame arrived");
}
