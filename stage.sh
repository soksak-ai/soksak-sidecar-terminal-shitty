#!/bin/sh
set -eu
dist=${1:-dist}
: "${SOKSAK_SHITTY_VT_SDK:?SOKSAK_SHITTY_VT_SDK is required}"
cargo build --release --bin soksak-sidecar-terminal-shitty
mkdir -p "$dist"
binary="${CARGO_TARGET_DIR:-target}/release/soksak-sidecar-terminal-shitty"
cp "$binary" "$dist/.soksak-sidecar-terminal-shitty.tmp"
chmod +x "$dist/.soksak-sidecar-terminal-shitty.tmp"
mv -f "$dist/.soksak-sidecar-terminal-shitty.tmp" "$dist/soksak-sidecar-terminal-shitty"
