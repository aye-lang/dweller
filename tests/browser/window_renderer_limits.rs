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

fn find_state(state: &str) -> (String, String) {
    let of = |head: &str| state.lines().find(|l| l.starts_with(head)).unwrap_or("").to_string();
    (of("matches "), of("renderer-find "))
}

/// Q2.15j: find in frames. The find bar's query goes to the renderer of
/// every frame of the tab: the count is the sum, the active match starts
/// in the page, Enter walks through the page's matches and then the
/// frame's and wraps, Shift+Enter walks back. A page without a match
/// starts in the frame.
#[test]
fn window_renderer_limits_find_in_frames() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/ui/oopif-find.html", srv.base);
    let find = "renderer:loaded renderer:child key:cmd+f text:needle renderer:find frame";
    let next = "key:return renderer:find frame";
    let previous = "key:shift+return renderer:find frame";
    let cases = [
        (find.to_string(), "renderer-find 3 active 0"),
        (format!("{find} {next}"), "renderer-find 3 active 1"),
        (format!("{find} {next} {next}"), "renderer-find 3 active 2"),
        (format!("{find} {next} {next} {next}"), "renderer-find 3 active 0"),
        (format!("{find} {previous}"), "renderer-find 3 active 2"),
        (format!("{find} {next} {previous}"), "renderer-find 3 active 0"),
    ];
    for (events, want) in &cases {
        let state = limits_run(&page, events, "2000", &[]);
        assert_eq!(find_state(&state), ("matches 3".to_string(), want.to_string()), "{events}\n{state}");
    }

    let state = limits_run(&format!("{page}?only"), &format!("{find} frame frame frame renderer:find frame"), "2000", &[]);
    assert_eq!(find_state(&state), ("matches 2".to_string(), "renderer-find 2 active 0".to_string()), "{state}");

    let state = limits_run(&format!("{page}?only"), &format!("{find} frame frame frame renderer:find {next}"), "2000", &[]);
    assert_eq!(find_state(&state).1, "renderer-find 2 active 1", "{state}");

    let state = limits_run(&page, &format!("{find} key:escape renderer:find frame"), "2000", &[]);
    assert_eq!(find_state(&state).0, "matches 0", "{state}");
}

/// Q2.15k: the clipboard outside text fields. With no field in focus,
/// Cmd+A selects the document's text and Cmd+C copies the selection (the
/// page gets a `copy` event); pasted into a field it is the page's text.
/// The same in the window's own process and in a renderer.
#[test]
fn window_renderer_limits_page_copy() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    for mode in ["", "tab", "site"] {
        let (loaded, input) = if mode.is_empty() { ("loaded frame", "frame frame") } else { ("renderer:loaded frame", "renderer:input frame") };
        let events = format!("{loaded} key:cmd+a {input} key:cmd+c {input} frame frame click:50,55 release:50,55 {input} key:cmd+v {input} frame");
        let mut command = dweller_command();
        if mode.is_empty() {
            command.env("DWELLER_PROCESS", "single");
        } else {
            command.env("DWELLER_PROCESS", mode);
        }
        let out = command
            .current_dir(root())
            .args(["browser/tests/ui/page-copy.html", "--font", "fixed", "--width", "400", "--height", "300"])
            .args(["--frames", "600", "--state", "--processes", "--text", "--events", &events])
            .output()
            .unwrap();
        let state = text(&out.stdout);
        assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {}\n{}", out.status, text(&out.stderr));
        assert!(state.contains("Copy copy"), "{mode}: the page got a copy event\n{state}");
        assert!(state.contains("pasted alpha beta") || state.contains("\nvalue alpha beta\n"), "{mode}: the page's text was copied and pasted\n{state}");
    }
}

/// Q2.15l: `IntersectionObserverEntry.rootBounds` of an observer with the
/// implicit root is null when the target's document is cross-origin with
/// the top document (a cross-site frame, in its own renderer or not), and
/// the viewport grown by `rootMargin` for the top document's own target.
#[test]
fn window_renderer_limits_root_bounds() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/ui/oopif-root.html", srv.base);
    for mode in ["", "site"] {
        let mut command = dweller_command();
        let wait = if mode.is_empty() {
            command.env("DWELLER_PROCESS", "single");
            "loaded ".to_string() + &"frame ".repeat(60)
        } else {
            command.env("DWELLER_PROCESS", mode);
            "renderer:loaded renderer:child renderer:title:true frame".to_string()
        };
        let out = command
            .current_dir(root())
            .args([&page, "--font", "fixed", "--width", "400", "--height", "400"])
            .args(["--frames", "1500", "--state", "--processes", "--text", "--events", &wait])
            .output()
            .unwrap();
        let state = text(&out.stdout);
        assert!(out.status.success() && out.stderr.is_empty(), "{mode}: {}\n{}", out.status, text(&out.stderr));
        let title = state.lines().find(|l| l.contains("P own ")).unwrap_or("");
        assert!(title.contains("P own rect ") && title.ends_with(" frame null true"), "{mode}:\n{state}");
    }
}

fn child_place(state: &str) -> (i64, String) {
    let line = state.lines().find(|l| l.starts_with("renderer-child ")).unwrap_or_else(|| panic!("no child in\n{state}"));
    let words: Vec<&str> = line.split(' ').collect();
    let at = words.iter().position(|w| *w == "at").unwrap();
    (words[at + 2].parse().unwrap(), line.split_once(" title ").map(|(_, t)| t.to_string()).unwrap_or_default())
}

/// Q2.15p: `overscroll-behavior` at a frame's boundary. A wheel over a
/// cross-site frame scrolls the frame, and what the frame cannot use
/// scrolls the page around it; with `overscroll-behavior: contain` on
/// the frame's root element the page stays where it is.
#[test]
fn window_renderer_limits_overscroll() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let wheels = "renderer:loaded renderer:child ".to_string() + &vec!["move:172,136 wheel:-3 renderer:input"; 3].join(" ") + " frame";

    let state = limits_run(&format!("{}/ui/oopif-input.html", srv.base), &wheels, "2000", &[]);
    let (y, title) = child_place(&state);
    assert!(y < 36 && title.ends_with("scroll 220"), "the rest of the wheel scrolls the page\n{state}");

    let state = limits_run(&format!("{}/ui/oopif-input.html?contain", srv.base), &wheels, "2000", &[]);
    let (y, title) = child_place(&state);
    assert!(y == 36 && title.ends_with("scroll 220"), "overscroll-behavior: contain keeps the page still\n{state}");
}
