use super::*;

// Runs one session of `window_spaces`: what the agent got, then `--- window` and the
// window's state without `loop_max_ms` (it varies).
fn window_agent_session(input: &Path, n: usize, env: &[(&str, &str)]) -> Result<String, String> {
    use std::io::{Read, Write};
    use std::process::Stdio;
    use std::time::{Duration, Instant};
    let name = input.file_name().unwrap().to_string_lossy().into_owned();
    // A short relative path: sockets' paths are limited to ~100 bytes.
    let sock = format!("target/sockets/agent-spaces-{}-{n}.sock", std::process::id());
    let _ = std::fs::remove_file(root().join(&sock));
    let mut args = vec![
        "--agent-socket".to_string(),
        sock.clone(),
        "--exit-when-idle".into(),
        "--font".into(),
        "fixed".into(),
    ];
    args.extend(["--width", "800", "--height", "600", "--state"].map(String::from));
    if let Ok(events) = std::fs::read_to_string(input.with_extension("events")) {
        args.push("--events".into());
        args.push(events.trim().to_string());
    }
    let mut window = dweller_command()
        .current_dir(root())
        .envs(env.iter().copied())
        .args(&args)
        .spawn_drained();
    let start = Instant::now();
    while !root().join(&sock).exists() {
        if start.elapsed() > Duration::from_secs(20) || window.try_wait().unwrap().is_some() {
            let _ = window.kill();
            let out = window.wait_with_output().unwrap();
            return Err(format!("the window does not listen: {}", text(&out.stderr)));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let requests = std::fs::read(input).unwrap();
    let mut got = String::new();
    if name.ends_with(".mcp.jsonl") {
        let mut connect = dweller_command()
            .current_dir(root())
            .args(["connect", &sock, "--mcp"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        connect.stdin.take().unwrap().write_all(&requests).unwrap();
        let out = connect.wait_with_output().unwrap();
        if !out.status.success() || !out.stderr.is_empty() {
            let _ = window.kill();
            return Err(format!(
                "dweller connect: {}\n{}",
                out.status,
                text(&out.stderr)
            ));
        }
        got.push_str(&text(&out.stdout));
    } else {
        // Relative to the test's directory (the root), to keep it short.
        let mut s = match std::os::unix::net::UnixStream::connect(&sock) {
            Ok(s) => s,
            Err(e) => {
                let _ = window.kill();
                return Err(format!("cannot connect to {sock}: {e}"));
            }
        };
        s.write_all(token_line(&sock).as_bytes()).unwrap();
        s.write_all(&requests).unwrap();
        s.shutdown(std::net::Shutdown::Write).unwrap();
        s.read_to_string(&mut got).unwrap();
    }
    let window_pid = window.id();
    // The window leaves once its last agent did.
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(20) {
            let _ = window.kill();
            return Err("the window did not exit after its agent left".into());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    if !out.status.success() || !out.stderr.is_empty() {
        return Err(format!("the window: {}\n{}", out.status, text(&out.stderr)));
    }
    if root().join(&sock).exists() {
        return Err("the window left its socket file".into());
    }
    no_spaces_left(window_pid)?;
    got.push_str("--- window\n");
    for line in text(&out.stdout).lines() {
        // With Spaces the status line shows the person's acts and the
        // Spaces' last actions in the order they reached the window,
        // which varies; the `space` lines have each Space's last action.
        if !line.starts_with("loop_max_ms ") && !line.starts_with("activity ") {
            got.push_str(line);
            got.push('\n');
        }
    }
    Ok(got)
}

/// The Spaces the window `pid` started and that still run (`pgrep` on
/// their sockets' directories, `/tmp/dweller-PID-*`), or with hosted
/// Spaces (`HOSTED`) the window's renderers: each Space of these tests
/// has one tab, and the person's tab has no renderer.
fn spaces_of(pid: u32) -> Vec<u32> {
    let processes = Command::new("pgrep")
        .args(["-f", &format!("/tmp/dweller-{pid}-")])
        .output()
        .unwrap();
    let renderers = Command::new("pgrep")
        .args(["-P", &pid.to_string(), "-f", " --renderer 3 "])
        .output()
        .unwrap();
    text(&processes.stdout)
        .lines()
        .chain(text(&renderers.stdout).lines())
        .filter_map(|l| l.trim().parse().ok())
        .collect()
}

// The environment of a window whose Spaces are hosted in it (Q2.12).
const HOSTED: &[(&str, &str)] = &[("DWELLER_SPACES", "renderer")];

/// After the window `pid` exited: none of its Spaces runs (within 5 s),
/// and their sockets' directories are gone.
fn no_spaces_left(pid: u32) -> Result<(), String> {
    use std::time::{Duration, Instant};
    let start = Instant::now();
    loop {
        let left = spaces_of(pid);
        if left.is_empty() {
            break;
        }
        if start.elapsed() > Duration::from_secs(5) {
            return Err(format!("Spaces left running after the window: {left:?}"));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let prefix = format!("dweller-{pid}-");
    let dirs: Vec<String> = std::fs::read_dir("/tmp")
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.starts_with(&prefix))
        .collect();
    if !dirs.is_empty() {
        return Err(format!("Spaces' directories left in /tmp: {dirs:?}"));
    }
    Ok(())
}

/// The window (B9 R2, ui/spaces.aye): each connection is served by a
/// Space, a process of its own, and the window copies the bytes.
/// `browser/tests/agent/spaces/X.jsonl` is sent over the window's socket
/// (through `dweller connect --mcp` for `X.mcp.jsonl`) while the window
/// runs headless with the scripted events of `X.events` (`agents:N`
/// waits for N requests); the responses, then the window's `--state`,
/// must be `X.out`. The agent's own work answers in its Space, the
/// user's tab stays as it was, and no Space is left after the window
/// exits. `pause` (Cmd+Shift+P pauses the Space, `paused` and `resumed`,
/// `stop:1` ends it with `stopped`), `pointing` (the person's context
/// and Cmd+Shift+K reach the Space, named by the hello), `attach` (a
/// hello that names no done Space). `waited N ms` stands for the time a
/// `wait` took.
#[test]
fn window_spaces() {
    window_spaces_with(&[]);
}

/// Q2.12: with `DWELLER_SPACES=renderer` the window hosts each Space
/// itself, as a group of tabs whose pages run in renderers (no `dweller
/// space` process), and every session answers as with a Space process,
/// but `popup`: a popup opens in a renderer of its own, where its
/// `opener` is null until openers reach across renderers (Q2.12's card).
#[test]
fn window_spaces_in_renderer() {
    window_spaces_with(&[("DWELLER_SPACES", "renderer")]);
}

fn window_spaces_with(env: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = root()
        .join("browser")
        .join("tests")
        .join("agent")
        .join("spaces");
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .filter(|p| env.is_empty() || !p.ends_with("popup.jsonl"))
        .collect();
    inputs.sort();
    assert!(
        inputs.len() >= 3,
        "expected at least 3 Space sessions, found {}",
        inputs.len()
    );
    let mut failures = Vec::new();
    for (i, input) in inputs.iter().enumerate() {
        let name = input.file_name().unwrap().to_string_lossy().into_owned();
        match window_agent_session(input, i + 100 * env.len(), env) {
            Ok(got) => check_golden(
                &input.with_extension("out"),
                &waited_n(&rooted(&got)),
                &name,
                &mut failures,
            ),
            Err(e) => failures.push(format!("{name}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} Space sessions failed:\n\n{}",
        failures.len(),
        inputs.len(),
        failures.join("\n")
    );
}

// `waited 512 ms` as `waited N ms` (and `timed out after 512 ms`): a
// `wait` in real time takes a little longer each run.
fn waited_n(s: &str) -> String {
    numbered(&numbered(s, "waited "), "timed out after ")
}

fn numbered(s: &str, marker: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(at) = rest.find(marker) {
        out.push_str(&rest[..at + marker.len()]);
        rest = &rest[at + marker.len()..];
        let digits = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
        if digits > 0 && rest[digits..].starts_with(" ms") {
            out.push('N');
            rest = &rest[digits..];
        }
    }
    out.push_str(rest);
    out
}

// What `window_spaces_responsive` measured in one window.
struct Responsive {
    loop_max_ms: u64,
    // C's `tabs` while A and B were busy, in ms.
    c_ms: u128,
    a: (String, u128),
    b: (String, u128),
    state: String,
    // The window's DWELLER_TIMING records, in order (its Spaces' own
    // `ui_loop` records left out).
    records: Vec<Record>,
}

// A `ui_loop` record of the window (ms), or the window relaying an
// agent's bytes to its Space or the Space's answer back (`ui_space_sent`
// and `ui_space_answer`, with the Space's name).
#[derive(Debug, PartialEq)]
enum Record {
    Loop(u64),
    Sent(String),
    Answer(String),
}

// The window's `Record`s in `err`, the stderr of a window with Spaces
// and DWELLER_TIMING, and the lines of `err` that are not records.
fn space_records(err: &str) -> (Vec<Record>, String) {
    let mut records = Vec::new();
    let mut rest = String::new();
    for line in err.lines() {
        let Some(record) = line.strip_prefix("info: DWELLER_TIMING\t") else {
            rest.push_str(line);
            rest.push('\n');
            continue;
        };
        let fields: Vec<&str> = record.split('\t').collect();
        assert_eq!(fields.len(), 3, "{line}");
        match (fields[0], fields[2]) {
            ("ui_loop", "") => {
                records.push(Record::Loop(fields[1].parse::<u64>().unwrap() / 1_000_000))
            }
            ("ui_space_sent", name) => records.push(Record::Sent(name.to_string())),
            ("ui_space_answer", name) => records.push(Record::Answer(name.to_string())),
            _ => {}
        }
    }
    (records, rest)
}

// Agent A runs a 1500 ms `eval` loop and B loads a page whose server
// waits 1500 ms; while both run, C asks for `tabs` and the user clicks
// into a text field and types (`agents:4`: after C's answer).
fn responsive_session(base: &str, env: &[(&str, &str)]) -> Responsive {
    let sock = format!("target/sockets/spaces-resp-{}-{}.sock", env.len(), std::process::id());
    let extra = vec![
        "--state",
        "--events",
        "agents:4 click:60,100 text:hello",
        "browser/tests/agent/site/notes.html",
    ];
    let envs: Vec<(&str, &str)> = [("DWELLER_TIMING", "1")].into_iter().chain(env.iter().copied()).collect();
    let window = spaces_window_env(&sock, &extra, &envs);
    let window_pid = window.id();
    // The records are read as they come (`spaces_window`).
    let mut a = AgentConn::open(&sock);
    let mut b = AgentConn::open(&sock);
    let mut c = AgentConn::open(&sock);
    // Each one's Space is up.
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"data:text/html,<title>A</title>"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = b.ask(r#"{"id":1,"cmd":"tabs"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = c.ask(r#"{"id":1,"cmd":"tabs"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let start = std::time::Instant::now();
    a.send(r#"{"id":2,"cmd":"eval","expression":"var t = Date.now(); while (Date.now() - t < 1500) {} 'spun'"}"#);
    b.send(&format!(r#"{{"id":2,"cmd":"navigate","url":"{base}/sleep?ms=1500"}}"#));
    std::thread::sleep(std::time::Duration::from_millis(300));
    let (got, c_ms) = c.ask(r#"{"id":2,"cmd":"tabs"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let a_got = a.line();
    let a_ms = start.elapsed().as_millis();
    let b_got = b.line();
    let b_ms = start.elapsed().as_millis();
    drop(a);
    drop(b);
    drop(c);
    let out = window.wait_with_output().unwrap();
    let (records, rest) = space_records(&text(&out.stderr));
    assert!(
        out.status.success() && rest.is_empty(),
        "the window: {}\n{rest}",
        out.status
    );
    let state = text(&out.stdout);
    let loop_max_ms = state
        .lines()
        .find_map(|l| l.strip_prefix("loop_max_ms "))
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| panic!("no loop_max_ms in the state:\n{state}"));
    no_spaces_left(window_pid).unwrap();
    Responsive {
        loop_max_ms,
        c_ms,
        a: (a_got, a_ms),
        b: (b_got, b_ms),
        state,
        records,
    }
}

/// B9 R2: while agent A runs a 1500 ms `eval` loop and agent B loads a
/// page whose server waits 1500 ms, the window never
/// waits for their Spaces: in the DWELLER_TIMING records, the window
/// relays A's `eval` to its Space and the answer back in different loop
/// iterations, with others between them, in which it also relays a third
/// agent's request and answer; it handles the user's click and typing.
/// What does not depend on the machine's load is asserted; the times are
/// bounded generously (a loaded machine stalls a loop iteration for tens
/// of ms, a wait for A's Space stalls one for over a second).
#[test]
fn window_spaces_responsive() {
    window_spaces_responsive_with(&[]);
}

/// Q2.12b: the same with the Spaces hosted in the window (`HOSTED`): B's
/// navigation is fetched in the background, so no loop iteration waits
/// for the server.
#[test]
fn window_spaces_responsive_in_renderer() {
    window_spaces_responsive_with(HOSTED);
}

fn window_spaces_responsive_with(env: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let r = responsive_session(&srv.base, env);
    assert!(
        r.a.0.contains("\"result\":\"spun\"") && r.a.1 >= 1400,
        "A: {} ms: {}",
        r.a.1,
        r.a.0
    );
    assert!(
        r.b.0.contains("\"title\":\"Slept\"") && r.b.1 >= 1400,
        "B: {} ms: {}",
        r.b.1,
        r.b.0
    );
    // The Spaces in the order their agents first asked: A, B, C. A hosted
    // Space's `tabs` is answered by the window (Q2.12), so C sends none.
    let hosted = !env.is_empty();
    let mut names: Vec<&str> = Vec::new();
    for rec in &r.records {
        if let Record::Sent(name) = rec {
            if !names.contains(&name.as_str()) {
                names.push(name);
            }
        }
    }
    assert_eq!(names.len(), if hosted { 2 } else { 3 }, "{:?}", r.records);
    // The index of the `n`th (from 0) record equal to `want`.
    let nth = |n: usize, want: Record| {
        r.records
            .iter()
            .enumerate()
            .filter(|(_, rec)| **rec == want)
            .nth(n)
            .map(|(i, _)| i)
            .unwrap_or_else(|| panic!("{want:?} {n} not found: {:?}", r.records))
    };
    let loops = |from: usize, to: usize| {
        r.records[from..to]
            .iter()
            .filter(|rec| matches!(rec, Record::Loop(_)))
            .count()
    };
    let (a_name, b_name) = (names[0].to_string(), names[1].to_string());
    let a_sent = nth(1, Record::Sent(a_name.clone()));
    let a_answer = nth(1, Record::Answer(a_name));
    // B's `navigate`, after its `tabs` (which a hosted Space does not send).
    let b_at = if hosted { 0 } else { 1 };
    let b_sent = nth(b_at, Record::Sent(b_name.clone()));
    let b_answer = nth(b_at, Record::Answer(b_name));
    let loops_between = loops(a_sent, a_answer);
    assert!(
        loops_between >= 3 && b_sent < b_answer && loops(b_sent, b_answer) >= 3,
        "A's eval out at {a_sent}, back at {a_answer}, {loops_between} loop iterations between; \
         B's navigate out at {b_sent}, back at {b_answer}: {:?}",
        r.records
    );
    if !hosted {
        let c_name = names[2].to_string();
        let c_sent = nth(1, Record::Sent(c_name.clone()));
        let c_answer = nth(1, Record::Answer(c_name));
        assert!(
            a_sent < c_sent && c_answer < a_answer,
            "A's eval out at {a_sent}, back at {a_answer}; C's tabs out at {c_sent}, back at {c_answer}: {:?}",
            r.records
        );
    }
    let longest = r
        .records
        .iter()
        .filter_map(|rec| match rec {
            Record::Loop(ms) => Some(*ms),
            _ => None,
        })
        .max()
        .unwrap_or(0);
    assert!(
        longest < 1000 && r.loop_max_ms < 1000,
        "longest loop {longest} ms\n{}",
        r.state
    );
    assert!(r.c_ms < 1000, "C waited {} ms", r.c_ms);
    assert!(r.state.contains("\nvalue hello\n"), "{}", r.state);
    println!(
        "loop_max_ms {} (C {} ms, {} loop iterations during A's eval)",
        r.loop_max_ms, r.c_ms, loops_between
    );
}

/// A Space that crashes (SIGKILL) ends only its own agent's connection,
/// with `{"id":null,"ok":false,"error":"the Space crashed"}`; the window
/// and the other agents go on. A Space whose window is killed exits (the
/// end of file on its control connection) and removes its directory.
#[test]
fn window_spaces_crash() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let sock = format!("target/sockets/spaces-crash-{}.sock", std::process::id());
    let mut window = spaces_window(&sock, &["--spaces"]);
    let mut a = AgentConn::open(&sock);
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let first = spaces_of(window.id());
    assert_eq!(first.len(), 1, "{first:?}");
    let mut b = AgentConn::open(&sock);
    let (got, _) = b.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/index.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    assert_eq!(spaces_of(window.id()).len(), 2);
    let status = Command::new("kill")
        .args(["-9", &first[0].to_string()])
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(
        a.line(),
        "{\"id\":null,\"ok\":false,\"error\":\"the Space crashed\"}\n"
    );
    assert_eq!(a.line(), "", "the connection ends");
    assert!(window.try_wait().unwrap().is_none(), "the window goes on");
    let (got, _) = b.ask(r#"{"id":2,"cmd":"snapshot"}"#);
    assert!(got.contains("\"title\":\"Site home\""), "{got}");
    b.stream.shutdown(std::net::Shutdown::Write).unwrap();
    assert_eq!(b.line(), "");
    let window_pid = window.id();
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    no_spaces_left(window_pid).unwrap();

    // The window killed: its Space exits by itself.
    let sock = format!("target/sockets/spaces-kill-{}.sock", std::process::id());
    let mut window = spaces_window(&sock, &["--spaces"]);
    let mut a = AgentConn::open(&sock);
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    assert_eq!(spaces_of(window.id()).len(), 1);
    window.kill().unwrap();
    window.wait().unwrap();
    let _ = std::fs::remove_file(root().join(&sock));
    no_spaces_left(window.id()).unwrap();
}

/// Q2.12c: with hosted Spaces a renderer that dies takes only its tab.
/// Agent A has two tabs, B one; A's first tab's renderer is killed while
/// A's `eval` runs in it: that request and every later page command on
/// the tab fail with "renderer crashed", `tabs` lists the tab as
/// `crashed`, A's other tab, B and the Space go on, and `tab_close`
/// closes the crashed tab.
#[test]
fn window_spaces_crash_in_renderer() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let sock = format!("target/sockets/spaces-crash-r-{}.sock", std::process::id());
    let window = spaces_window_env(&sock, &["--spaces"], HOSTED);
    let pid = window.id();
    let mut a = AgentConn::open(&sock);
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let first = spaces_of(pid);
    assert_eq!(first.len(), 1, "{first:?}");
    let (got, _) = a.ask(r#"{"id":2,"cmd":"tab_new"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = a.ask(r#"{"id":3,"cmd":"navigate","url":"browser/tests/agent/site/index.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = a.ask(r#"{"id":4,"cmd":"tab_select","tab":1}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let mut b = AgentConn::open(&sock);
    let (got, _) = b.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/index.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    assert_eq!(spaces_of(pid).len(), 3);
    a.send(r#"{"id":5,"cmd":"eval","expression":"(function () { var t = Date.now(); while (Date.now() - t < 5000) {} return 1 })()"}"#);
    std::thread::sleep(std::time::Duration::from_millis(300));
    let status = Command::new("kill").args(["-9", &first[0].to_string()]).status().unwrap();
    assert!(status.success());
    let crashed = |id: u32| format!("{{\"id\":{id},\"ok\":false,\"error\":\"renderer crashed\"}}\n");
    assert_eq!(a.line(), crashed(5));
    let (got, _) = a.ask(r#"{"id":6,"cmd":"snapshot"}"#);
    assert_eq!(got, crashed(6));
    let (got, _) = a.ask(r#"{"id":7,"cmd":"wait","ms":10}"#);
    assert_eq!(got, crashed(7));
    let (got, _) = a.ask(r#"{"id":8,"cmd":"tabs"}"#);
    assert!(got.contains("\"agent\":true,\"crashed\":true}"), "{got}");
    assert_eq!(got.matches("\"crashed\"").count(), 1, "{got}");
    let (got, _) = a.ask(r#"{"id":9,"cmd":"spaces"}"#);
    assert!(got.contains("\"mine\":true,\"state\":\"working\"") && !got.contains("crashed"), "{got}");
    let (got, _) = a.ask(r#"{"id":10,"cmd":"tab_select","tab":2}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = a.ask(r#"{"id":11,"cmd":"snapshot"}"#);
    assert!(got.contains("\"title\":\"Site home\""), "{got}");
    let (got, _) = b.ask(r#"{"id":2,"cmd":"snapshot"}"#);
    assert!(got.contains("\"title\":\"Site home\""), "{got}");
    let (got, _) = a.ask(r#"{"id":12,"cmd":"tab_close","tab":1}"#);
    assert!(got.contains("\"ok\":true") && got.contains("closed tab 1"), "{got}");
    let (got, _) = a.ask(r#"{"id":13,"cmd":"tabs"}"#);
    assert!(got.contains("1 tab") && !got.contains("crashed"), "{got}");
    assert_eq!(spaces_of(pid).len(), 2);
    for c in [&mut a, &mut b] {
        c.stream.shutdown(std::net::Shutdown::Write).unwrap();
        assert_eq!(c.line(), "");
    }
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    no_spaces_left(pid).unwrap();
}

/// Q2.12c: an `isolated` Space is hosted in the window too, on a
/// partition of its own: a cookie jar and `localStorage` in memory.
/// Iso's cookies (from script and from the network) and storage are
/// seen neither by the shared Space Alpha nor by the person's tab, nor
/// by a second isolated Space; what Alpha and the person set is not
/// seen by Iso. No Space process starts, and nothing of Iso's reaches
/// the profile directory (its cookies, its `localStorage`, an origin
/// only Iso visited).
#[test]
fn window_spaces_isolated_in_renderer() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let alias = base.replace("127.0.0.1", "localhost");
    let dir = profile("q212c-profile");
    let sock = format!("target/sockets/spaces-iso-{}.sock", std::process::id());
    let events = format!("agents:14 key:cmd+l {} key:return", typed(&format!("{base}/login/shared.html")));
    let start = format!("{base}/net/start");
    let envs = [HOSTED[0], ("DWELLER_PROFILE", dir.to_str().unwrap())];
    let window = spaces_window_env(&sock, &["--spaces", "--state", "--text", "--events", &events, &start], &envs);
    let pid = window.id();
    let ok = |got: &str| assert!(got.contains("\"ok\":true"), "{got}");
    let go = |id: u32, at: &str| format!(r#"{{"id":{id},"cmd":"navigate","url":"{at}"}}"#);
    let eval = |id: u32, js: &str| format!("{{\"id\":{id},\"cmd\":\"eval\",\"expression\":{}}}", serde_like(js));
    let read = r#"{"id":0,"cmd":"read"}"#;
    let mut a = AgentConn::open(&sock);
    a.send(r#"{"name":"Alpha"}"#);
    ok(&a.ask(&go(1, &format!("{base}/login/read.html"))).0);
    let (got, _) = a.ask(&eval(2, "document.cookie = 'shared=1; Path=/'; localStorage.setItem('token', 'from-shared'); sessionStorage.setItem('tab', 'alpha'); document.cookie"));
    assert!(got.contains("shared=1\""), "{got}");
    let mut b = AgentConn::open(&sock);
    b.send(r#"{"name":"Iso","profile":"isolated"}"#);
    ok(&b.ask(&go(3, &format!("{base}/login/read.html"))).0);
    let set = "var seen = localStorage.getItem('token') + '|' + document.cookie + '|' + sessionStorage.getItem('tab'); document.cookie = 'iso=1; Path=/'; localStorage.setItem('iso', '1'); sessionStorage.setItem('tab', 'iso'); seen + '/' + document.cookie + '|' + localStorage.getItem('iso') + '|' + sessionStorage.getItem('tab')";
    let (got, _) = b.ask(&eval(4, set));
    assert!(got.contains("\"null||null/iso=1|1|iso\""), "{got}");
    ok(&b.ask(&go(5, &format!("{base}/headers"))).0);
    let (got, _) = b.ask(read);
    assert!(got.contains("Cookie: iso=1") && !got.contains("shared=1"), "Iso's /headers: {got}");
    ok(&b.ask(&go(7, &format!("{alias}/login/read.html"))).0);
    let (got, _) = b.ask(&eval(8, "document.cookie = 'isohost=1; Path=/'; localStorage.setItem('isohost', '1'); document.cookie"));
    assert!(got.contains("\"isohost=1\""), "{got}");
    let mut c = AgentConn::open(&sock);
    c.send(r#"{"name":"Iso2","profile":"isolated"}"#);
    ok(&c.ask(&go(9, &format!("{base}/login/read.html"))).0);
    let (got, _) = c.ask(&eval(10, "localStorage.getItem('iso') + '|' + localStorage.getItem('token') + '|' + document.cookie"));
    assert!(got.contains("\"null|null|\""), "Iso2 sees another partition: {got}");
    let (got, _) = a.ask(&eval(11, "document.cookie + '|' + localStorage.getItem('iso') + '|' + sessionStorage.getItem('tab')"));
    assert!(got.contains("shared=1|null|alpha\"") && !got.contains("iso"), "Alpha sees Iso's: {got}");
    ok(&a.ask(&go(12, &format!("{base}/headers"))).0);
    let (got, _) = a.ask(read);
    assert!(got.contains("shared=1") && !got.contains("iso"), "Alpha's /headers: {got}");
    let (got, _) = a.ask(r#"{"id":14,"cmd":"tabs"}"#);
    ok(&got);
    let processes = Command::new("pgrep").args(["-f", &format!("/tmp/dweller-{pid}-")]).output().unwrap();
    assert!(text(&processes.stdout).is_empty(), "a Space process: {}", text(&processes.stdout));
    assert_eq!(spaces_of(pid).len(), 3, "one renderer per Space");
    let begun = std::time::Instant::now();
    loop {
        let (got, _) = a.ask(r#"{"id":15,"cmd":"user_context"}"#);
        if got.contains("\"title\":\"Shared done\"") {
            break;
        }
        assert!(begun.elapsed().as_secs() < 20, "the person's page did not finish: {got}");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    ok(&b.ask(&go(16, &format!("{base}/login/read.html"))).0);
    let (got, _) = b.ask(&eval(16, "document.cookie + '|' + localStorage.getItem('from') + '|' + localStorage.getItem('token')"));
    assert!(got.contains("\"iso=1|null|null\""), "Iso sees the person's: {got}");
    let (got, _) = b.ask(r#"{"id":17,"cmd":"spaces"}"#);
    assert!(got.contains("\"name\":\"Iso\",\"mine\":true") && got.contains("\"profile\":\"isolated\""), "{got}");
    let (got, _) = b.ask(r#"{"id":18,"cmd":"space_close"}"#);
    assert!(got.contains("closing Space 2"), "{got}");
    assert_eq!(b.line(), "");
    for conn in [&mut a, &mut c] {
        conn.stream.shutdown(std::net::Shutdown::Write).unwrap();
        assert_eq!(conn.line(), "");
    }
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    let page = text(&out.stdout);
    assert!(page.contains("token: from-shared; cookie: ") && page.contains("shared=1"), "{page}");
    assert!(!page.contains("iso"), "the person sees Iso's: {page}");
    let sent = page.lines().find(|l| l.starts_with("Cookie: ")).unwrap_or("");
    assert!(sent.contains("shared=1") && sent.contains("person=1"), "the person's /headers: {page}");
    let mut files = vec![dir.clone()];
    let mut seen = Vec::new();
    while let Some(p) = files.pop() {
        if p.is_dir() {
            files.extend(std::fs::read_dir(&p).unwrap().map(|e| e.unwrap().path()));
            continue;
        }
        let name = p.strip_prefix(&dir).unwrap().to_string_lossy().into_owned();
        let bytes = String::from_utf8_lossy(&std::fs::read(&p).unwrap()).into_owned();
        assert!(!name.contains("localhost") && !bytes.contains("localhost"), "the profile has Iso's origin: {name}: {bytes}");
        assert!(!bytes.contains("iso"), "the profile has Iso's: {name}: {bytes}");
        seen.push(name);
    }
    assert!(seen.iter().any(|n| n == "cookies") && seen.iter().any(|n| n.starts_with("storage/")), "{seen:?}");
    no_spaces_left(pid).unwrap();
}

// Waits (at most 5 s) until the window `pid` runs `n` Spaces.
fn wait_for_spaces(pid: u32, n: usize) {
    let start = std::time::Instant::now();
    while spaces_of(pid).len() != n {
        assert!(
            start.elapsed() < std::time::Duration::from_secs(5),
            "expected {n} Spaces, found {:?}",
            spaces_of(pid)
        );
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

/// B9 R4, R6 (P9.3): the window controls each Space over its control
/// connection. Two agents, A and B, named by their hellos: the person
/// pauses A (the scripted word `pause:1`): A gets `paused` and its
/// actions fail with "paused by user" while B's work; `resume:1` lets A
/// go on; `stop:2` ends B (`stopped`, then the end of its connection,
/// and its Space's process exits) while A goes on. `spaces` lists the
/// Spaces with their names and states. A Space whose agent left stays
/// `done`, and a new connection attaches to it (`"space":3` in the
/// hello); attaching to a Space that has an agent, or to none, fails;
/// `space_close` ends the Space. No Space is left after the window.
#[test]
fn window_spaces_control() {
    window_spaces_control_with(&[]);
}

/// Q2.12: the same with the Spaces hosted in the window (`HOSTED`).
#[test]
fn window_spaces_control_in_renderer() {
    window_spaces_control_with(HOSTED);
}

fn window_spaces_control_with(env: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let sock = format!("target/sockets/spaces-control-{}-{}.sock", std::process::id(), env.len());
    let window = spaces_window_env(
        &sock,
        &[
            "--spaces",
            "--state",
            "--events",
            "agents:2 pause:1 agents:5 resume:1 agents:7 stop:2",
        ],
        env,
    );
    let pid = window.id();
    let mut a = AgentConn::open(&sock);
    a.send(r#"{"name":"Alpha"}"#);
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let mut b = AgentConn::open(&sock);
    b.send(r#"{"name":"Beta"}"#);
    let (got, _) = b.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/index.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    assert_eq!(spaces_of(pid).len(), 2);
    // The person pauses A only.
    assert_eq!(a.line(), "{\"event\":\"paused\",\"space\":1}\n");
    let (got, _) = a.ask(r#"{"id":2,"cmd":"click","ref":"e4"}"#);
    assert_eq!(got, "{\"id\":2,\"ok\":false,\"error\":\"paused by user\"}\n");
    let (got, _) = b.ask(r#"{"id":2,"cmd":"click","ref":"e6"}"#);
    assert!(got.contains("\"result\":\"clicked button \\\"Subscribe\\\" [e6]\""), "{got}");
    assert_eq!(a.line(), "{\"event\":\"resumed\",\"space\":1}\n");
    let (got, _) = a.ask(r#"{"id":3,"cmd":"click","ref":"e4"}"#);
    assert!(got.contains("\"result\":\"checked checkbox"), "{got}");
    // The person stops B: its connection ends, its Space exits, A goes on.
    assert_eq!(b.line(), "{\"event\":\"stopped\",\"space\":2}\n");
    assert_eq!(b.line(), "", "B's connection ends");
    wait_for_spaces(pid, 1);
    let (got, _) = a.ask(r#"{"id":4,"cmd":"spaces"}"#);
    assert!(
        got.starts_with("{\"id\":4,\"ok\":true,\"result\":\"1 space\",\"spaces\":[{\"id\":1,\"name\":\"Alpha\",\"mine\":true,\"state\":\"working\",\"last\":\"click checkbox"),
        "{got}"
    );
    // C leaves: its Space stays, done.
    let mut c = AgentConn::open(&sock);
    c.send(r#"{"name":"Gamma"}"#);
    let (got, _) = c.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/docs.html"}"#);
    assert!(got.contains("\"title\":\"Docs\""), "{got}");
    c.stream.shutdown(std::net::Shutdown::Write).unwrap();
    assert_eq!(c.line(), "");
    assert_eq!(spaces_of(pid).len(), 2, "a done Space stays");
    let (got, _) = a.ask(r#"{"id":5,"cmd":"spaces"}"#);
    assert!(got.contains("\"result\":\"2 spaces\""), "{got}");
    assert!(
        got.contains("{\"id\":3,\"name\":\"Gamma\",\"mine\":false,\"state\":\"done\",\"last\":\"navigate browser/tests/agent/site/docs.html\",\"tabs\":1,\"profile\":\"shared\",\"watched\":false}"),
        "{got}"
    );
    // D attaches to it and finds C's page.
    let mut d = AgentConn::open(&sock);
    d.send(r#"{"name":"Delta","space":3}"#);
    let (got, _) = d.ask(r#"{"id":1,"cmd":"snapshot"}"#);
    assert!(got.contains("\"title\":\"Docs\""), "{got}");
    let (got, _) = d.ask(r#"{"id":2,"cmd":"spaces"}"#);
    assert!(
        got.contains("[{\"id\":3,\"name\":\"Delta\",\"mine\":true,\"state\":\"working\""),
        "{got}"
    );
    assert!(got.contains("{\"id\":1,\"name\":\"Alpha\",\"mine\":false"), "{got}");
    // Only a done Space can be attached to; a profile is shared or isolated.
    for (hello, why) in [
        (r#"{"space":3}"#, "Space 3 has an agent: only a done Space can be attached to"),
        (r#"{"space":9}"#, "no Space 9: see \"spaces\""),
        (r#"{"profile":"other"}"#, "\"profile\" is \"shared\" or \"isolated\""),
    ] {
        let mut e = AgentConn::open(&sock);
        e.send(hello);
        assert_eq!(
            e.line(),
            format!("{{\"id\":null,\"ok\":false,\"error\":{}}}\n", serde_like(why)),
            "{hello}"
        );
        assert_eq!(e.line(), "", "{hello}");
    }
    // D closes its Space.
    let (got, _) = d.ask(r#"{"id":3,"cmd":"space_close"}"#);
    assert_eq!(got, "{\"id\":3,\"ok\":true,\"result\":\"closing Space 3\"}\n");
    assert_eq!(d.line(), "");
    wait_for_spaces(pid, 1);
    let (got, _) = a.ask(r#"{"id":6,"cmd":"spaces"}"#);
    assert!(got.contains("\"result\":\"1 space\""), "{got}");
    a.stream.shutdown(std::net::Shutdown::Write).unwrap();
    assert_eq!(a.line(), "");
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    let state = text(&out.stdout);
    assert!(
        state.contains("\nspace 1 \"Alpha\" done: spaces\n")
            || state.contains("\nspace 1 \"Alpha\" done: click checkbox"),
        "{state}"
    );
    no_spaces_left(pid).unwrap();
}

const PARALLEL_SCRIPTS: [(&str, &[&str]); 3] = [
    (
        "Alpha",
        &[
            r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#,
            r#"{"id":2,"cmd":"type","ref":"e2","text":"Ada"}"#,
            r#"{"id":3,"cmd":"click","ref":"e4"}"#,
            r#"{"id":4,"cmd":"read"}"#,
        ],
    ),
    (
        "Beta",
        &[
            r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/index.html"}"#,
            r#"{"id":2,"cmd":"click","ref":"e6"}"#,
        ],
    ),
    (
        "Gamma",
        &[
            r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/docs.html"}"#,
            r#"{"id":2,"cmd":"snapshot"}"#,
            r#"{"id":3,"cmd":"read"}"#,
        ],
    ),
];

// A connection that said hello as `name`.
fn named_conn(sock: &str, name: &str) -> AgentConn {
    let mut conn = AgentConn::open(sock);
    conn.send(&format!("{{\"name\":\"{name}\"}}"));
    conn
}

// Sends `script` one request at a time; the answers.
fn answers(conn: &mut AgentConn, script: &[&str]) -> Vec<String> {
    script.iter().map(|l| conn.ask(l).0).collect()
}

/// B9 R3 (P9.9): three agents in three Spaces navigate, type and read
/// at once, and each one's answers equal its answers alone. The person
/// pauses Beta (`pause:2`): Beta is refused while Alpha and Gamma go on,
/// and the person's tab strip still shows one tab.
#[test]
fn window_spaces_parallel() {
    window_spaces_parallel_with(&[]);
}

/// Q2.12: the same with the Spaces hosted in the window (`HOSTED`).
#[test]
fn window_spaces_parallel_in_renderer() {
    window_spaces_parallel_with(HOSTED);
}

fn window_spaces_parallel_with(env: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let alone: Vec<Vec<String>> = PARALLEL_SCRIPTS
        .iter()
        .enumerate()
        .map(|(i, (name, script))| {
            let sock = format!("target/sockets/spaces-alone{i}-{}-{}.sock", std::process::id(), env.len());
            let window = spaces_window_env(&sock, &[], env);
            let got = answers(&mut named_conn(&sock, name), script);
            let out = window.wait_with_output().unwrap();
            assert!(out.status.success() && out.stderr.is_empty(), "{}", text(&out.stderr));
            assert!(got.iter().all(|l| l.contains("\"ok\":true")), "{name}: {got:?}");
            got
        })
        .collect();
    let sock = format!("target/sockets/spaces-parallel-{}-{}.sock", std::process::id(), env.len());
    let window = spaces_window_env(&sock, &["--state", "--events", "agents:9 pause:2"], env);
    let pid = window.id();
    let mut conns: Vec<AgentConn> = PARALLEL_SCRIPTS
        .iter()
        .map(|(name, _)| named_conn(&sock, name))
        .collect();
    let together: Vec<Vec<String>> = std::thread::scope(|scope| {
        let runs: Vec<_> = conns
            .iter_mut()
            .zip(PARALLEL_SCRIPTS.iter())
            .map(|(conn, (_, script))| scope.spawn(move || answers(conn, script)))
            .collect();
        runs.into_iter().map(|r| r.join().unwrap()).collect()
    });
    assert_eq!(together, alone);
    assert_eq!(spaces_of(pid).len(), 3);
    assert_eq!(conns[1].line(), "{\"event\":\"paused\",\"space\":2}\n");
    let (got, _) = conns[1].ask(r#"{"id":3,"cmd":"click","ref":"e6"}"#);
    assert_eq!(got, "{\"id\":3,\"ok\":false,\"error\":\"paused by user\"}\n");
    let (got, _) = conns[0].ask(r#"{"id":5,"cmd":"snapshot"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = conns[2].ask(r#"{"id":4,"cmd":"read"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    for conn in &conns {
        conn.stream.shutdown(std::net::Shutdown::Write).unwrap();
    }
    let out = window.wait_with_output().unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{}", text(&out.stderr));
    let state = text(&out.stdout);
    assert!(state.contains("\ntabs 1, showing 1\n"), "{state}");
    no_spaces_left(pid).unwrap();
}

/// B9 R5 (P9.7): a `shared` Space starts with the window's cookies
/// (session and persistent) and `localStorage`, and each side gets the
/// other's changes over the control connection; an `isolated` Space
/// starts empty and shares nothing. The person's tab got the cookies of
/// `/net/start`; Alpha's `/headers` sends them, Alpha sets a cookie and a
/// `localStorage` item by script, Iso (isolated) sets its own; then the
/// person's page (`login/shared.html`) sees Alpha's and not Iso's, and
/// what it sets reaches Alpha and not Iso.
#[test]
fn window_spaces_logins() {
    window_spaces_logins_with(&[]);
}

/// Q2.12: the same with the Spaces hosted in the window (`HOSTED`).
#[test]
fn window_spaces_logins_in_renderer() {
    window_spaces_logins_with(HOSTED);
}

fn window_spaces_logins_with(env: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let sock = format!("target/sockets/spaces-logins-{}-{}.sock", std::process::id(), env.len());
    let events = format!(
        "agents:8 key:cmd+l {} key:return",
        typed(&format!("{base}/login/shared.html"))
    );
    let start = format!("{base}/net/start");
    let window = spaces_window_env(
        &sock,
        &["--spaces", "--state", "--text", "--events", &events, &start],
        env,
    );
    let pid = window.id();
    let mut a = AgentConn::open(&sock);
    a.send(r#"{"name":"Alpha"}"#);
    let headers = format!(r#"{{"id":1,"cmd":"navigate","url":"{base}/headers"}}"#);
    let page = format!(r#"{{"id":3,"cmd":"navigate","url":"{base}/login/read.html"}}"#);
    let (got, _) = a.ask(&headers);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = a.ask(r#"{"id":2,"cmd":"read"}"#);
    assert!(got.contains("Cookie: "), "{got}");
    for c in ["a=1", "h=7", "p=5"] {
        assert!(got.contains(c), "Alpha's /headers lacks {c}: {got}");
    }
    let (got, _) = a.ask(&page);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = a.ask(r#"{"id":4,"cmd":"eval","expression":"document.cookie = 'space=9; Path=/'; localStorage.setItem('token', 'from-space'); document.cookie"}"#);
    assert!(got.contains("space=9"), "{got}");
    let mut b = AgentConn::open(&sock);
    b.send(r#"{"name":"Iso","profile":"isolated"}"#);
    let (got, _) = b.ask(&headers);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = b.ask(r#"{"id":2,"cmd":"read"}"#);
    assert!(!got.contains("a=1") && !got.contains("space=9"), "Iso sees cookies: {got}");
    let (got, _) = b.ask(&page);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = b.ask(r#"{"id":4,"cmd":"eval","expression":"document.cookie = 'iso=1; Path=/'; localStorage.setItem('iso', '1'); localStorage.getItem('token') + '|' + document.cookie"}"#);
    assert!(got.contains("null|iso=1\""), "{got}");
    let begun = std::time::Instant::now();
    loop {
        let (got, _) = a.ask(r#"{"id":5,"cmd":"user_context"}"#);
        if got.contains("\"title\":\"Shared done\"") {
            break;
        }
        assert!(begun.elapsed().as_secs() < 20, "the person's page did not finish: {got}");
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    let (got, _) = a.ask(r#"{"id":6,"cmd":"eval","expression":"document.cookie + '|' + localStorage.getItem('from')"}"#);
    assert!(got.contains("person=1") && got.contains("|person\""), "{got}");
    let (got, _) = b.ask(r#"{"id":5,"cmd":"eval","expression":"document.cookie + '|' + localStorage.getItem('from')"}"#);
    assert!(got.contains("\"iso=1|null\""), "{got}");
    let (got, _) = b.ask(r#"{"id":6,"cmd":"spaces"}"#);
    assert!(got.contains("\"name\":\"Iso\",\"mine\":true") && got.contains("\"profile\":\"isolated\""), "{got}");
    assert!(got.contains("\"name\":\"Alpha\",\"mine\":false,\"state\":\"idle\""), "{got}");
    assert!(got.contains("\"profile\":\"shared\""), "{got}");
    for c in [&mut a, &mut b] {
        c.stream.shutdown(std::net::Shutdown::Write).unwrap();
        assert_eq!(c.line(), "");
    }
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    let page = text(&out.stdout);
    assert!(page.contains("token: from-space; cookie: "), "{page}");
    assert!(page.contains("space=9") && page.contains("a=1"), "{page}");
    assert!(!page.contains("iso"), "the person sees Iso's: {page}");
    let sent = page.lines().find(|l| l.starts_with("Cookie: ")).unwrap_or("");
    for c in ["a=1", "h=7", "space=9", "person=1"] {
        assert!(sent.contains(c), "the person's /headers lacks {c}: {page}");
    }
    no_spaces_left(pid).unwrap();
}

/// B9 R6 (P9.4): the toolbar's agent area shows "You" and a chip per
/// Space (a dot by its state, its name); the mouse over a chip shows the
/// Space's state and last action in the status line; a click on a chip
/// opens its menu: Pause (Resume) and Stop act on that Space only. Three
/// agents: the person pauses Alpha with its chip (Alpha's actions fail
/// while Beta's work), stops Gamma (it gets `stopped`, its Space exits,
/// its chip goes), then points at Beta's chip. The person's page keeps
/// its scroll and its height (the chips sit in the toolbar, the menu
/// over the page), and the window never waits for a Space (its
/// DWELLER_TIMING records: loop iterations between each request and its
/// answer; no iteration near a second). The last
/// frame, with two Spaces (Alpha paused, Beta idle) and Beta's status,
/// is `browser/tests/ui/spaces_chips.png` (`DWELLER_BLESS=1` rewrites it).
#[test]
fn window_spaces_chips() {
    window_spaces_chips_with(&[]);
}

/// Q2.12: the same with the Spaces hosted in the window (`HOSTED`).
#[test]
fn window_spaces_chips_in_renderer() {
    window_spaces_chips_with(HOSTED);
}

fn window_spaces_chips_with(env: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    // The address bar is in the PNG: the person's page is a copy at a
    // fixed path.
    let fixed_dir = "/tmp/dweller-chips";
    let page = format!("{fixed_dir}/long.html");
    std::fs::create_dir_all(fixed_dir).unwrap();
    std::fs::write(&page, std::fs::read(root().join("browser/tests/ui/long.html")).unwrap()).unwrap();
    // The person's scroll after the same wheel, without agents.
    let alone = run_window(&[
        &page, "--font", "fixed", "--width", "800", "--height", "600", "--frames", "1", "--state",
        "--events", "wheel:-3",
    ]);
    let alone = text(&alone.stdout);
    let scroll = alone.lines().find(|l| l.starts_with("scroll ")).unwrap().to_string();
    assert_ne!(scroll, "scroll 0", "{alone}");
    // With --font fixed in 800 px, the chips are You 503-536, Alpha
    // 541-599, Beta 604-656, Gamma 661-719 (y 9-31); a menu's items are
    // 24 px from y 38, under its chip. Without Gamma, Beta is 667-719.
    let sock = format!("target/sockets/spaces-chips-{}-{}.sock", std::process::id(), env.len());
    let png = scratch().join(format!("spaces_chips{}.png", env.len()));
    let events = "wheel:-3 agents:3 click:570,20 click:560,50 \
                  agents:6 click:690,20 click:680,74 \
                  agents:8 move:690,20 key:cmd+q";
    let window = spaces_window_env(
        &sock,
        &["--spaces", "--state", "--save", png.to_str().unwrap(), "--events", events, &page],
        &[env, &[("DWELLER_TIMING", "1")]].concat(),
    );
    let pid = window.id();
    // The records are read as they come (`spaces_window`).
    let mut agents = Vec::new();
    for (name, site) in [("Alpha", "form.html"), ("Beta", "index.html"), ("Gamma", "docs.html")] {
        let mut c = AgentConn::open(&sock);
        c.send(&format!("{{\"name\":\"{name}\"}}"));
        let (got, _) = c.ask(&format!(
            "{{\"id\":1,\"cmd\":\"navigate\",\"url\":\"browser/tests/agent/site/{site}\"}}"
        ));
        assert!(got.contains("\"ok\":true"), "{name}: {got}");
        agents.push(c);
    }
    let mut gamma = agents.pop().unwrap();
    let mut beta = agents.pop().unwrap();
    let mut alpha = agents.pop().unwrap();
    // Pause on Alpha's chip: Alpha only.
    assert_eq!(alpha.line(), "{\"event\":\"paused\",\"space\":1}\n");
    let (got, _) = beta.ask(r#"{"id":2,"cmd":"click","ref":"e6"}"#);
    assert!(got.contains("\"result\":\"clicked button \\\"Subscribe\\\" [e6]\""), "{got}");
    let (got, _) = alpha.ask(r#"{"id":2,"cmd":"click","ref":"e4"}"#);
    assert_eq!(got, "{\"id\":2,\"ok\":false,\"error\":\"paused by user\"}\n");
    // Stop on Gamma's chip: Gamma ends, the others stay.
    assert_eq!(gamma.line(), "{\"event\":\"stopped\",\"space\":3}\n");
    assert_eq!(gamma.line(), "", "Gamma's connection ends");
    wait_for_spaces(pid, 2);
    let (got, _) = alpha.ask(r#"{"id":3,"cmd":"spaces"}"#);
    assert!(got.contains("\"result\":\"2 spaces\""), "{got}");
    assert!(got.contains("{\"id\":1,\"name\":\"Alpha\",\"mine\":true,\"state\":\"paused\""), "{got}");
    assert!(got.contains("{\"id\":2,\"name\":\"Beta\",\"mine\":false,\"state\":\"idle\""), "{got}");
    // The person points at Beta's chip and closes the window.
    let out = window.wait_with_output().unwrap();
    let (records, rest) = space_records(&text(&out.stderr));
    assert!(
        out.status.success() && rest.is_empty(),
        "the window: {}\n{rest}",
        out.status
    );
    let state = text(&out.stdout);
    for want in [
        format!("\n{scroll}\n"),
        "\nfocus page\n".to_string(),
        "\nviewport 800x532\n".to_string(),
        "\nstatus Beta (idle): click button \"Subscribe\" [e6]\n".to_string(),
        "\nspace 1 \"Alpha\" paused".to_string(),
        "\nspace 2 \"Beta\" idle: click button \"Subscribe\" [e6]\n".to_string(),
    ] {
        assert!(state.contains(&want), "{want:?} in:\n{state}");
    }
    assert!(!state.contains("\nspace 3 "), "Gamma's chip goes:\n{state}");
    assert!(state.contains("\nactivity Gamma stopped\n"), "{state}");
    // The window never waits for a Space: each one's first request (its
    // navigate, which opens a page in the Space) goes out and comes back
    // in different loop iterations, with others between them.
    for name in ["Alpha", "Beta", "Gamma"] {
        let sent = records
            .iter()
            .position(|r| *r == Record::Sent(name.to_string()));
        let answer = records
            .iter()
            .position(|r| *r == Record::Answer(name.to_string()));
        let loops = match (sent, answer) {
            (Some(s), Some(a)) if s < a => records[s..a]
                .iter()
                .filter(|r| matches!(r, Record::Loop(_)))
                .count(),
            _ => 0,
        };
        assert!(
            loops >= 3,
            "{name}'s first request: {loops} loop iterations before its answer: {records:?}"
        );
    }
    let loop_max: u64 = state
        .lines()
        .find_map(|l| l.strip_prefix("loop_max_ms "))
        .unwrap()
        .parse()
        .unwrap();
    assert!(loop_max < 1000, "loop_max_ms {loop_max}");
    drop(alpha);
    drop(beta);
    no_spaces_left(pid).unwrap();
    let golden = root().join("browser/tests/ui/spaces_chips.png");
    let got = std::fs::read(&png).unwrap();
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
    } else {
        assert!(
            got == std::fs::read(&golden).unwrap(),
            "the chips differ from {}: {}",
            golden.display(),
            png.display()
        );
    }
}

/// P9.6: the person shows Alpha (its chip) and clicks in its page: its
/// tab is taken over (`taken_over`; its acting commands fail), Beta goes
/// on; handing back (`handback:1`) sends `handed_back` and Alpha acts again.
#[test]
fn window_spaces_takeover() {
    window_spaces_takeover_with(&[]);
}

/// Q2.12: the same with the Spaces hosted in the window (`HOSTED`).
#[test]
fn window_spaces_takeover_in_renderer() {
    window_spaces_takeover_with(HOSTED);
}

fn window_spaces_takeover_with(env: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let sock = format!("target/sockets/spaces-takeover-{}-{}.sock", std::process::id(), env.len());
    let window = spaces_window_env(
        &sock,
        &[
            "--spaces",
            "--events",
            "agents:2 click:690,20 key:escape click:700,500 agents:5 handback:1",
        ],
        env,
    );
    let pid = window.id();
    let mut a = AgentConn::open(&sock);
    a.send(r#"{"name":"Alpha"}"#);
    let (got, _) = a.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/form.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = a.ask(r#"{"id":9,"cmd":"snapshot"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    assert_eq!(a.line(), "{\"event\":\"taken_over\",\"space\":1,\"tab\":1}\n");
    let mut b = AgentConn::open(&sock);
    b.send(r#"{"name":"Beta"}"#);
    let (got, _) = b.ask(r#"{"id":1,"cmd":"navigate","url":"browser/tests/agent/site/index.html"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    let (got, _) = a.ask(r#"{"id":2,"cmd":"click","ref":"e4"}"#);
    assert_eq!(
        got,
        "{\"id\":2,\"ok\":false,\"error\":\"the user took over tab 1 (wait for handed_back)\"}\n"
    );
    let (got, _) = b.ask(r#"{"id":2,"cmd":"click","ref":"e6"}"#);
    assert!(got.contains("\"result\":\"clicked button"), "{got}");
    assert_eq!(a.line(), "{\"event\":\"handed_back\",\"space\":1,\"tab\":1}\n");
    let (got, _) = a.ask(r#"{"id":3,"cmd":"click","ref":"e4"}"#);
    assert!(got.contains("\"result\":\"checked checkbox"), "{got}");
    drop(a);
    drop(b);
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    no_spaces_left(pid).unwrap();
}

/// B9 R6 (P9.5): a click on a Space's chip shows the Space: the window
/// sends `watch`, the Space writes its page to its frame file and sends
/// `frame`, and the window draws the newest frame in its page area. The
/// agent's page is green; after the click the centre of the window's
/// page area is that green in `--save`'s PNG, and the window never waited
/// on the Space (`loop_max_ms` under 50).
#[test]
fn window_spaces_watch() {
    window_spaces_watch_with(&[]);
}

/// Q2.12: the same with the Spaces hosted in the window (`HOSTED`).
#[test]
fn window_spaces_watch_in_renderer() {
    window_spaces_watch_with(HOSTED);
}

fn window_spaces_watch_with(env: &[(&str, &str)]) {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let green = scratch().join("green.html");
    std::fs::write(&green, "<body style=\"background:#00aa00\">green</body>").unwrap();
    let sock = format!("target/sockets/spaces-watch-{}-{}.sock", std::process::id(), env.len());
    let png = scratch().join(format!("spaces_watch{}.png", env.len()));
    // With --font fixed in 800 px, Alpha's chip is 661-719 (y 9-31).
    let window = spaces_window_env(
        &sock,
        &[
            "--spaces",
            "--state",
            "--save",
            png.to_str().unwrap(),
            "--events",
            "agents:1 click:690,20 key:escape agents:2 key:cmd+q",
            "browser/tests/ui/long.html",
        ],
        env,
    );
    let pid = window.id();
    let mut a = AgentConn::open(&sock);
    a.send(r#"{"name":"Alpha"}"#);
    let (got, _) = a.ask(&format!(
        r#"{{"id":1,"cmd":"navigate","url":{}}}"#,
        serde_like(green.to_str().unwrap())
    ));
    assert!(got.contains("\"ok\":true"), "{got}");
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let (got, _) = a.ask(r#"{"id":2,"cmd":"spaces"}"#);
    assert!(
        got.contains("\"mine\":true") && got.contains("\"watched\":true"),
        "{got}"
    );
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success(),
        "the window: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    let state = text(&out.stdout);
    assert!(state.contains("\nviewport 800x532\n"), "{state}");
    let loop_max: u64 = state
        .lines()
        .find_map(|l| l.strip_prefix("loop_max_ms "))
        .unwrap()
        .parse()
        .unwrap();
    assert!(loop_max < 50, "loop_max_ms {loop_max}");
    drop(a);
    no_spaces_left(pid).unwrap();
    let script = r#"
import struct, sys, zlib
data = open(sys.argv[1], 'rb').read()
w, h = struct.unpack('>II', data[16:24])
chunks, offset = [], 8
while offset < len(data):
    n = int.from_bytes(data[offset:offset+4], 'big')
    if data[offset+4:offset+8] == b'IDAT':
        chunks.append(data[offset+8:offset+8+n])
    offset += n + 12
raw = zlib.decompress(b''.join(chunks))
x, y = w // 2, 600 - 532 + 532 // 2
i = y * (w * 4 + 1) + 1 + x * 4
print(raw[i], raw[i + 1], raw[i + 2])
"#;
    let Ok(py) = Command::new("python3")
        .args(["-c", script, png.to_str().unwrap()])
        .output()
    else {
        println!("skipped: no python3");
        return;
    };
    assert!(py.status.success(), "{}", text(&py.stderr));
    assert_eq!(text(&py.stdout), "0 170 0\n", "the centre of the page area");
}

/// Q2.12: a hosted Space's page runs in a renderer like a tab's (Q2.4c,
/// Q2.8b): with a profile, the window starts no `dweller space` process;
/// the Space's renderer, which shows a page that uses `document.cookie`
/// and `localStorage` over http, holds no file of the profile open and
/// no regular file but the binary and system libraries; it runs under
/// seccomp (Linux: `Seccomp: 2`), and its sandbox probe
/// (`DWELLER_TEST_SANDBOX_PROBE=1`) finds the profile, /tmp, TCP and fork
/// refused.
#[test]
fn window_spaces_renderer_sandboxed() {
    if !has_sdl() || !cfg!(any(target_os = "macos", target_os = "linux")) {
        println!("skipped: macOS or Linux with SDL2 only");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let home = profile("q212-home");
    let dir = home.join(".dweller");
    std::fs::create_dir_all(&dir).unwrap();
    let dir_text = dir.canonicalize().unwrap().to_str().unwrap().to_string();
    let sock = format!("target/sockets/q212-{}.sock", std::process::id());
    let envs = [
        HOSTED[0],
        ("HOME", home.to_str().unwrap()),
        ("DWELLER_PROFILE", dir.to_str().unwrap()),
        ("DWELLER_TEST", "1"),
        ("DWELLER_TEST_SANDBOX_PROBE", "1"),
    ];
    let window = spaces_window_env(&sock, &[], &envs);
    let pid = window.id();
    let mut conn = AgentConn::open(&sock);
    let page = format!("{}/storage/script-cookies.html", srv.base);
    let (got, _) = conn.ask(&format!("{{\"id\":1,\"cmd\":\"navigate\",\"url\":{}}}", serde_like(&page)));
    assert!(got.contains("\"ok\":true"), "{got}");
    let used = "(localStorage.setItem('q212', 'kept'), localStorage.getItem('q212') + ' ' + document.cookie)";
    let (got, _) = conn.ask(&format!("{{\"id\":2,\"cmd\":\"eval\",\"expression\":{}}}", serde_like(used)));
    assert!(got.contains("kept js=1"), "{got}");
    let processes = Command::new("pgrep").args(["-f", &format!("/tmp/dweller-{pid}-")]).output().unwrap();
    assert!(text(&processes.stdout).is_empty(), "a Space process: {}", text(&processes.stdout));
    let renderers: Vec<String> = spaces_of(pid).iter().map(|p| p.to_string()).collect();
    assert!(!renderers.is_empty(), "no renderer for the Space");
    let exe = dweller().canonicalize().unwrap();
    let system = ["/System/", "/usr/", "/Library/Fonts/", "/dev/"];
    for r in &renderers {
        let files = crate::window_renderer::open_files(r);
        assert!(!files.is_empty(), "no open files listed for {r}");
        for (fd, regular, name) in &files {
            assert!(!name.starts_with(&dir_text), "renderer {r} holds {name} (fd {fd}) of the profile");
            let allowed = !regular || fd == "txt" || Path::new(name) == exe || system.iter().any(|s| name.starts_with(s));
            assert!(allowed, "renderer {r} holds the regular file {name} (fd {fd}): {files:?}");
        }
        if cfg!(target_os = "linux") {
            let status = std::fs::read_to_string(format!("/proc/{r}/status")).unwrap();
            assert!(status.contains("\nSeccomp:\t2\n"), "renderer {r} is not under seccomp:\n{status}");
        }
    }
    drop(conn);
    let out = window.wait_with_output().unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "the window: {}\n{err}", out.status);
    let denied = ["sandbox-probe home=1 tmp=1 tcp=1 fork=1\n", "sandbox-probe home=13 tmp=1 tcp=1 fork=1\n"];
    assert!(denied.iter().any(|d| err.contains(d)), "{err}");
    assert!(dir.join("storage").exists() || dir.join("cookies").exists(), "the browser kept nothing in the profile");
    no_spaces_left(pid).unwrap();
}
