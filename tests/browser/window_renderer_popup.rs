use super::*;

const OPEN_SAME: &str = "renderer:loaded click:50,62 renderer:input renderer:title:?same";

fn popup_window(base: &str, events: &str) -> String {
    popup_window_in("tab", base, events)
}

fn popup_window_in(mode: &str, base: &str, events: &str) -> String {
    let page = format!("{base}/ui/popup_opener.html");
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", mode)
        .args([page.as_str(), "--font", "fixed", "--width", "400", "--height", "400"])
        .args(["--frames", "1000", "--until-idle", "--state", "--processes", "--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{mode} {events}: {}\n{}", out.status, text(&out.stderr));
    assert!(out.stderr.is_empty(), "{mode} {events}: {}", text(&out.stderr));
    text(&out.stdout)
}

fn tab_pids(state: &str) -> Vec<String> {
    let line = state.lines().find(|l| l.starts_with("renderer-tabs ")).unwrap_or("");
    line.split(' ').skip(1).map(String::from).collect()
}

/// Q2.12d: with `DWELLER_PROCESS=tab`, a `window.open` popup that keeps
/// its opener and is same-site with it runs in its opener's renderer
/// process (one pid for both tabs): it sees `opener`, reads and sets
/// `opener.document.title` and reads the new one back, all in its own
/// script's turn, and has the name its opener gave it. A `noopener`
/// popup gets a renderer of its own, where `opener` is null.
#[test]
fn window_renderer_popup_opener() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let state = popup_window(&srv.base, OPEN_SAME);
    let p = tab_pids(&state);
    assert!(state.contains("\ntabs 2, showing 2\n"), "{state}");
    assert!(state.contains(&format!("\nrenderer {} title opener Opener>set-by?same name w1 ?same\n", p[0])), "{state}");
    assert!(p.len() == 2 && p[0] == p[1] && p[0] != "0", "a same-site popup shares its opener's process: {p:?}\n{state}");

    let state = popup_window(&srv.base, "renderer:loaded click:50,106 renderer:input renderer:title:?none");
    let p = tab_pids(&state);
    assert!(state.contains(&format!("\nrenderer {} title opener null name ?none\n", p[1])), "{state}");
    assert!(p.len() == 2 && p[0] != p[1] && p[0] != "0" && p[1] != "0", "a renderer of its own: {p:?}\n{state}");
}

/// Q2.15n: a popup opened on another site with its opener kept runs in
/// a renderer of its own and still has its opener and its name: reading
/// `opener.document` is a SecurityError, `opener.postMessage` reaches
/// the opener, whose `event.source` is the window `window.open`
/// returned, and the answer posted to that source comes back with
/// `event.source === window.opener`. In `tab` and `site` mode.
#[test]
fn window_renderer_popup_cross_opener() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let open = "renderer:loaded click:50,146 renderer:input renderer:title:?cross!";
    for mode in ["tab", "site"] {
        let state = popup_window_in(mode, &srv.base, open);
        let p = tab_pids(&state);
        assert!(state.contains(&format!("\nrenderer {} title pong-true name w2 ?cross!\n", p[1])), "{mode}:\n{state}");
        assert!(p.len() == 2 && p[0] != p[1] && p[0] != "0" && p[1] != "0", "{mode}: a renderer of its own: {p:?}\n{state}");
        let state = popup_window_in(mode, &srv.base, &format!("{open} key:cmd+1 renderer:title:got-hi-true"));
        assert!(state.contains(&format!("\nrenderer {} title got-hi-true\n", tab_pids(&state)[0])), "{mode}:\n{state}");
    }
}

/// Q2.12d: a popup that shares its opener's renderer and navigates to
/// another site leaves the process: it commits in a renderer of its own,
/// where its opener is a proxy (Q2.15n: a message to it is answered,
/// the name stays), and the opener's renderer lives and takes input.
#[test]
fn window_renderer_popup_leaves() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let away = format!("{OPEN_SAME} click:50,92 renderer:input renderer:title:?away!");
    let state = popup_window(&srv.base, &away);
    let p = tab_pids(&state);
    assert!(state.contains(&format!("\nrenderer {} title pong-true name w1 ?away!\n", p[1])) && state.contains("\nrenderer-site http://localhost\n"), "{state}");
    assert!(p.len() == 2 && p[0] != p[1] && p[0] != "0" && p[1] != "0", "{p:?}\n{state}");

    let state = popup_window(&srv.base, &format!("{away} key:cmd+1 click:50,212 renderer:input renderer:title:opener-alive"));
    assert!(state.contains(&format!("\nrenderer {} title opener-alive\n", tab_pids(&state)[0])), "{state}");
}

/// Q2.12d: the opener's tab closes while its popup lives in the same
/// renderer: the process stays and the popup takes input.
#[test]
fn window_renderer_popup_outlives_opener() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let state = popup_window(&srv.base, &format!("{OPEN_SAME} key:cmd+1 key:cmd+w click:50,106 renderer:input renderer:title:popup-alive"));
    let p = tab_pids(&state);
    assert!(p.len() == 1 && p[0] != "0", "{p:?}\n{state}");
    assert!(state.contains(&format!("\nrenderer {} title popup-alive\n", p[0])), "{state}");
}

/// Q2.12d: a crash of a renderer that two tabs share ends both: each
/// shows the sad tab.
#[test]
fn window_renderer_popup_crash() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    for (then, showing) in [("", 2), (" key:cmd+1 frame", 1)] {
        let state = popup_window(&srv.base, &format!("{OPEN_SAME} renderer:kill renderer:gone frame{then}"));
        let p = tab_pids(&state);
        assert!(state.contains(&format!("\ntabs 2, showing {showing}\n")), "{state}");
        assert!(state.contains(&format!("\nrenderer {} gone crashed\nsad-tab This page crashed\n", p[0])) && p[0] == p[1], "tab {showing}:\n{state}");
    }
}

/// Q2.12d: a click in a popup that shares its opener's renderer is the
/// gesture for a popup of its own, which joins the same process and
/// sees the first popup as its opener.
#[test]
fn window_renderer_popup_of_popup() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let state = popup_window(&srv.base, &format!("{OPEN_SAME} click:50,172 renderer:input renderer:title:?third"));
    let p = tab_pids(&state);
    assert!(state.contains("\ntabs 3, showing 3\n"), "{state}");
    assert!(p.len() == 3 && p[0] == p[1] && p[1] == p[2] && p[0] != "0", "{p:?}\n{state}");
    assert!(
        state.contains(&format!("\nrenderer {} title opener opener Opener>set-by?same name w1 ?same>set-by?third name w3 ?third\n", p[0])),
        "{state}"
    );
}

/// Q2.15m: a popup that left its opener's renderer for another site and
/// navigates back to the opener's site (a sign-in round trip) commits in
/// its opener's process again, in the context its opener made: `opener`
/// and its name are there, and it scripts the opener's document. The
/// same with `DWELLER_PROCESS=site`.
#[test]
fn window_renderer_popup_returns() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let events = format!("{OPEN_SAME} click:50,92 renderer:input renderer:title:?away! click:50,92 renderer:input renderer:title:?back");
    for mode in ["tab", "site"] {
        let state = popup_window_in(mode, &srv.base, &events);
        let p = tab_pids(&state);
        assert!(state.contains(&format!("\nrenderer {} title opener got-hi-true>set-by?back name w1 ?back\n", p[0])), "{mode}:\n{state}");
        assert!(p.len() == 2 && p[0] == p[1] && p[0] != "0", "{mode}: back in its opener's process: {p:?}\n{state}");
        assert!(state.contains("\nrenderer-site http://127.0.0.1\n"), "{mode}:\n{state}");
    }
}

/// Q2.15m: the opener's tab navigates to another site while its popup
/// lives in the same renderer: the popup's tab and process stay and take
/// input, and the opener commits in a renderer of its own.
#[test]
fn window_renderer_popup_opener_leaves() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    for mode in ["tab", "site"] {
        let state = popup_window_in(mode, &srv.base, &format!("{OPEN_SAME} key:cmd+1 click:50,252 renderer:input renderer:title:?away key:cmd+2 click:50,132 renderer:input renderer:title:popup-alive"));
        let p = tab_pids(&state);
        assert!(p.len() == 2 && p[0] != p[1] && p[0] != "0" && p[1] != "0", "{mode}: {p:?}\n{state}");
        assert!(state.contains(&format!("\nrenderer {} title popup-alive\n", p[1])), "{mode}:\n{state}");
    }
}

/// Q2.15m: the opener's reference to its popup follows the popup's tab,
/// not its renderer: `closed` is false while the popup is on another
/// site in another process, and true once its tab is closed, at home or
/// away.
#[test]
fn window_renderer_popup_closed() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let away = format!("{OPEN_SAME} click:50,92 renderer:input renderer:title:?away!");
    for mode in ["tab", "site"] {
        let state = popup_window_in(mode, &srv.base, &format!("{away} key:cmd+1 click:50,292 renderer:input renderer:title:closed-false"));
        assert!(state.contains(" title closed-false\n"), "{mode}:\n{state}");
        for open in [OPEN_SAME, away.as_str()] {
            let state = popup_window_in(mode, &srv.base, &format!("{open} key:cmd+w click:50,264 renderer:input renderer:title:closed-true"));
            assert!(state.contains(" title closed-true\n") && tab_pids(&state).len() == 1, "{mode} {open}:\n{state}");
        }
    }
}

/// Q2.15n: a popup opened on another site with its opener kept (a
/// sign-in window) runs in a renderer of its own; when it navigates to
/// its opener's site it commits in the opener's process, in the context
/// the opener made for it: `opener` and the name `w2` are there, and it
/// scripts the opener's document.
#[test]
fn window_renderer_popup_cross_returns() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let events = "renderer:loaded click:50,146 renderer:input renderer:title:?cross! click:50,92 renderer:input renderer:title:?away";
    for mode in ["tab", "site"] {
        let state = popup_window_in(mode, &srv.base, events);
        let p = tab_pids(&state);
        assert!(state.contains(&format!("\nrenderer {} title opener got-hi-true>set-by?away name w2 ?away\n", p[0])), "{mode}:\n{state}");
        assert!(p.len() == 2 && p[0] == p[1] && p[0] != "0", "{mode}: in its opener's process: {p:?}\n{state}");
    }
}

/// Q2.15n: the opener sets `location.href` of a popup that runs in
/// another renderer: the browser navigates the popup's tab, which keeps
/// its opener and its name.
#[test]
fn window_renderer_popup_cross_navigate() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let events = "renderer:loaded click:50,146 renderer:input renderer:title:?cross! key:cmd+1 click:50,332 renderer:input key:cmd+2 renderer:title:?moved!";
    for mode in ["tab", "site"] {
        let state = popup_window_in(mode, &srv.base, events);
        let p = tab_pids(&state);
        assert!(state.contains(&format!("\nrenderer {} title pong-true name w2 ?moved!\n", p[1])), "{mode}:\n{state}");
        assert!(p.len() == 2 && p[0] != p[1], "{mode}: {p:?}\n{state}");
    }
}

/// Q2.15o: a `MessagePort` in the transfer list of a `postMessage` to a
/// window in another renderer arrives as `event.ports[0]` (the same
/// object as the port inside the data), and messages on the channel go
/// both ways between the two processes.
#[test]
fn window_renderer_popup_port() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let events = "renderer:loaded click:50,146 renderer:input renderer:title:?cross! click:50,212 renderer:input renderer:title:?cross!!";
    for mode in ["tab", "site"] {
        let state = popup_window_in(mode, &srv.base, events);
        let p = tab_pids(&state);
        assert!(state.contains(&format!("\nrenderer {} title port2-ping-true ?cross!!\n", p[1])) && p[0] != p[1], "{mode}:\n{state}");
        let state = popup_window_in(mode, &srv.base, &format!("{events} key:cmd+1 renderer:title:port1-echo-ping"));
        assert!(state.contains(&format!("\nrenderer {} title port1-echo-ping\n", tab_pids(&state)[0])), "{mode}:\n{state}");
    }
}
