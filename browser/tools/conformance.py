#!/usr/bin/env python3
"""Per-area conformance table: run, table, check.

  conformance.py run [-j N] [AREA...]   run wpt.py / test262.py per area
  conformance.py table                  write docs/conformance.md and the per-area baselines
  conformance.py check                  exit 1 when an area fell more than 0.5 points
"""
import argparse
import csv
import datetime
import json
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
TOOLS = REPO / "browser" / "tools"
BASELINES = TOOLS / "conformance"
HISTORY = BASELINES / "history.csv"
DOC = REPO / "docs" / "conformance.md"
WPT_RESULTS = REPO / "target" / "wpt-results"
T262_RESULTS = REPO / "target" / "test262-results"
WEBGL_RESULTS = REPO / "target" / "webgl-results"
T262_AREAS = {"test262/built-ins": "test/built-ins", "test262/language": "test/language"}
T262_THRESHOLD = 0.80
TOLERANCE = 0.005


def wpt_areas():
    return json.loads((TOOLS / "wpt_areas.json").read_text())["areas"]


def planned_areas():
    lines = (BASELINES / "planned.txt").read_text().splitlines()
    return [line.split("\t") for line in lines if line]


def webgl_areas():
    files = [*BASELINES.glob("webgl*.json"), *WEBGL_RESULTS.glob("webgl*.json")]
    return sorted({json.loads(f.read_text())["area"] for f in files})


def is_webgl(area):
    return area.startswith("WebGL ")


def all_areas():
    return list(wpt_areas()) + list(T262_AREAS) + webgl_areas()


def file_key(area):
    return area.replace("WebGL ", "webgl").replace(": ", "_").replace("/", "_")


def run_command(area, jobs):
    if area in T262_AREAS:
        return [sys.executable, str(TOOLS / "test262.py"), "run", T262_AREAS[area], "-j", str(jobs)]
    return [sys.executable, str(TOOLS / "wpt.py"), "run", area, "-j", str(jobs)]


def cmd_run(args):
    for area in args.areas or list(wpt_areas()) + list(T262_AREAS):
        print(f"== {area}", flush=True)
        subprocess.run(run_command(area, args.j), check=True)


def wpt_measure(area):
    data = json.loads(results_path(area).read_text())
    tests = {f["path"]: [f["passed"], f["total"]] for f in data["files"]}
    threshold = data["threshold"] if is_webgl(area) else wpt_areas()[area]["threshold"]
    return data["rev"], tests, data["skipped"], threshold


def t262_measure(area):
    name = T262_AREAS[area].replace("/", "-")
    data = json.loads((T262_RESULTS / f"{name}.json").read_text())
    tests = {t: [int(v[0] == "pass"), 1] for t, v in data["tests"].items() if v[0] != "skip"}
    return data["rev"], tests, data["skipped"], T262_THRESHOLD


def measure(area):
    rev, tests, skipped, threshold = (t262_measure if area in T262_AREAS else wpt_measure)(area)
    passed = sum(p for p, _ in tests.values())
    total = sum(n for _, n in tests.values())
    return {"area": area, "rev": rev, "files": len(tests), "skipped": skipped,
            "passed": passed, "total": total, "threshold": threshold,
            "score": passed / total if total else 0.0, "tests": dict(sorted(tests.items()))}


def results_path(area):
    if area in T262_AREAS:
        return T262_RESULTS / f"{T262_AREAS[area].replace('/', '-')}.json"
    return (WEBGL_RESULTS if is_webgl(area) else WPT_RESULTS) / f"{file_key(area)}.json"


def available(area):
    return results_path(area).exists()


def read_baseline(area):
    path = BASELINES / f"{file_key(area)}.json"
    return json.loads(path.read_text()) if path.exists() else None


def last_scores():
    if not HISTORY.exists():
        return {}
    with HISTORY.open() as f:
        return {row["area"]: float(row["score"]) for row in csv.DictReader(f)}


def append_history(rows):
    fresh = not HISTORY.exists()
    with HISTORY.open("a", newline="") as f:
        w = csv.writer(f)
        if fresh:
            w.writerow(["date", "commit", "area", "passed", "total", "score"])
        w.writerows(rows)


def head_commit():
    out = subprocess.run(["git", "rev-parse", "--short", "HEAD"], cwd=REPO, capture_output=True, text=True)
    return out.stdout.strip()


def status_of(m):
    if not m["threshold"]:
        return "baseline"
    return "done" if m["score"] >= m["threshold"] else "below"


def change_cell(m, before):
    if m["area"] not in before:
        return "new"
    return f"{(round(m['score'], 4) - before[m['area']]) * 100 + 0.0:+.1f}"


def table_row(m, before):
    th = f"{m['threshold'] * 100:.0f}" if m["threshold"] else "-"
    return (f"| `{m['area']}` | {m['files']} | {m['skipped']} | {m['passed']}/{m['total']} "
            f"| {m['score'] * 100:.1f} | {th} | {status_of(m)} | {change_cell(m, before)} |")


def render(measured, before, revs, today, commit):
    head = ["# Conformance", "",
            f"Measured {today} at Dweller {commit}. WPT `{revs['wpt']}`, test262 `{revs['t262']}`, WebGL `{revs['webgl']}`.",
            "Score is passed over total subtests (WPT, WebGL) or tests (test262), skipped files excluded.",
            "Change is in points since the previous table. Regenerate with `conformance.py run`, then `table`.",
            "", "| Area | Files | Skipped | Passed | Score % | Threshold % | Status | Change |",
            "|---|---:|---:|---:|---:|---:|---|---:|"]
    rows = [table_row(m, before) for m in measured]
    notes = ["", "## Not yet measured", "", "| Area | Threshold % |", "|---|---:|"]
    notes += [f"| {name} | {th} |" for name, th in planned_areas()]
    return "\n".join(head + rows + notes) + "\n"


def revision(path):
    return path.read_text().split()[0][:10]


def cmd_table(args):
    fresh = [measure(a) for a in all_areas() if available(a)]
    kept = [m for m in map(read_baseline, all_areas()) if m and m["area"] not in {f["area"] for f in fresh}]
    measured = sorted(fresh + kept, key=lambda m: all_areas().index(m["area"]))
    before = last_scores()
    revs = {"wpt": revision(TOOLS / "wpt.rev"), "t262": revision(TOOLS / "test262.rev"),
            "webgl": revision(TOOLS / "webgl_cts.rev")}
    today = datetime.date.today().isoformat()
    DOC.write_text(render(measured, before, revs, today, head_commit()))
    for m in fresh:
        (BASELINES / f"{file_key(m['area'])}.json").write_text(json.dumps(m, indent=0) + "\n")
    append_history([[today, head_commit(), m["area"], m["passed"], m["total"], f"{m['score']:.4f}"] for m in fresh])
    print(f"wrote {DOC} for {len(measured)} areas")


def regressions(old, new):
    return [t for t, (p, n) in old["tests"].items() if p == n and new["tests"].get(t, [0, 1])[0] < n]


def check_area(area):
    old = read_baseline(area)
    if old is None or not available(area):
        return []
    new = measure(area)
    if old["score"] - new["score"] <= TOLERANCE:
        return []
    drop = (old["score"] - new["score"]) * 100
    return [f"{area}: {old['score'] * 100:.1f} -> {new['score'] * 100:.1f} (-{drop:.1f} points)"] + \
           [f"  {t}" for t in regressions(old, new)]


def cmd_check(args):
    problems = [line for area in all_areas() for line in check_area(area)]
    print("\n".join(problems) if problems else "conformance: no area fell")
    return 1 if problems else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("run")
    r.add_argument("areas", nargs="*")
    r.add_argument("-j", type=int, default=4)
    sub.add_parser("table")
    sub.add_parser("check")
    args = ap.parse_args()
    return {"run": cmd_run, "table": cmd_table, "check": cmd_check}[args.cmd](args) or 0


if __name__ == "__main__":
    sys.exit(main())
