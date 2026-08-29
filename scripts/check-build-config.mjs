#!/usr/bin/env node
import fs from "node:fs";

const read = (name) => fs.readFileSync(name, "utf8");
const dependencyDocument = JSON.parse(read("build-dependencies.json"));
const dependency = dependencyDocument.dependencies?.[0];
const makefile = read("Makefile");
const workflow = read(".github/workflows/release.yml");
const build = read("build.rs");
const prepare = read("scripts/prepare-shitty-sdk.sh");
const preflight = read("scripts/check-build-environment.sh");
const cargo = read("Cargo.toml");
const keys = (value) => Object.keys(value).sort().join("\n");

if (dependency?.commit !== "dbc42af98907fadd5b057d2922b890b2725c016c") {
  throw new Error("Shitty SDK must be pinned to the live mouse encoder revision");
}
if (!cargo.includes('soksak-kit-sidecar-terminal = { git = "https://github.com/soksak-ai/soksak-kit-sidecar-terminal", rev = "c89ddd642ae63122e48081bc25e38700094aab02"')) {
  throw new Error("terminal Kit must be pinned to the focus-presentation revision");
}

if (dependencyDocument.schema !== "soksak-build-dependencies-v1" || dependencyDocument.dependencies.length !== 1 ||
    keys(dependency) !== ["commit", "id", "repository", "targets", "tools"].join("\n")) {
  throw new Error("Shitty build dependency document is not flat and exact");
}
if (dependency.id !== "shitty-vt-sdk" || dependency.repository !== "https://github.com/min-median-max/shitty.git" ||
    !/^[a-f0-9]{40}$/.test(dependency.commit)) {
  throw new Error("Shitty SDK identity is invalid");
}
if (keys(dependency.tools) !== ["llvm", "python", "ragel"].join("\n") ||
    Object.values(dependency.tools).some((value) => !/^\d+[.]\d+[.]\d+$/.test(value))) {
  throw new Error("Shitty SDK tool versions are not exact");
}
const targets = JSON.parse(read("release/targets.json")).map(({ target }) => target).sort();
if (JSON.stringify(targets) !== JSON.stringify(Object.keys(dependency.targets).sort())) throw new Error("Shitty target sets differ");
for (const target of targets) {
  const expected = [{ path: `targets/${target}/shitty-vt-sdk`, type: "tree" }];
  if (JSON.stringify(dependency.targets[target].outputs) !== JSON.stringify(expected)) throw new Error(`Shitty output differs for ${target}`);
}
for (const [name, source] of [["Makefile", makefile], ["workflow", workflow], ["build.rs", build]]) {
  for (const duplicated of [dependency.repository, dependency.commit, ...Object.values(dependency.tools)]) {
    if (source.includes(duplicated)) throw new Error(`${name} duplicates build-dependencies.json metadata`);
  }
}
for (const target of ["preflight", "prepare", "build", "stage", "verify"]) {
  if (!new RegExp(`^${target}:`, "m").test(makefile)) throw new Error(`Makefile target is missing: ${target}`);
}
if (!prepare.includes("SOURCE_DATE_EPOCH") || !prepare.includes("git -C \"$source\" show -s --format=%ct") ||
    !prepare.includes("vterm-c-sdk") || !prepare.includes("soksak-validate build-receipt-create") ||
    !prepare.includes("SHITTY_SDK_RECOVERED") || !prepare.includes('AR="$archiver"') ||
    !preflight.includes('archiver=$(dirname -- "$compiler")/llvm-ar')) {
  throw new Error("Shitty SDK preparation does not own reproducible source build and receipt creation");
}
if (build.includes("SOKSAK_SHITTY_VT_SDK") || !build.includes("SOKSAK_BUILD_DEPENDENCY_ROOT")) {
  throw new Error("build.rs retains an ambient SDK path instead of the verified build root");
}
if (workflow.includes("repository: min-median-max/shitty") || !workflow.includes('make stage TARGET="${{ matrix.target }}" STAGE=dist')) {
  throw new Error("release workflow bypasses the declarative SDK or Make");
}

console.log("Shitty build configuration contract: passed");
