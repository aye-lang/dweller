# TC3i: build the test browser with `--mode safe --leak-check`

- goal: make the CPU-bound browser tests faster without losing the leak check, by building the test browser with `-O2` and the General allocator's tracking.
- cause: `tests/browser.rs` built the browser with plain `aye build browser`, which is debug mode (`-O0`). Debug mode was kept because it is the leak check: the General allocator's tracking writes leak reports to stderr, and the tests require empty stderr. A CI experiment (closed draft PR #241, run 41233350745) showed every test passes on `--mode safe` (`-O2` + bounds and overflow checks), and the CPU-bound tests get about 3x faster (macOS `js_conformance` 124 → 41 s). Safe mode alone has no leak check. Aye PR #24 (TC3h, merged as 095390509a84e2f23cdaa6dad422d2ca06b01c31) adds `--leak-check`: the General allocator's tracking (leak report, double free, write after free) in any mode.
- design: pin `AYE_REV` to 095390509a84e2f23cdaa6dad422d2ca06b01c31, so CI installs an aye with `--leak-check` (the aye cache key in `ci.yml` hashes `AYE_REV`, so `ci.yml` is unchanged). `dweller()` in `tests/browser.rs` passes `--mode safe --leak-check` to `aye build browser`. The `aye test browser` call is unchanged.
- files: `AYE_REV`, `tests/browser.rs`, `docs/plans/cards/TC3i.md` (new)
- verify: with `AYE` set to an aye built at 0953905, `cargo test --release --test browser -- --exact js_conformance` and `-- --exact jsdom_fixtures` pass; `python3 browser/tools/check_taste.py` passes.

## Status: done (local checks), 2026-11-01

- `aye build --help` of 0953905 lists `--leak-check`.
- macOS, local, through `scripts/capped.sh`, each `finished in` includes the browser build:
  - `js_conformance`: ok, 15.67 s.
  - `jsdom_fixtures`: ok, 10.55 s.
- `check_taste.py` passes.
- Not checked locally: the whole suite and the shard times on GitHub. `tests/browser-times.txt` still holds the debug-mode times; refresh it from the first green `main` run with this change (as in TC3f2).
