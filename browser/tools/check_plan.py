#!/usr/bin/env python3
"""Check rule 2 of docs/plans/velocity.md: a status has one home.

Step ids checked:
  Q  rows `| Qn.m |` in docs/plans/foundations.md (open) and
     docs/plans/foundations-done.md (done), cards docs/plans/cards/Qn.m.md
  DV rows `| DVn |` in the "Cards" table of docs/plans/velocity.md, whose
     Status column is the row's state (done or not), cards cards/DVn.md
Cards of other lanes (P4, P10, P11, TC, ...) have their rows in
docs/plans/browser.md and are not checked. The V lane (vision.md) has no
cards in cards/.

A card's status is its first `## Status:` line; it is done when the text
starts with "done". A card with a row and no such line is a violation.
A card with no row of its own is fine when its parent step has a row: a
slice card such as Q4.16a (parent Q4.16) or Q1.10d2 (parents Q1.10d,
Q1.10).

Violations (`file:line: message`, exit 1 if any):
  - a Q id is a row in both foundations files, or a card exists for an id
    with no row and no parent row;
  - a card says done and its row is open, or its row is done and the card
    says it is not, or the card has no `## Status:` line;
  - a row in foundations.md is longer than 600 characters;
  - "Next, per lane" names a done step.
Then one line per phase: open rows, done rows, Left items in open cards.

Usage: check_plan.py [--root DIR]  (DIR holds docs/plans; default: repo root)
"""
import argparse
import os
import re
import sys

PLANS = "docs/plans"
OPEN, DONE, VELOCITY = f"{PLANS}/foundations.md", f"{PLANS}/foundations-done.md", f"{PLANS}/velocity.md"
MAX_ROW = 600
STEP_ID = r"(?:Q\d+\.\d+[a-z0-9]*|DV\d+)"
Q_ROW = re.compile(r"^\| (Q\d+\.\d+[a-z0-9]*) \|")
DV_ROW = re.compile(r"^\| (DV\d+) \|.*\| ([^|]*) \|\s*$")
STATUS = re.compile(r"^## Status:\s*(.*)")
LEFT = re.compile(r"^(\s*)(?:(?:- )?left\b[^:]*:|## left\s*$)\s*(.*)$", re.IGNORECASE)
MENTION = re.compile(rf"\b({STEP_ID})\b")
PHASE_HEADING = re.compile(r"^## (Q\d+):")


def read_lines(root, rel):
    with open(os.path.join(root, rel), encoding="utf-8") as fh:
        return fh.read().split("\n")


def rows(lines, pattern):
    """{id: (line number, line)} for rows matching pattern; the first wins."""
    found = {}
    for n, line in enumerate(lines, 1):
        m = pattern.match(line)
        if m and m.group(1) not in found:
            found[m.group(1)] = (n, line)
    return found


def phase(step):
    return "DV" if step.startswith("DV") else step.split(".")[0]


def parents(step):
    """Q4.16a -> [Q4.16]; Q1.10d2 -> [Q1.10d, Q1.10]."""
    out = []
    m = re.match(r"^(Q\d+\.\d+[a-z]+)\d+$", step)
    if m:
        out.append(m.group(1))
    m = re.match(r"^(Q\d+\.\d+)[a-z]", step)
    if m:
        out.append(m.group(1))
    return out


def left_items(lines):
    """Top-level items of the card's Left lists ("Left:", "- left:", "## Left")."""
    count = 0
    for i, line in enumerate(lines):
        m = LEFT.match(line)
        if not m:
            continue
        indent = len(m.group(1)) + (2 if line.lstrip().startswith("- ") else 0)
        count += 1 if m.group(2) else 0
        rest = lines[i + 1:]
        while line.startswith("## ") and rest and not rest[0].strip():
            rest = rest[1:]
        for item in rest:
            depth = len(item) - len(item.lstrip())
            if not item.strip() or depth < indent or (depth == indent and not item.lstrip().startswith("- ")):
                break
            count += 1 if depth == indent else 0
    return count


def read_cards(root):
    """{id: (status line number or None, done, Left items)} for Q and DV cards."""
    cards = {}
    folder = os.path.join(root, PLANS, "cards")
    for f in sorted(os.listdir(folder)):
        step = f[:-3]
        if not f.endswith(".md") or not re.fullmatch(STEP_ID, step):
            continue
        lines = read_lines(root, f"{PLANS}/cards/{f}")
        status = [(n, STATUS.match(line).group(1)) for n, line in enumerate(lines, 1) if STATUS.match(line)]
        status_line, text = status[0] if status else (None, "")
        cards[step] = (status_line, text.lower().startswith("done"), left_items(lines))
    return cards


def next_per_lane(lines):
    """[(line number, id)] named in the "Next, per lane" table."""
    out = []
    inside = False
    for n, line in enumerate(lines, 1):
        if line.startswith("#"):
            inside = line.strip() == "### Next, per lane"
            continue
        if inside and line.startswith("|"):
            out.extend((n, s) for s in MENTION.findall(line))
    return out


def check(root):
    open_lines, done_lines, vel_lines = (read_lines(root, f) for f in (OPEN, DONE, VELOCITY))
    open_rows, done_rows = rows(open_lines, Q_ROW), rows(done_lines, Q_ROW)
    dv_rows = rows(vel_lines, DV_ROW)
    dv_done = {s for s, (_, line) in dv_rows.items() if DV_ROW.match(line).group(2).strip().startswith("done")}
    cards = read_cards(root)
    bad = []

    for step in sorted(open_rows.keys() & done_rows.keys()):
        bad.append(f"{OPEN}:{open_rows[step][0]}: {step} is open here and done in {DONE}:{done_rows[step][0]}")
    for step, (n, line) in open_rows.items():
        if len(line) > MAX_ROW:
            bad.append(f"{OPEN}:{n}: {step}'s row is {len(line)} characters (at most {MAX_ROW}; the detail belongs in its card)")

    has_row = open_rows.keys() | done_rows.keys() | dv_rows.keys()
    for step, (status_line, done, _) in cards.items():
        card = f"{PLANS}/cards/{step}.md"
        if step not in has_row:
            if not any(p in has_row for p in parents(step)):
                where = VELOCITY if step.startswith("DV") else f"{OPEN} or {DONE}"
                bad.append(f"{card}:1: {step} has a card but no row in {where}")
            continue
        if status_line is None:
            bad.append(f"{card}:1: {step}'s card has no `## Status:` line")
            continue
        row_done = step in done_rows or step in dv_done
        row_file, row_line = (VELOCITY, dv_rows[step][0]) if step in dv_rows else \
            (DONE, done_rows[step][0]) if row_done else (OPEN, open_rows[step][0])
        if done and not row_done:
            bad.append(f"{row_file}:{row_line}: {step} is open here but {card}:{status_line} says done")
        elif row_done and not done:
            bad.append(f"{row_file}:{row_line}: {step} is done here but {card}:{status_line} says it is not")

    done_steps = done_rows.keys() | dv_done | {s for s, c in cards.items() if c[1]}
    for n, step in next_per_lane(open_lines):
        if step in done_steps:
            bad.append(f'{OPEN}:{n}: "Next, per lane" names {step}, which is done')
    return bad, summary(open_lines, open_rows, done_rows, dv_rows, dv_done, cards)


def summary(open_lines, open_rows, done_rows, dv_rows, dv_done, cards):
    phases = [m.group(1) for m in map(PHASE_HEADING.match, open_lines) if m] + ["DV"]
    counts = {p: [0, 0, 0] for p in phases}
    open_steps = list(open_rows) + [s for s in dv_rows if s not in dv_done]
    for step in open_steps:
        c = counts.setdefault(phase(step), [0, 0, 0])
        c[0] += 1
        c[2] += cards[step][2] if step in cards else 0
    for step in list(done_rows) + sorted(dv_done):
        counts.setdefault(phase(step), [0, 0, 0])[1] += 1
    return [f"{p}: {o} open, {d} done, {l} Left items in the open cards" for p, (o, d, l) in counts.items()]


def main():
    default = os.path.normpath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
    parser = argparse.ArgumentParser(description="Check the plan's one-home rule.")
    parser.add_argument("--root", default=default, help="directory that holds docs/plans (default: the repository root)")
    bad, lines = check(parser.parse_args().root)
    for line in bad + lines:
        print(line)
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
