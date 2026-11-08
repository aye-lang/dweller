use super::*;

/// `browser/tests/agent/X.html`: `dweller dump --a11y` must print
/// `X.a11y` and `dweller dump --md` must print `X.md`; a few fixtures
/// also have `--boxes`, `--json` and `--refs` variants.
#[test]
fn agent_fixtures() {
    let dir = root().join("browser").join("tests").join("agent");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 15,
        "expected at least 15 agent fixtures, found {}",
        htmls.len()
    );
    let mut runs: Vec<(Vec<String>, PathBuf)> = Vec::new();
    for h in &htmls {
        let rel = h
            .strip_prefix(root())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        runs.push((vec!["--a11y".into(), rel.clone()], h.with_extension("a11y")));
        runs.push((vec!["--md".into(), rel], h.with_extension("md")));
    }
    let page = |name: &str| format!("browser/tests/agent/{name}.html");
    runs.push((
        vec!["--a11y".into(), "--boxes".into(), page("long_page")],
        dir.join("long_page.boxes.a11y"),
    ));
    runs.push((
        vec!["--a11y".into(), "--json".into(), page("form_login")],
        dir.join("form_login.json"),
    ));
    runs.push((
        vec![
            "--a11y".into(),
            "--json".into(),
            "--boxes".into(),
            page("tables"),
        ],
        dir.join("tables.boxes.json"),
    ));
    runs.push((
        vec!["--md".into(), "--refs".into(), page("article")],
        dir.join("article.refs.md"),
    ));
    let mut failures = Vec::new();
    for (args, golden) in &runs {
        let mut all = vec!["dump"];
        all.extend(args.iter().map(|s| s.as_str()));
        let out = run(&all);
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{args:?}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        check_golden(
            golden,
            &rooted(&text(&out.stdout)),
            &format!("{args:?}"),
            &mut failures,
        );
    }
    assert!(
        failures.is_empty(),
        "{} of {} agent dumps failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n")
    );
}

/// `browser/tests/agent/sessions/X.jsonl`: requests fed to `dweller
/// agent` (or `dweller mcp` for `X.mcp.jsonl`) on stdin; the responses
/// must be `X.out` (`X.mcp.out`).
#[test]
fn agent_sessions() {
    agent_sessions_with(&[]);
}

/// Q2.3b: with `DWELLER_PROCESS=tab` the agent's commands are answered by
/// a renderer process, with the same expected output.
#[test]
fn agent_sessions_in_renderer() {
    agent_sessions_with(&[("DWELLER_PROCESS", "tab")]);
}

/// Q2.3b: a renderer that answers a request id it was not sent is killed.
#[test]
fn agent_sessions_bad_reply() {
    let out = relay_snapshot(&[("DWELLER_TEST", "1"), ("DWELLER_TEST_EVIL", "agent")]);
    assert!(!out.status.success(), "the relay goes on after a bad answer");
    assert!(out.stdout.is_empty(), "a bad answer reached the agent: {}", text(&out.stdout));
    assert!(
        text(&out.stderr).contains("bad message: bad agent_reply "),
        "no bad agent_reply: {}",
        text(&out.stderr)
    );
}

/// Q2.7b: a compromise hook without `DWELLER_TEST=1` does nothing: the
/// renderer answers as it would without it.
#[test]
fn agent_sessions_hook_needs_test() {
    let out = relay_snapshot(&[("DWELLER_TEST_EVIL", "agent")]);
    assert!(out.status.success(), "{}: {}", out.status, text(&out.stderr));
    assert!(text(&out.stdout).starts_with("{\"id\":1,"), "{}", text(&out.stdout));
    assert!(!text(&out.stderr).contains("bad message"), "{}", text(&out.stderr));
}

// `dweller agent` with `DWELLER_PROCESS=tab` and `env`, asked one snapshot.
fn relay_snapshot(env: &[(&str, &str)]) -> std::process::Output {
    use std::io::Write;
    let mut child = dweller_command()
        .current_dir(root())
        .arg("agent")
        .env("DWELLER_PROCESS", "tab")
        .env_remove("DWELLER_TEST")
        .envs(env.iter().copied())
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"{\"id\":1,\"cmd\":\"snapshot\"}\n")
        .unwrap();
    child.wait_with_output().unwrap()
}

fn agent_sessions_with(env: &[(&str, &str)]) {
    use std::io::Write;
    let dir = root()
        .join("browser")
        .join("tests")
        .join("agent")
        .join("sessions");
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    inputs.sort();
    assert!(
        inputs.len() >= 4,
        "expected at least 4 agent sessions, found {}",
        inputs.len()
    );
    let mut failures = Vec::new();
    for input in &inputs {
        let name = input.file_name().unwrap().to_string_lossy().into_owned();
        let mode = if name.ends_with(".mcp.jsonl") {
            "mcp"
        } else {
            "agent"
        };
        let mut child = dweller_command()
            .current_dir(root())
            .arg(mode)
            .envs(env.iter().copied())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&std::fs::read(input).unwrap())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{name}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        check_golden(
            &input.with_extension("out"),
            &rooted(&text(&out.stdout)),
            &name,
            &mut failures,
        );
    }
    assert!(
        failures.is_empty(),
        "{} of {} agent sessions failed:\n\n{}",
        failures.len(),
        inputs.len(),
        failures.join("\n")
    );
}

#[test]
fn agent_errors() {
    let usage = "error: usage: dweller dump --a11y [--boxes] [--json] | --md [--refs] [--js] [--width PX] [--no-scroll | --scroll-steps N] FILE|URL\n";
    expect(&["dump", "--a11y"], 2, "", usage);
    expect(
        &[
            "dump",
            "--md",
            "--width",
            "x",
            "browser/tests/agent/article.html",
        ],
        2,
        "",
        usage,
    );
    expect(
        &["dump", "--md", "browser/tests/agent/missing.html"],
        1,
        "",
        &format!(
            "error: cannot fetch file://{}/browser/tests/agent/missing.html: not found\n",
            root().display()
        ),
    );
    expect(
        &["agent", "--color"],
        2,
        "",
        "error: usage: dweller agent|mcp [--width PX] [--height PX] [--no-js]\n",
    );
    expect(
        &["dump", "--a11y", "data:text/html,<button>OK</button>"],
        0,
        "- button \"OK\" [e1]\n",
        "",
    );
    expect(
        &["dump", "--md", "data:text/html,<h2>Hi</h2><p>a <b>b</b>"],
        0,
        "## Hi\n\na **b**\n",
        "",
    );
}

fn base64_decode(s: &str) -> Vec<u8> {
    let digit = |c: u8| match c {
        b'A'..=b'Z' => c - b'A',
        b'a'..=b'z' => c - b'a' + 26,
        b'0'..=b'9' => c - b'0' + 52,
        b'+' => 62,
        b'/' => 63,
        _ => panic!("not base64: {:?}", c as char),
    };
    let (mut out, mut acc, mut bits) = (Vec::new(), 0u32, 0);
    for &c in s.as_bytes().iter().filter(|&&c| c != b'=') {
        acc = acc << 6 | digit(c) as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((acc >> bits) as u8);
        }
    }
    out
}

/// The string value of `"key":"..."` in one JSON line (no escapes in it).
fn json_str<'a>(line: &'a str, key: &str) -> &'a str {
    let k = format!("\"{key}\":\"");
    let at = line
        .find(&k)
        .unwrap_or_else(|| panic!("no {key} in {line}"))
        + k.len();
    &line[at..at + line[at..].find('"').unwrap()]
}

/// `screenshot` in `dweller agent` and `browser_screenshot` in `dweller
/// mcp`: PNGs (base64) of the viewport, the whole page and one element.
#[test]
fn agent_screenshot() {
    use std::io::Write;
    let session = |mode: &str, input: &str| -> Vec<String> {
        let mut child = dweller_command()
            .current_dir(root())
            .args([mode, "--width", "400", "--height", "300"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        text(&out.stdout).lines().map(String::from).collect()
    };
    let png_size = |b64: &str, name: &str| {
        let path = scratch().join(name);
        std::fs::write(&path, base64_decode(b64)).unwrap();
        check_png(&path)
    };
    let lines = session(
        "agent",
        concat!(
            "{\"id\":1,\"cmd\":\"screenshot\"}\n",
            "{\"id\":2,\"cmd\":\"navigate\",\"url\":\"browser/tests/agent/form_login.html\"}\n",
            "{\"id\":3,\"cmd\":\"screenshot\",\"ref\":\"e2\"}\n",
            "{\"id\":4,\"cmd\":\"screenshot\",\"ref\":\"e99\"}\n",
            "{\"id\":5,\"cmd\":\"snapshot\"}\n",
            "{\"id\":6,\"cmd\":\"navigate\",\"url\":\"browser/tests/paint/full.html\"}\n",
            "{\"id\":7,\"cmd\":\"screenshot\"}\n",
            "{\"id\":8,\"cmd\":\"screenshot\",\"full\":true}\n",
        ),
    );
    assert_eq!(lines.len(), 8);
    assert_eq!(
        lines[0],
        "{\"id\":1,\"ok\":false,\"error\":\"no page: navigate first\"}"
    );
    let (w, h) = png_size(json_str(&lines[2], "png"), "agent_ref.png");
    assert!(w < 400 && h < 300, "element {w}x{h}");
    assert!(
        lines[2].contains("\"result\":\"screenshot of textbox \\\"Email address\\\" [e2]"),
        "{}",
        lines[2]
    );
    assert!(
        lines[3].contains("\"ok\":false,\"error\":\"unknown ref e99"),
        "{}",
        lines[3]
    );
    // The page is laid out as before the screenshots: refs keep working.
    assert!(
        lines[4].contains("textbox \\\"Email address\\\" [e2]"),
        "{}",
        lines[4]
    );
    assert!(
        lines[6].contains("\"result\":\"screenshot of the viewport, 400x300 px"),
        "{}",
        lines[6]
    );
    assert_eq!(
        png_size(json_str(&lines[6], "png"), "agent_viewport.png"),
        (400, 300)
    );
    let (w, h) = png_size(json_str(&lines[7], "png"), "agent_full.png");
    assert!(w == 400 && h > 300, "whole page {w}x{h}");
    assert!(
        json_str(&lines[7], "result")
            .starts_with(&format!("screenshot of the whole page, 400x{h} px")),
        "{}",
        lines[7]
    );

    let lines = session(
        "mcp",
        concat!(
            "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"tools/call\",\"params\":{\"name\":\"browser_screenshot\",\"arguments\":{}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/call\",\"params\":{\"name\":\"browser_navigate\",\"arguments\":{\"url\":\"browser/tests/agent/links.html\"}}}\n",
            "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"browser_screenshot\",\"arguments\":{}}}\n",
        ),
    );
    assert_eq!(lines.len(), 3);
    assert!(
        lines[0].contains("\"isError\":true") && !lines[0].contains("\"image\""),
        "{}",
        lines[0]
    );
    assert!(
        lines[2].contains("\"type\":\"image\"")
            && lines[2].contains("\"mimeType\":\"image/png\"")
            && lines[2].contains("\"isError\":false"),
        "{}",
        &lines[2][..200.min(lines[2].len())]
    );
    assert_eq!(
        png_size(json_str(&lines[2], "data"), "mcp_viewport.png"),
        (400, 300)
    );
}

/// Q3.12: the window's socket is `0600` in a `0700` directory it makes,
/// next to a `0600` token file; a connection whose first line is not
/// `{"token":...}` with that token is refused and runs nothing, and a
/// directory other users can open is refused.
#[test]
fn window_agent_socket_token() {
    use std::io::{Read, Write};
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let dir = format!("/tmp/dw-{}-token", std::process::id());
    let _ = std::fs::remove_dir_all(&dir);
    let sock = format!("{dir}/agent.sock");
    let window = spaces_window(&sock, &["--state", "browser/tests/basic.html"]);
    let mode = |p: &str| std::fs::metadata(root().join(p)).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&dir), 0o700);
    assert_eq!(mode(&sock), 0o600);
    assert_eq!(mode(&format!("{sock}.token")), 0o600);
    let token = std::fs::read_to_string(root().join(format!("{sock}.token"))).unwrap();
    assert!(token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit()), "{token}");
    let refused = "{\"id\":null,\"ok\":false,\"error\":\"no token: the first line must be {\\\"token\\\":\\\"...\\\"} with the token in the socket's .token file\"}\n";
    let wrong = format!("{{\"token\":\"{}\"}}\n", "0".repeat(64));
    for first in ["", wrong.as_str()] {
        let mut s = std::os::unix::net::UnixStream::connect(root().join(&sock)).unwrap();
        s.write_all(format!("{first}{{\"id\":1,\"cmd\":\"tab_new\"}}\n").as_bytes()).unwrap();
        let mut got = String::new();
        s.read_to_string(&mut got).unwrap();
        assert_eq!(got, refused);
    }
    let mut conn = AgentConn::open(&sock);
    let (got, _) = conn.ask(r#"{"id":1,"cmd":"tabs"}"#);
    assert!(got.contains("\"ok\":true"), "{got}");
    conn.stream.shutdown(std::net::Shutdown::Write).unwrap();
    assert_eq!(conn.line(), "");
    let out = window.wait_with_output_within(std::time::Duration::from_secs(30)).unwrap();
    assert!(out.status.success() && out.stderr.is_empty(), "{}", text(&out.stderr));
    assert!(text(&out.stdout).contains("tabs 1, showing 1\n"), "{}", text(&out.stdout));
    assert!(!root().join(&sock).exists() && !root().join(format!("{sock}.token")).exists());
    let open = format!("/tmp/dw-{}-open", std::process::id());
    let _ = std::fs::remove_dir_all(&open);
    std::fs::DirBuilder::new().mode(0o755).create(&open).unwrap();
    std::fs::set_permissions(&open, std::fs::Permissions::from_mode(0o755)).unwrap();
    let out = run_window(&["--agent-socket", &format!("{open}/x.sock"), "--frames", "1"]);
    assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
    assert_eq!(
        text(&out.stderr),
        format!("error: cannot listen on {open}/x.sock: other users can open its directory (it must be 0700)\n")
    );
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&open);
}

/// `dweller connect` and `--agent-socket` errors.
#[test]
fn window_agents_errors() {
    expect(
        &["connect"],
        2,
        "",
        "error: usage: dweller connect SOCKET [--mcp] [--name NAME] [--space N]\n",
    );
    expect(
        &["connect", "target/no-such.sock"],
        1,
        "",
        "error: cannot connect to target/no-such.sock: is a window listening (dweller --agent-socket target/no-such.sock)?\n",
    );
    if has_sdl() {
        let out = run_window(&[
            "--agent-socket",
            "target/no-such-dir/sub/x.sock",
            "--frames",
            "1",
        ]);
        assert_eq!(out.status.code(), Some(1), "{}", text(&out.stderr));
        assert_eq!(
            text(&out.stderr),
            "error: cannot listen on target/no-such-dir/sub/x.sock\n"
        );
    }
    expect(
        &["--exit-when-idle", "browser/tests/basic.html"],
        2,
        "",
        &format!("error: {}\n", WINDOW_USAGE),
    );
}

/// The agent over HTTP: a link followed from a page served by the test
/// server.
#[test]
fn agent_http() {
    use std::io::Write;
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = &srv.base;
    let mut child = dweller_command()
        .current_dir(root())
        .arg("agent")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let input = format!(
        "{{\"id\":1,\"cmd\":\"navigate\",\"url\":\"{base}/agent/site/index.html\"}}\n{{\"id\":2,\"cmd\":\"click\",\"ref\":\"e1\"}}\n"
    );
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    let lines: Vec<&str> = got.lines().collect();
    assert_eq!(lines.len(), 2, "{got}");
    assert!(lines[0].contains("\"title\":\"Site home\""), "{got}");
    assert!(
        lines[1].contains(&format!("\"url\":\"{base}/agent/site/docs.html\"")),
        "{got}"
    );
    assert!(
        lines[1].contains("heading \\\"Documentation\\\" [level=1] [e10]"),
        "{got}"
    );
}

/// Form submission over HTTP: `browser/tests/agent/http/X.jsonl` (with
/// `$BASE` for the test server's address) fed to `dweller agent` must
/// print `X.out`. The forms in `browser/tests/forms/` submit to the
/// server's `/echo`, whose page shows the request (method, path,
/// Content-Type, fields, raw body).
#[test]
fn agent_forms_http() {
    use std::io::Write;
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let dir = root()
        .join("browser")
        .join("tests")
        .join("agent")
        .join("http");
    let mut inputs: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "jsonl"))
        .collect();
    inputs.sort();
    assert!(
        inputs.len() >= 2,
        "expected at least 2 HTTP sessions, found {}",
        inputs.len()
    );
    let mut failures = Vec::new();
    for input in &inputs {
        let name = input.file_name().unwrap().to_string_lossy().into_owned();
        let mut child = dweller_command()
            .current_dir(root())
            .arg("agent")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        let requests = std::fs::read_to_string(input)
            .unwrap()
            .replace("$BASE", &srv.base);
        child
            .stdin
            .take()
            .unwrap()
            .write_all(requests.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{name}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        let got = rooted(&text(&out.stdout)).replace(&srv.base, "$BASE");
        check_golden(&input.with_extension("out"), &got, &name, &mut failures);
    }
    assert!(
        failures.is_empty(),
        "{} of {} HTTP sessions failed:\n\n{}",
        failures.len(),
        inputs.len(),
        failures.join("\n")
    );
}

/// Q2.7c: `dweller agent` with `DWELLER_PROCESS=tab` follows a
/// navigation to another site into a new renderer, and the old page's
/// refs are refused there.
#[test]
fn agent_sessions_relay_sites() {
    use std::io::{BufRead, Write};
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/ui/renderer_sites.html", srv.base);
    let other = page.replace("127.0.0.1", "localhost");
    let mut child = dweller_command()
        .current_dir(root())
        .arg("agent")
        .env("DWELLER_PROCESS", "tab")
        .env_remove("DWELLER_TEST")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = std::io::BufReader::new(child.stdout.take().unwrap());
    let pid = child.id().to_string();
    let mut ask = move |line: String| {
        stdin.write_all(format!("{line}\n").as_bytes()).unwrap();
        let mut got = String::new();
        stdout.read_line(&mut got).unwrap();
        got
    };
    let kids = || text(&Command::new("pgrep").args(["-P", &pid]).output().unwrap().stdout);
    let went = ask(format!("{{\"id\":1,\"cmd\":\"navigate\",\"url\":\"{page}\"}}"));
    assert!(went.contains("\"ok\":true"), "{went}");
    let before = kids();
    let old = ref_named(&ask("{\"id\":2,\"cmd\":\"snapshot\"}".into()), "same site");
    let went = ask(format!("{{\"id\":3,\"cmd\":\"navigate\",\"url\":\"{other}\"}}"));
    assert!(went.starts_with("{\"id\":3,\"ok\":true,") && went.contains(&format!("\"url\":\"{other}\"")), "{went}");
    let after = kids();
    let clicked = ask(format!("{{\"id\":4,\"cmd\":\"click\",\"ref\":\"{old}\"}}"));
    assert!(clicked.contains(&format!("\"ok\":false,\"error\":\"unknown ref {old}:")), "{clicked}");
    let title = ask("{\"id\":5,\"cmd\":\"eval\",\"expression\":\"document.title\"}".into());
    assert!(title.contains("\"result\":\"localhost\""), "{title}");
    drop(ask);
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success(), "{}: {}", out.status, text(&out.stderr));
    assert!(!before.is_empty() && !after.is_empty() && before != after, "no swap: {before:?} {after:?}");
}
