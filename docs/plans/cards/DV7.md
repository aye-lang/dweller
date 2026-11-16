# DV7: S1 is a count

- goal: criterion S1 ("no flaky test in 50 consecutive `ci` runs") is a number a command prints.
- design: `browser/tools/scoreboard.py flaky` reads the finished `merge_group` runs of `ci`, newest first (`gh api`, 100 a page, at most 500), and counts the green ones up to the first run that had a flaky test. A run had one when it uploaded a `flaky-tests-*` artifact (`ci.yml`: a test failed, then passed on the re-run of its shard) or when it is green on an attempt after the first. A red or cancelled run without such an artifact is not counted and does not end the count: a red queue run is a real failure of that pull request. `flaky --all` lists every such run instead. The weekly scoreboard (DV6) prints the count.
- files: `browser/tools/scoreboard.py`, `docs/plans/cards/DV7.md`

## Status: done, 2026-11-16

- measured from the API responses recorded 2026-11-16 (all 105 merge-queue runs of `ci`, 2026-11-01 to 11-12: 97 green, 5 red, 3 cancelled; none has a second attempt), through a stand-in `gh`:
  - `scoreboard.py flaky`: `S1: 22 green merge-queue runs of ci since the last flaky test; last flaky: run 42475181353 (2026-11-08): flaky-tests-ubuntu-24.04-3.` So S1 stands at 22 of 50.
  - `scoreboard.py flaky --all`: 2 of 105: runs 42475181353 (green) and 42467354589 (cancelled), both 2026-11-08, both `flaky-tests-ubuntu-24.04-3`.
- the card's verify does not hold as written: the hand count of 2026-11-11 was four flaky uploads from 2026-11-04 to 11-09, this finds two. The hand count was over shard jobs of every `ci` run ("4 of 1 817 green shard jobs"), pull-request runs included; S1 here counts merge-queue runs only, as the design says. The other two were not looked up (the pull-request runs of those days were not recorded), so "they were on pull-request runs" is the likely reason, not a checked one.
- no `flaky` issue is open, though run 42475181353 was a merge-queue run and `ci.yml` files one for it: the issue was closed since, or the step failed (`continue-on-error`). Not checked.

Left:
- decide whether S1 counts merge-queue runs only (this card) or every `ci` run; with pull-request runs the count is lower and a branch's own flaky test would reset it.
- the two flaky uploads of 2026-11-04 to 11-09 that are not on merge-queue runs: find them once, to close the difference above.
