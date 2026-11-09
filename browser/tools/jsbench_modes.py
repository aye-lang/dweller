#!/usr/bin/env python3
"""Median wall time of jsbench workloads for several builds, interleaved.

    python3 browser/tools/jsbench_modes.py [--runs N] NAME=BIN... [-- WORKLOAD...]

Without WORKLOADs it runs every workload of the suite. Each round runs every build once per workload, in a rotated order, so a
change of machine load hits all builds alike. Prints four rows per workload
(wall, then CPU time user+sys of the child; median, then minimum) with the
seconds per build and the ratio to the first build. On a loaded machine
the minimum rows are the steadier ones.
Q4.24 compares the safe build, the fast build and the prototypes.
"""
import math, pathlib, resource, statistics, subprocess, sys, tempfile, time

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import jsbench


def main(argv):
    runs = 5
    builds = []
    names = []
    args = list(argv)
    while args:
        a = args.pop(0)
        if a == "--runs":
            runs = int(args.pop(0))
        elif a == "--":
            names = args
            break
        else:
            n, b = a.split("=", 1)
            builds.append((n, b))
    if not names:
        names = [p.stem for p in sorted(jsbench.DIR.glob("*.js"))]
    tmp = tempfile.mkdtemp(prefix="jsmodes-")
    ratios = [[] for _ in builds[1:]]
    print("workload\t" + "\t".join(n for n, _ in builds) + "\t" + "\t".join(f"{n}/{builds[0][0]}" for n, _ in builds[1:]))
    for name in names:
        path = jsbench.DIR / (name + ".js")
        golden = path.with_suffix(".out").read_bytes()
        script = jsbench.prepare(path, tmp)
        dom = jsbench.is_dom(path)
        times = {n: [] for n, _ in builds}
        cpu = {n: [] for n, _ in builds}
        for i in range(runs):
            order = builds[i % len(builds):] + builds[:i % len(builds)]
            for n, b in order:
                before = resource.getrusage(resource.RUSAGE_CHILDREN)
                times[n].append(jsbench.run_once(jsbench.command([b], script, dom), golden, f"{name} {n}", dom=dom))
                after = resource.getrusage(resource.RUSAGE_CHILDREN)
                cpu[n].append(after.ru_utime + after.ru_stime - before.ru_utime - before.ru_stime)
        for label, data, stat in ((name + " wall median", times, statistics.median), (name + " wall min", times, min),
                                  (name + " cpu median", cpu, statistics.median), (name + " cpu min", cpu, min)):
            med = [stat(data[n]) for n, _ in builds]
            print(label + "\t" + "\t".join(f"{m:.3f}" for m in med) + "\t" + "\t".join(f"{m / med[0]:.3f}" for m in med[1:]), flush=True)
            if stat is min and data is times:
                for r, m in zip(ratios, med[1:]):
                    r.append(m / med[0])
    if len(names) > 1:
        print("geomean of the wall-min ratios\t" + "\t".join(f"{n}/{builds[0][0]} {math.exp(sum(map(math.log, r)) / len(r)):.3f}" for (n, _), r in zip(builds[1:], ratios)))


main(sys.argv[1:])
