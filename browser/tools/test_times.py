#!/usr/bin/env python3
"""Refresh tests/browser-times.txt from the shard logs of one CI run (DV2).

Reads the `--report-time` lines (`test NAME ... ok <1.234s>`) of the
`browser (OS, i/n)` shard jobs of a GitHub Actions run and rewrites the
times file that .github/scripts/browser-shard.sh balances the shards with:
one line per test, `NAME UBUNTU_SECONDS MACOS_SECONDS`, rounded to the
nearest second, sorted by name in byte order. NAME is the full name the
logs print (`agent::agent_http`), which the script matches first.

A test the run executed on one system only keeps its old time for the
other system, or DEFAULT (the script's time for an unknown test). A test
of the old file that the run did not execute at all is dropped. A test
re-run after a failure counts with its last passing time.

--split prints what the script's greedy split gives, with the old file
and the new one, for the tests in the logs: the sum of the times per
shard, per system, and the ten slowest macOS tests.

Usage: test_times.py RUN_ID            (logs through `gh run view --log`)
       test_times.py --logs DIR        (*.log files already downloaded)
       options: --file PATH (default tests/browser-times.txt), --split
"""
import argparse
import json
import os
import re
import subprocess
import sys

DEFAULT = 10
# A line of `gh run view --log`: job name, step, then the test's output.
LINE = re.compile(r"^browser \((\w+)[^,]*, \d+/(\d+)\)\t.*\btest (\S+) \.\.\. ok <([0-9.]+)s>")
SYSTEMS = ("ubuntu", "macos")  # column 2 and column 3 of the file


def gh(*args):
    return subprocess.run(["gh", *args], check=True, capture_output=True, text=True).stdout


def fetch_logs(run_id):
    jobs = json.loads(gh("run", "view", run_id, "--json", "jobs"))["jobs"]
    for job in jobs:
        if job["name"].startswith("browser ("):
            yield gh("run", "view", "--job", str(job["databaseId"]), "--log")


def read_logs(directory):
    for name in sorted(os.listdir(directory)):
        if name.endswith(".log"):
            with open(os.path.join(directory, name), encoding="utf-8", errors="replace") as fh:
                yield fh.read()


def parse(logs):
    """Return ({system: {test: seconds}}, {system: shard count})."""
    times = {s: {} for s in SYSTEMS}
    shards = {}
    for text in logs:
        for line in text.splitlines():
            m = LINE.match(line)
            if not m or m.group(1) not in times:
                continue
            system, n, test, seconds = m.groups()
            times[system][test] = float(seconds)
            shards[system] = int(n)
    return times, shards


def read_table(path):
    table = {}
    with open(path, encoding="utf-8") as fh:
        for line in fh:
            fields = line.split()
            if len(fields) == 3:
                table[fields[0]] = (int(fields[1]), int(fields[2]))
    return table


def lookup(table, test):
    """The script's lookup: the full name, else the name after the last `::`."""
    if test in table:
        return test
    short = test.rsplit("::", 1)[-1]
    return short if short in table else None


def refresh(old, times):
    new = {}
    kept = 0
    for test in set(times["ubuntu"]) | set(times["macos"]):
        key = lookup(old, test)
        row = []
        for col, system in enumerate(SYSTEMS):
            if test in times[system]:
                row.append(int(times[system][test] + 0.5))
            else:
                row.append(old[key][col] if key else DEFAULT)
                kept += 1
        new[test] = tuple(row)
    return new, kept


def split(table, tests, col, shards):
    """browser-shard.sh's greedy split; returns the sum of times per shard."""
    rows = []
    for test in tests:
        key = lookup(table, test)
        rows.append((table[key][col] if key else DEFAULT, key or test.rsplit("::", 1)[-1], test))
    rows.sort(key=lambda r: (-r[0], r[1], r))
    load = [0] * shards
    for t, _, _ in rows:
        load[load.index(min(load))] += t
    return load


def report_split(old, new, times, shards):
    for col, system in enumerate(SYSTEMS):
        tests = sorted(times[system])
        n = shards.get(system)
        if not n:
            continue
        before = split(old, tests, col, n)
        after = split(new, tests, col, n)
        print(f"{system} {n} shards, sum of seconds per shard: before {' / '.join(map(str, before))}, "
              f"after {' / '.join(map(str, after))}")
    print("ten slowest macOS tests:")
    for test, seconds in sorted(times["macos"].items(), key=lambda kv: -kv[1])[:10]:
        print(f"  {seconds:8.1f}s  {test}")


def main():
    root = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("run_id", nargs="?", help="GitHub Actions run id (logs through gh)")
    ap.add_argument("--logs", help="directory of downloaded job logs (*.log), instead of gh")
    ap.add_argument("--file", default=os.path.join(root, "tests", "browser-times.txt"))
    ap.add_argument("--split", action="store_true", help="print the predicted shard sums")
    args = ap.parse_args()
    if bool(args.run_id) == bool(args.logs):
        ap.error("give a run id or --logs DIR")

    times, shards = parse(read_logs(args.logs) if args.logs else fetch_logs(args.run_id))
    if not times["ubuntu"] and not times["macos"]:
        sys.exit("no `test NAME ... ok <Ns>` lines in the logs (run without --report-time?)")
    old = read_table(args.file)
    new, kept = refresh(old, times)
    with open(args.file, "w", encoding="utf-8") as fh:
        for test in sorted(new):
            fh.write(f"{test} {new[test][0]} {new[test][1]}\n")

    matched = {lookup(old, t) for t in new} - {None}
    print(f"{args.file}: {len(old)} tests before, {len(new)} after; "
          f"measured on ubuntu {len(times['ubuntu'])}, on macos {len(times['macos'])}; "
          f"{kept} test/system times not measured (old time or {DEFAULT}); "
          f"{len(old) - len(matched)} old tests not in the run, dropped")
    if args.split:
        report_split(old, new, times, shards)


if __name__ == "__main__":
    main()
