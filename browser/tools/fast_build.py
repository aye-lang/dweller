import hashlib
import sys
from pathlib import Path


def is_fast(binary):
    marker = Path(f"{binary}.fast")
    return marker.is_file() and marker.read_text().strip() == hashlib.sha256(Path(binary).read_bytes()).hexdigest()


def require(binary, allow_debug):
    if not allow_debug and not is_fast(binary):
        print(f"{binary} is not a fast build (browser/build.sh); its timings are 10x too slow (--allow-debug to measure anyway)",
              file=sys.stderr)
        raise SystemExit(2)
