# Process model and site isolation

Design note for Q2 of `docs/plans/foundations.md` (2026-11-04). Steps
Q2.2–Q2.14 implement it. Chromium paths are in the M148 checkout.
Statements about Firefox are from memory, not from source, and are
marked "(FF, unchecked)".

## 0. Today

As built (2026-11-17): by default a tab's page runs in the window's
process; with `DWELLER_PROCESS=tab` or `site` (§3, Modes), and always
for a Space's tabs (§8), pages run in sandboxed renderer processes.
The page state is a `PageView` (`pageview/`), which the window holds
for a tab in its own process and a renderer holds otherwise.

Before Q2 Dweller was one process, except for Spaces:
- The window (`ui/window.aye`) had the chrome, the SDL event loop, and
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
- A Space (`ui/spaces.aye`) was the whole window program again,
  headless, started with `posix_spawnp`, with a `socketpair` of JSON
  control lines and a two-slot frame file in `/tmp/dweller-PID-XXXXXX/`
  (about 25 ms to start, about 45 MB). Since Q2.12e (2026-11-16) it is
  gone: a Space is tabs of the window in renderers (§8).

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
| page half of `ui/`: `PageDraw` and `damage.aye` (now `pageview/damage.aye`), `forms.aye`, focus, find matching, hover, wheel targets, the `select` model | renderer, as a new "page view" | the browser draws the `select` popup from a message |
| `ui/agents.aye`, `ui/spaces.aye`, `ui/socket.aye` | browser | the Space process goes (§8) |
| `agent/` (snapshot, refs, actions, eval) | renderer, per frame | the browser joins the parts of a tab |
| `net/` (http, loader, multi, cache, cookies, session, profile, archive, auth) | browser | the renderer uses only its types (`Request`, `Response`, `Kind`) and calls `ipc/` instead |
| `net/psl.aye`, `url/`, `std`, `encoding/`, `perf/` | both | `perf` lines carry the role and pid |
| `engine/` (pages, DOM API, scripts, local frames, proxies) | renderer | `engine/storage.aye` keeps a cache per origin; saving moves to the browser. `SessionHistory` (`engine/history.aye`) moves to the browser: it sends `session_history` (39) and `traverse` (41), and the renderer asks with `history_go` (40) |
| `html dom css layout paint font image svg webfont animation js wasm gpu audio` | renderer | `paint` and `font` also in the browser, for the chrome and compositing |
| new `ipc/` | both | framing, codec, shared buffers, fd passing |
| new `proc/` | browser | spawn, lock, reuse, cap, kill, the policy checks, crash and hang. As built there is no `proc/`: this is in `ui/` (`TabProcess`, `spawnRenderer`, `startTabProcess`, `takesSwap`, `originHosted`, `inSiteLock`, `renderer-sent.aye`, `child-sent.aye`) |

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
would print each message as one text line on stderr (not built). Type codes are numbered in the order the messages are added (Q2.2a: 1 `hello`, 2 `navigate`, 3 `loaded`, 4 `shutdown`; `ipc/message.aye`); flags and request id stay 0 until a message uses them. Why not the JSON lines
of Spaces: bodies are binary (base64 costs a third more), and the
browser parses what a compromised process writes; a fixed binary codec
has less to get wrong than a JSON parser. JSON stays for the agent
protocol.

**Rules.**
- The browser never waits for a renderer. Its sockets do not block,
  each renderer has an out buffer (`TabProcess.outbox`), and it reads
  only what is ready.
- An IO thread in the browser `poll`s the renderer sockets and pushes
  an SDL user event to wake the main loop. All handling stays on the
  main thread (`ui/io-waker.aye`). The 15 ms poll of the Space
  process was too slow for frames and input acks.
- Only a renderer makes sync calls (flag `sync`). While it waits for
  the reply, it reads and queues the other messages. The browser answers
  a sync call from its own state, never by asking another renderer, so
  no cycle can deadlock.
- Frame ids are global u32s that the browser gives. A message's frame id
  must be a frame that this process hosts (or, for proxy messages, a
  proxy that it holds).

**Messages** (B = browser, R = renderer; "(s)": a sync call). The
table is the design; a message with a number or a Q step is built, and
the built set is `ipc/message.aye`: 1 `hello`, 2 `navigate{url, post,
content_type, body}`, 3 `loaded`, 4 `shutdown`, 5 `resize{width,
height, scale, buffer}` (with the two buffer fds), 6 `frame{buffer,
width, height, scroll_x, scroll_y, damage}`, 7 `frame_ack`, 8 `mouse`,
9 `wheel`, 10 `key`, 11 `text`, 12 `input_ack`, 13 `begin_navigation`,
14 `popup`, 15 `popup_choice`, 16 `focus`, 17 `title`, 18
`same_document`, 19 `cursor`, 20 `target_url`, 21 `long_task`, 22
`take_focus`, 23 `redirect`, 24 `agent`, 25 `agent_reply`, 26 `fetch`,
27 `fetch_head`, 28 `fetch_body`, 29 `fetch_end`, 30 `fetch_fail`, 31
`cookie_get{doc}`, 32 `cookies`, 33 `cookie_set{doc, line}`, 34
`storage{doc, context, session, op, index, key, value}`, 35
`storage_reply`, 36 `storage_event`, 37 `ping`, 38 `pong`, 39
`session_history`, 40 `history_go`, 41 `traverse`, 42 `out_of_memory`,
43 `subframes`, 44 `wheel_unused`, 45 `scroll_bubble`, 46
`focus_frame`, 47 `advance_focus`, 48 `viewport_intersection`, 50
`fetch_cancel`, 51 `post_message`, 52 `message`, 53 `frame_owner`, 54
`frame_loaded`, 55 `proxy_navigate`, 56 `load_stopped`, 60 `edit`, 61
`clipboard`, 62 `find`, 63 `find_reply`, 64 `context`, 70
`agent_acted`, 71 `agent_paused`, 72 `agent_window`, 73
`agent_window_reply`, 74 `attach_tab`, 78 `settled`, 80 `ask_state`, 81
`state`, 82 `agent_viewport`, 83 `visible`, 90 `context_closed`, 100
`fetch_remote`, 140 to 143 `socket_*`, 144 `download_hint`, 145 to 148
`permission_*`. Not built: `init` (the settings are spawn arguments:
`dweller --renderer 3 --font fixed|system [--no-js] [--site-frames]
[--console] [--steady-caret]`), `buffers` (the fds come with `resize`),
`commit_navigation` and the `load*` group (a navigation and a
subresource are both a `fetch`), the `proxy_*` messages other than
`proxy_navigate`, `hit_test`, `dialog` (`alert`, `confirm` and
`prompt` are console messages in the renderer; `confirm` gives false
and `prompt` null), `file_chooser`, `font_open`, `agent_event`,
`page_state` and `memory_pressure`. Cookie and storage messages name a
document id (`doc`), and the browser uses its own record of that
document's origin.

| Group | Messages |
|---|---|
| lifecycle | R→B `hello{version, pid}` (the first message, frame 0; the browser checks the version and that the pid is the one it spawned); Q2.2a only, until Q2.6 brings the navigation group: B→R `navigate{url}` (frame = the tab), R→B `loaded{title, url}` (the renderer loads the page itself and runs its scripts and timers); `font` and `js` of `init` go as spawn arguments for now (`--font fixed|system`, `--no-js`, Q2.2c); B→R `init{lock, partition, scale, font, js, flags}`; B→R `shutdown` (SIGKILL after 1 s, as for Spaces); B→R `visible{frame, on}` (hidden: timers throttled, no frames); B→R `memory_pressure`; Q2.5: B→R `ping{n}` (frame 0, at most once a second, one open at a time), R→B `pong{n}` (the same `n`, else `bad pong`); Q2.14d: R→B `settled{on}` (78: the top-level page has no frame due, no timer, no animation frame, image, decode, canvas frame or navigation pending; sent when the value changes, never by an embedded frame; frame = the tab, `on` 0 or 1, else `bad settled`; the browser reads it only for `--until-idle`); Q2.14b: B→R `ask_state{with_text}` (80) and R→B `state{scroll, frames, focus, content}` (81), both on frame 0: at the end of a headless run with `--state` or `--text` the window asks once, and the renderer lands its loads, runs its due tasks and answers with its scroll offset, the `frames` / `frame_urls` and `element` / `value` lines of `--state` and, when asked, the document's text; `state` outside an ask is a bad message, a hung renderer is not asked, and the window waits at most 5 s; Q2.14g: B→R `agent_viewport{width, height}` (82: `dweller agent|mcp --width --height` in a renderer mode, before the first `agent`); Q2.14h: B→R `visible{on}` (83, frame = the tab or the child: the window is hidden or shown, or the tab goes to or comes from the background; the renderer fires `visibilitychange`, runs no animation frame and runs its timers once a second while hidden, as the window does for its own pages) |
| navigation | R→B `begin_navigation{frame, url, method, body, referrer, target, gesture, replace}` (links, forms, `location`, `window.open`); B→R `commit_navigation{frame, nav, url, origin, status, headers, entry}`, then `body_data{nav, bytes}` and `body_done{nav}`; R→B `did_commit{frame, nav, url, origin}`; R→B `same_document{frame, url, state, replace}` (pushState, fragments); B→R `navigate{frame, entry}` (back, forward, reload); B→R `before_unload` and `unload{frame}`, with replies; R→B `loading{frame, state}`, `title{frame, text}`; Q2.3d has R→B `title{text}` and `same_document{url}`, and the browser kills a renderer whose `loaded` or `same_document` URL is not the one it navigated to, the same document, or the same origin (`bad url`); Q2.14g: `begin_navigation` ends with `user` (0 or 1, else `bad begin_navigation`: the page had transient activation), which the browser uses for `Sec-Fetch-User` as Chromium uses the renderer's `has_user_gesture` |
| frames | B→R `buffers{frame, w, h}` with 2 fds; R→B `frame{frame, buffer, seq, damage[], children[], over?}` (§5); B→R `frame_ack{frame, buffer}`; B→R `resize{frame, w, h, scale}`; R→B `child_size{child, w, h}`; B→R `viewport_intersection{frame, rect}`; Q2.9 has R→B `subframes{frames}` (43: per remote frame its key, rect, clip in the parent's buffer and URL; the browser starts, resizes and ends the child renderers from it); Q2.15d: the rect and clip are the ones the parent's display list gives the frame (ancestors' overflow clips, scroll offsets, translations), and the record has the frame's opacity, which the window blends with, and a child renderer gets the existing `navigate`, `resize`, `frame_ack` on a frame id of its own |
| input | Q2.3a: B→R `mouse{seq, kind (0 move, 1 down, 2 up, 3 leave), x, y, mx, my, button, buttons, mods, clicks}`, `wheel{seq, x, y, dx, dy, precise}` (CSS px, coalesced per loop iteration; x = y = -1 when the mouse is not over the page), `key{seq, down, key, code, repeat, location, mods}` (DOM `key` and `code`), `text{seq, text}`, all in the tab's viewport coordinates; R→B `input_ack{seq}` (every input up to `seq` handled and its frame sent; later `consumed, dx, dy` for scroll that bubbles up); Q2.3d: B→R `focus{on}` (the page has the window's focus or not: the window lost it, or the toolbar took it; the renderer hides the caret and stops its blink); Tab, the focus ring, the caret and the selection of text fields are the renderer's; Q2.10 (with `DWELLER_PROCESS=site`): the browser routes mouse and wheel input by the children's rects and clips from `subframes` (top-most child at the point, recursively, in the child's coordinates; a mouse down captures its frame until the last button is up) and keys and text to the focused frame; R→B `wheel_unused{seq, dx, dy}` (44: the part of a wheel or bubbled scroll the child did not use; the browser kills a child that claims more than it was sent) becomes B→R `scroll_bubble{seq, x, y, dx, dy}` (45) to the embedder, which scrolls from that point without a wheel event; R→B `focus_frame{key, how}` (46: Tab or Shift+Tab into a remote frame, or its owner's `focus()`), B→R `advance_focus{key, reverse}` (47: Tab into the frame from its start or end, or onward from remote child `key`, which sent `take_focus`); B→R `viewport_intersection{x0, y0, x1, y1}` (48: the child's visible rect in the top's viewport, its own device px; IntersectionObserver's implicit root clips by it); Q4.25e: R→B `scroll_info{max_y, pan, background}` (120, a tab's top renderer only, when it changes: how far the page scrolls, whether the window may shift the last frame by a wheel before the renderer's own frame comes (no fixed or sticky box, no wheel cancelled or taken by an inner scroller so far), and the canvas color for the strip the frame does not hold; a child renderer that sends it is killed); R→B `cursor`, `status_url`, `focus_changed{frame}`, `advance_focus{frame, dir}`; B→R `focus{frame, dir}`; B→R `hit_test{frame, x, y}`, R→B its reply; B→R `find{text, dir}`, R→B `find_reply{count, active, rects}`; B→R `edit{copy, paste, select_all}`; R→B `popup{frame, rect, items}`, B→R `popup_choice`; R→B `long_task{ms}` |
| input | Q2.3a: B→R `mouse{seq, kind (0 move, 1 down, 2 up, 3 leave, 4 cancel: the window lost the focus with a button held, Q2.14h), x, y, mx, my, button, buttons, mods, clicks}`, `wheel{seq, x, y, dx, dy, precise}` (CSS px, coalesced per loop iteration; x = y = -1 when the mouse is not over the page), `key{seq, down, key, code, repeat, location, mods}` (DOM `key` and `code`), `text{seq, text}`, all in the tab's viewport coordinates; R→B `input_ack{seq}` (every input up to `seq` handled and its frame sent; later `consumed, dx, dy` for scroll that bubbles up); Q2.3d: B→R `focus{on}` (the page has the window's focus or not: the window lost it, or the toolbar took it; the renderer hides the caret and stops its blink); Tab, the focus ring, the caret and the selection of text fields are the renderer's; Q2.10 (with `DWELLER_PROCESS=site`): the browser routes mouse and wheel input by the children's rects and clips from `subframes` (top-most child at the point, recursively, in the child's coordinates; a mouse down captures its frame until the last button is up) and keys and text to the focused frame; R→B `wheel_unused{seq, dx, dy}` (44: the part of a wheel or bubbled scroll the child did not use; the browser kills a child that claims more than it was sent) becomes B→R `scroll_bubble{seq, x, y, dx, dy}` (45) to the embedder, which scrolls from that point without a wheel event; R→B `focus_frame{key, how}` (46: Tab or Shift+Tab into a remote frame, or its owner's `focus()`), B→R `advance_focus{key, reverse}` (47: Tab into the frame from its start or end, or onward from remote child `key`, which sent `take_focus`); B→R `viewport_intersection{x0, y0, x1, y1}` (48: the child's visible rect in the top's viewport, its own device px; IntersectionObserver's implicit root clips by it); R→B `cursor`, `status_url`, `focus_changed{frame}`, `advance_focus{frame, dir}`; B→R `focus{frame, dir}`; B→R `hit_test{frame, x, y}`, R→B its reply; B→R `find{text, dir}`, R→B `find_reply{count, active, rects}`; B→R `edit{copy, paste, select_all}`; R→B `popup{frame, rect, items}`, B→R `popup_choice`; R→B `long_task{ms}` |
| network | R→B `load{id, frame, url, method, headers, body, kind, mode, credentials, initiator}`; B→R `load_response{id, status, url, mime, charset, headers}`, `load_data{id, bytes}`, `load_done{id, error}`; R→B `load_cancel{id}`; R→B (s) `load_sync{...}` (sync XHR, and today's blocking `net.request` callers until they become async); Q5.7 (WebSocket, the connection is the browser's): R→B `socket_open{id, doc, url, protocols}` (140: `id` is the renderer's, new and not 0; the scheme `ws` or `wss`, no fragment, else `bad socket`; the referrer and the top are the browser's own record of document `doc`), R→B `socket_send{id, binary, more, data}` (141: chunks of 256 KiB, at most 16 MiB a message), R→B `socket_close{id, code, reason}` (142: 1000, 1001 when the page goes away, 1005 for no code, 3000–4999; a reason of at most 123 bytes), B→R `socket_note{id, kind, clean, code, more, data}` (143: kind 0 opened with the subprotocol, 1 text, 2 binary, 3 closed with the reason); a renderer holds at most 255 sockets, and they end with it; Q5.6 (downloads): R→B `download_hint{url, name}` (144: the next navigation to `url`, an `http(s)` URL inside the site lock, is a download named `name`), and a navigation whose response the browser saved ends with `fetch_fail{id, 19}`: the renderer keeps its document; Q5.12 (permissions; the store and the prompt are the browser's): R→B (s) `permission_get{doc, kind}` (145), B→R `permission_state{state}` (146: 0 prompt, 1 granted, 2 denied, for the origin of the browser's record of document `doc`), R→B `permission_ask{id, doc, kind}` (147), B→R `permission_answer{id, state}` (148: when the person answered the bar, or 0 when it was dismissed by a navigation or could not be shown) |
| cookies, storage | R→B (s) `cookie_get{frame, url}`; R→B `cookie_set{frame, url, line}`; R→B (s) `storage_open{frame, kind, origin}` returns every item; R→B `storage_set`, `storage_remove`, `storage_clear`; B→R `storage_event{origin, kind, key, old, new, url}` to the other renderers of the origin |
| frame tree, proxies | R→B (s) `frame_created{parent, name, sandbox}` returns the id; R→B `frame_detached{frame}`; B→R `proxy_create{frame, parent, opener, origin, name, sandbox}`, `proxy_update`, `proxy_delete`; B→R `swap_out{frame}` (the frame now lives in another process: a proxy replaces it); R→B `post_message{from, to, source_origin, target_origin, data}`, B→R `message{to, from, origin, data}`; R→B `proxy_navigate{to, url}`, `proxy_focus`, `proxy_blur`, `proxy_close`, `opener_cleared`, `name_changed`; Q2.11 (with `DWELLER_PROCESS=site`) has: R→B `post_message{key, frame, page, source_page, source_origin, target_origin, data, buffers}` (51: to remote frame `key` of the sender, or to page `page` (0: the root) of the renderer of frame `frame`; `data` is the workers' structured-clone JSON (`engine/workers/wire.js`), `buffers` its ArrayBuffers), B→R `message{page, from_key, from_frame, from_page, origin, target_origin, data, buffers}` (52: `from_key` when the sender is the receiver's own remote child, so `event.source` is that iframe's `contentWindow`), B→R `frame_owner{parent_frame, parent_page, top_frame, sandbox, name}` (53: before a child's navigation: its `parent` and `top` proxies, the iframe's sandbox flags and name; the `subframes` record now carries the holder page, sandbox and name), B→R `frame_loaded{key}` (54: the owner element's `load`), R→B `proxy_navigate{key, frame, page, url}` (55: `location` set on a remote window), R→B `load_stopped` (56: a child's load event fired); Q2.12d: B→R `attach_tab{context}` with one socket fd (74: the renderer serves a second tab on that socket, showing popup context `context` its page opened; sent for a popup that keeps its opener and is same-site with it, so both run in one process; `context` (64) says whether the new context keeps its opener) |
| dialogs, permissions | R→B (s) `dialog{frame, kind, text, default}` (alert, confirm, prompt, beforeunload); R→B `permission{frame, kind}`, B→R `permission_result`; R→B `file_chooser{frame, accept, multiple}`, the reply with read-only fds (Q5.5; as built: R→B `choose_files{multiple, accept}` (190), B→R `chosen_file{name, mime, modified}` (191), `chosen_bytes{chunk}` (192, at most 256 KiB each), `chosen_end{count}` (193, 0 for a canceled picker): the browser reads the files and sends bytes, never a path or an fd; a second request while one waits is answered with `chosen_end{0}`); Q5.15: B→R `print{paper}` (194), R→B `printed{chunk}` (195) and `printed_end{pages}` (196; the browser kills a renderer that sends either without a print it asked for, or more than 512 MiB), R→B `print_ask` (197, `window.print()`); Q5.4: R→B `caret_rect{on, x, y, w, h}` (198: the focused text field's caret in the frame's viewport, sent when it changes; the browser places the input method's candidate window there), B→R `compose{seq, typed}` (199: the text an input method is composing, empty to cancel; the commit is `text`); R→B `download{frame, url}`; R→B `clipboard_write`, (s) `clipboard_read`; R→B (s) `font_open{name}` (Linux, §6) |
| agent | B→R `agent{id, frame, request}`; R→B `agent_reply{id, json}`; R→B `agent_event{frame, json}` (loads, console, dialogs); Q2.12: R→B `agent_acted{what}` (70: the action's text for the Space's last action, before `agent_reply`), B→R `agent_paused{on}` (71: the Space is paused; acting calls inside a `run` are refused too), R→B (s) `agent_window{line}` (72: a window command inside a `run`), B→R `agent_window_reply{ok, message, fields, detail}` (73) |

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
| a frame's URL that redirects | the site of the URL after redirects (Q2.15a: the browser keeps a response that left the asking renderer's site, answers `fetch_remote{id}` (100), and commits it in a renderer of the final site) | the same |
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
checked. `site`: locks, rules 1 and 3. OOPIF: rule 2. As built, `tab`
is one renderer per tab, locked to the site of its first commit and
swapped on a cross-site top-level navigation or a COOP response
(`takesSwap`), with cross-site iframes in the parent's process; `site`
adds the renderers of cross-site iframes (the spawn argument
`--site-frames`).

**Modes.** `DWELLER_PROCESS` picks one: `tab` and `site` as above, and
`single`, the page in the window's own process (there is no
`--single-process` flag). Unset, or any other value, is `single` until Q2.14
makes `site` the default. A test arm that means the window's own process
sets `single` (Q2.14c), so it keeps that meaning after the flip.

## 4. The site-lock checks

The browser knows each frame's committed origin from its own
`commit_navigation`, and it checks every origin that a renderer states.
`canAccess(process, origin)` is true when the site of the origin (of its
precursor, for an opaque one) is the process's lock. This is Chromium's
"jail" check (`ChildProcessSecurityPolicyImpl::CanAccessDataForOrigin`,
then `PerformJailAndCitadelChecks`). The "citadel" check (an unlocked
process may not touch sites that need a lock) has nothing to protect in
Dweller once every process is locked. As built the check is
`originHosted` and `inSiteLock` (`ui/origin-hosted.aye`,
`ui/in-site-lock.aye`), the same in `tab` and `site`.

| Message | Check | Failure |
|---|---|---|
| every message | well formed, known type, the frame id hosted by this process | kill |
| `did_commit` | the origin is the one sent in `commit_navigation` | kill |
| `same_document` | the new URL has the frame's origin | kill |
| `load`, `load_sync` | `initiator` is the frame's committed origin (Chromium `request_initiator_origin_lock`); no `Host`, `Cookie`, `Origin`, `Sec-*` header | kill |
| `load`, `load_sync` | the scheme is allowed for the initiator (`file:` only from `file:`, Q3.1) | a network error (a page may try) |
| `cookie_get`, `cookie_set` | `canAccess` for the URL's origin, which is a frame's origin | kill |
| `storage_*` | the origin is the frame's committed origin | kill |
| `post_message` | `source_origin` is the sender frame's origin (Chromium kills with `RFPH_POST_MESSAGE_INVALID_SOURCE_ORIGIN`); Q2.11: `null` or a serialized http(s) origin in the sender's site lock (`HostsOrigin`), `target_origin` `*` or a serialized origin, `buffers` well formed | kill |
| `post_message` | `target_origin` is the target's current origin; Q2.11: the target renderer's committed origin for its root (opaque when its iframe is sandboxed without `allow-same-origin`), its site lock for a page below the root (the receiving renderer compares the exact origin) | drop (a navigation race, not an attack) |
| `proxy_*` | the sender holds a proxy of that frame (same BCG); the sandbox flags allow it (Q3.9); Q2.11: the target is in the sender's tab (else dropped); `proxy_navigate` of an ancestor or another frame needs the sender's frame sandbox (the flags the browser sent in `frame_owner`) to allow it, `allow-top-navigation` for the top | kill / ignore |
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

**Kill.** The browser logs `bad message: REASON PID`, sends
SIGKILL, and the frames of the process go to the crash path (§7).
Chromium `bad_message::ReceivedBadMessage` does the same. The Q2.7 test
hook `DWELLER_TEST=1 DWELLER_TEST_EVIL=NAME` (the names are the
`evilTest` calls in `renderer/`, such as `commit`, `site`, `initiator`,
`post-message`, `sandbox-cookie`) makes a
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
--renderer FD` after the font preload and before it sends `hello`
(`renderer/serve-on.aye`); a renderer that cannot enter it exits with
status 3. `DWELLER_NO_SANDBOX=1` skips the sandbox, with a warning on
stderr. It never starts SDL.

Its stderr is the browser's (fd 2 is kept by `spawnRenderer`, `write`
is allowed), and `DWELLER_*` variables pass to it (`rendererKeeps`;
not `DWELLER_PROFILE`, `DWELLER_COOKIE_FILE` or
`DWELLER_TELEMETRY_DIR`), so
its `DWELLER_TIMING` records (`perf.end`, one `write` per record) and
its console lines go to the same stream as the browser's, unmarked.
The console is echoed when the browser's is (`--console`, from
`engine.consoleEchoed()`: a window without `--frames`, or
`DWELLER_CONSOLE`), as in one process. A renderer can write anything
there, so stderr is a log for people and tests, never input the
browser reads (Q2.14d).

A headless run's `--state` prints the page's lines the same in every
mode; the lines about the renderer itself (`renderer`, `renderer-site`,
`renderer-pids`, `renderer-frame`, ...) come only with `--processes`
(Q2.14b). With `DWELLER_DAMAGE_CHECK` the renderer compares each frame
it sends with a whole drawing, and the window compares its page pixels
with a whole copy of the renderers' frames (Q2.14e).

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

**Linux** (as built in Q2.8b, `browser/renderer/enter-linux-sandbox.aye`):
`prctl(PR_SET_NO_NEW_PRIVS)`, then two layers.
- Layer 1, Landlock (any kernel ≥ 5.13 that has it; ABI 7 here):
  read-only access beneath `/usr` (libraries, `/usr/share/fonts`) and
  the executable's directory, nothing writable, no TCP bind or connect
  (ABI ≥ 4), no signals or abstract sockets outside the sandbox
  (ABI ≥ 6). It replaces Chromium's broker and namespace chroot
  (`sandbox/linux/syscall_broker/`, `sandbox/linux/services/credentials.cc`):
  it confines paths unprivileged and in-process, while Ubuntu 23.10 and
  later forbid unprivileged user namespaces (AppArmor).
- Layer 2, seccomp-bpf (x86_64), with `SECCOMP_FILTER_FLAG_TSYNC`: an
  allowlist in the shape of Chromium's baseline and renderer policies
  (`read`, `write`, `recvmsg`, `sendmsg` on held fds, memory incl.
  `PROT_EXEC` for wasmtime, `futex`, clocks and sleeps, signals on
  itself, `getrandom`, `rseq`, exit); `open`/`openat` only without write
  flags and only when Landlock is on; `clone` only with thread flags,
  `clone3` ENOSYS (glibc falls back to `clone`); everything else
  (`socket`, `connect`, `execve`, `fork`, `ptrace`, `kill`, writes by
  path) fails with `EPERM`.
- Fonts: the system faces are read before entry; fallback faces are
  opened later by path under `/usr/share/fonts`, which Landlock allows.
  Without Landlock seccomp refuses every open and those faces are
  missing; a `font_open` from the browser (Chromium's font IPC,
  `docs/linux/sandbox_ipc.md`) would bring them back.

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
navigates away and at most once a second (Chromium `PageState`); as
built there is no `page_state`, and the browser saves the session
itself (`sessions/`, `DWELLER_RESTORE`). The
browser writes the session atomically (`net/profile.aye`
`writeAtomic`). A renderer crash does not change the session. At
startup only the shown tab loads; the others load when first shown.
Q1.9 (session restore, one process) should already save this shape, so
that Q2.6 moves the code without changing the file.

## 8. Spaces become renderers (Q2.12)

Before Q2.12 a Space was a full copy of the window program, with its
own tabs, engine and jar. A `shared` Space got the whole jar and every
origin's `localStorage` in its `share` message, and it sent cookie and
storage lines back (`ui/share-with-space.aye`, `ui/apply-shared.aye`,
removed in Q2.12e). Under site isolation that gives every site's
cookies to one process, so it went.

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

**In-process mode (decided in Q2.12).** Spaces always use renderers, in
every `DWELLER_PROCESS` mode, also while the person's tabs run in the
window's process. There is no in-process Space path: a request on the
window's thread is what Spaces exist to avoid (the window never waits
for an agent), and Chromium's `--single-process` is a debugging switch,
not a mode it ships.

**As built (Q2.12, first slice, 2026-11-09).** Behind
`DWELLER_SPACES=renderer` the window hosts each `shared` Space itself:
the `Space` keeps its tabs (`own_tabs`, numbered from 1 in the Space,
frame ids from 2^29), each with a sandboxed renderer started at once
and polled with the person's tabs in every mode; page commands and
`wait` go to the tab's renderer, the window commands are answered from
the Space, and the cookie jar and `localStorage` are the browser's
(Q2.4b), so nothing is copied. Watching draws the renderer's buffer of
the Space's first tab in the page area, and input while it is shown
goes to that renderer. Messages 70–73 (§2) carry the last action, the
pause and a `run`'s window commands. Not yet (Q2.12c did the first and the last, below): `isolated` Spaces still
get a Space process (one jar in the browser), openers do not reach
across renderers, and a renderer crash fails the request with "the
tab's renderer is gone". Without the switch the Space process of §0
runs as before. The card `Q2.12` lists the rest.

**Navigation fetches (Q2.12b, 2026-11-09).** A renderer's navigation
`fetch` (top-level or a frame's, a tab's or a hosted Space's) is a
held fetch like a subresource's (Q2.4e): `net.startNavigation` runs it
on the loader's multi handle with the synchronous path's semantics
(redirects with the chain's SameSite and `Sec-Fetch-Site` state, Basic
auth retries, the 32 MB document limit, `Sec-Fetch-User`), and the
window polls it. When it lands, the browser does what it did after the
blocking request: `X-Frame-Options`/`frame-ancestors` for a frame, the
process choice (`takesSwap`, COOP), then the reply; the commit's
site-lock check is unchanged. So a slow server no longer stalls the
window's loop. A `file:` or `data:` navigation, a replayed archive and
a primed response (after a swap) are still answered at once.

**Partitions and crashes (Q2.12c, 2026-11-09).** With the switch an
`isolated` Space is hosted too, on a partition of its own: the window
gives it a cookie jar (`net.openPartition`, keyed by the Space's id) and
a `localStorage` area prefix (`#ID ORIGIN`) in the in-memory store that
`sessionStorage` uses, so neither is ever saved to the profile. The
partition is a field of each of the Space's tab processes
(`TabProcess.partition`, given at `startTabProcess` and inherited by
swaps, OOPIFs and popups) and is read from the browser's own record, never
from a message: `answerFetch` puts it on the request (`Request.partition`,
kept by the loader's loads and redirects), whose cookies come from and go
to that jar, whose network partition key (cache, connection pool) starts
with it, and which notes no HSTS; `answerCookie` and `answerStorage` use
it; `storage` events go only to renderers of the same partition. The jar
and the areas go when the Space is freed (`dropPartition`). A hosted
tab whose renderer dies stays in its Space as `crashed` (`tabs`); the
request on it and later page commands fail with "renderer crashed"
(`failCrashed`), and `tab_close` frees it. Not yet: a sad tab while the
person watches a crashed hosted tab, the Basic auth cache, the preflight
cache and HSTS per partition.

**Popups in their opener's process (Q2.12d, 2026-11-16).** A popup
that keeps its opener and is same-site with it is a second tab of its
opener's renderer: the browser sends `attach_tab` (74) with a socket on
the opener's connection and keeps a `TabProcess` per tab with the same
pid. Which records share a process is read from the pid: the process
is waited for when its last tab ends, and a crash, a bad message or
Stop ends every tab on it. In the renderer one loop serves every tab's
connection, and a tab whose connection fails ends alone.

**The flip (Q2.12e, 2026-11-16).** Hosted Spaces are the only Spaces:
`DWELLER_SPACES` is read nowhere, and `dweller space`, the relay, the
control connection, the sharing of cookies and `localStorage` with a
Space and the frame file are removed. What the paragraphs above call
"with the switch" is what the window does.

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
   namespaces best effort, and a CI test that seccomp is on. As built:
   Landlock and seccomp, no namespaces (§6).
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
   `dump` and `screenshot`. CI runs one smoke test in it. As built
   there is no flag: an unset `DWELLER_PROCESS` is this mode.

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
