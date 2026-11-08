use super::*;

// ---- page scripts and the DOM bindings (browser/engine/script.aye)

/// The window, headless: a click and a key reach the page's listeners,
/// which change the text (`--text` prints the document's text at the
/// end); with `--no-js` nothing changes.
#[test]
fn window_scripts() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let cases: &[(&[&str], &str)] = &[
        (&["--events", "click:50,65"], "\nclicked\n"),
        (&["--events", "key:x"], "\nkey x\n"),
        (&["--events", "click:50,200"], "\nbefore\n"),
        (&["--events", "click:50,65", "--no-js"], "\nbefore\n"),
    ];
    for (extra, want) in cases {
        let mut args = vec![
            "browser/tests/ui/script.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "1",
            "--text",
        ];
        args.extend_from_slice(extra);
        let out = run_window(&args);
        let got = text(&out.stdout);
        assert!(
            out.status.success() && out.stderr.is_empty(),
            "{args:?}: {}\n{}",
            out.status,
            text(&out.stderr)
        );
        assert!(
            got.contains(want),
            "{args:?}: expected {want:?} in the text:\n{got}"
        );
    }
}

/// Timers that never let the queue empty (animation callbacks that take
/// longer than a frame, a task that queues itself at 0 ms) must not
/// starve the window: each event-loop turn runs only the timers queued
/// before it, then the window draws, so `--frames N` ends.
#[test]
fn window_timers_yield_to_frames() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut window = dweller_command()
        .current_dir(root())
        .args([
            "browser/tests/ui/raf_overload.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "10",
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not draw 10 frames in 60 s: timers starve it");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert!(
        text(&out.stdout).contains("animating"),
        "{}",
        text(&out.stdout)
    );
}

/// The index of the first iteration in which a phase `name` whose label
/// contains `label` ends.
fn iteration_of(loops: &[Iteration], name: &str, label: &str) -> usize {
    loops
        .iter()
        .position(|it| it.has(name, label))
        .unwrap_or_else(|| panic!("no {name} record of {label:?}: {loops:#?}"))
}

/// Page fetch() and XMLHttpRequest run in the background of the window's
/// event loop (`browser/tests/net/window_fetch.html`): while the server
/// holds a fetch for 1.5 s, scripted input types into a field and
/// scrolls, the XHR completes, and the fetch resolves afterwards with
/// its body. In the DWELLER_TIMING records, their responses end
/// iterations after the one whose task sent them, and no iteration waits
/// for the network (`assert_no_net_wait`).
#[test]
fn window_fetch_in_background() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/window_fetch.html", srv.base);
    let mut window = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "300",
            "--events",
            "frame frame frame frame frame text:abc frame wheel:-3 frame wheel:-3 frame",
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not draw 300 frames in 60 s");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    let err = text(&out.stderr);
    let loops = iterations(&err);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(loops.len() > 100, "{} loop iterations", loops.len());
    assert_no_net_wait(&loops, &page);
    // The page's first task sends the requests.
    let sent = iteration_of(&loops, "timers", &page);
    let xhr = iteration_of(&loops, "background_requests", "/slow/window.txt");
    let fetched = iteration_of(&loops, "background_requests", "/sleep?ms=1500");
    assert!(
        sent + 3 <= xhr && xhr + 3 <= fetched,
        "sent in iteration {sent}, the XHR's response in {xhr}, the fetch's in {fetched}"
    );
    let got = text(&out.stdout);
    let want = [
        "requests sent",
        "input while the fetch waits: true",
        "scroll while the fetch waits: true",
        "xhr: 200 4 bytes while the fetch waits: true",
        "fetched: true, field abc, scrolled true",
    ];
    let at: Vec<Option<usize>> = want.iter().map(|w| got.find(w)).collect();
    assert!(
        at.iter().all(|a| a.is_some()) && at.windows(2).all(|w| w[0] < w[1]),
        "{got}"
    );
}

#[test]
fn window_raf_vsync() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let mut window = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args([
            "browser/tests/ui/raf_vsync.html",
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "1200",
            "--until-idle",
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not end in 60 s");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    let period_ms = err
        .lines()
        .find_map(|l| l.split("DWELLER_TIMING\tui_display_period\t").nth(1))
        .map(|r| r.split('\t').next().unwrap().parse::<f64>().unwrap() / 1e6)
        .expect("no ui_display_period record");
    let got = text(&out.stdout);
    let line = got
        .lines()
        .find(|l| l.starts_with("frames "))
        .unwrap_or_else(|| panic!("no result:\n{got}"));
    assert!(
        line.starts_with("frames 120 same true median ") && line.ends_with(" fractional true coarse true"),
        "{line}"
    );
    let median: f64 = line.split(' ').nth(5).unwrap().parse().unwrap();
    assert!(
        (median - period_ms).abs() <= period_ms * 0.1,
        "median rAF interval {median} ms, display period {period_ms} ms"
    );
    println!("rAF median interval {median:.2} ms, display period {period_ms:.2} ms");
}

/// P11.24b: the window does not end a task of the page. A script that
/// runs 4 s of CPU (`browser/tests/ui/long_task.html`) finishes and sets
/// its marker, and the window then handles the typed input. `dump` keeps
/// the step budget: a script without end is terminated and the page goes
/// on.
#[test]
fn window_long_task() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = run_window(&[
        "browser/tests/ui/long_task.html",
        "--font",
        "fixed",
        "--width",
        "400",
        "--height",
        "300",
        "--frames",
        "4",
        "--events",
        "frame text:abc frame frame",
        "--text",
    ]);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert!(got.contains("ran 4 s, typed abc"), "{got}");
    let out = run(&["dump", "--text", "--js", "browser/tests/ui/long_task_forever.html"]);
    let got = text(&out.stdout);
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    assert!(got.contains("before") && !got.contains("ran\n"), "{got}");
    assert!(
        got.contains("Script terminated: it ran too long")
            || text(&out.stderr).contains("Script terminated: it ran too long"),
        "{got}"
    );
}

/// Q1.7: a click starts `try { while (1) {} } finally { ... }`
/// (`browser/tests/ui/stuck.html`). After 5 s the window shows "This page
/// is not responding"; Wait gives the script 5 s more, and the bar comes
/// again; Stop ends it as V8's TerminateExecution does (no `catch` and no
/// `finally` runs). A later click on the other button runs its handler.
#[test]
fn window_stuck_script() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let out = run_window(&[
        "browser/tests/ui/stuck.html",
        "--font",
        "fixed",
        "--width",
        "400",
        "--height",
        "300",
        "--frames",
        "2",
        "--state",
        "--text",
        "--events",
        "click:50,65 click:288,56 frame click:360,56 frame click:50,115",
    ]);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert!(got.contains("\nunresponsive 2\n"), "{got}");
    assert!(got.contains("\nstart hang after\n"), "{got}");
}

/// Q1.8: a page that allocates without end (`out-of-memory.html`) passes
/// `DWELLER_JS_HEAP_LIMIT` after a full collection: its script ends and
/// the tab shows "This page ran out of memory". The window lives, and a
/// second tab runs a page that allocates 40 MB of strings, drops them and
/// allocates them again (`allocate-and-free.html`) without hitting the limit.
#[test]
fn window_out_of_memory() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let run = |last: &str| -> (String, String) {
        let events = format!(
            "painted frame key:cmd+t text:browser/tests/ui/ text:allocate-and-free.html key:return loaded frame {last}"
        );
        let out = dweller_command()
            .current_dir(root())
            .env("DWELLER_CONSOLE", "1")
            .env("DWELLER_JS_HEAP_LIMIT", "64000000")
            .args([
                "browser/tests/ui/out-of-memory.html",
                "--font", "fixed", "--width", "400", "--height", "300", "--frames", "300",
                "--state", "--text", "--events", &events,
            ])
            .output()
            .unwrap();
        assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
        (text(&out.stdout), text(&out.stderr))
    };
    let (got, err) = run("");
    assert!(got.contains("tabs 2, showing 2\n") && got.contains("\nfreed 40000\n"), "{got}");
    assert!(err.contains("[error] Script terminated: out of memory"), "{err}");
    let (got, _) = run("key:cmd+1 frame");
    assert!(got.contains("tabs 2, showing 1\n"), "{got}");
    assert!(got.contains("This page ran out of memory") && got.contains("Reload"), "{got}");
}

/// Q1.7b: a headless window never waits for a click on the stuck bar. A
/// page whose first task runs 6 s ends with `--until-idle`: the bar
/// chooses Wait and logs one line. The `stuck:stop` word chooses Stop.
#[test]
fn window_stuck_headless() {
    use std::time::Duration;
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let window = |args: &[&str]| {
        dweller_command()
            .current_dir(root())
            .args(args)
            .spawn_drained()
            .wait_with_output_within(Duration::from_secs(60))
            .unwrap()
    };
    let out = window(&[
        "browser/tests/ui/stuck_load.html",
        "--font",
        "fixed",
        "--frames",
        "200",
        "--until-idle",
        "--state",
        "--text",
    ]);
    let got = text(&out.stdout);
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(
        err.starts_with("stuck: page busy ")
            && err.ends_with(" ms, waiting\n")
            && err.lines().count() == 1,
        "{err}"
    );
    assert!(
        got.contains("\nunresponsive 1\n") && got.contains("start done"),
        "{got}"
    );
    let out = window(&[
        "browser/tests/ui/stuck.html",
        "--font",
        "fixed",
        "--width",
        "400",
        "--height",
        "300",
        "--frames",
        "2",
        "--state",
        "--text",
        "--events",
        "click:50,65 stuck:stop frame click:50,115",
    ]);
    let got = text(&out.stdout);
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    assert!(
        got.contains("\nunresponsive 1\n") && got.contains("\nstart hang after\n"),
        "{got}"
    );
}

/// P4.33: the window paints a page once its document and its sheet are
/// in (0.3 s), before its script and its image arrive (1.5 s), and the
/// click and the wheel during the load are handled before the script
/// runs. In the DWELLER_TIMING records, the first paint comes iterations
/// before the script runs and the image is read, and no iteration waits
/// for the network (`assert_no_net_wait`).
#[test]
fn window_paints_early() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/early_paint.html", srv.base);
    // The click waits for the first paint (`painted`), which comes
    // 0.3 s into the 1.5 s the script is delayed, not for a number of
    // frames (40 take longer than 1.5 s on a loaded runner).
    let events = "painted click:14,54 frame wheel:-3 frame";
    let mut window = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "300",
            "--events",
            events,
            "--text",
        ])
        .spawn_drained();
    let start = Instant::now();
    while window.try_wait().unwrap().is_none() {
        if start.elapsed() > Duration::from_secs(60) {
            let _ = window.kill();
            panic!("the window did not draw 300 frames in 60 s");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let out = window.wait_with_output().unwrap();
    let err = text(&out.stderr);
    let loops = iterations(&err);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(loops.len() > 100, "{} loop iterations", loops.len());
    assert_no_net_wait(&loops, &page);
    let painted = iteration_of(&loops, "ui_first_paint", &page);
    let script = iteration_of(&loops, "script", "/delay/1500/net/early_paint.js");
    let image = iteration_of(&loops, "requests", "/delay/1500/paint/img/badge.png");
    assert!(
        painted + 3 <= script && painted + 3 <= image,
        "first paint in iteration {painted}, the script runs in {script}, the image is read in {image}"
    );
    let got = text(&out.stdout);
    assert!(
        got.contains("script ran; checked before it: true, scrolled before it: true, color: rgb(0, 128, 0)"),
        "{got}"
    );
}

/// The console lines (`DWELLER_CONSOLE`) a window printed for a page in
/// `browser/tests/ui/` driven by the script `events`.
fn console_of(page: &str, events: &str, frames: &str) -> Vec<String> {
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_CONSOLE", "1")
        .args([
            page, "--font", "fixed", "--width", "400", "--height", "300", "--frames", frames,
            "--events", events, "--text",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    text(&out.stderr)
        .lines()
        .filter_map(|l| l.strip_prefix("[log] "))
        .map(String::from)
        .collect()
}

fn position(lines: &[String], line: &str) -> usize {
    lines
        .iter()
        .position(|l| l == line)
        .unwrap_or_else(|| panic!("no {line:?} in {lines:?}"))
}

/// P4.55: a hidden window runs no animation frames (`window:hidden` is
/// SDL's minimized event): none between the two `visibilitychange`
/// events, and they run again once the window is shown.
#[test]
fn window_hidden_raf() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let frames = "frame ".repeat(10);
    let events = format!("painted {frames}window:hidden {frames}window:shown {frames}");
    let lines = console_of("browser/tests/ui/hidden-raf.html", &events, "400");
    let hidden = position(&lines, "vis hidden true");
    let shown = position(&lines, "vis visible false");
    assert!(hidden < shown, "{lines:?}");
    assert!(lines[..hidden].iter().any(|l| l == "raf"), "{lines:?}");
    assert!(!lines[hidden..shown].iter().any(|l| l == "raf"), "{lines:?}");
    assert!(lines[shown..].iter().any(|l| l == "raf"), "{lines:?}");
}

/// P4.75o: a cross-origin frame below the viewport runs no animation
/// frames (Blink throttles its rendering); they run once it is scrolled
/// into view.
#[test]
fn window_offscreen_frame_raf() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let frames = "frame ".repeat(20);
    let events = format!("painted {frames}key:end {frames}");
    let lines = console_of("browser/tests/ui/offscreen-frame-raf.html", &events, "400");
    let scrolled = position(&lines, "scrolled");
    let before = lines[..scrolled].iter().filter(|l| *l == "frame raf").count();
    let after = lines[scrolled..].iter().filter(|l| *l == "frame raf").count();
    assert!(before <= 2, "{lines:?}");
    assert!(after >= 5, "{lines:?}");
}

/// P4.55: the tab in the background runs its 100 ms interval about once
/// a second and no animation frame; both run as before when it is shown.
#[test]
fn window_background_tab_throttled() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let frames = "frame ".repeat(300);
    let events = format!(
        "painted key:cmd+t text:browser/tests/ui/ text:hidden-tab-b.html key:return loaded {frames}key:cmd+1 {frames}"
    );
    let lines = console_of("browser/tests/ui/hidden-tab-a.html", &events, "1000");
    let hidden = position(&lines, "a vis hidden");
    let shown = position(&lines, "a vis visible");
    let between = &lines[hidden..shown];
    assert!(!between.iter().any(|l| l == "a raf"), "{lines:?}");
    let ticks: Vec<u64> = between
        .iter()
        .filter_map(|l| l.strip_prefix("a tick ")?.parse().ok())
        .collect();
    assert!(ticks.len() >= 2, "the background tab's interval did not run: {lines:?}");
    assert!(
        ticks.windows(2).all(|w| w[1] - w[0] >= 900),
        "ticks closer than a second: {ticks:?}"
    );
    let after = &lines[shown..];
    assert!(after.iter().any(|l| l == "a raf"), "{lines:?}");
    assert!(after.iter().filter(|l| l.starts_with("a tick ")).count() >= 5, "{lines:?}");
}

/// P4.43: twenty import()s whose modules take 200-390 ms, a module whose
/// twenty imports do the same, and a frame whose document takes 1 s load
/// in the background: each import() settles with its module, the imports
/// evaluate in their order, and the load event waits for the frame. In
/// the DWELLER_TIMING records, the modules are read in several
/// iterations, the frame's document ends iterations after the page's
/// script ran, and no iteration waits for the network
/// (`assert_no_net_wait`).
#[test]
fn window_dynamic_scripts() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/dynamic_scripts.html", srv.base);
    let events = format!("{}wheel:-3 frame", "frame ".repeat(40));
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "400",
            "--events",
            events.as_str(),
            "--text",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    let loops = iterations(&err);
    assert!(out.status.success(), "{}\n{err}", out.status);
    assert!(loops.len() > 300, "{} loop iterations", loops.len());
    assert_no_net_wait(&loops, &page);
    // The iterations in which a module is read (its task runs).
    let modules: Vec<usize> = (0..loops.len())
        .filter(|&i| {
            loops[i].has("requests", "/net/dynamic_module.js")
                || loops[i].has("requests", "/net/dynamic_root.js")
        })
        .collect();
    let ran = iteration_of(&loops, "script", "inline");
    let frame = iteration_of(
        &loops,
        "background_requests",
        "/delay/1000/net/dynamic_frame.html",
    );
    assert!(
        modules.len() >= 3 && ran + 3 <= frame,
        "modules read in iterations {modules:?}, the page's script ran in {ran}, the frame's document ended in {frame}"
    );
    let got = text(&out.stdout);
    let graph: Vec<String> = (0..20).map(|i| format!("g={i}")).collect();
    let want = format!(
        "imports settled: 20, each with its module: true; graph: root {}; frame loaded: true",
        graph.join(",")
    );
    assert!(got.contains(&want), "{got}");
}

/// P4.45: scripts that a script inserts (`net/inserted_scripts.html`):
/// an async one, twenty `async = false` ones whose loads end last first,
/// a webpack-style JSONP chunk, a missing script and one of another
/// origin served as HTML, in `dump` (twice:
/// the same output) and in the window, where no loop iteration waits
/// for their loads.
#[test]
fn inserted_scripts() {
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let page = format!("{}/net/inserted_scripts.html", srv.base);
    let ordered: Vec<String> = (0..20).map(|i| format!("o{i}")).collect();
    let want = format!(
        "async: true; ordered: {}; chunk: lazy module; missing: true; html of another origin: blocked; loads: 21",
        ordered.join(",")
    );
    let port = srv.base.rsplit(':').next().unwrap();
    let missing = format!(
        "[error] Failed to load script: {}/net/inserted_missing.js\n[error] Failed to load script: http://localhost:{port}/log/inserted.html\n",
        srv.base
    );
    let mut dumps = Vec::new();
    for _ in 0..2 {
        let out = run(&["dump", "--text", "--js", &page]);
        assert!(out.status.success(), "{}", text(&out.stderr));
        assert_eq!(text(&out.stderr), missing);
        let got = text(&out.stdout);
        assert!(got.contains(&want), "{got}");
        dumps.push(got);
    }
    assert_eq!(dumps[0], dumps[1]);
    if !has_sdl() {
        println!("window part skipped: no SDL2");
        return;
    }
    let events = format!("{}wheel:-3 frame", "frame ".repeat(40));
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_TIMING", "1")
        .args([
            page.as_str(),
            "--font",
            "fixed",
            "--width",
            "400",
            "--height",
            "300",
            "--frames",
            "400",
            "--events",
            events.as_str(),
            "--text",
        ])
        .output()
        .unwrap();
    let err = text(&out.stderr);
    assert!(out.status.success(), "{}\n{err}", out.status);
    // The iterations' times, apart from those that run the page's script.
    let mut loops = Vec::new();
    let mut ran_script = false;
    for line in err.lines() {
        let fields: Vec<&str> = line.trim_start_matches("info: ").split('\t').collect();
        if fields.len() < 3 || fields[0] != "DWELLER_TIMING" {
            continue;
        }
        let ms = fields[2].parse::<u64>().unwrap() / 1_000_000;
        match fields[1] {
            "ui_loop" if ran_script => ran_script = false,
            "ui_loop" => loops.push(ms),
            "scripts" => ran_script = true,
            _ => {}
        }
    }
    let longest = loops.iter().copied().max().unwrap_or(0);
    assert!(
        loops.len() > 300 && longest < 100,
        "longest loop {longest} ms of {}",
        loops.len()
    );
    let got = text(&out.stdout);
    assert!(got.contains(&want), "{got}");
}

/// Local telemetry (`browser/perf/telemetry.aye`): a window on a temporary
/// `DWELLER_PROFILE` showing `browser/tests/telemetry/page.html` (a JS
/// throw, an unknown CSS property and value, a missing API, a slow task,
/// a 404 image and a corrupt image) writes one record of each to the day
/// file, with origins only: no path, query, page text or form value.
/// `telemetry.py summary` lists them; `DWELLER_TELEMETRY=0` and a test
/// window without a profile write nothing; a crash (SIGSEGV) is recorded
/// with the window's last actions.
#[test]
fn window_telemetry() {
    use std::time::{Duration, Instant};
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let base = std::env::temp_dir().join(format!("dweller-telemetry-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let profile = base.join("profile");
    let off = base.join("off");
    let home = base.join("home");
    for d in [&profile, &off, &home] {
        std::fs::create_dir_all(d).unwrap();
    }
    let page = format!("{}/telemetry/page.html?user=QUERYSECRET-2", srv.base);
    let window = |profile: Option<&Path>, telemetry: Option<&str>, frames: &str| {
        let mut c = dweller_command();
        c.current_dir(root())
            .env("HOME", &home)
            .env_remove("DWELLER_PROFILE")
            .env_remove("DWELLER_TELEMETRY")
            .env_remove("DWELLER_TELEMETRY_DIR")
            .args([
                page.as_str(),
                "--font",
                "fixed",
                "--width",
                "400",
                "--height",
                "300",
                "--frames",
                frames,
                "--events",
                "frame wheel:-3 frame wheel:-3 frame wheel:-3 frame",
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        if let Some(p) = profile {
            c.env("DWELLER_PROFILE", p);
        }
        if let Some(t) = telemetry {
            c.env("DWELLER_TELEMETRY", t);
        }
        c
    };
    let records = |profile: &Path| -> String {
        let dir = profile.join("telemetry");
        let mut all = String::new();
        if let Ok(entries) = std::fs::read_dir(&dir) {
            let mut names: Vec<PathBuf> = entries.map(|e| e.unwrap().path()).collect();
            names.sort();
            for n in names {
                all.push_str(&std::fs::read_to_string(n).unwrap());
            }
        }
        all
    };

    let out = window(Some(&profile), None, "200").output().unwrap();
    assert!(
        out.status.success() && out.stderr.is_empty(),
        "{}\n{}",
        out.status,
        text(&out.stderr)
    );
    let day = chrono_free_today();
    let file = profile.join("telemetry").join(format!("{day}.jsonl"));
    let got = std::fs::read_to_string(&file).unwrap_or_else(|e| panic!("{}: {e}", file.display()));
    let origin = srv.base.clone();
    for want in [
        "\"ev\":\"js_error\",".to_string(),
        "\"type\":\"Error\",\"message\":\"failed at ".to_string() + &origin + "\"",
        "\"feature\":\"css_property\",\"name\":\"zoomy-frobnicate\"".to_string(),
        "\"feature\":\"css_value\",\"name\":\"display: grid-lanes\"".to_string(),
        "\"feature\":\"js_global\",\"name\":\"XRSession\"".to_string(),
        "\"kind\":\"image\",\"status\":404,\"origin\":\"".to_string() + &origin + "\"",
        "\"ev\":\"image_decode\",".to_string(),
        "\"ev\":\"page\",".to_string(),
        "\"ev\":\"scroll\",".to_string(),
    ] {
        assert_eq!(got.matches(&want).count(), 1, "one {want} in:\n{got}");
    }
    // The slow task's loop iteration, by its script phase (a debug build
    // may have other slow ones).
    let slow_script = got
        .lines()
        .filter(|l| l.contains("\"ev\":\"slow_loop\""))
        .any(|l| {
            l.split("\"script\":")
                .nth(1)
                .and_then(|v| v.split(|c: char| !c.is_ascii_digit()).next())
                .and_then(|v| v.parse::<u64>().ok())
                .is_some_and(|ms| ms >= 200)
        });
    assert!(slow_script, "no slow_loop of the slow script in:\n{got}");
    for line in got.lines() {
        assert!(
            line.starts_with("{\"t\":\"") && line.ends_with('}'),
            "not a record: {line}"
        );
        assert!(
            line.contains(&format!("\"site\":\"{origin}\"")),
            "not of the page's site: {line}"
        );
    }
    // Origins only: no path, query, page text or form value.
    for secret in [
        "?",
        "/telemetry",
        "page.html",
        "missing.png",
        "broken.png",
        "QUERYSECRET",
        "PATHSECRET",
        "PAGETEXT",
        "FORMVALUE",
        "only the page has this text",
    ] {
        assert!(!got.contains(secret), "{secret} in:\n{got}");
    }

    let summary = Command::new("python3")
        .current_dir(root())
        .args([
            "browser/tools/telemetry.py",
            "summary",
            "--days",
            "1",
            "--profile",
        ])
        .arg(&profile)
        .output()
        .unwrap();
    let listed = text(&summary.stdout);
    assert!(summary.status.success(), "{}", text(&summary.stderr));
    for want in [
        "js_error      Error: failed at <origin>",
        "unsupported   css_property zoomy-frobnicate",
        "unsupported   css_value display: grid-lanes",
        "unsupported   js_global XRSession",
        "slow_loop     loop > 100 ms, mostly script",
        "load_failed   image 404",
        "image_decode  png ",
    ] {
        assert!(listed.contains(want), "{want} not in:\n{listed}");
    }

    // Off: DWELLER_TELEMETRY=0, and a test window without a profile.
    let out = window(Some(&off), Some("0"), "60").output().unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!off.join("telemetry").exists(), "{}", records(&off));
    let out = window(None, None, "60").output().unwrap();
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(!home.join(".dweller").exists());

    // A crash: the records so far and the crash, with the last actions.
    {
        let crashed = base.join("crashed");
        std::fs::create_dir_all(&crashed).unwrap();
        let mut child = window(Some(&crashed), None, "3000").spawn().unwrap();
        std::thread::sleep(Duration::from_millis(1500));
        let killed = Command::new("kill")
            .args(["-SEGV", &child.id().to_string()])
            .status()
            .unwrap();
        assert!(killed.success());
        let start = Instant::now();
        let status = loop {
            if let Some(status) = child.try_wait().unwrap() {
                break status;
            }
            if start.elapsed() > Duration::from_secs(30) {
                let _ = child.kill();
                panic!("the window did not end on SIGSEGV");
            }
            std::thread::sleep(Duration::from_millis(20));
        };
        use std::os::unix::process::ExitStatusExt;
        assert_eq!(status.signal(), Some(11), "{status:?}");
        let got = records(&crashed);
        let crash = got
            .lines()
            .find(|l| l.contains("\"ev\":\"crash\""))
            .unwrap_or_else(|| panic!("no crash record in:\n{got}"));
        assert!(
            crash.contains("\"signal\":\"SIGSEGV\"")
                && crash.contains("\"backtrace\":[\"0x")
                && crash.contains("\"actions\":[\"wheel\",\"wheel\",\"wheel\"]"),
            "{crash}"
        );
        assert!(got.contains("\"ev\":\"js_error\""), "{got}");
    }

    // A stack overflow on macOS: Aye's panic line, then the crash record,
    // and the window ends on SIGSEGV.
    if cfg!(target_os = "macos") {
        use std::os::unix::process::ExitStatusExt;
        let overflowed = base.join("overflowed");
        std::fs::create_dir_all(&overflowed).unwrap();
        let out = dweller_command()
            .current_dir(root())
            .env("HOME", &home)
            .env("DWELLER_PROFILE", &overflowed)
            .env_remove("DWELLER_TELEMETRY")
            .env_remove("DWELLER_TELEMETRY_DIR")
            .args([page.as_str(), "--font", "fixed", "--frames", "300", "--events", "frame overflow"])
            .output()
            .unwrap();
        let err = text(&out.stderr);
        assert_eq!(out.status.signal(), Some(11), "{err}");
        assert!(err.contains("panic: stack overflow (deep recursion"), "{err}");
        let got = records(&overflowed);
        assert!(
            got.lines()
                .any(|l| l.contains("\"ev\":\"crash\"") && l.contains("\"signal\":\"SIGSEGV\"")),
            "{got}"
        );
    }
    let _ = std::fs::remove_dir_all(&base);
}

/// Today's date in UTC as YYYY-MM-DD (the name of telemetry's day file).
fn chrono_free_today() -> String {
    let days = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        / 86400;
    // civil-from-days (Howard Hinnant), as std.time does.
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + if month <= 2 { 1 } else { 0 };
    format!("{year:04}-{month:02}-{day:02}")
}
