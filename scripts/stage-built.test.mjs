import assert from "node:assert/strict";
import { chmodSync, mkdirSync, mkdtempSync, readFileSync, realpathSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import test from "node:test";

const repository = join(import.meta.dirname, "..");

test("stage replaces a previous patch version and remains idempotent", (context) => {
  const root = mkdtempSync(join(realpathSync(tmpdir()), "shitty-stage-"));
  context.after(() => rmSync(root, { recursive: true, force: true }));
  const target = "aarch64-apple-darwin";
  const binary = join(root, "target", target, "release", "soksak-sidecar-terminal-shitty");
  const out = "dist";
  mkdirSync(join(root, "scripts"));
  mkdirSync(join(root, "target", target, "release"), { recursive: true });
  writeFileSync(join(root, "scripts", "stage-built.sh"), readFileSync(join(repository, "scripts", "stage-built.sh")));
  chmodSync(join(root, "scripts", "stage-built.sh"), 0o700);
  const manifest = (version) => JSON.stringify({
    id: "soksak-sidecar-terminal-shitty",
    version,
    process: "dist/soksak-sidecar-terminal-shitty",
  });
  const run = () => spawnSync("sh", ["scripts/stage-built.sh", out, target], {
    cwd: root,
    encoding: "utf8",
  });

  writeFileSync(binary, "first\n");
  writeFileSync(join(root, "sidecar.json"), manifest("0.0.1"));
  assert.equal(run().status, 0);

  writeFileSync(binary, "changed-without-version\n");
  const conflict = run();
  assert.notEqual(conflict.status, 0);
  assert.match(conflict.stderr, /NOT_DETERMINISTIC/);

  writeFileSync(join(root, "sidecar.json"), manifest("0.0.2"));
  const replaced = run();
  assert.equal(replaced.status, 0, replaced.stderr);
  assert.equal(readFileSync(join(root, out, "soksak-sidecar-terminal-shitty"), "utf8"), "changed-without-version\n");
  assert.equal(JSON.parse(readFileSync(join(root, out, "sidecar.json"), "utf8")).version, "0.0.2");

  const repeated = run();
  assert.equal(repeated.status, 0, repeated.stderr);
  assert.match(repeated.stdout, /SHITTY_STAGED_UNCHANGED/);
});
