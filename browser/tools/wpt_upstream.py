"""Upstream `wpt serve` from the pinned checkout (P10.5)."""
import atexit
import json
import os
import shutil
import signal
import socket
import subprocess
import sys
import time
from pathlib import Path

HOSTS = [
    "web-platform.test",
    "www.web-platform.test",
    "www1.web-platform.test",
    "www2.web-platform.test",
    "xn--n8j6ds53lwwkrqhv28a.web-platform.test",
    "xn--lve-6lad.web-platform.test",
    "not-web-platform.test",
]
HTTP_PORTS = (8000, 8001)
HTTPS_PORTS = (8443, 8444)
START_WAIT = 90


def resolve_setting():
    return ",".join(f"{h}:{p}:127.0.0.1" for h in HOSTS for p in HTTP_PORTS + HTTPS_PORTS)


def listening(port):
    with socket.socket() as s:
        s.settimeout(0.5)
        return s.connect_ex(("127.0.0.1", port)) == 0


def live_members(pgid):
    out = subprocess.run(["ps", "-A", "-o", "pid=,pgid=,stat="], capture_output=True, text=True, check=True).stdout
    rows = (line.split() for line in out.splitlines())
    return [int(r[0]) for r in rows if len(r) == 3 and r[1] == str(pgid) and not r[2].startswith("Z")]


def signal_pid(pid, sig, kill):
    try:
        kill(pid, sig)
    except ProcessLookupError:
        pass


def signal_group(pgid, sig):
    try:
        signal_pid(pgid, sig, os.killpg)
    except PermissionError:
        for pid in live_members(pgid):
            signal_pid(pid, sig, os.kill)


class Upstream:
    """`wpt serve` on web-platform.test:8000 (and :8001, https on :8443
    and :8444 with the checkout's tools/certs), answered by
    127.0.0.1 through DWELLER_RESOLVE; our testharnessreport.js replaces
    the checkout's."""

    def __init__(self, root, report_js):
        self.root = Path(root)
        self.report_js = report_js
        self.base = f"http://{HOSTS[0]}:{HTTP_PORTS[0]}"
        self.https_base = f"https://{HOSTS[0]}:{HTTPS_PORTS[0]}"
        self.proc = None

    def config(self):
        path = self.root / "_dweller_serve.json"
        path.write_text(json.dumps({
            "server_host": "127.0.0.1",
            "check_subdomains": False,
            "ports": {"http": list(HTTP_PORTS), "https": list(HTTPS_PORTS)},
        }))
        return path

    def start(self):
        shutil.copyfile(self.report_js, self.root / "resources" / "testharnessreport.js")
        os.environ["DWELLER_RESOLVE"] = resolve_setting()
        log = open(self.root / "_dweller_serve.log", "w")
        self.proc = subprocess.Popen(
            [sys.executable, "wpt", "serve", "--no-h2", "--config", str(self.config())],
            cwd=self.root, stdout=log, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL,
            start_new_session=True)
        atexit.register(self.stop)
        deadline = time.time() + START_WAIT
        while not (listening(HTTP_PORTS[-1]) and listening(HTTPS_PORTS[-1])):
            if self.proc.poll() is not None or time.time() > deadline:
                self.stop()
                sys.exit(f"wpt serve did not start (see {self.root}/_dweller_serve.log)")
            time.sleep(0.5)

    def stop(self):
        if not self.proc:
            return
        signal_group(self.proc.pid, signal.SIGTERM)
        try:
            self.proc.wait(30)
        except subprocess.TimeoutExpired:
            pass
        signal_group(self.proc.pid, signal.SIGKILL)
        self.proc.wait()
        self.proc = None
