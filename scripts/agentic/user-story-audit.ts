#!/usr/bin/env bun

import { mkdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";

type StoryResultStatus = "pass" | "fail_closed" | "runtime_failure" | "timeout";

type StoryResult = {
  id: string;
  recipe: string;
  story: string;
  command: string[];
  status: StoryResultStatus;
  exitCode: number | null;
  durationMs: number;
  summary: string | null;
  missingReceipt: string | null;
  failClosed: boolean;
  failureCode: string | null;
  warnings: string[];
  outputPreview: string;
};

const ALREADY_EXERCISED_THIS_THREAD = new Set([
  "visible-text-clipping-overlap-stress",
  "layout-measurement-regression-stress",
  "screenshot-semantics-visual-consistency-stress",
  "visual-contrast-readable-state-stress",
  "long-text-wrap-resize-surface-stress",
  "div-container-scroll-overflow-stress",
  "main-menu-dynamic-choice-resize-stress",
  "notes-window-resize-stress",
  "actions-command-discoverability-noop-stress",
]);

function parseArgs(argv: string[]) {
  let limit = 100;
  let maxMs = 30_000;
  let includeKnown = false;
  let dryRun = false;

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--limit") {
      limit = Number(argv[++i] ?? limit);
    } else if (arg === "--max-ms") {
      maxMs = Number(argv[++i] ?? maxMs);
    } else if (arg === "--include-known") {
      includeKnown = true;
    } else if (arg === "--dry-run") {
      dryRun = true;
    }
  }

  return { limit, maxMs, includeKnown, dryRun };
}

function humanizeRecipe(recipe: string) {
  return recipe
    .replace(/-stress$/, "")
    .replace(/-/g, " ")
    .replace(/\bacp\b/g, "ACP")
    .replace(/\bux\b/g, "UX")
    .replace(/\bui\b/g, "UI");
}

function storyForRecipe(recipe: string) {
  const subject = humanizeRecipe(recipe);
  return `As a Script Kit user exercising ${subject}, I expect visible state, focus, layout, ownership, and cleanup to stay coherent while I interact with the app.`;
}

function extractStressRecipes(indexSource: string) {
  const recipes = new Set<string>();
  const pattern = /case "([^"]+-stress)"/g;
  let match: RegExpExecArray | null;
  while ((match = pattern.exec(indexSource)) !== null) {
    recipes.add(match[1]);
  }
  return [...recipes];
}

function extractJsonObject(output: string): any | null {
  for (let index = output.lastIndexOf("{"); index >= 0; index = output.lastIndexOf("{", index - 1)) {
    const candidate = output.slice(index).trim();
    try {
      return JSON.parse(candidate);
    } catch {
      // Keep walking backward until the final pretty-printed receipt parses.
    }
  }
  return null;
}

function classify(parsed: any | null, exitCode: number | null, timedOut: boolean): StoryResultStatus {
  if (timedOut) return "timeout";
  if (parsed?.status === "pass") return "pass";
  if (parsed?.failClosed === true || parsed?.failureMode === "fail_closed") return "fail_closed";
  if (exitCode === 0 && parsed?.status === "pass") return "pass";
  return "runtime_failure";
}

async function runStory(recipe: string, index: number, maxMs: number): Promise<StoryResult> {
  const session = `story-audit-${Date.now()}-${String(index + 1).padStart(3, "0")}`;
  const command = ["bun", "scripts/agentic/index.ts", recipe, "--session", session, "--json"];
  const started = Date.now();
  const proc = Bun.spawn(command, {
    stdout: "pipe",
    stderr: "pipe",
    env: {
      ...Bun.env,
      SCRIPT_KIT_AGENTIC_AUDIT: "100-user-stories",
    },
  });

  let timedOut = false;
  const timeout = setTimeout(() => {
    timedOut = true;
    proc.kill();
  }, maxMs);

  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited.catch(() => null),
  ]);
  clearTimeout(timeout);

  const output = `${stdout}\n${stderr}`.trim();
  const parsed = extractJsonObject(output);
  const status = classify(parsed, exitCode, timedOut);

  return {
    id: `ux-story-${String(index + 1).padStart(3, "0")}`,
    recipe,
    story: storyForRecipe(recipe),
    command,
    status,
    exitCode,
    durationMs: Date.now() - started,
    summary: parsed?.summary ?? null,
    missingReceipt: parsed?.missingReceipt ?? parsed?.proofBundle?.missingReceipt ?? null,
    failClosed: parsed?.failClosed === true || parsed?.proofBundle?.failClosed === true,
    failureCode: parsed?.failure?.code ?? parsed?.proofBundle?.failure?.code ?? null,
    warnings: parsed?.proofBundle?.warnings ?? parsed?.warnings ?? [],
    outputPreview: output.slice(0, 1600),
  };
}

async function main() {
  const { limit, maxMs, includeKnown, dryRun } = parseArgs(Bun.argv.slice(2));
  const indexSource = await readFile("scripts/agentic/index.ts", "utf8");
  const candidates = extractStressRecipes(indexSource)
    .filter((recipe) => includeKnown || !ALREADY_EXERCISED_THIS_THREAD.has(recipe))
    .slice(0, limit);

  if (candidates.length < limit) {
    throw new Error(`Only found ${candidates.length} eligible stress recipes for limit ${limit}`);
  }

  const startedAt = new Date().toISOString();
  const results: StoryResult[] = [];

  if (!dryRun) {
    for (const [index, recipe] of candidates.entries()) {
      const result = await runStory(recipe, index, maxMs);
      results.push(result);
      console.error(`${result.id} ${result.status} ${recipe}`);
    }
  }

  const statusCounts = results.reduce<Record<string, number>>((acc, result) => {
    acc[result.status] = (acc[result.status] ?? 0) + 1;
    return acc;
  }, {});

  const artifact = {
    schemaVersion: 1,
    audit: "agentic-100-user-story-ux-audit",
    startedAt,
    completedAt: new Date().toISOString(),
    requestedStoryCount: limit,
    selectedStoryCount: candidates.length,
    skippedAlreadyExercisedThisThread: includeKnown ? [] : [...ALREADY_EXERCISED_THIS_THREAD],
    dryRun,
    maxMs,
    statusCounts,
    stories: dryRun
      ? candidates.map((recipe, index) => ({
          id: `ux-story-${String(index + 1).padStart(3, "0")}`,
          recipe,
          story: storyForRecipe(recipe),
        }))
      : results,
  };

  await mkdir(".test-output", { recursive: true });
  const artifactPath = join(".test-output", `agentic-100-user-story-audit-${startedAt.replace(/[:.]/g, "-")}.json`);
  await writeFile(artifactPath, `${JSON.stringify(artifact, null, 2)}\n`);
  console.log(JSON.stringify({ ...artifact, artifactPath }, null, 2));
}

await main();
