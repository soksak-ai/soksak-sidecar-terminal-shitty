#!/bin/sh
set -eu
dist=${1:?dist directory is required}
target=${2:?target is required}
out=${3:?archive path is required}
package=$(mktemp -d "${TMPDIR:-/tmp}/soksak-shitty-release.XXXXXX")
trap 'rm -rf -- "$package"' EXIT HUP INT TERM
mkdir -p "$package/dist"
cp sidecar.json "$package/sidecar.json"
cp LICENSE LICENSE.GPL3 LICENSE.MIT "$package/"
cp "$dist/soksak-sidecar-terminal-shitty" "$package/dist/"
find "$package" -type l -exec false {} +
tar -czf "$out" -C "$package" LICENSE THIRD-PARTY-NOTICES sidecar.json dist
contents=$(tar -tzf "$out" | LC_ALL=C sort)
expected='./
./LICENSE
./LICENSE.GPL3
./LICENSE.MIT
./dist/
./dist/soksak-sidecar-terminal-shitty
./sidecar.json'
test "$contents" = "$expected"
printf 'packaged %s for %s\n' "$out" "$target"
