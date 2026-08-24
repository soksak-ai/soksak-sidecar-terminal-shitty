#!/bin/sh
set -eu
[ "$#" -eq 2 ] && [ -n "$1" ] && [ -n "$2" ] || { echo 'usage: stage-built.sh <out> <target>' >&2; exit 2; }
out=$1
target=$2
case "$out" in /*|*..*) echo 'stage output must be repository-relative' >&2; exit 2 ;; esac
binary=target/$target/release/soksak-sidecar-terminal-shitty
[ -f "$binary" ] || { echo "release binary is missing: $binary" >&2; exit 1; }
mkdir -p "$out"
temporary=$out/.soksak-sidecar-terminal-shitty.next.$$
trap 'rm -f "$temporary"' EXIT HUP INT TERM
cp "$binary" "$temporary"
chmod +x "$temporary"
if [ -e "$out/soksak-sidecar-terminal-shitty" ]; then
  cmp -s "$temporary" "$out/soksak-sidecar-terminal-shitty" || { echo 'staged binary conflicts with current build' >&2; exit 1; }
  rm -f "$temporary"
else
  mv "$temporary" "$out/soksak-sidecar-terminal-shitty"
fi
echo "SHITTY_STAGED target=$target output=$out/soksak-sidecar-terminal-shitty"
