#!/usr/bin/env python3
"""The 1 h soak (Q1.11, scoreboard S4). Standard library only.

    soak.py [--minutes 60] [--binary target/dweller-soak] [--archives target/sites]
            [--site NAME] [--dwell 40] [--out DIR]

Build the binary first: `aye build browser --mode safe --leak-check -o
target/dweller-soak` (the shipped safe mode, with the allocator's leak
report at exit).

One headless window (SDL's dummy driver, a fresh DWELLER_PROFILE) in the
default process mode replays the archives of every judged site of
sites.json at once (DWELLER_REPLAY on a directory of links to all their
responses). It runs as `--agent-socket S --exit-when-idle --events
SCRIPT`: this tool connects with the token and sends nothing, so the
window has no frame cap, starts no Space, and quits when the tool hangs
up.

A cycle opens each site in a new tab (Cmd+T, the URL typed, Return),
scrolls down and up three times, follows the page's first link by
keyboard (Tab, Return), and closes the three newest tabs when four are
open; at its first site it goes back and forward once. It ends with all
tabs but the first closed and a marker tab (a data: page) that waits
--dwell seconds, then logs `soak-cycle-N`. A URL with `_` (which `text:`
types as a space) goes through a data: page that replaces itself with it.

Every 10 s and at each marker the RSS of the window and of every process
under it goes to DIR/soak.csv; stderr goes to DIR/soak.log, each line
after the seconds since the start. After the
first marker past --minutes the tool hangs up and reads the leak report.

Budget: from the end of cycle 1, the total RSS at the end of each later
cycle grows at most 10% and at most 100 MB; no leak; no crash (a crash
report of the binary, a panic, a bad message, a stuck page, a sad tab or
the window ending early); no hang (no marker for --dwell + 300 s).
Exits 1 when the verdict is FAIL.
"""
import argparse
import atexit
import base64
import csv
import glob
import os
import queue
import re
import shutil
import socket
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

from window_env import window_env
from window_report import archive_url, sites

SAMPLE_S = 10
HANG_S = 300
GROWTH_PCT = 10
GROWTH_MB = 100
TEXT_MAX = 31
SCROLL = ["wheel:-10", "renderer:input"] * 3 + ["wheel:10", "renderer:input"] * 3
MARK = re.compile(r"soak-cycle-(\d+)")
LEAK = re.compile(r"^(\S+): leak: (\d+) bytes")
TROUBLE = re.compile(r"panic|bad message|stuck:|crash|Segmentation", re.I)
MISS = re.compile(r"archive miss: (\S+)")


def typed(url):
    if "_" in url:
        url = f'data:text/html,<script>location.replace(atob("{base64.b64encode(url.encode()).decode()}"))</script>'

    return [f"text:{url[i:i + TEXT_MAX]}" for i in range(0, len(url), TEXT_MAX)]


def cycle(n, urls, dwell):
    words, tabs = [], 1
    for i, url in enumerate(urls):
        words += ["key:cmd+t", *typed(url), "key:return", "renderer:loaded", *SCROLL, "key:tab", "key:return", "renderer:loaded"]
        if i == 0:
            words += ["key:cmd+[", "renderer:loaded", "key:cmd+]", "renderer:loaded"]
        tabs += 1
        if tabs == 4 or i == len(urls) - 1:
            words += ["key:cmd+w"] * (tabs - 1)
            tabs = 1
    page = f'data:text/html,<script>setTimeout(()=>{{console.log("soak-cycle-{n}");document.title="z"}},{dwell * 1000})</script>'

    return words + ["key:cmd+t", *typed(page), "key:return", "renderer:title:z", "key:cmd+w"]


def merged_archive(a, names):
    path = tempfile.mkdtemp(prefix="dweller-soak-")
    atexit.register(shutil.rmtree, path, True)
    for name in names:
        for f in (Path(a.archives) / name).glob("*.response"):
            target = Path(path) / f.name
            if not target.exists():
                target.symlink_to(f.resolve())

    return path


def tree_rss(root):
    """RSS in KB of `root` and of each process under it."""
    rows = [l.split() for l in subprocess.run(["ps", "-A", "-o", "pid=,ppid=,rss="], capture_output=True, text=True).stdout.splitlines()]
    rss = {r[0]: int(r[2]) for r in rows if len(r) == 3}
    own, grew = {str(root)}, True
    while grew:
        more = {r[0] for r in rows if len(r) == 3 and r[1] in own} - own
        grew = bool(more)
        own |= more

    return rss.get(str(root), 0), {p: rss[p] for p in own - {str(root)} if p in rss}


def outlived(binary, kids):
    """Kills the processes of the last sample that are still the binary's."""
    rows = [l.split(None, 1) for l in subprocess.run(["ps", "-A", "-o", "pid=,command="], capture_output=True, text=True).stdout.splitlines()]
    left = [r[0] for r in rows if len(r) == 2 and r[0] in kids and r[1].startswith(binary)]
    for pid in left:
        os.kill(int(pid), 9)

    return [f"process {pid} outlived the window" for pid in left]


def read_stderr(stream, log, t0, marks, seen):
    """Each line goes to the log after the seconds since the start."""
    for raw in stream:
        t = time.monotonic() - t0
        log.write(b"%7.1f " % t + raw)
        log.flush()
        line = raw.decode("utf-8", "replace").rstrip("\n")
        if m := MARK.search(line):
            marks.put((int(m.group(1)), time.monotonic()))
        elif m := LEAK.match(line):
            seen["exit_leaks" if seen["hung_up"] else "leaks"].append((m.group(1), int(m.group(2)), round(t)))
        elif m := MISS.search(line):
            seen["misses"].append(m.group(1))
        elif TROUBLE.search(line) and not line.startswith("["):
            seen["trouble"].append(line[:200])


def connect(sock_path, p):
    while not os.path.exists(sock_path + ".token"):
        if p.poll() is not None:
            sys.exit(f"the window ended at its start ({p.returncode})")
        time.sleep(0.1)
    conn = socket.socket(socket.AF_UNIX)
    conn.connect(sock_path)
    conn.sendall(('{"token":"%s"}\n' % Path(sock_path + ".token").read_text().strip()).encode())

    return conn


def soak(a):
    names = sites(a)
    urls = [archive_url(a, n) for n in names]
    cycles = a.minutes * 60 // a.dwell + 2
    script = " ".join(w for n in range(1, cycles + 1) for w in cycle(n, urls, a.dwell))
    if len(script) > 900_000:
        sys.exit(f"the script of {cycles} cycles is {len(script)} bytes, over ARG_MAX: raise --dwell")
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    sock_dir = tempfile.mkdtemp(prefix="dweller-soak-sock-")
    atexit.register(shutil.rmtree, sock_dir, True)
    sock_path = os.path.join(sock_dir, "a.sock")
    env = window_env(False, DWELLER_REPLAY=merged_archive(a, names), DWELLER_CONSOLE="1")
    env.pop("DWELLER_PROCESS", None)
    cmd = [a.binary, "--agent-socket", sock_path, "--exit-when-idle", "--events", script, "--state", "--processes", urls[0]]
    log = open(out / "soak.log", "wb")
    reports = set(glob.glob(os.path.expanduser(f"~/Library/Logs/DiagnosticReports/{Path(a.binary).name}*")))
    t0 = time.monotonic()
    p = subprocess.Popen(cmd, stdout=open(out / "soak.state", "wb"), stderr=subprocess.PIPE, env=env)
    marks, seen = queue.Queue(), {"leaks": [], "exit_leaks": [], "misses": [], "trouble": [], "hung_up": False}
    reader = threading.Thread(target=read_stderr, args=(p.stderr, log, t0, marks, seen))
    reader.start()
    conn = connect(sock_path, p)
    print(f"soak: {len(names)} sites, {a.minutes} min, dwell {a.dwell} s, window pid {p.pid}, script {len(script)} bytes", flush=True)
    rows, ends, last_mark, hang, kids = [], [], t0, None, {}
    sample = csv.writer(open(out / "soak.csv", "w", newline="", buffering=1))
    sample.writerow(["t_s", "cycle", "kind", "window_kb", "renderers_kb", "total_kb", "renderers"])

    def take(kind):
        window, now = tree_rss(p.pid)
        kids.clear()
        kids.update(now)
        row = [round(time.monotonic() - t0), len(ends), kind, window, sum(now.values()), window + sum(now.values()), len(now)]
        sample.writerow(row)
        rows.append(row)

        return row

    next_sample = t0
    while p.poll() is None:
        try:
            n, at = marks.get(timeout=max(0.1, next_sample - time.monotonic()))
            ends.append(n)
            last_mark = at
            row = take("end")
            print(f"cycle {n} at {row[0] / 60:.1f} min: {row[5] / 1024:.0f} MB, {row[6]} processes", flush=True)
            if at - t0 >= a.minutes * 60:
                break
        except queue.Empty:
            pass
        if time.monotonic() >= next_sample:
            take("sample")
            next_sample += SAMPLE_S
        if time.monotonic() - last_mark > a.dwell + HANG_S:
            hang = f"no marker for {a.dwell + HANG_S} s after cycle {len(ends)}"
            break
    early = p.poll()
    seen["hung_up"] = True
    conn.close()
    try:
        p.wait(120)
    except subprocess.TimeoutExpired:
        hang = hang or "the window did not quit within 120 s of the hang-up"
        p.kill()
        p.wait()
    reader.join()
    log.close()
    seen["trouble"] += outlived(a.binary, kids)
    crashes = sorted(set(glob.glob(os.path.expanduser(f"~/Library/Logs/DiagnosticReports/{Path(a.binary).name}*"))) - reports)
    if early is not None:
        crashes.append(f"the window ended early with status {early}")
    elif p.returncode != 0:
        crashes.append(f"the window exited with status {p.returncode}")
    crashes += [l for l in (out / "soak.state").read_text().splitlines() if "sad-tab" in l or " gone " in l]

    return summary(rows, ends, seen, crashes, hang, set(urls))


def summary(rows, ends, seen, crashes, hang, urls):
    end_rows = [r for r in rows if r[2] == "end"]
    print(f"\n{'cycle':>5} {'min':>6} {'total MB':>9} {'window MB':>10} {'renderers MB':>13} {'procs':>6}")
    for n, r in zip(ends, end_rows):
        print(f"{n:>5} {r[0] / 60:>6.1f} {r[5] / 1024:>9.1f} {r[3] / 1024:>10.1f} {r[4] / 1024:>13.1f} {r[6]:>6}")
    fails = []
    if len(end_rows) < 2:
        fails.append(f"only {len(end_rows)} cycles")
    else:
        base = end_rows[0][5]
        worst = max(end_rows[1:], key=lambda r: r[5])
        grow = (worst[5] - base) / 1024
        limit = min(base / 1024 * GROWTH_PCT / 100, GROWTH_MB)
        print(f"\ngrowth from cycle 1 ({base / 1024:.0f} MB): {grow:+.1f} MB ({100 * grow * 1024 / base:+.1f}%) at most, "
              f"{(end_rows[-1][5] - base) / 1024:+.1f} MB at the last; budget {limit:.0f} MB")
        if grow > limit:
            fails.append(f"growth {grow:.0f} MB over {limit:.0f} MB")
    peak = max(rows, key=lambda r: r[5], default=[0] * 7)
    print(f"peak: {peak[5] / 1024:.0f} MB at {peak[0] / 60:.1f} min; processes under the window: at most {max((r[6] for r in rows), default=0)}")
    for what, leaks in (("at the exit", seen["exit_leaks"]), ("during the run (renderers that ended)", seen["leaks"])):
        sites_of = sorted({s for s, _, _ in leaks})
        print(f"leak reports, {what}: {len(leaks)} leaks, {sum(b for _, b, _ in leaks)} bytes" + "".join(f"\n  {s}" for s in sites_of))
    if seen["leaks"] or seen["exit_leaks"]:
        fails.append("leaks")
    missed = [u for u in seen["misses"] if u in urls]
    print(f"archive misses: {len(seen['misses'])} ({len(missed)} of the sites' own pages)")
    for what, items in (("crashes", crashes), ("trouble on stderr", seen["trouble"])):
        print(f"{what}: {len(items)}" + "".join(f"\n  {i}" for i in items[:10]))
        if items:
            fails.append(what)
    if hang:
        print(f"hang: {hang}")
        fails.append("hang")
    print(f"verdict: {'FAIL ' + ', '.join(fails) if fails else 'PASS'}")

    return not fails


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--minutes", type=int, default=60)
    ap.add_argument("--binary", default="target/dweller-soak")
    ap.add_argument("--archives", default="target/sites")
    ap.add_argument("--site", action="append", help="only this site (repeatable)")
    ap.add_argument("--dwell", type=int, default=40, help="seconds the marker tab waits at the end of a cycle")
    ap.add_argument("--out", default="target/soak", help="the directory of soak.csv, soak.log and soak.state")
    a = ap.parse_args()
    if not os.path.exists(a.binary):
        sys.exit(f"no binary {a.binary} (aye build browser --mode safe --leak-check -o {a.binary})")
    sys.exit(0 if soak(a) else 1)


if __name__ == "__main__":
    main()
