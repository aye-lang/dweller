# JS bytecode VM (design, P4.30)

Decision of 2026-10-15: Dweller keeps its own engine (`browser/js/`,
about 26k lines of Aye). The engine gets a bytecode compiler and an
interpreter with inline caches, so that its speed is in the class of
QuickJS and of V8 without its JIT (`node --jitless`). There is no V8
embed and no JIT in this plan. The tree walker (`interp.aye`) stays until
the VM runs everything, then its execution paths are removed.

Why now: `PERF.md` measured that shapes, caches and fast expression
recipes give 1.95× and that "the general interpreter is still the
bottleneck". The user builds 3D web games (three.js, WebGL is designed
in parallel), so JS speed per frame matters.

This file is the design. The cards are in `docs/plans/browser.md`
(P4.59 to P4.79; P4.30 is the umbrella).

## 1. Measure first

### Where we are (2026-10-15, Apple M2 Pro, fast build of `320be32`)

The suite in `browser/tests/jsbench/` (median of 3 runs, whole process).
For reference, `node --jitless` is V8 with only the Ignition interpreter:
it is the target class of this plan.

| workload | dweller (s) | node --jitless (s) | ratio |
| --- | ---: | ---: | ---: |
| array | 0.117 | 0.05 | 2.3× |
| closures | 1.124 | 0.22 | 5.1× |
| json | 0.275 | 0.22 | 1.3× |
| methods | 0.149 | 0.07 | 2.1× |
| navier-stokes | 5.412 | 0.88 | 6.2× |
| richards | 0.771 | 0.12 | 6.4× |
| strings | 0.057 | 0.04 | 1.4× |

Node's start-up (about 0.03 s) is in its times, so the short workloads
understate the gap; the long ones (richards, closures, navier-stokes)
show it: 5× to 6.4×.

Three small loops written for this design (they become the `loop`,
`globals` and `calls` workloads of P4.59):

| loop | dweller (s) | node --jitless (s) | ratio |
| --- | ---: | ---: | ---: |
| `s += i & 7`, 10M times, locals of a function | 0.99 | 0.13 | 7.6× |
| the same loop at the top level (`var` = global object property) | 3.53 | 0.13 | 27× |
| `p = p.add(d)`, 2M times (method call, `new`, 2 property writes) | 1.69 | 0.15 | 11× |

Real sites, replayed from the archives (`page_load.py run`, median of
3, seconds). `scripts` is the time inside page scripts; `timers` is
timer and rAF callbacks, including the layout they force:

| site | wall | scripts | timers |
| --- | ---: | ---: | ---: |
| youtube | 2.862 | 1.445 | 1.285 |
| github | 5.284 | 0.301 | 3.092 |
| react | 0.500 | 0.146 | 0.247 |
| mistral (three.min.js, rAF) | 25.73 | 1.056 | 24.58 |

Command (the archives are local and untracked):
`python3 browser/tools/page_load.py run --archive target/sites --binary target/dweller --site youtube,react,github,mistral --repeat 3 --json target/js-sites.json`

### The benchmark set (P4.59, P4.60)

All workloads run with `dweller js FILE` (DOM: `dweller dump --js`),
print a checksum that Node also prints, and are timed by
`browser/tests/jsbench/run.sh`. P4.59 adds a reference column from
`node --jitless` and commits the numbers of the day as
`browser/tests/jsbench/baseline.json`. Every VM card quotes its numbers
against that file.

| workload | what it measures | source and license |
| --- | --- | --- |
| richards, navier-stokes | (present) calls, property access, float arrays | Octane, BSD-3 / MIT (present) |
| splay | object allocation, GC, tree of objects | Octane `splay.js`, BSD-3 (V8 authors) |
| crypto | integer arithmetic on arrays (RSA with big numbers in arrays) | Octane `crypto.js`, Tom Wu's MIT-style license (notice kept) |
| raytrace | small objects, method calls, float math | written for this suite (Octane's `raytrace.js` has no license statement) |
| loop, globals, calls | the three loops above | written for this suite |
| three | three.js r170 scene graph: 2000 `Object3D`s, each frame sets position and quaternion and calls `scene.updateMatrixWorld()`, plus `Vector3`/`Matrix4`/`Quaternion` math; 300 frames; checksum of the world matrices | `three.module.min.js` 0.170.0 vendored with its MIT license |
| dom | 20k `createElement`/`appendChild`/`setAttribute`/`classList`/`querySelector` and 10k event dispatches, run by `dweller dump --js` on a local page | written for this suite |

Not vendored: Octane `deltablue.js` is GPL-2 (Maloney and Wolczko), and
`raytrace.js` has no license statement. The repository is MIT OR
Apache-2.0, so neither can be copied. The new `raytrace` and the
present `methods` workload cover the same code shapes.

The user's game `~/Documents/f1/web` (three ^0.170, physics in
WebAssembly) is the real 3D workload. Its per-frame JS cost can only be
measured when Dweller has WebGL (designed in parallel) and WebAssembly
(no design yet). Card P4.79 records it then. Until then the `three`
workload stands in for it.

### Correctness gates

Every VM card runs these (only the ones named in its verify line):

- `cargo test --release --test browser -- js_conformance js_conformance_vm js_command_line`:
  the 261 scripts of `browser/tests/js/` with the Node goldens, with
  the VM off and on. P4.63 adds the `js_conformance_vm` test.
- the same with `DWELLER_JS_GC=stress` for the VM. Frames and registers
  are roots, so this checks the rooting rules of section 4.
- `cargo test --release --test browser -- jsdom_fixtures jsdom_gc_stress`:
  the 103 DOM fixtures.
- `python3 browser/tools/page_load.py check`: the Markdown of each of
  the 22 recorded sites is unchanged (local run).
- test262 (P10.3): `python3 browser/tools/test262.py run test/language -j 8`.
  The pass count with the VM must be at least the pass count without
  it. When P10.3 has not landed, this gate is skipped and the card says
  so.

## 2. VM shape

### Decision: accumulator and registers, as V8's Ignition

Instructions read registers and an implicit accumulator, and write the
result to the accumulator: `Ldar r1; Add r2, [slot]; Star r3`.
Registers are slots of the function's frame on the value stack.

Reasons:

1. **Aye has only switch dispatch.** Aye compiles to C; a `match` on a
   `u8` opcode becomes an if/else chain, and clang `-O2` turns it into
   a jump table with one shared indirect branch. This was checked for
   this design: a 48-arm `match` in a loop compiles to
   `ldrb; adr; add; br x13` with a `.byte` offset table. Aye has no
   computed goto, no labels as values and no guaranteed tail calls, so
   threaded dispatch is not available. Each dispatch costs a bounds
   check, a table load and a hard-to-predict indirect branch, so the
   design must execute **few instructions per JS operation**. A stack
   VM (QuickJS) needs about one instruction per operand push: `a = b + c`
   is `get_loc b; get_loc c; add; put_loc a`, four dispatches. QuickJS
   pays for that with computed-goto dispatch, which Aye cannot emit.
   Ignition needs `Ldar b; Add c; Star a`, three, and with the register
   optimizer (P4.77) often two.
2. **The reference implementation is in the user's checkout.** Every
   part has a V8 counterpart to port from (BSD-3, attribution in the file
   header): `src/interpreter/bytecodes.h` (the instruction set),
   `bytecode-generator.cc` (AST to bytecode), `bytecode-array-builder.h`,
   `bytecode-register-allocator.h`, `control-flow-builders.h` (loops,
   switch, try/finally), `handler-table-builder.h`,
   `bytecode-register-optimizer.h`, `src/objects/feedback-vector.h`,
   and `src/ic/`. A stack design would leave the cards to work it out
   from scratch.
3. **The accumulator stays in a machine register.** It is a local of
   the dispatch loop, so clang keeps it in a register. Registers live on
   `Interp.stack`, so the existing collector already treats them as
   roots (section 4).
4. **Three-address code (Lua 5)** would need even fewer dispatches, but
   it has wider instructions and no reference implementation here. If
   P4.77 measures hot `Ldar`/`Star` pairs that the register optimizer
   cannot remove, fused three-address forms of the numeric operators are
   added there.

If Aye ever gains labels as values or `musttail`, only the dispatch loop
changes. This would be a request to the Aye repository, not a
requirement of this plan.

### Instructions

The encoding is one byte for the opcode and one byte for each operand.
`Wide` and `ExtraWide` prefixes give 16-bit and 32-bit operands, as in
V8. Operand kinds: register, register list (first register and count),
8-bit immediate, constant-pool index, feedback-slot index, context depth
and slot, jump offset. The first set, grouped as in V8's `bytecodes.h`
(`r` = register, `[s]` = feedback slot):

- accumulator loads: `LdaUndefined`, `LdaNull`, `LdaTrue`, `LdaFalse`,
  `LdaZero`, `LdaSmi imm`, `LdaConstant k`, `LdaTheHole`, `Ldar r`,
  `Star r`, `Mov r, r`
- contexts (captured variables): `LdaContextSlot depth, slot`,
  `StaContextSlot depth, slot`, `LdaCurrentContextSlot slot`,
  `PushContext r`, `PopContext r`, `CreateBlockContext n`,
  `CopyContext` (one `let` copy per loop iteration, section 2.4),
  `ThrowReferenceErrorIfHole name` (TDZ)
- globals: `LdaGlobal name, [s]`, `StaGlobal name, [s]`,
  `TypeOfGlobal name, [s]` (`typeof x` for an undeclared `x`)
- properties: `GetNamedProperty r, name, [s]`, `SetNamedProperty r,
  name, [s]`, `DefineNamedOwnProperty`, `GetKeyedProperty r, [s]`,
  `SetKeyedProperty r, r, [s]`, `DeletePropertyStrict/Sloppy r`,
  `GetSuperProperty`, `GetPrivate`/`SetPrivate`
- operators: `Add r, [s]` (and `Sub`, `Mul`, `Div`, `Mod`, `Exp`,
  `BitwiseOr/And/Xor`, `ShiftLeft/Right/RightLogical`), the `Smi` forms
  `AddSmi imm, [s]` and so on, `Inc`, `Dec`, `Negate`, `BitwiseNot`,
  `ToNumeric`, `LogicalNot`, `TypeOf`, `TestEqual r`,
  `TestEqualStrict r`, `TestLessThan r` (and `>`, `<=`, `>=`),
  `TestInstanceOf r`, `TestIn r`, `TestUndetectable`, `TestNull`,
  `TestUndefined`
- control: `Jump off`, `JumpIfTrue off`, `JumpIfFalse off`,
  `JumpIfToBooleanTrue/False off`, `JumpIfNull`, `JumpIfUndefined`,
  `JumpIfUndefinedOrNull`, `JumpLoop off` (the budget tick and the
  interrupt check, as V8's `JumpLoop`), `SwitchOnSmi table`, `Return`,
  `Throw`, `ReThrow`, `SetPendingMessage`
- calls: `CallProperty r_fn, r_args, n, [s]` (`this` in r_args[0]),
  `CallUndefinedReceiver r_fn, r_args, n, [s]`, `CallWithSpread`,
  `Construct r_fn, r_args, n, [s]`, `ConstructWithSpread`,
  `CallRuntime id, r_args, n` (the slow paths: class definition,
  template objects, spread into a literal)
- creation: `CreateClosure func`, `CreateObjectLiteral k`,
  `CreateArrayLiteral k`, `CreateEmptyArrayLiteral`,
  `CreateRegExpLiteral k`, `CreateArguments`, `CreateRestParameter`
- iteration: `GetIterator r, [s]`, `ForInPrepare`, `ForInNext`,
  `ForInStep`
- generators: `SuspendGenerator r_gen, r_regs, n, id`,
  `ResumeGenerator r_gen, r_regs, n`, `SwitchOnGeneratorState r_gen,
  table`

New instructions are added only when a card needs them. Each has one
entry in a table (`bytecode.aye`) with its name, operands and two flags:
**may allocate** (section 4) and **may throw**.

As built (P4.61): Aye has no `enum: u8` and no named constants in
`match` patterns, so an opcode is a `pub let BC_NAME: u8` and the table
`BC_INFO` is indexed by it; the dispatch `match` of P4.63 uses the
numbers with the name in a comment. A forward jump's operand width is
fixed when it is emitted by reserving a constant-pool entry (V8's
`ConstantArrayBuilder` slices); an offset that does not fit at `bind`
goes to that entry and the jump becomes its `Jump*Constant` twin. Jump
offsets count from the opcode byte (after a prefix). `SwitchOnSmi k, n,
base` reads its targets from pool entries `k` to `k + n - 1`.

As built (P4.63): `compiler.aye` and `vm.aye`. The dispatch `match`
has one arm per opcode (an arm with several patterns split it); clang
`-O2` (fast build, x86-64) makes it two jump tables, opcodes 2–43 and
44–72, the second reached through the first's bounds check (checked
with `objdump -d` of `aye_f_0js_3a_3avmRun`). The frame record is the
existing `Frame` without new fields: no instruction calls JS-to-JS yet,
so `vmCall` runs one dispatch loop per call in C; the `code`/`pc`/`fp`
fields come with calls (P4.65). Unary `+` got its own instruction,
`ToNumber` (a BigInt throws, unlike `ToNumeric`). `let`/`const` read
before their declaration in the source (the dead zone) and the
`js_conformance_vm` GC-stress pass leaving out the allocation and deep
recursion scripts (minutes each under stress) are P4.63 limits.

As built (P4.64): jumps are all forward except `JumpLoop`, one per
loop (`continue` jumps forward to it). `JumpLoop` ticks the budget
(`Interp.tick`), so a runaway compiled loop ends on the step limit or
deadline like a tree-walked one. The VM reads a jump's offset from the
opcode byte; `SwitchOnSmi` falls through for a value that is not an
integer in range, to the `Jump` to the default that follows it.

As built (P4.65): `CallProperty r, argc, [n]`, `CallUndefinedReceiver`
and `Construct` take the callee, `this` and the arguments in
consecutive registers from `r`; `[n]` is the AST `Call`/`New` node (the
callee's text for the TypeError). The values are copied to the top of
the stack, where `callAt`/`constructAt` expect them. A plain compiled
function is entered in the same loop: `vmPushFrame` builds its frame
above the copy, the caller's code, pc, fp and arguments go to
`VmState.saved`, and its `Return` pops back (a construction keeps the
new object unless an object is returned); everything else (natives,
bound functions, proxies, class constructors, generators, functions on
the tree walker) goes through `callAt`/`constructAt` and recurses in C.
`MAX_DEPTH` counts the frames either way. A throw unwinds the frames
the loop pushed. The saved callers live in `VmState.saved`, not in new
`Frame` fields: `Frame` stays as it was. Until the inline caches of
P4.67, `LdaGlobal [n]`, `GetNamedProperty r, [n]` and
`SetNamedProperty r, [n]` are generic and use the tree walker's
per-node property cache (`[n]` the `Ident` or `Member` node).
`CreateArguments` (unmapped; a sloppy function with simple parameters
has a mapped one in the context and falls back) and
`CreateRestParameter k` read the arguments the frame keeps (`argBase`,
`argc`). Non-simple parameters are registers (the resolver's choice, as
V8): copied from their slot at entry, in order, with defaults and the
rest array; each is in its dead zone until then.

As built (P4.66): the current context is `Interp.env`, and a context
is the tree walker's `Env` (a binding keeps its tree-walker slot), so
closures move freely between the engines. `vmPushFrame` sets
`Interp.env` to the callee's closure environment and `vmPopFrame`
restores the caller's (the `Frame.env` it saved). A function makes its
context at entry (`CreateFunctionContext n`) only when it creates
closures or has context slots; a block or `for` scope with slots that
creates closures uses `PushContext r, n` (it makes the block context
itself; the outer one is saved in r as a number, an ancestor the
collector reaches through the current context) and `PopContext r`,
which `break`/`continue` also emit for each context they leave. Scopes
without a context are skipped when counting `LdaContextSlot depth,
slot`. `CopyContext` replaces the loop context with a copy (same
parent) before the update expression. `LdaClosure` loads the running
function (a named function expression's own name).
`ThrowReferenceErrorIfHole name` follows reads of block-level and outer
bindings; function-level contexts start undefined, as the tree
walker's.

As built (P4.67): `StaGlobal [n]`, `TypeOfGlobal [n]`,
`GetKeyedProperty r, [s]`, `SetKeyedProperty r_obj, r_key, [s]`,
`DeleteProperty r` (key in the accumulator), `TestIn`,
`TestInstanceOf`, `CreateObjectLiteral [n]` (the `ObjectLit` node: a
`literalShape` literal gets its final shape and slots at creation),
`DefineNamedOwnProperty r, [p], k` and `CreateArrayLiteral`,
`StaInArrayLiteral r`. Named loads and stores and globals keep the
per-node `PropCache` (one node is one site); feedback-vector slots and
the compiled top level are P4.67b. The DOM gates run with the VM on.

As built (P4.67b): a classic script's top level is compiled as a
function body with no parameters and no function object
(`compileScript`, run by `runScript` in a frame whose `func` is NONE,
so strictness comes from `script_strict`; the code is freed when the
script ends). Top-level `var` and function declarations are global
object properties (`StaGlobal`, or `StaGlobalInit [n]` for a hoisted
function), top-level `let`/`const` initialize their `glex` slot with
`StaGlobalInit [n]` (`assignIdent` with `init`), and every expression
statement stores its value in a register the compiler reserves, which
the script returns: `evalScript` takes it as the completion. A script
the compiler cannot handle stays on the tree walker. Global loads and
stores of `var`s check the global object's property at the cached
slot by key (`vmGlobalSlot`), which holds in dictionary mode too (the
global is one after any deletion). Named property sites still use the
per-node `PropCache`; feedback-vector slots are left for a later card.

A compiled function (`BytecodeArray`, kept beside the AST's `FuncInfo`,
not on the JS heap) has: the code bytes, a constant pool (numbers,
atoms, function ids, literal templates; no heap values), the register
count, the parameter count, the feedback-slot count, a handler table
(try ranges), and a source-position table (bytecode offset to AST node,
V8's `src/codegen/source-position-table.h`) for error traces,
`DWELLER_JS_TRACE` and the telemetry `last_stmt`.

Compilation is lazy, on the first call, as in V8. A function the
compiler does not support records why and runs on the tree walker for
the rest of the engine's life (section 5). `dweller js --print-bytecode
FILE` prints the disassembly (V8's `--print-bytecode`).

### Frame layout

The caller already pushes `[function, this, args...]` on `Interp.stack`
(`callAt`). A VM frame starts right above them, so parameters need no
copy (V8 also gives parameters negative register indices):

```
stack: ... | fn | this | a0 a1 ... a(argc-1) | (undefined up to nparams) | r0 r1 ... r(nregs-1) |
                                               fp ^
```

- Parameter `i` is register `-(nparams - i)`; `this` is below them.
  Missing arguments are padded with `undefined` at entry. Extra
  arguments stay below the parameters, where `CreateArguments` and
  `CreateRestParameter` read them.
- Register 0 is the accumulator's home (section 4).
- The frame record is the existing `Frame` in `Interp.frames`, with new
  fields: `code` (BytecodeArray id), `pc`, `fp`, `argc`, `ctx`
  (current `Env`), and `entry` (true for the frame where a native or the
  tree walker entered the VM; returning from it leaves the loop). The
  collector marks `ctx`. `PushContext r` saves the outer context in
  register `r` as a number; the saved context is always an ancestor of
  `ctx`, so the collector reaches it through `ctx`.
- **JS-to-JS calls do not recurse in C.** `Call` and `Construct` to a
  compiled function push a frame and continue the same dispatch loop;
  `Return` pops it. Only a call into a native or into the tree walker,
  and a native calling back into JS, enter a new dispatch loop. Deep JS
  recursion then uses no C stack, and `MAX_DEPTH` counts frames.
- `Interp.stack` is a `List(Value)` that can move when it grows. The
  loop holds `fp` as an index, not a pointer. On entry each frame
  reserves `nregs` plus the largest outgoing argument list, and the
  loop reloads its base pointer after any instruction that may call.

### Scopes, closures and `let`

Today every variable is a slot of a heap `Env`, and every call makes
one (`invoke` frees it at exit when no closure was created).

- `dweller js --print-scopes FILE` shows the resolver's allocation
  (P4.62; goldens in `browser/tests/js/scopes/`).
- The resolver (P4.62) marks each binding **captured** when a nested
  function refers to it (V8: `Variable::IsContextSlot`). A binding that
  is not captured is a register. A captured one is a slot of the
  function's context `Env`, which is made at entry only when the
  function has captured bindings. Most hot functions (richards, three.js
  math) then allocate nothing per call.
- `LdaContextSlot depth, slot` walks `depth` parents from the current
  context. Depths are known statically, as today.
- Blocks with captured `let`/`const`/`class` push a block context
  (`CreateBlockContext`, `PushContext`/`PopContext`). A `for (let ...)`
  loop whose variables are captured copies its context once per
  iteration (`CopyContext`), as the tree walker does now (`execFor`) and
  as V8 does. Uncaptured loop variables stay registers, with no copy.
- TDZ: a `let` register or slot starts as `TheHole` (`Value.Empty`), and
  a read the resolver cannot prove initialized emits
  `ThrowReferenceErrorIfHole`.
- `this`, `new.target` and the function itself are registers. An arrow
  function reads them from its context when it captures them, as today.

### Exceptions and `finally`

- A JS exception is still the Aye error `JsError.Thrown` with the value
  in `Interp.exception`. Natives and the tree walker do not change.
- A thrown error inside the loop goes to an unwind step: it looks up
  the current pc in the frame's handler table (try start, try end,
  handler pc, context register; V8's `handler-table-builder.h`). When
  there is an entry, it restores the context, puts the exception in the
  accumulator and jumps to the handler. When there is none, it pops the
  frame and repeats. At an `entry` frame it returns the error to the
  caller.
- `finally` follows V8's `ControlScopeForTryFinally`
  (`bytecode-generator.cc`): each way out of the try (fall through,
  `return`, `break`/`continue` to a label outside, throw) stores a token
  and a value in two registers and jumps to the one copy of the finally
  block. The block ends with `SwitchOnSmi` on the token that
  continues the way out.
- `JsError.Terminated` (budget) and `OutOfMemory` skip all handlers.
  `ShortCircuit` (optional chaining) is not needed: the compiler emits
  `JumpIfUndefinedOrNull` to the end of the chain.
- Generator `return(v)` keeps the special `gen_return` value that no
  `catch` sees but `finally` blocks run for.

### Generators and async functions: resumable frames

Today a generator or async call runs on a `std.coro` coroutine: a C
stack of its own (256 MB reserved, at most 4000 alive), because the tree
walker keeps its state on the C stack.

A VM frame keeps all its state in its registers, its context and its
pc. A `yield` or `await` can only appear directly in the generator's
own body, never inside a nested call. So the VM suspends a generator as
V8 does (`SuspendGenerator`, `ResumeGenerator`,
`SwitchOnGeneratorState`; `src/objects/js-generator.h`): it copies the
frame's registers into the generator object, records the suspend id and
returns. `next()` copies them back and jumps to the resume point. No C
stack is needed.

- The generator object's saved registers are traced by the collector
  as part of the object (a new `ObjData` case).
- Async functions use the same mechanism. `await` suspends and hands
  the value to the driver in `lib.aye`, which resumes the frame when the
  promise settles. The driver does not change.
- Generators and async functions that the VM cannot compile keep
  running on `coroutine.aye`. When none are left, `coroutine.aye` is
  removed (P4.78).

### `eval`, `with` and `arguments`

- `with` is not supported by the parser today, so the VM needs no
  dynamic scope.
- `eval` is always indirect (`globalEval` runs `evalScript` in the
  global scope). The VM compiles the evaluated script like any other
  script. If direct eval is ever added, a function containing it falls
  back to the tree walker.
- `arguments` is an unmapped copy today (`invoke`), and stays one:
  `CreateArguments` copies from the frame.

## 3. Inline caches

### Feedback per function

Each compiled function has a **feedback vector**: one entry for each
`[s]` operand (V8: `src/objects/feedback-vector.h`). It replaces the
per-AST-node `prop_caches` for code the VM runs. Entries hold shape ids,
slots and atoms only, never JS values, so they are not GC roots (the
rule of `shapes.aye`).

| site | entry | hit |
| --- | --- | --- |
| named load | up to 4 `(shape, holder, slot)` | receiver shape equal; own slot or holder slot read now |
| named store, existing | up to 4 `(shape, slot)` | shape equal, slot writable (flags read now) |
| named store, add | `(old shape, new shape, slot)` | shape equal; the object gets the new shape (constructors: `this.x = x`) |
| keyed load/store | element kind: dense array, typed array (kind), string, arguments; else a named-load entry for a constant key | class and bounds checked |
| global load/store | `(global shape, slot)`; let/const at the top level are resolved statically to `glex` slots and need no IC | global shape equal |
| call | target is a compiled function with `n` parameters | skips the callable and class checks |
| `instanceof` | the `prototype` value's object id | id equal |

States: uninitialized, then monomorphic, then polymorphic (2 to 4
entries), then **megamorphic**: a global stub cache, a hash table of
`(shape, atom) -> (holder, slot)` with 4096 entries (V8:
`src/ic/stub-cache.h`). A site that is megamorphic stays so.

### What `shapes.aye` must provide (P4.73, P4.74)

1. **Transitions in the IC**: `shapeTransition` already shares the
   `(shape, atom) -> shape` result. The store IC needs the new slot
   index and whether `Obj.props` must grow, so that an add is a check
   plus a write.
2. **The prototype in the shape.** Today a shape is only the own
   layout, and the cache checks the immediate prototype's shape and
   follows the current link. Deeper chains (class hierarchies,
   three.js's `Object3D` methods two levels up) miss. The change: each
   prototype object gets its own root shape (`rootShape(proto)`), so a
   shape fixes the prototype. `Object.setPrototypeOf` moves the object
   to a new root (rebuilding its layout) or to dictionary mode.
3. **A prototype epoch** instead of V8's validity cells
   (`src/objects/prototype-info.h`): objects used as prototypes are
   marked, and any change of their keys, flags or prototype increments
   one counter, `Heap.proto_epoch`. A load entry for a holder up the
   chain records the epoch, and a hit needs the receiver shape and the
   epoch to be equal. Pages change prototypes mostly while they load,
   so the counter settles. If measurements show churn, per-prototype
   cells replace the single counter.
4. **Dictionary objects** (after `delete`, more than 128 properties, or
   past the shape cap) miss every IC, as today.

Proxies, host objects with named properties (`HOST_NAMED`), string
wrappers, array `length` and accessors keep their present slow paths:
the IC records "slow" for such a receiver and calls the generic
`getProp`/`setProp`.

## 4. GC and values

### Roots and safepoints

- Registers and arguments are on `Interp.stack`, which the collector
  already marks. The extra frame fields hold ids (`code`) and an `Env`
  (`ctx`), which the collector marks with the frame's existing `env`.
- The collector runs only at allocation (`Interp.maybeGc`), as today.
  The safepoints are therefore the instructions with the **may
  allocate** flag (literals, closures, calls, string `Add`, contexts,
  property adds that grow `props`). The rule for the loop: **before an
  instruction that may allocate, the accumulator is written to its home
  register r0**. Numeric `Add` with two numbers, loads and stores to
  existing slots, compares and jumps keep it only in the local.
  `DWELLER_JS_GC=stress` checks the rule, as it checks the tree walker's
  rooting now.
- The constant pool holds atoms, numbers and ids, which are not
  collected. Template objects stay in `Interp.tagged`, which is already
  rooted.
- A suspended generator's registers are traced through its object.

### Allocation fast paths

`newObject` now goes through `maybeGc` and the free list. The VM keeps
that for P4.63 to P4.75. Two cheap gains are planned with the IC work:
`CreateObjectLiteral` for a cached literal shape reserves `props` once
(`literalShape` does this already for the tree walker), and `new F()`
whose constructor's store ICs all transition from the root shape
reserves the final slot count (V8's slack tracking, simplified). P4.21
(GC threshold factor) and P4.24 (smaller objects) stay separate cards;
after the VM is on by default, GC will be a larger share of the time,
so they come next.

### Value representation: no change now

`Value` stays the 16-byte enum (`Undefined`, `Null`, `Bool`, `Num` f64,
`Big`, `Str`, `Obj`, `Empty`).

- It is used everywhere: 26k lines of engine and the DOM bindings. A
  16-byte struct is passed in two registers on arm64 and x86-64, so
  passing values costs nothing extra.
- NaN-boxing (8 bytes) would halve the size of registers, arrays and
  property slots. It is a memory and cache gain, not a dispatch gain,
  and it would touch every file. It is reconsidered with P4.24 when
  memory profiles show the value size matters.
- No small-integer tag. On the M2, f64 add and multiply cost the same
  as integer ones. Index conversion is one compare (`numIndex`), and
  the bitwise paths already avoid libc (`PERF.md`). A tag is reconsidered
  only if profiles of `crypto` or `three` show conversions as hot.

## 5. Migration

### Two engines, one heap

The VM and the tree walker share `Interp`: the heap, the stack, the
frames, the natives and the host. `invoke` asks for the function's
bytecode (compiling it on the first call). With bytecode, it enters the
VM. Without bytecode (unsupported construct, or the VM off), it walks
the tree as now. Each engine calls the other through the existing
calling convention. The step from a tree-walked caller to a compiled
callee is one function call, so mixed code is correct from the first
card.

- `DWELLER_JS_VM=0|1`: off by default until P4.75, on by default after
  it. `0` stays as an escape hatch until P4.78.
- `DWELLER_JS_VM_STATS=1` prints at exit: functions compiled, functions
  left on the tree walker by reason (`unsupported: ClassExpr`, ...),
  and the instruction count. Each card's verify checks that its
  constructs leave the reasons list.
- The script top level and `evalScript` code compile like a function
  body (P4.67, with globals).
- The tree walker remains the reference: `js_conformance` runs every
  script both ways against the same Node goldens.

### Order of constructs

Each card moves a group of constructs from "unsupported" to compiled,
so each PR can ship on its own:

1. format, builder, disassembler (no execution)
2. resolver: register or context for each binding
3. straight-line code: literals, registers, operators, assignments,
   `return`
4. control flow: `if`, loops, labels, `switch`, `?:`, `&&`/`||`/`??`
5. calls and `new` (base constructors), `arguments`, default and rest
   parameters
6. closures, contexts, block scopes, per-iteration `let`, TDZ
7. properties, literals, globals, `delete`, `in`, `instanceof`,
   `typeof`; the script top level
8. `throw`, `try`/`catch`/`finally`
9. the rest of the syntax: destructuring, spread, `for-in`/`for-of`,
   templates, optional chaining, regexp literals
10. classes: methods, accessors, fields, private names, `super`
11. generators, then async functions and `for await`
12. ICs: polymorphic, megamorphic, store transitions, calls, elements
13. prototype shapes and the epoch
14. on by default
15. typed arrays over real bytes (three.js and WebGL)
16. register optimizer, fused instructions
17. remove the tree walker's execution paths, `fast_expr.aye` and
    `coroutine.aye`

### Gates for each step

- Every card: `js_conformance` and `js_conformance_vm`, the VM
  under `DWELLER_JS_GC=stress`, and the jsbench workloads it targets
  against `baseline.json` (checksums must match).
- From step 7 on: `jsdom_fixtures`, `jsdom_gc_stress`, and
  `page_load.py check` on the 22 recorded sites with the VM on.
- Steps 9 to 11 and 14: test262 with the VM on, no fewer passes than
  off (when P10.3 has landed).
- Step 14 (default on): all of the above, plus `cargo test --release
  --test browser` once on `main` after the merge (CI), and
  `DWELLER_JS_VM_STATS=1` reasons empty on the js suite and the 22
  sites.
- Step 17 removes code only after step 14 has been on `main` with no
  fallback reason in the js suite, test262 and the 22 sites.

## 6. Expected speedups (estimates, not measurements)

Relative to `baseline.json`. Each card measures and records the real
number in its status line. The plan's target is the suite within 1.5×
of `node --jitless` (now 5× to 11× slower).

| after | estimate |
| --- | --- |
| P4.63 + P4.64 (straight-line, control flow) | `loop` 3–5×; suite about 1.0–1.2× (few hot functions fully compile yet) |
| P4.65 to P4.67 (calls, closures, properties, globals) | `calls` 3–5×, `globals` 5–10×; richards, closures, navier-stokes 1.8–2.5× |
| P4.68 to P4.70 (exceptions, syntax, classes) | coverage, not speed: site scripts mostly compile; site `scripts` phase 1.3–2× |
| P4.71, P4.72 (resumable frames) | generator and async code 2–5×; no 256 MB coroutine stacks |
| P4.73, P4.74 (ICs, prototype shapes) | another 1.2–1.5× on richards, splay, raytrace, three |
| P4.75 (on by default) | suite geomean 3–4× against today; site `scripts` 1.5–2.5× |
| P4.76 (typed arrays over bytes) | typed-array loops 5–20×; `three` 1.5–3× |
| P4.77 (register optimizer, fused instructions) | another 1.1–1.25× |

## 7. V8 reference map

V8 is BSD-3. Porting an algorithm or a small piece is allowed with
attribution in the Aye file's header ("Ported from V8
`src/...`, Copyright the V8 project authors, BSD-3; see
`browser/js/LICENSE-V8`"). The first card that ports code adds
`browser/js/LICENSE-V8` with V8's license text. Paths are relative to
`~/Documents/chromium/src/v8/`, read with the Read and Grep tools.

| area | V8 path |
| --- | --- |
| instruction set | `src/interpreter/bytecodes.h`, `bytecode-operands.h` |
| compiler | `src/interpreter/bytecode-generator.cc`, `bytecode-array-builder.h`, `bytecode-register-allocator.h`, `bytecode-label.h` |
| loops, switch, try/finally | `src/interpreter/control-flow-builders.h`, `bytecode-generator.cc` (`ControlScope*`) |
| handler table | `src/interpreter/handler-table-builder.h` |
| source positions | `src/codegen/source-position-table.h` |
| instruction semantics | `src/interpreter/interpreter-generator.cc` (one handler per instruction) |
| register optimizer | `src/interpreter/bytecode-register-optimizer.h` |
| feedback and ICs | `src/objects/feedback-vector.h`, `src/ic/ic.h`, `src/ic/ic.cc`, `src/ic/stub-cache.h`, `src/ic/handler-configuration.h` |
| prototype validity | `src/objects/prototype-info.h` |
| generators | `src/objects/js-generator.h`, `bytecode-generator.cc` (`VisitYield`, `BuildSuspendPoint`) |
| typed arrays | `src/objects/js-array-buffer.h`, `src/builtins/builtins-typed-array.cc` |
| number formatting and parsing | `src/base/numbers/fast-dtoa.h`, `bignum-dtoa.h`, `strtod.cc`, `src/numbers/conversions.cc` |
| Date parsing | `src/date/dateparser.h`, `dateparser-inl.h` |
| regexp | `src/regexp/regexp-bytecode-generator.h`, `regexp-interpreter.h` |
| string search | `src/strings/string-search.h` |

## 8. Out of scope

- A JIT (baseline or optimizing). If the VM reaches its target and
  games still need more, a baseline compiler is a new design.
- WebAssembly. The user's game needs it for its physics. It needs its
  own design and is noted in the plan's backlog.
- Changing the parser or the AST. The compiler reads the present AST
  and the resolver's output.
