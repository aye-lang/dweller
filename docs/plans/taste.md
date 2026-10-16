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
| T1 | T1 | pilot: `url/` by the rules, the template for the rest | – | done 2026-10-16, awaiting CEO review |
| T1b | T1b | `Url` public fields `query`/`fragment` become `?[]u8`, callers updated | T1 | open |
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
- status: done 2026-10-16. Findings: (1) Aye accepts `-` in file names (`aye check`/`test`/`build` fine), so files are kebab-case (`read-authority.aye`); unit tests keep the `_test.aye` suffix (`percent-decode_test.aye`). (2) Methods of one struct may not live in several files: the spec (`aye spec structs`) says a struct's methods are all in its body by design, and other files take the struct as a parameter; so the rule-5 split of a long struct is free functions, not methods (no compiler gap to file). (3) `aye fmt` keeps the blank lines it finds, collapses a run of them to one and adds none (not even between top-level functions), so rule 7 has to be checked by `check_taste.py`. Before / after: 1 → 43 code files + 4 test files + `URL.md` (the module's supported subset and the `Url` field meanings, moved out of comments); 733 → 810 lines (blank lines and per-file imports); 56 → 13 comment lines (all `// spec:` links); largest file 733 → 52 lines (`parse_test.aye`; largest code file `add-segments.aye`, 44); no code file has two functions over 5 lines. Internal names now carry the comments (`Parts.opaque_path: ?[]u8`, `query`/`fragment: ?[]u8`, `schemeLength` and `hexValue` return `?T`, `trimControlsAndSpaces`, `copySchemeAndAuthority`, `portColon`); the serializer records offsets as `Span`s and lower-cases with `std.text.toLower`. The public `Url` fields keep their names (`scheme`, not `scheme_lower`; `has_query` + `query`) because callers use them; renaming them is a card of its own. Behavior: an old/new differential run over 352 parse/resolve cases printed every field byte-identically; `-- url aye_unit_tests` and `-- dump_fixtures js_conformance` pass; no golden changes.

#### T1b: `Url` query and fragment as `?[]u8`
- goal: rule 1 (`?T` for absence) on the public `Url` fields T1 left alone: `has_query` + `query` become `query: ?[]u8`, `has_fragment` + `fragment` become `fragment: ?[]u8`; every caller in `browser/` updated (`engine/history.aye`, `engine/window_api.aye`, `agent/actions.aye` and the rest `rg` finds); `URL.md` drops the sentence on the old names.
- files: `browser/url/`, the callers
- verify: `cargo test --release --test browser -- url aye_unit_tests dump_fixtures js_conformance > target/t1b.log 2>&1; echo $?` → 0; `aye check browser`; no golden changes
- depends on: T1; time box: 45 min; model: sonnet

#### T2: the prelude's JavaScript in `.js` files
- goal: `engine/prelude.aye` and the other `*_prelude.aye` files hold JavaScript in Aye string literals; move each into `.js` files under `browser/engine/js/` (one file per web API area), read with `@embedFile`, joined in the same order. Byte-for-byte the same script at run time.
- verify: `cargo test --release --test browser -- js_conformance jsdom > target/t2.log 2>&1; echo $?` → 0; the joined prelude equals the old one (a one-off check by hand)
- depends on: T1; time box: 45 min; model: sonnet
