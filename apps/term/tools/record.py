#!/usr/bin/env python3
"""Record what a program writes to an 80x24 terminal, for the parser tests.

    record.py OUT.bytes SECONDS KEYS -- PROGRAM [ARGS...]

Runs PROGRAM on a pty of 80 columns and 24 rows with TERM=xterm-256color,
waits SECONDS, sends KEYS (Python escapes: "\\x1b:q!\\r"), and writes
every byte the program wrote until it exits (or 2 more seconds pass) to
OUT.bytes. `--until-keys` stops the recording before the keys are sent, so
the file ends with the program's screen still shown.
"""
import fcntl
import os
import pty
import select
import struct
import sys
import termios
import time


def main():
    args = sys.argv[1:]
    until_keys = "--until-keys" in args
    if until_keys:
        args.remove("--until-keys")
    cut = args.index("--")
    out, seconds, keys = args[0], float(args[1]), args[2].encode().decode("unicode_escape").encode("latin-1")
    command = args[cut + 1:]
    pid, fd = pty.fork()
    if pid == 0:
        env = dict(os.environ, TERM="xterm-256color", LANG="C.UTF-8", COLUMNS="80", LINES="24")
        os.execvpe(command[0], command, env)
    fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack("HHHH", 24, 80, 0, 0))
    data = bytearray()
    sent = False
    start = time.time()
    end = start + seconds + 2
    while time.time() < end:
        ready, _, _ = select.select([fd], [], [], 0.05)
        if ready:
            try:
                chunk = os.read(fd, 65536)
            except OSError:
                break
            if not chunk:
                break
            data += chunk
        if not sent and time.time() >= start + seconds:
            sent = True
            if until_keys:
                with open(out, "wb") as f:
                    f.write(data)
            os.write(fd, keys)
    if not until_keys:
        with open(out, "wb") as f:
            f.write(data)
    try:
        os.kill(pid, 9)
    except OSError:
        pass


if __name__ == "__main__":
    main()
