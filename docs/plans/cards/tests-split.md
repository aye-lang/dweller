# tests-split: split `tests/browser.rs` into area modules

- goal: stop the merge-queue conflicts in `tests/browser.rs` (12 344 lines, 194 tests): every card appended its test at the same place.
- design: `tests/browser.rs` keeps the shared helpers and one `#[path = "browser/<area>.rs"] mod <area>;` pair per file, alphabetical. Each test moves unchanged, with its doc comment, into `tests/browser/<area>.rs` (`use super::*;`). A helper used by one area only moves with that area; a helper used by two or more stays in `tests/browser.rs`, so no visibility changes. Module names keep the filters: every test in a module contains its name (`agent`, `webgl`, `window_login`, `window_renderer`, `window_spaces`, `fixtures`), or the name has no `_` and is no test's full name (`cli`, `history`, `input`, `js`, `jsdom`, `media`, `net`, `paint`, `screenshot`, `scripts`, `security`, `storage`, `wpt`).
- sharding: `--list` now prints `area::name`. `.github/scripts/browser-shard.sh` looks the name up in `tests/browser-times.txt` as it is (the `perf::` keys), else with the module path stripped, and breaks ties by that same key, so every shard gets the same tests as before; `--exact` runs the full paths. `tests/browser-times.txt` is unchanged.
- files: `tests/browser.rs`, `tests/browser/*.rs` (new area files), `.github/scripts/browser-shard.sh`, `docs/browser.md`, `docs/practices/browser.md`, `docs/plans/cards/tests-split.md` (new)
- verify: the sorted bare names of `--list` equal those of `origin/main` (202); every full test name and every `_`-bounded part of a name selects the same tests as a filter; the shard pipeline on both lists gives the same tests per shard (Ubuntu 4, macOS 2, and 2/4 for each column); `cargo build --release --tests` has no warnings; a sample run passes.

## Status: done (local checks), 2026-11-09

- 202 tests before and after, same bare names.
- Filters: of 1814 filters made from test-name parts, the only ones that select more are bare module names (`net`, `js`, `paint`, ...), which now also select their file.
- Shards: identical for col 2 and col 3, 2 and 4 shards.
- Sample: `aye_unit_tests fixtures csp_script_src window_renderer_site_lock`, 10 passed.
- Flaky issues are now titled with the full path (`flaky: net::hsts`), so an open issue with a bare name is not matched again.
