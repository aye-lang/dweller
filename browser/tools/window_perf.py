#!/usr/bin/env python3
"""Where the window's time goes on a live page (browser/engine/PERF.md,
"Window"). Standard library only.

    window_perf.py scroll URL [--binary B] [--frames N] [--wheel D] [--real-window]
                       [--step WORDS] [--run-frames M]
        Headless window (--frames): the load, then N frames that each
        turn the wheel 3 notches (`wheel:-3 frame`; `--wheel -0.37` is a
        trackpad's 3.7 px a frame). Prints the first
        paint, the scroll frames' p50/p95 and the longest loop iteration,
        the delay from each wheel or click to the next present
        (`input_to_present`) and the longest script task (`timers`,
        `scripts` or `ui_raf` record; with DWELLER_PROCESS=site these come
        from the renderers, the `ui_` loop records from the window).
        --step replaces `wheel:D` in each frame (Q4.25, with a renderer:
        `renderer:loaded wheel:-3 renderer:input`, or a click
        `renderer:loaded click:X,Y release:X,Y renderer:input`), then
        closes the tab; the window runs at most M iterations (--run-frames,
        default N + 10).
    window_perf.py idle URL [--binary B] [--real-window] [--minimize] [--settle S]
        A window, no input: the CPU% of the 10 s that start 5 s
        after the first frame (--settle S: S s), and what the loop did
        meanwhile.
        --minimize feeds the window's minimized event (`window:hidden`,
        P4.55) after the first frame; the window itself is not touched.
    window_perf.py gate BUDGETS.json [--binary B] [--update]
        The CI perf gate (P4.40, perf_gate.py): the synthetic pages of
        browser/tests/perf/ over browser/tests/server.py, the median of
        3 runs of each; exits 1 when a counter (`style_full`,
        `layout_full`, `net_sync`, `raster_px`) is over its budget or
        more than 20% under it. --update lowers the budgets (never
        raises them).
    window_perf.py report (--replay | --live --chrome) [--site NAME] [--archives DIR]
        W7 and W8 (P4.41, window_report.py), a row per non-excluded site of
        sites.json; exits 1 when a row fails. --replay: `scroll` and `idle`
        on the archives of DIR (default target/sites) against W7's limits.
        --live --chrome: Dweller's first paint against Chrome's first
        contentful paint (agent-browser), the median of 3 alternating runs.

Both use SDL's dummy video driver; --real-window opens a real window
that does not take the focus (DWELLER_BACKGROUND=1).

Both read the DWELLER_TIMING records (`ui_navigate`, `ui_first_paint`,
`ui_relayout`, `ui_draw`, `ui_events`, `ui_timers`, `ui_loop`, and
`layout_full`/`layout` vs `layout_dirty`); --log
keeps them.
"""
import argparse
import json
import os
import re
import subprocess
import sys
import threading
import time

import fast_build
from window_env import window_env

RECORD = re.compile(r"DWELLER_TIMING\t([^\t]+)\t(\d+)\t([^\n]*)")


def records(err):
    return [(m.group(1), int(m.group(2)) / 1e6, m.group(3)) for m in RECORD.finditer(err)]


def iterations(recs):
    """The records of each loop iteration (each ends with `ui_loop`)."""
    its, cur = [], []
    for r in recs:
        cur.append(r)
        if r[0] == "ui_loop":
            its.append(cur)
            cur = []
    return its


def pct(xs, q):
    if not xs:
        return 0.0
    xs = sorted(xs)
    return xs[min(len(xs) - 1, int(q * len(xs)))]


def total(it, phase):
    return sum(v for k, v, _ in it if k == phase)


def scroll(a):
    step = a.step or f"wheel:{a.wheel}"
    # After the --step frames the window closes its tab (and quits), so
    # that --run-frames is only a cap.
    events = " ".join(["frame"] * 3 + [f"{step} frame"] * a.frames + (["key:cmd+w"] if a.step else []))
    env = window_env(a.real_window, DWELLER_TIMING="1")
    cmd = [a.binary, a.url, "--frames", str(a.run_frames or a.frames + 10), "--events", events, "--state"]
    p = subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)
    timer = threading.Timer(600, p.kill)
    timer.start()
    # When each step's first present reached us. With a renderer the window
    # presents before the renderer's frame, and the next step waits for the
    # renderer's ack (`renderer:input`): the time between two of them is a
    # step as the person sees it, to one window iteration.
    lines, shown_at = [], []
    for line in p.stderr:
        lines.append(line)
        if b"\tinput_to_present\t" in line:
            shown_at.append(time.monotonic())
    state = p.stdout.read().decode("utf-8", "replace")
    p.wait()
    timer.cancel()
    err = b"".join(lines).decode("utf-8", "replace")
    if a.log:
        open(a.log, "w").write(err)
    recs = records(err)
    first = {}
    for k, v, _ in recs:
        first.setdefault(k, v)
    its = iterations(recs)
    frames = [it for it in its if any(k == "ui_events" and l == "script" for k, _, l in it)][3:]
    out = {
        "url": a.url,
        # `ui_first_paint` (P4.33): from the navigation to the first frame
        # that shows the page; before it, the load ran before the loop.
        "first_paint_ms": round(first.get("ui_first_paint", first.get("ui_navigate", 0) + first.get("ui_relayout", 0) + first.get("ui_draw", 0))),
        "navigate_ms": round(first.get("ui_navigate", 0)),
        "first_relayout_ms": round(first.get("ui_relayout", 0)),
        "relayouts": sum(1 for k, _, _ in recs if k == "ui_relayout"),
        "longest_loop_ms": round(max((total(it, "ui_loop") for it in its), default=0)),
        "longest_task_ms": round(max((v for k, v, _ in recs if k in ("timers", "scripts", "ui_raf")), default=0)),
        "state": state.split("\n", 1)[0],
    }
    for phase in ("ui_loop", "ui_draw", "ui_timers", "ui_relayout"):
        xs = [total(it, phase) for it in frames]
        out[phase + "_p50_p95"] = [round(pct(xs, 0.5), 1), round(pct(xs, 0.95), 1)]
    shown = [v for k, v, _ in recs if k == "input_to_present"]
    out["input_to_present_n_p50_p95"] = [len(shown), round(pct(shown, 0.5), 1), round(pct(shown, 0.95), 1)]
    steps = [1000 * (b - a) for a, b in zip(shown_at, shown_at[1:])]
    out["step_ms_n_p50_p95_max"] = [len(steps), round(pct(steps, 0.5), 1), round(pct(steps, 0.95), 1), round(max(steps, default=0), 1)]
    print(json.dumps(out))


def idle(a):
    env = window_env(a.real_window, DWELLER_TIMING="1")
    path = a.log
    if not path:
        import tempfile
        fd, path = tempfile.mkstemp(suffix=".log")
        os.close(fd)
    f = open(path, "w")
    t0 = time.monotonic()
    cmd = [a.binary, a.url]
    if a.minimize:
        cmd += ["--frames", "16000", "--events", "painted window:hidden"]
    p = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=f, env=env)

    def cpu():
        s = subprocess.run(["ps", "-o", "time=", "-p", str(p.pid)], capture_output=True, text=True).stdout.strip()
        # [[DD-]HH:]MM:SS (procps) or MM:SS.ss (macOS).
        days, _, s = s.rpartition("-")
        t = 0.0
        for part in s.split(":"):
            t = t * 60 + float(part)
        return t + int(days or 0) * 86400

    try:
        drawn = None
        while time.monotonic() - t0 < 300 and p.poll() is None:
            if "\tui_draw\t" in open(path).read():
                drawn = time.monotonic() - t0
                break
            time.sleep(0.1)
        time.sleep(a.settle)
        start = os.path.getsize(path)
        c0, w0 = cpu(), time.monotonic()
        time.sleep(10)
        c1, w1 = cpu(), time.monotonic()
    finally:
        p.terminate()
        try:
            p.wait(5)
        except subprocess.TimeoutExpired:
            p.kill()
        f.close()
    recs = records(open(path).read()[start:])
    loops = [v for k, v, _ in recs if k == "ui_loop"]
    print(json.dumps({
        "url": a.url,
        "first_frame_s": round(drawn, 1) if drawn else None,
        "idle_cpu_pct": round(100 * (c1 - c0) / (w1 - w0), 1),
        "iterations": len(loops),
        "relayouts": sum(1 for k, _, _ in recs if k == "ui_relayout"),
        # The layouts scripts and relayouts did: of the whole document,
        # or of the boxes of the elements that changed (P4.57).
        "full_layouts": sum(1 for k, _, _ in recs if k in ("layout_full", "layout")),
        "dirty_layouts": sum(1 for k, _, _ in recs if k == "layout_dirty"),
        "draws": sum(1 for k, _, _ in recs if k == "ui_draw"),
        "longest_loop_ms": round(max(loops, default=0)),
    }))
    if not a.log:
        os.remove(path)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("mode", choices=["scroll", "idle", "gate", "report"])
    ap.add_argument("url", nargs="?", help="the page, or the budgets of `gate`")
    ap.add_argument("--binary", default="target/dweller")
    ap.add_argument("--allow-debug", action="store_true", help="time a binary that browser/build.sh did not build")
    ap.add_argument("--frames", type=int, default=40)
    ap.add_argument("--wheel", default="-3")
    ap.add_argument("--step", help="scroll: the words of each frame instead of `wheel:D`")
    ap.add_argument("--run-frames", type=int, help="scroll: the window's iterations (default --frames + 10)")
    ap.add_argument("--real-window", action="store_true", help="open a real window, without taking the focus")
    ap.add_argument("--minimize", action="store_true", help="idle: the window is minimized after its first frame")
    ap.add_argument("--settle", type=int, default=5, help="idle: seconds between the first frame and the measure")
    ap.add_argument("--log", help="keep the DWELLER_TIMING records here")
    ap.add_argument("--replay", action="store_true", help="report: W7 on the archives")
    ap.add_argument("--live", action="store_true", help="report: W8 on the live sites")
    ap.add_argument("--chrome", action="store_true", help="report --live: compare with Chrome")
    ap.add_argument("--site", action="append", help="report: only this site (repeatable)")
    ap.add_argument("--archives", default="target/sites", help="report: the archive directory")
    ap.add_argument("--update", action="store_true", help="gate: lower the budgets to the values measured")
    a = ap.parse_args()
    if not os.path.exists(a.binary):
        sys.exit(f"no binary {a.binary} (browser/build.sh)")
    if a.mode != "gate":
        fast_build.require(a.binary, a.allow_debug)
    if a.mode == "gate":
        import perf_gate
        perf_gate.gate(a)
    elif a.mode == "report":
        import window_report
        window_report.report(a)
    else:
        scroll(a) if a.mode == "scroll" else idle(a)


if __name__ == "__main__":
    main()
