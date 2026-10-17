#!/usr/bin/env python3
"""Where the window's time goes on a live page (browser/engine/PERF.md,
"Window"). Standard library only.

    window_perf.py scroll URL [--binary B] [--frames N] [--wheel D] [--real-window]
        Headless window (--frames): the load, then N frames that each
        turn the wheel 3 notches (`wheel:-3 frame`; `--wheel -0.37` is a
        trackpad's 3.7 px a frame). Prints the first
        paint, the scroll frames' p50/p95 and the longest loop iteration.
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
import time

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
    events = " ".join(["frame"] * 3 + [f"wheel:{a.wheel} frame"] * a.frames)
    env = window_env(a.real_window, DWELLER_TIMING="1")
    cmd = [a.binary, a.url, "--frames", str(a.frames + 10), "--events", events, "--state"]
    r = subprocess.run(cmd, capture_output=True, env=env, timeout=600)
    err = r.stderr.decode("utf-8", "replace")
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
    }
    for phase in ("ui_loop", "ui_draw", "ui_timers", "ui_relayout"):
        xs = [total(it, phase) for it in frames]
        out[phase + "_p50_p95"] = [round(pct(xs, 0.5), 1), round(pct(xs, 0.95), 1)]
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
    ap.add_argument("mode", choices=["scroll", "idle", "gate"])
    ap.add_argument("url", help="the page, or the budgets of `gate`")
    ap.add_argument("--binary", default="target/dweller")
    ap.add_argument("--frames", type=int, default=40)
    ap.add_argument("--wheel", default="-3")
    ap.add_argument("--real-window", action="store_true", help="open a real window, without taking the focus")
    ap.add_argument("--minimize", action="store_true", help="idle: the window is minimized after its first frame")
    ap.add_argument("--settle", type=int, default=5, help="idle: seconds between the first frame and the measure")
    ap.add_argument("--log", help="keep the DWELLER_TIMING records here")
    ap.add_argument("--update", action="store_true", help="gate: lower the budgets to the values measured")
    a = ap.parse_args()
    if not os.path.exists(a.binary):
        sys.exit(f"no binary {a.binary} (browser/build.sh)")
    if a.mode == "gate":
        import perf_gate
        perf_gate.gate(a)
    else:
        scroll(a) if a.mode == "scroll" else idle(a)


if __name__ == "__main__":
    main()
