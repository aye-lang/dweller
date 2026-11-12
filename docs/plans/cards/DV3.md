# DV3: docs-only changes pass in under a minute

- goal: a PR that changes only `docs/**` and `*.md` files reports `browser` and `perf` green in under a minute, on the pull request and in the queue.
- cause: nine plan-only PRs since 11-01 ran six shards and `perf` each and took 19 to 112 minutes to land.
- design, as built (`.github/workflows/ci.yml` only):
  - New first job `changes` (`ubuntu-24.04`, `if: github.event_name != 'push'`, `contents: read`). Output `code`.
  - How the files are listed: with git, not the API.
    - `pull_request`: the checkout is the test merge commit `refs/pull/N/merge`. Its first parent is the base it was merged onto, so `git diff HEAD^1 HEAD` is exactly what the PR changes against that base (the same as a diff against the merge base, without a deep clone). `fetch-depth: 2` gets both commits. The step checks that `HEAD` has two parents and fails otherwise.
    - `merge_group`: `git fetch --depth=1 origin <merge_group.base_sha>`, then `git diff <base_sha> <head_sha>`. A tree diff needs the two commits, not the history between them. For a batch, `base_sha` is the base of the whole queue entry.
    - Why not `gh api .../pulls/N/files`: it covers only `pull_request` (the queue needs `compare`, which lists at most 300 files), it lists at most 3000 files, and it needs the `pull-requests` permission. The git way is the same two lines for both events and reads the exact tree that CI tests.
    - `git diff --no-renames --name-only`: a rename lists its old and new path, so moving `browser/x.aye` to `docs/x.md` counts as code.
  - Classification (step `Classify`): `code=false` when the list is non-empty and every line matches `^docs/` or `\.md$`; otherwise `code=true`. An empty list is code (fail closed). One awk program, POSIX.
  - `python3 browser/tools/check_plan.py` moved from Ubuntu shard 0 to `changes`, so it runs on every PR and queue run, docs-only or not. `check_taste.py` stays on shard 0: it reads only `browser/**/*.aye` and its baseline, so a docs-only change cannot break it.
  - `browser-shard`: `needs: changes`, `if: !cancelled() && github.event_name != 'push' && needs.changes.outputs.code != 'false'`. Chosen over keeping the jobs and skipping their steps: a job-level `if` starts no runner (six runners, two of them macOS, in the queue). `!cancelled()` so that the shards still run when `changes` failed (at the plan check after `code` was set, or before it, with `code` empty): a code PR with a broken plan still gets its test results.
  - `browser` (required): `needs: [changes, browser-shard]`, `if: always() && github.event_name != 'push'`. It passes only when `changes` succeeded and either `code` is `false` and the shards are `skipped`, or `code` is not `false` and the shards are `success`. So `skipped` is accepted only after a successful docs-only classification.
  - `perf` (required): `needs: changes`, `if: !cancelled() && (merge_group or label macos) && (changes did not succeed or code != 'false')`. Docs-only: skipped, which reports success (DV1's mechanism). If `changes` failed, `perf` runs where it would run anyway (queue, label) and its first step `test "$CHANGES" = success` fails it before the checkout.
  - No workflow-level path filters (they leave required checks pending).
- files: `.github/workflows/ci.yml`, `docs/plans/cards/DV3.md` (new)

## What runs, by event

Read from the final YAML. `caches` is unchanged (push only).

Docs-only change (`changes` passes, `code=false`):

| event | `changes` | `browser-shard` | `browser` | `perf` | required `browser` | required `perf` |
|---|---|---|---|---|---|---|
| `pull_request`, no label | runs (list, classify, plan check) | skipped | runs | skipped | success (`skipped` accepted) | success (skipped) |
| `pull_request`, label `macos` | runs | skipped | runs | skipped | success | success (skipped) |
| `merge_group` | runs | skipped | runs | skipped | success | success (skipped) |
| `push` to `main` | skipped | skipped | skipped | skipped | not required | not required |

Runners for a docs-only landing: 2 Ubuntu jobs on the PR and 2 in the queue (`changes`, `browser`), no macOS.

Code change (`code=true`, for example one `.aye` and one `.md` file):

| event | `changes` | `browser-shard` | `browser` | `perf` | required `browser` | required `perf` |
|---|---|---|---|---|---|---|
| `pull_request`, no label | runs | 4 Ubuntu | runs | skipped | result of 4 shards | success (skipped) |
| `pull_request`, label `macos` | runs | 2 macOS + 4 Ubuntu | runs | runs | result of 6 shards | result of perf |
| `merge_group` | runs | 2 macOS + 4 Ubuntu | runs | runs | result of 6 shards | result of perf |
| `push` to `main` | skipped | skipped | skipped | skipped | not required | not required |

`changes` fails (any event but push): the shards run if `code` is not `false`; `browser` fails; `perf` fails in the queue or with the label, and is skipped (success) on a PR without the label, where `browser` already blocks.

Cost for a code change: the shards and `perf` now wait for `changes` (a depth-2 checkout and the plan check, about 15 s).

## Verify

- `ruby -ryaml -e 'YAML.load_file(".github/workflows/ci.yml"); puts :ok'`: `ok`.
- `actionlint .github/workflows/ci.yml`: one finding, SC2013 (info) in the "File flaky test issues" script; it was there before. Nothing new.
- The `Classify` step, extracted from the YAML with Ruby and run on sample lists, with macOS awk and with mawk (Ubuntu 24.04 in a container, the runner's default awk); same answers:
  - `docs/plans/cards/X.md` → false; `README.md` → false; `browser/README.md` → false; `docs/img.png` → false
  - `docs/a.md` + `browser/css/x.aye` → true; `.github/workflows/ci.yml` → true; empty list → true; rename `browser/x.aye` → `docs/x.aye` (both paths listed) → true
- The `List the changed files` step, extracted from the YAML and run on local shallow clones (`--depth 2`) of a repository with a merge commit: `pull_request` lists only the PR's files (not the base's own new commits) and both paths of a rename; `merge_group` fetches the base by sha at depth 1 and lists the same; a non-merge `HEAD` on `pull_request` and an unknown event fail.
- `python3 browser/tools/check_plan.py; echo $?`: 0.
- The card's CI verify (a docs-only PR lands in under 5 minutes; a PR with one `.aye` and one `.md` file runs every shard) is for the integrator on the real runs.

## Status: in progress, 2026-11-12

Built and checked locally; waiting for the first real docs-only PR and the first code PR.

Left:
- First docs-only PR: `changes` prints the file list and `code=false`; the shard check shows as skipped; `browser` and `perf` are green and the PR enters the queue. In the queue run the same, and `perf` skipped counts as passing for the queue (DV1 saw this only on a PR). Time to land under 5 minutes (card verify).
- First code PR: `changes` prints `code=true`; the 4 Ubuntu shards start after it; in the queue, 6 shards and `perf` run. The `git fetch` of `base_sha` in the queue works with the checkout's token.
- A PR with one `.aye` and one `.md` file runs every shard (card verify).
