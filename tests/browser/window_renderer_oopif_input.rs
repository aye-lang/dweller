use super::*;

/// Runs `browser/tests/ui/oopif-input.html` (a `127.0.0.1` page with a
/// `localhost` iframe whose content box is at window (22, 76), 200x100)
/// with `DWELLER_PROCESS=site` and `events`; gives the `--state` text.
fn run_input(base: &str, query: &str, events: &str, envs: &[(&str, &str)]) -> String {
    let out = dweller_command()
        .env("DWELLER_PROCESS", "site")
        .current_dir(root())
        .envs(envs.iter().copied())
        .arg(format!("{base}/ui/oopif-input.html{query}"))
        .args(["--font", "fixed", "--width", "400", "--height", "400"])
        .args(["--frames", "1000", "--until-idle", "--state", "--save"])
        .arg(scratch().join("oopif_input.png"))
        .args(["--events", &format!("renderer:loaded renderer:child {events}")])
        .output()
        .unwrap();
    assert!(out.status.success(), "{query} {events}: {}\n{}", out.status, text(&out.stderr));
    if envs.is_empty() {
        assert!(out.stderr.is_empty(), "{query} {events}: {}", text(&out.stderr));
    }
    text(&out.stdout) + &text(&out.stderr)
}

fn parent_title(state: &str) -> &str {
    let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or("");
    line.split(" title ").nth(1).unwrap_or("")
}

fn child_line(state: &str) -> &str {
    state.lines().find(|l| l.starts_with("renderer-child ")).unwrap_or("")
}

fn child_title(state: &str) -> &str {
    child_line(state).split(" title ").nth(1).unwrap_or("")
}

fn child_y(state: &str) -> i64 {
    let at = child_line(state).split(" at ").nth(1).unwrap_or("");
    at.split(' ').nth(1).and_then(|y| y.parse().ok()).unwrap_or(i64::MIN)
}

fn skip() -> Option<Server> {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return None;
    }
    let srv = server();
    if srv.is_none() {
        println!("skipped: cannot start python3 for the test server");
    }
    srv
}

/// Q2.10: the browser hit-tests the child's rect, so a click over the
/// cross-site iframe reaches the child's listener at child-local
/// coordinates; a drag started in the child keeps going to it (capture)
/// even outside the frame, and the parent sees none of it.
#[test]
fn window_renderer_oopif_input_mouse() {
    let Some(srv) = skip() else { return };
    let state = run_input(&srv.base, "", "click:172,136 release:172,136 renderer:input click:172,136 move:300,300 release:300,300 renderer:input", &[]);
    assert_eq!(child_title(&state), "C down 150,60,click,up 150,60,down 150,60,click,up 278,224", "{state}");
    assert_eq!(parent_title(&state), "P", "{state}");

    let state = run_input(&srv.base, "", "click:300,300 release:300,300 renderer:input", &[]);
    assert_eq!(parent_title(&state), "P up", "{state}");
    assert_eq!(child_title(&state), "C", "{state}");
}

/// Q2.10: keys and text go to the focused frame: after a click into the
/// child's input, typed text and keys reach the child and not the parent;
/// a parent's `iframe.focus()` (on the key `f`) moves the keyboard focus into the child.
#[test]
fn window_renderer_oopif_input_keys() {
    let Some(srv) = skip() else { return };
    let state = run_input(&srv.base, "", "click:32,86 release:32,86 renderer:input text:hi key:x renderer:input", &[]);
    assert_eq!(child_title(&state), "C down 10,10,focus,click,up 10,10,value hi,key x", "{state}");
    assert_eq!(parent_title(&state), "P", "{state}");

    let state = run_input(&srv.base, "", "key:f renderer:input key:x renderer:input", &[]);
    assert_eq!(parent_title(&state), "P key f", "{state}");
    assert_eq!(child_title(&state), "C key x", "{state}");
}

/// Q2.10: Tab crosses the process boundary both ways: from the parent's
/// first input into the child's input, out of the child to the parent's
/// next input, and back with Shift+Tab.
#[test]
fn window_renderer_oopif_input_tab() {
    let Some(srv) = skip() else { return };
    let events = "click:10,52 release:10,52 renderer:input key:tab renderer:input key:tab renderer:input key:shift+tab renderer:input key:shift+tab renderer:input";
    let state = run_input(&srv.base, "", events, &[]);
    assert_eq!(parent_title(&state), "P a,up,key Tab,b,key Tab,a", "{state}");
    assert_eq!(child_title(&state), "C focus,blur,focus,blur", "{state}");
}

/// Q2.10: a wheel over the child scrolls the child; what the child cannot
/// use at its end bubbles to the parent, which scrolls (the child's rect
/// moves up by the rest); the parent gets no wheel event for it.
#[test]
fn window_renderer_oopif_input_wheel() {
    let Some(srv) = skip() else { return };
    let one = "move:172,136 wheel:-3 renderer:input";
    let cases = [(1, 36, "C scroll 120"), (2, 16, "scroll 220"), (3, -104, "scroll 220")];
    for (n, y, title) in cases {
        let events = vec![one; n].join(" ");
        let state = run_input(&srv.base, "", &events, &[]);
        assert_eq!(child_y(&state), y, "{n} wheels\n{state}");
        assert!(child_title(&state).ends_with(title), "{n} wheels\n{state}");
    }

    let state = run_input(&srv.base, "", "move:300,300 wheel:-3 renderer:input", &[]);
    assert_eq!(child_y(&state), -84, "a wheel over the parent scrolls the parent\n{state}");
    assert_eq!(child_title(&state), "C", "{state}");
}

/// Q2.10: the child learns its visible rect in the top's viewport, so its
/// IntersectionObserver fires when the parent scrolls it into view.
#[test]
fn window_renderer_oopif_input_intersection() {
    let Some(srv) = skip() else { return };
    let state = run_input(&srv.base, "?io", "renderer:child-title:io_false", &[]);
    assert_eq!(child_title(&state), "C io false", "{state}");
    let state = run_input(&srv.base, "?io", "move:300,300 wheel:-5 renderer:input renderer:child-title:io_true", &[]);
    assert_eq!(child_title(&state), "C io false,io true", "{state}");
}

/// Q2.10: the browser checks the new messages: a child that says it did
/// not use more scroll than it was sent, or a renderer that moves the focus
/// into a frame it does not hold, is killed.
#[test]
fn window_renderer_oopif_input_checks() {
    let Some(srv) = skip() else { return };
    let evil = [("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "wheel")];
    let state = run_input(&srv.base, "", "move:172,136 wheel:-1 renderer:child-gone", &evil);
    assert!(child_line(&state).ends_with(" gone bad wheel_unused"), "{state}");
    assert!(state.contains("error: bad message: bad wheel_unused "), "{state}");
    assert_eq!(parent_title(&state), "P", "{state}");

    let evil = [("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "focus-frame")];
    let state = run_input(&srv.base, "", "renderer:gone", &evil);
    let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or("");
    assert!(line.ends_with(" gone bad focus_frame"), "{state}");
    assert!(state.contains("error: bad message: bad focus_frame "), "{state}");
}
