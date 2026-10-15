#!/usr/bin/env bash
# Builds ANGLE standalone at the revision in browser/gpu/angle.rev
# (docs/webgl.md §1) and packages libEGL, libGLESv2, the SwiftShader
# Vulkan ICD, the headers and the license files.
#
#   browser/gpu/build_angle.sh [sync|gen|build|package|all]
#
# Needs depot_tools on PATH. Environment:
#   ANGLE_SRC   checkout (default ~/.cache/dweller/angle-src)
#   ANGLE_DEST  output dir (default target/angle/<rev>)
#   ANGLE_JOBS  ninja -j (default: autoninja's choice)
#
# `package` also writes target/angle/angle-<rev>-<os>-<arch>.tar.zst.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
REV=$(tr -d ' \n' < "$ROOT/browser/gpu/angle.rev")
SRC=${ANGLE_SRC:-$HOME/.cache/dweller/angle-src}
OUT=$SRC/out/dweller
DEST=${ANGLE_DEST:-$ROOT/target/angle/$REV}
URL=https://chromium.googlesource.com/angle/angle.git
STEP=${1:-all}

case "$(uname -s)" in
  Darwin) OS=macos; SO=dylib ;;
  Linux) OS=linux; SO=so ;;
  *) echo "build_angle.sh: unsupported OS $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  arm64|aarch64) ARCH=arm64 ;;
  x86_64) ARCH=x64 ;;
  *) echo "build_angle.sh: unsupported arch $(uname -m)" >&2; exit 1 ;;
esac
ASSET=angle-$REV-$OS-$ARCH.tar.zst

# Leave the caller's depot_tools checkout as it is (CI sets 1 so that a
# fresh clone bootstraps itself).
export DEPOT_TOOLS_UPDATE=${DEPOT_TOOLS_UPDATE:-0}

sync() {
  # A shallow checkout of the pinned commit plus the `.gclient` that
  # `fetch angle` writes; `--no-history` keeps the deps shallow too.
  # The OpenCL (LLVM) and Dawn deps are not needed for libEGL/libGLESv2.
  mkdir -p "$SRC"
  if [ ! -d "$SRC/.git" ]; then
    git -C "$SRC" init -q
    git -C "$SRC" remote add origin "$URL"
  fi
  if [ "$(git -C "$SRC" rev-parse HEAD 2>/dev/null || true)" != "$REV" ]; then
    git -C "$SRC" fetch -q --depth 1 origin "$REV"
    git -C "$SRC" checkout -q --detach FETCH_HEAD
  fi
  cat > "$SRC/.gclient" <<EOF
solutions = [
  {
    "name": ".",
    "url": "$URL",
    "deps_file": "DEPS",
    "managed": False,
    "custom_vars": {
      "checkout_angle_cl_deps": False,
      "checkout_angle_dawn_deps": False,
    },
  },
]
EOF
  (cd "$SRC" && gclient sync -D --no-history)
}

gen() {
  local gl=true metal=false
  if [ "$OS" = macos ]; then gl=false; metal=true; fi
  mkdir -p "$OUT"
  cat > "$OUT/args.gn" <<EOF
is_debug = false
is_component_build = false
angle_build_all = false
angle_build_tests = false
angle_enable_null = true
angle_enable_swiftshader = true
angle_enable_vulkan = true
angle_enable_metal = $metal
angle_enable_gl = $gl
angle_has_frame_capture = false
EOF
  (cd "$SRC" && gn gen "$OUT")
}

build() {
  local jobs=()
  if [ -n "${ANGLE_JOBS:-}" ]; then jobs=(-j "$ANGLE_JOBS"); fi
  (cd "$SRC" && autoninja ${jobs[@]+"${jobs[@]}"} -C "$OUT" libEGL libGLESv2 swiftshader_libvulkan icd_file)
}

# Third-party code in the built libraries, as name:path-in-checkout.
LICENSES=(
  angle:LICENSE
  swiftshader:third_party/SwiftShader/LICENSE.txt
  # SwiftShader's JIT is LLVM 10, whose trimmed copy has no LICENSE file;
  # libc++'s is the same LLVM Project license (Apache-2.0 with LLVM
  # exceptions).
  llvm:third_party/libc++/src/LICENSE.TXT
  subzero:third_party/SwiftShader/third_party/subzero/LICENSE.TXT
  marl:third_party/SwiftShader/third_party/marl/LICENSE
  astc-encoder:third_party/astc-encoder/src/LICENSE.txt
  spirv-tools:third_party/spirv-tools/src/LICENSE
  spirv-headers:third_party/spirv-headers/src/LICENSE
  glslang:third_party/glslang/src/LICENSE.txt
  vulkan-headers:third_party/vulkan-headers/src/LICENSE.md
  vulkan-loader:third_party/vulkan-loader/src/LICENSE.txt
  vulkan-utility-libraries:third_party/vulkan-utility-libraries/src/LICENSE.md
  vulkan_memory_allocator:third_party/vulkan_memory_allocator/LICENSE.txt
  zlib:third_party/zlib/LICENSE
  abseil-cpp:third_party/abseil-cpp/LICENSE
  libc++:third_party/libc++/src/LICENSE.TXT
  libc++abi:third_party/libc++abi/src/LICENSE.TXT
)

package() {
  rm -rf "$DEST"
  mkdir -p "$DEST/lib" "$DEST/include" "$DEST/licenses"
  local f libs=(libEGL.$SO libGLESv2.$SO libvk_swiftshader.$SO vk_swiftshader_icd.json)
  # On Linux libGLESv2 loads the Vulkan loader it was built with
  # (angle_shared_libvulkan); on macOS the loader is linked in.
  if [ "$OS" = linux ]; then libs+=(libvulkan.so.1); fi
  for f in "${libs[@]}"; do
    cp "$OUT/$f" "$DEST/lib/"
  done
  if [ "$OS" = macos ]; then
    # ANGLE's install names are ./libEGL.dylib and ./libGLESv2.dylib,
    # which resolve against the working directory; make them @rpath and
    # re-sign (ad hoc), as editing the header breaks the signature.
    for f in libEGL libGLESv2; do
      install_name_tool -id "@rpath/$f.dylib" "$DEST/lib/$f.dylib"
      codesign -f -s - "$DEST/lib/$f.dylib"
    done
  fi
  for f in EGL GLES2 GLES3 KHR; do
    cp -R "$SRC/include/$f" "$DEST/include/"
  done
  local entry name path
  for entry in "${LICENSES[@]}"; do
    name=${entry%%:*}; path=${entry#*:}
    if [ -f "$SRC/$path" ]; then
      mkdir -p "$DEST/licenses/$name"
      cp "$SRC/$path" "$DEST/licenses/$name/"
    else
      echo "build_angle.sh: no license at $path" >&2
      exit 1
    fi
  done
  echo "$REV" > "$DEST/REVISION"
  (cd "$DEST" && tar -cf - REVISION lib include licenses) \
    | zstd -q -19 -T0 -f -o "$ROOT/target/angle/$ASSET"
  (cd "$ROOT/target/angle" && shasum -a 256 "$ASSET")
}

case "$STEP" in
  sync) sync ;;
  gen) gen ;;
  build) build ;;
  package) package ;;
  all) sync; gen; build; package ;;
  *) echo "usage: $0 [sync|gen|build|package|all]" >&2; exit 2 ;;
esac
