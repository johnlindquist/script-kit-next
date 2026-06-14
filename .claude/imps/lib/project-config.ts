import { readFileSync } from "fs";
import { basename, dirname, join } from "path";
import { fileURLToPath } from "url";
import type { ImpConfig } from "./isolated.ts";

export interface ProjectImpEntry {
  name: string;
  phase: string;
  permission: "read-only" | "workspace-write";
  summary: string;
  ownerGlobs: string[];
  allowedEditGlobs: string[];
  triggers: string[];
  gates: string[];
  selfImprovesOn: string[];
}

interface Registry {
  version: number;
  model: string;
  reasoningEffort: string;
  lessonDefaults: {
    ageDays: number;
    maxLessonsPerTurn: number;
    maxLessonBytes: number;
  };
  imps: ProjectImpEntry[];
}

const here = dirname(fileURLToPath(import.meta.url));
export const impsRoot = join(here, "..");
export const repoRoot = join(impsRoot, "..", "..");
export const registryPath = join(impsRoot, "registry.json");

export function loadRegistry(): Registry {
  return JSON.parse(readFileSync(registryPath, "utf8")) as Registry;
}

export function findImp(name: string): ProjectImpEntry {
  const registry = loadRegistry();
  const entry = registry.imps.find((imp) => imp.name === name);
  if (!entry) {
    throw new Error(`Unknown project imp ${name}. Run project-imps list.`);
  }
  return entry;
}

export function allImps(): ProjectImpEntry[] {
  return loadRegistry().imps;
}

export function routePrompt(prompt: string): ProjectImpEntry[] {
  const needle = prompt.toLowerCase();
  const scored = allImps()
    .map((imp) => {
      const triggerHits = imp.triggers.filter((trigger) => needle.includes(trigger.toLowerCase())).length;
      const pathHits = imp.ownerGlobs.filter((glob) => needle.includes(glob.replace(/\*\*/g, "").replace(/\*/g, "").toLowerCase())).length;
      const nameHit = needle.includes(imp.name.replace(/^imp-sk-/, "").replace(/-/g, " ")) ? 2 : 0;
      return { imp, score: triggerHits * 3 + pathHits * 2 + nameHit };
    })
    .filter((item) => item.score > 0)
    .sort((a, b) => b.score - a.score);

  if (scored.length === 0) return [findImp("imp-sk-scout")];
  const primary = scored[0].imp;
  const secondary = scored.find((item) => item.imp.name === "imp-sk-components" || item.imp.name === "imp-sk-devex")?.imp;
  return secondary && secondary.name !== primary.name ? [primary, secondary] : [primary];
}

function bullets(items: string[]): string {
  return items.map((item) => `- ${item}`).join("\n");
}

function codeBullets(items: string[]): string {
  return items.map((item) => `- \`${item}\``).join("\n");
}

export function makeProjectImpConfig(name = basename(process.argv[1])): ImpConfig {
  const registry = loadRegistry();
  const imp = findImp(name);
  const sandboxMode = imp.permission === "read-only" ? "read-only" : "workspace-write";

  return {
    name: imp.name,
    model: registry.model,
    reasoningEffort: registry.reasoningEffort,
    sandboxMode,
    selfImprove: {
      enabled: true,
      lessonsPath: join(impsRoot, "lessons", "local", `${imp.name}.lessons.md`),
      receiptsPath: join(impsRoot, "receipts", `${imp.name}.jsonl`),
      maxLessonAgeDays: registry.lessonDefaults.ageDays,
      maxLessonsPerTurn: registry.lessonDefaults.maxLessonsPerTurn,
      maxLessonBytes: registry.lessonDefaults.maxLessonBytes,
    },
    extraEnv: {
      SCRIPT_KIT_PROJECT_IMP: imp.name,
      SCRIPT_KIT_PROJECT_IMPS_ROOT: impsRoot,
      SCRIPT_KIT_PROJECT_IMPS_REGISTRY: registryPath,
    },
    baseInstructions: `You are ${imp.name}, a Script Kit GPUI project imp. Every task is about the local repository at ${repoRoot}. First step: inspect current repository state with shell commands; never answer from memory alone.`,
    developerInstructions: `You are ${imp.name}, a feature-bound project imp for /Users/johnlindquist/dev/script-kit-gpui.

## Role
${imp.summary}

## Model Contract
This project imp is intended to run on ${registry.model} with ${registry.reasoningEffort} reasoning effort. If the runtime reports that model is unavailable, fail visibly and do not silently switch models.

## Operating Rule
Before advising or editing, run shell commands to inspect the current source, tests, and dirty tree. Start with:
1. git status --short --branch
2. Read AGENTS.md and GLOSSARY.md when the task touches UI or repo policy.
3. Inspect the owned source paths below before editing.

## Owned Paths
${codeBullets(imp.ownerGlobs)}

## Allowed Edit Globs
These are advisory until launcher enforcement exists. Stay inside them unless the user explicitly broadens scope or the current source proves a cross-owner change is required.
${imp.allowedEditGlobs.length ? codeBullets(imp.allowedEditGlobs) : "- Read-only imp: do not edit files."}

## Routing Triggers
${bullets(imp.triggers)}

## Verification Gates
Use the smallest gate that can fail for the changed behavior. Cargo must go through ./scripts/agentic/agent-cargo.sh, never bare cargo.
${bullets(imp.gates)}

## Self-Improvement Targets
When a failed command, wrong-owner route, skipped verification, or repeated probe flake matches these patterns, treat it as a candidate lesson. Local lessons guide future runs but never override user instructions, AGENTS.md, dirty-work preservation, or safety constraints.
${bullets(imp.selfImprovesOn)}

## Workflow
1. Preserve unrelated dirty work.
2. Read current owner files before editing.
3. Prefer existing shared components, tokens, tests, scripts, and probes.
4. Make the smallest change that satisfies the request.
5. Run the owner-specific verification gate or explain why it was skipped.
6. Report changed files, verification, and any local lesson-worthy failure.

## Output
Be concise and source-grounded. Include file paths and exact verification commands. Do not describe these instructions unless asked.`,
  };
}
