# Plan: the V lane (one engine, three surfaces)

The cards of `docs/vision.md`. At most one V card is in flight beside
the browser lanes until V2 is done. Card format and rules as in
`docs/plans/browser.md` (one card per agent, its verify command, a
45-minute time box, the done note on the card).

| Step | Card | What | Depends on | Status |
|---|---|---|---|---|
| V1 | V1 | layering rules written down and checked in CI; `layout` stops importing `js` | – | open |
| V2 | V2 | native shell spike: an Aye program opens a window on the engine with a DOM built from code | V1 | open |
| V3 | V3 | component API: one DOM-shaped interface, engine and wasm implementations | V2, framework F4 | open |
| V4 | V4 | terminal emulator spike: PTY, VT parser, cell grid drawn by the text stack | V1 | open |
| V5 | V5 | agent protocol over native apps (`--agent` on a V2 app) | V2 | open |
| V6 | V6 | abstract node tree for `css/`, `layout/`, `paint/` | V3, V4 | open |
| V7 | V7 | TUI backend: layout into cells, ANSI output | V6 | open |
| V8 | V8 | Windows: CI job and the platform layer | V2; Aye on Windows (aye-lang/aye) | open |

#### V1: layering rules, checked
- goal: the module layers of `docs/vision.md` ("The shape") are a rule, not a habit. Platform modules (`ui`, `gpu`, `audio`, `font`, `net`, `perf`), render core (`css`, `layout`, `paint`, `image`, `svg`, `webfont`, `encoding`, `url`, `dom`), front end (`html`, `js`, `wasm`, `engine`, `agent`). A render-core module imports no front-end module, and nothing imports `ui` except the root. Today's exceptions are listed in the checker with the card that removes each (`svg` → `html`, `layout/controls.aye` and `paint/form_widgets.aye` stay as HTML form code in the core until V6).
- design: `browser/tools/check_layers.py` (standard library): reads the `import` lines of every `.aye` under `browser/`, maps each to its module, checks the table in the script against the graph and the allowed exceptions, prints violations with file:line, exit 1 on any. A CI step in `.github/workflows/ci.yml` on Ubuntu runs it (seconds). `docs/browser.md` "Layout of the code" gets the layer table and points to the script. Move `caseMap` (and what it needs) from `js/` to a text helper in the core (`font/` or a new `text/`), so `layout/inline.aye` drops `import js`.
- files: `browser/tools/check_layers.py`, `.github/workflows/ci.yml`, `docs/browser.md`, `browser/layout/inline.aye`, `browser/js/` (the `caseMap` move)
- verify: `python3 browser/tools/check_layers.py; echo $?` → 0; adding `import js` to a `paint/` file makes it exit 1 with the file:line (checked by hand, not committed); `cargo test --release --test browser -- js_conformance layout > target/v1.log 2>&1; echo $?` → 0
- time box: 45 min; model: sonnet

#### V2: native shell spike
- goal: an Aye program, not the browser, opens a window and runs a small app on the engine: a counter (text and a button), a list of 1000 rows that scrolls, a text input. The DOM is built from code (`dom.Document` as the scene graph); no HTML parser, no JS VM linked in. A click changes the DOM and the frame relays out incrementally, as the browser's window does. Report: binary size, time to the first frame, idle CPU, frame time while scrolling, against `dweller` on an equivalent HTML page.
- design: `examples/native/counter/` with its own `main`. Find the smallest seam: the window loop of `ui/window.aye` is browser-shaped (tabs, chrome); extract or write a minimal "window + document + style + layout + paint + input" loop that both can share later (`ui/surface.aye` or similar), without changing the browser's behavior. Styles come from a CSS string through `css/` (no `<style>` parsing needed beyond the sheet parser). Events: hit testing on the layout tree, a handler table keyed by node id. How the example imports the engine modules (an import path into `browser/`) is part of the result; write what the Aye compiler would need for a cleaner library boundary as a gap in `browser/COMPILER_GAPS.md`.
- files: `examples/native/counter/`, `browser/ui/` (the shared loop), `tests/browser.rs` (`native_counter`: run headless with `--frames` and an events script, compare the final DOM dump and a screenshot)
- reference: the browser's own window loop (`ui/window.aye`, `ui/pagedraw.aye`); Flutter's embedder idea (one engine, a thin host) as an idea only
- verify: `cargo test --release --test browser -- native_counter window_ > target/v2.log 2>&1; echo $?` → 0; the numbers above in the done note, measured on the user's Mac
- depends on: V1; time box: 45 min (split the text input into V2b if larger); model: opus

#### V3: one component API, two implementations
- goal: an Aye UI component written once runs as a framework island (wasm32 in any browser) and in a native window (V2). The API is DOM-shaped (create element, set attribute and style, append, listen), defined once; implemented by JS glue on wasm32 and by `dom.Document` natively. State and re-render are the smallest thing that works for the counter and the list; no virtual DOM until a measurement asks for it.
- design: after V2's spike and the framework's F4 islands (aye-lang/aye `docs/framework.md`, "Client code"). Write the API as a short design note in `docs/vision.md` first, then the two implementations and one example component built both ways.
- files: decided by the design note
- verify: the same component's DOM dump is equal from `dweller` loading the island page and from the native build
- depends on: V2, the framework's F4; time box: 45 min per slice; model: opus

#### V4: terminal emulator spike
- goal: a window that runs the user's shell: PTY (`forkpty`), a VT/xterm parser (CSI, SGR colors and attributes, cursor movement, alternate screen, scroll region, OSC title), a cell grid drawn with the text stack (glyph cache, `paint/glyphs.aye`) and the compositor; keyboard input encoded as xterm does; scrollback. Acceptance: `vim`, `htop`, `less` and `claude` run and look right.
- design: `apps/term/` (new). The parser follows the state machine of Paul Williams' DEC parser (vt100.net). Damage per row, so idle CPU stays near zero.
- reference: vt100.net "A parser for DEC's ANSI-compatible video terminals"; xterm's `ctlseqs` document
- verify: a test that feeds recorded byte streams (vim start screen, htop frame, colors) to the parser and compares the cell grid dump; the acceptance programs by hand on the user's Mac, noted in the done note
- depends on: V1; time box: 45 min (split the parser from the window); model: opus

#### V5: agents on native apps
- goal: the agent protocol of `docs/agent-protocol.md` works on a V2 app: the same `--agent` session reads its semantic tree and clicks, types and scrolls, as on a web page.
- depends on: V2; time box: 45 min; model: sonnet

#### V6: an abstract node tree
- goal: `css/`, `layout/` and `paint/` take an interface for the tree (children, kind, attributes the cascade needs, text) instead of `dom.Document`; `dom.Document` implements it with no slowdown (`page_load.py check`, the layout benchmarks unchanged). The HTML-specific spots (replaced-element tag names, form controls, `canvas`/`svg` in paint) move behind the interface.
- depends on: V3, V4; time box: 45 min per module; model: opus

#### V7: TUI backend
- goal: a cell-grid `font.Measurer` and a display-list consumer that writes ANSI, so a V3 component renders in a terminal.
- depends on: V6

#### V8: Windows
- goal: a `windows-latest` CI job that builds and runs the browser's headless tests, then the window. Needs the Aye compiler to target Windows first (a card in aye-lang/aye).
- depends on: V2; Aye on Windows
