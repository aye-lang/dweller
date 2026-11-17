use super::*;

const REGISTERED: &[&str] = &[
    "before true undefined | registered /serviceworker/ worker.js installing truetrue | ready true true | states installed,activating,activated",
    "controller true | worker says {\"seen\":\"install,activate\",\"scope\":\"/serviceworker/\",\"isWorker\":true} | same true true list 1 | worker.js SecurityError | plain.txt SecurityError | missing.js TypeError | wide scope SecurityError | unregister true false undefined",
];
const FETCHED: &str = "made 201 worker made by the worker for POST  | proxied: plain text from the server | plain plain text from the server | broken TypeError | xhr 201 made by the worker for GET  | clients fetch.html";
const NAVIGATED: &str = "a page made by the worker, controlled true, navigate document, made by the worker for GET document";

fn has_all(got: &str, want: &[&str], what: &str) {
    assert!(want.iter().all(|w| got.contains(w)), "{what}: want {want:?}, got:\n{got}");
}

fn dumped(base: &str, page: &str, dir: Option<&Path>) -> String {
    let mut c = dweller_command();
    c.current_dir(root());
    if let Some(d) = dir {
        c.env("DWELLER_PROFILE", d);
    }
    let out = c.args(["dump", "--text", "--js", &format!("{base}/serviceworker/{page}")]).output().unwrap();
    assert_eq!(text(&out.stderr), "", "{page}");
    text(&out.stdout)
}

/// Q5.14a: `register` fetches the script, the worker gets `install` and
/// `activate` (each waits for `waitUntil`), its states reach the page as
/// `statechange`, `ready` resolves, `clients.claim()` makes it the
/// page's controller, messages go both ways, a second `register` of the
/// same script is the same registration, a cross-origin script, a
/// script that is not JavaScript, a missing script and a scope above
/// the script's directory are refused, and `unregister` removes it.
#[test]
fn serviceworker_register() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    has_all(&dumped(&srv.base, "register.html", None), REGISTERED, "register");
}

/// Q5.14a: a controlled page's `fetch` and XHR go to the worker's
/// `fetch` event: `respondWith` answers with a made response or with
/// what the worker fetched itself, a rejected promise is a network
/// error, and no `respondWith` falls through to the network.
#[test]
fn serviceworker_fetch() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    has_all(&dumped(&srv.base, "fetch.html", None), &["claimed | ", FETCHED], "fetch");
}

/// Q5.14a: with a profile the registration is saved, and the next run's
/// navigation into its scope is controlled from the start.
#[test]
fn serviceworker_profile() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let dir = profile("serviceworker_profile");
    has_all(&dumped(&srv.base, "fetch.html", Some(&dir)), &["claimed | ", FETCHED], "first run");
    has_all(&dumped(&srv.base, "fetch.html", Some(&dir)), &["controlled at load | ", FETCHED], "second run");
}

/// Q5.14a: in the window a navigation into the worker's scope is
/// answered by its `fetch` event and the new page is controlled; with
/// `DWELLER_PROCESS=tab` and `site` the registry is the browser's and
/// the worker runs in the page's renderer, with the same results.
#[test]
fn window_renderer_serviceworker() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let at = |page: &str| format!("{}/serviceworker/{page}", srv.base);
    let frames = vec!["frame"; 40].join(" ");
    has_all(&login_window_with(&at("navigate.html"), &frames, None, &[]), &[NAVIGATED], "one process");
    for mode in ["tab", "site"] {
        let envs = [("DWELLER_PROCESS", mode)];
        let shown = |page: &str| login_window_with(&at(page), "renderer:title:done", None, &envs);
        has_all(&shown("register.html"), REGISTERED, mode);
        has_all(&shown("fetch.html"), &["claimed | ", FETCHED], mode);
        has_all(&shown("navigate.html"), &[NAVIGATED], mode);
    }
}
