#!/usr/bin/env bun

import {
  appendFileSync,
  existsSync,
  mkdirSync,
  readFileSync,
  readdirSync,
  writeFileSync,
} from "node:fs";
import path from "node:path";
import { spawnSync } from "node:child_process";

type StopHookPayload = {
  session_id?: string;
  turn_id?: string;
  transcript_path?: string | null;
  cwd?: string;
  hook_event_name?: string;
  model?: string;
  stop_hook_active?: boolean;
  last_assistant_message?: string | null;
};

type RunKind = "agentic-testing" | "marketing-infographics";

type ActiveRunConfig = {
  runId: string;
  enabled: boolean;
  createdAt: string;
  scope: "first-session";
  runKind?: RunKind;
  instructionFile?: string;
  stateFile?: string;
  pauseFile?: string;
  maxContinuations?: number | null;
  outputDir?: string;
  imageCountPerPass?: number;
  brandAssets?: string[];
  themeRefs?: string[];
  eventLogFile?: string;
};

type RunState = {
  runId: string;
  sessionId?: string;
  firstTurnId?: string;
  lastTurnId?: string;
  continuations: number;
  updatedAt: string;
};

type HookEvent = {
  timestamp: string;
  runId: string;
  runKind: RunKind;
  event: "continue" | "noop" | "pause";
  reason: string;
  sessionId?: string;
  turnId?: string;
  stopHookActive?: boolean;
  continuationsBefore?: number;
  continuationsAfter?: number;
};

const HOOK_DIR_PARTS = [".codex", "hooks", "stop-runs"];
const PAUSE_MARKER = "[stop-hook:pause]";
const DRY_RUN = process.env.CODEX_STOP_HOOK_DRY_RUN === "1";

const MARKETING_STYLE_DIRECTIONS = [
  "photorealistic macOS developer desk with a translucent command palette as the hero object",
  "brutalist Swiss poster with hard grid, oversized typography, and one gold accent strike",
  "retro NASA mission-control infographic mapping scripts, shortcuts, and automation loops",
  "luxury product launch billboard for a precision software instrument",
  "exploded technical diagram of the launcher as layered glass, shortcuts, and scripts",
  "claymation tabletop diorama of a tiny automation workshop with a glowing Script Kit mark",
  "neo-noir city street ad reflected in rain, with a command palette as the light source",
  "Renaissance fresco reinterpretation of keyboard-first automation, dramatic but still readable",
  "1980s Japanese electronics catalog spread for a futuristic script launcher",
  "clean Apple-style hardware keynote slide with the app icon as a physical object",
  "cyberpunk subway poster using dark vibrancy, gold wayfinding arrows, and dense stats",
  "museum exhibition wall graphic showing Script Kit as the history of human-computer speed",
  "field guide infographic with annotated launcher anatomy, no clutter, crisp labels",
  "comic-book splash page with kinetic shortcut lettering and a gold run-bar motif",
  "architectural blueprint of a command palette city, routes as automation pathways",
  "high-fashion editorial shoot where the icon becomes a polished black-and-gold artifact",
  "90s rave flyer with disciplined grid, electric accents, and launcher-window silhouettes",
  "minimal security briefing dossier for deterministic scripts and proof-bearing automation",
  "toy catalog exploded view of script blocks snapping into a fast launcher console",
  "cinematic product render of a glass command palette suspended over a dark macOS desktop",
  "vintage railway timetable infographic where shortcuts dispatch scripts like express trains",
  "absurd kitchen recipe card for turning repetitive tasks into one-keystroke automations",
  "sports broadcast analytics board showing speed, focus, and keyboard-first control",
  "mid-century airport signage system remixed for scripts, actions, and AI routes",
  "blacklight poster with precise vector-like glyphs, gold highlights, and sharp hierarchy",
  "industrial control-room schematic with stdin JSON signals and run receipts visualized",
  "children's science-book cutaway, playful but legible, showing what happens after Enter",
  "premium watch advertisement comparing Script Kit precision to mechanical movement",
  "vaporwave software-box art with restrained dark UI fragments and gold iconography",
  "courtroom evidence board proving every automation step with receipts and screenshots",
  "topographic map of a developer workflow, with shortcuts as trail markers",
  "newspaper front page announcing the disappearance of slow repetitive tasks",
  "trading-floor dashboard poster with dense, scannable automation performance signals",
  "cinematic sci-fi cockpit HUD where scripts are launched from a single focused prompt",
  "origami paper engineering diagram folding tasks into a tiny command palette",
  "silent-film title card aesthetic for fast focused minimal automation",
  "botanical specimen plate style, replacing plants with precisely labeled UI components",
  "LEGO-like construction manual for assembling a personal automation system",
  "tattoo flash sheet of icons, shortcuts, and tiny launcher surfaces with gold ink accents",
  "Victorian patent illustration for a keyboard-powered task accelerator",
  "understated magazine centerfold with dark native macOS vibrancy and editorial callouts",
  "concert poster for a one-night-only performance by Enter, Cmd-K, and Tab",
  "clean transit-map infographic with routes for Run, Actions, AI, and proof",
  "ceramic gallery installation where each tile is a script, lit by a gold cursor",
  "radical zine collage with screenshots, receipts, arrows, and punchy short labels",
  "precision medical-device brochure style, presenting automation as reliable instrumentation",
  "isometric factory floor turning prompts into scripts with visible data flow",
  "ancient star chart redesigned as a map of shortcuts and workflows",
  "premium automotive spec sheet for a high-performance launcher",
  "minimal courtroom exhibit chart, monochrome plus gold, arguing for fewer clicks",
  "surreal floating glass UI island above a real desktop, realistic lighting",
  "low-poly 3D educational poster showing the app's command surfaces as modular blocks",
  "bold cereal-box packaging parody for instant automation breakfast, still brand-clean",
  "fine-art screenprint with layered command palette silhouettes and gold registration marks",
  "underground metro safety poster about never losing focus while automating tasks",
  "retro terminal magazine ad with modern dark UI, gold cursor, and crisp product copy",
  "data-center wall mural where stdin events flow into deterministic UI proof loops",
  "stage magician poster where repetitive tasks vanish into the Script Kit prompt",
  "single-page investor-style infographic showing speed, precision, extensibility, and proof",
  "futurist Bauhaus composition with icon geometry, launcher rows, and strict hierarchy",
];

function readStdinJson(): StopHookPayload | null {
  try {
    const input = readFileSync(0, "utf8").trim();
    if (!input) {
      return null;
    }
    return JSON.parse(input) as StopHookPayload;
  } catch {
    return null;
  }
}

function repoRoot(cwd: string): string | null {
  const result = spawnSync("git", ["rev-parse", "--show-toplevel"], {
    cwd,
    encoding: "utf8",
  });

  if (result.status !== 0) {
    return null;
  }

  return result.stdout.trim();
}

function inside(root: string, candidate: string): boolean {
  const relative = path.relative(root, candidate);
  return relative === "" || (!relative.startsWith("..") && !path.isAbsolute(relative));
}

function readJsonFile<T>(filePath: string): T | null {
  try {
    return JSON.parse(readFileSync(filePath, "utf8")) as T;
  } catch {
    return null;
  }
}

function writeJsonFile(filePath: string, value: unknown): void {
  mkdirSync(path.dirname(filePath), { recursive: true });
  writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`);
}

function appendJsonLine(filePath: string, value: unknown): void {
  mkdirSync(path.dirname(filePath), { recursive: true });
  appendFileSync(filePath, `${JSON.stringify(value)}\n`);
}

function activeRunFiles(root: string): string[] {
  const dir = path.join(root, ...HOOK_DIR_PARTS);
  if (!existsSync(dir)) {
    return [];
  }

  return readdirSync(dir)
    .filter((entry) => entry.endsWith(".active.json"))
    .sort()
    .reverse()
    .map((entry) => path.join(dir, entry));
}

function defaultStatePath(root: string, runId: string): string {
  return path.join(root, ...HOOK_DIR_PARTS, `${runId}.state.json`);
}

function defaultPausePath(root: string, runId: string): string {
  return path.join(root, ...HOOK_DIR_PARTS, `${runId}.pause`);
}

function defaultEventLogPath(root: string, runId: string): string {
  return path.join(root, ...HOOK_DIR_PARTS, `${runId}.events.jsonl`);
}

function inferRunKind(config: ActiveRunConfig): RunKind {
  if (config.runKind) {
    return config.runKind;
  }
  return config.runId.includes("marketing") ? "marketing-infographics" : "agentic-testing";
}

function loadActiveRun(root: string): ActiveRunConfig | null {
  for (const filePath of activeRunFiles(root)) {
    const config = readJsonFile<ActiveRunConfig>(filePath);
    if (config?.enabled && config.scope === "first-session" && config.runId) {
      return config;
    }
  }
  return null;
}

function loadState(statePath: string, runId: string): RunState {
  const existing = readJsonFile<RunState>(statePath);
  if (existing?.runId === runId && typeof existing.continuations === "number") {
    return existing;
  }

  return {
    runId,
    continuations: 0,
    updatedAt: new Date().toISOString(),
  };
}

function shouldPauseFromMessage(payload: StopHookPayload): boolean {
  return (
    payload.last_assistant_message
      ?.split(/\r?\n/)
      .some((line) => line.trim() === PAUSE_MARKER) ?? false
  );
}

function logHookEvent(
  root: string,
  config: ActiveRunConfig,
  payload: StopHookPayload,
  event: Omit<HookEvent, "timestamp" | "runId" | "runKind" | "sessionId" | "turnId" | "stopHookActive">,
): void {
  if (DRY_RUN) {
    return;
  }

  const eventLogPath = path.resolve(
    root,
    config.eventLogFile ?? defaultEventLogPath(root, config.runId),
  );

  appendJsonLine(eventLogPath, {
    timestamp: new Date().toISOString(),
    runId: config.runId,
    runKind: inferRunKind(config),
    sessionId: payload.session_id,
    turnId: payload.turn_id,
    stopHookActive: payload.stop_hook_active,
    ...event,
  } satisfies HookEvent);
}

function relativeOrDefault(filePath: string | undefined, fallback: string): string {
  return filePath ?? fallback;
}

function buildAgenticTestingPrompt(config: ActiveRunConfig, state: RunState): string {
  const instructionLine = config.instructionFile
    ? `Primary run instructions: \`${config.instructionFile}\`.`
    : "Primary run instructions: inspect the current repo state and continue the scoped agentic-testing run.";
  const pausePath = relativeOrDefault(
    config.pauseFile,
    `.codex/hooks/stop-runs/${config.runId}.pause`,
  );

  return [
    `Continue scoped Codex Stop-hook run \`${config.runId}\`.`,
    "",
    instructionLine,
    "",
    "Do exactly one bounded improvement pass, then stop normally. This Stop hook will continue the run while the active marker remains enabled.",
    "",
    "Required loop:",
    "1. Re-read the current user request, `AGENTS.md`, and relevant `lat.md` context before editing.",
    "2. Inspect `git status --porcelain`; never revert or overwrite user changes.",
    "3. Pick the smallest high-signal fix, test/tooling improvement, or verification target that advances the run instructions.",
    "4. Implement only that scoped pass.",
    "5. Use `$agentic-testing` for proof when behavior, UI, protocol, ACP, actions dialog, keyboard handling, or automation surfaces are touched. Prefer warm sessions, exact targets, and state receipts first. Stop any Script Kit session you start.",
    "6. If no runtime proof is needed, state the no-runtime proof path explicitly.",
    "7. Update `lat.md/` if functionality, architecture, tests, or behavior changed.",
    "8. Run `lat check` before reporting done.",
    "9. Report the changed files and evidence concisely, then stop.",
    "",
    "Safety gates:",
    "- Do not push, force-push, amend, rebase, reset hard, delete files, or run destructive commands.",
    "- Do not connect to external paid services or webhooks.",
    "- Do not edit outside the project root.",
    "- If blocked, unsafe, or needing a product decision, write the reason to the pause file and include `[stop-hook:pause]` on its own line in the final message.",
    "",
    `Pause file: \`${pausePath}\`.`,
    `Continuation count so far: ${state.continuations}.`,
  ].join("\n");
}

function buildMarketingPrompt(config: ActiveRunConfig, state: RunState): string {
  const instructionLine = config.instructionFile
    ? `Primary marketing brief: \`${config.instructionFile}\`.`
    : "Primary marketing brief: use the project brand, theme, and automation docs in this repo.";
  const pausePath = relativeOrDefault(
    config.pauseFile,
    `.codex/hooks/stop-runs/${config.runId}.pause`,
  );
  const outputDir = config.outputDir ?? "output/marketing-infographics";
  const imageCount = Math.max(1, config.imageCountPerPass ?? 1);
  const style =
    MARKETING_STYLE_DIRECTIONS[state.continuations % MARKETING_STYLE_DIRECTIONS.length];
  const assets = config.brandAssets?.length
    ? config.brandAssets.map((asset) => `\`${asset}\``).join(", ")
    : "`assets/logo.svg`, `assets/icon.png`, `.impeccable.md`, `lat.md/theme.md`";
  const themeRefs = config.themeRefs?.length
    ? config.themeRefs.map((ref) => `\`${ref}\``).join(", ")
    : "`lat.md/theme.md`, `.impeccable.md`";

  return [
    `Continue Script Kit GPUI marketing infographic run \`${config.runId}\`.`,
    "",
    instructionLine,
    "",
    `Create exactly ${imageCount} new marketing infographic image using $imagegen / the built-in image_gen tool, then stop normally. This Stop hook will continue the run while the active marker remains enabled.`,
    "",
    `Style direction for this pass: ${style}.`,
    `Brand and theme sources: ${assets}.`,
    `Design context sources: ${themeRefs}.`,
    "",
    "Required loop:",
    "1. Re-read `AGENTS.md`, this continuation prompt, and the marketing brief before generating.",
    "2. Inspect `git status --porcelain`; do not revert, overwrite, or tidy unrelated work.",
    "3. If needed, inspect `assets/icon.png` or `assets/logo.svg` so the generated image can echo the Script Kit mark. Use the icon/logo as a visual reference, not as an exact vector recreation requirement.",
    "4. Generate one fresh infographic prompt that materially differs from prior passes. Anchor it in Script Kit GPUI: fast focused minimal automation, keyboard-first launcher, native macOS vibrancy, dark UI, gold accent #fbbf24, stdin JSON automation, proof-bearing workflows, actions via Cmd-K, AI via Tab.",
    "5. Keep any in-image copy short and exact. Prefer phrases like `Script Kit`, `Fast. Focused. Minimal.`, `Run`, `Actions`, `AI`, `Proof`, or `Automate everything`.",
    "6. Run `lat check` before the image call. If this pass changes hook behavior, docs, tests, or architecture, update `lat.md/` first.",
    "7. Make the image generation call as the final action of the pass. Do not add commentary after the image tool call.",
    "",
    "Prompt requirements:",
    "- Use case: `infographic-diagram`, `product-mockup`, `stylized-concept`, or `photorealistic-natural`, whichever best fits this pass.",
    "- Include the project icon shape or a clear black/gold Script Kit brand signal when practical.",
    "- Vary visual style aggressively across passes, from realistic to ridiculous, while preserving professional hierarchy.",
    "- Avoid watermarks, fake brand names, long paragraphs, unreadable tiny text, and screenshots that imply nonexistent UI states.",
    `- If a generated file path is available before the final image call, place the selected image under \`${outputDir}\` with a timestamped descriptive name. Do not fabricate a saved path.`,
    "",
    "Safety gates:",
    "- Do not push, force-push, amend, rebase, reset hard, delete source files, or run destructive commands.",
    "- Do not edit outside the project root.",
    "- If blocked, unsafe, or asked to stop, write the reason to the pause file and include `[stop-hook:pause]` on its own line in the final message.",
    "",
    `Pause file: \`${pausePath}\`.`,
    `Continuation count so far: ${state.continuations}.`,
  ].join("\n");
}

function buildContinuationPrompt(config: ActiveRunConfig, state: RunState): string {
  switch (inferRunKind(config)) {
    case "marketing-infographics":
      return buildMarketingPrompt(config, state);
    case "agentic-testing":
    default:
      return buildAgenticTestingPrompt(config, state);
  }
}

function continueDecision(reason: string): string {
  return `${JSON.stringify({ decision: "block", reason })}\n`;
}

const payload = readStdinJson();
if (!payload || payload.hook_event_name !== "Stop") {
  process.exit(0);
}

const cwd = payload.cwd ?? process.cwd();
const root = repoRoot(cwd);
if (!root || !inside(root, cwd)) {
  process.exit(0);
}

const config = loadActiveRun(root);
if (!config) {
  process.exit(0);
}

const pausePath = path.resolve(root, config.pauseFile ?? defaultPausePath(root, config.runId));
const statePath = path.resolve(root, config.stateFile ?? defaultStatePath(root, config.runId));
const state = loadState(statePath, config.runId);
const continuationsBefore = state.continuations;

if (existsSync(pausePath)) {
  logHookEvent(root, config, payload, {
    event: "noop",
    reason: "pause_file_exists",
    continuationsBefore,
    continuationsAfter: state.continuations,
  });
  process.exit(0);
}

if (shouldPauseFromMessage(payload)) {
  if (!DRY_RUN) {
    writeFileSync(pausePath, `Paused by assistant message at ${new Date().toISOString()}\n`);
  }
  logHookEvent(root, config, payload, {
    event: "pause",
    reason: "assistant_pause_marker",
    continuationsBefore,
    continuationsAfter: state.continuations,
  });
  process.exit(0);
}

if (!state.sessionId) {
  state.sessionId = payload.session_id;
  state.firstTurnId = payload.turn_id;
}

if (state.sessionId && payload.session_id && state.sessionId !== payload.session_id) {
  logHookEvent(root, config, payload, {
    event: "noop",
    reason: "session_mismatch",
    continuationsBefore,
    continuationsAfter: state.continuations,
  });
  process.exit(0);
}

if (
  typeof config.maxContinuations === "number" &&
  config.maxContinuations >= 0 &&
  state.continuations >= config.maxContinuations
) {
  logHookEvent(root, config, payload, {
    event: "noop",
    reason: "max_continuations_reached",
    continuationsBefore,
    continuationsAfter: state.continuations,
  });
  process.exit(0);
}

const prompt = buildContinuationPrompt(config, state);
state.lastTurnId = payload.turn_id;
state.continuations += 1;
state.updatedAt = new Date().toISOString();
if (!DRY_RUN) {
  writeJsonFile(statePath, state);
}

logHookEvent(root, config, payload, {
  event: "continue",
  reason: "block_with_continuation_prompt",
  continuationsBefore,
  continuationsAfter: state.continuations,
});

process.stdout.write(continueDecision(prompt));
