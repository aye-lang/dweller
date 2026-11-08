# Process model and site isolation

Design note for Q2 of `docs/plans/foundations.md` (2026-11-04). Steps
Q2.2–Q2.14 implement it. Chromium paths are in the M148 checkout.
Statements about Firefox are from memory, not from source, and are
marked "(FF, unchecked)".

## 0. Today

Today Dweller is one process, except for Spaces:
- The window (`ui/window.aye`) has the chrome, the SDL event loop, and
  the page state of the shown tab, all in one struct `Ui`: `tab`,
  `page` (pixels and damage), `list`, `focus`, `matches`, `hover`,
  `dropdown`, `wheel_el`.
- Each `Page` has its own JS engine and global. Frames are `Page`s on
  the same thread (`engine/frames.aye`). The window of another page is
  a stand-in object whose properties "ask the browser"
  (`engine/windows.aye`). That design is already proxy-shaped.
- `net/` fetches with blocking calls (`net.request`). The main thread
  drives one libcurl multi handle (`net/loader.aye`). The cookie jar,
  the cache and the profile are in the same process.
- A Space (`ui/spaces.aye`) is the whole window program again, headless,
  started with `posix_spawnp`. It has a `socketpair` with JSON control
  lines and a two-slot frame file in `/tmp/dweller-PID-XXXXXX/`. It
  starts in about 25 ms and takes about 45 MB.

The decision is in "Decisions" of the plan and is not open here.

## 1. What each process owns

| Process | Owns | Never has |
|---|---|---|
| Browser (one, not sandboxed) | the windows and SDL; chrome, tabs, address bar; session history and restore; the frame tree of every tab; the process table and the site locks; all network (libcurl, HTTP cache, TLS, 401 prompts); the cookie jar; the storage backends (`localStorage`, `sessionStorage`, later IndexedDB); the profile and its lock; permissions, dialogs, downloads, file pickers; the agent socket and the Spaces list; compositing and hit testing | page script, parsing of page bytes (HTML, CSS, images, fonts) |
| Renderer (many, sandboxed, locked to one site) | for each local frame: DOM, HTML parsing, style, layout, paint into shared memory, JS and its dedicated workers (threads), image and webfont decoding; replicas (proxies) of the remote frames of its browsing context group; the agent snapshot and actions of its frames | files, sockets other than its IPC socket, the profile, other sites' cookies or storage, HttpOnly cookies, other renderers' pixels |
| Utility (later, Q5.9a) | media decoding | a site lock (it gets bytes only), files, network |

Where the modules go:

| Module | Process | Note |
|---|---|---|
| `ui/window.aye`, `chrome.aye`, `tabs.aye`, `field.aye`, `sdl.aye`, `auth.aye` | browser | `window.aye` loses its page state (next row) |
| page half of `ui/`: `PageDraw` and `damage.aye`, `forms.aye`, focus, find matching, hover, wheel targets, the `select` model | renderer, as a new "page view" | the browser draws the `select` popup from a message |
| `ui/agents.aye`, `ui/spaces.aye`, `ui/socket.aye` | browser | the Space process goes (§8) |
| `agent/` (snapshot, refs, actions, eval) | renderer, per frame | the browser joins the parts of a tab |
| `net/` (http, loader, multi, cache, cookies, session, profile, archive, auth) | browser | the renderer uses only its types (`Request`, `Response`, `Kind`) and calls `ipc/` instead |
| `net/psl.aye`, `url/`, `std`, `encoding/`, `perf/` | both | `perf` lines carry the role and pid |
| `engine/` (pages, DOM API, scripts, local frames, proxies) | renderer | `engine/storage.aye` keeps a cache per origin; saving moves to the browser. `engine/tab.aye` `SessionHistory` moves to the browser |
| `html dom css layout paint font image svg webfont animation js wasm gpu audio` | renderer | `paint` and `font` also in the browser, for the chrome and compositing |
| new `ipc/` | both | framing, codec, shared buffers, fd passing |
| new `proc/` | browser | spawn, lock, reuse, cap, kill, the policy checks, crash and hang |

## 2. IPC

**Transport.** As Spaces do now: the browser makes a
`socketpair(AF_UNIX, SOCK_STREAM)`, its end gets `FD_CLOEXEC` and
`SO_NOSIGPIPE`, and it starts the same binary with `posix_spawnp`
(`dweller --renderer FD`). It does not fork, so the browser's
memory is not copied. One socket per renderer carries every message,
so all messages of a renderer are in one order. Pixel buffers, and
later chosen files and fonts, go as fds with `SCM_RIGHTS` (`sendmsg`
and `recvmsg` externs, new in Q2.2). The browser makes the buffers
(Linux `memfd_create`; macOS `shm_open` then `shm_unlink` at once), so
a renderer opens no file. This replaces the frame file of P9.5, which
a sandboxed renderer could not open.

**Framing.** Binary. A 16-byte header: payload length (u32), type
(u16), flags (u16: reply, sync), frame id (u32, 0 for the process),
request id (u32, 0 for none). The payload has the fields of the type in
a fixed order: u32, u64 and f64 little-endian, bytes as a u32 length
and the data. One `ipc.Reader` checks the bounds of every read. These
are bad messages (§4): a read past the end, bytes left over, an unknown
type, a payload over 1 MiB (body chunks: 256 KiB). `DWELLER_IPC_LOG=1`
prints each message as one text line on stderr. Type codes are numbered in the order the messages are added (Q2.2a: 1 `hello`, 2 `navigate`, 3 `loaded`, 4 `shutdown`; `ipc/message.aye`); flags and request id stay 0 until a message uses them. Why not the JSON lines
of Spaces: bodies are binary (base64 costs a third more), and the
browser parses what a compromised process writes; a fixed binary codec
has less to get wrong than a JSON parser. JSON stays for the agent
protocol.

**Rules.**
- The browser never waits for a renderer. Its sockets do not block,
  each renderer has an out buffer (as `ctl_out` now), and it reads
  only what is ready.
- An IO thread in the browser `poll`s the renderer sockets and pushes
  an SDL user event to wake the main loop. All handling stays on the
  main thread. Today's `POLL_MS` (15 ms) is too slow for frames and
  input acks.
- Only a renderer makes sync calls (flag `sync`). While it waits for
  the reply, it reads and queues the other messages. The browser answers
  a sync call from its own state, never by asking another renderer, so
  no cycle can deadlock.
- Frame ids are global u32s that the browser gives. A message's frame id
  must be a frame that this process hosts (or, for proxy messages, a
  proxy that it holds).

**Messages** (B = browser, R = renderer; "(s)": a sync call):

| Group | Messages |
|---|---|
| lifecycle | R→B `hello{version, pid}` (the first message, frame 0; the browser checks the version and that the pid is the one it spawned); Q2.2a only, until Q2.6 brings the navigation group: B→R `navigate{url}` (frame = the tab), R→B `loaded{title, url}` (the renderer loads the page itself and runs its scripts and timers); `font` and `js` of `init` go as spawn arguments for now (`--font fixed|system`, `--no-js`, Q2.2c); B→R `init{lock, partition, scale, font, js, flags}`; B→R `shutdown` (SIGKILL after 1 s, as for Spaces); B→R `visible{frame, on}` (hidden: timers throttled, no frames); B→R `memory_pressure`; Q2.5: B→R `ping{n}` (frame 0, at most once a second, one open at a time), R→B `pong{n}` (the same `n`, else `bad pong`) |
| navigation | R→B `begin_navigation{frame, url, method, body, referrer, target, gesture, replace}` (links, forms, `location`, `window.open`); B→R `commit_navigation{frame, nav, url, origin, status, headers, entry}`, then `body_data{nav, bytes}` and `body_done{nav}`; R→B `did_commit{frame, nav, url, origin}`; R→B `same_document{frame, url, state, replace}` (pushState, fragments); B→R `navigate{frame, entry}` (back, forward, reload); B→R `before_unload` and `unload{frame}`, with replies; R→B `loading{frame, state}`, `title{frame, text}`; Q2.3d has R→B `title{text}` and `same_document{url}`, and the browser kills a renderer whose `loaded` or `same_document` URL is not the one it navigated to, the same document, or the same origin (`bad url`) |
| frames | B→R `buffers{frame, w, h}` with 2 fds; R→B `frame{frame, buffer, seq, damage[], children[], over?}` (§5); B→R `frame_ack{frame, buffer}`; B→R `resize{frame, w, h, scale}`; R→B `child_size{child, w, h}`; B→R `viewport_intersection{frame, rect}` |
| input | Q2.3a: B→R `mouse{seq, kind (0 move, 1 down, 2 up, 3 leave), x, y, mx, my, button, buttons, mods, clicks}`, `wheel{seq, x, y, dx, dy, precise}` (CSS px, coalesced per loop iteration; x = y = -1 when the mouse is not over the page), `key{seq, down, key, code, repeat, location, mods}` (DOM `key` and `code`), `text{seq, text}`, all in the tab's viewport coordinates; R→B `input_ack{seq}` (every input up to `seq` handled and its frame sent; later `consumed, dx, dy` for scroll that bubbles up); Q2.3d: B→R `focus{on}` (the page has the window's focus or not: the window lost it, or the toolbar took it; the renderer hides the caret and stops its blink); Tab, the focus ring, the caret and the selection of text fields are the renderer's; R→B `cursor`, `status_url`, `focus_changed{frame}`, `advance_focus{frame, dir}`; B→R `focus{frame, dir}`; B→R `hit_test{frame, x, y}`, R→B its reply; B→R `find{text, dir}`, R→B `find_reply{count, active, rects}`; B→R `edit{copy, paste, select_all}`; R→B `popup{frame, rect, items}`, B→R `popup_choice`; R→B `long_task{ms}` |
| network | R→B `load{id, frame, url, method, headers, body, kind, mode, credentials, initiator}`; B→R `load_response{id, status, url, mime, charset, headers}`, `load_data{id, bytes}`, `load_done{id, error}`; R→B `load_cancel{id}`; R→B (s) `load_sync{...}` (sync XHR, and today's blocking `net.request` callers until they become async) |
| cookies, storage | R→B (s) `cookie_get{frame, url}`; R→B `cookie_set{frame, url, line}`; R→B (s) `storage_open{frame, kind, origin}` returns every item; R→B `storage_set`, `storage_remove`, `storage_clear`; B→R `storage_event{origin, kind, key, old, new, url}` to the other renderers of the origin |
| frame tree, proxies | R→B (s) `frame_created{parent, name, sandbox}` returns the id; R→B `frame_detached{frame}`; B→R `proxy_create{frame, parent, opener, origin, name, sandbox}`, `proxy_update`, `proxy_delete`; B→R `swap_out{frame}` (the frame now lives in another process: a proxy replaces it); R→B `post_message{from, to, source_origin, target_origin, data}`, B→R `message{to, from, origin, data}`; R→B `proxy_navigate{to, url}`, `proxy_focus`, `proxy_blur`, `proxy_close`, `opener_cleared`, `name_changed` |
| dialogs, permissions | R→B (s) `dialog{frame, kind, text, default}` (alert, confirm, prompt, beforeunload); R→B `permission{frame, kind}`, B→R `permission_result`; R→B `file_chooser{frame, accept, multiple}`, the reply with read-only fds (Q5.5); R→B `download{frame, url}`; R→B `clipboard_write`, (s) `clipboard_read`; R→B (s) `font_open{name}` (Linux, §6) |
| agent | B→R `agent{id, frame, request}`; R→B `agent_reply{id, json}`; R→B `agent_event{frame, json}` (loads, console, dialogs) |

Sync access between windows (`frames.length`, `parent`, `top`,
`closed`, named access) is answered from the replica of the frame tree
in the renderer, with no IPC. Only `postMessage`, `location` set,
`focus()`, `blur()` and `close()` on a remote window cross processes,
and all of them are async (Chromium: `RenderFrameProxyHost` and
`RemoteFrame`, `remote_dom_window.cc`).

## 3. Sites, locks and reuse

**Site**: the scheme and the registrable domain, from `net/psl.aye`
`siteOf`. Its list has the private section (`github.io`), as Chromium's
`INCLUDE_PRIVATE_REGISTRIES`. The port is not part of it (Chromium
`SiteInfo::GetSiteForOrigin`, `content/browser/site_info.cc`). So
`http://a.com` and `https://a.com` are two sites, and `a.com:8000` and
`a.com:8001` are one. A host without a registrable domain (an IP
address, `localhost`) is its own site. Dweller has no `document.domain`
setter, so a site lock is coarser than needed. It stays site because
cookies are scoped by site.

| URL | Process | Chromium |
|---|---|---|
| `file:` | one lock `file:` for every file page; web pages never commit there; each file document has an opaque origin (Q3.1) | the same |
| `about:blank`, `about:srcdoc` | the creator's process (inherited origin) | the same |
| `data:` | the creator's process (opaque origin, the creator as precursor); top-level `data:` from the web is blocked (Q3.11) | the same |
| `blob:` | the site of the creator's origin | the same |
| sandboxed iframe without `allow-same-origin` | the parent's process in v1 | its own process on desktop since M127 |
| the browser's pages (error, sad tab, interstitial) | drawn by the browser | an error-page process |

**Lock.** It is set before the first byte of content commits, and it
never changes while the process lives (Chromium `ProcessLock`). Only a
spare process is unlocked.

**Browsing context group** (BCG): a tab and the windows that it opened
with an opener (Chromium `BrowsingInstance`). In one BCG each site has
exactly one process, because its pages can script each other
synchronously. This is a rule, not a heuristic. A new BCG starts with
`noopener`, COOP (Q3.10), or a cross-site navigation that the person
started (address bar, bookmark).

**Reuse**, in this order:
1. The same BCG and the same site: its one process (required).
2. A cross-site iframe: any process of that site and partition, in any
   BCG; else a new one. Chromium does this "aggressive reuse" for
   subframes.
3. A top-level page in a new BCG: a new process while under the cap.
   Over the cap: a random process of the same site and partition. If
   there is none, a new process all the same.
4. Never two sites in one process, and never two partitions (the
   person's profile and an `isolated` Space) in one process.

**Cap** (soft, as Chromium). Chromium: half the RAM divided by 85 MB,
clamped to [3, half the OS process limit, or 82 when unknown]
(`render_process_host_impl.cc` `GetMaxRendererProcessCount`); 16 GB
gives 96. When the cap is hit, Chromium reuses a random same-site
process, and else goes over the cap. Dweller uses the same formula with
its own cost per process (Q2.13 measures it; a Space is about 45 MB
now). Firefox instead allows a few processes per site
(`dom.ipc.processCount.webIsolated`) (FF, unchecked).

**Stages.** `tab`: one unlocked process per BCG, no OOPIF, partition
checked. `site`: locks, rules 1 and 3. OOPIF: rule 2.

## 4. The site-lock checks

The browser knows each frame's committed origin from its own
`commit_navigation`, and it checks every origin that a renderer states.
`canAccess(process, origin)` is true when the site of the origin (of its
precursor, for an opaque one) is the process's lock. This is Chromium's
"jail" check (`ChildProcessSecurityPolicyImpl::CanAccessDataForOrigin`,
then `PerformJailAndCitadelChecks`). The "citadel" check (an unlocked
process may not touch sites that need a lock) has nothing to protect in
Dweller once every process is locked. In stage `tab` no check is
possible.

| Message | Check | Failure |
|---|---|---|
| every message | well formed, known type, the frame id hosted by this process | kill |
| `did_commit` | the origin is the one sent in `commit_navigation` | kill |
| `same_document` | the new URL has the frame's origin | kill |
| `load`, `load_sync` | `initiator` is the frame's committed origin (Chromium `request_initiator_origin_lock`); no `Host`, `Cookie`, `Origin`, `Sec-*` header | kill |
| `load`, `load_sync` | the scheme is allowed for the initiator (`file:` only from `file:`, Q3.1) | a network error (a page may try) |
| `cookie_get`, `cookie_set` | `canAccess` for the URL's origin, which is a frame's origin | kill |
| `storage_*` | the origin is the frame's committed origin | kill |
| `post_message` | `source_origin` is the sender frame's origin (Chromium kills with `RFPH_POST_MESSAGE_INVALID_SOURCE_ORIGIN`) | kill |
| `post_message` | `target_origin` is the target's current origin | drop (a navigation race, not an attack) |
| `proxy_*` | the sender holds a proxy of that frame (same BCG); the sandbox flags allow it (Q3.9) | kill / ignore |
| `frame` | the buffer index and size are the ones the browser gave | kill |
| `agent_reply` | the request id is open for this process | kill |
| `dialog`, `permission`, `file_chooser` | the origin is taken from the browser's tree, never from the message | – |

The browser also limits what it sends: no `Set-Cookie` header goes to a
renderer, no HttpOnly cookie is in `cookie_get`, a response goes only to
the process of its frame, and ORB (Q3.3) runs in the browser. Dweller
blocks third-party cookies, so a cross-site subresource never carries
cookies; ORB still matters for responses that are authorized in other
ways (an intranet, cached HTTP auth). `X-Frame-Options` and
`frame-ancestors` are checked in the browser at commit (Q3.6).

**Kill.** The browser logs `bad message: REASON pid site`, sends
SIGKILL, and the frames of the process go to the crash path (§7).
Chromium `bad_message::ReceivedBadMessage` does the same. The Q2.7 test
hook `DWELLER_TEST_EVIL=cookie|storage|commit|message|load` makes a
renderer send one forged message; the test checks that it is dead and
that the other tabs live.

## 5. Compositing and hit testing

A renderer paints each of its local roots (a main frame, or an OOPIF
with its same-site descendants) into a buffer in shared memory, as a
Space does now. Each local root has two buffers, made by the browser.
The renderer writes a buffer only after its `frame_ack`. P9.5 has two
slots and no ack, so the window can copy a slot while the Space writes
it.

A `frame` message has the damage rects, and for each remote child: its
frame id, its rect (the iframe's content box, in the frame's device px),
its clip (the ancestors' overflow clips and the viewport), its opacity,
and flags (transformed, `pointer-events: none`, covered). The parent
paints nothing in a remote child's rect: a remote-frame item in its
display list, like Blink's `RemoteFrame` and `RemoteFrameView`. When
the parent paints something above a child (a fixed header, a modal over
an embed), it paints those items into a second "over" buffer, which
exists only then.

The window composes a tab from the top: the parent's buffer, each child
clipped at its offset and opacity (recursively), then the parent's over
buffer. It applies translation, clip and opacity only. A rotated or
scaled iframe is drawn at its bounding box without the transform. So is
parent content between two overlapping sibling iframes. Both are known
gaps; Chromium composites any transform in viz, and Q8 (P11.22) is the
place for it.

**Scrolling.** A scroll of the parent moves the child's rect in the
parent's next frame. The window moves the child's last pixels without
asking the child. A child scrolls its own content in its renderer. The
damage is the parent's damage, the child's damage (moved and clipped),
and a moved child's old and new rects. The window draws only that, as
`damage.aye` does in one process.

**Sizes.** The parent's layout sets the child's size: `child_size` goes
to the browser, which sends `resize` to the child's process. Until a
frame of the new size comes, the window shows the old frame clipped, or
white. The clip also gives `viewport_intersection`, which feeds
`IntersectionObserver` and throttles hidden OOPIFs.

**Hit testing**, in the browser. From the last frame of each local
root, it walks the children in reverse paint order. A point in a
child's clipped rect goes to that child when no flag is set, in the
child's coordinates. When a flag is set (transformed, covered by the
over buffer, `pointer-events: none`), the browser asks the parent's
renderer (`hit_test`) and queues input until the reply. This is
Chromium's fast path and slow path (`components/viz/service/hit_test/`,
`components/input/render_widget_host_input_event_router.cc`). A mouse down captures its
target until the mouse up. Scroll that a wheel target does not use
(`input_ack`) bubbles to the parent's process. Keys, text and IME go to
the focused frame (`focus_changed`). Tab focus that leaves a frame
(`advance_focus`) goes to the next frame in the browser's tree.

## 6. Sandbox

A renderer may: use its socket, map the fds that it gets, allocate
memory, make threads (workers, Q4.10 decoding), read clocks, read font
files, exit. It may not: open any other file (`~/.dweller`, `/tmp`),
make a socket or connect, exec, fork, ptrace, signal another process,
talk to the window server or the GPU. It enters the sandbox in `dweller
renderer` after `init` and the font preload, before the first commit.
It never starts SDL.

**macOS: seatbelt**, with `sandbox_init_with_parameters` as Chromium
does (`sandbox/mac/seatbelt.cc`; a private API, stable for years). The
profile has the shape of `sandbox/policy/mac/common.sb` and
`renderer.sb`:
- `(version 1)`, `(deny default)`.
- Read: the executable and its directory (parameters); the system
  libraries that dyld needs; the font directories (`/System/Library/
  Fonts`, `/Library/Fonts`, the user's); `/dev/urandom`; basic sysctls.
- Write: `/dev/null` only.
- No `network*` (Chromium allows only the syslog socket), no
  `process-fork`, no `process-exec`, and no `mach-lookup` at all if
  possible. Dweller reads font files with FreeType, not CoreText, so
  Q2.8 can try an empty mach list. Q2.8 also checks what wasmtime's
  machine code needs.

**Linux.** Layer 1, required: seccomp-bpf after
`prctl(PR_SET_NO_NEW_PRIVS)`.
- Allowed: `read`, `write`, `readv`, `writev`, `recvmsg`, `sendmsg` on
  open fds, `poll`, `close`, `fstat`, `lseek`, `mmap`, `munmap`,
  `mprotect`, `madvise`, `brk`, `futex`, `clone` with thread flags
  only, `exit`, `exit_group`, clocks and sleeps, `getpid`, `gettid`, signal masks and returns,
  `sched_yield`, `getrandom`.
- `open` and `openat` fail with `EPERM`, so a stray lazy open fails
  softly. `socket`, `connect`, `execve`, `fork`, `ptrace` and `kill` of
  another process kill the renderer (`SIGSYS`).
- One syscall table each for x86_64 and aarch64.
- `PROT_EXEC` stays allowed. The JS has no JIT, but WebAssembly runs on
  wasmtime, which writes machine code.

Layer 2, best effort: `unshare` of the user, net and IPC namespaces,
and a chroot to an empty directory. Ubuntu 23.10 and later can forbid
unprivileged user namespaces (AppArmor). Then the renderer runs with
seccomp only and prints one warning. Chromium uses both layers
(`sandbox/linux/seccomp-bpf/`, the namespace sandbox). Fonts: the
renderer opens the fixed list of `font/freetype.aye` before it enters
the sandbox. A fallback font that it needs later comes as an fd from the
browser (`font_open`), as Chromium's font IPC does
(`docs/linux/sandbox_ipc.md`).

The utility process gets the same profile without the fonts.

## 7. Crash, hang and session restore

**Crash**: the end of file on the socket, then `waitpid`. For each frame
of the process: a main frame shows the sad-tab page (drawn by the
browser) with Reload; an OOPIF shows a sad frame in its rect, and its
parent goes on; the other renderers get `proxy_update` (gone). Open
agent requests fail with "renderer crashed". With same-site reuse one
crash ends every tab of that process; Chromium is the same. After three
crashes of one site within 30 s, Reload is not automatic.

**Hang** has two signals:
- An input that is not acked within 5 s (Chromium's hung renderer).
- A shared page with two words per renderer: its loop writes the time
  on each iteration; the time is more than 5 s old while it has work.

Then the "page unresponsive" bar of Q1.7 shows. Stop sets the second
word, the interrupt flag. The VM checks it on back-edges and calls (the
flag of Q1.7, moved to shared memory), and it throws the uncatchable
termination. A renderer that does not react within 2 s gets SIGKILL and
takes the crash path. So the browser can stop a script although the
renderer's only thread does not read its socket.

Q2.5 ships the first half: the hang signal is a `ping` the renderer has
not answered for 10 s (`DWELLER_HANG_MS`), and Stop is SIGKILL, so the
tab shows the sad tab "This page was stopped". The interrupt word in
shared memory, which would keep the page, is not built yet.

**The browser ends** (quit or crash): each renderer reads the end of
file and exits, as a Space does now.

**Session restore.** The session is the browser's: tabs, and history
entries (URL, title, site, serialized state, scroll, form state).
Renderers send `page_state{frame, scroll, form}` when the frame
navigates away and at most once a second (Chromium `PageState`). The
browser writes the session atomically (`net/profile.aye`
`writeAtomic`). A renderer crash does not change the session. At
startup only the shown tab loads; the others load when first shown.
Q1.9 (session restore, one process) should already save this shape, so
that Q2.6 moves the code without changing the file.

## 8. Spaces become renderers (Q2.12)

Today a Space is a full copy of the window program, with its own tabs,
engine and jar. A `shared` Space gets the whole jar and every origin's
`localStorage` in its `share` message, and it sends cookie and storage
lines back (`ui/share-with-space.aye`, `ui/apply-shared.aye`). Under
site isolation that gives every site's cookies to one process, so it
goes.

After Q2.12:
- A Space is a group of tabs in the browser with an agent as owner. Its
  tabs are not in the person's tab strip. Its frames run in ordinary
  renderers, under the same locks and reuse rules.
- The partition is the person's for `shared`. For `isolated` it is a
  jar and storage of its own in the browser, and its processes are
  never shared with the person's.
- The browser keeps the fields of today's `struct Space`: id, name,
  state, last action, paused, taken tabs, watched.
- The agent connection stays in the browser (`ui/agents.aye`). The
  browser sends each command to the renderer of its target frame
  (`agent`). It knows the loading state, so it serves `wait` itself.
- Watching shows the Space's tab as any tab is shown; the frame file
  goes. Taking over sends input to that tab's renderer.

What changes for agents (`docs/agent-protocol.md`):
- Refs stay as they are. The browser joins the snapshot parts of a
  tab's renderers and renumbers their refs into one list, so a ref
  still names one element of the tab. It keeps the map from the ref to
  the frame and the renderer's ref.
- A Space no longer crashes as a whole. A renderer crash fails the
  requests on its tabs ("renderer crashed"), and `tabs` shows that tab
  as `crashed`. The `crashed` state of a Space goes.
- `eval` runs in the frame's renderer. It cannot reach into a
  cross-site frame, as in a real page.
- The internal `dweller space --control FD` command goes. The agent
  socket stays (its token: Q3.12).

## 9. Risks and open questions, ranked

1. **`Ui` mixes the chrome and the page.** Every later step touches it.
   Recommendation: a new step Q2.1b splits it in one process into the
   window and a page view that talk through the messages of §2 as
   function calls, with all tests green, before any process work.
2. **Navigation and history move to the browser.** The engine fetches
   documents itself, loads frames inline (`loadFrames`), and keeps live
   pages in history. Recommendation: split Q2.6 (below). In v1 there is
   no back-forward cache across processes: a cross-site back loads the
   page again.
3. **Tests across processes.** The Spaces tests are the flakiest today.
   Recommendation: the headless `--frames` driver counts a frame only
   after every renderer acked it, and `--until-idle` waits for every
   renderer's idle. No sleeps in tests. `DWELLER_IPC_LOG` in the logs
   of a failed test.
4. **Latency and deadlock of sync calls.** Recommendation: only
   renderer-to-browser sync calls, answered from the browser's state;
   the IO thread wakes the loop. Q2.3 measures input to frame; the
   budget is 2 ms p95 over one process.
5. **Linux sandbox availability.** Recommendation: seccomp required,
   namespaces best effort, and a CI test that seccomp is on.
6. **Compositing gaps** (transforms, interleaved z-order).
   Recommendation: the under and over buffers; translation, clip and
   opacity; the gaps listed in the Q2.9 card.
7. **Memory.** Recommendation: measure in Q2.13. The binary and the
   font files are shared by the OS page cache; a spare process only if
   the start time shows in W8.
8. **Open: sandboxed frames in a process of their own?** Recommendation:
   not in v1; after Q2.11 if it is cheap.
9. **Open: `--single-process`.** Recommendation: the same interfaces,
   bound to direct calls (no IPC, no checks), for debugging and for
   `dump` and `screenshot`. CI runs one smoke test in it.

**Changes to the Q2 step list:**
- Add **Q2.1b** before Q2.2: the in-process split of `ui/window.aye`
  (risk 1). Without it, Q2.2 is a refactor and a new process in one PR.
- **Q2.2** starts from the headless Space path (it already runs one
  tab's timers, waits on sockets, draws its page area into shared
  memory, and takes input words), and adds the binary framing, the
  buffers by fd with acks, and the IO thread.
- Add **Q2.3b** after Q2.3: agent commands on the window's own tabs go
  to their renderer. The `window_*` and agent tests need it with
  `DWELLER_PROCESS=tab`, long before Q2.12.
- **Q2.4** also adds `proc/policy.aye` with `canAccess` and the checks
  of the messages it adds. Each later step adds the checks of its own
  messages. **Q2.7** then becomes the audit of the §4 table and the
  compromised-renderer tests. Messages without checks never exist.
- Split **Q2.6**: **Q2.6a** navigation and session history in the
  browser (`begin_navigation`, `commit_navigation`, still one process
  per tab); **Q2.6b** the process swap on a cross-site navigation, the
  BCG rule and the special URLs of §3.
- Move **Q2.8** before Q2.6. Once the network is in the browser
  (Q2.4), the renderer needs no file and no socket. The sandbox then
  protects users before site locks do. Split it: **Q2.8a** macOS
  seatbelt, **Q2.8b** Linux seccomp and namespaces (two mechanisms, two
  test machines).
- **Q2.5** depends on Q1.7 (the interrupt flag that moves to shared
  memory). Q1.7 should land first.
- **Q3.6** (`frame-ancestors`) needs the browser's frame tree, which
  exists after Q2.6a, not Q2.9.
- **Q1.9** should save the session in the shape of §7.

## Firefox (FF, unchecked)

Fission has been on by default in desktop Firefox since version 95. Its
content processes are typed by site ("webIsolated" with the site), and
cross-site iframes run in other processes. The parent process holds the
canonical browsing-context tree, and content processes hold replicas of
it; that is the model of §2. Its sandboxes use seatbelt on macOS and
seccomp-bpf with namespaces on Linux. It allows several processes per
site, where Chromium uses one per site and BCG. Dweller follows
Chromium where they differ, because the Chromium source is at hand to
check against.
