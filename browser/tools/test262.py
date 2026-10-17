#!/usr/bin/env python3
"""test262 over `dweller js --test262` (docs/plans/browser.md, P10.3).

    test262.py fetch [--root DIR]
    test262.py run [PATH...] [-j N] [--root DIR] [--dweller BIN] [--timeout S]
                   [--skip FILE] [--out DIR]

`fetch` checks out the revision in `test262.rev` into `target/test262`.
`run` runs every test under each PATH (a file or a directory, relative to
the test262 root, e.g. `test/built-ins/Array`; default `test`): it reads
the front matter, skips what `test262_skip.txt` names, builds a script of
the harness files plus the test (once sloppy and once with "use strict",
as the flags ask), runs it and judges the exit status and output.
Results go to `target/test262-results/<path with / as ->.json`; the last
lines printed are one score per PATH.

`--root` points at another test262 tree (the fixtures in
`browser/tests/test262` have the same layout). Standard library only.
"""

import argparse
import concurrent.futures
import json
import os
import re
import subprocess
import sys
import tempfile
import time

HERE = os.path.dirname(os.path.abspath(__file__))
REPO = os.path.dirname(os.path.dirname(HERE))
REV_FILE = os.path.join(HERE, "test262.rev")
SKIP_FILE = os.path.join(HERE, "test262_skip.txt")
DEFAULT_ROOT = os.path.join(REPO, "target", "test262")
DEFAULT_OUT = os.path.join(REPO, "target", "test262-results")
REMOTE = "https://github.com/tc39/test262"

# Host features the runner does not provide: a test whose source uses one
# of them is skipped (the front matter's `features` does not always say).
NEEDS = [
    b"$262.createRealm",
    b"$262.agent",
    b"$262.IsHTMLDDA",
    b"$262.AbstractModuleSource",
    b"$262.destroy",
]


def read_rev():
    with open(REV_FILE) as f:
        return f.read().split()[0]


def git(args, cwd):
    return subprocess.run(["git"] + args, cwd=cwd, check=True,
                          stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def fetch(root):
    rev = read_rev()
    os.makedirs(root, exist_ok=True)
    if not os.path.isdir(os.path.join(root, ".git")):
        git(["init", "-q"], root)
        git(["remote", "add", "origin", REMOTE], root)
    head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=root,
                          stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    if head.returncode == 0 and head.stdout.decode().strip() == rev \
            and os.path.isdir(os.path.join(root, "harness")):
        print("test262 %s already in %s" % (rev[:10], root))
        return 0
    last = None
    for attempt in range(4):
        try:
            git(["fetch", "-q", "--depth", "1", "origin", rev], root)
            git(["checkout", "-q", "-f", rev], root)
            print("test262 %s in %s" % (rev[:10], root))
            return 0
        except subprocess.CalledProcessError as e:
            last = e.stderr.decode(errors="replace")
            time.sleep(2 << attempt)
    print("fetch failed: " + (last or ""), file=sys.stderr)
    return 1


# ---- front matter

FRONT = re.compile(rb"/\*---(.*?)---\*/", re.S)


def parse_front(src):
    """The front matter as a dict: includes/flags/features lists, negative
    {phase, type}. Only the subset of YAML test262 uses."""
    m = FRONT.search(src)
    meta = {"includes": [], "flags": [], "features": [], "negative": None}
    if not m:
        return meta
    text = m.group(1).decode("utf-8", "replace")
    key = None
    neg = None
    for line in text.split("\n"):
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = len(line) - len(line.lstrip())
        s = line.strip()
        if indent == 0:
            km = re.match(r"([A-Za-z_]+):\s*(.*)$", s)
            if not km:
                continue
            key, val = km.group(1), km.group(2)
            if key in ("includes", "flags", "features"):
                if val.startswith("["):
                    meta[key] = [x.strip() for x in val.strip("[]").split(",")
                                 if x.strip()]
                elif val:
                    meta[key] = [val]
            elif key == "negative":
                neg = {}
                meta["negative"] = neg
        elif key in ("includes", "flags", "features") and s.startswith("- "):
            meta[key].append(s[2:].strip())
        elif key == "negative" and neg is not None:
            km = re.match(r"(phase|type):\s*(\S+)", s)
            if km:
                neg[km.group(1)] = km.group(2)
    return meta


def load_skips(path):
    features, prefixes = set(), []
    if os.path.exists(path):
        with open(path) as f:
            for line in f:
                line = line.split("#", 1)[0].strip()
                if not line:
                    continue
                if "/" in line:
                    prefixes.append(line)
                else:
                    features.add(line)
    return features, prefixes


# ---- running one test

class Runner:
    def __init__(self, root, dweller, timeout, features, prefixes):
        self.root = root
        self.dweller = dweller
        self.timeout = timeout
        self.features = features
        self.prefixes = prefixes
        self.env = dict(os.environ)
        self.harness = {}

    def harness_file(self, name):
        if name not in self.harness:
            with open(os.path.join(self.root, "harness", name), "rb") as f:
                self.harness[name] = f.read()
        return self.harness[name]

    def judge(self, rel):
        """('pass' | 'fail' | 'skip', reason)."""
        for p in self.prefixes:
            if rel.startswith(p):
                return "skip", "path"
        with open(os.path.join(self.root, rel), "rb") as f:
            src = f.read()
        meta = parse_front(src)
        flags = meta["flags"]
        for ft in meta["features"]:
            if ft in self.features:
                return "skip", "feature " + ft
        for n in NEEDS:
            if n in src:
                return "skip", "host " + n.decode()
        modes = []
        raw = "raw" in flags
        module = "module" in flags
        if module or raw:
            modes = [False]
        elif "onlyStrict" in flags:
            modes = [True]
        elif "noStrict" in flags:
            modes = [False]
        else:
            modes = [False, True]
        reason = ""
        for strict in modes:
            ok, why = self.run_one(rel, src, meta, strict, module, raw)
            if not ok:
                reason = ("strict: " if strict else "") + why
                return "fail", reason
        return "pass", ""

    def build(self, src, meta, strict, raw):
        parts = []
        if strict:
            parts.append(b'"use strict";\n')
        if not raw:
            names = ["assert.js", "sta.js"]
            if "async" in meta["flags"]:
                names.append("doneprintHandle.js")
            names += meta["includes"]
            seen = set()
            for n in names:
                if n not in seen:
                    seen.add(n)
                    parts.append(self.harness_file(n))
                    parts.append(b"\n")
        parts.append(src)
        return b"".join(parts)

    def run_one(self, rel, src, meta, strict, module, raw):
        code = self.build(src, meta, strict, raw)
        suffix = ".mjs" if module else ".js"
        # A module's imports are relative to the test: its file goes next
        # to it.
        d = os.path.dirname(os.path.join(self.root, rel)) if module else None
        fd, tmp = tempfile.mkstemp(suffix=suffix, prefix=".t262-", dir=d)
        try:
            with os.fdopen(fd, "wb") as f:
                f.write(code)
            try:
                p = subprocess.run([self.dweller, "js", "--test262", tmp],
                                   stdout=subprocess.PIPE,
                                   stderr=subprocess.PIPE,
                                   timeout=self.timeout, env=self.env)
            except subprocess.TimeoutExpired:
                return False, "timeout"
        finally:
            try:
                os.unlink(tmp)
            except OSError:
                pass
        out = p.stdout.decode("utf-8", "replace")
        err = p.stderr.decode("utf-8", "replace")
        lines = [l.strip() for l in err.strip().split("\n") if l.strip()]
        first = lines[0] if lines else ""
        detail = " ".join(lines[:3])
        neg = meta["negative"]
        if neg:
            want = neg.get("type", "")
            if p.returncode != 1:
                return False, "expected %s, ran to exit %d" % (want, p.returncode)
            if not first.startswith("Uncaught"):
                return False, "no uncaught error: " + first[:120]
            if want not in first:
                return False, "expected %s, got %s" % (want, first[:120])
            if neg.get("phase") in ("parse", "early") and out.strip():
                return False, "ran before the %s error" % neg.get("phase")
            return True, ""
        if p.returncode < 0:
            return False, "signal %d" % -p.returncode
        if p.returncode != 0:
            return False, detail[:200] or ("exit %d" % p.returncode)
        if "async" in meta["flags"]:
            if "Test262:AsyncTestFailure" in out:
                line = [l for l in out.split("\n") if "AsyncTestFailure" in l][0]
                return False, line[:160]
            if "Test262:AsyncTestComplete" not in out:
                return False, "async test did not complete"
        return True, ""


def collect(root, args):
    found = []
    for a in args:
        full = a if os.path.isabs(a) else os.path.join(root, a)
        if os.path.isfile(full):
            found.append((a, [os.path.relpath(full, root)]))
            continue
        files = []
        for dp, dns, fns in os.walk(full):
            dns.sort()
            for fn in sorted(fns):
                if fn.endswith(".js") and not fn.endswith("_FIXTURE.js") \
                        and not fn.startswith(".t262-"):
                    files.append(os.path.relpath(os.path.join(dp, fn), root))
        found.append((a, files))
    return found


def run(args):
    root = os.path.abspath(args.root)
    if not os.path.isdir(os.path.join(root, "harness")):
        print("no test262 tree at %s (run `test262.py fetch`)" % root,
              file=sys.stderr)
        return 2
    dweller = args.dweller or os.environ.get("DWELLER") \
        or os.path.join(REPO, "target", "dweller")
    if not os.path.exists(dweller):
        subprocess.run(["sh", os.path.join(REPO, "browser", "build.sh"),
                        dweller], check=True)
    features, prefixes = load_skips(args.skip)
    runner = Runner(root, os.path.abspath(dweller), args.timeout, features,
                    prefixes)
    paths = args.paths or ["test"]
    os.makedirs(args.out, exist_ok=True)
    summary = []
    total_pass = total_run = 0
    for label, files in collect(root, paths):
        results = {}

        def one(rel):
            try:
                return rel, runner.judge(rel)
            except Exception as e:  # a runner bug is a failed test, not a crash
                return rel, ("fail", "runner: %s: %s" % (type(e).__name__, e))

        with concurrent.futures.ThreadPoolExecutor(max_workers=args.jobs) as ex:
            for rel, (status, why) in ex.map(one, files):
                results[rel] = [status, why]
        passed = sum(1 for v in results.values() if v[0] == "pass")
        failed = sum(1 for v in results.values() if v[0] == "fail")
        skipped = sum(1 for v in results.values() if v[0] == "skip")
        ran = passed + failed
        name = label.strip("/").replace("/", "-").replace(".js", "") or "all"
        with open(os.path.join(args.out, name + ".json"), "w") as f:
            json.dump({"rev": read_rev() if root == os.path.abspath(DEFAULT_ROOT) else "",
                       "path": label, "total": len(results), "passed": passed,
                       "failed": failed, "skipped": skipped,
                       "tests": dict(sorted(results.items()))}, f, indent=1)
        rate = 100.0 * passed / ran if ran else 0.0
        summary.append("test262 %s: %d/%d passed (%.1f%%), %d skipped"
                       % (label, passed, ran, rate, skipped))
        total_pass += passed
        total_run += ran
    for line in summary:
        print(line)
    if len(summary) > 1:
        print("test262 total: %d/%d passed" % (total_pass, total_run))
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    f = sub.add_parser("fetch")
    f.add_argument("--root", default=DEFAULT_ROOT)
    r = sub.add_parser("run")
    r.add_argument("paths", nargs="*")
    r.add_argument("-j", "--jobs", type=int, default=os.cpu_count() or 4)
    r.add_argument("--root", default=DEFAULT_ROOT)
    r.add_argument("--dweller")
    r.add_argument("--timeout", type=float, default=10.0)
    r.add_argument("--skip", default=SKIP_FILE)
    r.add_argument("--out", default=DEFAULT_OUT)
    args = ap.parse_args()
    if args.cmd == "fetch":
        return fetch(args.root)
    return run(args)


if __name__ == "__main__":
    sys.exit(main())
