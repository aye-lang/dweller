import json
import os
import statistics
import subprocess
import sys
from pathlib import Path

TOOLS = Path(__file__).resolve().parent
LOOP_MS = 100
SCROLL_P95_MS = 33
IDLE_CPU_PCT = 5
FIRST_PAINT_GAP_MS = 1000
LIVE_RUNS = 3
SESSION = f"window-report-{os.getpid()}"
FCP_SCRIPT = "JSON.stringify(performance.getEntriesByType('paint').sort((a, b) => a.name < b.name ? -1 : 1).map(e => e.startTime))"


def sites(a):
    names = json.loads((TOOLS / "sites.json").read_text())["sites"]

    return [n for n, v in names.items() if v["status"] != "excluded" and (not a.site or n in a.site)]


def archive_url(a, name):
    return json.loads((Path(a.archives) / name / "manifest.json").read_text())["url"]


def measure(a, mode, url, *extra, **env):
    cmd = [sys.executable, str(TOOLS / "window_perf.py"), mode, url, "--binary", a.binary, *extra]
    r = subprocess.run(cmd, capture_output=True, text=True, env=dict(os.environ, **env), timeout=900)

    return json.loads(r.stdout.strip().splitlines()[-1]) if r.returncode == 0 and r.stdout.strip() else None


def replay_row(a, name):
    archive = str((Path(a.archives) / name).resolve())
    url = archive_url(a, name)
    scroll = measure(a, "scroll", url, DWELLER_REPLAY=archive)
    idle = measure(a, "idle", url, DWELLER_REPLAY=archive)

    return name, scroll, idle


def replay_failures(scroll, idle):
    if not scroll or not idle:
        return ["no measurement"]
    p95 = scroll["ui_loop_p50_p95"][1]
    checks = [
        (scroll["longest_loop_ms"] >= LOOP_MS, f"loop {scroll['longest_loop_ms']} ms"),
        (p95 >= SCROLL_P95_MS, f"scroll p95 {p95} ms"),
        (idle["idle_cpu_pct"] >= IDLE_CPU_PCT, f"idle {idle['idle_cpu_pct']}%"),
    ]

    return [why for failed, why in checks if failed]


def replay_line(name, scroll, idle):
    bad = replay_failures(scroll, idle)
    if not scroll or not idle:
        return f"{name:<14} {'-':>8} {'-':>14} {'-':>6}  FAIL {bad[0]}", False
    p50, p95 = scroll["ui_loop_p50_p95"]
    cells = f"{name:<14} {scroll['longest_loop_ms']:>8} {p50:>6}/{p95:<7} {idle['idle_cpu_pct']:>6}"

    return f"{cells}  {'FAIL ' + ', '.join(bad) if bad else 'ok'}", not bad


def report_replay(a):
    print(f"{'site':<14} {'loop ms':>8} {'scroll p50/p95':>14} {'idle %':>6}  W7", flush=True)
    results = []
    for name in sites(a):
        line, ok = replay_line(*replay_row(a, name))
        print(line, flush=True)
        results.append(ok)

    return all(results)


def dweller_first_paint(a, url):
    out = measure(a, "scroll", url, "--frames", "3")

    return out["first_paint_ms"] if out else None


def browser(*args, timeout=120):
    cmd = ["agent-browser", "--session", SESSION, *args]

    return subprocess.run(cmd, capture_output=True, text=True, timeout=timeout).stdout.strip()


def chrome_fcp(url):
    browser("open", url)
    out = browser("eval", FCP_SCRIPT)
    browser("close")
    times = json.loads(json.loads(out or '"[]"'))

    return times[0] if times else None


def live_row(a, name):
    url = archive_url(a, name)
    dweller, chrome = [], []
    for _ in range(LIVE_RUNS):
        dweller.append(dweller_first_paint(a, url))
        chrome.append(chrome_fcp(url))

    return name, dweller, chrome


def median(xs):
    xs = [x for x in xs if x is not None]

    return statistics.median(xs) if xs else None


def live_line(name, dweller, chrome):
    d, c = median(dweller), median(chrome)
    if d is None or c is None:
        return f"{name:<14} {'-':>10} {'-':>10} {'-':>8}  FAIL no measurement", False
    gap = d - c

    return f"{name:<14} {d:>10.0f} {c:>10.0f} {gap:>8.0f}  {'ok' if gap <= FIRST_PAINT_GAP_MS else 'FAIL gap over 1000 ms'}", gap <= FIRST_PAINT_GAP_MS


def report_live(a):
    print(f"{'site':<14} {'dweller ms':>10} {'chrome ms':>10} {'gap ms':>8}  W8", flush=True)
    results = []
    for name in sites(a):
        line, ok = live_line(*live_row(a, name))
        print(line, flush=True)
        results.append(ok)

    return all(results)


def report(a):
    if a.live and not a.chrome:
        sys.exit("report --live needs --chrome")
    ok = report_live(a) if a.live else report_replay(a)
    sys.exit(0 if ok else 1)
