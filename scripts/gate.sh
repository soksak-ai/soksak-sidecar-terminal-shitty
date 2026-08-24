#!/bin/sh
set -eu
[ "$#" -eq 1 ] && [ -n "$1" ] || { echo 'usage: gate.sh <target>' >&2; exit 2; }
target=$1
cargo test --locked --release --target "$target"
cargo test --locked --release --target "$target" --test bench -- --ignored --nocapture
echo "SHITTY_GATE_PASS target=$target"
