# Dweller browser

A web browser written in Aye, for two kinds of users at once:

- **Humans** get a window: address bar, back and forward, scrolling,
  links, forms, and pages with CSS and JavaScript.
- **AI agents** get the same engine without pixels: a page as clean
  Markdown, an accessibility tree whose interactive elements carry
  stable refs (`[e12] link "Docs"`), and actions on those refs (`click
  e12`, `type e7 "query"`). Agents talk to it over stdio as an MCP
  server or as JSON lines. A human window can also accept an agent, so
  both can drive one page and see what the other did.

It is the P3 validation of design.md: DOM nodes live in `Pool(Node)`, JS
holds `Handle(Node)`, everything is allocated explicitly, and the engine
only calls C for the platform (SDL2 for windows and input, FreeType for
glyphs, libcurl for HTTP and TLS). Parsing, style, layout, painting,
JavaScript and the agent protocol are Aye.

## Layout of the code

```
browser/
  main.aye        CLI: window mode, `dump`, `agent`, `mcp`, `connect`
  url/            URL parsing and resolution (WHATWG subset)
  net/            fetch: file:, data:, http(s) through libcurl, cache
  html/           tokenizer and tree builder (WHATWG subset), entities
  dom/            Node, Document, Pool(Node), queries, mutation; the
                  state of form controls (values, checkedness, focus)
  css/            tokenizer, parser, selectors, cascade, computed style
  layout/         block, inline (line breaking, UAX 14, bidi), lists,
                  tables, flex, grid, floats, positioned and sticky
                  boxes, transforms, framesets, text measurement
  engine/         Page (load, style, layout), Tab (history, scroll),
                  hit testing, find in page, images, default actions
                  and form submission, page scripts (DOM bindings,
                  prelude, event loop)
  image/          PNG (zlib inflate), GIF, JPEG (baseline, progressive,
                  CMYK) and WebP (VP8L, VP8) decoders
  svg/            SVG parsing and rasterizing (paths, shapes, gradients)
  font/           FreeType faces, fallback chain, HarfBuzz shaping
  webfont/        WOFF and WOFF 2 decoding for `@font-face`
  encoding/       WHATWG encodings: windows-1252 and the CJK ones
  paint/          display list, software rasterizer, glyph cache,
                  backgrounds, masks, shadows, PNG
  js/             lexer, parser, interpreter, GC, builtins, host interface
  agent/          snapshot (a11y tree with refs), Markdown, actions,
                  JSON-lines and MCP stdio servers, a window's agents
  ui/             SDL2 window, chrome (address bar, tabs), input, the
                  agent socket (`--agent-socket`, `connect`), Spaces
                  (`--spaces`)
  tests/          fixtures (HTML/CSS/JS) and golden outputs
```

## Milestones

Same rules as `milestones.md`: tests first; a milestone is done when its
acceptance tests pass. Tests run headless and offline (`file:` fixtures
and a local test server), so CI needs no network and no display.

**Status.** B1–B7 are done; B8 and B9 are under way:

| | State | Tests (`cargo test --test browser`) |
|---|---|---|
| B1 fetch, HTML, DOM | done | 31 `dump --tree`/`--text` fixtures, `data:`, `http:` (local server) |
| B2 CSS and style | done | 25 `dump --style` fixtures |
| B3 layout | done | 31 `dump --layout` fixtures (fixed font), system-font check |
| B4 paint and window | done | 13 golden PNGs, PNG checks, headless window tests |
| B5 forms and images | done | form sessions over HTTP, window form scripts, image decoder unit tests, form and image screenshots |
| B6 JavaScript | done | 212 conformance scripts (`dweller js`), 30 DOM fixtures (`dump --tree --js`, also under GC stress), a to-do app session, window script test, GC unit tests |
| B7 agent interface | done | 17 snapshot/Markdown fixtures, session and MCP transcripts (with `eval` and `wait`), shared-window sessions over its socket (JSON lines, MCP through `dweller connect`, pause, pointing, tabs) |
| B8 the real web | in progress | net over the local server (`net_http`, the `net_cookies` and `base` agent sessions), HTML/CSS fixtures (templates, quirks, hints, `var()`/`calc()`, at-rules, `@import`), layout fixtures of site shapes (floats, positioned boxes, grid, `details`), hit testing by layer (`window_hit_layers`), JS conformance scripts for generators, async, Symbol, the web APIs and ES modules (231–237, 242–245), DOM fixtures for module scripts, layout queries, Shadow DOM, custom elements, `MutationObserver` and Web Animations, background image loads (`net/bg_images.html`, `net/bg_image_events.html`), BigInt and property descriptor scripts (246–250), `fetch`/XHR methods and a failed image over the local server (`net/fetch_methods.html`, `net/xhr_methods.html`, `net/failed_image_once.html`), session history (`jsdom/history.html`, the `history` and `history_paths` agent sessions), Reflect, `setPrototypeOf` and Map/Set scripts (251–254), DOM fixtures for the body mixin, DOMParser, `IntersectionObserverEntry`, the virtual clock and endless feeds, replaced sizing layout fixtures (`replaced_sizing`, `replaced_minmax`) |
| B9 agents without disturbing the user | in progress (P9) | R1: `window_agents` `isolation` (agents act in their own tabs; the person's tab, scroll, selection, focus, field text and page height stay); R2: `window_spaces` (with `--spaces`, sessions in Spaces), `window_spaces_responsive` (`loop_max_ms` under 50 while agents load and `eval` for 1.5 s), `window_spaces_crash` (a killed Space, a killed window) |

All the front ends share `engine/`: the window, `screenshot` and the
agent sessions load pages through `engine.load` and keep history in a
`Tab`. What remains:

- **B8:** event listeners on another frame's nodes, moving nodes
  between documents, one joint session history for a tab and its
  frames (a frame's `history.length` counts only its own entries);
  calendar, color and file pickers, `datalist`; rotations and skews
  other than quarter turns are not drawn; sticky only against the viewport; no public suffix list for
  cookies (a bare top-level name is refused, `co.uk` is not); no AVIF;
  page loads are slow on big sites (in progress), and so is the JS
  interpreter (in progress). The list of public sites is in
  [`docs/browser-sites.md`](browser-sites.md), their expected state in
  `browser/tools/sites.json` (22 sites, 2026-10-08: 16 ✅, 4 ⚠️, 2
  excluded, both bot walls; the run of 2026-10-06, before the work
  below, had 8 ✅, 7 ⚠️, 5 ❌ of 20).

The remaining work, in phases and steps, is in [`docs/plan.md`](plan.md).

The compiler gaps the browser met, and which of them M9.5/M9.6 fixed,
are in `browser/COMPILER_GAPS.md`.

### B1: Fetch, HTML, DOM, text

- URL parsing and relative resolution; fetch for `file:`, `data:`,
  `http:` and `https:` (libcurl), with redirects and charset sniffing
  (UTF-8 and Latin-1).
- HTML tokenizer (data, tags, attributes, comments, doctype, character
  references, raw text for `script`/`style`, RCDATA for `title`/
  `textarea`) and a tree builder with implied `html`/`head`/`body`,
  void elements, auto-closing `p`/`li`/`td`/... and misnested formatting
  tags handled simply. It never fails.
- DOM in `Pool(Node)`: elements, text, comments; attributes;
  `getElementById`, `getElementsByTagName`, parent/children/siblings.
- **Acceptance:** `dweller dump --tree FILE` and `dweller dump --text
  FILE` golden tests on 20 fixtures, including malformed HTML.

### B2: CSS and style

- CSS tokenizer and parser (rules, declarations, `@media` with
  `screen`/width conditions, other at-rules skipped).
- Selectors: type, `.class`, `#id`, `*`, attribute (`[a]`, `[a=v]`),
  descendant, child, sibling combinators, `:first-child`, `:last-child`,
  `:not()`, `:hover`, `:link`, lists.
- Cascade with specificity, `!important`, inline `style`, a UA
  stylesheet, inheritance, `inherit`/`initial`.
- Computed values: display, position (static, relative, absolute,
  fixed; sticky as relative), float, clear, z-index, color,
  background-color, font-size/weight/style/family, line-height,
  margin/padding/border (widths, colors, styles), width/height/min/max,
  text-align, text-decoration, white-space, list-style-type, visibility,
  overflow; units px, em, rem, %, vw/vh, named and hex/rgb() colors.
- **Acceptance:** `dweller dump --style` golden tests.

### B3: Layout

- Block formatting (margins with collapsing between siblings), inline
  formatting with line breaking and `white-space`, `text-align`, list
  markers, `<br>`, `<hr>`, replaced elements with intrinsic size
  (`img` placeholder boxes), simple tables (auto column widths), simple
  flexbox (row/column, gap, justify-content, align-items, flex-grow).
- Text measurement through FreeType with a bundled-path fallback font;
  headless tests use a fixed-metric font so goldens are stable.
- **Acceptance:** `dweller dump --layout` golden tests (box tree with
  coordinates).

### B4: Paint and window

- Display list (rects, borders, text runs, images); software rasterizer
  into an RGBA buffer with alpha-blended glyphs from a glyph cache.
- `dweller screenshot FILE -o out.png` (PNG encoder in Aye).
- SDL2 window: address bar, back/forward/reload, scrolling (wheel and
  keys), hover cursor on links, clicking links, window resize, Cmd+L,
  Cmd+[ and Cmd+], find in page (Cmd+F).
- Window input (`ui/window.aye` `wheel`, `flushWheel`): wheel and
  trackpad deltas as macOS gives them. SDL's fractional deltas come from
  a precise device (trackpad, Magic Mouse: NSEvent `deltaY`, which is
  `scrollingDeltaY` / 10) and scroll 10 px a unit, with the momentum
  events macOS sends (no smoothing of our own); a whole delta is a wheel
  notch of 40 px, unless a fractional one came within 500 ms. The sign
  already follows the system's scrolling direction (natural scrolling
  sets `SDL_MOUSEWHEEL_FLIPPED` on deltas SDL inverted already), so it
  is not flipped again. X scrolls a wide page sideways; Y scrolls the
  frame under the mouse, else the page. The wheel events of one loop
  iteration are summed: one `wheel` event at the element under the
  mouse (deltaMode 0, CSS px, positive right and down; canceling it
  stops the scroll, but listeners of the window, document, root and
  body are passive by default), one scroll, one repaint without layout.
- **Acceptance:** screenshot golden tests compared by pixel hash; a
  manual checklist run once per milestone for the window.
- Done: `dweller screenshot FILE|URL -o out.png [--width W] [--height
  H] [--font fixed|system] [--full]` and `dweller [open] FILE|URL`.
  Golden PNGs (`browser/tests/paint/`, fixed font, compared byte for
  byte), PNG checks, and headless window tests (SDL's dummy driver,
  `--frames N` with scripted `--events`, `--state`, `--save`; see
  `browser/ui/script.aye`). The interactive checks are in
  `docs/browser-manual-checklist.md`. Simplifications: positioned boxes
  are painted after the rest of their layer in z-index order (negative
  ones first) and hit testing searches in the reverse order, but there
  are no other stacking contexts (opacity, transforms), clips are rectangles,
  the 3-D border styles are solid, and the style sheets are parsed again
  when the window is resized.

### B5: Forms and images

- `input` (text, password, checkbox, radio, submit), `button`,
  `textarea`, `select`; focus and keyboard editing; form submission by
  GET and POST (urlencoded).
- Images: PNG (zlib inflate in Aye) and GIF first frame; JPEG through
  a later milestone.
- **Acceptance:** a fixture form submitted to the local test server
  produces the expected request; image fixtures render to golden
  screenshots.
- Done:
  - *State* (`dom/controls.aye`): one place for what the window, the
    agent and scripts share: `doc.controls` keyed by node (a field's
    value and dirty flag, checkedness, the selectedness of options, the
    caret and selection, a field's scroll offset) and the focus.
    Attributes keep the defaults; a reset forgets the state. Functions:
    `dom.valueOf`/`setValue`, `isChecked`/`setChecked` (radio groups
    exclusive), `selectedOption`/`selectOption`/`isSelected`/
    `selectedIndex`, `formOwner`, `isDisabled` (fieldsets, optgroups),
    `resetForm`, and editing (`insertText`, `deleteBackward`,
    `deleteForward`, `moveCaret`, `selectAll`, `setCaret`). CSS matches
    `:checked`, `:disabled`, `:enabled` and `:focus` against it.
  - *Default actions* (`engine/forms.aye`), for the event loop to call
    after dispatching (unless the default was prevented):
    `engine.activate(page, node)` returns what the caller does next:
    `Follow { target }` (a link), `Submit { form, submitter }` (fire
    `submit`, then `engine.submitForm`), `OpenSelect { el }`, `Changed`
    (a checkbox, radio button, reset button, label, summary, or the
    focus moved: style and lay out again) or `Nothing`.
    `engine.implicitSubmit(page, field)` is Enter in a text field (the
    first submit button, or no button when the form has one field).
    `engine.submitForm` builds the request: GET (fields in the query)
    or POST as application/x-www-form-urlencoded, multipart/form-data
    (a fixed boundary; file inputs send an empty file) or text/plain;
    `formaction`, `formmethod`, `formenctype`; the submitter's name and
    value, an image button's `x`/`y`. `engine.navigateTo(tab, sub)`
    loads it (POST through `net.fetchWith` with a body and
    `Content-Type`; a redirect after a POST continues with a GET). The
    history keeps URLs only: going back to a POST result loads it by
    GET. Focus: `setFocus`, `nextFocus` (Tab order with `tabindex`).
  - *Layout and paint* (`layout/controls.aye`): text fields, selects and
    input buttons are inline-blocks with generated text (bullets for a
    password, the placeholder in gray, the selected option's label);
    `size`, `cols`, `rows` give default sizes; a `select` with
    `multiple` or `size` > 1 is a list box with its selected options
    highlighted; a `textarea` wraps; text fields scroll to their caret;
    checkboxes and radio buttons are drawn replaced boxes; disabled
    controls are gray (UA sheet). `fieldset`/`legend` are plain blocks
    with a border (the legend does not sit on the border).
  - *Window* (`ui/forms.aye`): click, Tab and Shift+Tab focus; a focus
    ring; typing (SDL text input, so IME and UTF-8), Backspace, Delete,
    arrows, Home/End, Shift to select, Cmd+A/C/X/V (SDL clipboard);
    Enter submits (a new line in a `textarea`), Space toggles and
    presses; a `select` opens a dropdown over the page (arrows, Enter,
    Escape, clicks). `--state` adds the focused element and its value.
  - *Agent*: `click` on a submit button and `type` with `submit` submit
    for real; `type`, `select`, `press` and the snapshot use the same
    state as the window.
  - *Images* (`image/`, `engine/images.aye`): PNG (inflate with stored,
    fixed and dynamic Huffman blocks; every color type and bit depth,
    `PLTE`, `tRNS`, Adam7), GIF (first frame, LZW, transparency,
    interlace), baseline JPEG (Huffman, IDCT, YCbCr, chroma
    subsampling, restart markers). Images are fetched once per URL
    (relative, `data:`) before each layout for elements that have none
    yet; the natural size and aspect ratio feed layout (`width`/`height`
    attributes and CSS), and paint scales them (bilinear, alpha
    blended). Not yet: `srcset`, `background-image`, progressive JPEG,
    animation.
  - Tests: `agent_forms_http` (sessions over the local server's
    `/echo`, which shows the request), `window_forms`, `fill_form`,
    the `image` module's decoder tests (PngSuite-like fixtures made by
    `browser/tests/image/make_fixtures.py`), the `paint/forms`,
    `paint/decoded` and `layout/images` goldens.

### B6: JavaScript

- Lexer, parser and tree-walking interpreter for an ES5 core plus
  `let`/`const`, arrow functions, template literals, classes,
  destructuring (simple), spread, `for...of`; closures, prototypes,
  exceptions; `Object`, `Array`, `String`, `Number`, `Math`, `JSON`,
  `Date.now`, `console`.
- Mark-and-sweep GC over the interpreter's own heap.
- DOM bindings following design.md's JS boundary: wrappers hold
  `Handle(Node)` and a `js_refs` count. `document.getElementById`,
  `querySelector(All)`, `createElement`, `appendChild`,
  `removeChild`, `textContent`, `innerHTML` (set), `setAttribute`,
  `classList`, `style.*`, `addEventListener` (click, input, submit,
  keydown, DOMContentLoaded, load), `setTimeout`, `fetch` (simple).
- **Acceptance:** a JS conformance suite of 150 small scripts with
  golden console output; DOM fixtures whose `dump --tree` after scripts
  matches goldens; a to-do app fixture that works in the window.

**The engine core (done).** `dweller js FILE` runs a
script; `browser/tests/js/` holds the conformance scripts with Node's
output as goldens.

- *Pipeline:* tokens (all up front) → AST in one `List(Node)` per engine
  → a resolver that gives every variable an environment slot (`depth`,
  `slot`) → a tree-walking interpreter. Exceptions are the Aye error
  `JsError.Thrown` (the value in `Interp.exception`), so they pass through
  native functions with `try`.
- *Strings:* WTF-8 bytes (UTF-8 that can hold lone surrogates) with the
  UTF-16 length cached, as decided in design.md: the DOM and the engine
  share UTF-8, and `length`, `charCodeAt` and indices follow UTF-16.
  ASCII strings index their bytes; others are decoded when an operation
  needs positions. String comparison is by bytes (code point order,
  which differs from UTF-16 order only between astral characters and
  U+E000..U+FFFF). A concatenation of 64 bytes or more makes a rope that
  is copied flat only when its bytes are needed, so `s += x` in a loop
  is linear.
- *Heap and GC:* objects, strings and environments are `u32` indices into
  three lists with free lists; mark and sweep from the roots (global
  object, intrinsics, environments, the value stack, frames, pending
  exception and return value, host roots), every 2× live allocations.
  A value held across an allocation must be on the interpreter's stack
  (calls take `[f, this, args...]` from it). `DWELLER_JS_GC=stress`
  collects at every allocation to check that discipline.
- *Host interface:* native functions are a table of `fn(*mut Interp,
  Args) JsError!Value`; `Interp.newHostObject(proto, tag, index,
  generation)` makes a wrapper that holds a `Handle(Node)` as three
  integers, and `Interp.host: ?dyn Host` gets `finalize(tag, index,
  generation)` when the collector frees one (the `js_refs` decrement).
  `addRoot`/`removeRoot` keep values (event listeners) alive.
- *Generators and async functions* (B8, `js/coroutine.aye`): a call
  that can stop in the middle of the tree walk runs on a `std.coro`
  coroutine (a C stack of its own, M12; a thread and two channels
  before), with its own value stack and frames, on the engine's thread. Promise
  and the job queue are the engine's (`js/lib.aye`, `Interp.queueJob`);
  `await` hands the value to a driver written in JavaScript that resumes
  the coroutine when the promise settles; `for await` and async
  generators go through the same driver.
- *Not yet:* `BigInt64Array`/`BigUint64Array` (BigInt itself is B8,
  below), typed arrays over shared bytes (a view of an
  `ArrayBuffer` is a copy), time zones (local time is UTC), full
  Unicode case mapping and normalization.
  A plain function call gets `this === undefined` (strict-mode rule);
  a script's top-level `this` is the global object, and top-level
  `var`/`function` are properties of it. HTML-like comments (`<!--`,
  and `-->` first on a line) are comments, as in classic scripts.

**Pages (done).** `engine/script.aye` gives every page an engine
(`Page.script`) when scripting is on: the window and the agent turn it
on (`--no-js` turns it off), `dump` with `--js`.

- *Loading:* after the parse, the `script` elements run in document
  order (inline, or `src` fetched relative to the page); external
  `defer` and `async` scripts run after the others (async as defer);
  module scripts run with the deferred ones (B8, below), data types are
  skipped; then DOMContentLoaded
  (`readyState` "interactive") and load ("complete"). An uncaught
  exception is a console message and the next script runs; so is a
  promise still rejected with no handler at the end of a microtask
  checkpoint ("Uncaught (in promise) ..."). With
  scripts on, `noscript` is `display: none` (a UA rule).
- *Bindings:* a node's wrapper is a host object holding its
  `Handle(Node)`, one per node (`Script.wrappers`), each counting in
  the node's `js_refs` (also the `style` and `dataset` objects). A
  collected wrapper decrements it (`Host.finalize`), and a tree out of
  the document is freed once no node of it is held
  (`Document.releaseIfUnused`). The collector asks the host to keep
  alive (`Host.trace`) the wrappers of nodes in the document and in any
  tree a reachable wrapper is in, so listeners and expando properties
  last as long as their nodes. A stale handle is a TypeError, never a
  use after free. Natives (`dom_api.aye`): Node (tree, text,
  `appendChild`/`insertBefore`/`removeChild`/`replaceChild`/`remove`,
  `cloneNode`, `contains`), Element (attributes, `id`, `className`,
  `innerHTML` get/set through the HTML parser with the context's table
  or select wrapping, `outerHTML`, `children`, `matches`, `closest`,
  `querySelector(All)` through the CSS selector matcher,
  `getElementsBy*`), Document (`body`, `head`, `title` get/set,
  `readyState`, `activeElement`, `currentScript`, `create*`), and form
  state through `doc.controls` (`value`, `checked`, `selected`,
  `selectedIndex`, focus). `style.camelCase` and `dataset.camelCase`
  are named host properties (`Host.getNamed`/`setNamed`) over the
  `style` and `data-*` attributes. `childNodes`, `children`,
  `getElementsBy*` and `options` are live lists (B8, `lists.aye`);
  `querySelectorAll` gives an array made at the call.
- *Prelude* (`prelude.aye`, JavaScript): EventTarget with capture,
  target (capturing listeners first) and bubble phases through the
  window, `once`, `stopPropagation`, `stopImmediatePropagation`,
  `preventDefault`; Event, CustomEvent, MouseEvent, KeyboardEvent,
  FocusEvent, InputEvent; `onclick`-style properties and attributes
  (compiled when first needed, last writer wins, `return false`
  cancels); `append`/`prepend`/`before`/`after`/`replaceWith`,
  `insertAdjacentHTML`, `classList`, reflected attributes, `click()`
  (the click event, then `engine.activate` unless canceled), `focus()`;
  timers, `requestAnimationFrame`, `queueMicrotask`, `location`
  (setting it navigates after the script returns), `history` (session
  history in B8, below),
  `localStorage` (in memory, per origin, while a page is open) and
  `sessionStorage`, `alert`/`confirm`/`prompt` (console messages, never
  blocking), microtasks (the engine's job queue, which Promise uses),
  and `fetch` (synchronous underneath, settling in a microtask;
  `text()`, `json()`, an aborted `signal`; methods and bodies in B8,
  below).
- *More web APIs* (B8, prelude): `URL` and `URLSearchParams` (parsed
  by `url/`), `atob`/`btoa`, `TextEncoder`/`TextDecoder` (UTF-8),
  `EventTarget` as a constructor, `AbortController`/`AbortSignal`,
  `matchMedia` (the CSS media queries against the viewport),
  `XMLHttpRequest` (on the same fetch; any method, B8 below),
  `crypto.getRandomValues` and `randomUUID` (from `Math.random`), `requestIdleCallback`,
  `reportError`, `Image`, `Option`, `origin`, and `customElements`
  (B8, below). Observer entries are `IntersectionObserverEntry` objects
  (a global, values in prototype getters), so a feature test finds it
  and does not install a polyfill (`jsdom/intersection_observer_entry.html`).
  `IntersectionObserver` checks its targets against the layout and the
  scroll offsets (`engine/geometry.aye`), so content below the fold is
  not intersecting until scrolled into view; `ResizeObserver` reports
  each target once. `loading="lazy"` images and iframes load at once,
  as if in view.
- *Event loop:* timers (ordered by due time, then by scheduling) and
  microtasks after every task. Time is virtual for `dump` and the agent
  (the loop jumps to the next timer, and inside a task the clock moves
  1 ms per million steps, so a scheduler that runs until `Date.now()`
  passes a deadline yields; `STEPS_PER_MS`, `engine/script.aye`;
  `dump --js` runs until no timer is
  left, at most 30 s of page time) and real in the window, whose
  `SDL_WaitEventTimeout` wakes for the next timer. Clicks and keys
  (window, agent) fire their events first and skip the default action
  when canceled; checkbox toggles fire `input` and `change`; a submit
  fires `submit`. A scroll of the window by the user, an agent or a
  script fires `scroll` at the document (bubbling to the window) as a
  task; the agent's viewport follows a script's `scrollTo`.
- *Scrolling through for dumps:* `dump --js` (every mode, `--md` and
  `--a11y` too) then scrolls through the page as a reader would, so
  that what scripts load when it comes into view (observed sections,
  lazy images, infinite lists) is in the dump
  (`engine.scrollThrough`): 500 ms of page time where the page is.
  Only a page that listens for the window's or document's `scroll` or
  observes intersections (`watches_scroll`) is then stepped down, one
  viewport height at a time (firing `scroll`) with 500 ms after each,
  until the bottom is shown and the document stopped growing, after 2
  growth rounds, or after 20 steps (`--scroll-steps N`; `--no-scroll`
  turns it off). A growth round is a step after which the content
  changed and the document is at least half a viewport taller
  (`SCROLL_GROWTH_ROUNDS`): an endless feed stops there, since a dump
  reads what a reader sees near the top (youtube-search loaded ~10 more
  pages of results before; `jsdom/endless_feed.html`), while a page that
  loads its lazy content once still scrolls to the bottom. After a
  step that changed no text or element (`contentSignature`), it stops
  if nothing changed since the load; else it jumps once to the bottom
  (where an infinite list loads more) and stops if that changed
  nothing either.
  The window then goes back to the top at once, without the page
  reacting, so content it would drop out of view stays in the dump. It
  is deterministic (virtual time). The agent's `read` and `snapshot` do not scroll:
  an agent scrolls with `scroll`, then `wait`. After scripts changed the document (`domVersion`),
  the front end styles (style sheets parsed again) and lays out again;
  a navigation they asked for (`takeNavigation`) happens then.
- *Console:* messages ("[log] ...", "[error] Uncaught ...") go to
  stderr for `dump --js` and the window, and with the agent's
  responses (`console`).
- *Stack:* page scripts run on the main thread, so a task may use 5 MB
  of its stack and nest 3000 calls (a RangeError beyond).
- *Budget:* a task (a script, a timer callback, an event, the
  microtasks of a checkpoint) that uses more than 100M steps (in the
  window also 2 s of processor time; on the virtual clock steps alone,
  so where a task ends does not depend on the machine's load, with a
  120 s backstop) ends with a console error, which no `catch` or
  `finally` sees.

### B7: Agent interface

- `dweller agent`: JSON lines on stdio. Commands: `navigate`, `back`,
  `forward`, `reload`, `snapshot` (accessibility tree with refs, roles,
  names, values, states), `read` (Markdown of the main content with
  link refs), `click`, `type`, `select`, `press`, `scroll`, `eval`,
  `screenshot`, `wait` (for load or a selector), `tabs`.
- `dweller mcp`: the same commands as MCP tools over stdio JSON-RPC.
- `dweller --agent-socket PATH` with the window: a human and an agent
  share the page; agent actions are highlighted in the window.
  `dweller connect PATH [--mcp]` bridges stdio to that socket, so an
  MCP client can drive the window a person sees.
- **Acceptance:** scripted agent sessions over fixtures (fill and submit
  a form, follow a link by ref, read a JS-rendered list) with golden
  transcripts; an MCP handshake and `tools/list` golden.
- The protocol is in `docs/agent-protocol.md`. Done so far: snapshot,
  read, navigation, click, type, select, press, scroll, find, `eval`
  and `wait` (scripts run; actions fire their events), `dump --a11y`
  and `dump --md` (`--js` to run scripts).
- The shared window (`ui/agents.aye`, `ui/socket.aye`, `ui/tabs.aye`,
  `agent/shared.aye`): the window polls a non-blocking Unix socket
  between its events, so requests run on its thread; each tab is an
  agent session laid out with the window's fonts. Tabs (Cmd+T, Cmd+W,
  Cmd+1..9, a tab strip) and the commands `tabs`, `tab_new`,
  `tab_select`, `tab_close` (an agent can work in a background tab);
  visible activity (a fading outline with the ref, an `AI` badge, the
  status line); pause (button, Cmd+Shift+P: actions fail with "paused
  by user"); Cmd+Shift+K sends a `user_pointed` event, and
  `user_context` returns the selection, the hovered and focused refs,
  the scroll offset and the URL. `wait` takes real time there.

### B8: The real web

- HTTPS sites render readably and agents can operate them: a fixed list
  of public pages (example.com, a Wikipedia article, Hacker News, a
  documentation site) checked manually and recorded in
  [`docs/browser-sites.md`](browser-sites.md); cookies, gzip, `<meta charset>`, `<base>`,
  `<noscript>`.
- Done so far:
  - *Net* (`net/session.aye`, `net/cookies.aye`, `net/http.aye`): one
    session per process with one libcurl handle (connections, TLS
    sessions and DNS kept), gzip/deflate/br, a cookie jar (Domain, Path,
    Expires, Max-Age, Secure, HttpOnly, `__Secure-`/`__Host-`;
    `DWELLER_COOKIE_FILE` keeps persistent cookies between runs,
    `document.cookie` reads and writes it), a subresource cache,
    redirects followed by hand (each hop stores its cookies), Accept by
    kind, Accept-Language, Referer (strict-origin-when-cross-origin), a
    User-Agent with "Dweller" (also `navigator.userAgent`), 32 MB / 16
    MB size limits, TLS errors. `<base href>` for links, forms, images,
    sheets, scripts, `fetch` and `location`. A page's images, sheets and
    scripts are fetched concurrently (libcurl multi, 6 connections per
    host). The `<meta charset>` prescan follows WHATWG's (all the labels
    of windows-1252 and UTF-8). Background loads (`net/loader.aye`)
    share one multi handle driven by the main thread between tasks.
  - *HTML and CSS:* `template` contents in an inert fragment, foster
    parenting, quirks mode from the doctype, `noscript` following
    scripting in the dumps, svg/math text kept out of them;
    presentational hints (`bgcolor`, `font`, `align`, `width`, table
    `border`, `cellpadding`, ...) and legacy color parsing; custom
    properties and `var()`, `calc()`/`min()`/`max()`/`clamp()` (also in
    media queries); `@supports`, `@layer` (as plain rules), `@import`
    (relative to the importing sheet, at most 4 deep), media queries
    level 4, `hsl()`, `light-dark()`, system colors; `font-family`
    names map to sans, serif or monospace faces, serif by default.
  - *Layout and paint:* floats and `clear`; `position: absolute` and
    `fixed` (out of flow, laid out last in the containing block);
    `display: grid` (track lists with `fr`, `minmax`, `repeat`, areas,
    line placement, auto-placement); replaced boxes (svg, iframe,
    embed, object, video, canvas) sized 300x150 by default, their
    `width`/`height` attributes presentational hints (any author value
    wins; `img` and `video` also get `aspect-ratio: auto W / H`), min
    and max sizes by the full CSS 2.1 §10.4 table, keeping the ratio
    (`layout/replaced_sizing.html`, `layout/replaced_minmax.html`); z-index
    layers in paint and hit testing; frames shown as links; a closed
    `details` shows only its `summary`.
  - *JavaScript:* generators and async functions, Symbol and the
    iterator protocol, iterator helpers (`Iterator.prototype.map`,
    `filter`, `take`, `drop`, `flatMap`, `reduce`, `toArray`,
    `forEach`, `some`, `every`, `find`, `Iterator.from`; `js/lib.aye`,
    `browser/tests/js` 240–241), `Function` and `eval` of strings, the
    web APIs above, a processor-time budget per task (B6, above).
    BigInt (`js/bigint.aye`, by GPT-6.1-sol through pi): literals,
    conversions, exact arithmetic, bitwise operations and comparisons
    with Numbers (`browser/tests/js` 246–248; limits in
    [`browser/js/BIGINT.md`](../browser/js/BIGINT.md)).
    `Object.defineProperty` follows ValidateAndApplyPropertyDescriptor:
    a generic descriptor keeps an accessor, a non-configurable property
    changes only as the spec allows (`js/builtins.aye`, 249–250).
    ToPropertyDescriptor checks run before any definition (a
    non-callable getter, value mixed with get/set); `Reflect`'s boolean
    methods return false only for a rejected operation and throw for a
    non-object target (251); `setPrototypeOf` rejects cycles,
    non-extensible objects and immutable prototypes, `Object.` and
    `__proto__` throwing, `Reflect.` returning false (252). ES modules
    (`js/modules.aye`, `js/lib.aye`): static imports and exports,
    `import()`, `import.meta`, cycles as cyclic module records (TDZ,
    top-level await), every re-export form with `export *` ambiguity,
    JSON modules, import maps, `<script type=module>` and
    `modulepreload`; checked against Node in `browser/tests/js`
    231–237 and 242–245. What is not supported is in
    [`browser/js/MODULES.md`](../browser/js/MODULES.md).
  - *JS and DOM APIs for real sites:* the init of a `for` loop is
    resolved (jQuery 3, minified bundles); Annex B block functions;
    sloppy-mode `this`; `\u` escapes in identifiers; `Proxy`; Array
    methods on array-likes. `createElementNS`, `DocumentFragment`,
    `CustomEvent`/`initCustomEvent`, the Event constructors, interface
    globals per tag that work with `instanceof` and prototype patching;
    select `add`/`remove`/`options`/`selectedIndex`, `:scope`,
    `performance` timing/navigation/mark/measure, live collections,
    `document.write` while parsing, TreeWalker/NodeIterator, Range,
    DOMParser (through the HTML parser; parsed nodes keep their node
    document until inserted elsewhere, which adopts the whole subtree;
    `jsdom/dom_parser.html`), Blob/File/FileReader, FormData, MessageChannel,
    constructable style sheets, `a` URL parts, Navigator;
    `getComputedStyle` styles only the element and its ancestors;
    `document.baseURI` and `Node.baseURI`. Custom elements (upgrades,
    reactions, `:defined`); shadow roots (`attachShadow`, declarative
    `shadowrootmode`, slots, `:host`, `::slotted()`, `::part()`; the
    flat tree is rendered; `dom/shadow.aye`, `engine/components.aye`);
    `MutationObserver` records from the document's mutation log. Web
    Animations basics (`Element.animate`, `Animation`, `KeyframeEffect`,
    `document.timeline`): an animation keeps time and fills write the
    inline style, no frame is interpolated. Layout queries
    (`engine/geometry.aye`): `getBoundingClientRect`, `getClientRects`,
    `offsetTop`/`Left`/`Width`/`Height`/`Parent`, `client*`,
    `scrollTop`/`Left`/`Width`/`Height`, `scroll`/`scrollTo`/`scrollBy`
    of elements and the window, `scrollX`/`scrollY`, `scrollIntoView`,
    through a forced synchronous layout; transforms are ignored and
    ranges have no geometry. The
    checklist's scripted sites (Python docs, rustdoc, CNN Lite,
    react.dev, YouTube, go.dev, BBC, Google, ZOL) log no errors.
  - *Fetch and XHR* (`engine/prelude.aye`, `net/http.aye`): `Request`,
    `Headers` (every method, sorted iteration, `set-cookie` kept
    apart), and the body mixin shared by Request and Response (`text()`,
    `json()`, `arrayBuffer()`, `blob()`, `formData()` of multipart and
    urlencoded bodies, `bodyUsed`, `body` as a minimal `ReadableStream`
    with `getReader`, and `tee` for `clone()` of either;
    `jsdom/body_mixin.html`);
    `new Response(body, init)` takes the standard arguments. `fetch` and `XMLHttpRequest` (`send(body)`,
    `setRequestHeader`) send any method, request headers and a string,
    URLSearchParams, FormData, Blob or ArrayBuffer body with its
    Content-Type default; redirects follow the Fetch method rules (a
    303, and a 301 or 302 of a POST, continue as a GET without the
    body). Tests: `jsdom/request.html`, `net/fetch_methods.html`,
    `net/xhr_methods.html`.
  - *Session history* (`engine/history.aye`, `engine/tab.aye`): one
    history per tab and per frame; `pushState`/`replaceState` add or
    change entries without loading (an empty URL resolves to the base
    URL, dropping the fragment; an omitted or null one keeps the URL),
    back and forward within a document
    fire `popstate` (and `hashchange` when the fragment changed),
    `history.length`, `state` and `scrollRestoration`. The agent's
    `url` follows (`docs/agent-protocol.md`).
  - *Canvas 2D* (`engine/canvas_api.aye`, `engine/canvas_prelude.aye`,
    by GPT-6.1-sol through pi): paths and Path2D, fills, strokes
    (joins, caps, dashes), transforms, clipping, linear and radial
    gradients, patterns, shadows, Porter-Duff composites, text and
    metrics, `drawImage` from images, canvases and ImageBitmap,
    ImageData, PNG `toDataURL`/`toBlob`, hit tests, OffscreenCanvas;
    cross-origin draws taint the canvas. Drawing reuses SVG's paths and
    coverage and paint's sampling and blur, and a draw renders only the
    pixels it can change (31–636x faster, `browser/tests/canvasbench`).
    Tests and limits are in
    [`browser/tests/canvas/README.md`](../browser/tests/canvas/README.md).
  - *Layout of current CSS:* `transform` and `transform-origin` (the
    translation moves the box after layout; scales, flips and quarter
    turns are drawn and hit-tested); a transformed box is a layer and
    the containing block of its absolute and fixed descendants.
    `position: fixed` boxes stay in the viewport while scrolling and
    `sticky` ones stick within their parent (`layout/scroll.aye`).
    `::before`/`::after` with `content` (strings, `attr()`, quotes);
    `:is()`/`:where()`; logical properties; `aspect-ratio`;
    `object-fit`; `clip: rect()` and `clip-path: inset()`. Percentage
    table columns and `<center>` around auto-width tables; grid gaps
    under spanning items; flex items that clip their overflow have no
    automatic minimum. `text-transform` (inherited) changes the case
    of inline text before it is measured (`layout/inline.aye`).
  - *SVG* (`svg/`): inline `<svg>` and SVG images (`<img>`, CSS
    images, `data:` URLs) become pictures: `path` (all commands, arcs as
    cubics), `rect` (rounded), `circle`, `ellipse`, `line`, `polyline`,
    `polygon`, `g`, nested `svg`, `use`/`symbol`, `transform`, `viewBox`
    and `preserveAspectRatio`. Presentation attributes are hints in the
    cascade, with the properties `fill`, `stroke`, their opacities,
    `stroke-width`, `fill-rule`, caps, joins, dashes, `opacity`,
    `stop-color`; `currentColor` works. Linear and radial gradients are
    paint servers; group `opacity` is drawn into its own layer;
    `clip-path` from a `clipPath`. Curves are flattened, strokes become
    polygons, fills use antialiased scanlines (nonzero and even-odd).
  - *Backgrounds:* `background-image` (images and linear, radial,
    conic and repeating gradients), several layers, `-position`,
    `-size`, `-repeat` (including space and round), `-origin`, `-clip`,
    a basic `fixed` attachment, the `background` shorthand; images in
    style sheets are fetched with the page's and resolved against their
    sheet. `mask-image` (and `mask`, `-webkit-` names), `border-radius`,
    blurred outer and inset `box-shadow`.
  - *Images:* progressive JPEG, Adobe CMYK and YCCK, EXIF orientation;
    WebP lossless (VP8L) and lossy (VP8, bit-exact with libwebp) with
    VP8X, ALPH alpha and the first frame of an animation; formats are
    sniffed. `picture` sources, `srcset` (x and w) and `sizes` pick an
    image's URL; image requests accept WebP. A script-visible http(s)
    `img` request loads in the background; then a task decodes it, sets
    `complete`, `naturalWidth` and `naturalHeight`, fires `load` or
    `error` and lays the page out again (`engine/image_api.aye`). The
    window's `load` event waits for those tasks, and `decode()` promises
    settle with them. A page fetches a failed image URL once
    (`engine/images.aye`, `net/failed_image_once.html`).
  - *Fonts and text* (`font/`, `webfont/`, `css/fontface.aye`,
    `layout/bidi.aye`, `layout/linebreak.aye`): a character the font
    lacks comes from a chain of system fallback fonts (PingFang,
    Hiragino, Apple SD Gothic Neo, Arial Unicode, Apple Symbols, STIX,
    Apple Color Emoji; Noto and DejaVu on Linux). Text is shaped by
    HarfBuzz when it is installed (complex scripts, emoji sequences,
    ligatures), and layout measures with the same shaping. East Asian
    line breaks follow basic UAX 14; bidi basics (`direction`, `dir`,
    levels, lines reordered). `@font-face` in TTF, OTF, WOFF and WOFF 2
    (Brotli), matched by family, weight, style and unicode-range, icon
    font ligatures included.
  - *Encodings and frames:* Shift_JIS, EUC-JP, ISO-2022-JP, GBK,
    gb18030, Big5 and EUC-KR (WHATWG decoders and encoders, tables
    generated from the index files) in the meta prescan and
    Content-Type; forms submit in the page's encoding or
    `accept-charset`. `<frameset>`/`<frame>`: the frameset replaces the
    body, rows and columns (px, %, *) are laid out, each frame's
    document is loaded, laid out, painted and scrolled in its
    rectangle; `--md` puts each frame under `## Frame: URL`, the
    snapshot has `frame` nodes.
  - *Agent Markdown:* invisible list items are dropped, layout tables
    are read row by row, `pre` keeps its line breaks (also in the
    snapshot).
  - *Iframes:* `srcdoc`, `src` or about:blank load as pages of their
    own; `sandbox` turns off scripts and gives the frame its own
    origin; at most 3 deep and 16 frames per tab. Scripts use
    `parent`, `top`, `frames[i]`, `frameElement`, `contentWindow` and
    `contentDocument` (same origin), `postMessage` between frames and
    iframe `load` events; iframes added by scripts, or whose `src`
    changes, load at the next layout. Links and forms follow their
    `target`. The snapshot and `read` give refs inside frames and agent
    actions work there; in the window, clicks reach the frame's
    document.
  - *HTML5 forms* (by GPT-6.1-sol through pi): text-like and calendar
    inputs (date, time, datetime-local, month, week), number controls,
    draggable range sliders, color swatches, file placeholders,
    `output`, `progress` and `meter`. Constraint validation (required,
    pattern, lengths, numeric and calendar bounds and steps, email and
    URL mismatches, custom errors) with live `ValidityState`,
    `checkValidity`/`reportValidity`/`setCustomValidity`,
    `requestSubmit`, `reset` and FormData from the form's entries.
    Invalid submissions fire `invalid`, focus the first invalid control
    and show a tooltip in the window or a message to the agent. Form
    owners (`form=`), labels, disabled fieldsets, submitter overrides
    (`formaction`, `formmethod`, `formenctype`, `formnovalidate`),
    `multipart/form-data`, the state pseudo-classes, and the a11y roles
    and states (spinbutton, slider, required, invalid).
  - *Load time and measurement:* computed styles are kept when no
    style input changed (`css/inputs.aye`: a fingerprint of the trees,
    the classes the sheets name, form state, viewport and container
    sizes). `browser/tools/page_load.py` captures every listed site to
    an archive (`target/sites/`) and replays it offline, with median
    wall time, CPU time and load average (each run's output goes to
    `target/page-load-out/<site>/last.stdout` and `last.stderr`, never
    into the archive; the CPU/wall warning needs a run of 1 s or more); `check` replays every site
    against `browser/tools/sites.json` (status, most console errors,
    Markdown it must contain) and prints each site's visual verdict
    and the W3 count, `check --agent` also runs one agent task
    per site (`browser/tools/agent_tasks/`). Capture also records what
    the `--screens` screenshot fetches (web fonts, images); on replay a
    non-GET request without a record of its body gets the latest
    response recorded for its URL ("archive fallback:"), so moved
    beacons do not fail a replay; `--screens` paints a contact sheet
    (`target/screens/index.html`), `run --live` loads from the network,
    `rebless --site X` stores today's replay as the archive's reference
    after an intended dump change, and `capture` refuses a page that
    misses its `sites.json` strings. Details and numbers in
    [`browser/engine/PERF.md`](../browser/engine/PERF.md).
  - *Performance on big pages:* layout changes deduplicated before the
    full-restyle cap (4096), the ancestor filter outside shadow trees, a
    rule index of its own for `::before`/`::after`, computed-style
    arrays shared between elements with equal values, a hash index for
    Map and Set keys, a shape table of 262144 shapes, and the document
    base URL cached per document. youtube-search CPU ~77 s → ~11.6 s
    (with the scroll budget above); a 40K-element page's peak RSS 5.58 →
    1.99 GB.

### B9: Agents without disturbing the user

Goal (2026-10-14): agents operate the browser while the person keeps
using it, and nothing the agents do changes what the person sees or
does. Each agent works in its own **Space**, a set of tabs inside the
same browser; the person sees which Spaces have an agent working, can
watch one, pause or stop its agent, and take a tab over. Agents use the
person's logins. The reference is ego lite (citrolabs/ego-lite): one
Space per agent, Spaces in parallel, and a "code mode" where one
request runs a script that calls the browser's tools. Plan: P9 in
[`docs/plans/browser.md`](plans/browser.md).

**Today (measured 2026-10-14 on `main` 6e2e701).** Probes drove a
headless window (`SDL_VIDEODRIVER=dummy`, `--agent-socket`) from two
connections; the person's page had a focused text field with a
selection and was scrolled to 200 px.

| | Requirement | Today | Evidence |
|---|---|---|---|
| R1 | No agent action changes the person's selected tab, focus, selection, scroll or pointer | done (P9.1, 2026-10-14) | A connection acts on a tab of its own (opened on its first command that needs a page, or by `tab_new`); the person's tabs are look-only (actions fail with "this is the user's tab: open your own with tab_new"); `show` is refused; the tab strip row stays while the window listens. Test: `window_agents` `isolation`. Before P9.1: in a background tab (`tab_new`), navigate, type, click, scroll, eval and `tab_close` left the person's scroll (200), selection and focus as they were. But a connection acts on the shown tab until it opens or selects another (`connSlot`, `ui/agents.aye`): a second connection's `type` replaced the person's text and cleared the selection. `tab_new`/`tab_select` with `show` switch the person's tab (`ui/agents.aye` `windowCommandOf`); the second tab makes the tab strip appear and the page viewport 28 px shorter, and closing it makes it taller (`openTab`, `removeTab` in `ui/tabs.aye`). The OS pointer is never moved. |
| R2 | No agent request blocks the window (input within 16 ms, frames unchanged) | done behind `--spaces` (P9.2, 2026-10-14) | With `--spaces` each connection is served by a Space process (`ui/spaces.aye`) and the window only copies bytes: while A ran a 1.5 s `eval` and B loaded a page whose server waited 1.5 s, C's `tabs` took 11 ms and the window's `loop_max_ms` was 13 (the test's debug build, during the full browser suite; 8 to 32 ms across runs on a loaded machine, 3 when idle), against 3122 ms (C: 2834 ms) without `--spaces`. A Space answers its first request about 20 to 35 ms after the connection's first line and takes about 43 MB after a small form page; it starts with `posix_spawnp`, about 1 ms of the window's loop. Tests: `window_spaces_responsive`, `window_spaces_crash`. Spaces become the default in P9.9. Before P9.2: requests run on the window's thread (`pollAgents` in `Ui.loop`, `ui/window.aye`). While connection A loaded a page whose server took 2 s, connection B's `user_context` waited 1733 ms; during a 2 s `eval` loop, 1714 ms; the person's input waits the same way. `wait` does not block (B answered in 4 ms during A's 2 s `wait`), nor did a small `screenshot`. Every request also laid out the shown tab again (`afterAgent`; since P9.1 only a request served on the shown tab or that changed it), and answers come at most every 15 ms (`POLL_MS`). Background tabs run their timers only during a `wait` (`checkWait`). |
| R3 | Each agent's tabs form a Space, shown apart from the person's tabs | no | Agent tabs are appended to the person's tab strip (`openTab`), count against the same 16 tabs, and `tabs` lists every tab; an agent can `tab_select` any tab, including the one the person sees. |
| R4 | Several agents at once, each in its own Space | partial | Up to 8 connections (`MAX_AGENTS`), each with its own tab. But one agent's load or `eval` stalls all the others (R2), pause is for all of them, and the activity line and outline show only the last action of any agent. |
| R5 | Spaces use the person's cookies and storage; an isolated profile is an option | partial | One net session per process (`net/session.aye`) and one `localStorage` per process (`engine/window_api.aye`): all tabs share the person's logins. There is no isolated profile. |
| R6 | The person sees which Spaces work, watches one live, pauses or stops one, takes over a tab | partial | One `AI` badge for all agents and one status line (`drawAgentChrome`, `agentStatus`). An agent's background tab can be watched by clicking it in the strip. Pause is global (`togglePause`); no stop per agent; input in an agent's tab does not stop the agent. |
| R7 | Code mode: one request runs a script calling snapshot, click, fill, wait, navigate, read, screenshot | no | `eval` runs a script in the page's realm and returns its last value: it can read the DOM and call `el.click()`, but has no refs, snapshot, `wait` or screenshot, and a navigation ends the realm it runs in. |

The probes were throwaway scripts; P9.1 and P9.2 turn them into tests.

**Acceptance checks.** Each one is a test in `tests/browser.rs` (P9
cards say which):

- R1: with the person's page scrolled, a text field focused and text
  selected (scripted events), agents in their Spaces navigate, type,
  click, scroll, `eval` and close tabs; then `--state` and
  `user_context` show the same tab, scroll, focus, selection, address
  bar text and page viewport height as before. An agent cannot act on
  a tab of the person's.
- R2: while one agent runs a 1.5 s `eval` loop and another loads a page
  whose server waits 1.5 s, the window's longest loop iteration
  (`loop_max_ms` in `--state`) stays under 50 ms (the target is 16 ms;
  50 leaves room for CI machines), and a third connection's `spaces`
  is answered in under 100 ms.
- R3: `tabs` in a Space lists only that Space's tabs; the person's tab
  strip and Cmd+1..9 never include them; `tab_new` in a Space puts the
  tab in that Space; the person's tab count and order are unchanged.
- R4: three agents in three Spaces work at once (each navigates, types
  and reads); each one's answers are the same as when it runs alone,
  and pausing one Space leaves the other two working.
- R5: a cookie the person's tab got (test server login) is sent by an
  agent's Space; a cookie a Space got is sent by the person's next
  request; the same for `localStorage`; a Space opened with `"profile":
  "isolated"` sends neither.
- R6: `--state` lists each Space with its state and last action; Pause
  and Stop act on one Space; input in a Space's tab makes that tab's
  actions fail with "the user took over tab N" until Hand back, while
  the Space's other tabs and the other Spaces go on; watching shows the
  Space's frames.
- R7: one `run` request navigates, types, clicks, waits and reads, and
  returns a JSON value, its log and the number of calls; an error names
  the failing call.

**Threading model: a process per Space.** R2 fails because the window
runs agent work on its own thread. Three ways out were weighed:

- *Cooperative slices on the window thread* (load with the libcurl
  multi handle the loader already has, run scripts on the coroutines
  generators already use, and yield every few ms). This fixes network
  waits and long scripts, but not style and layout of big pages
  (YouTube search: 11.6 s of CPU), which would need yield points inside
  the cascade, where module-level state lives.
- *A thread per Space.* The engine keeps process-wide state in about 26
  module-level variables: the net session and loader
  (`net/session.aye`, `net/loader.aye`), `localStorage`
  (`engine/window_api.aye`), the CSS scratch state of `css/calc.aye`,
  `css/selector.aye`, `css/media.aye` and `css/vars.aye`, and script
  flags (`engine/script.aye`). Every one would move into a per-engine
  context first; std.thread also wants an allocator per thread, and a
  panic in any thread ends the program, so a page that crashes an agent
  closes the person's window.
- *A process per Space* (chosen). The window already runs headless
  (`SDL_VIDEODRIVER=dummy`) with `--agent-socket`, which every window
  agent test uses: a Space is that program, started by the window. It
  starts in about 25 ms with about 45 MB resident on a small page. Its
  globals are its own, a crash ends only that Space, and its event loop
  runs its own tabs' timers. The costs: memory per Space, cookies and
  `localStorage` copied between processes (R5), and frames copied to
  the window when the person watches (R6).

How it works:

- `dweller space --control FD [--width W --height H]` (internal) is the
  window program, headless, with a private agent socket (in a
  directory only the user can read) and a control connection to its
  parent window (a `socketpair`, JSON lines).
- The window keeps its socket. For each new connection it reads the
  first line (the hello, below), starts or finds the Space, and from
  then on copies bytes both ways between the agent and the Space's
  socket without reading them (non-blocking, as `dweller connect`
  does). The window's thread does no agent work, so its loop time does
  not depend on what agents do.
- The Space serves the whole protocol, MCP included, as the window
  serves it today; the commands that need the person (`user_context`,
  pause) are answered from what the window sent over the control
  connection.
- The person's tabs stay in the window process. Agents can look at the
  shown one (`user_context` with `snapshot`) but not act on it.

The control connection (internal; both ways JSON lines, `{"ctl":...}`):

| From | `ctl` | Members | Meaning |
|---|---|---|---|
| window | `pause`, `resume`, `stop` | | the person paused, resumed or stopped this Space's agent |
| window | `input` | `events`: words of `--events` (`click:X,Y text:abc`) | the person's input while watching, in the Space's window coordinates |
| window | `watch` | `on` (bool), `width`, `height`, `scale` | send frames (or stop); resize the Space's window |
| window | `hand_back` | `tab` | the person gives a tab back to the agent |
| window | `context` | the members of `user_context` | the person's context, sent when it changes |
| window | `pointed` | the members of the `user_pointed` event | Cmd+Shift+K |
| both | `cookie` | `url`, `line` (a Set-Cookie line), `clock` | a cookie the sender's jar stored |
| both | `storage` | `origin`, `key`, `value` (null: removed; key null: cleared) | a `localStorage` change |
| Space | `status` | `state`, `last`, `tabs`, `title`, `url` | for the Space's chip |
| Space | `frame` | `seq`, `width`, `height` | a new frame is in the Space's frame file (RGBA, the canvas size) |

**The Space model.**

- A Space has an id (a number, never reused), a name (from the hello,
  else "Agent N"), a profile (`shared` or `isolated`), its tabs (ids
  are the Space's own, from 1) and a state: `working` (a request is
  running), `waiting` (a `wait` or a load), `idle`, `paused`,
  `taken_over` (the person has at least one of its tabs), `done` (its
  agent left), `crashed`.
- A new connection gets a new Space by default; its first tab is
  opened on its first request that needs a page. At most 8 Spaces.
- When the agent leaves, the Space stays (`done`) so the person can
  look at it, until the person closes it or an agent attaches to it
  (`"space": N` in the hello). A crashed Space is shown as such; its
  connection gets an error event and is closed.
- `profile: "shared"` (default): the Space starts with the window's
  cookies and `localStorage` and both sides send each other every
  change (the jar's `store`, the store's writes). Cookies of the
  session (no Expires) are copied too. `isolated`: an empty jar and
  store, nothing sent either way.

**Protocol additions** (the JSON lines of
[`agent-protocol.md`](agent-protocol.md); MCP tools are `browser_` and
the name).

The hello, extended. The first line may be an object without `cmd`
and without `jsonrpc`; no response:

```
{"mcp":true,"name":"Claude Code","space":"new","profile":"shared"}
```

`mcp` (bool, default false), `name` (string), `space` (`"new"`, the
default, or the id of a `done` Space to attach to), `profile`
(`"shared"`, the default, or `"isolated"`). `dweller connect PATH
[--mcp] [--name NAME] [--space N] [--isolated]` writes it.

| `cmd` | arguments | response fields |
| --- | --- | --- |
| `spaces` | | `result` ("2 spaces"), `spaces`: `[{"id":3,"name":"Claude Code","mine":true,"state":"working","last":"click button \"Sign in\" [e3]","tabs":2,"profile":"shared","watched":false}]` |
| `space_close` | | `result`; closes this connection's Space and then the connection |
| `tabs`, `tab_new`, `tab_select`, `tab_close` | as today, without `show` | only this Space's tabs; `show` is refused: "the user chooses what they see" |
| `user_context` | `snapshot` (bool) | as today, plus `snapshot` of the person's shown tab when asked (to look at; its refs do not act) |
| `run` | `code` (the body of a function, at most 1 MB), `ms` (at most this long, default 60000) | `result` ("ran 4 calls in 812 ms"), `value` (what the code returned, as JSON; `undefined` is `null`), `log` (array of strings), `calls`, `url`, `title`; an error: `error` ("click: no such ref e99 (call 3)"), `log`, `calls` |

Events (as `user_pointed`: a JSON line, or an MCP
`notifications/message`):

```
{"event":"paused","space":3}
{"event":"resumed","space":3}
{"event":"taken_over","space":3,"tab":2}
{"event":"handed_back","space":3,"tab":2}
{"event":"stopped","space":3}
```

Refusals: actions fail with `"paused by user"` while the Space is
paused, `"the user took over tab 2 (wait for handed_back)"` on a tab
the person has, and after `stopped` the connection is closed.

**Code mode (`run`).** The code runs in a realm of its own in the Space
process (not the page's, so a navigation does not end it and the page
cannot see it), with these functions. Each one does what the command of
the same name does, waits for it, and returns its response object
without `id` and `ok` (`url`, `title`, `snapshot` or `diff`, `result`,
...); a failure throws an `Error` whose message is the error:
`navigate(url)`, `back()`, `forward()`, `reload()`, `snapshot(opts)`,
`read(opts)`, `find(text)`, `click(ref)`, `type(ref, text, opts)`
(alias `fill`), `select(ref, option)`, `press(key)`, `scroll(refOrDy)`,
`wait(opts)`, `evaluate(expression)` (in the page; its `result`),
`screenshot(opts)` (`width`, `height`, `png`), `tabs()`, `tabNew()`,
`tabSelect(id)`, `tabClose(id)`, `log(...values)`. Between calls the
Space reads its control connection, so pause and stop take effect at
the next call. Example:

```
{"id":4,"cmd":"run","code":"navigate('browser/tests/agent/site/form.html'); fill('e2', 'Ada'); click('e4'); return snapshot().snapshot.includes('[checked]')"}
{"id":4,"ok":true,"result":"ran 4 calls in 40 ms","value":true,"log":[],"calls":4,"url":"file:///.../form.html","title":"Sign up"}
```

**What the person sees.** Kept small; nothing in the window changes
height because of agents:

- The toolbar's agent area (where the `AI` badge and Pause are now)
  holds one chip per Space: a dot (purple working or waiting, grey
  idle or done, amber paused, blue taken over, red crashed) and the
  name; the status bar shows a chip's last action when the mouse is
  over it. The area grows sideways (the address bar gets shorter), never
  downwards.
- A click on a chip shows that Space: the page area shows the Space's
  window (its own toolbar, its tab strip of the Space's tabs, its page)
  from its frames, live, with Pause/Resume, Stop and, for a taken-over
  tab, Hand back in the agent area. A click on "You" (the first chip)
  or Cmd+Shift+0 goes back to the person's tabs; Cmd+Shift+1..8 show a
  Space.
- The person's tab strip shows only the person's tabs. The strip's row
  is kept while the window listens for agents, so it never appears or
  goes away because of them.
- Input (click, key, text, wheel) in a shown Space goes to that Space
  and takes the tab over; moving the mouse does not.
- Cmd+Shift+P pauses the shown Space, or every Space when the person's
  tabs are shown.

**Risks.**

- *Logins across processes (R5).* Cookies and `localStorage` are copied
  by messages, so a Space and the window can disagree for a moment (a
  login in one is seen by the next request of the other, not the
  current one), and two writes of one cookie settle by arrival order.
  `sessionStorage`, IndexedDB and in-memory page state are not shared
  (Dweller has no IndexedDB). Today `localStorage` is not saved
  between runs at all.
- *Frames (R6).* A 1600×1200 canvas (800×600 at scale 2) is 7.7 MB per
  frame; frames are sent only while the person watches, at most 30 per
  second, through a file both processes map.
- *Memory.* About 45 MB per idle Space, far more on big pages; at most 8
  Spaces.
- *Code mode (R7).* A second JS realm in a process whose page scripts
  run at the same time, with calls from that realm into actions that
  run page scripts: the interpreter has run one realm per page so far.
  The engine's script flags (`engine/script.aye`) are process-wide.
  P9.8 starts with a test of two realms before it adds the functions.
- *Compatibility.* Connections no longer act on the person's tab by
  default, and `show` is gone: `shared.jsonl` and the MCP instructions
  change (P9.1).
