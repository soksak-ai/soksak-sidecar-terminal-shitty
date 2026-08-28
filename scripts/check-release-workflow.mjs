#!/usr/bin/env node
import fs from "node:fs";

const workflow = fs.readFileSync(".github/workflows/release.yml", "utf8");
const targets = JSON.parse(fs.readFileSync("release/targets.json", "utf8"));
const stage = fs.readFileSync("scripts/stage-built.sh", "utf8");
const makefile = fs.readFileSync("Makefile", "utf8");
const gate = fs.readFileSync("scripts/gate.sh", "utf8");
if (!/^lock: preflight$/m.test(makefile) || !makefile.includes("cargo metadata --format-version 1")) throw new Error("Makefile must own Cargo lock regeneration");
if (!fs.readFileSync("README.md", "utf8").includes("make lock TARGET=")) throw new Error("README must document the owner lock target");
for (const target of ["require-tooling", "require-out", "release", "attest"]) if (!new RegExp(`^${target}:`, "m").test(makefile)) throw new Error(`Makefile target is missing: ${target}`);
if (!/^STAGE \?= dist$/m.test(makefile) || /^OUT \?= dist$/m.test(makefile)) throw new Error("Makefile must separate STAGE from release OUT");
for (const value of ["command -v soksak-sdk", "SDK_VERSION", "soksak-sdk pack-target", "soksak-sdk package", "soksak-sdk attest"]) if (!makefile.includes(value)) throw new Error(`Makefile release boundary is missing: ${value}`);
if (!fs.readFileSync("README.md", "utf8").includes("make attest TARGET=") || !fs.readFileSync("README.md", "utf8").includes("OUT=/absolute/")) throw new Error("README must document owner attestation");
const required = [
  "spec_url:", "spec_sha256:", "${{ inputs.spec_url }}", "${{ inputs.spec_sha256 }}",
  "node-version-file: soksak-sidecars/soksak-sidecar-terminal-shitty/.dependency/spec-package/package.json",
  "python-version: ${{ steps.dependency-tools.outputs.python }}",
  'formula="llvm@${llvm%%.*}"', 'echo "$tool_root" >> "$GITHUB_PATH"',
  'make verify TARGET="${{ matrix.target }}" OUT=dist',
  'make stage TARGET="${{ matrix.target }}" STAGE=dist',
  "release-template/sidecar/pack-target.mjs",
  "release-template/sidecar/build-release.mjs",
  "release-template/sidecar/validate-with-spec.mjs",
  "release-template/publish-canonical-release.mjs",
  "SOKSAK_RELEASE_TOKEN: ${{ steps.release-token.outputs.token }}",
];
for (const value of required) if (!workflow.includes(value)) throw new Error(`release workflow omits ${value}`);
for (const { target, runner } of targets) {
  if (!workflow.includes(`target: ${target}`) || !workflow.includes(`runner: ${runner}`)) throw new Error(`release matrix omits ${target}/${runner}`);
}
for (const bypass of ["repository: min-median-max/shitty", "./build -B", "SOKSAK_SHITTY_VT_SDK", "scripts/package-release.sh", "pnpm/action-setup", "repository: soksak-ai/soksak-spec", "KyleMayes/install-llvm-action"]) {
  if (workflow.includes(bypass)) throw new Error(`release workflow bypasses owner commands through ${bypass}`);
}
if (workflow.indexOf('formula="llvm@${llvm%%.*}"') > workflow.indexOf("uses: actions/setup-python@")) {
  throw new Error("release workflow replaces the declared Python after tool installation");
}
for (const match of workflow.matchAll(/^\s*-?\s*uses:\s*([^\s#]+)/gm)) {
  if (!/^[^@\s]+@[a-f0-9]{40}$/.test(match[1])) throw new Error(`workflow action is not commit-pinned: ${match[1]}`);
}
if (/windows|pc-windows/i.test(workflow)) throw new Error("Shitty release must not declare Windows");
if (!stage.includes("absolute candidate output")) throw new Error("stage-built does not permit isolated absolute output");
if (!stage.includes("sidecar.json")) throw new Error("stage-built does not emit the sidecar manifest");
if (!/^benchmark:/m.test(makefile) || /--test bench/.test(gate)) throw new Error("benchmark ownership is not separated from verification");
console.log("Shitty release workflow contract: passed");
