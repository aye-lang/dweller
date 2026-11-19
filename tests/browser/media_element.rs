use super::*;

/// Q5.9a slice 1: the media element (`browser/tests/media/element.html`,
/// a WAV file of half a second) must print `element.txt`: `canPlayType`,
/// the events of a load in the standard's order, `play()` to `ended` on
/// the media clock with `timeupdate`s and the `played` range, a seek,
/// `playbackRate`, `loop`, `volume`'s range, the autoplay policy (an
/// unmuted `play()` without a gesture is `NotAllowedError`), a resource
/// that is not media (`MEDIA_ERR_SRC_NOT_SUPPORTED`), `<source>` selection
/// with its `error` events, and `load()`'s `abort` and `emptied`. On the
/// virtual clock (`dump`) and in the window with `DWELLER_PROCESS=site`.
#[test]
fn media_element_wav() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/media/element.html", srv.base);
    let out = run(&["dump", "--text", "--js", &href]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let golden = root().join("browser/tests/media/element.txt");
    let mut failures = Vec::new();
    check_golden(&golden, &text(&out.stdout), "media_element_wav", &mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let want = std::fs::read_to_string(golden).unwrap();
    let site = login_window_with(&href, "frame", None, &[("DWELLER_PROCESS", "site")]);
    for line in want.lines().filter(|l| l.contains(": ")) {
        assert!(site.contains(line), "{line} in {site}");
    }
}
