"""Upstream `wpt serve` from the pinned checkout (P10.5)."""
import json
import os
import shutil
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
START_WAIT = 90


def resolve_setting():
    return ",".join(f"{h}:{p}:127.0.0.1" for h in HOSTS for p in HTTP_PORTS)


def listening(port):
    with socket.socket() as s:
        s.settimeout(0.5)
        return s.connect_ex(("127.0.0.1", port)) == 0


class Upstream:
    """`wpt serve` on web-platform.test:8000 (and :8001), answered by
    127.0.0.1 through DWELLER_RESOLVE; our testharnessreport.js replaces
    the checkout's."""

    def __init__(self, root, report_js):
        self.root = Path(root)
        self.report_js = report_js
        self.base = f"http://{HOSTS[0]}:{HTTP_PORTS[0]}"
        self.proc = None

    def config(self):
        path = self.root / "_dweller_serve.json"
        path.write_text(json.dumps({
            "server_host": "127.0.0.1",
            "check_subdomains": False,
            "ports": {"http": list(HTTP_PORTS)},
        }))
        return path

    def start(self):
        shutil.copyfile(self.report_js, self.root / "resources" / "testharnessreport.js")
        os.environ["DWELLER_RESOLVE"] = resolve_setting()
        log = open(self.root / "_dweller_serve.log", "w")
        self.proc = subprocess.Popen(
            [sys.executable, "wpt", "serve", "--no-h2", "--config", str(self.config())],
            cwd=self.root, stdout=log, stderr=subprocess.STDOUT, stdin=subprocess.DEVNULL)
        deadline = time.time() + START_WAIT
        while not listening(HTTP_PORTS[-1]):
            if self.proc.poll() is not None or time.time() > deadline:
                self.stop()
                sys.exit(f"wpt serve did not start (see {self.root}/_dweller_serve.log)")
            time.sleep(0.5)

    def stop(self):
        if self.proc and self.proc.poll() is None:
            self.proc.terminate()
            self.proc.wait(30)
