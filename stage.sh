#!/bin/sh
set -eu
dist=${1:-dist}
target=${2:-}
: "${SOKSAK_SHITTY_VT_SDK:?SOKSAK_SHITTY_VT_SDK is required}"
target_args=""
release_dir=release
if [ -n "$target" ]; then
  target_args="--target $target"
  release_dir="$target/release"
fi
cargo build --release $target_args --bin soksak-sidecar-terminal-shitty
mkdir -p "$dist"
binary="${CARGO_TARGET_DIR:-target}/$release_dir/soksak-sidecar-terminal-shitty"
cp "$binary" "$dist/.soksak-sidecar-terminal-shitty.tmp"
chmod +x "$dist/.soksak-sidecar-terminal-shitty.tmp"
mv -f "$dist/.soksak-sidecar-terminal-shitty.tmp" "$dist/soksak-sidecar-terminal-shitty"
