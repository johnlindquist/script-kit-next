#!/usr/bin/env bun
/**
 * Closed-world native capture phases for the five landing stories.
 * Writes receipts under .test-output/scriptkit-landing/proofs/<phase>/.
 *
 * Usage:
 *   bun scripts/agentic/scriptkit-landing-story-proof.ts all
 *   bun scripts/agentic/scriptkit-landing-story-proof.ts search
 *
 * Phases map to design-reference-capture screens where available.
 */
import { mkdirSync, writeFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

const ROOT = join(import.meta.dir, "../..");
const OUT = join(ROOT, ".test-output/scriptkit-landing/proofs");
const phase = process.argv[2] || "all";

const PHASES: Record<
  string,
  { screen: string; storyId: string; dims: { w: number; h: number } }
> = {
  search: { screen: "main", storyId: "01-search", dims: { w: 1500, h: 960 } },
  "actions-confirm": {
    screen: "actions",
    storyId: "02-actions-confirm",
    dims: { w: 680, h: 600 },
  },
  clipboard: {
    screen: "clipboard",
    storyId: "03-clipboard",
    dims: { w: 1500, h: 960 },
  },
  "day-page": {
    screen: "day-page",
    storyId: "04-day-page",
    dims: { w: 1500, h: 960 },
  },
  notes: { screen: "notes", storyId: "05-notes", dims: { w: 700, h: 560 } },
};

function runPhase(name: string) {
  const cfg = PHASES[name];
  if (!cfg) {
    console.error("unknown phase", name);
    process.exit(2);
  }
  const dir = join(OUT, name);
  mkdirSync(dir, { recursive: true });
  const png = join(dir, `app-rest@2x.png`);
  const capture = spawnSync(
    "bun",
    [
      join(ROOT, "scripts/agentic/design-reference-capture.ts"),
      "--screen",
      cfg.screen,
      png,
    ],
    { cwd: ROOT, encoding: "utf8", env: process.env },
  );
  const receipt = {
    phase: name,
    storyId: cfg.storyId,
    screen: cfg.screen,
    exitCode: capture.exitCode,
    stdout: (capture.stdout || "").slice(-2000),
    stderr: (capture.stderr || "").slice(-2000),
    png,
    pngExists: existsSync(png),
    expectedPhysical: cfg.dims,
    at: new Date().toISOString(),
  };
  writeFileSync(join(dir, "runtime.json"), JSON.stringify(receipt, null, 2));
  console.log(JSON.stringify({ phase: name, ok: capture.exitCode === 0 && receipt.pngExists, png }, null, 2));
  if (capture.exitCode !== 0) process.exitCode = 1;
}

const list = phase === "all" ? Object.keys(PHASES) : [phase];
for (const p of list) runPhase(p);
