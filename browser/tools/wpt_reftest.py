"""Reftests and crashtests for wpt.py (P10.2)."""
import os
import posixpath
import re
import subprocess
import tempfile
from pathlib import Path

import wpt_png

SHOT_WALL = 60
WAIT_CLASS = "reftest-wait"
LINK = re.compile(r"""<link\b[^>]*\brel\s*=\s*["']?(match|mismatch)["']?[^>]*>""", re.I)
HREF = re.compile(r"""\bhref\s*=\s*["']?([^"'\s>]+)""", re.I)
FUZZY_META = re.compile(r"""<meta\b[^>]*\bname\s*=\s*["']?fuzzy["']?[^>]*\bcontent\s*=\s*["']([^"']*)["']""", re.I)
FUZZY_FIELD = re.compile(r"(maxDifference|totalPixels)\s*=\s*(\d+)(?:-(\d+))?")


def is_crashtest(rel):
    parts = rel.split("/")
    return "crashtests" in parts[:-1] or re.search(r"-crash\.[a-z]+$", parts[-1]) is not None


def has_reftest_link(text):
    return LINK.search(text) is not None


def links_of(text):
    """[(kind, href)] of the match and mismatch links of a page."""
    found = []
    for m in LINK.finditer(text):
        href = HREF.search(m.group(0))
        if href:
            found.append((m.group(1).lower(), href.group(1)))
    return found


def fuzzy_of(text):
    """(largest allowed channel difference, most differing pixels)."""
    allowed = {"maxDifference": 0, "totalPixels": 0}
    m = FUZZY_META.search(text)
    for name, low, high in FUZZY_FIELD.findall(m.group(1) if m else ""):
        allowed[name] = int(high or low)
    return allowed["maxDifference"], allowed["totalPixels"]


def screenshot(dweller, url, out, root, wait):
    cmd = [str(dweller), "screenshot", url, "-o", str(out), "--width", "800", "--height", "600"]
    if wait:
        cmd += ["--wait-class", WAIT_CLASS]
    env = dict(os.environ, DWELLER_FONT_DIRS=str(Path(root) / "fonts"))
    p = subprocess.run(cmd, capture_output=True, text=True, timeout=SHOT_WALL,
                       errors="replace", env=env, stdin=subprocess.DEVNULL)
    if p.returncode != 0:
        raise RuntimeError(f"exit {p.returncode}: {(p.stderr.strip().splitlines() or [''])[-1][:200]}")
    return Path(out).read_bytes()


def same_enough(png_a, png_b, fuzzy):
    if png_a == png_b:
        return True
    diff = wpt_png.difference(png_a, png_b)
    return diff is not None and diff[0] <= fuzzy[1] and diff[1] <= fuzzy[0]


def read_page(root, rel):
    return (Path(root) / rel).read_text(errors="replace")


def resolve(rel, href):
    href = href.split("#")[0].split("?")[0]
    if href.startswith("/"):
        return href.lstrip("/")
    return posixpath.normpath(posixpath.join(posixpath.dirname(rel), href))


def comparisons(root, rel, text):
    """[(page a, page b, kind, fuzzy)] of root-relative paths: the test
    against each reference, and each reference against the references it
    names."""
    out = []
    for kind, href in links_of(text):
        ref = resolve(rel, href)
        out.append((rel, ref, kind, fuzzy_of(text)))
        try:
            ref_text = read_page(root, ref)
        except OSError:
            continue
        out += [(ref, resolve(ref, h), k, fuzzy_of(ref_text)) for k, h in links_of(ref_text)]
    return out


def compare(dweller, root, base_url, tmp, shots, comparison, wait):
    a, b, kind, fuzzy = comparison
    for rel in (a, b):
        if rel not in shots:
            shots[rel] = screenshot(dweller, f"{base_url}/{rel}", Path(tmp) / f"{len(shots)}.png", root, wait)
    if kind == "match":
        return same_enough(shots[a], shots[b], fuzzy), f"{a} differs from {b}"
    return shots[a] != shots[b], f"{a} equals {b}"


def verdict(status, ok, message):
    sub = {"name": "reftest", "status": "PASS" if ok else "FAIL", "message": None if ok else message}
    return {"status": status, "message": None, "subtests": [sub]}


def run_reftest(dweller, root, base_url, test):
    text = read_page(root, test["path"])
    shots = {}
    try:
        with tempfile.TemporaryDirectory() as tmp:
            for c in comparisons(root, test["path"], text):
                ok, message = compare(dweller, root, base_url, tmp, shots, c, WAIT_CLASS in text)
                if not ok:
                    return verdict("OK", False, message)
    except subprocess.TimeoutExpired:
        return {"status": "TIMEOUT", "message": f"screenshot over {SHOT_WALL} s", "subtests": []}
    except (RuntimeError, ValueError, OSError) as e:
        return {"status": "CRASH", "message": str(e), "subtests": []}
    return verdict("OK", True, None)


def run_crashtest(dweller, root, base_url, test):
    wait = WAIT_CLASS in read_page(root, test["path"])
    try:
        with tempfile.TemporaryDirectory() as tmp:
            screenshot(dweller, base_url + test["url"], Path(tmp) / "shot.png", root, wait)
    except subprocess.TimeoutExpired:
        return {"status": "TIMEOUT", "message": f"no result in {SHOT_WALL} s", "subtests": []}
    except (RuntimeError, OSError) as e:
        return {"status": "CRASH", "message": str(e), "subtests": []}
    return verdict("OK", True, None)
