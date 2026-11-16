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
        .args(["--font", "fixed", "--width", "400", "--height", "300", "--frames", "2000", "--until-idle", "--state"])
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
