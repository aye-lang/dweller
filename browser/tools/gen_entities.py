#!/usr/bin/env python3
"""Generates browser/html/entity-table.aye, the named character references
that entities.aye searches, from the HTML standard's list
(https://html.spec.whatwg.org/entities.json).

    python3 -I browser/tools/gen_entities.py entities.json [OUT]

One Entity per name without `&` and `;`, sorted by bytes for a binary
search; `legacy` marks the names that also match without the final `;`.
"""

import json
import os
import sys


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "html", "entity-table.aye")
    with open(sys.argv[1], encoding="utf-8") as f:
        data = json.load(f)
    names = {}
    for key, value in data.items():
        name = key[1:].rstrip(";")
        points = value["codepoints"] + [0]
        legacy = not key.endswith(";") or names.get(name, (0, 0, False))[2]
        names[name] = (points[0], points[1], legacy)
    rows = [f'    Entity {{ name: "{n}", code: {c}, code2: {c2}, legacy: {"true" if l else "false"} }},'
            for n, (c, c2, l) in sorted(names.items(), key=lambda kv: kv[0].encode("ascii"))]
    header = "// https://html.spec.whatwg.org/entities.json via browser/tools/gen_entities.py"
    with open(out, "w", encoding="ascii", newline="\n") as f:
        f.write(f"{header}\nlet ENTITIES = [{len(rows)}]Entity{{\n" + "\n".join(rows) + "\n}\n")


if __name__ == "__main__":
    main()
