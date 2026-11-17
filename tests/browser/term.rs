use super::*;

pub(super) fn term_exe() -> &'static Path {
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| {
        let exe = scratch().join("term");
        let out = aye()
            .args(["build", "apps/term", "--mode", "safe", "--leak-check", "-o"])
            .arg(&exe)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "building apps/term failed:\n{}",
            text(&out.stderr)
        );
        exe
    })
}

/// V4: recorded byte streams (`apps/term/tests/X.bytes`: vim, htop, less
/// on an 80x24 pty, and two hand-made ones) through the VT parser; the
/// cell grid must be `X.grid`.
#[test]
fn term_vt_streams() {
    let dir = root().join("apps/term/tests");
    let mut streams: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "bytes"))
        .collect();
    streams.sort();
    assert!(streams.len() >= 7, "only {} streams in {}", streams.len(), dir.display());
    let mut failures = Vec::new();
    for stream in &streams {
        let out = Command::new(term_exe())
            .arg("dump")
            .arg(stream)
            .output()
            .unwrap();
        let name = stream.file_name().unwrap().to_string_lossy().into_owned();
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{name}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        check_golden(&stream.with_extension("grid"), &text(&out.stdout), &name, &mut failures);
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// V4: the cell grid drawn by the text stack (`term shot`, fixed font).
#[test]
fn term_shot() {
    let dir = root().join("apps/term/tests");
    let mut failures = Vec::new();
    for name in ["vim-edit", "htop", "colors"] {
        let got = scratch().join(format!("term-{name}.png"));
        let out = Command::new(term_exe())
            .arg("shot")
            .arg(dir.join(format!("{name}.bytes")))
            .arg(&got)
            .output()
            .unwrap();
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{name}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        let golden = dir.join(format!("{name}.png"));
        let image = std::fs::read(&got).unwrap();
        if std::env::var_os("DWELLER_BLESS").is_some() {
            std::fs::write(&golden, &image).unwrap();
        } else if image != std::fs::read(&golden).unwrap_or_default() {
            failures.push(format!(
                "{name}: the image differs from {} (new one: {})",
                golden.display(),
                got.display()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// V4: the window runs a command on a pty: its output is parsed, typed
/// keys reach it (the tty echoes them), it sees 80x24, and the window
/// closes when it exits.
#[test]
fn term_pty_window() {
    let out = Command::new(term_exe())
        .env("SDL_VIDEODRIVER", "dummy")
        .args(["--font", "fixed", "--dump", "--type", "hello\\r", "--exec"])
        .arg("printf '\\033[31mred\\033[0m plain\\n'; read line; echo \"got:$line\"; stty size")
        .output()
        .unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let got = text(&out.stdout);
    // The echo of the typed line and the command's first line come in
    // either order.
    let mut rows: Vec<&str> = got.lines().skip(1).take(4).collect();
    rows.sort();
    assert!(
        got.starts_with("size 80x24 cursor 4,0 scrollback 0\n")
            && rows == ["|24 80|", "|got:hello|", "|hello|", "|red plain|"],
        "{got}"
    );
    assert!(got.contains(": 0-3 fg=1;\n"), "{got}");
}

#[test]
fn term_unit_tests() {
    let out = aye().args(["test", "apps/term"]).output().unwrap();
    let stdout = text(&out.stdout);
    assert!(
        out.status.success() && stdout.contains(" passed; 0 failed"),
        "`aye test apps/term` failed:\n{stdout}{}",
        text(&out.stderr)
    );
}
