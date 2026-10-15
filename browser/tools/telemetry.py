#!/usr/bin/env python3
"""Summarizes Dweller's local telemetry (docs/browser.md, "Telemetry").

    python3 browser/tools/telemetry.py summary [--days N] [--profile DIR]

Reads `<profile>/telemetry/*.jsonl` of the last N days (default 7; the
profile is --profile, else $DWELLER_PROFILE, else ~/.dweller), groups the
events into issues and ranks them by frequency times the number of sites
affected, as input for new cards. Then it lists each site's page timings
(median first paint and load) and scroll frames (median p50/p95), and the
slow loops by the phase that took longest.
"""

import argparse
import datetime
import json
import os
import re
import statistics
import sys


def profile_dir(arg):
    if arg:
        return arg
    if os.environ.get("DWELLER_PROFILE"):
        return os.environ["DWELLER_PROFILE"]
    return os.path.join(os.path.expanduser("~"), ".dweller")


def day_files(directory, days):
    """The .jsonl files of the last `days` days (UTC), oldest first."""
    if not os.path.isdir(directory):
        return []
    first = (datetime.datetime.now(datetime.timezone.utc).date()
             - datetime.timedelta(days=days - 1)).isoformat()
    names = []
    for name in os.listdir(directory):
        m = re.match(r"^(\d{4}-\d{2}-\d{2})(-\d+)?\.jsonl$", name)
        if m and m.group(1) >= first:
            names.append(name)
    return [os.path.join(directory, n) for n in sorted(names)]


def read_events(files):
    events = []
    for path in files:
        with open(path, encoding="utf-8", errors="replace") as f:
            for line in f:
                line = line.strip()
                if not line:
                    continue
                try:
                    events.append(json.loads(line))
                except ValueError:
                    # A line cut by a crash.
                    continue
    return events


def normalize(message):
    """Origins and numbers vary between occurrences of one issue."""
    message = re.sub(r"[A-Za-z][A-Za-z0-9+.-]*://[^\s\"'),<>]*", "<origin>", message or "")
    return re.sub(r"\d+", "N", message)


def issue_of(ev):
    """(event, issue) naming what an event is about, or None for the
    timing records."""
    kind = ev.get("ev")
    if kind == "crash":
        frames = ev.get("backtrace") or []
        return kind, "%s at %s" % (ev.get("signal", "?"), frames[0] if frames else "?")
    if kind == "js_error":
        return kind, "%s: %s" % (ev.get("type", "?"), normalize(ev.get("message")))
    if kind == "unsupported":
        return kind, "%s %s" % (ev.get("feature", "?"), ev.get("name", "?"))
    if kind == "load_failed":
        status = ev.get("status", 0)
        return kind, "%s %s" % (ev.get("kind", "?"), status if status else "network error")
    if kind == "image_decode":
        return kind, "%s %s" % (ev.get("format", "?"), ev.get("error", "?"))
    if kind == "slow_loop":
        phases = ev.get("phases") or {}
        top = max(phases, key=lambda p: phases[p]) if phases else "?"
        if phases and phases[top] == 0:
            top = "other"
        return kind, "loop > 100 ms, mostly %s" % top
    return None


def summary(events, out):
    issues = {}
    pages = {}
    scrolls = {}
    for ev in events:
        site = ev.get("site") or "(no site)"
        if ev.get("ev") == "page":
            pages.setdefault(site, []).append(ev)
            continue
        if ev.get("ev") == "scroll":
            scrolls.setdefault(site, []).append(ev)
            continue
        key = issue_of(ev)
        if key is None:
            continue
        entry = issues.setdefault(key, {"count": 0, "sites": set()})
        entry["count"] += 1
        entry["sites"].add(site)
    ranked = sorted(issues.items(),
                    key=lambda kv: (-kv[1]["count"] * len(kv[1]["sites"]), kv[0]))
    out.write("%d events, %d issues\n\n" % (len(events), len(ranked)))
    if ranked:
        out.write("%6s %6s %5s  %-13s %s\n" % ("score", "count", "sites", "event", "issue"))
        for (kind, issue), e in ranked:
            sites = sorted(e["sites"])
            more = " (+%d)" % (len(sites) - 3) if len(sites) > 3 else ""
            out.write("%6d %6d %5d  %-13s %s\n" % (
                e["count"] * len(sites), e["count"], len(sites), kind, issue))
            out.write("%27s%s%s\n" % ("", ", ".join(sites[:3]), more))
    if pages or scrolls:
        out.write("\n%-40s %6s %12s %8s %10s %10s\n" % (
            "site", "loads", "first paint", "load", "scroll p50", "scroll p95"))
        for site in sorted(set(pages) | set(scrolls)):
            p = pages.get(site, [])
            s = scrolls.get(site, [])

            def med(items, field):
                values = [i[field] for i in items if isinstance(i.get(field), (int, float))]
                return "%d ms" % statistics.median(values) if values else "-"

            out.write("%-40s %6d %12s %8s %10s %10s\n" % (
                site, len(p), med(p, "first_paint_ms"), med(p, "load_ms"),
                med(s, "p50_ms"), med(s, "p95_ms")))


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    s = sub.add_parser("summary", help="rank the issues of the last days")
    s.add_argument("--days", type=int, default=7)
    s.add_argument("--profile", help="the profile directory (default $DWELLER_PROFILE or ~/.dweller)")
    args = parser.parse_args(argv)
    if args.days < 1:
        parser.error("--days must be at least 1")
    directory = os.path.join(profile_dir(args.profile), "telemetry")
    files = day_files(directory, args.days)
    if not files:
        print("no telemetry in %s for the last %d day(s)" % (directory, args.days))
        return 0
    summary(read_events(files), sys.stdout)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
