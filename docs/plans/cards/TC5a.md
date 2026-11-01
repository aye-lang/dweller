# TC5a: merge queue for `main`

- goal: a green PR lands on `main` without anyone pressing merge, and `main` stays green, because every PR is tested merged with the latest `main` before it lands.
- cause: after CI got fast (`main` `ci` 8.2 min), most of the time from PR to `main` is the wait after green (p90 118 min: nobody merges). Also, a PR's CI tests its merge with `main` as it was when the run started; `main` can move before the PR lands, so a green PR can still break `main`.
- design: GitHub's merge queue on `main`. `ci.yml` also runs on `merge_group`, so the queue gets the required checks for each queue entry (the entry is the PR merged with the latest `main` and the entries ahead of it). The concurrency group is `ci-${{ github.event.pull_request.number || github.ref }}`; for `merge_group`, `github.ref` is the `refs/heads/gh-readonly-queue/main/pr-<N>-<sha>` ref, unique per entry, so queue runs do not cancel each other or the PR runs. The required checks are the `browser` aggregate job and `perf`; both run on every event. `angle` and `conformance` are not required. Merges stay merge commits (never squash, so chained branches do not conflict).
- settings: repository ruleset `main merge queue` (target branch, `~DEFAULT_BRANCH`, active):
  - `merge_queue`: merge method MERGE, grouping ALLGREEN, build up to 5 entries, merge 1 to 5 entries, wait at most 1 min for more entries, check timeout 60 min.
  - `required_status_checks`: `browser`, `perf`; not strict (the queue already tests against the latest `main`).
  - bypass: repository admin role, always (emergency fixes only).
  - repository setting `allow_auto_merge` on, so `--auto` queues a PR whose checks are still running.
- land a PR now: `gh pr merge <N> --repo aye-lang/dweller --merge --auto`. The PR joins the queue when its own checks are green, the queue runs `ci` on the merge with the latest `main`, and GitHub merges it when that run is green. Do not wait for CI before running the command. A failed queue run removes the PR from the queue; fix the branch and run the command again.
- files: `.github/workflows/ci.yml`, `docs/plans/cards/TC5a.md` (new)
- verify: `ruby -ryaml -e 'YAML.load_file(".github/workflows/ci.yml")'` and `python3 browser/tools/check_taste.py` pass; a second PR lands through the queue with `--auto`, a `merge_group` run of `ci` passes, and the PR lands as a merge commit.

## Status: in progress, 2026-11-01

- `ci.yml` runs on `merge_group`.
