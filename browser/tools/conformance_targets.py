"""Targets beside the floors of docs/conformance.md (Q0.3b).

A floor (`threshold`) stops a fall and `conformance.py check` enforces
it. A target says where an area is ready: `share` of the subtests that
need no deferred feature, over all subtests of the area, in whole
points. conformance/targets.json names each area's deferred features as
path patterns with the reason; `conformance.py targets` counts them in
the baseline (or in this machine's results where there is no baseline),
writes the numbers back and refreshes the column and the "Targets"
section of the document. Nothing checks a target.
"""
import fnmatch
import json
import math

import conformance

HEADING = "## Targets"


def targets_file():
    return conformance.BASELINES / "targets.json"


def read():
    return json.loads(targets_file().read_text())


def measured(area):
    return conformance.read_baseline(area) or (conformance.measure(area) if conformance.available(area) else None)


def deferred_subtests(tests, patterns):
    return sum(n for t, (_, n) in tests.items() if any(fnmatch.fnmatch(t, p) for p in patterns))


def compute(area, entry, data):
    m = measured(area)

    if m is None:
        return entry

    patterns = [*entry["deferred"], *data["everywhere"]]
    counted = m["total"] - deferred_subtests(m["tests"], patterns)
    target = max(math.floor(data["share"] * counted / m["total"] * 100), round(m["threshold"] * 100))
    return {**entry, "target": target, "counted": counted, "total": m["total"]}


def recompute():
    data = read()
    data["areas"] = {area: compute(area, entry, data) for area, entry in data["areas"].items()}
    targets_file().write_text(json.dumps(data, indent=1, ensure_ascii=False) + "\n")
    return data


def cell(area, data):
    entry = data["areas"].get(area, {})
    return str(entry["target"]) if "target" in entry else "-"


def reason(area, entry, data):
    why = sorted(set(entry["deferred"].values()) | set(data["everywhere"].values()))
    return (f"- `{area}`: {entry['target']}. {data['share'] * 100:.0f}% of the {entry['counted']} of {entry['total']} "
            f"subtests that need none of: {'; '.join(why)}.")


def section(data):
    head = [HEADING, "",
            "A threshold is a floor: `conformance.py check` fails when an area falls, and the floor was set under a measured score.",
            f"A target is where the area is ready: {data['share'] * 100:.0f}% of the subtests that need no deferred feature, over all its subtests.",
            "Nothing checks a target. The patterns and reasons are in `browser/tools/conformance/targets.json`; "
            "`conformance.py targets` counts them again.", ""]
    return head + [reason(a, e, data) for a, e in data["areas"].items() if "target" in e]


def target_cell(cells, data):
    if cells[1].strip() == "Area":
        return " Target % "

    return "---:" if cells[1].startswith("---") else f" {cell(cells[1].strip(' `'), data)} "


def with_column(line, at, data):
    cells = line.split("|")
    cells = cells[:at] + cells[at + 1:] if len(cells) == 11 else cells
    return "|".join(cells[:at] + [target_cell(cells, data)] + cells[at:])


def apply(text, data):
    lines = text.split(f"\n{HEADING}\n")[0].rstrip("\n").split("\n")
    first = next(i for i, line in enumerate(lines) if line.startswith("| Area | Files"))
    last = next(i for i in range(first, len(lines) + 1) if i == len(lines) or not lines[i].startswith("|"))
    at = lines[first].replace(" Target % |", "").split("|").index(" Threshold % ") + 1
    table = [with_column(line, at, data) for line in lines[first:last]]
    return "\n".join(lines[:first] + table + lines[last:] + ["", *section(data)]) + "\n"


def cmd_targets(args):
    data = recompute()
    conformance.DOC.write_text(apply(conformance.DOC.read_text(), data))
    print(f"wrote {len([e for e in data['areas'].values() if 'target' in e])} targets to {targets_file().name} and {conformance.DOC.name}")
