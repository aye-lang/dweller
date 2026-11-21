#!/usr/bin/env python3
"""The CI cells of the scoreboard (docs/plans/foundations.md), as Markdown.

  scoreboard.py summary        every CI cell, for the run summary of the weekly `conformance` workflow
  scoreboard.py flaky [--all]  S1: merge-queue runs of `ci` since the last flaky test

summary reads the conformance results under target/ where a job of the
run left them and the committed baseline for every other area. The rest
comes from GitHub through `gh` (GH_TOKEN and GH_REPO, or a checkout).

A merge-queue run had a flaky test when it uploaded a `flaky-tests-*`
artifact (a test failed, then passed on the re-run of its shard) or when
it is green on a second attempt. flaky counts the green runs newer than
the newest such run, and with --all it lists every such run instead.
"""
import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import conformance
import conformance_targets

NEAR_FLOOR = 0.035
REPO_API = "repos/{owner}/{repo}"
MAX_RUNS = 500


def gh(*args):
    return subprocess.run(["gh", *args], check=True, capture_output=True, text=True).stdout


def api(path):
    return json.loads(gh("api", f"{REPO_API}/{path}"))


def area_now(area):
    return conformance.measure(area) if conformance.available(area) else conformance.read_baseline(area)


def conformance_lines():
    fresh = [a for a in conformance.all_areas() if conformance.available(a)]
    areas = [m for m in map(area_now, conformance.all_areas()) if m]
    at = [m for m in areas if conformance.status_of(m) == "done"]
    below = [m for m in areas if conformance.status_of(m) == "below"]
    near = [m for m in at if m["score"] - m["threshold"] <= NEAR_FLOOR]
    fell = [line for a in fresh for line in conformance.check_area(a)]
    lines = ["### W9 conformance", "",
             f"{len(at)} areas at their floor, {len(below)} below, {len(areas) - len(at) - len(below)} without a floor; "
             f"{len(near)} of the {len(at)} are within {NEAR_FLOOR * 100:.1f} points of the floor. "
             f"Measured in this run: {len(fresh)} of {len(areas)} areas (the others are their baselines).", ""]
    lines += [f"- below: `{m['area']}` {m['score'] * 100:.1f} (floor {m['threshold'] * 100:.0f})" for m in below]
    lines += [f"- near: `{m['area']}` {m['score'] * 100:.1f} (floor {m['threshold'] * 100:.0f})" for m in near]

    lines += ["", target_line(areas)]

    return lines + ["", "Fell against the baseline:" if fell else "No area fell against its baseline.", *fell]


def target_line(areas):
    targets = {a: e["target"] for a, e in conformance_targets.read()["areas"].items() if "target" in e}
    reached = [m["area"] for m in areas if m["area"] in targets and m["score"] * 100 >= targets[m["area"]]]
    names = ": " + ", ".join(f"`{a}`" for a in reached) if reached else ""
    return f"{len(reached)} of the {len(targets)} areas with a target (X2, Q6) are at it{names}."


def count_lines(directory, name):
    return sum(len(f.read_text().split()) for f in Path(directory).glob(f"*/{name}"))


def single_mode_lines():
    runs = api("actions/workflows/single-mode.yml/runs?branch=main&status=completed&per_page=1")["workflow_runs"]

    if not runs:
        return ["### The debugging mode (Q2.14)", "", "No finished `single-mode` run on `main`."]

    run = runs[0]

    with tempfile.TemporaryDirectory() as shards:
        gh("run", "download", str(run["id"]), "--pattern", "single-mode-*", "--dir", shards)
        passed, failed = count_lines(shards, "ok.txt"), count_lines(shards, "failed.txt")

    return ["### The debugging mode (Q2.14)", "",
            f"{passed} of {passed + failed} tests pass with `DWELLER_PROCESS=single` "
            f"(run {run['id']}, {run['created_at'][:10]}, {run['head_sha'][:8]})."]


def open_issues(label):
    found = api(f"issues?labels={label}&state=open&per_page=100")
    return [i for i in found if "pull_request" not in i]


def issue_lines():
    lines = ["### Open `flaky` and `fuzz` issues", ""]

    for label in ["flaky", "fuzz"]:
        issues = open_issues(label)
        lines.append(f"- `{label}`: {len(issues)} open")
        lines += [f"  - #{i['number']} {i['title']}" for i in issues]

    return lines


def queue_runs():
    for page in range(1, MAX_RUNS // 100 + 1):
        runs = api(f"actions/workflows/ci.yml/runs?event=merge_group&status=completed&per_page=100&page={page}")["workflow_runs"]
        yield from runs

        if len(runs) < 100:
            return


def flaky_artifacts(run):
    artifacts = api(f"actions/runs/{run['id']}/artifacts?per_page=100")["artifacts"]
    return sorted(a["name"] for a in artifacts if a["name"].startswith("flaky-tests-"))


def flaky_reason(run):
    names = flaky_artifacts(run)

    if names:
        return ", ".join(names)

    return f"green on attempt {run['run_attempt']}" if run["run_attempt"] > 1 and run["conclusion"] == "success" else ""


def describe(run, reason):
    return f"run {run['id']} ({run['created_at'][:10]}): {reason}"


def all_flaky():
    runs = list(queue_runs())
    flaky = [describe(r, reason) for r in runs if (reason := flaky_reason(r))]
    return [f"{len(flaky)} of {len(runs)} merge-queue runs of `ci` had a flaky test.", *[f"- {line}" for line in flaky]]


def flaky_lines():
    green = 0

    for run in queue_runs():
        reason = flaky_reason(run)

        if reason:
            return [f"S1: {green} green merge-queue runs of `ci` since the last flaky test; last flaky: {describe(run, reason)}."]

        green += run["conclusion"] == "success"

    return [f"S1: {green} green merge-queue runs of `ci`, none with a flaky test (every run GitHub lists, at most {MAX_RUNS})."]


def cmd_summary(args):
    sections = [["## Scoreboard, the CI cells"], conformance_lines(), single_mode_lines(), issue_lines(),
                ["### S1 no flaky test in 50 runs", "", *flaky_lines()],
                ["### W2 and W7", "", "Not measured on CI: the site archives are on the owner's Mac (DV5)."]]
    print("\n\n".join("\n".join(s) for s in sections))


def cmd_flaky(args):
    print("\n".join(all_flaky() if args.all else flaky_lines()))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    sub.add_parser("summary")
    sub.add_parser("flaky").add_argument("--all", action="store_true", help="list every run with a flaky test")
    args = ap.parse_args()
    return {"summary": cmd_summary, "flaky": cmd_flaky}[args.cmd](args) or 0


if __name__ == "__main__":
    sys.exit(main())
