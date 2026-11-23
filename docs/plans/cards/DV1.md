# DV1: macOS only where it decides

- goal: a landed PR uses three macOS jobs, not nine, and the wait for a macOS runner on a busy day stays under 5 minutes at the median. On 11-08, 687 macOS jobs queued (median wait 9.8 min, p90 43).
- cause: `ci` ran all six shards and `perf` on `pull_request`, `merge_group` and `push` to `main`. The push run tests the tree the merge queue already tested.
- design, as built (`.github/workflows/ci.yml` only):
  - The required checks are the job names `browser` (a job that needs `browser-shard`, `if: always()`, and fails unless the shard result is `success`) and `perf` (a plain job). The ruleset "main merge queue" requires exactly `browser` and `perf`; the matrix names `browser (os, part)` are not required.
  - `browser-shard`: job `if: github.event_name != 'push'`. Its `include` is one expression, `fromJSON(format('[{0}{1}]', <macOS entries> or '', <Ubuntu entries>))`. The macOS entries are present when `github.event_name != 'pull_request'` or the PR has the label `macos`. Chosen over a setup job that writes the matrix: no extra job and no extra wait; the entries are the same six as before, now as JSON. A job-level `if` cannot read `matrix`, so a per-entry `if` was not possible.
  - `browser`: `if: always() && github.event_name != 'push'`. On a PR without the label it needs only the four Ubuntu shards, so it reports their result.
  - `perf`: `if: github.event_name == 'merge_group' || contains(github.event.pull_request.labels.*.name, 'macos')`. A job skipped by its own `if` reports success, so the required `perf` passes on a PR without the label. On `merge_group` `github.event.pull_request` is null and `contains` is false; the first operand decides.
  - `pull_request` gets `types: [opened, synchronize, reopened, labeled]`, so adding the label starts a run that includes macOS. Any label starts a run (and cancels the one in progress, by the concurrency group).
  - `caches` (new, `push` only, `ubuntu-24.04` and `macos-latest`): restores and saves `aye`, ANGLE and wasmtime with the same keys as the shards. A run restores caches of its own ref and of the default branch; a merge-queue run saves to its temporary `gh-readonly-queue/...` ref, which no other run reads. So without a run on `main`, every new PR and every queue run would build `aye` cold. On a hit every step is a no-op (the fetch scripts exit early; system packages and `cargo install` run only on an `aye` miss). It is one short macOS job per landed batch.
  - The flaky-issue step (`github.event_name != 'pull_request'`) now runs only in the merge queue, as the shards no longer run on push.
- files: `.github/workflows/ci.yml`, `docs/plans/cards/DV1.md` (new)

## What runs, by event

Read from the final YAML.

| event | `browser-shard` | `browser` | `perf` | `caches` | required `browser` | required `perf` |
|---|---|---|---|---|---|---|
| `pull_request`, no label `macos` | 4 Ubuntu | runs | skipped | skipped | result of 4 Ubuntu shards | success (skipped) |
| `pull_request`, label `macos` | 2 macOS + 4 Ubuntu | runs | runs | skipped | result of 6 shards | result of perf |
| `merge_group` | 2 macOS + 4 Ubuntu | runs | runs | skipped | result of 6 shards | result of perf |
| `push` to `main` | skipped | skipped | skipped | Ubuntu + macOS | not required on push (skipped) | not required on push (skipped) |

macOS jobs per landed PR: 3 in the queue (2 shards + `perf`), plus 1 short `caches` job per pushed batch; 0 on the PR unless labelled.

## Verify

- `ruby -ryaml -e 'YAML.load_file(".github/workflows/ci.yml"); puts :ok'`: `ok`.
- `actionlint .github/workflows/ci.yml`: one finding, SC2013 (info) in the "File flaky test issues" script; the same finding is on the file before this change. Nothing new.
- The matrix expression, evaluated by hand in Python for both values of the condition: 6 entries (2 macOS, 4 Ubuntu) and 4 entries (Ubuntu only), with the same `os`, `shard`, `shards`, `part` as before.
- The card's CI verify (30 landings, open-to-merge median, a deliberately broken macOS test leaves the queue) is for the integrator on the real runs.

## Verify on CI, 2026-11-23

35 landings since 48fa418b (#365 to #413), every `ci` run of each: its PR runs, its queue attempts and the push run of its merge.
- macOS jobs that ran, per landed PR: median 5, the same over the 20 most recent:
  - a PR without the label `macos`: 0 on the PR (`perf` skipped);
  - 4 in the queue: 3 macOS shards (DV2 split the suite into three) and `perf`; until #372, 3 (two shards and `perf`);
  - 1 `caches` on the push to `main`.

  The PRs with the label (#373, #378, #386, #388, #394) and those that went through the queue more than once ran more: at most 31 (#373, nine PR runs and four queue attempts). Before DV1 every landing ran the macOS shards and `perf` on its PR, in the queue and on the push: three times.
- wait for a macOS runner (job start minus job creation), 58 of the jobs:

  | days | jobs | median | p90 |
  |---|---|---|---|
  | all | 58 | 0.05 min | 0.13 min |
  | 2026-11-22, the busiest day (13 landings) | 16 | 0.03 min | 0.05 min |
  | 2026-11-23 | 25 | 0.05 min | 2.25 min |

  The longest wait, 6.8 min, was on 2026-11-23, when #407's queue run started 34 s after #409's. On 11-08: median 9.8 min, p90 43.
- open to merge: median 21.1 min over the 35, against 28.1 min over the 22 landings of 11-08 and 11-09.
- a macOS failure leaves the queue: of the 6 failed queue runs since 11-12, 5 had a failed macOS job (`browser (macos-latest, N/3)` or `perf`; runs 44654065728 #373, 44954054235 #378, 44959242524 and 44963994657 #379, 44990613216 #386). None of those PRs landed from that attempt; each landed from a later green one. These were real macOS failures, not a deliberate one; they show the same thing.
- runs read, the landing queue run of each PR: #365 43156048023, #366 43159305222, #367 43161465351, #370 43162986930, #371 43997007014, #372 44002006311, #373 44681296007, #377 44949382214, #378 44956488177, #379 44968179705, #382 44983721861, #384 44987415404, #386 44997685501, #388 44999876412, #389 45003848579, #390 45009974483, #392 45011774124, #393 45018289320, #394 45029414934, #395 45034360224, #396 45034594814, #397 45041506917, #399 45055982022, #400 45064413475, #401 45064791745, #402 45073595170, #403 45086385470, #405 45090101908, #406 45104311080, #408 45114959836, #410 45115277560, #409 45116820150, #407 45116882858, #411 45117848996, #413 45118634565. The push run lists `perf` as a skipped macOS job beside `caches`; the queue of a docs-only change lists `perf` skipped and runs no macOS job (#395, #396, #400, #401, #410).

## Status: done, 2026-11-23

Left:
- two queue runs that start together share the macOS runners: their shards wait for each other (up to 6.8 min on 2026-11-23). If that becomes common, a concurrency limit on the queue's macOS jobs is the next step, not more shards.
