# Vision: one engine, three surfaces

Status: direction, 2026-10-16. Cards: `docs/plans/vision.md` (V lane).

**Decision (user and CTO, 2026-10-16).** The Aye ecosystem covers the
browser, native apps and the terminal, for every kind of user. Dweller's
engine is the shared runtime for all three. The user delegated the
choices below to the CTO.

## Why

Dweller already holds a full app runtime: text shaping (FreeType,
HarfBuzz), CSS cascade and layout, paint with damage tracking and a
raster cache, GPU through ANGLE, input, audio, networking, an event
loop, a JS VM and the agent protocol. The browser is only its first
front end. Electron grew a desktop platform out of Chromium and Flutter
out of Skia; we own the whole stack and write it in our own language,
so the same engine can carry native apps and the terminal at little
extra cost.

## The shape

```
                    Aye language + std
                           |
  platform: window and input (ui/sdl), GPU (gpu/), audio (audio/),
            fonts (font/), network (net/), event loop (engine/)
                           |
  render core: style (css/) -> layout (layout/) -> paint (paint/)
               -> composite; the semantic tree and the agent protocol
                           |
     +---------------------+----------------------+
  HTML + CSS + JS       Aye UI components      cell-grid output
  -> Dweller browser    -> native apps         -> terminal, TUI
                        (the same components
                         also build to wasm32
                         as framework islands)
```

- **Browser (Dweller).** The open web, for people and agents. The
  hardest surface (the compatibility tax), and the strictest test of the
  engine: an engine that runs real sites runs any app. WPT and test262
  gates protect the other surfaces too, so the engine is never forked.
- **Native apps.** An Aye program drives the render core directly: no
  HTML parser, no JS VM, no IPC. Electron's developer experience at a
  native size and start-up (target: a few MB, a window in under 100 ms).
- **Terminal.** Two products on the same stack: a GPU terminal emulator
  (the text stack and the compositor, as Ghostty or WezTerm), then a TUI
  backend where the same layout engine fills a grid of cells, so one
  set of Aye UI components also runs in a terminal.

**The through-line is agents.** The agent protocol (`docs/agent-protocol.md`:
the semantic tree, actions, Spaces) moves down into the render core, so
every interface built with Aye, not only web pages, can be read and
operated by an agent without screenshots. That is what spans every user
group:

| Who | Surface | What they get |
|---|---|---|
| people | Dweller, apps built with Aye | fast, small, an agent beside them |
| developers | terminal, `aye` CLI, TUI | one language for CLI, TUI, GUI and web |
| product teams | native app packaging | one codebase for macOS, Linux, Windows and the web |
| agents | one protocol | the same actions on every Aye interface |

## Decisions (2026-10-16)

1. **Native apps are the first surface after the browser.** They reuse
   the most code and give the Aye language its strongest reason to be
   chosen.
2. **The terminal is both:** the emulator first (a daily tool for
   developers and our own dogfood), the TUI backend after the native
   component model is stable.
3. **anyknown stays separate.** The ecosystem is where agents act;
   anyknown (the company's harness product, its own repository) drives
   it. The brands stay apart.
4. **Platforms:** macOS and Linux first. Windows after the native shell
   works, once the Aye compiler targets Windows (an `aye-lang/aye` card).
   Mobile is a horizon, not planned.
5. **Order:** the engine is layered now, while it is cheap; then the
   native shell; then the terminal emulator; then TUI. The browser lanes
   (P4, P10, P11) continue as they are.

## Where the engine is today (audit, 2026-10-16)

- The module graph is acyclic and layered in practice: nothing below
  `engine/` imports `engine`; `paint/` imports neither `engine`, `js` nor
  `ui`. The display list (`paint/paint.aye`, `Item`, `DisplayList`) and
  the rasterizer (`paint/raster.aye`, std only) are already free of SDL.
- Text measurement is an interface (`font.Measurer`, `font/font.aye`),
  so a cell-grid measurer fits as it is.
- The blocker: `css/`, `layout/` and `paint/` take the concrete
  `dom.Document` (`css/cascade.aye` `computeStyles`, `layout/layout.aye`
  `Tree.doc`, `paint/paint.aye`). There is no abstract node tree.
- HTML-specific spots in the core: replaced-element tag names
  (`layout/layout.aye`), form controls (`layout/controls.aye`,
  `paint/form_widgets.aye`), `canvas`/`svg` checks in paint, and `svg/`
  importing `html`.
- `layout/inline.aye` imports `js` only for `js.caseMap`.
- No layering rule is written down.

**Consequence.** Native apps do not wait for a refactor: a native app
builds a `dom.Document` from code and uses it as the retained scene
graph, skipping `html/` and `js/`. The abstract node tree comes when the
TUI needs a second implementation (V6), not before.

## One component model, two targets

The framework (`docs/framework.md` in aye-lang/aye) already plans
islands as "one module, two targets": an island renders HTML natively
for SSR and handles events in wasm32 through imported DOM functions. The
native app is the third target of the same module: the same DOM
functions, implemented by Dweller's `dom.Document` in-process instead of
by JS glue in a browser. So the Aye UI component API is one DOM-shaped
interface with two implementations (wasm glue, the engine), and a
component written once runs in a page, in a native window and, after V6,
in a terminal.

## Risks

- **Spreading thin.** The browser's compatibility work has no end. The
  V lane takes at most one card in flight beside the browser lanes until
  V2 has shipped.
- **API lock-in.** The component API is designed after the V2 spike
  shows what native code needs, not before.
- **Package boundary.** Native apps need the engine as an Aye library,
  not a program. V2 finds the smallest way (an import path into this
  repository); a published package waits for the Aye package story.
