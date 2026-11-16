use super::*;

/// Q5.8: EventSource (`browser/tests/net/event_source.html` through the
/// test server's `/event-stream`) must print `event_source.txt`: a named
/// event with its id, the reconnection after `retry` that sends
/// `Last-Event-ID`, `close()`, and a wrong MIME type failing the
/// connection. The same with `DWELLER_PROCESS=site`, where the browser
/// process does the network.
#[test]
fn eventsource_last_event_id() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/net/event_source.html", srv.base);
    let out = run(&["dump", "--text", "--js", &href]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/event_source.txt"),
        &text(&out.stdout),
        "eventsource_last_event_id",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let want = std::fs::read_to_string(root().join("browser/tests/net/event_source.txt")).unwrap();
    let site = login_window_with(&href, "frame", None, &[("DWELLER_PROCESS", "site")]);
    assert!(site.contains(want.trim_end()), "{site}");
}

/// Q5.8, slice 2 and 3: an event of a stream that stays open arrives
/// while the response runs. `/event-stream-open` sends "held" and ends
/// only after `/event-stream-end`, which the page asks for from the
/// handler of "held" ("timeout" instead of "last" would mean the event
/// came only when the server gave up). In one process, and with
/// `DWELLER_PROCESS=site`, where the browser process forwards the parts.
#[test]
fn eventsource_open_stream() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/net/event_source_open.html", srv.base);
    let out = run(&["dump", "--text", "--js", &format!("{href}?dump")]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let mut failures = Vec::new();
    check_golden(
        &root().join("browser/tests/net/event_source_open.txt"),
        &text(&out.stdout),
        "eventsource_open_stream",
        &mut failures,
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return;
    }
    let want = std::fs::read_to_string(root().join("browser/tests/net/event_source_open.txt")).unwrap();
    let window = login_window_with(&format!("{href}?window"), "frame", None, &[]);
    assert!(window.contains(want.trim_end()), "{window}");
    let site = login_window_with(&format!("{href}?site"), "frame", None, &[("DWELLER_PROCESS", "site")]);
    assert!(site.contains(want.trim_end()), "{site}");
}
