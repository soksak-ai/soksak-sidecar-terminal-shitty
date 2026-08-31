import assert from "node:assert/strict";
import { chmodSync, copyFileSync, existsSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";

const repository = join(import.meta.dirname, "..");

test("prepare recovers a committed target whose receipt move was interrupted", (context) => {
  const root = mkdtempSync(join(realpathSync(tmpdir()), "shitty-sdk-recovery-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const target = "aarch64-apple-darwin";
  const buildRoot = "target/build-dependencies/shitty-vt-sdk";
  const output = join(root, buildRoot, "targets", target, "shitty-vt-sdk");
  mkdirSync(join(root, "scripts"));
  mkdirSync(output, { recursive: true });
  writeFileSync(join(output, "archive"), "complete\n");
  writeFileSync(join(root, "build-dependencies.json"), "{}\n");
  copyFileSync(join(repository, "scripts/prepare-shitty-sdk.sh"), join(root, "scripts/prepare-shitty-sdk.sh"));

  const bin = join(root, "bin");
  mkdirSync(bin);
  const validator = join(bin, "soksak-sdk");
  writeFileSync(validator, [
    "#!/bin/sh", "set -eu", "test \"$1\" = validate", "shift", "command=$1", "shift",
    "case $command in",
    "  build-receipt-create)",
    "    out=", "    while [ $# -gt 0 ]; do if [ \"$1\" = --out ]; then out=$2; break; fi; shift; done",
    "    test -n \"$out\"", "    mkdir -p \"$(dirname -- \"$out\")\"", "    printf '{}\\n' > \"$out\"", "    ;;",
    "  build-receipt) test -f \"$1\" ;;",
    "  *) exit 2 ;;", "esac", "",
  ].join("\n"));
  chmodSync(validator, 0o700);

  const run = () => spawnSync("sh", ["scripts/prepare-shitty-sdk.sh", target, buildRoot], {
    cwd: root, encoding: "utf8", env: { ...process.env, PATH: `${bin}:/usr/bin:/bin` },
  });
  const first = run();
  assert.equal(first.status, 0, first.stderr);
  assert.match(first.stdout, /SHITTY_SDK_RECOVERED/);
  const receipt = join(root, buildRoot, "receipts", `${target}.json`);
  assert.equal(existsSync(receipt), true);
  assert.equal(readFileSync(receipt, "utf8"), "{}\n");
  const second = run();
  assert.equal(second.status, 0, second.stderr);
  assert.match(second.stdout, /SHITTY_SDK_REUSED/);
});

test("Make addresses the SDK cache by the build dependency document digest", (context) => {
  const root = mkdtempSync(join(realpathSync(tmpdir()), "shitty-sdk-address-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  copyFileSync(join(repository, "Makefile"), join(root, "Makefile"));
  const manifest = join(root, "build-dependencies.json");
  const cacheRoot = () => spawnSync("make", ["-s", "build-dependency-root"], {
    cwd: root,
    encoding: "utf8",
    env: process.env,
  });

  writeFileSync(manifest, '{"commit":"one"}\n');
  const first = cacheRoot();
  assert.equal(first.status, 0, first.stderr);
  assert.match(first.stdout, /^target\/build-dependencies\/shitty-vt-sdk\/[a-f0-9]{64}\n$/);

  writeFileSync(manifest, '{"commit":"two"}\n');
  const second = cacheRoot();
  assert.equal(second.status, 0, second.stderr);
  assert.match(second.stdout, /^target\/build-dependencies\/shitty-vt-sdk\/[a-f0-9]{64}\n$/);
  assert.notEqual(second.stdout, first.stdout);
});
