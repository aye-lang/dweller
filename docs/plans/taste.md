# Plan: the T lane (code taste)

The CEO said on 2026-10-16 that the codebase lacks code taste; the rules
are `.claude/skills/code-taste/SKILL.md`. Measured that day on
`browser/`: 209 `.aye` files, 126 405 lines, 11 389 comment lines; 132
files over 300 lines, 38 over 1 000 (`engine/prelude.aye` 4 951,
`js/interp.aye` 4 696).

Rules of the lane:
- A T card changes structure only: behavior, test output and goldens stay
  the same. One module per card (split a large module by subdirectory).
- A T card takes only a module that no card in flight touches (check the
  open PRs and running agents first). Feature cards that touch a module
  after its T card follow the rules from then on.
- Every implementation prompt, in every lane, pastes the rules of the
  skill that apply to the code it writes.
- Before T1 is accepted by the CEO, no other T card runs: T1 is the
  template the others copy.

| Step | Card | What | Depends on | Status |
|---|---|---|---|---|
| T0 | T0 | `check_taste.py` with a baseline ratchet, in CI | – | open |
| T1 | T1 | pilot: `url/` by the rules, the template for the rest | – | open |
| T2 | T2 | `engine/prelude.aye` JavaScript into `.js` files with `@embedFile` | T1 | open |
| T3+ | T3… | one module per card, smallest first: `encoding`, `perf`, `image`, `webfont`, `font`, `net`, `html`, `dom`, `svg`, `css`, `layout`, `paint`, `gpu`, `audio`, `wasm`, `agent`, `ui`, `engine`, `js` | T1 | open |

#### T0: `check_taste.py`
- goal: the mechanical rules of the skill are checked in CI. Checks per `.aye` file under `browser/` (vendored and generated files excluded by a list in the script): comment lines (allowed: tool directives and one-line comments that hold a URL or a reference path starting `v8:`, `blink:`, `angle:`, `spec:`), line count ≤ 300, at most one function over 5 lines unless the file is one of the exemptions (one struct and its methods, an enum and its match table, a data table, a registry; marked in the baseline by hand), blank-line rules (rule 7). A baseline file `browser/tools/taste_baseline.json` records today's numbers per file; a file may not get worse than its baseline, a file not in the baseline must pass, and `--update` lowers the baseline after an improvement (never raises it).
- files: `browser/tools/check_taste.py`, `browser/tools/taste_baseline.json`, `.github/workflows/ci.yml` (one step on Ubuntu)
- verify: `python3 browser/tools/check_taste.py; echo $?` → 0 on main; a new file with a header comment fails with file:line (by hand, not committed); adding a line to `js/interp.aye` fails
- time box: 45 min; model: sonnet

#### T1: pilot on `url/`
- goal: `browser/url/url.aye` (733 lines) rewritten by every rule of the skill: no comments but reference links, one function per file, names that carry what the comments said, blank lines. The result is the template for the other T cards, and the CEO reviews it before they start. Find out and write in the done note: whether Aye accepts `-` in file names, whether methods of one struct may live in several files of a module, and what `aye fmt` keeps.
- files: `browser/url/`
- verify: `cargo test --release --test browser -- url aye_unit_tests > target/t1.log 2>&1; echo $?` → 0; `aye check browser`; no golden changes
- time box: 45 min; model: opus

#### T2: the prelude's JavaScript in `.js` files
- goal: `engine/prelude.aye` and the other `*_prelude.aye` files hold JavaScript in Aye string literals; move each into `.js` files under `browser/engine/js/` (one file per web API area), read with `@embedFile`, joined in the same order. Byte-for-byte the same script at run time.
- verify: `cargo test --release --test browser -- js_conformance jsdom > target/t2.log 2>&1; echo $?` → 0; the joined prelude equals the old one (a one-off check by hand)
- depends on: T1; time box: 45 min; model: sonnet
