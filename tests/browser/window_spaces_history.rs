use super::*;

// The ref of the button named `name` in an answer's snapshot.
fn button_ref(answer: &str, name: &str) -> String {
    let at = answer
        .find(&format!("button \\\"{name}\\\" ["))
        .unwrap_or_else(|| panic!("no button {name} in {answer}"));
    let rest = &answer[at..];
    rest[rest.find('[').unwrap() + 1..rest.find(']').unwrap()].to_string()
}

fn click(conn: &mut AgentConn, page: &str, name: &str, mode: &str) {
    let (got, _) = conn.ask(&format!(r#"{{"id":2,"cmd":"click","ref":"{}"}}"#, button_ref(page, name)));
    assert!(got.contains("\"ok\":true"), "{mode}: click {name}: {got}");
}

// `location.href history.length n` of the agent's page: `n` of
// `history.state` (`undefined` for `{}`), or `null`.
fn page_place(conn: &mut AgentConn) -> String {
    let expr = "location.href + ' ' + history.length + ' ' + (history.state && history.state.n)";
    let (got, _) = conn.ask(&format!(r#"{{"id":3,"cmd":"eval","expression":"{expr}"}}"#));
    let start = got.find("\"result\":\"").unwrap_or_else(|| panic!("{got}")) + 10;
    got[start..start + got[start..].find('"').unwrap()].to_string()
}

fn window_spaces_history_with(base: &str, mode: &str) {
    let sock = format!("target/sockets/spaces-history-{}-{mode}.sock", std::process::id());
    let window = spaces_window_env(&sock, &[], &[("DWELLER_PROCESS", mode)]);
    let mut a = AgentConn::open(&sock);
    let first = format!("{base}/ui/agent_push_state.html");
    let other = first.replace("127.0.0.1", "localhost");
    let next = other.replace("agent_push_state", "renderer_history");

    let (page, _) = a.ask(&format!(r#"{{"id":1,"cmd":"navigate","url":"{first}"}}"#));
    click(&mut a, &page, "push", mode);
    assert_eq!(page_place(&mut a), format!("{first}?b 2 undefined"), "{mode}: pushState");
    click(&mut a, &page, "back", mode);
    assert_eq!(page_place(&mut a), format!("{first} 2 null"), "{mode}: history.back()");
    click(&mut a, &page, "again", mode);
    assert_eq!(page_place(&mut a), format!("{first}?c 2 2"), "{mode}: a second pushState");

    let (page, _) = a.ask(&format!(r#"{{"id":4,"cmd":"navigate","url":"{other}"}}"#));
    click(&mut a, &page, "push", mode);
    assert_eq!(page_place(&mut a), format!("{other}?b 4 undefined"), "{mode}: pushState after a cross-site navigation");
    let (got, _) = a.ask(&format!(r#"{{"id":5,"cmd":"navigate","url":"{next}"}}"#));
    assert!(got.contains("\"ok\":true"), "{mode}: {got}");
    let (page, _) = a.ask(r#"{"id":6,"cmd":"back"}"#);
    assert_eq!(page_place(&mut a), format!("{other}?b 5 undefined"), "{mode}: the agent's back: {page}");
    click(&mut a, &page, "back", mode);
    assert_eq!(page_place(&mut a), format!("{other} 5 null"), "{mode}: history.back() in the document the agent went back to");

    drop(a);
    let out = window.wait_with_output().unwrap();
    assert!(out.status.success(), "{mode}: the window: {}\n{}", out.status, text(&out.stderr));
    assert_eq!(text(&out.stderr), "", "{mode}: the renderer of the agent's page lives");
}

/// Q4.27b: a page that a Space's agent loaded has its entry in the
/// browser's session history, as a page the person loaded has
/// (`window_renderer_history`): the page's pushState, history.back()
/// and a second pushState keep its renderer, and `location.href`,
/// `history.length` and `history.state` follow. The same after a
/// cross-site navigation (a new renderer in site and tab mode) and after
/// the agent's `back` into a document of two entries. In site (the
/// default), tab and single mode. A renderer that reports a pushState at
/// an entry the browser does not have is still killed.
#[test]
fn window_spaces_history() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: no python3");
        return;
    };
    for mode in ["site", "tab", "single"] {
        window_spaces_history_with(&srv.base, mode);
    }

    let sock = format!("target/sockets/spaces-history-{}-evil.sock", std::process::id());
    let evil = [("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "history-index")];
    let window = spaces_window_env(&sock, &[], &evil);
    let mut a = AgentConn::open(&sock);
    let (page, _) = a.ask(&format!(r#"{{"id":1,"cmd":"navigate","url":"{}/ui/agent_push_state.html"}}"#, srv.base));
    let _ = a.ask(&format!(r#"{{"id":2,"cmd":"click","ref":"{}"}}"#, button_ref(&page, "push")));
    let (got, _) = a.ask(r#"{"id":3,"cmd":"snapshot"}"#);
    assert_eq!(got, "{\"id\":3,\"ok\":false,\"error\":\"renderer crashed\"}\n");
    drop(a);
    let out = window.wait_with_output().unwrap();
    let err = text(&out.stderr);
    assert!(err.starts_with("error: bad message: bad same_document ") && err.lines().count() == 1, "{err}");
}
