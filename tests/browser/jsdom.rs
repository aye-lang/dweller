use super::*;

/// One run of a DOM fixture: `args` then the page; stdout must be
/// `golden`, and stderr (console messages, uncaught exceptions) must be
/// `err_golden`, or empty when there is no such file.
fn jsdom_case(
    args: &[&str],
    page: &Path,
    golden: &Path,
    err_golden: Option<&Path>,
    bless: bool,
) -> Option<String> {
    let rel = page
        .strip_prefix(root())
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let mut all: Vec<&str> = args.to_vec();
    all.push(&rel);
    // Each forced layout that keeps boxes is compared with a full one; a
    // difference goes to stderr ("LAYOUT CHECK").
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_LAYOUT_CHECK", "1")
        .env("DWELLER_ALLOW_FILE_ACCESS_FROM_FILES", "1")
        .args(&all)
        .output()
        .unwrap();
    let got = rooted(&text(&out.stdout));
    let stderr = rooted(&text(&out.stderr));
    if bless {
        std::fs::write(golden, &got).unwrap();
        if let Some(e) = err_golden {
            if stderr.is_empty() {
                let _ = std::fs::remove_file(e);
            } else {
                std::fs::write(e, &stderr).unwrap();
            }
        }
        return None;
    }
    let mut problems = Vec::new();
    if !out.status.success() {
        problems.push(format!("status {}", out.status));
    }
    let want = std::fs::read_to_string(golden).unwrap_or_default();
    if got != want {
        problems.push(format!(
            "stdout differs from {}\n--- expected\n{want}--- got\n{got}",
            golden.display()
        ));
    }
    let want_err = err_golden
        .and_then(|e| std::fs::read_to_string(e).ok())
        .unwrap_or_default();
    if stderr != want_err {
        problems.push(format!(
            "stderr differs\n--- expected\n{want_err}--- got\n{stderr}"
        ));
    }
    if problems.is_empty() {
        None
    } else {
        Some(format!("{rel} {args:?}: {}", problems.join("\n")))
    }
}

/// `browser/tests/jsdom/X.html`: `dweller dump --tree --js` must print
/// `X.tree` (the document after its scripts and timers ran), with the
/// console on stderr equal to `X.err` (or empty). `X.style` and `X.a11y`,
/// when they exist, are `dump --style --js` and `dump --a11y --js`.
#[test]
fn jsdom_fixtures() {
    let dir = root().join("browser").join("tests").join("jsdom");
    let mut pages: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    pages.sort();
    assert!(
        pages.len() >= 25,
        "expected at least 25 DOM fixtures, found {}",
        pages.len()
    );
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut runs: Vec<(Vec<&str>, PathBuf, PathBuf, Option<PathBuf>)> = Vec::new();
    for p in &pages {
        runs.push((
            vec!["dump", "--tree", "--js"],
            p.clone(),
            p.with_extension("tree"),
            Some(p.with_extension("err")),
        ));
        for (flag, ext) in [("--style", "style"), ("--a11y", "a11y")] {
            let golden = p.with_extension(ext);
            if golden.exists() {
                runs.push((vec!["dump", flag, "--js"], p.clone(), golden, None));
            }
        }
    }
    dweller();
    let failures: Vec<String> = common::run_queue(&runs, |(args, page, golden, err)| {
        jsdom_case(args, page, golden, err.as_deref(), bless)
    })
    .into_iter()
    .flatten()
    .collect();
    assert!(
        failures.is_empty(),
        "{} of {} DOM fixture runs failed:\n\n{}",
        failures.len(),
        runs.len(),
        failures.join("\n\n")
    );
}

/// P4.48: a timer that toggles a class and reads `offsetWidth` 50 times,
/// on a page with a shadow host elsewhere (mistral's consent SDK), styles
/// again only the toggled element: not one cascade of the whole document
/// per read (51 before).
#[test]
fn jsdom_fixtures_restyle_beside_shadow() {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args([
            "dump",
            "--tree",
            "--js",
            "browser/tests/jsdom/shadow_class_toggle.html",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    let full = err.matches("DWELLER_TIMING\tstyle_full\t").count();
    assert!(full <= 5, "{full} full cascades:\n{err}");
}

/// Q4.16b: in a rAF with a running CSS transition,
/// `browser/tests/ui/transition_reads.html` writes a style and reads
/// getBoundingClientRect 100 times: the reads see the frame's transition
/// value (the value getComputedStyle gives), the transition walk
/// (`transition_walk`) runs at most once more than with 1 read (99 more
/// before), and a style change between two reads is seen by the second.
#[test]
fn geometry_reads_restyle_once() {
    let run = |reads: &str| {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_TIMING", "1")
            .args([
                "dump",
                "--text",
                "--js",
                &format!("browser/tests/ui/transition_reads.html?{reads}"),
            ])
            .output()
            .unwrap();
        let err = text(&out.stderr);
        assert!(out.status.success(), "{}\n{err}", out.status);
        let walks = err.matches("DWELLER_TIMING\ttransition_walk\t").count();
        (text(&out.stdout), walks)
    };
    let (many, many_walks) = run("100");
    let (one, one_walks) = run("1");
    assert!(many.contains("1 true true 50 70"), "{many}");
    assert_eq!(many, one);
    assert!(
        many_walks <= one_walks + 1,
        "{many_walks} transition walks with 100 reads, {one_walks} with 1"
    );
}

/// P4.75q: subtrees inserted on a page with `:has()` of any element and
/// of siblings are styled again in place when no element in them (or
/// beside them) has what the arguments look at: 3 of the 7 forced
/// layouts restyle (none before); the others match a full one
/// (`has_insert.tree`, `DWELLER_LAYOUT_CHECK` in `jsdom_fixtures`).
#[test]
fn jsdom_has_insert_restyles_partially() {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .env("DWELLER_LAYOUT_WHY", "1")
        .args([
            "dump",
            "--tree",
            "--js",
            "browser/tests/jsdom/has_insert.html",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    let full = err.matches("DWELLER_LAYOUT_WHY\tSTYLE_FULL\t").count();
    let restyles = err.matches("DWELLER_TIMING\trestyle\t").count();
    assert!(full <= 4 && restyles >= 3, "{full} full, {restyles} restyles:\n{err}");
}

/// Under `DWELLER_BENCHMARK=1`, `Date` is a fixed epoch plus the virtual
/// clock, which a task's steps move: a loop that waits for `Date.now()`
/// ends, after the same work on every run.
#[test]
fn benchmark_date_follows_virtual_clock() {
    let page = "browser/tests/benchmark/date_now.html";
    let dump = || {
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_BENCHMARK", "1")
            .args(["dump", "--text", "--js", page])
            .output()
            .unwrap();
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{}\n{}",
            out.status,
            text(&out.stderr)
        );
        text(&out.stdout)
    };
    let got = dump();
    assert_eq!(
        got,
        "date loop of 5 ms: n=2499999\nepoch: true\nnew Date: true\ntimer 100 ms later: true\n"
    );
    assert_eq!(dump(), got);
}

/// The DOM fixtures again with a collection at every allocation
/// (`DWELLER_JS_GC=stress`): the bindings keep what they hold reachable.
#[test]
fn jsdom_gc_stress() {
    let dir = root().join("browser").join("tests").join("jsdom");
    let mut pages: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "html"))
        .collect();
    pages.sort();
    // The slowest pages first, so that they do not all run at the end.
    const SLOW: [&str; 4] = [
        "virtual_clock.html",
        "wrapper_identity.html",
        "weak_nodes.html",
        "layout_many_changes.html",
    ];
    pages.sort_by_key(|p| !SLOW.iter().any(|s| p.ends_with(s)));
    // A page takes up to about 2 minutes here; one that hangs is killed.
    let limit = std::time::Duration::from_secs(600);
    dweller();
    let failures: Vec<String> = common::run_queue(&pages, |page| {
        let rel = page
            .strip_prefix(root())
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_JS_GC", "stress")
            .args(["dump", "--tree", "--js", &rel])
            .spawn_drained()
            .wait_with_output_within(limit)
            .unwrap();
        let want = std::fs::read_to_string(page.with_extension("tree")).unwrap_or_default();
        if !out.status.success() || rooted(&text(&out.stdout)) != want {
            Some(format!("{rel}: {}\n{}", out.status, text(&out.stderr)))
        } else {
            None
        }
    })
    .into_iter()
    .flatten()
    .collect();
    assert!(
        failures.is_empty(),
        "{} DOM fixtures failed under GC stress:\n\n{}",
        failures.len(),
        failures.join("\n\n")
    );
    // P9.8: code mode's realm calls into the page's realm and back.
    let run = root().join("browser/tests/agent/sessions/run.jsonl");
    // A run's 30 s default budget is wall time, which GC stress takes
    // most of on a slow runner: runs without their own `ms` get 300 s.
    let input: String = std::fs::read_to_string(&run)
        .unwrap()
        .lines()
        .map(|l| {
            if l.contains("\"cmd\":\"run\"") && !l.contains("\"ms\":") {
                format!("{{\"ms\":300000,{}\n", &l[1..])
            } else {
                format!("{l}\n")
            }
        })
        .collect();
    let mut child = dweller_command()
        .current_dir(root())
        .env("DWELLER_JS_GC", "stress")
        .arg("agent")
        .stdin(std::process::Stdio::piped())
        .spawn_drained();
    std::io::Write::write_all(&mut child.child.stdin.take().unwrap(), input.as_bytes()).unwrap();
    let out = child.wait_with_output_within(limit).unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "run.jsonl under GC stress: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert_eq!(
        std::fs::read_to_string(run.with_extension("out")).unwrap(),
        rooted(&text(&out.stdout)),
        "run.jsonl under GC stress"
    );
}
