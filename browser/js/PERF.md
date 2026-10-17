# JS performance (B8)

## Result and measurement

**The 3× geometric-mean target is not reached.** The final five-run,
interleaved comparison gives **1.95×** end-to-end wall-time speedup. An
earlier five-run interleaved comparison gave 1.82×. The host is shared
with other agents; absolute times, especially regexp thread creation,
vary substantially. These are measured results, not Octane scores or
claims about a fully functional React application.

Environment: Apple M2 Pro, arm64 macOS, Node v22.22.2. Both native
binaries use Aye's fast mode (clang -O2). The saved baseline was built
from repo/compiler `5dd6afb`, before engine changes. The initial
three-run baseline is recorded separately in
`../tests/jsbench/README.md`; it is not silently replaced by the later
remeasurement below. Startup, builtin/prelude installation, parsing,
execution, GC, and teardown are all timed. Each result is checked
against its `.out` and Node's stdout.

Final comparison: median of five subprocess runs per binary, alternating
which binary runs first each round, on the unchanged workloads:

| Benchmark | Baseline (s) | After (s) | Speedup | Checksum |
| --- | ---: | ---: | ---: | ---: |
| array | 0.130128 | 0.090359 | 1.44× | 96751118 |
| closures | 1.457652 | 0.967120 | 1.51× | 306584040 |
| JSON | 0.370532 | 0.267565 | 1.38× | 20672500 |
| methods | 0.146244 | 0.112598 | 1.30× | 267460185993 |
| NavierStokes | 5.194493 | 2.958638 | 1.76× | 77 |
| Richards | 0.465513 | 0.442238 | 1.05× | 2916400 |
| strings | 1.532163 | 0.053901 | 28.43× | 650400 |
| **Geometric mean** | | | **2.14×** | |

Reproduce (the baseline binary must be saved before editing the engine):

```sh
browser/build.sh
cp target/dweller /tmp/dweller-baseline
# Build the changed engine, then:
browser/build.sh
BENCH_BIN=target/dweller BENCH_BASELINE=/tmp/dweller-baseline \
  BENCH_RUNS=5 browser/tests/jsbench/run.sh
```

The runner can also time one binary, or select workloads by name.
Richards and NavierStokes retain the upstream Octane algorithms,
assertions, and license notices; only their harness is ported. The other
five workloads are original to this suite.

### Real page

`dweller dump --md --js https://react.dev/` succeeded for both builds.
The initial baseline was 5.640 s. The final paired measurement was
**3.897 s baseline → 4.191 s after**; an earlier pair was 3.586 → 3.593 s.
Both final outputs were byte-identical (10,126 bytes of Markdown), with
empty stderr. There is **no demonstrated page-load speedup**. This test
includes network, HTML, style, layout, and Markdown generation, and the
engine's existing limitations still apply (including skipped modules).

## Profiling and changes

Native macOS `sample` identified costs before changing the engine:

```sh
target/dweller js browser/tests/jsbench/navier-stokes.js & pid=$!
sample "$pid" 3 1 -file /tmp/navier.sample
wait "$pid"
```

Short workloads were also repeated/scaled in temporary profiling inputs,
not in the timed benchmarks. Ignore idle/main-thread join samples when
attributing execution costs to the script runner's thread.

- **Regexp:** string matching/replacement stacks went through `search`
  into `pthread_create`/`pthread_join`, VM mapping, and unmapping. A
  subject longer than 2,000 bytes used a new 1 GB virtual-stack thread
  for every search, regardless of actual match depth. `regexp.aye` now
  tries at depth 128 in the current thread. Any depth exhaustion retries
  from the original search position on the original large-stack path,
  even if a shallower alternative succeeded. Steps, captures, and
  repetition registers restart; shallow matches need no thread. Small
  subjects and the existing large-stack/failure limits are unchanged.
- **Tree walk:** NavierStokes' baseline execution samples were dominated
  by `eval` (1,302 top-of-stack samples) and `readIdent` (423), ahead of
  `getIndex` (103) and `fastNumeric` (69). Local/constant leaves now skip
  the general evaluator and its stack probe. Numeric local updates avoid
  redundant conversion and assignment lookup; compound numeric operators
  use the same numeric fast path as binary expressions.
- **JSON:** baseline native samples showed `Stringifier.quote` (194),
  `JsonParser.stringBytes` (110), atom interning (88), and libc number
  formatting/parsing. Unescaped input strings/keys use source spans;
  output quotes copy ordinary spans in bulk. Validated decimal numbers
  with at most 15 digits and no exponent use an exact integer numerator
  and power-of-ten denominator followed by one correctly rounded division.
  Other numbers retain libc parsing. Small exact eighths avoid the
  snprintf/strtod shortest-decimal search; other formatting is unchanged.

### Shared layouts and caches (`shapes.aye`, `heap.aye`)

An object's shape describes its insertion-ordered own keys/slots. A
transition `(parent shape, key atom)` shares the next layout among all
objects built the same way. Large layouts share an atom-to-slot index
instead of allocating one per object. Values and descriptors remain in
`Obj.props`, the existing slot array: this deliberately keeps the host
interface and reflection code compatible rather than introducing a
second representation for descriptor-heavy objects.

Deletion switches that object permanently to dictionary mode; its
remaining order/index behavior is unchanged. There are at most 262,144
shapes (16,384 until P4.17: youtube-search needs about 67,000), and a layout wider than 128 properties becomes a dictionary, so
one growing object cannot accumulate quadratic permanent index metadata.
Static, unique, data-only object literals cache their final shape and
reserve/fill their slots directly; spread, computed/duplicate keys,
methods, accessors, and `__proto__` keep the original builder.

Each AST node has a monomorphic property cache: receiver shape, slot,
and optionally the immediate prototype shape. Member reads, writes,
method lookup, and global variable reads load the *current* slot value.
Prototype hits follow the *current* link and validate both shapes.
Flags/accessors/frozen state are rechecked; descriptor mutation needs no
invalidation epoch. Proxies, host hooks, string wrappers, array length
and numeric-index semantics bypass these caches. No function value or
prototype object is retained in a cache. Deeper prototype chains and
polymorphic sites fall back to the original lookup.

Property keys were already interned atoms in the baseline. This work
uses those atoms throughout layouts/caches rather than claiming a new
string-to-atom subsystem.

### Guarded recipes and calls (`fast_expr.aye`, `interp.aye`)

After resolution, eligible expressions get flat recipes for constants,
local slots, own data-property reads, dense array reads, numeric unary
operators, and numeric binary operators. Scratch uses unboxed numbers,
object/string indices and kind tags. Recipes are bounded to 64
instructions/32 scratch entries, and total code is capped at 1M
instructions. Unsupported/deep expressions retain tree walking.

A guard failure reruns the AST, **before any observable operation**:
recipes cannot write, call JS, invoke accessors/proxies/host hooks, or
allocate a JS value. TDZ values, holes, nonnumeric operands and special
property semantics fall back. A simple, single-return function can run
its recipe directly from the rooted argument stack/parent environment,
without constructing a frame/environment. Strict/sloppy/arrow `this`,
missing arguments, budget ticks and depth checks are preserved.
Generators, async functions, constructors, defaults/rest, `arguments`,
named self bindings, and general bodies retain normal invocation.

Uncaptured function/block environments are returned to the heap free
lists at scope exit. Resolver closure flags are conservative; escaping
closures and coroutine contexts remain GC-managed. Ordinary functions
also defer the *value* of their own `prototype`: the descriptor exists
immediately, and only observation/construction/reflection materializes
the prototype and its constructor back-reference. Reflection resolves
only the lazy own slot, never invoking an inherited/accessor getter.

Numbers remain f64; there is no new integer Value tag. Integer-range
ToInt32/ToUint32 casts and signed-small-integer remainder avoid libc
calls, with explicit preservation of negative zero and fallback for
large/fractional/nonfinite inputs.

### GC and compatibility

Layouts and caches contain only atom/layout/slot integers, not GC roots.
Recipe scratch is used only in a nonallocating, non-reentrant attempt;
its stale entries are never subsequently read without being overwritten.
Calls still use the existing `[function, this, args...]` rooted stack.
Lazy prototype allocation roots its owner, and reflection roots the
pending descriptor. Literal slots are published incrementally for GC
tracing before later value expressions allocate. Host APIs are unchanged.

Regression scripts `224`–`228` cover bounded regexp retries, descriptor
and prototype changes, deletion/dictionaries, freezing, proxies, recipe
bailout/coercion/TDZ, escaping environments, JSON/number edge cases, and
lazy-prototype reflection (including getter non-invocation).

## P11.3: `ArrayBuffer` on one contiguous byte store

An `ArrayBuffer` is now an object whose data is `ObjData.Bytes`: one
native allocation (V8's BackingStore), outside the lists the collector
scans, freed with the object (`heap.freeObj`) or on `transfer()`
(detach). Views (`ObjData.View`) name the buffer itself and read/write
little-endian bytes; `set` (typed source), `slice`, `copyWithin`,
`fill`, `ArrayBuffer.prototype.slice`/`transfer` are memmove/memcpy
natives (`typed.aye`). Same machine, `dweller js`, before (main
c87a96f) / after:

| | before | after |
|---|---|---|
| `new Float32Array(1 << 20)` | 264 ms | 11–14 ms (19–24x) |
| `new Float32Array(4 << 20)` (16 MB) | 991 ms | 44–45 ms |
| fill 16 MB element by element (`a[i] = i * 0.5`, global `var` loop) | 15466 ms | 14733–14853 ms |
| sum 16 MB element by element | 16276 ms | 15533–16294 ms |
| `Float32Array(1 << 20).fill(1.5)` | 705 ms (generic Array `fill`) | 0 ms |

The element loops are the interpreter's: an empty `for` over 1M global
`var`s takes 1.8–2.4 s, a plain `Array` store loop 3.7 s, the typed one
3.5 s. Element access was never the cost there; the loops need the
interpreter work of "Next steps".

## P4.63: bytecode VM core (straight-line functions)

`DWELLER_JS_VM=1` compiles a function on its first call (`compiler.aye`)
and runs it in the dispatch loop (`vm.aye`) when it is straight-line
code: literals, parameters and uncaptured locals in registers, every
unary, binary, comparison and update operator, assignments to
registers, `var`/`let`/`const`, blocks without functions, `return`.
Anything else (calls, properties, globals, control flow, closures) keeps
the function on the tree walker; `DWELLER_JS_VM_STATS=1` prints the
reasons at exit. The VM is tried after the tree walker's frameless
one-expression path (`fast_expr.aye`), which is cheaper per call for the
functions it handles (array callbacks: +0.1 s on `array` when the VM went
first).

Fast build, 4-core Xeon 2.1 GHz Linux, whole process, median of 2–3:

| | VM off | VM on |
|---|---:|---:|
| 1M calls of a 10-statement arithmetic function (87 instructions per call), from a tree-walked loop | 1.53–1.70 s | 1.03–1.16 s |
| the same loop with `return 0` as the body (call and loop cost) | 0.51–0.61 s | 0.54–0.56 s |
| function body alone (difference) | about 1.0–1.1 s | about 0.5 s (2–2.5×) |

About 6 ns per instruction. The jsbench suite (`jsbench.py`, one run
each, VM off / on) is unchanged within the noise of single runs (±10%),
every checksum equal: array 0.197/0.197, calls 4.03/4.31, closures
2.90/2.91, crypto 3.53/3.37, dom 5.02/5.44, globals 2.57/2.71, json
0.52/0.66, loop 1.99/2.00, methods 0.59/0.43, navier-stokes 14.5/16.1,
raytrace 1.83/2.13, richards 1.63/1.68, splay 3.11/3.17, strings
0.11/0.12, three 11.8/11.8 s. Their hot functions call, read properties
or loop, so almost none compile yet (`calls`, `richards`, `raytrace`: 0
functions; `crypto` and `navier-stokes`: 1 each). On the 261 scripts of
`browser/tests/js/`: 194 functions compiled, 3355 on the tree walker
(1008 context variable, 695 Call, 295 FuncDecl, 270 ForIn, 190 If, 128
generator or async, 112 Member, 109 Cond, 103 class constructor, ...).

## P4.75: the VM on by default

`DWELLER_JS_VM` defaults to 1 (`DWELLER_JS_VM=0` runs the tree
walker). jsbench, fast build, median of 3 against
`browser/tests/jsbench/baseline.json` (f888c35, same machine type):
geomean 2.45× faster than the baseline (navier-stokes 6.12×, loop
4.19×, globals 4.14×, richards 3.72×, crypto 3.59×; dom 1.10×, array
1.26×, json 1.30×), 3.28× slower than `node --jitless` (calls 7.4×,
globals 7.6×, closures 5.3×, splay 5.0×; navier-stokes 1.8×, loop
1.9×). All checksums equal. The per-workload table is in `BYTECODE.md`
section 6.

## Next steps

The general interpreter is still the bottleneck. Small property tables
were cheap already, so shapes/caches alone do not supply 3×. A real
bytecode/register VM is the next substantial step: compile statements,
loops, lvalue evaluation and calls, with rooted register windows and
shared layouts. Keep the existing interpreter as a differential oracle.
The bounded recipes are useful groundwork, not a replacement VM.
The VM's design and cards: `BYTECODE.md` (P4.30, 2026-10-15).

Then measure polymorphic/deeper-chain caches, value-only slots with a
separate descriptor fallback, virtual call frames for general uncaptured
functions, and a nonrecursive regexp VM. A proper shortest-decimal
algorithm could replace the remaining snprintf/strtod loop.

Experiments splitting all hot AST nodes into another dispatch and
transactional assignment recipes were discarded: they added complexity
and did not reliably improve the suite. Do not count their transient
numbers or change workload sizes to meet the target. Preserve both fast
and debug conformance, GC stress, and DOM/host coverage during further
work.

## Final validation

- `cargo build --release` and `browser/build.sh`: passed (fast binary).
- `target/release/aye check browser`: passed.
- All 228 conformance scripts match stdout/stderr/exit-status goldens in
  fast mode, and all 228 pass with `DWELLER_JS_GC=stress`.
- All eight benchmark checksums agree with Node v22.22.2.
- `cargo test --test browser`: 34 passed (including debug JS conformance,
  DOM GC stress, browser/Aye unit tests, and host/UI fixtures).
- Full `cargo test`: passed — 31 unit tests, 34 browser tests, 4 formatter
  tests, 242 golden cases, and 5 spec tests; no failures. The existing
  ThreadSanitizer CLI runtime case was skipped because clang's TSan
  cannot run on this host.

No compiler, std, DOM binding, or `docs/browser.md` edits were needed.
The encountered boolean-to-number cast limitation and workaround are
recorded as entry 48 at the end of the open list in
`browser/COMPILER_GAPS.md`.
