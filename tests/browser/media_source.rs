use super::*;

/// Q5.9b: Media Source Extensions (`browser/tests/media/source.html`,
/// `stream.mkv` of `make_clip.py`: ten clusters of uncompressed video
/// and PCM) must print `source.txt`: `isTypeSupported` and the
/// exceptions of the API, the events of each `appendBuffer`, the
/// metadata after the initialization segment (also when it comes in two
/// parts), `buffered` after two clusters and after two later ones (a
/// gap), a seek into the gap that ends when the gap is filled and shows
/// the frame of its time, `remove`, `endOfStream` and its duration,
/// playing to the end, a change that reopens the source, removing the
/// buffer and detaching. On the virtual clock and in the window with
/// `DWELLER_PROCESS=site`.
#[test]
fn media_source_stream() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/media/source.html", srv.base);
    let out = run(&["dump", "--text", "--js", &href]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let golden = root().join("browser/tests/media/source.txt");
    let mut failures = Vec::new();
    check_golden(&golden, &text(&out.stdout), "media_source_stream", &mut failures);
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
