#!/usr/bin/env python3
"""Check the module layers of docs/vision.md ("The shape").

Reads the `import` lines of every .aye file under browser/ and checks them
against the layer table below:

  platform   ui gpu audio font net perf
  core       css layout paint image svg webfont encoding url dom
  front end  html js wasm engine agent

Rules: a core module, or a platform module other than `ui` (the window
shell, which hosts the engine), imports no front-end module, and nothing
imports `ui` except the root (the .aye files directly in browser/, which
are the executables and their tests).

Exceptions are listed in EXCEPTIONS with the card that removes each.
Prints `file:line: message` per violation; exit 1 if any, else 0.
"""
import os
import re
import sys

PLATFORM = {"ui", "gpu", "audio", "font", "net", "perf"}
CORE = {"css", "layout", "paint", "image", "svg", "webfont", "encoding", "url", "dom"}
FRONT = {"html", "js", "wasm", "engine", "agent"}

# (path prefix relative to browser/, imported module): removed by card.
EXCEPTIONS = {
    ("svg/", "html"): "V6 (svg stops parsing through html)",
    ("layout/controls.aye", "html"): "V6 (form code leaves the core)",
    ("paint/form_widgets.aye", "html"): "V6 (form code leaves the core)",
}

IMPORT = re.compile(r"^\s*import\s+([A-Za-z_][A-Za-z0-9_]*)")


def excepted(rel, mod):
    for (prefix, m), _ in EXCEPTIONS.items():
        if m == mod and rel.startswith(prefix):
            return True
    return False


def main():
    root = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), ".."))
    bad = 0
    for d, _, files in os.walk(root):
        for f in sorted(files):
            if not f.endswith(".aye"):
                continue
            path = os.path.join(d, f)
            rel = os.path.relpath(path, root).replace(os.sep, "/")
            top = rel.split("/")[0] if "/" in rel else None  # None: root file
            with open(path, encoding="utf-8") as fh:
                for n, line in enumerate(fh, 1):
                    m = IMPORT.match(line)
                    if not m:
                        continue
                    mod = m.group(1)
                    if mod == "ui" and top is not None and top != "ui":
                        if not excepted(rel, mod):
                            print(f"browser/{rel}:{n}: imports ui; only the root may")
                            bad += 1
                    elif top in (CORE | (PLATFORM - {"ui"})) and mod in FRONT:
                        if not excepted(rel, mod):
                            layer = "core" if top in CORE else "platform"
                            print(f"browser/{rel}:{n}: {layer} module {top} imports front-end module {mod}")
                            bad += 1
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
