#!/usr/bin/env node
import fs from "node:fs";

const workflow = fs.readFileSync(".github/workflows/release.yml", "utf8");
const targets = JSON.parse(fs.readFileSync("release/targets.json", "utf8"));
const stage = fs.readFileSync("scripts/stage-built.sh", "utf8");
const required = [
  "spec_url:", "spec_sha256:", "${{ inputs.spec_url }}", "${{ inputs.spec_sha256 }}",
  "node-version-file: soksak-sidecars/soksak-sidecar-terminal-shitty/.dependency/spec-package/package.json",
  "python-version: ${{ steps.dependency-tools.outputs.python }}",
  "version: ${{ steps.dependency-tools.outputs.llvm }}",
  "brew install ragel",
  'make verify TARGET="${{ matrix.target }}" OUT=dist',
  'make stage TARGET="${{ matrix.target }}" OUT=dist',
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
for (const bypass of ["repository: min-median-max/shitty", "./build -B", "SOKSAK_SHITTY_VT_SDK", "scripts/package-release.sh", "pnpm/action-setup", "repository: soksak-ai/soksak-spec"]) {
  if (workflow.includes(bypass)) throw new Error(`release workflow bypasses owner commands through ${bypass}`);
}
for (const match of workflow.matchAll(/^\s*-?\s*uses:\s*([^\s#]+)/gm)) {
  if (!/^[^@\s]+@[a-f0-9]{40}$/.test(match[1])) throw new Error(`workflow action is not commit-pinned: ${match[1]}`);
}
if (/windows|pc-windows/i.test(workflow)) throw new Error("Shitty release must not declare Windows");
if (!stage.includes("absolute candidate output")) throw new Error("stage-built does not permit isolated absolute output");
if (!stage.includes("sidecar.json")) throw new Error("stage-built does not emit the sidecar manifest");
console.log("Shitty release workflow contract: passed");
