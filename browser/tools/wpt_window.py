"""`wpt.py run --process tab|site` (Q2.11): a testharness test in the
window with DWELLER_PROCESS set, so its page runs in renderer processes
(and, with `site`, its cross-site iframes in renderers of their own).
The renderer echoes its console to stderr (DWELLER_CONSOLE=1) and calls
the harness's timeout() after the test's timeout in real time
(DWELLER_WPT_TIMEOUT_MS, with DWELLER_TEST=1); the result is the
"[log] DWELLER_WPT " line, as `dweller wpt` prints it in process."""
import json
import os
import select
import signal
import subprocess
import time

PREFIX = "[log] DWELLER_WPT "


def run_window_test(dweller, url, timeout, mode):
    env = dict(os.environ, DWELLER_PROCESS=mode, DWELLER_CONSOLE="1", SDL_VIDEODRIVER="dummy",
               DWELLER_WPT_TIMEOUT_MS=str(int(timeout * 1000)))
    cmd = [str(dweller), url, "--width", "800", "--height", "600", "--frames", "16384"]
    p = subprocess.Popen(cmd, stdin=subprocess.DEVNULL, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
                         env=env, start_new_session=True)
    try:
        return read_result(p, time.monotonic() + timeout + 20)
    finally:
        try:
            os.killpg(p.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        p.wait()


def read_result(p, deadline):
    pending = b""
    while time.monotonic() < deadline:
        ready, _, _ = select.select([p.stderr], [], [], max(0.0, deadline - time.monotonic()))
        if not ready:
            break
        chunk = os.read(p.stderr.fileno(), 65536)
        if not chunk:
            return {"status": "CRASH", "message": f"exit {p.wait()} before a result", "subtests": []}
        pending += chunk
        *lines, pending = pending.split(b"\n")
        for line in lines:
            text = line.decode("utf-8", "replace")
            if text.startswith(PREFIX):
                try:
                    return json.loads(text[len(PREFIX):])
                except ValueError:
                    return {"status": "ERROR", "message": "bad result line", "subtests": []}
    return {"status": "TIMEOUT", "message": "no result from testharnessreport.js in the window", "subtests": []}
