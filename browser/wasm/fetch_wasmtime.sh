#!/bin/sh
# Downloads wasmtime's C API (the release and SHA-256 that
# browser/wasm/wasmtime.lock pins) into target/wasmtime/<version>/ of
# this checkout: lib/ (libwasmtime.so or .dylib) and
# lib/pkgconfig/wasmtime.pc. The .pc file names the library directory
# with an rpath (per OS below), so a browser built with
# PKG_CONFIG_PATH=target/wasmtime/<version>/lib/pkgconfig finds
# libwasmtime at run time; tests/browser.rs and browser/build.sh set
# that path when the directory exists. The headers are vendored in
# browser/wasm/include. Without the library the browser still builds and
# `WebAssembly` is undefined (docs/webgl.md §6).
#
#     browser/wasm/fetch_wasmtime.sh        prints the pkgconfig directory
set -eu
cd "$(dirname "$0")/../.."
lock=browser/wasm/wasmtime.lock
version=$(sed -n 's/^version //p' "$lock")
case "$(uname -s)-$(uname -m)" in
    Linux-x86_64) platform=x86_64-linux ;;
    Darwin-arm64) platform=aarch64-macos ;;
    *) echo "fetch_wasmtime: no wasmtime C API release for $(uname -s) $(uname -m)" >&2; exit 1 ;;
esac
asset="wasmtime-v$version-$platform-c-api.tar.xz"
sum=$(grep "  $asset\$" "$lock" | cut -d' ' -f1)
if [ -z "$sum" ]; then
    echo "fetch_wasmtime: $asset is not in $lock" >&2
    exit 1
fi
dir="target/wasmtime/$version"
lib="$PWD/$dir/lib"
if [ -f "$dir/$asset.ok" ] && [ -f "$lib/pkgconfig/wasmtime.pc" ]; then
    echo "$lib/pkgconfig"
    exit 0
fi
mkdir -p "$dir"
tmp="$dir/$asset.part"
curl -fsSL --retry 3 -o "$tmp" "https://github.com/bytecodealliance/wasmtime/releases/download/v$version/$asset"
if command -v sha256sum >/dev/null 2>&1; then
    got=$(sha256sum "$tmp" | cut -d' ' -f1)
else
    got=$(shasum -a 256 "$tmp" | cut -d' ' -f1)
fi
if [ "$got" != "$sum" ]; then
    rm -f "$tmp"
    echo "fetch_wasmtime: $asset has SHA-256 $got, expected $sum" >&2
    exit 1
fi
rm -rf "$lib" "$dir/x"
mkdir -p "$dir/x" "$lib/pkgconfig"
tar -xJf "$tmp" -C "$dir/x"
rm -f "$tmp"
src="$dir/x/wasmtime-v$version-$platform-c-api"
cp "$src/LICENSE" "$dir/LICENSE"
case "$platform" in
    *-linux)
        cp "$src/lib/libwasmtime.so" "$lib/"
        libs="-L$lib -lwasmtime -Wl,-rpath,$lib -lpthread -ldl -lm"
        ;;
    *-macos)
        # The dylib's install name is @rpath/libwasmtime.dylib.
        cp "$src/lib/libwasmtime.dylib" "$lib/"
        libs="-L$lib -lwasmtime -Wl,-rpath,$lib"
        ;;
esac
rm -rf "$dir/x"
cat > "$lib/pkgconfig/wasmtime.pc" <<EOF
Name: wasmtime
Description: wasmtime $version C API (browser/wasm/fetch_wasmtime.sh)
Version: $version
Cflags:
Libs: $libs
EOF
touch "$dir/$asset.ok"
echo "$lib/pkgconfig"
