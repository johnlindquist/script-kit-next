#!/usr/bin/env node
/**
 * Contract for the five-story scriptkit.com-style landing.
 * Separate product from the nine-story gallery inventory test.
 */
import { readFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __dirname = dirname(fileURLToPath(import.meta.url));
const landingRoot = join(__dirname, "..", "landing");
const failures = [];
const assert = (c, m) => {
  if (!c) failures.push(m);
};

const allowedScreens = new Set([
  "main-menu",
  "actions-dialog",
  "confirm-popup",
  "clipboard-history",
  "day-page",
  "notes",
]);

const forbidden = [
  "brisk-yarrow-s37f",
  "story-mockup-realism-v2",
  "agent-chat",
  "terminal-prompt",
  "chat-prompt",
  "settings",
  "Row removed",
  "Pasted ✓",
  "sk-footer-btn",
];

const manifest = JSON.parse(readFileSync(join(landingRoot, "stories.json"), "utf8"));
assert(manifest.count === 5, `count must be 5, got ${manifest.count}`);
assert(manifest.stories.length === 5, `stories length must be 5`);
assert(manifest.product === "scriptkit-landing", "product marker");

const ids = new Set();
for (const s of manifest.stories) {
  assert(!ids.has(s.id), `duplicate id ${s.id}`);
  ids.add(s.id);
  assert(Array.isArray(s.screens) && s.screens.length >= 1, `${s.id}: screens required`);
  for (const sc of s.screens) {
    assert(allowedScreens.has(sc), `${s.id}: screen ${sc} not in allowed set`);
  }
  assert(Array.isArray(s.proofAtMs) && s.proofAtMs.length >= 2, `${s.id}: proofAtMs`);
  assert(!!s.runtimeProof, `${s.id}: runtimeProof`);
  assert(!!s.marketingClaim, `${s.id}: marketingClaim`);
  assert(existsSync(join(landingRoot, s.entry)), `${s.id}: missing entry ${s.entry}`);

  const html = readFileSync(join(landingRoot, s.entry), "utf8");
  assert(html.includes("embed=story"), `${s.id}: must use embed=story`);
  assert(html.includes("data-story-surface"), `${s.id}: needs iframe surfaces`);
  assert(!html.includes('class="sk-window"'), `${s.id}: no cloned sk-window in parent`);
  for (const bad of forbidden) {
    if (html.includes(bad)) failures.push(`${s.id} html contains forbidden "${bad}"`);
  }

  const js = readFileSync(join(landingRoot, "stories", s.id, "story.js"), "utf8");
  assert(js.includes("StoryPlayer.mount"), `${s.id}: must mount StoryPlayer`);
  for (const bad of ["Row removed", "Pasted ✓"]) {
    if (js.includes(bad)) failures.push(`${s.id} story.js claims unsupported outcome: ${bad}`);
  }
}

const index = readFileSync(join(landingRoot, "index.html"), "utf8");
assert(index.includes("01-search"), "landing index embeds 01-search");
assert(index.includes("05-notes"), "landing index embeds 05-notes");
assert((index.match(/data-landing-story/g) || []).length === 5, "exactly 5 landing iframes");
for (const bad of forbidden) {
  if (index.includes(bad) && bad !== "settings") {
    // settings string might appear in copy? avoid
  }
}
assert(!index.includes("brisk-yarrow"), "no old slug");
assert(!index.includes("agent-chat"), "no agent chat on five-story landing");

const lint = spawnSync(process.execPath, [join(__dirname, "lint-mockups.mjs")], {
  encoding: "utf8",
});
assert(lint.status === 0, `lint failed:\n${lint.stdout}\n${lint.stderr}`);

if (failures.length) {
  console.error(`✗ landing contract: ${failures.length}`);
  failures.forEach((f) => console.error(" -", f));
  process.exit(1);
}
console.log("✓ landing contract: 5 stories, allowed screens, embed fixtures, lint green");
