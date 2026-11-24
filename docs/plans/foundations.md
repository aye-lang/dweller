# Plan: foundations, measured against Chromium (2026-11-04)

Goal: the user browses daily in Dweller, logged in, and it is safe, it
does not lag, it does not crash, and the basic features work as in
Chromium.

This plan sets the order of work from 2026-11-04. It replaces the
running order of "Roadmap: phases and steps (2026-10-15 evening)" in
`docs/plans/browser.md`. The cards and the Chromium module map there
stay valid, and the steps below cite them. WebGL, WebAssembly, Web Audio
and the game (Phase G there) move to the end (Q8).

Reference: the Chromium checkout at
`/Users/solemnis/Documents/chromium/src` (M148, 2026-07-10, read-only).
Paths below are relative to it.

## Start here (2026-11-11)

How to read this plan:
- This file holds the open steps only. A done step's row is in
  `foundations-done.md`, and its done note and Left list are in its card
  (`cards/<id>.md`). The two "State" sections of 2026-11-04 and
  2026-11-09 are in `foundations-done.md` too.
- A step's status has one home: its card once it has one, else its row
  here. Nothing else repeats it.
- How work lands, how a step is sized, and the cards that make the loop
  faster are in `velocity.md`.
- The items marked "proposed" below come from the review of 2026-11-11
  (`velocity.md`). They are not decided until the owner answers the
  matching line of "Waiting on the owner".

### Scoreboard

One row per done criterion. "Where" says which machine can measure it:
that decides who can close it.

| Criterion | Now | Measured | Where | What is missing |
|---|---|---|---|---|
| S1 no flaky test in 50 runs | 1 green merge-queue run of `ci` since the last one with a flaky test (45041506917, macOS shard, #398); 13 of the last 130 queue runs had one. Open `flaky` issues: #398, #385, #380, each seen only on macOS (the Mac queue) | 2026-11-22 (`scoreboard.py flaky`) | CI | 50 green queue runs in a row; the three macOS issues |
| The debugging mode (Q2.14) | the suite runs in site mode, the default since Q2.14: 390 / 390 (#399's queue run 45055982022); `DWELLER_PROCESS=single` 390 / 390 (`single-mode` run 45056889555) | 2026-11-22 | CI (the daily `single-mode` job) | – |
| S2 a tab crash stays in the tab | holds in the default build: `window_renderer_hang`, `window_renderer_sad_tab` (Q2.5) pass in site mode, queue run 45055982022 | 2026-11-22 | CI | – |
| S3 fuzz nightly, no open crash | 6 of 6 targets green in runs 45042466169 and 45056892424 (`main` be60dbd6 and bfa2c365), after the JS replay hang was fixed (#376, closed); the scheduled runs of 11-20 and 11-22 failed on that hang. No open `fuzz` issue | 2026-11-22 | CI | a scheduled run after the fix (the next is 11-23) |
| S4 1 h soak | no test | – | the Mac (archives) | Q1.11 |
| X1 a compromised renderer reads nothing | holds in the default build: the Q2.7, Q2.8a, Q2.8b tests (`window_renderer_site_lock`, `window_renderer_sandbox`, `window_spaces_renderer_sandboxed`, `relay_snapshot`) pass in site mode, queue run 45055982022 | 2026-11-22 | CI | – |
| X2 web security areas at threshold | 5 of 5 areas at their target: `content-security-policy` 81.0% (target 81, 3341/4127), `mixed-content` 96.3% (89, 2196/2281), `x-frame-options` 100% (90), `html/browsers/origin` 46.2% (40, 312/676), `html/browsers/sandboxing` 65.6% (64, 21/32) | 2026-11-22 (`conformance` run 45078360529; `docs/conformance.md` of 2026-11-23, Q0.3c) | CI, cloud | – |
| X3 bad TLS refused | holds: `tls_errors`, `cert_bypass` (Q3.4, Q3.4b) pass, queue run 45055982022 | 2026-11-22 | CI | – |
| W7 interactivity, 20 sites | 9 of 20 with the fast build, 8 of 20 with the safe build that ships; the 4 worst re-measured 2026-11-08: none passes, 1.05× (react) to 12× (mistral) over the loop budget | 2026-11-04 | the Mac only | never measured with site isolation; decision 2 |
| W8 first paint, 20 sites | 18 of 20 in site mode (19 and 18 in two runs): github fails both (2.1–2.6 s, Chrome 0.54–0.59 s; 2.6 s with `DWELLER_PROCESS=single` too), zol fails 1 of 2 (gap 320 and 1097 ms; its document no longer times out) | 2026-11-25 (site mode) | the Mac only (live, Chrome) | github: scripts and a sheet wait run before the first frame; zol: the window answers image fetches synchronously and the document waits; Q4.12 |
| M1 memory per tab | no budget | – | the Mac | Q2.13 |
| F1 basic features | 17 of 18 (`check_plan.py`: Q5 1 open) | 2026-11-22 | per row | Q5.9b (YouTube, the Mac) |
| W9 conformance | 47 areas at their floor, 0 below, 51 without a floor; 10 of the 47 within 3.5 points of it; 9 of the 20 areas with a target at it; `conformance.py check`: no area fell (98 of 98 areas measured in the run, 59 in its `conformance` job, 39 in its `webgl` job) | 2026-11-22 (`conformance` run 45078360529; `docs/conformance.md` of 2026-11-23, Q0.3c) | cloud; the weekly CI job, green again since #399 fetched libwasmtime for it | Q6, Q0.1 |
| W2 sites load and agents work | passed | 2026-10-15 | the Mac (archives) | Q0.3 |
| W3 sites look right | 7 of 20 (9 if a font-face swap alone does not count: hn, go-docs). Since 10-15 fixed: python-docs sidebar (P3.68), BBC wordmark (P3.59), react thumbnail (P3.72), Mistral nav logo | 2026-11-25 | the Mac: compared with Chrome by the Mac session; screenshots (live Dweller and Chrome, same 1280 × 2000 viewport, light scheme; archive screens as reference) | top causes by sites: vertical centring in flex rows and buttons (5: github, duckduckgo, youtube, youtube-search, react; P3.51); row layout that wraps, stacks or drops items (4: github, mdn, abehiroshi, zol); script-driven parts that differ (4: mdn, youtube-search, react, mistral); local font faces not matched, Verdana and Menlo become Arial and Courier New (3: hn, go-docs, zol); `sup` not raised (2: both Wikipedias) |

### Critical path

1. **Q2.14, the default flip.** S2, X1, M1 and the final W7 and W8
   numbers hold only after it, and all of Q5 waits for it. Nothing
   measured in one process counts as done.
2. Before the flip, in this order: Q2.14a (measure how much of the suite
   passes with the flag), the rest of Q2.12 (Spaces), the readiness list
   of Q2, Q2.13.
3. Beside it, with no Mac: Q6 areas and the Q5 rows that live inside the
   renderer (decision 4).
4. The perf lane takes only work that survives the flip (style, layout
   and script inside a renderer) until decision 2 says what W7 means with
   a renderer per site.
5. Last: Q7 on the Mac.

### Waiting on the owner

Each line has a default, taken if there is no answer when the lane
reaches it.

1. **T1 review** (`taste.md`, waiting since 2026-10-21). It blocks T1b,
   T2 and every T3 card. Default: T1 stands as the template and T1b
   starts.
2. **What W7 means with a renderer per site** (proposed). In one process
   the page's script runs on the window's loop, so W7's "longest loop
   under 100 ms" asks for script speed: bbc's longest loop is one 326 ms
   timer task of the page. The ten perf steps of 2026-11-08 moved the
   four worst sites by 11 to 44% and none passes (Q4.21); bbc, zol and
   mistral are still 4 to 12 times over the loop budget, and there is no
   JIT by decision. With a renderer per site the window's
   loop no longer runs the script. Q4.25 reported on 2026-11-12 (its card): with a renderer per
   site the window's loop passes on bbc and react, but the page's real
   frame waits for the renderer's task. Proposed criterion: the window's
   longest loop under 100 ms, and the time from a wheel to the renderer's
   frame under 33 ms p95 (a click under 100 ms p95). Default until the
   owner answers: the perf lane works to the proposed criterion (Q4.25a
   to Q4.25e), keeps Q4.2 to Q4.8, Q4.11 and Q4.16, and takes Q4.12 out
   of W7.
3. **Floors and targets in `docs/conformance.md`** (proposed). A
   threshold set just under the first score stops a fall; it does not say
   the area is ready. Default: the column stays as the floor, and a
   "Target" column is added for the X2 areas and the Q6 areas, set from
   what the 20 sites and the daily sites use (Q0.3b).
4. **Q5 before the flip** (proposed). Site mode exists behind the flag
   and its tests run in CI, so a Q5 row can be built for site mode now.
   What blocks it is not the flip but two lanes editing `ui/` and `ipc/`
   at once. Default: rows inside the renderer (Q5.2, Q5.8, Q5.10) start
   now; rows that add IPC messages run one at a time in the proc lane,
   after the Spaces flip.
5. **Q4.24b**: bytecode validated once, operand reads unchecked in
   `vmLoop` (about 5% of jsbench). Default: not done.
6. **The site archives off the Mac** (`velocity.md` DV5): a private
   place the cloud sessions and one weekly CI job can read. Needs the
   owner's account. Default: W2, W7 and the soak stay Mac-only.

### The Mac queue

Work only the owner's Mac can verify, kept in one list so that one
sitting clears it (`velocity.md`, "Rules", 6):
- Q0.3: W2 and W3 on the current `main` (W3 compared with Chrome 2026-11-25, P1.S9).
- W7 and W8 on all 20 sites, in process and with `DWELLER_PROCESS=site`
  (last full run 2026-11-04), now the default (Q2.14).
- WebGL in a renderer on macOS (Q2.14h): Metal in Seatbelt, SwiftShader in
  the macOS ANGLE package.
- Q2.13: memory per tab against Chromium.
- Q1.11: the soak.
- macOS sandbox Left items (Q2.8a) and Q5.4 (IME) when they come up.
- Three `flaky:` issues seen only on the macOS runners (Q1.4b): #398, `inserted_scripts` with one window iteration of 118 ms where it allows 100 (queue run 45041506917, macOS shard 3; the re-run passed); #385, a tab-mode renderer that crashed in `window_renderer_popup_opener` (queue run 44987415404; CI kept no crash report), and #380, `window_frame_lists_reused` building the changing iframe in 3 frames, not 5 or more (run 44963994657). #380 and #385 did not fail in 12 rounds under 8 busy loops on Linux; on the Mac, run each 20 times (#398 with `DWELLER_TIMING=1`, to see which iteration is long) with `DWELLER_PROCESS=tab` and keep the crash report from `~/Library/Logs/DiagnosticReports`.

### Next, per lane

The wave of 2026-11-16 runs these lanes as one worker session each
(`waves/2026-11-16.md`: its lanes, their order and the IPC ranges).

| Lane | Next | Then |
|---|---|---|
| base | – | – |
| proc | – (Q2 is done; W7/W8 with the flip are in the Mac queue) | – |
| sec | – (the 5 X2 areas are at their targets) | `javascript:` URLs of links, `location` and frames, and replaced pages kept alive while a script holds them (Left of the last card) |
| perf | – (Q6 is done; Q4.25e and Q4.12: their verify is in their Mac queues; the last two ranked Q4.16 cards are done) | Q4.6, Q4.16 (their verify needs the site archives) |
| feat | – (Q5.9b: slices 1 to 5 done, YouTube is for the Mac) | – |

## Decisions (2026-11-04)

- **Full site isolation, as in Chromium and Firefox (Fission).** This
  reverses "no process per site" of 2026-10-15. Security comes first,
  so the 2–3× cost against a tab-per-process model is accepted.
- **The browser process.** It holds the UI, the network, cookies,
  storage and the profile.
- **Renderer processes.** Each renderer is locked to one site (scheme +
  eTLD+1) and runs in an OS sandbox. A renderer does HTML, DOM, CSS,
  layout, paint and JS.
- **Cross-site iframes run out of process.** The window composites the
  frames of all renderers from shared memory, as Spaces do now. There
  is no GPU process.
- **What stays inside a renderer.** One renderer keeps the design of
  today: one thread owns its DOM, scripts, style, layout and paint.
- **Staging.** The work goes the way Chromium and Firefox went: first a
  tab in its own process, then a process per site on navigation, then
  out-of-process iframes. Each stage ships behind
  `DWELLER_PROCESS=...` until Q2.14 turns it on.
- **The shipped build is checked.** It runs with bounds and overflow
  checks (`--mode safe`). Q0.2 measured the cost (loop +9.5%, jsbench
  +17.6%), and it is accepted (Q1.1).
- **Real sites stay the final check, not the driver.**

## Done criteria

Each axis has a command that exits 0. The W criteria are those of
`docs/plans/browser.md`.

| Axis | Criterion | Command |
|---|---|---|
| Stable | S1: no flaky test in 50 consecutive `ci` runs on `main` and in the merge queue (the TC5b record shows none) | `gh issue list --label flaky --state open` is empty, and the TC5b summaries show none |
| Stable | S2: a crash or hang in a tab never takes down the browser or another tab | the Q2.5 tests |
| Stable | S3: every fuzz target runs nightly and has no open crash | the Q1.10 job |
| Stable | S4: a 1 h soak over the recorded archives keeps RSS under budget, with no leak | the Q1.11 test |
| Secure | X1: a renderer cannot read data of another site or the profile, even when compromised | the Q2.7 and Q2.8 tests (a test hook acts as a compromised renderer) |
| Secure | X2: the web security areas of WPT are at threshold (`cors`, `content-security-policy`, `mixed-content`, `x-frame-options`, `html/browsers/sandboxing`, `html/browsers/origin`) | `conformance.py check` |
| Secure | X3: bad TLS (expired, self-signed, wrong host, TLS < 1.2) is refused with an interstitial | the Q3.4 test |
| Fast | W7 and W8 on all 20 sites, with site isolation on | `window_perf.py report --replay`, `report --live --chrome` |
| Fast | M1: memory per tab is within budget (budget set by Q2.13 from Chromium's numbers on the same sites) | the Q2.13 report |
| Complete | F1: every row of the basic-feature table (Q5) is "yes" | the per-step tests |
| Correct | W9 (every conformance area at threshold), W2, W3 | `conformance.py check`, `page_load.py check --agent` |

## Lanes and running order

| Lane | Files | Phases |
|---|---|---|
| base | `.github/`, `tests/`, `browser/tools/` (measuring) | Q0, Q1 (CI and test steps) |
| proc | `ui/`, `engine/tab.aye`, `engine/windows.aye`, `engine/frames.aye`, a new `ipc/`, `ui/spaces.aye` | Q2 |
| sec | `net/`, `engine/window_api.aye`, `engine/frames.aye` (sandbox) | Q3 |
| perf | `css/`, `layout/`, `paint/`, `font/`, `js/gc.aye` | Q4, then Q6 |
| feat | per step | Q5 |

The order:

1. Q0 runs first. It is small, and it gives numbers to steer by.
2. Then Q1, Q2 and Q4 run in parallel, one step at a time per lane.
3. Q3 runs beside Q2:
   - Q3.1–Q3.5 (`net/`) come before Q2.4, which moves the network to
     the browser process.
   - Q3.6 and later come after Q2.4.
4. Q5 starts after Q2.14, so the features are built on the new process
   model (decision 4 of "Start here" proposes to start part of it
   earlier, for site mode only).
5. Q6 follows Q4 in the perf lane.
6. Q7 runs last of the foundations.
7. Q8 runs after Q7.

Common to every step:
- It is one card. Cards land in batches (`velocity.md`, "Rules", 1),
  not one PR each.
- It runs only the tests that its verify names. It never runs two
  browser test runs at once.
- Logs go under `target/`.
- If the same verify fails twice, the step stops with a report.
- On the user's Mac, it runs `scripts/capped.sh`. Heavy steps go to
  cloud sessions.
- Each card names the Chromium files it read and, in one line, what
  Dweller takes from them.

## Q0: measure the truth (lane base)

Exit: every "Now" of the scoreboard is measured on the current `main`,
and the weekly workflow is green.

| Step | What | Verify | Chromium |
|---|---|---|---|

Done (rows in `foundations-done.md`, notes in `cards/`): Q0.2, Q0.4, Q0.5, Q0.1, Q0.3b, Q0.3, Q0.3f.

## Q1: stable in one process (lanes base and perf)

Exit: S1, S3 and S4 hold. The shipped build is checked. A runaway script
can be stopped.

| Step | What | Verify | Chromium |
|---|---|---|---|
| Q1.4 | Flaky tests: only when the TC5b record files a `flaky:` issue (none since 2026-10-21 in 38 `ci` runs; `window_spaces` and `window_css_animation` were last seen before Q1.3) | 50 runs in a row | – |
| Q1.11 | Soak: 1 h of scripted browsing over the archives, with RSS sampled and `--leak-check` at exit | RSS growth stays under the budget, with no leak | – |

Q1.1 was listed as open at the stop, but its verify holds on `main`
(`browser/build.sh` builds `--mode safe` by default, and the required
`browser` job runs "Smoke the shipped build"); closed 2026-11-11 in its
card.

Done (rows in `foundations-done.md`, notes in `cards/`): Q1.1, Q1.2, Q1.3, Q1.6, Q1.7, Q1.8, Q1.9, Q1.9b, Q1.10a, Q1.10a2, Q1.10b, Q1.10c, Q1.10d, Q1.10d2, Q1.10e, Q1.12, Q1.4b.

## Q2: process model and site isolation (lane proc)

Exit: X1 and S2 hold with `DWELLER_PROCESS=site` as the default.
Chromium: `docs/process_model_and_site_isolation.md`,
`docs/security/` and `content/browser/site_instance_impl.cc`.
Firefox's Fission design is the cross-check. The design is
`docs/process-model.md` (Q2.1).

How the steps run:
- Each step adds the site-lock checks for its own messages. Q2.7 is
  the audit of all of them, plus the compromised-renderer tests.
- The table is in running order. The sandbox (Q2.8) comes before the
  process swap (Q2.6).
- Q1.7 lands before Q2.5.

| Step | What | Verify | Chromium |
|---|---|---|---|

Done (rows in `foundations-done.md`, notes in `cards/`): Q2.1, Q2.1b, Q2.2a, Q2.2b, Q2.2c, Q2.3, Q2.3a, Q2.3c, Q2.3d, Q2.3e, Q2.3b, Q2.3f, Q2.4, Q2.4a, Q2.4d, Q2.4b, Q2.4e, Q2.4c, Q2.5, Q2.8a, Q2.8b, Q2.6a, Q2.6b, Q2.7, Q2.7b, Q2.7c, Q2.9, Q2.10, Q2.11, Q2.12b, Q2.12c, Q2.14a, Q2.14d, Q2.12, Q2.14b, Q2.14c, Q2.14e, Q2.14f, Q2.14g, Q2.13, Q2.14h, Q2.15, Q2.16, Q2.14.

### Readiness list for Q2.14 (proposed 2026-11-11)

The done steps of Q2 and Q3 carry about 85 Left items in their rows
(counted by clause). None was a step, so the flip looked three steps
away. The items below are the ones a person, a login or criterion X1
meets once the flag is the default; the step that left each is in
brackets. Each is one slice unless Q2.14a
shows more. The proc lane takes them in this order after the Spaces
flip.

Security (X1):
- The choice of process for a frame uses the URL before redirects, so a
  frame that redirects to another site stays in the wrong renderer
  [Q2.9].
- `blob:` documents have no site [Q2.7].
- Tab and site mode do not check the initiator's sandbox flags [Q3.9].

Logins and popups (the goal says "logged in"):
- `window.open` children and frames are dropped with the old renderer
  when a navigation swaps it [Q2.6b].
- `opener`, named targets and popup openers across renderers [Q2.3f,
  Q2.11, Q2.12].
- No `MessagePort` transfer across processes, and the window proxy is
  one level deep [Q2.11].

What a person sees:
- An out-of-process frame's clip ignores overflow, transform and
  opacity [Q2.9].
- No hit-test slow path: content above a frame, `pointer-events` and
  transforms send the click to the wrong process [Q2.10].
- `location.replace()` pushes a history entry [Q2.6a].
- On Linux the sandbox loses the fallback fonts that are opened late
  [Q2.8b].

Speed with the flag (W7, W8):
- The renderer blocks while its navigation document loads, and a
  document's body is not streamed, so the first paint waits for the
  whole document [Q2.4e, Q2.12b].
- The browser polls held fetches every 2 ms [Q2.4e].
- A renderer does not reuse its display list [Q4.20].
- Hidden child frames are not throttled, and no process is reused
  [Q2.9, Q2.10]; these go with Q2.13.

Tests:
- Session restore has no `site` run [Q1.9b].
- 48 tests name `DWELLER_PROCESS=tab` and 2 name `site`; the other 180
  or so run in one process. Q2.14a measures them.

Not needed for the flip (they stay Left items of their cards): trackpad
latching and `overscroll-behavior` across frames, `rootBounds`, a child
frame's history entries, a rule for a site that keeps crashing, `HOME`
in the renderer's environment, a seccomp table for aarch64, wasmtime in
the sandbox (Q8), the agent highlight, find in frames, the clipboard
outside text fields.

## Q3: web security (lane sec)

Exit: X2 and X3 hold. The new WPT areas are in `docs/conformance.md`
with thresholds.

| Step | What | Verify | Chromium |
|---|---|---|---|

Done (rows in `foundations-done.md`, notes in `cards/`): Q3.1, Q3.1c, Q3.1b, Q3.2a, Q3.2b, Q3.3, Q3.4, Q3.4b, Q3.5, Q3.5b, Q3.6, Q3.7, Q3.7b, Q3.8, Q3.9, Q3.10, Q3.10b, Q3.10c, Q3.10d, Q3.11, Q3.12, Q3.13, Q3.14, Q3.14b, Q3.14c, Q3.15.

## Q4: interactivity (lane perf)

Exit: W7 holds on all 20 sites, first in process and then again after
Q2.14.

Running order, from the Q4.1 profile (2026-11-04, `PERF.md` "Q4.1"):
Q4.6, Q4.13, Q4.9, Q4.14, Q4.15, Q4.10, Q4.3, Q4.2 with Q4.4, Q4.11,
Q4.16, Q4.17, Q4.12. Q4.5, Q4.7 and Q4.8 moved nothing measured on
their own; they run only if a later profile shows them. Each step
states the W7 sites it moves.

| Step | What | Verify | Chromium |
|---|---|---|---|
| Q4.25a | With `DWELLER_PROCESS=site`, a wheel on zol never gets `renderer:input`: 3 of 3 runs hung to the tool's 600 s timeout with the window and the renderer at 0% CPU (Q4.25's card has the command) | the zol scroll row of `window_perf.py` finishes with the flag | – |
| Q4.25b | What the window waits on for 0.7 to 1.5 s on mistral with the flag: the wait has no window record | the wait has a record and a cause in the card | – |
| Q4.25e | The window scrolls from the renderer's last frame while the renderer is busy (the texture offset of P4.36c) | the wheel-to-frame p95 of bbc with the flag is under 33 ms while a renderer task runs | `cc/input/input_handler.h`, `cc/trees/layer_tree_host_impl.cc` |
| Q4.6 | Why `relayoutDirty` refuses (mistral: 1301 times): fix the exits | the mistral refusals fall under 10 | `core/layout/layout_object.cc` (`MarkContainerChainForLayout`) |
| Q4.12 | HTTP disk cache with revalidation and expiry, request priorities by kind, and the preload scanner checked. Includes Q0.4d: zol's live document load hits the 30 s timeout | live W8 does not drop, zol's W8 passes when Chrome paints under 5 s, and a revisit is served from cache | `net/http/http_cache.cc`, `core/html/parser/html_preload_scanner.cc`, `platform/loader/fetch/resource_fetcher.cc` |
| Q4.16 | JS interpreter speed from the hottest bytecodes on bbc, youtube, wikipedia-en and go-docs: fused instructions, missing inline caches, native call cost (P4.75s, P4.75u). No JIT: a JIT needs executable memory, which the renderer sandbox (Q2.8) forbids (measured by Q4.16a, #303: mistral is natives and forced restyle, bbc is calls and inline-cache misses; the ranked cards Q4.16b–l are in its card) | bbc's script ms falls by a third; jsbench does not drop | `v8/src/interpreter/interpreter-generator.cc`, `v8/src/ic/` |
| Q4.27 | GitHub's Turbo soft navigation completes: the agent's click lands on `/rust-lang/rust/issues` (done: an action moves the page clock through the responses and frames it waits for; Q4.27b: a Space's renderer is no longer killed at Turbo's `pushState`); left: a background fetch's `response.url` is empty when its body came whole | `check --agent --site github` exit 0; the window click lands on the issues page | `content/browser/renderer_host/render_frame_host_impl.cc` (`DidCommitSameDocumentNavigation`) |

Done (rows in `foundations-done.md`, notes in `cards/`): Q0.4a, Q4.1, Q4.6a, Q4.6b, Q4.6c, Q4.9, Q4.10, Q4.13, Q4.14, Q4.15, Q4.15b, Q4.16c, Q4.16e, Q4.16g, Q4.16m, Q4.16k, Q4.16n, Q4.21, Q4.22, Q4.17, Q4.18, Q4.19, Q4.20, Q4.23, Q4.24, Q4.25, Q4.3, Q4.4, Q4.5, Q4.2, Q4.8, Q4.25c, Q4.25d, Q4.7, Q4.11.

## Q5: basic features at Chromium parity (lane feat, after Q2.14)

Each step is one feature, with its WPT subset or a window test.
Ordered by daily use.

Size (estimate of 2026-11-11, `velocity.md` "What the history says"):
the started phases went from 53 planned rows to 111 in five days, a
factor of 2.1. By that factor these 18 rows are about 40 slices; by what
each row names they are 40 to 60: one for Q5.8, two or three for most,
five or more each for Q5.9a, Q5.10 and Q5.14a. A row gets a card that
lists its slices before any of it is dispatched.

Where a row can be verified:
- In the cloud, by WPT or a headless window test: Q5.2, Q5.3, Q5.6,
  Q5.7, Q5.8, Q5.10, Q5.11a, Q5.11b, Q5.12, Q5.14a, Q5.14b.
- Only on the Mac, by the owner: Q5.1 (the keys and the look), Q5.4
  (a real input method), Q5.5 (the system picker), Q5.9a and Q5.9b
  (the system decoders, sound), Q5.13, Q5.15. These go to the Mac queue
  with a checklist line each.

| Step | Feature | Chromium |
|---|---|---|
| Q5.9b | Media 2: Media Source Extensions (YouTube) | `third_party/blink/renderer/modules/mediasource/` |

Autofill and password saving stay out, as decided before
(`docs/plans/browser.md` "Login readiness").

## Q6: correctness to threshold (lane perf, after Q4)

One step per area, from its largest failure cluster with one cause, until
the area is at threshold. The order follows site impact (W3).
Verify: the area's `wpt.py run` with the gain, and `conformance.py
check`. Chromium references come from the module map of
`docs/plans/browser.md`.

| Step | Area (score, threshold) |
|---|---|
| Q6.16 | test262 `language`, `built-ins`, `annexB` toward their target 90 (86.4, 86.2, 82.4 on 2026-11-23): array element and `length` descriptors first |
| Q6.17 | `html/semantics/forms` toward its target 82 (71.5 -> 73.6, 2026-11-23: validity flags of barred controls); next: `cloneNode` state, `autocomplete`, `<meter>` |

Done (rows in `foundations-done.md`, notes in `cards/`): Q6.1, Q6.11, Q6.11b, Q6.12, Q6.15, Q6.6, Q6.7, Q6.5, Q6.10, Q6.14, Q6.2, Q6.4, Q6.8, Q6.9, Q6.3, Q6.13, Q6.18, Q6.19.

Areas can split into more steps. A step stops at its time budget with a
"left" list, as the P10 cards do now.

Two notes of 2026-11-11:
- The thresholds here are floors (decision 3). Q6.1 closed at 65.1
  against 65, so an unrelated change can reopen it; a step that closes an
  area leaves at least 2 points of room or says why it cannot.
- Q6.2 is blocked by Q6.14 for about 1 900 of the 2 200 subtests it
  still needs, so Q6.14 runs before the next Q6.2 slice. Every area here
  is measured in the cloud; none needs the Mac.

## Q7: acceptance and release (lane sites)

| Step | What |
|---|---|
| Q7.1 | W2 and W3 on all 20 sites, and W7 and W8 with site isolation on |
| Q7.2 | The user's daily sites (P3.80): a bundle with telemetry on. Reports become cards in Q3–Q6 |
| Q7.3 | The manual checklist on macOS (W5) |
| Q7.4 | Docs pass and a tag |

## Q8: deferred

Phase G of `docs/plans/browser.md` comes after Q7: WebGL (P11.12–P11.15,
P11.29), GPU compositing (P11.22, P11.23), Web Audio, WebAssembly, the
game (P11.24) and P4.79. Q2's shared-memory frames are the base for
P11.22.

## Firefox, for comparison

Fission (on by default in desktop Firefox since version 95, 2021) has
the same shape as Chromium's site isolation:
- A parent process holds the UI and the canonical browsing-context
  tree.
- Content processes are isolated per site (scheme + eTLD+1), and
  cross-site iframes run in other processes ("remote subframes").
- Networking, GPU and media decoding run in processes of their own.

The ideas Dweller takes are the ones the two browsers share:
- A privileged browser process that alone holds the network, cookies
  and storage.
- Sandboxed renderers locked to one site.
- Cross-site frames out of process.
- The window as the single compositor.
