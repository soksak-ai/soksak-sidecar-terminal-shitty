SHELL := /bin/sh

BUILD_DEPENDENCY_ROOT := target/build-dependencies/shitty-vt-sdk
OUT ?= dist

.PHONY: require-target preflight prepare build stage verify

require-target:
	@test '$(origin TARGET)' = 'command line' && test -n '$(TARGET)' || { echo 'TARGET must be an explicit Make command-line variable' >&2; exit 2; }

preflight: require-target
	@scripts/check-build-environment.sh '$(TARGET)'
	@soksak-validate build-dependencies build-dependencies.json --dependency shitty-vt-sdk --target '$(TARGET)' >/dev/null

prepare: preflight
	@scripts/prepare-shitty-sdk.sh '$(TARGET)' '$(BUILD_DEPENDENCY_ROOT)'

build: prepare
	@SOKSAK_BUILD_DEPENDENCY_ROOT='$(CURDIR)/$(BUILD_DEPENDENCY_ROOT)' cargo build --locked --release --target '$(TARGET)' --bin soksak-sidecar-terminal-shitty

stage: build
	@scripts/stage-built.sh '$(OUT)' '$(TARGET)'

verify: stage
	@node scripts/check-build-config.mjs
	@node scripts/check-release-workflow.mjs
	@soksak-validate build-receipt '$(BUILD_DEPENDENCY_ROOT)/receipts/$(TARGET).json' --dependencies build-dependencies.json --output-root '$(BUILD_DEPENDENCY_ROOT)'
	@SOKSAK_BUILD_DEPENDENCY_ROOT='$(CURDIR)/$(BUILD_DEPENDENCY_ROOT)' scripts/gate.sh '$(TARGET)'
