#!/usr/bin/env python3
"""A run of the user's game in Dweller (docs/plans/browser.md P11.24a).
Standard library only.

    game_check.py GAME [--binary B] [--out DIR] [--seconds S] [--real-window]

GAME is a built copy of the game (a directory with dist/, or dist/
itself); it is never part of the repo. The tool serves dist/ on
127.0.0.1 (`.wasm` as application/wasm), then for each HTML entry runs
`dweller dump --text --js` (console messages, uncaught exceptions) and
`dweller screenshot` at 1280x720, and runs index.html in a headless
window at 1280x720 for about S seconds of frames with key events
(DWELLER_CONSOLE, DWELLER_TIMING). Screenshots and logs go to DIR
(target/game). It prints, per page, the console errors grouped by cause
and, for the window, the frame rate. Exit 0 means the tool ran, not that
the game works.

The window uses SDL's dummy video driver; --real-window opens a real one
that does not take the focus (DWELLER_BACKGROUND=1).
"""
import argparse
import functools
import http.server
import os
import re
import subprocess
import sys
import threading
import time

import fast_build
from window_env import window_env

CAUSES = [
    ("worker", re.compile(r"Worker|postMessage|importScripts")),
    ("wasm", re.compile(r"WebAssembly|wasm", re.I)),
    ("webgl", re.compile(r"WebGL|GL_|shader|getContext|THREE\.WebGL|framebuffer|texture", re.I)),
    ("audio", re.compile(r"Audio|MediaStream|decodeAudioData", re.I)),
    ("perf", re.compile(r"ran too long|timed out|timeout", re.I)),
    ("missing-api", re.compile(r"is not defined|is not a constructor|is not a function|not supported|unsupported|Not implemented", re.I)),
    ("network", re.compile(r"404|Failed to fetch|NetworkError|load failed", re.I)),
    ("js-engine", re.compile(r"SyntaxError|TypeError|RangeError|ReferenceError|InternalError")),
]
PROBLEM = re.compile(r"^\[(error|warn)\]|Uncaught|not supported|unsupported", re.I)
RECORD = re.compile(r"DWELLER_TIMING\t([^\t]+)\t(\d+)\t")
KEYS = ["KeyW", "ArrowUp", "KeyA", "KeyD", "Space", "KeyS"]


class Handler(http.server.SimpleHTTPRequestHandler):
    extensions_map = {**http.server.SimpleHTTPRequestHandler.extensions_map, ".wasm": "application/wasm", ".js": "text/javascript", ".mjs": "text/javascript"}
    missing = []

    def log_message(self, fmt, *args):
        pass

    def send_error(self, code, message=None, explain=None):
        if code == 404:
            Handler.missing.append(self.path)
        super().send_error(code, message, explain)


def serve(root):
    srv = http.server.ThreadingHTTPServer(("127.0.0.1", 0), functools.partial(Handler, directory=root))
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv, f"http://127.0.0.1:{srv.server_address[1]}"


def dist_dir(game):
    d = os.path.join(game, "dist")
    root = d if os.path.isdir(d) else game
    if not os.path.isfile(os.path.join(root, "index.html")):
        sys.exit(f"no index.html in {root} (build the copy: pnpm install --frozen-lockfile && pnpm build)")
    return os.path.abspath(root)


def run(cmd, env, timeout, log):
    t0 = time.monotonic()
    try:
        r = subprocess.run(cmd, capture_output=True, env=env, timeout=timeout)
        out, code = r.stderr.decode("utf-8", "replace"), r.returncode
    except subprocess.TimeoutExpired as e:
        out, code = (e.stderr or b"").decode("utf-8", "replace"), "timeout"
    with open(log, "w") as f:
        f.write(out)
    return out, code, time.monotonic() - t0


def problems(err):
    return [l.strip() for l in err.splitlines() if "DWELLER_TIMING" not in l and PROBLEM.search(l)]


def cause(line):
    return next((name for name, rx in CAUSES if rx.search(line)), "other")


def group(lines):
    out = {}
    for l in lines:
        out.setdefault(cause(l), []).append(l)
    return out


def page_check(a, base, name):
    stem = name[:-5]
    env = dict(os.environ)
    err, code, secs = run([a.binary, "dump", "--text", "--js", f"{base}/{name}"], env, 300, os.path.join(a.out, f"{stem}.dump.log"))
    shot = os.path.join(a.out, f"{stem}.png")
    _, scode, _ = run([a.binary, "screenshot", f"{base}/{name}", "-o", shot, "--width", "1280", "--height", "720"], env, 300, os.path.join(a.out, f"{stem}.shot.log"))
    return {"page": name, "exit": code, "secs": round(secs, 1), "problems": problems(err), "shot": shot if os.path.exists(shot) else f"none (exit {scode})"}


def events(frames):
    words = ["painted", "click:640,400"] + ["frame"] * 30
    for k in KEYS:
        words += [f"key:{k}"] + ["frame"] * (frames // (2 * len(KEYS))) + [f"keyup:{k}", "frame"]
    return " ".join(words)


def fps(times):
    if len(times) < 2:
        return 0.0
    gaps = sorted(b - a for a, b in zip(times, times[1:]))
    return round(1 / gaps[len(gaps) // 2], 1) if gaps[len(gaps) // 2] > 0 else 0.0


def window_check(a, base):
    frames = int(a.seconds * 60)
    env = window_env(a.real_window, DWELLER_TIMING="1", DWELLER_CONSOLE="1")
    shot = os.path.join(a.out, "index.window.png")
    cmd = [a.binary, f"{base}/index.html", "--width", "1280", "--height", "720", "--frames", str(frames), "--events", events(frames), "--save", shot]
    log = os.path.join(a.out, "index.window.log")
    draws, readbacks, lines = [], [], []
    t0 = time.monotonic()
    with open(log, "w") as f:
        p = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, env=env)
        killer = threading.Timer(a.seconds * 20 + 60, p.kill)
        killer.start()
        for raw in p.stderr:
            line = raw.decode("utf-8", "replace")
            f.write(line)
            m = RECORD.search(line)
            if not m:
                lines.append(line)
            elif m.group(1) == "ui_draw":
                draws.append(time.monotonic())
            elif m.group(1) == "webgl_readback":
                readbacks.append(time.monotonic())
        code = p.wait()
        killer.cancel()
    return {
        "page": "index.html (window)", "exit": code, "secs": round(time.monotonic() - t0, 1),
        "problems": problems("".join(lines)), "shot": shot if os.path.exists(shot) else "none",
        "frames": len(draws), "window_fps": fps(draws), "webgl_frames": len(readbacks), "webgl_fps": fps(readbacks),
    }


def report(r):
    extra = f", {r['frames']} frames, median {r['window_fps']} fps, {r['webgl_frames']} WebGL frames at {r['webgl_fps']} fps" if "frames" in r else ""
    print(f"== {r['page']}: exit {r['exit']}, {r['secs']} s, {len(r['problems'])} problems{extra}")
    print(f"   screenshot: {r['shot']}")
    for c, ls in sorted(group(r["problems"]).items()):
        print(f"   [{c}] {len(ls)}")
        for l in sorted(set(ls))[:8]:
            print(f"      {l[:240]}")


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("game")
    ap.add_argument("--binary", default="target/dweller")
    ap.add_argument("--allow-debug", action="store_true", help="time a binary that browser/build.sh did not build")
    ap.add_argument("--out", default="target/game")
    ap.add_argument("--seconds", type=float, default=10)
    ap.add_argument("--real-window", action="store_true", help="open a real window, without taking the focus")
    a = ap.parse_args()
    if not os.path.exists(a.binary):
        sys.exit(f"no binary {a.binary} (browser/build.sh)")
    fast_build.require(a.binary, a.allow_debug)
    root = dist_dir(a.game)
    os.makedirs(a.out, exist_ok=True)
    srv, base = serve(root)
    pages = sorted(n for n in os.listdir(root) if n.endswith(".html"))
    results = [page_check(a, base, n) for n in pages] + [window_check(a, base)]
    srv.shutdown()
    print(f"game: {root} ({len(pages)} pages)")
    for r in results:
        report(r)
    print(f"404s: {len(Handler.missing)} {sorted(set(Handler.missing))[:10]}")


if __name__ == "__main__":
    main()
