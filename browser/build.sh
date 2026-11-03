#!/bin/sh
# Builds the Dweller browser for real use: `--mode safe` (optimized, with
# the bounds and overflow checks, so a panic names the line). The output
# goes to target/dweller in this checkout (your worktree) unless a path is
# given; a relative path is taken from the checkout's root. Do not give a
# path outside your worktree or a fixed name that other agents share (the
# session scratchpad): another agent's build can replace your binary
# before your tests run.
#
#     browser/build.sh [OUT] [--mode safe|fast|debug]
#
# In the Aye repository it builds the compiler first and uses
# target/release/aye. Anywhere else (the Dweller repository) it uses the
# `aye` named by $AYE, or `aye` on the PATH.
#
# A safe or fast build also writes `OUT.safe` or `OUT.fast` (the binary's
# sha256): the timing tools (browser/tools/fast_build.py) refuse a binary
# without one.
#
# `--mode fast` drops the checks: Q0.2 measured the safe build's longest
# loop 9.5% and its jsbench time 17.6% over fast (browser/engine/PERF.md).
set -e
cd "$(dirname "$0")/.."
out=target/dweller
mode=safe
while [ $# -gt 0 ]; do
    case "$1" in
        --mode) mode="$2"; shift 2 ;;
        *) out="$1"; shift ;;
    esac
done
if [ -z "${AYE:-}" ]; then
    if grep -qs '^name = "aye"$' Cargo.toml; then
        cargo build --release --quiet
        AYE=target/release/aye
    else
        AYE=aye
    fi
fi
# libwasmtime, when browser/wasm/fetch_wasmtime.sh fetched it (P11.17).
wasmtime_pc="target/wasmtime/$(sed -n 's/^version //p' browser/wasm/wasmtime.lock)/lib/pkgconfig"
if [ -f "$wasmtime_pc/wasmtime.pc" ]; then
    PKG_CONFIG_PATH="$PWD/$wasmtime_pc${PKG_CONFIG_PATH:+:$PKG_CONFIG_PATH}"
    export PKG_CONFIG_PATH
fi
mkdir -p "$(dirname "$out")"
rm -f "$out.fast" "$out.safe"
"$AYE" build browser -o "$out" --mode "$mode"
if [ "$mode" != debug ]; then
    shasum -a 256 "$out" | cut -d' ' -f1 > "$out.$mode"
fi
echo "$out ($mode)"
