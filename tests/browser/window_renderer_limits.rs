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

/// Q2.15h: a site whose renderer crashed three times within 30 s is
/// told apart: the sad tab says "This page keeps crashing" where the
/// first and second crash say "This page crashed". Reload stays the
/// person's choice and still works.
#[test]
fn window_renderer_limits_crashing() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let crash = "renderer:kill renderer:gone frame";
    let reload = "key:cmd+r frame renderer:loaded";
    let page = "browser/tests/ui/form.html";

    let state = limits_run(page, &format!("renderer:loaded {crash} {reload} {crash}"), "600", &[]);
    assert!(state.contains("\nsad-tab This page crashed\n"), "two crashes\n{state}");

    let state = limits_run(page, &format!("renderer:loaded {crash} {reload} {crash} {reload} {crash}"), "600", &[]);
    assert!(state.contains("\nsad-tab This page keeps crashing\n"), "three crashes\n{state}");

    let state = limits_run(page, &format!("renderer:loaded {crash} {reload} {crash} {reload} {crash} {reload} frame"), "600", &[]);
    assert!(state.contains(" title Form\n") && !state.contains("sad-tab"), "Reload still loads the page\n{state}");
}

fn history_state(state: &str) -> (String, String, String) {
    let of = |head: &str| state.lines().find(|l| l.starts_with(head)).unwrap_or("").to_string();
    let title = |l: String| l.split_once(" title ").map(|(_, t)| t.to_string()).unwrap_or_default();
    (of("history "), title(of("renderer ")), title(of("renderer-child ")))
}

/// Q2.15i: a cross-site frame's own navigation is an entry of the tab's
/// history. A click on a link in the frame gives `history 2 of 2`; Back
/// navigates the frame back and does not reload the page around it;
/// Forward navigates the frame again.
#[test]
fn window_renderer_limits_frame_history() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/ui/oopif-history.html", srv.base);
    let follow = "renderer:loaded renderer:child click:50,60 release:50,60 renderer:input renderer:child-title:H?second frame";
    let back = "key:cmd+[ renderer:child-title:H frame";
    let forward = "key:cmd+] renderer:child-title:H?second frame";

    let state = limits_run(&page, follow, "2000", &[]);
    assert_eq!(history_state(&state), ("history 2 of 2".into(), "P loads 1".into(), "H?second".into()), "{state}");

    let state = limits_run(&page, &format!("{follow} {back}"), "2000", &[]);
    assert_eq!(history_state(&state), ("history 1 of 2".into(), "P loads 1".into(), "H".into()), "{state}");

    let state = limits_run(&page, &format!("{follow} {back} {forward}"), "2000", &[]);
    assert_eq!(history_state(&state), ("history 2 of 2".into(), "P loads 1".into(), "H?second".into()), "{state}");
}
