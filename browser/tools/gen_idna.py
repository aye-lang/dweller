#!/usr/bin/env python3
"""Generates the binary tables of browser/idna/ (UTS #46 mapping, NFC,
joining types, bidi classes, combining marks) from the Unicode Character
Database and the IDNA mapping table, Unicode 18.0.0:

    python3 -I browser/tools/gen_idna.py UCD_DIR [OUT_DIR]

UCD_DIR holds IdnaMappingTable.txt (https://www.unicode.org/Public/idna/latest/),
UnicodeData.txt, DerivedNormalizationProps.txt, DerivedCombiningClass.txt,
DerivedJoiningType.txt, DerivedBidiClass.txt and DerivedGeneralCategory.txt
(https://www.unicode.org/Public/UCD/latest/ucd/ and its extracted/).

Every file is little-endian 32-bit words. A range table is a count and
then (lo, hi, value) per range, sorted. idna-mapping.bin and
decomposition.bin continue with a pool of (n, code point * n); their
value is the pool offset in words (the pool starts with a 0 word, so no
offset is 0), with the status of a mapping table row
in the top four bits (0 disallowed, 1 valid, 2 mapped, 3 ignored).
composition.bin is a count and then (first, second, composite) sorted.
Statuses follow URL Standard host parsing: UseSTD3ASCIIRules false, so
disallowed_STD3_valid is valid and disallowed_STD3_mapped is mapped;
Transitional_Processing false, so a deviation is valid.
"""

import os
import struct
import sys

STATUS = {"disallowed": 0, "valid": 1, "mapped": 2, "ignored": 3, "deviation": 1,
          "disallowed_STD3_valid": 1, "disallowed_STD3_mapped": 2}
BIDI = {"L": 1, "R": 2, "AL": 3, "AN": 4, "EN": 5, "ES": 6, "ET": 7, "CS": 8, "NSM": 9, "BN": 10, "ON": 11}
BIDI_OTHER = 12
JOINING = {"L": 1, "R": 2, "D": 3, "T": 4}
MAPPED = 2


def rows(path):
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.split("#")[0].strip()
            if line:
                yield [part.strip() for part in line.split(";")]


def span(cps):
    lo, _, hi = cps.partition("..")
    return int(lo, 16), int(hi or lo, 16)


def words(values):
    return struct.pack(f"<{len(values)}I", *values)


def merge(entries):
    out = []
    for lo, hi, value in sorted(entries):
        if out and out[-1][1] + 1 == lo and out[-1][2] == value:
            out[-1][1] = hi
        else:
            out.append([lo, hi, value])
    return out


def flatten(entries):
    merged = merge(entries)
    flat = [len(merged)]
    for lo, hi, value in merged:
        flat += [lo, hi, value]
    return flat


class Pool:
    def __init__(self):
        self.words = [0]
        self.offsets = {}

    def add(self, seq):
        if seq not in self.offsets:
            self.offsets[seq] = len(self.words)
            self.words += [len(seq), *seq]
        return self.offsets[seq]


def mapping_table(path):
    pool, entries = Pool(), []
    for row in rows(path):
        lo, hi = span(row[0])
        status = STATUS[row[1]]
        offset = pool.add(tuple(int(x, 16) for x in row[2].split())) if status == MAPPED else 0
        entries.append((lo, hi, status << 28 | offset))
    return words(flatten(entries) + pool.words)


def decompositions(path):
    out = {}
    for line in open(path, encoding="utf-8"):
        fields = line.split(";")
        if fields[5] and not fields[5].startswith("<"):
            out[int(fields[0], 16)] = [int(x, 16) for x in fields[5].split()]
    return out


def full(cp, decomposition):
    if cp not in decomposition:
        return [cp]
    return [x for part in decomposition[cp] for x in full(part, decomposition)]


def normalization(ucd):
    decomposition = decompositions(os.path.join(ucd, "UnicodeData.txt"))
    excluded = set()
    for row in rows(os.path.join(ucd, "DerivedNormalizationProps.txt")):
        if row[1] == "Full_Composition_Exclusion":
            lo, hi = span(row[0])
            excluded.update(range(lo, hi + 1))
    pool = Pool()
    entries = [(cp, cp, pool.add(tuple(full(cp, decomposition)))) for cp in sorted(decomposition)]
    pairs = sorted((d[0], d[1], cp) for cp, d in decomposition.items() if len(d) == 2 and cp not in excluded)
    composition = [len(pairs)] + [x for pair in pairs for x in pair]
    return words(flatten(entries) + pool.words), words(composition)


def property_table(path, mapper):
    entries = []
    for row in rows(path):
        value = mapper(row[1])
        if value:
            lo, hi = span(row[0])
            entries.append((lo, hi, value))
    return words(flatten(entries))


def main():
    if len(sys.argv) < 2:
        sys.exit(__doc__)
    ucd = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "idna")
    decomposition, composition = normalization(ucd)
    files = {
        "idna-mapping.bin": mapping_table(os.path.join(ucd, "IdnaMappingTable.txt")),
        "decomposition.bin": decomposition,
        "composition.bin": composition,
        "combining-class.bin": property_table(os.path.join(ucd, "DerivedCombiningClass.txt"), int),
        "joining-type.bin": property_table(os.path.join(ucd, "DerivedJoiningType.txt"), lambda v: JOINING.get(v, 0)),
        "bidi-class.bin": property_table(os.path.join(ucd, "DerivedBidiClass.txt"), lambda v: BIDI.get(v, BIDI_OTHER)),
        "mark.bin": property_table(os.path.join(ucd, "DerivedGeneralCategory.txt"), lambda v: 1 if v in ("Mn", "Mc", "Me") else 0),
    }
    for name, data in files.items():
        with open(os.path.join(out, name), "wb") as f:
            f.write(data)


if __name__ == "__main__":
    main()
