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
mkdir -p "$out/dist"
[ ! -L "$out" ] || { echo 'stage output must not be a symbolic link' >&2; exit 2; }
[ ! -L "$out/dist" ] || { echo 'stage process directory must not be a symbolic link' >&2; exit 2; }
staged_binary=$out/dist/soksak-sidecar-terminal-shitty
staged_manifest=$out/sidecar.json
staged_process_manifest=$out/dist/sidecar.json
next_binary=$out/dist/.soksak-sidecar-terminal-shitty.next.$$
next_manifest=$out/.sidecar.json.next.$$
next_process_manifest=$out/dist/.sidecar.json.next.$$
trap 'rm -f "$next_binary" "$next_manifest" "$next_process_manifest"' EXIT HUP INT TERM
cp "$binary" "$next_binary"
chmod +x "$next_binary"
cp sidecar.json "$next_manifest"
cp sidecar.json "$next_process_manifest"
for path in "$staged_binary" "$staged_manifest" "$staged_process_manifest"; do
  [ ! -L "$path" ] || { echo "STAGED_STATE_INVALID: symbolic link: $path" >&2; exit 1; }
done

identity() {
  node -e 'const {readFileSync}=require("node:fs");const v=JSON.parse(readFileSync(process.argv[1],"utf8"));if(typeof v.id!=="string"||typeof v.version!=="string")process.exit(1);process.stdout.write(v.id+"\n"+v.version)' "$1"
}
next_identity=$(identity "$next_manifest") || { echo 'STAGED_STATE_INVALID: source manifest identity' >&2; exit 1; }
next_id=$(printf '%s\n' "$next_identity" | sed -n '1p')
next_version=$(printf '%s\n' "$next_identity" | sed -n '2p')

if [ -f "$staged_manifest" ]; then
  current_identity=$(identity "$staged_manifest") || { echo 'STAGED_STATE_INVALID: staged manifest identity' >&2; exit 1; }
  current_id=$(printf '%s\n' "$current_identity" | sed -n '1p')
  current_version=$(printf '%s\n' "$current_identity" | sed -n '2p')
  [ "$current_id" = "$next_id" ] || { echo 'STAGED_STATE_INVALID: component identity changed' >&2; exit 1; }
  if cmp -s "$next_manifest" "$staged_manifest"; then
    [ ! -e "$staged_binary" ] || cmp -s "$next_binary" "$staged_binary" || { echo "STAGED_BUILD_NOT_DETERMINISTIC: $next_id@$next_version" >&2; exit 1; }
    [ ! -e "$staged_process_manifest" ] || cmp -s "$next_process_manifest" "$staged_process_manifest" || { echo "STAGED_STATE_INVALID: process manifest differs: $staged_process_manifest" >&2; exit 1; }
    if [ -f "$staged_binary" ] && [ -f "$staged_process_manifest" ]; then
      echo "SHITTY_STAGED_UNCHANGED target=$target output=$staged_binary"
      exit 0
    fi
  else
    [ "$current_version" != "$next_version" ] || { echo "STAGED_MANIFEST_CONFLICT: $next_id@$next_version" >&2; exit 1; }
  fi
elif [ -e "$staged_manifest" ]; then
  echo "STAGED_STATE_INVALID: manifest is not a regular file: $staged_manifest" >&2
  exit 1
elif [ -e "$staged_binary" ] && ! cmp -s "$next_binary" "$staged_binary"; then
  echo "STAGED_STATE_INVALID: binary has no matching manifest: $staged_binary" >&2
  exit 1
elif [ -e "$staged_process_manifest" ] && ! cmp -s "$next_process_manifest" "$staged_process_manifest"; then
  echo "STAGED_STATE_INVALID: process manifest has no matching root manifest: $staged_process_manifest" >&2
  exit 1
fi

# Root manifest last. If interrupted, the old root manifest still authorizes replacement; an
# initial partial stage is completed when its binary and process manifest equal the new inputs.
mv "$next_binary" "$staged_binary"
mv "$next_process_manifest" "$staged_process_manifest"
mv "$next_manifest" "$staged_manifest"
echo "SHITTY_STAGED target=$target output=$staged_binary"
