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
