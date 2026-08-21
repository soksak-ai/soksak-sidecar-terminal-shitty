# soksak-sidecar-terminal-shitty 0.0.1

Shitty terminal-state provider for `soksak-spec-sidecar-terminal` 0.0.1. The repository contains
only the Shitty engine adapter, provider identity, and conformance seat. Recovery lifecycle, PTY
observation, alt-screen preservation and restore serialization are provided by
`soksak-kit-sidecar-terminal` 0.0.1.

Build the pinned Shitty source SDK and declare it explicitly:

```sh
cd /path/to/shitty
CC=/path/to/llvm/bin/clang CXX=/path/to/llvm/bin/clang++ \
  ./build -B .build-vterm-arm --target arm64-apple-darwin vterm-c-sdk
SOKSAK_SHITTY_VT_SDK=/path/to/shitty/.build-vterm-arm/vterm-c cargo test
```

No source checkout path is guessed. The manifest declares four targets: aarch64-apple-darwin, x86_64-apple-darwin,
aarch64-unknown-linux-gnu and x86_64-unknown-linux-gnu.
