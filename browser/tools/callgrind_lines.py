#!/usr/bin/env python3
"""Instruction counts of two callgrind outputs, per function and per source line.

    python3 browser/tools/callgrind_lines.py SAFE.out FAST.out [TOP]

Both outputs must come from builds of the same source with line info
(`aye build browser --mode fast --cflags -g`; safe mode has it already).
Prints the TOP (default 25) functions and then the TOP lines with the
largest SAFE - FAST difference. A function's cost is its self cost, so a
callee that one build inlines and the other does not shows up as a
function of one build only; a line's cost is summed over every function
it was inlined into, and line 0 is code the compiler emitted for no line
(the checks and their panic calls).
"""
import re
import sys
from collections import defaultdict


def parse(path):
    names = {}
    by_line = defaultdict(int)
    by_fn = defaultdict(int)
    fl = fn = None
    line = 0
    skip = False
    for raw in open(path):
        raw = raw.rstrip("\n")
        m = re.match(r"^(fl|fi|fe|fn|cfi|cfl|cfn|ob|cob)=\((\d+)\)(?: (.*))?$", raw)
        if m:
            kind, ident, name = m.groups()
            space = "fn" if kind.endswith("fn") else "fl"
            if name:
                names[(space, ident)] = name
            if kind in ("fl", "fi", "fe"):
                fl = names.get(("fl", ident))
            elif kind == "fn":
                fn = names.get(("fn", ident))
            continue
        if raw.startswith("calls="):
            skip = True
            continue
        m = re.match(r"^([+-]\d+|\d+|\*)\s+(\d+)", raw)
        if not m:
            continue
        pos, c = m.groups()
        if pos[0] in "+-":
            line += int(pos)
        elif pos != "*":
            line = int(pos)
        if skip:
            skip = False
            continue
        by_line[(fl, line)] += int(c)
        by_fn[fn] += int(c)
    return by_line, by_fn


def short(fn):
    return re.sub(r"^aye_[a-z]_\d+", "", fn or "?")


def main(argv):
    (la, fa), (lb, fb) = parse(argv[1]), parse(argv[2])
    top = int(argv[3]) if len(argv) > 3 else 25
    print("total\t%d\t%d\t%d" % (sum(fa.values()), sum(fb.values()), sum(fa.values()) - sum(fb.values())))
    print("\nfunction\tsafe\tfast\tdiff")
    for f in sorted(set(fa) | set(fb), key=lambda f: -abs(fa.get(f, 0) - fb.get(f, 0)))[:top]:
        print(f"{short(f)}\t{fa.get(f, 0)}\t{fb.get(f, 0)}\t{fa.get(f, 0) - fb.get(f, 0)}")
    print("\nfile:line\tsafe\tfast\tdiff")
    for k in sorted(set(la) | set(lb), key=lambda k: -abs(la.get(k, 0) - lb.get(k, 0)))[:top]:
        print(f"{k[0]}:{k[1]}\t{la.get(k, 0)}\t{lb.get(k, 0)}\t{la.get(k, 0) - lb.get(k, 0)}")


main(sys.argv)
