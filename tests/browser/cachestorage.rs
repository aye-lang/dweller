use super::*;

const CACHES: &str = "empty 0 false false | names v1,v2 true | match 201 Made a first again first | miss undefined ignoreSearch first | vary two undefined 2 | added plain text from the server | keys made.txt?a=1,varied,varied,plain.txt | replaced second 4 | errors TypeError TypeError TypeError TypeError  | storage match one undefined | delete true false 3 true v2 | databases 0";

/// Q5.14b: `caches.open` makes a cache, `put` and `add` store
/// responses, `match` finds them by URL (with `ignoreSearch`) and by
/// the request headers `Vary` names (unless `ignoreVary`), a second
/// `put` replaces, a POST request, a 206, `Vary: *` and a failed fetch
/// are TypeErrors, `caches.match` searches the caches in order, and the
/// reserved database is not listed by `indexedDB.databases()`.
#[test]
fn cachestorage() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    expect(&["dump", "--text", "--js", &format!("{}/cachestorage/caches.html", srv.base)], 0, &format!("{CACHES}\n"), "");
}

/// Q5.14b: the same with `DWELLER_PROCESS=tab` and `site`: the caches
/// are in the browser process.
#[test]
fn window_renderer_cachestorage() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    for mode in ["tab", "site"] {
        let got = login_window_with(&format!("{}/cachestorage/caches.html", srv.base), "renderer:title:done", None, &[("DWELLER_PROCESS", mode)]);
        assert!(got.contains(CACHES), "{mode}: {got}");
    }
}
