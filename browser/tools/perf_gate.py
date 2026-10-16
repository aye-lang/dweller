import json
import os
import statistics
import subprocess
import sys

from window_perf import iterations, pct, records, total

TESTS = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "tests")
RUNS = 3
RATCHET = 0.8
SLACK = {"raster_px": 1.02, "layout_full": 1.2}
UNRATCHETED = {"net_sync"}
SETTLE = 60
SCROLLS = 30


def serve():
    p = subprocess.Popen([sys.executable, os.path.join(TESTS, "server.py"), TESTS], stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)

    return p, int(p.stdout.readline())


def script():
    words = ["painted"] + ["frame"] * SETTLE + ["wheel:-3 frame"] * SCROLLS + ["frame"] * SETTLE

    return " ".join(words), len(words) + 10


def run(binary, port, page):
    events, frames = script()
    url = f"http://127.0.0.1:{port}/perf/{page}.html"
    env = dict(os.environ, DWELLER_TIMING="1", SDL_VIDEODRIVER=os.environ.get("SDL_VIDEODRIVER", "dummy"))
    r = subprocess.run([binary, url, "--frames", str(frames), "--events", events], capture_output=True, env=env, timeout=300)

    return records(r.stderr.decode("utf-8", "replace"))


def count(recs, *names):
    return sum(1 for k, _, _ in recs if k in names)


def raster(it):
    return sum(int(l.split()[0]) for k, _, l in it if k == "raster_px")


def scrolled(recs):
    its = iterations(recs)
    painted = next((i for i, it in enumerate(its) if any(k == "ui_first_paint" for k, _, _ in it)), 0)
    start = painted + SETTLE + 1

    return its[start:start + SCROLLS]


def measure(recs):
    scrolls = scrolled(recs)
    loops = [total(it, "ui_loop") for it in iterations(recs)]
    first = next((v for k, v, _ in recs if k == "ui_first_paint"), 0)
    counters = {
        "style_full": count(recs, "style_full"),
        "layout_full": count(recs, "layout_full", "layout"),
        "net_sync": count(recs, "net_sync"),
        "raster_px": round(sum(raster(it) for it in scrolls) / SCROLLS),
    }
    times = {
        "first_paint_ms": round(first),
        "longest_loop_ms": round(max(loops, default=0)),
        "scroll_p95_ms": round(pct([total(it, "ui_loop") for it in scrolls], 0.95), 1),
    }

    return counters, times


def median_of(page, binary, port):
    runs = [measure(run(binary, port, page)) for _ in range(RUNS)]
    counters = {k: statistics.median_low([c[k] for c, _ in runs]) for k in runs[0][0]}
    times = {k: statistics.median_low([t[k] for _, t in runs]) for k in runs[0][1]}

    return counters, times


def verdict(metric, value, budget):
    if budget is None:
        return "new"
    if value > budget * SLACK.get(metric, 1):
        return "OVER"
    if metric not in UNRATCHETED and value < budget * RATCHET:
        return "LOWER"

    return "ok"


def table(results, budgets):
    print(f"{'page':<12} {'metric':<16} {'value':>10} {'budget':>10}  verdict")
    bad = 0
    for page, (counters, times) in results.items():
        for k, v in counters.items():
            b = budgets.get(page, {}).get(k)
            mark = verdict(k, v, b)
            bad += mark in ("OVER", "LOWER", "new")
            print(f"{page:<12} {k:<16} {v:>10} {'-' if b is None else b:>10}  {mark}")
        for k, v in times.items():
            print(f"{page:<12} {k:<16} {v:>10} {'-':>10}  info")

    return bad


def lowered(results, budgets):
    out = {}
    for page, (counters, _) in results.items():
        old = budgets.get(page, {})
        out[page] = {k: min(v, old.get(k, v)) for k, v in counters.items()}

    return out


def gate(a):
    budgets = json.load(open(a.url))
    server, port = serve()
    try:
        results = {page: median_of(page, a.binary, port) for page in budgets}
    finally:
        server.terminate()
    bad = table(results, budgets)
    if a.update:
        json.dump(lowered(results, budgets), open(a.url, "w"), indent=2)
        open(a.url, "a").write("\n")
        print(f"budgets written to {a.url}")
        return
    if bad:
        sys.exit(f"perf gate: {bad} metric(s) over budget or due a ratchet (`gate --update` lowers budgets)")
