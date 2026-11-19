use super::*;

const CLIPBOARD: &str = "browser/tests/ui/clipboard.html";
const WANT: [&str; 6] = [
    "api true true false",
    "early write NotAllowedError",
    "early read NotAllowedError",
    "read from the page é",
    "types text/plain",
    "item an item",
];

fn clipboard_report(out: &str) -> Vec<&str> {
    out.lines()
        .filter(|l| ["api ", "early ", "read ", "types ", "item ", "failed "].iter().any(|p| l.starts_with(p)))
        .collect()
}

/// Q5.10, slice 1: `navigator.clipboard` (`browser/tests/ui/clipboard.html`).
/// Without a user gesture `writeText` and `readText` are refused
/// (NotAllowedError); from a click the page writes text, reads it back,
/// and does the same with a `ClipboardItem` of text/plain. In one
/// process the engine's hooks reach the window's clipboard; with
/// `DWELLER_PROCESS=site` the renderer asks the browser process (IPC
/// `ClipboardWrite`, `ClipboardRead`, `ClipboardText`).
#[test]
fn window_clipboard_async_api() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let one = login_window_with(CLIPBOARD, "frame click:50,62 frame frame", None, &[]);
    assert_eq!(clipboard_report(&one), WANT, "{one}");
    let site = login_window_with(CLIPBOARD, "click:50,62 renderer:input frame frame", None, &[("DWELLER_PROCESS", "site")]);
    assert_eq!(clipboard_report(&site), WANT, "{site}");
}
