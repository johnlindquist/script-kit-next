#!/usr/bin/env node
/**
 * Pure timeline tests: StoryPlayer.reduce is deterministic and seek-stable.
 */
import { readFileSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const __dirname = dirname(fileURLToPath(import.meta.url));
const playerCode = readFileSync(
  join(__dirname, "..", "stories", "shared", "story-player.js"),
  "utf8",
);
const sandbox = {
  console,
  requestAnimationFrame: () => 1,
  cancelAnimationFrame: () => {},
  document: { hidden: false },
};
sandbox.globalThis = sandbox;
sandbox.window = sandbox;
vm.runInNewContext(playerCode, sandbox);
const { reduce, typePrefix } = sandbox.StoryPlayer;

const failures = [];
const assert = (c, m) => {
  if (!c) failures.push(m);
};

// typePrefix
assert(typePrefix("fruit", 0) === "f" || typePrefix("fruit", 0) === "", "prefix@0");
assert(typePrefix("fruit", 1) === "fruit", "prefix@1");
const mid = new Set([0.2, 0.4, 0.6, 0.8, 1].map((p) => typePrefix("fruit", p)));
assert(mid.size >= 3, `fruit prefixes need ≥3, got ${[...mid]}`);

const story = {
  durationMs: 3000,
  surfaces: [{ id: "main-menu", initial: true }],
  actions: [
    { at: 0, kind: "showSurface", surface: "main-menu" },
    { at: 100, duration: 1000, kind: "type", surface: "main-menu", text: "hello", as: "filter" },
    { at: 1200, kind: "setSelection", surface: "main-menu", index: 2 },
    { at: 2000, duration: 800, kind: "walkSelection", surface: "main-menu", from: 0, to: 3 },
  ],
};

const a = reduce(story, 500);
const b = reduce(story, 500);
assert(JSON.stringify(a) === JSON.stringify(b), "reduce is pure/deterministic");

const s0 = reduce(story, 0).semantic["main-menu"] || {};
const s1 = reduce(story, 600).semantic["main-menu"] || {};
const s2 = reduce(story, 1500).semantic["main-menu"] || {};
const s3 = reduce(story, 2400).semantic["main-menu"] || {};
assert((s1.search || s1.filter || "").length > 0, "typing produces search text");
assert(s2.selectedIndex === 2, `setSelection index 2, got ${s2.selectedIndex}`);
assert(
  typeof s3.selectedIndex === "number" && s3.selectedIndex >= 0,
  "walkSelection yields index",
);

// seek stability: same t always same digest regardless of prior t
const d1 = JSON.stringify(reduce(story, 800).semantic);
const _ = reduce(story, 2000);
const d2 = JSON.stringify(reduce(story, 800).semantic);
assert(d1 === d2, "seek-from-any-prior-t is deterministic");

if (failures.length) {
  console.error(`✗ timeline: ${failures.length}`);
  failures.forEach((f) => console.error(" -", f));
  process.exit(1);
}
console.log("✓ stories timeline: reduce pure, type prefixes, selection walk, seek-stable");
