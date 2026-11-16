# DV6: one scheduled run fills the scoreboard

- goal: the CI cells of the scoreboard (`docs/plans/foundations.md`) are never older than a week, and no one measures them by hand.
- design: the weekly `conformance` workflow ends with a job `scoreboard` (`needs: [conformance, webgl]`, `if: always()`, read-only token: `contents`, `actions`, `issues`). It downloads the two result artifacts of its own run (each `continue-on-error`, so a job that died leaves the baselines) and appends `python3 browser/tools/scoreboard.py summary` to the run summary. It pushes nothing.
- `scoreboard.py summary` prints:
  - W9: areas at their floor, below, without a floor, and within 3.5 points of the floor (each named), from this run's results where there are any and from the committed baseline otherwise; then the lines of `conformance.py check` for the areas measured;
  - flip readiness (Q2.14a): passed over total of the newest finished `site-mode` run on `main` (its `site-mode-N` artifacts, `ok.txt` and `failed.txt`);
  - the open `flaky` and `fuzz` issues, by number and title;
  - S1 (DV7): green merge-queue runs of `ci` since the last flaky test;
  - W2 and W7: one line, not measured on CI until DV5.
- Q0.1 did not make the `webgl` job "allowed to fail" (its card): the scoreboard job runs whatever the two jobs end as.
- files: `.github/workflows/conformance.yml`, `browser/tools/scoreboard.py`, `docs/plans/cards/DV6.md`

## Status: done offline, 2026-11-16; the on-CI verify is pending

- offline (cloud VM): `scoreboard.py summary` with a stand-in `gh` on `PATH` that answers from responses recorded the same day through the GitHub API (105 merge-queue runs of `ci` with the artifact list of each, the `site-mode` runs, the open `flaky` and `fuzz` issues): exit 0, every section printed. W9 from the baselines and six areas measured here: 31 at their floor, 14 below, 43 without a floor, 11 within 3.5 points, no area fell. Issues: 0 and 0. S1: 22 (DV7's card).
- the flip-readiness line was checked for its arithmetic only: the stand-in writes `ok.txt` and `failed.txt` with 151 and 98 names over four shards (the count of 2026-11-12) and the line reads `151 of 249`. The real artifacts of run 43880764914 (2026-11-16, four `site-mode-N`, 848 to 1358 bytes) were listed, not downloaded.
- `ruby -ryaml` parses the workflow; `python3 -m py_compile` and `check_taste.py` pass. `actionlint` was not run: it could not be installed in this session.
- pending on CI: `gh workflow run conformance` on `main` (or Monday's schedule): the `scoreboard` job is green and its summary has the five sections. Two things only that run can show: `gh run download --pattern 'site-mode-*'` with the job's token (`actions: read`), and the artifact layout (`conformance-results` unpacks to `target/wpt-results` and `target/test262-results`; if it does not, the W9 line says "Measured in this run: 0").

Left:
- the on-CI run above, and `actionlint` on the workflow.
- "at target" in the W9 section, after Q0.3b.
- W2 and W7 by replay, after DV5.
- the scoreboard job is green whatever the cells say; it reports, it does not gate.
