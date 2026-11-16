use super::*;

fn term_exe() -> &'static Path {
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
