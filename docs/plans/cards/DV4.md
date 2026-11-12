# DV4: `check_plan.py`

- goal: rule 2 of `velocity.md` ("a status has one home") is checked, not remembered.
- design, as built: `browser/tools/check_plan.py`, standard library only, in the style of `check_layers.py`. `--root DIR` names the directory that holds `docs/plans` (default: the repository root). It prints `file:line: message` per violation and exits 1 if there is any, else 0. Then it prints one line per phase: open rows, done rows, and the top-level Left items in the cards of the open rows.
  - Id families checked:
    - `Qn.m` (with any letter or digit suffix): rows `| Qn.m |` in `foundations.md` (open) and `foundations-done.md` (done); cards `cards/Qn.m.md`.
    - `DVn`: rows in the "Cards" table of `velocity.md`; the row's Status column is its state (done or not); cards `cards/DVn.md`.
    - Not checked: the cards of older lanes (P4, P10, P11, TC, `tests-split`), whose rows are in `browser.md`; the V lane (`vision.md`, no cards in `cards/`); ids that `velocity.md` only mentions in prose (Q2.14a, Q0.3b and Q4.25 have rows in `foundations.md`).
  - A card's status is its first `## Status:` line (Q3.5b has two); it is done when the text after `Status:` starts with "done" ("done (local checks)" counts).
  - Violations:
    - a Q id is a row in both foundations files;
    - a card exists for an id with no row, and its parent step has no row either. A slice card without a row of its own is not a violation when the parent step has a row: Q1.4a, Q1.7b, Q2.4f, Q3.3b, Q3.3c, Q3.4c, Q3.9b, Q4.14b, Q4.16a, Q4.16b, Q4.16d, Q4.16f today (parent: the id without its letter suffix; Q1.10d2 tries Q1.10d, then Q1.10);
    - a card says done and its row is open (`foundations.md`, or a DV row whose Status column is not "done"), or the row is done and the card says it is not;
    - a card that has a row but no `## Status:` line (the card is the status's home);
    - a row in `foundations.md` longer than 600 characters (Python `len`, so characters, not bytes);
    - an id in the "Next, per lane" table of `foundations.md` that is done (a done row, a DV row marked done, or a done card).
  - Phases: one line per `## Qn:` heading of `foundations.md`, then DV. A row counts in the phase of its id (Q0.4a's row is in the Q4 table of `foundations-done.md` and counts in Q0).
- files: `browser/tools/check_plan.py` (new), `docs/plans/cards/DV4.md` (new). The CI step in `.github/workflows/ci.yml` is added by the integrator in the batch (another card is changing that file).
- verify:
  - `python3 browser/tools/check_plan.py; echo $?` on the tree at bd2e1695 plus this card: exit 1, seven violations: the Q2.12 row (773 characters) and the Q4.25 row (791), Q2.1 done in `foundations-done.md` while its card says "in review", the Q1.9 and Q4.13 cards with no `## Status:` line, and DV4 (this card says done; the `velocity.md` row says open, and "Next, per lane" names it). The integrator fixes them in the batch.
  - A copy of `docs/plans` under `target/dv4/ok` with those seven fixed by hand: exit 0.
  - The same copy with the Q0.2 row moved back from `foundations-done.md` into the Q0 table of `foundations.md`: exit 1, `docs/plans/foundations.md:212: Q0.2 is open here but docs/plans/cards/Q0.2.md:15 says done`.
  - A second copy with the Q1.9 row copied (not moved) into `foundations.md`, Q2.12c named in "Next, per lane" and a card `Q9.1.md` with no row: exit 1 with one line for each.
  - `python3 browser/tools/check_taste.py; echo $?`: 0 (it checks `.aye` files only).

## Status: done, 2026-11-12 (the ci.yml step lands with it in the batch)

Left:
- The CI step, for the integrator: in the "Code taste" group of Ubuntu shard 0 of `ci.yml`, `- name: Plan one-home rule` with `run: python3 browser/tools/check_plan.py`.
- The DV rows keep a Status column in `velocity.md`, which repeats the cards' status (rule 2 says no section repeats one). The tool treats the column as the DV row's state; if the DV rows later move into the foundations files, the DV branch of the tool goes.
- The "Done (rows in `foundations-done.md`, ...)" lines under each phase of `foundations.md` list done ids a second time. They are not checked; the phase lines of this tool can replace them.
- "Next, per lane" is checked for done ids only, not for ids with no row.
