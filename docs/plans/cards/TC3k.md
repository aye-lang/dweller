# TC3k: `--until-idle` ends a `--frames` run when the page is settled

- goal: make the slow `window_*` tests faster on macOS (the CI critical path: test step about 7 min, `window_*` tests 46% of its test seconds) without changing what they check.
- cause: a `--frames N` run draws on every loop iteration and returns only when N frames are drawn (`browser/ui/window.aye`, `drawn >= frames`). An idle iteration waits 10 ms, so a run takes at least N × 10 ms, also after the event script ended and the page is quiet. Example: `window_history_steps` is 5 runs × `--frames 600`, so at least 30 s (33.4 s measured).
- design: a new flag `--until-idle`, valid only with `--frames N` (N stays the upper bound). With it, the loop returns through the `drawn >= frames` exit (so `--save`, `--state` and `--text` are still written) after the page is settled for 3 iterations in a row, after at least one drawn frame. `pageSettled` (`browser/ui/page-settled.aye`) is true when all of these are true:
  - the event script is done (`rest` is empty)
  - no `dirty` or `repaint`
  - the tab is not `early`. `stepping` is set only while `early` is set (`Tab.dropEarly` clears both), so `stepping` needs no separate check.
  - no network load is running (`net.loadsRunning`: fetch, XHR, scripts, style sheets, images, all from the loader)
  - no background image loads (`engine.imagesLoading`)
  - no WebGL frames pending (`engine.canvasFramesPending`)
  - no navigation pending (`engine.navigationPending`)
  - no timer (`engine.msUntilNextTimer` is null). This also covers busy workers (`workerWait`), a playing AudioContext, and tasks that wait for a load.
  - no requestAnimationFrame callback (`engine.animationFrameRequested`)

  A page with a timer or rAF loop that never ends still runs to N. The flag is opt-in, so runs without it do not change.
- files: `browser/ui/page-settled.aye` (new), `browser/ui/window.aye` (flag, usage text, loop wiring), `tests/browser.rs` (`WINDOW_USAGE`, the flag on four tests), `docs/plans/cards/TC3k.md` (new)
- verify: `cargo test --release --test browser -- --exact <name>` for each test, once before and 3 times with the flag; `python3 browser/tools/check_taste.py`; `aye check browser`.

## Status: done (local checks), 2026-11-01

macOS, local, through `scripts/capped.sh`, `--report-time`, one test thread. The before and after times do not include the browser build.

| test | before | with `--until-idle` (3 runs) |
|---|---|---|
| `window_history_steps` | 33.4 s | 8.7, 8.9, 8.9 s |
| `window_input_priority` | 5.2 s | 2.5, 2.5, 2.5 s |
| `window_lazy_images` | 4.4 s | 0.82, 0.81, 0.90 s |
| `window_lazy_images_in_scroller` | 4.8 s | 0.73, 0.74, 0.77 s |

- No `--until-idle` on `inserted_scripts`: it requires more than 300 loop iterations (`loops.len() > 300`), so it needs the full `--frames 400`. With the flag the run ended after 142 iterations and the test failed ("longest loop 7 ms of 142"). Without the flag it passes, about 7.5 s, the same as before.
- Sum of the four tests: 47.8 s before, about 13 s after.
- `check_taste.py` passes. `aye check browser` passes.
- Not checked locally: the whole suite and the CI shard times. `tests/browser-times.txt` still has the old times. Refresh it from the first green `main` run with this change.
