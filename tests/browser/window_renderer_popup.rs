use super::*;

const OPEN_SAME: &str = "renderer:loaded click:50,62 renderer:input renderer:title:?same";

fn popup_window(base: &str, events: &str) -> String {
    let page = format!("{base}/ui/popup_opener.html");
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "tab")
        .args([page.as_str(), "--font", "fixed", "--width", "400", "--height", "400"])
        .args(["--frames", "1000", "--until-idle", "--state", "--processes", "--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
    assert!(out.stderr.is_empty(), "{events}: {}", text(&out.stderr));
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
/// script's turn, and has the name its opener gave it. A
/// `noopener` popup and a cross-site one each get a renderer of their
/// own, where `opener` is null (and the cross-site one's name `w2` does
/// not reach its renderer yet: names across renderers are Q2.12d's Left).
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

    for (y, query) in [(106, "?none"), (146, "?cross")] {
        let state = popup_window(&srv.base, &format!("renderer:loaded click:50,{y} renderer:input renderer:title:{query}"));
        let p = tab_pids(&state);
        assert!(state.contains(&format!("\nrenderer {} title opener null name {query}\n", p[1])), "{query}:\n{state}");
        assert!(p.len() == 2 && p[0] != p[1] && p[0] != "0" && p[1] != "0", "{query}: a renderer of its own: {p:?}\n{state}");
    }
}

/// Q2.12d: a popup that shares its opener's renderer and navigates to
/// another site leaves the process: it commits in a renderer of its own
/// with `opener` null, and the opener's renderer lives and takes input.
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
    let away = format!("{OPEN_SAME} click:50,92 renderer:input renderer:title:?away");
    let state = popup_window(&srv.base, &away);
    let p = tab_pids(&state);
    assert!(state.contains(&format!("\nrenderer {} title opener null name ", p[1])) && state.contains("\nrenderer-site http://localhost\n"), "{state}");
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
