use super::*;

/// Content-Security-Policy `script-src` (Q3.7), the pages of
/// `browser/tests/csp/` served with the policies of their `.policy`
/// files: an external script the policy does not allow is not fetched,
/// fires `error` and logs the directive; `'self'` and a host source
/// allow theirs; a `<meta>` policy applies; two policies must both allow
/// (the second through `default-src`). Inline scripts run with the
/// policy's nonce or sha256 hash, and `'unsafe-inline'` is ignored next
/// to them. `eval` and `new Function` throw EvalError without
/// `'unsafe-eval'`.
#[test]
fn csp_script_src() {
    let (Some(srv), Some(other)) = (server(), server()) else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let refused = |src: &str, directive: &str| {
        format!(
            "[error] Refused to load the script '{src}' because it violates the following Content Security Policy directive: \"{directive}\"."
        )
    };
    let (parsed, same, cross) = (
        format!("{}/csp/parsed.js", srv.base),
        format!("{}/csp/same.js", srv.base),
        format!("{}/csp/cross.js", other.base),
    );
    let host = format!("script-src 'nonce-r' {}", other.base);
    let inline = "[error] Refused to execute inline script because it violates the following Content Security Policy directive: \"script-src 'nonce-r' 'sha256-PNUJduEfysGozdHcUzEt/SHPlhpzZRXdDLB9QVvFYEk=' 'unsafe-inline'\".".to_string();
    let cases = [
        (
            "self",
            "parsed: ran true\nsame: load, ran true\ncross: error, ran false\ndone\n",
            vec![refused(&cross, "script-src 'self' 'nonce-r'")],
        ),
        (
            "host",
            "parsed: ran false\nsame: error, ran false\ncross: load, ran true\ndone\n",
            vec![refused(&parsed, &host), refused(&same, &host)],
        ),
        (
            "two",
            "parsed: ran true\nsame: load, ran true\ncross: error, ran false\ndone\n",
            vec![refused(&cross, "default-src 'self' 'nonce-r'")],
        ),
        (
            "meta",
            "parsed: ran false\nsame: error, ran false\ncross: error, ran false\ndone\n",
            vec![
                refused(&parsed, "script-src 'nonce-r'"),
                refused(&same, "script-src 'nonce-r'"),
                refused(&cross, "script-src 'nonce-r'"),
            ],
        ),
        ("inline", "nonce match: ran\nhash match: ran\ndone\n", vec![inline.clone(), inline]),
        ("eval", "eval: EvalError\nFunction: EvalError\n", vec![]),
        ("unsafe-eval", "eval: 2\nFunction: 2\n", vec![]),
    ];
    for (page, want, errors) in cases {
        let out = run(&[
            "dump",
            "--text",
            "--js",
            &format!("{}/csp/{page}.html?other={}", srv.base, other.base),
        ]);
        assert!(out.status.success(), "{page}: {}\n{}", out.status, text(&out.stderr));
        assert_eq!(text(&out.stdout), want, "{page}: {}", text(&out.stderr));
        let logged: Vec<String> = text(&out.stderr)
            .lines()
            .filter(|l| l.starts_with("[error] "))
            .map(String::from)
            .collect();
        assert_eq!(logged, errors, "{page}");
    }
}

/// Q3.6: a document refused by `X-Frame-Options` or CSP
/// `frame-ancestors` shows an error document of an opaque origin (no
/// `contentDocument`, the `load` event still fires) and logs a console
/// error; a cross-origin frame has no `contentDocument` either, so the
/// log tells the refused ones from the allowed. `DENY`, `SAMEORIGIN` against every ancestor, conflicting
/// values, `frame-ancestors` `'none'` and `'self'`, the CSP policy
/// overriding XFO, a `<meta>` policy and a report-only one not blocking.
#[test]
fn csp_frame_ancestors() {
    let (Some(srv), Some(other)) = (server(), server()) else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&[
        "dump",
        "--text",
        "--js",
        &format!("{}/csp/framing.html?other={}", srv.base, other.base),
    ]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let mut got: Vec<String> = text(&out.stdout).lines().map(String::from).collect();
    got.sort();
    let want = [
        "conflict cross: no document",
        "conflict same: no document",
        "deny cross: no document",
        "deny same: no document",
        "meta cross: no document",
        "meta same: document",
        "none cross: no document",
        "none same: no document",
        "override cross: no document",
        "override same: document",
        "report cross: no document",
        "report same: document",
        "sameorigin cross: no document",
        "sameorigin same: document",
        "self cross: no document",
        "self same: document",
    ];
    assert_eq!(got, want, "{}", text(&out.stderr));
    let logged = |needle: &str| text(&out.stderr).lines().filter(|l| l.contains(needle)).count();
    assert_eq!(logged("to 'deny'"), 2, "{}", text(&out.stderr));
    assert_eq!(logged("to 'sameorigin'"), 1, "{}", text(&out.stderr));
    assert_eq!(logged("conflicting values"), 2, "{}", text(&out.stderr));
    assert_eq!(logged("directive: \"frame-ancestors 'none'\""), 2, "{}", text(&out.stderr));
    assert_eq!(logged("directive: \"frame-ancestors 'self'\""), 2, "{}", text(&out.stderr));
}

/// Q3.9b: a person's click on a `target=_top` link or form in a
/// sandboxed frame loads in the tab only with `allow-top-navigation` or
/// `allow-top-navigation-by-user-activation`.
#[test]
fn window_sandbox_targets() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let ui = format!("file://{}/browser/tests/ui/", root().display());
    let cases = [
        ("click:10,60", "url sandbox_targets.html\n"),
        ("click:10,100", "url other.html\n"),
        ("click:10,140", "url sandbox_targets.html\n"),
        ("click:10,180", "url other.html?\n"),
    ];
    let mut failures = Vec::new();
    for (events, want) in cases {
        let out = run_window(&[
            "browser/tests/ui/sandbox_targets.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "340",
            "--frames",
            "1",
            "--state",
            "--events",
            events,
        ]);
        let got = text(&out.stdout).replace(&ui, "");
        if !out.status.success() || !out.stderr.is_empty() || !got.starts_with(want) {
            failures.push(format!("--events {events:?}: expected {want:?}:\n{got}{}", text(&out.stderr)));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Q3.1: a web page cannot read or show a local file
/// (`browser/tests/no-file/`, net/initiator-may-load.aye). From an http
/// page, fetch, XHR, img, script, a style sheet and an iframe of a
/// `file:` URL fail as network errors, and `location`, `window.open` and
/// a link click to one show no byte of it. The same pages as `file:`
/// pages load and show it all, and the browser opens a `file:` URL it
/// is given.
#[test]
fn no_file_from_web() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let web = format!("{}/no-file/", srv.base);
    let local = format!("file://{}/browser/tests/no-file/", root().display());
    let secret = "LOCAL SECRET";
    let loads = |dir: &str| {
        let out = run(&["dump", "--js", "--text", &format!("{dir}web.html?{local}")]);
        assert!(out.status.success(), "{dir}: {}", text(&out.stderr));
        text(&out.stdout).lines().next().unwrap_or("").to_string()
    };
    assert_eq!(
        loads(&web),
        "fetch TypeError, iframe load, img error, script error, style rgb(0, 0, 0), xhr error"
    );
    assert_eq!(
        loads(&local),
        "fetch read secret = 'LOCAL SECRET', framed LOCAL SECRET, iframe load, img load, script load, style rgb(255, 0, 0), xhr read secret = 'LOCAL SECRET'"
    );
    let goes = |dir: &str, how: &str, events: &str| {
        let out = run_window(&[
            &format!("{dir}go.html?{how}&{local}"),
            "--font", "fixed", "--width", "400", "--height", "300", "--frames", "300",
            "--until-idle", "--events", events, "--state", "--text",
        ]);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{dir} {how}: {}\n{}",
            out.status,
            text(&out.stderr)
        );
        text(&out.stdout)
    };
    for (how, events) in [
        ("location", "frame frame frame"),
        ("open", "frame frame frame"),
        ("click", "frame click:12,50 frame frame frame"),
    ] {
        let got = goes(&web, how, events);
        assert!(!got.contains(secret), "{how} from the web:\n{got}");
        let got = goes(&local, how, events);
        assert!(
            got.starts_with(&format!("url {local}secret.html\n")) && got.contains(secret),
            "{how} from a file:\n{got}"
        );
    }
}
