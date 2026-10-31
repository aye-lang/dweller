# TC3f2: refresh the browser test times from run 41232362999

- goal: give the time-balanced shards of `browser-shard` (TC3f) times measured with the final setup, so the Ubuntu and macOS shards finish closer together.
- cause: `tests/browser-times.txt` came from the TC3b run, where one shard held the 15-min `jsdom_gc_stress` and the other times were inflated; the `jsdom_gc_stress_0` … `_3` lines were a quarter of the old test each (estimates). The first `main` run with TC3e and TC3f (run 41232362999, commit 3749645) still had unbalanced shards. Sum of the `--report-time` seconds per shard in that run: Ubuntu 592 / 1030 / 405 / 508 s, macOS 1470 / 1428 s; `finished in`: Ubuntu 199 / 320 / 164 / 237 s, macOS 500 / 295 s.
- design: regenerate the file from the `--report-time` lines of the six shard logs of that run: one line per test that ran, `NAME UBUNTU_SECONDS MACOS_SECONDS`, rounded to the nearest second, `LC_ALL=C` sorted by name. All 149 tests ran on both OSes. The old `jsdom_gc_stress` line is gone. `ci.yml` is unchanged.
- files: `tests/browser-times.txt`, `docs/plans/cards/TC3f2.md` (new)
- verify: the names in the file equal the names of `cargo test --release --test browser -- --list --format terse`; the assignment pipeline of `ci.yml` on that list gives balanced predicted sums for Ubuntu/4 and macOS/2; `python3 browser/tools/check_taste.py` passes.

## How to regenerate

Pick a green `main` run (here 41232362999) and run, in an empty directory:

```sh
RUN=41232362999
gh api --paginate /repos/aye-lang/dweller/actions/runs/$RUN/jobs \
  --jq '.jobs[] | select(.name | test("^browser \\(")) | "\(.id)\t\(.name)"' \
  | while IFS=$'\t' read id name; do
      os=$(echo "$name" | sed -E 's/.*\((ubuntu|macos).*/\1/')
      gh api /repos/aye-lang/dweller/actions/jobs/$id/logs > $os-$id.log
    done
awk 'FNR == 1 { os = FILENAME ~ /macos-/ ? 3 : 2 } / test [^ ]+ \.\.\. ok </ { t = $NF; gsub(/[<>s]/, "", t); v[$3, os] = sprintf("%d", t + 0.5); seen[$3] = 1 } END { for (n in seen) print n, v[n, 2], v[n, 3] }' ubuntu-*.log macos-*.log | LC_ALL=C sort > browser-times.txt
awk 'NF != 3' browser-times.txt
```

The last command prints the tests that are missing on one OS; give them the other OS's value by hand. Then copy `browser-times.txt` to `tests/browser-times.txt` and compare its names with `--list`.

## Status: done (local checks), 2026-11-01

- 149 tests in the logs (Ubuntu 34 + 36 + 44 + 35, macOS 61 + 88), each on both OSes; the names equal the local `--list` (149).
- Predicted sums (pipeline of `ci.yml` on the local list):
  - before, Ubuntu 4: 624 / 624 / 624 / 623 s; macOS 2: 1557 / 1556 s.
  - after, Ubuntu 4: 637 / 637 / 637 / 637 s; macOS 2: 1448 / 1448 s.
- `check_taste.py` passes.
- The macOS sums of the run were already close (1470 / 1428 s), but the wall times were 500 / 295 s. The sum does not model the parallel test threads; the long tests (`jsdom_gc_stress_2` 194 s, `_3` 181 s, `js_conformance` 124 s on macOS) and the start order set the wall time. New times alone may not close the macOS gap.
- Not checked locally: the shard times on GitHub.
