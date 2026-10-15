# Plan: the Dweller browser lane

The browser lane of [`docs/plan.md`](../plan.md): the work on `browser/`.
**Maintenance mode since 2026-10-08:** no new site fixes or features.
The repository (`aye-lang/dweller`) is public and open source under MIT
OR Apache-2.0 since 2026-10-15; maintenance mode covers what the lane
works on, not who can read or contribute.
The lane keeps `cargo test --release` green (the browser tests are the
language's biggest integration test), fixes breakage caused by language
changes, and records language gaps in `browser/COMPILER_GAPS.md`. It
resumes when Aye criteria O6–O9 hold ([`aye.md`](aye.md)), in the same
step that moves `browser/` into its own repo (P8.2). Finished steps:
[`done.md`](done.md).

## Done criteria

**Freeze (now)**

- M1: `cargo test --release --test browser` exits 0 on `main` after
  every merge, whichever lane made it.
- M2: the wrap-ups are merged: `git merge-base --is-ancestor <branch>
  main` exits 0 for `worktree-agent-ae5e6f75817463272` (P3.48) and
  `worktree-agent-a0b64f7b014b9b6ea` (P3.58), and `done.md` lists both.
  (The user deletes branches and worktrees.)
- M3: every limitation of Aye or std the browser works around has an
  entry in `browser/COMPILER_GAPS.md` with **Hit by**.

**Browser done (after the resume)**

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

## Current state (2026-10-08)

- Sites: `page_load.py check` passes on the recorded archives; the
  screens review of 2026-10-08 found the visual defects now in the
  backlog (P3.50–P3.61). The screens review of 2026-10-14 (P1.14) put
  a `visual` verdict on every judged site in `sites.json`: `check`
  counts W3 at 4 of 20 (hn, cnn-lite, lobsters, google); the defects
  are P3.41, P3.51, P3.56, P3.59, P3.61 and P3.66–P3.73 (P3.65 done: W3 6 of 20; P3.73b done: W3 7 of 20). P3.50 is done:
  html.duckduckgo.com has no scripts and its server prints the raw date
  (`2026-07-20T00:00:00.0000000`); Chrome shows the same text, so the
  defect was not ours. `Date.parse` now takes fractions of any length. P3.69 and
  P3.73 are done (font weights, SVG image scaling, `noscript` sheets);
  P3.73b (CSS `filter`) is done too.
- Archives live in each checkout's `target/sites/` (untracked), so steps
  that replay sites or take screenshots run locally.
- P3.48 is not merged: the merge 34f6f93 passed `cargo test` but failed
  the site check (mdn FAIL, mistral 4 s → 40 s wall), so it was reset
  away. P3.48b bisects it. Its recaptured Mistral archive is in
  `.claude/worktrees/agent-ae5e6f75817463272/target/sites/mistral`.
- P3.58 is half done: WIP 34c2234 on `worktree-agent-a0b64f7b014b9b6ea`,
  not built (native `TypedArray` in `browser/js/typed.aye`, test
  `browser/tests/js/256_typed_array_views.js`).
- Open gaps the browser met: none (2026-10-14: gaps 49 to 56 fixed or
  declined, aye criterion O3; Dweller #1, 215e3a1).
- Maintenance steps done 2026-10-14 (squash commits on `main`): #1 215e3a1
  `COMPILER_GAPS.md` gaps 49 to 56 closed; #2 8546630 P-flaky
  `window_relayout_after_image_load`; #3 c13bc8b O1.17c `virtual_clock.html`
  reads the clock once per 1000 increments (`jsdom_gc_stress` about 140 s
  to 90 s); #4 aee0c41 P7 flaky `http_failed_image_fetched_once`; #5 55b4b48
  O1.14c `isDarwin`/`uname` replaced by `@targetOs()`, `clock()` declared
  `c_long`, `AYE_REV` a067dfd; #6 1198067 P8.x `browser/net/archive.aye`
  lists the archive directory with `std.fs.readDir`.

## Phases

| Phase | Goal | Exit check | Status |
|---|---|---|---|
| P0 | One clean `main` | – | done |
| P1 | "Done" checked by a command | `page_load.py check` | done (P1.6 in backlog) |
| PW | Wrap-up before the freeze | M2 | current |
| P7 | Maintenance: tests green while the language changes | M1, M3 after every language merge | next (ongoing until P8) |
| P8 | Split `browser/` into its own repo and resume | the browser repo builds against Aye with a path or URL dependency (O7) and W1 passes there | after O6–O9 |
| P3 | Sites correct (W2, W3) | `page_load.py check --agent` exits 0; `docs/browser-sites.md` rerun | paused (backlog) |
| P4 | Load times (W4) | `run --live --repeat 5` medians meet the targets | paused (backlog) |
| P5 | B8 "What remains" resolved (W6) | the list is empty or out of scope | paused |
| P6 | Release | manual checklist recorded; final site run; tag | paused |
| P9 | Agents without disturbing the user (B9, R1–R7) | the P9 tests pass: `window_agents`, `window_spaces*`, `agent_sessions` | P9.1 and P9.2 done 2026-10-14; P9.3 next |

## P8 split plan

P8.1 was a dry run (2026-10-11, no repo created). `browser/` was copied
to a scratch directory with nothing else of this repo around it.
`aye check`, `aye build` and `aye test` on the copy exit 0 (131 tests, 15
modules), and the `tests/browser.rs` harness, run from a scratch root
against the copy, passed 62 of 62 with one changed line.

**What broke out of tree.** Almost nothing:

- `browser/` has no symlinks, no import of `std/` or `framework/` files
  (std is built into the compiler), and no source-level path into the
  repo. The only `@cimport`s are system headers (`pthread.h`,
  `dirent.h` (gone since P8.x, 2026-10-14, Dweller #6), SDL2, FreeType, libcurl, HarfBuzz, brotli), found through
  `pkg-config` and `~/.cache/aye`.
- `aye build` named the binary after the directory (`browser`), not
  `dweller`. Fixed: `browser/aye.toml` has `name = "dweller"` and no
  `[deps]` (the browser imports no library besides std).
- `browser/build.sh` ran `cargo build` and `target/release/aye` from the
  parent directory, so it failed anywhere but the Aye repo. Fixed: it
  uses `$AYE`, else `aye` on the PATH, and runs cargo only when the
  parent directory is the Aye repo (`Cargo.toml` with `name = "aye"`).
- `browser/README.md` showed only the Aye repo build. Fixed: it shows
  the `aye build browser` and `AYE=... browser/build.sh` forms.
- `tests/browser.rs` is the one piece that still needs a change at split
  time (below): `aye()` uses `env!("CARGO_BIN_EXE_aye")`, which only
  exists in the Aye crate.

**Layout decision: keep the paths.** The new repo keeps the same
paths (`browser/...`, `tests/browser.rs`, `docs/browser.md`), so no path
in source comments, docs, `page_load.py`, `jsbench/run.sh` (they
`cd` three levels up and call `browser/build.sh`), `sites.json` or the
tests changes. `git subtree split` takes one prefix, so P8.2 uses `git
filter-repo --path` for several (as the backlog already said). Flattening
`browser/` to the repo root is a later, separate change.

**Files that move** (history kept):

- `browser/` whole: sources, `tests/` fixtures and goldens (including
  `tests/screenshot`, `tests/paint` PNGs and `tests/server.py`),
  `tools/page_load.py`, `tools/sites.json`, `tools/agent_tasks/`,
  `COMPILER_GAPS.md`, `README.md`, `aye.toml`, `build.sh`. There is no
  `scripts/page_load.py` and no tracked `sites/` directory: the site
  archives live in untracked `target/sites/`, so they do not move (copy
  them by hand when the lane resumes).
- `tests/browser.rs` and `tests/browser/perf.rs`.
- Docs: `docs/browser.md`, `docs/agent-protocol.md`,
  `docs/browser-sites.md`, `docs/browser-manual-checklist.md`,
  `docs/practices/browser.md`, `docs/plans/browser.md` (this file).

**What stays in the Aye repo** (changed in P8.3, after the new repo is
proven): `tests/common/mod.rs` (the new repo gets a copy; 79 lines, no
Aye internals), `docs/practices/workflow.md` and `aye.md`,
`scripts/setup-ubuntu.sh` (its comment names `browser/build.sh`; the
new repo gets its own copy of the apt line), `scripts/rename-aye.py`
(one-off, browser paths in its table), `README.md` (the Octane and
jQuery notices name `browser/tests/jsbench` and `jsdom/scripts`; the
notices move with the files), `CONTRIBUTING.md` (`browser/build.sh`),
`site/public/index.html` (a link to the new repo), comments in
`std/net/net.aye` and `std/http/http.aye` naming `browser/` files, and
`docs/decisions.md`, `docs/plan.md`, `docs/plans/aye.md` (mentions of
the browser as the largest program).

**How the new repo finds the compiler.** `aye` is a tool, not a library:
std is built in, so the browser has no `[deps]` entry. The repo pins the
compiler in a file `AYE_REV` (a commit of `aye-lang/aye`; the Aye repo
has no tags or releases yet) and installs it with `cargo install --git
https://github.com/aye-lang/aye --rev $(cat AYE_REV) --locked aye`.
Developers with a checkout of Aye set `AYE=/path/to/aye/target/release/aye`
(read by `browser/build.sh` and by the test harness). When the framework
(F4, P3.60) is split out, it comes in as `[deps] framework = { url =
..., }` in `browser/aye.toml` (O4.3); until then P8's exit check "builds
against Aye with a path or URL dependency" is met by the pinned
compiler, since a library dependency does not exist.

**How `tests/browser.rs` becomes the new repo's test.** It is a plain
Rust integration test: it runs `aye build browser -o <tmp>/dweller`
once, then spawns that binary on fixtures under `browser/tests` and a
Python `server.py`; it also runs `aye test browser`. The new repo gets a
`Cargo.toml` with no dependencies (package `dweller-tests`,
`publish = false`) and the three files above. One line changes: `aye()`
starts `std::env::var_os("AYE")` or `aye` instead of
`env!("CARGO_BIN_EXE_aye")`. `current_dir(root())` and every
`browser/...` path stay valid because the paths are kept. Run in a
scratch root with `AYE` set, this passed (62 tests, 191 s).

**P8.2 and later.** Run P8.2 when O6–O9 hold. It leaves the Aye repo
untouched; P8.3 removes the browser from it after P8.2 is verified.

#### P8.2: create `aye-lang/dweller` with history
- lane: browser
- goal: a new repo `aye-lang/dweller` holds `browser/`, its tests and docs with their git history, builds against a pinned Aye, and passes the browser tests in CI on macOS and Ubuntu.
- serves: W1 (and P8's exit check)
- files: in the new repo only: the moved files above, plus `Cargo.toml` (tests only), `AYE_REV`, `.github/workflows/ci.yml`, `scripts/setup-ubuntu.sh` (apt line copied), `README.md` at the root (one paragraph and the build commands), the one-line change in `tests/browser.rs`, and links fixed in `browser/README.md` (`../docs/practices/README.md` becomes `docs/practices/browser.md`)
- steps: (1) `git clone --no-local` the Aye repo to a scratch directory, then `git filter-repo --path browser --path tests/browser.rs --path tests/browser --path docs/browser.md --path docs/agent-protocol.md --path docs/browser-sites.md --path docs/browser-manual-checklist.md --path docs/practices/browser.md --path docs/plans/browser.md`; (2) copy `tests/common/mod.rs` and the apt line; (3) add `Cargo.toml`, `AYE_REV` (the current `main` of Aye), the `AYE` change and the CI file; (4) the user creates the repo and pushes (do not push or create it from an agent); (5) CI: matrix macOS and Ubuntu 24.04, install the SDL2, FreeType, libcurl, HarfBuzz, brotli packages and python3, `cargo install --git ... --rev $(cat AYE_REV) --locked aye` (cached), then `aye check browser`, `aye test browser` and `cargo test --release --test browser`.
- verify: in a fresh clone of the new repo with only the pinned `aye` installed, run one at a time: `aye check browser; echo $?` → 0; `aye test browser` → exit 0 and `131 passed; 0 failed; 15 modules` (or the then-current counts); `AYE=$(command -v aye) cargo test --release --test browser; echo $?` → 0 (62 tests); `aye build browser` writes `./dweller`; `git log --oneline --follow browser/engine/sheets.aye | wc -l` is the same as in the Aye repo; the CI run is green on both systems.
- depends on: O6–O9 hold; no `browser/` step in flight (merge them first; a branch cut before the split is rebased by `git filter-repo`'s commit map or redone by hand)
- parallel with: nothing in `browser/`; Aye-lane steps may continue
- where: local (the filter-repo run and the pinned install); the CI run is on GitHub
- model: sonnet for the files; opus for the filter-repo path list check
- time box: 90 min
- notes: P8.3 (separate card, after this one): in the Aye repo delete the moved files, fix the stay-side mentions listed above, and add a downstream check (`scripts/downstream.sh`: clone `aye-lang/dweller` at a rev recorded in `tests/downstream/ANYKNOWN_REV`, run `aye test browser` and the browser test with `AYE=target/release/aye`) that the merge agent runs after codegen, sema or std merges. That replaces M1, which `cargo test --release` no longer covers.

## P9: agents without disturbing the user

Design, measurements and acceptance checks R1–R7: `docs/browser.md`,
B9. The user asked for this on 2026-10-14; it is the one feature phase
allowed during the maintenance mode. Cards are ordered so that R1
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

#### P9.4: Space chips
- lane: browser
- goal: the toolbar's agent area shows "You" and one chip per Space (dot by state, the name; the status bar shows the last action while the mouse is over a chip); Pause/Resume and Stop act on the shown Space. No new row: nothing in the window changes height.
- serves: R6
- files: `browser/ui/chrome.aye`, `browser/ui/spaces.aye`, `browser/ui/agents.aye`, `browser/tests/agent/spaces/chips.*`, a golden PNG of the toolbar with two Spaces (as the B4 window PNGs)
- verify: `cargo test --release --test browser window_spaces > target/p9.4.log 2>&1; echo $?` → 0
- time box: 60 min; model: sonnet

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

## Cards: PW wrap-up

All wrap-up cards are done (`done.md`); the lane is frozen.

## Cards: P7 maintenance (next, recurring)

Main writes a P7.n card from this form when a merge breaks the browser
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

## Backlog

Paused until P8. Before any site item becomes a card, its symptom is
confirmed on current `main` (`docs/plan.md` rule 8). Source in brackets.

- P7 flaky `http_failed_image_fetched_once` under load: the failed image was requested twice (`GET /fail/once.png` ×2) in the full suite on main 0e61757 with three agents building; 3/3 alone pass. P3.23 removed a sleep; a second fetch still races. Same class on Linux arm64: `window_relayout_after_image_load` failed once in the full suite (main 71fd28a, an amd64 Rosetta run beside it), 3/3 alone pass. Diagnose step; verify 30× under load (merge-agent check) (main, 2026-10-08). Done 2026-10-14: an image's load/error fires in the task that finishes its background load (Dweller #2, 8546630); a dropped in-flight image load is held until it ends, so a failed URL is fetched once per document (Dweller #4, aee0c41).
- P3.63 Mistral costs ~40 s CPU (was 4 s) since P3.48: its real animations run, and each forced layout re-runs layout over the whole document; the inline-style subtree restyle (0d7dddf) helps little because the main page notes other changes between writes, and one iframe uses container queries (`g.styles.queried`). Needs incremental layout; first log what the main page notes between layouts (P3.48b/d, 2026-10-08). Done 2026-10-14 (diagnosed, one contained fix): the main page's 697 full cascades came from Tailwind `group-has-*` (`:where(.group\/toggle):has(:checked) *`) setting `RestyleDeps.whole`; the keys of a one-compound `:where()`/`:is()` now anchor the `:has()`, so they restyle a subtree. Mistral CPU 45.4 → 32.6 s, wall 46.3 → 33.0 s; Markdown and screenshot unchanged. Findings table and the incremental-layout design in `browser/engine/PERF.md` (Dweller, this PR).
- P3.64 incremental layout for forced reads: paint-only `style` writes skip layout; a text/size change lays out its subtree and walks up only while sizes change (design in `browser/engine/PERF.md`, P3.63). Mistral's remaining ~10 s of layout; its canvas raster (~10 s, `canvasComposite`) is separate. Done 2026-10-14 (slice 1): forced layouts keep the boxes when only values layout does not read changed (`css.paintOnly`), else build and lay out again the nearest block in block flow around each dirty element and walk up while its size changes (`browser/layout/incremental.aye`); floats, container queries, shadow trees and a walk to the root stay whole-document. `DWELLER_LAYOUT_CHECK=1` compares each with a full layout (DOM fixtures run with it; new `layout_incremental.html`). Mistral CPU 32.5 → 26.2 s, wall 33.0 → 26.4 s (main page script layout 4.8 → 1.2 s); zol, github unchanged; Markdown and screenshots unchanged (Dweller, this PR). Next slice (proposal): the visual-intelligence iframe's reads are now style work: restyle subtrees in a container-query document when no container's size changed, and skip the whole-document `stylesCurrent` hash for attribute writes (details in PERF.md).
- P8.2, P8.3 split `browser/` into its own repo with `git filter-repo`, move `tests/browser.rs` and the browser docs with it (decision 2026-10-08; P8.1 dry run done 2026-10-11; cards in "P8 split plan").
- P1.6 `page_load.py` writes `last.stdout`/`last.stderr` into the archive folder; move them under `target/page-load-out/`; set a floor for the 0.8 CPU/wall warning (P1.5). Done 2026-10-14: output is in `target/page-load-out/<site>/`; the warning fires only when wall >= 1 s (the browser's fixed non-CPU time of tens of ms is under 10% of wall there; a 0.6 s `example` replay had cpu/wall 0.16 with nothing wrong) (Dweller, this PR).
- P1.14 `visual` field per site in `sites.json` (ok / defects / date), printed by `check`; W3 needs it (screens review 2026-10-08). Done 2026-10-14: `check` prints a visual column, the defects and the W3 count; the screens review of 2026-10-14 (`check --screens` against the live pages in Chrome) filled the field.
- P3.7 Mistral: short iframe widgets, `setTransform` errors; recheck after P3.48 (old P3 table).
- P3.9 dump virtual clock: an `onload` set inside `setTimeout(0)` after `src` misses the load event (async images follow-up).
- P3.10 scripts run while parsing see an 800px viewport instead of `--width` (async images follow-up).
- P3.16 a script changing an img in another same-origin frame may record the failed load in the wrong page's cache (P4.6 follow-up).
- P3.19 URL parser accepts `http://[`; WHATWG IPv6 host parsing (P3.15).
- P3.21 fetch guards: immutable Response headers, forbidden Request header names, abort listener removed on settle (P3.15).
- P3.25 send `Origin` on non-GET and CORS-mode fetches (P3.17).
- P3.31 XHR `loadstart`/`progress` events and `xhr.upload` (P3.24).
- P3.32 a frame's `history.length` counts only its own entries; 1 during initial load (P3.29).
- P3.35 YouTube search agent task: expect results URL plus a title; recapture with the `youtubei/v1/search` POST (P3.30).
- P3.41 BBC ☰ menu button missing in replay; confirm, then diagnose (P1.4 screens). Still seen 2026-10-14: no menu/search button at the left, no Register/Sign In at the right.
- P3.42 Proxy invariant checks against the target (P3.39).
- P3.51 form controls: default-beveled buttons on Google, misaligned GitHub commit rows; confirm, then diagnose (screens review). Google buttons not reproduced 2026-10-14: Chrome draws the archived page with the same bevelled buttons (its CSS). Still seen 2026-10-14: GitHub commit messages about 10 px above the file names; also DuckDuckGo's query text at the top of its box and YouTube's search-button icon at the top of the button (content not centred vertically).
- P3.53 ellipsis and clamp follow-ups: right-to-left, `-webkit-line-clamp` into child blocks, unprefixed `line-clamp` (P3.52).
- P3.54 `grid-template` with area strings and the `grid` shorthand (P3.46).
- P3.56 `<video poster>` loaded and painted like an image; recapture wikipedia-en (P3.45). Still seen 2026-10-14 (empty video frame on wikipedia-en).
- P3.59 intrinsic sizing passes no container height (`browser/layout/intrinsic.aye`, `cb_h = -1`); BBC NEWS `<h1>` 300px instead of 100px (P3.47). Still seen 2026-10-14: the NEWS wordmark is 100 px left of centre.
- P3.60 move the JS lexer, parser and AST out of the `js` module for a bundler; pure move (framework F4, `docs/framework.md`).
- P3.61 `screenshot` runs the page as long as `dump` (Mistral's title animation caught half-way) (P3.48). Still seen 2026-10-14: "Introducing Mistral Large 4" not drawn, "Le chonk" 240 px low.
- P3.65 `opacity` not drawn on some text: example.com's script-added `p+p {opacity: .6}`, go.dev's `.Article-idLink {opacity: 0}` heading ¶ links (screens review 2026-10-14). Done 2026-10-14: paint only skipped `opacity: 0` on block boxes; other opacities and inline boxes were not drawn. Now a box with `opacity` below 1 is a layer (a stacking context, painted like a positioned box) drawn as a group: its items go into a layer bounded by their extent, blended below with the opacity (`EndLayer.alpha`); an inline box's items on a line are a group too; 0 draws nothing but keeps layout and hit testing (Chrome hits opacity-0 elements too). An inline `svg`'s own opacity is its box's (no longer applied twice). New fixture `browser/tests/paint/opacity.html`; example and go-docs match Chrome, W3 4 → 6 of 20 (Dweller, this PR).
- P3.66 Wikipedia (en, ja): the Appearance panel is empty; Chrome shows the Text, Width and Color radio buttons (screens review 2026-10-14).
- P3.67 MDN: the "Try it" demo shows an `about:blank` link instead of the code editor and output; the language switcher is drawn twice (screens review 2026-10-14).
- P3.68 Python docs: the sidebar is drawn right of the text; Chrome draws it on the left (screens review 2026-10-14).
- P3.69 bold CJK text drawn regular (Wikipedia ja lead term and notice box) (screens review 2026-10-14). Done 2026-10-14 (with P3.73): the CJK fallback (PingFang UI on macOS) is a variable face and was drawn at its default weight; variable faces now follow the CSS weight through their `wght` axis (one FreeType face per weight), and a bold font whose face is lighter than 600 gets synthetic bold (outline grown 1/24 em like Skia, advances kept), so Noto CJK Regular on Linux is bold too.
- P3.70 abehiroshi: the news column drawn below the photo instead of beside it, the tiled background image missing, menu bullets above their links (screens review 2026-10-14).
- P3.71 ZOL: 找产品 product names wrap and the rows misalign; the 新品首发/本月销量最高 tabs stack (screens review 2026-10-14).
- P3.72 react.dev: the "My video" demo thumbnail is an empty white box (Chrome: blue gradient with a play icon) (screens review 2026-10-14).
- P3.73 rustdoc: the Rust logo is a black blob; headings and sidebar titles drawn regular instead of Fira Sans Medium/bold; the copy-path icon is missing (screens review 2026-10-14). Done 2026-10-14: (1) "Fira Sans", "Inter", "Source Code Pro"... were in the list of system families, which ended the web-font lookup before the page's `@font-face` faces; an `@font-face` family now comes first (also MDN and Mistral's Inter, GitHub's Mona Sans). (2) An SVG image without a viewBox (the logo, 106 px) is scaled to its box (48 px) like a bitmap; it was drawn 1:1 and clipped. (3) `noscript.css` (inside `<noscript>`) applied while scripts run and hid `#copy-path`; sheets inside `noscript` now apply only with scripting off.
- P3.73b CSS `filter` is not drawn: rustdoc's copy-path and toolbar icons are black, Chrome draws them gray (`filter: invert(50%)`) (P3.73). Done 2026-10-14: `filter` is a paint-only property (kept as text, invalid lists dropped at parse time). A box (or an inline box's items on a line) with a filter is a group like opacity: drawn into its own layer, which grows by the reach of blurs and shadows, then `Item.Filter` applies the functions to the layer before `EndLayer` clips and blends it with the opacity. `invert`, `grayscale`, `sepia`, `saturate`, `hue-rotate`, `brightness`, `contrast` and `opacity` are the spec's color matrices on unpremultiplied sRGB; `blur` is three box blurs per channel (an approximation of the Gaussian; content outside the visible clip does not blur in); `drop-shadow` is the alpha moved, blurred by half its radius, colored, below the content; `url()` is ignored. New fixture `browser/tests/paint/filter.html` (pixels equal the spec matrices: invert(50%) 128, grayscale 118, sepia 165/147/114, hue-rotate(90deg) 50/146/35...). rustdoc's icons are #808080 as in Chrome; W3 6 → 7 of 20 (dweller, this PR).
- P4.1 react.dev: diagnose live vs replay time (DNS/TLS, discovery, module waits, HTTP/2) (P1.4).
- P4.2 react.dev: fix the cause found by P4.1.
- P4.3 ZOL live: separate discovery waits from transfer waits (`browser/engine/PERF.md`).
- P4.4 YouTube startup scripts, 0.84 s of interpreter time (P3.2).
- P4.5 decode offscreen images at paint time (`PERF.md`).
- P4.8 ZOL and Mistral replays spend most time in timers; re-measure with CPU time, then diagnose (P1.1).
- P4.10 container-query re-pass restyles only container subtrees (P4.9).
- P4.11 partial restyle for DOM mutations incl. shadow roots and `:has()` (P4.10).
- P4.14 event dispatch and MutationObserver from prelude JS into native Aye (~15% of JS samples) (P1.11).
- P4.18b Map/Set iterators are snapshots; make them live (P4.18).
- P4.21 GC threshold factor (GC 14.5% of JS samples) (P4.16).
- P4.24 smaller JS objects, property storage that shrinks, GC threshold relative to live size (P4.23).
- P4.28 slab for unshared computed-style arrays (P4.27).
- P4.29 `Prop` 48 → 32 B, accessors stored separately (P4.26).
- P4.30 decision: next large JS speedup is a bytecode compiler; design doc first, its own phase after P3 (P4.16, P4.17, P4.20).
- P5 one step per `docs/browser.md` "What remains" item, each tied to a listed site or declared out of scope.
- P6.1 manual checklist on macOS, recorded; P6.2 final `docs/browser-sites.md` run; P6.3 docs pass and a tag.

## Decisions

- **Maintenance mode (2026-10-08):** see the top of this file and
  `docs/plan.md`, "Standing decisions".
- **Renamed to Dweller and open-sourced (user, 2026-10-15):** the browser
  and its repository, formerly `aye-lang/anyknown`, are named Dweller
  (`aye-lang/dweller`, binary `dweller`, env vars `DWELLER_*`) and
  licensed MIT OR Apache-2.0; the repository is public.
- **Load-time target (W4):** per site, live median minus replay median
  ≤ 0.5 s over `run --live --repeat 5`; absolute targets in `sites.json`
  after P1.4 measured them.
- **P9 during the maintenance mode (2026-10-14):** the user asked for
  agent Spaces (B9); P9 is the one feature phase allowed while the rest
  of the lane stays in maintenance. A Space is a process, not a thread
  (B9, threading model).
- **Bot walls:** Stack Overflow and Reddit are excluded from W3 while
  they serve a bot wall; `sites.json` marks them `excluded`.
- **B8 remains:** an item no listed site needs is out of scope (P5).
