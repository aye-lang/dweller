import hashlib
import sys
from pathlib import Path


def is_optimized(binary):
    markers = [Path(f"{binary}.{mode}") for mode in ("safe", "fast")]

    return any(m.is_file() and m.read_text().strip() == hashlib.sha256(Path(binary).read_bytes()).hexdigest() for m in markers)


def require(binary, allow_debug):
    if not allow_debug and not is_optimized(binary):
        print(f"{binary} is not a safe or fast build (browser/build.sh); its timings are 10x too slow (--allow-debug to measure anyway)",
              file=sys.stderr)
        raise SystemExit(2)
