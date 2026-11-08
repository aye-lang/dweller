#!/usr/bin/env python3
"""Files each crash of a `browser/tools/fuzz.py run --report` JSON as a GitHub
issue labelled `fuzz`, titled `fuzz TARGET: KEY` (KEY: the panic's file:line:col,
else the panic text, the signal or `timeout`), or comments on the open issue
with that title, so a repeat crash does not open a duplicate.

    GH_TOKEN=... python3 .github/scripts/fuzz-file-issues.py TARGET REPORT RUN_URL
"""
import json
import subprocess
import sys


def gh(*args):
    return subprocess.run(["gh", *args], check=True, capture_output=True, text=True).stdout


def file_crash(target, crash, run_url):
    title = f"fuzz {target}: {crash['key'][:100]}"
    name = crash["input"].rsplit("/", 1)[-1]
    body = (
        f"{crash['message']}\n\n{crash['hits']} hits. Minimized input `{name}` in the artifact `fuzz-crashes-{target}` of {run_url}.\n\n"
        f"Replay: put the input alone in a directory DIR, then `dweller fuzz {target} --seconds 0 --corpus DIR`."
    )
    found = json.loads(gh("issue", "list", "--label", "fuzz", "--state", "open", "--search", f'in:title "{crash["key"][:100]}"', "--json", "number,title"))
    same = [i["number"] for i in found if i["title"] == title]
    if same:
        gh("issue", "comment", str(same[0]), "--body", body)
        print(f"commented on #{same[0]}: {title}")
    else:
        print(gh("issue", "create", "--title", title, "--label", "fuzz", "--body", body).strip(), title)


def main():
    target, report, run_url = sys.argv[1:4]
    gh("label", "create", "fuzz", "--force", "--color", "D93F0B", "--description", "crash found by the nightly fuzz job")
    for crash in json.load(open(report)):
        file_crash(target, crash, run_url)


if __name__ == "__main__":
    main()
