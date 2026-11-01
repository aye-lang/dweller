# TC5b: re-run failed browser tests once, record flakes loudly

- goal: a flaky test does not kick a PR out of the merge queue, and every flake is still visible. A failed test is re-run once in the same job. Passes on the re-run: the job passes, and the flake is recorded as a warning, in the step summary, as an artifact and (on `push` and `merge_group`) as an issue. Fails again: the job fails as before. Never silent.
- cause: last month 20% of dweller `ci` run attempts failed; 12 of 92 failures were flaky (the same commit passed on a re-run). Since 2026-11-01 `main` has a merge queue (TC5a, required checks `browser` and `perf`), so a flaky failure removes a PR from the queue and someone must queue it again.
- design:
  - `.github/scripts/browser-shard.sh <shard> <shards>` holds the `browser-shard` test step: the shard selection from `tests/browser-times.txt` (unchanged), then the run with `tee` to `target/browser-tests.log`. On a non-zero exit it takes the names from `test NAME ... FAILED` lines. No names (build error, harness panic): exit 1 without a re-run. Else it re-runs exactly those names once with `--exact` and the same flags. The re-run fails: the script fails. The re-run passes: per name `::warning title=flaky test::NAME failed, then passed on re-run` and a line in `$GITHUB_STEP_SUMMARY`; the names go to `target/flaky-tests.txt`.
  - step `Upload flaky test names`: when `target/flaky-tests.txt` exists, `actions/upload-artifact@v4` as `flaky-tests-<os>-<shard>`.
  - step `File flaky test issues`: when the file exists and the event is not `pull_request` (no noise from WIP branches). `gh label create flaky --force` (creates it or keeps it), then per name the open issue titled exactly `flaky: NAME` gets a comment with the run URL, or a new issue with label `flaky` is created. `continue-on-error`, so an API error does not fail a job whose tests passed. `GH_TOKEN: ${{ github.token }}`.
  - job `browser-shard` `permissions`: `contents: read`, `issues: write` (there was no block before).
- files: `.github/workflows/ci.yml`, `.github/scripts/browser-shard.sh` (new), `docs/plans/cards/TC5b.md` (new)
- verify: `ruby -ryaml -e 'YAML.load_file(".github/workflows/ci.yml")'` and `python3 browser/tools/check_taste.py` pass. The script with a fake `cargo` on `PATH`: (a) one test fails, then passes: exit 0, warning printed, names file and summary line written; (b) fails twice: exit 101; (c) cargo fails with no `... FAILED` line: exit 1, no re-run. The issue step with a fake `gh`: a comment on the existing issue for one name, a new issue for the other. On CI: the PR's `ci` run is green and it lands through the merge queue.

## Status: in progress, 2026-11-01

- script, workflow steps and local fake-`cargo`/fake-`gh` checks done; PR open with `--auto`.
