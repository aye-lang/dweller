# Page-load performance

Measured 2026-10-07 on an Apple M2 Pro (10 CPUs, 32 GB), macOS 26.5.
The workload is `dweller dump --md --js URL`, fast mode, fixed-metric
layout as used by the Markdown CLI. No JS interpreter, script-budget or
existing golden changes are included.

## Timing and offline replay

`DWELLER_TIMING=1` writes tab-separated records to **stderr**:

```
info: DWELLER_TIMING<TAB>phase<TAB>nanoseconds<TAB>source
```

Stdout is unchanged. Disabled hooks allocate nothing and do not read the
clock. `browser/perf/perf.aye` is shared by engine, CSS, net, Markdown
and paint. The runner aggregates phases, lists the five slowest document
scripts, and saves individual runs, medians and output hashes as JSON.
Each run also records the browser's CPU time (user+sys) and the 1-minute
load average; the `run` and `check` tables show median `wall`, `cpu` and
`load` columns, and `--compare` reports the CPU improvement (wall in
parentheses). A site whose CPU/wall is below 0.8 gets a warning: the
machine was too busy for wall time to mean anything, so compare CPU.

```sh
cargo build --release
browser/build.sh
# Choose a fresh directory; capture refuses to overwrite a snapshot.
python3 browser/tools/page_load.py capture --archive target/page-load-new
python3 browser/tools/page_load.py run --archive target/page-load-new \
    --repeat 7 --json target/page-load-new/after.json
# Alternating AB/BA order; one excluded warmup per binary and site:
python3 browser/tools/page_load.py run --archive target/page-load-new \
    --compare target/dweller-perf-before --repeat 7 \
    --json target/page-load-new/comparison.json
```

`capture` refuses a page that misses the site's `sites.json` `md`
strings (a consent or bot page): it prints the strings, exits non-zero
and saves nothing, so a good archive is never replaced. It also writes
`reference.md` (the Markdown whose hash `run` compares, in
`manifest.json` `capture.sha256`).

When a dump change is intended (P4.25: dumps stop after two growth
rounds), `run` fails with "replay Markdown differs from capture". Then:

```sh
python3 browser/tools/page_load.py rebless --site youtube-search --archive target/sites
```

`rebless` replays each site with the binary, refuses (exit 1, archive
untouched) when the replay misses its `sites.json` strings, prints the
lines added/removed against the old `reference.md` (archives from before
P1.13 have none: it prints the line and byte counts instead), and stores the new Markdown and hash.

The URLs are in `browser/tools/page_load.py`. Without `--site` it loads
these four pages; `--site zol` (or `wikipedia`, `github`, `youtube`)
selects a subset, and `--site all` loads every site of
`docs/browser-sites.md`. The default archive is `target/sites`, one
directory per site. Only Python's
standard library is needed. Each load has a 300-second timeout.

`run --live` loads each site from the network instead (no archive; the
last output goes to `target/live/<site>`), `--repeat` times after the
warmup, and prints the same table: median wall, CPU and phases. The
Markdown is not compared, since the network serves today's page. The
CPU/wall warning fires on most live rows, because waiting for the
network is wall time without CPU; read the `load` column instead.

`--screens` (with `check` or `run`, replay or live) also takes a
screenshot of each site, 1280 px wide and its first 2000 px, into
`target/screens/<site>.png`, and writes the contact sheet
`target/screens/index.html`: a thumbnail per site that links to its
image, labelled with its status in `sites.json`, its verdict, its
console errors and what went wrong. Use it to see a site's symptom on
current `main` before a site step starts:

```sh
python3 browser/tools/page_load.py check --site all --screens
open target/screens/index.html
```

The screenshot fetches what `dump --md` does not (web fonts, images).
`capture` (since P3.40) also takes the screenshot, replaying what the
dump recorded and recording only the rest, so a new capture replays the
screenshot without misses. Archives captured before P3.40 miss those
resources; the misses are listed under the thumbnail, and the page is
painted without them.

### Archive semantics

- `DWELLER_RECORD=DIR` saves the first response for each URL/POST body:
  status, MIME, charset, redirect destination, raw decompressed body, or
  fetch failure. Original URLs, HTML, CSS and JS are **not rewritten**.
- `DWELLER_REPLAY=DIR` serves these files and disables HTTP prefetch.
  A miss fails locally, is printed, and **never falls through to HTTP**.
  With `DWELLER_RECORD` set as well (capture's screenshot), a miss is
  fetched and recorded instead. A non-GET request (a POST beacon) whose
  body has no record gets the latest-recorded non-GET response for its
  URL, whatever its body; this prints `archive fallback:` and `check`
  shows it on a `fallback:` line, not as a miss (P1.12). GET requests
  match exactly.
  The runner rejects misses, changed resource hashes, capture/replay
  Markdown differences, and before/after output differences.
- The manifest lists every requested resource, body size and SHA-256.
  Each `.response` has six newline-delimited metadata fields, then body
  bytes. Filenames are stable 64-bit FNV hashes of URL and optional POST
  content type/body; reading also verifies the original URL.
- The runner sets `DWELLER_BENCHMARK=1`: a benchmark-only engine prelude
  freezes JS Date at 2026-10-07 00:00 UTC and seeds Math.random with 1.
  Otherwise timestamp/random cache-busters cannot replay. Normal browsing
  retains the real builtins; the existing virtual timer clock is unchanged.
- This is an immutable benchmark snapshot, **not a general HTTP cache**.
  Response headers, cookie-jar updates, expiry and authenticated sessions
  are not replayed. The runner clears `DWELLER_COOKIE_FILE`; avoid
  personalized pages. Keep snapshots private (site directories are mode
  0700). If dynamic state still changes requests/output, the runner fails
  rather than silently substituting another resource (the non-GET
  fallback above is reported).

The measured snapshots remain locally in `target/page-load-fixed/`
(ignored by git), with manifests, bodies, traces, `final.json` and samples.
Third-party bodies are not committed. `perf-results.json` beside this file
preserves all seven measured wall/phase runs and their shared output hashes.

| Site | Recorded requests | Body bytes | Markdown bytes |
|---|---:|---:|---:|
| zol | 818 | 10996175 | 102606 |
| wikipedia | 29 | 857473 | 61283 |
| github | 24 | 2837200 | 8355 |
| youtube | 17 | 16673389 | 0 |

YouTube's preserved Markdown is empty. This measures the existing
browser's load performance, not successful rendering of its JS application.

### Rebuilding the baseline

The baseline is `ec6961c` (timing/archive machinery only), with later
timer/request/image timing hooks, **without** performance optimizations.
The compiler, C optimization mode and frozen responses are the same.

```sh
mkdir -p target/perf-before-src
git archive ec6961c | tar -x -C target/perf-before-src
git diff ec6961c 8f3ace3 -- browser/net/net.aye browser/net/multi.aye \
    browser/engine/images.aye browser/engine/script.aye | \
    git apply --directory target/perf-before-src
target/release/aye build target/perf-before-src/browser \
    -o target/dweller-perf-before --mode fast
```

Use a fresh source directory. Either binary replays the saved snapshot
offline, including frames, CSS/imports, dynamic scripts and images.

### Phase boundaries and paint

Phases are wall time, **not additive**: requests/prefetch overlap
fetch/scripts/sheets/images; CSS parse overlaps sheets and style-reading
scripts; lazy cascade overlaps scripts/timers; image decode is in images.
Startup scripts include prelude setup, fetching and load events. Timer
callbacks/checkpoints are separate. Individual `script` spans include
parsing, execution, microtasks and document.write (nested scripts overlap).

Cascade includes candidate-index construction, matching, declaration
competition, inline styles, inheritance, computed values and pseudos.
Layout includes tree construction/text measurement; frames report their
own phases. Document decode is encoding-to-UTF-8. `sheets=0` may mean a
reader's parsed sheet was transferred to full styling, not skipped CSS.
Startup/teardown and some frame synchronization are only in total wall.

Markdown does not paint. To benchmark the separate screenshot CLI:

```sh
python3 browser/tools/page_load.py run --archive target/page-load-fixed \
    --site wikipedia --screenshot --repeat 3
```

This uses screenshot defaults and fixed fonts, not the final scripted
Markdown document. PNG hashes must agree across repeats (and binaries
with `--compare`). Engine hooks expose display-list/raster timings whenever
that path runs.

## Offline results

Seven alternating before/after loads after one warmup per binary/site.
Capture and both binaries produce identical Markdown bytes. The optimized
binary is `b97bd1a`; raw measured runs are in `perf-results.json`.

| Site | Before (s) | After (s) | Faster | Before range (s) | After range (s) |
|---|---:|---:|---:|---|---|
| zol | 2.644 | 1.260 | 52.3% | 2.444–4.822 | 1.113–2.136 |
| wikipedia | 1.236 | 0.183 | 85.2% | 0.390–2.962 | 0.126–0.401 |
| github | 3.444 | 0.514 | 85.1% | 3.309–8.362 | 0.504–0.774 |
| youtube | 1.201 | 0.935 | 22.1% | 1.156–1.578 | 0.882–1.263 |

Concurrent agents were building/testing. Ranges are retained, especially
Wikipedia's large baseline scheduling outliers. Fresh executables also
had startup outliers, excluded by warmup. These are reproducible inputs
and measured medians, not isolated-machine performance guarantees.

| Phase, milliseconds (before → after) | ZOL | Wikipedia | GitHub | YouTube |
|---|---:|---:|---:|---:|
| Document/frame fetch | 1.01 → 0.93 | 0.27 → 0.34 | 0.30 → 0.34 | 0.86 → 0.85 |
| Document Unicode decode | 4.96 → 4.84 | 0.49 → 0.55 | 0.38 → 0.39 | 1.88 → 1.89 |
| HTML parse | 21.02 → 21.51 | 3.79 → 4.07 | 2.94 → 2.94 | 7.25 → 7.29 |
| Startup scripts + load events | 181.78 → 191.59 | 9.31 → 8.58 | 3.74 → 3.67 | 826.52 → 839.31 |
| Timer tasks + microtasks | 1235.83 → 146.44 | 0.00 → 0.00 | 0.00 → 0.00 | 0.23 → 0.19 |
| Author sheet load/assembly | 5.76 → 0.69 | 3.93 → 3.64 | 44.23 → 44.59 | 43.13 → 0.00 |
| Sheet parse (nested) | 4.30 → 3.18 | 3.13 → 2.91 | 34.23 → 35.49 | 110.63 → 73.62 |
| Lazy ancestor cascade (nested) | 168.12 → 47.31 | 0.00 → 0.00 | 0.00 → 0.00 | 2.60 → 10.15 |
| Full cascade + selector matching | 642.24 → 338.51 | 1117.90 → 89.72 | 3320.18 → 393.53 | 259.88 → 13.45 |
| Images: fetch + decode | 440.17 → 429.58 | 13.32 → 8.39 | 1.75 → 1.28 | 0.19 → 0.10 |
| Image decode (nested) | 348.83 → 351.64 | 6.29 → 6.09 | 0.40 → 0.38 | 0.00 → 0.00 |
| Layout | 22.23 → 22.00 | 13.36 → 4.66 | 7.42 → 3.85 | 0.37 → 0.24 |
| Display-list build | 0.00 → 0.00 | 0.00 → 0.00 | 0.00 → 0.00 | 0.00 → 0.00 |
| Raster paint | 0.00 → 0.00 | 0.00 → 0.00 | 0.00 → 0.00 | 0.00 → 0.00 |
| Markdown | 1.74 → 2.06 | 0.85 → 0.74 | 0.25 → 0.24 | 0.05 → 0.04 |
| All request calls (nested) | 33.20 → 32.78 | 1.38 → 1.34 | 3.01 → 3.04 | 10.12 → 8.94 |
| Concurrent prefetch batches (nested) | 0.01 → 0.01 | 0.00 → 0.00 | 0.00 → 0.00 | 0.00 → 0.01 |

ZOL is under five seconds **offline**. Wikipedia/GitHub exceed 30%
improvement. YouTube improves 22.1%, **not** 30%; about 90% of its remaining
wall is startup scripts. Interpreter work belongs to the concurrent agent.

## Profile and fixes, in measured order

`--sample --site zol --repeat 1` saves a macOS `sample` report
(`sample-after.txt`, also `sample-before.txt` with `--compare`).
Sampling perturbs execution; do not use sampled runs for the table.
The sampler is bounded/reaped; symbolication is outside child wall time.
Raw `sample PID 3 1 -file report.txt` works too.

1. **Full-DOM frame lookups.** The first ZOL sample's hottest non-idle
   leaf was `frameElements` (842 samples), ahead of `css.matchFrom`
   (533). Window/frame property reads repeatedly walked the document.
   A per-page live frame index caches document order. A follow-up still
   showed 446 frame-lookup leaf samples: image src/style writes invalidated
   the general DOM version. `Document.tree_version` now changes only
   with child/sibling links, not attributes/text data. Insert/remove/
   reorder and allocation-failure fallback preserve live-list behavior.
   ZOL's timer total drops from 1.236 s to 0.146 s.
2. **Cascade scans.** GitHub spent about 3.3 s in cascade. The new
   `css/rule_index.aye` selects required rightmost id/class/type keys;
   positive `:is()`/`:where()` alternatives are recursively indexed.
   Negative, attribute-only, universal, unsupported and depth-limited
   cases fall back. Candidates are deduplicated in source order; the
   original matcher still checks alternatives and their maximum matching
   specificity. Declaration/important logic, pseudos and quirks remain.
   Tests compare candidates with full matching and cover 2,000 unrelated
   rules, alternatives, case/whitespace and mutations.
3. **Duplicate navigation layout.** `newDocument` laid out, then
   `settleScripts` laid out again even with no due task. Skip the second
   pass only in that proven-clean case; actions/tasks still recompute.
4. **Repeated sheet parsing.** YouTube parsed the same large linked
   sheet for style readers and final layout. `sheet_cache.aye` fingerprints
   actual sources/media/order/first base href. Empty styles/preloads do
   not force reparsing; ordinary mutations retain parsed sheets, and full
   layout can take a reader sheet's ownership and compute fresh values.
   Viewport/source changes still reparse. A regression test covers media
   changes (the old reader key omitted media). YouTube sheet parsing drops
   from 111 ms to 74 ms; final author-sheet loading disappears.

The final ZOL sample no longer lists frame lookups or matching among hot
leaf stacks. Image decode/JPEG store dominate (151/110 samples, followed
by Huffman/receive); decode is still about 352 ms. System-font fallback
is not on fixed-font Markdown's hot path. No offscreen image was silently
skipped, and decoder/font output is unchanged.

**youtube-search, 2026-10-07/08** (plan P1.8–P1.11, P4.12–P4.27): CPU
per replay ~77 s → ~11.6 s.

- P1.9/P1.10: a global `IntersectionObserverEntry` (YouTube's polyfill
  no longer replaces our observer) and a virtual clock that moves inside
  a task (its scheduler yields).
- P4.12: a repeated layout change is kept once, `MAX_RESTYLE` 4096;
  full cascades per replay 100 → 16.
- P4.13, P4.15: the ancestor Bloom filter outside shadow trees; a rule
  index of its own for `::before`/`::after`.
- P4.17: `MAX_SHAPES` 16384 → 262144; cached-site hits 27% → 48%,
  shapeless reads 61% → 22%.
- P4.18, P4.19: Map/Set keys through a hash index; the document base URL
  cached per document.
- P4.25: dumps stop after 2 growth rounds (was ~10 more pages of
  results: 100K nodes, 4.69 GB RSS, ~50 s CPU).
- P4.27: elements with equal computed values share one array; cards
  page peak RSS 10K 1.40 → 0.51 GB, 40K 5.58 → 1.99 GB (139 → 49 KB per
  card).

**Mistral, 2026-10-14** (P3.63): CPU per replay 45.4 s → 32.6 s
(medians of 3 alternating runs; wall 46.3 → 33.0 s; Markdown, check
strings and screenshot PNG unchanged).

Temporary logging in `geometry()` (not committed) recorded, for each
forced layout, the changes noted since the last one and why
`restyle()` refused a subtree restyle. Forced layouts and their cost
per document (`DWELLER_TIMING` `script_layout`, one run):

| Document | Layouts | Before | After | What it notes between layouts | Path before → after |
|---|---:|---:|---:|---|---|
| main page | 872 | 17.1 s | 4.9 s | 655× text of one `p.text-button-small` (a ticker), 194× `style` of `span.word`, 206× `style` of an `iframe`, 142+48× `style` of title `p`, 28× other attributes, ~40× `:defined` | full cascade (697) → subtree restyle; the rest is layout |
| visual-intelligence iframe | 6931 | 7.3 s | 8.5 s | 5922× `style` of `div.frame`, 1230× `style` of `aside.sc-prompt-panel` | 5954 layout only; ~970 full cascades: container queries (`queried`) and 14 shadow roots |
| ctf-speedrun iframe | 232 | 1.0 s | 0.8 s | `style` of bar `i`/row `div` | full cascade (200) → subtree restyle |

The main page's sheet set `RestyleDeps.whole`, so every change but an
inline style restyled the whole document: Tailwind's `group-has-*`
variant compiles to `:where(.group\/toggle):has(:checked) *`, whose
`:has()` compound has no type, id, class or attribute of its own, and
`addHas` took "no keys, any depth" for "any element may be its
element". `RestyleDeps.walk` now takes the keys of a one-compound
`:is()`/`:where()` argument as keys of the compound (`.group/toggle`
here), which every element that matches it has. Test:
`browser/tests/jsdom/layout_restyle_where_has.html`.

What is left (~33 s CPU): timers 31 s, of which forced layouts ~14 s
(main page ~4.2 s of pure layout, 872 × 4.9 ms over the whole document;
visual-intelligence ~5.9 s of layout and 2.6 s of cascade) and the rest
JS and canvas raster in the iframes (a 10 s sample during the timers:
`canvasComposite`, `canvasPaint`, `svg.Rasterizer.scan`,
`paint.sampleImage` lead the non-idle leaves).

**Mistral, 2026-10-14** (P3.64, incremental layout for forced reads):
CPU per replay 32.5 s → 26.2 s, wall 33.0 → 26.4 s (medians of 3
alternating runs; zol 28.1 → 28.2 s and github 5.48 → 5.43 s CPU, noise).
Markdown of every site equals its reference; the screenshots of mistral,
bbc, github and wikipedia-en are byte-identical.

`geometry()` (`browser/engine/geometry.aye`) now keeps the layout tree
when every change since the last layout was noted (`Script.layout_noted`
counts them; an image's new source or bitmap is not) and is a children,
`style` or `class` change:

- The subtree restyle reports the elements whose values that layout or
  script geometry read changed (`Styles.moved`, `moved_els`;
  `css.paintOnly` lists the rest: colors, backgrounds, borders' styles
  and colors, `visibility`, `opacity`, `z-index`, shadows, radii, masks,
  SVG paint; not `transform`, which moves boxes here). None: the boxes
  stay as they are. The restyle then keeps the old value arrays
  (`keep`), which the boxes still point at.
- Else the dirty elements (those, and the parents of changed children)
  are built again from the nearest element whose block box is in block
  flow up to the root (`layout/incremental.aye` `canRebuild`; a flex
  item's text: the block around the flex container), laid out in the
  containing block of the last layout (`Box.cb_w`, `cb_h`), and up
  through the ancestors while size, margins or baselines change; then
  its absolutely positioned, transformed, fixed and sticky boxes as a
  full layout does them (`finish`).
- Whole document still: floats anywhere, container queries, shadow
  trees, a dirty element with no such block (or the walk reaching the
  root), a fixed box under a sticky one, more than 64 moved elements,
  and when dropped boxes (`rebuild` leaves them in the arena) pass twice
  the boxes of the last full layout.

`DWELLER_LAYOUT_CHECK=1` lays out the whole document after each kept or
partial layout and logs the first box that differs ("LAYOUT CHECK"); the
DOM fixtures run with it (`tests/browser.rs`), and so did a Mistral
replay (0 differences). Fixture:
`browser/tests/jsdom/layout_incremental.html` (paint-only writes, text in
a block, a size that propagates, flex, sticky, transforms, `attr()`).

| Document | Forced layouts | script_layout before | after | Path now |
|---|---:|---:|---:|---|
| main page | 872 | 4.8 s | 1.2 s | 64 boxes kept, 767 one subtree (mostly the ticker in the sticky footer), 41 whole |
| visual-intelligence iframe | 6931 | 7.1 s | 5.2 s | 5951 boxes kept; 979 full cascades (container queries) |

### Next card proposal (P3.64 slice 2)

What is left of forced reads is style work, not layout. On the
visual-intelligence iframe each kept read still hashes the whole
document (`stylesCurrent` → `css.styleInputs`) 5951 times, and 979
reads restyle the whole document because it uses container queries
(`restyle` refuses `g.styles.queried`). Proposal: restyle subtrees in a
`queried` document when no query container's size changed
(`Styles.sizes`), and skip `stylesCurrent` when the noted changes are
attribute writes that the subtree restyle can take. Measure per-read
costs first (`DWELLER_TIMING` `script_layout`, `cascade`). The canvas
raster (~10 s, `canvasComposite`) stays a separate card.

## Live network check and remaining work

One separate normal live load per site (real Date/random, no archive)
gave these seconds. These are **not paired identical responses**, nor the
offline speedup table:

| Site | Original reported wall (s) | Current live wall (s) | Fetch | Scripts | Timers | Images | Full cascade |
|---|---:|---:|---:|---:|---:|---:|---:|
| zol | 16–20 | 20.399 | 3.863 | 8.647 | 2.561 | 4.855 | 0.319 |
| wikipedia | 5.3 | 2.502 | 0.716 | 0.140 | 0.000 | 1.344 | 0.069 |
| github | 5.2 | 2.694 | 0.925 | 0.004 | 0.000 | 0.459 | 0.602 |
| youtube | 5.0 | 3.879 | 0.890 | 2.896 | 0.000 | 0.000 | 0.015 |

The requested live ZOL target is **not met**: 20.399 s, with 9.228 s in
prefetch and 9.750 s in request calls (overlapping other phases). Only
0.357 s is image decoding. CPU improvements do not remove these
serial/dynamic waits. Wikipedia/GitHub are below the reported starting
times; YouTube's single live check also falls short of a 30% reduction.
Do not mistake offline sub-five-second ZOL for an internet-load guarantee.

Next, in priority order:

- Merge/rebenchmark the JS agent's interpreter work, retaining the same
  snapshots, budgets and output checks; YouTube startup dominates.
- Profile ZOL's live dynamic script/telemetry chain. Separate discovery
  and transfer waits before changing scheduling/concurrency; do not gain
  speed by dropping scripts/resources.
- Separate intrinsic image dimensions from raster decode, then defer
  offscreen decoding until paint. Preserve EXIF orientation, SVG sizes,
  density selection and invalid-image behavior; first add viewport/full-
  page PNG and frame-layout equivalence tests.
- Keep rule indices across computed-value invalidation if measurements
  justify it. A lazy ancestor query on huge sheets can spend more building
  an index than a one-off scan (YouTube's lazy-cascade time rose).
- Dirty-subtree restyle/layout requires ancestor/sibling/inheritance/custom
  property/pseudo dependencies, plus the concurrently changing Shadow DOM,
  flat tree and CSS features. Existing mutations are already batched until
  readers/tasks; subtree-only invalidation is not assumed safe here.
- Profile fallback/shaping caches on system-font screenshots, a different
  workload from fixed-font Markdown.
- Add response-header/cookie journals before authenticated benchmarks.

## Window: where the time goes for a person (2026-10-15)

The user's report: "still too laggy; some sites are extremely slow, while
Chrome shows them with no problem at all". Measured in the window, not
`dump`: live network (Taiwan, about 220 ms to US hosts), fast build,
1024×768 at scale 2, same M2 Pro, at 194c357 (before P4.31's wheel
change). Cards P4.32–P4.39 in `docs/plans/browser.md`.

**Method.** With `DWELLER_TIMING=1` the window loop writes `ui_navigate`
(a navigation, page load included), `ui_relayout` (`Ui.relayout`:
`layoutPage` and the display list), `ui_draw` (raster and texture
upload), `ui_events` (input handling), `ui_timers` (due timers and the
layout after them) and `ui_loop` (one iteration without its wait;
labelled `space` in a Space, which shares the window's stderr). With
`--spaces` the window also writes `ui_space_sent` and `ui_space_answer`
(labelled with the Space's name) when it relays an agent's bytes to its
Space or the Space's answer back. Each record is one write. The
`--events` word `frame` makes the rest of the script wait for the next
frame, so `wheel:-3 frame` repeated scrolls one step per frame.
`loop_max_ms` in `--state` only counts while agents are connected, so the
tool reads `ui_loop` instead:

```sh
browser/build.sh target/dweller-perf
python3 browser/tools/window_perf.py scroll URL --binary target/dweller-perf  # load + 40 wheel frames
python3 browser/tools/window_perf.py idle URL --binary target/dweller-perf    # CPU% of 10 idle s
```

Chrome: `agent-browser` (bare), first-contentful-paint, load event and
buffered `longtask` entries. Scroll frames are headless `--frames` runs
(each iteration draws). Idle CPU is a real window, 5 s after its first
frame, for 10 s.

| Site | First paint Dweller / Chrome FCP (load) s | Scroll frame p50 / p95 ms | Idle CPU | Longest loop ms | Main cost (profile, records) |
|---|---|---|---:|---:|---|
| github.com/aye-lang/aye | 5.4 / 1.45 (2.0) | 18 / 419 | 21.5% | 2898 | 129 synchronous requests (3.0 s) before paint; each DOM change after load = 2 full cascades (~330 ms each, container queries) + layout 370 ms; typing 1157 ms p50 per key while timers run |
| en.wikipedia.org | 1.8 / 0.89 (1.6) | 9 / 10 | 0.3% | 38 | serial network: sheets fetched after the scripts, images (0.43 s) before the first paint |
| developer.mozilla.org | 5.1 / 0.48 (0.57) | 11 / 13 | 0.1% | 53 | `index.js` takes 3.6 s: 48 webpack chunks fetched one by one, synchronously, inside it |
| www.bbc.com/news | 4.1 / 2.0 (5.8) | 23 / 31 | 1.1% | 455 | prefetch 2.5 s + 91 synchronous requests before paint; draw 20 ms |
| www.youtube.com | 15.8 / 0.63 (2.5) | 16 / 750 | 6.6% | 5440 | the first layout waits 6.1 s for images; timers make synchronous `videoplayback`/XHR requests (6 × 0.6 s in one iteration) |
| react.dev | 1.7 / 0.56 (0.8) | 201 / 534 | 0.9% | 749 | draw (92% of main-thread samples): gradient tiles 41%, SVG 23%, opacity/filter layers 23%, blurred shadows 19%, every frame |
| mistral.ai | 2.6 / 0.75 (2.5) | 61 / 70 | **100.9%** | 230 | an animation changes the DOM every tick: `Ui.relayout` → `restyleWithSizes` (full cascade) is 71% of samples, forever |
| news.ycombinator.com | 1.7–2.4 / 1.2 cold | 7 / 9 | 0.1% | 16 | serial network: `news.css` fetched after `hn.js`, on a new TLS connection (753 ms; 260 ms reused) |
| www.apple.com | 1.7 / 0.65 (1.3) | 27 / 182 | 7.5% | 1139 | relayouts (0.65–0.95 s) with image prefetch inside; draw 28 ms |
| medium.com | 0.5 / 0.29 (0.41) | 26 / 34 | 0.0% | 73 | draw 25 ms |

stackoverflow.com/questions (Cloudflare challenge) and www.reddit.com (block
page) served Dweller no content; not comparable. The longest loop does not
include the first load: the first navigation runs before `Ui.loop`, and
the window shows nothing and takes no input until it ends (the "first
paint" column). Network wait before the first paint (synchronous requests
plus prefetch batches): YouTube 14.1 of 15.8 s, MDN 4.8 of 5.1, GitHub
4.1 of 5.4, BBC 3.6 of 4.1, HN 1.6 of 1.7, Wikipedia 1.7 of 1.8.

### Root causes, by what a person feels

1. **The window thread waits for the network.** `Tab.navigate` loads the
   document, fetches and runs every script, then `relayout` fetches the
   sheets and every image, and only then the first frame is drawn; a
   batch is per kind (scripts, then sheets, then images), each waiting for
   the previous one. Page `fetch()`/XHR and dynamic script chunks are
   synchronous (`akFetch`; prelude XHR "synchronous underneath"), so each
   costs a round trip (0.1–0.8 s) of a frozen window, also after load
   (GitHub partials, YouTube's player). Chrome paints before scripts and
   images and never blocks input on the network.
2. **Every DOM change restyles and lays out the whole document.** The
   window's `relayout` calls `layoutPage`, which runs `restyleWithSizes`
   (full cascade; twice with container queries) and a full layout, then
   rebuilds the display list. The P3.63/P3.64 subtree restyle and
   incremental layout serve only the scripts' forced reads (a separate
   `Geometry`). GitHub: 0.4–0.9 s per change; Mistral: every animation
   tick, so the idle page takes a full core.
3. **Paint keeps nothing between frames.** Each frame rasterizes the whole
   visible display list again at device resolution: CSS gradients pixel
   by pixel (`fillGradient`), SVGs, blurred shadows and layers, then
   uploads the full texture. react.dev scrolls at ~5 fps; simple pages
   take 7–25 ms per frame at 1024×768 (more in a larger window).
4. **Busy pages stay busy.** Timers keep the idle CPU at 21.5% (GitHub),
   7.5% (Apple), 6.6% (YouTube); this is causes 1 and 2 repeating.
5. **Connections are not reused across batches** (HN: the sheet batch
   opened a new TLS connection, 753 ms instead of 260 ms).

Interaction itself is cheap where nothing else runs: a wheel step costs
its frame's draw (`ui_events` ≈ 0 ms; scroll listeners run as tasks),
typing in the address bar 7 ms, a key in GitHub's "Go to file" field
~30–50 ms (a full `relayout` per key, styles current). The lag is the
work above sharing the one thread.

The user's log lines are from mistral.ai: `SVGAElement is not defined`
from Astro's `ClientRouter` click handler (`t instanceof SVGAElement`,
on every click that is not on an `<a>`), and `e.getTotalLength is not a
function` from its `layout` script's SVG path polyfill.

### P4.32: page fetch() and XHR in the background (2026-10-15)

Page `fetch()` and asynchronous `XMLHttpRequest` start on the loader's
libcurl multi handle (`net.startRequest`) and settle in a networking task
that the window's loop runs once the request ended (it polls the multi
handle with a zero timeout, at most 8 ms apart while a task waits); on the
virtual clock that task is due at once and waits for its request, so
`dump` stays deterministic. Their records are `background_requests`
(start to end, nobody waits); `requests` are the synchronous ones left.
Same machine and network as above, `window_perf.py` with the fast build,
before (7017742) / after:

| Site | scroll: longest loop ms | scroll: first paint ms | idle: longest loop ms, CPU |
|---|---|---|---|
| github.com/aye-lang/aye before | 2467, 1763 | 4758, 3109 | 735, 6.6% |
| github.com/aye-lang/aye after | 1912, 2061, 1977 | 3674, 2942, 4176 | 421, 8.0% |
| www.youtube.com before | 6749, 3684, 3910, 3061 | 6258, 8788, 4543, 3836 | 3575, 10.7% |
| www.youtube.com after | 654, 631, 616, 771, 588 | 6987, 4712, 21873\*, 4223, 6646 | 119, 5.6% |

\* that run's network: the document's script prefetch took 5.9 s (0.8 s
in the next run) and the first relayout's image prefetch 9.8 s, before any
page request ran in the background.

No loop iteration waits for a page fetch or XHR any more. What the longest
iterations still do: GitHub's first one after the load (1.9–2.1 s) runs
two relayouts (1.8 s, cascades with container queries: cause 2, P4.34) and
**92 synchronous dynamic script loads** (webpack `chunk-lazy-*.js`, 0.8 s
one by one: P4.43); its later ones (0.9–1.4 s) are cascade, script layout
and relayout without network. YouTube's longest (0.6–0.8 s) is a relayout
fetching sheets and prefetching images (0.37 s) plus timer callbacks
(0.23 s). The card's < 100 ms needs P4.34 and P4.43 (GitHub) and P4.33
(YouTube's relayout prefetch).

### P4.34: the window's relayout is incremental (2026-10-15)

After a full styling and layout, the page's tree and styles become the
layout scripts read (geometry.aye `adoptLayout`); the window's next
`layoutPage` brings that one state up to date through `geometry` (subtree
restyle, kept boxes or `relayoutDirty`), as a forced read does, instead
of `restyleWithSizes` and a full layout. Container-query documents and
shadow trees stay on the full path (P4.44). mistral.ai live,
`window_perf.py idle` (fast build), per loop iteration of the 10 idle
seconds, before (c99b872) / after:

| | loops/s | loop ms | `ui_relayout` ms | full cascades | draw ms | idle CPU |
|---|---:|---:|---:|---:|---:|---:|
| before | 22.6 | 44.6 | 31.1 | 1.01 | 9.6 | 100.9% |
| after | 53.3 | 18.8 | 6.8 | 0.01 | 9.7 | 102.0% |

The page's animation is `requestAnimationFrame` (a 16 ms timer): with
each frame 2.4 times cheaper it runs at 53 frames/s instead of 22, so the
CPU stays at one core. Per frame now: draw 9.7 ms (P4.35, P4.36), timers
2.6, the scripts' forced read 2.5 (restyle 1.0), `paint_build` 2.2, image
walk 0.8. GitHub's `ui_relayout` is unchanged (342–1054 ms: container
queries, P4.44).

### P4.33: progressive first paint (2026-10-15)

The window's navigation is incremental (`Tab.navigateEarly`, `Ui.go`):
the document is fetched and parsed, one batch on the loader's multi handle
starts every sheet, script and image it names (`engine.startEarly`), the
loop draws the page once its sheets are in and runs its scripts once they
are in too. Images and web fonts never block a layout of the window's
pages; they fill in as they arrive. `ui_first_paint` is the time from the
navigation to the first frame that shows the page (`window_perf.py` reads
it; before, `ui_navigate + ui_relayout + ui_draw`). Same machine and
network, fast build, `window_perf.py scroll`, before (9bca5cc) / after,
three runs each:

| Site | first paint before ms | after ms | median before → after |
|---|---|---|---|
| news.ycombinator.com | 1525, 1237, 1226 | 607, 622, 636 | 1237 → 622 |
| en.wikipedia.org | 956, 952, 1028 | 652, 656, 643 | 956 → 652 |
| www.youtube.com | 6285, 4155, 6598 | 2084, 1981, 2178 | 6285 → 2084 |

YouTube after: the document in 0.2–0.25 s, its five sheets 1.7 s later
(they share the connection with 2+ MB of scripts and the images; libcurl
gives no priorities), layout 0.08 s. Its longest loop is now the one that
runs the scripts (1.8–2.0 s: 1.4 s of JavaScript, 0.19 s for the sign-in
iframe, fetched synchronously by `loadFrames`), which used to run before
the loop started; the window shows the page and takes input until then.
`dump` and `screenshot` keep the synchronous load: Markdown and screenshots
of the 22 archived sites are byte-identical with the previous binary.

### P4.43: import() and frames in the background, scripts as tasks (2026-10-15)

On the window's clock `import()` loads its module and, a level at a
time, the modules of its graph on the loader's multi handle (lib.aye
`loadInBackground`, `graphInBackground`; window_api.aye `moduleLoad`);
the module evaluates in a networking task once they are in, from the
session's cache. The page's scripts run as tasks of at most 50 ms an
iteration (one at least; `stepPageScripts`, `Ui.stepScripts`): the
window takes input and draws, with the layout from before them, in
between; its timers wait until they ran. The documents of its frames
load in the background (`startFrameLoads`) before the load event.
A classic `<script>` that a script inserts still does not run (P4.45).
`dump` keeps its synchronous loads: Markdown and screenshots of the 22
archived sites are byte-identical with main's binary. `window_perf.py
scroll --frames 600..1200`, fast build, longest loop iteration after
the first paint, before (e030237) / after:

| Site | before ms | what it did | after ms | what it does |
|---|---|---|---|---|
| github.com/aye-lang/aye | 3107 | the page's scripts (474), 2 relayouts (1589), a timer with 33 synchronous `import()`s (887 ms of requests) | 1276, 1291 | 2 relayouts (1080, container queries: P4.44) and timers (660, forced layouts: P4.42); no synchronous request |
| www.youtube.com | 1715 | all the scripts (1426), the sign-in frame fetched synchronously (152) | 1279 | one script task (`kevlar_base`, 1221) and 57 ms of others |
| www.youtube.com, apart from that task | 426 | timers | 617, 382 | timers and a relayout fetching the sheets scripts inserted (`net.prefetch`, 224 ms) |

Synchronous requests of more than 5 ms in loop iterations after the
navigation: GitHub 13 (887 ms) → 0, YouTube 2 (175 ms) → 1 (a frame of
the sign-in frame, 23 ms). The < 100 ms target is left to P4.44
(GitHub's relayouts), P4.42 (a time budget for timers) and the
scripts' later sheets (synchronous `net.prefetch`).

### P4.35: damage tracking and the raster cache (2026-10-15)

The window keeps the page's pixels between frames and draws only what
changed: each display item's signature (a hash of what it draws with and
its context, and its area) is compared with the last frame's, and a
scroll by whole pixels moves the pixels and draws the band that came into
view. Costly results (layers, gradient tiles, shadows) are kept in a
raster cache. Same machine, fast builds, main (c50a841) / this branch,
`window_perf.py`:

| | loop p50 / p95 ms | draw p50 / p95 ms | idle CPU | damage per frame |
|---|---:|---:|---:|---:|
| react.dev scroll, main | 190.4 / 521.8 | 167.5 / 489.1 | | the whole page |
| react.dev scroll, P4.35 | 43.5 / 249.1 | 25.7 / 111.7 | | band and sticky header, 1.15 of 2.98 Mpx |
| mistral.ai idle, main | 20.8 | 10.1 | 101.7% | the whole page |
| mistral.ai idle, P4.35 | 14.2 | 6.0 | 98.6% | 0.91 Mpx (a carousel moved 1 px) |

react.dev per scroll frame now: paint ~15–20 ms (the band's conic
gradient ~12 ms, each row computed once and kept), `paint_build` 3–7 ms
(the list again for the sticky header), upload ~2 ms; in half the frames
its scroll listeners run (timers 5 ms, relayout 15 ms). Misses (a layer
or a shadow seen for the first time) make the p95. mistral.ai: the
animation changes visible pixels every frame, so it keeps a core busy at
59 frames/s: relayout 6.8 ms, scripts and the 6 ms draw per frame. What
would help there is compositing a transform animation without layout or
paint; rAF in hidden windows and background tabs is P4.55.

### P4.44: incremental restyle and layout with container queries (2026-10-15)

Documents whose styles use container queries now take the P4.34 path:
the page's layout is adopted after `layoutQueried`, a change restyles
the noted subtrees and lays out again incrementally, and then
`settleContainers` (geometry.aye) compares the query containers' sizes
in the tree with those the styles were computed for
(containers.aye `changedContainers`). The subtrees of exactly the
containers that moved (or are new) are styled again with the new sizes,
the elements whose values moved are laid out again (`relayoutDirty`),
and that repeats while a container moves, at most 2 rounds, as
`layoutQueried`; beyond, the full path. An element whose
`container-type` or `container-name` changes has all its descendants
styled again (cascade.aye `sameContainer`). A container flip often
changes the document's height, so two limits of P3.64 went: the
relayout walk lays out the root again instead of giving up, and a block
it lays out again keeps the layout of its children that did not change
when their containing block is the same (`Box.laid`, `Tree.keep_under`);
a side column's flip went from a full layout (~40 ms here) to 3.5 ms.

Fixture `browser/tests/ui/container_feed.html` on the test server
(`server.py`): ~4000 boxes, 245 `@container` rules on nested
containers, 60 frames that each insert and remove a list item (itself a
container); every 15th flips the side column's width, every 7th makes
a box a container or not, every 11th resizes a `size` container, every
10th sets the title. 1024×768, `--frames 90`, fast build, before
(d514728) / after:

| | `ui_relayout` p50 / p90 ms | full cascades after load | full layouts after load | `ui_loop` total ms |
|---|---:|---:|---:|---:|
| before | 64.9 / 82.4 | 122 | 61 | 4581 |
| after | 5.8 / 10.2 | 0 | 0 | 919 |

After: 60 container rounds (`container_restyle`, 0.1–0.6 ms each), and
3 layouts with the styles kept when the boxes rebuilt passed twice the
tree (P3.64's bound). `DWELLER_LAYOUT_CHECK=1` checks each of the 60
layouts against a full styling (with the same container sizes) and a
full layout: no difference. Live github.com/aye-lang/aye is unchanged
(`window_perf.py scroll`: relayouts after load p50 460–570 ms, 15
cascades): its document has one shadow root, and shadow trees still
take the full path (`adoptLayout`); that is GitHub's next step.

### P4.56: transform and opacity changes composited (2026-10-15)

A change of `transform` or `opacity` alone keeps the boxes (a
transformed subtree placed again from where the last pass of
transforms found it, `layout.retransform`), and the window moves or
blends the kept layer of a promoted box (`paint.patchLayer`) instead of
building the display list and rasterizing its items again
(`ui_composite`, `ui_layers`). Test page `browser/tests/ui/carousel.html`
(a 600x300 track 1 px a frame, a fading box), `--frames 200`, fast build:

| | relayouts after the first frame | composited frames | layers rasterized per frame | layers blended from the cache per frame |
|---|---:|---:|---:|---:|
| main | 60 (one per frame) | 0 | | |
| P4.56 | 0 | 58 | 0 | 2 |

mistral.ai, `window_perf.py idle`, main / P4.56: idle CPU 97.9 / 91.4%,
relayouts per loop 0.97 / 0.79, draw 6.9 / 4.4 ms per loop. Its animation
is not a transform: its scripts change the `width` of `.js-square` divs
each frame (layout), and most frames lay the whole document out because
a `layout_version` bump is not noted (P4.57).

### P4.45: scripts that a script inserts run (2026-10-15)

Before, a `<script>` that a script connected never ran (no fetch at
all): webpack/Vite JSONP chunks, consent and analytics SDKs, jQuery's
JSONP and `globalEval`. Now an inline one runs at once and an external
one loads on the loader's multi handle and runs in a task (`async` as it
arrives, `async = false` in insertion order), with `load`/`error` at the
element (`browser/engine/inserted.aye`). In `dump` the task waits for
its load in its turn, so the output does not depend on the network: two
replays give the same bytes.

The archives of `target/sites/` lacked the newly fetched scripts:
each was completed by replaying with `DWELLER_RECORD` set to the same
directory (`dump --md --js`, `screenshot` and an agent navigation; misses
fetched live on 2026-10-15 and recorded, nothing else re-recorded),
then replayed without it: no archive miss. Review of every site whose
Markdown, 1280x2000 screenshot or console changed, main (320be32)
against this branch, both on the completed archives, compared with
Chrome (agent-browser on the live site):

| Site | What changed | Chrome | Verdict |
|---|---|---|---|
| bbc | Register / Sign In at the top right, the ad slot ("ADVERTISEMENT"), the story images instead of grey placeholders (Markdown: 4 placeholder images gone) | the same | closer |
| wikipedia-en | Appearance panel (Text, Width, Color), table-of-contents toggles, the video's 0:24 badge, "Jump up to:" back-links (6), the navboxes collapsed | the same (innerText: 6 "Jump up to", "Duration: 24 seconds", navbox groups `mw-collapsed`) | closer |
| wikipedia (ja) | the same as wikipedia-en ("以下の位置に戻る" 2, navboxes and authority control collapsed) | the same; screenshots side by side match | closer |
| mistral | the logo's background a shade lighter (115x20 px) | - | neutral |
| zol | Markdown and screenshot unchanged; console 0 → 4: pc-common-module.js line 452 reads `_zpv_cfg.custom.ns` (undefined), and three JSONP endpoints answer `text/html` from other origins | Chrome throws the same (the expression throws there; a muted "Script error."), and fires `error` for both `text/html` JSONP scripts tried (opaque response blocking) | neutral |
| reddit (excluded) | the login wall's background pattern drawn smaller; a `blob:` script fails | (live: a different page) | neutral; `blob:` URLs filed as P4.47 |
| stackoverflow (excluded) | console: `pO is not a function` on its challenge page | - | neutral |
| the other 15 | byte-identical Markdown, screenshot and console | | |

None got worse. W3 7 → 8 of 20: wikipedia's only defect (P3.66, the
empty Appearance panel) is gone; BBC's P3.41 (no Register/Sign In) is
gone, its P3.59 stays; wikipedia-en keeps P3.56.

What the newly running scripts hit, fixed here (each a crash, hang or
`Uncaught` on a recorded site): a `/` after the `}` of a function
declaration or a block was taken for division (zol's ad SDK:
`function f(){}/re/.test(ua)`); `static async[key]()` was a syntax
error; `Array.prototype.every.call({length: -1})` walked 2^32 indices
(ToUint32 instead of LengthOfArrayLike: BBC's dump hung);
`[].push.call(arrayLike)` copied the object each call (ZOL: O(n²), past
600 s); `then` and the Promise statics ignored species, so core-js took
the native Promise for broken and left `Promise.prototype.finally`
undefined (BBC's prebid: `race(...).finally is not a function`);
`script.innerHTML = code` parsed the code as HTML (`i<n` lost: BBC's
`__tcfapi` stub); `document.elementFromPoint` was missing (BBC). And,
as Chrome: a script of another origin served as `text/html` does not
run (zol).

Cost: `dump` CPU, main → this branch (`page_load.py check`): bbc 0.36 →
6.7 s, wikipedia-en 0.12 → 0.62 s, wikipedia 0.11 → 0.50 s, youtube 3.0
→ 4.5 s, mistral 26 → 45 s (timers 24.7 → 46.8 s, of which forced
layouts 6.2 → 26.4 s and cascade 3.0 → 17.9 s, after gtag and the
axept.io consent SDK run), reddit 0.05 → 16 s; filed as P4.48.
Window (`inserted_scripts` test): twenty `async = false` scripts of
200–390 ms in order, longest loop iteration 3 ms (fast build).

### P4.57: width animations lay out the changed boxes (2026-10-15)

Each relayout of a page with images counted every image as new
(`loadElementImage` dropped the mapping before putting it again), and
that unnoted `layout_version` bump laid the whole document out again.
Images now count a change only for a new bitmap, and note the elements
whose image changed; flex and grid containers in block flow are built
and laid out again on their own (`layout/incremental.aye`). New
DWELLER_TIMING records: `layout_full` (a script layout of the whole
document) and `layout_dirty` (of the changed boxes). Test page
`browser/tests/ui/squares.html` (6 `.js-square` widths in a grid of flex
cells, images, 500 paragraphs, 60 rAF frames), `--frames 100`, fast
build:

| | full layouts after the first frame | dirty relayouts | `ui_relayout` p50 |
|---|---:|---:|---:|
| main | 54 | 54 | 4.7 ms |
| P4.57 | 0 | 42 | 1.4 ms |

mistral.ai, `window_perf.py idle` (Xvfb, cloud box), main / P4.57: full
layouts per loop 1.01 / 0.02, loop 31.2 / 16.2 ms, `ui_relayout` 15.3 /
5.5 ms per loop, 417 / 700 iterations in 10 s, idle CPU ~110 / 100% (the
loop now keeps up with the frame rate). Left per loop: `restyle` 4.6 ms,
timers 3.3 ms, `ui_draw` 3.3 ms, `paint_build` 2.4 ms.

### P4.42: input, frame, then tasks in a 50 ms budget (2026-10-16)

`Ui.loop` handles input, draws a due frame, then runs timers and
networking tasks until 50 ms are spent (`engine.runDueTimersUntil`); the
rest waits for the next iteration. Before, one iteration ran every task
due when it began, however long that took.

Page `browser/tests/net/timer_storm.html` (test `window_input_priority`):
for 1.5 s, 200 `setTimeout(0)`, 20 chains of `setTimeout(0)`, 5
`setInterval(1)` and 20 responses of `/sleep` (200–770 ms), each task
busy 3 ms; text typed after 8 frames. 300 frames, 400×300, dummy driver,
M2 Pro, `DWELLER_TIMING=1`, main (fc66613) / P4.42:

| | main | P4.42 |
|---|---:|---:|
| longest iteration (`ui_loop`) | 700 ms | 54 ms |
| longest task phase (`timers` summed per iteration) | 694 ms | 51 ms |
| longest single task | 4.1 ms | 4.4 ms |
| iteration p50 / p95 | 1.0 / 18 ms | 1.0 / 51 ms |

The p95 rises because the storm's tasks now spread over many iterations
of about one budget each instead of a few long ones; the total work is
the same. Input waits at most one budget plus a frame.

### P4.41: the W7 and W8 report (2026-10-17)

`window_perf.py report --replay` runs `scroll` (40 frames of 3 wheel
notches) and `idle` (10 s from 5 s after the first frame) for every
non-excluded site of `sites.json`, replayed from `target/sites/<name>`
(`--archives DIR`) in a headless window (dummy driver), one site after the
other. A row fails when the longest loop iteration is 100 ms or more, the
scroll frame p95 is 33 ms or more, or the idle CPU is 5% or more (W7).
`report --live --chrome` runs `scroll` live (first paint) and
`agent-browser`'s paint entry of Chrome (`first-contentful-paint`;
`first-paint` when the page has none) alternately 3 times; a row fails
when the medians differ by more than 1000 ms (W8). Exit 1 when a row
fails. Fast build of main (8069cf4), M2 Pro, 1024×768.

W7 (replay), 2026-10-17:

| Site | Longest loop ms | Scroll frame p50 / p95 ms | Idle CPU | W7 |
|---|---:|---:|---:|---|
| example | 5 | 0.0 / 0.0 | 0.6% | ok |
| wikipedia-en | 255 | 6.2 / 16.6 | 0.7% | FAIL loop |
| hn | 29 | 0.0 / 1.8 | 0.5% | ok |
| github | 2395 | 2.4 / 1646.7 | 46.2% | FAIL loop, scroll, idle |
| mdn | 150 | 3.2 / 7.8 | 1.0% | FAIL loop |
| python-docs | 14 | 4.0 / 7.6 | 1.2% | ok |
| rustdoc | 253 | 7.1 / 18.1 | 0.9% | FAIL loop |
| bbc | 426 | 2.7 / 210.8 | 0.8% | FAIL loop, scroll |
| cnn-lite | 4 | 1.3 / 1.6 | 0.9% | ok |
| lobsters | 41 | 0.0 / 1.9 | 0.9% | ok |
| go-docs | 99 | 3.3 / 10.0 | 0.9% | ok |
| duckduckgo | 10 | 0.0 / 1.8 | 0.9% | ok |
| google | 12 | 0.0 / 0.0 | 0.9% | ok |
| wikipedia (ja) | 105 | 2.2 / 8.2 | 0.9% | FAIL loop |
| abehiroshi | 7 | 0.0 / 0.0 | 0.8% | ok |
| zol | 6918 | 409.7 / 6799.0 | 18.2% | FAIL loop, scroll, idle |
| youtube | 346 | 0.1 / 259.4 | 5.3% | FAIL loop, scroll, idle |
| youtube-search | 802 | 177.4 / 556.8 | 41.8% | FAIL loop, scroll, idle |
| react | 102 | 0.0 / 19.2 | 0.9% | FAIL loop |
| mistral | 2098 | 86.2 / 243.2 | 97.7% | FAIL loop, scroll, idle |

9 of 20 pass. The other 11 fail on the loop (all 11), the scroll p95 (6:
github, bbc, zol, youtube, youtube-search, mistral) or the idle CPU (5:
github, zol, youtube, youtube-search, mistral). Not profiled here; the
2026-10-15 table of this section named the cause of github (cascades and
layout after each DOM change), mistral (an animation restyles the whole
document every tick), bbc and youtube (long relayouts, synchronous
requests). wikipedia-en, mdn, rustdoc, wikipedia (ja) and react miss only
the loop limit (102–255 ms); their scroll p95 is under 33 ms.

W8 (live, Dweller first paint against Chrome's first contentful paint,
median of 3 alternating runs), 2026-10-17:

| Site | Dweller ms | Chrome ms | Gap ms | W8 |
|---|---:|---:|---:|---|
| example | 200 | 356 | -156 | ok |
| wikipedia-en | 724 | 720 | 4 | ok |
| hn | 712 | 1104 | -392 | ok |
| github | 462 | 1020 | -558 | ok |
| mdn | 510 | 648 | -138 | ok |
| python-docs | 621 | 952 | -331 | ok |
| rustdoc | 1162 | 1120 | 42 | ok |
| bbc | 465 | 416 | 49 | ok |
| cnn-lite | 402 | 436 | -34 | ok |
| lobsters | 923 | 1636 | -713 | ok |
| go-docs | 536 | 832 | -296 | ok |
| duckduckgo | 1169 | 924 | 245 | ok |
| google | 408 | 476 | -68 | ok |
| wikipedia (ja) | 1745 | 1592 | 153 | ok |
| abehiroshi | 215 | 228 | -13 | ok (Chrome has no contentful paint on this page: its first-paint) |
| zol | 3013 | 3032 | -19 | ok |
| youtube | 491 | 660 | -169 | ok |
| youtube-search | 857 | 964 | -107 | ok |
| react | 656 | 764 | -108 | ok |
| mistral | 662 | 576 | 86 | ok |

20 of 20 pass.

## Validation

- Release compiler build, `browser/build.sh` (fast) and
  `target/release/aye check browser`: passed.
- `cargo test --test browser`: **40 passed**, including unchanged goldens,
  GC stress, live-frame mutation, timing stdout/PNG equality, media
  invalidation, and replay with the HTTP server killed (CSS/JS/image,
  redirect/failure). Engine/CSS/DOM units cover source keys, candidate
  coverage and structural invalidation.
- Full `cargo test`: passed (31 compiler units, 40 browser tests,
  4 FFI tests, **242 goldens**, 5 spec tests). The existing ThreadSanitizer
  CLI case reports its platform skip; no new skips or golden changes.
- No compiler workaround was needed; `COMPILER_GAPS.md` is unchanged.
  `docs/browser.md` and `docs/browser-sites.md` are untouched.
