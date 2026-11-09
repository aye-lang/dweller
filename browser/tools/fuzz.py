#!/usr/bin/env python3
import argparse
import hashlib
import json
import random
import re
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
MINIMIZE_TRIES = 300
PANIC = re.compile(r"^(?:(\S+:\d+:\d+): )?panic: (.*)$", re.M)
STATS = re.compile(r"^fuzz \S+: (\d+) inputs in (\d+) ms", re.M)


def run_binary(binary, target, seconds, corpus, seed):
    cmd = [str(binary), "fuzz", target, "--seconds", str(seconds), "--corpus", str(corpus), "--seed", str(seed)]
    try:
        return subprocess.run(cmd, cwd=ROOT, capture_output=True, text=True, errors="replace", timeout=seconds + 60)
    except subprocess.TimeoutExpired as e:
        return subprocess.CompletedProcess(cmd, "timeout", e.stdout or "", e.stderr or "")


def crash_of(proc):
    if proc.returncode == 0:
        return None
    m = PANIC.search(proc.stderr)
    if m:
        return m.group(1) or m.group(2), m.group(0)
    if proc.returncode == "timeout":
        return "timeout", "no exit within the time limit"
    if proc.returncode < 0:
        name = signal.Signals(-proc.returncode).name
        return name, f"killed by {name}"
    return f"exit {proc.returncode}", proc.stderr.strip()[-500:]


def crash_key_of(binary, target, work, data):
    work.mkdir(parents=True, exist_ok=True)
    (work / "input").write_bytes(data)
    crash = crash_of(run_binary(binary, target, 0, work, 0))
    return crash and crash[0]


def minimize(binary, target, work, data, key):
    tries = 0
    chunks = 2
    while len(data) > 1 and tries < MINIMIZE_TRIES:
        size = max(1, len(data) // chunks)
        for start in range(0, len(data), size):
            candidate = data[:start] + data[start + size:]
            tries += 1
            if crash_key_of(binary, target, work, candidate) == key:
                data = candidate
                chunks = max(chunks - 1, 2)
                break
            if tries >= MINIMIZE_TRIES:
                return data
        else:
            if size == 1:
                return data
            chunks = min(chunks * 2, len(data))
    return data


def run(args):
    args.binary = args.binary or ("target/dweller-fuzz-cov" if args.coverage else "target/dweller-fuzz")
    binary = ROOT / args.binary
    if not binary.exists():
        build = [str(ROOT / "browser/build.sh"), args.binary]
        subprocess.run(build + (["--coverage", "--mode", "debug"] if args.coverage else []), check=True)
    out = ROOT / "target/fuzz" / args.target
    crashes = out / "crashes"
    crashes.mkdir(parents=True, exist_ok=True)
    corpus = ROOT / (args.corpus or f"browser/tests/fuzz/{args.target}")
    end = time.monotonic() + args.minutes * 60
    found = {}
    inputs = ms = 0
    while time.monotonic() < end:
        seconds = max(1, int(end - time.monotonic()))
        proc = run_binary(binary, args.target, seconds, corpus, random.getrandbits(63))
        stats = STATS.search(proc.stdout)
        if stats:
            inputs += int(stats.group(1))
            ms += int(stats.group(2))
        crash = crash_of(proc)
        if not crash:
            continue
        key, message = crash
        if key in found:
            found[key][1] += 1
            continue
        data = (out / "current").read_bytes()
        name = hashlib.sha1(data).hexdigest()[:16]
        (crashes / name).write_bytes(data)
        (crashes / f"{name}.txt").write_text(message + "\n")
        small = data if key == "timeout" else minimize(binary, args.target, out / "minimize", data, key)
        small_name = hashlib.sha1(small).hexdigest()[:16] + ".min"
        (crashes / small_name).write_bytes(small)
        found[key] = [message, 1, small_name, len(data), len(small)]
    rate = inputs * 1000 // max(ms, 1)
    print(f"fuzz {args.target}: {inputs} inputs in completed runs, {rate}/s, {len(found)} unique crashes")
    for key, (message, hits, small_name, size, small_size) in found.items():
        print(f"crash {message} ({hits} hits) {crashes / small_name} ({small_size} bytes, from {size})")
    if args.report:
        report = [{"key": key, "message": message, "hits": hits, "input": str(crashes / small_name)} for key, (message, hits, small_name, _, _) in found.items()]
        Path(args.report).write_text(json.dumps(report, indent=1) + "\n")
    return 1 if found else 0


def main():
    parser = argparse.ArgumentParser(description="Runs `dweller fuzz TARGET` and collects its crashes under target/fuzz/TARGET/crashes.")
    sub = parser.add_subparsers(dest="command", required=True)
    p = sub.add_parser("run")
    p.add_argument("target")
    p.add_argument("--minutes", type=float, default=10)
    p.add_argument("--coverage", action="store_true", help="builds the binary (when missing) with --coverage --mode debug: the corpus grows from new edges")
    p.add_argument("--binary", help="default target/dweller-fuzz, or target/dweller-fuzz-cov with --coverage")
    p.add_argument("--corpus", help="default browser/tests/fuzz/TARGET")
    p.add_argument("--report", help="writes the unique crashes there as JSON")
    return run(parser.parse_args())


if __name__ == "__main__":
    sys.exit(main())
