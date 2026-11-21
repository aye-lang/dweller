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
