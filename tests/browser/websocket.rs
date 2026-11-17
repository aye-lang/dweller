use super::*;

fn websocket_page(envs: &[(&str, &str)]) -> Option<(String, String)> {
    let srv = server()?;
    let href = format!("{}/net/websocket.html", srv.base);
    let golden = root().join("browser/tests/net/websocket.txt");
    if envs.is_empty() {
        let out = run(&["dump", "--text", "--js", &href]);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        let mut failures = Vec::new();
        check_golden(&golden, &text(&out.stdout), "websocket", &mut failures);
        assert!(failures.is_empty(), "{}", failures.join("\n"));
    }
    if !has_sdl() {
        println!("skipped the window: no SDL2");
        return None;
    }
    let want = std::fs::read_to_string(golden).unwrap();
    Some((want, login_window_with(&href, "frame", None, envs)))
}

/// Q5.7: `WebSocket` (`browser/tests/net/websocket.html` against the test
/// server's `/ws/`) must print `websocket.txt`: the constructor's and
/// `close()`'s argument checks, text, binary and Blob messages, a
/// fragmented message around a ping, the subprotocol, the `Origin` and
/// cookies of the handshake and its `Set-Cookie`, both close handshakes,
/// and the failures (a reserved bit, a dropped connection, a refused
/// handshake, a wrong `Sec-WebSocket-Accept`, a subprotocol not offered).
/// On the virtual clock (`dump`) and in the window.
#[test]
fn websocket_echo() {
    let Some((want, window)) = websocket_page(&[]) else {
        println!("skipped: no test server or no SDL2");
        return;
    };
    assert!(window.contains(want.trim_end()), "{window}");
}

/// Q5.7: the same page with a renderer process per tab and per site: the
/// socket is the browser process's (IPC `SocketOpen`, `SocketSend`,
/// `SocketClose`, `SocketNote`), the lines are the same.
#[test]
fn websocket_echo_in_renderers() {
    for mode in ["tab", "site"] {
        let Some((want, window)) = websocket_page(&[("DWELLER_PROCESS", mode)]) else {
            println!("skipped: no test server or no SDL2");
            return;
        };
        assert!(window.contains(want.trim_end()), "{mode}: {window}");
    }
}
