use super::*;

fn page_lines(got: &str) -> String {
    got.lines()
        .filter(|l| !l.starts_with("renderer") && !l.starts_with("frames "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Q2.4e: with `DWELLER_PROCESS=tab` the renderer's loads are in flight at
/// once (`browser/tests/net/in_flight.html`: the server answers three
/// images, a fetch and an XHR only once all five are held), as in the
/// window's own process.
#[test]
fn window_renderer_loads_in_flight() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/net/in_flight.html", srv.base);
    let want = "fetch d slow\nheld at once: 5\nimage a 64\nimage b 64\nimage c 64\nxhr e slow";
    let own = login_window_with(&href, "frame", None, &[]);
    assert!(own.contains(want), "{own}");
    let began = std::time::Instant::now();
    let renderer = login_window_with(&href, "frame", None, &[("DWELLER_PROCESS", "tab")]);
    assert!(renderer.contains(want), "{renderer}");
    assert!(began.elapsed() < std::time::Duration::from_secs(20), "the held loads waited for the server's limit");
    assert_eq!(page_lines(&renderer), page_lines(&own));
}

/// Q2.4e: a load cancelled in the renderer stops in the browser: an
/// aborted fetch, the fetch of a removed frame
/// (`browser/tests/net/abort_fetch.html`) and the fetch of a page left
/// (`leave_fetch.html`) close their connections at the server, and the
/// aborted promise only rejects; the same as in the window's own process.
#[test]
fn window_renderer_loads_cancel() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    for (page, want) in [("abort_fetch.html", "a: AbortError\nserver: a closed f closed"), ("leave_fetch.html", "server: n closed")] {
        let href = format!("{}/net/{page}", srv.base);
        let own = login_window_with(&href, "frame", None, &[]);
        assert!(own.contains(want), "{page}: {own}");
        let renderer = login_window_with(&href, "frame", None, &[("DWELLER_PROCESS", "tab")]);
        assert!(renderer.contains(want) && !renderer.lines().any(|l| l.starts_with("late answer")), "{page}: {renderer}");
    }
}
