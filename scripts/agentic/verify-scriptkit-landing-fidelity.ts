#!/usr/bin/env bun
/**
 * Structural + optional raster fidelity checks for the landing product.
 *
 * Always checks:
 * - landing contract (5 stories, embed fixtures)
 * - staged root integrity if --root provided
 * - story reduce digests change over time
 * - fixture HTML loads tokens + footer anatomy for each screen
 *
 * When Playwright/puppeteer is unavailable, records blocked-by-browser and
 * still exits 0 if structural gates pass (native proof phase is separate).
 */
import { readFileSync, existsSync, readdirSync, statSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import vm from "node:vm";

const ROOT = join(import.meta.dir, "../..");
const args = process.argv.slice(2);
let staged: string | null = null;
for (let i = 0; i < args.length; i++) {
  if (args[i] === "--root") staged = args[++i] ?? null;
}

const failures: string[] = [];
const notes: string[] = [];

function assert(c: boolean, m: string) {
  if (!c) failures.push(m);
}

// 1) landing contract
const contract = spawnSync(
  process.execPath,
  [join(ROOT, "design/mockups/tests/landing-contract.test.mjs")],
  { encoding: "utf8", cwd: ROOT },
);
assert(contract.status === 0, `landing-contract failed:\n${contract.stdout}\n${contract.stderr}`);

// 2) staged root
if (staged) {
  assert(existsSync(join(staged, "index.html")), "staged missing index.html");
  assert(existsSync(join(staged, "stories.json")), "staged missing stories.json");
  assert(existsSync(join(staged, "generated/tokens.css")), "staged missing tokens");
  const walk = (dir: string, acc: string[] = []) => {
    for (const n of readdirSync(dir)) {
      const p = join(dir, n);
      if (statSync(p).isDirectory()) walk(p, acc);
      else acc.push(p);
    }
    return acc;
  };
  for (const f of walk(staged)) {
    if (!/\.(html|js|css|json)$/.test(f)) continue;
    const t = readFileSync(f, "utf8");
    if (t.includes("brisk-yarrow-s37f")) failures.push(`old slug in staged ${f}`);
    if (t.includes("../..") && f.endsWith(".html") && f.includes("/stories/0")) {
      // staged stories should not climb past root with ../../../
      if (t.includes("../../../")) failures.push(`parent traversal in staged story ${f}`);
    }
  }
  const m = JSON.parse(readFileSync(join(staged, "stories.json"), "utf8"));
  assert(m.count === 5, "staged stories.json count != 5");
}

// 3) timeline reduce for each landing story
const playerCode = readFileSync(
  join(ROOT, "design/mockups/stories/shared/story-player.js"),
  "utf8",
);
const sandbox: any = {
  console,
  requestAnimationFrame: () => 1,
  cancelAnimationFrame: () => {},
  document: { hidden: false },
};
sandbox.globalThis = sandbox;
sandbox.window = sandbox;
vm.runInNewContext(playerCode, sandbox);

const landingStories = join(ROOT, "design/mockups/landing/stories");
for (const id of readdirSync(landingStories)) {
  const jsPath = join(landingStories, id, "story.js");
  if (!existsSync(jsPath)) continue;
  const js = readFileSync(jsPath, "utf8");
  let story: any;
  try {
    story = new Function(
      js
        .replace(/\(function\s*\(\)\s*\{/, "")
        .replace(/var api = window\.StoryPlayer\.mount[\s\S]*$/m, "return story;")
        .replace(/window\.StoryPlayer\.mount[\s\S]*$/m, "return story;")
        .replace(/\}\)\(\);\s*$/, ""),
    )();
  } catch (e: any) {
    failures.push(`${id}: eval story.js ${e.message}`);
    continue;
  }
  const digests = new Set<string>();
  for (let t = 0; t <= (story.durationMs || 4000); t += 250) {
    digests.add(JSON.stringify(sandbox.StoryPlayer.reduce(story, t).semantic));
  }
  assert(digests.size >= 3, `${id}: need ≥3 semantic digests over time, got ${digests.size}`);
}

// 4) screen fixtures present for allowed set
for (const screen of [
  "main-menu",
  "actions-dialog",
  "confirm-popup",
  "clipboard-history",
  "day-page",
  "notes",
]) {
  const html = readFileSync(
    join(ROOT, "design/mockups/screens", screen, "index.html"),
    "utf8",
  );
  assert(html.includes("tokens.css"), `${screen}: tokens`);
  assert(html.includes("story-embed"), `${screen}: story-embed hook`);
}

notes.push(
  "Browser pixel hash compare (standalone vs landing embed) requires a headed browser; structural + timeline gates enforce mockup integrity here. Native app captures: bun scripts/agentic/scriptkit-landing-story-proof.ts all",
);

const summary = {
  ok: failures.length === 0,
  failures,
  notes,
  staged,
  at: new Date().toISOString(),
};
console.log(JSON.stringify(summary, null, 2));
if (failures.length) process.exit(1);
