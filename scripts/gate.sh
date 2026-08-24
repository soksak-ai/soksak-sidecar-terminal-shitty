#!/bin/sh
set -eu
[ "$#" -eq 2 ] && [ -n "$1" ] && [ -n "$2" ] || { echo 'usage: gate.sh <target> <stage-out>' >&2; exit 2; }
target=$1
case "$2" in /*) stage_out=$2 ;; *) stage_out=$PWD/$2 ;; esac
[ -d "$stage_out" ] || { echo "stage output is missing: $stage_out" >&2; exit 1; }
SOKSAK_STAGE_OUT="$stage_out" cargo test --locked --release --target "$target"
echo "SHITTY_GATE_PASS target=$target"
