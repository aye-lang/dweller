#!/usr/bin/env python3
"""Writes browser/css/property-ids.aye: `NPROPS` and one id constant per
longhand, its position in `PROPERTIES` (browser/css/css.aye), named after
the property (`border-collapse` is `BORDER_COLLAPSE`). `--check` fails when
the checked-in file is stale.

    python3 browser/tools/gen_property_ids.py [--check]
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parents[2]
TABLE = ROOT / "browser/css/css.aye"
OUT = ROOT / "browser/css/property-ids.aye"

NAMES = {
    "top": "INSET_TOP",
    "-webkit-border-horizontal-spacing": "BORDER_SPACING_H",
    "-webkit-border-vertical-spacing": "BORDER_SPACING_V",
}


def source():
    names = re.findall(r'^    Property \{ name: "([^"]+)"', TABLE.read_text(), re.M)
    lines = [f"pub let NPROPS: usize = {len(names)}"]
    for i, name in enumerate(names):
        constant = NAMES.get(name, name.lstrip("-").upper().replace("-", "_"))
        lines.append(f"pub let {constant}: usize = {i}")
    return "\n".join(lines) + "\n"


def main():
    if sys.argv[1:] == ["--check"]:
        if OUT.read_text() != source():
            sys.exit(f"{OUT.relative_to(ROOT)} is stale: run python3 browser/tools/gen_property_ids.py")
        return
    OUT.write_text(source())


if __name__ == "__main__":
    main()
