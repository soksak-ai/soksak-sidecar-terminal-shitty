# soksak-sidecar-terminal-shitty 0.0.8

Shitty terminal-state provider for `soksak-spec-sidecar-terminal` 0.0.1. The repository contains
only the Shitty engine adapter, provider identity, and conformance seat. Recovery lifecycle, PTY
observation, alt-screen preservation and restore serialization are provided by
`soksak-kit-sidecar-terminal` 0.0.2.

The build dependency is declared once in `build-dependencies.json`. That manifest names the
source repository and pins one exact commit, the Python, LLVM and Ragel versions, and the SDK
tree output each target produces.

```sh
make build TARGET=aarch64-apple-darwin
make verify TARGET=aarch64-apple-darwin
```

Make resolves the manifest, checks out the exact fork commit into repository-owned build state,
derives `SOURCE_DATE_EPOCH` from that commit, builds the SDK twice in independent roots and different
timezones, requires byte-identical output, creates the canonical tree receipt and then builds the
Rust Sidecar from that receipt. `build.rs` accepts no raw SDK path. No source checkout path is
guessed. The manifest declares four targets: aarch64-apple-darwin, x86_64-apple-darwin,
aarch64-unknown-linux-gnu and x86_64-unknown-linux-gnu.

The SDK links libplt_headless.a; it contains no Cocoa or Wayland window backend.
Linux builds link the xxhash backend selected by libstd at SDK build time.
