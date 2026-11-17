use super::*;

fn limits_run(url: &str, events: &str, frames: &str, envs: &[(&str, &str)]) -> String {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "site")
        .envs(envs.iter().copied())
        .arg(url)
        .args(["--font", "fixed", "--width", "400", "--height", "400"])
        .args(["--frames", frames, "--state", "--processes", "--events", events])
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{events}: {}\n{}", out.status, text(&out.stderr));
    text(&out.stdout)
}

fn tab_pids(state: &str) -> Vec<String> {
    let line = state.lines().find(|l| l.starts_with("renderer-tabs ")).unwrap_or("");
    line.split(' ').skip(1).map(String::from).collect()
}

/// Q2.13: the cap on renderer processes. Under the cap every tab has a
/// renderer of its own. At the cap (`DWELLER_MAX_RENDERERS=1`) a new tab
/// of a site that has a renderer joins it, and a tab of another site
/// still gets a new one (the cap is soft, as Chromium's).
#[test]
fn window_renderer_limits_cap() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let first = format!("{}/ui/oopif-child.html", srv.base);
    let same = format!("{}/ui/oopif-child.html?two", srv.base);
    let other = format!("{}/ui/oopif-child.html?two", srv.base.replace("127.0.0.1", "localhost"));
    let open = |url: &str| format!("renderer:loaded key:cmd+t {} renderer:loaded renderer:title:?two frame", go_to(url));
    let capped = [("DWELLER_MAX_RENDERERS", "1")];

    let state = limits_run(&first, &open(&same), "1000", &capped);
    let pids = tab_pids(&state);
    assert!(pids.len() == 2 && pids[0] == pids[1] && pids[0] != "0", "at the cap a same-site tab joins the site's renderer\n{state}");
    assert!(state.contains(" title Child 127.0.0.1?two\n"), "{state}");

    let state = limits_run(&first, &open(&other), "1000", &capped);
    let pids = tab_pids(&state);
    assert!(pids.len() == 2 && pids[0] != pids[1], "another site gets a renderer over the cap\n{state}");
    assert!(state.contains(" title Child localhost?two\n"), "{state}");

    let state = limits_run(&first, &open(&same), "1000", &[]);
    let pids = tab_pids(&state);
    assert!(pids.len() == 2 && pids[0] != pids[1], "under the cap each tab has its own renderer\n{state}");
}

fn throttle_counts(state: &str) -> (u64, u64) {
    let line = state.lines().find(|l| l.starts_with("renderer-child ")).unwrap_or_else(|| panic!("no child in\n{state}"));
    let words: Vec<&str> = line.split(' ').collect();
    let after = |w: &str| words.iter().position(|x| *x == w).and_then(|i| words.get(i + 1)).and_then(|n| n.parse().ok()).unwrap_or_else(|| panic!("no {w} in {line}"));
    (after("raf"), after("ticks"))
}

/// Q2.13: a cross-site frame that is out of its embedder's view is
/// throttled in its own renderer: no animation frame runs and its 50 ms
/// interval runs about once a second; once the parent scrolls it into
/// view both run as usual.
#[test]
fn window_renderer_limits_hidden_frame() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/ui/oopif-throttle.html", srv.base);
    let frames = "frame ".repeat(300);

    let state = limits_run(&page, &format!("renderer:loaded {frames}"), "2000", &[]);
    let (raf, ticks) = throttle_counts(&state);
    assert_eq!(raf, 0, "no animation frame out of view\n{state}");

    let state = limits_run(&page, &format!("renderer:loaded {frames}key:end renderer:input {frames}"), "2000", &[]);
    let (shown_raf, shown_ticks) = throttle_counts(&state);
    assert!(shown_raf >= 5, "animation frames run in view\n{state}");
    assert!(shown_ticks >= ticks + 10 && shown_ticks >= 3 * ticks.max(1), "the interval runs unthrottled in view: {ticks} hidden, {shown_ticks} after\n{state}");
}
