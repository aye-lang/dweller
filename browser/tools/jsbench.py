#!/usr/bin/env python3
"""Runs the standalone JS benchmarks of browser/tests/jsbench/ under `dweller js`.

    python3 browser/tools/jsbench.py [--smoke] [--json FILE] [WORKLOAD...]

Prints the median whole-process wall time of BENCH_RUNS runs (default 3) and
checks every stdout against the committed golden (`NAME.out`), and against
Node when it is installed. With `node` on the PATH (or BENCH_REF=node) a
`node --jitless` column is added (V8 with only the Ignition interpreter).

--smoke       run every workload once at a small size (SMOKE=10) and compare
              the checksum with `NAME.smoke.out`; no timing table.
--json FILE   write the medians and the machine description to FILE.
--update-smoke  regenerate the `NAME.smoke.out` goldens from Node.

Environment: BENCH_BIN (binary, default target/dweller), BENCH_RUNS,
BENCH_BASELINE (an older binary: interleaved, adds speedup columns),
BENCH_REF (`node` by default when found, `none` to disable).
"""
import math, os, pathlib, platform, shutil, statistics, subprocess, sys, tempfile, time

ROOT = pathlib.Path(__file__).resolve().parents[2]
DIR = ROOT / "browser" / "tests" / "jsbench"
SMOKE = 10


def machine():
    cpu = ""
    try:
        for line in open("/proc/cpuinfo"):
            if line.startswith("model name"):
                cpu = line.split(":", 1)[1].strip()
                break
    except OSError:
        pass
    if not cpu:
        cpu = platform.processor() or platform.machine()
    mem = ""
    try:
        for line in open("/proc/meminfo"):
            if line.startswith("MemTotal"):
                mem = f"{int(line.split()[1]) // 1024} MiB"
                break
    except OSError:
        pass
    return {
        "cpu": cpu,
        "cores": os.cpu_count(),
        "memory": mem,
        "os": f"{platform.system()} {platform.release()} {platform.machine()}",
    }


def git(*args):
    try:
        return subprocess.run(["git", *args], cwd=ROOT, capture_output=True, text=True).stdout.strip()
    except OSError:
        return ""


def node_version(node):
    return subprocess.run([node, "--version"], capture_output=True, text=True).stdout.strip()


def with_smoke(path, tmp):
    out = pathlib.Path(tmp) / path.name
    out.write_text(f"var SMOKE = {SMOKE};\n" + path.read_text())
    return out


def run_once(cmd, golden, what, ignore_stderr=False):
    start = time.perf_counter()
    r = subprocess.run(cmd, capture_output=True, timeout=600)
    elapsed = time.perf_counter() - start
    if r.returncode or r.stdout != golden or (r.stderr and not ignore_stderr):
        raise SystemExit(f"{what}: failed (status {r.returncode}): stdout {r.stdout!r} stderr {r.stderr[:300]!r}, expected {golden!r}")
    return elapsed


def main(argv):
    smoke = update = False
    json_out = None
    names = []
    args = list(argv)
    while args:
        a = args.pop(0)
        if a == "--smoke":
            smoke = True
        elif a == "--update-smoke":
            update = True
        elif a == "--json":
            json_out = args.pop(0)
        elif a.startswith("-"):
            raise SystemExit(f"unknown option {a}")
        else:
            names.append(a)
    binary = os.environ.get("BENCH_BIN", str(ROOT / "target" / "dweller"))
    runs = int(os.environ.get("BENCH_RUNS", "3"))
    if runs < 1:
        raise SystemExit("BENCH_RUNS must be at least 1")
    baseline = os.environ.get("BENCH_BASELINE")
    ref = os.environ.get("BENCH_REF", "node")
    node = shutil.which("node") if ref != "none" else None
    paths = sorted(DIR.glob("*.js"))
    if names:
        paths = [p for p in paths if p.stem in names]
    if not paths:
        raise SystemExit("no matching benchmarks")

    tmp = tempfile.mkdtemp(prefix="jsbench-")
    try:
        if update or smoke:
            return smoke_mode(paths, binary, node, tmp, update)
        return timing_mode(paths, binary, baseline, node, runs, json_out)
    finally:
        shutil.rmtree(tmp, ignore_errors=True)


def smoke_mode(paths, binary, node, tmp, update):
    failed = []
    for path in paths:
        script = with_smoke(path, tmp)
        golden_path = path.with_suffix(".smoke.out")
        if update:
            r = subprocess.run(["node", str(script)], capture_output=True, check=True)
            golden_path.write_bytes(r.stdout)
            print(f"{path.stem}\t{r.stdout.decode().strip()}")
            continue
        golden = golden_path.read_bytes()
        try:
            t = run_once([binary, "js", str(script)], golden, f"{path.stem} (dweller)")
            if node:
                run_once([node, str(script)], golden, f"{path.stem} (node)")
        except SystemExit as e:
            failed.append(str(e))
            continue
        print(f"{path.stem}\tok\t{golden.decode().strip()}\t{t:.2f}s", flush=True)
    if failed:
        raise SystemExit("\n".join(failed))
    return 0


def timing_mode(paths, binary, baseline, node, runs, json_out):
    cols = ["benchmark", "dweller_s"]
    if baseline:
        cols = ["benchmark", "baseline_s", "dweller_s", "speedup"]
    if node:
        cols.append("node_jitless_s")
        cols.append("ratio")
    cols.append("checksum")
    print("\t".join(cols), flush=True)
    results = {}
    speedups = []
    ratios = []
    for path in paths:
        golden = path.with_suffix(".out").read_bytes()
        if node:
            run_once([node, str(path)], golden, f"{path.stem} (node)")
        exes = {"dweller": [binary]}
        if baseline:
            exes["baseline"] = [baseline]
        times = {k: [] for k in exes}
        if node:
            exes["node"] = [node, "--jitless"]
            times["node"] = []
        for i in range(runs):
            order = list(exes)
            if i % 2:
                order.reverse()
            for k in order:
                cmd = exes[k] + (["js"] if k != "node" else []) + [str(path)]
                times[k].append(run_once(cmd, golden, f"{path.stem} ({k})", ignore_stderr=(k == "node")))
        med = {k: statistics.median(v) for k, v in times.items()}
        row = [path.stem]
        if baseline:
            row.append(f"{med['baseline']:.3f}")
        row.append(f"{med['dweller']:.3f}")
        if baseline:
            sp = med["baseline"] / med["dweller"]
            speedups.append(sp)
            row.append(f"{sp:.2f}")
        if node:
            row.append(f"{med['node']:.3f}")
            ratio = med["dweller"] / med["node"]
            ratios.append(ratio)
            row.append(f"{ratio:.1f}x")
        row.append(golden.decode().strip())
        print("\t".join(row), flush=True)
        results[path.stem] = {"checksum": golden.decode().strip(), **{k: round(v, 4) for k, v in med.items()}}
    if speedups:
        print(f"geometric_mean_speedup\t{math.exp(sum(map(math.log, speedups)) / len(speedups)):.3f}")
    if ratios:
        print(f"geometric_mean_ratio_to_node_jitless\t{math.exp(sum(map(math.log, ratios)) / len(ratios)):.2f}x")
    if json_out:
        import json
        doc = {
            "machine": machine(),
            "build": "fast (browser/build.sh --mode fast)",
            "commit": git("rev-parse", "--short", "HEAD"),
            "date": time.strftime("%Y-%m-%d"),
            "runs": runs,
            "statistic": "median wall seconds, whole process",
            "node": node_version(node) + " --jitless" if node else None,
            "benchmarks": {k: {("node_jitless" if kk == "node" else kk): vv for kk, vv in v.items()} for k, v in results.items()},
        }
        pathlib.Path(json_out).parent.mkdir(parents=True, exist_ok=True)
        pathlib.Path(json_out).write_text(json.dumps(doc, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
