#!/usr/bin/env node
/**
 * Footer contract: no generic sk-footer-btn in stories; screen fixtures keep
 * sk-footer-action + sk-keycap anatomy.
 */
import { readFileSync, readdirSync, statSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const mockups = join(__dirname, "..");
const failures = [];
const assert = (c, m) => {
  if (!c) failures.push(m);
};

function walk(dir, acc = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, acc);
    else if (/\.(html|js|css)$/.test(name)) acc.push(p);
  }
  return acc;
}

for (const file of walk(join(mockups, "stories"))) {
  const text = readFileSync(file, "utf8");
  if (text.includes("sk-footer-btn")) {
    failures.push(`${file}: contains forbidden sk-footer-btn`);
  }
}

// Canonical main-menu fixture still has correct anatomy
const main = readFileSync(join(mockups, "screens", "main-menu", "index.html"), "utf8");
assert(main.includes("sk-footer-host"), "main-menu missing sk-footer-host");
assert(main.includes("sk-footer-rail"), "main-menu missing sk-footer-rail");
assert(main.includes("sk-footer-spacer"), "main-menu missing sk-footer-spacer");
assert(main.includes("sk-footer-action"), "main-menu missing sk-footer-action");
assert(main.includes("sk-keycap"), "main-menu missing sk-keycap");
assert(main.includes("sk-footer-label"), "main-menu missing sk-footer-label");
// Compound shortcuts as separate keycaps
assert(main.includes(">⌘</kbd>") || main.includes(">⌘<"), "⌘ keycap present");
assert(main.includes(">K</kbd>") || main.includes(">K<"), "K keycap present");

if (failures.length) {
  console.error(`✗ footer contract: ${failures.length}`);
  failures.forEach((f) => console.error(" -", f));
  process.exit(1);
}
console.log("✓ stories footer contract: no sk-footer-btn; main-menu keycap anatomy intact");
