use super::*;

fn kept_list_run(base: &str, mode: Option<&str>, png: &Path, events: &str) -> usize {
    let mut window = dweller_command();
    window.env("DWELLER_PROCESS", mode.unwrap_or(IN_PROCESS.1));
    let out = window
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .arg(format!("{base}/ui/kept_list.html"))
        .args(["--font", "fixed", "--width", "400", "--height", "300"])
        .args(["--frames", "2000", "--until-idle", "--save"])
        .arg(png)
        .args(["--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    text(&out.stderr).lines().filter(|l| l.starts_with("info: DWELLER_TIMING\tpaint_build\t")).count()
}

/// Q2.16c: with `DWELLER_PROCESS=tab` the renderer keeps its display list
/// over frames that cannot change it: ten canvas draws and six wheel
/// scrolls of `browser/tests/ui/kept_list.html` build it at most 4 times
/// (before: once per frame, over 16), and the window's pixels after the
/// scroll equal those of the window that runs the page in its own process.
#[test]
fn window_renderer_list_kept() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let dir = scratch();
    let own = dir.join("kept-list-own.png");
    let tab = dir.join("kept-list-tab.png");
    let wheels = "move:200,150 wheel:-3 frame wheel:-3 frame wheel:-3 frame wheel:-3 frame wheel:-3 frame wheel:-3 frame";
    kept_list_run(&srv.base, None, &own, wheels);
    let waited = wheels.replace("frame", "renderer:input frame");
    let builds = kept_list_run(&srv.base, Some("tab"), &tab, &format!("renderer:title:drawn {waited}"));
    assert!(builds >= 1 && builds <= 4, "{builds} display list builds");
    assert!(std::fs::read(&tab).unwrap() == std::fs::read(&own).unwrap(), "{} differs from {}", tab.display(), own.display());
    println!("{builds} builds");
}
