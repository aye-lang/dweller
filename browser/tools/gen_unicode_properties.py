#!/usr/bin/env python3
"""Generates browser/js/unicode-property-table.aye, the code point ranges of
the Unicode properties a RegExp `\\p{...}` may name (ECMA-262, "Table 67:
Non-binary Unicode property aliases" and "Table 68: Binary Unicode property
aliases"), from the Unicode Character Database.

    python3 -I browser/tools/gen_unicode_properties.py UCD_DIR [OUT]

UCD_DIR holds these files of https://www.unicode.org/Public/17.0.0/ucd/
(flat, by base name): Scripts.txt, ScriptExtensions.txt, PropList.txt,
DerivedCoreProperties.txt, extracted/DerivedGeneralCategory.txt,
extracted/DerivedBinaryProperties.txt, DerivedNormalizationProps.txt,
emoji/emoji-data.txt, PropertyValueAliases.txt.

The table has three arrays: the ranges of every set (lo, hi pairs,
ascending and disjoint), one (start, count) per set, and the names sorted
by (kind, name) for a binary search. Kind 0 is a binary property, 1 a
General_Category value, 2 a Script value, 3 a Script_Extensions value.
"""

import os
import sys

VERSION = "17.0.0"
LAST = 0x10FFFF

BINARY = [
    ("ASCII",), ("ASCII_Hex_Digit", "AHex"), ("Alphabetic", "Alpha"), ("Any",), ("Assigned",),
    ("Bidi_Control", "Bidi_C"), ("Bidi_Mirrored", "Bidi_M"), ("Case_Ignorable", "CI"), ("Cased",),
    ("Changes_When_Casefolded", "CWCF"), ("Changes_When_Casemapped", "CWCM"),
    ("Changes_When_Lowercased", "CWL"), ("Changes_When_NFKC_Casefolded", "CWKCF"),
    ("Changes_When_Titlecased", "CWT"), ("Changes_When_Uppercased", "CWU"), ("Dash",),
    ("Default_Ignorable_Code_Point", "DI"), ("Deprecated", "Dep"), ("Diacritic", "Dia"), ("Emoji",),
    ("Emoji_Component", "EComp"), ("Emoji_Modifier", "EMod"), ("Emoji_Modifier_Base", "EBase"),
    ("Emoji_Presentation", "EPres"), ("Extended_Pictographic", "ExtPict"), ("Extender", "Ext"),
    ("Grapheme_Base", "Gr_Base"), ("Grapheme_Extend", "Gr_Ext"), ("Hex_Digit", "Hex"),
    ("IDS_Binary_Operator", "IDSB"), ("IDS_Trinary_Operator", "IDST"), ("ID_Continue", "IDC"),
    ("ID_Start", "IDS"), ("Ideographic", "Ideo"), ("Join_Control", "Join_C"),
    ("Logical_Order_Exception", "LOE"), ("Lowercase", "Lower"), ("Math",),
    ("Noncharacter_Code_Point", "NChar"), ("Pattern_Syntax", "Pat_Syn"),
    ("Pattern_White_Space", "Pat_WS"), ("Quotation_Mark", "QMark"), ("Radical",),
    ("Regional_Indicator", "RI"), ("Sentence_Terminal", "STerm"), ("Soft_Dotted", "SD"),
    ("Terminal_Punctuation", "Term"), ("Unified_Ideograph", "UIdeo"), ("Uppercase", "Upper"),
    ("Variation_Selector", "VS"), ("White_Space", "space"), ("XID_Continue", "XIDC"),
    ("XID_Start", "XIDS"),
]
BINARY_FILES = ["PropList.txt", "DerivedCoreProperties.txt", "DerivedBinaryProperties.txt",
                "DerivedNormalizationProps.txt", "emoji-data.txt"]
GROUPS = {"LC": "Ll Lt Lu", "L": "Ll Lm Lo Lt Lu", "M": "Mc Me Mn", "N": "Nd Nl No",
          "P": "Pc Pd Pe Pf Pi Po Ps", "S": "Sc Sk Sm So", "Z": "Zl Zp Zs", "C": "Cc Cf Cn Co Cs"}


def rows(path):
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.split("#")[0].strip()
            if not line:
                continue
            fields = [part.strip() for part in line.split(";")]
            lo, _, hi = fields[0].partition("..")
            yield int(lo, 16), int(hi or lo, 16), fields[1:]


def merged(ranges):
    out = []
    for lo, hi in sorted(ranges):
        if out and lo <= out[-1][1] + 1:
            out[-1][1] = max(out[-1][1], hi)
        else:
            out.append([lo, hi])
    return out


def complement(ranges):
    out, lo = [], 0
    for a, b in merged(ranges):
        if a > lo:
            out.append([lo, a - 1])
        lo = b + 1
    if lo <= LAST:
        out.append([lo, LAST])
    return out


def aliases(path, prop):
    """Short value name -> every name of the value."""
    out = {}
    with open(path, encoding="utf-8") as f:
        for line in f:
            fields = [part.strip() for part in line.split("#")[0].split(";")]
            if fields[0] == prop:
                out[fields[1]] = fields[1:]
    return out


def binary_sets(ucd):
    found = {}
    for name in BINARY_FILES:
        for lo, hi, fields in rows(os.path.join(ucd, name)):
            if len(fields) == 1:
                found.setdefault(fields[0], []).append((lo, hi))
    return found


def script_sets(ucd, short_of):
    sc, per_cp = {}, {}
    for lo, hi, fields in rows(os.path.join(ucd, "Scripts.txt")):
        short = short_of[fields[0]]
        sc.setdefault(short, []).append((lo, hi))
        for cp in range(lo, hi + 1):
            per_cp[cp] = [short]
    for lo, hi, fields in rows(os.path.join(ucd, "ScriptExtensions.txt")):
        for cp in range(lo, hi + 1):
            per_cp[cp] = fields[0].split()
    scx = {}
    for cp, shorts in per_cp.items():
        for short in shorts:
            scx.setdefault(short, []).append((cp, cp))
    sc["Zzzz"] = scx["Zzzz"] = complement([(cp, cp) for cp in per_cp])

    return sc, scx


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    ucd = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(
        os.path.dirname(os.path.abspath(__file__)), "..", "js", "unicode-property-table.aye")
    sets, names = [], []

    def add(kind, all_names, ranges):
        sets.append(merged(ranges))
        for n in dict.fromkeys(all_names):
            names.append((kind, n, len(sets) - 1))

    gc = {}
    for lo, hi, fields in rows(os.path.join(ucd, "DerivedGeneralCategory.txt")):
        gc.setdefault(fields[0], []).append((lo, hi))
    for group, parts in GROUPS.items():
        gc[group] = [r for part in parts.split() for r in gc[part]]
    binary = binary_sets(ucd)
    binary["Any"] = [(0, LAST)]
    binary["ASCII"] = [(0, 0x7F)]
    binary["Assigned"] = complement(gc["Cn"])
    for entry in BINARY:
        add(0, entry, binary[entry[0]])
    for short, all_names in sorted(aliases(os.path.join(ucd, "PropertyValueAliases.txt"), "gc").items()):
        add(1, all_names, gc[short])
    script_names = aliases(os.path.join(ucd, "PropertyValueAliases.txt"), "sc")
    short_of = {all_names[1]: short for short, all_names in script_names.items()}
    sc, scx = script_sets(ucd, short_of)
    for short in sorted(sc):
        add(2, script_names[short], sc[short])
        add(3, script_names[short], scx[short])
    flat = [v for ranges in sets for pair in ranges for v in pair]
    lines = ["// https://www.unicode.org/Public/%s/ucd/ via browser/tools/gen_unicode_properties.py" % VERSION,
             "let UNICODE_PROPERTY_RANGES = [%d]u32{" % len(flat)]
    for i in range(0, len(flat), 12):
        lines.append("    " + ", ".join(str(v) for v in flat[i:i + 12]) + ",")
    lines += ["}", "", "let UNICODE_PROPERTY_SETS = [%d]UnicodePropertySet{" % len(sets)]
    start = 0
    for ranges in sets:
        lines.append("    UnicodePropertySet { start: %d, count: %d }," % (start, len(ranges)))
        start += len(ranges)
    lines += ["}", "", "let UNICODE_PROPERTY_NAMES = [%d]UnicodePropertyName{" % len(names)]
    for kind, n, index in sorted(names):
        lines.append('    UnicodePropertyName { kind: %d, name: "%s", set: %d },' % (kind, n, index))
    lines += ["}", ""]
    with open(out, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))
    print("%s: %d sets, %d ranges, %d names" % (out, len(sets), len(flat) // 2, len(names)))


if __name__ == "__main__":
    main()
