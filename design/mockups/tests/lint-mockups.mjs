#!/usr/bin/env node
// Mockup honesty lint: hand-written mockup CSS may not contain visual
// literals — every color, size, radius, gap, opacity, and font size must
// resolve through a generated --sk-* custom property, keeping HTML mockups
// incapable of drifting from the Rust design contract.
//
// Generated files (design/mockups/generated/**) are exempt. Values allowed
// in hand-written CSS: 0, 1 (flex factors), 100%, auto, none, inherit,
// currentColor, transparent, var(...), calc() over vars, and --sk-emulator-*
// declarations (browser-only calibration, annotated in known-divergence).
//
// Usage: node design/mockups/tests/lint-mockups.mjs
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join, relative } from "node:path";

const root = new URL("..", import.meta.url).pathname;
const failures = [];

function* cssFiles(dir) {
  for (const entry of readdirSync(dir)) {
    const path = join(dir, entry);
    if (statSync(path).isDirectory()) {
      if (entry === "generated" || entry === "node_modules") continue;
      yield* cssFiles(path);
    } else if (entry.endsWith(".css")) {
      yield path;
    }
  }
}

const DECL_RE = /([\w-]+)\s*:\s*([^;{}]+);/g;
// Properties whose values must be token-derived when they carry magnitude.
const VISUAL_PROPS =
  /^(color|background|background-color|border|border-.*|outline|box-shadow|font-size|font-weight|line-height|letter-spacing|opacity|padding.*|margin.*|gap|row-gap|column-gap|width|min-width|max-width|height|min-height|max-height|top|right|bottom|left|inset.*|border-radius|backdrop-filter|-webkit-backdrop-filter|fill|stroke|stroke-width)$/;
// vh/vw are allowed: they position the harness stage around the window and
// cannot encode app-design magnitudes.
const LITERAL_RE =
  /(#[0-9a-fA-F]{3,8}\b|\brgba?\(|\bhsla?\(|\d*\.?\d+(px|pt|rem|em)\b)/;

function valueIsClean(value) {
  // Strip var() references and calc arithmetic over vars before scanning.
  const stripped = value
    .replace(/var\(--sk-[\w-]+\)/g, "VAR")
    .replace(/calc\(([^()]|\([^()]*\))*\)/g, (m) =>
      /\d*\.?\d+(px|pt|rem|em)/.test(m.replace(/var\(--sk-[\w-]+\)/g, "")) ? m : "CALC",
    );
  return !LITERAL_RE.test(stripped);
}

for (const file of cssFiles(root)) {
  const css = readFileSync(file, "utf8");
  const rel = relative(root, file);
  let match;
  while ((match = DECL_RE.exec(css))) {
    const [, prop, value] = match;
    // Emulator variables are declared literals by design — allowed, but only
    // under the --sk-emulator- namespace.
    if (prop.startsWith("--sk-emulator-")) continue;
    if (prop.startsWith("--")) {
      // Alias hooks (host-parameterized shared components, e.g. mapping
      // --sk-compact-caret-* onto a screen's generated tokens) are allowed
      // as long as the value itself is literal-free: pure var()/calc-over-var
      // indirection cannot smuggle in a design magnitude.
      if (!valueIsClean(value)) {
        failures.push(`${rel}: custom property ${prop} carries a literal outside the emulator namespace`);
      }
      continue;
    }
    if (!VISUAL_PROPS.test(prop)) continue;
    if (!valueIsClean(value)) {
      failures.push(`${rel}: literal visual value in "${prop}: ${value.trim()}"`);
    }
  }
}

if (failures.length) {
  console.error(`✗ mockup lint: ${failures.length} literal(s) found`);
  for (const failure of failures) console.error("  " + failure);
  process.exit(1);
}
console.log("✓ mockup lint: all visual values are token-derived");
