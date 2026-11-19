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
