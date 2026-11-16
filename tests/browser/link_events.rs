use super::*;

fn dump_link_page(path: &str, golden: &str) -> Vec<String> {
    let (Some(srv), Some(other)) = (server(), server()) else {
        println!("skipped: cannot start python3 for the test server");
        return Vec::new();
    };
    let out = run(&["dump", "--text", "--js", &format!("{}/{path}?other={}", srv.base, other.base)]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let mut failures = Vec::new();
    check_golden(&root().join("browser/tests").join(golden), &text(&out.stdout), golden, &mut failures);
    failures
}

/// Q3.10c: `<link>` elements fire `load` and `error`: a parsed and an
/// inserted style sheet (from the network and from the cache, empty, a
/// 404, a network error, nosniff, CORS), `preload`, `modulepreload` and
/// `prefetch`, none for `icon`, a refire when `href` changes or the link
/// is inserted again, and `sheet` null until the load. A CSP `style-src`
/// refusal is an `error`. (`net/link-events.html` must print
/// `net/link-events.txt`, `csp/link-events.html` `csp/link-events.txt`.)
#[test]
fn link_events() {
    let mut failures = dump_link_page("net/link-events.html", "net/link-events.txt");
    failures.extend(dump_link_page("csp/link-events.html", "csp/link-events.txt"));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Q3.10d: two links and an inserted one to one `no-store` sheet, and a
/// second style pass, fetch it once. (`net/link-shared.html` must print
/// `net/link-shared.txt`.)
#[test]
fn link_sheets_shared() {
    let failures = dump_link_page("net/link-shared.html", "net/link-shared.txt");
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Q3.10c: the same events with `DWELLER_PROCESS=tab`, where the sheet
/// loads cross IPC. Q3.10d: the window waits for the page to set its
/// title to `... done` instead of for a number of frames.
#[test]
fn link_events_in_renderer() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let (Some(srv), Some(other)) = (server(), server()) else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let href = format!("{}/net/link-events.html?other={}", srv.base, other.base);
    let want = std::fs::read_to_string(root().join("browser/tests/net/link-events.txt")).unwrap();
    for (events, envs) in [("", &[IN_PROCESS][..]), ("renderer:title:done", &[("DWELLER_PROCESS", "tab")][..])] {
        let got = login_window_with(&href, events, None, envs);
        for line in want.lines().filter(|l| l.contains(": ")) {
            assert!(got.contains(line), "{envs:?}: missing {line:?} in\n{got}");
        }
    }
}
