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

## Status: in progress, 2026-11-12

Built and checked locally; waiting for the first real PR and queue run.

Left:
- The label `macos` does not exist in `aye-lang/dweller` yet: `gh label create macos --description "run the macOS shards and perf on this PR"`.
- No change in the ruleset: it requires `browser` and `perf`, and both keep reporting.
- On the first PR: the run shows 4 shards and `perf` as skipped, and the PR is mergeable (queue entry not blocked by a pending `perf`). In its queue run, 6 shards and `perf` run. After the merge, the push run has only `caches` (2 jobs).
- The 30-landing count and the deliberate macOS break (card verify).
