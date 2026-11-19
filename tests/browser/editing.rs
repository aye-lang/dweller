use super::*;

/// Q5.10, slices 2 and 3: `browser/tests/editing/exec_command.html` runs
/// the core of `document.execCommand` on a `contenteditable` element
/// through the `Selection`: insertText, delete, forwardDelete, bold on
/// and off, insertParagraph, insertLineBreak, insertHTML, selectAll;
/// each prints the host's markup and the caret, and at the end the
/// `beforeinput` and `input` events seen. Outside an editing host a
/// command returns false; a canceled `beforeinput` changes nothing.
#[test]
fn editing_exec_command() {
    let out = run(&["dump", "--text", "--js", "browser/tests/editing/exec_command.html"]);
    assert!(out.status.success() && out.stderr.is_empty(), "{}\n{}", out.status, text(&out.stderr));
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/editing/exec_command.txt"),
        &text(&out.stdout),
        "editing_exec_command",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn typed_report(out: &str) -> Vec<&str> {
    out.lines()
        .filter(|l| ["focus true", "insert", "delete"].iter().any(|p| l.starts_with(p)))
        .collect()
}

/// Q5.10, slice 5: a person types into a `contenteditable` element
/// (`browser/tests/editing/typing.html`). A click gives it the focus and
/// a caret at the end of its content; text, Return, Backspace and
/// Shift+Return become `insertText`, `insertParagraph`,
/// `deleteContentBackward` and `insertLineBreak`, each with an `input`
/// event that prints the element's markup. In one process and with
/// `DWELLER_PROCESS=site`, where the keys reach the renderer.
#[test]
fn editing_typing_in_host() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let want = [
        "focus true",
        "insertText abc",
        "insertParagraph abc<div><br></div>",
        "insertText abc<div>d</div>",
        "deleteContentBackward abc<div><br></div>",
        "insertLineBreak abc<div><br><br></div>",
    ];
    let page = "browser/tests/editing/typing.html";
    let one = login_window_with(page, "frame click:50,70 text:c key:return text:d key:backspace key:shift+return frame", None, &[]);
    assert_eq!(typed_report(&one), want, "{one}");
    let site = login_window_with(
        page,
        "click:50,70 renderer:input text:c renderer:input key:return renderer:input text:d renderer:input \
         key:backspace renderer:input key:shift+return renderer:input frame",
        None,
        &[("DWELLER_PROCESS", "site")],
    );
    assert_eq!(typed_report(&site), want, "{site}");
}

fn caret_report(out: &str) -> Vec<&str> {
    out.lines().filter(|l| l.starts_with("caret ") || l.starts_with("insertText ")).collect()
}

/// Q5.10, slice 4: the caret of an editing host
/// (`browser/tests/editing/caret.html`, "hello   world" in the fixed
/// font). A click places it at the nearest character boundary of the
/// text as shown (the three spaces are one on screen); Left, Home and
/// End move it; typed text goes in at the caret. In one process and
/// with `DWELLER_PROCESS=site`.
#[test]
fn editing_caret_by_click_and_keys() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let want = [
        "caret #text 6",
        "insertText hello X  world",
        "insertText helloY X  world",
        "insertText AhelloY X  world",
        "insertText AhelloY X  worldE",
    ];
    let page = "browser/tests/editing/caret.html";
    let one = login_window_with(
        page,
        "frame click:75,62 release:75,62 text:X key:left key:left text:Y key:home text:A key:end text:E frame",
        None,
        &[],
    );
    assert_eq!(caret_report(&one), want, "{one}");
    let site = login_window_with(
        page,
        "click:75,62 release:75,62 renderer:input text:X renderer:input key:left renderer:input key:left renderer:input \
         text:Y renderer:input key:home renderer:input text:A renderer:input key:end renderer:input text:E renderer:input frame",
        None,
        &[("DWELLER_PROCESS", "site")],
    );
    assert_eq!(caret_report(&site), want, "{site}");
}

/// Q5.10, slice 4: the window draws the caret of a focused editing
/// host: after a click at the end of "hello   world" (20 px fixed font)
/// above the letters, right of the last one, there is a column of the
/// caret's ink, and it is not there before the click.
#[test]
fn editing_caret_is_drawn() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let shot = |events: &str, name: &str| {
        let png = scratch().join(name);
        let out = dweller_command()
            .current_dir(root())
            .args([
                "browser/tests/editing/caret.html", "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400",
                "--until-idle", "--events", events, "--save",
            ])
            .arg(&png)
            .output()
            .unwrap();
        assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
        png_rgba(&std::fs::read(&png).unwrap())
    };
    let dark_column = |(w, _, px): &(usize, usize, Vec<u8>)| {
        (116..128).find(|x| (55..59).all(|y| px[(y * w + x) * 4] < 60 && px[(y * w + x) * 4 + 1] < 60))
    };
    let before = shot("frame", "caret_before.png");
    let after = shot("frame click:250,62 release:250,62 frame", "caret_after.png");
    assert_eq!(dark_column(&before), None, "no caret before the click");
    assert!(dark_column(&after).is_some(), "the caret after the click");
}

fn clip_report(out: &str) -> Vec<&str> {
    out.lines()
        .filter(|l| ["copy \"", "cut \"", "paste \"", "insertText ", "deleteContent", "custom \""].iter().any(|p| l.starts_with(p)))
        .collect()
}

/// Q5.10, slice 6: Cmd+A, Cmd+C, Cmd+X and Cmd+V in an editing host
/// (`browser/tests/editing/clip.html`) fire `copy`, `cut` and `paste`
/// (ClipboardEvent with `clipboardData`) and do their default: the
/// selection's text goes to the clipboard, a cut deletes it, a paste
/// inserts the clipboard's text. In one process a second host's `copy`
/// handler replaces the data. With `DWELLER_PROCESS=site` the same keys
/// reach the renderer as edit commands and the clipboard is the
/// browser's.
#[test]
fn editing_copy_cut_paste_in_host() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let host = [
        "copy \"\" true",
        "paste \"one two\" true",
        "insertText \"one twoone two\"",
        "cut \"\" true",
        "deleteContentBackward \"\"",
        "paste \"one twoone two\" true",
        "insertText \"one twoone two\"",
    ];
    let page = "browser/tests/editing/clip.html";
    let one = login_window_with(
        page,
        "frame click:50,62 release:50,62 key:cmd+a key:cmd+c key:end key:cmd+v key:cmd+a key:cmd+x key:cmd+v \
         click:50,102 release:50,102 key:cmd+a key:cmd+c key:end key:cmd+v frame",
        None,
        &[],
    );
    let mut want = host.to_vec();
    want.push("custom \"plainmade by the page\"");
    assert_eq!(clip_report(&one), want, "{one}");
    let site = login_window_with(
        page,
        "click:50,62 release:50,62 renderer:input key:cmd+a renderer:input key:cmd+c renderer:input key:end renderer:input \
         key:cmd+v renderer:input key:cmd+a renderer:input key:cmd+x renderer:input key:cmd+v renderer:input frame",
        None,
        &[("DWELLER_PROCESS", "site")],
    );
    assert_eq!(clip_report(&site), host, "{site}");
}

fn history_report(out: &str) -> Vec<&str> {
    out.lines().filter(|l| l.starts_with("insertText ") || l.starts_with("historyUndo ") || l.starts_with("historyRedo ")).collect()
}

/// Q5.10, slice 7: Cmd+Z and Cmd+Shift+Z in an editing host
/// (`browser/tests/editing/typing.html`) undo and redo its edits, each
/// with an `input` event (`historyUndo`, `historyRedo`); a new edit
/// after an undo drops what could be redone. In one process and with
/// `DWELLER_PROCESS=site` (edit commands 4 and 5 of the browser).
#[test]
fn editing_undo_redo_in_host() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let want = [
        "insertText abc",
        "insertText abcd",
        "historyUndo abc",
        "historyUndo ab",
        "historyRedo abc",
        "insertText abce",
    ];
    let page = "browser/tests/editing/typing.html";
    let one = login_window_with(
        page,
        "frame click:50,70 text:c text:d key:cmd+z key:cmd+z key:cmd+z key:cmd+shift+z text:e key:cmd+shift+z frame",
        None,
        &[],
    );
    assert_eq!(history_report(&one), want, "{one}");
    let site = login_window_with(
        page,
        "click:50,70 renderer:input text:c renderer:input text:d renderer:input key:cmd+z renderer:input key:cmd+z renderer:input \
         key:cmd+z renderer:input key:cmd+shift+z renderer:input text:e renderer:input key:cmd+shift+z renderer:input frame",
        None,
        &[("DWELLER_PROCESS", "site")],
    );
    assert_eq!(history_report(&site), want, "{site}");
}

fn selected_report(out: &str) -> Vec<&str> {
    out.lines().filter(|l| l.starts_with("selected ")).collect()
}

/// Q5.10, Left of slice 4: a drag selects page text
/// (`browser/tests/editing/select.html`, "hello world again" in the 20 px
/// fixed font). The `Selection` holds the dragged range with its
/// direction, `selectionchange` fires, `Range.getBoundingClientRect`
/// gives the selected text's box, and the window draws the highlight.
/// In one process (forward and backward) and with `DWELLER_PROCESS=site`.
#[test]
fn editing_drag_selects_page_text() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let page = "browser/tests/editing/select.html";
    let png = scratch().join("select.png");
    let out = dweller_command()
        .current_dir(root())
        .args([
            page, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--until-idle", "--state", "--text",
            "--events", "frame click:72,62 move:112,62 move:122,62 release:122,62 frame click:122,62 move:32,62 release:32,62 frame",
            "--save",
        ])
        .arg(&png)
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{}\n{}", out.status, text(&out.stderr));
    let one = text(&out.stdout);
    assert_eq!(
        selected_report(&one),
        [
            "selected \"world\" forward rect 70,13,50,20 changes true",
            "selected \"llo world\" backward rect 30,13,90,20 changes true"
        ],
        "{one}"
    );
    let (w, _, px) = png_rgba(&std::fs::read(&png).unwrap());
    let at = |x: usize, y: usize| px[(y * w + x) * 4..(y * w + x) * 4 + 3].to_vec();
    assert!(at(80, 55)[2] > 200 && at(80, 55)[0] < 200, "the highlight over the selection: {:?}", at(80, 55));
    assert_eq!(at(150, 55), [255, 255, 255], "no highlight right of the selection");
    let site = login_window_with(
        page,
        "click:72,62 move:122,62 release:122,62 renderer:input frame frame",
        None,
        &[("DWELLER_PROCESS", "site")],
    );
    assert_eq!(selected_report(&site), ["selected \"world\" forward rect 70,13,50,20 changes true"], "{site}");
}

/// Q5.10, Left of slice 4: Shift with Left, Right, Home and End extends
/// the selection of an editing host (`typing.html`, "ab"); typed text
/// replaces what is selected.
#[test]
fn editing_shift_arrows_extend() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    for (keys, want) in [
        ("key:shift+left key:shift+left text:X", "insertText X"),
        ("key:home key:shift+right text:X", "insertText Xb"),
        ("key:shift+home text:X", "insertText X"),
        ("key:home key:shift+end text:X", "insertText X"),
    ] {
        let out = login_window_with(
            "browser/tests/editing/typing.html",
            &format!("frame click:50,62 release:50,62 {keys} frame"),
            None,
            &[],
        );
        assert!(out.lines().any(|l| l == want), "{keys}: {out}");
    }
}

/// Q5.10 (left list): Up and Down in an editing host
/// (`browser/tests/editing/lines.html`: three lines "abcdef", "ghi",
/// "mnopqr" in the fixed font). The caret goes to the line above or
/// below at the same x (the end of a shorter line), keeps that x over
/// several moves, and goes to the start or the end of the host from its
/// first or last line; Shift+Down extends the selection, and typing
/// replaces it. In one process and with `DWELLER_PROCESS=site`.
#[test]
fn editing_caret_up_and_down() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let want = [
        "caret abcdef 6",
        "abcdefghiXmnopqr at ghiX 4",
        "abcdefghiXmnopYqr at mnopYqr 5",
        "abcdeZfghiXmnopYqr at abcdeZf 6",
        "abcdeZfghiXmnopYqrE at mnopYqrE 8",
        "AabcdeZfghiXmnopYqrE at AabcdeZf 1",
        "AabcdeZfgSnopYqrE",
    ];
    let report = |out: &str| -> Vec<String> {
        out.lines()
            .filter(|l| l.starts_with("caret ") || (l.contains(" at ") && !l.contains("=>")))
            .map(|l| if l.starts_with("AabcdeZfgS") { l.split(" at ").next().unwrap().to_string() } else { l.to_string() })
            .collect()
    };
    let page = "browser/tests/editing/lines.html";
    let keys = "key:down text:X key:down text:Y key:up key:up text:Z key:down key:down key:down text:E \
                key:up key:up key:up text:A key:down key:shift+down text:S";
    let one = login_window_with(page, &format!("frame click:75,62 release:75,62 {keys} frame"), None, &[]);
    assert_eq!(report(&one), want, "{one}");
    let waited = keys.replace(' ', " renderer:input ");
    let site = login_window_with(
        page,
        &format!("click:75,62 release:75,62 renderer:input {waited} renderer:input frame"),
        None,
        &[("DWELLER_PROCESS", "site")],
    );
    assert_eq!(report(&site), want, "{site}");
}
