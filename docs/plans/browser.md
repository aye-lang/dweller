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
as tasks on that thread; no compositor thread, no process per site, and no GPU except for
WebGL and, later, compositing (P11, `docs/webgl.md`). Spaces (P9) are
the one place with more than one process.

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

## Current state (2026-10-15 evening, `main` e9bb877)

- **CI: red on `main`.** `window_paints_early` failed on `macos-latest`
  in 2 of the last 7 pushes (runs 38701882826 at #57 and 38710550094 at
  #60; Ubuntu green both times). The test clicks a checkbox after 40
  `frame` words; on the slow runner those frames take longer than the
  1500 ms the page's script is delayed, so the click lands after the
  script ("checked before it: false"). It waits on wall time, which #42
  removed from the other timing tests. Fixed by P4.80 (2026-10-16): the
  click waits for the first paint (`--events` word `painted`).
- **Layer 0 (P4).** Done: P4.31–P4.36, P4.38, P4.39, P4.43–P4.45,
  P4.50–P4.53, P4.56–P4.58, and the VM preparation P4.59–P4.62 (benchmark
  set, bytecode format, resolver). The window never waits on page
  fetch/XHR, scripts, `import()` or frames; the first paint comes after
  the head sheets; relayout is incremental (container queries included,
  not shadow trees); paint keeps the page's pixels with damage tracking
  and a raster cache; transform and opacity changes composite; a scroll
  moves the page as a texture offset and fixed and sticky boxes as layers
  (P4.36a–c). Open: P4.37, P4.40–P4.42, P4.46–P4.48, P4.55b; backlog P4.54, P3.64
  slice 2, P4.11, P4.14. W7 and W8 are not re-measured since the P4.32
  table in `PERF.md` (P4.41 does that); mistral idle CPU is still over
  25% (P4.57b's note).
- **Login readiness.** P10.20–P10.23 done; P9.7 open; then
  the user bundle P3.80.
- **Sites.** `page_load.py check` passes on the recorded archives; W3 is
  8 of 20. Archives stay in each checkout's `target/sites/` (untracked).
- **Conformance (P10).** Not measured: `wpt.py`, `test262.py` and
  `conformance.py` do not exist yet (P10.1–P10.4 not started).
- **P9.** P9.1–P9.4 done; P9.5 next.
- **P11.** P11.1–P11.5 and P11.16 done; P11.17's API done (its
  `wasm_sim_bench` and the wasm-bindgen glue check wait for P11.24's
  copy of the game); P11.6 next.
- **JS VM (P4.30).** P4.59–P4.62 done; P4.63 (the dispatch loop) next,
  behind `DWELLER_JS_VM=1`.
- **Aye.** `AYE_REV` a067dfd. P3.48 is not merged in the Aye repo
  (P3.48b bisects it); P3.58 is superseded by P11.3. Open compiler gap:
  57 (`@ptrCast` to `?fn` / from `void *`), met by P11.2.

## Phases

| Layer | Phase | Goal | Exit check | Status (2026-10-15 evening) |
|---|---|---|---|---|
| – | P0, P1, PW | one clean `main`; "done" checked by a command; wrap-up | – | done |
| – | P8 | Dweller in its own repo | CI green on macOS and Ubuntu | P8.2 done 2026-10-15; P8.3 (removal from the Aye repo, downstream check) is the Aye repo's card |
| – | P7 | tests green while the language changes | M1, M3 after every `AYE_REV` bump | ongoing; M1 broken by the P4.80 flake |
| 0 | P4 | the window responds and paints early | W7, W8, the CI perf gate (P4.40) green | 17 of 26 window cards done (list above); P4.80, P4.42, P4.37, P4.46 next; the VM cards P4.63–P4.78 run as their own lane |
| 0 ∥ | P9 | agents without disturbing the user (B9, R1–R7) | the P9 tests pass: `window_agents`, `window_spaces*`, `agent_sessions` | P9.1–P9.4 done, P9.5 next |
| 0 ∥ | P11 | WebGL 2 and the APIs the user's game needs (`docs/webgl.md`) | P11.24: the user's game runs in Dweller at ≥ 50 fps | P11.1–P11.5, P11.16 done, P11.17 partial; P11.6 next |
| 1 | P10 | components measured by WPT and test262 | W9 | harness P10.1–P10.4 not started; P10.20–P10.22 (login) done |
| 2 | P3 | real sites correct | W2, W3; `docs/browser-sites.md` rerun | W3 8 of 20; P3.80 (the user bundle) is the acceptance |
| 2 | P5 | B8 "What remains" resolved (W6) | the list is empty or out of scope | items re-filed under P10 areas |
| 2 | P6 | release | W1–W9; manual checklist recorded; tag | last |

## Roadmap: phases and steps (2026-10-15 evening)

This replaces the flat "Order of work" of the morning. A **phase** is a
goal with an exit check; a **step** is one card, one PR, in the order
given. Cards keep their detail in the sections below; a step row says
only what it is, what it waits on, its state, and the Chromium
reference to read first (paths relative to
`/Users/solemnis/Documents/chromium/src`; the module map at the end of
this section says what each reference is for). "Lane" names the files a
phase edits: phases in different lanes run at the same time, two steps
in one lane never do.

| Lane | Files | Phases |
|---|---|---|
| loop | `ui/window.aye`, `ui/tabs.aye`, `engine/script.aye`, `engine/tab.aye`, `engine/history.aye`, `net/` | A |
| paint | `paint/`, `ui/pagedraw.aye`, the draw part of `ui/window.aye`, `tools/window_perf.py` | B |
| harness | `browser/tools/`, `main.aye` (CLI modes), `browser/tests/wpt/`, `browser/tests/test262/` | C |
| login | `engine/prelude.aye` (forms), `ui/forms.aye`, `ui/spaces.aye` (P9.7) | D |
| spaces | `ui/spaces.aye`, `ui/agents.aye`, `agent/` | E |
| vm | `browser/js/`, `browser/tests/jsbench/` | F |
| gpu | `browser/gpu/`, `engine/webgl_*.aye`, `browser/wasm/`, `browser/audio/`; P11.10, P11.22, P11.23 also `ui/` (not beside A or B) | G |
| components | `css/`, `layout/`, `dom/`, `html/`, `engine/` by area | H |
| sites | `tools/sites.json`, `docs/browser-sites.md`, the manual checklist | I |

Running order: A1 first (green `main`), then A, B, C, E, F and G in
parallel, one step at a time per lane; D after A and C4; H after C4,
by area; I last. Common to every step: run only the tests its verify
names, never two browser test runs at once, build and test output to a
log under `target/` read with `tail`/`grep`; the same verify failing
twice in a row stops the step with a report. On the user's Mac (10
cores, 32 GB, shared with daily use and a self-hosted Actions runner)
every build and test runs under `nice -n 10` with `AYE_TEST_JOBS=3`
and `cargo ... -j 4`, at most one local agent builds or tests at a
time, and heavy cards go to cloud sessions (2026-10-17).

### Phase A: green `main` and the rest of the loop (Layer 0, lane loop)

Exit: `main` green on both OSes; no synchronous network wait on the
window thread in any loop iteration; the perf-gate pages' longest loop
under 100 ms with a timer storm.

| Step | Card | What | After | State | Chromium |
|---|---|---|---|---|---|
| A1 | P4.80 | `window_paints_early` waits on the loop's records, not on 40 frames of wall time | – | done | – |
| A2 | P4.42 | loop priorities (input, frame, then timers and network tasks in a 50 ms budget) and the threading-model note | A1 | done | `platform/scheduler/TaskSchedulingInBlink.md`, `platform/scheduler/main_thread/main_thread_scheduler_impl.cc` (queue priorities), `docs/threading_and_tasks.md` |
| A3 | P4.37 | one connection pool: one multi handle and share handle per process, 6 per host, HTTP/2 multiplexed | – | done 2026-10-16 | `net/socket/client_socket_pool_manager.cc`, `net/docs/life-of-a-url-request.md` |
| A4 | P4.46 | back, forward and reload through the incremental navigation, scroll restored | – | done | `core/loader/frame_loader.cc`, `core/frame/history.cc`, `core/loader/history_item.h`, `docs/navigation_concepts.md` |
| A5 | P4.47 | `blob:` URLs for scripts, fetch and XHR | – | done | `core/fileapi/public_url_manager.cc`, `core/fileapi/blob.cc` |
| A6 | P4.48 | the cost of the inserted site scripts in `dump` (mistral 26 → 45 s): forced layouts and cascades | A5 | partial (shadow-root cascades fixed; reddit, bbc JS left) | `core/dom/document.cc` (`UpdateStyleAndLayout`, what a forced read recomputes), `core/frame/local_frame_view.cc` (`UpdateLifecyclePhases`) |
| A7 | P4.55 | rAF and timers throttled in hidden windows and background tabs | A2 | done (P4.55b open) | `platform/scheduler/main_thread/frame_scheduler_impl.cc`, `platform/scheduler/common/throttling/budget_pool.cc`, `core/frame/local_frame_view.cc` (throttled frames) |
| A8 | P4.54 | macOS crash record exits with 128+sig | – | backlog | – |

### Phase B: paint and the gate (Layer 0, lane paint)

Exit: the `perf` CI job green (P4.40); `window_perf.py report --replay`
and `report --live --chrome` exit 0 (W7, W8).

| Step | Card | What | After | State | Chromium |
|---|---|---|---|---|---|
| B1 | P4.36 | scroll: fixed and sticky items kept apart so a scroll frame rebuilds no list; fractional trackpad scrolls draw the band only | – | done | `platform/graphics/paint/paint_controller.cc` (display items reused between frames), `cc/trees/damage_tracker.cc`, `docs/how_cc_works.md` (scrolling as an offset) |
| B2 | P4.40 | CI perf gate: four synthetic pages, counters (`style_full`, `layout_full`, `net_sync`, `raster_px`), budgets with a ratchet | A2, B1 | P4.40a, P4.40b, P4.40c, P4.40d done | – |
| B3 | P4.41 | the W7 and W8 report over the archives and live against Chrome; a dated table in `PERF.md` | B2 | open | – |
| B4 | P3.64 slice 2, P4.11 | restyle subtrees when no container's size changed; partial restyle for DOM mutations with shadow roots and `:has()` | B3 shows the need | backlog | `core/css/style-invalidation.md`, `core/css/invalidation/` (invalidation sets), `core/css/style_engine.cc`, `core/style/stylerecalc.md` |
| B5 | P4.14 | event dispatch and `MutationObserver` from prelude JS into Aye | B3 shows the need | backlog | `core/dom/events/event_dispatcher.cc`, `event_path.cc`, `core/dom/mutation_observer.cc` |

### Phase C: the conformance harness (Layer 1, lane harness)

Starts now, beside A and B. Exit: `docs/conformance.md` holds a baseline
for every area of the table in "Layer 1: P10", `conformance.py check`
exits 0, the weekly job runs.

| Step | Card | What | After | State | Chromium |
|---|---|---|---|---|---|
| C1 | P10.1 | WPT testharness runner: `wpt.py fetch/serve/run`, `dweller wpt URL` | – | done | `third_party/blink/tools/blinkpy/web_tests/` (the runner's shape: expectations, timeouts, parallel workers), `third_party/blink/web_tests/resources/testharnessreport.js` |
| C2 | P10.3 | test262 runner over `dweller js --test262` | – (beside C1) | done | – (`v8/test/test262/` is empty in the checkout; the harness files come from the pin) |
| C3 | P10.2 | reftest and crashtest runner, Ahem font, fuzzy matching | C1 | done | `third_party/blink/tools/blinkpy/web_tests/port/` (reftest pixel comparison, `fuzzy`) |
| C4 | P10.4 | the conformance table, first baseline, `check`, weekly CI | C1–C3 | done | – |
| C5 | P10.5 | fetch and XHR under upstream `wpt serve` (`DWELLER_RESOLVE`) | C4 | open | – |
| C6 | P10.6 | `testdriver.js` subset through `--events` | C4 | open | `third_party/blink/web_tests/resources/testdriver-vendor.js` |

### Phase D: login readiness and the user bundle (lane login)

Exit: P3.80 handed to the user with telemetry on.

| Step | Card | What | After | State | Chromium |
|---|---|---|---|---|---|
| D1 | P10.23 | sign-in page details: `autocomplete`, `navigator.credentials` absent, `Set-Cookie` on fetch/XHR, Enter submits, `requestSubmit` | – | done | `core/html/forms/html_form_element.cc` (implicit submission, `requestSubmit`) |
| D2 | P9.7 | shared logins with Spaces: jar and `localStorage` copied both ways; `"profile":"isolated"` | E1 not needed; after P4.50/P4.51 (done) | open | – (B9's own design) |
| D3 | P3.80 | the user bundle: W1–W9 and D1, D2 hold; the user's daily sites; reports become cards matched to a Layer 1 area | A, B3, C4, D1, D2 | open | – |

### Phase E: Spaces (lane spaces, parallel track)

Exit: R1–R7 of B9; Spaces by default.

| Step | Card | What | After | State | Chromium |
|---|---|---|---|---|---|
| E1 | P9.5 | watching a Space: frames through a shared file, chips, Cmd+Shift+0..8 | – | open | `base/memory/shared_memory_mapping.h` (a mapped region per frame; idea only) |
| E2 | P9.6 | take over and hand back | E1 | open | – |
| E3 | P9.8 | code mode `run` in a realm of its own | – (beside E1, E2) | open | – |
| E4 | P9.9 | Spaces by default, three agents in parallel, the in-process acting path removed | E2, E3, D2 | open | `docs/process_model_and_site_isolation.md` (why a process boundary, not a thread) |

### Phase F: the JS bytecode VM (lane vm, parallel track)

Exit: P4.75 on `main` with no fallback reason; then P4.77, P4.78. Each
card names its V8 files (`browser/js/BYTECODE.md` section 7).

| Step | Card | What | After | State |
|---|---|---|---|---|
| F1 | P4.63 | VM core: frames, dispatch, straight-line code (`DWELLER_JS_VM=1`) | – | done |
| F2 | P4.64 | control flow | F1 | done |
| F3 | P4.65 | calls and `new` | F2 | done |
| F4 | P4.66 | closures and contexts | F3 | done |
| F5 | P4.67 | properties, literals, globals, the top level | F4 | done |
| F5b | P4.67b | the top level, `evalScript` and per-slot caches | F5 | partly done |
| F6 | P4.68 | exceptions | F5 | done |
| F7 | P4.69 | the rest of the syntax | F6 | done |
| F8 | P4.70 | classes | F7 | done |
| F9 | P4.71 | generators on resumable frames | F8 | done |
| F10 | P4.72 | async functions on resumable frames | F9 | done |
| F11 | P4.73 | inline caches | F10 | open |
| F12 | P4.74 | prototype shapes and the prototype epoch | F11 | open |
| F13 | P4.75 | the VM on by default | F12 | open |
| F14 | P4.77 | register optimizer and fused instructions | F13 | open |
| F15 | P4.78 | remove the tree walker's execution paths | F13 on `main` with no fallback | open |

P4.76 is covered by P11.3 (done); P4.79 is G19.

### Phase G: WebGL, WebAssembly and the game (lane gpu, parallel track)

Exit: P11.24, the user's game at ≥ 50 fps. The cards cite `docs/webgl.md`
and their Blink, ANGLE and V8 files.

| Step | Card | What | After | State |
|---|---|---|---|---|
| G1 | P11.6 | textures | – | done |
| G2 | P11.7 | WebGL canvases in the window by readback, screenshots, `toDataURL` | G1 | done |
| G3 | P11.8 | WebGL 2 state objects (VAOs, instancing, MRT, UBOs, samplers, 3D textures) | G1 | done |
| G4 | P11.9 | three.js hello cube | G2, G3 | done |
| G5 | P11.10 | rAF at vsync, frame timestamp, `performance.now()` precision (touches `ui/window.aye`: not beside A2, A7) | A2 | done |
| G6 | P11.11 | key codes, pointer events, Pointer Lock, Fullscreen, `structuredClone` | – | done |
| G7 | P11.17 rest | `wasm_sim_bench` and the wasm-bindgen glue check | G18's copy of the game | partial |
| G8 | P11.18 | dedicated Workers | G6 | done |
| G9 | P11.19 | Web Audio core | – | done |
| G10 | P11.20 | Web Audio effects | G9 | done |
| G11 | P11.12 | the rest of WebGL 2 | G3 | open |
| G12 | P11.13 | WebGL 2 extensions | G3 | open |
| G13 | P11.14 | WebGL conformance runner and baseline | G4, C1 (or `server.py`) | open |
| G14 | P11.15 | context loss, many contexts, limits | G12 | open |
| G15 | P11.21 | the game's frame budget | G4, G5, G7, G8, G9 | open |
| G16 | P11.22 | GPU compositing 1: the window presents through ANGLE (lane paint too: not beside B) | G2, G5 | open |
| G17 | P11.23 | GPU compositing 2: canvases, layers and scrolling on the GPU | G16 | open |
| G18 | P11.24 | acceptance: the user's game | G4–G17 | open |
| G19 | P4.79 | per-frame JS cost of the game | G18, F13 | open |
| G20 | P11.29 | WebGL 1 (other sites) | G12 | open |

P11.25–P11.28 only if the game comes to need them.

### Phase H: components by area (Layer 1, lane components)

Starts after C4 gives the baseline. Order: first the eleven items the
telemetry ranked by what sites lose (pre-filed as P10.24–P10.34), then
per area the largest cluster of failing WPT tests with one cause, until
the area is at its threshold. Verify is the area's `wpt.py run` with the
stated gain and `conformance.py check` exiting 0. The per-area backlog
(the P3.x items) joins the card that covers its tests.

| Step | Card | Area | What | Chromium |
|---|---|---|---|---|
| H1 (done) | P10.34 | events | `pointer-events: none` in hit testing | `core/layout/hit_test_result.cc`, `core/paint/paint_layer.cc` (`HitTest`), `core/input/event_handler.cc` |
| H2 | P10.24 | box model and sizing | `max-content`, `min-content`, `fit-content` keywords (done 2026-10-16) | `core/layout/block_layout_algorithm.cc`, `core/layout/layout_box.cc` (intrinsic sizes), `core/layout/constraint_space.h` |
| H3 (done) | P10.27 | inline and text | `vertical-align` (done 2026-10-16) | `core/layout/inline/` (line boxes, `line_breaker.cc`), `core/layout/table/` (cell alignment) |
| H4 (done) | P10.28 | inline and text | `letter-spacing`, `word-spacing` (done 2026-10-16) | `platform/fonts/shaping/harfbuzz_shaper.cc` (spacing applied after shaping) |
| H5 (partial) | P10.33 | transitions | `transition*` parse and compute, events and interpolated values for elements getComputedStyle reads (2026-10-16; left: elements only laid out, see the card) | `core/animation/css/css_transition.cc`, `core/animation/css/css_animations.cc` |
| H6 (done) | P10.32 | transforms | `translate`, `rotate`, `scale` | `core/css/properties/`, `core/layout/layout_object.cc` (transform order) |
| H7 (done) | P10.31 | backgrounds and borders | `outline` and the focus ring | `core/paint/box_painter.cc`, `core/paint/outline_painter.cc` |
| H8 | P10.29 | inline and text | `text-wrap`, `text-shadow` | `core/layout/inline/line_breaker.cc`, `core/paint/text_painter.cc` |
| H9 | P10.26 | floats and tables | `border-collapse`, `border-spacing` | `core/layout/table/table_layout_algorithm.cc`, `table_borders.cc` |
| H10 (done) | P10.25 | values, units, color | `lh`, `rlh`, `env()`, `round()`/`mod()`/`rem()` (done 2026-10-16) | `core/css/css_math_expression_node.cc`, `core/css/parser/` |
| H11 | P10.30 | inline and text | text decoration details | `core/paint/text_decoration_info.cc` |
| H12 | P10.35 | encoding | label table, `TextDecoder`/`TextEncoder` options, single-byte decoders, URL and form query in the document encoding | `encoding.spec.whatwg.org`, `core/encoding/text_decoder.cc`, `core/url/dom_url.cc` (query encoding) |
| H13+ | P10.36+ | per area | from the baseline's failure clusters; the per-area backlog of "Layer 1 component backlog" | the module map below |

### Phase I: sites and release (Layer 2, lane sites)

| Step | Card | What | After |
|---|---|---|---|
| I1 | P3.7, P3.35, P3.61, P3.67, P3.72 | the open site items re-checked on `main` after B3 and the H cards; each matched to a Layer 1 area or kept as a site card | B3, H |
| I2 | P4.1–P4.8 | W4 site load times, rechecked after A3 (they may be gone) | A3, B3 |
| I3 | P5 | W6: every "What remains" item done or declared out of scope | H |
| I4 | P6.1–P6.3 | manual checklist on macOS, the final `docs/browser-sites.md` run, docs pass and a tag | all |

### Chromium module map: from utils to engine

What to read in the local checkout for each Dweller module, and what
Dweller takes. The rules of "Decisions" apply: Blink is an algorithm
reference only (some files are LGPL); V8 and ANGLE (BSD) may be ported
with attribution. Dweller keeps its design: one window thread, no
compositor thread, no process per site.

| Dweller | Chromium (under `src/`) | Take | Not taken |
|---|---|---|---|
| `url/` | `url/third_party/mozilla/url_parse.cc`, `url/url_canon_host.cc`, `url/url_canon_ip.cc` | host canonicalization, IPv6 parsing (P3.15, P3.19) | GURL's API |
| `encoding/` | `third_party/blink/renderer/platform/wtf/text/text_codec_*.cc` | label tables, BOM sniffing, replacement rules | ICU |
| `net/` (http, cache, cookies, auth) | `net/docs/life-of-a-url-request.md`, `net/socket/client_socket_pool_manager.cc`, `net/http/http_cache.cc`, `net/cookies/cookie_monster.cc`, `net/dns/host_resolver.h` | one pool per process keyed by scheme, host and port (A3); cache keys and validation; cookie eviction and SameSite rules | a network process; the socket layer (libcurl plays it) |
| `net/loader.aye` (prefetch, priorities) | `core/html/parser/html_preload_scanner.cc`, `platform/loader/fetch/resource_fetcher.cc`, `core/loader/preload_helper.cc` | preload discovery while parsing; request priorities by kind | a memory cache per renderer |
| `html/` | `core/html/parser/html_tokenizer.cc`, `html_tree_builder.cc`, `html_document_parser.cc`, `core/script/script_runner.cc`, `html_parser_script_runner.cc` | parser yielding to the loop; async/defer and inserted-script ordering (P4.45 done) | the background tokenizer thread |
| `dom/` | `core/dom/README.md`, `core/dom/node.cc`, `element.cc`, `dom/events/event_dispatcher.cc`, `event_path.cc`, `mutation_observer.cc`, `core/intersection_observer/` | event path building, capture/bubble/`composed`; observer delivery as microtasks (B5) | Oilpan |
| `css/` parser, cascade, selectors | `core/css/parser/css_tokenizer.cc`, `css_parser_impl.cc`, `core/css/rule_set.cc`, `selector_checker.cc`, `element_rule_collector.cc`, `resolver/style_resolver.cc`, `resolver/cascade_map.h`, `container_query_evaluator.cc`, `media_query_evaluator.cc`, `core/css/properties/README.md` | bucketed rule index by id/class/tag/attribute; cascade priority encoding; per-property handlers | generated property classes |
| style invalidation (`css/invalidation.aye`, `engine/`) | `core/css/style-invalidation.md`, `style-calculation.md`, `core/css/invalidation/` (`invalidation_set.h`, `rule_invalidation_data.h`, `style_invalidator.cc`), `core/css/style_engine.cc`, `core/style/stylerecalc.md`, `core/css/style_recalc_change.h` | descendant and sibling invalidation sets built from selectors; recalc flags per node (B4) | the full `StyleRecalcChange` lattice |
| `layout/` | `core/layout/README.md`, `layout_ng.md`, `block_layout.md`, `constraint_space.h`, `block_layout_algorithm.cc`, `inline/` (`line_breaker.cc`), `flex/`, `grid/`, `table/`, `absolute_utils.cc`, `layout_box.cc`, `layout_object.cc` | constraint space and fragment results (immutable inputs make incremental layout safe); the algorithms per area (H) | fragmentation, multicol, writing modes (out of scope) |
| `font/`, `webfont/` | `platform/fonts/README.md`, `platform/fonts/shaping/harfbuzz_shaper.cc` | shaping runs, fallback per script, spacing after shaping (H4) | font service |
| `image/`, `svg/` | `platform/image-decoders/`, `core/svg/`, `platform/graphics/path.cc` | decoder contracts (progressive, incremental), path length and point-at-length (P4.39 done) | Skia |
| `paint/` | `core/paint/README.md`, `platform/graphics/paint/README.md`, `platform/graphics/paint/paint_controller.cc`, `docs/website/site/blink/how-repaint-works/index.md`, `cc/trees/damage_tracker.cc`, `cc/tiles/picture_layer_tiling.cc` | display items reused between frames (B1); damage rectangles; tiles for the GPU window (G16) | layer trees, a compositor thread |
| `engine/` lifecycle, timers, rAF | `core/dom/document_lifecycle.h`, `core/frame/local_frame_view.cc` (`UpdateLifecyclePhases`, throttling), `core/dom/scripted_animation_controller.cc`, `core/page/page_animator.cc`, `core/animation/document_animations.cc` | one lifecycle per frame: rAF, style, layout, paint, in that order (P4.34 done, G5) | – |
| `ui/window.aye` loop | `platform/scheduler/TaskSchedulingInBlink.md`, `platform/scheduler/main_thread/main_thread_scheduler_impl.cc`, `frame_scheduler_impl.cc`, `platform/scheduler/common/throttling/budget_pool.cc`, `core/scheduler/` (`scheduler.postTask`), `docs/threading_and_tasks.md` | queue priorities and a time budget (A2); background throttling with 1 s alignment (A7) | many task queues, frame scheduling heuristics |
| `ui/` input and scroll | `core/input/event_handler.cc`, `pointer_event_manager.cc`, `core/paint/paint_layer_scrollable_area.cc`, `docs/graphics/scrolling/scroll_jank_metric.md` | pointer before mouse events (G6); scroll as an offset; the jank metric's definition for W7 | impl-thread scrolling |
| navigation, history | `core/loader/frame_loader.cc`, `document_loader.cc`, `core/frame/history.cc`, `core/loader/history_item.h`, `docs/navigation_concepts.md`, `docs/bfcache.md` | traversal in steps with scroll restoration (A4) | the browser-process navigation request |
| `agent/`, `ui/spaces.aye` | `docs/process_model_and_site_isolation.md`, `base/memory/shared_memory_mapping.h` | why a process boundary; a mapped frame region (E1) | Mojo |
| `js/` | `v8/` per `browser/js/BYTECODE.md` section 7 | Ignition's format, generator and ICs (F) | TurboFan, Maglev, Sparkplug |
| `gpu/`, `wasm/`, `audio/` | per the P11 cards: `third_party/angle/`, `modules/webgl/`, `platform/graphics/gpu/drawing_buffer.cc`, `v8/src/wasm/wasm-js.cc`, `modules/webaudio/` | validation order and error codes; the JS API's shape | the GPU process |
| `tools/` (WPT, test262) | `third_party/blink/tools/blinkpy/web_tests/`, `third_party/blink/web_tests/external/wpt/` (explore only; the pin is the source) | runner shape, expectations and fuzzy matching (C) | the expectations format |

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
- status: done 2026-10-15. The prelude's mutation hook asks `ak.scriptPrepare` (`browser/engine/inserted.aye`) about every script element a script connects, gives text to or gives a `src`: an inline classic script, module or import map runs at once (`document.currentScript` set, restored after: it may run inside another script); an external one is loaded on the loader's multi handle (on the virtual clock its task waits for the load in its turn, so `dump` stays deterministic) and runs in a task that delays the load event, `async` ones as they arrive, `async = false` ones in insertion order; `load`/`error` at the element (error for a failed load, a status that is not 2xx, an empty `src`, and, as Chrome's opaque response blocking, a script of another origin served as `text/html`); `type` (module, importmap, non-JavaScript types ignored) and `noModule` respected; `document.write` from a fetched inserted script is ignored. A script runs once: the parser's scripts, those of innerHTML, outerHTML and DOMParser, and every script that ran are already started (`Script.started`), clones keep the flag, `createContextualFragment` clears it. `crossorigin` and CSP/`nonce` are not implemented (no CORS, no CSP). Engine gaps the newly running site scripts hit, fixed here: a `/` after a function declaration's or a block's `}` starts a regex (zol's ad SDK), `async [key]()` methods, `{length: -1}` array-likes (LengthOfArrayLike, not ToUint32: BBC hung), `push` on an array-like object in O(arguments) (ZOL: O(n²)), Promise species (`then` and the statics construct `this`/the species: core-js replaced BBC's `Promise.prototype.finally` with undefined), `script.innerHTML` as raw text, `document.elementFromPoint`/`elementsFromPoint`. Golden review against Chrome (`PERF.md` "P4.45"): closer on bbc, wikipedia, wikipedia-en; neutral on mistral, zol (console 0 → 4, Chrome's own errors), reddit and stackoverflow (excluded); the 15 others byte-identical; none worse. W3 7 → 8 of 20 (wikipedia's Appearance panel, P3.66, now filled; BBC's Register/Sign In of P3.41 now drawn). Tests `inserted_scripts` (dump twice, the window: 20 `async = false` scripts of 200–390 ms in order, longest loop < 100 ms), the jsdom fixture `inserted_scripts.html`, `js/258_site_script_fixes.js`.

#### P4.47: blob: URLs
- goal: `URL.createObjectURL(blob)` registers the Blob under its URL (until `revokeObjectURL`) and `<script src="blob:...">`, `fetch` and XHR of that URL read it; old.reddit.com's login page inserts a `blob:` script (P4.45 review: "Failed to load script: blob:https://old.reddit.com/...").
- files: `browser/engine/prelude.aye` (createObjectURL, the Blob store), `browser/engine/inserted.aye`, `browser/engine/window_api.aye` (fetch)
- chromium: `third_party/blink/renderer/core/fileapi/public_url_manager.cc`, `blob.cc`. Take: a per-origin registry from the URL to the Blob, revoked explicitly or with the document; a fetch of a `blob:` URL answers from the registry with the Blob's type.
- test: a jsdom fixture: a script inserted from a `blob:` URL runs and fires load; `fetch(blobUrl)` resolves with the Blob's text; after `revokeObjectURL` both fail.
- verify: `cargo test --release --test browser -- jsdom_fixtures > target/p4.47.log 2>&1; echo $?` → 0
- status: done 2026-10-16. `URL.createObjectURL` keeps the Blob in the document's store (`blobStore` in the prelude, gone with the document) until `revokeObjectURL`; `fetch` and XHR of a `blob:` URL answer from it with the Blob's type (GET only; any other method, a revoked or unknown URL fails), and a script inserted from one runs the Blob's text (`ak.scriptRun(el, source)`, `run-blob-script.aye`) and fires load, or error once revoked. The workers' own store is this one. Test: jsdom fixture `blob_urls`. WPT `FileAPI/url`: 44 → 64 of 94 subtests (fetch 8 → 12, xhr 9 → 12, sandboxed-iframe 23 → 32, url-in-tags 0 → 4). Not done: a fetch Request snapshots its blob at creation, blob URL fragments in `<img>`/`<iframe>`, a per-origin registry shared between same-origin frames.

#### P4.48: what the newly running site scripts cost in dump
- goal: since P4.45 the scripts sites insert run, and `dump` CPU grew: mistral 26 → 45 s (timers 24.7 → 46.8 s: forced layouts 6.2 → 26.4 s, cascade 3.0 → 17.9 s, after gtag and the axept.io consent SDK run), reddit's login wall 0.05 → 16 s, bbc 0.36 → 6.7 s (`PERF.md` "P4.45"). Find which timers force the layouts and cascades (`DWELLER_TIMING` per callback) and remove the work Chrome does not do (a layout per read that changed nothing, a cascade per class toggle).
- files: `browser/engine/geometry.aye`, `browser/engine/script.aye` (timers)
- chromium: `third_party/blink/renderer/core/dom/document.cc` (`UpdateStyleAndLayout`: a forced read recomputes only what is dirty), `core/frame/local_frame_view.cc` (`UpdateLifecyclePhases`). Take: a forced read after a change that touched only inline style of one subtree stays a subtree job; count the full cascades and layouts per site as the perf gate does (`style_full`, `layout_full`).
- test: a jsdom fixture reproducing the pattern found, with its forced layouts counted.
- verify: `page_load.py run --site mistral,reddit,bbc` CPU within 20% of before P4.45 (26 s, 0.05 s, 0.36 s) or the remaining time explained as Chrome's own work; `page_load.py check` exit 0
- status: partial 2026-10-16. Measured (`DWELLER_TIMING`, a log per failed subtree restyle): mistral's full cascades came from `restyle` (geometry.aye) refusing any subtree restyle once the document has a shadow root; since P4.45 the axept.io consent SDK attaches one to mistral's page, so each class or style change before a forced read cascaded the whole document. Now the subtree restyle runs with shadow trees when no restyle root is inside, above or around a shadow host (`clearOfShadows`); the layout after it stays full (`relayoutDirty` still refuses shadow trees). `DWELLER_LAYOUT_CHECK=1` on mistral: no STYLE/LAYOUT CHECK. Archives re-captured live 2026-10-16 (the sites changed since P4.45: mistral live 125 s on the old binary), `page_load.py run` CPU before / after: mistral 125.3 → 84.4 s (`style_full` 2780 → 1970, cascade 45.4 → 8.2 s, `script_layout` 58.6 → 19.6 s; main-page timers 77.7 → 35.9 s), reddit 158.6 → 155.3 s, bbc (live; its archive misses its ad requests) 25.2 → 24.7 s. Left: reddit's login wall is pure JavaScript (36 timer callbacks on `/login/` take 162 s, cascade 0.1 s), as is bbc (timers 30 s, cascade 0.2 s): the next step is a JS profile of those callbacks, not geometry; mistral's visual-intelligence iframe (44 s of timers) keeps full cascades because it has container queries and 14 shadow roots and is not adopted (`adoptLayout` needs no shadow trees). Test `jsdom_fixtures_restyle_beside_shadow` (`browser/tests/jsdom/shadow_class_toggle.html`: 50 class toggles each read by `offsetWidth`, beside a shadow host): full cascades 51 → 5, full layouts 51 → 5. `page_load.py check`: mistral pass; the other sites have no archive in this checkout (exit 1 for that).

#### P4.46: back, forward and reload in steps
- goal: the window's back, forward and reload (`Ui.goBy`, `Ui.reload`, the history traversals scripts ask for) go through the incremental navigation of P4.33/P4.43 (`navigateEarly`, `stepPageScripts`) instead of a synchronous load: the page is shown once its sheets are in, its scripts run as tasks, its frames load in the background; the scroll position restored after its scripts ran.
- files: `browser/engine/tab.aye`, `browser/engine/history.aye`, `browser/ui/window.aye`
- chromium: `third_party/blink/renderer/core/loader/frame_loader.cc`, `core/frame/history.cc`, `core/loader/history_item.h` (scroll restoration state), `docs/navigation_concepts.md`. Take: a traversal is a navigation like any other, with the history entry's scroll restored after the first layout that has the target. Not taken: the browser-process navigation request, bfcache.
- test: a window test going back and forward between two local pages with a delayed sheet and script: the address and the old page answer input during the load, the scroll position comes back, longest loop < 100 ms outside script tasks.
- verify: `cargo test --release --test browser window_ > target/p4.46.log 2>&1; echo $?` → 0
- status: done 2026-10-16. `Ui.goBy` (back, forward, reload, and the traversals and reloads scripts ask for) goes through `engine.traverseEarly` (`browser/engine/traverse-early.aye`): an entry of the document shown moves in the document as before; else its page is fetched and parsed (`loadParsed`, the entry's state as `history.state`), takes the entry's place (`Tab.moveTo`) and loads in steps like `Ui.go` (`showEarly`, `startEarly`, `advanceLoad`): painted once its sheets are in, its scripts as tasks, its frames in the background. The entry's scroll offset is applied at the first paint as far as the page reaches (`firstPaintScroll`) and again after its scripts ran (`settleScroll`), where a script's own scroll still wins. The synchronous `Tab.go`/`back`/`forward`/`reload` are gone (the agents' traversal in `agent/actions.aye` keeps `loadState`). Test `window_history_steps` (`browser/tests/history/`: a sheet of 0.3 s and a script of 0.6 s that makes the page tall, both `no-store`, which `server.py` now sends for a `/delay/` name with "nostore"): back, forward and reload each restore the entry's offset, back is painted before its script runs, typing into the address bar during the load is kept, no loop iteration waits for the network. The `--events` word `loaded` waits until the page's scripts ran. Not done: a person's scroll during the load does not cancel the restoration.

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
- status: done 2026-10-15 for documents without shadow trees. The page's layout is adopted with container queries too (`adoptLayout` after `layoutQueried`); a change restyles its subtrees (`restyle`, now with `queried`) and lays out again incrementally, then `settleContainers` (geometry.aye) reads the query containers' sizes off the tree (containers.aye `changedContainers`), restyles the subtrees of exactly those whose size changed or that are new (`css.restyleSubtrees`, keeping the old values), lays out again the elements whose values moved (`relayoutDirty`), and repeats while a container moves, at most 2 rounds as `layoutQueried` (`CONTAINER_ROUNDS`); beyond, or a relayout that is not local, the full path. A subtree restyle styles all of an element's descendants when its `container-type` or `container-name` changed (cascade.aye `sameContainer`). Two limits of P3.64 that container flips hit are lifted: the relayout walk lays out the root box again instead of giving up (incremental.aye), and the blocks it lays out again keep the layout of their unchanged children when their containing block is the same (`Box.laid`, `Tree.keep_under`); a dirty element without a box under `display: none` (the `<head>`) is skipped, and `document.title =` is noted at the title. Fixture `browser/tests/ui/container_feed.html` over the test server (GitHub-shaped: 245 `@container` rules on nested containers, ~4000 boxes; 60 frames each change a list, others flip a side column's width, toggle a container and resize a `size` container), 1024×768, fast build, before (d514728) / after: `ui_relayout` after load p50 64.9 → 5.8 ms (p90 82.4 → 10.2), full cascades after load 122 → 0, full layouts 61 → 0 (3 layouts with the styles kept, when rebuilt boxes pass twice the tree, P3.64), `ui_loop` total 4581 → 919 ms; 60 container rounds (new list items and the flips). `DWELLER_LAYOUT_CHECK=1` on it: 0 differences (60 checks); final screenshots equal main's. Live GitHub is unchanged (`ui_relayout` after load p50 460–570 ms, 15 cascades): its document has a shadow root, which keeps the full path; that is the next step for GitHub (shadow trees in `restyleSubtrees` and `RestyleDeps`). Test `window_container_queries` (no cascade after the load, container rounds, LAYOUT_CHECK silent); `DWELLER_LAYOUT_CHECK=1 cargo test --release --test browser -- jsdom_fixtures layout_fixtures script_layout interleaved window_` → 0, the full `--test browser` 79 passed.

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
- status P4.40a: done 2026-10-16 (split: the gate itself; CI is P4.40b). Pages `browser/tests/perf/{feed,animated,paint_heavy,article}.html` (+ their css/js), served from `browser/tests/` by `server.py`'s existing `/delay/MS/NAME` (no new route); feed has 110 delayed images, 4 sheets, 6 scripts and a ticker that `fetch()`es `/sleep?ms=600` every 500 ms into a fixed-height box. New DWELLER_TIMING records: `style_full` (css `computeStyles`), `net_sync` (each synchronous HTTP transfer in http.aye and each `net.waitLoad`), `raster_px` ("PIXELS RECTS" of a frame that drew, pagedraw.aye); `layout_full` counts the existing `layout_full` and `layout` records. `window_perf.py gate BUDGETS [--update]` (`browser/tools/perf_gate.py`): per page, 3 headless runs (`painted`, 60 frames, 30 frames of `wheel:-3`, 60 frames; 1024×768), median of each; `raster_px` is the mean over the 30 scroll frames (P4.42 and P4.36c: a wheel's frame draws in the next loop iteration, which the window is aligned on). Exit 1 when a counter is over budget (slack: `layout_full` ×1.2, `raster_px` ×1.02, since image arrivals and the ticker are paced by the wall clock, ±3 full layouts between runs here) or under 0.8 × budget (ratchet: lower it with `--update`, which never raises a budget). Times (first paint, longest loop, scroll p95) are printed, not gated. Run time ~30 s. On this Linux box: exit 0 three runs in a row; with `restyle` forced to fail in geometry.aye (a full restyle per relayout, not committed) exit 1 (`animated` `style_full` 1 → 160, `layout_full` 55 → 320). `cargo test --release --test browser -- window_` 45 passed.

#### P4.40b: the CI `perf` job
- goal: P4.40a's gate runs on every PR: a `perf` job in `.github/workflows/ci.yml` on `macos-latest`, after the build, with the cached `aye`: `python3 browser/tools/window_perf.py gate browser/tests/perf/budgets.json --binary target/dweller`.
- budgets: recorded on Linux (P4.40a); run `gate --update` on the macOS runner once (a workflow_dispatch input or the job's printed table) and commit those values, after checking the spread of `layout_full` over a few runs there; widen its slack only if the runner needs it.
- files: `.github/workflows/ci.yml`, `browser/tests/perf/budgets.json`
- verify: the `perf` job green on the PR; a branch with the deliberate regression of P4.40a fails it
- time box: 45 min; model: sonnet
- status P4.40b: done 2026-10-16 (PR #108). The `perf` job (macos-latest, own copy of the browser job's setup, `aye build browser -o target/dweller`, then the gate) runs about 2.5 min. Budgets re-recorded from the runner: 5 gate runs in one job plus the first run, no `--update` (it never raises a budget; the gate's table prints on failure). Spread on the runner: `layout_full` feed 18–20, article 13–15, animated 55 (steady); `raster_px` animated 397995 or 422161 (budget the higher), the others steady; `net_sync` of feed 1–76 (image load tasks call `net.waitLoad` as far as the wall clock lets them run, 110 images): budget 110, and `net_sync` has no ratchet (`UNRATCHETED` in `perf_gate.py`). Slack unchanged. With a full restyle forced per relayout (throwaway commit, reverted) the job fails (`animated` `style_full` 159, `layout_full` 318, `feed` `style_full` 8); green again after the revert.

#### P4.40c: net_sync counts only synchronous network waits
- goal: `net_sync` is a deterministic count of the waits where the window thread blocks on the network (Phase A exit: none in a loop iteration), so it is back in the ratchet. P4.40b found it ranging 1–76 on `feed` because `net.waitLoad` was counted for every timer task of an image or async fetch completion, which the wall clock paces.
- change: `waitLoad` (loader.aye) no longer records; `prefetch` (multi.aye), the one forced wait in the loader, records `net_sync` around each wait; the synchronous transfer in http.aye still does. Image and async completions keep their own `image_wait` record (script.aye `runTimer`), reported by `window_perf.py` and not in the budgets. `UNRATCHETED` is gone from `perf_gate.py`.
- files: `browser/net/loader.aye`, `browser/net/multi.aye`, `browser/tools/perf_gate.py`, `browser/tests/perf/budgets.json`
- verify: `gate` five times in a row: `net_sync` identical on every page
- time box: 30 min; model: sonnet
- status P4.40c: done 2026-10-16. `net_sync` is 1 on `feed`, `animated`, `paint_heavy` and `article` (the page's own document fetch), identical in 5 gate runs on Linux; budgets set to 1. The count is deterministic, so the macOS runner's values must match (1 each; any other value there is a regression, not noise). Left alone: on this Linux box `article` `layout_full` measures 26–28 (budget 15, from the macOS runner) and `feed` 13 (budget 20), on unmodified main too, so the gate as a whole exits 1 here for those two; `net_sync` rows are `ok`.

#### P4.40d: style_full and layout_full count deterministically
- goal: `layout_full` is a deterministic count, so the ratchet (a metric under 0.8 × budget) may fail the gate again. It varies 10-20 between runs on CI because it depends on wall-clock pacing, as `net_sync` did before P4.40c: it counts full layouts per frame, and image arrivals and the ticker are paced by the clock. Count the relayouts caused by document changes (one per change of the document, not one per frame).
- change: until then the ratchet is a hint: a metric under 0.8 × budget is printed `LOWER` plus one line naming how many could be lowered with `gate --update`, and only `OVER` and `new` fail the gate. When `layout_full` is deterministic, `LOWER` fails again.
- files: `browser/layout/`, `browser/ui/`, `browser/tools/perf_gate.py`, `browser/tests/perf/budgets.json`
- verify: `gate` five times in a row: `layout_full` identical on every page, on Linux and on the macOS runner
- time box: 60 min; model: opus
- widened (2026-10-16) to `style_full` too: main at c5747e9 failed CI with `feed` `style_full` 3 (2 idle, 3-8 with the CPU loaded).
- status P4.40d: done 2026-10-16. The hint (`LOWER` no longer fatal) landed first as P4.40: LOWER is a hint, not a failure (PRs #112, #113, #114 failed at random on it). Causes: (1) each batch of image arrivals (paced by `IMAGE_RELAYOUT_MS` and the wall clock) laid the page out again: in full before its scripts ran, and after them by rebuilding the nearest block-flow ancestor (`main`, a grid: the whole feed), whose dead boxes forced a full layout every few rebuilds; (2) a change to an element's children counted as "deep" (`RestyleDeps.isDeep`), so the ticker's update restyled the ticker and its following siblings (`main`, the whole document) each time, and after `RESTYLE_ROUNDS` of those the script's layout fell back to a full cascade and layout, then the window did its own (the double `style_full` of P4.69's diagnosis). Fix, after Blink (`LayoutImage::ImageChanged`, `Element::ChildrenChanged`/`CheckForEmptyStyleChange`, `Document::UpdateStyleAndLayout`): an image whose box is sized by its style (`layout.sizedByStyle`: width and height are lengths) only repaints (`imagesKeepLayout`, and `noteImageChanges` skips it); `Page.layoutPage` keeps a clean layout (`keepCleanLayout`: same viewport, sheets, DOM version, current styles, no font change, only such images; the script adopts it as `layoutPage` does); a children change is deep only when a selector looks at `:empty` outside its subject, else its restyle root is the element's subtree. The forced read and the frame share one subtree restyle and one incremental relayout (the geometry's version is the clean flag). Gate on Linux, 3 runs idle and 3 with 4 `yes` loops: `style_full` feed 2, animated 1, paint_heavy 1, article 1 in every run; `layout_full` feed 1 (was 13-18), paint_heavy 1 (1-2), article 2 (23-29), animated 54 idle and 53-54 loaded. Scratch pages with the ticker running all along (scripts inline, no images; one with an `offsetHeight` read per update), 700 frames: `style_full` 2 and full layouts 2 idle and loaded, any number of updates (was 4 and 4). Budgets: `layout_full` feed 20 → 1, paint_heavy 2 → 1, article 15 → 2 (`raster_px` and animated left at the macOS values). Deterministic now: `style_full` and `net_sync` everywhere, `layout_full` of feed, paint_heavy, article. Not: `layout_full` of animated (a class change on a flex item rebuilds its whole flex container, 3000 items, whose dead boxes force a full layout every third frame: incremental flex relayout is a card of its own) and `raster_px` (P4.40b), so `LOWER` stays a hint in `perf_gate.py`. `cargo test --release --test browser -- window_ jsdom_fixtures dump_fixtures layout errors` 59 passed; `aye test browser` 155 passed.

#### P4.35: raster cache
- goal: gradient tiles, SVG images, blurred shadows and filter/opacity layers are rasterized once per display item at device size and reused while the item and its size are unchanged (cause 3). P4.5 (decode offscreen images at paint time) joins here.
- files: `browser/paint/raster.aye`, `browser/paint/background.aye`, `browser/paint/filter.aye`, `browser/paint/paint.aye`, `browser/svg/`
- chromium: `third_party/blink/renderer/core/paint/README.md` ("Paint result caching"), `third_party/blink/renderer/platform/graphics/compositing/README.md` (raster invalidation). Take: an item keeps its result while its object is not invalidated, so a frame redraws only invalidated items. Not taken: paint chunks, property trees.
- verify: `window_perf.py scroll https://react.dev` frame p50 < 30 ms (now 201); the paint and screenshot goldens unchanged: `cargo test --release --test browser -- screenshot_ check_png > target/p4.35.log 2>&1; echo $?` → 0; the perf gate's `paint_heavy` budget lowered
- time box: 120 min; model: opus
- status: done 2026-10-15, with damage tracking (decision of the main session: do what browsers do, no frame-rate cap). The window keeps the page's pixels between frames (`ui/pagedraw.aye`, drawn from the page's own origin): each display item has a signature (paint/damage.aye: a hash of everything it draws with, its context of clips and layers mixed in, and its area), and a frame draws only the items that differ from the last frame's (the two lists aligned on their common items) plus, after a scroll by whole pixels, the band that came into view (the rest is moved); a layer whose filter blurs is drawn whole when touched. No damage and nothing else changed: nothing is drawn or presented. A frame of the timers or a scroll copies and uploads only the damage (all of the page after a scroll) without composing the chrome. The raster cache (paint/rcache.aye) keeps layers of at most 2 Mpx (over their whole box, ready to blend), gradient tiles of at most 8 Mpx (their colors, each row computed the first time it shows) and shadows (their blurred coverage), keyed by their items and their position inside the pixel grid. In the window, SVGs, gradients, shadows and rounded corners are rasterized from whole pixels of their own, so a scroll or a kept result gives the pixels a new drawing gives; `screenshot` and `dump` keep the previous arithmetic (goldens and the 22 archived screenshots byte-identical). The rounded-box coverage rasterizes one row between the corners and copies it (both paths, identical bytes). Canvas bitmaps count their changes (`Bitmap.version`, in `Item.Image`). `window_perf.py scroll https://react.dev` (against a build of main): loop p50/p95 190/522 → 43.5/249 ms, draw p50/p95 167/489 → 25.7/112 ms; the loop p50 stays over 30 ms because react.dev's scroll listeners change the DOM in half the frames (timers and relayout ~20 ms: P4.42, P4.44) and each frame draws the 240 px band (its conic gradient ~12 ms) and the sticky header again (`paint_build` 3–7 ms rebuilds the list for it: P4.36). `window_perf.py idle https://mistral.ai`: 101.7 → 98.6%; its rAF animation moves a carousel 1 px a frame, so every frame damages 0.91 Mpx (31% of the window, median); per frame the draw is 10.1 → 6.0 ms and the loop 20.8 → 14.2 ms, and the animation runs at 59 instead of 40 frames/s; what is left is each frame's relayout (6.8 ms) and its scripts: browsers composite such a transform animation without layout or paint. `DWELLER_DAMAGE_CHECK=1` draws every frame whole too and logs "DAMAGE CHECK" with the pixels that differ (and when the window differs from the page): none over the 22 recorded sites (a dummy window and a real one at scale 2, 60 frames with wheel steps), live react.dev, and the window tests (`window_fetch_in_background` then misses its 300 frames in 60 s in the debug build, each frame being drawn twice). `page_load.py check --screens` exit 0, screenshots byte-identical with main. The perf gate (P4.40) has not landed: no budget to lower. Test `window_damage` (a canvas animation under the check: partial frames, frames with nothing drawn). rAF throttling went to P4.55.

#### P4.56: transform and opacity changes skip layout and re-raster
- goal: as browsers do, a change of `transform` or `opacity` alone lays nothing out and rasterizes nothing again: the element's layer is kept and moved or blended. Promote an element whose transform or opacity changed in 2+ frames in a row, and one with `will-change: transform|opacity`.
- status: done 2026-10-15. The cascade notes elements whose values changed only in what paint reads, or in what layout reads only in their transform (`css.Styles.restyled`); geometry keeps their boxes, points them at the new values (before, a kept layout left the boxes on the old ones: an opacity-only change never reached the window) and places a transformed subtree again from where the last pass of transforms found it (`layout.retransform`, `Box.ux/uy/uroot`: the same arithmetic as a full layout, so `DWELLER_LAYOUT_CHECK` matches). The window promotes the layer boxes of elements whose transform or opacity changed in 2 relayouts in a row, or whose `will-change` names them (new property, paint-only): `paint.buildPromoted` paints such a box as a group whose items are placed without its translation (`layout.refold`), the translation kept beside the list (`DisplayList.layers`) and added in whole device pixels when the layer is blended; the raster cache keeps layers before their opacity, which blending applies. A relayout whose changes were all transforms and opacities of promoted boxes (`engine.paintChanges`) patches the list (`paint.patchLayer`: the `Layer` signature and its items' offsets, the `EndLayer` opacity) instead of building it: DWELLER_TIMING `ui_composite` instead of `ui_relayout`; the damage is the layer where it was and where it is, and the kept layer is blended there (`ui_layers`: layers blended from the cache, layers rasterized). Test `window_transform_composite` (60-frame rAF carousel, 600x300 track 1 px a frame and a fading box, under both checks): 58 composited frames, 0 relayouts and 0 layers rasterized after the first. `DWELLER_DAMAGE_CHECK=1 DWELLER_LAYOUT_CHECK=1`: no difference on the window tests and live mistral.ai, react.dev, news.ycombinator.com; `page_load.py check --screens` exit 0 (screenshots and dumps do not promote). mistral.ai is not a transform animation: per frame its scripts change the `width` of `.js-square` divs (a layout property: browsers lay it out too) and a layout bump that is not noted (`layoutVersion` moves without `layout_noted`) makes the frame a full layout; `window_perf.py idle` main / this: idle CPU 97.9 / 91.4%, relayouts per loop 0.97 / 0.79, draw 6.9 / 4.4 ms per loop. That is P4.57.

#### P4.58: window tests drain the window's output
- goal: `window_paints_early` and `window_fetch_in_background` pipe the window's stdout and stderr and read them only after it exits; with DWELLER_TIMING their stderr is ~57 KB, so a few more records per frame fill the 64 KB pipe and the window blocks (found in P4.56: one more record per frame failed both on CI as "did not draw 300 frames in 60 s"). Read the pipes on threads while waiting.
- status: done 2026-10-15. tests/browser.rs `Command::spawn_drained` (trait `SpawnDrained`) pipes stdout and stderr and reads each on a thread; the `Drained` it returns has `Child`'s `id`, `try_wait`, `kill`, `wait` and `wait_with_output` (which joins the readers). Every test that spawns the window and reads its output after it exits uses it: `window_images_load_in_background`, `window_relayout_after_image_load`, the agent session runner (`window_agent_session`), `spaces_window`/`spaces_window_env` (whose two DWELLER_TIMING tests drained stderr by hand), `window_timers_yield_to_frames`, `window_fetch_in_background`, `window_paints_early`. The tests that write stdin and then `wait_with_output` at once already read both pipes while waiting.

#### X1: tests and checks never take the user's focus
- lane: browser (Layer 0, tests)
- goal: the user works on the same Mac while agents run tests, and real SDL windows steal the keyboard focus. Tests and checks run with SDL's dummy video driver unless asked; the rare real-window measurement runs as a background app that is never raised.
- files: `tests/browser.rs`, `tests/browser/dweller_command.rs`, `tests/browser/perf.rs`, `browser/tools/window_env.py`, `browser/tools/window_perf.py`, `browser/tools/game_check.py`, `browser/ui/background-hint.aye`, `browser/ui/window.aye`, `docs/practices/browser.md`
- verify: `cargo test --release --test browser -- window_ > target/x1.log 2>&1; echo $?` → 0 with no window appearing (the frontmost app unchanged before and after); `DWELLER_BACKGROUND=1 target/dweller --frames 30 about:blank` leaves the frontmost app unchanged (`osascript -e 'tell application "System Events" to get name of first process whose frontmost is true'` before and after)
- time box: 45 min; model: sonnet
- status: done 2026-10-16. Every test spawn of the browser goes through `dweller_command()` (`tests/browser/dweller_command.rs`), which sets `SDL_VIDEODRIVER=dummy`, or `DWELLER_BACKGROUND=1` when `DWELLER_TEST_REAL_WINDOW=1`; the 24 `.env("SDL_VIDEODRIVER", "dummy")` lines are gone and no test needs a real driver. `window_perf.py` and `game_check.py` use the dummy driver and take `--real-window` (`window_env.py`: `DWELLER_BACKGROUND=1`). `DWELLER_BACKGROUND=1` sets the SDL hint `SDL_MAC_BACKGROUND_APP` before `SDL_Init` (`ui/background-hint.aye`); the window is never raised. Verify: 44 window tests pass; the real-window run (30 frames) left the frontmost app unchanged.

#### P4.80: `window_paints_early` waits on records, not wall time
- lane: browser (Layer 0, tests)
- goal: the test clicks its checkbox after 40 `frame` words and expects the click before the page's script, which the server delays by 1500 ms; on the macOS CI runner 40 frames take longer than that, so the click lands after the script and "checked before it" is false (runs 38701882826, 38710550094; Ubuntu green). Make the ordering a property of the loop, as #42 did for the other timing tests: the events script waits for the first paint record (a `--events` word that waits for `ui_first_paint`, or a `frame` count derived from the records), and the delay is checked against the `script` record, not against frames. No change to the window.
- files: `tests/browser.rs` (`window_paints_early`, the `--events` word if new), `browser/ui/script.aye` (the word)
- verify: `cargo test --release --test browser -- window_paints_early window_fetch_in_background > target/p4.80.log 2>&1; echo $?` → 0, five runs in a row under `nice -n 19` with the machine busy (`yes > /dev/null &` ×4), and green on the PR's macOS job
- time box: 45 min; model: sonnet
- status: done 2026-10-16. New `--events` word `painted` (`browser/ui/script.aye`): the words after it wait until the page's first paint (`Ui.paint_began` is 0, the `ui_first_paint` record). `window_paints_early` now runs `painted click:14,54 frame wheel:-3 frame`, so the click comes 0.3 s into the script's 1.5 s delay on any runner; the `script` and image records are still checked against the first paint's iteration. The window is unchanged. Verify: the two tests pass; five runs in a row under `nice -n 19` with 4 `yes` processes: 5 of 5 (6.6-6.7 s each).

#### P4.81: `window_spaces_chips` does not bound the loop by wall time
- lane: browser (Layer 0, tests)
- goal: `window_spaces_chips` asserted `loop_max_ms` under 120, a wall-time bound; it failed on 2026-10-16 while other builds ran and passes alone. The property it stands for (the window never waits for a Space) is already checked from the loop's records: loop iterations between each Space's first request and its answer. Keep the records, and leave only a wall-time bound that a broken window reaches.
- files: `tests/browser.rs` (`window_spaces_chips`)
- verify: `cargo test --release --test browser -- window_spaces_chips > target/p4.81.log 2>&1; echo $?` → 0, five runs in a row under `nice -n 19` with 4 `yes > /dev/null &` running
- time box: 45 min; model: sonnet
- status: done 2026-10-16. Relaxed, not moved: the 120 ms bound is now 1000 ms, as in `window_spaces_responsive` (a window that waits for a Space holds an iteration for the Space's whole answer); the ordering check (at least 3 loop iterations between each request and its answer) stays as it was and is what proves the window does not wait. Verify: 5 of 5 under `nice -n 19` with 4 `yes` processes (4.9-5.1 s each).

#### P4.57: mistral.ai idle CPU
- goal: mistral.ai idle under 25% CPU. Found in P4.56: its rAF animation changes the `width` of `.js-square` elements a pixel a frame (`moved` in the cascade every frame), and in ~70% of the frames `localChanges` fails on `version - g.version != layout_noted - g.noted` (a `layout_version` bump without a note: `image_api.aye` new bitmap, `dom_api.aye:106` or `engine.aye` web fonts / image changes), so the frame styles and lays out the whole document instead of `relayoutDirty`. Find the bump, keep such frames incremental, and measure what is left (the scripts' timers, `paint_build`).
- status: done 2026-10-15 (the full layouts; idle CPU is not under 25% yet). The bump: `images.loadElementImage` dropped an element's mapping (`Images.map.remove`) before putting it again, so `Images.noteChange` saw every image as new, and each relayout of a page with images moved `Images.changes`; `engine.aye` then bumped `layout_version` unnoted ("new bitmaps") and the frame laid the whole document out again (on mistral.ai after the incremental one). Now `Images.drop`/`settle` count a change only when the element's image is not the one before, and the elements whose image did change are recorded (`Images.changed`, or `any_changed` for a style sheet's image or more than 256) and noted as changes to them (`geometry.noteImageChanges`, from the window's relayout and from `imageLoad`), as is an image attribute (`src`, `alt`, ...: `dom_api.changedAttr`); the only unnoted bumps left lay everything out on purpose (web fonts, sheet images). Then the squares climbed to `body` to be built again (a flex item or a grid item is not in block flow), and the boxes built again outgrew the `relayoutDirty` bound every few frames: `layout/incremental.aye` now builds and lays out again a block-level flex or grid container in block flow (`rebuildKind`, `blockLevelKind`), its items with it. `relayoutDirty` points the kept tree at the page's images (made after the first layout, a background image). New DWELLER_TIMING records `layout_full` and `layout_dirty` (script layouts of the whole document, of the changed boxes); `window_perf.py idle` counts them (`full_layouts`, `dirty_layouts`) and reads Linux `ps` times. Test `window_width_animation` (`browser/tests/ui/squares.html`: 6 `.js-square` divs in the flex cells of a grid, images, 500 paragraphs; 60 rAF frames, DWELLER_LAYOUT_CHECK): 0 full layouts after the first frame, one dirty relayout a frame (main: 60 full and 60 dirty); `DWELLER_LAYOUT_CHECK=1` on all window and layout fixture tests: no difference. mistral.ai `window_perf.py idle` (Xvfb, 4-CPU cloud box) main / this: full layouts per loop 1.01 / 0.02, loop 31.2 / 16.2 ms, `ui_relayout` 15.3 / 5.5 ms per loop, iterations in 10 s 417 / 700, idle CPU ~110 / 100% (the loop now runs at the frame rate). Left: per loop `restyle` 4.6 ms, the scripts' timers 3.3 ms, `ui_draw` 3.3 ms, `paint_build` 2.4 ms; a whole-document restyle every ~40 frames (`RESTYLE_ROUNDS`, which frees what restyles allocate).

#### P4.57b: mistral.ai idle CPU, the rest
- goal: mistral.ai shown and idle under 25% CPU (93.4% on the user's Mac after P4.57, P4.55b's measurement); Chrome's cost there is the reference (`window_perf.py` has no `--chrome`: not measured). Profile one idle second per phase and remove the largest cost Chrome does not pay, then the next.
- blink: `third_party/blink/renderer/core/layout/layout_object.cc` (relayout boundaries), `core/frame/local_frame_view.cc`, `core/paint/paint_invalidator.cc`
- verify: `window_perf.py idle https://mistral.ai` before/after with the per-phase split; `window_perf.py gate browser/tests/perf/budgets.json --binary target/dweller` exit 0; `cargo test --release --test browser -- window_ layout dump_fixtures screenshot_fixtures errors > target/p4.57b.log 2>&1; echo $?` → 0 with `DWELLER_DAMAGE_CHECK=1`
- status: partly done 2026-10-16 (idle CPU is not under 25% yet). The largest cost: mistral.ai has shadow trees, and a document with shadow trees never shared its layout with the scripts' (`adoptLayout` was skipped), so every frame's `layoutPage` styled the whole document again (`restyleWithSizes`, 66 ms) and laid it out in full. Now the page's layout is shared with shadow trees too: a change whose restyle roots are clear of shadow trees (`clearOfShadows`, as `geometry.restyle` already required) styles only those subtrees, and `relayoutDirty` builds boxes again when the elements it rebuilds are clear of them (else the full layout as before). Linux, dummy driver, `window_perf.py idle` main / this: idle CPU 99.9 / 89.9% (ratio 0.90; the loop now runs 4.9x the frames: 71 / 346 iterations in 10 s), per loop 134 / 32 ms; per loop ms: `style_full` 65.8 / 2.4, `ui_relayout` 79.1 / 7.4, `paint_build` 6.2 / 2.6, `restyle` 6.2 / 5.0, `script_layout` 15.5 / 6.9, `ui_raf` (scripts) 92.0 / 20.6, `timers` 37.4 / 17.9, `ui_draw` 16.7 / 4.9; full layouts 95 / 22. `DWELLER_LAYOUT_CHECK=1 DWELLER_DAMAGE_CHECK=1` on live mistral.ai: no difference. Left: the page's own scripts (rAF and timers, ~38 ms a loop: the interpreter), the display list built whole after each dirty relayout (`paint_whole`, ~6 ms; Blink repaints only the invalidated boxes), the full restyle every `RESTYLE_ROUNDS` (13 in 10 s, 61 ms each); `window_webgl` not run here (ANGLE's archive did not download).

#### P4.55: rAF and timers throttled when not seen
- goal: as the HTML spec lets browsers do, a page nobody sees runs no animation frames: `requestAnimationFrame` callbacks wait while the window is minimized or hidden (`SDL_WINDOWEVENT_HIDDEN`/`MINIMIZED`, back on `SHOWN`/`RESTORED`) and in background tabs; timers of background tabs run, clamped to at least 1 s (today they do not run at all: the loop runs the shown tab's timers only); an offscreen iframe's rAF may wait too. A fully occluded window needs SDL3's `SDL_EVENT_WINDOW_OCCLUDED` or a Cocoa `occlusionState` query; note it if neither fits. Visible pages keep their frame rate (decision for P4.35).
- files: `browser/engine/script.aye` (rAF timers flagged, held while the page is hidden), `browser/engine/prelude.aye` (`requestAnimationFrame`), `browser/ui/window.aye` (window events, the timers of other tabs), `browser/ui/tabs.aye`, `browser/ui/sdl.aye`
- chromium: `third_party/blink/renderer/core/frame/local_frame_view.cc` (throttled frames), `third_party/blink/renderer/platform/scheduler/` (background timer throttling, 1 s alignment).
- verify: `window_perf.py idle https://mistral.ai` with the window minimized after its first frame: idle CPU < 2% (98.6% shown); a test with two tabs: the background tab's 100 ms interval runs at most once a second and its rAF not at all, both resume when shown; `cargo test --release --test browser -- window_ > target/p4.55.log 2>&1; echo $?` → 0
- time box: 60 min; model: sonnet
- status: done 2026-10-16, except the real-window measurement (P4.55b). A page is hidden while its window is minimized or hidden or its tab is in the background (`syncTabs`, ui/sync-tabs.aye, each loop iteration and before the timers): `engine.setHidden` marks it (and its frames) and fires `visibilitychange` once per change; a hidden page runs no animation frame (`animationFrameRequested` is false, its rAF timers never run, `msUntilNextTimer` is null). The timers of a hidden page run at most once a second (`runBackgroundTimers`: each tab's `next_run_ns`, the shown tab too while the window is hidden); the loop's 1 s wait wakes them. Not done: an offscreen iframe's rAF, a fully occluded window (SDL2 has no occlusion event), `visibilityState` of iframe documents. SDL's dummy driver sends HIDDEN events for a window its flags say is shown, so a real event counts only when the window's flags are hidden or minimized (`windowGone`; a scripted event has no window id and counts). The script words `window:hidden` and `window:shown` (SDL's minimized and restored) already existed and feed the same handler, so no `hide`/`show` words were added. Tests `window_hidden_raf` (no rAF between the two visibilitychange events, rAF back after) and `window_background_tab_throttled` (background interval ticks at least 1 s apart, its rAF not at all, both resume when shown).

#### P4.55b: idle CPU of a minimized window
- goal: measure P4.55 in a real window: `window_perf.py idle https://mistral.ai` with the window minimized after its first frame: idle CPU < 2% (98.6% shown).
- how: local macOS check with `DWELLER_BACKGROUND=1` (the window is a background app that is never raised, X1); the Linux cloud box runs SDL's dummy driver, which cannot minimize.
- verify: the idle CPU of the minimized window < 2%, shown ~ as before.
- time box: 20 min; model: sonnet
- status: measured 2026-10-16 on this Mac (`DWELLER_BACKGROUND=1`, mistral.ai, build of main). Shown: 93.4% (98.6% before). Minimized: `window_perf.py idle URL --real-window --minimize --settle 20` feeds `window:hidden` after the first paint (the window itself is not touched: SDL has no minimize call that is sure not to move the focus, and the real loop only reads scripted words under `--frames`, whose loop wakes every 10 ms and draws every iteration): 3.3%, of which the page's timers are 69 ms in 10 s (0.7%, no rAF, no relayout, no damage) and the rest is that test loop (a static page measures 1.5% the same way). With the default 5 s settle the page's late scripts were still loading: 6.4%. The real loop (1 s wait, no draw while hidden) is under the 3.3%, so < 2% holds by that sum but is not measured directly; a direct measure needs a real minimize or a script feed in the real loop.

#### P4.36: scroll by moving the last frame
- goal: a scroll moves the previous frame's pixels and draws only the band that scrolled in, then fixed and sticky boxes over it; a frame with no change draws nothing (cause 3).
- note 2026-10-15: P4.35's damage tracking moves the pixels on a scroll by whole pixels and draws the band and what changed (fixed and sticky boxes too), and a frame with no change draws nothing. Left: the list is built again on each scroll frame for fixed and sticky boxes (`relist`, react.dev 3–7 ms), and a scroll by a fraction of a device pixel (a trackpad) draws everything (from the raster cache); the verify numbers are not measured yet.
- files: `browser/ui/window.aye` (draw), `browser/paint/raster.aye` (draw a band), `browser/paint/paint.aye` (fixed/sticky items apart)
- chromium: `docs/how_cc_works.md` (tiles, scrolling, damage), `docs/life_of_a_frame.md`. Take: scrolling is an offset over pixels already drawn, and a frame draws only its damage rectangle. Not taken: a compositor thread, tiles, layer trees.
- verify: Wikipedia and Medium scroll frame p50 < 4 ms (now 9, 26); `cargo test --release --test browser -- window_wheel window_scroll window_sideways_scroll > target/p4.36.log 2>&1; echo $?` → 0; the perf gate's `raster_px` per scroll frame ≤ the band's area
- status P4.36b: done 2026-10-16. Took Chrome's cc snapping (`cc/trees/property_tree.cc` `TransformTree::UpdateSnapping`): the scroll offset stays a float (the scripts and the wheel's sum see it exact), but the window draws the page, places fixed and sticky boxes (`followScroll`) and its overlays at that offset rounded to whole device pixels (`ui/snapped-scroll.aye`), so a trackpad's 0.3 px step moves the last frame by 0 or 1 device pixel and draws the band, the fixed and sticky boxes, or nothing; text never lands between pixels. Not taken: holding the remainder back in the wheel sum (keyboard, smooth and script scrolls are fractional too). Scroll frame p50/p95 ms, archives, `window_perf.py scroll --wheel=` at scale 2 (main → this): ja.wikipedia 3.7 px a frame 23.7/25.0 → 6.9/7.6, 0.3 px 23.8/25.1 → 6.0/8.2; en.wikipedia 3.7 px 8.3/9.9 → 7.0/7.5, 0.3 px 9.5/10.0 → 5.5/7.1; whole notches unchanged (3.7–4.3). No Medium archive in `target/sites`. What is left in a frame that draws one row: shifting, composing and uploading the whole window's pixels (memmove, Metal texture upload), as in a whole-pixel scroll; the < 4 ms p50 needs the page scrolled as a texture offset on the GPU. `DWELLER_DAMAGE_CHECK=1`: no difference at scale 1 and 2. Test `window_fractional_scroll`.
- time box: 90 min; model: sonnet

#### P4.36c: the page scrolls as a texture offset
- goal: a scroll frame uploads only the band that came into view, not the whole window's pixels: scroll frame p50 < 4 ms on ja.wikipedia and en.wikipedia (archives, `window_perf.py scroll --wheel=` at scale 2, 0.3 and 3.7 px a frame; P4.36b: 6.0–7.3 ms), without P11.22's GPU compositing; fixed and sticky boxes and the overlays stay correct; `DWELLER_DAMAGE_CHECK` still matches whole drawings.
- files: `browser/ui/pagedraw.aye`, `browser/ui/window.aye`, `browser/ui/script.aye`, `browser/ui/sdl.aye` (wiring only), new `browser/ui/*.aye` files, one function each
- chromium: `docs/how_cc_works.md` (scrolling as an offset over pixels already drawn). Take: the page's pixels are a ring of rows in a texture of their own; a scroll moves the ring's origin and the texture is drawn as two rects. Not taken: uploading only the window's damaged rects (after a scroll every pixel of the page differs), tiles, a compositor thread.
- verify: `cargo test --release --test browser -- window_fractional_scroll window_wheel window_scroll window_sideways_scroll window_damage > target/p4.36c.log 2>&1; echo $?` → 0; scroll frame p50/p95 before/after on both archives with 0.3 and 3.7 px steps
- status: done 2026-10-16. Chose the ring buffer over damaged-rect uploads: after a scroll every window pixel of the page differs, so only a texture offset avoids the full upload. `PageDraw` keeps `ring` (the canvas row of the page's row 0) and a page texture `tex`; a vertical scroll rolls `ring` (`rollRing`) instead of moving the rows (a sideways one still shifts the rows and uploads them all); each damage rect is drawn as up to two parts with the view moved by their offset (`ringPart`, `drawRinged`), and only those parts are uploaded (`uploadPage`, `SDL_UpdateTexture` on sub-rects). A frame without overlays presents the window texture as it was (the chrome) and the page texture as two rects (`presentPage`): no memmove, no compose, no full upload. Frames with the chrome dirty or an overlay still compose the window on the CPU (`blitPage` unrolls the ring), and `--save` composes the page first when the window canvas is stale. `DWELLER_DAMAGE_CHECK` now reads the presented pixels back (`SDL_RenderReadPixels`, `checkPresent`) and compares them with the page: no difference on both archives (dummy and Cocoa at scale 2; 0.3 px, 3.7 px and whole-notch steps, down and up), and it does log differences when one of the two rects is left out on purpose. Saved screenshots byte-identical with P4.36b. Scroll frame `ui_loop` p50/p95 ms, two runs each (P4.36b → this): ja.wikipedia 0.3 px 6.2/6.9, 6.0/6.7 → 0.3/0.4, 0.3/0.7; 3.7 px 7.2/7.7, 7.2/7.6 → 0.3/0.6, 0.4/0.8; en.wikipedia 0.3 px 6.2/7.0, 6.3/6.9 → 0.3/0.4, 0.3/0.6; 3.7 px 7.3/7.5, 7.3/7.5 → 0.3/0.5, 0.4/0.5 (`ui_draw` 0.2–0.3 ms p50: the GPU copies are asynchronous). Tests `window_fractional_scroll`, `window_wheel`, `window_scroll`, `window_sideways_scroll`, `window_damage` pass.
- time box: 45 min; model: sonnet

#### P4.36a: fixed and sticky items kept apart
- goal: a scroll frame builds no display list: fixed and sticky boxes are kept as their own item ranges, moved with the scroll offset at draw time, so `followScroll` sets no `relist` (react.dev `paint_build` 3–7 ms a scroll frame → 0).
- files: `browser/ui/window.aye` (`followScroll`, `draw`: wiring only), `browser/paint/` (a new file per function for the kept ranges), `browser/layout/scroll.aye`
- chromium: `platform/graphics/paint/paint_controller.cc` (display items reused between frames), `cc/trees/property_tree.cc` (scroll and sticky position as property-tree offsets). Take: fixed and sticky items carry a scroll offset of their own, applied at draw. Not taken: property trees, a compositor thread.
- verify: react.dev archive `window_perf.py scroll` `ui_relayout` p50 0 on frames without script changes; `cargo test --release --test browser -- window_wheel window_scroll window_sideways_scroll window_damage window_fractional_scroll > target/p4.36a.log 2>&1; echo $?` → 0 (`DWELLER_DAMAGE_CHECK=1` in `window_damage` and `window_fractional_scroll`)
- time box: 60 min; model: opus
- status: done 2026-10-16. In the window's build each fixed or sticky box (`Tree.scrolled`) is a promoted layer (P4.56) whose items are placed without the box's `scroll_dy` (`pinScroll`), with `scroll_dy` as the layer's translation; `followScroll` still moves the tree's boxes (hit tests, the scripts) and then moves those layers (`followPinned`, through `patchLayer`) instead of setting `relist`. It falls back to `relist` when a fixed or sticky box sits inside a promoted or transformed box (`DisplayList.unpinned`) or inside another layer. The layers come from the raster cache, so a scroll frame blends the sticky header instead of rasterizing it. react.dev archive, `window_perf.py scroll` (dummy driver; the page ends after 10 wheel frames): display lists built in the 39 scroll frames 16 → 6, all six inside a script relayout; on the frames without one, `paint_build` 3.4–3.9 ms → none, `ui_draw` 9.9–15.8 → 5.1–11.5 ms, `ui_loop` 15.1–20.9 → 10.6–16.8 ms. `fractional_scroll.html` (68 wheel frames): lists built 48 → 1. `DWELLER_DAMAGE_CHECK=1`: no difference there and on react.dev. Test `window_fractional_scroll` now asserts at most 2 lists built.

#### P4.37: one connection pool
- goal: connections, TLS sessions and DNS are reused across prefetch batches, page fetches and navigations: one curl multi handle (with a share handle) for the process, at most 6 connections per host, HTTP/2 multiplexed (cause 5).
- files: `browser/net/multi.aye`, `browser/net/session.aye`, `browser/net/loader.aye`
- chromium: `net/socket/client_socket_pool_manager.cc` (6 sockets per group), `net/docs/life-of-a-url-request.md`. Take: one pool per process keyed by scheme, host and port, idle sockets kept for the next request. With libcurl: `CURLMOPT_MAX_HOST_CONNECTIONS` 6, `CURLMOPT_PIPELINING` multiplex, one connection cache.
- verify: HN `DWELLER_TIMING` `sheets` < 350 ms live (now 753); `cargo test --release --test browser -- net_http http_ > target/p4.37.log 2>&1; echo $?` → 0
- time box: 60 min; model: sonnet
- status: done 2026-10-16. Every request of the session, sync `net.fetch` ones included, now runs on the loader's one multi handle (`performShared`), which with the session's share handle (connections, DNS, TLS sessions) is the process's one pool; `net/pool-options.aye` sets 6 connections per host, 16 total and `CURLMOPT_PIPELINING` multiplex explicitly. Live HN (`dump --layout`, 9 runs, behind the sandbox proxy): one TCP connection for the whole load (strace), the sheet prefetch 109–199 ms (median 111; card said `sheets` 753, which `dump` does not emit, so the `prefetch` batch is the number), the images batch 108–179 ms; one run hit the 10 s connect timeout (proxy). `net_http http_ fetch errors`: 11 of 11 pass.

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
- status: done 2026-10-16. `Ui.loop` handles input (SDL events, then the wheel), draws a due frame, then runs timers and networking tasks until 50 ms are spent (`engine.runDueTimersUntil(p, deadline)`, TASK_BUDGET_MS in ui/window.aye; `runDueTimers` keeps no deadline for `dump`, agents and Spaces); the rest waits for the next iteration, and the loop does not sleep while a frame is due. `docs/browser.md` "Threading model". The perf gate (P4.40) has not landed, so the storm is its own page: `browser/tests/net/timer_storm.html` (1.5 s of 3 ms tasks: 200 `setTimeout(0)`, 20 chains, 5 `setInterval(1)`, 20 slow `/sleep` responses), test `window_input_priority` (each iteration's `timers` ≤ budget + its longest task; input handled during the storm, tasks still run after it). Main / P4.42: longest iteration 700 → 54 ms, longest task phase 694 → 51 ms (`PERF.md` "P4.42"). `-- window_`: 43 of 44 pass; `window_pointer_lock` fails identically with main's binary on this machine (`SDL_SetRelativeMouseMode` fails under the dummy driver here: "locked failed UnknownError"), not this card.

### Login readiness and telemetry (before the user bundle)

The user's daily sites all need a login (P3.80). Audit of 2026-10-15,
read from code, for logged-in browsing in the human window:

| Area | Status | Evidence |
|---|---|---|
| Cookies persisted and restored | yes (P4.50) | Profile directory `~/.dweller` (`DWELLER_PROFILE` overrides; `browser/net/profile.aye`); the jar is written atomically within a second of a change and at exit, session cookies included, expired ones dropped on load; a second process on the profile is read-only. `DWELLER_COOKIE_FILE` still works (persistent cookies only). Test: `window_login_cookies`. |
| Cookie attributes | yes (P10.21) | Domain/Path/Expires/Max-Age/Secure/HttpOnly, `__Secure-`/`__Host-` checked; SameSite with a Lax default, `None` needs Secure; a pinned public suffix list (`net/psl.txt`) rejects `Domain=co.uk`; third-party cookies blocked (`cookies.aye contextOf`). Test: `net_samesite`. |
| localStorage / sessionStorage / IndexedDB | yes (P4.51), no IndexedDB | `localStorage` per origin in `<profile>/storage/` (`browser/engine/storage.aye`), loaded on first use, saved like the jar, 5 MB per origin, `storage` events to the origin's other pages; `sessionStorage` per tab and origin across navigations, copied to a popup with an opener. Test: `window_login_storage`. No IndexedDB. |
| Form login and redirects | yes | Redirects followed in `net/http.aye` with each hop's `Set-Cookie` stored (`http.aye:1-6`, `:237`); 303 and 301/302-after-POST become GET (`:253-258`); POST forms (`window_forms`, `tests/browser.rs:1466`); password fields masked (`ui/forms.aye:435`, `layout/controls.aye:284`). No autofill or password manager (not planned). |
| OAuth/SSO popups | yes (P4.52) | `window.open`, `target=_blank`, `opener`, `close()`, `closed` and `postMessage` between tabs (`engine/contexts.aye`); test `window_login_popup`. Popup blocking not done. Cross-domain redirect chains work (cookies per hop). |
| HTTP Basic auth | yes (P10.22) | A 401 Basic challenge opens a sign-in sheet in the person's window (`ui/auth.aye`); credentials per origin and realm for the process, sent at once under their directory (`net/auth.aye`); URL userinfo works and is cached; agents get `auth_required`. Tests: `window_login_basic`, `window_login_basic_prompt`, `window_login_basic_cancel`, `net_basic_auth`. No Digest. |
| CSRF tokens, 2FA code pages | yes | Plain forms and scripts; nothing special once cookies and forms work. |
| WebAuthn / passkeys | no; out of scope for now | No `navigator.credentials` anywhere. The user can log in with password and a 2FA code. Unknown whether any of their sites force passkeys; if one does, their report makes it a card. |
| fetch/XHR `credentials` | yes (P10.21) | `omit`, `same-origin` (default) and `include`, XHR `withCredentials`, passed to `ak.fetch`/`ak.fetchStart`; `omit` sends and stores nothing. No CORS enforcement (a compatibility gain, not a login blocker), so no `Access-Control-Allow-Credentials` check. |
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
- status: done 2026-10-15. `net/cookies.aye`: SameSite parsed (missing or unknown: Lax; `None` without Secure rejected), name and value over 4096 bytes rejected, a public-suffix Domain rejected unless it is the host (host-only); the jar file gets a SameSite field after the expiry (older lines load as Lax). `net/psl.aye` + `net/psl.txt`: Mozilla's list pinned at VERSION 2026-10-07_07-28-19_UTC (COMMIT 3929462652695bad04f0a27afb600974014a3c8b, MPL 2.0 note in the header), both sections in punycode, sorted, embedded with `@embedFile` and searched by binary search (`browser/tools/gen_psl.py` regenerates it). `contextOf` gives each hop a context: same-site; cross-site top-level GET (Lax and None sent) or POST (None only), sticky over a redirect chain, the initiator being the navigating page or the chain's first URL; third-party (another site than the top-level page: `Request.top` for a frame's document, the frame registry `net.noteFrame` for a frame's subresources, else the referrer) or omit. Third-party policy: the goal's "except `SameSite=None; Secure`" was dropped for a full block, as in Safari and Firefox ETP (the routine's instruction): nothing sent, `Set-Cookie` not stored, `document.cookie` empty in a third-party frame; no exceptions yet. `fetch` sends `credentials` (default `same-origin`, checked per hop against the page's origin), XHR `withCredentials` (`include`, else `same-origin`); `omit` sends and stores nothing. CORS is not implemented, so there is no `Access-Control-Allow-Credentials` check. Test `net_samesite` (server `/samesite/`, `127.0.0.1` vs `localhost`, a profile between runs: Lax default sent on a cross-site GET navigation (a redirect, a form) but not a POST; Strict never cross-site; `SameSite=None` without Secure rejected; a third-party iframe, script, fetch and XHR get nothing; fetch `omit`/`same-origin`/`include`, `omit` not storing); unit tests "net: public suffixes and registrable domains" (`co.uk`, `github.io`, wildcard and exception rules), "net: SameSite, the public suffix list and the size limit", "net: cookie contexts: navigations, subresources, credentials". `wpt.py` and `conformance.py` do not exist in the tree; the whole `cargo test --release --test browser` passes instead. Not done: Chrome's Lax+POST allowance, schemeful same-site, CHIPS, a Storage Access API.

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
- status: done 2026-10-16. Already working, now covered: Enter in a field (implicit submission, no button with one text field, none with two), `requestSubmit()` validation and submitter errors, `Set-Cookie` on `fetch` and XHR reaching the jar and `document.cookie`, the agent tree masking passwords. Added: `autocomplete` reflected on form, input, select and textarea and `navigator.credentials` (`get`, `create`, `store` reject `NotSupportedError`; `PublicKeyCredential` stays absent) in `engine/sign-in.js`; the `submit` event is a `SubmitEvent` whose `submitter` is the button (`engine/dispatch-submit.aye`, `ak.submitter`); the agent's `typed` message shows `[password]`. Test server `/api/login` (JSON or form, sets cookies without a redirect); fixtures `login/details.html`, `enter.html`, `enter-two.html`, `fetch-login.html`; test `window_login_forms`. Not done: `form.elements` is missing (found while probing). WPT `html/semantics/forms/form-submission-0`: 30/362 (8.3%) before, 34/362 (9.4%) after.

#### P4.53: local telemetry
- lane: browser (Layer 0)
- goal: the window and `dump` append events to `~/.dweller/telemetry/YYYY-MM-DD.jsonl` (each file capped at 5 MB, rotating to `-1`, `-2`; the oldest deleted past 30 files). Local only, no upload; a remote sink such as Sentry is a later product decision. Off: `DWELLER_TELEMETRY=0` or a settings flag. Events: crash (signal, backtrace, the last few window actions); uncaught JS error (message, type, script origin and line); unsupported feature hit (unknown CSS property or value, missing DOM/JS API such as a ReferenceError on a global like `SVGAElement`, unimplemented method); loop iteration over 100 ms with the phase breakdown (network wait, style, layout, paint, script) from `DWELLER_TIMING` (PR #23); per-origin first paint and load time and scroll frame p50/p95; failed resource load (kind, status, origin); image decode failure. Privacy: scheme and host only, never a path, query, page text, form value, cookie or header; messages cut to 200 characters with every URL reduced to its origin. `browser/tools/telemetry.py summary [--days N]` groups events and ranks issues by frequency times sites affected, as input for new cards. The user bundle ships with it on.
- files: `browser/perf/`, `browser/engine/script.aye`, `browser/css/` (unknown property hook), `browser/engine/prelude.aye` (missing API), `browser/ui/window.aye`, `browser/tools/telemetry.py`, `docs/browser.md`
- verify: `cargo test --release --test browser telemetry > target/p4.53.log 2>&1; echo $?` → 0: a test-server page triggers a JS throw, an unknown CSS property, a missing API, a slow script, a 404 image and a corrupt image; the day file holds one record of each; a grep for `?`, a path and the page's text finds nothing; `python3 browser/tools/telemetry.py summary --days 1` lists all six
- time box: 120 min; model: opus
- status: done 2026-10-15. `perf/telemetry.aye` (no allocation; a fixed buffer written every 5 s, at exit and by the crash handler). Records: the person's window and its Spaces (`DWELLER_TELEMETRY_DIR`, set by the window) and any run with `DWELLER_PROFILE` (`dump` too); headless test windows and `dump` without a profile record nothing, so no golden changed. Off: `DWELLER_TELEMETRY=0` or `telemetry=off` in `<profile>/settings`. Events `crash`, `js_error` (line: the last statement run, via a store per statement in `exec` and a table of parsed sources in the interpreter), `unsupported` (`css_property`, `css_value`, `js_global`, `js_method`), `slow_loop` (phases summed from the `DWELLER_TIMING` ones, which are now timed while telemetry is on), `page`, `scroll`, `load_failed`, `image_decode`; format and privacy rules in `docs/browser.md` "Telemetry". Test `window_telemetry` (also SIGSEGV with the last actions, `DWELLER_TELEMETRY=0`, a test window without a profile under a temporary `HOME`).

### JS engine: the bytecode VM (P4.30)

P4.30 is the umbrella: the engine gets a bytecode compiler and an
accumulator-and-register interpreter with inline caches (Ignition-like).
The design, the baseline numbers and the reasons are in
`browser/js/BYTECODE.md`. The cards touch `browser/js/` and
`browser/tests/jsbench/` only, so they run beside the window cards. They
run in order: each one builds on the one before.

Common to these cards:
- Each card names its V8 reference (paths under `~/Documents/chromium/src/v8/`, read with the Read and Grep tools; `BYTECODE.md` section 7). Ported code carries the V8 attribution in the file header.
- Speedups are against `browser/tests/jsbench/baseline.json` (P4.59). The estimates below are estimates; the status line records the measured number (`BENCH_BASELINE=<main's binary> BENCH_RUNS=5 browser/tests/jsbench/run.sh <workloads>`).
- The VM gates (`BYTECODE.md` section 5): `js_conformance` and `js_conformance_vm` in every card, also with `DWELLER_JS_GC=stress`; from P4.67 on also `jsdom_fixtures jsdom_gc_stress` and `page_load.py check` with `DWELLER_JS_VM=1`; test262 with the VM on, no fewer passes than off, once P10.3 has landed.
- `DWELLER_JS_VM_STATS=1` must show the card's constructs gone from the fallback reasons.
- time box: 45 min each; stop rules as usual (the same verify fails twice: stop and report).

#### P4.59: JS benchmark set and baseline
- goal: the benchmark set of `BYTECODE.md` section 1 that does not need three.js or the DOM, a reference column from `node --jitless`, and the numbers of the day committed.
- design: vendor Octane `splay.js` (BSD-3) and `crypto.js` (Tom Wu's license) as the present `richards.js` is (harness removed, assertions kept, deterministic checksum, license notice in the header; README license table). Write `raytrace.js` (a small ray tracer: spheres, planes, lights, a 64×64 image, checksum of the pixels) and `loop.js`, `globals.js`, `calls.js` (the three loops of `BYTECODE.md` section 1). `run.sh`: `BENCH_REF=node` adds a `node --jitless` column; `--json FILE` writes the medians. Commit `baseline.json` (fast build of the merge base, median of 5) and the table in the README. Do not vendor Octane `deltablue.js` (GPL-2) or `raytrace.js` (no license).
- files: `browser/tests/jsbench/` (`run.sh`, `README.md`, new `.js`/`.out`, `baseline.json`)
- v8: none (Octane: github.com/chromium/octane)
- verify: `BENCH_RUNS=5 BENCH_REF=node browser/tests/jsbench/run.sh --json target/jsbench.json > target/p4.59.log 2>&1; echo $?` → 0 with every checksum equal to Node's
- estimate: no speedup (measurement)
- model: sonnet
- status: done 2026-10-15. Vendored Octane `splay.js` (BSD-3) and `crypto.js` (Tom Wu's license) with the harness removed and Octane's deterministic `Math.random`; wrote `raytrace`, `loop`, `globals`, `calls`. The runner is `browser/tools/jsbench.py` (`run.sh` builds and calls it): `node --jitless` column when node is installed, `--json FILE`, and `--smoke` (every workload once at a tenth of its size against `NAME.smoke.out`; test `jsbench_smoke`). `baseline.json` is from a 4-core Xeon 2.1 GHz Linux box, fast build, median of 5, every checksum equal to Node's; geometric mean 6.7× node --jitless (calls 17×, globals 25×, navier-stokes 9×, richards 10×, strings 1.9×). The three-loop sizes are smaller than the BYTECODE.md table (globals 3M, calls 2M iterations) and the old M2 Pro numbers there are a different machine: compare ratios, not seconds. Octane `deltablue.js` (GPL-2) and `raytrace.js` (no license) not vendored.

#### P4.60: three.js and DOM benchmarks
- goal: the `three` and `dom` workloads of `BYTECODE.md` section 1, in the baseline.
- design: vendor `three.module.min.js` 0.170.0 (the version of `~/Documents/f1/web`) with its MIT license as `browser/tests/jsbench/vendor/three/`. `three.mjs` imports it, builds a scene of 2000 `Object3D`s (groups of 20 under 100 parents), and for 300 frames sets each object's position and quaternion (`setFromAxisAngle`), calls `scene.updateMatrixWorld()`, and does `Vector3.applyMatrix4`/`Matrix4.multiplyMatrices` work; the checksum is a rounded sum of the world matrices. If `dweller js` cannot run the module build, find out why and fix the engine bug in the same card, or use `three.cjs` wrapped (note which). `dom.html`: 20k `createElement`/`appendChild`/`setAttribute`/`classList.add`, 2k `querySelectorAll`, 10k `dispatchEvent` to a listener; run through `dweller dump --js` and timed by its `scripts` phase (DWELLER_TIMING). Add both to `run.sh` and `baseline.json`.
- files: `browser/tests/jsbench/` (`three.mjs`, `vendor/three/`, `dom.html`, `run.sh`, `README.md`, `baseline.json`)
- v8: none
- verify: `BENCH_RUNS=5 BENCH_REF=node browser/tests/jsbench/run.sh three dom > target/p4.60.log 2>&1; echo $?` → 0, checksums equal to Node's (dom: to a recorded golden)
- estimate: no speedup (measurement)
- model: sonnet
- status: done (2026-10-15). Used the CommonJS build (`vendor/three/three.cjs`, unmodified r170) inlined by the runner (`// @include`), since `dweller js` has no `require`; no JS gaps hit. 100 frames / 100k math iterations (300 frames took 32 s). `dom.js` (`// @dom`) is wrapped in a page and run with `dump --text --js`; 10k nodes, 500 queries, 10 forced layouts (200 took 100 s: forced layout of 20k boxes is ~0.5 s). Baseline: three 11.9 s vs node --jitless 1.29 s (9.2×); dom 5.2 s (no reference, golden of dweller).

#### P4.61: bytecode format, builder, disassembler
- goal: the data structures of the VM, with no execution: the instruction table, `BytecodeArray`, a builder with labels and jump patching, the constant pool, the handler table, the source-position table, and a disassembler.
- design: `browser/js/bytecode.aye`: `enum Bc: u8` with the first set of `BYTECODE.md` section 2 (only what P4.63 to P4.64 use; later cards add theirs), a table of name, operand kinds, may-allocate and may-throw flags; one-byte operands with `Wide`/`ExtraWide` prefixes; `Builder` (emit, labels, forward jumps, `JumpLoop` backward) as in V8's `BytecodeArrayBuilder`; `disassemble(code) -> text` in V8's `--print-bytecode` style. Unit tests build small arrays by hand and compare the disassembly. Adds `browser/js/LICENSE-V8`.
- files: `browser/js/bytecode.aye`, `browser/js/unit_tests.aye`, `browser/js/LICENSE-V8`
- v8: `src/interpreter/bytecodes.h`, `bytecode-operands.h`, `bytecode-array-builder.h`, `bytecode-label.h`, `handler-table-builder.h`, `src/codegen/source-position-table.h`
- verify: `cargo test --release --test browser -- aye_unit_tests > target/p4.61.log 2>&1; echo $?` → 0
- estimate: no speedup
- model: opus (the format decides later cards)
- status: done 2026-10-15. `browser/js/bytecode.aye`: 73 opcodes (`Wide`/`ExtraWide`, loads, 12 binary operators with their `Smi` forms, unary and update operators, compares, 8 forward jumps with their `*Constant` twins, `JumpLoop`, `SwitchOnSmi`, `Return`) as `pub let BC_*: u8` with the table `BC_INFO` (name, operand kinds, may-allocate and may-throw flags), because Aye has neither `enum: u8` nor named constants in patterns. Operands are one byte, or two or four after a prefix (little endian, registers signed: parameters negative, `<this>` below them). `BcBuilder` emits with the narrowest width, allocates registers as a stack and feedback slots, keeps source positions (V8's varint/zigzag deltas), handler entries and `SwitchOnSmi` tables; a forward jump reserves a constant-pool entry that fixes its width (V8's pool slices 0–255, 256–65535, then the rest) and on `bind` becomes `Jump*Constant` when the offset does not fit. Constants dedup (numbers by their bits). `disassemble` prints V8's `--print-bytecode` layout without addresses; `dweller js --print-bytecode FILE` prints a hand-assembled loop until the compiler exists. 12 unit tests ("js: bytecode: ..."), `js_command_line` checks the command.

#### P4.62: resolver: registers or context slots
- goal: the resolver tells for every binding whether a nested function refers to it (captured). The tree walker ignores the new data; the compiler uses it.
- design: in `resolve.aye`, mark a binding captured when an identifier resolves to it across a function boundary (V8: `Variable::IsContextSlot`); per scope, the count of captured bindings and a map from slot to register or context slot. `arguments`, the function's own name and `this` captured by arrows follow the same rule. Unit tests on small sources print the classification.
- files: `browser/js/resolve.aye`, `browser/js/ast.aye` (`ScopeInfo` fields), `browser/js/unit_tests.aye`
- v8: `src/ast/scopes.h` (`Scope::AllocateVariablesRecursively`, `MustAllocateInContext`), `src/ast/variables.h`
- verify: `cargo test --release --test browser -- aye_unit_tests js_conformance > target/p4.62.log 2>&1; echo $?` → 0
- estimate: no speedup (no behavior change)
- model: sonnet
- status: done 2026-10-15. `resolve.aye` `allocate` writes, per scope, a run of `Ast.vars` (`VarInfo`: register `r<i>`, parameter `a<i>`/`this`, context slot `c<i>` or module cell, and why) and `ScopeInfo` fields (kind, parent, registers with r0 reserved, context slots, per-iteration, eval, mapped arguments, parameter scope), after V8's `AllocateVariablesRecursively`/`MustAllocateInContext`/`AllocateParameterLocals`: captured across a function boundary or under a direct `eval` → context; block-scope registers come from the enclosing function; a sloppy simple-parameter function using `arguments` puts its parameters in the context (mapped); `for` scopes with context slots copy per iteration; module imports and exported locals are module cells; catch bindings are registers unless captured (V8 always contexts them). Arrows capture the enclosing `this`/`arguments`. `dweller js --print-scopes FILE` prints the tree; test `js_print_scopes` with 10 goldens in `browser/tests/js/scopes/` (closures, nested functions, loops with captured let, catch, classes, defaults with closures, `arguments`, arrows, eval, a module). Tree walker unchanged (`js_conformance`, `jsdom_*` green).

#### P4.63: VM core: frames, dispatch, straight-line code
- goal: the dispatch loop runs compiled functions behind `DWELLER_JS_VM=1`: literals, registers (parameters and uncaptured locals), all unary, binary, comparison and update operators, assignments to registers, expression statements and `return`. Anything else leaves the function on the tree walker.
- design: `browser/js/vm.aye` (the loop: a `match` on the opcode, the accumulator in a local) and `browser/js/compiler.aye` (AST to bytecode, V8's `BytecodeGenerator` shape: visit an expression into the accumulator, a register allocator for temporaries). Lazy compilation from `invoke`, with the result or the reason kept per `FuncInfo`. The frame of `BYTECODE.md` section 2 (above `[fn, this, args]`, r0 the accumulator's home, new `Frame` fields); the may-allocate rule of section 4. Operators call the existing `Interp` helpers (`add`, `binary`, `lessThan`, `looseEquals`, ...) after a numeric fast path. `DWELLER_JS_VM_STATS=1`. New test `js_conformance_vm` in `tests/browser.rs`: the js scripts with `DWELLER_JS_VM=1` against the same goldens, and a second pass with `DWELLER_JS_GC=stress` too. Check that the dispatch `match` still compiles to one jump table (`aye emit-c` plus `clang -S`, note in `BYTECODE.md`).
- files: `browser/js/vm.aye`, `browser/js/compiler.aye`, `browser/js/bytecode.aye`, `browser/js/interp.aye` (`invoke`, `Frame`), `tests/browser.rs`
- v8: `src/interpreter/bytecode-generator.cc` (`VisitBinaryOperation`, `VisitAssignment`, `VisitCountOperation`), `bytecode-register-allocator.h`, `interpreter-generator.cc` (handler semantics)
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm aye_unit_tests > target/p4.63.log 2>&1; echo $?` → 0
- estimate: straight-line arithmetic functions 2–3×; suite unchanged (loops not compiled yet)
- model: opus
- status: done 2026-10-15. `compiler.aye` (V8 `BytecodeGenerator` shape: expressions into the accumulator, a local's own register as the left operand when the right side cannot write it, `Smi` forms for small literals, temporaries from the builder's register stack) and `vm.aye` (one `match` arm per opcode, the accumulator a local, registers a slice of `Interp.stack` reloaded after anything that may call; before an allocating or converting slow path the accumulator goes to r0). Compiled on the first call from `invoke`, after the frameless one-expression path; the result or the reason is kept per function; parameters are used in place above `[fn, this]` (padded or copied up when the count differs); `this` gets a register. Compiles literals, registers, all unary (but `delete`), binary, comparison and update operators, assignments and compound assignments to registers, `var`/`let`/`const`, blocks without functions, expression statements, `return`; a `let` read before its declaration falls back. New instruction `ToNumber` (unary `+`). `DWELLER_JS_VM_STATS=1` prints compiled/fallback counts, calls, instructions and the reasons. Test `js_conformance_vm`: the js scripts with the VM, then with `DWELLER_JS_GC=stress` too (leaving out 12 allocation and recursion scripts, minutes each under stress); new script `259_vm_straight_line.js`. Two jump tables (2–43, 44–72; BYTECODE.md). Straight-line arithmetic function bodies 2–2.5× (about 6 ns per instruction); jsbench unchanged within noise, checksums equal (PERF.md); suite: 194 functions compiled, 3355 on the tree walker. The `Frame` fields of the design come with calls (P4.65).

#### P4.64: control flow
- goal: `if`, `while`, `do`, `for` (with `var` or uncaptured `let`), `break`/`continue` with and without labels, `switch`, `?:`, `&&`, `||`, `??`, the comma operator, `void`.
- design: V8's `LoopBuilder`, `SwitchBuilder`, `BreakableControlFlowBuilder`; `JumpIfToBooleanFalse` and friends; `JumpLoop` does the budget tick (`Interp.tick`) and the interrupt check; `SwitchOnSmi` for dense integer `case`s.
- files: `browser/js/compiler.aye`, `browser/js/vm.aye`, `browser/js/bytecode.aye`
- v8: `src/interpreter/control-flow-builders.h`, `bytecode-generator.cc` (`VisitIfStatement`, `VisitForStatement`, `VisitSwitchStatement`, `VisitLogicalOrExpression`)
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm > target/p4.64.log 2>&1; echo $?` → 0; `DWELLER_JS_VM=1 browser/tests/jsbench/run.sh loop` checksum equal
- estimate: `loop` 3–5×; suite 1.0–1.2×
- model: sonnet
- status: done 2026-10-16. `compiler.aye` compiles `if`, `while`, `do`, `for` (a `let` or `var` init; `for` scopes with context slots still fall back with their variables), `break`/`continue` with and without labels (a control-scope list as V8's `ControlScope`; `a: b: loop` shares one target), `switch`, `?:`, `&&`, `||`, `??`, the comma operator and `void`; after V8's `LoopBuilder` shape: header, test, body, continue target, one `JumpLoop`, break label (a `do` loop is the body, then the test jumping out, then `JumpLoop`). A condition that is always a boolean (comparison, `!`, literal) uses `JumpIfFalse`, else `JumpIfToBooleanFalse`. `switch` saves the discriminant and runs `TestEqualStrict` per case in order; three or more distinct integer literal cases within a dense range (at most twice their count plus 8) use `SwitchOnSmi` with the gaps bound to the default; a `switch` with a `let`/`const`/function directly in a case (or context slots) falls back. `vm.aye` has the jump arms (immediate and `Jump*Constant` forms, wide operands) and `JumpLoop`, which does `Interp.tick` (the budget and termination check). Test: new script `260_vm_control_flow.js` (all of the above, wide jumps, `SwitchOnSmi` gaps, a loop of 3 million iterations); `js_conformance` and `js_conformance_vm` green, the new script also equal under `DWELLER_JS_GC=stress`. `jsbench loop`: 1.98 s tree walker to 0.59 s with `DWELLER_JS_VM=1` (3.4×), checksum 35000000 equal. Found: the tree walker returns `undefined` from a loop with two labels (`a: b: for (...) { continue a }`), the VM gets it right; the test avoids it. Left: `for-in`/`for-of`, `try`, logical assignment (`||=` ...), loops with captured `let` (P4.66), the suite speedup (nearly every function still has a call or a property access).

#### P4.65: calls and `new`
- goal: calls and constructions from compiled code: `f(...)`, `o.m(...)` (with the property load still generic until P4.67), `new F(...)` for functions and base classes, natives and bound functions, `arguments`, default and rest parameters, missing and extra arguments, sloppy and strict `this`.
- design: `CallProperty`, `CallUndefinedReceiver`, `Construct`. A call to a compiled function pushes a frame and continues the loop (no C recursion); others go through `callAt`/`constructAt`. `Return` pops; an `entry` frame leaves the loop. `MAX_DEPTH` counts frames; the C stack check stays for entries from natives. `CreateArguments`, `CreateRestParameter`.
- files: `browser/js/vm.aye`, `browser/js/compiler.aye`, `browser/js/interp.aye` (`callAt`, `constructAt`, `invoke`)
- v8: `bytecode-generator.cc` (`VisitCall`, `VisitCallNew`, `BuildCreateArguments`), `interpreter-generator.cc` (`CallProperty`, `Construct`), `src/builtins/builtins-call-gen.cc` (receiver conversion)
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm js_command_line > target/p4.65.log 2>&1; echo $?` → 0; `calls`, `richards` checksums equal
- estimate: `calls` 1.5–2× (property access still generic)
- model: opus
- status: done 2026-10-16. `compiler.aye` compiles calls (`f(...)`, `o.m(...)`) and `new F(...)` without spread or optional chaining (V8 `VisitCall`/`VisitCallNew`: callee, `this` and arguments in consecutive registers; `CallProperty`, `CallUndefinedReceiver`, `Construct` with the `Call`/`New` node as operand for the error text), and the generic loads and stores they need until P4.67: global reads (`LdaGlobal`), `o.x` reads (`GetNamedProperty`) and `o.x = v` (`SetNamedProperty`), on the tree walker's per-node property caches. `vm.aye`: a call or construction of a compiled plain function pushes its frame and stays in the dispatch loop (`vmPushFrame`, the caller in `VmState.saved`, `Return` pops; no C recursion); natives, bound functions, proxies, class constructors, generators and functions on the tree walker go through `callAt`/`constructAt`. `MAX_DEPTH` counts frames; a throw pops the frames the loop pushed. `arguments` (unmapped: strict or non-simple parameters; the mapped sloppy one is in the context, P4.66), rest and default parameters (`CreateArguments`, `CreateRestParameter`; non-simple parameters are registers copied in order at entry, each in its dead zone until then, so `function f(a = b, b)` falls back), missing and extra arguments, sloppy and strict `this`. New script `261_vm_calls.js`, also equal under `DWELLER_JS_GC=stress`; `js_conformance`, `js_conformance_vm`, `js_command_line` green. jsbench (fast build, median of 3, tree walker vs `DWELLER_JS_VM=1`): `richards` 1.70 s to 1.21 s (1.4×), `calls` 3.99 s to 4.14 s (1.0×: its loop is at the top level, on the tree walker until P4.67; the same loop inside a function 2.73 s to 2.33 s, 1.17×), checksums equal (2916400, 66000000). Suite: 478 functions compiled, 3014 on the tree walker (P4.63: 194). Also fixed: the tree walker's loop with two labels (`a: b: for (...) { continue a }`), now in `260_vm_control_flow.js`. Left: spread arguments, optional calls, `super` calls and class constructors, compound assignment to properties, keyed access (P4.67).

#### P4.66: closures and contexts
- goal: nested functions and captured variables: `CreateClosure`, the function context for captured bindings, block contexts, per-iteration copies of captured `let` loop variables, TDZ checks, named function expressions.
- design: `BYTECODE.md` section 2 "Scopes, closures and `let`": context `Env` made at entry only when the function has captured bindings; `LdaContextSlot depth, slot`, `PushContext`/`PopContext`, `CopyContext` once per iteration, `ThrowReferenceErrorIfHole`. Closures created by the VM are the same function objects as the tree walker's (`makeClosure`).
- files: `browser/js/compiler.aye`, `browser/js/vm.aye`, `browser/js/interp.aye` (`makeClosure`)
- v8: `bytecode-generator.cc` (`BuildVariableLoad`, `BuildVariableAssignment`, `VisitBlock`, `BuildNewLocalBlockContext`, the per-iteration copy in `VisitIterationBody`), `src/ast/scopes.h`
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm > target/p4.66.log 2>&1; echo $?` → 0; `closures` checksum equal
- estimate: `closures` 2–3×
- model: opus
- status: done 2026-10-16. Compiled code makes nested functions (`CreateClosure`, the tree walker's `makeClosure`, so a closure from either engine runs on either) and reads and writes captured bindings. The VM's current context is `Interp.env` itself, laid out as the tree walker's `Env` (a binding's tree-walker slot, the uncaptured ones left unused), so depths and slots agree between engines: a function makes its context at entry (`CreateFunctionContext`, filled with undefined) only when it creates closures or has context slots (a sloppy function with `arguments` puts its parameters there, now compiled), copying in a captured `this` and simple parameters; a block or `for` scope with slots that creates closures pushes one (`PushContext r, n`: the outer context saved in register r, `PopContext r`, also emitted by `break`/`continue` leaving it); a `for (let ...)` loop copies its context once per iteration (`CopyContext`, where the tree walker's `execFor` copies). Scopes without closures keep their bindings in registers and make no context, so the compiler counts only the scopes with a context in `LdaContextSlot depth, slot`/`StaContextSlot`; reads beyond the function walk the closure's chain (`vmPushFrame` sets `Interp.env` to the closure's environment, `vmPopFrame` restores the caller's). TDZ: block and outer context reads and assignments check `ThrowReferenceErrorIfHole name` (function-level contexts start undefined, as the tree walker's); register `let`s keep the compile-time dead-zone check. Function declarations are hoisted at function and block entry; a named function expression binds itself (`LdaClosure`). Module bindings read through `readImport`. Files: `vm-context.aye`, `vm-ident.aye`, `vm-context-depth.aye`, `vm-may-be-hole.aye`, `vm-load-context.aye`, `vm-assign-context.aye`, `vm-update-context.aye`, `vm-enter-scope.aye`, `vm-exit-scope.aye`, `vm-hoist.aye`, `vm-function-context.aye` (compiler), `vm-env-at.aye`, `vm-push-context.aye`, `vm-new-iteration-context.aye` (VM). New script `262_vm_closures.js` (counters, per-iteration `let` with `break`/`continue`, labeled breaks out of block contexts, hoisting, named function expressions, TDZ reads and writes, arrows capturing `this` and `arguments`, mapped-arguments functions, `const` writes), also equal under `DWELLER_JS_GC=stress`; `js_conformance`, `js_conformance_vm` green; `aye_unit_tests` (opcode count) green. jsbench (fast build, median of 3): `closures` 1.96 s tree walker to 1.90 s with `DWELLER_JS_VM=1` (1.03×, checksum 306584040 equal): `factory` and the returned closure still fall back on the object literal and `state.x += ...` (P4.67), so only the driver loop runs compiled; `richards`, `calls`, `loop`, `raytrace`, `splay` checksums equal. Suite: 1127 functions compiled, 2412 on the tree walker (P4.65: 478, 3014). Left: captured non-simple parameters (fall back), contexts in `switch`, `--print-bytecode` of compiled functions.

#### P4.67: properties, literals, globals, the top level
- goal: named and keyed get/set, object and array literals, `delete`, `in`, `instanceof`, `typeof`, global reads and writes, top-level `let`/`const` (`glex`), and the script's top level and `evalScript` code compiled as a function body.
- design: `GetNamedProperty r, name, [s]` and friends; each `[s]` gets the present monomorphic `PropCache` (one per feedback slot instead of per AST node); `LdaGlobal`/`StaGlobal` cache `(global shape, slot)`; keyed loads of dense arrays and strings skip the generic path; `CreateObjectLiteral` uses `literalShape`; `CreateArrayLiteral`. From this card on, the DOM gates run with the VM on.
- files: `browser/js/compiler.aye`, `browser/js/vm.aye`, `browser/js/shapes.aye`, `browser/js/interp.aye` (`runScript`, `evalScript`)
- v8: `bytecode-generator.cc` (`VisitProperty`, `VisitObjectLiteral`, `VisitArrayLiteral`, `BuildGlobalLoad`), `src/ic/ic.cc` (`LoadIC::Load`, `StoreIC::Store`)
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures jsdom_gc_stress > target/p4.67.log 2>&1; echo $?` → 0; `DWELLER_JS_VM=1 python3 browser/tools/page_load.py check` exit 0 (local)
- estimate: `globals` 5–10×, `calls` 3–5×; richards, navier-stokes, methods 1.8–2.5×
- model: opus
- status: done 2026-10-16. Compiled code reads and writes properties and globals and builds literals; the script top level is left for P4.67b. New instructions (`bytecode.aye`, 91–101): `StaGlobal [n]` (a global `var` store hits the `(global shape, slot)` cache of its `Ident` node, `vm-global-cache-hit.aye`; `let`/`const` at the top level and misses go through `assignIdent`, so `const` writes and strict undeclared writes throw as on the tree walker), `TypeOfGlobal [n]` (no ReferenceError for an undeclared name), `GetKeyedProperty r, [s]` and `SetKeyedProperty r, r, [s]` (`getIndex`/`setIndex`, whose dense-array, typed-array and string paths come first), `DeleteProperty r` (the key in the accumulator), `TestIn r, [s]`, `TestInstanceOf r, [s]`, `CreateObjectLiteral [n]` (a literal with a `literalShape` gets its final shape and slots at once, with undefined values the defines then fill), `DefineNamedOwnProperty r, [p], k` (data, method, getter, setter and `__proto__` properties; methods get their home object), `CreateArrayLiteral` and `StaInArrayLiteral r` (holes kept). The compiler (`vm-target-kind.aye` picks a local, global or property target) compiles `o[k]` loads, calls `o[k](...)`, assignments, compound assignments and `++`/`--` to `o.x`, `o[k]` and globals (the object and key registers copied when the right side may write them, `vm-protected-reg.aye`), `delete`, `in`, `instanceof`, `typeof` of globals, and object and array literals without computed keys or spread. Named loads and stores still use the per-node `PropCache` (a site is one node, so they are per site already; feedback-vector slots are P4.67b). From this card on `jsdom_fixtures` and `jsdom_gc_stress` run with `DWELLER_JS_VM=1`: `virtual_clock.tree` counts 998000 instead of 997000 (the VM ticks the step budget per call and loop back edge, the tree walker per statement, and the page's prelude functions now compile). New script `263_vm_properties.js` (keyed and named compound updates, literals with methods, accessors, `__proto__` and duplicate keys, `in`/`instanceof`/`delete`/`typeof`, global and top-level `let` writes, setters through compound assignment, a hot loop of literals), also equal under `DWELLER_JS_GC=stress`; `js_conformance`, `js_conformance_vm`, `jsdom_fixtures`, `jsdom_gc_stress`, `aye_unit_tests` (opcode count) green. `page_load.py check` not run (needs the local site archives). jsbench (fast build, one run each, tree walker vs `DWELLER_JS_VM=1`, checksums equal): `navier-stokes` 13.66 s to 2.92 s (4.7×), `richards` 1.54 s to 0.91 s (1.7×), `methods` 0.358 s to 0.244 s (1.5×), `closures` 2.57 s to 1.83 s (1.4×), `globals` 2.34 s to 2.25 s and `calls` 3.63 s to 3.67 s (1.0×: their loops are at the top level, P4.67b); geomean ratio to `node --jitless` over the six 10.9× to 7.2×. Left (P4.67b): the top level and `evalScript`, per-feedback-slot caches, computed keys and spread in literals.

#### P4.67b: the top level, `evalScript` and per-slot caches
- goal: the script's top level and `evalScript` code run compiled, so top-level loops (`globals`, `calls`) leave the tree walker; named and keyed sites get their `PropCache` from the function's feedback vector instead of the AST node.
- design: `compileScript(ast, root)` beside `compileFunction`: the Program node as a function body with no parameters, `this` the global object, top-level `var` and functions as global object properties (`LdaGlobal`/`StaGlobal`, P4.67), top-level `let`/`const`/`class` as `glex` slots (a declaration stores with `init`, so a `const` may be initialized: a new `StaGlobalInit [n]` or a flag), function declarations hoisted as `exec(Program)` does now. `runScript` enters the VM with a frame whose function is the script (no function object: `LdaClosure` never emitted, `isStrictCode` from `script_strict`); `evalScript` keeps the completion value (the last expression statement's value stored in a register the compiler reserves, returned at the end) instead of `Interp.completion`. A script the compiler cannot handle (try, for-in, classes until P4.68–P4.70) stays on the tree walker. Per-slot caches: `BytecodeArray` gets `nslots` `PropCache`s; `GetNamedProperty r, name, [s]` reads the name from the constant pool and the cache from the slot; LdaGlobal/StaGlobal likewise. Object literals with computed keys (`DefineKeyedOwnProperty`) and spread (`CallRuntime` copyDataProps) and array spread.
- files: `browser/js/compiler.aye` (wiring), new `vm-*.aye` files, `browser/js/vm.aye` (arms), `browser/js/interp.aye` (`runScript`, `evalScript`: wiring only)
- v8: `bytecode-generator.cc` (`GenerateBytecodeBody` for scripts, `VisitDeclarations`, `BuildReturn` for eval completion), `src/objects/feedback-vector.h`
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures jsdom_gc_stress > target/p4.67b.log 2>&1; echo $?` → 0; `globals`, `calls`, `richards` checksums equal with `DWELLER_JS_VM=1`
- estimate: `globals` 5–10×, `calls` 3–5×
- model: opus
- status: partly done 2026-10-16. The script top level and `evalScript` code run compiled (`vm-compile-script.aye`, `vm-visit-script.aye`, `vm-run-script.aye`; `runScript` wiring only): the Program block as a function body with no parameters and no function object (frame `func` NONE, so `isStrictCode` reads `script_strict`), its code freed when the script ends; top-level functions hoisted and `let`/`const` initialized with the new `StaGlobalInit [n]` (102, `assignIdent` with `init`, `vm-init-binding.aye`, `vm-global-decl.aye`), `var` stores `StaGlobal`; each expression statement stores into a reserved completion register that the script returns, and `vmRunScript` sets `Interp.completion` at the tree walker's depth rule, so `evalScript` is unchanged. Scripts the compiler cannot handle stay on the tree walker (`DWELLER_JS_VM_STATS=1` prints why). Global `var` loads and stores (`vm-lda-global.aye`, `vm-sta-global.aye`, `vm-global-slot.aye`) check the property at the cached slot by key, which also holds in dictionary mode: the `js` shell's global object is a dictionary, so the shape-keyed cache of P4.67 never hit and every global access took `findProp` plus `getFrom`. `vmIdx` decodes 2- and 4-byte operands inline and `vmImm` 2-byte ones (AST node operands are wide; `bcGet` was 11% of `calls`). `virtual_clock.tree` counts 999000 (its top-level busy loop now ticks per back edge). `js_conformance`, `js_conformance_vm`, `jsdom_fixtures`, `jsdom_gc_stress`, `errors` green; `check_layers.py` passes; `aye_unit_tests` not run (`BC_INFO` and `BC_COUNT` both grew by one). jsbench (fast build, median of 3, tree walker vs `DWELLER_JS_VM=1`, checksums equal): `globals` 2.13 s to 0.55 s (3.9×), `calls` 3.31 s to 1.86 s (1.8×), `richards` 1.38 s to 0.69 s (2.0×). Below the estimate: `calls` spends its time in the constructor's `this.x = x` adds (no store-transition cache: `setOn` walks the chain, `findProp`, `addProp`, the transition map) and in pushes of the call frames. Left (P4.67c): per-feedback-slot `PropCache` (`GetNamedProperty r, name, [s]` with the name in the constant pool, so operands stop being wide node ids), a store-transition cache for adds, computed keys and spread in literals, array spread.

#### P4.68: exceptions
- goal: `throw`, `try`/`catch`/`finally` in compiled code, with unwinding through VM frames, natives and the tree walker, and error traces and telemetry positions from the source-position table.
- design: `BYTECODE.md` section 2 "Exceptions and `finally`": handler table, the unwind step, the token-and-value `finally` of V8's `ControlScopeForTryFinally`; `Terminated` and `OutOfMemory` skip handlers; `gen_return` keeps its rule. `appendTrace`, `DWELLER_JS_TRACE` and `last_stmt` read positions from the table.
- files: `browser/js/compiler.aye`, `browser/js/vm.aye`, `browser/js/interp.aye` (`appendTrace`)
- v8: `bytecode-generator.cc` (`VisitTryCatchStatement`, `VisitTryFinallyStatement`, `ControlScopeForTryFinally`), `control-flow-builders.h` (`TryCatchBuilder`, `TryFinallyBuilder`), `handler-table-builder.h`
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures > target/p4.68.log 2>&1; echo $?` → 0
- estimate: coverage (functions with `try` compile); no suite change
- model: opus
- status: done 2026-10-16. Compiled code throws and catches: `throw` (`Throw [n]`, 103: the exception from the accumulator, `last_stmt` set to the Throw node, so an uncaught throw still names its line), `try`/`catch` (V8 `VisitTryCatchStatement`: `SaveContext r` (104) keeps the context at the try in a register, a handler-table entry covers the try range, the handler stores the exception and binds the catch parameter, in a block context when captured; a destructuring parameter falls back) and `try`/`finally` (V8 `ControlScopeForTryFinally`: a token and a value register, one copy of the finally block, then a dispatch on the token: rethrow, `return` (also through outer finally blocks), and each `break`/`continue` that leaves the try, re-resolved outward; a script keeps its completion value across the finally block). Unwinding (`vm-unwind.aye`): `vmLoop` records its frame (code, pc, fp, arguments) in an `errdefer` (no cost on the normal path), `vmRun` looks the faulting pc up in the frame's handler table, restores the context, truncates the stack to the frame's registers and resumes at the handler; without one it pops the frame the loop pushed and looks again in the caller, and at the entry frame the error leaves the loop as before. `Terminated`, `OutOfMemory` and a generator's `gen_return` skip handlers. The prelude's typed-array helpers `liveBuffer` and `TypedArray.prototype.set` (P11.24f: most of the game's tree-walker calls) now compile with no change to their source. `DWELLER_JS_VM_STATS=1` also prints the calls that ran on the tree walker. A typed-array script (`fill`, `set`, `subarray`, `slice`, views, 2000 iterations): tree-walker calls 34033 of 72201 (47%) before, 14031 of 66201 (21%) after; the rest are the typed-array class constructors (P4.70). New script `264_vm_exceptions.js` (catch and finally through calls, natives, deep recursion and stack overflow, `break`/`continue`/`return` through nested finally blocks, catch with closures, `switch`, the top level, a hot loop of throws), also equal under `DWELLER_JS_GC=stress`. `js_conformance`, `js_conformance_vm`, `jsdom_fixtures`, `jsdom_gc_stress`, `errors`, `aye_unit_tests` (opcode count, also stale since P4.67b) green; `check_layers.py` and `check_taste.py` pass (also fixed the violations that came with main and P4.67b: `coarse-ms.aye`, `window.aye`, `vm-lda-global.aye`, `vm-visit-script.aye`, one long function moved out of `vm.aye` into `vm-idx.aye`). jsbench with `DWELLER_JS_VM=1`: all 15 checksums equal; `richards`, `calls`, `navier-stokes` unchanged within noise against the base. Left: `appendTrace`, `DWELLER_JS_TRACE` and `last_stmt` from the source-position table for errors other than `throw` (natives, property access) in compiled code; destructuring catch parameters (P4.69).

#### P4.69: the rest of the syntax
- goal: destructuring (declarations, assignments, parameters), spread in calls, arrays and objects, `for-in`, `for-of` with iterator close, template literals and tagged templates, optional chaining, `**`, `new.target`, regexp literals, getters and setters in object literals.
- design: `GetIterator`, `ForInPrepare`/`ForInNext`/`ForInStep`, `CallWithSpread`; optional chaining as `JumpIfUndefinedOrNull` to the end of the chain (no `ShortCircuit`); slow pieces through `CallRuntime` into the existing `Interp` helpers (`bindPattern`'s logic, `evalTagged`, `copyDataProps`).
- files: `browser/js/compiler.aye`, `browser/js/vm.aye`, `browser/js/bytecode.aye`
- v8: `bytecode-generator.cc` (`BuildDestructuringArrayAssignment`, `BuildDestructuringObjectAssignment`, `VisitForOfStatement`, `VisitForInStatement`, `BuildGetIteratorRecord`, `BuildIteratorClose`, `VisitOptionalChain`)
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures > target/p4.69.log 2>&1; echo $?` → 0; test262 `test/language/statements/for-of test/language/expressions/assignment` with the VM on, no fewer passes than off (once P10.3 has landed)
- estimate: coverage; site `scripts` phase starts to drop (1.2–1.5×)
- model: opus
- status: done 2026-10-16. The VM compiles destructuring in declarations, assignments and parameters (`vmBindPattern`: array patterns over `GetIterator`/`IteratorStep`/`IteratorRest`/`IteratorClose` on a three-register record (iterator, `next`, done; a native iterable is walked by index like the tree walker), object patterns by `GetKeyedProperty` with a RequireObjectCoercible check and `ObjectRest` for `...rest`, defaults by `JumpIfUndefined`, leaves through `vmStoreTarget` to registers, contexts, globals and properties); spread in calls and `new` (`CallWithSpread` over an arguments array) and in array literals (`SpreadIntoArray`); object literals with spread (`CopyDataProperties`), computed keys and computed accessors (`DefineKeyedOwnProperty`); `for-of` (V8 `VisitForOfStatement`: the loop inside a try/finally on the P4.68 machinery, so `break`, labeled jumps, `return` and a throw close the iterator, a throwing `next()` does not) and `for-in` (`ForInPrepare` collects the keys like `execForIn`, `ForInNext` skips deleted ones; V8's `ForInStep` is folded into `ForInNext`), both with a fresh context per iteration for captured `let`/`const`; template literals (`TemplateString`), tagged templates (`GetTemplateObject`, the per-site object shared with `evalTagged` through `templateObject`, then an ordinary call); optional chaining (each `?.` link a `JumpIfUndefinedOrNull` to one nullish label per chain); regexp literals (`CreateRegExpLiteral`); `new.target` (`LdaNewTarget`); `**` already compiled. 15 opcodes (105–119). New script `265_vm_syntax.js`. test262 at d53ebc8b19 (-j 4), VM off → on: `statements/for-of` 583 → 604/751, `expressions/assignment` 324 → 332/485, `expressions/template-literal` 52 → 52/57, `expressions/optional-chaining` 36 → 36/38; total 995 → 1024/1331, no test that passes off fails on. `js_conformance`, `js_conformance_vm`, `jsdom_fixtures`, `aye_unit_tests` (opcode count), `errors` green; `check_layers.py` and `check_taste.py` pass. Left: rest parameters with a pattern (`...[a, b]`), the Annex B `for (var x = init in o)`, `for await` (P4.72), closing the iterator when a default or nested store throws inside an array pattern, ToPropertyKey of a computed key before its value is evaluated, the tree walker's message for destructuring null.
- perf gate 2026-10-16: macOS `perf` feed `style_full` 4 > 2 is not this card's: main c5747e9 fails it too (3), the gate runs with the VM off and the branch changes only `browser/js/`; on a loaded CPU (`taskset -c 0` with busy loops) main and this branch both count 2/8/8/8, each ticker update restyling in full twice (the script's geometry and the window's `styleFor`); unloaded both count 2. No fix here, budget unchanged.

#### P4.70: classes
- goal: class declarations and expressions: methods, accessors, static members, fields and static blocks, private names, computed keys, `super.x` and `super(...)`, derived constructors (`this_obj`, `ctor_this`), default constructors.
- design: one `CallRuntime` for the class definition (reusing `evalClass`'s object building), then compiled bodies for methods, field initializers and constructors; derived constructor `this` in a register with the TDZ check after `super()`.
- files: `browser/js/compiler.aye`, `browser/js/vm.aye`, `browser/js/interp.aye` (`evalClass`, `superConstruct`, `initFields`)
- v8: `bytecode-generator.cc` (`BuildClassLiteral`, `VisitSuperCallReference`, `BuildPrivateBrandCheck`, `BuildInstanceMemberInitialization`)
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures > target/p4.70.log 2>&1; echo $?` → 0; `DWELLER_JS_VM=1 python3 browser/tools/page_load.py check` exit 0; test262 `test/language/statements/class` VM on ≥ off
- estimate: `three` 1.5–2× (its classes compile); site `scripts` 1.3–2×
- model: opus
- status: done 2026-10-16. The VM compiles class declarations and expressions (V8 `BuildClassLiteral`): the heritage and the computed keys are visited into consecutive registers, then `CreateClass [c], r, named` (120) runs `defineClass` (`define-class.aye`, `define-class-element.aye`), the object building that `evalClass` now shares: the tree walker evaluates the heritage and every computed key first and pushes them, so the keys are all evaluated before any static field runs, as in the spec. A named class expression's scope is a block scope (`vmEnterScope`); its name is written into the class context before static elements run, or stored to its register after. Methods, accessors, field initializers and static blocks are ordinary compiled functions; private names stay `#x` properties, so `this.#x`, `#m()` and `#x in o` need nothing new. Class constructors compile (base, derived and default ones) and keep running through `constructAt`/`superConstruct` (fields, `this_obj`, `ctor_this` unchanged). `super(...)` is `SuperCall r, argc, spread` (121, V8 `VisitCallSuper`): arguments from registers or, with spread, from an array (a default derived constructor forwards its rest array); it updates the constructor frame's `this_obj` and the result is stored into `this` with an init store. In a derived constructor every read of `this` (register or context) is followed by `ThrowReferenceErrorIfHole`. `super.x`, `super[k]` and `super.m(...)` are `GetSuperProperty r, [n]` (122) over `superGet`. `currentFunction` skips the VM's script frames (no function object), so an arrow made by indirect eval in a field initializer finds its home object as on the tree walker. Falls back: a computed instance-field key (the tree walker evaluates it per construction, in the constructor's environment), `super` in a script. New script `266_vm_classes.js` (Node golden; equal on both engines). test262 at d53ebc8b19 (-j 4), VM off → on: `statements/class` 2504 → 2544/4366, `expressions/class` 2346 → 2386/4048; total 4850 → 4930/8414, no test that passes off fails on. No class reason left in `DWELLER_JS_VM_STATS` over the js suite. `js_conformance`, `js_conformance_vm`, `jsdom_fixtures`, `aye_unit_tests` (opcode count), `errors` green; `check_layers.py` and `check_taste.py` pass. `DWELLER_JS_VM=1 page_load.py check` (archives captured today; github, google, zol and mistral did not capture): bbc fails on both engines; react fails only with the VM on, on an archive miss of a Google Analytics beacon whose `tfd` (elapsed ms) is 0 instead of the recorded 1, timing not behaviour. Left: computed instance-field keys evaluated once at definition (both engines), `super()` called twice should throw, `super.x = v`, the tree walker's `super[k](...)` passes `undefined` as `this`.

#### P4.71: generators on resumable frames
- goal: `function*` runs in the VM without a coroutine: `yield`, `yield*`, `return(v)` and `throw(e)` with `finally`.
- design: `BYTECODE.md` section 2 "Generators and async functions": `SuspendGenerator` copies the registers and context into the generator object (a new `ObjData` case traced by the collector), `ResumeGenerator` and `SwitchOnGeneratorState` jump back; the generator object and its `next`/`throw`/`return` natives stay the same for both engines.
- files: `browser/js/compiler.aye`, `browser/js/vm.aye`, `browser/js/coroutine.aye`, `browser/js/heap.aye`, `browser/js/gc.aye`
- v8: `bytecode-generator.cc` (`VisitYield`, `VisitYieldStar`, `BuildSuspendPoint`, `BuildGeneratorPrologue`), `src/objects/js-generator.h`, `src/runtime/runtime-generator.cc`
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm > target/p4.71.log 2>&1; echo $?` → 0 (also with `DWELLER_JS_GC=stress`); test262 `test/language/expressions/yield test/built-ins/GeneratorPrototype` VM on ≥ off
- estimate: generator-heavy code 2–5×; no 256 MB stack per generator
- model: opus
- status: done 2026-10-16. A sync `function*` (declarations, expressions, object and class methods) compiles and runs in the VM without a coroutine (V8 `BuildGeneratorPrologue`, `BuildSuspendPoint`, `VisitYield`, `VisitYieldStar`). Five opcodes (123–127): `SwitchOnGeneratorState r_gen` at pc 0 (on a resumed frame `r_gen` holds the generator, and the loop jumps to the resume pc stored in it; V8 keeps a suspend id and a jump table instead), `CreateGeneratorObject` (after the context, parameters and hoisting, so defaults and `arguments` are evaluated at the call as in the spec; prototype from the function's `prototype`), `SuspendGenerator r_gen` (copies the parameters and registers into the generator's `elems`, saves the context and the resume pc, and returns the accumulator from the entry frame), `ResumeGenerator r_gen, r` (the sent value into `r`, the resume mode into the accumulator, then `SwitchOnSmi` to a `Throw` or to the ordinary `return` path, so `return(v)` runs `finally` blocks through the P4.68 machinery and a `yield` inside `finally` works) and `YieldStarStep r` (one step of `yield*` over a four-register record: iterator, `next`, mode, value; `throw`/`return` forwarded, a missing `throw` calls the iterator's `return` and throws a TypeError). The call itself suspends at an initial suspend point and returns the generator object. The generator object is `Class.Generator` with a new `ObjData.GenFrame` case (`VmGenFrame`: function, closure, context, resume pc, state, mode, input), whose context and input the collector traces (the registers are elements, already traced). `next`/`throw`/`return` stay the same natives for both engines: `generatorResume` hands a `GenFrame` generator to `vmGeneratorResume`, which pushes a frame, copies the registers back and runs it in a dispatch loop of its own; the iterator results and the done-state answers are the coroutine path's. `callCoroutine` calls a generator the VM compiles through `invoke`; async functions and async generators stay on `coroutine.aye` (P4.72). No C stack per generator, no 4000 limit (`267_vm_generators.js` keeps 3000 alive so both engines agree). New script `267_vm_generators.js` (Node golden; equal on both engines and under `DWELLER_JS_GC=stress`). test262 at d53ebc8b19 (-j 4), VM off → on: `expressions/yield` 46 → 54/63, `statements/generators` 162 → 224/266, `built-ins/GeneratorPrototype` 48 → 48/61; total 256 → 326/390, no test that passes off fails on. `js_conformance`, `js_conformance_vm`, `aye_unit_tests` (opcode count), `errors` green; under `DWELLER_JS_GC=stress` the VM scripts 264, 266 and 267 are equal to their goldens, the whole stress run of the two js tests did not finish in 30 minutes here and is still to run; `check_layers.py` and `check_taste.py` pass. Left: `yield*` re-wraps the inner result (reads `value` of a result that is not done, as the tree walker does; 4 test262 tests); the full `DWELLER_JS_GC=stress` run; the tree walker's `yield*` rethrows instead of a TypeError when `throw` is missing; `%GeneratorFunction.prototype.prototype%` is not reachable from `Object.getPrototypeOf(function* () {})` (13 `GeneratorPrototype` tests, both engines); the rest are parser early errors.

#### P4.72: async functions on resumable frames
- goal: `async function`, async arrows and methods, `await`, async generators, `for await`, and module top-level await run in the VM on the frames of P4.71, with the driver in `lib.aye` unchanged.
- design: `await` suspends with kind "await" and returns to the driver; the driver resumes with the settled value or throws into the frame. Modules: the module function compiles like an async function. `coroutine.aye` keeps only the tree walker's uses.
- files: `browser/js/compiler.aye`, `browser/js/vm.aye`, `browser/js/coroutine.aye`, `browser/js/modules.aye`
- v8: `bytecode-generator.cc` (`VisitAwait`, `BuildAwait`, `BuildAsyncReturn`, async generator paths), `src/builtins/builtins-async-function-gen.cc`
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures window_dynamic_scripts > target/p4.72.log 2>&1; echo $?` → 0; test262 `test/language/expressions/await test/language/statements/for-await-of` VM on ≥ off
- estimate: async-heavy site code 2–3×
- model: opus
- status: done 2026-10-16. `async function`, async arrows, methods and class methods, async generators and the module function compile and run in the VM on the frames of P4.71 (V8 `VisitAwait`, `BuildAwait`); the drivers in `lib.aye` are unchanged. An async function is a `GenFrame` generator with the generator prologue; `await x` is a suspend point with kind 1 (`SuspendGenerator r_gen, kind` takes the kind as a second operand, kept in `VmGenFrame.kind`, which `__engine.kindOf` reads through `vmKindOf`), so the driver resumes it with the settled value or throws into it; `return` is the ordinary return. `callCoroutine` hands a function the VM compiles to `vmCallAsync`: `invoke` runs to the initial suspend and returns the frame's generator, which goes to `asyncDriver` (or `asyncGenDriver`); an exception thrown while binding an async function's parameters becomes a rejected promise (`vmAsyncRejected`), an async generator's is thrown. `for await` is `vmVisitForOf` with three opcodes: `GetAsyncIterator r` (128; `Symbol.asyncIterator`, else the sync iterator with a flag in `r+3`), `AsyncIteratorNext r` (129) and `IteratorResult r` (130; TypeError for a non-object, the done flag, the value); each result is awaited, and on the sync path also the value, which orders the ticks as Node does (the tree walker awaits once). Modules: `moduleRecord` makes the module's generator with `vmModuleGenerator`; module variables (`LOC_MODULE`) are context slots of a context the module function always makes (`LdaContextSlot` already reads import cells), import declarations are const stores of the cells, and `CallModuleRuntime [node], r` (131) runs `ModuleLocals`, `import.meta` and `import()` (the specifier and options in `r`, `r+1`). `coroutine.aye` now runs only what the VM does not compile. New script `268_vm_async.js` (Node golden; equal on both engines and under `DWELLER_JS_GC=stress`). test262 at d53ebc8b19 (-j 4), VM off → on: `expressions/await` 20 → 20/22, `statements/for-await-of` 1071 → 1137/1234, `statements/async-function` 47 → 47/74, `expressions/async-arrow-function` 31 → 31/60 (total 1169 → 1235/1390); also `statements/async-generator` 156 → 217/301, `expressions/async-generator` 320 → 442/622, `module-code` 441 → 441/595, `expressions/dynamic-import` 310 → 310/621, `expressions/import.meta` 20 → 20/22; no test that passes off fails on. `js_conformance`, `js_conformance_vm` (with its GC stress pass), `jsdom_fixtures`, `window_dynamic_scripts`, `aye_unit_tests` (opcode count), `errors` green; the async, module and VM scripts equal their goldens under `DWELLER_JS_GC=stress` on both engines; `check_layers.py` and `check_taste.py` pass. Left: `yield*` in an async generator (falls back to the tree walker), an assignment to an import (falls back; it throws at run time), closing an async iterator does not await `return()` (both engines).

#### P4.73: inline caches: polymorphic, megamorphic, stores, calls, elements
- goal: the IC table of `BYTECODE.md` section 3, except prototype chains deeper than one (P4.74).
- design: a feedback vector per compiled function; load and store entries go monomorphic → up to 4 → megamorphic (a global 4096-entry stub cache keyed by `(shape, atom)`); store transitions record `(old shape, new shape, slot)` and whether `props` grows (`shapes.aye` returns both); call entries skip checks for a compiled target; keyed entries by element kind (dense array, string, arguments; typed arrays in P4.76). `DWELLER_JS_VM_STATS=1` adds IC state counts.
- files: `browser/js/vm.aye`, `browser/js/shapes.aye`, `browser/js/compiler.aye` (slot allocation)
- v8: `src/objects/feedback-vector.h`, `src/ic/ic.h`, `src/ic/ic.cc`, `src/ic/stub-cache.h`, `src/ic/handler-configuration.h`
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures jsdom_gc_stress > target/p4.73.log 2>&1; echo $?` → 0; `DWELLER_JS_VM=1 browser/tests/jsbench/run.sh` all checksums equal
- estimate: 1.2–1.4× on richards, splay, raytrace, three
- model: opus

#### P4.74: prototype shapes and the prototype epoch
- goal: loads that find the property up a prototype chain of any depth hit the IC (class hierarchies, three.js methods).
- design: `BYTECODE.md` section 3, items 2 and 3: a root shape per prototype object (`rootShape(proto)`), so a shape fixes the prototype; `Object.setPrototypeOf` and `__proto__` writes move an object to the new root or to dictionary mode; objects used as prototypes are marked, and a change of their keys, flags or prototype increments `Heap.proto_epoch`; load entries for a holder up the chain record the epoch. Count epoch bumps after load on the 22 sites (`DWELLER_JS_VM_STATS=1`) and note them.
- files: `browser/js/shapes.aye`, `browser/js/heap.aye`, `browser/js/interp.aye` (prototype writes), `browser/js/vm.aye`
- v8: `src/objects/prototype-info.h`, `src/objects/map.h` (prototype in the map), `src/ic/ic.cc` (`LoadIC::ComputeHandler` holder lookups)
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures jsdom_gc_stress > target/p4.74.log 2>&1; echo $?` → 0; `DWELLER_JS_VM=1 python3 browser/tools/page_load.py check` exit 0
- estimate: 1.1–1.3× on three, raytrace, methods
- model: opus

#### P4.75: the VM on by default
- goal: `DWELLER_JS_VM` defaults to 1; `DWELLER_JS_VM=0` keeps the tree walker as an escape hatch.
- design: all gates of `BYTECODE.md` section 5 with the VM on: js suite (also GC stress), DOM fixtures, the window tests, `page_load.py check` on the 22 sites, test262 VM on ≥ off; `DWELLER_JS_VM_STATS=1` fallback reasons empty on the js suite and listed for the 22 sites (each reason left becomes a card or is fixed here). Record the suite against `baseline.json` and `node --jitless`, and the site `scripts`/`timers` phases, in `BYTECODE.md` and `PERF.md`.
- files: `browser/js/interp.aye`, `browser/js/BYTECODE.md`, `browser/js/PERF.md`, `docs/browser.md` (engine pipeline)
- v8: none
- verify: `cargo test --release --test browser > target/p4.75.log 2>&1; echo $?` → 0 (the one full run of this plan, on the branch before the merge; then CI); `python3 browser/tools/page_load.py check` exit 0
- estimate: suite geomean 3–4× against the baseline; site `scripts` phase 1.5–2.5×
- model: opus

#### P4.76: typed arrays over real bytes (covered by P11.3)
- goal: `ArrayBuffer` keeps its bytes in native memory, not in an array of `Value`s (`_bytes` in `lib.aye`, 16 bytes per byte), and typed-array element get/set in compiled code is a keyed IC fast path. Needed for three.js `BufferAttribute` and for WebGL uploads.
- design: an `ObjData` byte store owned by the buffer (freed with it); `typed.aye` reads and writes it directly; `Float32Array`/`Uint8Array`/... element kinds in the keyed IC with bounds and detach checks; `DataView` through the same store. The WebGL design gets a zero-copy view of the bytes from this store (coordinate with that card).
- files: `browser/js/typed.aye`, `browser/js/lib.aye`, `browser/js/heap.aye`, `browser/js/gc.aye`, `browser/js/vm.aye`
- v8: `src/objects/js-array-buffer.h`, `src/builtins/builtins-typed-array.cc`, `src/builtins/typed-array-*.tq`, `src/objects/elements-kind.h`
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm jsdom_fixtures > target/p4.76.log 2>&1; echo $?` → 0; test262 `test/built-ins/TypedArray test/built-ins/ArrayBuffer test/built-ins/DataView` ≥ before; `browser/tests/jsbench/run.sh three` checksum equal
- estimate: typed-array loops 5–20×; `three` 1.5–3×
- model: opus

#### P4.77: register optimizer and fused instructions
- goal: fewer instructions per JS operation, chosen by measurement.
- design: port V8's register optimizer (removes `Ldar`/`Star` pairs and `Mov`s while emitting); count instruction pairs over the suite and the 22 sites (`DWELLER_JS_VM_STATS=pairs`) and add fused forms only for the top pairs that win on the suite (candidates: compare-and-jump, `GetNamedProperty` on `this`, three-address numeric operators). Keep only changes that improve the suite geomean; record the rest as tried in `BYTECODE.md`.
- files: `browser/js/compiler.aye`, `browser/js/bytecode.aye`, `browser/js/vm.aye`, `browser/js/BYTECODE.md`
- v8: `src/interpreter/bytecode-register-optimizer.h`, `bytecode-register-optimizer.cc`, `bytecode-peephole-table.h` (if present in the checkout)
- verify: `cargo test --release --test browser -- js_conformance js_conformance_vm > target/p4.77.log 2>&1; echo $?` → 0; `BENCH_BASELINE=<main's binary> BENCH_RUNS=5 browser/tests/jsbench/run.sh` geomean > 1.05
- estimate: 1.1–1.25×
- model: opus

#### P4.78: remove the tree walker's execution paths
- goal: once P4.75 has been on `main` with no fallback reason on the js suite, test262 and the 22 sites, delete the tree walker's evaluation (`eval`/`exec` paths of `interp.aye` the VM does not call), `fast_expr.aye`, the per-AST-node `prop_caches`, the coroutine path of `coroutine.aye`, and `DWELLER_JS_VM`.
- design: the AST, the resolver and the `Interp` helpers stay. A construct that still falls back is compiled first, in this card or a new one, not kept on the tree walker.
- files: `browser/js/interp.aye`, `browser/js/fast_expr.aye`, `browser/js/coroutine.aye`, `browser/js/shapes.aye`, `tests/browser.rs` (`js_conformance_vm` folds into `js_conformance`), `docs/browser.md`, `browser/js/PERF.md`
- v8: none
- verify: `cargo test --release --test browser -- js_conformance js_command_line jsdom_fixtures jsdom_gc_stress window_dynamic_scripts > target/p4.78.log 2>&1; echo $?` → 0; `python3 browser/tools/page_load.py check` exit 0; `wc -l browser/js/*.aye` smaller
- estimate: no speedup; less code
- model: sonnet

#### P4.79: per-frame JS cost of the f1 game
- goal: the user's game `~/Documents/f1/web` (three ^0.170, read-only) loads in Dweller and its per-frame JS time is measured: `window_perf.py` gets a `frames URL` mode that prints the p50/p95 of script time per animation frame (the `ui_timers` records of rAF callbacks), recorded in `BYTECODE.md` with the matching numbers from Chrome.
- depends on: the WebGL cards (designed in parallel) and WebAssembly (the game's physics; no design yet, see the Layer 0 backlog)
- files: `browser/tools/window_perf.py`, `browser/js/BYTECODE.md`
- v8: none
- verify: `python3 browser/tools/window_perf.py frames http://localhost:5173/ --binary target/dweller` prints frame-script p50/p95 (the game served by `pnpm dev` in `~/Documents/f1/web`)
- estimate: measurement
- model: sonnet

### Layer 0 backlog

Not yet cards. Before one becomes a card, main checks it is still needed
after the cards above.

- P4.54 macOS crash record: install the telemetry crash handler on macOS so it writes the record and the process exits with 128+sig; today the handler runs but the process stays alive (SDL/Cocoa signal handling suspected); verify: `window_telemetry` SIGSEGV step on macOS (P4.53).
- P3.64 slice 2: in a container-query document, restyle subtrees when no container's size changed; skip the whole-document `stylesCurrent` hash for attribute writes (details in `PERF.md`).
- P4.11 partial restyle for DOM mutations incl. shadow roots and `:has()` (P4.10).
- P4.14 event dispatch and MutationObserver from prelude JS into native Aye (~15% of JS samples) (P1.11).
- JS engine speed: P4.21 GC threshold factor (GC 14.5% of JS samples); P4.24 smaller JS objects, property storage that shrinks, GC threshold relative to live size; P4.28 slab for unshared computed-style arrays; P4.29 `Prop` 48 → 32 B, accessors stored separately; P4.30 bytecode VM: now an umbrella, designed in `browser/js/BYTECODE.md`, cards P4.59–P4.79 above.
- JS builtins speed, after P4.75 (V8 references in `BYTECODE.md` section 7): shortest-decimal number formatting and exact parsing instead of the snprintf/strtod loop (`json` workload; v8 `src/base/numbers/fast-dtoa.h`, `bignum-dtoa.h`, `strtod.cc`); a nonrecursive regexp engine (no 1 GB thread for deep matches; v8 `src/regexp/regexp-bytecode-generator.h`, `regexp-interpreter.h`).
- WebAssembly: the user's game (`~/Documents/f1/web`) runs its physics in WASM; Dweller has none. Needs its own design (an interpreter in Aye first, as this plan does for JS); P4.79 waits on it and on WebGL.

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


## Parallel track: P11 WebGL and the APIs games need

Decision of 2026-10-15 (Decisions): the user builds 3D web games, so
WebGL is required. The design is in `docs/webgl.md`, and every card
below cites its section there.

- **Acceptance target:** the user's game, `~/Documents/f1/web` (Vite +
  TypeScript, three 0.170.0, read-only).
- **three.js r163 and later need WebGL 2,** so WebGL 2 comes first.
- **GPU backend:** ANGLE, through EGL + GLES 3.0 headers via
  `@cimport`. Contexts use ANGLE's WebGL compatibility mode, which
  validates and translates every shader.
- **Pixels reach the window in two steps:** a per-frame readback into
  the existing raster cache first (P11.7), GPU compositing of the whole
  page later (P11.22, P11.23).

P11 runs beside P4. The GPU compositing cards (P11.22, P11.23) edit
`ui/pagedraw.aye` and `ui/window.aye`, so they do not run at the same
time as a P4 card on those files.

**Rules for every card:**

- **Reference first.** Every card names a reference in the read-only
  checkout `/Users/solemnis/Documents/chromium/src` (paths below are
  relative to it). Read the known implementation before writing code.
- **Licenses.** V8 and ANGLE (BSD): porting is allowed, with a comment
  naming the source file and the license in `browser/licenses/`. Blink:
  algorithm reference only, never copy code (some files are LGPL).
- **Tests.**
  - Run only the tests named in verify, with output to a log under
    `target/`.
  - WebGL tests run headless with `DWELLER_GL=swiftshader`.
  - Fixtures live in `browser/tests/webgl/`.
  - Rust test names start with `webgl_` or `gpu_`.

#### P11.1: ANGLE binaries
- goal: pin ANGLE (`browser/gpu/angle.rev`: the `angle_revision` of the reference checkout's `DEPS`), and add:
  - `browser/gpu/build_angle.sh`: standalone `fetch angle` + `gclient sync` into `~/.cache/dweller/angle-src`, `gn gen` with the args of `docs/webgl.md` §1, `autoninja libEGL libGLESv2` + the SwiftShader ICD, and the outputs, headers and license files copied to `target/angle/<rev>/`;
  - `browser/gpu/fetch_angle.sh`: downloads the release asset and checks `browser/gpu/angle.lock`;
  - `.github/workflows/angle.yml` (manual or on `angle.rev` change): builds on `macos-latest` and `ubuntu-24.04` and uploads `angle-<rev>-<os>-<arch>.tar.zst` to release `angle-<rev>`;
  - in the `browser` job of `ci.yml`: `fetch_angle.sh`, cached by `angle.lock`.
- files: `browser/gpu/`, `.github/workflows/angle.yml`, `.github/workflows/ci.yml`, `scripts/setup-ubuntu.sh` (Vulkan loader, if the build needs it)
- reference: `third_party/angle/doc/DevSetup.md`, `third_party/angle/gni/angle.gni`, `third_party/angle/DEPS`
- verify: `browser/gpu/build_angle.sh > target/angle-build.log 2>&1; echo $?` → 0 (run in the background, one blocking wait); `nm -gU target/angle/*/lib/libEGL.dylib | grep -c eglGetPlatformDisplay` → 1; the `angle` workflow green on both OSes, and the asset hash in `angle.lock`
- time box: 45 min of agent work, plus the build wait; model: sonnet
- status: done 2026-10-15. Pinned `9bbe9ce189115e468aea2d06001b1eddbb772958`. `build_angle.sh [sync|gen|build|package|all]` (`ANGLE_SRC`, `ANGLE_DEST`, `ANGLE_JOBS`): instead of `fetch angle`, a depth-1 fetch of the pinned commit plus the `.gclient` that `fetch angle` writes, `gclient sync -D --no-history` with `checkout_angle_cl_deps` and `checkout_angle_dawn_deps` off (no LLVM or Dawn checkout: 7.4 GB); args of `docs/webgl.md` §1 plus `angle_build_tests = false`; targets `libEGL libGLESv2 swiftshader_libvulkan icd_file`. Package: macOS install names made `@rpath/libEGL.dylib`/`@rpath/libGLESv2.dylib` (ANGLE's are `./…`, relative to the working directory) and re-signed ad hoc; Linux adds `libvulkan.so.1` (`angle_shared_libvulkan`); a missing license file fails the build (LLVM 10 in SwiftShader has none in the checkout, so `licenses/llvm` is libc++'s copy of the same LLVM Project license). Local macOS arm64 (M2 Pro, `-j4`): sync 4 min, build 10 min, archive 8.1 MB (libGLESv2 9.0 MB, libvk_swiftshader 22.6 MB, libEGL 0.1 MB); `nm -gU libEGL.dylib | grep -c eglGetPlatformDisplay` → 2 (the core and `EXT` entry points). `scripts/angle-smoke.c` (pbuffer 16×16, ES 3.0, clear to red, read back): metal → 255,0,0,255 ("ANGLE Metal Renderer: Apple M2 Pro"), swiftshader → 255,0,0,255 (SwiftShader with LLVM 10 works on arm64, so the pixel goldens can run on macOS too), null → no error. `angle.yml` builds on a pushed `angle-<rev>` tag (uploads, never replaces an asset) or by hand (run artifacts), and runs the smoke test on SwiftShader. The tag is pushed and both assets are in release `angle-9bbe9ce189115e468aea2d06001b1eddbb772958`; their hashes are in `angle.lock` (the Linux asset is from the first run, whose rebuild hashes differently: the build is not byte-reproducible, and `angle.yml` never replaces an asset, so `angle.lock` holds the hash of the asset in the release, not of the latest run).

#### P11.2: EGL/GLES `@cimport` smoke test
- goal: `browser/gpu/gl.aye`:
  - `dlopen` of libEGL and libGLESv2 (the search order of §1), entry points from `eglGetProcAddress`, types and constants from `@cimport` of `EGL/egl.h`, `EGL/eglext.h`, `GLES3/gl3.h` and `GLES2/gl2ext.h`;
  - one lazy display per process (`DWELLER_GL`: metal, vulkan, gl, swiftshader, null, off; §2);
  - a surfaceless ES 3.0 context with the WebGL attributes of §2.

  New CLI mode `dweller gpu-smoke`: a 64x64 FBO cleared to red, then `glReadPixels`; prints the backend, `GL_RENDERER` and `ff0000ff`.
- Check whether Aye can call a C function pointer from an `@cimport` typedef. If it cannot, add an entry to `browser/COMPILER_GAPS.md` and use `@link` + rpath instead.
- Record whether SwiftShader works on the macOS arm64 runner. If it does not, macOS CI uses `null` (§2).
- files: `browser/gpu/gl.aye` (new), `browser/main.aye`, `tests/browser.rs` (`gpu_smoke`), `browser/COMPILER_GAPS.md` if needed
- reference: `ui/gl/gl_context_egl.cc` (attributes), `ui/gl/generate_bindings.py` (run-time binding), `third_party/angle/include/EGL/eglext_angle.h`
- verify: `DWELLER_GL=swiftshader target/dweller gpu-smoke` prints `ff0000ff`; `cargo test --release --test browser -- gpu_smoke > target/p11.2.log 2>&1; echo $?` → 0 on both CI OSes
- depends on: P11.1; time box: 45 min; model: opus (FFI, gap triage)
- status: done 2026-10-15. Aye calls C function pointers: `eglGetProcAddress` gives `?fn()`, unwrapped and `@ptrCast` to a written-out `fn(...)` per entry point (25, in `gpu.Api`); no `@link`. Two `@ptrCast` gaps (COMPILER_GAPS.md 57, not blocking): it cannot make a header's `PFN...PROC` (`?fn`), nor a function from `dlsym`'s `void *` (gl.aye declares `dlsym` returning `?fn()`). `browser/gpu/include/` holds the pinned ANGLE's EGL/GLES/KHR headers (read through `angle.h`, with `EGL_EGL_PROTOTYPES=0`, `GL_GLES_PROTOTYPES=0`, `EGL_NO_X11`), so Dweller builds without ANGLE. Libraries: `DWELLER_ANGLE`, the binary's directory, `../Frameworks`, `target/angle/<rev>/lib` (relative to the working directory). The display is lazy and per process; the context is surfaceless ES 3.0 with no config, and, as in `gl_context_egl.cc`, each WebGL attribute is passed only when the display has its extension (Metal has no `EGL_EXT_create_context_robustness`: passing it gives `EGL_BAD_ATTRIBUTE`). `gpu-smoke` (64x64 RGBA8 renderbuffer FBO) on M2 Pro: default/metal "ANGLE Metal Renderer: Apple M2 Pro" `ff0000ff`, swiftshader "SwiftShader Device (LLVM 10.0.0)" `ff0000ff`, null no GL error (reads `2a2a2a2a`), all "OpenGL ES 3.0"; no ANGLE, `DWELLER_GL=off` or an unknown name print "WebGL unavailable: ..." and exit 0. Test `gpu_smoke` (swiftshader): without ANGLE it checks the unavailable message and skips, unless `angle.lock` has hashes (then it fails). SwiftShader on the macOS arm64 runner is checked by that test once `angle.lock` has hashes.

#### P11.3: `ArrayBuffer` on one contiguous byte store
- goal: an `ArrayBuffer` keeps its bytes in one heap-owned block (pointer, length, detached flag), freed by the GC. It does not keep them in a JS array of numbers (`lib.aye` `_bytes`). The views (`typed.aye`) and `DataView` read and write the block. Native code gets a slice of `(ptr, len)` for a view.
- Behavior and results stay the same, and creating a `Float32Array` of 1M elements becomes 10x faster or more.
- First check the Aye repo's WIP P3.58 ("native `TypedArray`", 34c2234) and take it over if it fits.
- Prerequisite for WebGL buffers and textures (§3), WASM memory and Worker transfer.
- files: `browser/js/lib.aye`, `browser/js/typed.aye`, `browser/js/heap.aye`, `browser/js/gc.aye`
- reference: `v8/src/objects/js-array-buffer.h` (backing store, detach), `v8/src/objects/backing-store.h`
- verify: `cargo test --release --test browser -- js_conformance > target/p11.3.log 2>&1; echo $?` → 0 (the 530 scripts of `browser/tests/js`, `256_typed_array_views.js` among them); test262 `built-ins/TypedArray*` and `ArrayBuffer` pass rate not lower (P10.3 when it exists, else `dweller js --test262` over that directory); a 1M `Float32Array` fill benchmark before and after in the PR
- time box: 45 min (split off the GC part if it is larger); model: opus
- status: done 2026-10-15. WIP P3.58 (34c2234) is in neither repo's history, so this is new work after V8's `JSArrayBuffer`/`BackingStore`. An `ArrayBuffer` is an object whose data is `ObjData.Bytes { mem, detached }`: one native allocation outside the scanned lists, freed in `heap.freeObj` when collected (a large store counts toward the next GC) or by `transfer()`/`transferToFixedLength()` (detach: views read length 0 and undefined, `set`/`fill`/`slice`/construct/DataView throw `TypeError`). Views name the buffer itself; elements are read and written as little-endian bytes (`viewRead`/`viewWrite`). Natives `bufNew`/`bufLen`/`bufCopy` (memmove)/`bufDetach`, `typedFill` (one conversion, then doubling copies), `typedSetFrom` (one memmove for the same kind, else converted from a copy when the two overlap); `ArrayBuffer(n)` takes ToIndex (negative → `RangeError`). `js.viewBytes(it, o)` gives native code a view's bytes (WebGL, WASM). On the bytes too: TextEncoder/TextDecoder (moved to `lib.aye`; native UTF-8 with the Encoding standard's U+FFFD per maximal subpart; `encodeInto`), Blob/Response/FileReader `arrayBuffer()` (`ak.latin1Into`/`latin1Of`), `crypto.getRandomValues` (random bytes; `TypeMismatchError` for floats, `QuotaExceededError` over 65536), canvas `ImageData.data` (a real `Uint8ClampedArray`; `canvasPixels` copies to/from its bytes), `structuredClone` (buffers copied, views share their cloned buffer). `new Float32Array(1 << 20)` 264 → 11–14 ms; element loops unchanged (interpreter-bound), numbers in `browser/js/PERF.md` "P11.3". Fixtures `258_array_buffer_bytes.js` (conversions, aliasing, DataView endianness, overlapping set/subarray, fill/copyWithin/slice, transfer/detach, UTF-8) and jsdom `array_buffer_bytes.html`; `cargo test --release --test browser -- js_ jsdom window canvas` 44 passed; `aye test browser` 142 passed. Left: resizable buffers (`maxByteLength`/`resize`), BigInt64 elements are still Numbers (pre-existing), XHR `responseType` has no `'arraybuffer'` yet, postMessage transfer lists; the repo has no test262 runner (P10.3), so the test262 rate was not measured.

#### P11.4: WebGL 2 context and drawing buffer
- goal:
  - `canvas.getContext('webgl2', attrs)` returns a `WebGL2RenderingContext` (one per canvas; `null` without ANGLE).
  - The drawing buffer is an FBO with color + depth/stencil and an MSAA renderbuffer resolved at frame end (§2).
  - Context attributes and `getContextAttributes`.
  - `drawingBufferWidth`/`Height`, resized on `canvas.width` and `height`.
  - `clearColor`, `clear`, `viewport`, `scissor`, `enable`/`disable`, `colorMask`, `readPixels` into a `Uint8Array`, and `getError` with the WebGL error rules (§3).
  - `getParameter` for the state above plus the limits three.js queries at startup (`MAX_TEXTURE_SIZE`, `MAX_*_UNIFORM_VECTORS`, `VERSION`, `SHADING_LANGUAGE_VERSION`, `MAX_SAMPLES`), and `getShaderPrecisionFormat`.
  - Constants generated from `gl3.h` into a table.
- files: `browser/engine/webgl_api.aye` (new), `browser/engine/webgl_prelude.aye` (new), `browser/engine/canvas_prelude.aye` (`getContext` dispatch), `tests/browser.rs` (`webgl_context`), `browser/tests/webgl/context.html`
- reference: `third_party/blink/renderer/modules/webgl/webgl_rendering_context_base.cc`, `webgl2_rendering_context_base.cc` (validation order, error codes), `third_party/blink/renderer/platform/graphics/gpu/drawing_buffer.cc`
- verify: `cargo test --release --test browser -- webgl_context > target/p11.4.log 2>&1; echo $?` → 0 (`dump --js` of `context.html` prints the cleared pixel, the attributes, `getError` sequences and limits matching its `.txt`)
- depends on: P11.2, P11.3; time box: 45 min; model: opus
- status: done 2026-10-15. `getContext('webgl2', attrs)` makes one context per canvas (another kind on the same canvas, and `webgl`/`experimental-webgl` until P11.29, give null; without ANGLE or with `DWELLER_GL=off` it gives null and 2D still works). The context hangs off the canvas's bitmap (`CanvasBitmap.gl`), so it is freed with the canvas (detached sweep, page teardown). Drawing buffer (`browser/gpu/drawing_buffer.aye`, after Blink's `DrawingBuffer`): an FBO with an RGBA8 (RGB8 for `alpha: false`) color texture, a depth/stencil renderbuffer per attributes, and with `antialias` a 4-sample MSAA FBO the page draws into, blitted into the texture before any read; sized to `canvas.width`/`height` (scaled down to `min(MAX_TEXTURE_SIZE, MAX_RENDERBUFFER_SIZE)`, at least 1x1; devicePixelRatio does not enter, as the spec says), resized and cleared on a width/height write; viewport and scissor box start at its size. Presentation: at the end of the animation frame callbacks of a frame (`runTimer`: the last due rAF) and before rendering (`Page.layoutPage`, so `screenshot`, the agent and the window), every context that drew since the last frame is read back into its canvas bitmap (rows flipped, un-premultiplied when `premultipliedAlpha`, alpha 255 without `alpha`) and, without `preserveDrawingBuffer`, cleared to 0/1/0 with the page's scissor, masks, clear values and bindings restored. `toDataURL`/`toBlob` read the drawing buffer as it is now (blank after a presented frame, as in Chrome), the canvas keeps its last frame; `drawImage(webglCanvas)` reads what was drawn since the last frame. API (`webgl_api.aye` natives, `webgl_prelude.aye` classes): `getContextAttributes` (granted `antialias`), `drawingBufferWidth/Height`, `isContextLost`, `clearColor/clearDepth/clearStencil/clear`, `viewport`, `scissor`, `enable/disable/isEnabled`, `colorMask/depthMask/stencilMask(Separate)`, `getError` (the front end's first error, then GL's), `getParameter` (state, the limits three.js reads, `Int32Array`/`Float32Array`/boolean arrays, VERSION `WebGL 2.0`, SHADING_LANGUAGE_VERSION `WebGL GLSL ES 3.00`, VENDOR/RENDERER `WebKit`/`WebKit WebGL`, bindings null until their cards, bad names `INVALID_ENUM`), `getShaderPrecisionFormat`, `readPixels` (RGBA/UNSIGNED_BYTE into `Uint8Array`/`Uint8ClampedArray` with offset; other valid pairs `INVALID_OPERATION`, too small `INVALID_OPERATION`, null `INVALID_VALUE`), `getSupportedExtensions` `[]`. ANGLE's WebGL-compatibility context checks the enums and values. The 558 constants are generated from `gl3.h` (minus the ES-only ones, plus the six `*_WEBGL`) by `browser/tools/gen_webgl_constants.py` into a table installed on first use of `WebGL2RenderingContext`; the object classes (`WebGLBuffer` ... `WebGLTransformFeedback` on a `WebGLObject` base of context, name, deleted; `WebGLUniformLocation`, `WebGLActiveInfo`) are the scaffold for P11.5. Tests: `webgl_context` (`dump --text --js` of `browser/tests/webgl/context.html` = `context.txt`: attributes, limits, error sequences, scissored and masked clear pixels, resize, `alpha: false`, MSAA readback, drawImage, cleared after a frame, preserved with `preserveDrawingBuffer`), `webgl_screenshot` (`clear.html` = `clear.png`: scissored clears, premultiplied half-transparent red over the page, a canvas drawn in rAF); both on swiftshader, and the same output on metal locally (M2 Pro); without ANGLE they check the null path while `angle.lock` has no hashes. `cargo test --release --test browser -- gpu webgl screenshot_` 7 passed with ANGLE; `-- gpu webgl canvas js_conformance jsdom` 8 passed without; `aye test browser` 155 passed. Left: `failIfMajorPerformanceCaveat` is reported but not enforced on SwiftShader; lost-context detection (`glGetGraphicsResetStatus`), `webglcontextlost`/`restored`, `WEBGL_lose_context` and the 16-context limit; `OffscreenCanvas.getContext('webgl2')`; compositing the texture without readback is P11.7.

#### P11.5: buffers, shaders, programs, draws: a triangle
- goal:
  - `createBuffer`, `bindBuffer`, `bufferData`, `bufferSubData`.
  - `createShader`, `shaderSource`, `compileShader`, `getShaderParameter`, `getShaderInfoLog`; `createProgram`, `attachShader`, `linkProgram`, `getProgramParameter`, `getProgramInfoLog`, `useProgram`.
  - `getAttribLocation`, `bindAttribLocation`, `vertexAttribPointer`, `enableVertexAttribArray`, `vertexAttrib*`.
  - `getUniformLocation` and `uniform*` / `uniformMatrix*` with the location validity rules; `getActiveAttrib`, `getActiveUniform`.
  - `drawArrays`, `drawElements`, and `delete*` / `is*`.
  - WebGL object wrappers with context ownership and deleted-state checks (§3).
  - Shaders go to ANGLE as given and are never parsed by Dweller (§4).
- files: `browser/engine/webgl_api.aye`, `webgl_prelude.aye`, `browser/tests/webgl/triangle.html` (+ golden PNG), `tests/browser.rs` (`webgl_triangle`)
- reference: as P11.4; `third_party/angle/src/libANGLE/Compiler.cpp` (`SelectShaderSpec`: WebGL spec applied by ANGLE)
- verify: `cargo test --release --test browser -- webgl_triangle webgl_context > target/p11.5.log 2>&1; echo $?` → 0: `readPixels` at 3 points gives the triangle's color, the background and the edge; a bad shader gives `COMPILE_STATUS` false and a non-empty log; a buffer from another context gives `INVALID_OPERATION`
- depends on: P11.4; time box: 45 min; model: sonnet
- status: done 2026-10-15. Natives in `webgl_api.aye` (`webglNew`, `webglBufferData`, `webglShaderSource`, `webglParam`, `webglLog`, `webglLocation`, `webglBindAttrib`, `webglActive`, `webglUniformv`, and `webglCall` ops for binds, attach/detach, link/compile/validate, attribute pointers and values, draws, deletes, scalar uniforms); 55 new entry points in `gl.aye` in the hand-written signature style (gap 57 landed in aye main as O1.30 but `AYE_REV` still pins the compiler before it; byte offsets are `usize` parameters). The prelude has `createBuffer`/`bindBuffer` (all eight WebGL 2 targets; an element array buffer stays one, INVALID_OPERATION otherwise)/`bufferData` (size, ArrayBuffer or view with `srcOffset`/`length`)/`bufferSubData`/`deleteBuffer`/`isBuffer`; shaders and programs (`shaderSource` over 512 KB INVALID_VALUE, `getShaderSource`, `getAttachedShaders`, `detachShader`, one shader per type); `getAttribLocation`/`bindAttribLocation`/`getUniformLocation`, `uniform1..4f/i/ui(v)`, `uniformMatrix{2,3,4,2x3,3x2,2x4,4x2,3x4,4x3}fv` (typed arrays read in place, sequences converted; `srcOffset`/`srcLength`; length not a multiple INVALID_VALUE); `getActiveAttrib/Uniform`; `vertexAttrib(I)Pointer`, `enable/disableVertexAttribArray`, `vertexAttrib1..4f(v)`, `vertexAttribI4i/ui(v)`; `drawArrays`, `drawElements`; `getParameter` of the buffer bindings and `CURRENT_PROGRAM`. Object rules after Blink: another context's object INVALID_OPERATION everywhere; a deleted object INVALID_OPERATION when bound or used, INVALID_VALUE when a program or shader GL has freed is queried (a deleted program in use and a deleted shader still attached live on); delete of a deleted object or null does nothing; a location of a program not in use or from before a relink INVALID_OPERATION, a null location ignored. Shader text goes to ANGLE as given (§4); type mismatches, aligned offsets, reserved `webgl_` names, unlinked programs and index ranges are ANGLE's WebGL validation. Out-of-range draws: with robust access (SwiftShader/Vulkan) ANGLE draws with clamped reads and no error; Metal has no `EGL_EXT_create_context_robustness`, so ANGLE range-checks and gives INVALID_OPERATION (the two lines differ in `objects.txt` on metal; both are allowed by the spec). Tests: `webgl_triangle` (`triangle.html` per-vertex colors with MSAA, `quad.html` drawElements with a uniform color and mat4 plus bufferSubData of the indices, `shader_errors.html` compile and link info logs in the DOM; each = its `.png` on SwiftShader), `webgl_objects` (`objects.html` = `objects.txt`: every error code above, active info, readPixels inside, outside and on the edge of a drawn triangle). Metal on an M2 Pro gives the same pixels for `shader_errors`, 71 pixels off by 1 in the triangle's interpolated colors, and 3 non-antialiased edge pixels of the rotated quad decided the other way by the two rasterizers. `cargo test --release --test browser -- gpu webgl screenshot_` 9 passed with ANGLE; `-- gpu webgl canvas js_conformance jsdom` 10 passed without; `aye test browser` 155 passed. Left: `getBufferParameter`, `getVertexAttrib(Offset)`, `getUniform`, `copyBufferSubData`/`getBufferSubData`, instanced draws and `drawRangeElements`/`drawBuffers` (P11.12); a bound PIXEL_PACK_BUFFER would capture the drawing buffer readback (with P11.8).

#### P11.6: textures
- goal:
  - `createTexture`, `bindTexture`, `texParameter*`, `generateMipmap`, `activeTexture`, `texStorage2D`.
  - `texImage2D` and `texSubImage2D` from an `ArrayBufferView` (type checked against format and type), `ImageData`, `HTMLImageElement`, `HTMLCanvasElement` (2D), `ImageBitmap` and `OffscreenCanvas`.
  - `pixelStorei` with `UNPACK_FLIP_Y_WEBGL`, `UNPACK_PREMULTIPLY_ALPHA_WEBGL`, `UNPACK_COLORSPACE_CONVERSION_WEBGL` and `UNPACK_ALIGNMENT`.
  - A non-origin-clean image throws `SecurityError` (§3).
  - `createImageBitmap` options `imageOrientation` and `premultiplyAlpha` (three's `ImageBitmapLoader` uses them).
- files: `browser/engine/webgl_api.aye`, `browser/engine/canvas_prelude.aye` (`ImageBitmap` options), `browser/tests/webgl/textures.html`, `tests/browser.rs` (`webgl_textures`)
- reference: `third_party/blink/renderer/modules/webgl/webgl_rendering_context_base.cc` (`TexImageHelper*`, `WebGLImageConversion`), `third_party/blink/renderer/platform/graphics/gpu/webgl_image_conversion.cc` (algorithm only)
- verify: `cargo test --release --test browser -- webgl_textures > target/p11.6.log 2>&1; echo $?` → 0: a textured quad from a PNG, a 2D canvas and an `ImageData`, read back with and without `FLIP_Y`; a cross-origin image throws `SecurityError`
- depends on: P11.5; time box: 45 min; model: sonnet
- status: done 2026-10-16. Natives `webglTex` (texImage2D/texSubImage2D) and ops for `bindTexture`/`activeTexture`/`deleteTexture`/`texParameteri`/`texParameterf`/`generateMipmap`/`texStorage2D`/`pixelStorei` in `webgl_api.aye` (7 new entry points in `gl.aye`); the prelude has `createTexture`/`isTexture`/`deleteTexture`/`bindTexture` (a texture keeps the target it was first bound to; per-unit bindings tracked for `getParameter(TEXTURE_BINDING_*)`), `activeTexture`, `texParameter*`, `generateMipmap`, `texStorage2D`, `pixelStorei` and all overloads of `texImage2D`/`texSubImage2D` (6/7 arguments with a source; 9/10 with an `ArrayBufferView` and `srcOffset`, or a source with an explicit size = its top left corner, larger gives `INVALID_VALUE`). Rules after Blink's `TexImageHelper*`: no bound texture `INVALID_OPERATION`, bad target `INVALID_ENUM`, a view's type against the GL type (Uint8/Uint8Clamped for UNSIGNED_BYTE, Uint16 for the packed 16-bit and HALF_FLOAT, Uint32 for the packed 32-bit, Float32 ...; mismatch `INVALID_OPERATION`), a view too small for width x height with UNPACK_ALIGNMENT/ROW_LENGTH/SKIP_ROWS/SKIP_PIXELS `INVALID_OPERATION` (`srcOffset` past the end `INVALID_VALUE`), `null` data only for `texImage2D`; ANGLE checks levels, sizes, format combinations, immutable storage and parameter values. `pixelStorei`: UNPACK_FLIP_Y/PREMULTIPLY_ALPHA/COLORSPACE_CONVERSION are the front end's (colorspace only accepts BROWSER_DEFAULT_WEBGL or NONE and is the identity: Dweller has no color profiles), the alignments (1, 2, 4, 8) and row length/skips go to GL. Flip and premultiply apply to views too (RGBA and LUMINANCE_ALPHA bytes; other formats are only flipped) by repacking tight rows with the unpack state reset and restored. DOM sources (ImageData, `<img>`, 2D and WebGL canvas, OffscreenCanvas, ImageBitmap; their pixels are straight RGBA8) are flipped, premultiplied (float scale, truncated, as Blink's `Pack`) and packed in Aye for RGBA/RGB/LUMINANCE_ALPHA/LUMINANCE/ALPHA/RED/RG with UNSIGNED_BYTE and RGBA 4444, RGBA 5551, RGB 565 (other pairs `INVALID_OPERATION`, unknown enums `INVALID_ENUM`); an image that is not loaded or a closed bitmap gives `INVALID_VALUE`; a source that is not origin-clean (`sourceTainted`, the check of `drawImage`) throws `SecurityError` before any GL call. `createImageBitmap(source, {imageOrientation, premultiplyAlpha})`: `flipY` is applied when the bitmap is made, invalid values reject with `TypeError` (it rejected nothing before: a bad source threw synchronously), and the bitmap's premultiplied state (`none` straight, `premultiply` and `default` premultiplied) decides how `texImage2D` uploads it while the pixel store flags are ignored for it (WebGL spec). Test `webgl_textures` (`textures.html` + `tex2x2.png` served over HTTP for the second origin = `textures.txt`): a quad samples a 2x2 texture with NEAREST at texel centers, so the corner pixels are the same on any backend (SwiftShader here, same on metal); PNG, canvas, OffscreenCanvas, WebGL canvas, ImageData (RGBA/RGB/LA/ALPHA/R8/565/4444/5551), ImageBitmaps and typed arrays with and without flip and premultiply, texStorage2D + texSubImage2D as three.js does, alignment/row length/skips, every error above, and `SecurityError` for a cross-origin image, a canvas drawn from it and an ImageBitmap of it. `cargo test --release --test browser -- webgl_textures` 1 passed; `-- webgl gpu canvas` 8 passed. Left: a bound PIXEL_UNPACK_BUFFER (and the byte-offset overloads) give `INVALID_OPERATION` until P11.8; `texImage2D` of DOM sources into FLOAT/HALF_FLOAT/integer/sRGB-packed types, `createImageBitmap` of `ImageData`/`Blob` and the crop arguments, `resizeWidth`/`colorSpaceConversion`; a srcset image uploads its bitmap pixels, not its natural size; premultiply of a view is only for RGBA/LUMINANCE_ALPHA bytes; `getTexParameter`, 3D/array textures and cube map completeness checks (P11.8), `copyTexImage2D`, compressed textures, `HTMLVideoElement` (P11.28), `ImageBitmap.close()` is not checked by `texImage2D` beyond its empty bitmap.

#### P11.6b: createImageBitmap sources and options
- goal: `createImageBitmap(source[, sx, sy, sw, sh][, options])` as the HTML spec says, enough for three.js's `ImageBitmapLoader` (`fetch(url).then(r => r.blob()).then(b => createImageBitmap(b, {premultiplyAlpha: 'none', colorSpaceConversion: 'none', imageOrientation: 'flipY'}))`, used by `GLTFLoader`).
  - Sources: `Blob` (PNG, JPEG, GIF, WebP through the existing decoders; undecodable bytes reject with `InvalidStateError`), `ImageData`, plus the existing image, canvas, `OffscreenCanvas` and `ImageBitmap`.
  - The crop rectangle (a negative `sw`/`sh` moves the origin; outside the source is transparent black; zero gives `RangeError`).
  - Options `imageOrientation`, `premultiplyAlpha`, `colorSpaceConversion`, `resizeWidth`/`resizeHeight`/`resizeQuality`.
  - `ImageBitmap.close()`; such a bitmap uploads with `texImage2D` (P11.6 path).
- files: `browser/engine/canvas_prelude.aye` (`ImageBitmap`, `createImageBitmap`), `browser/engine/canvas_api.aye` (`canvasDecode`), `browser/tests/webgl/imagebitmap.html`, `imagebitmap.txt`, `bitmap3x2.png`, `tests/browser.rs` (`imagebitmap_sources`)
- reference: `third_party/blink/renderer/core/imagebitmap/image_bitmap.cc`, `image_bitmap_factories.cc` (algorithm only)
- verify: `cargo test --release --test browser -- imagebitmap_sources webgl_textures canvas > target/p11.6b.log 2>&1; echo $?` → 0: a fetched PNG as a Blob, an ImageData, crops, `flipY` and `premultiplyAlpha: 'none'` read back through a 2D canvas, and a Blob bitmap uploaded with `texImage2D` and read back
- depends on: P11.6; time box: 45 min; model: sonnet
- status: done 2026-10-16. New native `canvasDecode(canvas, bytes)` (`canvas_api.aye`): decodes a Blob's bytes with `image.decode` (the sniffing decoder of `<img>`), sets the canvas size to the image's and stores the pixels straight (transparent pixels as 0,0,0,0); `null` (-> `InvalidStateError`) for bytes that do not decode or are larger than a canvas. In `canvas_prelude.aye`, `sourceNode` makes a temporary canvas for an `ImageData` or a `Blob` and throws `InvalidStateError` for a source without pixels (an image that is not loaded, an empty or closed canvas/bitmap); `ImageBitmap(source, options, crop)` follows the spec's order, crop (rect = origin moved by negative sizes, clipped to the source), resize (only one of `resizeWidth`/`resizeHeight`: the other keeps the aspect ratio, rounded up as in Blink; 0 gives `InvalidStateError`), then `flipY`. A canvas-like source copied at its own scale is copied byte for byte with `canvasPixels` (so `premultiplyAlpha: 'none'` keeps (200,100,50,128) exactly); an image, a resize or an origin-unclean canvas goes through `drawImage` (taint propagates; `resizeQuality: 'pixelated'` turns smoothing off, the others use the bilinear sampler). Invalid `imageOrientation`/`premultiplyAlpha`/`colorSpaceConversion`/`resizeQuality` reject with `TypeError`; a call with 2 or 3 numbers after the source is a `TypeError`. Test `imagebitmap_sources` (`imagebitmap.html` + `bitmap3x2.png` fetched over HTTP, golden `imagebitmap.txt`): sizes and pixels of every source, crops, flips, resizes and errors, then `texImage2D` of Blob bitmaps (`none`, `premultiply`, `default`, `flipY`, ignored `UNPACK_FLIP_Y_WEBGL`, closed bitmap `INVALID_VALUE`). `cargo test --release --test browser -- imagebitmap_sources webgl_textures canvas` 4 passed; `jsdom/canvas_api` and `paint/canvas_images` unchanged. Left: `imageOrientation: 'from-image'` is `none` (no EXIF rotation), no color profiles (`colorSpaceConversion` only validated), `HTMLVideoElement`/`SVGImageElement` sources, bitmaps are decoded on the calling thread and resolve in a microtask (no off-thread decode), `resizeQuality` has no separate `medium`/`high` filters, a crop of a WebGL canvas goes through `drawImage` (not byte exact).

#### P11.7: WebGL canvases in the window, screenshots and `toDataURL` (readback compositing)
- goal: step 1 of §5.
  - At frame end, a context that drew resolves its MSAA buffer and reads into a pixel-pack buffer with a fence. The next frame maps it into the canvas's `image.Bitmap` (Y flip; un-premultiplied only when `alpha && premultipliedAlpha`) and bumps `Bitmap.version`, so the damage tracking and raster cache work unchanged.
  - `preserveDrawingBuffer: false` clears after the take.
  - `toDataURL`, `toBlob`, `drawImage(webglCanvas)`, `screenshot` and `dump` take the frame synchronously when needed.
  - The DWELLER_TIMING record `webgl_readback`.
- files: `browser/engine/webgl_api.aye`, `browser/engine/script.aye` (frame end hook), `browser/ui/window.aye`, `browser/shot.aye`, `tests/browser.rs` (`window_webgl`, `webgl_screenshot`)
- reference: `third_party/blink/renderer/platform/graphics/gpu/drawing_buffer.cc` (`PrepareTransferableResource`, preserve semantics), `docs/how_cc_works.md` (TextureLayer, idea only)
- verify: `cargo test --release --test browser -- window_webgl webgl_screenshot window_damage > target/p11.7.log 2>&1; echo $?` → 0. `window_webgl` runs a 120-frame rAF spinning triangle at 1280x720 under `DWELLER_DAMAGE_CHECK=1`: 120 canvas updates, no damage outside the canvas. The PR records `webgl_readback` p50/p95 ms at 1280x720 on the user's Mac (estimate 3–5 ms).
- depends on: P11.6; time box: 45 min; model: opus
- status: done 2026-10-16. Readback after Blink's `DrawingBuffer` (`browser/gpu/drawing_buffer.aye`): `startReadback` resolves the MSAA buffer, reads the frame with `glReadPixels` into the context's `GL_PIXEL_PACK_BUFFER` (STREAM_READ, sized to the drawing buffer), sets a `glFenceSync` and flushes, then clears without `preserveDrawingBuffer` as before; `finishReadback` waits on the fence (1 s at most), maps the buffer (`glMapBufferRange`), copies the rows flipped into the canvas bitmap (or scratch pixels scaled to it when the drawing buffer was capped), and un-premultiplies only with `alpha && premultipliedAlpha` (two pixels tested per u64, so an opaque frame costs one pass; without alpha the RGB8 buffer already reads alpha 255). The frame reads set the pack state to tight rows and restore the page's (`PACK_ALIGNMENT` etc. from `pixelStorei`); a resize or `destroyWebGL` drops a pending readback. 6 entry points in `gl.aye` (`glFlush`, `glFenceSync`, `glClientWaitSync`, `glDeleteSync`, `glMapBufferRange`, `glUnmapBuffer`; a GLsync is `?[*]u8`). When (`webgl_api.aye`): at the end of a frame's animation frame callbacks (`script.aye` `runTimer` → `endCanvasFrame`) the frame before is mapped if it was not yet and every canvas that drew starts its readback; before rendering (`Page.layoutPage` → `presentCanvases`) a canvas that drew outside rAF starts one, and outside the window (dump, `screenshot`, the agent) the readback is mapped at once, so `shot.aye` needed no change. The window (`engine.readbackNextFrame(true)` in `ui/window.aye`) maps the readbacks at the start of its next iteration (`engine.takeCanvasFrames` before `runDueTimers`; a mapped frame bumps the page's version, so the canvas is laid out and damaged as a 2D canvas is), and while one is pending it waits at most 16 ms (`READBACK_MS`): one frame of latency, the fence long signaled. `drawImage(webglCanvas)` and `texImage2D(webglCanvas)` map a pending frame first, then read what was drawn since synchronously (`frameNow`); `toDataURL`/`toBlob` read the drawing buffer as it is now, as before. DWELLER_TIMING `webgl_readback` (label `WxH`): the readback's main-thread time, its start at frame end plus the map, flip and copy. M2 Pro, metal, 1280x720, 121 frames: p50 0.93 ms, p95 1.04 ms (max 8.6 ms, the first frame); swiftshader p50 0.87, p95 1.02. Of that, the fence wait is 0.01 ms and the map and copy 0.15 ms; the per-pixel alpha loop was 4.7 ms before the u64 test (the doc's 3–5 ms estimate). Test `window_webgl` (`browser/tests/webgl/spin.html`, swiftshader, window 1280x800, `DWELLER_DAMAGE_CHECK=1`, 300 frames): ≥ 120 `webgl_readback` records of 1280x720, every frame's damage 0, the whole page (the first) or exactly the canvas and the 1-px antialiasing margin below it (1280x721), ≥ 120 such canvas updates (121: the cleared frame and the 120 drawn), no DAMAGE CHECK; it prints the p50/p95. `cargo test --release --test browser -- window_webgl webgl_screenshot window_damage` 3 passed; `-- webgl gpu canvas` 9 passed. Left: GPU compositing without readback (P11.22, P11.23); a page that binds its own PIXEL_PACK_BUFFER (P11.8) is unaffected, since the frame read binds and restores it.

#### P11.8: WebGL 2 state objects
- goal:
  - VAOs (`createVertexArray` and the rest), instancing (`drawArraysInstanced`, `drawElementsInstanced`, `vertexAttribDivisor`), `drawRangeElements`, `vertexAttribIPointer`.
  - MRT (`drawBuffers`); framebuffers and renderbuffers (`framebufferTexture2D`, `framebufferTextureLayer`, `renderbufferStorageMultisample`, `checkFramebufferStatus`).
  - Uniform buffers (`getUniformBlockIndex`, `uniformBlockBinding`, `bindBufferBase`, `bindBufferRange`).
  - Samplers.
  - 3D and 2D-array textures (`texImage3D`, `texStorage3D`, `texSubImage3D`), and integer and float formats (`R16F`, `RGBA16F`, `R32F`, `RGBA32F`, `DEPTH24_STENCIL8`, `DEPTH_COMPONENT32F`, sRGB).
  - `uniform[1234]ui`, `uniformMatrix[234]x[234]fv`.
- files: `browser/engine/webgl_api.aye`, `webgl_prelude.aye`, `browser/tests/webgl/webgl2_objects.html`, `tests/browser.rs` (`webgl2_objects`)
- reference: `third_party/blink/renderer/modules/webgl/webgl2_rendering_context_base.cc`
- verify: `cargo test --release --test browser -- webgl2_objects > target/p11.8.log 2>&1; echo $?` → 0: instanced quads, rendering to a float MRT FBO then sampling it, a UBO-driven color, and a 2D-array texture layer, each read back
- depends on: P11.5, P11.6; time box: 45 min (split UBO, samplers and 3D textures into P11.8b if larger); model: sonnet
- status: done 2026-10-16. The whole goal list landed in one card (UBOs, samplers and 3D textures did not need P11.8b; P11.8b holds the getters). Natives: `webglCall` ops 41–68 (framebuffers, renderbuffers incl. multisample, attachments, `drawBuffers`, `readBuffer`, `blitFramebuffer`, instanced and range draws, `vertexAttribDivisor`, VAOs, `bindBufferBase/Range`, `uniformBlockBinding`, samplers, `texStorage3D`, `clearBuffer{f,i,ui}v`), `webglQuery` (`checkFramebufferStatus`, `getUniformBlockIndex`, enabling an ANGLE extension), `webglTex3D` (`texImage3D`/`texSubImage3D` from typed arrays), `webglReadFormat` (`readPixels` of any format/type from a framebuffer of the page's own); 27 new `gl.aye` entry points. Rules: the drawing buffer is a real GL framebuffer, so `bindFramebuffer(null)` binds it (`drawFbo`), `drawBuffers`/`readBuffer` on it accept only BACK/NONE (mapped to `COLOR_ATTACHMENT0`/NONE), and the attachment calls with it bound are INVALID_OPERATION; only draws, clears and blits that end in it mark the canvas dirty. The element array buffer binding is per VAO in the prelude. `getExtension` now enables `EXT_color_buffer_float`, `EXT_color_buffer_half_float` and `OES_texture_float_linear` with `glRequestExtensionANGLE` (`GL_`-prefixed names, checked in `GL_EXTENSIONS`), otherwise null; float and half-float targets need it, as in Blink. Test `webgl2_objects` (`webgl2_objects.html` = `.txt`, SwiftShader, 16x16): instanced strip and indexed quads, `drawRangeElements`, `vertexAttribIPointer`, a two-target RGBA32F MRT framebuffer read as FLOAT then sampled, RGBA16F + R32UI targets, a 4-sample renderbuffer blit-resolved, a DEPTH24_STENCIL8 depth test, a UBO by base and by range, a 2D-array texture by sampler object and as a `framebufferTextureLayer` attachment, a R8UI 3D texture with `texelFetch`, and the errors of each. `-- webgl gpu canvas` 9 passed. Left: P11.8b.

#### P11.8b: getters and sources of the P11.8 objects
- goal:
  - `getSamplerParameter`, `getFramebufferAttachmentParameter`, `getRenderbufferParameter`, `getInternalformatParameter`, `getIndexedParameter`, `getVertexAttrib`, `getBufferParameter`.
  - `getActiveUniformBlockParameter/Name`, `getUniformIndices`, `getActiveUniforms`, `getUniform`.
  - `clearBufferfi`, `invalidateFramebuffer`, `invalidateSubFramebuffer`, `copyTexSubImage3D`, `compressedTexImage3D`.
  - `texImage3D`/`texSubImage3D` from DOM sources and a `PIXEL_UNPACK_BUFFER`, and `texStorage2D`/`3D` argument validation as Blink's (levels, formats).
- files: `browser/engine/webgl_api.aye`, `webgl_prelude.aye`, `browser/tests/webgl/webgl2_getters.html`, `tests/browser.rs` (`webgl2_getters`)
- reference: `third_party/blink/renderer/modules/webgl/webgl2_rendering_context_base.cc`
- verify: `cargo test --release --test browser -- webgl2_getters > target/p11.8b.log 2>&1; echo $?` → 0
- depends on: P11.8; time box: 45 min; model: sonnet

#### P11.9: three.js hello cube
- goal:
  - Copy `three.module.min.js` r170 and its `LICENSE` (MIT) from `~/Documents/f1/web/node_modules/three/build/` into `browser/tests/webgl/three/`.
  - Add `browser/tests/webgl/three_cube.html`: a `WebGLRenderer` (default options), a `MeshStandardMaterial` cube with a PNG texture, one directional light, `setAnimationLoop`.
  - Fix what three's startup needs: `getExtension` returns `null` for unsupported ones; `KHR_parallel_shader_compile` is absent or works.
  - The PR records the JS time per frame (DWELLER_TIMING `scripts` in rAF) and the GPU-side time.
- files: `browser/tests/webgl/three/` (new, vendored), `three_cube.html` (+ golden), `tests/browser.rs` (`webgl_three_cube`), `browser/engine/webgl_*.aye`
- reference: three's own `src/renderers/webgl/WebGLExtensions.js`, `WebGLCapabilities.js` (what it queries) in the vendored copy's source on npm; `third_party/blink/renderer/modules/webgl/` for the API calls it hits
- verify: `cargo test --release --test browser -- webgl_three_cube > target/p11.9.log 2>&1; echo $?` → 0: no JS error, the screenshot golden (SwiftShader) matches, and `window_webgl`-style 60 frames draw in the window
- depends on: P11.7, P11.8; time box: 45 min; model: opus (unknowns)
- status: done 2026-10-16. three.js r170 (`browser/tests/webgl/three/three.module.min.js` + `LICENSE`, MIT) loads as an ES module import and draws with its defaults. The one gap was the fixed-function state, which no card had: `blendColor`, `blendEquation(Separate)`, `blendFunc(Separate)`, `cullFace`, `frontFace`, `depthFunc`, `depthRange`, `hint`, `lineWidth`, `polygonOffset`, `sampleCoverage`, `stencilFunc(Separate)`, `stencilOp(Separate)`, `finish`, `flush` (`webglCall` ops 69–83 in `webgl_api.aye`, 14 `gl.aye` entry points; ANGLE's WebGL validation checks the enums). `getExtension` already returned null for the extensions it does not have (three asks for `KHR_parallel_shader_compile`, `EXT_texture_filter_anisotropic`, `WEBGL_debug_renderer_info` ... and works without them). Page `three_cube.html`: a 240x180 `WebGLRenderer`, a `MeshStandardMaterial` cube with `crate16.png` (sRGB), one `DirectionalLight`, `setAnimationLoop` from the texture's load for 60 frames, rotation by frame count (so the last frame is the same on the virtual and the real clock); `#finish` adds `gl.finish()` after each render. Test `webgl_three_cube` (SwiftShader): `dump --js` logs only the page's message; `screenshot` = `three_cube.png` with at most 1% of pixels off by more than 8 in a channel (a stored/fixed-Huffman PNG decoder in the test, `png_rgba`); in the window (`DWELLER_DAMAGE_CHECK=1`) ≥ 60 `webgl_readback` records of 240x180 and ≥ 60 drawn frames; it prints the drawing frames' timer p50/p95, plain and with `#finish`. Per frame, test build, M2 Pro: SwiftShader JS (rAF callback with three's render and the readback start, the `timers` record) p50 1.6–1.9 ms, p95 1.9–3.6 ms; GPU (`#finish` minus plain) about 0.25 ms p50. Metal: JS p50 2.0 ms, p95 2.5 ms; GPU about 0.5 ms p50. First frame 125 ms on SwiftShader (shader compile; the readback waits 113 ms on it); the module's `scripts` records at load 87 + 225 ms. `cargo test --release --test browser -- webgl imagebitmap` 9 passed.

#### P11.10: rAF at display vsync, frame timestamp, `performance.now()` precision
- goal: §6 frame timing.
  - rAF callbacks run in the window's frame step, before style, layout and paint, all with the same frame timestamp, at the display refresh rate (`SDL_GetWindowDisplayMode`), presented with `SDL_RENDERER_PRESENTVSYNC`. They are not a 16 ms timer.
  - `performance.now()` comes from a microsecond monotonic clock coarsened to 100 µs. The virtual clock of `dump` is unchanged.
  - Measure input latency from SDL event timestamp to present (`DWELLER_TIMING` `input_to_present`).
  - Hidden windows stay throttled (P4.55).
- files: `browser/engine/prelude.aye` (rAF), `browser/engine/script.aye` (`tick`, timers), `browser/ui/window.aye` (frame loop, vsync), `tests/browser.rs` (`window_raf_vsync`)
- reference: `third_party/blink/renderer/core/frame/frame_request_callback_collection.cc`, the HTML spec "update the rendering"; `third_party/blink/renderer/core/timing/performance.cc` (coarsening, algorithm only)
- verify: `cargo test --release --test browser -- window_raf_vsync window_damage window_transform_composite > target/p11.10.log 2>&1; echo $?` → 0. `window_raf_vsync`: 120 rAF frames; all callbacks of a frame see the same timestamp; intervals median within 10% of the display period; `performance.now()` values not all whole ms.
- depends on: none (can run after P11.2 in parallel with P11.4–P11.9, but not with a P4 card on `ui/window.aye`); time box: 45 min; model: opus
- status: done 2026-10-16. On the window's clock `requestAnimationFrame` callbacks no longer run as 16 ms timers: the loop's frame step (`ui/run-animation-frame.aye`, before the draw, after input) runs every callback queued before it (`engine.runAnimationFrames`, page and frames), each with the frame's time, then lays out and draws; frames come every display period (`SDL_GetWindowDisplayMode`, 60 Hz when it gives none; `ui_display_period` record), the loop waits for the next one (`frameTimeout`) and a late frame starts at once without catching up. The renderer is created with `SDL_RENDERER_PRESENTVSYNC`. `performance.now()` and timers read a nanosecond monotonic clock coarsened to 100 µs (`engine.coarseMs`), and the loop's wait for a timer rounds up to whole ms (`msUntilNextTimer`: a wait under 1 ms truncated to 0 made `--frames` iterations spin past the timers on Ubuntu CI, `window_scroll_follows_scripts`, `window_wheel`); `dump`'s virtual clock and its 16 ms rAF are unchanged, as are tabs the window never showed (agents', Spaces'). `DWELLER_TIMING` `input_to_present`: from an SDL key, text, button or wheel event's timestamp to the next present. This Mac (built-in display, ProMotion): `SDL_GetWindowDisplayMode` reports 120 Hz (8.33 ms), rAF in a real window runs at a median of 8.30 ms (120 frames); `input_to_present` with scripted keys on a page that animates by rAF, real window: p50 8.5 ms, p95 9.5 ms (60 keys). Test `window_raf_vsync` (`browser/tests/ui/raf_vsync.html`); `window_webgl`, `window_input_priority`, `window_damage`, `window_transform_composite`, `window_timers_yield_to_frames`, `window_paints_early` pass. Left: a tab sent to the background after it ran frames runs no rAF until shown again (what P4.55 wants; its timers and minimized windows are still P4.55); the frame clock is the display period, not the present's vsync time (P11.22 presents through ANGLE).

#### P11.11: game input: key codes, pointer events, Pointer Lock, Fullscreen
- goal:
  - Trusted `KeyboardEvent`s carry `code` (from the SDL scancode), `repeat`, `location` and the modifier flags.
  - The window dispatches `pointerdown`, `pointermove`, `pointerup`, `pointercancel` and `pointerleave` (mouse `pointerId` 1, `isPrimary`) before the mouse events, as the spec orders them.
  - `MouseEvent.movementX`/`Y`.
  - `element.requestPointerLock()` returns a promise; it uses `SDL_SetRelativeMouseMode` and allows only a user activation. `document.pointerLockElement`, `pointerlockchange`, `pointerlockerror`; Escape exits.
  - `requestFullscreen` / `exitFullscreen` / `fullscreenElement` / `fullscreenchange` via `SDL_SetWindowFullscreen`.
  - `visibilitychange` and `document.visibilityState`.
  - `structuredClone` (the algorithm P11.18 reuses).
- files: `browser/ui/window.aye`, `browser/ui/sdl.aye`, `browser/engine/prelude.aye`, `tests/browser.rs` (`window_pointer_lock`, `window_input_codes`), `browser/tests/js/NNN_structured_clone.js`
- reference: `third_party/blink/renderer/core/page/pointer_lock_controller.cc`, `third_party/blink/renderer/core/fullscreen/fullscreen.cc`, `third_party/blink/renderer/core/input/pointer_event_manager.cc` (algorithm only); `ui/events/keycodes/dom/dom_code_data.inc` (scancode → `code` table, BSD)
- verify: `cargo test --release --test browser -- window_pointer_lock window_input_codes js_conformance > target/p11.11.log 2>&1; echo $?` → 0
- depends on: none; time box: 45 min (split Fullscreen into P11.11b if needed); model: sonnet
- status: done 2026-10-16. Fullscreen stayed in this card (no P11.11b). Keys: the window's `key`/`keyUp` pass `key\ncode\nrepeat\nlocation\nmods` to the prelude's `hostEvent`; `code` is the USB scancode through the table in `browser/ui/keys.aye` (`domCode`, after `dom_code_data.inc`), `key` of non-characters (Shift, Control, Alt, Meta, F1..F24, Tab, Insert, CapsLock, numpad) from the code, `location` 1/2/3 for left/right/numpad, modifier flags, `repeat` from `SDL_KeyboardEvent.repeat`; `keyup` is new (SDL_KEYUP); modifier keys alone reach the page even with Ctrl or Cmd held. Pointer events (`Ui.pointerEvent`): `pointerdown`/`pointermove`/`pointerup` for the left, middle and right buttons (`button`, `buttons`), each followed by the mouse event of the same name unless a listener canceled it; `pointerleave` on the window's LEAVE, `pointercancel` when the window loses focus with a button held; pointerId 1, `pointerType` mouse, `isPrimary`; `MouseEvent.movementX/Y` from `SDL_MouseMotionEvent.xrel/yrel`. `click` still fires on the press (no release-time click yet). Pointer Lock: `requestPointerLock()` returns a promise, needs a transient user activation (5 s after a trusted keydown other than Escape, mousedown, pointerdown/up or click; `navigator.userActivation`), else rejects `NotAllowedError` with `pointerlockerror`; the window answers with `SDL_SetRelativeMouseMode` through `ak.hostRequest` / `engine.takeHostRequests` and `__host` events (`lock:1`, `lockerror`...); `document.pointerLockElement`, `pointerlockchange`, `exitPointerLock`; while locked, pointer and mouse move/down/up and `click` go to the locked element at the position where the lock began, with the relative motion; Escape leaves lock and fullscreen and is not delivered. Fullscreen: `requestFullscreen` (consumes the activation, `TypeError` + `fullscreenerror` without one), `exitFullscreen`, `fullscreenElement`, `fullscreenEnabled`, `fullscreenchange` via `SDL_SetWindowFullscreen(FULLSCREEN_DESKTOP)` and a relayout on the size change. `visibilitychange` and `document.visibilityState`/`hidden` follow SDL's minimized/hidden and restored/shown. `structuredClone` already existed; it now throws `TypeError` without an argument and takes `{transfer}` (ArrayBuffers move, the source is detached, views stay on the moved buffer); golden `260_structured_clone`. New `--events` words: `keyup:NAME`, key names that are `KeyboardEvent.code` values, prefixes `alt+`, `ctrl+`, `repeat+`, `move:X,Y,DX,DY`, `click:X,Y,right|middle`, `release:X,Y,right|middle`, `window:hidden|shown|blur|leave`. Tests `window_input_codes` (`input_codes.html`), `window_pointer_lock` (`pointer_lock.html`; under SDL's dummy driver `SDL_SetRelativeMouseMode` returns 0, the DOM side is what is checked), `js_conformance`: `cargo test --release --test browser -- window_pointer_lock window_input_codes js_conformance` 4 passed; `-- window_` 41 passed, `window_webgl` needs ANGLE (not fetched here). Left: a `click` at release time after `pointerup`; `pointerenter`/`pointerover`/`pointerout` and `mouseenter`/`mouseover` from the window; `contextmenu` for the right button; `unadjustedMovement`; the `:fullscreen` pseudo-class and `::backdrop`; locks and fullscreen survive a navigation until Escape.

#### P11.11c: Pointer Lock on macOS
- goal:
  - `requestPointerLock()` locks the mouse in a real cocoa window, and `window_pointer_lock` passes on every host.
- files: `browser/ui/lock-pointer.aye`, `browser/ui/sdl.aye`, `browser/ui/window.aye` (one call)
- reference: `src/video/SDL_video.c` of SDL 2.32 (`SDL_GetCurrentVideoDriver`)
- verify: `cargo test --release --test browser -- window_pointer_lock window_input_codes > target/p11.11c.log 2>&1; echo $?` → 0 on a Mac with Homebrew SDL 2.32
- depends on: P11.11; time box: 45 min; model: sonnet
- status: done 2026-10-16. Cause: SDL 2.32 (Homebrew) has no relative mode in its `dummy` and `offscreen` drivers, so `SDL_SetRelativeMouseMode(1)` returns -1 ("No relative mode implementation available"); the CI's older SDL returned 0 there. The test sets `SDL_VIDEODRIVER=dummy`; the code was right. Fix: `lockPointer` in `ui/lock-pointer.aye` grants the lock when SDL refuses it under `dummy` or `offscreen` (no pointer exists there), and under any real driver still reports a refusal as `lockerror`. By hand with the cocoa driver (no `SDL_VIDEODRIVER`, `--events "click:50,70 move:70,90,13,-7 click:60,80"` on `pointer_lock.html`): `SDL_SetRelativeMouseMode(1)` returns 0, `pointerlockchange`, `locked true`, `pointermove m13,-7`, `mousemove 50,30 m13,-7`, the same log as the test. The `--events` motion is injected by the script, not by the OS mouse.

#### P11.12: the rest of WebGL 2
- goal:
  - Queries (`OCCLUSION`, `ANY_SAMPLES_PASSED`; results available only after a frame, as the spec requires); `fenceSync`, `clientWaitSync` (timeout clamped to 0 on the main thread), `getSyncParameter`.
  - Transform feedback.
  - `blitFramebuffer`, `readBuffer`, `readPixels` into a `PIXEL_PACK_BUFFER`, `getBufferSubData`, `copyBufferSubData`, `copyTexSubImage2D`/`3D`, `clearBuffer[fiuv]`, `invalidateFramebuffer`, `getIndexedParameter`, `getInternalformatParameter`.
- files: `browser/engine/webgl_api.aye`, `browser/tests/webgl/webgl2_rest.html`, `tests/browser.rs` (`webgl2_rest`)
- reference: `third_party/blink/renderer/modules/webgl/webgl2_rendering_context_base.cc`, `webgl_query.cc`, `webgl_sync.cc`
- verify: `cargo test --release --test browser -- webgl2_rest > target/p11.12.log 2>&1; echo $?` → 0
- depends on: P11.8; time box: 45 min; model: sonnet

#### P11.13: WebGL 2 extensions
- goal: `getSupportedExtensions` and `getExtension` for the WebGL 2 set of §3, each enabled with `glRequestExtensionANGLE` only when ANGLE lists it as requestable (`GL_REQUESTABLE_EXTENSIONS_ANGLE`):
  - `EXT_color_buffer_float`, `EXT_color_buffer_half_float`, `EXT_float_blend`
  - `EXT_texture_filter_anisotropic`, `OES_texture_float_linear`, `EXT_texture_norm16`
  - `KHR_parallel_shader_compile`, `WEBGL_lose_context`, `WEBGL_debug_renderer_info` (generic strings)
  - the compressed formats ANGLE has: `WEBGL_compressed_texture_s3tc`/`_s3tc_srgb`/`_etc`/`_astc`, `EXT_texture_compression_bptc`/`_rgtc`, with `compressedTexImage2D`/`3D` and the size checks of each extension spec.
- files: `browser/engine/webgl_api.aye`, `webgl_prelude.aye`, `browser/tests/webgl/extensions.html`, `tests/browser.rs` (`webgl_extensions`)
- reference: `third_party/blink/renderer/modules/webgl/` (one file per extension), `third_party/angle/include/GLES2/gl2ext_angle.h`
- verify: `cargo test --release --test browser -- webgl_extensions > target/p11.13.log 2>&1; echo $?` → 0: render to an `RGBA16F` FBO, anisotropy set and read back, an S3TC or ETC2 texture uploaded (whichever SwiftShader has), and `getExtension('NOPE')` is `null`
- depends on: P11.8; time box: 45 min; model: sonnet

#### P11.14: WebGL conformance runner and baseline
- goal: §7.
  - `browser/tools/webgl_cts.rev` pins KhronosGroup/WebGL. The suite is fetched and cached like WPT, not vendored.
  - `browser/tools/webgl_conformance.py run|check` serves the suite (the P10.1 static server) and runs each page of the first `conformance2/` subset in `dump --js` with `DWELLER_GL=swiftshader`. A prelude hook defines `window.webglTestHarness` (`reportResults`, `notifyFinished`). Each page has a 30 s timeout.
  - The script writes "WebGL 2: <dir>" rows into `docs/conformance.md`.
  - A weekly CI job on Ubuntu.
- files: `browser/tools/webgl_conformance.py` (new), `browser/tools/webgl_cts.rev`, `browser/engine/prelude.aye` (the hook behind a flag), `docs/conformance.md`, `.github/workflows/conformance.yml`
- reference: `third_party/webgl/src/sdk/tests/00_test_list.txt`, `conformance2/00_test_list.txt`, `js/js-test-pre.js` (`reportTestResultsToHarness`, `notifyFinishedToHarness`)
- verify: `python3 browser/tools/webgl_conformance.py run --subset first > target/p11.14.log 2>&1; echo $?` → 0 (the run completes); the baseline table committed
- depends on: P11.9 (and P10.1 for the server; if P10.1 has not landed, use `browser/tests/server.py`); time box: 45 min; model: sonnet

#### P11.15: context loss, many contexts, limits
- goal: §2 and §4.
  - Robustness reset check each frame. `webglcontextlost` / `webglcontextrestored` with `preventDefault`, and every call's lost-context return value.
  - `WEBGL_lose_context` `loseContext` / `restoreContext`.
  - At most 16 live contexts per page (the least recently used one is lost, with a console warning).
  - The canvas GC finalizer frees its context.
  - The limit clamps of §4 (texture size, drawing buffer size, shader source size).
- files: `browser/engine/webgl_api.aye`, `browser/tests/webgl/context_loss.html`, `tests/browser.rs` (`webgl_context_loss`)
- reference: `third_party/blink/renderer/modules/webgl/webgl_rendering_context_base.cc` (`LoseContextImpl`, `kMaxGLActiveContexts`), `webgl_context_group.cc`
- verify: `cargo test --release --test browser -- webgl_context_loss > target/p11.15.log 2>&1; echo $?` → 0; the conformance rows for `conformance2/context` not lower
- depends on: P11.13; time box: 45 min; model: sonnet

#### P11.16: WebAssembly runtime choice (measurement, no browser code)
- goal: decide the WASM engine with numbers (§6).
  - Copy `~/Documents/f1` to the scratchpad; never modify the user's repo.
  - Add a small `sim-bench` crate there: it steps the sim of `crates/sim-core` 20,000 times at 200 Hz on a fixed input, with no wasm-bindgen imports, and builds for `wasm32-wasip1`.
  - Measure the median step time natively and under: wasmtime (Cranelift), WAMR (fast-interp and, on x86-64, Fast JIT), wasm3. The machine is the user's Mac (arm64); add Ubuntu x64 if available.
  - Record the table and the choice in `docs/webgl.md` §6. The rule: the fastest runtime with a C API that runs on macOS arm64 and Linux x64 and meets ≤ 0.5 ms per step. Also record the library size and license.
- files: `docs/webgl.md` only
- reference: `v8/src/wasm/` (the semantics the JS API must keep: memory growth, traps)
- verify: the table in the PR, with the commands to rerun it
- depends on: none (any time); time box: 45 min; model: sonnet
- done 2026-10-15: wasmtime 49.0.3 with Cranelift. Numbers and reasons are in `docs/webgl.md` §6 (WebAssembly runtime). Median µs per step: native 9.0, wasmtime 29.1, node 22 (V8) 32.3, wasm3 362, WAMR fast-interp 500. wasm3 and WAMR also take 12 to 19 s to build the racing line, against 1.2 s in V8. Differences from the card: the bench wraps `sim_wasm::SimHandle` (as `main.ts` calls it) and is built for `wasm32-unknown-unknown` plus `wasm-opt -O`, as `wasm-pack` builds the shipped module; its five wasm-bindgen imports are never called and become traps. WAMR AOT and LLVM JIT were not measured: `wamrc` does not build against LLVM 23, and AOT does not fit wasm fetched at run time. There was no Ubuntu x64 machine. The harness and the commands to rerun are in the PR.

#### P11.17: the WebAssembly JS API
- goal: on wasmtime with Cranelift (P11.16; `docs/webgl.md` §6, WebAssembly runtime). Its C API (`wasm.h`, `wasmtime.h`) comes through `@cimport`. Dweller builds nothing: `browser/wasm/wasmtime.lock` pins the version (49.0.3 to start) and the SHA-256 of the upstream release assets `wasmtime-v<ver>-{aarch64-macos,x86_64-linux}-c-api.tar.xz`. `browser/wasm/fetch_wasmtime.sh` downloads and checks them into `target/wasmtime/<ver>/`; CI caches that directory, keyed by the lock. Dweller links `libwasmtime` with an rpath. Without the library, `WebAssembly` is undefined. The functions to implement:
  - `WebAssembly.Module`, `Instance`, `instantiate`, `instantiateStreaming` (needs `application/wasm`), `compile`, `validate`.
  - `Memory`: its `buffer` is an `ArrayBuffer` over the linear memory (P11.3, external store), detached on `grow`.
  - `Table`, `Global`, `CompileError`, `LinkError`, `RuntimeError`.
  - Imports call JS functions; exports are JS functions; traps throw `RuntimeError`.
  - wasm-bindgen glue (`sim_wasm.js`: `TextDecoder`, `FinalizationRegistry`, `externref` tables) works.
- files: `browser/wasm/` (new), `browser/engine/prelude.aye`, `browser/js/lib.aye` (external `ArrayBuffer` store), `tests/browser.rs` (`wasm_api`, `wasm_sim_bench`), `browser/tests/wasm/`
- reference: `v8/src/wasm/wasm-js.cc` (the JS API), `v8/src/wasm/wasm-objects.h`; the WPT `wasm/jsapi/` tests
- verify: `cargo test --release --test browser -- wasm_api > target/p11.17.log 2>&1; echo $?` → 0; `wasm_sim_bench` loads a copy of `sim_wasm_bg.wasm` with its glue and prints the step time, within 1.5x of P11.16's standalone number (29 µs per step on the user's Mac)
- depends on: P11.3, P11.16; time box: 45 min (split Memory/Table/Global into P11.17b if larger); model: opus
- partial 2026-10-15: the API is done; `wasm_sim_bench` and the wasm-bindgen glue check remain (they need the game's `sim_wasm_bg.wasm` and `sim_wasm.js`, P11.24's copy).
  - `browser/wasm/`: `wasmtime.lock` (49.0.3, both SHA-256), `fetch_wasmtime.sh` (into `target/wasmtime/<ver>/lib`, with a `wasmtime.pc` whose `Libs` carry the rpath per OS), the C headers vendored in `include/` so the browser builds without the library, and `wasm.aye`: `@linkOptional("wasmtime")` and `@cimport("wasmtime.h")`. libclang expands `wasm.h`'s `WASM_DECLARE_*` macros, so no declaration is written by hand (765 functions; only its inline helpers are skipped). Without the library, or with `--no-link wasmtime`, `WebAssembly` is undefined.
  - `engine/wasm_prelude.aye`: `Module` (`imports`, `exports`; `customSections` gives `[]`), `Instance`, `Memory`, `Table`, `Global`, the three errors, `validate`, `compile`, `instantiate` (both overloads), `compileStreaming` and `instantiateStreaming` (`application/wasm`, which `file:` now gives `.wasm`). Values: i32, i64 as BigInt, f32, f64, externref, funcref; multi-value results are arrays. A trap is a `RuntimeError`; an exception thrown by a JS import comes back through wasm as itself.
  - `Memory.buffer` is an `ArrayBuffer` whose store is external (`js.setExternalBytes`): it aliases the linear memory, the GC does not free it, `transfer()` throws. `grow`, and a `memory.grow` inside wasm (checked after each call), detach it.
  - `fetch` gives `arrayBuffer()` the raw bytes when decoding the body changed them (binary bodies were decoded as text).
  - `tests/browser.rs` `wasm_api` (`browser/tests/wasm/api.wat`: 46 checks) and `wasm_loop_bench` (`loop.rs`, built for `wasm32-unknown-unknown`): 20M steps take 31-34 ms in Dweller and 36-42 ms in node 22 on this Linux x64 machine. CI fetches wasmtime on both OSes (cached by the lock).
  - One store serves all pages and is freed with the last; modules and externs stay until then.

#### P11.18: dedicated Workers
- goal:
  - `new Worker(url, {type: 'classic'|'module'})`: one OS thread with its own `Interp` and event loop.
  - `postMessage` and `onmessage` both ways, with structured clone (P11.11) and `transfer` of `ArrayBuffer`s (P11.3); `importScripts`; `terminate`; `self.close`; `error` events.
  - `URL.createObjectURL` gets a real blob registry (`blob:` resolves in fetch and Worker), plus `revokeObjectURL`.
  - Audit the module-level state of `browser/js/` (1 `var`) and `browser/engine/` (15) first, and record what is thread-local.
  - The worker global has `fetch`, timers, `performance`, `WebAssembly`, `OffscreenCanvas` (2D) and `TextDecoder`.
- files: `browser/engine/workers.aye` (new), `browser/engine/prelude.aye`, `browser/engine/script.aye`, `tests/browser.rs` (`worker_basic`, `worker_module`, `worker_transfer`)
- reference: `third_party/blink/renderer/core/workers/` (`dedicated_worker.cc`, `worker_global_scope.cc`; algorithm only); the HTML spec "Workers"; the WPT `workers/` tests
- verify: `cargo test --release --test browser -- worker_ > target/p11.18.log 2>&1; echo $?` → 0, including a module worker that computes a 512x512 noise `Float32Array` and transfers it back (as the game's `cloudNoise.worker.ts` does)
- depends on: P11.3, P11.11; time box: 45 min (split the blob registry into P11.18b if needed); model: opus (threads)
- status: done 2026-10-16. The blob registry is in this card (no P11.18b).
  - Audit of module-level state: no `var` of `browser/js/` or `browser/engine/` is thread-local (Aye has no thread-locals), and a worker reaches none that changes. `js/gc.aye` `trace_mode` is set once before any thread starts and only read after. The 15 of `engine/` (`webgl_api` `size_buf`, `next_frame`; `frames` `next_page_id`; `geometry` `check_read`, `check_on`; `script` `scripting_on`, `real_clock`, `echo_console`, `write_nesting`; `contexts` `bc_table`, `bc_tab_host`; `canvas_api` `canvas_fixed`; `storage` `local_store`, `session_store`, `store_users`, `storage_saved_ms`; `sheets` `import_depth`; `audio_api` `next_audio_owner`) and the 2 new ones (`worker-api.aye` `page_links`, `next_worker_id`) belong to the main thread: a worker's `Interp` runs only `js` builtins and the natives of `engine/workers/`, never engine code. Outside these two modules, a worker would also reach `net` (`session`, `auth`, `loader`, through `net.request`) and `wasm` (`state`, `error_factory_root`). So workers load through `net.fetchLocal` (`file:`, `data:`; `Interp.local_loads_only` sends module loads there and skips the prefetch) and get no `WebAssembly`. `perf/telemetry.aye` is the one shared writer left: the interpreter notes unknown globals there, but only while telemetry is on (`DWELLER_TELEMETRY`), never in tests.
  - `browser/engine/workers/` (module `engine.workers`, one function per file): a `Link` per worker (a mutex, its own `General` allocator that is touched only under the mutex, an inbox and an outbox of `Message`s). `spawnWorker` starts a thread with a 1 GiB reserved stack. `runWorker` makes an `Interp`, installs the builtins, `__worker` natives, `wire.js` and `worker-scope.js`, runs the script (classic) or `runModuleScript` (module), then `serveWorker` turns its event loop: the inbox, then due timers, then microtasks and unhandled rejections, sleeping 1 ms when idle.
  - Messages: `wire.js` encodes the structured clone graph (the types of P11.11's `structuredClone`, cycles, BigInt, -0 and NaN) as JSON. The bytes of each `ArrayBuffer` are copied into the message, and the buffers of `transfer` are detached in the sender with `transfer()`. Views come back on their own buffer. A function or a symbol throws `DataCloneError`.
  - Page side (`engine/worker-api.aye`, `wk-*.aye`, `workers/worker-page.js`): `Worker` (an `EventTarget`; `postMessage` with an array or `{transfer}`, `terminate`, `message`, `messageerror`, and cancelable `error` events, an uncanceled one goes to `console.error`). `pumpWorkers` delivers in `runDueTimers` and `runNextTimer`. With no timer left, `dump` waits for busy workers (at most 60 s); in the window, `msUntilNextTimer` is at most 4 ms while a worker is busy. `destroyScript` stops and joins the page's workers.
  - The worker global: `self`, `location`, `postMessage`, `onmessage`, `addEventListener`, `close`, `importScripts`, timers, `performance.now`, `fetch` (`file:` and `data:`), `TextDecoder`/`TextEncoder`, `console` (forwarded to the page's console).
  - Blob registry: `URL.createObjectURL` keeps the blob under its URL, and `revokeObjectURL` drops it. `fetch` and `new Worker` resolve `blob:` URLs of the page; a revoked URL throws `SecurityError` from `new Worker`.
  - Not done: `WebAssembly` and `OffscreenCanvas` in workers, `http(s)` loads from workers (all need `net` and `wasm` to be thread-safe or a main-thread proxy), `blob:` URLs made inside a worker, `MessagePort` in `transfer`, and `terminate` of a worker inside a long synchronous loop (the join waits for the loop).
  - `tests/browser.rs` `worker_basic`, `worker_module`, `worker_transfer` (`browser/tests/workers/`): the module worker imports `noise.js`, computes 512x512 value noise in a `Float32Array` and transfers it back; the page checks samples against the same module (equal) and the length (262144 floats, 1 MiB).

#### P11.19: Web Audio core
- goal: §6 Web Audio.
  - `AudioContext` (`state`, `resume`/`suspend` with user activation, `currentTime`, `sampleRate`, `destination`).
  - Output through `SDL_OpenAudioDevice` (float32 stereo 48 kHz), fed from a ring buffer that the window thread fills in 128-frame quanta about 30 ms ahead.
  - `AudioBuffer`, `createBuffer`, `decodeAudioData` (WAV: PCM 8, 16 and 24 bit, float32).
  - `AudioBufferSourceNode` (`start`/`stop` with `when`, `loop`, `loopStart`/`loopEnd`, `playbackRate`, `detune`, `ended`); `GainNode`.
  - `AudioParam` (`value`, `setValueAtTime`, `linearRampToValueAtTime`, `exponentialRampToValueAtTime`, `setTargetAtTime`, `cancelScheduledValues`); `connect`/`disconnect`.
  - Headless runs use an offline sink, so tests compare samples: `OfflineAudioContext` with the same graph code.
- files: `browser/audio/` (new: graph, nodes, WAV decoder), `browser/ui/sdl.aye` (audio), `browser/engine/audio_api.aye` (new), `tests/browser.rs` (`webaudio_core`)
- reference: `third_party/blink/renderer/modules/webaudio/` (`audio_buffer_source_handler.cc`, `audio_param_timeline.cc`; algorithm only); the WPT `webaudio/` tests
- verify: `cargo test --release --test browser -- webaudio_core > target/p11.19.log 2>&1; echo $?` → 0. An `OfflineAudioContext` renders a looped buffer through a gain ramp; samples match the expected values within 1e-4.
- depends on: P11.3; time box: 45 min (split AudioParam automation into P11.19b if needed); model: opus
- done 2026-10-16: all of it, AudioParam automation included (no P11.19b).
  - `browser/audio/` (module `audio`): `audio.aye` is the graph. Contexts and nodes are indices; a node renders once per 128-frame quantum, pulled from the destination. Buses have 1 or 2 channels and mix by the speakers rules. AudioParam timelines: setValue, linear and exponential ramps, setTarget, cancelScheduledValues; events that ended collapse into one SetValue, so long sessions stay small. `AudioBufferSourceNode`: sample-accurate `start`/`stop`, `offset`, `duration`, `loop` with `loopStart`/`loopEnd`, k-rate `playbackRate` and `detune`, linear interpolation, `ended`. `wav.aye`: PCM 8, 16, 24 and 32 bit, float32, WAVE_FORMAT_EXTENSIBLE. `out.aye`: the SDL device.
  - Output: `SDL_OpenAudioDevice` (float32 stereo at the context's rate, 48 kHz by default) fed with `SDL_QueueAudio` from the window thread, 30 ms ahead. SDL's queue is the ring buffer, so no callback and no Aye code run on SDL's audio thread. The SDL audio externs are in `out.aye`, not `ui/sdl.aye`, because the engine cannot import `ui`. The device opens only on the window clock: `dump` and CI never open one, and a running context there renders by the page clock and drops the samples. While a context plays, `msUntilNextTimer` is at most 10 ms, so the window loop pumps it. No edit to `ui/window.aye`.
  - `engine/audio_api.aye`: the `__audioHost` natives and `AUDIO_PRELUDE` (BaseAudioContext, AudioContext, OfflineAudioContext with `oncomplete`, AudioBuffer, AudioParam, AudioNode, AudioDestinationNode, GainNode, AudioScheduledSourceNode, AudioBufferSourceNode). Buffers are copied when a source starts, as the spec says ("acquire the content"). `decodeAudioData` resamples linearly to the context's rate. Autoplay: an `AudioContext` starts `running` only after user activation (`keydown`, `mousedown`, `pointerdown`, `pointerup`, `touchend`, `click` through `engine.dispatchEvent`). Otherwise it starts `suspended`. `resume()` is not gated on activation.
  - Not done: connecting a node to an `AudioParam` (it is ignored), `setValueCurveAtTime` and `cancelAndHoldAtTime`, channel counts above 2 (extra channels render silent), and `channelCount`/`channelCountMode` (fixed to max/speakers).
  - `tests/browser.rs` `webaudio_core` (`browser/tests/audio/core.html`, `core.txt`): the looped buffer through linear, exponential and setTarget gain automation matches the test's own formula within 1e-4 over 1600 frames. Also covered: loop points, rate and detune, interpolation, up-mix, disconnect, the errors, WAV decoding and resampling, and the realtime state changes. Checked by hand on the user's Mac: in the window, coreaudiod held an `audio-out BuiltInSpeakerDevice` assertion for the process, and `currentTime` kept pace with the wall clock.

#### P11.20: Web Audio effects
- goal: `BiquadFilterNode` (all 8 types, `getFrequencyResponse`), `StereoPannerNode`, `DynamicsCompressorNode` (`reduction`), `ConvolverNode` (`normalize`, partitioned FFT convolution), as the spec defines them. These are the nodes the game uses.
- files: `browser/audio/`, `browser/engine/audio_api.aye`, `tests/browser.rs` (`webaudio_effects`)
- reference: the Web Audio spec (biquad coefficients from the Audio EQ Cookbook); `third_party/blink/renderer/platform/audio/biquad.cc`, `dynamics_compressor.cc`, `reverb_convolver.cc` (algorithm only)
- verify: `cargo test --release --test browser -- webaudio_effects > target/p11.20.log 2>&1; echo $?` → 0: offline renders match reference sample files made with Chrome's `OfflineAudioContext`, within 1e-3
- depends on: P11.19; time box: 45 min; model: sonnet
- done 2026-10-16: all four nodes.
  - `browser/audio/fx.aye` (new, module `audio`): `BiquadFilterNode` (the 8 types with Blink's `Biquad::Set*Params` edge cases; Q in dB for lowpass and highpass; a-rate: coefficients per frame when any of the four params has events, else once per quantum; `getFrequencyResponse`, NaN outside 0..Nyquist), `StereoPannerNode` (equal power, mono and stereo laws, a-rate), `DynamicsCompressorNode` (a port of `dynamics_compressor.cc`: 6 ms look-ahead pre-delay, detector, adaptive release polynomial, makeup gain, warp, metering; k-rate params clamped to their nominal ranges; `reduction`), `ConvolverNode` (uniformly partitioned overlap-save FFT, 128-frame partitions and a 256-point radix-2 FFT in f64, one convolver per output path as `reverb.cc`: 1, 2 or 4 channel responses, true stereo; Blink's normalization: RMS power, -58 dB calibration, 44100/rate, 0.5 for 4 channels; `normalize` is read when `buffer` is set). `audio.aye`: node kinds, the effect state (`Node.fx`), the post-mix step in `renderNode`. `engine/audio_api.aye`: the natives and the four JS classes, `createBiquadFilter`/`createStereoPanner`/`createDynamicsCompressor`/`createConvolver`.
  - Test `webaudio_effects` (`browser/tests/audio/effects.html`): 60 reference lines (`effects.ref`, 8 biquad types plus their responses, a frequency sweep, stereo, 7 pan cases, 2 compressor settings and `reduction`, 7 convolver cases) were rendered by Chrome's `OfflineAudioContext` through `agent-browser` (the page's own output; `browser/tests/audio/README.md`); all match within 1e-3 (the compressor and filters within 1e-4). The non-numeric lines (defaults, ranges, automation rates, errors) are Chrome's too (`effects.txt`). One Chrome deviation: with stereo input and a mono response its right channel is off by up to 1.3e-3 from the plain convolution, so that channel is checked against the direct sum, not Chrome (README). Timing, debug build: 2 s rendered through a 2 s stereo response in 1.6 s.
  - Not done: connecting a node to an `AudioParam` (still ignored), `channelCount`/`channelCountMode`/`channelInterpretation` setters (fixed to each node's default), a compressor's `automationRate` guard, silence skipping for nodes whose tail ended (they render on), 3-channel and larger inputs (extra channels render silent, as in P11.19).

#### P11.21: the game's frame budget
- goal: measure where a frame of the game goes before the acceptance run.
  - Build a copy of the game (as P11.24) and load its main scene in the window at 1280x720.
  - Record per frame: JS in rAF (split three's `render` from the game's code), `webgl_readback`, `ui_draw`, GPU wait, WASM step time. Use 300 frames, p50 and p95.
  - If JS is over 8 ms p50, write the follow-up into the plan: the bytecode compiler (`browser/js/PERF.md`, P4.30) or targeted builtins (`Float32Array` math, property access in three's `Matrix4`).
- files: `browser/engine/PERF.md` (a "WebGL game" section), `docs/plans/browser.md`
- reference: `browser/js/PERF.md`
- verify: the table in `PERF.md`, with the command to rerun it
- depends on: P11.9, P11.10, P11.17, P11.18, P11.19; time box: 45 min; model: sonnet

#### P11.22: GPU compositing, part 1: the window presents through ANGLE
- goal: step 2 of §5, first half.
  - The SDL window gets an EGL window surface: `SDL_WINDOW_METAL` + `SDL_Metal_GetLayer` on macOS; the X11 window or `wl_egl_window` on Linux.
  - `eglSwapInterval(1)`.
  - `pagedraw.aye`'s pixels become 256x256 texture tiles, and only damaged tiles are uploaded. The chrome is its own texture.
  - `ui/compositor.aye` draws quads with one shader. `SDL_Renderer` stays as the fallback (`DWELLER_GL=off`, no ANGLE).
  - `DWELLER_DAMAGE_CHECK` compares the composited frame, read back, with a whole CPU frame.
- files: `browser/ui/compositor.aye` (new), `browser/ui/window.aye`, `browser/ui/pagedraw.aye`, `browser/ui/chrome.aye`, `browser/ui/sdl.aye`
- reference: `docs/how_cc_works.md` (tiles, damage, idea only); `ui/gl/gl_surface_egl.cc` (window surfaces); the sample windows in `third_party/angle/util/`
- verify: `DWELLER_DAMAGE_CHECK=1 cargo test --release --test browser -- window_ > target/p11.22.log 2>&1; echo $?` → 0 on both OSes; `window_perf.py scroll https://react.dev` draw p95 not higher than with `DWELLER_GL=off`
- depends on: P11.7, P11.10; not in parallel with P4 cards on `ui/`; time box: 45 min (split the tiles from the present path if larger); model: opus

#### P11.23: GPU compositing, part 2: canvases, layers and scrolling on the GPU
- goal:
  - WebGL canvases are drawn from their color texture through an `EGLImage` (`EGL_KHR_gl_texture_2D_image`), with two textures swapped for `preserveDrawingBuffer: false`. There is no readback in the window (screenshots still read back).
  - Promoted layers (P4.56) are textures with transform and opacity in the quad shader.
  - Scrolling moves tile quads, and only exposed tiles are rasterized.
- files: `browser/ui/compositor.aye`, `browser/ui/pagedraw.aye`, `browser/engine/webgl_api.aye`
- reference: `docs/how_cc_works.md` (TextureLayer, scrolling on the compositor, idea only); `third_party/blink/renderer/platform/graphics/gpu/drawing_buffer.cc` (buffer swap)
- verify: `DWELLER_DAMAGE_CHECK=1 cargo test --release --test browser -- window_webgl window_transform_composite window_damage > target/p11.23.log 2>&1; echo $?` → 0; `webgl_readback` absent from the window's timing; `window_perf.py scroll` p95 on react.dev and github.com lower than after P11.22 (recorded)
- depends on: P11.22; time box: 45 min; model: opus

#### P11.24a: first run of the user's game (defects become cards)
- goal: run the game now, before every P11 card is done, and turn what breaks into cards (P11.24b–P11.24f). No fixes here beyond one-line ones.
  - Copy `~/Documents/f1/web` without `node_modules` (to `target/f1web`), `pnpm install --frozen-lockfile && pnpm build`.
  - `browser/tools/game_check.py GAME`: serves `dist/` (`.wasm` as `application/wasm`), runs `dump --text --js` and `screenshot` (1280x720) on each of the six HTML entries, and runs `index.html` in a headless window at 1280x720 for 600 frames with key events (`DWELLER_CONSOLE=1`, `DWELLER_TIMING=1`). It groups the console errors by cause and prints the window and WebGL frame rates. Screenshots and logs go to `target/game/`.
- files: `browser/tools/game_check.py` (new), `browser/ui/window.aye` (`DWELLER_CONSOLE=1` keeps the console on stderr in `--frames` runs), `docs/plans/browser.md`
- reference: P11.24
- verify: `python3 browser/tools/game_check.py <copy> > target/p11.24a.log 2>&1; echo $?` → 0 with the summary printed and a PNG per page in `target/game/`
- depends on: none; time box: 45 min; model: opus
- status: done 2026-10-16. The build runs (pnpm 10, `tsc && vite build`, 42 MB `dist/`). With libwasmtime fetched (`browser/wasm/fetch_wasmtime.sh`; without it `WebAssembly is not defined`), stock binary: `duel.html` 0 errors (0.6 s); `object-lab.html` 0 errors (3.6 s, the kerb scene renders); `garage.html` 0 errors but 30–40 s to load (the car renders in WebGL; `/api/livery` 404 is tolerated); `recorder.html` and `tunnel.html` 1 error each, `Uncaught ReferenceError: Worker is not defined` (P11.18, in flight); `index.html` 1 error, `Script terminated: it ran too long`: the startup task ends after 2 s and the page stays at "loading sim…" (window 600 frames at 85 fps of HUD only, 1 WebGL frame). With the task budget raised by hand (diagnosis only, not committed): the startup task takes 104 s, then the game draws at 5.6 WebGL fps (rAF `ui_timers` p50 139 ms, `paint` 27.5 ms, `ui_relayout` 9.9 ms), and one fetch fails with `Failed to fetch: timed out` after the blocked task; three warns that `KHR_parallel_shader_compile` is absent (harmless). Audio and Pointer Lock were not reached (the keys arrived during the startup task). New cards: P11.24b (task budget and fetch timeout in the window), P11.24c (HUD paint/relayout per frame), P11.24d (garage load time), P11.24e (startup JS in the tree walker); per-frame JS stays with P11.21, Workers with P11.18.

#### P11.24b: the window does not end a long task of the page
- goal: the game's startup is one task of about 100 s CPU today (P11.24a); the window ends it after `TASK_MS` (2 s), so `index.html` stays at "loading sim…".
  - Chrome never ends a page's task; its hang monitor offers "wait or exit" after about 30 s and the task goes on.
  - In the window (real clock), raise the task budget to that of Chrome's hang monitor and keep the 2 s budget only for `dump` and the agent (virtual clock), or make the window's budget a prompt later. A page that ends a task early must still log it.
  - The fetch of a blocked page must not time out because the loop was blocked (with the budget raised by hand, the game logged `Uncaught (in promise) TypeError: Failed to fetch: timed out` after its 104 s task): count the request's timeout from the network side, or deliver a response that already arrived.
- files: `browser/engine/script.aye` (`TASK_MS`, `startBudget`), the fetch timeout in `browser/net/`, `tests/browser.rs` (`window_long_task`)
- reference: `content/browser/renderer_host/render_widget_host_impl.cc` (hang monitor timeout), `chrome/browser/ui/hung_renderer/`; `third_party/blink/renderer/core/fetch/fetch_manager.cc` (no fetch timeout in Blink)
- verify: `cargo test --release --test browser -- window_long_task > target/p11.24b.log 2>&1; echo $?` → 0 (a page whose script runs 5 s of CPU after starting a fetch finishes the script and gets the fetch's body in the window), and `game_check.py` with the stock binary shows WebGL frames for `index.html`
- status: done 2026-10-16. The window has no task budget (CTO decision: Chrome never ends a task; `enter` starts none when `Script.real`, microtask checkpoints are unbounded too); a task over 10 s shows "A page task ran for N s" on the status line when it ends (`engine.noteLongTask`/`takeLongTask`, `ui.showLongTask`). `dump`, `screenshot` and the agent keep their step budgets. The fetch: a loop pass that took over 5 s lengthens the connect and total timeouts of every running transfer by that time (`net.extendForStall`), so a response that came in during a blocked task is delivered, not timed out. `window_long_task` (4 s script finishes, typed input after it lands; `dump` of an endless script still terminates). Not run: `game_check.py` (f1 paused by the user); the stall extension has no test (a 30 s stall is too slow).
- depends on: none; time box: 45 min; model: opus

#### P11.24c: the HUD's frame cost in the window
- goal: with the game running (large budget), every window frame spends `paint` 27.5 ms and `ui_relayout` 9.9 ms (p50), so the window cannot reach 50 fps even with no JS. The HUD changes text every frame over a full-window WebGL canvas.
  - Measure which boxes the HUD dirties per frame (`ui_damage_px`, `layout_dirty`) and make the frame paint and lay out only them: a text change in a fixed-size box must not relayout the document or repaint the canvas under it.
  - Target: `paint` ≤ 4 ms and `ui_relayout` ≤ 2 ms p50 on the game page; GPU compositing (P11.22, P11.23) may own the rest.
- files: `browser/ui/window.aye`, `browser/paint/`, `browser/layout/`, `browser/engine/PERF.md`
- reference: `third_party/blink/renderer/core/paint/paint_invalidator.cc`, `third_party/blink/renderer/core/layout/layout_object.cc` (`SetNeedsLayout`, relayout boundaries), algorithm only
- verify: `python3 browser/tools/game_check.py <copy> --binary <large-budget build>` and the `paint`/`ui_relayout` p50 of `target/game/index.window.log`; a reduced page `browser/tests/perf/hud.html` (a full-window canvas plus a HUD whose text changes every rAF) in `tests/browser.rs` (`window_hud_frame`)
- depends on: P11.24b for the game run (the reduced page does not); time box: 45 min; model: opus

#### P11.24d: the garage page's 33–40 s load
- goal: `dump --js garage.html` takes 33–40 s before it prints (no error; the car renders). Find where the time goes (GLB parse in `GLTFLoader`, texture decode, shader compile, or JS) and fix the top item or write it as a card.
- files: `browser/engine/PERF.md` (a "garage load" row), the fix's files
- reference: `browser/js/PERF.md`; three's `GLTFLoader` in `node_modules/three/examples/jsm/loaders/` of the copy
- verify: `DWELLER_TIMING=1 target/dweller dump --text --js <server>/garage.html` phase totals before and after, in the PR; the load under 10 s, or the follow-up card written
- depends on: none; time box: 45 min; model: sonnet

#### P11.24e: the game's startup JS (about 100 s CPU in the tree walker)
- goal: `index.html`'s startup task (track, terrain with 287,263 grass clumps in 995 chunks, pit complex, 867 ad slots, then three's material precompile) takes about 100 s in the tree-walking interpreter (`Interp_eval`, `evalFastExpr`, `Heap_findProp` top a `sample` profile); V8 does it in a few seconds at most.
  - Profile with `DWELLER_JS_VM=1` and `DWELLER_JS_VM_STATS=1`; record the fallback reasons and the hot functions of the game's bundle.
  - Write the result into the Phase F cards (P4.63–P4.78) as the game's numbers, and pick targeted builtins if one dominates (typed-array fill, `Math`, array sort).
  - Target: startup ≤ 10 s; the per-frame JS (rAF `ui_timers` p50 139 ms, 5.6 WebGL fps) belongs to P11.21.
- files: `browser/js/PERF.md` (a "game startup" section), `docs/plans/browser.md`
- reference: `browser/js/BYTECODE.md` section 7 (V8 files), `src/interpreter/` of V8
- verify: `browser/js/PERF.md` has the table and the command to rerun it; the startup time with the VM on and off
- depends on: P11.24b (to run the startup to its end); time box: 45 min; model: opus

#### P11.24: acceptance: the user's game in Dweller
- goal:
  - Copy `~/Documents/f1/web` to the scratchpad without `node_modules`, then `pnpm install --frozen-lockfile && pnpm build` (`tsc && vite build`; the six HTML entries of `vite.config.ts` go into `dist/`). Never modify the user's repo.
  - Serve `dist/` with a static server that sends `application/wasm` (`browser/tests/server.py` or `python3 -m http.server`). The `/api/*` routes exist only under `vite dev`; the game must tolerate their 404, and if it does not, note it for the user.
  - Open `index.html` in the window at 1280x720. A scripted run (agent protocol plus `--events`) starts a session and drives for 30 s with keys.
  - Acceptance: no JS errors; median frame rate at or above 50 fps; the car moves (the wasm sim steps; the screen changes); audio plays (the SDL device is open and samples are non-zero); Pointer Lock works in the garage page.
  - Screenshots go side by side with Chrome's for the user to judge.
- files: `browser/tools/game_check.py` (new; the path to the game comes from an argument, never from a committed path), `docs/plans/browser.md` (results)
- reference: the cards above
- verify: `python3 browser/tools/game_check.py ~/Documents/f1/web > target/p11.24.log 2>&1; echo $?` → 0, and the user's judgement of the screenshots
- depends on: P11.9–P11.23; time box: 45 min, and defects become cards; model: opus

**If the user's game needs it** (the game's source does not use these
today; main checks again before a card starts):

- **P11.25 OffscreenCanvas in workers and WebGL in OffscreenCanvas.**
  - Today: main-thread 2D only. The game uses OffscreenCanvas only in the recorder page, with insertable streams.
  - The card: `transferControlToOffscreen`, `OffscreenCanvas` 2D and `webgl2` in a Worker (P11.18), and `transferToImageBitmap` to the main thread.
  - reference: `third_party/blink/renderer/core/offscreencanvas/`.
  - verify: `worker_offscreen` test.
- **P11.26 KTX2 / Basis compressed textures.**
  - The game has no `KTX2Loader` today; its textures are WebP.
  - If added, three's `KTX2Loader` runs the Basis transcoder as WASM in a Worker (P11.17, P11.18) and needs the compressed-format extensions (P11.13).
  - The card runs three's KTX2 example on a vendored `.ktx2` and checks that the transcoder picks ASTC or ETC2 on Apple Silicon (Metal) and BC7 or S3TC on Linux.
  - verify: `webgl_ktx2` screenshot test.
- **P11.27 Gamepad API.**
  - `navigator.getGamepads`, `gamepadconnected` and `gamepaddisconnected` from `SDL_GameController`, using the standard mapping.
  - reference: `third_party/blink/renderer/modules/gamepad/`.
  - verify: `window_gamepad` with SDL's virtual joystick (`SDL_JoystickAttachVirtual`).
- **P11.28 Video textures and `requestVideoFrameCallback`.**
  - These need video decoding, which Dweller does not have (a separate design: platform decoders or a C library).
  - After that: `texImage2D(video)` and `requestVideoFrameCallback`.

**For other sites (not the game):**

#### P11.29: WebGL 1
- goal:
  - `getContext('webgl')` and `'experimental-webgl'`: an ES 2.0 WebGL compatibility context (ANGLE applies `SH_WEBGL_SPEC`, with its loop and index rules), sharing all of P11.4–P11.7's code.
  - The WebGL 1 extension set of §3: `OES_vertex_array_object`, `ANGLE_instanced_arrays`, `WEBGL_depth_texture`, `OES_texture_float`, `OES_texture_half_float`, `OES_element_index_uint`, `OES_standard_derivatives`, `EXT_shader_texture_lod`, `EXT_frag_depth`, `EXT_blend_minmax`, `WEBGL_draw_buffers`, `EXT_sRGB`, plus the shared ones.
  - Conformance `conformance/` rows.
- files: `browser/engine/webgl_api.aye`, `webgl_prelude.aye`, `tests/browser.rs` (`webgl1_basic`)
- reference: `third_party/blink/renderer/modules/webgl/webgl_rendering_context.cc`, the extension files
- verify: `cargo test --release --test browser -- webgl1_basic > target/p11.29.log 2>&1; echo $?` → 0; `webgl_conformance.py run --suite 1` baseline committed
- depends on: P11.13; time box: 45 min; model: sonnet

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
- status: P10.1a done 2026-10-16: the browser side. `dweller wpt FILE|URL [--timeout S]` (`browser/main.aye`) loads the page with scripts as `dump --js` does, runs the event loop (`engine.runUntilConsole`, `browser/engine/script.aye`) until `browser/tools/wpt/testharnessreport.js` logs `DWELLER_WPT {json}` or S s (default 10) of the page's virtual time pass, then calls the harness's `timeout()` (the report sets `explicit_timeout`). It prints one line on stdout, `{"status":"OK|ERROR|TIMEOUT|CRASH","message":null|"...","subtests":[{"name":"...","status":"PASS|FAIL|TIMEOUT|NOTRUN","message":null|"..."}]}` (PRECONDITION_FAILED passes through as testharness names it); exit 0, or 1 with CRASH when the page does not load; TIMEOUT with `[]` subtests when no report came. Uncaught exceptions now fire an `ErrorEvent` at the window (`fireErrorEvent`), so testharness reports harness ERROR. `browser/tests/wpt/`: `resources/testharness.js` (WPT e58225ab, BSD `LICENSE.md`) and fixtures pass, fail, error, timeout, slow (`--timeout 60`), `sum.any.js` with a hand-written wrapper; they load the report by the relative path `../../tools/wpt/testharnessreport.js` over file URLs. `tests/browser.rs` `wpt_runner` checks each line exactly. P10.1b still has to do `browser/tools/wpt.py` (fetch at `wpt.rev`, serve with `.headers`, generated `.any.js`/`.window.js` wrappers and `/resources/testharnessreport.js` mapped to ours, run with `-j`, `timeout=long` → `--timeout 60`, CRASH for a process that exits without a line), `wpt.rev`, `wpt_areas.json`, and the second verify (`wpt.py run dom/nodes`).
- status: P10.1b done 2026-10-16: `browser/tools/wpt.py` (standard library only). `fetch` does a sparse checkout (cone, blob-less, depth 1) of `wpt.rev` (e58225ab, WPT master that day) into `target/wpt` with the dirs of `wpt_areas.json` plus `resources/`, `common/`, `fonts/`. `serve` is a static server on 127.0.0.1 with `.headers` sidecars, generated `X.any.html` / `X.window.html` pages for `.any.js` / `.window.js` (their `// META: script=` lines, `timeout=long`, `self.GLOBAL`; `.any.js` with a `global=` that excludes window is skipped), and `testharnessreport.js` for any `resources/` or `tools/wpt/` request. `run AREA|PATH... [-j N] [--wpt-root DIR] [--results-dir DIR] [--dweller BIN]` discovers testharness tests (skips reftests, `-manual`, `-ref`, `.https.`, workers, `.sub.`; `resources/` and `support/` dirs), runs `dweller wpt URL --timeout 10|60` per test (CRASH for a process without a result line; a wall-time cap), prints each file's subtests passed/total and status, writes `target/wpt-results/<area>.json`, and ends with the area line (files, skipped, passed/total, score, threshold). A file with no subtests counts as one failure. `wpt_areas.json` has dom/nodes, dom/events, html/semantics/forms, css/cssom, fetch/api/basic, xhr, url, encoding, webaudio (threshold 0 until P10.4's baseline). `tests/browser.rs` `wpt_py_serve` runs `wpt.py run` on the fixture dir. Scores at e58225ab: dom/nodes 4418/9378 (47.1%, 386 files, no CRASH); url 4850/9998 (48.5%, 34 files); dom/events 401/835 (48.0%, 178 files). Not run: html/semantics/forms, css/cssom, fetch/api/basic, xhr, encoding, webaudio. Left: no variants (`// META: variant=`) or `.sub.` substitution, no wptserve `.py` handlers (fetch/xhr tests that need them fail until P10.5), no https, workers or reftests (P10.2); `check` and the table are P10.4.

#### P10.2: reftest and crashtest runner
- goal: reftests compare the test's screenshot with its reference's; crashtests pass when the page loads without a crash or timeout.
- design: `wpt.py run` detects `<link rel=match|mismatch href=...>` (a reference's own reference followed one level), `<meta name=fuzzy content=...>` (maxDifference, totalPixels), the `reftest-wait` class (new `dweller screenshot --wait-class reftest-wait`, at most 5 s), and files under `crashtests/` or named `*-crash.*`. Screenshots at 800×600 (WPT's reftest size) with `--font fixed` off; the Ahem font from `target/wpt/fonts/` is a system family through a new `DWELLER_FONT_DIRS` (WPT CSS tests name `font-family: Ahem` without `@font-face`). Pixel comparison in the tool (a small PNG decoder over `zlib`).
- files: `browser/tools/wpt.py`, `browser/main.aye` (`screenshot --wait-class`), `browser/font/` (`DWELLER_FONT_DIRS`), `browser/tests/wpt/` (fixtures: match, mismatch, fuzzy, reftest-wait, crashtest), `tests/browser.rs` (`wpt_reftest`)
- verify: `cargo test --release --test browser wpt_reftest > target/p10.2.log 2>&1; echo $?` → 0; `python3 browser/tools/wpt.py run css/css-flexbox -j 8 > target/wpt-flex.log 2>&1; tail -3 target/wpt-flex.log` prints the score
- depends on: P10.1
- time box: 90 min; model: sonnet
- status: done 2026-10-16. `wpt.py run` now sorts each HTML page into `testharness`, `reftest` (`<link rel=match|mismatch>`) or `crashtest` (`crashtests/` dir or `*-crash.*`); `browser/tools/wpt_reftest.py` screenshots test and reference with `dweller screenshot` at 800×600 (system fonts, `DWELLER_FONT_DIRS=<root>/fonts`), follows a reference's own match/mismatch links one level, honours `<meta name=fuzzy>` (the upper bounds of `maxDifference` and `totalPixels`; a `ref.html:` prefix is ignored) and passes a crashtest when the screenshot exits 0 within 60 s. Identical PNG bytes short-circuit; otherwise `wpt_png.py` (zlib, no dependencies) decodes both and counts differing pixels. A reftest is one subtest (`reftest`); a crash, a timeout or a size mismatch fails it. `DWELLER_FONT_DIRS` (colon-separated; `font.addEnvFonts`, `browser/font/add-*.aye`) registers each `Name.ttf|otf` of those directories as the web face family `Name` (weights 100–900) for `screenshot`. `screenshot --wait-class NAME` only raises the page's virtual-time budget from 1 s to 5 s (timers run until idle, so a class removed by a timer is seen; a class removed by an event that never comes is not waited for). `css/css-flexbox` is an area of `wpt_areas.json` (so `fetch` gets it). Fixtures in `browser/tests/wpt-reftest/` (Ahem, mismatch, fuzzy pass and fail, reftest-wait, two crashtests, a failing match); `tests/browser.rs` `wpt_reftest`. Score at e58225ab: `css/css-flexbox` 640/1691 (37.8%; 1422 files, 469 skipped, 1198 OK, 180 TIMEOUT, 44 ERROR, no CRASH; 47 s at -j 8). Left: `-ref` pages that use `reftest-wait` on the reference only, `<link rel=match>` chains of more than two levels, `reftest-wait` events that need real time, print reftests, the `DWELLER_FONT_DIRS` weights (one face per family).

#### P10.3: test262 runner
- goal: test262 runs through the existing `dweller js` runner with the harness includes.
- design: `browser/tools/test262.py` (standard library only): `fetch` (at `test262.rev` into `target/test262`); `run [PATH...] [-j N]`: reads each test's front matter (`includes`, `flags`: `onlyStrict`, `noStrict`, `module`, `raw`, `async`; `negative`: phase and type; `features`), skips features in `test262_skip.txt`, writes a temporary file of `harness/assert.js`, `sta.js`, `doneprintHandle.js` (async), the includes and the test (with `"use strict";` for the strict run; `.mjs` for modules), runs `dweller js --test262 FILE` with a 10 s timeout. Pass: exit 0 with no uncaught error; async: `Test262:AsyncTestComplete` printed; negative: the expected error type, at parse time for `phase: parse` (nothing printed before it). `--test262` adds the host hooks: `print`, and `$262` with `global`, `evalScript`, `gc`, `detachArrayBuffer` (`createRealm` when P9.8's realms exist; `agent` stays skipped with `Atomics`). Results to `target/test262-results/<dir>.json`.
- files: `browser/tools/test262.py`, `browser/tools/test262.rev`, `browser/tools/test262_skip.txt`, `browser/main.aye` and `browser/js/` (`--test262` host hooks), `browser/tests/test262/` (vendored harness files with license; fixtures: pass, fail, negative parse, async, strict only, module), `tests/browser.rs` (`test262_runner`)
- verify: `cargo test --release --test browser -- test262_runner js_command_line > target/p10.3.log 2>&1; echo $?` → 0; `python3 browser/tools/test262.py fetch && python3 browser/tools/test262.py run test/built-ins/Array -j 8 > target/t262-array.log 2>&1; tail -3 target/t262-array.log` prints the score
- depends on: nothing (parallel with P10.1)
- time box: 90 min; model: sonnet
- status: done 2026-10-16. `dweller js --test262 FILE` (js.aye `runTest262File`) adds `print` and `$262` (`global`, `evalScript` over `Interp.evalScript`, `gc` = `collectGarbage`, `detachArrayBuffer` = the engine's `bufDetach`); built by a JS prelude over four natives on a hidden global, so a script sees neither the natives nor `$262` without the flag. `browser/tools/test262.py`: `fetch` (shallow `git fetch` of `test262.rev` = d53ebc8b19 into `target/test262`), `run [PATH...] [-j N] [--root] [--dweller] [--timeout] [--skip] [--out] [--vm]` with the front matter (includes, flags `onlyStrict`/`noStrict`/`module`/`raw`/`async`, `negative` phase and type, features), sloppy and strict runs (both must pass), a module's temp `.mjs` next to its test so imports resolve, 10 s timeout, a test is skipped for a feature in `test262_skip.txt` or when its source uses `$262.createRealm`/`agent`/`IsHTMLDDA`; results to `target/test262-results/<path>.json` (test → status, reason), one score line per PATH. Vendored in `browser/tests/test262` (assert, sta, doneprintHandle, compareArray, propertyHelper, the BSD LICENSE) with 12 fixtures; `test262_runner` checks the hooks and the fixture verdicts (8 of 11 pass, 3 fail by design, 1 skipped). Scores at d53ebc8b19: `test/built-ins/Array` 2162/3067 (70.5%, 16 skipped, 93 s at -j 8); `test/language/expressions` 7014/10697 (65.6%, 405 skipped, 131 s). Found on the way: assignment to an undeclared name in strict code does not throw `ReferenceError`. Left: `$262.createRealm` (P9.8), `$262.agent` and Atomics, `IsHTMLDDA`; `--vm` only sets `DWELLER_JS_VM=1` (P4.69's comparison is still to run); `language/` and `annexB` totals, the skip list's size, and the failure clusters of both runs are for P10.4's baseline.

#### P10.4: the conformance table
- goal: a per-area pass-rate table committed to the repo and tracked over time, with a check that fails when an area drops.
- design: `browser/tools/conformance.py`: `run [-j N] [AREA...]` (calls `wpt.py run` and `test262.py run` for the areas of `wpt_areas.json` and the test262 directories); `table` writes `docs/conformance.md` (per area: files, score, threshold, status, change since the last table; the pinned revisions; the date and Dweller commit), `browser/tools/conformance/<area>.json` (test path → passed/total, sorted, so a regression shows as a diff of named tests) and appends a row per area to `browser/tools/conformance/history.csv`; `check` exits 1 when an area's score is more than 0.5 points below the committed table, listing the tests that went from pass to fail. The first baseline is committed in this PR. CI: `.github/workflows/conformance.yml` on `macos-latest`, weekly and on `workflow_dispatch`: fetch (cached by revision), build, `run`, `check`; the results are an artifact. A component card runs `run AREA` and `check` for its area in its verify and commits the new table rows.
- files: `browser/tools/conformance.py`, `browser/tools/conformance/`, `docs/conformance.md`, `.github/workflows/conformance.yml`
- verify: `python3 browser/tools/conformance.py run -j 8 > target/conformance.log 2>&1 && python3 browser/tools/conformance.py table && python3 browser/tools/conformance.py check; echo $?` → 0; `docs/conformance.md` lists every area of the table above
- depends on: P10.1, P10.2, P10.3
- time box: 90 min; model: sonnet
- status: done 2026-10-16. `browser/tools/conformance.py` (`run [-j N] [AREA...]`, `table`, `check`), `docs/conformance.md`, per-area baselines `browser/tools/conformance/<area>.json` (test → passed/total), `history.csv`, `planned.txt` (areas of the plan not yet measured), and `.github/workflows/conformance.yml` (weekly and `workflow_dispatch`, `ubuntu-24.04`, fetch cached by revision). Thresholds of `wpt_areas.json` filled from the plan table (cssom, webaudio have none). Baseline (WPT e58225ab, test262 d53ebc8b19): dom/nodes 47.2, dom/events 48.1, forms 36.1, css/cssom 54.0, fetch/api/basic 23.4, xhr 15.5, url 48.5, encoding 0.0 (280/742368: a few files with hundreds of thousands of subtests), webaudio 63.3, css/css-flexbox 37.8, test262 built-ins 64.2 (11792/18373, 5450 skipped), language 65.0 (15080/23210). A full run took about 17 min at -j 4. Left: the 17 plan areas in `planned.txt` (need their directories in `wpt_areas.json` and `fetch`), test262 annexB, the macOS job, `encoding` scored per file rather than per subtest.

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

#### P10.35: encoding

- goal: WPT `encoding` from 0.0% (280 of 742368 subtests) to its 80% threshold. The first table's count is dominated by the `legacy-mb-*` encode tests: 730,000 subtests that load a page in a legacy charset and check what an `a.href` setter (`encode-href-common.js`) and a GET form submitted into an iframe (`encode-form-common.js`) write into the query.
- cause: no test failed for a missing decoder. The query of every URL was UTF-8, whatever the document's encoding; a form submitted to a frame by a script (`form.submit()`) was not followed in `dweller wpt` (and only one `submit()` per script run survived); `<body onload>` did not reach the window; `form.acceptCharset` and a `window.location` of a frame (`search`, `hash`, ...) were missing; a frame in markup was not loaded in `dweller wpt`; the runner split result lines at U+2028. `TextDecoder` knew UTF-8 only.
- design: `url.resolveIn(alloc, base, input, encoding)` writes a special URL's query in the document's encoding through `encoding.fromUtf8With` (a code point it cannot write is `%26%23N%3B`); `Page.resolve` passes the page's encoding. `engine/submit-into-frame.aye` follows `form.submit()` into a named frame while `runUntilConsole` runs (forms submitted before it are queued in `Script.queued_forms`). `js/text-codec.js` (embedded, run after `LIB`) is `TextDecoder` and `TextEncoder`: the WHATWG label table and the 27 single-byte indexes (`js/text-codec-data.js`, generated by `tools/gen_text_codec_data.py` from encodings.json and index-*.txt; licence in `browser/licenses/WHATWG-ENCODING`), `fatal`, `ignoreBOM`, `stream`, UTF-16LE/BE, x-user-defined, `encodeInto`; the CJK encodings call `encoding.toUtf8` (`legacyDecode`).
- files: `browser/url/`, `browser/encoding/encoding.aye`, `browser/engine/` (`engine.aye`, `script.aye`, `window_api.aye`, `prelude.aye`, `submit-into-frame.aye`), `browser/js/` (`text-codec.js`, `text-codec-data.js`, `legacy-decode.aye`, `install-text-codec*.aye`, `typed.aye`), `browser/tools/wpt.py`
- verify: `python3 browser/tools/conformance.py run encoding -j $(nproc)`; `python3 browser/tools/conformance.py table`; `python3 browser/tools/conformance.py check; echo $?` → 0; `cargo test --release --test browser -- encoding dump_fixtures errors`
- depends on: P10.4
- time box: 150 min; model: opus
- status: done 2026-10-16. `encoding` 280/742368 (0.0%) before; 1138861/1139871 (99.9%) after (more subtests run: the decode pages now load). Left (1010): `TextDecoderStream` and `TextEncoderStream` (about 120, need a Streams transform), `SharedArrayBuffer` and detached-buffer cases of `encodeInto`/`decode` (about 60), single-byte-decoder tests through XHR `overrideMimeType` and `document.characterSet` of iframes (504), `unsupported-labels` (needs `window.open`-less `fetch` of each label, times out), gb18030/GBK/Big5/ISO-2022-JP encoder tests that call `encoder` APIs directly (about 340: the encoders read by `encoding.fromUtf8` differ in details: Euro in GBK is `0x80`, PUA pointers, Big5 excluded pointers, ISO-2022-JP katakana/SO/SI), UTF-32 sniffing and `application/xml`, JSON documents, `idlharness`.

#### P10.32: translate, rotate, scale

- goal: the individual transform properties `translate`, `rotate` and `scale` (CSS Transforms 2) parse, compute (computed values as the spec serializes them, read by `getComputedStyle`) and apply in the spec's order (translate, then rotate, then scale, then `transform`, around `transform-origin`) in layout, paint and hit testing, through the existing `transform` path; sites move menus and carousels with `translate` and the boxes stayed put.
- design: three `PKind.Raw` properties (`css.TRANSLATE`, `ROTATE`, `SCALE`), parsed by `individualTransformValue` into their computed text: `translate` drops a zero z and then a zero y (`0em` is `0px`, `calc()` as `calc(-10% + 10px)`), `rotate` writes the angle in `deg` and names an axis along x, y or z (`x 360deg`, `-360deg` for `0 0 -1`), `scale` writes percentages as numbers and drops a z of 1, then a y equal to x; an invalid value drops the declaration (so `CSS.supports` is right). Layout composes them in `boxTransform` (`layout/box-transform.aye`) from the transform functions that `transform` uses; `rotate` about another axis is the 2-D part of its `rotate3d()` matrix (`axisRotation`: `rotate: x 60deg` draws as a vertical scale of 0.5). A box with any of the four is transformed (a layer and a containing block); a change in only these, `transform` or `transform-origin` takes the retransform path of P4.56 (`css.isTransformPart`).
- files: `browser/css/` (`css.aye`, `values.aye` wiring, `individual-transform-value.aye`, `write-translate.aye`, `write-rotate.aye`, `write-axis-angle.aye`, `write-scale.aye`, `named-axis.aye`, `scale-number.aye`, `zero-as-px.aye`, `is-transform-part.aye`, `has-any-transform.aye`, `cascade.aye` wiring), `browser/layout/` (`transform.aye` wiring, `box-transform.aye`, `rotate-part.aye`, `axis-rotation.aye`), `browser/engine/geometry.aye` (wiring), fixture `browser/tests/layout/individual_transforms.html`, `browser/tools/wpt.py` (`css/support` fetched: the parsing tests load `parsing-testcommon.js` from it), `wpt_areas.json` (area `css/css-transforms`)
- chromium: `third_party/blink/renderer/core/css/properties/longhands/` (translate, rotate, scale), `core/style/computed_style.cc` (`ApplyTransform`: the order), `core/layout/layout_object.cc`; spec https://drafts.csswg.org/css-transforms-2/#individual-transforms
- verify: `python3 browser/tools/wpt.py run css/css-transforms/parsing/{translate,rotate,scale}-parsing-{computed,valid,invalid}.html css/css-transforms/individual-transform`; `python3 browser/tools/conformance.py check; echo $?` → 0; `cargo test --release --test browser -- layout dump_fixtures screenshot_fixtures errors` → 0
- depends on: P10.1, P10.2
- status: done 2026-10-16. Before / after: parsing 36/178 / 108/178 (computed 0/80 / 72/80; valid 36/75 and invalid 0/23 unchanged), `individual-transform` 2/18 / 7/18 (5 skipped); 38/196 / 115/196 in all. Left: computed `scale` with `sign()` (8; `calc()` has no `sign()`), `el.style` keeps any text as it was set (the valid and invalid tests: a CSSOM declared-value card, every property), and reftests that fail with `transform` too: a negative `z-index` child of a non-positioned transformed box (`stacking-context-002..004`), `scale: 0` still drawn (`individual-transform-3`), `perspective()` and 3-D (`individual-transform-1`), SVG `transform-box` (`translate-fill-box`, `-view-box`), the rAF `change-*-property` pages (the change is not reached with `transform` either). `cargo test ... layout dump_fixtures screenshot_fixtures errors`: 10 passed, no golden changed (new fixture `layout/individual_transforms`); `window_transform_composite` passes. `css/css-transforms` is in `wpt_areas.json` but not yet in the conformance table (only the paths above were run).

#### P10.27: `vertical-align`

- goal: `vertical-align` complete in inline layout (`baseline`, `sub`, `super`, `text-top`, `text-bottom`, `middle`, `top`, `bottom`, lengths, percentages of the element's `line-height`; on inline boxes, replaced elements and inline-blocks) and in table cells (`baseline`, `top`, `middle`, `bottom`), with the `valign` hint. Before this card the property did not exist (`layout.aye`: "vertical-align is always baseline"; cells sat at their top).
- files: `browser/css/` (`css.aye` property 120, `ua.aye` table defaults, `hints.aye` + `valign-hint.aye`), `browser/font/` (`x-height-of.aye`, `Metrics.x_height`), `browser/layout/` (`line-align.aye`, `align-line.aye`, `align-piece.aye`, `align-close.aye`, `close-open.aye`, `align-shift.aye`, `align-top-bottom.aye`, `top-owner.aye`, `move-entries.aye`, `box-dy.aye`, `align-cells.aye`, `content-extent.aye`, `shift-contents.aye`; wiring in `inline.aye`, `bidi.aye`, `table.aye`), `browser/engine/` (`unwrap-cdata.aye`, `without-cdata.aye`, wiring in `engine.aye`), `browser/net/file.aye` (`.xht`)
- chromium: `core/layout/inline/inline_box_state.cc` (`ComputeAlignmentBaselineShift`, pending `top`/`bottom` descendants aligned to the nearest `top`/`bottom` ancestor or the line, `MetricsForTopAndBottomAlign`), `core/layout/table/` (cell alignment: baseline from the first line, cells without a baseline left out of the row baseline)
- verify: `python3 browser/tools/wpt.py run` over the 199 files of `css/CSS2/linebox/vertical-align-*`, `css/CSS2/visudet`, `css/CSS2/tables`, `css/css-inline`, `css/css-tables` that name `vertical-align` (155 run; 44 `.xht` without `rel=match` are manual); `python3 browser/tools/conformance.py check` → 0; `cargo test --release --test browser -- layout dump_fixtures screenshot_fixtures errors` → 0; fixture `layout/vertical_align.html`
- status: done 2026-10-16. 127/501 subtests (15 of 155 files fully passing) before; 245/501 (101 files) after, none newly failing. Each line keeps an aligned-subtree extent per inline box (its strut, its children shifted); a box's shift is applied to the line's pieces when it closes (`LineAlign.dy`, read by the fragment code of `inline.aye` and `bidi.aye`), `top`/`bottom` subtrees wait for their owner's extent. `middle` reads the font's OS/2 x-height (`Fixed`: 0.5em). Cells: the row grows to fit baseline-aligned cells, content moves by its offset (`shiftContents`: lines, fragments, atomic and block children), `middle`/`bottom` use the content's extent, not a specified `height`. UA sheet as HTML: `thead, tbody, tfoot, table > tr { vertical-align: middle }`, `tr, td, th { vertical-align: inherit }`. The largest cluster (88 of 104 failing reftests) was not layout: the CSS2 references wrap `<style>` in `<![CDATA[ ]]>`, which the HTML parser keeps, dropping the first rule; an `application/xhtml+xml` document now has the markers removed from its `<style>` text (there is no XML parser). Goldens changed for the better: `layout/compressible` and `layout/table_widths` (a short cell beside a tall one is centered, as Chrome's default `middle`), `paint/forms_html5` (range, color, progress, meter take the UA `vertical-align: middle` and center on their labels), `paint/css_backgrounds` (its `vertical-align: top` tiles align at the top). Left: `css-inline/parsing` (the css-inline-3 split into `alignment-baseline`/`baseline-shift`, serializing `0` as `baseline`, and CSSOM rejecting invalid values: 51 subtests), `baseline-source` (22 × 4), `ex` units in `vertical-align-091`/`092`, `vertical-align-121`/`122` (`top`/`bottom` beside a border; the layout matches the spec reading, the reference differs), `sub-001`/`super-001` (absolute boxes), `visibility: collapse` tables.

#### P10.28: `letter-spacing`, `word-spacing`

- goal: `letter-spacing` and `word-spacing` (`normal`, lengths, percentages of the font size per css-text-4, the computed length inherited) parse, compute (getComputedStyle: `letter-spacing` `0px` reads `normal`, `word-spacing` `normal` reads `0px`) and apply after shaping: letter-spacing after every typographic character unit (after the last one of a line too, as Chrome), not inside cursive scripts, common ligatures off when it is not 0; word-spacing after each word separator. Both feed line breaking and intrinsic sizes. Before this card neither property existed.
- files: `browser/css/` (`css.aye` properties 130 and 131, `fix-spacing.aye`, wiring in `cascade.aye`), `browser/font/` (`Font.letter_spacing`/`word_spacing`, `spacing-of.aye`, `space-glyphs.aye`, `joins-cluster.aye`, `no-ligatures.aye`; wiring in `font.aye`, `freetype.aye`, `harfbuzz.aye`), `browser/layout/` (`spacing-px.aye`, wiring in `values.aye`), `browser/paint/` (`glyphs.aye` fixed-font pen, `paint.aye` spacing scaled with transformed text), `browser/unit_tests.aye`
- chromium: `platform/fonts/shaping/shape_result_spacing.cc` (`ComputeSpacing`: letter spacing per character unless zero-width or cursive, word spacing on spaces and no-break spaces), `platform/fonts/shaping/harfbuzz_shaper.cc` (spacing applied to the shaped glyphs)
- verify: `python3 browser/tools/wpt.py run css/css-text/letter-spacing css/css-text/word-spacing` plus the seven `css/css-text/parsing/{letter,word}-spacing-*.html` (needs `css/css-text`, `css/support` in the sparse checkout); `python3 browser/tools/conformance.py check` → 0; `cargo test --release --test browser -- layout dump_fixtures screenshot_fixtures errors` → 0; fixture `layout/spacing.html`
- status: done 2026-10-16. 28/102 subtests before (letter-spacing 12/35, word-spacing 2/21, parsing 14/46); 57/102 after (9/35, 15/21, 33/46). The spacing is part of the font: `Measurer.width` adds it (both `Fixed` and `System`), `System.shapeText` shifts the placed glyphs by the spacing of the text before their cluster (after it for right-to-left), so measuring, line breaking, intrinsic widths, caret and hit testing agree. A typographic character unit skips ignorable characters, combining marks and Indic marks and consonants joined by a virama (Bengali ya-phala). Two font fixes the tests needed: a space was always taken from the system face, never the web font (an Ahem space was 6px, not 1em; Chrome takes it from the primary font), and an ignorable character (U+FEFF, U+2060) got a glyph advance when shaped glyph by glyph. No golden changed. Newly failing, where the old pass came from ignoring the property on both sides: `letter-spacing-bidi-001`/`002`, `nesting-002` (they want no spacing at element boundaries and after bidi reordering; Chrome fails them too), `letter-spacing-animating-letter-spacing` and `word-spacing-animating-word-spacing` (Web Animations of the property). Left: `ch` resolves to 0.5em without the font's "0" (`word-spacing-002`/`003`, `trim-start-001`), the CSSOM setter accepts invalid values and keeps specified `calc()`/`0` unserialized (`*-invalid`, `*-valid`: 11), `calc(` without its `)`, `font-variant-ligatures`/`font-kerning` (`ligatures-002`..`005`), ruby (`212`), animations, `letter-spacing-200`..`206` and `end-of-line-001` (no spacing at line ends, which Chrome does not do either).

#### P10.31: `outline` and the focus ring

- goal: `outline`, `outline-width`, `outline-style` (with `auto`), `outline-color` (with `auto` and `currentcolor`; `invert` is rejected at parse time, so its fallback is the initial value) and `outline-offset` (with `inset`) parse, compute and serialize in `getComputedStyle` (the `outline` shorthand as width, style, color), and paint outside the border box after the box's content without changing layout or the scrollable size, following `border-radius`. `:focus-visible` (matching already in `css/selector.aye`: keyboard focus yes, a mouse click on a button no, text fields always) gets the UA rule `:focus-visible { outline: auto 1px }`, drawn as Blink's two-ring focus ring.
- design: four longhands at the end of `css.PROPERTIES` (`OUTLINE_COLOR` .. `OUTLINE_OFFSET`, paint-only); `PKind.Offset` is a length of any sign without percentages. `css/outline-shorthand.aye` parses `outline` (width, style and color in any order); `css/write-computed.aye` resolves `currentcolor`, and `auto` unless the style is `auto` (`css/auto-is-current.aye`), to the element's color in `getComputedStyle` (the cascade keeps the initial `auto`, so the style dump omits it) (border colors too, as in Chrome); `css/write-outline.aye` is the computed shorthand. `paint/outline.aye` draws after a box's children (and per fragment of an inline box); `paint/outline-ring.aye` draws the ring as four `side`s (dashed, dotted, double) or as a rounded SVG ring, with `Builder.fixed` set so the outline does not extend the page; `paint/focus-ring.aye` is `outline-style: auto`: a 2 px ring in the outline color (#101010 for `auto`) inside a 1 px white one, pulled 1 px in when every border is at least 1 px, corners at least 1 px round. The window's own blue ring around any focused element (`ui/forms.aye`) is gone: the page's `:focus-visible` outline replaces it, so `outline: none` and mouse clicks on buttons behave as in Chrome.
- files: `browser/css/` (`css.aye`, `values.aye`, `cascade.aye`, `ua.aye`, `outline-shorthand.aye`, `auto-is-current.aye`, `write-computed.aye`, `write-outline.aye`), `browser/paint/` (`paint.aye`, `outline.aye`, `outline-ring.aye`, `focus-ring.aye`), `browser/engine/` (`window_api.aye`, `prelude.aye`), `browser/ui/forms.aye`, `browser/unit_tests.aye`, fixture `browser/tests/paint/outline.html`
- chromium: `third_party/blink/renderer/core/paint/outline_painter.cc` (`PaintFocusRing`: a 3 px stroke split 2:1 into an inner ring in the outline color and an outer white one; `FocusRingOffset`: 1 px inside a border of at least 1 px; `DefaultFocusRingCornerRadius`), `core/paint/box_painter.cc` (the outline phase after the box's content), `core/css/selector_checker.cc` (`:focus-visible`). Algorithm reference only (LGPL). Not taken: the outline of an inline box or an `auto` ring as the union of its descendants' rects (Blink's `OutlineRects`), `accent-color` for `auto`, groove/ridge/inset/outset shading.
- verify: `python3 browser/tools/wpt.py run` over `css/css-ui/outline-*`, `css/css-ui/parsing/outline-*`, `css/css-ui/parsing/canonical-order-outline-*` and `css/selectors/focus-visible-*` (123 files; the directories are not a `wpt_areas.json` area, so `git -C target/wpt sparse-checkout add css/css-ui css/selectors css/reference css/support` first); `python3 browser/tools/conformance.py check; echo $?` → 0; `cargo test --release --test browser -- paint dump_fixtures screenshot_fixtures window_ errors`
- depends on: P10.1, P10.2
- status: done 2026-10-16. Before → after: parsing 39/117 → 72/117 subtests, paint reftests 21/67 → 28/67, `:focus-visible` 0/173 → 37/173 (`focus-visible-017-2`: 37/38; the rest drive the keyboard through `testdriver.js` actions or time out). Reftests that passed before only because neither side drew an outline now fail for causes outside this card: `outline-color-002`..`004` (`accent-color`), `outline-auto-width-001` (its two `match` refs, one for `auto` as a ring and one for `auto` as `solid`, must both match), `outline-with-padding-001` (the `auto` ring of an inline box around its descendants), `outline-auto-dynamic-change` and `outline-019` (the ring and the square are drawn, at other pixels than the refs). Goldens: `browser/tests/ui/forms_html5.validation.png` changed for the better: the focused text field shows the `:focus-visible` ring (dark with a white edge, as Chrome draws it) instead of the window's blue overlay, which also drew on buttons focused by a click and over `outline: none`. New golden `browser/tests/paint/outline.png`. `-- paint dump_fixtures screenshot_fixtures window_ errors`: 52 passed, `window_webgl` fails without ANGLE (not installed here; unrelated). Left: outlines of inline boxes and `auto` rings around descendants' overflow, `accent-color`, `invert`.

#### P10.33: transition events and computed values

- goal: `transition`, `transition-property`, `-duration`, `-delay`, `-timing-function`, `-behavior` parse and compute (getComputedStyle serialization, list repetition); a style change on a transitionable property starts a transition per CSS Transitions 1 (before-change and after-change style, combined duration > 0, interruption and reversal shortening), fires `transitionrun`, `transitionstart`, `transitionend`, `transitioncancel` on the loop's frame clock, and the computed value during the transition interpolates (numbers, lengths, colors, opacity; discrete properties do not transition). Sites that wait for `transitionend` to open a menu or drop an overlay hang today (anoma.ly sets all five).
- files: `browser/css/` (`css.aye` and `values.aye` wiring; `transition-list.aye`, `add-transition.aye`, `write-single-transition.aye`, `transition-slot.aye`, `write-transition-item.aye`, `write-transition-property.aye`, `write-easing*.aye`, `write-cubic-bezier.aye`, `write-steps.aye`, `function-args.aye`, `time-seconds.aye`, `write-seconds.aye`, `write-transition.aye`, `list-item.aye`, `list-length.aye`), the new module `browser/animation/` (core layer: `Transitions`, `style-change.aye`, `change-property.aye`, `start-transition.aye`, `update-transitions.aye`, `value-at.aye`, `interpolate.aye`, `parse-easing.aye`, `ease-at.aye`, `bezier-at.aye`), `browser/engine/` (`transition-frame.aye`, `restyle-transitioning.aye`; wiring in `script.aye`, `window_api.aye`, `prelude.aye`), `browser/tools/wpt.py` (`css/support` fetched), `wpt_areas.json` (area `css/css-transitions`)
- chromium: `third_party/blink/renderer/core/animation/css/css_animations.cc` (`CalculateTransitionUpdate`, `CalculateTransitionUpdateForProperty`: the matching `transition-property` item is the last one naming the property, `all` or a shorthand of it; a running transition whose end value stays keeps running; the reversing-adjusted start value and shortening factor), `css_transition.cc`, `ui/gfx/geometry/cubic_bezier.cc` (the easing curve). Spec: https://drafts.csswg.org/css-transitions-1/, events https://drafts.csswg.org/css-transitions-2/#event-dispatch.
- design: the longhands are `PKind.Raw` lists normalized at parse time (times in seconds, `step-end` as `steps(1)`, `all`/`none` lower case), so getComputedStyle prints them as stored; `transition` is a shorthand into all five (`initial` lists for the missing parts) and its computed serialization (`css.writeTransition`) leaves out default parts. `Script.transitions` keeps a before-change snapshot of each element getComputedStyle read whose `transition-*` give a combined duration > 0 (interpolable values only, so no pointer into a freed sheet). getComputedStyle runs the style change for the element and answers the interpolated value; while any element is watched the prelude keeps one animation frame callback (`ak.transitionFrame`), which restyles the watched elements, advances the transitions to the frame time (`s.now`, the rAF clock) and returns the events, dispatched as `TransitionEvent`s. `display: none` and disconnecting cancel.
- verify: `python3 browser/tools/wpt.py run css/css-transitions -j 8` before/after; `python3 browser/tools/conformance.py check` → 0; `cargo test --release --test browser -- jsdom_fixtures dump_fixtures window_ errors > target/p10.33.log 2>&1; echo $?` → 0
- depends on: P10.4; time box: 45 min; model: opus
- status: partial 2026-10-16. `css/css-transitions` (WPT e58225ab, `css/support` now fetched): 184/3341 (5.5%) before, 335/3341 (10.0%) after; `parsing/` 44/252 → 94/252 (the `-valid`/`-invalid` files test the inline style's CSSOM, which keeps any text); `events-*` 1/23 → 16/23; `properties-value-*`, `transition-00x`, `inheritance` 0/2080 → 67/2080. `-- jsdom_fixtures dump_fixtures window_ errors`: 54 passed after merging main (`window_webgl` needs ANGLE: `browser/gpu/fetch_angle.sh` under bash with `zstd`). Left: an element enters the watch only when getComputedStyle reads it, so a class or style change on an element only laid out (`offsetWidth`, the window's relayout) starts no transition: hook the page's restyle (`engine.aye styleFor`, old styles as before-change style) next; that is most of `properties-value-*` and `transitioncancel-001`; `calc()` in times and `steps()`; `transition-behavior: allow-discrete`; `transition-property` on custom properties; `CSSTransition`/`getAnimations()` and the event's `animation`; `@starting-style`; pseudo-elements; transitions inside frames.

#### P10.25: `lh`, `rlh`, `env()`, `round()`/`mod()`/`rem()`

- goal: the `lh` and `rlh` units (the computed line-height of the element and of the root; in `font-size` and `line-height` themselves `lh` is the parent's, and on the root `rlh` there is the initial value's), `env()` with the `safe-area-inset-*` variables (0px on a desktop window) and its fallback (unknown names without one make the declaration invalid at computed-value time), and the stepped math functions `round()` (`nearest`, `up`, `down`, `to-zero`; the step may be left out for a number), `mod()`, `rem()`, `abs()`, `sign()` in `calc()` trees with type checking (numbers with numbers, lengths with lengths; `sign()` of a length is a number), the IEEE rules (`infinity`, `-infinity`, `NaN`, signed zero, division by zero) and the top-level censoring (NaN as 0, infinities clamped). Today any of them drops the declaration: `right: max(0px, env(safe-area-inset-right))` and `min-height: 1lh` were lost on anoma.ly.
- design: `CalcParser.term` reads an optional rounding keyword (`roundMode`) and collects the arguments of the stepped functions like `clamp()`'s; `steppedValue` checks the arity, `scalarsOf` the types (a percentage mixed with px needs the property's percentage base, else the value is invalid), then `roundToStep` / `modulo` / `math.abs` / `math.sign`; `finiteQuantity` censors the top-level result. `lh`/`rlh` are context units (kept as tokens until computed): `computeValues` computes `line-height` first (`lineHeightFirst`) and sets `lh_px`/`rlh_px` (`useLineHeight`, `lineHeightPx`: `normal` is 1.2 em, the cascade has no font metrics). `env()` rides on `var()` substitution (`hasVar`, `substituteEnv`), with the name/indices/fallback grammar checked at parse time (`validEnv`). `z-index` (and any integer property) takes math functions, rounded.
- files: `browser/css/` (`calc.aye`, `cascade.aye`, `values.aye`, `vars.aye` wiring; `round-mode.aye`, `stepped-function.aye`, `stepped-value.aye`, `scalars.aye`, `scalars-of.aye`, `quantity-of.aye`, `round-to-step.aye`, `infinite-step.aye`, `modulo.aye`, `math-constant.aye`, `finite-quantity.aye`, `line-height-units.aye`, `line-height-unit-px.aye`, `line-height-px.aye`, `line-height-first.aye`, `safe-area-inset.aye`, `substitute-env.aye`, `env-fallback-start.aye`, `valid-env.aye`), `browser/unit_tests.aye` (the telemetry example of an unsupported unit is now `cap`)
- chromium: `third_party/blink/renderer/core/css/css_math_expression_node.cc` (stepped-value operators, type checking), `core/css/parser/css_math_parser`/`css_parser_token_stream` (argument parsing); algorithm reference only (LGPL). Specs https://drafts.csswg.org/css-values-4/#lh, #round-func, #mod-func, #calc-ieee, https://drafts.csswg.org/css-env-1/
- verify: `git -C target/wpt sparse-checkout add css/css-values css/css-env css/reference` (not a `wpt_areas.json` area); `python3 browser/tools/wpt.py run` over `css/css-values/{lh,rlh,round,mod-,rem-,signs,signed,sign-,calc-serial}*` and `css/css-env` before and after; `python3 browser/tools/conformance.py check` → 0; `cargo test --release --test browser -- style_fixtures dump_fixtures layout errors > target/p10.25.log 2>&1; echo $?` → 0
- status: done 2026-10-16. 29/1164 subtests before, 751/1164 after: `round-function` 0 → 217/219, `round-mod-rem-computed` 0 → 209/243, `signs-abs-computed` 0 → 167/233, `signed-zero` 0 → 105/162, `round-mod-rem-serialize` 1 → 6/24, `lh-unit-001`/`002`, `rlh-on-root-lengths` 0 → 1, `css-env` 11/43 → 30/43 (`syntax` 4 → 17/18). No golden changed. Left: math in angles and times (`rotate: calc(10deg)`, `transition-delay: calc(1s)`: no math there at all, 34 of the computed tests), a percentage that the property resolves only at layout (`margin-left: abs(10%)`), signed zero in `min()`/`max()`/`clamp()`, `el.style` keeping any text as set (the `-invalid` and `-serialize` tests, `calc-serialization*`, `env-parsing`, `indexed-env`: a CSSOM declared-value card for every property), `lh` of `line-height: normal` from font metrics (`lh-rlh-on-root-001` 4/8), restyle when the root's line-height changes (`rlh-invalidation`), `env()` in `@supports`/`CSS.supports` script and `revert-rule`.

### Layer 1 component backlog

Component cards (P10.7 onward) are written from the baseline: per area,
the largest cluster of failing tests with one cause first; verify is
`wpt.py run <dirs>` (or `test262.py run <dir>`) with the score at least
the stated gain and `conformance.py check` exiting 0. The items below
were filed by sites or the B8 list; each becomes part of the card for
the failing tests that cover it, or its own card if no test covers it.
Before a site-found item becomes a card, its symptom is confirmed on
current `main`.

Telemetry `unsupported` CSS names come from `css.isProperty` (the
cascade's longhands and shorthands) since 2026-10-15, and a math
function's value names the piece `calc()` cannot evaluate. The anoma.ly
list (P4.53) is all truly unsupported; ranked by what real sites lose:
`pointer-events` (P10.34), intrinsic size keywords (P10.24),
`vertical-align` (P10.27), `letter-spacing`/`word-spacing` (P10.28),
`transition*` (P10.33), `translate`/`rotate`/`scale` (P10.32;
`will-change` arrived with P4.56), `outline` (P10.31), `text-wrap` and `text-shadow` (P10.29),
`border-collapse` (P10.26), `lh` and `env()` (P10.25), text decoration
details (P10.30). Accepted as no-ops if they ever need silencing, not
cards: `font-feature-settings`, `font-variation-settings`,
`text-rendering`, `text-size-adjust`, `appearance`, `resize`,
`user-select`, `touch-action`, `isolation`, `unicode-bidi`,
`scrollbar-width`/`-gutter`/`-color`, `scroll-margin-*`.

- **HTML parsing:** P3.10 scripts run while parsing see an 800px viewport instead of `--width` (async images follow-up).
- **DOM:** P4.38 `SVGAElement` (with `SVGElement` as its prototype): Mistral's Astro `ClientRouter` throws on every click that is not on an `<a>` (user log 2026-10-15); verify also: a fixture clicking a `<span>` with `t instanceof SVGAElement` in a listener, no Uncaught on mistral.ai. Done 2026-10-15 (branch `telemetry-unsupported-audit`): `SVGAElement`, `SVGGElement`, `SVGUseElement`, `SVGImageElement`, `SVGTextElement` under `SVGGraphicsElement`, and `SVGGeometryElement` with `SVGPathElement`, `SVGRectElement`, `SVGCircleElement`, `SVGEllipseElement`, `SVGLineElement`, `SVGPolylineElement`, `SVGPolygonElement` (`engine/dom_api.aye`, `Script.svg_tag_protos`); fixture `jsdom/svg_geometry.html` (an HTML `<a>` click is not an `SVGAElement`). P3.32 a frame's `history.length` counts only its own entries; 1 during initial load (P3.29). B8: moving nodes between documents; one joint session history for a tab and its frames.
- **Events and timers:** P10.34 `pointer-events: none` in hit testing (the click, hover and agent `click` go to the box beneath; `auto` on a descendant re-enables it); overlays and decorative layers on many sites swallow clicks today. P3.9 dump virtual clock: an `onload` set inside `setTimeout(0)` after `src` misses the load event. B8: event listeners on another frame's nodes. Status: done 2026-10-16. `pointer-events` (inherited; `auto`, `none` and the SVG keywords, which hit like `auto`) is a paint-only property; `engine/hit.aye` skips the box and the text fragments of a box whose value is `none` and keeps searching its children and the layers beneath (`pass-through.aye`), so click, hover, `linkAt` and `elementFromPoint` follow the box beneath and a descendant with `auto` takes them back. The agent's `click` on an element that computes `none` goes to what is at the middle of its box (`click-target.aye`). Tests: `window_pointer_events` (`browser/tests/ui/pointer_events.html`: click and hover through an overlay, an `auto` child, `none` inherited), agent session `pointer_events` (`sessions/pointer_events.jsonl`). `-- window_ agent_ errors`: 53 passed, `window_webgl` fails without ANGLE (not installed here; unrelated). No WPT area covers `css/css-ui` or `pointerevents` in `wpt_areas.json`, so no before/after.
- **CSS box model and sizing:** P10.24 intrinsic size keywords: `width`/`height`/`min-*`/`max-*`/`flex-basis` take `max-content`, `min-content`, `fit-content` and `fit-content(<length>)` (layout already computes min/max-content for flex and grid items; today the declaration is dropped, so `width: fit-content` buttons and chips stretch to the full line; telemetry `width: max-content`, `width: fit-content` on anoma.ly); verify: `wpt.py run css/css-sizing` gains, a layout fixture of a `fit-content` block. Done 2026-10-16: the three keywords and `fit-content(<length-percentage>)` (`Value.FitContent`, absolute lengths, percentages and `calc()`; `em` inside is still dropped) on `width`, `min-width`, `max-width` and `flex-basis` of block, atomic inline, absolute, flex and grid items, from the cached min/max-content widths (`layout/content-size-of.aye`, `prime-intrinsic.aye`); on heights they act as `auto`/`none`. Fixture `layout/intrinsic_sizes.html`. `wpt.py run css/css-sizing` (area added to `wpt_areas.json`): 383/1882 (20.4%) before, 439/1882 (23.3%) after. P3.59 intrinsic sizing passes no container height (`browser/layout/intrinsic.aye`, `cb_h = -1`); BBC NEWS `<h1>` 300px instead of 100px (P3.47; still seen 2026-10-14: the NEWS wordmark 100 px left of centre).
- **CSS floats and tables:** P10.26 `border-collapse` and `border-spacing` (collapsed borders are drawn twice as thick today; most data tables and CSS resets set `collapse`). P3.70 abehiroshi: the news column drawn below the photo instead of beside it, menu bullets above their links. P3.71 ZOL: 找产品 product names wrap and the rows misalign; the 新品首发/本月销量最高 tabs stack. P3.68 Python docs: the sidebar drawn right of the text (Chrome: left); diagnose to its area first.
- **CSS values, units, color:** (done, see the P10.25 card) P10.25 the `lh` and `rlh` units (line-height of the element and of the root), `env()` (`safe-area-inset-*` as 0, else its fallback) and the stepped math functions `round()`, `mod()`, `rem()`, `abs()`, `sign()` in `calc()` (`css/calc.aye`); today any of them drops the whole declaration, so `right: max(0px, env(safe-area-inset-right))` or `min-height: 1lh` is lost (telemetry `min-height: lh`, `right: max()` on anoma.ly, which now report `lh`/`env()`); verify: `wpt.py run css/css-values` gains, a style fixture.
- **CSS overflow:** P3.53 ellipsis and clamp follow-ups: right-to-left, `-webkit-line-clamp` into child blocks, unprefixed `line-clamp` (P3.52).
- **CSS grid:** P3.54 `grid-template` with area strings and the `grid` shorthand (P3.46).
- **CSS inline and text:** P10.27 (done 2026-10-16) `vertical-align` (inline boxes: `middle`, `top`, `bottom`, `text-top`, `text-bottom`, `sub`, `super`, lengths; table cells: `middle`/`bottom`, and the `valign` hint in `css/hints.aye`); icons next to text and table cells sit on the baseline/top today (`layout/layout.aye`: "vertical-align is always baseline"). P10.28 `letter-spacing` and `word-spacing` in text measurement and painting (they change line breaks; logos and nav labels set them), `tab-size` in `pre`. P10.29 `text-wrap` as the CSS Text 4 shorthand of `white-space`'s wrapping (`nowrap` must not wrap; `balance` and `pretty` lay out as `wrap`), and `text-shadow` painted under the text (one layer is enough to start). P10.30 text decoration details: `text-decoration-color`, `-thickness`, `-style`, `text-underline-offset` (link underlines in the wrong color today). Verify each: `wpt.py run css/css-text` (or `css/css-text-decor`) gains, a fixture.
- **CSS positioning:** B8: sticky only against the viewport.
- **CSS backgrounds and borders:** P3.70 abehiroshi's tiled background image missing. P10.31 `outline` (shorthand, `-width`, `-style`, `-color`, `outline-offset`) painted outside the border box, and the UA focus ring for `:focus-visible` (keyboard users and agents' activity outline see where focus is). Done 2026-10-16 (card P10.31).
- **CSS transforms:** P10.32 the individual transform properties `translate`, `rotate`, `scale` (applied before `transform`, CSS Transforms 2; `will-change` is parsed since P4.56); sites move menus and carousels with `translate` today and the boxes stay put. B8: rotations and skews other than quarter turns are not drawn.
- **CSS transitions (no WPT area yet):** P10.33 `transition`, `transition-property`, `-duration`, `-delay`, `-timing-function`: no animation (the new value applies at once), but `transitionrun`/`transitionstart`/`transitionend` fire after delay + duration when a listed property changes, and `getComputedStyle()` reports the longhands; scripts that wait for `transitionend` to open a menu or remove an overlay hang today (anoma.ly sets all five). Verify: a jsdom fixture waiting on `transitionend`, `wpt.py run css/css-transitions` (event tests) as a new area. Status: partial 2026-10-16, card P10.33.
- **Forms:** P3.51 form control content not centred vertically: GitHub commit messages about 10 px above the file names, DuckDuckGo's query text at the top of its box, YouTube's search-button icon at the top of the button (Google's bevelled buttons not reproduced 2026-10-14: Chrome draws the archive the same). P3.66 Wikipedia's Appearance panel was script, not form controls: filled since P4.45 (inserted scripts run). B8: calendar, color and file pickers, `datalist`.
- **Images and SVG:** P3.56 `<video poster>` loaded and painted like an image (empty video frame on wikipedia-en). P3.16 a script changing an img in another same-origin frame may record the failed load in the wrong page's cache. P4.39 `getTotalLength()`/`getPointAtLength()` on SVG geometry elements (Mistral's `layout` script); verify also: a fixture path's length matches Chrome's within 0.5 px. Done 2026-10-15 (branch `telemetry-unsupported-audit`): `svg/geometry.aye` walks the outline paint draws (`svg.shapePath`: path, line, polyline, polygon, circle, ellipse, rect with corners; arcs by `arcTo`, SVG's endpoint parameterization), cubics flattened within 0.001 user units; the distance clamped to the outline; `DOMPoint`/`DOMPointReadOnly` (and `SVGPoint`) in the prelude; detached elements work. Fixture `jsdom/svg_geometry.html` checks 30 lengths and points within 0.5 px. B8: no AVIF (out of scope for now).
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
- P3.41 BBC header: Register/Sign In drawn since P4.45 (inserted scripts run, 2026-10-15); the ☰/search button at the left was drawn already. Done.
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

- **WebGL (user, 2026-10-15):** the user builds 3D web games, so
  Dweller needs WebGL. The acceptance target is the user's game
  (`~/Documents/f1/web`, three 0.170.0). WebGL 2 comes first, because
  three.js r163 and later need it. The GPU backend is ANGLE, built from
  a pinned revision and shipped as prebuilt release assets. ANGLE's
  WebGL compatibility mode validates and translates every shader. Pixels
  reach the window first by a per-frame readback, later by GPU
  compositing. Design: `docs/webgl.md`; cards: P11.
- **Reference sources (user, 2026-10-15):** cards name the known
  implementation to read in the local checkout (V8, Blink, ANGLE).
  Porting from V8 and ANGLE (BSD) is allowed with attribution. Blink is
  an algorithm reference only, because some of its files are LGPL.
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
