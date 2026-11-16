use super::*;

const TYPED: &str = "click:36,134 click:36,134 click:36,134 frame click:98,134 click:36,134 frame click:120,176 text:hello_world key:left key:backspace";

fn native_counter_exe() -> &'static Path {
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| {
        let exe = scratch().join("native-counter");
        let out = aye()
            .args(["build", "examples/native/counter", "--mode", "safe", "--leak-check", "-o"])
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "building examples/native/counter failed:\n{}",
            text(&out.stderr)
        );
        exe
    })
}

fn native_counter_run(events: &str, shot: &Path, extra: &[&str]) -> String {
    let out = Command::new(native_counter_exe())
        .env("SDL_VIDEODRIVER", "dummy")
        .current_dir(root())
        .args(["--font", "fixed", "--width", "400", "--height", "300"])
        .args(["--frames", "8", "--events", events, "--save"])
        .arg(shot)
        .args(extra)
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    text(&out.stdout)
}

fn native_counter_png(golden: &Path, got: &Path, failures: &mut Vec<String>) {
    let image = std::fs::read(got).unwrap();
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(golden, &image).unwrap();
        return;
    }
    if image != std::fs::read(golden).unwrap_or_default() {
        failures.push(format!(
            "the image differs from {} (new one: {})",
            golden.display(),
            got.display()
        ));
    }
}

/// V2: an Aye program on the engine without the browser. Clicks change
/// the DOM built from code, text goes into the focused field, the wheel
/// scrolls 1000 rows; the changes are laid out incrementally and give the
/// pixels of a full layout.
#[test]
fn native_counter() {
    let expected = root().join("examples/native/counter/expected");
    let dir = scratch();
    let mut failures = Vec::new();

    let typed = dir.join("native-typed.png");
    native_counter_run(TYPED, &typed, &[]);
    native_counter_png(&expected.join("typed.png"), &typed, &mut failures);

    let added = format!("{TYPED} frame click:258,176 frame wheel:-2000");
    let last = dir.join("native-final.png");
    let dump = native_counter_run(&added, &last, &["--dump"]);
    check_golden(&expected.join("final.tree"), &dump, "native counter", &mut failures);
    native_counter_png(&expected.join("final.png"), &last, &mut failures);

    let timing = native_counter_run(&added, &last, &["--timing"]);
    assert!(
        timing.contains("\nfull-layouts 1\n") && !timing.contains("partial-layouts 0\n"),
        "{timing}"
    );

    let full = dir.join("native-full.png");
    native_counter_run(&added, &full, &["--full-layout"]);
    assert!(
        std::fs::read(&full).unwrap() == std::fs::read(&last).unwrap(),
        "the incremental layout's pixels ({}) differ from a full layout's ({})",
        last.display(),
        full.display()
    );

    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn native_agent_session(mut command: Command, requests: &str) -> String {
    use std::io::Write;
    use std::process::Stdio;
    let mut child = command
        .current_dir(root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(requests.as_bytes()).unwrap();
    let out = child.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    text(&out.stdout)
}

/// V5: the JSON lines of `docs/agent-protocol.md` on a native app
/// (`counter --agent`). The same requests to `dweller agent` on the same
/// app as a page (`counter.html`) get the same answers, but for the URL.
#[test]
fn native_counter_agent() {
    let dir = root().join("examples/native/counter");
    let requests = std::fs::read_to_string(dir.join("expected/agent.jsonl")).unwrap();

    let mut app = Command::new(native_counter_exe());
    app.arg("--agent");
    let got = native_agent_session(app, &requests);
    let mut failures = Vec::new();
    check_golden(&dir.join("expected/agent.out"), &got, "native counter agent", &mut failures);
    assert!(failures.is_empty(), "{}", failures.join("\n"));

    let page = dir.join("counter.html");
    let navigate = format!("{{\"cmd\":\"navigate\",\"url\":\"{}\"}}\n", page.display());
    let mut browser = dweller_command();
    browser.arg("agent");
    let web = native_agent_session(browser, &format!("{navigate}{requests}"));
    let address = format!("\"url\":\"file://{}\"", page.display());
    let web: String = web
        .lines()
        .skip(1)
        .map(|l| l.replace(&address, "\"url\":\"app:counter\"") + "\n")
        .collect();
    assert!(got == web, "the app's answers:\n{got}\nthe page's answers:\n{web}");
}
