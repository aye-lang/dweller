# DV2: the macOS shards, balanced and under 9 minutes

- goal: the slower macOS shard runs its tests in under 9 minutes (it was 13.0, and the test step went from 7.1 to 14.8 minutes in five days).
- cause: `tests/browser-times.txt` had 146 names and a run executes 249 tests; 103 of them had no time and counted as 10 s each. Nothing refreshed the file.
- design, as built:
  - `browser/tools/test_times.py RUN_ID` (standard library) lists the run's `browser (OS, i/n)` jobs with `gh run view RUN_ID --json jobs`, reads each job's log with `gh run view --job ID --log`, and takes the `--report-time` lines `test NAME ... ok <1.234s>`. The system and the shard count come from the job name at the start of each log line. `--logs DIR` reads `*.log` files already downloaded instead of calling `gh`.
  - It rewrites `tests/browser-times.txt` in the same format: `NAME UBUNTU MACOS`, seconds rounded to the nearest second, sorted by name in byte order. NAME is now the full name the logs print (`agent::agent_http`), which `browser-shard.sh` matches first, so two modules with the same test name cannot collide. A test that ran on one system only keeps its old time (or 10) for the other; an old test that did not run is dropped; a test re-run after a failure counts with its last passing time.
  - `--split` prints the greedy split of `browser-shard.sh` (same sort, same "lightest shard first" rule) for the tests in the logs, with the old file and the new one, and the ten slowest macOS tests.
  - The integrator runs `python3 browser/tools/test_times.py RUN_ID --split` on a green merge-queue run when a batch adds tests, and commits the file.
- files: `browser/tools/test_times.py` (new), `tests/browser-times.txt`, `docs/plans/cards/DV2.md` (new)

## The refresh

Run 43156048023 (`merge_group`, 2026-11-12, green; 2 macOS and 4 Ubuntu shards).

`python3 browser/tools/test_times.py --logs target/dv2 --split`: 146 tests before, 249 after; 249 measured on Ubuntu, 249 on macOS; 0 times kept from the old file or defaulted; 0 old tests dropped (every old name matches a test of the run, by full or short name).

Sum of the `--report-time` seconds per shard (the shards run their tests on parallel threads, so a sum is not a wall time):

| | macOS 1 / 2 | Ubuntu 1 / 2 / 3 / 4 |
|---|---|---|
| predicted by the old file (103 tests at 10 s) | 1771 / 1770 | 723 / 723 / 722 / 722 |
| measured in the run (old split) | 2659 / 2669 | 422 / 489 / 446 / 472 |
| predicted by the new file | 2659 / 2659 | 455 / 455 / 455 / 454 |
| `finished in` of the run (wall) | 909 / 899 s | 111 / 126 / 161 / 131 s |

The script's own `awk` pipeline, run on the 249 names of the logs, gives the same sums as `--split` for both files, the same shard sizes as the run for the old file (108 / 141, 57 / 61 / 70 / 61), and no test at the default 10 with the new file (103 with the old one).

## The 9 minutes: not reached by the refresh

The macOS shards were already balanced by chance: measured 2659 / 2669 s, walls 909 / 899 s. The new file moves the sums to 2659 / 2659 and leaves the wall times where they are, about 15 minutes each. The shards run about 2.9 tests at a time (sum / wall). Under 9 minutes (540 s) on two shards needs the macOS sum near 3200 s, not 5318. A third shard alone gives about 600 s per shard, still over.

The ten slowest macOS tests (seconds macOS, Ubuntu):

| test | macOS | Ubuntu |
|---|---|---|
| `jsdom::jsdom_gc_stress` | 169.0 | 108 |
| `link_events::link_events` | 140.4 | 1 |
| `net::coop_popups` | 106.0 | 2 |
| `net::fetch_metadata` | 75.2 | 7 |
| `net::cert_bypass` | 73.2 | 9 |
| `media::http_failed_image_fetched_once` | 73.1 | 3 |
| `link_events::link_events_in_renderer` | 72.8 | 2 |
| `media::window_lazy_images` | 71.5 | 2 |
| `media::window_lazy_images_in_scroller` | 71.0 | 1 |
| `security::csp_script_src` | 70.9 | 1 |

Nine of the ten take under 10 s on Ubuntu and 70 to 140 s on macOS; six sit at 71 to 75 s. That looks like one macOS-only wait of about 70 s (a timeout or a process start), not slow work. Finding it is worth more than a third shard.

## Verify

- `python3 browser/tools/test_times.py --logs target/dv2 --split`: output above.
- The script's `awk` pipeline on the names of the logs, both files, both columns: sums above, 0 fallbacks with the new file.
- `bash -n .github/scripts/browser-shard.sh`: ok (unchanged). `python3 browser/tools/check_taste.py`: exit 0.
- The card's CI verify (the two macOS shards of the next green run differ by under 1 minute, the slower under 9) is for the integrator on the next queue run: the first half is predicted to hold (it already held), the second is predicted to fail.

## After DV2b (2026-11-22)

- the macOS test steps of today's green queue runs: run 894 (#388) 1/3 9 min 44 s; run 897 (#389) 7 min 48 s, 8 min 42 s and 5 min 23 s. The slowest shard is now set by `jsdom::jsdom_gc_stress` alone, 537 s on macOS (305 on Ubuntu): one test cannot be split over shards, and its pages already run in parallel inside it.
- change: the test is three tests, `jsdom_gc_stress_1` to `_3`, each with a third of the fixtures (the six slowest first, spread round robin; `_3` also runs code mode, P9.8). The slow list was stale: `frames_many` (118 s alone here) was not in it and could start last. Nothing is loosened: every fixture still runs under `DWELLER_JS_GC=stress` with the same check.
- times here (Linux x86_64, 4 cores, each part alone): 180, 136 and 146 s; the fixtures one by one: 874 s in all, the parts 324, 281 and 269 s of it. `tests/browser-times.txt` gets the three names with the old row's times split by these shares (Ubuntu 119, 90, 96; macOS 209, 158, 170) until the next refresh from a green queue run (`test_times.py RUN_ID`).
- expected: the slowest macOS shard under 9 minutes, its floor now about 3.5 minutes (the largest part) instead of 9.

- measured (queue run 45011774124, #392, the split's own run): macOS test steps 8 min 4 s, 6 min 32 s and 5 min 59 s (before: 9 min 44 s in run 894, 8 min 42 s in run 897); Ubuntu 5 min 21 s at most; the whole run 9 min.

## Status: done, 2026-11-22 (the slowest macOS shard 8 min 4 s; the shards within 2 minutes of each other)

The tool and the refreshed file are built; DV2b and the split above met the 9-minute goal.

Left:
- the next refresh of `tests/browser-times.txt` from a green queue run (`test_times.py RUN_ID`) replaces the split's estimated times with measured ones.

