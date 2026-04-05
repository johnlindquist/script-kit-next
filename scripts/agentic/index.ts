#!/usr/bin/env bun
/**
 * scripts/agentic/index.ts
 *
 * Phase-2 thin wrapper over the lower-level agentic helpers.
 * Orchestrates common multi-step flows without hiding the underlying
 * proof receipts from each tool.
 *
 * Usage:
 *   bun scripts/agentic/index.ts <recipe> [--session NAME] [--json]
 *
 * Recipes:
 *   acp-enter-accept   Run the ACP picker-accept-via-Enter golden path
 *   acp-tab-accept     Run the ACP picker-accept-via-Tab golden path
 *   acp-open           Open ACP and verify it reaches ready state
 *   preflight          Check all prerequisites (session, window, permissions)
 *   help               Show this help
 *
 * All output is JSON on stdout. Each recipe returns the underlying
 * tool receipts so the agent can inspect proof at every step.
 */

import { resolve } from "path";

const SCHEMA_VERSION = 1;
const PROJECT_ROOT = resolve(import.meta.dir, "../..");

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface RecipeReceipt {
  schemaVersion: number;
  recipe: string;
  status: "pass" | "fail" | "error";
  steps: StepReceipt[];
  summary: string;
}

interface StepReceipt {
  name: string;
  status: "pass" | "fail" | "error" | "skipped";
  output: unknown;
  durationMs: number;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

async function runTool(
  cmd: string[],
  _label: string
): Promise<{ exitCode: number; stdout: string; stderr: string }> {
  const proc = Bun.spawn(cmd, {
    stdout: "pipe",
    stderr: "pipe",
    cwd: PROJECT_ROOT,
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const exitCode = await proc.exited;
  return { exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
}

function parseJson(text: string): unknown {
  try {
    return JSON.parse(text);
  } catch {
    return { raw: text };
  }
}

async function step(
  name: string,
  fn: () => Promise<{ exitCode: number; stdout: string }>
): Promise<StepReceipt> {
  const start = Date.now();
  try {
    const { exitCode, stdout } = await fn();
    return {
      name,
      status: exitCode === 0 ? "pass" : exitCode === 2 ? "error" : "fail",
      output: parseJson(stdout),
      durationMs: Date.now() - start,
    };
  } catch (e: any) {
    return {
      name,
      status: "error",
      output: { error: e.message ?? String(e) },
      durationMs: Date.now() - start,
    };
  }
}

/**
 * Send a protocol command via session.sh rpc and return structured result.
 * Surfaces the full waitForResult / batchResult trace receipt on failure.
 */
async function rpc(
  session: string,
  jsonCmd: string,
  opts: { expect?: string; timeout?: number } = {}
): Promise<{ exitCode: number; stdout: string; stderr: string }> {
  const args = [
    "bash",
    "scripts/agentic/session.sh",
    "rpc",
    session,
    jsonCmd,
  ];
  if (opts.expect) {
    args.push("--expect", opts.expect);
  }
  if (opts.timeout) {
    args.push("--timeout", String(opts.timeout));
  }
  return runTool(args, "rpc");
}

/**
 * Fire-and-forget send via session.sh send.
 */
async function send(
  session: string,
  jsonCmd: string
): Promise<{ exitCode: number; stdout: string; stderr: string }> {
  return runTool(
    ["bash", "scripts/agentic/session.sh", "send", session, jsonCmd],
    "send"
  );
}

function parseArgs() {
  const args = process.argv.slice(2);
  const recipe = args[0] ?? "help";
  const sessionIdx = args.indexOf("--session");
  const session =
    sessionIdx >= 0 && args[sessionIdx + 1] ? args[sessionIdx + 1] : "default";
  return { recipe, session };
}

// ---------------------------------------------------------------------------
// Recipes
// ---------------------------------------------------------------------------

async function recipePreflight(session: string): Promise<RecipeReceipt> {
  const steps: StepReceipt[] = [];

  // Check session health
  steps.push(
    await step("session-status", () =>
      runTool(
        ["bash", "scripts/agentic/session.sh", "status", session],
        "session-status"
      )
    )
  );

  // Check window status
  steps.push(
    await step("window-status", () =>
      runTool(["bun", "scripts/agentic/window.ts", "status"], "window-status")
    )
  );

  // Check native input prerequisites
  steps.push(
    await step("input-check", () =>
      runTool(
        ["bun", "scripts/agentic/macos-input.ts", "check"],
        "input-check"
      )
    )
  );

  const allPass = steps.every((s) => s.status === "pass");
  return {
    schemaVersion: SCHEMA_VERSION,
    recipe: "preflight",
    status: allPass ? "pass" : "fail",
    steps,
    summary: allPass
      ? "All prerequisites met"
      : `Failed: ${steps
          .filter((s) => s.status !== "pass")
          .map((s) => s.name)
          .join(", ")}`,
  };
}

async function recipeAcpOpen(session: string): Promise<RecipeReceipt> {
  const steps: StepReceipt[] = [];

  // 1. Show window + trigger ACP in one batch, then waitFor acpReady
  steps.push(
    await step("show-and-trigger-acp", () =>
      send(session, '{"type":"show"}')
    )
  );

  // macOS focus-settling delay: the window needs a moment to
  // become frontmost after show before triggerBuiltin can target it.
  await Bun.sleep(300);

  steps.push(
    await step("trigger-acp", () =>
      send(session, '{"type":"triggerBuiltin","name":"tab-ai"}')
    )
  );

  // 2. Wait for ACP to be ready using waitFor instead of fixed sleep
  steps.push(
    await step("wait-acp-ready", () =>
      rpc(
        session,
        JSON.stringify({
          type: "waitFor",
          requestId: "w-acp-ready",
          condition: { type: "acpReady" },
          timeout: 8000,
          pollInterval: 25,
          trace: "onFailure",
        }),
        { expect: "waitForResult", timeout: 10000 }
      )
    )
  );

  // 3. Focus window
  steps.push(
    await step("focus-window", () =>
      runTool(["bun", "scripts/agentic/window.ts", "focus"], "focus")
    )
  );

  // 4. Verify ACP ready via verify-shot (state receipt before screenshot)
  steps.push(
    await step("verify-acp-ready", () =>
      runTool(
        [
          "bun",
          "scripts/agentic/verify-shot.ts",
          "--session",
          session,
          "--label",
          "acp-open",
          "--acp-context-ready",
        ],
        "verify-ready"
      )
    )
  );

  const allPass = steps.every((s) => s.status === "pass");
  return {
    schemaVersion: SCHEMA_VERSION,
    recipe: "acp-open",
    status: allPass ? "pass" : steps.some((s) => s.status === "error") ? "error" : "fail",
    steps,
    summary: allPass
      ? "ACP opened and context ready"
      : `Failed at: ${steps
          .filter((s) => s.status !== "pass")
          .map((s) => s.name)
          .join(", ")}`,
  };
}

async function recipeAcpPickerAccept(
  session: string,
  acceptKey: "enter" | "tab"
): Promise<RecipeReceipt> {
  const steps: StepReceipt[] = [];

  // 1. Open ACP first
  const openResult = await recipeAcpOpen(session);
  steps.push({
    name: "acp-open",
    status: openResult.status,
    output: openResult,
    durationMs: openResult.steps.reduce((sum, s) => sum + s.durationMs, 0),
  });

  if (openResult.status !== "pass") {
    return {
      schemaVersion: SCHEMA_VERSION,
      recipe: `acp-${acceptKey}-accept`,
      status: "error",
      steps,
      summary: "Cannot proceed: ACP open failed",
    };
  }

  // 2. Type @ to open picker (native input)
  steps.push(
    await step("type-at-trigger", () =>
      runTool(
        ["bun", "scripts/agentic/macos-input.ts", "type", "@"],
        "type-at"
      )
    )
  );

  // 3. Wait for picker to open using waitFor instead of fixed sleep
  steps.push(
    await step("wait-picker-open", () =>
      rpc(
        session,
        JSON.stringify({
          type: "waitFor",
          requestId: "w-picker-open",
          condition: { type: "acpPickerOpen" },
          timeout: 3000,
          pollInterval: 25,
          trace: "onFailure",
        }),
        { expect: "waitForResult", timeout: 5000 }
      )
    )
  );

  // 4. Verify picker opened (state receipt first)
  steps.push(
    await step("verify-picker-open", () =>
      runTool(
        [
          "bun",
          "scripts/agentic/verify-shot.ts",
          "--session",
          session,
          "--label",
          "picker-open",
          "--acp-picker-open",
        ],
        "verify-picker"
      )
    )
  );

  // 5. Accept with native key
  steps.push(
    await step(`native-${acceptKey}`, () =>
      runTool(
        ["bun", "scripts/agentic/macos-input.ts", "key", acceptKey],
        `native-${acceptKey}`
      )
    )
  );

  // 6. Wait for picker to close and item to be accepted
  steps.push(
    await step("wait-item-accepted", () =>
      rpc(
        session,
        JSON.stringify({
          type: "waitFor",
          requestId: "w-item-accepted",
          condition: { type: "acpItemAccepted" },
          timeout: 3000,
          pollInterval: 25,
          trace: "onFailure",
        }),
        { expect: "waitForResult", timeout: 5000 }
      )
    )
  );

  // 7. Verify picker closed + item accepted (state receipt + screenshot)
  steps.push(
    await step("verify-accepted", () =>
      runTool(
        [
          "bun",
          "scripts/agentic/verify-shot.ts",
          "--session",
          session,
          "--label",
          `${acceptKey}-accepted`,
          "--acp-picker-closed",
          "--acp-item-accepted",
        ],
        "verify-accepted"
      )
    )
  );

  const allPass = steps.every((s) => s.status === "pass");
  return {
    schemaVersion: SCHEMA_VERSION,
    recipe: `acp-${acceptKey}-accept`,
    status: allPass ? "pass" : steps.some((s) => s.status === "error") ? "error" : "fail",
    steps,
    summary: allPass
      ? `ACP picker accepted via ${acceptKey}`
      : `Failed at: ${steps
          .filter((s) => s.status !== "pass")
          .map((s) => s.name)
          .join(", ")}`,
  };
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const { recipe, session } = parseArgs();

let result: RecipeReceipt;

switch (recipe) {
  case "preflight":
    result = await recipePreflight(session);
    break;

  case "acp-open":
    result = await recipeAcpOpen(session);
    break;

  case "acp-enter-accept":
    result = await recipeAcpPickerAccept(session, "enter");
    break;

  case "acp-tab-accept":
    result = await recipeAcpPickerAccept(session, "tab");
    break;

  case "help":
  case "--help":
    console.log(`Usage: bun scripts/agentic/index.ts <recipe> [--session NAME]

Recipes:
  preflight          Check prerequisites (session, window, permissions)
  acp-open           Open ACP and verify ready state
  acp-enter-accept   Full ACP picker accept via Enter golden path
  acp-tab-accept     Full ACP picker accept via Tab golden path
  help               Show this help

Each recipe returns a JSON receipt with per-step proof from the underlying tools.
The wrapper is intentionally thin — it does not replace the lower-level commands.

Examples:
  bun scripts/agentic/index.ts preflight --session default
  bun scripts/agentic/index.ts acp-enter-accept --session default --json`);
    process.exit(0);
    break;

  default:
    result = {
      schemaVersion: SCHEMA_VERSION,
      recipe,
      status: "error",
      steps: [],
      summary: `Unknown recipe: ${recipe}. Run with 'help' for options.`,
    };
    break;
}

console.log(JSON.stringify(result!, null, 2));
process.exit(result!.status === "pass" ? 0 : result!.status === "error" ? 2 : 1);
