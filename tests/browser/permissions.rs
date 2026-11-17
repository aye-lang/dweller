use super::*;

const ANSWERS: &str = "asked click:288,56 asked click:360,56 asked click:288,56 asked click:360,56 frame frame";

/// Q5.12: without a window nobody can answer a prompt, so every request
/// is dismissed: the four permissions stay `prompt`, the APIs answer as
/// without the permission, nothing is stored
/// (`browser/tests/net/permissions.txt`).
#[test]
fn permissions_dismissed_without_a_window() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&["dump", "--text", "--js", &format!("{}/net/permissions.html", srv.base)]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/permissions.txt"),
        &text(&out.stdout),
        "permissions",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Q5.12: the window's prompt. The page asks for notifications (Allow),
/// the location (Block), the microphone (Allow) and the camera (Block):
/// each API answers by the click, the answers are kept in the profile per
/// origin, and a second run is not asked again. In one process and with
/// a renderer per tab and per site (the store and the prompt are the
/// browser's).
#[test]
fn permissions_prompt_and_remember() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let href = format!("{}/net/permissions.html", srv.base);
    for mode in ["single", "tab", "site"] {
        let prof = profile(&format!("permissions-{mode}"));
        let envs = [("DWELLER_PROCESS", mode)];
        let asked = login_window_with(&href, "frame", Some(&prof), &envs);
        assert!(asked.contains(&format!("\npermission {} notifications\n", srv.base)), "{mode}: {asked}");
        assert!(!asked.contains("first requestPermission"), "{mode}: {asked}");
        let answered = login_window_with(&href, &format!("frame {ANSWERS}"), Some(&prof), &envs);
        for line in [
            "first before: notifications=prompt geolocation=prompt camera=prompt microphone=prompt Notification.permission=default",
            "first requestPermission: granted changes=1 status=granted",
            "first notification: show hello b t",
            "first geolocation: error 1 true User denied Geolocation",
            "first getUserMedia audio: NotFoundError",
            "first getUserMedia video: NotAllowedError",
            "first after: notifications=granted geolocation=denied camera=denied microphone=granted Notification.permission=granted",
            "done",
        ] {
            assert!(answered.contains(line), "{mode}: {line} in {answered}");
        }
        assert!(!answered.contains("\npermission "), "{mode}: {answered}");
        let again = login_window_with(&href, "frame", Some(&prof), &envs);
        assert!(
            again.contains("first before: notifications=granted geolocation=denied camera=denied microphone=granted")
                && again.contains("first geolocation: error 1 true User denied Geolocation")
                && again.contains("first notification: show hello b t")
                && again.contains("done")
                && !again.contains("\npermission "),
            "{mode}: {again}"
        );
    }
}
