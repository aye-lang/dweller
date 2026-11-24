use super::*;

/// Q4.25c: with `DWELLER_PROCESS=tab` and `DWELLER_TIMING=1` the window
/// writes an `input_to_frame` record for each input the renderer acked
/// (labelled with its kind) and the renderer an `r_task` record for each
/// turn of its loop, labelled with its pid.
#[test]
fn window_renderer_perf_records() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "tab")
        .env("DWELLER_TIMING", "1")
        .arg(format!("{}/ui/kept_list.html", srv.base))
        .args(["--font", "fixed", "--width", "400", "--height", "300", "--frames", "2000", "--until-idle", "--state", "--processes"])
        .args(["--events", "renderer:loaded move:200,150 wheel:-3 renderer:input wheel:-3 renderer:input click:200,150 release:200,150 renderer:input"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let err = text(&out.stderr);
    let state = text(&out.stdout);
    let pid = state.lines().find_map(|l| l.strip_prefix("renderer-pids ")).unwrap_or("").trim().to_string();
    let labels = |phase: &str| -> Vec<String> {
        let head = format!("info: DWELLER_TIMING\t{phase}\t");
        err.lines().filter_map(|l| l.strip_prefix(head.as_str())).filter_map(|l| l.split('\t').nth(1)).map(str::to_string).collect()
    };
    let held = labels("input_to_frame");
    assert!(held.iter().filter(|l| *l == "wheel").count() >= 1 && held.iter().any(|l| l == "click"), "{held:?}");
    let tasks = labels("r_task");
    assert!(!pid.is_empty() && tasks.len() >= 3 && tasks.iter().all(|l| *l == pid), "pid {pid:?}, {} tasks", tasks.len());
}

/// Q4.25d: with `DWELLER_PROCESS=tab` the scripts of a load are tasks of
/// the renderer's loop and an input is served between them: a wheel sent
/// once the page's first frame is shown (`browser/tests/perf/staged_scripts.html`,
/// eight inline scripts of 100 ms) is acked while scripts are still to
/// run (at least 3 `scripts` records follow its `input_to_frame`), and no
/// `scripts` record is as long as the eight together.
#[test]
fn window_renderer_perf_input_between_scripts() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = dweller_command()
        .current_dir(root())
        .env("DWELLER_PROCESS", "tab")
        .env("DWELLER_TIMING", "1")
        .arg(format!("{}/perf/staged_scripts.html", srv.base))
        .args(["--font", "fixed", "--width", "400", "--height", "300", "--frames", "4000", "--until-idle", "--text"])
        .args(["--events", "renderer:frame move:200,150 wheel:-1 renderer:input renderer:loaded frame"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    assert!(text(&out.stdout).contains("ran: external 0 1 2 3 4 5 6 7"), "{}", text(&out.stdout));
    let err = text(&out.stderr);
    let lines: Vec<&str> = err.lines().filter(|l| l.starts_with("info: DWELLER_TIMING\tscripts\t") || l.starts_with("info: DWELLER_TIMING\tinput_to_frame\t")).collect();
    let acked = lines.iter().position(|l| l.contains("\tinput_to_frame\t")).expect("no input_to_frame record");
    let after = lines.len() - acked - 1;
    let longest = lines.iter().filter(|l| l.contains("\tscripts\t")).filter_map(|l| l.split('\t').nth(2)?.parse::<u64>().ok()).max().unwrap_or(0) / 1_000_000;
    assert!(after >= 3, "{after} scripts records after the input's ack:\n{}", lines.join("\n"));
    assert!(longest < 600, "one scripts task of {longest} ms");
    println!("{after} scripts records after the ack, longest {longest} ms");
}

/// Q4.29: a page in a renderer has a large response about as soon as in
/// the window's own process: the 16 MiB body of
/// `browser/tests/perf/large_body.html` arrives in under 2 s, in whatever
/// `DWELLER_PROCESS` the run has. The browser writes a renderer's
/// responses once a loop iteration, as much as the socket holds: with
/// macOS's 8 KiB the body took 4.5 s (in the window's own process 0.1 s).
#[test]
fn window_renderer_perf_large_body() {
    if !has_sdl() {
        println!("skipped: no SDL2");
        return;
    }
    let Some(srv) = server() else {
        println!("skipped: cannot start python3 for the test server");
        return;
    };
    let out = dweller_command()
        .current_dir(root())
        .arg(format!("{}/perf/large_body.html", srv.base))
        .args(["--font", "fixed", "--width", "400", "--height", "300", "--frames", "4000", "--until-idle", "--text"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}\n{}", out.status, text(&out.stderr));
    let got = text(&out.stdout);
    let ms: u64 = got
        .lines()
        .find_map(|l| l.strip_prefix("body 16777216 bytes in ")?.strip_suffix(" ms")?.parse().ok())
        .unwrap_or_else(|| panic!("no body line in {got}"));
    assert!(ms < 2000, "the 16 MiB body took {ms} ms");
    println!("the 16 MiB body took {ms} ms");
}
