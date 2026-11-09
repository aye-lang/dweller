use super::*;

/// Runs `browser/tests/ui/oopif-proxy.html` (a `127.0.0.1` page with a
/// `localhost` iframe named `kid`) with `DWELLER_PROCESS=site` until
/// `events`; gives the `--state` text and stderr.
fn run_proxy(base: &str, query: &str, events: &str, envs: &[(&str, &str)]) -> String {
    let out = dweller_command()
        .env("DWELLER_PROCESS", "site")
        .current_dir(root())
        .envs(envs.iter().copied())
        .arg(format!("{base}/ui/oopif-proxy.html{query}"))
        .args(["--font", "fixed", "--width", "400", "--height", "400"])
        .args(["--frames", "1000", "--until-idle", "--state", "--save"])
        .arg(scratch().join("oopif_proxy.png"))
        .args(["--events", &format!("renderer:loaded renderer:child {events}")])
        .output()
        .unwrap();
    assert!(out.status.success(), "{query} {events}: {}\n{}", out.status, text(&out.stderr));
    if envs.is_empty() {
        assert!(out.stderr.is_empty(), "{query} {events}: {}", text(&out.stderr));
    }
    text(&out.stdout) + &text(&out.stderr)
}

fn title_of<'a>(state: &'a str, prefix: &str) -> &'a str {
    let line = state.lines().find(|l| l.starts_with(prefix)).unwrap_or("");
    line.split(" title ").nth(1).unwrap_or("")
}

fn skip() -> Option<Server> {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return None;
    }
    let srv = server();
    if srv.is_none() {
        println!("skipped: cannot start python3 for the test server");
    }
    srv
}

/// Q2.11: the parent and its out-of-process child see each other through
/// window proxies. The iframe's `load` fires after the child's own load
/// (the child answers `ping` with `complete`); `frames.length`,
/// `frames[0] === contentWindow`, and the child's `parent`, `top`, `name`
/// are right; cross-origin `document` and `location.href` throw
/// `SecurityError` both ways. `postMessage` goes both ways with a
/// structured clone (array, Map, Date, Uint8Array), `event.origin` and an
/// `event.source` that answers; a message for another `targetOrigin` is
/// dropped. `frames[0].focus()` focuses the iframe (and so the child).
#[test]
fn window_renderer_oopif_proxy_messages() {
    let Some(srv) = skip() else { return };
    let child = srv.base.replace("127.0.0.1", "localhost");
    let state = run_proxy(&srv.base, "", "renderer:title:kid", &[]);
    let parent = format!("load 1 frames 1 true doc SecurityError href SecurityError,focused true,state complete {child} kid,bytes 7.8.9 {child} kid");
    assert_eq!(title_of(&state, "renderer "), format!("P {parent}"), "{state}");
    let kid = format!("C parent true top true name kid length 0 doc SecurityError href SecurityError closed false,msg {{\"a\":[1,\"two\"],\"m\":[[\"k\",2]],\"d\":\"1970-01-01T00:00:00.005Z\"}} {} true", srv.base);
    assert_eq!(title_of(&state, "renderer-child "), kid, "{state}");
}

/// Q2.11: a grandchild (another renderer, below the child) reaches its
/// parent with `parent.postMessage` (the child's `event.source` is its
/// `frames[0]`) and the top with `top.postMessage`; the top's reply to
/// that `event.source` reaches the grandchild, whose `event.source` is its
/// `top`.
#[test]
fn window_renderer_oopif_proxy_nested() {
    let Some(srv) = skip() else { return };
    let state = run_proxy(&srv.base, "?nested", "renderer:title:done renderer:child-title:true", &[]);
    assert!(title_of(&state, "renderer ").contains(&format!("gctop {} other", srv.base)), "{state}");
    assert!(title_of(&state, "renderer-child ").contains(&format!("gc {} true", srv.base)), "{state}");
    let grandchild = state.lines().filter(|l| l.starts_with("renderer-child ")).nth(1).unwrap_or("");
    assert!(grandchild.ends_with(&format!("title C back {} true", srv.base)), "{state}");
    assert!(title_of(&state, "renderer ").contains(&format!("got back true {} other,done", srv.base)), "{state}");
}

/// Q2.11: the parent's `frames[0].location = URL` navigates the child
/// through the browser (same renderer, same site); the iframe's `load`
/// fires again after the child's new document loaded. A child's
/// `top.location = URL` navigates the tab; in an iframe with
/// `sandbox="allow-scripts"` it throws `SecurityError` and the tab stays.
#[test]
fn window_renderer_oopif_proxy_navigate() {
    let Some(srv) = skip() else { return };
    let state = run_proxy(&srv.base, "?nav", "renderer:child-title:moved renderer:title:load_2_frames_1_true_doc_SecurityError_href_SecurityError", &[]);
    assert!(title_of(&state, "renderer ").ends_with(",load 2 frames 1 true doc SecurityError href SecurityError"), "{state}");
    assert!(title_of(&state, "renderer-child ").ends_with(",moved"), "{state}");

    let state = run_proxy(&srv.base, "?top", "renderer:title:arrived", &[]);
    assert!(state.contains(&format!("url {}/ui/oopif-proxy.html?arrived", srv.base)), "{state}");

    let state = run_proxy(&srv.base, "?sandboxed", "renderer:child-title:SecurityError", &[]);
    assert_eq!(title_of(&state, "renderer "), "P", "{state}");
    assert!(title_of(&state, "renderer-child ").ends_with(",nav SecurityError"), "{state}");
}

/// Q2.11: a compromised child (`DWELLER_TEST_EVIL=post-message`) that
/// claims another site's origin for its `postMessage` is killed by the
/// browser; the parent lives and gets nothing.
#[test]
fn window_renderer_oopif_proxy_forged_origin() {
    let Some(srv) = skip() else { return };
    let state = run_proxy(&srv.base, "?evil", "renderer:child-gone", &[("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "post-message")]);
    assert!(state.contains("bad message: bad post_message"), "{state}");
    assert!(state.contains("gone bad post_message"), "{state}");
    assert_eq!(title_of(&state, "renderer "), "P", "{state}");
}
