#!/bin/sh
# Installs what `aye check browser`, `aye test browser` and `cargo test
# --release --test browser` need on Ubuntu 24.04, besides Rust and the
# pinned aye (see README.md). The apt line is the Aye repo's
# scripts/setup-ubuntu.sh one, plus HarfBuzz and brotli (optional to the
# browser, installed so their code paths are built and tested).
#
#     sh scripts/setup-ubuntu.sh

set -eu

if [ "$(id -u)" -eq 0 ]; then
    SUDO=
else
    SUDO=sudo
fi

export DEBIAN_FRONTEND=noninteractive
$SUDO apt-get update
$SUDO apt-get install -y --no-install-recommends \
    ca-certificates curl git gnupg build-essential pkg-config \
    clang lld libclang-dev libclang-rt-18-dev \
    libcurl4-openssl-dev libfreetype-dev libsdl2-dev \
    fonts-dejavu-core python3 apache2-utils golang-go \
    libharfbuzz-dev libbrotli-dev
echo "setup-ubuntu: done"
