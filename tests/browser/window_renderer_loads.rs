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

/// Q2.12b: with `DWELLER_PROCESS=tab` the browser fetches a tab's
/// navigation in the background: while the server waits 1 s before it
/// answers (`/sleep?ms=1000`), the window's loop keeps running (no
/// `ui_loop` record over 500 ms, and many of them while the page loads)
/// and takes the typing into the address bar; the page then loads as
/// before.
#[test]
fn window_renderer_loads_slow_navigation() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/sleep?ms=1000", srv.base);
    let mut window = dweller_command();
    window
        .current_dir(root())
        .env("DWELLER_PROCESS", "tab")
        .env("DWELLER_TIMING", "1")
        .args([
            &href, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "4000",
            "--until-idle", "--state", "--text", "--events", "key:cmd+l text:abc renderer:loaded frame",
        ]);
    let began = std::time::Instant::now();
    let out = window.output().unwrap();
    let took = began.elapsed().as_millis();
    let got = text(&out.stdout);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let loops: Vec<u64> = text(&out.stderr)
        .lines()
        .filter_map(|l| l.strip_prefix("info: DWELLER_TIMING\tui_loop\t"))
        .filter_map(|l| l.split('\t').next()?.parse::<u64>().ok())
        .map(|ns| ns / 1_000_000)
        .collect();
    let longest = loops.iter().copied().max().unwrap_or(0);
    assert!(took >= 1000 && got.contains("slept 1000 ms") && got.contains("\naddress abc\n"), "{took} ms:\n{got}");
    assert!(longest < 500 && loops.len() >= 20, "longest iteration {longest} ms of {}:\n{got}", loops.len());
    println!("longest iteration {longest} ms, {} iterations in {took} ms", loops.len());
}
