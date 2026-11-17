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

fn native_base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | *chunk.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            if k <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * k) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}

/// The lines of a tree dump under the first line that is `opening`,
/// without that line's indentation.
fn native_subtree(dump: &str, opening: &str) -> String {
    let mut lines = dump.lines().skip_while(|l| l.trim_start() != opening);
    let depth = lines.next().map_or(0, |l| l.len() - l.trim_start().len());
    lines
        .take_while(|l| l.len() - l.trim_start().len() > depth)
        .map(|l| format!("{}\n", &l[depth + 2..]))
        .collect()
}

/// V3: one component (`examples/native/counter/widget`), two targets. It
/// is the native app's whole document, and, built for wasm32, it runs in
/// a page of `dweller` over `widget/glue.js`. After the same clicks and
/// the same typed text, the two documents are equal.
#[test]
fn native_counter_component() {
    let dir = root().join("examples/native/counter");
    let wasm = scratch().join("widget.wasm");
    let built = aye()
        .args(["build", "examples/native/counter/widget", "--target", "wasm32-freestanding", "-o"])
        .arg(&wasm)
        .output()
        .unwrap();
    let has_linker = Command::new("wasm-ld").arg("--version").output().is_ok();
    if !built.status.success() && !has_linker {
        eprintln!("native_counter_component: no wasm-ld here; only the native half runs");
        return;
    }
    assert!(built.status.success(), "{}", text(&built.stderr));

    let page = scratch().join("component.html");
    std::fs::write(
        &page,
        format!(
            "<!doctype html>\n<div id=\"root\"></div>\n<script src=\"file://{}\"></script>\n<script>\n\
             var bytes = Uint8Array.from(atob('{}'), function (c) {{ return c.charCodeAt(0); }});\n\
             mountComponent(document.getElementById('root'), bytes).then(function () {{\n\
             var b = document.querySelectorAll('button');\n\
             b[0].click(); b[0].click(); b[0].click(); b[1].click(); b[0].click();\n\
             document.getElementById('name').value = 'hello word';\n\
             b[2].click();\n\
             }});\n</script>\n",
            dir.join("widget/glue.js").display(),
            native_base64(&std::fs::read(&wasm).unwrap())
        ),
    )
    .unwrap();
    let web = run(&["dump", "--tree", "--js", page.to_str().unwrap()]);
    assert!(web.status.success(), "{}", text(&web.stderr));
    let in_page = native_subtree(&text(&web.stdout), "<div id=\"root\">");

    let events = format!("{TYPED} frame click:258,176");
    let shot = scratch().join("native-component.png");
    let in_app = native_subtree(&native_counter_run(&events, &shot, &["--dump"]), "<body>");

    assert!(in_app.lines().count() > 2000, "{in_app}");
    assert!(in_app == in_page, "the app's document:\n{in_app}\nthe page's:\n{in_page}");
}

/// V7: the same component in a terminal. `counter --tui` lays the
/// document out in cells (1 CSS px is one cell) and writes one frame as
/// ANSI; the frame is read back by the VT parser of `apps/term` and its
/// cell grid compared.
#[test]
fn native_counter_tui() {
    let expected = root().join("examples/native/counter/expected");
    let typed = "click:3,5 click:3,5 click:4,7 text:typed_text";
    let added = format!("{typed} click:26,7 wheel:-25");
    let mut failures = Vec::new();
    for (name, events) in [("tui", typed), ("tui-added", added.as_str())] {
        let frame = Command::new(native_counter_exe())
            .args(["--tui", "--width", "40", "--height", "12", "--events", events])
            .output()
            .unwrap();
        assert!(
            frame.status.success() && frame.stderr.is_empty(),
            "{}\n{}",
            frame.status,
            text(&frame.stderr)
        );
        let file = scratch().join(format!("{name}.ansi"));
        std::fs::write(&file, &frame.stdout).unwrap();
        let grid = Command::new(super::term::term_exe())
            .arg("dump")
            .arg(&file)
            .args(["--cols", "40", "--rows", "12"])
            .output()
            .unwrap();
        assert!(grid.status.success(), "{}", text(&grid.stderr));
        check_golden(&expected.join(format!("{name}.grid")), &text(&grid.stdout), name, &mut failures);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
