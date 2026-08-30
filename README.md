# soksak-sidecar-terminal-shitty

Shitty terminal-state provider for `soksak-spec-sidecar-terminal` 0.0.2. The repository contains
only the Shitty engine adapter, provider identity, and conformance seat. Recovery lifecycle, PTY
observation, alt-screen preservation and restore serialization are provided by
`soksak-kit-sidecar-terminal` 0.0.33.

The build dependency is declared once in `build-dependencies.json`. That manifest names the
source repository and pins one exact commit, the Python, LLVM and Ragel versions, and the SDK
tree output each target produces.

```sh
make lock TARGET=aarch64-apple-darwin
make build TARGET=aarch64-apple-darwin
make verify TARGET=aarch64-apple-darwin
make stage TARGET=aarch64-apple-darwin STAGE=dist
make attest TARGET=aarch64-apple-darwin OUT=/absolute/shitty-release
```

`make lock` is the only owner operation that projects changed Cargo declarations into
`Cargo.lock`. Normal build and verification remain `--locked`.

Make addresses repository-owned SDK state by the SHA-256 of `build-dependencies.json`, resolves the
manifest, checks out the exact fork commit,
derives `SOURCE_DATE_EPOCH` from that commit, builds the SDK twice in independent roots and different
timezones, requires byte-identical output, creates the canonical tree receipt and then builds the
Rust Sidecar from that receipt. `build.rs` accepts no raw SDK path. No source checkout path is
guessed. The manifest declares four targets: aarch64-apple-darwin, x86_64-apple-darwin,
aarch64-unknown-linux-gnu and x86_64-unknown-linux-gnu.
If a process ends after committing the target tree but before committing its receipt, the next
`make prepare` reconstructs and validates the canonical receipt from that exact tree. An incomplete
tree still fails rather than being accepted or silently replaced.

The SDK links `libplt_headless.a`; it contains no Cocoa or Wayland window backend. Its base64 and
hash implementations are self-contained, so ambient simdutf and xxhash headers cannot add an
undeclared link dependency. The SDK target links and runs its C smoke program before publishing the
tree.

Cursor shape, DECSCUSR blink state, and the provider animation interval come from the Shitty
snapshot ABI. The adapter does not parse terminal input to reconstruct them. DECTCEM remains the
separate visibility mode.

The maintained Vterm fork owns OSC 4/10/11/12 state and exposes exact override-presence flags and
a 256-entry palette mask through its C snapshot ABI. The Rust adapter maps that snapshot to
`TerminalThemeOverrides`; it does not parse OSC or compare effective colors with defaults. Reset
clears the matching flag or mask bit so the common renderer reveals the current host base theme.

Wheel input has an explicit owner boundary: the common Kit accumulates device deltas and owns
ordinary scrollback, while this adapter handles only live mouse reports and alternate-screen
cursor-key scrolling. See [Terminal input ownership](docs/TERMINAL-INPUT.md) for the supported
encodings, stale-route refusal, and known public-contract limitations.

`make stage STAGE=<directory>` writes the canonical runtime tree: root `sidecar.json`,
`dist/soksak-sidecar-terminal-shitty`, and the process-local `dist/sidecar.json`. That directory is
the direct `soksak-sdk pack-target --source` input.
