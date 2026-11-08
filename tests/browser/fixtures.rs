use super::*;

#[test]
fn fixtures() {
    let dir = root().join("browser").join("tests");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 20,
        "expected at least 20 fixtures, found {}",
        htmls.len()
    );
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut failures = Vec::new();
    for html in &htmls {
        let rel = html
            .strip_prefix(root())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        for (flag, ext) in [("--tree", "tree"), ("--text", "txt")] {
            let out = run(&["dump", flag, &rel]);
            let got = text(&out.stdout);
            let golden = html.with_extension(ext);
            if !out.status.success() || !out.stderr.is_empty() {
                failures.push(format!(
                    "{rel} {flag}: {}\n{}",
                    out.status,
                    text(&out.stderr)
                ));
                continue;
            }
            if bless {
                std::fs::write(&golden, &got).unwrap();
                continue;
            }
            let want = std::fs::read_to_string(&golden).unwrap_or_default();
            if got != want {
                failures.push(format!(
                    "{rel} {flag}: output differs from {}\n--- expected\n{want}--- got\n{got}",
                    golden.display()
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} fixture runs failed:\n\n{}",
        failures.len(),
        htmls.len() * 2,
        failures.join("\n")
    );
}

/// `browser/tests/style/X.html`: `dweller dump --style` must print
/// `X.style`; `media.html` is also dumped 400px wide into
/// `media.w400.style`.
#[test]
fn style_fixtures() {
    let dir = root().join("browser").join("tests").join("style");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 20,
        "expected at least 20 style fixtures, found {}",
        htmls.len()
    );
    let mut runs: Vec<(Vec<String>, PathBuf)> = htmls
        .iter()
        .map(|h| {
            (
                vec![
                    h.strip_prefix(root())
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                ],
                h.with_extension("style"),
            )
        })
        .collect();
    runs.push((
        vec![
            "--width".into(),
            "400".into(),
            "browser/tests/style/media.html".into(),
        ],
        dir.join("media.w400.style"),
    ));
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut failures = Vec::new();
    for (args, golden) in &runs {
        let mut all = vec!["dump", "--style"];
        all.extend(args.iter().map(|s| s.as_str()));
        let out = run(&all);
        let got = text(&out.stdout);
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{args:?}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        if bless {
            std::fs::write(golden, &got).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(golden).unwrap_or_default();
        if got != want {
            failures.push(format!(
                "{args:?}: output differs from {}\n--- expected\n{want}--- got\n{got}",
                golden.display()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} style runs failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n")
    );
}

/// `browser/tests/layout/X.html`: `dweller dump --layout` (fixed-metric
/// font, 800x600 viewport) must print `X.layout`; `percentages.html` is
/// also laid out 400px wide into `percentages.w400.layout`.
#[test]
fn layout_fixtures() {
    let dir = root().join("browser").join("tests").join("layout");
    let mut htmls: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    htmls.sort();
    assert!(
        htmls.len() >= 25,
        "expected at least 25 layout fixtures, found {}",
        htmls.len()
    );
    let mut runs: Vec<(Vec<String>, PathBuf)> = htmls
        .iter()
        .map(|h| {
            (
                vec![
                    h.strip_prefix(root())
                        .unwrap()
                        .to_string_lossy()
                        .into_owned(),
                ],
                h.with_extension("layout"),
            )
        })
        .collect();
    runs.push((
        vec![
            "--width".into(),
            "400".into(),
            "browser/tests/layout/percentages.html".into(),
        ],
        dir.join("percentages.w400.layout"),
    ));
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut failures = Vec::new();
    for (args, golden) in &runs {
        let mut all = vec!["dump", "--layout"];
        all.extend(args.iter().map(|s| s.as_str()));
        let out = run(&all);
        let got = text(&out.stdout);
        if !out.status.success() || !out.stderr.is_empty() {
            failures.push(format!("{args:?}: {}\n{}", out.status, text(&out.stderr)));
            continue;
        }
        if bless {
            std::fs::write(golden, &got).unwrap();
            continue;
        }
        let want = std::fs::read_to_string(golden).unwrap_or_default();
        if got != want {
            failures.push(format!(
                "{args:?}: output differs from {}\n--- expected\n{want}--- got\n{got}",
                golden.display()
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} of {} layout runs failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n")
    );
}
