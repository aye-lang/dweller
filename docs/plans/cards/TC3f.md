# TC3f: balance the browser test shards by measured time

- goal: make the shards of `browser-shard` finish at about the same time, so the slowest shard sets a lower wall time for a green `ci` run.
- cause: TC3b gives the sorted names to the shards round-robin, with no regard to how long each test takes. Its first run was badly unbalanced: Ubuntu shards 2.4 / 3.4 / 16.5 / 2.6 min, macOS 14.1 / 8.3 min. One test, `jsdom_gc_stress`, takes 908 s on Ubuntu and 674 s on macOS; the next longest are 113 s (`js_conformance`, Ubuntu) and 64 s (`window_container_queries`, macOS).
- design: `tests/browser-times.txt` has one line per test, `NAME UBUNTU_SECONDS MACOS_SECONDS`, rounded to whole seconds and sorted by name, from the `--report-time` lines of the six shard logs of that run (146 tests). It also has `jsdom_gc_stress_0` … `_3` with a quarter of the old test's time each, for the split in TC3e; the old line stays (it does nothing when the name does not exist). The step takes column 2 on Linux and column 3 on macOS (`RUNNER_OS`). It gives each listed name its time from the file (10 s when the name is not in the file), sorts by time, longest first, then by name (`LC_ALL=C sort -k1,1nr -k2,2`), and gives each name to the shard with the smallest total so far, the lowest shard index on a tie (awk). The order and the ties are fixed, so every shard computes the same assignment and keeps only its own names. Everything else from TC3b stays: an empty list fails the step, `--exact`, `RUSTC_BOOTSTRAP=1` with `-Z unstable-options --report-time`, the shard-0-only steps, the `browser` job and `perf`.
- files: `.github/workflows/ci.yml`, `tests/browser-times.txt` (new), `docs/plans/cards/TC3f.md` (new)
- verify: the YAML parses (`ruby -ryaml -e 'YAML.load_file(".github/workflows/ci.yml")'`); `python3 browser/tools/check_taste.py` passes; the exact pipeline of the YAML step, run on the real `--list` output for Ubuntu/4 and macOS/2, partitions the list (union = all names, no overlap), with today's names and with `jsdom_gc_stress_0` … `_3` in place of `jsdom_gc_stress`; the `ci` run of the PR shows the shard times.

## Status: done (local checks), 2026-11-01

- The YAML parses; `check_taste.py` passes.
- Partition check on the real list (146 names), with the pipeline text taken from the YAML (only `cargo test … --list` replaced by `cat` of the saved list): every case partitions the list.
- Predicted totals (sum of the per-test seconds of each shard; tests in a shard run in parallel threads, so the wall time is lower, but the longest single test is a floor):
  - today, Ubuntu 4: 908 / 529 / 529 / 529 s (shard 0, part 1/4, has only `jsdom_gc_stress`); macOS 2: 1558 / 1557 s.
  - after TC3e, Ubuntu 4: 624 / 624 / 624 / 623 s; macOS 2: 1557 / 1556 s.
- Until TC3e lands, `jsdom_gc_stress` alone sets the Ubuntu wall time (about 15 min).
- Not checked locally: the run on GitHub. When the times drift, regenerate the file from the `--report-time` lines of a green run.
