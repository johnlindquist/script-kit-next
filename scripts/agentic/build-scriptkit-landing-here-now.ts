#!/usr/bin/env bun
/**
 * Stage a clean here.now publish root for the five-story scriptkit landing.
 * Copies an allowlist only; rewrites story asset paths to root-relative.
 *
 * Usage:
 *   bun scripts/agentic/build-scriptkit-landing-here-now.ts
 *   bun scripts/agentic/build-scriptkit-landing-here-now.ts --out .test-output/here-now/scriptkit-landing
 */
import {
  cpSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
  existsSync,
  readdirSync,
  statSync,
} from "node:fs";
import { join, dirname, relative } from "node:path";

const ROOT = join(import.meta.dir, "../..");
const args = process.argv.slice(2);
let out = join(ROOT, ".test-output/here-now/scriptkit-landing");
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--out") out = args[++i] ?? out;
}

const SCREENS = [
  "main-menu",
  "actions-dialog",
  "confirm-popup",
  "clipboard-history",
  "day-page",
  "notes",
] as const;

function copyFile(src: string, dest: string) {
  mkdirSync(dirname(dest), { recursive: true });
  cpSync(src, dest);
}

function copyDirFiltered(src: string, dest: string, skip: (name: string) => boolean) {
  mkdirSync(dest, { recursive: true });
  for (const name of readdirSync(src)) {
    if (skip(name)) continue;
    const s = join(src, name);
    const d = join(dest, name);
    if (statSync(s).isDirectory()) copyDirFiltered(s, d, skip);
    else copyFile(s, d);
  }
}

rmSync(out, { recursive: true, force: true });
mkdirSync(out, { recursive: true });

// Landing shell
copyFile(join(ROOT, "design/mockups/landing/index.html"), join(out, "index.html"));
copyFile(join(ROOT, "design/mockups/landing/landing.css"), join(out, "landing.css"));
copyFile(join(ROOT, "design/mockups/landing/landing.js"), join(out, "landing.js"));
copyFile(join(ROOT, "design/mockups/landing/stories.json"), join(out, "stories.json"));

// Generated tokens + shared
copyFile(
  join(ROOT, "design/mockups/generated/tokens.css"),
  join(out, "generated/tokens.css"),
);
copyFile(
  join(ROOT, "design/mockups/generated/tokens.json"),
  join(out, "generated/tokens.json"),
);
copyFile(
  join(ROOT, "design/mockups/shared/components.css"),
  join(out, "shared/components.css"),
);
copyFile(
  join(ROOT, "design/mockups/shared/story-embed.css"),
  join(out, "shared/story-embed.css"),
);
copyFile(
  join(ROOT, "design/mockups/shared/story-embed.js"),
  join(out, "shared/story-embed.js"),
);

// Story runtime (shared)
for (const f of ["story-player.js", "surface-adapters.js", "story-shell.css"]) {
  copyFile(
    join(ROOT, "design/mockups/stories/shared", f),
    join(out, "stories/shared", f),
  );
}

// Screens (allowlisted only; skip reference PNGs and heavy receipts)
for (const screen of SCREENS) {
  const srcDir = join(ROOT, "design/mockups/screens", screen);
  const destDir = join(out, "screens", screen);
  copyDirFiltered(srcDir, destDir, (name) => {
    if (name === "reference") return true;
    if (name.endsWith(".png") || name.endsWith(".jpg")) return true;
    if (name.includes("runtime-receipt") || name.includes("image-diff")) return true;
    return false;
  });
  // rewrite embed paths in screen HTML: ../../shared -> ../../shared (same)
  // screen index uses ../../generated and ../../shared — correct for staged root
}

// Landing stories with rewritten paths
const storiesSrc = join(ROOT, "design/mockups/landing/stories");
for (const id of readdirSync(storiesSrc)) {
  const srcDir = join(storiesSrc, id);
  if (!statSync(srcDir).isDirectory()) continue;
  const destDir = join(out, "stories", id);
  mkdirSync(destDir, { recursive: true });
  for (const name of readdirSync(srcDir)) {
    const raw = readFileSync(join(srcDir, name), "utf8");
    // From stories/01-search/ staged: ../../../X -> ../../X for screens/generated/stories/shared
    // Source paths from landing/stories/id used ../../../screens and ../../../stories/shared
    // Staged structure: stories/id + screens + generated + stories/shared
    // so from stories/id: ../../screens, ../../generated, ../../stories/shared — wait
    // staged:
    //   stories/01-search/index.html
    //   stories/shared/
    //   screens/
    //   generated/
    // relative from stories/01-search: ../shared, ../../screens? 
    // stories/01-search -> ../shared = stories/shared ✓
    // stories/01-search -> ../../screens = screens ✓
    // stories/01-search -> ../../generated = generated ✓
    let text = raw
      .replaceAll("../../../generated/", "../../generated/")
      .replaceAll("../../../stories/shared/", "../shared/")
      .replaceAll("../../../screens/", "../../screens/")
      .replaceAll("../../index.html", "../../index.html");
    // marketing bootstrap already present
    writeFileSync(join(destDir, name), text);
  }
}

// Fix landing index iframe srcs already ./stories/... — good
// Fix screen embed script paths: screens/main-menu uses ../../shared — from screens/main-menu that's shared/ ✓

// Sanity: no old slug
function walk(dir: string, acc: string[] = []) {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    if (statSync(p).isDirectory()) walk(p, acc);
    else acc.push(p);
  }
  return acc;
}
for (const file of walk(out)) {
  if (!/\.(html|js|css|json|md)$/.test(file)) continue;
  const t = readFileSync(file, "utf8");
  if (t.includes("brisk-yarrow-s37f")) {
    console.error("old slug found in", relative(out, file));
    process.exit(2);
  }
}

const receipt = {
  out,
  screens: SCREENS,
  stories: readdirSync(join(out, "stories")).filter((n) => n !== "shared"),
  builtAt: new Date().toISOString(),
};
writeFileSync(join(out, "build-receipt.json"), JSON.stringify(receipt, null, 2));
console.log(JSON.stringify({ ok: true, ...receipt }, null, 2));
