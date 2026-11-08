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

## State on 2026-11-04 (`main` 7de2c08)

Facts from four audits run on 2026-11-04; the details are in the steps.

- **Stable.**
  - CI: in the month to 11-01, 20% of `ci` runs failed and 12 of 92
    failures were flaky. Since 11-01 a failed test is re-run once.
  - Flaky tests: `window_spaces` (`tests/browser.rs:2990`, `:3044`) and
    `jsdom_gc_stress`, which once gave a different `run.jsonl` under GC
    stress (run 39080652106). Also `window_spaces_watch`,
    `window_css_animation`, `agent_sessions`, `js_print_scopes` and
    `aye_unit_tests`.
  - The weekly `conformance` workflow has been red for three weeks. Only
    its webgl job fails: the 2/4/16 GB wasm-memory tests.
  - A tab crash ends the whole window. There is no watchdog for a
    runaway script and no session restore.
  - The shipped build is `--mode fast` (no bounds checks, `build.sh:2`).
  - There is no fuzzing.
- **Secure.**
  - The CORS check runs only in page JS (`engine/fetch-api.js`), so a
    compromised renderer skips it, and XHR reads the body of any
    origin (it filters only headers). `net/permissive-cors.aye` only
    fills headers for archived responses that lack them (corrected by
    Q3.2a; the first audit overstated it).
  - `fetchWith` (`engine/window_api.aye:611`) lets an http page request
    a `file:` URL. It checks only the method.
  - TLS runs on libcurl defaults (`net/http.aye:44`). There is no HSTS,
    no mixed-content check and no certificate interstitial.
  - There is no CSP, X-Frame-Options, nosniff or COOP.
  - iframe `sandbox` has only `allow-scripts` and `allow-same-origin`.
  - The agent socket has no permission mode and no token.
  - Already right: script-level same-origin checks
    (`engine/windows.aye:129`) and the cookie rules (`net/cookies.aye`).
- **Fast.**
  - W8 passes on 20 of 20 sites. W7 passes on 9 of 20 (2026-10-17).
    Q0.2 and Q0.4 (2026-11-04): same count; the rows that changed come
    from the archives re-recorded on 2026-10-18 (more scripts), one
    engine regression (react, 922271a) and zol's live load hitting the
    30 s timeout.
  - The main causes: restyle falls back to the whole document for shadow
    trees, `:has()` and sheet changes, and relayout is refused
    (`relayoutDirty` 1301 times on mistral). zol and bbc are not
    profiled.
- **Complete.** Missing: zoom, dark mode, downloads, the file picker,
  `prompt`, WebSocket, EventSource, IndexedDB, Service Worker,
  `<video>`/`<audio>` playback, contenteditable, async clipboard, IME
  composition, permissions, bookmarks and print.
- **Correct.** 13 of 29 conformance areas are at threshold (2026-10-21).
  W3 was 8 of 20 sites (2026-10-15).

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
   model.
5. Q6 follows Q4 in the perf lane.
6. Q7 runs last of the foundations.
7. Q8 runs after Q7.

Common to every step:
- It is one card and one PR.
- It runs only the tests that its verify names. It never runs two
  browser test runs at once.
- Logs go under `target/`.
- If the same verify fails twice, the step stops with a report.
- On the user's Mac, it runs `scripts/capped.sh`. Heavy steps go to
  cloud sessions.
- Each card names the Chromium files it read and, in one line, what
  Dweller takes from them.

## Q0: measure the truth (lane base)

Exit: every number of "State" above is re-measured on the current
`main`, and the weekly workflow is green.

| Step | What | Verify | Chromium |
|---|---|---|---|
| Q0.1 | Weekly `conformance`: the webgl job moves to a job of its own that is allowed to fail, and the 2/4/16 GB wasm-memory tests are skipped on runners with less memory than they need | the next scheduled run (or `workflow_dispatch`) is green | – |
| Q0.2 | W7 and W8, plus `jsbench`, with a fast and with a safe build. A dated table goes in `PERF.md`, with the cost of safe mode | `window_perf.py report --replay` runs for both builds and the table is committed | – |
| Q0.3 | Re-run W2, W3 and the conformance table on `main` | `page_load.py check --agent`; `conformance.py run` and then `table` are committed | – |
| Q0.4 | Find why results changed since 2026-10-17 (Q0.2). zol's first paint went from 3013 to 30079 ms. The scroll p95 of wikipedia-en, wikipedia (ja), react and go-docs went from about 17 to about 225 ms. bbc's idle CPU rose to 10.7%. First decide whether the cause is the measurement (the `--until-idle` change 2c25b94, `window_perf.py` history) or the engine, by measuring the affected sites at 7de2c08 and at c0a6fa2. Then bisect. This step only finds the cause and files a card for it | each changed row is explained, naming a commit or a measurement artifact | – |
| Q0.5 | bbc under replay ran 64 of its scripts, not 91, since Q3.3b: old archive entries gave CORS-mode module scripts no `Access-Control-Allow-Origin`, and `permissiveCors` now adds it for `Mode.Cors` too (done, #321) | `permissive-cors_test.aye`, `module_cors`; bbc's replay dump runs 96 scripts and 204 requests | – |

## Q1: stable in one process (lanes base and perf)

Exit: S1, S3 and S4 hold. The shipped build is checked. A runaway script
can be stopped.

| Step | What | Verify | Chromium |
|---|---|---|---|
| Q1.1 | The shipped build becomes `--mode safe`, and CI runs a smoke test on the shipped mode. Q0.2 measured the cost (loop +9.5%, jsbench +17.6%), and it is accepted: until Q2.8, a memory bug in an unsandboxed process exposes everything. Q4 wins the time back. A card in the Aye repo asks for bounds-check elimination in loops (navier-stokes 0.66×) | `build.sh` builds safe by default, and the smoke job is green | Chromium ships libc++ with its hardening checks on (`build/config/compiler/BUILD.gn`, `use_safe_libcxx` → `_LIBCPP_HARDENING_MODE_EXTENSIVE`) |
| Q1.2 | The `jsdom_gc_stress` divergence of run 39080652106: done 2026-11-04, no GC bug — the agent `run` hit its 30 s wall-clock budget under stress on a busy runner (fixed by 3261b7d); 30 runs under stress were identical | – | – |
| Q1.3 | Window tests do not depend on how many frames fit in wall time: 63 tests use `--frames`, 18 were at risk and now stop their page loops after fixed ticks and run `--until-idle` (#267, #268, #273); the rule is in `docs/practices/browser.md` | the changed tests pass under 6 CPU hogs | – |
| Q1.4 | Flaky tests: only when the TC5b record files a `flaky:` issue (none since 2026-10-21 in 38 `ci` runs; `window_spaces` and `window_css_animation` were last seen before Q1.3) | 50 runs in a row | – |
| Q1.6 | `jsdom_gc_stress` comes back to one test under 10 min, with `focus_freed` shrunk to just above the GC threshold and back in the stress run | one run under 10 min on CI | – |
| Q1.7 | Script interrupt: the VM checks a flag on back-edges and calls. After 5 s a bar says "page unresponsive" and offers Stop or Wait; Stop throws an uncatchable termination into the script | a page with `while(1){}` shows the bar, and Stop leaves the tab working | `chrome/browser/ui/hung_renderer/`, `v8/src/execution/stack-guard.cc` (`TerminateExecution`) |
| Q1.8 | A JS heap limit in bytes, not values (`js/gc.aye:283`). Out of memory ends the tab's scripts with an error page, never the process (done, PR #315 open; left: a frame or worker that runs out gets no error page, and native memory (DOM, layout, images) does not count) | a page that allocates without end gets the error page, and the window lives | `gin/v8_initializer.cc`, `v8/src/heap/heap.cc` (`NearHeapLimitCallback`) |
| Q1.9 | Session restore: tabs and their history come back after a quit or a crash (done, PR #339 open; `DWELLER_PROCESS=tab` is Q1.9b) | kill the window, start again, and the same tabs and URLs are there | `components/sessions/core/` |
| Q1.9b | Session restore for `DWELLER_PROCESS=tab`: the browser builds a `SavedWindow` from its history model and fills the tabs from `decodeSession` | `window_session_restore_after_kill` passes with `DWELLER_PROCESS=tab` | `components/sessions/core/` |
| Q1.10a | Fuzz harness: `dweller fuzz TARGET` (in process, coverage optional), a seed corpus, and the HTML parser target | a 10 min run finds no crash, or the crashes found are filed | `testing/libfuzzer/README.md`, `third_party/blink/renderer/platform/fuzzer/` |
| Q1.10a2 | Coverage-guided fuzzing once Aye has edge counters (aye-lang/aye#26); until then the fuzzer is blind | the HTML target's corpus grows from new edges | `testing/libfuzzer/` |
| Q1.10b | Fuzz targets for the CSS parser and the URL parser (done, PR #330 open; left: nested rules are not serialized through `selectorText`, and the `url` target uses one base) | the same | – |
| Q1.10c | A fuzz target for the JS parser and bytecode (done, PR #332 open; left: about 50 times slower than the html target, the byte-blind mutator mostly stops in the lexer, and direct `eval` is not fuzzed) | the same | `v8/test/fuzzer/` |
| Q1.10d | Fuzz targets for the image decoders (PNG, JPEG, GIF, WebP) and fonts (WOFF/WOFF2) (done, PR #340 open; the 705 MB image peak is Q1.10d2; the `font` target stops before FreeType) | the same | – |
| Q1.10d2 | A 256 MiB cap on the memory one image decode may use, checked from the header and the working buffers; a JPEG past it decodes downscaled and keeps its natural size (done, PR #343 open; left: PNG inflates all rows before unfiltering, so a very large PNG is refused where the reference streams it) | `downscaled_jpeg_natural_size`; the `image` fuzz peak falls from 705 to 297 MB; WPT `images` unchanged | `platform/image-decoders/image_decoder.cc` (`max_decoded_bytes_`), `jpeg/jpeg_image_decoder.cc` (`DesiredScaleNumerator`) |
| Q1.10e | A nightly CI job that runs each target for 10 min. It keeps the corpus and files issues (done, #320; left: each matrix job builds its own binary) | the job is green, and a planted crash is filed | – |
| Q1.11 | Soak: 1 h of scripted browsing over the archives, with RSS sampled and `--leak-check` at exit | RSS growth stays under the budget, with no leak | – |

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
| Q2.1 | Design note `docs/process-model.md`, about one page: what the browser and the renderers own, the IPC messages (navigation, frames, input, network, storage, frames tree), the site definition, the process reuse policy, and how Spaces fit in. Compare with Chromium and Fission | the user reads it | `docs/process_model_and_site_isolation.md`, `content/browser/renderer_host/render_process_host_impl.cc`, `content/browser/site_instance_impl.cc` |
| Q2.1b | Split `ui/window.aye` into the window and a page view, still in one process, along the boundary that the IPC will cut | the `window_*` tests pass with no golden change | – |
| Q2.2a | The first half of Q2.2: `dweller --renderer FD`, `browser/ipc/` framing, lifecycle messages (`hello`, `navigate`, `loaded`, `shutdown`), the browser's checks on each message, crash noted (done, #263) | `window_renderer_lifecycle` | – |
| Q2.2b | The second half of Q2.2: frames. Renderer skeleton: `dweller --renderer`, launched by `posix_spawn`, with a `socketpair` for IPC (the mechanism of `ui/spaces.aye`). It renders one tab, and its frames go through shared memory to the window. Behind `DWELLER_PROCESS=tab` (done, #281) | a window test loads a page in a renderer, and the pixels match the in-process golden | `content/renderer/render_thread_impl.cc`, `components/viz/service/surfaces/` (idea only) |
| Q2.2c | Q2.2b's rest: every queued message and fd read, `--font` and the script setting passed to the renderer, frames carry only damaged rects (#284) | `window_renderer_frames` | `mojo/core/channel_posix.cc` |
| Q2.3 | Input and UI across the boundary, in the parts below: mouse, wheel, keys, text input, focus, selection, find, cursor, title, URL, status, and the long-task chip | the `window_*` tests pass with `DWELLER_PROCESS=tab` | `content/browser/renderer_host/render_widget_host_impl.cc`, `input/` |
| Q2.3a | Mouse, wheel, keys and text input go to the renderer; it scrolls and paints at the offset; the window does not act on what it forwards (#285) (done, #285) | `window_renderer_input` | `content/browser/renderer_host/render_widget_host_impl.cc` |
| Q2.3c | Default actions in the renderer: link clicks and form submit ask the browser to navigate, which checks them; button keys, Escape, key scrolling, range drag; `select` popups asked of the browser (done, #289) | `window_renderer_input` with links, POST, `file:` refused, select | `content/browser/renderer_host/render_frame_host_impl.cc` (`BeginNavigation`) |
| Q2.3d | Focus traversal and the focus ring, caret, selection and the clipboard, find (done, #292; the clipboard and find move to Q2.3f) | the focus, selection and find `window_*` tests pass with the flag | `content/browser/renderer_host/render_widget_host_impl.cc` |
| Q2.3e | Cursor, title, URL, status and the long-task chip from the renderer; the window's own copy of the page goes away (done, #295) | the `window_*` tests pass with `DWELLER_PROCESS=tab` | – |
| Q2.3b | Agent commands routed to the tab's renderer; the browser renumbers agent refs, so the protocol stays the same (done, #299; refs needed no renumbering: one agent session per renderer) | `agent_sessions` passes with `DWELLER_PROCESS=tab` | – |
| Q2.3f | The rest of Q2.3 under the flag: clipboard (Cmd/Ctrl+A/C/X/V), find, link and form `target` (other tab, `_blank`), arrows on focused select/range/number, clicks in iframes, `wait` and the agent highlight | the `window_*` tests that Q2.3e lists as not running under the flag pass with it | – |
| Q2.4 | Network in the browser process, in the parts below: a renderer asks for loads over IPC. Cookies, the HTTP cache, `localStorage` and the profile stay in the browser. A renderer opens no profile file | `window_login` passes with the flag, and a test shows the renderer has no profile path or open jar | `services/network/url_loader.cc`, `services/network/public/mojom/url_loader_factory.mojom` |
| Q2.4a | A renderer's loads (documents, subresources, fetch/XHR, modules, workers' scripts) go to the browser's one `net` session over IPC; the browser picks the initiator from its own record (done, #302) | `window_login_in_renderer`, `window_renderer_net` | `services/network/url_loader.cc` |
| Q2.4d | The browser derives `top`, the referrer, the method and the headers of a renderer's request; forbidden headers and methods never pass | compromised-renderer hooks in `window_renderer_net` | `services/network/cors/cors_url_loader_factory.cc` (`IsValidRequest`) |
| Q2.4b | `document.cookie`, `localStorage` and `sessionStorage` over IPC to the browser's stores, each checked against the renderer's site | `storage_between_tabs` and the cookie tests pass with the flag | `content/browser/dom_storage/`, `services/network/restricted_cookie_manager.cc` |
| Q2.4e | Loads in the background: the renderer does not block on a load over IPC, and a load can be cancelled | page load time with the flag within 10% of without, on the 20 sites | – |
| Q2.4c | Proof: a renderer opens no profile file and holds no cookie jar (done, #322; left: `HOME` stays in the renderer's environment; close-on-exec by default comes with aye 5180d0c (O2.9), PR #335 open) | a test lists the renderer's open files | – |
| Q2.5 | Crash and hang: a dead renderer shows a sad-tab page with Reload, and the other tabs go on. A renderer that does not answer for 10 s gets "page unresponsive", and Stop kills it (done, PR #324 open; left: Stop loses the page, and no rule stops reloads of a site that keeps crashing) | tests kill a renderer, and hang one with `while(1){}` | `chrome/browser/ui/sad_tab.cc`, `chrome/browser/ui/hung_renderer/` |
| Q2.8a | OS sandbox for renderers on macOS: a seatbelt profile with no file reads outside the bundle and no network (done, PR #327 open; left: wasmtime's code in the sandbox is untested, and `HOME` is still read for fonts) | a renderer that tries to open `~/.dweller` or a socket fails | `sandbox/mac/`, `sandbox/policy/mac/renderer.sb` |
| Q2.8b | OS sandbox for renderers on Linux: seccomp-bpf and namespaces. Executable memory stays allowed only for wasmtime (Q8) | the same test on Ubuntu | `sandbox/linux/seccomp-bpf/`, `sandbox/policy/linux/` |
| Q2.6a | Navigation and session history move to the browser. The renderer asks, and the browser decides and commits (done, PR #331 open; left: `location.replace()` pushes an entry, and an agent's same-site navigation commits without a history entry) | the `window_*` history and navigation tests pass with the flag | `content/browser/renderer_host/navigation_request.cc`, `navigation_controller_impl.cc` |
| Q2.6b | Process per site on navigation: a cross-site navigation swaps the renderer, and a same-site one reuses it. The site is scheme plus eTLD+1 (`net/psl.aye`) (done, PR #334 open; left: `window.open` children and frames are dropped with the old renderer on a swap) | a test goes a → b → a and checks pids and history | `content/browser/renderer_host/render_frame_host_manager.cc`, `content/browser/site_instance_impl.cc` |
| Q2.7 | Site-lock audit: every IPC message is checked against the renderer's site (cookies, storage, `file:`, `postMessage` origin, navigation targets). A violation kills the renderer. The checks were added by each step; this step lists them, closes the gaps and adds the tests (done, #338 into #334's branch; left: a cross-site frame in its parent's renderer gets no cookies or storage until Q2.9, and `blob:` documents have no site) | a test hook acts as a compromised renderer: it asks for another site's cookies and is killed | `content/browser/child_process_security_policy_impl.cc` (`CanAccessDataForOrigin`) |
| Q2.7b | An agent's cross-site navigation (navigate, click, submit, back, forward, reload) commits in a new renderer, and every renderer test hook needs `DWELLER_TEST=1` (done, #341 into #334's branch) | `window_renderer_agent_sites` | `content/browser/renderer_host/navigation_request.cc` (`OnResponseStarted`), `content/browser/renderer_host/render_frame_host_manager.cc` (`GetFrameHostForNavigation`) |
| Q2.7c | An agent's ref from before a navigation is refused after it, a request outstanding across a renderer swap gets an answer, and `dweller agent` follows a cross-site navigation into a new renderer (done, PR #342 open; left: `carryAnswer` has no test, and a cross-site eval or click is answered as a `navigate`) | `window_renderer_agent_refs` | `content/browser/devtools/protocol/dom_handler.cc` (idea only), `content/browser/renderer_host/render_frame_host_manager.cc` (`CommitPending`) |
| Q2.9 | Out-of-process iframes 1: a cross-site iframe gets its own renderer. The parent lays out a remote-frame placeholder, and the window composites the child's frame at its rect, clip and scroll | a cross-site iframe test matches its pixel golden | `third_party/blink/renderer/core/frame/remote_frame.cc`, `content/browser/renderer_host/render_frame_proxy_host.cc` |
| Q2.10 | OOPIF 2: hit testing sends input to the right process. Also focus traversal, nested scrolling, `IntersectionObserver` across processes, and resize | tests for click, keyboard focus, wheel and IO in a cross-site iframe | `components/viz/service/hit_test/`, `content/browser/renderer_host/render_widget_host_input_event_router.cc` |
| Q2.11 | OOPIF 3: cross-process window proxies: `postMessage`, `parent`, `top`, `frames`, `location` set, `opener`, `name`, and `focus()`/`blur()` | WPT `html/browsers/origin` and the `postMessage` subsets are at threshold | `third_party/blink/renderer/core/frame/remote_dom_window.cc` |
| Q2.12 | Spaces become renderers: an agent's Space uses the same renderer processes, and the second copy of the engine in `ui/spaces.aye` goes away. Shared logins go through the browser's jar | the `window_spaces*` tests pass | – |
| Q2.13 | Process limits and memory: a cap on processes, the reuse policy, and memory per tab against Chromium on the 20 sites. This sets the M1 budget | the report table in `PERF.md` | `content/browser/renderer_host/render_process_host_impl.cc` (`GetMaxRendererProcessCount`) |
| Q2.14 | `DWELLER_PROCESS=site` becomes the default. `--single-process` is kept for debugging only, as in Chromium | the full CI is green, and W7/W8 are re-measured | – |

## Q3: web security (lane sec)

Exit: X2 and X3 hold. The new WPT areas are in `docs/conformance.md`
with thresholds.

| Step | What | Verify | Chromium |
|---|---|---|---|
| Q3.1 | `file:`: a test first shows that an http page cannot fetch, iframe, navigate to or load a `file:` URL. Then the fix: the initiator scheme rule (done, #253) | the test passes | `content/browser/child_process_security_policy_impl.cc` (`CanRequestURL`) |
| Q3.1c | `net.Request.initiator` is an `Initiator` (`Browser` or `Page`) with no default, so every request names who asked (done, #254) | `aye check browser` | – |
| Q3.1b | Each `file:` document gets an opaque origin; the 3 tests that rely on one shared `file:` origin (`jsdom_fixtures`, `agent_sessions`, `storage_between_tabs`) move to served origins | those tests and `no_file_from_web` pass | `url/origin.cc` (file URLs are opaque) |
| Q3.2a | CORS enforced in `net/` for fetch and XHR, keyed on the initiator: a failed check is a network error, and page JS only types and filters. Non-simple cross-origin requests fail until Q3.2b | a two-origin test; WPT `fetch/api`, `xhr` do not drop | `services/network/cors/cors_url_loader.cc`, `services/network/public/cpp/cors/cors.cc` |
| Q3.2b | Preflight with a cache, `Access-Control-Expose-Headers`, and redirects in CORS mode | WPT `fetch/api/cors`, `cors/` and `xhr` gain, and `conformance.py check` passes | `services/network/cors/preflight_controller.cc`, `preflight_cache.cc` |
| Q3.3 | `crossorigin` on script, link and img, plus ORB (opaque responses that a no-cors fetch must not expose) (done, #286; module scripts in Q3.3b, #288; script events and `import()` credentials in Q3.3c, #304) | WPT `fetch/orb` and `html/cross-origin` subsets | `services/network/orb/` |
| Q3.4 | TLS: set peer and host verification, TLS ≥ 1.2 and the system CA store explicitly. A bad certificate gets an interstitial with no bypass on HSTS hosts | a local server with expired, self-signed and wrong-host certificates; each is refused | `net/cert/`, `components/security_interstitials/` |
| Q3.4b | A certificate-error bypass with a per-host exception store (none on HSTS hosts), and a wrong-clock hint on the expired page (done, #293; exceptions pinned to the accepted key and no HSTS over a bypass in Q3.4c, #296) | a test proceeds past a self-signed cert once and is asked again in a new profile | `components/security_interstitials/core/` |
| Q3.5 | HSTS (the header and the preload list), mixed content (active content blocked, passive content upgraded) and `upgrade-insecure-requests` (done, #282; the preload list is a 560-entry subset of the Chrome list) | WPT `mixed-content`, `upgrade-insecure-requests`; an HSTS test | `net/http/transport_security_state.cc`, `core/loader/mixed_content_checker.cc` |
| Q3.5b | Q3.5's rest: the WPT runner serves https so `mixed-content` and `upgrade-insecure-requests` are measured; `upgrade-insecure-requests` for the page's navigations and its frames; audio and video upgraded, not blocked (done, #300: `mixed-content` 53.3%, `upgrade-insecure-requests` 28.1%) | the two WPT areas in `docs/conformance.md` with thresholds | `core/loader/mixed_content_checker.cc` |
| Q3.6 | X-Frame-Options and CSP `frame-ancestors` (after Q2.6a, because the check needs navigation in the browser) | WPT `x-frame-options`, `content-security-policy/frame-ancestors` | `content/browser/renderer_host/ancestor_throttle.cc` |
| Q3.7 | CSP 1: parsing, `default-src` and `script-src` for external scripts, inline scripts with nonce and hash, and `eval` (done, PR #333 open: `content-security-policy/script-src` 31.6%; the rest is Q3.7b) | WPT `content-security-policy/script-src` | `services/network/public/cpp/content_security_policy/`, `core/frame/csp/` |
| Q3.7b | Q3.7's rest: module scripts, `'strict-dynamic'`, sha384/512, inline handlers, `javascript:` URLs, workers and srcdoc frames, the violation event's fields, and `<meta>` applying only to later elements | WPT `content-security-policy/script-src` gains | `services/network/public/cpp/content_security_policy/`, `core/frame/csp/` |
| Q3.8 | CSP 2: `style-src`, `img-src`, `connect-src`, `font-src`, `frame-src`, `base-uri`, `form-action` and report-only (no reports are sent) | WPT `content-security-policy` at threshold | the same |
| Q3.9 | iframe `sandbox`: every token (`allow-top-navigation*`, `allow-popups*`, `allow-forms`, `allow-modals`, `allow-downloads`, and the others) | WPT `html/browsers/sandboxing` | `core/frame/sandbox_flags.cc` |
| Q3.10 | `X-Content-Type-Options: nosniff`, and COOP (browsing-context-group switch, opener severed; after Q2.6b) | WPT `fetch/nosniff`, `html/cross-origin-opener-policy` subsets | `services/network/cross_origin_opener_policy.cc`, `content/browser/renderer_host/` (COOP) |
| Q3.11 | Small leaks: `Authorization` is removed on a cross-origin redirect, `window.name` is cleared on a cross-site navigation, top-level `data:` navigation is blocked, and private network access is blocked (public to local) (done, #298; left: PNA preflight, and PNA for frames, workers and documents) | one test each | `services/network/private_network_access_checker.cc` |
| Q3.12 | Agent socket: a `0700` directory, a `0600` socket and a token handshake. A process without the token is refused (done, #290; the socket moved to `~/.dweller/agent.sock`) | a test without the token is refused | – |
| Q3.13 | The HTTP cache keyed by the top-level site and the frame site (Chrome's network isolation key), and by credentials mode; the same for the connection pool | a test shows site B gets no cache hit for a response site A loaded | `net/http/http_cache.cc` (`GenerateCacheKey`), `net/base/network_isolation_key.h` |
| Q3.14 | Fetch Metadata (`Sec-Fetch-Site`, `-Mode`, `-Dest`, `-User`) on requests to potentially trustworthy URLs, `Origin` on POST navigations, and the page's referrer policy applied in `net/` (done, PR #313 open) | `fetch_metadata` | `services/network/sec_header_helpers.cc`, `net/url_request/referrer_policy.h` |
| Q3.14b | `referrerpolicy` on `a`, `area`, `iframe` and `script`, `rel=noreferrer`, `fetch(url, {referrerPolicy})`, a redirect's `Referrer-Policy`, and no `Origin` after a 303 (done, PR #319 open; left: the image cache does not key on the policy) | `fetch_metadata` | `core/html/html_anchor_element.cc`, `net/url_request/redirect_util.cc` |
| Q3.14c | `Sec-Fetch-Dest` for every destination (fonts, workers), `Sec-Fetch-User` only with transient activation, and WPT `fetch/metadata` (29.6%) and `referrer-policy` (49.2%) measured (done, PR #325 open; left: fonts load `no-cors`, not `cors`; activation is per page; in the tab process a frame's and a `_blank` tab's document fetch carry no referrer) | `fetch_metadata`; the two WPT areas in `docs/conformance.md` with thresholds | `services/network/sec_header_helpers.cc` (`SetSecFetchDestHeader`, `SetSecFetchUserHeader`), `third_party/blink/public/common/frame/user_activation_state.h` |

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
| Q0.4a | The react scroll regression of 922271a (train-1: P10.36b with P4.74 or P10.26): split it and fix it | react scroll p95 under 25 ms on `target/sites-old-20261017/react` | – |
| Q4.1 | Profile zol and bbc (`DWELLER_TIMING`, `DWELLER_LAYOUT_WHY`), and confirm the causes on github, mistral and youtube-search. Also profile the extra scripts of the re-recorded archives (Q0.4b: wikipedia-en and wikipedia (ja) scroll p95 under 33 ms; Q0.4c: go-docs's W7 row ok). This step only measures; it ends with cards ranked by ms | a table in `PERF.md` | – |
| Q4.2 | Shadow trees in subtree restyle (`css/invalidation.aye` `restyleSubtrees`, `RestyleDeps`) | the github and mistral `STYLE_FULL` counts fall, and `window_perf` rows improve | `core/css/style-invalidation.md`, `core/css/style_engine.cc` |
| Q4.3 | Invalidation sets for class, id and attribute changes (descendant and sibling); covers P3.64 slice 2 and P4.11 | the perf gate's `style_full` falls, and the W7 rows improve | `core/css/invalidation/invalidation_set.h`, `rule_invalidation_data.h` |
| Q4.4 | `:has()` invalidation without a whole-sheet restyle | the same | `core/css/invalidation/` (`:has` sets), `core/css/check_pseudo_has_*` |
| Q4.5 | Sheet insertion and change invalidate by rule (github: 3 of 7 forced layouts) | the github forced-layout time falls | `core/css/style_engine.cc` (`InvalidateForRuleSetChanges`) |
| Q4.6 | Why `relayoutDirty` refuses (mistral: 1301 times): fix the exits | the mistral refusals fall under 10 | `core/layout/layout_object.cc` (`MarkContainerChainForLayout`) |
| Q4.6a | `relayout:floats`: lay out again from the nearest formatting root that holds floats (done, #268: zol refusals 1304 → 0, full layouts 1430 → 179) | `window_relayout_floats` | – |
| Q4.6b | Attribute changes take the class/style path; `layoutReadsAttr` lists attributes layout reads (done, #271: `local:attr` 0 on mistral and zol) | `window_relayout_attr` | – |
| Q4.6c | `relayout:dropped` (zol 84): class toggles inside a fixed-position widget rebuild the whole `body` | zol `relayout:dropped` under 10 | `core/layout/layout_object.cc` (`ObjectIsRelayoutBoundary`) |
| Q4.7 | A layout result cache keyed by the constraint space | the W7 loop of bbc and youtube falls | `core/layout/layout_box.cc` (`CachedLayoutResult`), `constraint_space.h`, `layout_result.h` |
| Q4.8 | Finish the matched-properties cache (P4.75w) | the restyle ms on the W7 sites falls | `core/css/resolver/matched_properties_cache.cc` |
| Q4.9 | A shape cache by word (`layout/text-width.aye`; done, #276) | the loop of wikipedia-en, rustdoc and mdn falls under 100 ms | `platform/fonts/shaping/caching_word_shaper.cc` |
| Q4.10 | Image decode on a decode thread, with the pixels returned as a task, plus a decoded-image cache with a size budget (done, #279: two decode threads, 256 MB budget per document; the W7 measurement is left) | the bbc and zol loop falls, and `images` WPT does not drop | `platform/graphics/image_decoding_store.cc`, `cc/tiles/` (decode off the main thread) |
| Q4.11 | GC pauses: measure them on the W7 sites. If p95 is over 16 ms, add incremental marking | GC p95 stays under 16 ms on every site | `v8/src/heap/incremental-marking.cc` |
| Q4.12 | HTTP disk cache with revalidation and expiry, request priorities by kind, and the preload scanner checked. Includes Q0.4d: zol's live document load hits the 30 s timeout | live W8 does not drop, zol's W8 passes when Chrome paints under 5 s, and a revisit is served from cache | `net/http/http_cache.cc`, `core/html/parser/html_preload_scanner.cc`, `platform/loader/fetch/resource_fetcher.cc` |
| Q4.13 | `querySelector(All)`, `getElementById`, `getElementsByClassName` and `window` named access use id, class and name indexes kept by the DOM, and a cache of parsed selectors (Q4.1: zol ~2700 ms in `querySelectorAll`, ~700 ms in named lookups; done, #270) | zol's loop falls by those ms; `dom/nodes` WPT does not drop | `core/css/selector_query.cc` (fast paths, `SelectorQueryCache`), `core/dom/tree_ordered_map.cc`, `core/html/window_name_collection.cc` |
| Q4.14 | CSS animation and transition checks visit only elements that have animations or transitions (Q4.1: zol, youtube-search, 1357 ms; ~16% idle CPU; done, #275, and Q4.14b runs the hook once per rendering update, #277) | the idle CPU of those sites falls under 5% | `core/animation/document_animations.cc`, `core/animation/css/css_animations.cc` |
| Q4.15 | No raster when a frame has no damage, and only the damaged rect otherwise (Q4.1: mistral 2193 ms idle, react 47 ms per frame; #283: the window redraws only damaged rects and nothing without damage; scroll and the other overlays still redraw it all) | mistral's and react's idle CPU and react's scroll p95 pass | `cc/trees/damage_tracker.cc`, `cc/scheduler/scheduler.cc` (no frame without damage) |
| Q4.15b | Measure Q4.10 and Q4.15 together: bbc, zol, mistral and react W7 on main before and after; whether the idle raster of mistral was the window redraw or its canvas (done, #291: no W7 row moved; the idle raster of mistral is its canvas) | the numbers in `PERF.md` | – |
| Q4.16 | JS interpreter speed from the hottest bytecodes on bbc, youtube, wikipedia-en and go-docs: fused instructions, missing inline caches, native call cost (P4.75s, P4.75u). No JIT: a JIT needs executable memory, which the renderer sandbox (Q2.8) forbids (measured by Q4.16a, #303: mistral is natives and forced restyle, bbc is calls and inline-cache misses; the ranked cards Q4.16b–l are in its card) | bbc's script ms falls by a third; jsbench does not drop | `v8/src/interpreter/interpreter-generator.cc`, `v8/src/ic/` |
| Q4.16e | `Function.prototype.call` and `apply` on a bytecode function stay in the dispatch loop, with no VM re-entry (done, PR #329 open: bbc saves ~10 ms, not 250, see Q4.21; left: bound `call` and `CallUndefinedReceiver` still take the native path) | test262 `Function/prototype/call` and `apply` unchanged; jsbench 0.995 | `v8/src/builtins/arm64/builtins-arm64.cc` (`Generate_FunctionPrototypeCall`, `Generate_FunctionPrototypeApply`) |
| Q4.16g | Spec-correct `String.prototype.{match,replace,search,split}` and RegExp symbol methods with named groups, with a native fast path for a plain RegExp, so core-js keeps the natives on bbc; Q4.16g2 keeps the fast path when core-js moves `test` or swaps `constructor` (done, PR #314 open) | test262 574 → 599/669, none that passed fails; bbc's core-js polyfills 5 → 0 | `v8/src/builtins/builtins-regexp-gen.cc` (`BranchIfFastRegExp`), `v8/src/builtins/string-replace.tq` |
| Q4.16m | `RegExp(re)` returns `re` and `RegExp.prototype.test` calls `exec`, so core-js leaves `RegExp` alone on bbc (done, PR #326 open; bbc's longest loop did not move, 407 → 406 ms) | test262 RegExp and String folders 1191 → 1205/1343, none that passed fails | `v8/src/builtins/builtins-regexp-gen.cc` (`RegExpConstructor`), `v8/src/builtins/regexp-test.tq` |
| Q4.21 | Re-measure W7 on bbc, zol, mistral and react after Q4.16b/d/e/f/g/m and the aye inline bump (c43a9c6, O7.6, #323), and re-profile before choosing the next Q4.16 step: Q4.16e saved ~10 ms instead of the estimated 250 ms because core-js no longer replaces built-ins, so Q4.16a's estimates are stale | a dated W7 table and a ranked profile in `PERF.md` | – |
| Q4.17 | Canvas 2D drawing cost (Q4.1: mistral 1268 ms idle) (done, #294: a canvas with unchanged pixels is not damaged; mistral raster 3535 → 257 ms per 10 s) | mistral's idle CPU passes | `modules/canvas/canvas2d/canvas_rendering_context_2d.cc` |
| Q4.18 | What relayouts mistral each frame (done, #297: a same-text `textContent` write; it cannot be skipped because MutationObservers watch it) | – | – |
| Q4.19 | Unchanged frames reuse their display list (#301: mistral builds per frame 6 → 1, `paint_build` 4.0 → 1.8 ms) | `window_frame_lists_reused` | `platform/graphics/paint/paint_controller.cc` |
| Q4.20 | Canvas draws (2D and WebGL) mark paint only, not the DOM version; measure first on a canvas-only page | a canvas-only rAF page runs no relayout | `core/html/canvas/html_canvas_element.cc` (`DidDraw`) |

## Q5: basic features at Chromium parity (lane feat, after Q2.14)

Each step is one feature, with its WPT subset or a window test.
Ordered by daily use.

| Step | Feature | Chromium |
|---|---|---|
| Q5.1 | Zoom (Cmd +/−/0), per site, remembered | `chrome/browser/ui/zoom/`, `components/zoom/` |
| Q5.2 | `prefers-color-scheme: dark`, a dark UA sheet, and `color-scheme` | `core/css/media_query_evaluator.cc`, `core/css/resolver/` (color-scheme) |
| Q5.3 | `alert`, `confirm` and `prompt` as real dialogs (`prompt` returns null today) | `core/frame/local_dom_window.cc`, `chrome/browser/ui/javascript_dialogs/` |
| Q5.4 | IME composition events and the candidate window placed at the caret (CJK) | `core/editing/ime/input_method_controller.cc` |
| Q5.5 | `<input type=file>`: the picker and a multipart upload with files | `core/html/forms/file_input_type.cc`, `core/fileapi/` |
| Q5.6 | Downloads: `Content-Disposition`, `download` attribute and a downloads list | `components/download/`, `chrome/browser/download/` |
| Q5.7 | WebSocket (through the browser's network) | `modules/websockets/`, `services/network/websocket.cc` |
| Q5.8 | EventSource | `modules/eventsource/` |
| Q5.9a | `<video>` and `<audio>` 1: progressive playback of MP4 (H.264, AAC), WebM (VP9, Opus) and MP3, with controls. Decoding uses the system decoders in a utility process | `media/`, `third_party/blink/renderer/core/html/media/` |
| Q5.9b | Media 2: Media Source Extensions (YouTube) | `third_party/blink/renderer/modules/mediasource/` |
| Q5.10 | contenteditable and the core of `execCommand`; the async Clipboard API | `core/editing/`, `modules/clipboard/` |
| Q5.11a | IndexedDB 1: databases, object stores, transactions, in the browser process | `content/browser/indexed_db/`, `modules/indexeddb/` |
| Q5.11b | IndexedDB 2: indexes, cursors and key ranges | the same |
| Q5.12 | Permissions with a prompt: notifications, geolocation, camera and microphone denied until asked | `components/permissions/`, `modules/permissions/` |
| Q5.13 | Bookmarks and address-bar search (search engine, suggestions from history) | `components/bookmarks/`, `components/omnibox/` |
| Q5.14a | Service Worker 1: registration, lifecycle and fetch interception | `content/browser/service_worker/`, `modules/service_worker/` |
| Q5.14b | Service Worker 2: the Cache API | `content/browser/cache_storage/` |
| Q5.15 | Print to PDF and `@media print` | `components/printing/`, `core/paint/` (print) |

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
| Q6.1 | `css/backgrounds-borders` (30.1, 65) |
| Q6.2 | `css/css-grid` (43.2, 60) |
| Q6.3 | `css/css-flexbox` (62.8, 70) |
| Q6.4 | `css/box-sizing` (51.5, 70) |
| Q6.5 | `css/css-fonts` (32.7, 50) |
| Q6.6 | `css/cascade-variables` (56.1, 75) |
| Q6.7 | `css/values-color` (57.9, 70) |
| Q6.8 | `css/positioning` (61.7, 70), `css/overflow` (54.3, 60) |
| Q6.9 | `css/floats-tables` (55.5, 65) |
| Q6.10 | `html/semantics/forms` (54.3, 60) |
| Q6.11 | `url` (66.7, 80) |
| Q6.12 | `css/css-animations` (50.9, 60) |
| Q6.13 | test262 `language`, `built-ins`, `annexB` (73.4, 72.7, 41.3; 80) |
| Q6.14 | writing modes and bidi: a new area `css/css-writing-modes` with its baseline and threshold |

Areas can split into more steps. A step stops at its time budget with a
"left" list, as the P10 cards do now.

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
