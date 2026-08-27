#!/bin/sh
set -eu

[ "$#" -eq 2 ] && [ -n "$1" ] && [ -n "$2" ] || { echo 'usage: prepare-shitty-sdk.sh <target> <build-root>' >&2; exit 2; }
target=$1
build_root=$2
case "$build_root" in /*|*..*) echo 'build root must be repository-relative' >&2; exit 2 ;; esac
root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
build_root=$root/$build_root
receipt=$build_root/receipts/$target.json
if [ -f "$receipt" ]; then
  soksak-validate build-receipt "$receipt" --dependencies "$root/build-dependencies.json" --output-root "$build_root"
  echo "SHITTY_SDK_REUSED target=$target"
  exit 0
fi
if [ -e "$build_root/targets/$target" ]; then
  mkdir -p "$build_root/receipts" "$build_root/.transactions"
  recovered=$build_root/.transactions/recover.$target.$$.json
  trap 'rm -f -- "$recovered"' EXIT HUP INT TERM
  soksak-validate build-receipt-create "$root/build-dependencies.json" --dependency shitty-vt-sdk \
    --target "$target" --output-root "$build_root" --out "$recovered"
  soksak-validate build-receipt "$recovered" --dependencies "$root/build-dependencies.json" --output-root "$build_root"
  if [ -e "$receipt" ]; then
    soksak-validate build-receipt "$receipt" --dependencies "$root/build-dependencies.json" --output-root "$build_root"
    echo "SHITTY_SDK_REUSED target=$target"
    exit 0
  fi
  mv "$recovered" "$receipt"
  trap - EXIT HUP INT TERM
  soksak-validate build-receipt "$receipt" --dependencies "$root/build-dependencies.json" --output-root "$build_root"
  echo "SHITTY_SDK_RECOVERED target=$target"
  exit 0
fi

mkdir -p "$build_root/sources" "$build_root/.transactions"
transaction=$build_root/.transactions/prepare.$target.$$
source_next=$build_root/sources/.next.$$
stage=$build_root/builds/$target
cleanup() {
  for candidate in "$transaction" "$source_next"; do
    case "$candidate" in "$build_root"/.transactions/*|"$build_root"/sources/.next.*) rm -rf -- "$candidate" ;; esac
  done
}
trap cleanup EXIT HUP INT TERM
mkdir -p "$transaction/targets/$target/shitty-vt-sdk"
resolution=$transaction/resolution.json
soksak-validate build-dependencies "$root/build-dependencies.json" --dependency shitty-vt-sdk --target "$target" > "$resolution"
repository=$(node -e 'const v=require(process.argv[1]);process.stdout.write(v.repository)' "$resolution")
commit=$(node -e 'const v=require(process.argv[1]);process.stdout.write(v.commit)' "$resolution")
python_version=$(node -e 'const v=require(process.argv[1]);process.stdout.write(v.tools.python)' "$resolution")
llvm_version=$(node -e 'const v=require(process.argv[1]);process.stdout.write(v.tools.llvm)' "$resolution")
ragel_version=$(node -e 'const v=require(process.argv[1]);process.stdout.write(v.tools.ragel)' "$resolution")
source=$build_root/sources/$commit
if [ -e "$source" ]; then
  [ -d "$source/.git" ] && [ "$(git -C "$source" remote get-url origin)" = "$repository" ] && \
    [ "$(git -C "$source" rev-parse HEAD)" = "$commit" ] && [ -z "$(git -C "$source" status --porcelain)" ] || {
      echo "cached Shitty source differs from build-dependencies.json" >&2; exit 79;
    }
else
  git init -q "$source_next"
  git -C "$source_next" remote add origin "$repository"
  git -C "$source_next" fetch -q --depth 1 origin "$commit"
  git -C "$source_next" -c advice.detachedHead=false checkout -q FETCH_HEAD
  [ "$(git -C "$source_next" rev-parse HEAD)" = "$commit" ] && [ -z "$(git -C "$source_next" status --porcelain)" ] || {
    echo "Shitty source checkout did not materialize the declared commit" >&2; exit 79;
  }
  mv "$source_next" "$source"
fi

if [ -e "$stage" ]; then
  [ -d "$stage" ] && [ -f "$stage/.soksak-build-resolution.json" ] && \
    cmp -s "$resolution" "$stage/.soksak-build-resolution.json" || { echo "Shitty build cache differs from declared inputs" >&2; exit 79; }
else
  mkdir -p "$stage"
  git -C "$source" archive --format=tar "$commit" | tar -xf - -C "$stage"
  cp "$resolution" "$stage/.soksak-build-resolution.json"
fi

source_date_epoch=$(git -C "$source" show -s --format=%ct "$commit")
compiler=$(command -v clang++)
tool_root=$(dirname -- "$compiler")
cc=$tool_root/clang
archiver=$tool_root/llvm-ar
[ -x "$cc" ] && [ -x "$compiler" ] && [ -x "$archiver" ] || { echo "declared LLVM tool closure is incomplete: $tool_root" >&2; exit 78; }
for build in first second; do
  case "$build" in first) timezone=Pacific/Kiritimati ;; second) timezone=Pacific/Pago_Pago ;; esac
  (cd "$stage" && SOURCE_DATE_EPOCH="$source_date_epoch" TZ="$timezone" CC="$cc" CXX="$compiler" AR="$archiver" \
    ./build -B ".build-vterm-$build" --target "$target" vterm-c-sdk)
done
first=$stage/.build-vterm-first/vterm-c
second=$stage/.build-vterm-second/vterm-c
for required in include/vterm_c.h lib/libshitty_vt.a lib/libplt_headless.a lib/libstd.a; do
  [ -f "$first/$required" ] && [ -f "$second/$required" ] || { echo "Shitty SDK output is missing: $required" >&2; exit 79; }
done
diff -qr "$first" "$second" >/dev/null || { echo "Shitty SDK outputs are not byte-identical" >&2; exit 79; }
cp -R "$first/." "$transaction/targets/$target/shitty-vt-sdk"
printf '%s\n' "$commit" > "$transaction/targets/$target/shitty-vt-sdk/source-commit.txt"
printf '%s\n' "$source_date_epoch" > "$transaction/targets/$target/shitty-vt-sdk/source-date-epoch.txt"
printf '%s\n' "$python_version" > "$transaction/targets/$target/shitty-vt-sdk/python-version.txt"
printf '%s\n' "$llvm_version" > "$transaction/targets/$target/shitty-vt-sdk/llvm-version.txt"
printf '%s\n' "$ragel_version" > "$transaction/targets/$target/shitty-vt-sdk/ragel-version.txt"
if find "$transaction/targets/$target/shitty-vt-sdk" -type l -print -quit | grep -q .; then
  echo "Shitty SDK output contains a symbolic link" >&2; exit 79
fi
find "$transaction/targets/$target/shitty-vt-sdk" -type f -exec chmod a-w {} +
find "$transaction/targets/$target/shitty-vt-sdk" -type d -exec chmod 0555 {} +
mkdir -p "$transaction/receipts"
soksak-validate build-receipt-create "$root/build-dependencies.json" --dependency shitty-vt-sdk \
  --target "$target" --output-root "$transaction" --out "$transaction/receipts/$target.json"
soksak-validate build-receipt "$transaction/receipts/$target.json" --dependencies "$root/build-dependencies.json" --output-root "$transaction"
mkdir -p "$build_root/targets" "$build_root/receipts"
[ ! -e "$build_root/targets/$target" ] && [ ! -e "$receipt" ] || { echo "Shitty SDK output appeared concurrently" >&2; exit 79; }
mv "$transaction/targets/$target" "$build_root/targets/$target"
mv "$transaction/receipts/$target.json" "$receipt"
soksak-validate build-receipt "$receipt" --dependencies "$root/build-dependencies.json" --output-root "$build_root"
echo "SHITTY_SDK_READY target=$target"
