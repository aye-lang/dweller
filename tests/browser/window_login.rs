use super::*;

/// Signing in at `/login`: the username, the password, Enter.
const SIGN_IN: &str = "key:tab text:alice key:tab text:wonderland key:return frame frame";

/// Form login in one run of the window: `/account` sends a stranger to
/// the sign-in form, a wrong password shows it again, a stale CSRF
/// token is refused, the right password lands on `/account` (a POST, a
/// 303 and two cookies), signing out ends the session; a redirect chain
/// across two hosts keeps each host's cookies apart; `localStorage`
/// holds a token between pages of the run; `/basic` is 401 without
/// credentials and 200 with URL userinfo. Later cards turn on what
/// lasts across a restart (`window_login_cookies`, P4.50, and
/// `window_login_storage`, P4.51), HTTP Basic (`window_login_basic`,
/// P10.22) and popups (`window_login_popup`, P4.52).
#[test]
fn window_login() {
    window_login_in(&[IN_PROCESS]);
}

/// Q2.4a: the same sign-in runs with `DWELLER_PROCESS=tab`, every load
/// made by the browser's net for the renderer (its cookies, redirects,
/// POSTs and 401s).
#[test]
fn window_login_in_renderer() {
    window_login_in(&[("DWELLER_PROCESS", "tab")]);
}

fn window_login_in(envs: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let account = format!("{base}/account");
    // Under the flag the committed URL is the address line's (`url` is
    // the window's own page).
    let has = |got: &str, want: &[&str], not: &[&str], what: &str| {
        let committed = |w: &&str| match w.strip_prefix("url ") {
            Some(rest) if in_renderer(envs) => got.contains(&format!("address {rest}")),
            _ => got.contains(w),
        };
        assert!(
            want.iter().all(|w| committed(w)) && !not.iter().any(|n| got.contains(n)),
            "{what}: want {want:?}, not {not:?}, got:\n{got}"
        );
    };
    let stranger = login_window_with(&account, "frame", None, envs);
    has(
        &stranger,
        &[
            &format!("url {base}/login?next=/account\n"),
            "Not signed in",
        ],
        &["Signed in as"],
        "a stranger at /account",
    );
    let wrong = login_window_with(
        &account,
        "key:tab text:alice key:tab text:queen key:return frame frame",
        None,
        envs,
    );
    has(
        &wrong,
        &[&format!("url {base}/login\n"), "Wrong password"],
        &["Signed in as"],
        "a wrong password",
    );
    let stale = login_window_with(&format!("{base}/login/stale.html"), SIGN_IN, None, envs);
    has(
        &stale,
        &["bad CSRF token"],
        &["Signed in as"],
        "a stale CSRF token",
    );
    let history = "history 2 of 2\n";
    let signed_in = login_window_with(&account, SIGN_IN, None, envs);
    has(
        &signed_in,
        &[
            &format!("url {account}\n"),
            history,
            "Signed in as Alice Liddell",
            "tab cookie: yes",
        ],
        &[],
        "signing in",
    );
    // Sign out (the button after the fields), then /account again.
    let out = login_window_with(
        &account,
        &format!(
            "{SIGN_IN} key:tab key:return frame frame {}",
            go_to(&account)
        ),
        None,
        envs,
    );
    has(
        &out,
        &[
            &format!("url {base}/login?next=/account\n"),
            "Not signed in",
        ],
        &["Signed in as"],
        "signing out",
    );
    // 127.0.0.1 -> localhost -> 127.0.0.1, a cookie at each hop: back on
    // 127.0.0.1 only its own cookie comes.
    let sso = login_window_with(&format!("{base}/sso/start"), "frame", None, envs);
    has(
        &sso,
        &[
            &format!("url {base}/sso/done\n"),
            "SSO done on 127.0.0.1, cookies: sso1=127.0.0.1",
        ],
        &["sso2"],
        "the SSO redirect chain",
    );
    // HTTP Basic: the 401 page without credentials, 200 with URL
    // userinfo (libcurl sends it). The sheet and the cache are P10.22's.
    let basic = format!("{base}/basic");
    let refused = login_window_with(&basic, "frame", None, envs);
    has(
        &refused,
        &["401: sign in to realm r"],
        &["signed in as alice"],
        "/basic without credentials",
    );
    let userinfo = login_window_with(
        &basic.replace("http://", "http://alice:wonderland@"),
        "frame",
        None,
        envs,
    );
    has(
        &userinfo,
        &["Basic: signed in as alice"],
        &[],
        "/basic with URL userinfo",
    );
    // localStorage between pages of one run.
    let storage = login_window_with(
        &format!("{base}/login/store.html"),
        &format!("frame {}", go_to(&format!("{base}/login/read.html"))),
        None,
        envs,
    );
    has(
        &storage,
        &["token: tok-alice-42"],
        &[],
        "localStorage in one run",
    );
}

/// P10.23: what sign-in pages meet that Dweller lacks or handles in
/// its own way: `autocomplete` read back, `navigator.credentials`
/// refusing and no `PublicKeyCredential`, `Set-Cookie` on `fetch` and
/// XHR reaching `document.cookie`, `requestSubmit` with and without a
/// submitter (and its validation), Enter in a lone text field with no
/// button, a `submit` handler with `preventDefault` plus a `fetch`
/// login, and no password value in the agent tree or the profile.
#[test]
fn window_login_forms() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let has = |got: &str, want: &[&str], not: &[&str], what: &str| {
        assert!(
            want.iter().all(|w| got.contains(w)) && !not.iter().any(|n| got.contains(n)),
            "{what}: want {want:?}, not {not:?}, got:\n{got}"
        );
    };
    let details = login_window(&format!("{base}/login/details.html"), "frame frame frame", None);
    has(
        &details,
        &[
            "pkc undefined false",
            "autocomplete username webauthn | current-password one-time-code",
            "invalid \n",
            "badsubmitter TypeError",
            "foreign NotFoundError",
            "submits plain:null,two:null,two:b2",
            "credentials NotSupportedError",
            "fetch 200 cookie tab=1",
            "xhr 200 cookie tab=1",
            "done\n",
        ],
        &["cookie sid", "; sid=", "credentials resolved\n"],
        "sign-in details",
    );
    let lone = login_window(
        &format!("{base}/login/enter.html"),
        "key:tab text:hello key:return frame frame",
        None,
    );
    has(
        &lone,
        &[&format!("url {base}/echo?q=hello\n")],
        &[],
        "Enter in a lone text field",
    );
    let two = login_window(
        &format!("{base}/login/enter-two.html"),
        "key:tab text:x key:return frame frame",
        None,
    );
    has(
        &two,
        &[&format!("url {base}/login/enter-two.html\n")],
        &["echo"],
        "Enter with two fields and no button",
    );
    let dir = profile("login-forms");
    let fetched = login_window(
        &format!("{base}/login/fetch-login.html"),
        "key:tab text:alice key:tab text:wonderland key:return frame frame frame",
        Some(&dir),
    );
    has(
        &fetched,
        &[
            &format!("url {base}/account\n"),
            "Signed in as Alice Liddell",
            "tab cookie: yes",
        ],
        &[],
        "a submit handler with fetch",
    );
    let tree = dweller_command()
        .args(["dump", "--a11y", &format!("{base}/login/details.html")])
        .output()
        .unwrap();
    has(&text(&tree.stdout), &["******"], &["s3cret"], "the agent tree");
    for entry in std::fs::read_dir(&dir).unwrap() {
        let bytes = std::fs::read(entry.unwrap().path()).unwrap_or_default();
        assert!(
            !String::from_utf8_lossy(&bytes).contains("wonderland"),
            "the password is in the profile"
        );
    }
}

/// P4.50: signed in at `/login`, the window is killed (no clean exit);
/// started again on the same profile it is still signed in, the
/// session-only cookie restored too (a browser that restores its
/// session).
#[test]
fn window_login_cookies() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let account = format!("{base}/account");
    let dir = profile("login-cookies");
    let mut window = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROFILE", &dir)
        .args([
            account.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "16384",
            "--events",
            SIGN_IN,
        ])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();
    // Cookies are saved within a second of a change (P4.50).
    let jar = dir.join("cookies");
    let start = Instant::now();
    while !std::fs::read_to_string(&jar).is_ok_and(|s| s.contains("sid")) {
        if start.elapsed() > Duration::from_secs(30) || window.try_wait().unwrap().is_some() {
            let _ = window.kill();
            panic!("no sid cookie in {} after signing in", jar.display());
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    window.kill().unwrap();
    window.wait().unwrap();
    // Only the person can read the jar; no temporary file is left.
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&jar).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "{}", jar.display());
    }
    let mut names: Vec<String> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["cookies", "lock", "telemetry"]);
    let again = login_window(&account, "frame", Some(&dir));
    assert!(
        again.contains(&format!("url {account}\n"))
            && again.contains("Signed in as Alice Liddell")
            && again.contains("tab cookie: yes"),
        "after the restart:\n{again}"
    );
    // Signing out is saved too: the next start is signed out.
    login_window(&account, "key:tab key:return frame frame", Some(&dir));
    let out = login_window(&account, "frame", Some(&dir));
    assert!(out.contains("Not signed in"), "after signing out:\n{out}");
}

/// P4.51: a token in `localStorage` is there after a restart on the same
/// profile; `sessionStorage` lasts across a navigation in the tab, not
/// across a restart.
#[test]
fn window_login_storage() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let read = format!("{base}/login/read.html");
    let dir = profile("login-storage");
    let first = login_window(
        &format!("{base}/login/store.html"),
        &format!("frame {}", go_to(&read)),
        Some(&dir),
    );
    assert!(
        first.contains("token: tok-alice-42") && first.contains("session: kept"),
        "in the first run:\n{first}"
    );
    let again = login_window(&read, "frame", Some(&dir));
    assert!(
        again.contains("token: tok-alice-42") && again.contains("session: null"),
        "after the restart:\n{again}"
    );
    // One file for the origin, mode 0600, no temporary file left.
    let files: Vec<_> = std::fs::read_dir(dir.join("storage"))
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    assert_eq!(files.len(), 1, "{files:?}");
    let saved = std::fs::read_to_string(&files[0]).unwrap();
    assert!(saved.contains("5 12\ntokentok-alice-42\n"), "{saved:?}");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = std::fs::metadata(&files[0]).unwrap().permissions().mode();
        assert_eq!(mode & 0o777, 0o600, "{:?}", files[0]);
    }
}

/// P10.22: credentials for `/basic` (here from URL userinfo) are cached
/// per origin and realm for the process: the same page without them, in
/// the same run, is 200 too.
#[test]
fn window_login_basic() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let basic = format!("{}/basic", srv.base);
    let url = basic.replace("http://", "http://alice:wonderland@");
    let got = login_window(&url, &format!("frame {}", go_to(&basic)), None);
    assert!(
        got.contains(&format!("url {basic}\n")) && got.contains("Basic: signed in as alice"),
        "{got}"
    );
}

/// P10.22: the sign-in prompt of a 401 with `WWW-Authenticate: Basic` in
/// the person's window (ui/auth.aye): typing the username, Tab, the
/// password and Enter loads `/basic` again with the credentials; a wrong
/// password brings the prompt back with the username kept and the focus
/// on the password. The prompt is `browser/tests/ui/auth_prompt.png`
/// (`DWELLER_BLESS=1` rewrites it; the fixed font draws every digit of
/// the server's port alike).
#[test]
fn window_login_basic_prompt() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let basic = format!("{}/basic", srv.base);
    let asked = login_window(&basic, "frame", None);
    assert!(
        asked.contains("focus auth_user\n")
            && asked.contains("auth r user \n")
            && asked.contains("401: sign in to realm r"),
        "{asked}"
    );
    let signed = login_window(
        &basic,
        "frame text:alice key:tab text:wonderland key:return frame frame",
        None,
    );
    assert!(
        signed.contains(&format!("url {basic}\nscroll 0\nfocus page\n"))
            && signed.contains("history 1 of 1\n")
            && signed.contains("Basic: signed in as alice")
            && !signed.contains("\nauth "),
        "{signed}"
    );
    let wrong = login_window(
        &basic,
        "frame text:alice key:tab text:queen key:return frame frame",
        None,
    );
    assert!(
        wrong.contains("focus auth_password\n")
            && wrong.contains("auth r user alice\n")
            && wrong.contains("401: sign in to realm r"),
        "{wrong}"
    );
    let again = login_window(
        &basic,
        "frame text:alice key:tab text:queen key:return frame frame \
         text:wonderland key:return frame frame",
        None,
    );
    assert!(again.contains("Basic: signed in as alice"), "{again}");
    // While the prompt is open a click on the page or a key for the
    // window leaves the focus in it.
    let held = login_window(&basic, "frame click:200,280 key:cmd+l text:al", None);
    assert!(
        held.contains("focus auth_user\n") && held.contains("auth r user al\n"),
        "{held}"
    );
    let png = scratch().join("auth_prompt.png");
    let out = dweller_command()
        .current_dir(root())
        .args([
            &basic,
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "400",
            "--until-idle",
            "--events",
            "frame text:alice key:tab text:wonder",
            "--save",
            png.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    let golden = root().join("browser/tests/ui/auth_prompt.png");
    let got = std::fs::read(&png).unwrap();
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
    } else {
        assert!(
            got == std::fs::read(&golden).unwrap(),
            "the sign-in prompt differs from {}: {}",
            golden.display(),
            png.display()
        );
    }
}

/// P10.22: Escape or Cancel closes the sign-in prompt and leaves the 401
/// page shown, the focus on the page; Sign in (a click) signs in. With
/// URL userinfo the address bar leaves `user:pass@` out.
#[test]
fn window_login_basic_cancel() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let basic = format!("{}/basic", srv.base);
    // In a 400 px window the sheet is x 30-370 from y 39; Cancel is at
    // x 206-278 and Sign in at 298-370, y 153-177.
    for events in [
        "frame text:alice key:escape frame",
        "frame text:alice click:240,165 frame",
    ] {
        let got = login_window(&basic, events, None);
        assert!(
            got.contains("focus page\n")
                && !got.contains("\nauth ")
                && got.contains("401: sign in to realm r"),
            "{events}: {got}"
        );
    }
    let clicked = login_window(
        &basic,
        "frame text:alice key:tab text:wonderland click:330,165 frame frame",
        None,
    );
    assert!(clicked.contains("Basic: signed in as alice"), "{clicked}");
    let userinfo = basic.replace("http://", "http://alice:wonderland@");
    let shown = login_window(&userinfo, "frame", None);
    assert!(
        shown.contains(&format!("url {userinfo}\n"))
            && shown.contains(&format!("address {basic}\n"))
            && shown.contains("Basic: signed in as alice"),
        "{shown}"
    );
}

/// P4.52: window.open, `target=_blank`, `window.opener` and postMessage
/// between tabs (engine/contexts.aye), on the test server's two origins
/// (`127.0.0.1` and `localhost`, `browser/tests/popup/`). The OAuth
/// popup round trip: a click opens the sign-in page of origin B in a
/// new tab, which sees its opener but not its document, and goes through
/// a redirect back to the opener's origin; the callback posts the code
/// to its opener (a message for the wrong origin is dropped) and closes
/// itself, so the opener's tab is shown with the message, its
/// `popup.closed` true and `event.source` the popup. `dump --js` loads
/// the popup in memory with the same result. `target=_blank` opens a tab
/// without an opener (with one for `rel=opener`); a named target is
/// reused by window.open and by a link. First the P10.20 fixture: a
/// click opens `login/popup.html`, which posts to its opener and closes,
/// and the opener shows the message.
#[test]
fn window_login_popup() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let opener = format!("{base}/login/opener.html");
    // The button is at y 10 to 40 of the page, under the 40 px toolbar.
    let got = login_window(&opener, "frame click:50,65 frame frame frame", None);
    assert!(
        got.contains(&format!("url {opener}\n")) && got.contains("message: alice"),
        "{got}"
    );
    let signed_in = "got code=c0de state=s1 seen=opener-true-SecurityError-signin from the popup closed true messages 1";
    let window = |page: &str, events: &str| {
        let url = format!("{base}/popup/{page}");
        let out = run_window(&[
            &url, "--font", "fixed", "--width", "400", "--height", "300", "--frames", "300",
            "--until-idle", "--events", events, "--state", "--text",
        ]);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{page} {events}: {}\n{}",
            out.status,
            text(&out.stderr)
        );
        text(&out.stdout)
    };
    let got = window(
        "opener.html",
        "frame click:50,60 frame frame frame frame frame",
    );
    assert!(
        got.starts_with(&format!("url {base}/popup/opener.html\n")),
        "{got}"
    );
    assert!(!got.contains("\ntabs "), "the popup did not close:\n{got}");
    assert!(got.contains(signed_in), "{got}");
    // Without a window: the popup loads in memory.
    let out = run(&[
        "dump",
        "--js",
        "--text",
        &format!("{base}/popup/opener.html#auto"),
    ]);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}",
        text(&out.stderr)
    );
    assert!(
        text(&out.stdout).contains(signed_in),
        "{}",
        text(&out.stdout)
    );
    // target=_blank: a new tab, shown, without an opener; rel=opener
    // gives it one.
    for (y, search, opener) in [(60, "blank", "null"), (110, "opener", "set")] {
        let got = window("links.html", &format!("frame click:50,{y} frame frame"));
        assert!(
            got.starts_with(&format!("url {base}/popup/target.html?{search}\n")),
            "{got}"
        );
        assert!(got.contains("\ntabs 2, showing 2\n"), "{got}");
        assert!(
            got.contains(&format!("opener {opener} name  at ?{search}")),
            "{got}"
        );
    }
    // window.open(url, 'w1') twice, then a link with target=w1: one tab
    // (the tab strip moves the page down 28 px once there are two).
    let got = window(
        "links.html",
        "frame click:50,160 frame key:cmd+1 frame click:50,190 frame key:cmd+1 frame click:50,240 frame key:cmd+2 frame",
    );
    assert!(
        got.starts_with(&format!("url {base}/popup/target.html?link\n")),
        "{got}"
    );
    assert!(got.contains("\nhistory 3 of 3\n"), "{got}");
    assert!(got.contains("\ntabs 2, showing 2\n"), "{got}");
    assert!(got.contains("opener set name w1 at ?link"), "{got}");
}
