import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { join } from "node:path";
import test from "node:test";

test("product documents do not describe the engine as a fork", () => {
  const body = readFileSync(join(import.meta.dirname, "..", "README.md"), "utf8").toLowerCase();
  assert.equal(/\b(fork|forked|upstream|external project)\b/.test(body), false);
});
