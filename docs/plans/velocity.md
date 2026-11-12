# Plan: the DV lane (development speed)

Written 2026-11-11 from the history of this repository: git, pull
requests, CI runs and the plan files. The owner asked how Dweller can be
built faster, looking at the tools around the code, the process and the
plan, not at the browser's own code.

Goal: the done criteria of `foundations.md` hold sooner. The measure is
not steps closed per day. It is scoreboard cells that turn green
(`foundations.md`, "Scoreboard").

This lane continues the TC lane of aye-lang/aye
(`docs/plans/toolchain.md`, TC3 and TC5), which made one CI run fast.
What is slow now is around the run.

## What the history says

### Pace

| What | Number |
|---|---|
| Days with commits, 2026-10-22 to 2026-11-11 | 5 of 21 (11-01, 11-03, 11-04, 11-08, 11-09) |
| Q steps closed (cards by their status date) | 23 on 11-04, 58 on 11-08, 33 on 11-09 |
| Q rows: planned on 2026-11-04, now | 92, 153 |
| Q rows done, open | 103, 50 (this review adds three open rows) |
| Left items in the rows of done steps (counted by clause) | about 140; none is a row |

A working day closes 20 to 60 steps. The 50 open rows are two days by
that count. They are not two days of work, for the reasons below.

### Rows grow, and "done" leaves a tail

| Phase | Rows planned 11-04 | Rows now | Done | Left items in done rows |
|---|---:|---:|---:|---:|
| Q1 | 15 | 17 | 15 | 18 |
| Q2 | 14 | 34 | 31 | 49 |
| Q3 | 12 | 25 | 25 | 38 |
| Q4 | 12 | 35 | 25 | 23 |
| Q1 to Q4 | 53 | 111 | 96 | 128 |

A planned row became 2.1 rows, and each done row left 1.3 items. Q3 has
no open step and 38 Left items.

### Steps closed against criteria moved

| Criterion | Steps closed for it since 11-04 | Moved |
|---|---|---|
| W7 (9 of 20 sites) | 25 Q4 rows | no site changed side; the four worst are 11 to 44% better and still 1.05 to 12 times over (Q4.21) |
| S2, X1 | 31 Q2 rows | hold behind `DWELLER_PROCESS`; the default build is unchanged |
| X2 | 25 Q3 rows | "done", with floors set under the first score (`html/browsers/origin` 2.4%, floor 2) |
| W9 | 4 Q6 rows | 3 areas crossed their floor; 13 below |
| W2, W3 | – | last measured 2026-10-15 |

Q4.16e is the plain case: estimated 250 ms on bbc, measured 10 ms,
because an earlier step had already removed the cause (Q4.21).

### Landing

Merged pull requests, open to merge, in minutes:

| Day | PRs | Median | p90 | How they landed |
|---|---:|---:|---:|---|
| 11-03 | 11 | 30 | 52 | one PR per card |
| 11-04 | 21 | 69 | 121 | one PR per card, merge queue |
| 11-08 | 72 | 90 | 365 | one PR per card, merge queue, two hand-made merge trains (13 and 12 cards) |
| 11-09 | 10 | 17 | 62 | one branch, 1 to 3 cards per PR, merged after the PR's own run |

On 11-09 the same session landed 29 cards in 5.2 hours with ten PRs. On
11-08 it took 17 hours for 72 PRs, and a card waited 90 minutes at the
median.

Nine PRs since 11-01 changed only the plan. Each ran the whole `ci` and
took 19 to 112 minutes to land. One of them (#280) restored plan rows
that a merge had dropped.

Files changed by the most landings on `main` since 11-04 (59 landings):
`tests/browser.rs` 31 (split on 11-08), `ui/window.aye` 15,
`docs/plans/foundations.md` 14, the four `ipc/` message files 11 to 12
each, `docs/conformance.md` and `tools/conformance/history.csv` 10 each.

### CI (335 `ci` runs, 2026-11-04 to 11-09)

| What | Number |
|---|---|
| Green run, wall time, median: pull request, merge group, push | 22.6, 19.9, 14.8 min |
| The same for pull requests, by day: 11-04, 11-08, 11-09 | 17.5, 29.6, 16.5 min |
| macOS job, wait for a runner, median / p90 / max: 11-04 | 4.3 / 16 / 24 min |
| the same on 11-08 | 9.8 / 43 / 92 min |
| the same on 11-09 | 0.1 / 0.1 / 4 min |
| Ubuntu job, wait for a runner (1 352 jobs) | median 0, max 2.6 min |
| macOS shard, time running, median | 13.0 and 10.8 min |
| Ubuntu shards, time running, median | 2.6 to 4.3 min |
| The macOS test step, median: 11-04, 11-08, 11-09 | 7.1, 9.1, 14.8 min |
| Tests a run executes: 11-04, 11-09 | about 150, about 231 |
| Names in `tests/browser-times.txt` | 146 |
| Runs on 11-08: pull request, merge group, push | 120, 59, 50 |
| Push runs cancelled by the next push (all days) | 44 of 83 |
| Job minutes on 11-08: macOS, Ubuntu | 3 859, 2 643 |
| Failed runs | 16 of 335; 27 of 28 failed jobs are real test failures, none is setup |
| Shard jobs that needed the flaky re-run | 4 of 1 817 |

A landed PR runs `ci` three times: on the pull request, in the merge
queue, and again on the push to `main`. Each run has three macOS jobs,
so one landing asks for nine macOS runners. On 11-08 that was 687 macOS
jobs, and a third of them waited more than 20 minutes for a runner. The
77 runs that took over 30 minutes had a median macOS wait of 38 minutes.

One test with a 10 s CPU limit (#315, `aye_unit_tests`) failed in the
merge queue twice and took #320 and #321 out with it. Each failure took
55 to 67 minutes to report.

The weekly `conformance` workflow has never passed on its schedule
(the last three scheduled runs: 10-26, 11-02, 11-09). The nightly `fuzz` workflow is green (three
runs).

### The plan files

| What | Number |
|---|---|
| `foundations.md` before 2026-11-11 | 64 KB, about 33 000 tokens; one read of a file stops at 25 000 |
| Longest table row | 1 158 characters |
| Places that hold one step's status | 3: its row, its card, the "State" section |
| `browser.md` | 510 KB; its running order is replaced, its cards are still cited |

Drift found on 2026-11-11:
- Q1.1 was listed open; its verify holds on `main`.
- `cards/TC5a.md` and `cards/TC5b.md` said "in progress" ten days after
  they landed.
- Q6.2's score is 47.9 in its row and 47.2 in its card and in
  `docs/conformance.md`; Q6.1 is 65.4 in its row and 65.1 in the table.
- Q1.3, Q1.6, Q1.7, Q1.10a, Q2.4b, Q2.4d and Q3.13 were done in their
  cards with no mark in their rows.

## Findings

1. **Writing the code is not the limit.** A day closes 20 to 60 steps.
   Work ran on 5 of the last 21 days. More agents in a day does not
   help; more days, and days that end green, do.
2. **Steps close and criteria stay.** A step is closed against its own
   verify, and no one asks whether a scoreboard cell moved. W7 is the
   clear case: 25 rows, no site gained. The plan had no place where
   that showed.
3. **"Done" hides the rest.** About 140 Left items sit inside done rows.
   The flip to site isolation looked three steps away; the readiness
   list in `foundations.md` names 16 more items that a person or a login
   meets.
4. **Landing is limited by macOS runners and by shared files, not by
   test time.** Nine macOS jobs per landed PR, and one PR per card, made
   the queue of 11-08. With the batches of 11-09 a card reached `main`
   five times sooner at the median, with the same tests.
5. **One machine and one person verify half the criteria.** W2, W3, W7,
   W8, M1 and S4 can only be measured on the owner's Mac, because the
   site archives live there. W3 is 27 days old. A cloud session cannot
   tell whether its perf step helped a site.
6. **The plan cost time to keep.** Three homes for a status, plan-only
   PRs through the whole `ci`, a file longer than one read.

## Rules (from 2026-11-11)

These replace "one card and one PR" in `foundations.md`.

1. **Land in batches.** Agents work in worktrees. One integrator merges
   finished cards into one wave branch, runs the tests the cards name
   plus `check_taste.py`, `check_layers.py` and `aye check browser`, and
   opens one PR per batch: at most three cards, or what is ready after
   30 minutes. It lands through the merge queue with `--auto`. A card
   gets its own PR only when no other card is in flight.
2. **A status has one home.** A card holds its status, done note and
   Left list. `foundations.md` holds open rows; a closed row moves to
   `foundations-done.md` in the batch that lands the step. No section
   repeats a status. Agents do not edit the plan files; the integrator
   does, once per batch, in the batch's PR. There is no plan-only PR
   while a batch is open.
3. **A Left item is triaged when its step closes.** The integrator puts
   each one in one of three places: a new row (it is needed for a done
   criterion), the card's Left list (it is not), or dropped with a
   reason. A phase's exit is checked against its criterion, not against
   an empty table.
4. **A row is sized before it is dispatched.** A row that names more
   than one thing gets a card that lists its slices first. Each slice
   has its own verify and fits the 45-minute box. Plan with the measured
   factor: one planned row is about two slices.
5. **A step names the scoreboard cell it moves, and by how much.** A
   step for W7, W8 or W9 says which site or area and the expected
   change. When three steps in a row for one cell miss their estimate,
   the lane stops and measures again (as Q4.21 did) before the next
   step.
6. **The Mac queue.** Work that only the owner's Mac can verify goes to
   the list in `foundations.md`. No cloud step waits on it; the owner
   clears the list in one sitting with `scripts/capped.sh`.
7. **Shared lists are the integrator's.** IPC message codes are given to
   a card before it starts (the card names its range). `docs/conformance.md`
   and `tools/conformance/history.csv` are written by the integrator
   once per batch; a card changes only its area's JSON file.
8. **A test with a time or CPU limit is run under load before it joins
   a batch** (the six CPU hogs of Q1.3, three runs).

## Cards

| Step | What | Depends on | Status |
|---|---|---|---|
| DV1 | `ci` asks for macOS only where it decides: one macOS run per landing, not three | – | in progress (landed 2026-11-12; counted over the next 30 landings) |
| DV2 | The macOS shards are balanced again and under 9 minutes | – | in progress (2026-11-12: every test has a measured time and the shards are balanced, but each takes about 15 minutes; the rest is DV2b) |
| DV2b | The macOS-only wait in the slowest tests is found and removed: six tests take 71 to 75 s and `link_events` 140 s on macOS, and under 10 s on Ubuntu (DV2's card). Needs the Mac | DV2 | open |
| DV3 | A docs-only change passes the required checks in under a minute | DV1 | done |
| DV4 | `check_plan.py`: the plan's one-home rule, checked in CI | – | done |
| DV5 | The site archives are readable off the Mac; W2 and W7 by replay run in the cloud | the owner (decision 6) | open |
| DV6 | One scheduled run fills the CI cells of the scoreboard | Q0.1, Q2.14a | open |
| DV7 | S1 is counted: consecutive green merge-queue runs | DV6 | open |
| DV8 | A wave can start without a session | DV1, DV4; the owner | open |

Q2.14a (the suite with `DWELLER_PROCESS=site`), Q0.3b (targets beside
floors) and Q4.25 (W7 with a renderer per site) come from the same
review; their rows are in `foundations.md`.

#### DV1: macOS only where it decides
- goal: a landed PR uses three macOS jobs, not nine, and the wait for a macOS runner on a busy day stays under 5 minutes at the median.
- cause: `ci` runs on `pull_request`, `merge_group` and `push` to `main`. The merge queue already tested the exact tree that the push run tests again; 44 of 83 push runs were cancelled by the next push. On 11-08, 687 macOS jobs queued for the few runners the account gets at once (median wait 9.8 min, p90 43).
- design: in `.github/workflows/ci.yml`:
  - `push` to `main` no longer runs the shards or `perf`. If the caches (`aye`, ANGLE, wasmtime) need a run on `main` to be shared with PR branches, a small job on `push` only restores and saves them.
  - on `pull_request` the matrix has the four Ubuntu shards only; the macOS shards and `perf` run when the PR has the label `macos` (for a card that touches platform code).
  - on `merge_group` everything runs, as now. The required checks stay `browser` and `perf`: on a pull request `perf` reports success without running, by a job-level condition that keeps the check name.
  - a batch lands through the queue (rule 1), so every landing still passes macOS before `main` moves.
- files: `.github/workflows/ci.yml`, `docs/plans/cards/DV1.md`
- verify: over the next 30 landings, macOS jobs per landed PR is 3; open-to-merge median under 25 min with the queue (it was 17 on 11-09 without the queue and 90 on 11-08 with it); a PR that breaks a macOS-only test is put out of the queue and `main` stays green (tried once on purpose with a draft PR).
- risk: a macOS failure shows one run later, in the queue. The label covers cards that know they touch macOS code.
- time box: 45 min

#### DV2: the macOS shards, balanced and under 9 minutes
- goal: the slower macOS shard runs its tests in under 9 minutes (it is 13.0 now, and the test step went from 7.1 to 14.8 minutes in five days).
- cause: `tests/browser-times.txt` has 146 names and a run executes about 231 tests; a test without a time counts as 10 s, so the 85 or so new tests (most are `window_renderer*`, which wait on processes) are spread blind. Nothing refreshes the file.
- design: `browser/tools/test_times.py` reads the `--report-time` lines of the six shard logs of one green `main` run (run id as its argument, logs through `gh run view --log`) and rewrites `tests/browser-times.txt`. The integrator runs it when a batch adds tests. If the slower shard is still over 9 minutes after the refresh, list the ten slowest macOS tests with what they wait on (TC3j's method) before adding a third macOS shard, because a third shard costs the runners DV1 saves.
- files: `browser/tools/test_times.py`, `tests/browser-times.txt`, `docs/plans/cards/DV2.md`
- verify: the two macOS shards of the next green run differ by under 1 minute and the slower is under 9; the card lists the ten slowest tests.
- time box: 45 min

#### DV3: docs-only changes pass in under a minute
- goal: a PR that changes only `docs/**` and `*.md` files reports `browser` and `perf` green in under a minute, on the pull request and in the queue.
- cause: nine plan-only PRs since 11-01 ran six shards and `perf` each and took 19 to 112 minutes to land. Rule 2 removes most of them; card files and practices entries still land alone at times.
- design: a first job `changes` lists the changed files (`git diff --name-only` against the merge base, or the queue entry's base) and sets `code=false` when every file is under `docs/` or ends in `.md`. The shard and `perf` jobs keep their names and skip their steps when `code` is false, so the required checks still report. Path filters on the workflow would leave the required checks pending, so they are not used.
- files: `.github/workflows/ci.yml`, `docs/plans/cards/DV3.md`
- verify: a docs-only PR lands through the queue in under 5 minutes; a PR that changes one `.aye` file and one `.md` file runs every shard.
- depends on: DV1 (same file)
- time box: 45 min

#### DV4: `check_plan.py`
- goal: rule 2 is checked, not remembered.
- design: `browser/tools/check_plan.py` (standard library), in the "Code taste" group of steps on Ubuntu shard 0. It fails with file:line when:
  - a step id is a row in both `foundations.md` and `foundations-done.md`, or in neither while a card for it exists;
  - a card says `## Status: done` and its row is still in `foundations.md`, or the other way round;
  - a row in `foundations.md` is longer than 600 characters (the detail belongs in the card);
  - "Next, per lane" names a step that is done.
  It prints one line per phase (open, done, Left items in the open cards), which the integrator pastes nowhere: the numbers are read from the tool.
- files: `browser/tools/check_plan.py`, `.github/workflows/ci.yml`, `docs/plans/cards/DV4.md`
- verify: `python3 browser/tools/check_plan.py; echo $?` → 0 on `main`; moving a done row back makes it exit 1 with the line (by hand, not committed).
- note: Q2.12's row is over 600 characters today (its first slice's note); the card that closes it shortens it.
- time box: 45 min

#### DV5: the archives off the Mac
- goal: a cloud session and one scheduled CI job can run `page_load.py check --agent` (W2) and `window_perf.py report --replay` (W7), so a perf or site step learns in its own session whether a site moved.
- cause: the 20 recorded sites live in `target/sites/` of the owner's checkout, untracked (they are other people's pages and cannot go in a public repository). Every W2, W3, W7 and W8 number since 2026-10-15 came from that one machine, under `capped.sh`, one job at a time.
- design: the owner makes a private store (a private repository or a bucket) and a read token as an environment secret. `page_load.py fetch-archives` downloads one dated set into `target/sites/` and checks a hash list that is in this repository (`browser/tools/sites.lock`: site, date, sha256). Then one measurement, once: the 20 W7 rows on a cloud VM and on an Ubuntu runner beside the Mac's rows of the same commit. If the pass/fail side of each site agrees, the cloud number is the working number and the Mac's is the acceptance number. If not, the cloud reports counts only (`style_full`, `layout_full`, forced layouts, script instructions), which do not depend on the machine.
- files: `browser/tools/page_load.py`, `browser/tools/sites.lock`, `docs/plans/cards/DV5.md`
- verify: in a cloud session, `python3 browser/tools/page_load.py fetch-archives && python3 browser/tools/page_load.py check --agent; echo $?` → 0; the three-machine W7 table is in the card.
- depends on: the owner (decision 6 of `foundations.md`): the store and the token.
- time box: 45 min, then 45 min for the table

#### DV6: one scheduled run fills the scoreboard
- goal: the CI cells of the scoreboard are never older than a week, and no one measures them by hand.
- cause: the weekly `conformance` workflow has failed on every schedule (its `webgl` job; on 11-09 the `conformance` job too), so the table is refreshed only when a step re-runs an area. Q0 was a phase of re-measuring by hand, and two of its steps are still open.
- design: finish Q0.1 first (the `webgl` job allowed to fail, with its own baseline). Then the weekly workflow ends with a `scoreboard` job that writes to the run summary: areas at floor and below (and at target, after Q0.3b), the Q2.14a count, open `flaky` and `fuzz` issues, the S1 count (DV7), and with DV5 the W2 result and the W7 rows by replay. The integrator copies the summary into "Scoreboard" at the first batch of a wave. The job does not push to `main`.
- files: `.github/workflows/conformance.yml`, `browser/tools/scoreboard.py`, `docs/plans/cards/DV6.md`
- verify: one scheduled run is green and its summary has every CI cell of the scoreboard.
- depends on: Q0.1, Q2.14a
- time box: 45 min

#### DV7: S1 is a count
- goal: criterion S1 ("no flaky test in 50 consecutive `ci` runs") is a number a command prints.
- cause: its command today is "`gh issue list --label flaky` is empty", which says nothing about how many runs passed since the last flake.
- design: `browser/tools/scoreboard.py flaky` lists the `merge_group` runs of `ci` newest first and counts back to the first run with a `flaky-tests-*` artifact or a failed attempt that passed on re-run.
- files: `browser/tools/scoreboard.py`, `docs/plans/cards/DV7.md`
- verify: the count for 2026-11-04 to 11-09 matches the four flaky uploads found by hand on 2026-11-11.
- depends on: DV6
- time box: 30 min

#### DV8: a wave without a session
- goal: work goes on, on the days nobody starts a session, for the steps that need neither the Mac nor a decision.
- cause: work ran on 5 of the last 21 days. On those days the limit was landing, not agents.
- design: a scheduled cloud routine. It reads "Next, per lane" and the open rows, takes at most six steps that are marked verifiable in the cloud and wait on nothing in "Waiting on the owner", runs them under rules 1 to 5, lands one or two batches through the queue, moves the closed rows, and stops. It never takes a Mac-queue item, never changes a floor or a decision, and stops at the first red queue run with a report. The owner sets how often it runs and how many steps it may take; it is off until then.
- files: none in this repository beyond the plan; the routine's prompt is kept in `docs/plans/cards/DV8.md`
- verify: one unattended wave lands at least three steps with a green queue and a correct plan (`check_plan.py`), and its report names what it did not take and why.
- depends on: DV1, DV4; the owner's decision on cost.
- time box: 45 min for the first wave's setup

## What is left, counted honestly

By rows, with the measured factor of two slices per row:

| Phase | Open rows | Slices, estimate | Needs the Mac |
|---|---:|---:|---|
| Q0, Q1 | 5 | 6 | Q0.3, Q1.11 |
| Q2 | 4, and 16 readiness items | 25 | Q2.13 |
| Q4 | 11 | not known until Q4.25 | every W7 number |
| Q5 | 18 | 40 to 60 | 7 rows |
| Q6 | 11 | 25 | none |
| Q7 | 4 | not known: Q7.2 turns the owner's daily use into cards | all |

That is 100 to 120 slices that can be counted, or three to five working
days at the pace of 11-08 and 11-09. The calendar will be longer than
that for three reasons, in this order: the days between sessions, the
Mac queue, and the two phases that cannot be counted yet (Q4 and Q7).
The rules and cards above aim at those three, not at the pace of a
working day.
