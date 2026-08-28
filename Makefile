SHELL := /bin/sh

BUILD_DEPENDENCY_DIGEST := $(shell node -e 'const {createHash}=require("node:crypto");const {readFileSync}=require("node:fs");process.stdout.write(createHash("sha256").update(readFileSync("build-dependencies.json")).digest("hex"))')
BUILD_DEPENDENCY_ROOT := target/build-dependencies/shitty-vt-sdk/$(BUILD_DEPENDENCY_DIGEST)
OUT ?= dist

.PHONY: require-target require-build-dependency-digest build-dependency-root preflight lock prepare build stage verify benchmark

require-target:
	@test '$(origin TARGET)' = 'command line' && test -n '$(TARGET)' || { echo 'TARGET must be an explicit Make command-line variable' >&2; exit 2; }

require-build-dependency-digest:
	@case '$(BUILD_DEPENDENCY_DIGEST)' in ''|*[!0-9a-f]*) echo 'build dependency digest is invalid' >&2; exit 2 ;; esac
	@test "$$(printf '%s' '$(BUILD_DEPENDENCY_DIGEST)' | wc -c | tr -d ' ')" = 64 || { echo 'build dependency digest is invalid' >&2; exit 2; }

build-dependency-root: require-build-dependency-digest
	@printf '%s\n' '$(BUILD_DEPENDENCY_ROOT)'

preflight: require-target require-build-dependency-digest
	@scripts/check-build-environment.sh '$(TARGET)' '$(BUILD_DEPENDENCY_ROOT)'
	@soksak-validate build-dependencies build-dependencies.json --dependency shitty-vt-sdk --target '$(TARGET)' >/dev/null

lock: preflight
	@cargo metadata --format-version 1 > /dev/null

prepare: preflight
	@scripts/prepare-shitty-sdk.sh '$(TARGET)' '$(BUILD_DEPENDENCY_ROOT)'

build: prepare
	@node scripts/check-cursor-contract.mjs
	@SOKSAK_BUILD_DEPENDENCY_ROOT='$(CURDIR)/$(BUILD_DEPENDENCY_ROOT)' cargo build --locked --release --target '$(TARGET)' --bin soksak-sidecar-terminal-shitty

stage: build
	@scripts/stage-built.sh '$(OUT)' '$(TARGET)'

verify: stage
	@node --test scripts/*.test.mjs
	@node scripts/check-build-config.mjs
	@node scripts/check-release-workflow.mjs
	@soksak-validate build-receipt '$(BUILD_DEPENDENCY_ROOT)/receipts/$(TARGET).json' --dependencies build-dependencies.json --output-root '$(BUILD_DEPENDENCY_ROOT)'
	@SOKSAK_BUILD_DEPENDENCY_ROOT='$(CURDIR)/$(BUILD_DEPENDENCY_ROOT)' scripts/gate.sh '$(TARGET)' '$(OUT)'

benchmark: verify
	@case '$(BENCH_OUT)' in /*) ;; *) echo 'BENCH_OUT must be an explicit absolute output directory' >&2; exit 2 ;; esac
	@test -x "$$SOKSAK_PTYD_BIN" || { echo 'SOKSAK_PTYD_BIN must name the product-owned PTY executable' >&2; exit 2; }
	@SOKSAK_BUILD_DEPENDENCY_ROOT='$(CURDIR)/$(BUILD_DEPENDENCY_ROOT)' SOKSAK_BENCH_OUT='$(BENCH_OUT)' cargo test --locked --release --target '$(TARGET)' --test bench -- --ignored --nocapture
