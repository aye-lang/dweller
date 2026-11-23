use super::*;

/// Q3.15b: `document.domain` can be set to the document's own host only,
/// and never in a sandboxed frame; `document.write` into a frame whose
/// parser ended (by its parent or by itself) makes a new document in the
/// frame's own engine that keeps the frame's URL and sandbox flags.
#[test]
fn sandbox_document_domain_and_written_documents() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let want = "domain: 127.0.0.1\n\
        own host: set\n\
        other host: SecurityError\n\
        empty: SecurityError\n\
        plain srcdoc: set at about:srcdoc\n\
        sandboxed srcdoc: SecurityError at about:srcdoc\n\
        written by the parent: SecurityError at blank.html\n\
        written plain: set at blank.html\n\
        written by itself: SecurityError at about:srcdoc\n";
    for mode in ["single", "site"] {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_PROCESS", mode)
            .args([
                &format!("{}/sandbox/document_domain.html", srv.base),
                "--frames", "300", "--until-idle", "--text",
            ])
            .output()
            .unwrap();
        assert!(out.status.success(), "{mode}: {}\n{}", out.status, text(&out.stderr));
        let got = text(&out.stdout);
        assert!(got.contains(want), "{mode}:\n{got}");
    }
}

/// Q3.15b: a popup that a frame without `allow-same-origin` opened does
/// not give it its document; a popup's `origin` is its opener's until
/// it loads; a `location` set on a popup before it loaded is followed;
/// a script's `click()` on a link of another frame's document follows it.
#[test]
fn sandbox_popups_of_frames() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&["dump", "--text", "--js", &format!("{}/sandbox/popups.html", srv.base)]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let got = text(&out.stdout);
    for want in [
        "document of the popup of an opaque frame: SecurityError\n",
        "origin before the popup loads is the opener's: true\n",
        "navigated before it loaded: Blank\n",
        "popup of a link the parent clicked: its scripts ran\n",
    ] {
        assert!(got.contains(want), "no {want:?} in:\n{got}\n{}", text(&out.stderr));
    }
}

/// Q3.15c: a srcdoc frame has its parent's origin as `self.origin`, so
/// its same-origin worker script loads, and the worker's messages reach
/// the frame; a module worker whose import cannot load gets `error`.
#[test]
fn sandbox_workers_of_frames() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = run(&["dump", "--text", "--js", &format!("{}/sandbox/frame_workers.html", srv.base)]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let got = text(&out.stdout);
    for want in [
        "origin of a srcdoc frame is its parent's: true\n",
        "worker of a srcdoc frame: ran\n",
        "module worker with a missing import: error\n",
    ] {
        assert!(got.contains(want), "no {want:?} in:\n{got}\n{}", text(&out.stderr));
    }
}

/// Q2.14h row 26: the Linux sandbox answers `stat` itself (a `SIGSYS`
/// handler opens the path read-only, so Landlock decides): the profile
/// stays refused and `/usr` can be looked at. Without Landlock seccomp
/// refuses both, as before.
#[test]
fn sandbox_stat_follows_landlock() {
    if !cfg!(all(target_os = "linux", target_arch = "x86_64")) || !has_sdl() {
        println!("skipped: the Linux x86_64 sandbox, with SDL2");
        return;
    }
    let home = scratch().join("sandbox-stat-home");
    std::fs::create_dir_all(home.join(".dweller")).unwrap();
    let run = |envs: &[(&str, &str)]| -> String {
        let out = dweller_command()
            .current_dir(root())
            .env("HOME", &home)
            .env("DWELLER_PROCESS", "site")
            .env("DWELLER_TEST", "1")
            .env("DWELLER_TEST_SANDBOX_STAT", "1")
            .envs(envs.iter().copied())
            .args(["browser/tests/ui/form.html", "--frames", "100", "--until-idle", "--events", "renderer:loaded"])
            .output()
            .unwrap();
        assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
        text(&out.stderr)
    };
    let err = run(&[]);
    assert!(err == "sandbox-stat home=13 usr=0\n" || err == "sandbox-stat home=1 usr=1\n", "{err}");
    assert_eq!(run(&[("DWELLER_TEST_NO_LANDLOCK", "1")]), "sandbox-stat home=1 usr=1\n");
}

fn javascript_url_page(page: &str) -> String {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return String::new();
    };
    let out = run(&["dump", "--text", "--js", &format!("{}/sandbox/{page}", srv.base)]);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    text(&out.stdout)
}

/// `window.open("javascript:...", name)` runs the script in the named
/// popup when the opener is of its origin, and a string result becomes
/// the popup's document, with the popup's URL and origin (an
/// `about:blank` popup has its opener's); a popup of another origin, or a
/// sandboxed frame of an opaque origin, runs nothing, and a result that is
/// not a string keeps the document. In
/// `dump` and in the window with renderers.
#[test]
fn javascript_url_in_named_popup() {
    let got = javascript_url_page("javascript_url.html");
    if got.is_empty() {
        return;
    }
    let Some(srv) = server() else {
        return;
    };
    let window = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "site")
        .args([&format!("{}/sandbox/javascript_url.html", srv.base), "--frames", "300", "--until-idle", "--text"])
        .output()
        .unwrap();
    assert!(window.status.success(), "{}\n{}", window.status, text(&window.stderr));
    for got in [got, text(&window.stdout)] {
        for want in ["same origin: about:blank OPENER\n", "a number keeps the document: true\n", "done\n"] {
            assert!(got.contains(want), "no {want:?} in:\n{got}");
        }
        assert!(!got.contains("cross origin:") && !got.contains("opaque frame:"), "{got}");
    }
}

/// A page whose CSP has no `'unsafe-inline'` cannot run a `javascript:`
/// URL in its popup or in a frame's `src`: the page gets a violation of
/// `script-src-elem` for each, and the frame stays empty.
#[test]
fn javascript_url_refused_by_csp() {
    let got = javascript_url_page("javascript_url_csp.html");
    if got.is_empty() {
        return;
    }
    assert!(got.contains("violation: script-src-elem inline\n"), "{got}");
    assert!(got.contains("frame: \"\"\n"), "the frame stays empty:\n{got}");
    assert!(got.contains("done\n") && !got.contains("ran"), "{got}");
}

/// A popup opened on `about:blank` shows an empty document, in the window
/// with one process too, where it showed the error page "this kind of
/// URL is not supported yet"; its opener can read its document.
#[test]
fn blank_popup_is_an_empty_document() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "single")
        .args([&format!("{}/sandbox/blank_popup.html", srv.base), "--frames", "300", "--until-idle", "--state", "--text"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let got = text(&out.stdout);
    assert!(got.contains("url about:blank\n") && got.contains("tabs 2, showing 2\n"), "{got}");
    assert!(!got.contains("Cannot load"), "{got}");
    let dump = run(&["dump", "--text", "--js", &format!("{}/sandbox/blank_popup.html", srv.base)]);
    assert!(dump.status.success(), "{}\n{}", dump.status, text(&dump.stderr));
    let got = text(&dump.stdout);
    assert!(got.contains("popup body \"\""), "{got}");
}

/// A frame whose `src` is a `javascript:` URL shows the string the URL
/// returns, and so does a frame that sets its own `location` to one; a
/// popup that sets its own `location` to one runs it, and shows the string
/// it returns. In the window with
/// one process (the popup's tab is shown last: `cmd+1` shows the opener).
#[test]
fn javascript_url_of_frames_and_location() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let frames = "frame ".repeat(30);
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "single")
        .args([
            &format!("{}/sandbox/javascript_url_frames.html", srv.base),
            "--frames", "300", "--until-idle", "--text", "--events", &format!("{frames}key:cmd+1 frame"),
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let got = text(&out.stdout);
    for want in ["the popup ran its own location\n", "src frame: from the src\n", "self frame: navigated itself\n", "popup: its own string\n", "done\n"] {
        assert!(got.contains(want), "no {want:?} in:\n{got}");
    }
}

/// A click on a link whose `href` is a `javascript:` URL runs it in the
/// link's page (a link the page's script clicks, in `dump` and in the
/// window with renderers); before, the click did nothing.
#[test]
fn javascript_url_link_runs() {
    let got = javascript_url_page("javascript_url_link.html");
    if got.is_empty() {
        return;
    }
    assert!(got.contains("the link ran\ndone\n"), "{got}");
    let Some(srv) = server() else {
        return;
    };
    let window = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "site")
        .args([&format!("{}/sandbox/javascript_url_link.html", srv.base), "--frames", "100", "--until-idle", "--text"])
        .output()
        .unwrap();
    assert!(window.status.success(), "{}\n{}", window.status, text(&window.stderr));
    assert!(text(&window.stdout).contains("the link ran\ndone\n"), "{}", text(&window.stdout));
}
