use super::*;

// ---- P10.1: WPT testharness.js pages (`dweller wpt`, browser/tests/wpt)

/// `dweller wpt` on each fixture of browser/tests/wpt (file URLs: the
/// pages load browser/tools/wpt/testharnessreport.js by a relative path)
/// prints its one JSON line on stdout, with an empty stderr.
#[test]
fn wpt_runner() {
    let ok = |name: &str| format!(r#"{{"name":"{name}","status":"PASS","message":null}}"#);
    let line = |status: &str, message: &str, subtests: &[String]| {
        format!(
            r#"{{"status":"{status}","message":{message},"subtests":[{}]}}"#,
            subtests.join(",")
        ) + "\n"
    };
    let cases: &[(&[&str], String)] = &[
        (
            &["browser/tests/wpt/pass.html"],
            line("OK", "null", &[ok("addition"), ok("a timer"), ok("a promise")]),
        ),
        (
            &["browser/tests/wpt/fail.html"],
            line(
                "OK",
                "null",
                &[
                    ok("passes"),
                    r#"{"name":"fails","status":"FAIL","message":"assert_equals: one is two expected 2 but got 1"}"#.into(),
                ],
            ),
        ),
        (
            &["browser/tests/wpt/error.html"],
            line(
                "ERROR",
                r#""Uncaught Error: outside any test""#,
                &[ok("before the error")],
            ),
        ),
        (
            &["browser/tests/wpt/timeout.html"],
            line(
                "TIMEOUT",
                "null",
                &[
                    ok("finishes"),
                    r#"{"name":"never finishes","status":"TIMEOUT","message":"Test timed out"}"#.into(),
                ],
            ),
        ),
        // 15 s of the page's (virtual) time: past the default 10 s.
        (
            &["browser/tests/wpt/slow.html"],
            line(
                "TIMEOUT",
                "null",
                &[r#"{"name":"after 15 s","status":"TIMEOUT","message":"Test timed out"}"#.into()],
            ),
        ),
        (
            &["browser/tests/wpt/slow.html", "--timeout", "60"],
            line("OK", "null", &[ok("after 15 s")]),
        ),
        (
            &["browser/tests/wpt/sum.any.html"],
            line("OK", "null", &[ok("reduce sums"), ok("split")]),
        ),
    ];
    for (args, stdout) in cases {
        let mut all = vec!["wpt"];
        all.extend_from_slice(args);
        expect(&all, 0, stdout, "");
    }
    // A page that does not load: CRASH, exit 1.
    let missing = root().join("browser/tests/wpt/missing.html");
    expect(
        &["wpt", "browser/tests/wpt/missing.html"],
        1,
        &format!(
            "{{\"status\":\"CRASH\",\"message\":\"cannot fetch file://{}: not found\",\"subtests\":[]}}\n",
            missing.display()
        ),
        "",
    );
    // A page without testharness.js never reports: TIMEOUT, no subtests.
    expect(
        &["wpt", "browser/tests/basic.html", "--timeout", "1"],
        0,
        "{\"status\":\"TIMEOUT\",\"message\":\"no result from testharnessreport.js\",\"subtests\":[]}\n",
        "",
    );
    let usage = "error: usage: dweller wpt FILE|URL [--timeout S]\n";
    expect(&["wpt"], 2, "", usage);
    expect(
        &["wpt", "browser/tests/wpt/pass.html", "--timeout", "0"],
        2,
        "",
        usage,
    );
}

// P10.6: `test_driver.click`, `send_keys` and `action_sequence` become
// events that `dweller wpt` feeds to the page.
#[test]
fn wpt_testdriver() {
    let pass = |name: &str| format!(r#"{{"name":"{name}","status":"PASS","message":null}}"#);
    let subtests = [pass("click"), pass("send_keys"), pass("action_sequence")].join(",");
    expect(
        &["wpt", "browser/tests/wpt/testdriver.html"],
        0,
        &format!("{{\"status\":\"OK\",\"message\":null,\"subtests\":[{subtests}]}}\n"),
        "",
    );
}

/// `browser/tools/wpt.py run` over the fixture directory (its own static
/// server, the generated wrapper of `sum.any.js`): the results file has
/// the status and subtest counts of each fixture.
#[test]
fn wpt_py_serve() {
    let out = root().join("target/wpt-py-serve");
    let _ = std::fs::remove_dir_all(&out);
    let run = Command::new("python3")
        .current_dir(root())
        .args(["browser/tools/wpt.py", "run", ".", "-j", "4"])
        .args(["--wpt-root", "browser/tests/wpt", "--results-dir"])
        .arg(&out)
        .arg("--dweller")
        .arg(dweller())
        .output();
    let Ok(run) = run else {
        println!("skipped: cannot start python3");
        return;
    };
    let stdout = String::from_utf8_lossy(&run.stdout);
    assert!(
        run.status.success(),
        "wpt.py run failed: {stdout}{}",
        String::from_utf8_lossy(&run.stderr)
    );
    for want in [
        "error.html: 1/1 ERROR",
        "fail.html: 1/2 OK",
        "pass.html: 3/3 OK",
        "slow.html: 1/1 OK",
        "sum.any.js: 2/2 OK",
        "testdriver.html: 3/3 OK",
        "timeout.html: 1/2 TIMEOUT",
        ": 7 files (0 skipped), 12/14 subtests passed, score 85.7%",
    ] {
        assert!(stdout.contains(want), "missing {want:?} in:\n{stdout}");
    }
    let json = std::fs::read_to_string(out.join("root.json")).expect("results file");
    assert!(
        json.contains("\"passed\": 12") && json.contains("\"total\": 14"),
        "{json}"
    );
}

// P10.2: `wpt.py run` over the reftest and crashtest fixtures in
// browser/tests/wpt-reftest (the Ahem font through DWELLER_FONT_DIRS).
#[test]
fn wpt_reftest() {
    let out = root().join("target/wpt-reftest");
    let _ = std::fs::remove_dir_all(&out);
    let run = Command::new("python3")
        .current_dir(root())
        .args(["browser/tools/wpt.py", "run", ".", "-j", "4"])
        .args(["--wpt-root", "browser/tests/wpt-reftest", "--results-dir"])
        .arg(&out)
        .arg("--dweller")
        .arg(dweller())
        .output();
    let Ok(run) = run else {
        println!("skipped: cannot start python3");
        return;
    };
    let stdout = text(&run.stdout);
    assert!(run.status.success(), "wpt.py run failed: {stdout}{}", text(&run.stderr));
    for want in [
        "ahem.html: 1/1 OK",
        "crashtests/broken.html: 1/1 OK",
        "crashtests/ok.html: 1/1 OK",
        "fail.html: 0/1 OK",
        "fuzzy-fail.html: 0/1 OK",
        "fuzzy.html: 1/1 OK",
        "mismatch.html: 1/1 OK",
        "wait.html: 1/1 OK",
        ": 8 files (4 skipped), 6/8 subtests passed, score 75.0%",
    ] {
        assert!(stdout.contains(want), "missing {want:?} in:\n{stdout}");
    }
}

// P10.3: `dweller js --test262` and browser/tools/test262.py over the
// fixtures in browser/tests/test262.
#[test]
fn test262_runner() {
    let hooks = root().join("target").join("t262-hooks.js");
    std::fs::create_dir_all(hooks.parent().unwrap()).unwrap();
    std::fs::write(
        &hooks,
        "print('p'); print($262.global === globalThis); \
         print($262.evalScript('var q = 4; q + 1'));\n\
         var ab = new ArrayBuffer(8); $262.detachArrayBuffer(ab); $262.gc(); print(ab.byteLength);\n",
    )
    .unwrap();
    let hooks = hooks.to_str().unwrap().to_string();
    expect(&["js", "--test262", &hooks], 0, "p\ntrue\n5\n0\n", "");
    // without the flag there is no print or $262
    let plain = run(&["js", &hooks]);
    assert_eq!(plain.status.code(), Some(1));
    assert!(
        text(&plain.stderr).starts_with("Uncaught ReferenceError"),
        "{}",
        text(&plain.stderr)
    );

    let tool = root().join("browser").join("tools").join("test262.py");
    let fixtures = root().join("browser").join("tests").join("test262");
    let out = root().join("target").join("t262-fixtures");
    let res = Command::new("python3")
        .arg(&tool)
        .args(["run", "test", "-j", "4", "--root"])
        .arg(&fixtures)
        .arg("--dweller")
        .arg(dweller())
        .arg("--out")
        .arg(&out)
        .output();
    let Ok(res) = res else {
        println!("skipped: cannot start python3");
        return;
    };
    assert!(res.status.success(), "{}", text(&res.stderr));
    let stdout = text(&res.stdout);
    assert!(
        stdout.contains("test262 test: 8/11 passed (72.7%), 1 skipped"),
        "{stdout}"
    );
    let json = std::fs::read_to_string(out.join("test.json")).unwrap();
    for (name, status) in [
        ("test/pass/basic.js", "pass"),
        ("test/pass/skipped.js", "skip"),
        ("test/fail/wrong.js", "fail"),
        ("test/negative/parse.js", "pass"),
        ("test/negative/runtime.js", "pass"),
        ("test/negative/missing.js", "fail"),
        ("test/async/promise.js", "pass"),
        ("test/async/never.js", "fail"),
        ("test/strict/only.js", "pass"),
        ("test/strict/both.js", "pass"),
        ("test/strict/nostrict.js", "pass"),
        ("test/module/basic.js", "pass"),
    ] {
        let at = json
            .find(&format!("\"{name}\""))
            .unwrap_or_else(|| panic!("{name}: {json}"));
        let rest = &json[at..];
        let line = &rest[..rest.find(']').unwrap()];
        assert!(line.contains(&format!("\"{status}\"")), "{name}: {line}");
    }
}
