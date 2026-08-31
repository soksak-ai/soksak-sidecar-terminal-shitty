#!/bin/sh
set -eu

[ "$#" -ge 1 ] && [ -n "$1" ] || { echo 'usage: check-build-environment.sh <target> [build-dependency-root]' >&2; exit 78; }
target=$1
build_root=${2:-}

# The SDK toolchain builds the SDK. A receipt that verifies is an SDK already built by the declared
# toolchain, so this host is asked only for what still has to run: the crate compiler and its runtime.
sdk=required
if [ -n "$build_root" ] && [ -f "$build_root/receipts/$target.json" ] && \
   soksak-sdk validate build-receipt "$build_root/receipts/$target.json" --dependencies build-dependencies.json --output-root "$build_root" >/dev/null 2>&1; then
  sdk=reused
fi
resolution=$(soksak-sdk validate build-dependencies build-dependencies.json --dependency shitty-vt-sdk --target "$target") || exit 78
tool() { printf '%s' "$resolution" | node -e 'let s="";process.stdin.on("data",c=>s+=c).on("end",()=>process.stdout.write(JSON.parse(s).tools[process.argv[1]]))' "$1"; }
python_expected=$(tool python)
llvm_expected=$(tool llvm)
ragel_expected=$(tool ragel)

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64) host_target=aarch64-apple-darwin; node_platform=darwin; node_arch=arm64; python_arch=arm64; compiler_pattern='^(arm64|aarch64)-apple-darwin' ;;
  Darwin-x86_64)
    if [ "$(sysctl -n hw.optional.arm64 2>/dev/null || true)" = 1 ]; then
      host_target=aarch64-apple-darwin; node_platform=darwin; node_arch=arm64; python_arch=arm64; compiler_pattern='^(arm64|aarch64)-apple-darwin'
    else
      host_target=x86_64-apple-darwin; node_platform=darwin; node_arch=x64; python_arch=x86_64; compiler_pattern='^x86_64-apple-darwin'
    fi
    ;;
  Linux-aarch64|Linux-arm64) host_target=aarch64-unknown-linux-gnu; node_platform=linux; node_arch=arm64; python_arch=aarch64; compiler_pattern='^aarch64-.*linux' ;;
  Linux-x86_64) host_target=x86_64-unknown-linux-gnu; node_platform=linux; node_arch=x64; python_arch=x86_64; compiler_pattern='^x86_64-.*linux' ;;
  *) echo "TOOLCHAIN_MISMATCH: unsupported Shitty host $(uname -s)-$(uname -m)" >&2; exit 78 ;;
esac

python_record=$(python3 -c 'import platform,sys;print(sys.version.split()[0]);print(platform.machine())' 2>/dev/null || true)
python_version=$(printf '%s\n' "$python_record" | sed -n '1p')
python_machine=$(printf '%s\n' "$python_record" | sed -n '2p')
compiler=$(command -v clang++ 2>/dev/null || true)
llvm_version=
llvm_ar_version=
compiler_target=
if [ -n "$compiler" ]; then
  archiver=$(dirname -- "$compiler")/llvm-ar
  llvm_version=$("$compiler" --version 2>/dev/null | sed -n '1p' | grep -Eo '[0-9]+[.][0-9]+[.][0-9]+' | sed -n '1p')
  llvm_ar_version=$("$archiver" --version 2>/dev/null | sed -n '1p' | grep -Eo '[0-9]+[.][0-9]+[.][0-9]+' | sed -n '1p')
  compiler_target=$("$compiler" -dumpmachine 2>/dev/null || true)
fi
ragel_version=$(ragel --version 2>/dev/null | sed -n '1s/.*version \([0-9][0-9]*\.[0-9][0-9]*\).*/\1.0/p')
rust_expected=$(sed -n 's/^channel = "\([^"]*\)"$/\1/p' rust-toolchain.toml)
rust_actual=$(rustc --version 2>/dev/null | awk '{print $2}' || true)
rust_host=$(rustc -vV 2>/dev/null | sed -n 's/^host: //p' || true)
node_actual_platform=$(node -p process.platform 2>/dev/null || true)
node_actual_arch=$(node -p process.arch 2>/dev/null || true)

sdk_mismatch=0
if [ "$sdk" = required ]; then
  if [ "$python_version" != "$python_expected" ] || [ "$python_machine" != "$python_arch" ] || \
     [ "$llvm_version" != "$llvm_expected" ] || [ "$llvm_ar_version" != "$llvm_expected" ] || \
     ! printf '%s\n' "$compiler_target" | grep -Eq "$compiler_pattern" || \
     [ "$ragel_version" != "$ragel_expected" ]; then
    sdk_mismatch=1
  fi
fi

if [ "$target" != "$host_target" ] || [ "$sdk_mismatch" = 1 ] || \
   [ "$rust_actual" != "$rust_expected" ] || [ "$rust_host" != "$target" ] || \
   [ "$node_actual_platform" != "$node_platform" ] || [ "$node_actual_arch" != "$node_arch" ]; then
  printf 'TOOLCHAIN_MISMATCH: sdk=%s target=%s hostTarget=%s python=%s/%s llvm=%s ar=%s compiler=%s ragel=%s rust=%s/%s node=%s/%s; expected python=%s/%s llvm=%s ragel=%s rust=%s/%s node=%s/%s\n' \
    "$sdk" \
    "$target" "$host_target" "${python_version:-missing}" "${python_machine:-unknown}" "${llvm_version:-missing}" "${llvm_ar_version:-missing}" "${compiler_target:-unknown}" \
    "${ragel_version:-missing}" "${rust_actual:-missing}" "${rust_host:-unknown}" "${node_actual_platform:-unknown}" "${node_actual_arch:-unknown}" \
    "$python_expected" "$python_arch" "$llvm_expected" "$ragel_expected" "$rust_expected" "$target" "$node_platform" "$node_arch" >&2
  exit 78
fi

printf 'BUILD_ENVIRONMENT_READY sdk=%s target=%s python=%s/%s llvm=%s compiler=%s ragel=%s rust=%s/%s node=%s/%s\n' \
  "$sdk" "$target" "${python_version:-unused}" "${python_machine:-unused}" "${llvm_version:-unused}" "${compiler_target:-unused}" "${ragel_version:-unused}" "$rust_actual" "$rust_host" "$node_actual_platform" "$node_actual_arch"
