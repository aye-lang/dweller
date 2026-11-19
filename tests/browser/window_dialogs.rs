use super::*;

const DIALOGS: &str = "browser/tests/ui/dialogs.html";

fn report(out: &str) -> Vec<&str> {
    out.lines()
        .filter(|l| ["alert ", "confirm ", "prompt ", "dialog "].iter().any(|p| l.starts_with(p)))
        .collect()
}

/// Q5.3: `alert`, `confirm` and `prompt` (`browser/tests/ui/dialogs.html`,
/// each behind a button) stop the script, show the dialog and return the
/// person's answer: Return accepts (`confirm` true, `prompt` its text,
/// the default or what was typed), Escape cancels (false, null). A
/// dialog nobody answers is dismissed when a headless run's events end.
#[test]
fn window_dialogs_answers() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = login_window_with(
        DIALOGS,
        "frame click:50,52 key:return frame click:50,82 key:return frame click:50,82 key:escape frame \
         click:50,112 key:return frame click:50,112 text:Ann key:return frame click:50,112 key:escape frame click:50,82",
        None,
        &[],
    );
    assert_eq!(
        report(&out),
        [
            "alert undefined",
            "confirm true",
            "confirm false",
            "prompt \"you\"",
            "prompt \"Ann\"",
            "prompt null",
            "confirm false"
        ],
        "{out}"
    );
}

/// Q5.3 with a renderer per site: the renderer asks the browser process
/// (IPC `Dialog`, `DialogReply`) and waits; the browser shows the dialog
/// on the tab and answers. `renderer:dialog` waits for the dialog to be
/// open.
#[test]
fn window_dialogs_site() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "site")
        .args([
            DIALOGS, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--until-idle",
            "--state", "--text", "--events",
            "renderer:loaded click:50,52 renderer:dialog key:return renderer:input \
             click:50,82 renderer:dialog key:return renderer:input \
             click:50,82 renderer:dialog key:escape renderer:input \
             click:50,112 renderer:dialog text:Ann key:return renderer:input \
             click:50,112 renderer:dialog",
        ])
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{}\n{}", out.status, text(&out.stderr));
    let got = text(&out.stdout);
    assert_eq!(
        report(&got),
        ["alert undefined", "confirm true", "confirm false", "prompt \"Ann\"", "prompt null"],
        "{got}"
    );
}

/// Q5.3: a dialog that stays open (`dialog:keep`, with a renderer per
/// site, where the window's loop goes on): `--state` names it, with the
/// title from the browser's own record of the document and what was
/// typed so far, and the window draws the sheet (`dialogs_prompt.png`).
#[test]
fn window_dialogs_open_sheet() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let png = scratch().join("dialogs_prompt.png");
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "site")
        .args([
            DIALOGS, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "400", "--until-idle",
            "--state", "--events", "renderer:loaded click:50,112 renderer:dialog text:Bo dialog:keep frame", "--save",
        ])
        .arg(&png)
        .output()
        .unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{}\n{}", out.status, text(&out.stderr));
    let got = text(&out.stdout);
    assert_eq!(report(&got), ["dialog prompt [This page says] name? input Bo"], "{got}");
    let golden = root().join("browser/tests/ui/dialogs_prompt.png");
    let shot = std::fs::read(&png).unwrap();
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &shot).unwrap();
    } else {
        assert!(shot == std::fs::read(&golden).unwrap(), "the dialog differs from {}: {}", golden.display(), png.display());
    }
}
