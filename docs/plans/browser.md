# Plan: the Dweller browser

The work on `browser/` in `aye-lang/dweller`, a public repository (MIT
OR Apache-2.0) split out of the Aye repo with its history (P8.2, done
2026-10-15). The lane is active again: the window work (P4.31, P4.32)
and the agent Spaces (P9) resumed it, and the strategy of 2026-10-15
(below) replaces the maintenance mode of 2026-10-08. Steps finished
before the split are in `docs/plans/done.md` of the Aye repo; steps
finished here keep their "done" note on their card.

## Strategy (2026-10-15)

**Change.** The user moved Dweller from site-driven work ("fix whatever
a recorded site needs", P3) to foundations first, in three layers. Real
sites become the final check, not the driver.

**Why.** Site-driven work made 20 recorded sites load and agents work
on them. Two problems remained:

- The core stayed synchronous. The window waits for the network, every
  DOM change restyles and lays out the whole document, and paint keeps
  nothing between frames (`browser/engine/PERF.md`, "Window"). The
  user's feedback (2026-10-15): the window lags, scrolling feels wrong,
  some sites are very slow where Chrome is instant, and CSS is often
  drawn wrong.
- Each component (HTML, DOM, CSS, images, events, fetch, JS) was measured
  only against those 20 sites, so nobody knew how complete a component
  was or whether a fix broke a case no site showed.

**The three layers.**

| Layer | What | Measured by | Phases |
|---|---|---|---|
| 0, core architecture | async event loop and networking, incremental rendering pipeline for the window, threading model; the agent Spaces architecture runs beside it | W7 (interactivity), W8 (first paint), a CI perf gate | P4 (window and pipeline); P9 in parallel |
| 1, components | HTML parsing, DOM, CSS (by area), images, events, fetch/XHR, JS | standard suites: WPT and test262, a per-area pass-rate table (W9) | P10 (conformance) |
| 2, real sites | the recorded sites, popular live sites the user uses | W2, W3, W4, `page_load.py` | P3 (sites), then P6 (release) |

Work goes Layer 0 first. The Layer 1 harness (P10.1–P10.4) starts right
after the first Layer 0 cards, in parallel, so that measurement exists
before component work. Layer 1 component cards come from failing WPT
and test262 tests, grouped by area. Layer 2 runs at the end of each
layer as a check; a site defect becomes a card only when it is truly
site-specific, else it goes to the Layer 1 area it belongs to.

**Chromium as a reference, not a template.** Layer 0 cards name the
Blink/Chromium docs and code to read first (a local checkout:
`/Users/solemnis/Documents/chromium/src`, read-only; ignore anything in
it that is not upstream Chromium). Each card says in one or two lines
what Dweller takes. Dweller keeps its simpler design: one window thread
that owns the DOM, scripts, style, layout and paint; network completions
as tasks on that thread; no compositor thread, no GPU, no process per
site. Spaces (P9) are the one place with more than one process.

## Done criteria

**Standing rules (from the freeze of 2026-10-08)**

- M1: `cargo test --release --test browser` exits 0 on `main` after
  every merge (CI, macOS and Ubuntu).
- M2: the Aye-repo wrap-ups are merged there: P3.48 and P3.58 (branches
  `worktree-agent-ae5e6f75817463272`, `worktree-agent-a0b64f7b014b9b6ea`
  of the Aye repo; still open, see Current state).
- M3: every limitation of Aye or std the browser works around has an
  entry in `browser/COMPILER_GAPS.md` with **Hit by**.

**Browser done**

- W1: `cargo test --release` passes, `jsdom_gc_stress` included.
- W2: `python3 browser/tools/page_load.py check --agent` over the
  recorded archives exits 0: no crash or timeout, JS errors at or under
  the expected count, expected Markdown text, one agent task per site.
- W3: every listed site is ✅ in content (`check`) and visually (the
  `visual` field of `browser/tools/sites.json`, P1.14), except sites
  excluded as bot walls.
- W4: load-time targets met over `run --live --repeat 5` (Decisions).
- W5: the manual checklist (`docs/browser-manual-checklist.md`) is run
  and recorded in its results table.
- W6: every `docs/browser.md` "What remains" item is done or declared
  out of scope.
- W7 (interactivity, Layer 0): on every non-excluded site of
  `sites.json`, replayed from its recorded archive in the window at the
  default size: the longest loop iteration from navigation start to 10 s
  after load is under 100 ms, the scroll frame p95 (40 frames of 3 wheel
  notches) is under 33 ms, and the idle CPU (10 s starting 5 s after the
  first frame) is under 5%. Command: `python3 browser/tools/window_perf.py
  report --replay` (P4.41) prints a row per site and exits 0 when all
  rows pass.
- W8 (first paint, Layer 0): on every non-excluded site, live, Dweller's
  first paint is within 1.0 s of Chrome's first contentful paint on the
  same machine and network (median of 3 alternating runs each). Command:
  `python3 browser/tools/window_perf.py report --live --chrome` (P4.41).
- W9 (conformance, Layer 1): every area of `docs/conformance.md` is at
  or above its threshold at the pinned WPT and test262 revisions, and
  `python3 browser/tools/conformance.py check` exits 0 (P10.4).

## Current state (2026-10-15)

- Window: the diagnosis of 2026-10-15 (PR #23, `browser/engine/PERF.md`
  "Window") measured 10 live sites against Chrome. Longest loop up to
  5440 ms (YouTube), scroll p95 up to 750 ms, idle CPU 101% on Mistral,
  first paint 1.7–15.8 s against Chrome's 0.3–2.0 s. Five root causes;
  cards P4.32–P4.37. P4.31 (macOS wheel and trackpad) and P4.32 (page
  fetch/XHR in the background) are done; P4.43 moved `import()` and
  frames to the background and runs the page's scripts as tasks; scripts
  that a script inserts do not run yet (P4.45).
- Sites: `page_load.py check` passes on the recorded archives; W3 is 7
  of 20 (P3.65 and P3.73b done). The open visual defects are re-filed
  below under their Layer 1 area or Layer 2.
- Archives live in each checkout's `target/sites/` (untracked), so steps
  that replay sites or take screenshots run locally.
- Conformance: not measured. The browser's own suites (530 files in
  `browser/tests/js`, the HTML/CSS/layout/paint fixtures) stay as
  regression tests; WPT and test262 are not run yet (P10).
- P9: P9.1–P9.4 done (P9.4: #24); P9.5 next.
- Aye repo: P3.48 is not merged there (the merge 34f6f93 failed the site
  check: mdn FAIL, mistral 4 s → 40 s wall; P3.48b bisects it); P3.58 is
  WIP 34c2234 (native `TypedArray`). Open compiler gaps: none.

## Phases

| Layer | Phase | Goal | Exit check | Status |
|---|---|---|---|---|
| – | P0, P1, PW | one clean `main`; "done" checked by a command; wrap-up | – | done |
| – | P8 | Dweller in its own repo | CI green on macOS and Ubuntu | P8.2 done 2026-10-15; P8.3 (removal from the Aye repo, downstream check) is the Aye repo's card |
| – | P7 | tests green while the language changes | M1, M3 after every `AYE_REV` bump | ongoing |
| 0 | P4 | the window responds and paints early | W7, W8, the CI perf gate (P4.40) green | current: P4.32, P4.33, P4.34 and P4.43 done; P4.44 next |
| 0 ∥ | P9 | agents without disturbing the user (B9, R1–R7) | the P9 tests pass: `window_agents`, `window_spaces*`, `agent_sessions` | parallel track; P9.1–P9.4 done, P9.5 next |
| 1 | P10 | components measured by WPT and test262 | W9 | next: harness P10.1–P10.4, parallel with P4 |
| 2 | P3 | real sites correct | W2, W3; `docs/browser-sites.md` rerun | final check after P4 and P10 |
| 2 | P5 | B8 "What remains" resolved (W6) | the list is empty or out of scope | items re-filed under P10 areas |
| 2 | P6 | release | W1–W9; manual checklist recorded; tag | last |

## Order of work

Cards are PR-sized and run in this order. P9 cards run beside them
(they touch `browser/ui/spaces.aye` and the agent code, not the
pipeline). P10.1–P10.4 touch `browser/tools/` and add CLI modes, so they
may run beside P4 cards; two cards never edit the same files at once.

| # | Card | Layer | One line |
|---|---|---|---|
| 1 | P4.32 | 0 | page `fetch()`/XHR complete as tasks; the window thread never waits on them (done) |
| 1b | P4.43 | 0 | `import()` and frames load in the background; the page's scripts run as tasks (done) |
| 1c | P4.45 | 0 | scripts that a script inserts (`<script src>`, JSONP chunk loaders) run, loaded in the background |
| 1d | P4.46 | 0 | back, forward and reload in steps, as the other navigations |
| 2 | P4.33 | 0 | first paint once the document and its head sheets are in; one prefetch batch for all subresources (done) |
| 3 | P4.34 | 0 | the window's relayout uses the subtree restyle and incremental layout; one document lifecycle (done) |
| 3b | P4.44 | 0 | incremental restyle and layout in documents with container queries (GitHub) |
| 4 | P4.40 | 0 | CI perf gate: synthetic heavy pages on the test server, budgets, a `perf` job |
| 5 | P4.35 | 0 | raster cache for gradients, SVGs, blurred shadows and filter layers; damage tracking (done) |
| 5b | P4.55 | 0 | `requestAnimationFrame` and timers throttled in hidden windows and background tabs (HTML spec) |
| 6 | P4.36 | 0 | scroll by moving the last frame and drawing only the new band |
| 7 | P4.37 | 0 | one connection pool across batches and navigations |
| 8 | P10.1 | 1 | WPT testharness runner: fetch at a pinned rev, static server, `dweller wpt` |
| 9 | P10.2 | 1 | WPT reftest and crashtest runner (screenshots against the reference) |
| 10 | P10.3 | 1 | test262 runner over `dweller js --test262` at a pinned rev |
| 11 | P10.4 | 1 | conformance table generator, first baseline, weekly CI run |
| 12 | P4.41 | 0 | W7/W8 report over the recorded archives and live against Chrome |
| 13 | P4.42 | 0 | loop priorities (input and frames before timers and network tasks) and the threading model note |
| 14 | P10.5 | 1 | fetch/XHR tests under upstream `wpt serve` (`DWELLER_RESOLVE`) |
| 15 | P10.6 | 1 | `testdriver.js` subset: clicks and keys through `--events` |
| 16 | P10.20 | 1 | login fixture on the test server and the `window_login` test (form login, redirects, CSRF, Basic, two hosts) (done) |
| 17 | P4.50 | 0 | profile directory `~/.dweller`: cookies saved on change, session cookies kept across restart (done) |
| 18 | P4.51 | 0 | `localStorage` saved per origin in the profile; `sessionStorage` per tab (done) |
| 19 | P4.52 | 0 | `window.open`, `target=_blank` and `window.opener` with `postMessage` between tabs (OAuth/SSO popups) (done) |
| 20 | P10.21 | 1 | cookies: SameSite (Lax default), third-party policy, `fetch`/XHR `credentials`, public suffix list |
| 21 | P10.22 | 1 | HTTP authentication: 401 Basic challenge, URL userinfo, per-realm cache (done) |
| 22 | P10.23 | 1 | sign-in page needs: `autocomplete`, `navigator.credentials` absent without throwing, `Set-Cookie` on fetch/XHR |
| 23 | P4.53 | 0 | local telemetry (JSONL under `~/.dweller/telemetry`) and `telemetry.py summary` (done) |
| 24 | P9.7 | 0 ∥ | shared logins with Spaces (after P4.50 and P4.51) |
| 25 | P3.80 | 2 | bundle for the user, once W1–W9 and the cards above hold |
| 26+ | P10.n | 1 | component cards per area from the baseline's failures (backlog below) |
| last | P3, P6 | 2 | sites and release |

Common to all cards: run only the tests named in verify, never two
browser test runs at once; the full suite runs once on `main` after the
merge (CI). Build and test output goes to a log under `target/`, read
with `tail`/`grep`.

## Layer 0: P4, the window and its pipeline

The five root causes of `PERF.md` "Window": (1) the window thread waits
for the network, (2) every DOM change restyles and lays out the whole
document, (3) paint keeps nothing between frames, (4) busy pages stay
busy (1 and 2 repeating), (5) connections are not reused. Each card
quotes its site numbers from that table; the CI perf gate (P4.40) keeps
them from coming back.

#### P4.32: the window never waits on the network
- goal: page `fetch()`/XHR and dynamic script loads complete as tasks while `Ui.loop` runs (the curl multi handle polled by the loop), not synchronously on the window thread (cause 1). Synchronous XHR (`open(..., false)`) stays synchronous, as the spec allows.
- files: `browser/net/loader.aye`, `browser/net/multi.aye`, `browser/engine/script.aye`, `browser/engine/prelude.aye` (XHR), `browser/ui/window.aye` (`Ui.loop`)
- chromium: `services/network/README.md`, `net/docs/life-of-a-url-request.md`, `third_party/blink/renderer/platform/scheduler/TaskSchedulingInBlink.md`. Take: a request's completion is a task posted to the main thread's queue; the main thread never blocks on a socket. Dweller has no network process: libcurl multi, polled by the loop, plays that part.
- verify: `python3 browser/tools/window_perf.py scroll https://github.com/aye-lang/aye --binary target/dweller` longest loop < 100 ms (was 2898); YouTube < 100 ms (was 5440); `cargo test --release --test browser -- net_http fetch_methods xhr_methods window_scripts > target/p4.32.log 2>&1; echo $?` → 0
- status: done 2026-10-15 for page `fetch()` and asynchronous XHR; dynamic script loads moved to P4.43. A script's request is a load of its own on the loader's multi handle (`net.startRequest`: any method, headers, body, redirects as `transfer`), settled by a networking task (script.aye's load timers: polled by the loop on the window's clock; due at once and waiting for the request on the virtual clock, so `dump` stays deterministic); other schemes, replays and no session answer at once. `fetch()` settles in that task and its signal aborts it; async XHR fires loadstart, readystatechange 2/3, progress, readystatechange 4, load, loadend; `abort()`/`open()` drop a request in flight; sync XHR still blocks. `window_perf.py scroll` longest loop: YouTube 3061–6749 → 588–771 ms (idle 3575 → 119), GitHub 1763–2467 → 1912–2061 (what is left is relayout, cause 2, and 92 synchronous chunk loads, 0.8 s: P4.43); no loop waits for a page fetch/XHR (`PERF.md` "P4.32"). The < 100 ms targets need P4.34 and P4.43 (GitHub) and P4.33 (YouTube's relayout prefetch). Page Markdown of every archived site unchanged. Tests `xhr_events`, `window_fetch_in_background` (1.5 s fetch while typing and scrolling: longest loop < 100 ms); `bg_images` log: the second script's fetch settles after the load event on the virtual clock. Async request records are `background_requests` (start to end), so `requests` are the synchronous ones.

#### P4.43: dynamic script loads in the background
- goal: a `<script src>` that a script inserts (webpack chunks, `import()`) loads on the loader's multi handle and runs as a task once it arrived, in the order the HTML standard gives (async by default; `async=false` ones in insertion order), instead of a synchronous request on the window thread. Left by P4.32: GitHub's first loop after the load waits 0.8 s for 92 `chunk-lazy-*.js` requests, one by one.
- files: `browser/engine/script.aye`, `browser/engine/sources.aye`, `browser/net/loader.aye`
- verify: `window_perf.py scroll https://github.com/aye-lang/aye`: no synchronous `requests` record for a script inside a `ui_loop`; `page_load.py check` exit 0 with every site's Markdown unchanged; `cargo test --release --test browser -- window_scripts jsdom_fixtures > target/p4.43.log 2>&1; echo $?` → 0
- status: done 2026-10-15, with two items of P4.33's notes (each script a task, frames in the background); back/forward/reload stay synchronous (P4.46). The premise was off for classic scripts: a `<script>` that a script inserts never ran (no fetch at all; P4.45); the synchronous loads were `import()`, GitHub's `chunk-lazy-*.js`. On the window's clock `import()` loads its module and its graph a level at a time on the loader's multi handle (lib.aye `loadInBackground`/`graphInBackground`, window_api.aye `moduleLoad`: a networking task per load) and evaluates from the cache once they are in; the page's scripts run as tasks of at most 50 ms an iteration, one at least (`engine.stepPageScripts`, `Ui.stepScripts`), the window keeping the layout from before them and its timers waiting until they ran; what is in when the navigation begins (file:, data:) still runs before the first paint; the documents of the page's frames load in the background (`startFrameLoads`, `net.startRequest`) before the load event. `window_perf.py scroll` longest loop after first paint, before (e030237) / after: GitHub 3107 → 1276–1291 ms (no synchronous request: 13 requests, 887 ms → 0; what is left is two container-query relayouts, P4.44, and timers with forced layouts, P4.42); YouTube 1715 → 1279 ms, a single script task (`kevlar_base`, 1221 ms) plus 57 ms, other iterations ≤ 617 ms (timers, P4.42, and a relayout whose `net.prefetch` of the sheets scripts inserted waits 224 ms) (`PERF.md` "P4.43"). `dump` unchanged: Markdown and screenshots of the 22 archived sites byte-identical with main's binary, `page_load.py check` exit 0. Test `window_dynamic_scripts` (20 `import()`s of 200–390 ms, a module with 20 such imports, a frame of 1 s: every iteration < 100 ms outside script tasks, each import settles with its module, imports evaluate in order, load waits for the frame; main's binary spends 7.1 s in one iteration there).

#### P4.45: scripts that a script inserts run
- goal: a classic `<script>` element that a script connects (`createElement('script')`, webpack/Vite JSONP chunk loaders, analytics snippets) runs as HTML's "prepare the script element" says: an inline one at once, an external one fetched (on the window's clock on the loader's multi handle, as `moduleLoad`) and run in a task when it arrived, `async` ones as they come, `async = false` ones in insertion order; `load`/`error` events at the element; `document.currentScript`. Today such a script never runs, so pages that load chunks this way miss them. It changes what `dump` shows on the recorded sites (more scripts run): review the Markdown and screenshot differences site by site and rebless.
- files: `browser/engine/script.aye`, `browser/engine/dom_api.aye` (insertion steps), `browser/engine/prelude.aye` (`resourceMutations`), `browser/engine/window_api.aye`
- test: a jsdom fixture (inline and external inserted scripts, `async = false` order, load/error events, a script moved or inserted twice runs once); the window test of P4.43 extended with 20 inserted `<script src>` of 200 ms delays: in order, every loop iteration < 100 ms outside script tasks.
- verify: `cargo test --release --test browser -- jsdom_fixtures window_dynamic_scripts > target/p4.45.log 2>&1; echo $?` → 0; `page_load.py check` exit 0 after the reviewed rebless

#### P4.46: back, forward and reload in steps
- goal: the window's back, forward and reload (`Ui.goBy`, `Ui.reload`, the history traversals scripts ask for) go through the incremental navigation of P4.33/P4.43 (`navigateEarly`, `stepPageScripts`) instead of a synchronous load: the page is shown once its sheets are in, its scripts run as tasks, its frames load in the background; the scroll position restored after its scripts ran.
- files: `browser/engine/tab.aye`, `browser/engine/history.aye`, `browser/ui/window.aye`
- test: a window test going back and forward between two local pages with a delayed sheet and script: the address and the old page answer input during the load, the scroll position comes back, longest loop < 100 ms outside script tasks.
- verify: `cargo test --release --test browser window_ > target/p4.46.log 2>&1; echo $?` → 0

#### P4.33: progressive first paint
- goal: the window draws as soon as the document is parsed and its render-blocking sheets (in `<head>`, media matching) are in, before scripts at the end of the body, async/defer scripts and images finish; images fill in as they decode. The first navigation runs inside `Ui.loop`, so the window takes input while the page loads. One prefetch batch for every sheet, script and image the document names (a scan of the markup ahead of the parser), not one batch per kind (cause 1).
- files: `browser/engine/tab.aye`, `browser/engine/engine.aye` (load phases), `browser/engine/sheets.aye`, `browser/engine/images.aye`, `browser/html/` (a URL scan of the unparsed rest), `browser/ui/window.aye`
- chromium: `third_party/blink/renderer/core/html/parser/html_preload_scanner.h` (and `background_html_scanner.h`), `third_party/blink/renderer/core/loader/render_blocking_resource_manager.h`. Take: a cheap tokenizer pass finds `src`/`href` URLs and starts their fetches while a script blocks the parser; only head sheets and parser-blocking scripts block rendering, so paint when they are done. Not taken: speculative parsing threads, preload priorities.
- test: a local page with a sheet at `/sleep?ms=300`-style delays, a body-end script delayed 1500 ms and images delayed 2000 ms: `--state` after the first frame shows the styled text before the script ran; a click during the load is handled.
- verify: `window_perf.py scroll` first paint HN ≤ 1.3 s (now 1.7–2.4), Wikipedia ≤ 1.0 s (1.8), YouTube ≤ 3 s (15.8); `cargo test --release --test browser window_ > target/p4.33.log 2>&1; echo $?` → 0
- time box: 120 min; model: opus (load ordering)
- status: done 2026-10-15. The window's navigations (`Ui.go`: the first one, the address bar, links, GET forms) go in steps (`Tab.navigateEarly`): the document is fetched and parsed, then one batch on the loader's multi handle starts every linked sheet whose media match, every script and modulepreload, and every `img` (`engine.startEarly`; the parser has the whole document, so the scan walks the tree instead of the unparsed markup); the loop lays the page out once the sheets are in (first paint, `ui_first_paint` record) and runs the scripts once they are in too, while it takes input. Images and web fonts (as `font-display: swap`) load in the background on the window's pages and fill in as they arrive (a relayout at most every 100 ms or twice the last relayout); frames load with the scripts. `net.prefetch` (the synchronous batches that are left: `dump`, scripts' later sheets) now runs on the loader's multi handle too, joining loads in flight. `window_perf.py scroll` first paint, median of 3, before (9bca5cc) / after: HN 1237 → 622 ms, Wikipedia 956 → 652 ms, YouTube 6285 → 2084 ms (`PERF.md` "P4.33"). Back/forward/reload and agents' navigations stay synchronous. Markdown and screenshots of all 22 archived sites byte-identical with the previous binary. Test `window_paints_early` (sheet 0.3 s, image and script 1.5 s: first paint < 1.2 s, a click and a wheel during the load handled, longest loop < 100 ms outside the scripts' run). Left: the iteration that runs the scripts (YouTube 1.8–2.0 s: 1.4 s of JavaScript plus a synchronous iframe fetch) is now inside the loop: each script as its own task (P4.42) and frames in the background; sheets compete with scripts for bandwidth (no priorities): YouTube's sheets arrive 1.7 s after the document.

#### P4.34: one lifecycle, incremental window relayout
- goal: `Ui.relayout` reuses the scripts' subtree restyle and incremental layout (P3.63/P3.64) instead of `restyleWithSizes` plus a full layout per DOM change (cause 2). One styles-and-geometry state per page, shared by the window's frames and the scripts' forced reads, with a lifecycle field (style clean, layout clean, display list clean); a frame advances it only as far as needed; a paint-only change rebuilds only the display list. Brings in P4.10 (container-query re-pass on container subtrees) where Mistral or GitHub need it.
- files: `browser/engine/engine.aye` (`layoutPage`), `browser/engine/containers.aye`, `browser/engine/geometry.aye`, `browser/layout/incremental.aye`, `browser/ui/window.aye` (`relayout`)
- chromium: `third_party/blink/renderer/core/dom/document_lifecycle.h`, `third_party/blink/renderer/core/css/style-invalidation.md`, `third_party/blink/renderer/core/layout/README.md`. Take: one lifecycle state per document that a frame and a forced read both advance in order (style, layout, paint), never past what is needed; invalidation marks only the elements a change can affect (Dweller's `RestyleDeps` keys are its invalidation sets). Not taken: sibling/descendant invalidation-set machinery, the fragment-tree cache.
- test: `DWELLER_LAYOUT_CHECK=1` over the jsdom and layout fixtures and the window tests (each incremental relayout compared with a full one); a fixture whose timer toggles a class 100 times: `DWELLER_TIMING` shows no `style_full` record after load.
- verify: `window_perf.py idle https://mistral.ai` CPU < 20% (now 101%); GitHub `ui_relayout` after load < 50 ms (now 370–900); `DWELLER_LAYOUT_CHECK=1 cargo test --release --test browser -- jsdom_fixtures layout_fixtures script_layout interleaved window_ > target/p4.34.log 2>&1; echo $?` → 0
- time box: 150 min; model: opus
- status: done 2026-10-15 for documents without container queries or shadow trees. After a full styling and layout the page's tree and styles become the scripts' layout (`adoptLayout`: `Geometry.shared`, `Page.tree_borrowed`); `layoutPage` then brings that one state up to date through `geometry` (subtree restyle, kept boxes or `relayoutDirty`) instead of `restyleWithSizes` plus a full layout, and the scripts' forced reads use the same state. The lifecycle is the shared geometry's version: a frame after no change does nothing, a paint-only change rebuilds only the display list. Falls back to the full path (and adopts again) when the sheets, the viewport, the form controls' state (`css.controlInputs`) or a change the scripts did not note (`Page.mutations`, now also `<details>` toggled by the front end) moved; new bitmaps or web fonts lay out again with the styles kept. Mistral live (`window_perf.py idle https://mistral.ai`): full cascades per loop 1.01 → 0.01, `ui_relayout` 31.1 → 6.8 ms, loop 44.6 → 18.8 ms; its rAF animation now runs at 53 instead of 22 frames/s, so idle CPU stays 101 → 102%: what is left per frame is the draw (9.7 ms, P4.35/P4.36), timers (JS, 2.6 ms), the forced reads (2.5 ms) and `paint_build` (2.2 ms). The < 20% target needs those, or a frame-rate cap for animations (a product decision). GitHub unchanged (`ui_relayout` 342–1054 ms): its styles use container queries, which stay on the full path (P4.44). `DWELLER_LAYOUT_CHECK=1` now also compares the kept styles with a full cascade ("STYLE CHECK"); the window path over the 22 recorded sites (`--frames 60` with wheel steps) logs no difference, screenshots byte-identical, `page_load.py check` exit 0. Test `window_relayout_incremental`.

#### P4.44: incremental restyle and layout with container queries
- goal: documents whose styles use container queries (`Styles.queried`: GitHub; Mistral's visual-intelligence iframe) take the P4.34 path: restyle the noted subtrees and lay out again incrementally, then compare the containers' sizes (`containerSizes`) and fall back to `layoutQueried` only when one moved. Left by P4.34, which keeps them on the full path (`adoptLayout`, `relayoutDirty`).
- files: `browser/engine/geometry.aye`, `browser/engine/containers.aye`, `browser/engine/engine.aye`
- verify: GitHub `ui_relayout` after load < 50 ms (now 342–1054); `DWELLER_LAYOUT_CHECK=1 cargo test --release --test browser -- jsdom_fixtures window_ > target/p4.44.log 2>&1; echo $?` → 0; the window path over the recorded sites logs no LAYOUT/STYLE CHECK

#### P4.40: CI perf gate
- goal: a PR that brings back a W7 regression fails CI, without network and without the recorded archives.
- fixtures (decision): synthetic pages under `browser/tests/perf/`, served by `browser/tests/server.py` with delays, each shaped like a heavy site of the diagnosis. Recorded archives are not committed: they hold third-party pages, scripts, images and fonts that a public MIT/Apache repo may not redistribute, they can hold cookies from the recording, they are tens of MB, and they drift as sites change. Synthetic pages are small, license-clean, deterministic, and each one isolates one root cause:
  - `feed.html` (GitHub/YouTube): 120 subresources with 50–300 ms delays; after load a timer fetches `/sleep?ms=600` every 500 ms and inserts rows; container queries. Measures the longest loop (P4.32) and relayout cost (P4.34).
  - `animated.html` (Mistral): 3000 elements; an animation changes one element's class and inline style every frame. Measures idle CPU (P4.34).
  - `paint_heavy.html` (react.dev): a long page of CSS gradients, inline SVGs, blurred `box-shadow`, `filter` and opacity layers. Measures the scroll frame p95 (P4.35, P4.36).
  - `article.html` (Wikipedia/HN): long text, 3 sheets, 2 scripts at the end of the body, 30 images, all delayed. Measures the first paint (P4.33).
- metrics: per page, the median of 3 runs of `window_perf.py scroll` and `idle` (`SDL_VIDEODRIVER=dummy`): longest loop, scroll p95, idle CPU, first paint; and counters from new `DWELLER_TIMING` records that do not depend on machine speed: full cascades (`style_full`), full layouts (`layout_full`), network waits on the window thread (`net_sync`), pixels rasterized per scroll frame (`raster_px`).
- budgets: `browser/tests/perf/budgets.json`. Counters must not exceed their recorded value. Times: the W7 limits (100 ms, 33 ms, 5%) for every page whose card has landed; before that, 1.5 × the value measured on the CI runner when the gate lands. A PR that improves a metric by more than 20% lowers its budget in the same PR (ratchet).
- CI: a `perf` job in `.github/workflows/ci.yml` on `macos-latest` (the user's platform), after the build, with the cached `aye`: `python3 browser/tools/window_perf.py gate browser/tests/perf/budgets.json --binary target/dweller`; it prints a table and exits 1 when a metric is over budget. Target run time: under 5 min.
- files: `browser/tools/window_perf.py` (`gate`, `--server`), `browser/tests/perf/*.html`, `browser/tests/perf/budgets.json`, `browser/tests/server.py` (`/delay?ms=N&type=css|js|png|json`), `browser/ui/window.aye` and `browser/engine/` (the counter records), `.github/workflows/ci.yml`
- verify: `python3 browser/tools/window_perf.py gate browser/tests/perf/budgets.json --binary target/dweller; echo $?` → 0 on this branch; with a deliberate regression (a full restyle forced per relayout, not committed) → 1; the `perf` job is green on the PR.
- time box: 120 min; model: opus (budget design), sonnet for the pages

#### P4.35: raster cache
- goal: gradient tiles, SVG images, blurred shadows and filter/opacity layers are rasterized once per display item at device size and reused while the item and its size are unchanged (cause 3). P4.5 (decode offscreen images at paint time) joins here.
- files: `browser/paint/raster.aye`, `browser/paint/background.aye`, `browser/paint/filter.aye`, `browser/paint/paint.aye`, `browser/svg/`
- chromium: `third_party/blink/renderer/core/paint/README.md` ("Paint result caching"), `third_party/blink/renderer/platform/graphics/compositing/README.md` (raster invalidation). Take: an item keeps its result while its object is not invalidated, so a frame redraws only invalidated items. Not taken: paint chunks, property trees.
- verify: `window_perf.py scroll https://react.dev` frame p50 < 30 ms (now 201); the paint and screenshot goldens unchanged: `cargo test --release --test browser -- screenshot_ check_png > target/p4.35.log 2>&1; echo $?` → 0; the perf gate's `paint_heavy` budget lowered
- time box: 120 min; model: opus
- status: done 2026-10-15, with damage tracking (decision of the main session: do what browsers do, no frame-rate cap). The window keeps the page's pixels between frames (`ui/pagedraw.aye`, drawn from the page's own origin): each display item has a signature (paint/damage.aye: a hash of everything it draws with, its context of clips and layers mixed in, and its area), and a frame draws only the items that differ from the last frame's (the two lists aligned on their common items) plus, after a scroll by whole pixels, the band that came into view (the rest is moved); a layer whose filter blurs is drawn whole when touched. No damage and nothing else changed: nothing is drawn or presented. A frame of the timers or a scroll copies and uploads only the damage (all of the page after a scroll) without composing the chrome. The raster cache (paint/rcache.aye) keeps layers of at most 2 Mpx (over their whole box, ready to blend), gradient tiles of at most 8 Mpx (their colors, each row computed the first time it shows) and shadows (their blurred coverage), keyed by their items and their position inside the pixel grid. In the window, SVGs, gradients, shadows and rounded corners are rasterized from whole pixels of their own, so a scroll or a kept result gives the pixels a new drawing gives; `screenshot` and `dump` keep the previous arithmetic (goldens and the 22 archived screenshots byte-identical). The rounded-box coverage rasterizes one row between the corners and copies it (both paths, identical bytes). Canvas bitmaps count their changes (`Bitmap.version`, in `Item.Image`). `window_perf.py scroll https://react.dev` (against a build of main): loop p50/p95 190/522 → 43.5/249 ms, draw p50/p95 167/489 → 25.7/112 ms; the loop p50 stays over 30 ms because react.dev's scroll listeners change the DOM in half the frames (timers and relayout ~20 ms: P4.42, P4.44) and each frame draws the 240 px band (its conic gradient ~12 ms) and the sticky header again (`paint_build` 3–7 ms rebuilds the list for it: P4.36). `window_perf.py idle https://mistral.ai`: 101.7 → 98.6%; its rAF animation moves a carousel 1 px a frame, so every frame damages 0.91 Mpx (31% of the window, median); per frame the draw is 10.1 → 6.0 ms and the loop 20.8 → 14.2 ms, and the animation runs at 59 instead of 40 frames/s; what is left is each frame's relayout (6.8 ms) and its scripts: browsers composite such a transform animation without layout or paint. `DWELLER_DAMAGE_CHECK=1` draws every frame whole too and logs "DAMAGE CHECK" with the pixels that differ (and when the window differs from the page): none over the 22 recorded sites (a dummy window and a real one at scale 2, 60 frames with wheel steps), live react.dev, and the window tests (`window_fetch_in_background` then misses its 300 frames in 60 s in the debug build, each frame being drawn twice). `page_load.py check --screens` exit 0, screenshots byte-identical with main. The perf gate (P4.40) has not landed: no budget to lower. Test `window_damage` (a canvas animation under the check: partial frames, frames with nothing drawn). rAF throttling went to P4.55.

#### P4.55: rAF and timers throttled when not seen
- goal: as the HTML spec lets browsers do, a page nobody sees runs no animation frames: `requestAnimationFrame` callbacks wait while the window is minimized or hidden (`SDL_WINDOWEVENT_HIDDEN`/`MINIMIZED`, back on `SHOWN`/`RESTORED`) and in background tabs; timers of background tabs run, clamped to at least 1 s (today they do not run at all: the loop runs the shown tab's timers only); an offscreen iframe's rAF may wait too. A fully occluded window needs SDL3's `SDL_EVENT_WINDOW_OCCLUDED` or a Cocoa `occlusionState` query; note it if neither fits. Visible pages keep their frame rate (decision for P4.35).
- files: `browser/engine/script.aye` (rAF timers flagged, held while the page is hidden), `browser/engine/prelude.aye` (`requestAnimationFrame`), `browser/ui/window.aye` (window events, the timers of other tabs), `browser/ui/tabs.aye`, `browser/ui/sdl.aye`
- chromium: `third_party/blink/renderer/core/frame/local_frame_view.cc` (throttled frames), `third_party/blink/renderer/platform/scheduler/` (background timer throttling, 1 s alignment).
- verify: `window_perf.py idle https://mistral.ai` with the window minimized after its first frame: idle CPU < 2% (98.6% shown); a test with two tabs: the background tab's 100 ms interval runs at most once a second and its rAF not at all, both resume when shown; `cargo test --release --test browser -- window_ > target/p4.55.log 2>&1; echo $?` → 0
- time box: 60 min; model: sonnet

#### P4.36: scroll by moving the last frame
- goal: a scroll moves the previous frame's pixels and draws only the band that scrolled in, then fixed and sticky boxes over it; a frame with no change draws nothing (cause 3).
- note 2026-10-15: P4.35's damage tracking moves the pixels on a scroll by whole pixels and draws the band and what changed (fixed and sticky boxes too), and a frame with no change draws nothing. Left: the list is built again on each scroll frame for fixed and sticky boxes (`relist`, react.dev 3–7 ms), and a scroll by a fraction of a device pixel (a trackpad) draws everything (from the raster cache); the verify numbers are not measured yet.
- files: `browser/ui/window.aye` (draw), `browser/paint/raster.aye` (draw a band), `browser/paint/paint.aye` (fixed/sticky items apart)
- chromium: `docs/how_cc_works.md` (tiles, scrolling, damage), `docs/life_of_a_frame.md`. Take: scrolling is an offset over pixels already drawn, and a frame draws only its damage rectangle. Not taken: a compositor thread, tiles, layer trees.
- verify: Wikipedia and Medium scroll frame p50 < 4 ms (now 9, 26); `cargo test --release --test browser -- window_wheel window_scroll window_sideways_scroll > target/p4.36.log 2>&1; echo $?` → 0; the perf gate's `raster_px` per scroll frame ≤ the band's area
- time box: 90 min; model: sonnet

#### P4.37: one connection pool
- goal: connections, TLS sessions and DNS are reused across prefetch batches, page fetches and navigations: one curl multi handle (with a share handle) for the process, at most 6 connections per host, HTTP/2 multiplexed (cause 5).
- files: `browser/net/multi.aye`, `browser/net/session.aye`, `browser/net/loader.aye`
- chromium: `net/socket/client_socket_pool_manager.cc` (6 sockets per group), `net/docs/life-of-a-url-request.md`. Take: one pool per process keyed by scheme, host and port, idle sockets kept for the next request. With libcurl: `CURLMOPT_MAX_HOST_CONNECTIONS` 6, `CURLMOPT_PIPELINING` multiplex, one connection cache.
- verify: HN `DWELLER_TIMING` `sheets` < 350 ms live (now 753); `cargo test --release --test browser -- net_http http_ > target/p4.37.log 2>&1; echo $?` → 0
- time box: 60 min; model: sonnet

#### P4.41: the W7 and W8 report
- goal: W7 and W8 are checked by one command. `window_perf.py report --replay` runs `scroll` and `idle` for every non-excluded site of `sites.json` with `DWELLER_REPLAY=target/sites/<name>`; `report --live --chrome` measures Dweller's first paint and Chrome's first contentful paint (through `agent-browser`, as in `PERF.md`), 3 alternating runs each. Both print a row per site with pass/fail and exit 1 when a row fails; the results go into `PERF.md` ("Window", a dated table).
- files: `browser/tools/window_perf.py`, `browser/engine/PERF.md`
- verify: `python3 browser/tools/window_perf.py report --replay > target/w7.log; tail -25 target/w7.log` prints 20 rows; the run is local (archives are untracked)
- time box: 60 min; model: sonnet

#### P4.42: loop priorities and the threading model
- goal: `Ui.loop` runs, in order: input, a due frame, then timers and network tasks until a 50 ms budget is spent (the rest waits for the next iteration), so a busy page cannot delay input by more than one budget. `docs/browser.md` gets a "Threading model" note: one window thread owns DOM, scripts, style, layout and paint; network is libcurl multi polled by the loop; image decode and raster may move to helper threads only when a measurement shows the need; Spaces are processes (B9).
- files: `browser/ui/window.aye`, `browser/engine/script.aye` (the task queue), `docs/browser.md`
- chromium: `docs/threading_and_tasks.md`, `third_party/blink/renderer/platform/scheduler/TaskSchedulingInBlink.md`. Take: tasks on sequences, not threads; the input queue has the highest priority; only the main thread touches the DOM. Not taken: many task queues, frame scheduling heuristics.
- verify: the perf gate's `feed.html` longest loop under budget with a timer storm added; `cargo test --release --test browser window_ > target/p4.42.log 2>&1; echo $?` → 0
- time box: 60 min; model: opus

### Login readiness and telemetry (before the user bundle)

The user's daily sites all need a login (P3.80). Audit of 2026-10-15,
read from code, for logged-in browsing in the human window:

| Area | Status | Evidence |
|---|---|---|
| Cookies persisted and restored | yes (P4.50) | Profile directory `~/.dweller` (`DWELLER_PROFILE` overrides; `browser/net/profile.aye`); the jar is written atomically within a second of a change and at exit, session cookies included, expired ones dropped on load; a second process on the profile is read-only. `DWELLER_COOKIE_FILE` still works (persistent cookies only). Test: `window_login_cookies`. |
| Cookie attributes | partial | Domain/Path/Expires/Max-Age/Secure/HttpOnly, `__Secure-`/`__Host-` checked (`cookies.aye:4-11`, `:124-151`). SameSite ignored (`cookies.aye:8`): no Lax default, every cookie is sent cross-site, no third-party policy; no public suffix list (`:11`, only a bare-TLD check at `:141`). |
| localStorage / sessionStorage / IndexedDB | yes (P4.51), no IndexedDB | `localStorage` per origin in `<profile>/storage/` (`browser/engine/storage.aye`), loaded on first use, saved like the jar, 5 MB per origin, `storage` events to the origin's other pages; `sessionStorage` per tab and origin across navigations, copied to a popup with an opener. Test: `window_login_storage`. No IndexedDB. |
| Form login and redirects | yes | Redirects followed in `net/http.aye` with each hop's `Set-Cookie` stored (`http.aye:1-6`, `:237`); 303 and 301/302-after-POST become GET (`:253-258`); POST forms (`window_forms`, `tests/browser.rs:1466`); password fields masked (`ui/forms.aye:435`, `layout/controls.aye:284`). No autofill or password manager (not planned). |
| OAuth/SSO popups | yes (P4.52) | `window.open`, `target=_blank`, `opener`, `close()`, `closed` and `postMessage` between tabs (`engine/contexts.aye`); test `window_login_popup`. Popup blocking not done. Cross-domain redirect chains work (cookies per hop). |
| HTTP Basic auth | yes (P10.22) | A 401 Basic challenge opens a sign-in sheet in the person's window (`ui/auth.aye`); credentials per origin and realm for the process, sent at once under their directory (`net/auth.aye`); URL userinfo works and is cached; agents get `auth_required`. Tests: `window_login_basic`, `window_login_basic_prompt`, `window_login_basic_cancel`, `net_basic_auth`. No Digest. |
| CSRF tokens, 2FA code pages | yes | Plain forms and scripts; nothing special once cookies and forms work. |
| WebAuthn / passkeys | no; out of scope for now | No `navigator.credentials` anywhere. The user can log in with password and a 2FA code. Unknown whether any of their sites force passkeys; if one does, their report makes it a card. |
| fetch/XHR `credentials` | partial | Parsed (`prelude.aye:1983`) but not passed to `ak.fetch` (`prelude.aye:2131-2152`): cookies always sent, `omit` ignored. No CORS enforcement (a compatibility gain, not a login blocker). |
| P9.7 shared logins with Spaces | planned, not started | R5 is "partial" (`docs/browser.md:754`): one jar and one `localStorage` per process, so a Space (its own process) does not see the person's logins; `"profile":"isolated"` is rejected until P9.7 (`docs/plans/browser.md:310`). Space profiles are therefore not separate by design but not shared yet either; P9.7 builds on P4.50/P4.51. |

IndexedDB is not filed: no daily-login blocker is known; the user's
reports decide. The fixture of P10.20 serves the other login cards.

#### P10.20: login fixture and `window_login`
- lane: browser (Layer 1)
- goal: `browser/tests/server.py` gets `/login` (GET form with a hidden CSRF token and a password field; POST checks the token, sets a session cookie and a persistent cookie on a 303 to `/account`), `/account` (200 only with the cookie, else 302 to `/login`), `/basic` (401 with `WWW-Authenticate: Basic realm="r"` until `Authorization` matches), `/sso/start` and `/sso/cb` (a redirect chain from `127.0.0.1` to `localhost` and back, each setting a cookie) and `/popup` (a page that posts a message to `window.opener`). A test `window_login` walks them through `--events` in the window; each later card turns on its assertions (the others `#[ignore]` with the card ID).
- files: `browser/tests/server.py`, `tests/browser.rs` (`window_login`), `browser/tests/login/`
- verify: `cargo test --release --test browser window_login > target/p10.20.log 2>&1; echo $?` → 0
- time box: 60 min; model: sonnet
- status: done 2026-10-15. `server.py` serves `/login` (GET: the form with a one-use CSRF token and `next`; POST: 403 for an unknown token, the form again with 401 "Wrong password", else a 303 setting `sid` (HttpOnly, SameSite=Lax, Max-Age 30 days, Secure over https) and the session-only `tab`), `/account` (the user's name with a valid `sid`, whether `tab` came; else 302 to `/login?next=/account`), `/logout` (expires both, 303 to `/login`), `/basic`, `/sso/start` → `/sso/cb` on the other host → `/sso/done`, and `browser/tests/login/` (`store.html`/`read.html`: a token in `localStorage` and a `sessionStorage` value; `stale.html`: a form with a stale token; `opener.html`/`popup.html`). `window_login` passes now: stranger redirected, wrong password, stale CSRF refused, sign-in lands on `/account`, sign-out, the two-host chain keeps each host's cookies, `/basic` 401 and 200 with URL userinfo (libcurl sends it already, so the audit's "no URL userinfo" holds only for the cache and the sheet), `localStorage` within a run. Ignored, each un-ignored by its card: `window_login_cookies` (P4.50: kill the window after sign-in, restart on `DWELLER_PROFILE`, still signed in with `tab`; then signing out is saved), `window_login_storage` (P4.51: token after restart; `sessionStorage` kept across a navigation, not a restart), `window_login_basic` (P10.22: the credentials reused for `/basic` without userinfo in the run), `window_login_popup` (P4.52: `opener.html` shows the popup's message). All four fail today.

#### P4.50: profile directory and cookie persistence
- lane: browser (Layer 0)
- goal: the window opens a profile directory (`~/.dweller`; `DWELLER_PROFILE` overrides; `dump`, `screenshot` and tests use none unless it is set; `DWELLER_COOKIE_FILE` still works). Cookies go to `<profile>/cookies` atomically (temp file, rename) after any change, at most once a second, and at exit. Session cookies are saved with a flag and restored on restart (a browser that restores its session); expired ones are dropped. A second window on the same profile does not corrupt the file.
- files: `browser/net/session.aye`, `browser/net/cookies.aye`, `browser/main.aye`, `browser/ui/window.aye`, `docs/browser.md`
- verify: `cargo test --release --test browser window_login_cookies > target/p4.50.log 2>&1; echo $?` → 0 (log in at `/login`, SIGKILL the window, start it on the same profile: `/account` is 200)
- depends on: P10.20; time box: 60 min; model: sonnet
- status: done 2026-10-15. `net/profile.aye` (directory, `flock` on `<profile>/lock`, atomic 0600 writes) and `net/session.aye` (`openProfile`, `saveCookiesIfDue` from the window's loop, at most once a second, and at exit). `~/.dweller` for a window that is not headless; `DWELLER_PROFILE` for any command but `dweller space` (Spaces keep a jar in memory until P9.7). Session cookies are saved with expiry `0` and restored; expired ones dropped on load. A second process on a held profile opens it read-only (loads, never writes). `DWELLER_COOKIE_FILE` replaces `<profile>/cookies` with the old behaviour (persistent cookies only). `window_login_cookies` un-ignored (its `--frames 100000` was over the 16384 limit: 16384 now) and checks mode 0600 and no temporary file; unit tests "net: the jar saved and loaded…" and "net: profile files are written atomically…". Documented in `docs/browser.md` (B8, Profile directory).

#### P4.51: localStorage on disk, sessionStorage per tab
- lane: browser (Layer 0)
- goal: `localStorage` is saved per origin under the profile (`<profile>/storage/`, 5 MB per origin, written after changes with the same rate limit) and loaded at start; the `storage` event fires in other pages of the origin. `sessionStorage` is kept per tab and origin across navigations and reloads, not across tabs or restart. The Spaces copy (P9.7) uses the same store.
- files: `browser/engine/window_api.aye`, `browser/engine/prelude.aye`, `browser/ui/tabs.aye`, `docs/browser.md` (drop "not saved between runs at all")
- verify: `cargo test --release --test browser window_login_storage > target/p4.51.log 2>&1; echo $?` → 0 (a page stores a token, the window restarts, a page reads it; `sessionStorage` survives a navigation, not a new tab)
- depends on: P4.50; time box: 60 min; model: sonnet
- status: done 2026-10-15. `engine/storage.aye` (moved out of `window_api.aye`): `localStorage` per origin, read from `<profile>/storage/<origin>` when a page of the origin first touches it, written atomically (0600) at most once a second after a change (`saveStorageIfDue` from the window's loop, beside the jar's), when the last page closes and at exit (`main.aye`); a read-only profile never writes (`net.profileWritable`); no profile (`dump`, tests, Spaces until P9.7): in memory as before. `sessionStorage` per top-level browsing context and origin (`contexts.aye`: dropped with the context, copied to a context opened with an opener, not with `noopener`), so it lasts across navigations and reloads of the tab. 5 MB of UTF-8 keys and values per area; past it `setItem` throws a `QuotaExceededError` DOMException. A change queues `storage` (`StorageEvent` with `storageArea`, `onstorage`) at the origin's other pages and frames sharing the area. `window_login_storage` un-ignored (and checks one storage file of mode 0600); tests `storage_quota`, `storage_between_tabs` (`browser/tests/storage/`), unit test "storage: file names and the item format". Not done: named properties (`localStorage.token`), SecurityError for opaque origins.

#### P4.52: popups, `target=_blank`, `window.opener`
- lane: browser (Layer 0)
- goal: `window.open(url, name, features)` returns a WindowProxy of a new tab (popups only after a click; a blocked one is shown in the chrome). `target=_blank` and `rel=noopener` follow the HTML rules. `window.opener.postMessage`, `window.close()` and `popup.closed` work between tabs of the process, with the origin check `postMessage` already has.
- files: `browser/engine/prelude.aye`, `browser/engine/frames.aye`, `browser/ui/tabs.aye`, `browser/ui/window.aye`
- verify: `cargo test --release --test browser window_login_popup > target/p4.52.log 2>&1; echo $?` → 0 (`/popup` posts to its opener, the opener shows the message, the popup closes)
- depends on: P10.20; time box: 90 min; model: opus
- status: done 2026-10-15. P10.20's `window_login_popup` (`login/popup.html` posts to its opener and closes) is turned on, its click moved onto the button (y 25 was the toolbar), and the test goes on with fixtures of this card (`browser/tests/popup/`, `/oauth/approve` on the test server; origins `127.0.0.1` and `localhost` of one server). Top-level browsing contexts are a process-wide registry (`engine/contexts.aye`): one per tab (person's, agent's, Space's), one per page loaded without a tab, one per popup; a window object of another tab holds the context id (stand-in generation 1), so it survives the popup's navigations, and `findPage` reaches pages of other tabs (same-origin `opener.document`). Cross-tab work is a request queue (open, navigate, close) that the window serves with tabs (`ui/tabs.aye serveContexts`, after scripts, after each agent request, each loop) and that `pumpContexts` serves in memory without a window: `dump --js` loads the popup in memory, runs its scripts and follows their navigations, and the page's dump shows what it was posted (deterministic: the pump runs before and after each task of `runUntilIdle`). Also fixed: the window now follows a navigation that a page's script asks for while it loads (it was dropped). `--state` prints `tabs N, showing K` without agents once there are two tabs. Not done: popup blocking (the card's "popups only after a click"), background tabs' timers (a message to a background opener waits until its tab is shown; the popup's close shows it), `window.open` features (size, position), `opener` of a frame. Tests `window_login_popup` (OAuth round trip in the window and in `dump --js`, `_blank` without opener, `rel=opener`, named reuse by window.open and a link) and the `window_agents` session `popup` (an agent's `_blank` link and window.open open agent tabs, the person's tab stays shown).

#### P10.21: cookies: SameSite, third-party policy, credentials
- lane: browser (Layer 1, fetch/storage)
- goal: `SameSite` is parsed; a missing one means Lax; `None` needs Secure. Lax and Strict cookies are not sent on cross-site subresource requests or cross-site POST navigations; Lax is sent on top-level GET navigations. Cookies from a third-party response are not sent when the top-level site differs, except `SameSite=None; Secure`. A pinned public suffix list rejects `Domain=co.uk`. `fetch`/XHR honour `credentials` and `withCredentials` (`omit` sends and stores nothing). `document.cookie` limit of 4096 bytes.
- files: `browser/net/cookies.aye`, `browser/net/http.aye`, `browser/engine/prelude.aye`, `browser/net/psl.txt`
- verify: `cargo test --release --test browser net_cookies window_login > target/p10.21.log 2>&1; echo $?` → 0; `python3 browser/tools/wpt.py run cookies -j 8` does not regress and `python3 browser/tools/conformance.py check; echo $?` → 0
- depends on: P10.20; time box: 90 min; model: opus

#### P10.22: HTTP authentication
- lane: browser (Layer 1, fetch)
- goal: a 401 with `WWW-Authenticate: Basic` shows a sign-in sheet in the window (Cancel shows the 401 page); the answer is cached per origin and realm for the process and sent preemptively; `http://user:pass@host/` URLs send the credentials. Digest only if libcurl gives it for free; proxy auth out of scope.
- files: `browser/net/http.aye`, `browser/net/session.aye`, `browser/ui/chrome.aye`, `browser/ui/window.aye`
- verify: `cargo test --release --test browser window_login_basic > target/p10.22.log 2>&1; echo $?` → 0 (`/basic` is 200 after the credentials; wrong ones show the sheet again)
- depends on: P10.20; time box: 60 min; model: sonnet
- status: done 2026-10-15. `net/auth.aye`: the credentials per origin and realm for the process (never written to the profile), sent at once (`Authorization: Basic`, unless the request has its own) to the origin's paths under the directory they were given for; a document 401 naming a kept realm on another path is asked again once with them, widening the directory to the common prefix (RFC 7617 2.2); a 401 to credentials sent drops them; URL userinfo that the server took is kept with the realm unknown (`window_login_basic` passes). `ui/auth.aye` and `ui/chrome.aye drawSheet`: the sign-in sheet under the toolbar for the person's navigations (not in a Space, not for agent tabs, not in `dump`): Username, Password masked, Cancel, Sign in; the focus stays in it; Enter signs in (reload of the entry), Escape or Cancel leaves the 401 page; a wrong password asks again, the username kept. The address bar leaves URL userinfo out. Agents: the response of an action that shows a Basic 401 page has `auth_required` (`{"scheme":"Basic","realm":...}`; MCP: a line), documented in `docs/agent-protocol.md`. Tests `window_login_basic_prompt` (sign in, wrong password, the focus held; golden `browser/tests/ui/auth_prompt.png`), `window_login_basic_cancel` (Escape, Cancel, Sign in by click, address without userinfo), `net_basic_auth` (agent `auth_required`, userinfo, the cache; `dump` shows the 401 page). Not done: Digest, proxy authentication, the prompt for frames and for subresources (cached credentials only).

#### P10.23: sign-in page details
- lane: browser (Layer 1, forms and JS)
- goal: sign-in pages do not break on what Dweller lacks. `autocomplete` values are accepted and ignored; `navigator.credentials.get()` rejects with `NotSupportedError` and `PublicKeyCredential` is absent, so pages offer the password path (passkeys are out of scope for now); `Set-Cookie` on `fetch`/XHR responses reaches the jar and a following `document.cookie` (check; fix if not); Enter in a field submits; `requestSubmit()` and a `submit` handler with `preventDefault` plus `fetch` login work; a password value is never in the agent tree or the telemetry.
- files: `browser/engine/prelude.aye`, `browser/ui/forms.aye`
- verify: `cargo test --release --test browser window_login_forms > target/p10.23.log 2>&1; echo $?` → 0
- depends on: P10.20; time box: 60 min; model: sonnet

#### P4.53: local telemetry
- lane: browser (Layer 0)
- goal: the window and `dump` append events to `~/.dweller/telemetry/YYYY-MM-DD.jsonl` (each file capped at 5 MB, rotating to `-1`, `-2`; the oldest deleted past 30 files). Local only, no upload; a remote sink such as Sentry is a later product decision. Off: `DWELLER_TELEMETRY=0` or a settings flag. Events: crash (signal, backtrace, the last few window actions); uncaught JS error (message, type, script origin and line); unsupported feature hit (unknown CSS property or value, missing DOM/JS API such as a ReferenceError on a global like `SVGAElement`, unimplemented method); loop iteration over 100 ms with the phase breakdown (network wait, style, layout, paint, script) from `DWELLER_TIMING` (PR #23); per-origin first paint and load time and scroll frame p50/p95; failed resource load (kind, status, origin); image decode failure. Privacy: scheme and host only, never a path, query, page text, form value, cookie or header; messages cut to 200 characters with every URL reduced to its origin. `browser/tools/telemetry.py summary [--days N]` groups events and ranks issues by frequency times sites affected, as input for new cards. The user bundle ships with it on.
- files: `browser/perf/`, `browser/engine/script.aye`, `browser/css/` (unknown property hook), `browser/engine/prelude.aye` (missing API), `browser/ui/window.aye`, `browser/tools/telemetry.py`, `docs/browser.md`
- verify: `cargo test --release --test browser telemetry > target/p4.53.log 2>&1; echo $?` → 0: a test-server page triggers a JS throw, an unknown CSS property, a missing API, a slow script, a 404 image and a corrupt image; the day file holds one record of each; a grep for `?`, a path and the page's text finds nothing; `python3 browser/tools/telemetry.py summary --days 1` lists all six
- time box: 120 min; model: opus
- status: done 2026-10-15. `perf/telemetry.aye` (no allocation; a fixed buffer written every 5 s, at exit and by the crash handler). Records: the person's window and its Spaces (`DWELLER_TELEMETRY_DIR`, set by the window) and any run with `DWELLER_PROFILE` (`dump` too); headless test windows and `dump` without a profile record nothing, so no golden changed. Off: `DWELLER_TELEMETRY=0` or `telemetry=off` in `<profile>/settings`. Events `crash`, `js_error` (line: the last statement run, via a store per statement in `exec` and a table of parsed sources in the interpreter), `unsupported` (`css_property`, `css_value`, `js_global`, `js_method`), `slow_loop` (phases summed from the `DWELLER_TIMING` ones, which are now timed while telemetry is on), `page`, `scroll`, `load_failed`, `image_decode`; format and privacy rules in `docs/browser.md` "Telemetry". Test `window_telemetry` (also SIGSEGV with the last actions, `DWELLER_TELEMETRY=0`, a test window without a profile under a temporary `HOME`).

### Layer 0 backlog

Not yet cards. Before one becomes a card, main checks it is still needed
after the cards above.

- P4.54 macOS crash record: install the telemetry crash handler on macOS so it writes the record and the process exits with 128+sig; today the handler runs but the process stays alive (SDL/Cocoa signal handling suspected); verify: `window_telemetry` SIGSEGV step on macOS (P4.53).
- P3.64 slice 2: in a container-query document, restyle subtrees when no container's size changed; skip the whole-document `stylesCurrent` hash for attribute writes (details in `PERF.md`).
- P4.11 partial restyle for DOM mutations incl. shadow roots and `:has()` (P4.10).
- P4.14 event dispatch and MutationObserver from prelude JS into native Aye (~15% of JS samples) (P1.11).
- JS engine speed: P4.21 GC threshold factor (GC 14.5% of JS samples); P4.24 smaller JS objects, property storage that shrinks, GC threshold relative to live size; P4.28 slab for unshared computed-style arrays; P4.29 `Prop` 48 → 32 B, accessors stored separately; P4.30 decision: the next large JS speedup is a bytecode compiler, design doc first.

## Parallel track: P9 agents without disturbing the user

Design, measurements and acceptance checks R1–R7: `docs/browser.md`,
B9. The user asked for this on 2026-10-14 and counts it as Dweller's
core value; it runs beside Layer 0 (its own files: `browser/ui/spaces.aye`,
the agent code). Cards are ordered so that R1
(isolation) and R2 (a responsive window) come first. Each card is one
PR; its tests go in `tests/browser.rs` in the style of `window_agents`
(a headless window, `SDL_VIDEODRIVER=dummy`, `--agent-socket`, scripted
`--events`, the responses and `--state` against a golden `.out`).
Common to all cards: run only the tests named in verify, never two
browser test runs at once; the full suite runs once on `main` after the
merge.

| Card | Requirement | One line | Verify |
|---|---|---|---|
| P9.1 | R1 | A connection acts on a tab of its own by default; the person's tabs are look-only; no `show`; the tab strip row stays while the window listens | `cargo test --release --test browser window_agents` |
| P9.2 | R2 | `dweller space`: a Space process per connection behind `--spaces`; the window copies bytes; `loop_max_ms` in `--state` | `cargo test --release --test browser window_spaces` |
| P9.3 | R4, R6 | The control connection: pause, resume and stop per Space, `user_context` and `user_pointed` relayed, Space status, `spaces`, `space_close`, the hello's `name`/`space`, `done` Spaces | `cargo test --release --test browser window_spaces` |
| P9.4 | R6 | Space chips in the toolbar's agent area, their state and last action, per-Space Pause and Stop | `cargo test --release --test browser window_spaces` |
| P9.5 | R6 | Watching: the Space's frames shown in the page area, Cmd+Shift+0..8 | `cargo test --release --test browser window_spaces_watch` |
| P9.6 | R6 | Take over and hand back: the person's input goes to the shown Space and takes that tab; events `taken_over`, `handed_back` | `cargo test --release --test browser window_spaces` |
| P9.7 | R5 | Shared logins: cookies and `localStorage` copied both ways; `"profile":"isolated"` | `cargo test --release --test browser window_spaces_logins` |
| P9.8 | R7 | Code mode: `run` with its functions in a realm of its own | `cargo test --release --test browser -- agent_sessions window_spaces` |
| P9.9 | R3, R4 | Spaces by default; three agents in parallel; the in-process acting path removed; protocol docs | `cargo test --release --test browser window_` |

P9.8 depends only on P9.2 and may run beside P9.3–P9.7. The others run
in order.

#### P9.1: agents never act on the person's tabs
- lane: browser
- goal: R1 in today's single process. A connection that has not opened or selected a tab gets its own background tab on its first command that needs a page (no longer the shown tab). Acting commands (`ACTING` in `agent/shared.aye`) on a tab the person has not given the agent fail with "this is the user's tab: open your own with tab_new"; looking (`snapshot`, `read`, `find`, `user_context`) still works. `show` in `tab_new`/`tab_select` is refused ("the user chooses what they see"). While the window listens for agents, the tab strip row is always there, so the page viewport never changes height because an agent opened or closed a tab. `afterAgent` lays out the shown tab only when the request touched it.
- serves: R1
- files: `browser/ui/agents.aye`, `browser/ui/tabs.aye`, `browser/ui/window.aye` (`top`), `browser/agent/shared.aye`, `browser/agent/server.aye` (MCP instructions), `browser/tests/agent/window/isolation.{jsonl,events,out}` (new), `shared.out` and `handshake.mcp.out` re-blessed, `docs/agent-protocol.md` (A shared window)
- test: `isolation.events` clicks into a text field of a long page, types, selects with shift+left and scrolls (`wheel`); `isolation.jsonl` checks `user_context`, then `tab_new`, `navigate`, `type`, `click`, `scroll`, `eval`, `tab_close`, a `type` on the person's tab (refused), `tab_new` with `show` (refused), and `user_context` again: the same `scroll`, `selection` and `focused`; the `.out` window state shows the person's URL, scroll, focus and `tabs N, showing 1`.
- verify: `cargo test --release --test browser window_agents > target/p9.1.log 2>&1; echo $?` → 0
- time box: 60 min; model: sonnet
- done 2026-10-14: as specified. Choices recorded in `docs/agent-protocol.md` (A shared window): a tab is an agent's when an agent opened it (any connection may act on it), else the person's; the commands that need a page (and so open the own tab) are `navigate back forward reload snapshot read find click type select press scroll eval screenshot wait`; on a person's tab `screenshot` and `wait` also work; `"show":false` is accepted; `tab_select` of a person's tab answers "looking at tab N (the user's tab: you can only look)". `--state` gains `viewport WxH` while agents listen (the R1 test checks the page height with it). The person in `shared`, `handshake.mcp`, `pause` and `frames` opens a page or watches the agent's tab by scripted events, so those goldens still show pointing, frames and focus.

#### P9.2: a process per Space
- lane: browser
- goal: R2. With `--spaces`, the window starts `dweller space --control FD` (the window program, headless with `SDL_VIDEODRIVER=dummy`, its agent socket in a private directory) for each connection after reading its first line, then copies bytes between the connection and the Space's socket without parsing them; the Space exits when its connection ends (P9.3 keeps it). The Space runs its own tabs' timers. The window's `--state` gets `loop_max_ms N`: the longest iteration of `Ui.loop` while an agent was connected. `browser/tests/server.py` gets `/sleep?ms=N`.
- serves: R2
- files: `browser/ui/socket.aye` (socketpair, a non-blocking copy), `browser/ui/agents.aye`, `browser/ui/window.aye` (`--spaces`, `space` mode, `loop_max_ms`), `browser/main.aye`, `browser/ui/script.aye`, `browser/tests/server.py`, `browser/tests/agent/spaces/` (new: copies of `isolation` and `frames` that must give the same `.out`), `tests/browser.rs` (`window_spaces`, `window_spaces_responsive`)
- test: `window_spaces` runs `browser/tests/agent/spaces/X.jsonl` like `window_agents` with `--spaces`. `window_spaces_responsive` opens three connections: A runs a 1500 ms `eval` loop, B navigates to `/sleep?ms=1500` on the test server, C sends `tabs` while both run: C's answer within 100 ms, and `loop_max_ms` under 50 in the window's `--state`.
- verify: `cargo test --release --test browser window_spaces > target/p9.2.log 2>&1; echo $?` → 0
- risk: the Space must die with the window (it gets an end of file on the control connection and exits) and a crashed Space must not stop the window (its socket is closed, the connection gets `{"id":null,"ok":false,"error":"the Space crashed"}`); the test kills a Space with SIGKILL and checks both.
- time box: 120 min; model: opus (process lifecycle)
- done 2026-10-14: as specified, in `browser/ui/spaces.aye`. Choices recorded in `docs/agent-protocol.md` (Spaces): a Space cannot see the person's tabs until P9.3 relays the context (`user_context` fails with "the user's context is not available in a Space yet"; `tabs` lists only the Space's tabs, `shown` false), so the `spaces/` copies of `isolation` and `frames` answer the agent's own work exactly as in the window (`frames` lines 1-5 are the same) but differ where the agent looked at the person's tab, and the window part shows the person's tab untouched; a third copy, `handshake.mcp`, runs MCP through a Space. A Space's tab 1 exists from the start and is its agent's. Pause, pointing and the activity line do not reach Spaces yet (P9.3, P9.4). The Space's socket is `/tmp/dweller-WINDOWPID-XXXXXX/agent.sock` (a 0700 directory per Space, removed by the Space, or by the window for a crashed one); the tests find Spaces with `pgrep -f` on that prefix. Spaces start with `posix_spawnp` (the runtime's fork took 5 to 15 ms of the window's loop per Space); the window's sockets are FD_CLOEXEC, so a Space holds no other connection open. While it relays, the window waits at most 4 ms for events; a Space waits on its sockets with `poll` and runs all its tabs' timers. `loop_max_ms` is in `--state` whenever the window listens (also without `--spaces`, where the same load gives about 3000); the window goldens drop that line. An MCP connection whose Space crashed gets a `notifications/message` of level `error`. The test is `window_spaces_responsive` (A: 1.5 s `eval`, B: `/sleep?ms=1500`, C: `tabs` within 100 ms, the person clicks and types while they run, `loop_max_ms` under 50; without `--spaces` at least 1000) and `window_spaces_crash` (SIGKILL of a Space: its agent gets the error and EOF, the other agent goes on, the window exits 0; SIGKILL of the window: its Space exits and removes its directory); every `window_spaces` session checks that no Space is left.

#### P9.3: the control connection
- lane: browser
- goal: the window and its Spaces talk over the control connection (B9 table): `pause`/`resume`/`stop`, `context` (sent when the person's context changes), `pointed`, `status`. Cmd+Shift+P pauses every Space when the person's tabs are shown. New commands `spaces` and `space_close`; the hello takes `name`, `space` and `profile` (only `shared` until P9.7); a Space whose agent left stays `done` and can be attached to; events `paused`, `resumed`, `stopped`. `--state` lists `space ID "NAME" STATE: LAST`.
- serves: R4, R6
- files: `browser/ui/agents.aye`, a new `browser/ui/spaces.aye` (the window's side), `browser/agent/shared.aye` (the commands and events), `browser/ui/socket.aye` (`connect --name/--space`), `browser/tests/agent/spaces/{pause,pointing,attach}.*`, `tests/browser.rs`
- verify: `cargo test --release --test browser window_spaces > target/p9.3.log 2>&1; echo $?` → 0
- time box: 90 min; model: sonnet
- done 2026-10-15: as specified, in `browser/ui/spaces.aye` (both sides). Choices recorded in `docs/agent-protocol.md` (Spaces): the window keeps a list of Spaces (at most 8; a new one replaces the oldest done or crashed one) apart from the connections; a Space whose agent leaves stays `done` (its process too) until an agent attaches (`"space":N`; it acts on the tab the last agent acted on), the person stops it, or the window closes. The window never waits on a control connection: it reads when readable and keeps what the socket does not take (`ctl_out`); the list of Spaces and the person's context are sent only when that is empty, newest only. The window sends each Space `spaces` (its own id and name, and the others' entries and MCP lines) whenever a Space comes, goes or its status changes; `spaces` answers the asking Space first (always `working` or `paused`), then the others as last heard. The context goes when it may have changed (after the person's input, a load, page timers), at most every 100 ms, computed without a new layout of the person's page except for a new document (refs need no layout); Cmd+Shift+K sends the context at once, then `pointed`. A Space sends `status` before and after each request (before the answer, so the window learns it before the agent its answer), on `wait`'s end, an agent coming or going, pause; it reads the control connection before each request and while waiting, so pause and stop count from the next request. Stop: the Space sends `stopped`, quits; the window kills it after 1 s if it does not exit (a long `eval`: no event then). Cmd+Shift+P and the Pause button pause every Space (the person's tabs are always shown until P9.5); a Space started while paused starts paused without an event. Without `--spaces`: `spaces` answers "0 spaces", `space_close` fails, the hello's `name`/`space`/`profile` are ignored. The hello is any first-line object without `cmd`/`jsonrpc`; a bad one (`profile` `isolated` until P9.7, an unknown or attached Space) gets an error line and the end. Per-Space pause and stop have no button until P9.4: the tests use the new `--events` words `pause:N`, `resume:N`, `stop:N`. `waited N ms` replaces wait times in the `spaces/` goldens; `spaces/handshake.mcp` gained a `browser_wait` so that its pointing reaches the Space deterministically. Fixed on the way: a Space's end of the control connection had no SO_NOSIGPIPE (a status to a killed window would kill it with SIGPIPE before it removed its directory). Tests: `window_spaces_control` (two agents: pause/resume/stop one while the other works, `spaces`, attach, attach errors, `space_close`, no Space left) and `spaces/{pause,pointing,attach}`.

#### P9.4: Space chips
- lane: browser
- goal: the toolbar's agent area shows "You" and one chip per Space (dot by state, the name; the status bar shows the last action while the mouse is over a chip); Pause/Resume and Stop act on the shown Space. No new row: nothing in the window changes height.
- serves: R6
- files: `browser/ui/chrome.aye`, `browser/ui/spaces.aye`, `browser/ui/agents.aye`, `browser/tests/agent/spaces/chips.*`, a golden PNG of the toolbar with two Spaces (as the B4 window PNGs)
- verify: `cargo test --release --test browser window_spaces > target/p9.4.log 2>&1; echo $?` → 0
- time box: 60 min; model: sonnet
- done 2026-10-15: in `browser/ui/chrome.aye` (layout, hit testing), `browser/ui/agents.aye` (drawing, menu, hover) and `browser/ui/spaces.aye`. Choices recorded in `docs/agent-protocol.md` (Spaces, Chips): with `--spaces` the chips replace the `AI` badge, left of the Pause button, which still pauses every Space; the address bar ends before the chips (at least 160 px; long names are cut), so nothing changes height. A click on a chip opens a menu under it, over the page (Pause/Resume and Stop; Close for a done or crashed Space); the click that closes it does nothing else; Escape closes it. A stopped Space's chip goes when its process ends (grey until then). The status line shows a Space's new last action (`NAME: LAST`), what the person did to one (`Alpha paused`), and `NAME (STATE): LAST` while the mouse is over its chip; `--state`'s `status` shows the latter. A Space now sends its `status` before the `paused`/`resumed` event, so the window knows the state before the agent. `--save` draws the last frame again when the last events changed it (Cmd+Q right after events). The `spaces/` goldens drop the `activity` line (its order varies; the `space` lines keep each last action). Fixed on the way: with `--spaces`, the first context sent to the Spaces laid the person's page out again (a new document's refs) while the window kept its display list of the old layout, so the person's page showed garbage glyphs (`syncSlot` now has the window list the shown page again). The card's `spaces/chips.*` session is the Rust test `window_spaces_chips` instead (three agents; a session file has one); its golden PNG is `browser/tests/ui/spaces_chips.png` (Alpha paused, Beta idle under the mouse).

#### P9.5: watching a Space
- lane: browser
- goal: a click on a chip (or Cmd+Shift+1..8; Cmd+Shift+0 for "You") shows the Space: the window sends `watch`, the Space writes its canvas to its frame file after each frame it draws (at most 30 a second) and sends `frame`, and the window draws the newest frame in its page area. The Space's window takes the size of the window's page area.
- serves: R6
- files: `browser/ui/spaces.aye`, `browser/ui/window.aye` (draw), `browser/ui/socket.aye` (`mmap`/`ftruncate` externs), `tests/browser.rs` (`window_spaces_watch`)
- test: an agent navigates its Space to a page whose body is `#00aa00`; the events click its chip; `--save` writes the window's frame, and the pixel at the centre of the page area is that green; `loop_max_ms` stays under 50.
- verify: `cargo test --release --test browser window_spaces_watch > target/p9.5.log 2>&1; echo $?` → 0
- time box: 90 min; model: opus (shared memory between processes)

#### P9.6: take over and hand back
- lane: browser
- goal: while a Space is shown, the person's clicks, keys, text and wheel go to it as `input` (words of `--events`, in the Space's coordinates); the first one on a tab takes that tab over: the Space's actions on it fail with "the user took over tab N (wait for handed_back)", the agent gets `taken_over`; Hand back (agent area) sends `hand_back`, the agent gets `handed_back`. The Space's other tabs and the other Spaces go on.
- serves: R6
- files: `browser/ui/spaces.aye`, `browser/ui/agents.aye` (a taken-over flag per tab, refusal), `browser/ui/script.aye` (input words from SDL events), `browser/tests/agent/spaces/takeover.*`
- verify: `cargo test --release --test browser window_spaces > target/p9.6.log 2>&1; echo $?` → 0
- time box: 90 min; model: sonnet

#### P9.7: shared logins
- lane: browser
- goal: R5. A `shared` Space starts with the window's jar (persistent and session cookies) and `localStorage`; each side sends every change (`cookie` after `Jar.store`, `storage` after a store write) and applies the other's. `"profile":"isolated"` starts empty and sends nothing.
- serves: R5
- order: after P4.50 (the persistent profile jar) and P4.51 (the stored `localStorage`); part of login readiness before the user bundle (P3.80).
- files: `browser/net/cookies.aye` (a hook after `store`, a save that includes session cookies), `browser/net/session.aye`, `browser/engine/window_api.aye` (a hook after store writes), `browser/ui/spaces.aye`, `tests/browser.rs` (`window_spaces_logins`)
- test: over the test server, the person's tab opens `/net/start` (sets cookies); a Space navigates `/headers` and its Markdown shows those cookies; the Space sets a cookie with `eval` (`document.cookie`) and a `localStorage` item, and the person's next `/headers` and a script on the person's page see them; an `isolated` Space sees none.
- verify: `cargo test --release --test browser window_spaces_logins > target/p9.7.log 2>&1; echo $?` → 0
- time box: 90 min; model: opus (consistency between processes)

#### P9.8: code mode (`run`)
- lane: browser
- goal: R7. `run` executes `code` as a function body in a realm of its own with the functions listed in B9, each a synchronous call into the session's command; it answers `value`, `log`, `calls`; pause and stop are checked between calls; `ms` bounds the run. It works in `dweller agent`, `dweller mcp` (`browser_run`) and in Spaces.
- serves: R7
- files: `browser/agent/run.aye` (new), `browser/agent/server.aye`, `browser/js/` (a realm with host functions), `browser/tests/agent/sessions/run.{jsonl,out}`, `browser/tests/agent/spaces/run.*`
- first: a unit test that a second realm calls into a page's realm and back (a click whose handler runs page script) under GC stress; if it fails, stop and report (B9 risks).
- verify: `cargo test --release --test browser -- agent_sessions window_spaces > target/p9.8.log 2>&1; echo $?` → 0
- time box: 120 min; model: opus

#### P9.9: Spaces by default
- lane: browser
- goal: R3 and R4 as the default. `--spaces` is the default; the in-process acting path goes (the person's tabs stay look-only from P9.1); at most 8 Spaces; `docs/agent-protocol.md` and the MCP instructions describe Spaces; `browser/tests/agent/window/` fixtures move to `spaces/` where they still apply. New `window_spaces_parallel`: three connections in three Spaces navigate, type and read at once; each one's output equals its output alone; pausing one leaves the others working; the person's tab strip still shows one tab.
- serves: R3, R4
- files: `browser/ui/agents.aye`, `browser/ui/window.aye`, `docs/agent-protocol.md`, `docs/browser.md` (B7, B9 status), `tests/browser.rs`
- verify: `cargo test --release --test browser window_ > target/p9.9.log 2>&1; echo $?` → 0
- time box: 90 min; model: sonnet


## Layer 1: P10, components measured by WPT and test262

**Suites and revisions.** WPT (web-platform-tests) for HTML, DOM, CSS,
images, events, fetch/XHR, URL and encoding; test262 for JS. Neither is
vendored. Both are fetched at a pinned commit into `target/` (untracked)
and cached in CI by that commit:

- WPT: `web-platform-tests/wpt` at the commit in `browser/tools/wpt.rev`
  (first pin: `929285422eb40f1b91bec40e70670eb27ffe8595`, 2026-10-15), a
  sparse checkout of the area directories plus `resources/`, `fonts/`,
  `common/` and `tools/`. The Chromium checkout's copy
  (`/Users/solemnis/Documents/chromium/src/third_party/blink/web_tests/external/wpt`,
  read-only, no recorded upstream commit) can be used locally with
  `--wpt-root` to explore; the committed table always comes from the pin.
- test262: `tc39/test262` at the commit in `browser/tools/test262.rev`
  (first pin: `d53ebc8b1931d9f1541e43d59650776e4b766248`, 2026-10-13).
  The Chromium checkout has no test262 data (`v8/test/test262/data` is
  empty).
- Why fetch, not vendor: test262 is about 50,000 files and the WPT areas
  below thousands more; vendoring them would dwarf the repository and
  make every update a huge diff. Both licenses (BSD-3) allow vendoring,
  so the small harness files the runner unit tests need
  (`testharness.js`, test262 `assert.js`, `sta.js`,
  `doneprintHandle.js`) are copied under `browser/tests/wpt/` and
  `browser/tests/test262/` with their license.

**Score.** As wpt.fyi: each test file scores passed subtests / subtests
(testharness) or 1/0 (reftest, crashtest); a harness error or timeout
scores 0; an area's score is the mean over its files, in percent.
test262: passed / run, per file and mode (strict and sloppy unless
flagged). Skipped tests are listed with their reason and are not in the
denominator.

**Excluded from the first subsets** (out of scope now): WebGL, WebGPU,
service workers, workers, WebRTC, WebXR, Web Audio, media source and EME,
WebCodecs, WebUSB/Bluetooth/HID/Serial, payments, notifications and
push, fenced frames, speculation rules, WebTransport, Houdini
(`css-paint-api`, `css-layout-api`), vertical writing modes, multicol,
paged media (`CSS2/pagination`, `css-page`, `css-break`), AVIF and APNG;
files named `*-manual.*`, `*.https.*`, `*.tentative.*`; `.any.js`
variants other than the window one; `*.sub.*` and tests with Python
handlers until P10.5; tests that load `testdriver.js` until P10.6 (216
of them in `uievents` and `html/semantics/forms`). test262: `intl402`,
`staging`, and the features in `browser/tools/test262_skip.txt`
(`Atomics`, `SharedArrayBuffer`, `Temporal`, `ShadowRealm`,
`decorators`, `IsHTMLDDA`, `tail-call-optimization`, and any feature
the first run shows Dweller lacks entirely, each with a line why).

**Areas, first subsets and thresholds.** Counts are files in the
Chromium copy before exclusions (testharness / reftest). The threshold
is the score at which an area is "done for now": its component cards
stop and work moves up a layer or to the next area. P10.4 replaces
the counts with the measured ones; thresholds change only by a recorded
decision.

| Area | WPT directories (first subset) | Files (th / ref) | Threshold |
|---|---|---:|---:|
| HTML parsing | `html/syntax/parsing` (html5lib tree tests) | 105 / 0 | 90 |
| DOM | `dom/nodes`, `dom/traversal`, `dom/ranges`, `dom/collections`, `dom/lists`, `html/dom` | 585 / 130 | 75 |
| Events and timers | `dom/events`, `uievents`, `html/webappapis/timers`, `html/webappapis/scripting/event-loops` | 248 / 0 | 70 |
| CSS selectors | `css/selectors` | 256 / 224 | 80 |
| CSS cascade and variables | `css/css-cascade`, `css/css-variables`, `css/CSS2/cascade*` | 135 / 271 | 75 |
| CSS values, units, color | `css/css-values`, `css/css-color` | 294 / 484 | 70 |
| CSS box model and sizing | `css/css-box`, `css/css-sizing`, `css/css-display`, `css/CSS2/normal-flow` | 279 / 1463 | 70 |
| CSS floats and tables | `css/CSS2/floats`, `css/CSS2/floats-clear`, `css/CSS2/tables`, `css/css-tables` | 134 / 841 | 65 |
| CSS inline and text | `css/css-inline`, `css/css-text`, `css/CSS2/linebox`, `css/CSS2/text` | 409 / 2093 | 60 |
| CSS overflow | `css/css-overflow` | 204 / 475 | 60 |
| CSS flexbox | `css/css-flexbox` | 346 / 933 | 70 |
| CSS grid | `css/css-grid` | 606 / 1194 | 60 |
| CSS positioning | `css/css-position`, `css/CSS2/positioning`, `css/CSS2/abspos`, `css/CSS2/stacking-context` | 105 / 777 | 70 |
| CSS backgrounds and borders | `css/css-backgrounds`, `css/css-borders`, `css/CSS2/backgrounds`, `css/CSS2/borders` | 188 / 1565 | 65 |
| CSS transforms (2D) | `css/css-transforms` without 3D tests | 104 / 772 | 50 |
| CSS fonts | `css/css-fonts` | 157 / 314 | 50 |
| Forms | `html/semantics/forms` | 417 / 108 | 60 |
| Images | `html/semantics/embedded-content/the-img-element`, `css/css-images`, `png`, `gif`; PngSuite (decoders, outside WPT) | 140 / 476, plus about 60 in `png`, `gif` | 60 (PngSuite 100) |
| URL and encoding | `url`, `encoding` | 184 / 3 | 80 |
| Fetch and XHR | `fetch/api`, `xhr` (P10.5) | 501 / 0 | 50 |
| JS | test262 `test/language`, `test/built-ins`, `test/annexB` minus skips | about 45,000 (P10.3 measures) | 80 |

The reftest-heavy CSS areas take the longest (about 11,400 screenshot
pairs in all); the weekly run is parallel (`-j`) and a card runs only
its area.

#### P10.1: WPT testharness runner
- goal: run WPT testharness.js tests in Dweller headless and report each subtest's status.
- design: `browser/tools/wpt.py` (standard library only): `fetch` (sparse checkout at `wpt.rev` into `target/wpt`); `serve` (a static server on 127.0.0.1: files, `.headers` sidecars, generated wrappers for `.any.js`/`.window.js` with their `// META: script=` and `timeout=long` lines, and our `resources/testharnessreport.js`, which registers `add_completion_callback` and writes one `DWELLER_WPT {json}` console line); `run AREA|PATH... [-j N] [--wpt-root DIR]` (areas from `browser/tools/wpt_areas.json`: directories, excludes, threshold), results to `target/wpt-results/<area>.json`. A new CLI mode `dweller wpt URL [--timeout S]` loads the page with scripts as `dump --js` does, runs the event loop until the completion line or the timeout (10 s, 60 s for `timeout=long`), and prints one JSON line: harness status (`OK`, `ERROR`, `TIMEOUT`, `CRASH`) and each subtest's name, status (`PASS`, `FAIL`, `TIMEOUT`, `NOTRUN`) and message.
- files: `browser/tools/wpt.py`, `browser/tools/wpt.rev`, `browser/tools/wpt_areas.json`, `browser/tools/wpt/testharnessreport.js`, `browser/main.aye` (`wpt` mode), `browser/engine/` (the wait), `browser/tests/wpt/` (a vendored `testharness.js` with its license; fixtures: all pass, one failing subtest, a harness error, a timeout, an `.any.js`), `tests/browser.rs` (`wpt_runner`)
- verify: `cargo test --release --test browser wpt_runner > target/p10.1.log 2>&1; echo $?` → 0; `python3 browser/tools/wpt.py fetch && python3 browser/tools/wpt.py run dom/nodes -j 8 > target/wpt-dom.log 2>&1; tail -3 target/wpt-dom.log` prints the area score and no runner exception
- depends on: nothing (parallel with P4 cards: it touches `browser/tools/`, `main.aye` and a new engine entry point)
- time box: 120 min; model: opus (the wait and the report protocol), sonnet for the server

#### P10.2: reftest and crashtest runner
- goal: reftests compare the test's screenshot with its reference's; crashtests pass when the page loads without a crash or timeout.
- design: `wpt.py run` detects `<link rel=match|mismatch href=...>` (a reference's own reference followed one level), `<meta name=fuzzy content=...>` (maxDifference, totalPixels), the `reftest-wait` class (new `dweller screenshot --wait-class reftest-wait`, at most 5 s), and files under `crashtests/` or named `*-crash.*`. Screenshots at 800×600 (WPT's reftest size) with `--font fixed` off; the Ahem font from `target/wpt/fonts/` is a system family through a new `DWELLER_FONT_DIRS` (WPT CSS tests name `font-family: Ahem` without `@font-face`). Pixel comparison in the tool (a small PNG decoder over `zlib`).
- files: `browser/tools/wpt.py`, `browser/main.aye` (`screenshot --wait-class`), `browser/font/` (`DWELLER_FONT_DIRS`), `browser/tests/wpt/` (fixtures: match, mismatch, fuzzy, reftest-wait, crashtest), `tests/browser.rs` (`wpt_reftest`)
- verify: `cargo test --release --test browser wpt_reftest > target/p10.2.log 2>&1; echo $?` → 0; `python3 browser/tools/wpt.py run css/css-flexbox -j 8 > target/wpt-flex.log 2>&1; tail -3 target/wpt-flex.log` prints the score
- depends on: P10.1
- time box: 90 min; model: sonnet

#### P10.3: test262 runner
- goal: test262 runs through the existing `dweller js` runner with the harness includes.
- design: `browser/tools/test262.py` (standard library only): `fetch` (at `test262.rev` into `target/test262`); `run [PATH...] [-j N]`: reads each test's front matter (`includes`, `flags`: `onlyStrict`, `noStrict`, `module`, `raw`, `async`; `negative`: phase and type; `features`), skips features in `test262_skip.txt`, writes a temporary file of `harness/assert.js`, `sta.js`, `doneprintHandle.js` (async), the includes and the test (with `"use strict";` for the strict run; `.mjs` for modules), runs `dweller js --test262 FILE` with a 10 s timeout. Pass: exit 0 with no uncaught error; async: `Test262:AsyncTestComplete` printed; negative: the expected error type, at parse time for `phase: parse` (nothing printed before it). `--test262` adds the host hooks: `print`, and `$262` with `global`, `evalScript`, `gc`, `detachArrayBuffer` (`createRealm` when P9.8's realms exist; `agent` stays skipped with `Atomics`). Results to `target/test262-results/<dir>.json`.
- files: `browser/tools/test262.py`, `browser/tools/test262.rev`, `browser/tools/test262_skip.txt`, `browser/main.aye` and `browser/js/` (`--test262` host hooks), `browser/tests/test262/` (vendored harness files with license; fixtures: pass, fail, negative parse, async, strict only, module), `tests/browser.rs` (`test262_runner`)
- verify: `cargo test --release --test browser -- test262_runner js_command_line > target/p10.3.log 2>&1; echo $?` → 0; `python3 browser/tools/test262.py fetch && python3 browser/tools/test262.py run test/built-ins/Array -j 8 > target/t262-array.log 2>&1; tail -3 target/t262-array.log` prints the score
- depends on: nothing (parallel with P10.1)
- time box: 90 min; model: sonnet

#### P10.4: the conformance table
- goal: a per-area pass-rate table committed to the repo and tracked over time, with a check that fails when an area drops.
- design: `browser/tools/conformance.py`: `run [-j N] [AREA...]` (calls `wpt.py run` and `test262.py run` for the areas of `wpt_areas.json` and the test262 directories); `table` writes `docs/conformance.md` (per area: files, score, threshold, status, change since the last table; the pinned revisions; the date and Dweller commit), `browser/tools/conformance/<area>.json` (test path → passed/total, sorted, so a regression shows as a diff of named tests) and appends a row per area to `browser/tools/conformance/history.csv`; `check` exits 1 when an area's score is more than 0.5 points below the committed table, listing the tests that went from pass to fail. The first baseline is committed in this PR. CI: `.github/workflows/conformance.yml` on `macos-latest`, weekly and on `workflow_dispatch`: fetch (cached by revision), build, `run`, `check`; the results are an artifact. A component card runs `run AREA` and `check` for its area in its verify and commits the new table rows.
- files: `browser/tools/conformance.py`, `browser/tools/conformance/`, `docs/conformance.md`, `.github/workflows/conformance.yml`
- verify: `python3 browser/tools/conformance.py run -j 8 > target/conformance.log 2>&1 && python3 browser/tools/conformance.py table && python3 browser/tools/conformance.py check; echo $?` → 0; `docs/conformance.md` lists every area of the table above
- depends on: P10.1, P10.2, P10.3
- time box: 90 min; model: sonnet

#### P10.5: fetch and XHR under `wpt serve`
- goal: the fetch/XHR tests that need WPT's Python handlers and `.sub.` substitution run, using upstream `wpt serve` from the pinned checkout. Its hosts (`web-platform.test`, `www.web-platform.test`, ...) resolve to 127.0.0.1 through a new `DWELLER_RESOLVE` (libcurl `CURLOPT_RESOLVE`), so no `/etc/hosts` change is needed. Adds the "Fetch and XHR" area and un-excludes `*.sub.*` in the others.
- files: `browser/net/session.aye` (`DWELLER_RESOLVE`), `browser/tools/wpt.py` (`serve --upstream`), `browser/tools/wpt_areas.json`
- verify: `python3 browser/tools/wpt.py run fetch/api xhr --upstream -j 8 > target/wpt-fetch.log 2>&1; tail -3 target/wpt-fetch.log` prints both scores; `python3 browser/tools/conformance.py check; echo $?` → 0
- depends on: P10.4
- time box: 90 min; model: sonnet

#### P10.6: `testdriver.js` subset
- goal: tests that call `test_driver.click`, `send_keys` and simple `action_sequence` pointer/key actions run: our `resources/testdriver-vendor.js` turns them into `--events` words that `dweller wpt` feeds to the page (element centre for clicks). Un-excludes those tests in "Events and timers" and "Forms".
- files: `browser/tools/wpt/testdriver-vendor.js`, `browser/main.aye` and `browser/ui/script.aye` (events in `wpt` mode), `browser/tools/wpt_areas.json`
- verify: `python3 browser/tools/wpt.py run uievents html/semantics/forms -j 8 > target/wpt-input.log 2>&1; tail -3 target/wpt-input.log`; `python3 browser/tools/conformance.py check; echo $?` → 0
- depends on: P10.4
- time box: 90 min; model: opus

### Layer 1 component backlog

Component cards (P10.7 onward) are written from the baseline: per area,
the largest cluster of failing tests with one cause first; verify is
`wpt.py run <dirs>` (or `test262.py run <dir>`) with the score at least
the stated gain and `conformance.py check` exiting 0. The items below
were filed by sites or the B8 list; each becomes part of the card for
the failing tests that cover it, or its own card if no test covers it.
Before a site-found item becomes a card, its symptom is confirmed on
current `main`.

- **HTML parsing:** P3.10 scripts run while parsing see an 800px viewport instead of `--width` (async images follow-up).
- **DOM:** P4.38 `SVGAElement` (with `SVGElement` as its prototype): Mistral's Astro `ClientRouter` throws on every click that is not on an `<a>` (user log 2026-10-15); verify also: a fixture clicking a `<span>` with `t instanceof SVGAElement` in a listener, no Uncaught on mistral.ai. P3.32 a frame's `history.length` counts only its own entries; 1 during initial load (P3.29). B8: moving nodes between documents; one joint session history for a tab and its frames.
- **Events and timers:** P3.9 dump virtual clock: an `onload` set inside `setTimeout(0)` after `src` misses the load event. B8: event listeners on another frame's nodes.
- **CSS box model and sizing:** P3.59 intrinsic sizing passes no container height (`browser/layout/intrinsic.aye`, `cb_h = -1`); BBC NEWS `<h1>` 300px instead of 100px (P3.47; still seen 2026-10-14: the NEWS wordmark 100 px left of centre).
- **CSS floats and tables:** P3.70 abehiroshi: the news column drawn below the photo instead of beside it, menu bullets above their links. P3.71 ZOL: 找产品 product names wrap and the rows misalign; the 新品首发/本月销量最高 tabs stack. P3.68 Python docs: the sidebar drawn right of the text (Chrome: left); diagnose to its area first.
- **CSS overflow:** P3.53 ellipsis and clamp follow-ups: right-to-left, `-webkit-line-clamp` into child blocks, unprefixed `line-clamp` (P3.52).
- **CSS grid:** P3.54 `grid-template` with area strings and the `grid` shorthand (P3.46).
- **CSS positioning:** B8: sticky only against the viewport.
- **CSS backgrounds and borders:** P3.70 abehiroshi's tiled background image missing.
- **CSS transforms:** B8: rotations and skews other than quarter turns are not drawn.
- **Forms:** P3.51 form control content not centred vertically: GitHub commit messages about 10 px above the file names, DuckDuckGo's query text at the top of its box, YouTube's search-button icon at the top of the button (Google's bevelled buttons not reproduced 2026-10-14: Chrome draws the archive the same). P3.66 Wikipedia's Appearance panel empty (Chrome: Text, Width, Color radio buttons); confirm it is form controls, not script. B8: calendar, color and file pickers, `datalist`.
- **Images and SVG:** P3.56 `<video poster>` loaded and painted like an image (empty video frame on wikipedia-en). P3.16 a script changing an img in another same-origin frame may record the failed load in the wrong page's cache. P4.39 `getTotalLength()`/`getPointAtLength()` on SVG geometry elements (Mistral's `layout` script); verify also: a fixture path's length matches Chrome's within 0.5 px. B8: no AVIF (out of scope for now).
- **URL and encoding:** P3.19 URL parser accepts `http://[`; WHATWG IPv6 host parsing (P3.15).
- **Fetch and XHR:** P3.21 fetch guards: immutable Response headers, forbidden Request header names, abort listener removed on settle. P3.25 send `Origin` on non-GET and CORS-mode fetches. P3.31 XHR `loadstart`/`progress` events and `xhr.upload`. B8: no public suffix list for cookies (a bare top-level name is refused, `co.uk` is not).
- **JS:** P3.42 Proxy invariant checks against the target (P3.39). P4.18b Map/Set iterators are snapshots; make them live (P4.18). P3.60 move the JS lexer, parser and AST out of the `js` module for a bundler; pure move (framework F4).

## Layer 2: P3 and P6, real sites as the final check

Real sites validate the layers below; they no longer choose the work.
The checks are W2, W3 (`page_load.py check --agent`, the `visual`
field), W4 (`run --live --repeat 5`) and, from P4.41, W7 and W8 over the
same list. A site defect found here is first matched to a Layer 1 area
(its WPT tests are run); it stays a Layer 2 card only when it is
specific to that site's page or to the tools.

- P3.80 user acceptance (decided by the user 2026-10-15): the sites the user uses every day all need a login, so Layer 2's final validation is the user's own daily use of a bundled build. Once W1–W9 hold and the login-readiness cards (P10.20, P4.50–P4.53, P10.21–P10.23) are done, the assistant bundles a build (with local telemetry on, P4.53); the user uses it on their logged-in daily sites and reports problems, which become cards (matched first to a Layer 1 area, as above). No site list is requested.
- P3.7 Mistral: short iframe widgets, `setTransform` errors; recheck after P3.48.
- P3.35 YouTube search agent task: expect results URL plus a title; recapture with the `youtubei/v1/search` POST (P3.30).
- P3.41 BBC ☰ menu button missing in replay (still seen 2026-10-14: no menu/search button at the left, no Register/Sign In at the right); diagnose to its area.
- P3.61 `screenshot` runs the page as long as `dump` (Mistral's title animation caught half-way; "Introducing Mistral Large 4" not drawn, "Le chonk" 240 px low).
- P3.67 MDN: the "Try it" demo shows an `about:blank` link instead of the code editor and output; the language switcher is drawn twice.
- P3.72 react.dev: the "My video" demo thumbnail is an empty white box (Chrome: blue gradient with a play icon).
- Site load times (W4), rechecked after P4.33 and P4.37, which may remove them: P4.1 react.dev live vs replay (DNS/TLS, discovery, module waits, HTTP/2) and P4.2 its fix; P4.3 ZOL live: discovery waits vs transfer waits; P4.4 YouTube startup scripts, 0.84 s of interpreter time; P4.8 ZOL and Mistral replays spend most time in timers.
- P5: the B8 "What remains" items are filed under their Layer 1 areas above; an item no area or site needs is declared out of scope (W6).
- P6.1 manual checklist on macOS, recorded; P6.2 final `docs/browser-sites.md` run; P6.3 docs pass and a tag.

### Done since 2026-10-14 (squash commits on `main`)

#1 215e3a1 `COMPILER_GAPS.md` gaps 49–56 closed; #2 8546630 and #4
aee0c41 P7 flaky image-load tests (an image's load/error fires in the
task that finishes its background load; a failed URL is fetched once per
document); #3 c13bc8b O1.17c `virtual_clock.html`; #5 55b4b48 O1.14c
`@targetOs()`; #6 1198067 P8.x `std.fs.readDir`; P3.63 Mistral CPU 45 →
33 s (`:where()`/`:is()` keys anchor `:has()`); P3.64 slice 1
incremental layout for forced reads (`browser/layout/incremental.aye`,
`DWELLER_LAYOUT_CHECK=1`; Mistral 33 → 26 s); P1.6 `page_load.py` output
in `target/page-load-out/`; P1.14 `visual` field in `sites.json`; P3.65
`opacity` as groups (#16); P3.69 and P3.73 variable font weights, SVG
image scaling, `noscript` sheets; P3.73b CSS `filter` (#18); P9.1 (#17),
P9.2 (#19), P9.3 (#21), P9.4 (#24); P4.31 macOS wheel and trackpad (#22); window
diagnosis (#23); rename to Dweller (#20). Details: the PRs and
`browser/engine/PERF.md`.

## P7: maintenance (recurring)

Main writes a P7.n card from this form when an `AYE_REV` bump breaks the browser
tests or a language change removes the need for a workaround. No P7
card adds a site fix or a feature.

#### P7.n: browser breakage from <language step id>
- lane: browser
- goal: `cargo test --release --test browser` passes again after <step>, by changing `browser/` to the new language or std behavior (never by reverting the language change).
- serves: M1 (and M3 when a workaround goes or a gap is found)
- files: the failing area of `browser/`, `browser/COMPILER_GAPS.md`
- verify: `cargo test --release --test browser > target/browser.log 2>&1; echo $?` → 0
- depends on: <step> merged
- parallel with: all except other `browser/` steps in the same area
- where: cloud when no site replay is needed; local when the failure is in `page_load.py` or screenshots
- model: sonnet; opus when the cause is unknown (diagnose first, rule 7)
- time box: 45 min
- notes: "Fix the branch side when main removed an API" (`docs/practices/workflow.md`).


## Decisions

- **Three layers, foundations first (user, 2026-10-15):** see "Strategy
  (2026-10-15)". It replaces the maintenance mode of 2026-10-08 (no new
  site fixes or features until Aye's O6–O9), which work had already left
  with P4.31, P4.32 and P9.
- **Perf gate fixtures (2026-10-15):** synthetic pages on the local test
  server, committed; recorded archives stay untracked (third-party
  content a public repo may not redistribute, possible cookies, size,
  drift). W7 itself is measured on the archives locally (P4.41).
- **Conformance suites fetched, not vendored (2026-10-15):** WPT and
  test262 at pinned commits in `browser/tools/wpt.rev` and
  `test262.rev`, cached in CI; only the small harness files the runner
  tests need are copied, with their BSD license. A revision bump is its
  own PR that also commits the new table.
- **Area thresholds (2026-10-15):** the table in "Layer 1"; an area at
  its threshold is done for now. A threshold changes only by a recorded
  decision here.
- **Renamed to Dweller and open-sourced (user, 2026-10-15):** the browser
  and its repository, formerly `aye-lang/anyknown`, are named Dweller
  (`aye-lang/dweller`, binary `dweller`, env vars `DWELLER_*`) and
  licensed MIT OR Apache-2.0; the repository is public.
- **Load-time target (W4):** per site, live median minus replay median
  ≤ 0.5 s over `run --live --repeat 5`; absolute targets in `sites.json`
  after P1.4 measured them.
- **P9 (2026-10-14):** the user asked for agent Spaces (B9); it runs as a
  parallel track beside Layer 0. A Space is a process, not a thread
  (B9, threading model).
- **Bot walls:** Stack Overflow and Reddit are excluded from W3, W7 and
  W8 while they serve a bot wall; `sites.json` marks them `excluded`.
- **B8 remains:** an item no Layer 1 area and no listed site needs is out
  of scope (P5).
