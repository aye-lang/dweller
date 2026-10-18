#!/usr/bin/env python3
"""Khronos WebGL conformance runner (P11.14).

  webgl_conformance.py fetch [--rev SHA]
  webgl_conformance.py run [conformance2|conformance|DIR...] [-j N]
  webgl_conformance.py check

run writes target/webgl-results/webgl2_<dir>.json; `conformance.py table` turns them into rows.
"""
import argparse
import concurrent.futures
import json
import os
import subprocess
import sys
from pathlib import Path

import conformance
from wpt import Server

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
ROOT = REPO / "target" / "webgl"
TESTS = ROOT / "sdk" / "tests"
RESULTS = conformance.WEBGL_RESULTS
URL = "https://github.com/KhronosGroup/WebGL.git"
VERSION = (2, 0, 1)
TIMEOUT = 30
PREFIX = {"conformance2": "WebGL 2", "conformance": "WebGL 1"}


def read_rev():
    return (HERE / "webgl_cts.rev").read_text().split()[0]


def git(*args):
    subprocess.run(["git", "-C", str(ROOT), *args], check=True)


def cmd_fetch(args):
    ROOT.mkdir(parents=True, exist_ok=True)
    if not (ROOT / ".git").exists():
        git("init", "-q")
        git("remote", "add", "origin", URL)
    git("sparse-checkout", "set", "--cone", "sdk/tests")
    git("fetch", "-q", "--depth", "1", "--filter=blob:none", "origin", args.rev or read_rev())
    git("checkout", "-q", "-f", "--detach", "FETCH_HEAD")


def version(text):
    return tuple(int(p) for p in text.split("."))


def wanted(opts):
    lo = opts.get("--min-version")
    hi = opts.get("--max-version")
    return (not lo or version(lo) <= VERSION) and (not hi or version(hi) >= VERSION)


def pages(list_file):
    found, skipped = [], []

    for line in list_file.read_text().splitlines():
        words = line.split("//")[0].split("#")[0].split()
        if not words:
            continue

        opts = dict(zip(words[:-1:2], words[1:-1:2]))
        path = list_file.parent / words[-1]

        if not wanted(opts):
            skipped.append(path)
        elif path.suffix == ".txt":
            sub, s = pages(path)
            found += sub
            skipped += s
        else:
            found.append(path)

    return found, skipped


def group_of(page):
    suite, top = page.relative_to(TESTS).parts[:2]
    return f"{PREFIX[suite]}: {top}"


def run_page(dweller, base, page):
    url = f"{base}/{page.relative_to(ROOT).as_posix()}"
    env = dict(os.environ, DWELLER_GL="swiftshader", DWELLER_WEBGL_HARNESS="1", SDL_VIDEODRIVER="dummy")
    cmd = [str(dweller), "dump", "--text", "--js", "--no-scroll", url]

    try:
        p = subprocess.run(cmd, env=env, capture_output=True, text=True, errors="replace",
                           timeout=TIMEOUT, stdin=subprocess.DEVNULL)
        lines, status = p.stderr.splitlines(), f"exit {p.returncode}"
    except subprocess.TimeoutExpired as e:
        lines, status = (e.stderr or b"").decode(errors="replace").splitlines(), "timeout"

    fails = [line.split("DWELLER_WEBGL fail ", 1)[1] for line in lines if "DWELLER_WEBGL fail " in line]
    done = [line.split("DWELLER_WEBGL done ", 1)[1] for line in lines if "DWELLER_WEBGL done " in line]

    errors = [line[:300] for line in lines if line.startswith("[error]")]
    error = errors[0] if errors else None

    if done:
        passed = int(done[0])
        return {"status": "FAIL" if fails else "PASS", "passed": passed, "total": passed + len(fails),
                "message": fails[0][:300] if fails else None, "error": error}

    return {"status": "TIMEOUT" if status == "timeout" else "CRASH", "passed": 0,
            "total": len(fails) + 1, "message": (fails or [status])[0][:300], "error": error}


def targets(names):
    found, skipped = [], []

    for name in names:
        list_file = TESTS / name / "00_test_list.txt"
        if not list_file.exists():
            sys.exit(f"{list_file} not found (webgl_conformance.py fetch)")

        p, s = pages(list_file)
        found += p
        skipped += s

    return found, [group_of(s) for s in skipped]


def write_group(area, rows, skipped):
    files = [{"path": page.relative_to(TESTS).as_posix(), **r} for page, r in rows]
    passed = sum(f["passed"] for f in files)
    total = sum(f["total"] for f in files)
    out = {"area": area, "rev": read_rev(), "files": files, "skipped": skipped,
           "passed": passed, "total": total, "threshold": 0.0}
    RESULTS.mkdir(parents=True, exist_ok=True)
    (RESULTS / f"{conformance.file_key(area)}.json").write_text(json.dumps(out, indent=1) + "\n")
    good = sum(f["status"] == "PASS" for f in files)
    print(f"{area}: {good}/{len(files)} pages, {passed}/{total} tests", flush=True)


def cmd_run(args):
    dweller = REPO / "target" / "dweller"
    found, skipped = targets(args.dirs)
    srv = Server(ROOT)
    srv.start()

    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.j) as pool:
            results = list(pool.map(lambda page: run_page(dweller, srv.base, page), found))
    finally:
        srv.stop()

    groups = {}
    for page, r in zip(found, results):
        groups.setdefault(group_of(page), []).append((page, r))

    for area, rows in groups.items():
        write_group(area, rows, skipped.count(area))


def cmd_check(args):
    problems = [line for area in conformance.webgl_areas() for line in conformance.check_area(area)]
    print("\n".join(problems) if problems else "webgl conformance: no area fell")
    return 1 if problems else 0


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    f = sub.add_parser("fetch")
    f.add_argument("--rev")
    r = sub.add_parser("run")
    r.add_argument("dirs", nargs="*", default=["conformance2"])
    r.add_argument("-j", type=int, default=4)
    sub.add_parser("check")
    args = ap.parse_args()
    return {"fetch": cmd_fetch, "run": cmd_run, "check": cmd_check}[args.cmd](args) or 0


if __name__ == "__main__":
    sys.exit(main())
