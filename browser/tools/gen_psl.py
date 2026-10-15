#!/usr/bin/env python3
"""Generates browser/net/psl.txt, the public suffix list that cookies.aye
embeds, from Mozilla's list (https://publicsuffix.org/list/).

    python3 -I browser/tools/gen_psl.py public_suffix_list.dat [OUT]

OUT defaults to browser/net/psl.txt: a header (where the list comes from,
its VERSION and COMMIT lines, its license), a blank line, then the rules
of both sections (ICANN and private), one per line, internationalized
names in their ASCII (punycode) form as URL hosts have them, lower case,
sorted by bytes and without duplicates, so that psl.aye finds a rule by
binary search over the embedded bytes.
"""

import os
import sys


def ascii_rule(rule):
    labels = []
    for label in rule.split("."):
        if label.isascii():
            labels.append(label.lower())
        else:
            labels.append("xn--" + label.lower().encode("punycode").decode("ascii"))
    return ".".join(labels)


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    src = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "net", "psl.txt")
    with open(src, encoding="utf-8") as f:
        lines = f.read().splitlines()
    version = next((l for l in lines if l.startswith("// VERSION:")), "")
    commit = next((l for l in lines if l.startswith("// COMMIT:")), "")
    rules = set()
    for line in lines:
        rule = line.strip().split()[0] if line.strip() else ""
        if rule == "" or rule.startswith("//"):
            continue
        rules.add(ascii_rule(rule))
    header = [
        "// The Public Suffix List (https://publicsuffix.org/list/public_suffix_list.dat),",
        "// both sections, in ASCII, sorted; made by browser/tools/gen_psl.py.",
        version,
        commit,
        "// This Source Code Form is subject to the terms of the Mozilla Public",
        "// License, v. 2.0. If a copy of the MPL was not distributed with this",
        "// file, You can obtain one at https://mozilla.org/MPL/2.0/.",
    ]
    body = sorted(rules, key=lambda r: r.encode("ascii"))
    with open(out, "w", encoding="ascii", newline="\n") as f:
        f.write("\n".join(header) + "\n\n" + "\n".join(body) + "\n")


if __name__ == "__main__":
    main()
