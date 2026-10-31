# TC3e: split jsdom_gc_stress into four parts

- goal: take the one long test off the critical path of the sharded browser test step (TC3b), so the shards share its work.
- cause: after TC3b (PR #238, 6e93a60) the slowest test is `jsdom_gc_stress` (`tests/browser.rs`): 908 s on Ubuntu, 674 s on macOS (`--report-time`); every other test takes 113 s or less. It runs about 110 jsdom pages under `DWELLER_JS_GC=stress` through `common::run_queue`, and as one test it sits in one shard.
- design: the body moves into `fn jsdom_gc_stress(part: usize)`. After the same page list, HEAVY exclusion and SLOW-first sort, it keeps the pages whose index mod 4 is `part` (`skip(part).step_by(4)`). The env, the 600 s limit, the checks and the failure message are unchanged. The `run.jsonl` agent session check runs only in part 3 (one of the two parts with 27 pages). Four tests `jsdom_gc_stress_0` to `jsdom_gc_stress_3` each call it with their number.
- files: `tests/browser.rs`, `docs/plans/cards/TC3e.md` (new)
- verify: `cargo test --release --test browser -- --list --format terse` shows the four tests and not the old one; `cargo test --release --test browser -- --exact jsdom_gc_stress_1` passes; a python check over `browser/tests/jsdom` with the same exclusion and sort shows the four parts partition the old list; `python3 browser/tools/check_taste.py` passes.

## Status: done (local checks), 2026-11-01

- Partition check: the old list has 110 pages; the parts get 28, 28, 27, 27. The union equals the old list and no page is in two parts. Each part gets one SLOW page.
- `check_taste.py` passes.
- `--list --format terse` shows `jsdom_gc_stress_0` to `jsdom_gc_stress_3` and no `jsdom_gc_stress`.
- `jsdom_gc_stress_1` passes locally (macOS, `scripts/capped.sh`, 3 test workers): 155.8 s.
- Not checked locally: parts 0, 2 and 3, and the times on the CI shards.
