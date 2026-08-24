#!/bin/sh
set -eu
[ "$#" -eq 2 ] && [ -n "$1" ] && [ -n "$2" ] || { echo 'usage: stage-built.sh <out> <target>' >&2; exit 2; }
out=$1
target=$2
repository=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
# An absolute candidate output is allowed only outside the source repository.
case "$out" in ''|/|.|*..*|"$repository"|"$repository"/*) echo 'stage output is unsafe or inside the source repository' >&2; exit 2 ;; esac
binary=target/$target/release/soksak-sidecar-terminal-shitty
[ -f "$binary" ] || { echo "release binary is missing: $binary" >&2; exit 1; }
mkdir -p "$out"
[ ! -L "$out" ] || { echo 'stage output must not be a symbolic link' >&2; exit 2; }
temporary=$out/.soksak-sidecar-terminal-shitty.next.$$
trap 'rm -f "$temporary" "$out/.sidecar.json.next.$$"' EXIT HUP INT TERM
cp "$binary" "$temporary"
chmod +x "$temporary"
if [ -e "$out/soksak-sidecar-terminal-shitty" ]; then
  cmp -s "$temporary" "$out/soksak-sidecar-terminal-shitty" || { echo 'staged binary conflicts with current build' >&2; exit 1; }
  rm -f "$temporary"
else
  mv "$temporary" "$out/soksak-sidecar-terminal-shitty"
fi
manifest=$out/.sidecar.json.next.$$
cp sidecar.json "$manifest"
if [ -e "$out/sidecar.json" ]; then
  cmp -s "$manifest" "$out/sidecar.json" || { echo 'staged manifest conflicts with source' >&2; exit 1; }
  rm -f "$manifest"
else
  mv "$manifest" "$out/sidecar.json"
fi
echo "SHITTY_STAGED target=$target output=$out/soksak-sidecar-terminal-shitty"
