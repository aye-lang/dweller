use super::*;

/// Q2.10: with `DWELLER_PROCESS=tab` a script's `scrollTo` scrolls the
/// page in the renderer, during the load (where the history's restored
/// offset would otherwise win) and from a key handler: the PNG is the
/// in-process window's.
#[test]
fn window_renderer_scroll_to() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = "browser/tests/ui/renderer_scroll_to.html";
    let run = |png: &Path, events: &str, mode: &str| {
        let mut command = dweller_command();
        if !mode.is_empty() {
            command.env("DWELLER_PROCESS", mode);
        }
        let out = command
            .current_dir(root())
            .args([page, "--font", "fixed", "--width", "300", "--height", "300"])
            .args(["--frames", "1000", "--until-idle", "--save"])
            .arg(png)
            .args(["--events", events])
            .output()
            .unwrap();
        assert!(out.status.success() && out.stderr.is_empty(), "{events}: {}\n{}", out.status, text(&out.stderr));
    };
    let golden = scratch().join("scroll_to_in_process.png");
    let shot = scratch().join("scroll_to_tab.png");
    for (in_process, tab) in [("loaded frame", "renderer:loaded renderer:frame"), ("loaded key:s frame", "renderer:loaded renderer:frame key:s renderer:input")] {
        run(&golden, in_process, "");
        run(&shot, tab, "tab");
        assert!(
            std::fs::read(&shot).unwrap() == std::fs::read(&golden).unwrap(),
            "{tab}: the renderer's frame {} differs from the in-process window {}",
            shot.display(),
            golden.display()
        );
    }
}
