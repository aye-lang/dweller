#!/usr/bin/env bash
# Downloads this machine's ANGLE archive from the `angle-<rev>` release,
# checks its SHA-256 against browser/gpu/angle.lock and unpacks it into
# target/angle/<rev>/ (docs/webgl.md §1). Does nothing when that
# directory already holds the pinned revision.
set -euo pipefail

ROOT=$(cd "$(dirname "$0")/../.." && pwd)
REV=$(tr -d ' \n' < "$ROOT/browser/gpu/angle.rev")
REPO=${DWELLER_REPO:-aye-lang/dweller}
DEST=$ROOT/target/angle/$REV

case "$(uname -s)" in
  Darwin) OS=macos ;;
  Linux) OS=linux ;;
  *) echo "fetch_angle.sh: unsupported OS $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  arm64|aarch64) ARCH=arm64 ;;
  x86_64) ARCH=x64 ;;
  *) echo "fetch_angle.sh: unsupported arch $(uname -m)" >&2; exit 1 ;;
esac
ASSET=angle-$REV-$OS-$ARCH.tar.zst

if [ "$(cat "$DEST/REVISION" 2>/dev/null || true)" = "$REV" ]; then
  echo "ANGLE $REV already in $DEST"
  exit 0
fi

SUM=$(awk -v a="$ASSET" '$2 == a { print $1 }' "$ROOT/browser/gpu/angle.lock")
if [ -z "$SUM" ]; then
  echo "fetch_angle.sh: no hash for $ASSET in browser/gpu/angle.lock" >&2
  exit 1
fi

TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT
curl -fsSL --retry 3 -o "$TMP/$ASSET" \
  "https://github.com/$REPO/releases/download/angle-$REV/$ASSET"
(cd "$TMP" && echo "$SUM  $ASSET" | shasum -a 256 -c -)
rm -rf "$DEST"
mkdir -p "$DEST"
zstd -dc "$TMP/$ASSET" | tar -xf - -C "$DEST"
echo "ANGLE $REV unpacked into $DEST"
