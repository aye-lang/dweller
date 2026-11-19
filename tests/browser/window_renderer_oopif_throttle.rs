use super::*;

fn throttle_ticks(base: &str, query: &str) -> u64 {
    let out = dweller_command()
        .env("DWELLER_PROCESS", "site")
        .current_dir(root())
        .arg(format!("{base}/ui/oopif-throttle.html{query}"))
        .args(["--font", "fixed", "--width", "400", "--height", "300", "--frames", "300", "--state", "--processes"])
        .args(["--events", "renderer:loaded renderer:child"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{query}: {}\n{}", out.status, text(&out.stderr));
    let state = text(&out.stdout);
    let line = state.lines().find(|l| l.starts_with("renderer-child ")).unwrap_or_else(|| panic!("no child:\n{state}"));
    line.rsplit(" title T ").next().and_then(|n| n.trim().parse().ok()).unwrap_or_else(|| panic!("no count: {line}"))
}

/// Q2.16d: with `DWELLER_PROCESS=site` a child frame's renderer of which
/// nothing is in view is throttled as a background tab's is (timers at
/// most once a second): over the same 300 window frames the 20 ms timer
/// of `browser/tests/ui/oopif-throttle-child.html` counts at least 40 in
/// view and at most 8 when the frame is 2000 px below the viewport.
#[test]
fn window_renderer_oopif_throttle_hidden() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let seen = throttle_ticks(&srv.base, "");
    let far = throttle_ticks(&srv.base, "?far");
    assert!(seen >= 40, "{seen} ticks in view");
    assert!(far <= 8, "{far} ticks out of view ({seen} in view)");
    println!("{seen} ticks in view, {far} out of view");
}
