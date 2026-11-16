use super::*;

/// `page` in a 400x340 window (with `DWELLER_PROCESS=tab` when `tab`):
/// the PNG saved to `png`, the `--state` text and the error text.
fn shoot(page: &str, png: &Path, events: &str, envs: &[(&str, &str)]) -> (String, String) {
    let out = dweller_command()
        .current_dir(root())
        .envs(envs.iter().copied())
        .args([page, "--font", "fixed", "--width", "400", "--height", "340"])
        .args(["--frames", "2000", "--until-idle", "--state", "--save"])
        .arg(png)
        .args(["--events", events])
        .output()
        .unwrap();
    assert!(out.status.success(), "{events}: {}\n{}", out.status, text(&out.stderr));
    (text(&out.stdout), text(&out.stderr))
}

/// Runs `events` in the in-process window and in the tab's renderer
/// (each `~` becomes `wait` for the renderer, nothing for the window):
/// the two PNGs must be the same. Gives the renderer's state.
fn same_pixels(page: &str, events: &str, wait: &str, name: &str) -> String {
    let golden = scratch().join(format!("{name}_in_process.png"));
    let shot = scratch().join(format!("{name}_tab.png"));
    let tab = [("DWELLER_PROCESS", "tab"), ("DWELLER_STEADY_CARET", "1")];
    let (_, err) = shoot(page, &golden, &format!("loaded {}", events.replace('~', "")), &[IN_PROCESS, ("DWELLER_STEADY_CARET", "1")]);
    assert!(err.is_empty(), "{events}: {err}");
    let renderer = format!("renderer:loaded renderer:frame {} {wait}", events.replace('~', wait));
    let (state, err) = shoot(page, &shot, &renderer, &tab);
    assert!(err.is_empty(), "{events}: {err}");
    assert!(
        std::fs::read(&shot).unwrap() == std::fs::read(&golden).unwrap(),
        "{events}: the renderer's frame {} differs from the in-process window {}\n{state}",
        shot.display(),
        golden.display()
    );
    state
}

/// The page's own picture without any input.
fn still(page: &str, name: &str) -> Vec<u8> {
    let png = scratch().join(format!("{name}_still.png"));
    let (_, err) = shoot(page, &png, "loaded", &[IN_PROCESS, ("DWELLER_STEADY_CARET", "1")]);
    assert!(err.is_empty(), "{err}");
    std::fs::read(&png).unwrap()
}

fn killed(page: &str, evil: &str, events: &str, reason: &str) {
    let png = scratch().join("renderer_edit_evil.png");
    let envs = [("DWELLER_PROCESS", "tab"), ("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", evil)];
    let (state, err) = shoot(page, &png, events, &envs);
    let line = state.lines().find(|l| l.starts_with("renderer ")).unwrap_or("");
    let pid = line.split(' ').nth(1).unwrap_or("");
    assert!(line.ends_with(&format!(" gone {reason}")), "{evil}: {state}");
    assert_eq!(err, format!("error: bad message: {reason} {pid}\n"), "{evil}");
}

/// Q2.3f: with `DWELLER_PROCESS=tab` Cmd+A, Cmd+C, Cmd+X and Cmd+V in a
/// text field (`browser/tests/ui/form.html`) give the in-process
/// window's page: the browser owns the clipboard, the renderer selects,
/// serves the selected text and takes the pasted text as typed input. A
/// renderer that writes the clipboard without a copy is killed.
#[test]
fn window_renderer_clipboard() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = "browser/tests/ui/form.html";
    let before = still(page, "clipboard");
    let cases = [
        ("copy", "click:60,60 key:cmd+a key:cmd+c ~ key:end key:cmd+v"),
        ("cut", "click:60,60 key:left key:shift+left key:cmd+x ~ key:end key:cmd+v"),
        ("select all", "click:60,60 key:cmd+a text:xy"),
    ];
    for (name, events) in cases {
        same_pixels(page, events, "renderer:input", &format!("clipboard_{name}"));
        let golden = scratch().join(format!("clipboard_{name}_in_process.png"));
        assert!(std::fs::read(golden).unwrap() != before, "{name}: the in-process window did not change the page");
    }
    killed(page, "clipboard", "renderer:loaded renderer:frame click:60,60 key:cmd+a renderer:gone", "bad clipboard");
}

/// Q2.3f: with `DWELLER_PROCESS=tab` Cmd+F opens the window's find bar
/// and the renderer finds the text in its page, paints the matches and
/// scrolls to the active one (`browser/tests/ui/long.html`): the PNG of
/// each case is the in-process window's, find bar included. A reply for
/// a query that was not asked kills the renderer.
#[test]
fn window_renderer_find() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = "browser/tests/ui/long.html";
    let cases = [
        "key:cmd+f text:the ~",
        "key:cmd+f text:the ~ key:return ~ key:return ~",
        "key:cmd+f text:the ~ key:return ~ key:return ~ key:shift+return ~",
        "key:cmd+f text:the ~ key:escape ~",
        "key:cmd+f text:zzz ~",
    ];
    for (i, events) in cases.iter().enumerate() {
        let state = same_pixels(page, events, "renderer:find", &format!("find_{i}"));
        let matches = if i == 4 { "matches 0" } else if i == 3 { "matches 0" } else { "matches 3" };
        assert!(state.contains(&format!("\n{matches}\n")), "{events}: {state}");
    }
    killed(page, "find", "renderer:loaded renderer:frame key:cmd+f text:t renderer:gone", "bad find_reply");
}

/// Q2.3f: with `DWELLER_PROCESS=tab` the arrow keys step a focused
/// number and range input and select an option of a focused select, and
/// a space released over a checkbox leaves the page laid out for the
/// next Tab: the PNGs are the in-process window's.
#[test]
fn window_renderer_arrow_keys() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let html5 = "browser/tests/ui/forms_html5.html";
    for (i, events) in ["key:tab key:up key:up", "key:tab text:3 key:up", "key:tab key:tab key:right key:down"].iter().enumerate() {
        same_pixels(html5, events, "renderer:input", &format!("arrows_html5_{i}"));
    }
    let form = "browser/tests/ui/form.html";
    let events = "key:tab key:tab key:tab key:space keyup:space key:tab key:down key:down";
    same_pixels(form, events, "renderer:input", "arrows_select");
}

/// Q2.3f: with `DWELLER_PROCESS=tab` clicks in the frames of a frameset
/// (`browser/tests/ui/frames.html`) load a link in its own frame, in the
/// frame its target names, or in the tab (`_top`), and the status line
/// shows a frame's link resolved there (the hover is checked by its
/// state, not its pixels).
#[test]
fn window_renderer_frame_click() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = "browser/tests/ui/frames.html";
    for (i, events) in ["click:20,58", "click:20,95"].iter().enumerate() {
        same_pixels(page, events, "renderer:input", &format!("frame_click_{i}"));
    }
    let state = same_pixels(page, "click:20,131", "renderer:input", "frame_click_top");
    let first = state.lines().next().unwrap_or("");
    assert!(first.ends_with("/browser/tests/ui/other.html"), "{state}");
    let png = scratch().join("frame_hover.png");
    let (state, err) = shoot(page, &png, "renderer:loaded renderer:frame move:20,58 renderer:input", &[("DWELLER_PROCESS", "tab")]);
    assert!(err.is_empty(), "{err}");
    assert!(state.contains("/browser/tests/ui/other.html\n") && state.contains("\nrenderer-cursor pointer\n"), "{state}");
}

/// Q2.3f: with `DWELLER_PROCESS=tab` a link or form with `target=_blank`
/// opens a tab, a name is reused by later links, a POST form carries its
/// body into the new tab, and a request without a user gesture or with
/// an unknown ask kills (or does nothing to) the renderer
/// (`browser/tests/ui/renderer_targets.html`).
#[test]
fn window_renderer_targets() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = "browser/tests/ui/renderer_targets.html";
    let png = scratch().join("renderer_targets.png");
    let lines = |state: &str| -> Vec<String> {
        state.lines().filter(|l| l.starts_with("url ") || l.starts_with("tabs ")).map(String::from).collect()
    };
    let cases = [
        ("blank", "click:10,60 ~", "tabs 2, showing 2", "other.html?blank"),
        ("form", "click:10,180 ~", "tabs 2, showing 2", "result.html?b=get"),
        (
            "named",
            "click:10,100 ~ key:cmd+1 ~ click:10,168 ~ key:cmd+2 ~",
            "tabs 2, showing 2",
            "other.html?second",
        ),
    ];
    for (name, events, tabs, url) in cases {
        let (in_process, err) = shoot(page, &png, &format!("loaded {}", events.replace('~', "frame frame")), &[IN_PROCESS]);
        assert!(err.is_empty(), "{name}: {err}");
        let tab_events = format!("renderer:loaded renderer:frame {}", events.replace('~', "renderer:input renderer:loaded"));
        let (tab, err) = shoot(page, &png, &tab_events, &[("DWELLER_PROCESS", "tab")]);
        assert!(err.is_empty(), "{name}: {err}");
        assert_eq!(lines(&tab), lines(&in_process), "{name}\n{tab}");
        assert!(tab.contains(&format!("\n{tabs}\n")) && tab.contains(url), "{name}\n{tab}");
    }

    let served = format!("{}/ui/renderer_targets.html", srv.base);
    let events = "renderer:loaded renderer:frame click:10,220 renderer:input renderer:loaded";
    let (state, err) = shoot(&served, &png, events, &[("DWELLER_PROCESS", "tab")]);
    assert!(err.is_empty(), "{err}");
    let log = text(&run(&["dump", "--text", &format!("{}/log", srv.base)]).stdout);
    assert!(state.contains("\ntabs 2, showing 2\n") && state.contains(" title Logged\n"), "{state}");
    assert!(log.contains("POST /log/renderer-post.html") && log.contains("body: b=post"), "{log}");

    killed(page, "context", "renderer:loaded renderer:frame click:10,60 renderer:input renderer:gone", "bad context");
}
