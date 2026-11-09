use super::*;

/// `browser/tests/wasm/X.html` (P11.17, the WebAssembly JS API on
/// wasmtime): `dweller dump --text --js` must print `X.txt` with an empty
/// stderr. Without libwasmtime (browser/wasm/fetch_wasmtime.sh not run)
/// `WebAssembly` is undefined and each page prints "no WebAssembly"; CI
/// fetches it, so there it must be present.
#[test]
fn wasm_api() {
    let present = wasmtime_pkgconfig().is_some();
    assert!(
        present || std::env::var_os("CI").is_none(),
        "CI must run browser/wasm/fetch_wasmtime.sh"
    );
    let bless = std::env::var_os("DWELLER_BLESS").is_some();
    let mut failures = Vec::new();
    for name in ["api"] {
        let page = format!("browser/tests/wasm/{name}.html");
        let golden = root().join(format!("browser/tests/wasm/{name}.txt"));
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_ALLOW_FILE_ACCESS_FROM_FILES", "1")
            .args(["dump", "--text", "--js", &page])
            .output()
            .unwrap();
        let got = text(&out.stdout);
        if bless && present {
            std::fs::write(&golden, &got).unwrap();
            continue;
        }
        let want = if present {
            std::fs::read_to_string(&golden).unwrap()
        } else {
            "no WebAssembly\n".to_string()
        };
        if !out.status.success() || got != want || !out.stderr.is_empty() {
            failures.push(format!(
                "{page}: {}\n--- expected\n{want}--- got\n{got}--- stderr\n{}",
                out.status,
                text(&out.stderr)
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// loop.wasm (browser/tests/wasm/loop.rs, built from Rust) in Dweller (the
/// page fetches it from `file:`, so file access from files is on): the
/// checksum must be node's; the median time of 20M steps is printed (with
/// node's when `node` is installed) for comparison, not checked.
#[test]
fn wasm_loop_bench() {
    if wasmtime_pkgconfig().is_none() {
        println!("skipped: no libwasmtime (browser/wasm/fetch_wasmtime.sh)");
        return;
    }
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_ALLOW_FILE_ACCESS_FROM_FILES", "1")
        .args(["dump", "--text", "--js", "browser/tests/wasm/bench.html"])
        .output()
        .unwrap();
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{got}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert!(got.starts_with("checksum 486553404.1931757\n"), "{got}");
    println!("dweller (wasmtime): {}", got.lines().nth(1).unwrap_or(""));
    let script = r#"
        const b = require('fs').readFileSync(process.argv[1]);
        WebAssembly.instantiate(b).then(({ instance }) => {
          instance.exports.run(1000);
          const t = [];
          let r;
          for (let k = 0; k < 5; k++) { const t0 = Date.now(); r = instance.exports.run(20000000); t.push(Date.now() - t0); }
          t.sort((a, b) => a - b);
          console.log('checksum ' + r + ' median ms ' + t[2].toFixed(1));
        });
    "#;
    if let Ok(node) = Command::new("node")
        .current_dir(root())
        .args(["-e", script, "browser/tests/wasm/loop.wasm"])
        .output()
    {
        println!("node (V8): {}", text(&node.stdout).trim());
    }
}

/// One JS conformance script: None when it passes, else what went wrong.
fn js_case(js: &Path, bless: bool, env: &[(&str, &str)]) -> Option<String> {
    let rel = js
        .strip_prefix(root())
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let out = dweller_command()
        .current_dir(root())
        .args(["js", &rel])
        .envs(env.iter().copied())
        .output()
        .unwrap();
    let got = text(&out.stdout);
    let golden = js.with_extension("out");
    if bless {
        std::fs::write(&golden, &got).unwrap();
        return None;
    }
    let want = std::fs::read_to_string(&golden).unwrap_or_default();
    let mut problems = Vec::new();
    if got != want {
        problems.push(format!(
            "stdout differs from {}\n--- expected\n{want}--- got\n{got}",
            golden.display()
        ));
    }
    let stderr = text(&out.stderr);
    match std::fs::read_to_string(js.with_extension("err")) {
        Ok(want_err) => {
            if stderr != want_err || out.status.code() != Some(1) {
                problems.push(format!(
                    "expected status 1 and stderr {want_err:?}, got {} and {stderr:?}",
                    out.status
                ));
            }
        }
        Err(_) => {
            if !out.status.success() || !stderr.is_empty() {
                problems.push(format!(
                    "expected status 0 and no stderr, got {}:\n{stderr}",
                    out.status
                ));
            }
        }
    }
    if problems.is_empty() {
        None
    } else {
        Some(format!("{rel}: {}", problems.join("\n")))
    }
}

/// Every workload of `browser/tests/jsbench/` once at a small size
/// (`SMOKE=10`); its checksum must equal the golden `NAME.smoke.out`.
/// Timing is `browser/tools/jsbench.py` without `--smoke`.
#[test]
fn jsbench_smoke() {
    let out = match Command::new("python3")
        .current_dir(root())
        .args(["browser/tools/jsbench.py", "--smoke"])
        .env("BENCH_BIN", dweller())
        .env("BENCH_REF", "none")
        .output()
    {
        Ok(out) => out,
        Err(_) => {
            println!("skipped: cannot start python3");
            return;
        }
    };
    assert!(
        out.status.success(),
        "jsbench smoke failed:\n{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The scripts and goldens, then the same under `DWELLER_JS_GC=stress`
/// (the rooting rules of BYTECODE.md section 4). The stress pass leaves
/// out the allocation and recursion workloads, which collect millions of
/// times (minutes each).
#[test]
fn js_conformance() {
    js_conformance_with(&[], &[]);
    js_conformance_with(
        &[("DWELLER_JS_GC", "stress")],
        &[
            "_gc_",
            "_perf_",
            "recursion",
            "many_shapes",
            "map_set_hash",
            "coroutine_stacks",
        ],
    );
}

fn js_conformance_with(env: &[(&str, &str)], skip: &[&str]) {
    let dir = root().join("browser").join("tests").join("js");
    let mut scripts: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "js"))
        .filter(|p| {
            let name = p.file_name().unwrap().to_string_lossy();
            !skip.iter().any(|s| name.contains(s))
        })
        .collect();
    scripts.sort();
    assert!(
        scripts.len() >= 150,
        "expected at least 150 JS scripts, found {}",
        scripts.len()
    );
    dweller();
    let bless = env.is_empty() && std::env::var_os("DWELLER_BLESS").is_some();
    let failures: Vec<String> = common::run_queue(&scripts, |js| js_case(js, bless, env))
        .into_iter()
        .flatten()
        .collect();
    assert!(
        failures.is_empty(),
        "{} of {} JS scripts failed:\n\n{}",
        failures.len(),
        scripts.len(),
        failures.join("\n\n")
    );
}

/// `browser/tests/js/scopes/X.js` and `X.mjs`: `dweller js --print-scopes`
/// prints `X.scopes`, exit 0 and an empty stderr.
#[test]
fn js_print_scopes() {
    let dir = root().join("browser/tests/js/scopes");
    let mut scripts: Vec<PathBuf> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "js" || e == "mjs"))
        .collect();
    scripts.sort();
    assert!(scripts.len() >= 10, "expected at least 10 scope scripts");
    let mut failures = Vec::new();
    for js in &scripts {
        let rel = js.strip_prefix(root()).unwrap().display().to_string();
        let out = run(&["js", "--print-scopes", &rel]);
        if out.status.code() != Some(0) || !out.stderr.is_empty() {
            failures.push(format!(
                "{rel}: status {:?}, stderr:\n{}",
                out.status.code(),
                text(&out.stderr)
            ));
            continue;
        }
        check_golden(
            &js.with_extension("scopes"),
            &text(&out.stdout),
            &rel,
            &mut failures,
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n\n"));
}

#[test]
fn js_command_line() {
    expect(
        &["js", "browser/tests/js/missing.js"],
        1,
        "",
        "error: cannot read browser/tests/js/missing.js\n",
    );
    expect(&["js"], 2, "", "error: usage: dweller js FILE\n");
    expect(
        &["js", "browser/tests/js/modules/entry.mjs"],
        0,
        "entry 42 true\n",
        "",
    );
    expect(
        &["js", "browser/tests/js/modules/thrown.mjs"],
        1,
        "",
        "Uncaught 42\n",
    );
    let syntax = run(&["js", "browser/tests/js/modules/syntax.mjs"]);
    assert_eq!(syntax.status.code(), Some(1));
    assert!(
        text(&syntax.stderr).starts_with("Uncaught SyntaxError:"),
        "{}",
        text(&syntax.stderr)
    );
    // P4.61: no compiler yet, a hand-assembled function's disassembly
    let bytecode = run(&["js", "--print-bytecode", "x.js"]);
    assert_eq!(bytecode.status.code(), Some(0));
    assert_eq!(text(&bytecode.stderr), "");
    let out = text(&bytecode.stdout);
    assert!(
        out.contains("@   29 : 46 17          JumpLoop [23] (@ 6)\n")
            && out.contains("Constant pool (size = 1)\n"),
        "{out}"
    );
}

/// `browser/tests/workers/<name>.html` (P11.18, dedicated Workers) under
/// `dump --text --js` must print `<name>.txt` with an empty stderr. With
/// `sorted`, the lines are compared in sorted order: the replies of one
/// worker to messages that arrive together may come in any order.
fn check_worker_page(name: &str, sorted: bool) {
    let page = format!("browser/tests/workers/{name}.html");
    let golden = root().join(format!("browser/tests/workers/{name}.txt"));
    let out = run(&["dump", "--text", "--js", &page]);
    let got = text(&out.stdout);
    if std::env::var_os("DWELLER_BLESS").is_some() {
        std::fs::write(&golden, &got).unwrap();
    }
    let want = std::fs::read_to_string(&golden).unwrap();
    let order = |s: &str| {
        let mut l: Vec<String> = s.lines().map(str::to_string).collect();
        if sorted {
            l.sort();
        }
        l
    };
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{page}: {}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert_eq!(order(&got), order(&want), "{page} differs from {}:\n{got}", golden.display());
}

/// A classic worker: structured clone both ways, timers, importScripts,
/// fetch, performance, terminate, an `error` event, and a worker made
/// from a `blob:` URL of the blob registry (fetch, revokeObjectURL).
#[test]
fn worker_basic() {
    check_worker_page("basic", true);
}

/// A module worker imports `noise.js`, computes a 512x512 value noise
/// `Float32Array` and transfers it back; the page checks samples against
/// the same module.
#[test]
fn worker_module() {
    check_worker_page("module", false);
}

/// The worker pages over the test server (P11.14g): the worker script,
/// its `importScripts`, module imports and a `blob:` worker load through
/// the page's network loader.
#[test]
fn worker_http() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    for (name, sorted) in [("basic", true), ("module", false), ("transfer", false)] {
        let page = format!("{}/workers/{name}.html", srv.base);
        let want =
            std::fs::read_to_string(root().join(format!("browser/tests/workers/{name}.txt")))
                .unwrap();
        let out = run(&["dump", "--text", "--js", &page]);
        let order = |s: &str| {
            let mut l: Vec<String> = s.lines().map(str::to_string).collect();
            if sorted {
                l.sort();
            }
            l
        };
        assert!(out.stderr.is_empty(), "{page}: {}", text(&out.stderr));
        assert_eq!(order(&text(&out.stdout)), order(&want), "{page}");
    }
}

/// `transfer` detaches the sender's ArrayBuffer and views on it move
/// with it; buffers not in the list are copied.
#[test]
fn worker_transfer() {
    check_worker_page("transfer", false);
}

/// P11.14h: a canvas hands its control to a worker
/// (`transferControlToOffscreen`, transferred through `postMessage`); the
/// worker clears its WebGL context in requestAnimationFrame and the frame
/// shows in the placeholder canvas of `dweller screenshot` as `canvas.png`.
#[test]
fn worker_canvas() {
    let out_png = scratch().join("worker_canvas.png");
    let res = dweller_command()
        .current_dir(root())
        .args([
            "screenshot",
            "browser/tests/workers/canvas.html",
            "-o",
            out_png.to_str().unwrap(),
            "--font",
            "fixed",
            "--width",
            "120",
            "--height",
            "80",
        ])
        .env("DWELLER_GL", "swiftshader")
        .output()
        .unwrap();
    assert!(
        res.status.success() && res.stderr.is_empty(),
        "{}\n{}",
        res.status,
        text(&res.stderr)
    );
    if !angle_present() {
        println!("skipped: no ANGLE (browser/gpu/angle.lock has no hashes yet)");
        return;
    }
    let got = std::fs::read(&out_png).unwrap();
    let golden = root().join("browser/tests/workers/canvas.png");
    assert!(
        got == std::fs::read(&golden).unwrap(),
        "the image differs from {} (new one: {})",
        golden.display(),
        out_png.display()
    );
}
