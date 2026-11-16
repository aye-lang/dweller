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
