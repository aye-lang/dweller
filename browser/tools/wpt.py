#!/usr/bin/env python3
"""WPT testharness runner for Dweller (P10.1b). Standard library only.

  wpt.py fetch [--rev SHA] [--wpt-root DIR]
      Sparse checkout of web-platform-tests at browser/tools/wpt.rev into
      target/wpt: the directories of browser/tools/wpt_areas.json plus
      resources/, common/, fonts/ and css/support/.
  wpt.py serve [--port N] [--wpt-root DIR]
      A static server on 127.0.0.1 for the checkout (see Server below).
  wpt.py run AREA|PATH... [-j N] [--wpt-root DIR] [--results-dir DIR]
      Runs `dweller wpt URL` for each testharness test of an area (a key
      of wpt_areas.json) or of a path (a directory or file under the
      root), and writes target/wpt-results/<area>.json. Reftests and
      crashtests (wpt_reftest.py) are screenshot comparisons and loads.
"""
import argparse
import concurrent.futures
import fnmatch
import json
import mimetypes
import os
import re
import subprocess
import sys
import threading
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from urllib.parse import unquote, urlsplit

import wpt_reftest

HERE = Path(__file__).resolve().parent
REPO = HERE.parent.parent
AREAS_FILE = HERE / "wpt_areas.json"
REPORT_JS = HERE / "wpt" / "testharnessreport.js"
WPT_URL = "https://github.com/web-platform-tests/wpt.git"
TIMEOUT_NORMAL = 10
TIMEOUT_LONG = 60
EXTRA_DIRS = ["resources", "common", "fonts", "css/support"]
# Directories that hold test support files, not tests.
SKIP_DIRS = {"resources", "support", "tools", "reference", "references"}
# Tests that need servers or globals Dweller does not have.
SKIP_NAME = re.compile(
    r"\.(https|serviceworker|sharedworker|worker|window-module|any\.worker)\.(html|js)$"
    r"|-manual\.|-ref\.|-notref\.|\.sub\.(html|js)$"
)
HTML_EXT = (".html", ".htm", ".xhtml", ".xht")
DEFAULT_GLOBALS = {"window", "dedicatedworker"}

mimetypes.add_type("text/javascript", ".js")
mimetypes.add_type("text/javascript", ".mjs")
mimetypes.add_type("application/xhtml+xml", ".xht")
mimetypes.add_type("application/xhtml+xml", ".xhtml")
mimetypes.add_type("text/html", ".htm")
mimetypes.add_type("font/ttf", ".ttf")


def load_areas():
    with open(AREAS_FILE) as f:
        return json.load(f)["areas"]


# ---------------------------------------------------------------- fetch

def git(root, *args):
    subprocess.run(["git", "-C", str(root), *args], check=True)


def read_rev():
    for line in (HERE / "wpt.rev").read_text().splitlines():
        line = line.split("#")[0].strip()
        if line:
            return line
    sys.exit("wpt.rev is empty")


def cmd_fetch(args):
    root = Path(args.wpt_root)
    rev = args.rev or read_rev()
    dirs = set(EXTRA_DIRS)
    for area in load_areas().values():
        dirs.update(area["dirs"])
    root.mkdir(parents=True, exist_ok=True)
    if not (root / ".git").exists():
        git(root, "init", "-q")
        git(root, "remote", "add", "origin", WPT_URL)
    git(root, "sparse-checkout", "set", "--cone", *sorted(dirs))
    git(root, "fetch", "-q", "--depth", "1", "--filter=blob:none", "origin", rev)
    git(root, "checkout", "-q", "--detach", "FETCH_HEAD")
    print(f"wpt {rev} in {root}: {', '.join(sorted(dirs))}")


# ---------------------------------------------------------------- tests

def parse_meta(text):
    """The `// META: name=value` lines at the top of an .js test."""
    meta = {"script": [], "global": [], "timeout": None}
    for line in text.splitlines():
        m = re.match(r"\s*//\s*META:\s*(\w+)=(.*)$", line)
        if not m:
            if line.strip() and not line.strip().startswith("//"):
                break
            continue
        key, value = m.group(1), m.group(2).strip()
        if key == "script":
            meta["script"].append(value)
        elif key == "global":
            meta["global"] += [g.strip() for g in value.split(",") if g.strip()]
        elif key == "timeout":
            meta["timeout"] = value
    return meta


def globals_of(meta):
    positive = [g for g in meta["global"] if not g.startswith("!")]
    negative = {g[1:] for g in meta["global"] if g.startswith("!")}
    if positive:
        base = {g for g in positive if g != "default"}
        if "default" in positive:
            base |= DEFAULT_GLOBALS
    else:
        base = set(DEFAULT_GLOBALS)
    return base - negative


def wrapper_html(js_name, meta, kind):
    """The page for X.any.js (kind "any") or X.window.js (kind "window")."""
    scripts = "".join(
        f'<script src="{s}"></script>\n' for s in meta["script"]
    )
    timeout = '<meta name="timeout" content="long">\n' if meta["timeout"] == "long" else ""
    return (
        "<!doctype html>\n<meta charset=utf-8>\n"
        f"{timeout}<title>{js_name}</title>\n"
        "<script>self.GLOBAL = {isWindow: function() { return true; },"
        " isWorker: function() { return false; },"
        " isShadowRealm: function() { return false; }};</script>\n"
        '<script src="/resources/testharness.js"></script>\n'
        '<script src="/resources/testharnessreport.js"></script>\n'
        f"<div id=log></div>\n{scripts}"
        f'<script src="{js_name}"></script>\n'
    )


def read_text(path):
    return path.read_text(encoding="utf-8", errors="replace")


def wrapper_source(path):
    """For a request path X.any.html / X.window.html whose source X.any.js /
    X.window.js exists: (js file name, kind), else None."""
    for kind in ("any", "window"):
        suffix = f".{kind}.html"
        if path.name.endswith(suffix):
            js = path.with_name(path.name[: -len(suffix)] + f".{kind}.js")
            if js.is_file():
                return js, kind
    return None


def html_kind(rel, text):
    """"crashtest", "reftest", "testharness" or None for an HTML page."""
    if wpt_reftest.is_crashtest(rel):
        return "crashtest"
    if wpt_reftest.has_reftest_link(text):
        return "reftest"
    return "testharness" if "testharness.js" in text else None


def discover(root, rel, excludes):
    """Testharness tests under root/rel: [{"path", "url", "timeout"}]."""
    base = root / rel
    files = [base] if base.is_file() else sorted(
        p for d, dn, fn in os.walk(base) for p in (Path(d) / f for f in fn)
        if not any(part in SKIP_DIRS or part.startswith(".")
                   for part in Path(d).relative_to(base).parts)
    )
    tests, skipped = [], 0
    for p in files:
        relp = p.relative_to(root).as_posix()
        if any(fnmatch.fnmatch(relp, x) or relp.startswith(x.rstrip("/") + "/")
               or relp == x for x in excludes):
            continue
        name = p.name
        if name.endswith((".any.js", ".window.js")):
            kind = "any" if name.endswith(".any.js") else "window"
            if SKIP_NAME.search(name):
                skipped += 1
                continue
            meta = parse_meta(read_text(p))
            if kind == "any" and "window" not in globals_of(meta):
                skipped += 1
                continue
            url = "/" + relp[:-3] + ".html"
            tests.append({"path": relp, "url": url, "kind": "testharness",
                          "timeout": TIMEOUT_LONG if meta["timeout"] == "long" else TIMEOUT_NORMAL})
        elif name.endswith(HTML_EXT):
            if SKIP_NAME.search(name):
                skipped += 1
                continue
            m = re.match(r"(.*\.(?:any|window))\.html$", name)
            if m and p.with_name(m.group(1) + ".js").is_file():
                continue  # the wrapper of an .any.js, listed above
            text = read_text(p)
            kind = html_kind(relp, text)
            if kind is None:
                continue
            long = re.search(r"""<meta\s+name\s*=\s*["']?timeout["']?\s+content\s*=\s*["']?long""", text)
            tests.append({"path": relp, "url": "/" + relp, "kind": kind,
                          "timeout": TIMEOUT_LONG if long else TIMEOUT_NORMAL})
    return tests, skipped


# ---------------------------------------------------------------- server

class Handler(BaseHTTPRequestHandler):
    protocol_version = "HTTP/1.0"
    root = Path(".")

    def log_message(self, *a):
        pass

    def send_body(self, status, ctype, body, extra=()):
        self.send_response(status)
        self.send_header("Content-Type", ctype)
        self.send_header("Content-Length", str(len(body)))
        self.send_header("Cache-Control", "no-store")
        for k, v in extra:
            self.send_header(k, v)
        self.end_headers()
        if self.command != "HEAD":
            self.wfile.write(body)

    def do_HEAD(self):
        self.do_GET()

    def do_GET(self):
        path = unquote(urlsplit(self.path).path)
        parts = [p for p in path.split("/") if p not in ("", ".")]
        if ".." in parts:
            return self.send_body(404, "text/plain", b"not found")
        rel = "/".join(parts)
        # Our report script, wherever a page asks for it.
        if rel.endswith("resources/testharnessreport.js") or rel.endswith("tools/wpt/testharnessreport.js"):
            return self.send_body(200, "text/javascript; charset=utf-8", REPORT_JS.read_bytes())
        file = self.root.joinpath(*parts)
        wrapper = wrapper_source(file) if not path.endswith("/") else None
        if wrapper:
            js, kind = wrapper
            meta = parse_meta(read_text(js))
            html = wrapper_html(js.name, meta, kind)
            return self.send_body(200, "text/html; charset=utf-8", html.encode())
        if file.is_dir():
            index = file / "index.html"
            if not index.is_file():
                return self.send_body(404, "text/plain", b"not found")
            file = index
        if not file.is_file():
            return self.send_body(404, "text/plain", b"not found")
        ctype = mimetypes.guess_type(file.name)[0] or "application/octet-stream"
        if ctype.startswith("text/") or ctype.endswith(("javascript", "json", "xml")):
            ctype += "; charset=utf-8"
        extra = []
        side = file.with_name(file.name + ".headers")
        if side.is_file():
            for line in read_text(side).splitlines():
                if ":" in line and line.strip():
                    k, v = line.split(":", 1)
                    if k.strip().lower() == "content-type":
                        ctype = v.strip()
                    else:
                        extra.append((k.strip(), v.strip()))
        self.send_body(200, ctype, file.read_bytes(), extra)


class Server:
    """Serves root on 127.0.0.1: files (with `NAME.headers` sidecars: one
    `Name: value` per line), generated X.any.html / X.window.html pages for
    X.any.js / X.window.js (with the `// META: script=` lines and a long
    timeout for `timeout=long`), and browser/tools/wpt/testharnessreport.js
    as /resources/testharnessreport.js."""

    def __init__(self, root, port=0):
        handler = type("H", (Handler,), {"root": Path(root)})
        self.httpd = ThreadingHTTPServer(("127.0.0.1", port), handler)
        self.httpd.daemon_threads = True
        self.port = self.httpd.server_address[1]

    def start(self):
        threading.Thread(target=self.httpd.serve_forever, daemon=True).start()

    def stop(self):
        self.httpd.shutdown()
        self.httpd.server_close()


def cmd_serve(args):
    srv = Server(args.wpt_root, args.port)
    print(f"serving {args.wpt_root} on http://127.0.0.1:{srv.port}/", flush=True)
    try:
        srv.httpd.serve_forever()
    except KeyboardInterrupt:
        pass


# ---------------------------------------------------------------- run

def run_test(dweller, root, base_url, test):
    kind = test["kind"]
    if kind == "reftest":
        return wpt_reftest.run_reftest(dweller, root, base_url, test)
    if kind == "crashtest":
        return wpt_reftest.run_crashtest(dweller, root, base_url, test)
    return run_harness_test(dweller, base_url, test)


def run_harness_test(dweller, base_url, test):
    cmd = [str(dweller), "wpt", base_url + test["url"], "--timeout", str(test["timeout"])]
    wall = test["timeout"] * 6 + 30
    try:
        p = subprocess.run(cmd, capture_output=True, text=True, timeout=wall,
                           errors="replace", stdin=subprocess.DEVNULL)
    except subprocess.TimeoutExpired:
        return {"status": "TIMEOUT", "message": f"no result in {wall} s of wall time", "subtests": []}
    for line in reversed(p.stdout.split("\n")):
        if line.startswith("{"):
            try:
                return json.loads(line)
            except ValueError:
                break
    tail = (p.stderr or p.stdout).strip().splitlines()[-1:] or [""]
    return {"status": "CRASH", "message": f"exit {p.returncode}: {tail[0][:200]}", "subtests": []}


def counts(result):
    subs = result["subtests"]
    if subs:
        return sum(1 for s in subs if s["status"] == "PASS"), len(subs)
    return 0, 1  # a file with no subtests (harness error, crash) is one failure


def run_area(name, spec, args, dweller, results_dir):
    root = Path(args.wpt_root)
    tests, skipped = [], 0
    for d in spec["dirs"]:
        if not (root / d).exists():
            sys.exit(f"{d} is not in {root} (run `wpt.py fetch`)")
        t, s = discover(root, d, spec.get("exclude", []))
        tests += t
        skipped += s
    srv = Server(root)
    srv.start()
    base = f"http://127.0.0.1:{srv.port}"
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.j) as pool:
            outs = list(pool.map(lambda t: run_test(dweller, root, base, t), tests))
    finally:
        srv.stop()
    files, passed, total = [], 0, 0
    for t, r in zip(tests, outs):
        p, n = counts(r)
        passed += p
        total += n
        files.append({"path": t["path"], "url": t["url"], "status": r["status"],
                      "message": r.get("message"), "passed": p, "total": n,
                      "subtests": r["subtests"]})
        print(f"  {t['path']}: {p}/{n} {r['status']}")
    score = passed / total if total else 0.0
    threshold = spec.get("threshold", 0.0)
    out = {"area": name, "rev": read_rev(), "files": files, "skipped": skipped,
           "passed": passed, "total": total, "score": score, "threshold": threshold}
    results_dir.mkdir(parents=True, exist_ok=True)
    fname = name.strip("/.").replace("/", "_") or "root"
    (results_dir / f"{fname}.json").write_text(json.dumps(out, indent=1) + "\n")
    print(f"{name}: {len(files)} files ({skipped} skipped), "
          f"{passed}/{total} subtests passed, score {score * 100:.1f}% "
          f"(threshold {threshold * 100:.1f}%)")
    return out


def cmd_run(args):
    dweller = Path(args.dweller) if args.dweller else REPO / "target" / "dweller"
    if not dweller.exists():
        sys.exit(f"{dweller} not found (browser/build.sh)")
    areas = load_areas()
    results_dir = Path(args.results_dir)
    for target in args.targets:
        if target in areas:
            spec = areas[target]
        else:
            rel = target.strip("/") or "."
            spec = {"dirs": [rel], "exclude": [], "threshold": 0.0}
            target = "." if rel == "." else rel
        run_area(target, spec, args, dweller, results_dir)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="cmd", required=True)
    default_root = str(REPO / "target" / "wpt")
    f = sub.add_parser("fetch")
    f.add_argument("--rev")
    f.add_argument("--wpt-root", default=default_root)
    s = sub.add_parser("serve")
    s.add_argument("--port", type=int, default=8000)
    s.add_argument("--wpt-root", default=default_root)
    r = sub.add_parser("run")
    r.add_argument("targets", nargs="+", metavar="AREA|PATH")
    r.add_argument("-j", type=int, default=4)
    r.add_argument("--wpt-root", default=default_root)
    r.add_argument("--results-dir", default=str(REPO / "target" / "wpt-results"))
    r.add_argument("--dweller")
    args = ap.parse_args()
    {"fetch": cmd_fetch, "serve": cmd_serve, "run": cmd_run}[args.cmd](args)


if __name__ == "__main__":
    main()
