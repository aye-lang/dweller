#!/usr/bin/env python3
"""Generates browser/layout/line-break-table.aye, the UAX #14 line breaking
classes that the East Asian line breaking rules use, from the Unicode
Character Database (https://www.unicode.org/Public/15.1.0/ucd/).

    python3 -I browser/tools/gen_line_break.py LineBreak.txt EastAsianWidth.txt [OUT]

One LineBreakRange per run of code points with the same class, sorted for
a binary search; classes the rules do not use are left out (AL). PO and
PR with East Asian Width A, F or W are PO_WIDE and PR_WIDE.
"""

import os
import sys

KEPT = {"OP", "CL", "CP", "NS", "EX", "IS", "IN", "PO", "PR", "BA", "HY", "GL", "QU", "CJ", "ID", "EB", "EM"}


def ranges(path):
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.split("#")[0].strip()
            if not line:
                continue
            cps, value = (part.strip() for part in line.split(";"))
            lo, _, hi = cps.partition("..")
            yield int(lo, 16), int(hi or lo, 16), value


def main():
    if len(sys.argv) < 3:
        sys.exit(__doc__)
    out = sys.argv[3] if len(sys.argv) > 3 else os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "layout", "line-break-table.aye")
    wide = set()
    for lo, hi, value in ranges(sys.argv[2]):
        if value in ("A", "F", "W"):
            wide.update(range(lo, hi + 1))
    classes = {}
    for lo, hi, value in ranges(sys.argv[1]):
        if value not in KEPT:
            continue
        for c in range(lo, hi + 1):
            name = "ID" if value in ("EB", "EM") else value
            if name in ("PO", "PR") and c in wide:
                name += "_WIDE"
            classes[c] = name
    runs = []
    for c in sorted(classes):
        if runs and runs[-1][1] == c - 1 and runs[-1][2] == classes[c]:
            runs[-1][1] = c
        else:
            runs.append([c, c, classes[c]])
    rows = [f"    LineBreakRange {{ lo: {lo}, hi: {hi}, class: LineBreakClass.{name} }}," for lo, hi, name in runs]
    header = "// https://www.unicode.org/Public/15.1.0/ucd/LineBreak.txt via browser/tools/gen_line_break.py"
    with open(out, "w", encoding="ascii", newline="\n") as f:
        f.write(f"{header}\nlet LINE_BREAK_RANGES = [{len(rows)}]LineBreakRange{{\n" + "\n".join(rows) + "\n}\n")


if __name__ == "__main__":
    main()
