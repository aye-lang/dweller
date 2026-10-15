# Standalone JS benchmarks

Run `browser/tests/jsbench/run.sh` from any directory (requires Python 3).
It builds
`target/dweller` in fast mode, verifies each stdout checksum (also with
Node when installed), and reports the median of three end-to-end wall
times in seconds. Startup, parsing, execution, and teardown are included.
Use `BENCH_RUNS=5` for more samples, `BENCH_BIN=/path/to/saved/binary`
to compare builds without rebuilding, and arguments to select workloads
(e.g. `run.sh richards methods`). Set `BENCH_BASELINE=/path/to/old/binary`
to interleave both builds, alternating their order each round, and print
per-workload speedups and their geometric mean. Both binaries must have
been built in fast mode. A mismatch or timeout is a failure.

Richards, NavierStokes, Splay and Crypto are standalone ports from
https://github.com/chromium/octane (the algorithms are unchanged). Their
source files retain the upstream license notices. The Octane harness is
removed, assertions retained, and deterministic checksums/iteration counts
added (`Math.random` is Octane's deterministic replacement, so the
checksums equal Node's). NavierStokes runs the original 128×128,
20-iteration solver for 15 frames and checks 77. Raytrace, loop, globals
and calls are written for this suite, as are the other five. Octane's
`deltablue.js` (GPL-2) and `raytrace.js` (no license statement) are not
vendored; see `browser/js/BYTECODE.md` section 1.

Licenses:

| file | license |
| --- | --- |
| `richards.js`, `splay.js` | BSD-3-Clause (Copyright 2006-2008 / 2009 the V8 project authors; notice in the file header) |
| `navier-stokes.js` | MIT (Copyright 2013 the V8 project authors, 2009 Oliver Hunt; notice in the file header) |
| `crypto.js` | Tom Wu's license (Copyright 2003-2005 Tom Wu; permission notice and disclaimer kept in the file header) |
| all other files | the repository license (MIT OR Apache-2.0) |

## Running

`browser/tests/jsbench/run.sh` (builds, then `python3 browser/tools/jsbench.py`;
use the script directly with `BENCH_BIN=target/dweller` to skip the build).
When `node` is installed the table gets a `node --jitless` column (V8 with
only the Ignition interpreter) and the ratio; `BENCH_REF=none` turns it off.
`--json FILE` writes the medians and the machine description.

`--smoke` runs every workload once at a tenth of its size (the script is
prefixed with `var SMOKE = 10;`) and compares the checksum with
`NAME.smoke.out`; `cargo test --release --test browser -- jsbench` runs it
with the test build. `--update-smoke` regenerates the goldens from Node.
Every VM card quotes its speedups against `baseline.json`:
`BENCH_BASELINE=<main's binary> BENCH_RUNS=5 browser/tests/jsbench/run.sh`.

## Baseline (`baseline.json`)

Intel(R) Xeon(R) Processor @ 2.10GHz, 4 cores, 16093 MiB, Linux 6.18.44-fc-v114 x86_64; fast build of `f888c35` (main with this card's files); median of 5 runs; Node v22.22.0 --jitless.

| workload | dweller (s) | node --jitless (s) | ratio | checksum |
| --- | ---: | ---: | ---: | ---: |
| array | 0.230 | 0.106 | 2.2× | 96751118 |
| calls | 4.604 | 0.266 | 17.3× | 66000000 |
| closures | 3.482 | 0.397 | 8.8× | 306584040 |
| crypto | 3.724 | 0.563 | 6.6× | 9359492 |
| globals | 2.873 | 0.117 | 24.6× | 10500000 |
| json | 0.716 | 0.337 | 2.1× | 20672500 |
| loop | 2.149 | 0.228 | 9.4× | 35000000 |
| methods | 0.476 | 0.082 | 5.8× | 267460185993 |
| navier-stokes | 16.298 | 1.744 | 9.3× | 77 |
| raytrace | 2.130 | 0.320 | 6.6× | 999554985 |
| richards | 1.962 | 0.202 | 9.7× | 2916400 |
| splay | 3.913 | 0.499 | 7.8× | 729063963 |
| strings | 0.150 | 0.081 | 1.9× | 650400 |

These are whole-process measurements, not Octane scores. The machine is
shared with other agents; use repeated/interleaved runs for comparisons.
See `browser/js/PERF.md` for the final comparison and profiling.
