#!/usr/bin/env python3
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASELINE = ROOT / "browser/tools/taste_baseline.json"
EXEMPT_KINDS = ("struct", "enum", "table", "registry", "generated")
GENERATED_OR_VENDORED = ("browser/encoding/tables.aye", "browser/html/entity-table.aye")
REFERENCE = re.compile(r"https?://|\b(v8|blink|chromium|angle|spec):")
DIRECTIVE = re.compile(r"//\s*(aye|taste):")
FN_START = re.compile(r"^\s*(pub )?fn ")
BLOCK_START = re.compile(r"^\s*(if|for|while|match|else|loop)\b.*\{\s*$")
GUARD = re.compile(r"^\s*if\b.*\{\s*(return|continue|break)\b[^{}]*\}\s*$")
LET = re.compile(r"^\s*(let|var)\b")
MAX_FILE_LINES = 300
MAX_FUNCTION_BODY = 5


def aye_files():
    for path in sorted((ROOT / "browser").rglob("*.aye")):
        rel = path.relative_to(ROOT).as_posix()
        if rel not in GENERATED_OR_VENDORED:
            yield rel


def split_comment(line):
    quote = None
    i = 0
    while i < len(line):
        c = line[i]
        if quote:
            if c == "\\":
                i += 1
            elif c == quote:
                quote = None
        elif c in "\"'":
            quote = c
        elif line.startswith("//", i):
            return line[:i], line[i:]
        i += 1
    return line, None


def comment_violations(lines):
    found = []
    flagged = [split_comment(line)[1] for line in lines]
    for i, comment in enumerate(flagged):
        if comment is None or DIRECTIVE.match(comment):
            continue
        whole_line = lines[i].strip().startswith("//")
        alone = whole_line and not (i > 0 and flagged[i - 1] and lines[i - 1].strip().startswith("//")) \
            and not (i + 1 < len(flagged) and flagged[i + 1] and lines[i + 1].strip().startswith("//"))
        if alone and REFERENCE.search(comment) and not comment.startswith("///"):
            continue
        found.append((i + 1, "comment"))
    return found


def code_of(line):
    return split_comment(line)[0].rstrip()


def strip_strings(code):
    return re.sub(r'"(\\.|[^"\\])*"|\'(\\.|[^\'\\])*\'', '""', code)


def find_functions(lines):
    """Yield (start, open, end) 0-based line indices for each fn with a body."""
    i = 0
    while i < len(lines):
        if not FN_START.match(lines[i]):
            i += 1
            continue
        depth = 0
        opened = None
        j = i
        while j < len(lines):
            code = strip_strings(code_of(lines[j]))
            for c in code:
                if c == "{":
                    if opened is None:
                        opened = j
                    depth += 1
                elif c == "}":
                    depth -= 1
            if opened is not None and depth == 0:
                yield i, opened, j
                break
            if opened is None and j > i and FN_START.match(lines[j]):
                break
            if opened is None and not code.strip():
                break
            j += 1
        i = max(j, i) + 1 if opened is not None else i + 1


def function_violations(lines, is_test):
    long_fns = []
    for start, opened, end in find_functions(lines):
        if end - opened - 1 > MAX_FUNCTION_BODY:
            long_fns.append(start + 1)
    return long_fns


def blank_violations(lines):
    found = []
    code = [code_of(line) for line in lines]
    n = len(lines)

    def blank(i):
        return not lines[i].strip()

    def prev_stmt(i):
        j = i - 1
        while j >= 0 and blank(j):
            j -= 1
        return j

    for start, opened, end in find_functions(lines):
        for i in range(opened + 1, end):
            line = code[i]
            if not line.strip():
                continue
            p = i - 1
            prev = code[p] if p >= 0 else ""
            prev_open = prev.rstrip().endswith("{") or p <= opened
            first = prev_open
            is_guard = bool(GUARD.match(line))
            prev_guard = bool(GUARD.match(prev))
            if BLOCK_START.match(line) and not is_guard and not first and not blank(p) \
                    and not line.lstrip().startswith("else") and not prev.strip().startswith("}"):
                found.append((i + 1, "blank-before-block"))
            elif BLOCK_START.match(line) and not is_guard and not first and not blank(p) \
                    and prev.strip() == "}" and not line.lstrip().startswith("else"):
                found.append((i + 1, "blank-before-block"))
            elif re.match(r"^\s*return\b", line) and not first and not blank(p) and not prev_guard:
                found.append((i + 1, "blank-before-return"))
            elif LET.match(line) is None and not line.lstrip().startswith("defer") and not first and not blank(p) and LET.match(prev) \
                    and not line.strip().startswith(("}", ")", "]", "."))  and not prev.rstrip().endswith(("(", "[", ",")):
                found.append((i + 1, "blank-after-let"))
            elif line.strip() == "}" or line.strip().startswith("} else"):
                pass
            elif not first and not blank(p) and prev.strip() == "}" and not line.strip().startswith(("}", ")", "else")) \
                    and not is_guard:
                found.append((i + 1, "blank-after-block"))
    for i in range(n):
        if FN_START.match(lines[i]) and i > 0:
            j = i - 1
            if not blank(j) and not lines[j].strip().startswith(("//", "@", "#")) \
                    and not lines[j].rstrip().endswith("{"):
                found.append((i + 1, "blank-between-items"))
    return found


def analyse(rel):
    lines = (ROOT / rel).read_text(encoding="utf-8").splitlines()
    is_test = bool(re.search(r"(_tests?|unit_tests)\.aye$", rel))
    comments = comment_violations(lines)
    long_fns = [] if is_test else function_violations(lines, is_test)
    blanks = blank_violations(lines)
    return {
        "lines": len(lines),
        "comment_lines": len(comments),
        "long_functions": len(long_fns),
        "blank_line_violations": len(blanks),
    }, {
        "comment_lines": comments,
        "long_functions": [(n, "long-function") for n in long_fns],
        "blank_line_violations": blanks,
        "lines": [(MAX_FILE_LINES + 1, "file-over-300-lines")] if len(lines) > MAX_FILE_LINES else [],
    }


NEW_FILE_LIMITS = {"comment_lines": 0, "lines": MAX_FILE_LINES, "long_functions": 1, "blank_line_violations": 0}
RATCHETED = ("comment_lines", "long_functions")


def main():
    args = sys.argv[1:]
    rebaseline = "--rebaseline" in args
    update = "--update" in args or rebaseline
    baseline = json.loads(BASELINE.read_text()) if BASELINE.exists() else {}
    files = list(aye_files())
    failed = False
    new_baseline = {}
    totals = {"comment_lines": 0, "over_300": 0, "lines_added": 0, "blank_added": 0}
    for rel in files:
        numbers, where = analyse(rel)
        totals["comment_lines"] += numbers["comment_lines"]
        totals["over_300"] += numbers["lines"] > MAX_FILE_LINES
        entry = baseline.get(rel)
        exempt = entry.get("exempt") if entry else None
        limits = {k: entry[k] for k in RATCHETED if k in entry} if entry else NEW_FILE_LIMITS
        if entry:
            totals["lines_added"] += max(0, numbers["lines"] - entry.get("lines", numbers["lines"]))
            totals["blank_added"] += max(0, numbers["blank_line_violations"] - entry.get("blank_line_violations", numbers["blank_line_violations"]))
        for key, limit in limits.items():
            if key == "long_functions" and exempt:
                continue
            if numbers[key] > limit:
                failed = True
                for line, rule in where[key]:
                    print(f"{rel}:{line}: {rule} ({key} {numbers[key]} > {limit})")
        if update and (entry or rebaseline):
            kept = dict(numbers)
            if entry and not rebaseline:
                for k in RATCHETED:
                    kept[k] = min(numbers[k], limits[k])
            if exempt:
                kept["exempt"] = exempt
            new_baseline[rel] = kept
    for rel in baseline:
        if rel not in files and not update:
            print(f"{rel}: in baseline but deleted (run --update)")
    if update:
        BASELINE.write_text(json.dumps(new_baseline, indent=1, sort_keys=True) + "\n")
        failed = False
    print(f"taste: {len(files)} files, {totals['comment_lines']} comment lines, {totals['over_300']} files over 300 lines, "
          f"baseline files grew by {totals['lines_added']} lines and {totals['blank_added']} blank-line violations (not ratcheted)")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
