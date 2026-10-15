#!/bin/sh
# Fast-mode end-to-end wall time (startup, parse, execution, teardown).
# Builds target/dweller unless BENCH_BIN names a binary. The work is done by
# browser/tools/jsbench.py (see its header for the options and variables):
#
#     browser/tests/jsbench/run.sh [--json FILE] [--smoke] [WORKLOAD...]
set -eu
cd "$(dirname "$0")/../../.."
if [ -z "${BENCH_BIN:-}" ]; then
    browser/build.sh
fi
exec python3 browser/tools/jsbench.py "$@"
