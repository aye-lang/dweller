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
/// the same wheel), the strip that came into view too: the renderer paints
/// rows above and below its viewport (slice 2b). The window's own status
/// bubble (20 px at the bottom left: "The page is busy" once the script
/// has run 5 s, which a slow machine reaches before the window ends) is
/// chrome, not the page, and is left out.
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
    let status = state.lines().any(|l| l.starts_with("status ") && l.len() > "status ".len());
    let bubble = |x: usize, y: usize| status && y >= h - 20 && x * 10 < w * 7;
    let rows: Vec<usize> = (0..h).filter(|&y| (0..w).any(|x| !bubble(x, y) && a[(y * w + x) * 4..(y * w + x + 1) * 4] != b[(y * w + x) * 4..(y * w + x + 1) * 4])).collect();
    assert!(rows.is_empty(), "the scrolled window differs from the in-process one in {} rows, {:?} to {:?}: {} and {}\n{}\n{}", rows.len(), rows.first(), rows.last(), tab.display(), own.display(), state, err.lines().filter(|l| l.contains("\tui_pan\t") || l.contains("\tpaint\t") || l.contains("\tinput_to_")).collect::<Vec<_>>().join("\n"));
}

/// Q4.25e slice 3: the window shifts the page ahead for a wheel outside
/// the regions the renderer said must see the wheel first, and not for
/// one over an inner scroller or over an element whose wheel listener may
/// cancel it (`browser/tests/ui/pan_regions.html`, while a script of 20 s
/// runs); a wheel the page handled there leaves the page's shifting on.
#[test]
fn window_renderer_pan_regions() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let png = scratch().join("pan-regions.png");
    let run = |query: &str, at: &str| {
        let events = format!("renderer:loaded click:100,100 release:100,100 renderer:title:busy move:{at} wheel:-3 frame frame");
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", "tab")
            .env("DWELLER_TIMING", "1")
            .arg(format!("{}/ui/pan_regions.html{query}", srv.base))
            .args(["--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--save"])
            .arg(&png)
            .args(["--events", &events])
            .output()
            .unwrap();
        assert!(out.status.success(), "{at}: {}\n{}", out.status, text(&out.stderr));
        text(&out.stderr).contains("DWELLER_TIMING\tui_pan\t")
    };
    assert!(run("?20000", "100,150"), "a wheel outside the regions is shifted ahead");
    assert!(!run("?20000", "300,80"), "a wheel over the inner scroller waits for the renderer");
    assert!(!run("?20000", "300,220"), "a wheel over a listener that may cancel it waits for the renderer");
    let events = "renderer:loaded move:300,80 wheel:-1 renderer:input click:100,100 release:100,100 renderer:title:busy move:100,150 wheel:-3 frame frame";
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "tab")
        .env("DWELLER_TIMING", "1")
        .arg(format!("{}/ui/pan_regions.html?20000", srv.base))
        .args(["--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--save"])
        .arg(&png)
        .args(["--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    assert!(text(&out.stderr).contains("DWELLER_TIMING\tui_pan\t"), "the inner scroller's wheel turned the shifting off");
}
