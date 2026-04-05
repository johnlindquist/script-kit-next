#!/usr/bin/env bun
/**
 * scripts/agentic/index.ts
 *
 * Thin orchestrator over the lower-level agentic helpers.
 * Orchestrates common multi-step flows without hiding the underlying
 * proof receipts from each tool.
 *
 * Usage:
 *   bun scripts/agentic/index.ts <recipe> [--session NAME] [--key enter|tab] [--vision]
 *     [--target-json '{"type":"kind","kind":"acpDetached","index":0}'] [--surface acp]
 *
 * Recipes:
 *   acp-accept         Full ACP picker accept; choose key with --key enter|tab
 *   acp-enter-accept   Compatibility alias for --key enter
 *   acp-tab-accept     Compatibility alias for --key tab
 *   acp-open           Open ACP and verify it reaches ready state
 *   acp-setup-recovery Recovery from ACP setup state; select agent with --select-agent ID
 *   preflight          Check all prerequisites (session, window, permissions)
 *   help               Show this help
 *
 * Target threading:
 *   --target-json JSON   ACP window target for all RPCs (getAcpState, getAcpTestProbe,
 *                        resetAcpTestProbe, waitFor). Reused consistently across all steps.
 *   --surface SURFACE    Automation surface for native input focus (main, acp, actions, notes, ai).
 *                        Must match the --target-json window so focus and proof stay on the same surface.
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

/**
 * Wire-compatible ACP window target. Same shape as Rust `AutomationWindowTarget`.
 * One target object must be reused for every RPC in a single verification run.
 */
type AutomationTargetJson =
  | { type: "focused" }
  | { type: "main" }
  | { type: "id"; id: string }
  | { type: "kind"; kind: string; index?: number }
  | { type: "titleContains"; text: string };

interface RecipeReceipt {
  schemaVersion: number;
  recipe: string;
  status: "pass" | "fail" | "error";
  steps: StepReceipt[];
  summary: string;
  /** When --vision is requested, the final verify-shot proof bundle is surfaced here unchanged. */
  proofBundle?: unknown;
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
 * Build a JSON command string, injecting `target` when present.
 */
function buildCmd(
  base: Record<string, unknown>,
  target?: AutomationTargetJson
): string {
  if (target) {
    return JSON.stringify({ ...base, target });
  }
  return JSON.stringify(base);
}

/**
 * Build native-input args with optional --surface and --ensure-focus.
 */
function nativeInputArgs(
  command: string,
  value: string,
  surface?: string
): string[] {
  const args = [
    "bun",
    "scripts/agentic/macos-input.ts",
    command,
    value,
    "--ensure-focus",
  ];
  if (surface) {
    args.push("--target", surface);
  }
  return args;
}

/**
 * Build verify-shot args with optional --target-json.
 */
function verifyArgs(
  base: string[],
  target?: AutomationTargetJson
): string[] {
  if (target) {
    return [...base, "--target-json", JSON.stringify(target)];
  }
  return base;
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

function parseTargetJson(raw: string | undefined): AutomationTargetJson | undefined {
  if (!raw) return undefined;
  try {
    return JSON.parse(raw) as AutomationTargetJson;
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    throw new Error(`Invalid --target-json: ${reason}`);
  }
}

function parseArgs() {
  const args = process.argv.slice(2);
  const recipe = args[0] ?? "help";
  const sessionIdx = args.indexOf("--session");
  const session =
    sessionIdx >= 0 && args[sessionIdx + 1] ? args[sessionIdx + 1] : "default";
  const keyIdx = args.indexOf("--key");
  const key =
    keyIdx >= 0 &&
    (args[keyIdx + 1] === "enter" || args[keyIdx + 1] === "tab")
      ? (args[keyIdx + 1] as "enter" | "tab")
      : "enter";
  const vision = args.includes("--vision");
  const selectAgentIdx = args.indexOf("--select-agent");
  const selectAgent =
    selectAgentIdx >= 0 && args[selectAgentIdx + 1]
      ? args[selectAgentIdx + 1]
      : undefined;
  const targetJsonIdx = args.indexOf("--target-json");
  const targetJson = parseTargetJson(
    targetJsonIdx >= 0 ? args[targetJsonIdx + 1] : undefined
  );
  const surfaceIdx = args.indexOf("--surface");
  const surface =
    surfaceIdx >= 0 && args[surfaceIdx + 1] ? args[surfaceIdx + 1] : undefined;
  const json = args.includes("--json");
  return { recipe, session, key, vision, selectAgent, targetJson, surface, json };
}

// ---------------------------------------------------------------------------
// Recipes
// ---------------------------------------------------------------------------

async function recipePreflight(session: string): Promise<RecipeReceipt> {
  const steps: StepReceipt[] = [];

  // Check session health via session.sh status
  const sessionStatusStep = await step("session-status", () =>
    runTool(
      ["bash", "scripts/agentic/session.sh", "status", session],
      "session-status"
    )
  );

  // Parse the session JSON and enforce health invariants
  const sessionJson = sessionStatusStep.output as Record<string, unknown> | null;
  if (sessionJson && typeof sessionJson === "object" && !("raw" in sessionJson)) {
    const status = sessionJson.status as string | undefined;
    const alive = sessionJson.alive as boolean | undefined;
    const forwarderAlive = sessionJson.forwarderAlive as boolean | undefined;
    const healthy = sessionJson.healthy as boolean | undefined;

    if (
      status === "not_found" ||
      alive === false ||
      forwarderAlive === false ||
      healthy === false
    ) {
      const issues = (sessionJson.issues as string[]) ?? [];
      sessionStatusStep.status = "fail";
      sessionStatusStep.output = {
        ...sessionJson,
        _preflightVerdict: "unhealthy",
        _failReasons: [
          ...(status === "not_found" ? ["status:not_found"] : []),
          ...(alive === false ? ["alive:false"] : []),
          ...(forwarderAlive === false ? ["forwarderAlive:false"] : []),
          ...(healthy === false ? ["healthy:false"] : []),
          ...issues.map((i: string) => `issue:${i}`),
        ],
      };
    }
  }
  steps.push(sessionStatusStep);

  // Check session health via session-state.ts (cross-validates)
  const sessionStateStep = await step("session-state", () =>
    runTool(
      ["bun", "scripts/agentic/session-state.ts", "--session", session],
      "session-state"
    )
  );
  // session-state.ts already exits non-zero when unhealthy, so step() maps that
  steps.push(sessionStateStep);

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

async function recipeAcpOpen(
  session: string,
  opts: { target?: AutomationTargetJson } = {}
): Promise<RecipeReceipt> {
  const steps: StepReceipt[] = [];

  // 1. Show window
  steps.push(
    await step("show", () => send(session, '{"type":"show"}'))
  );

  // macOS focus-settling delay: the window needs a moment to
  // become frontmost after show before triggerBuiltin can target it.
  await Bun.sleep(300);

  // 2. Trigger ACP
  steps.push(
    await step("trigger-acp", () =>
      send(session, '{"type":"triggerBuiltin","name":"tab-ai"}')
    )
  );

  // 3. Wait for ACP to be ready using waitFor instead of fixed sleep
  steps.push(
    await step("wait-acp-ready", () =>
      rpc(
        session,
        buildCmd(
          {
            type: "waitFor",
            requestId: "w-acp-ready",
            condition: { type: "acpReady" },
            timeout: 8000,
            pollInterval: 25,
            trace: "onFailure",
          },
          opts.target
        ),
        { expect: "waitForResult", timeout: 10000 }
      )
    )
  );

  // 4. State-only verification: no screenshot, no probe
  steps.push(
    await step("verify-acp-ready", () =>
      runTool(
        verifyArgs(
          [
            "bun",
            "scripts/agentic/verify-shot.ts",
            "--session",
            session,
            "--label",
            "acp-open",
            "--skip-screenshot",
            "--skip-probe",
            "--acp-context-ready",
          ],
          opts.target
        ),
        "verify-ready"
      )
    )
  );

  const allPass = steps.every((s) => s.status === "pass");
  return {
    schemaVersion: SCHEMA_VERSION,
    recipe: "acp-open",
    status: allPass
      ? "pass"
      : steps.some((s) => s.status === "error")
        ? "error"
        : "fail",
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
  acceptKey: "enter" | "tab",
  opts: { emitVision?: boolean; target?: AutomationTargetJson; surface?: string } = {}
): Promise<RecipeReceipt> {
  const steps: StepReceipt[] = [];

  // 1. Open ACP first
  const openResult = await recipeAcpOpen(session, { target: opts.target });
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

  // 2. Reset probe before native interaction to avoid stale accepted items
  steps.push(
    await step("reset-probe", () =>
      send(
        session,
        buildCmd(
          {
            type: "resetAcpTestProbe",
            requestId: `reset-${acceptKey}-${Date.now()}`,
          },
          opts.target
        )
      )
    )
  );

  // 3. Type @ to open picker (native input with focus enforcement)
  const typeAtStep = await step("type-at-trigger", () =>
    runTool(
      nativeInputArgs("type", "@", opts.surface),
      "type-at"
    )
  );
  steps.push(typeAtStep);

  if (typeAtStep.status !== "pass") {
    return {
      schemaVersion: SCHEMA_VERSION,
      recipe: `acp-${acceptKey}-accept`,
      status: "fail",
      steps,
      summary: `Native input failed at type-at-trigger: focus not confirmed or input not delivered`,
    };
  }

  // 4. Wait for picker to open using waitFor instead of fixed sleep
  steps.push(
    await step("wait-picker-open", () =>
      rpc(
        session,
        buildCmd(
          {
            type: "waitFor",
            requestId: `w-picker-open-${acceptKey}`,
            condition: { type: "acpPickerOpen" },
            timeout: 3000,
            pollInterval: 25,
            trace: "onFailure",
          },
          opts.target
        ),
        { expect: "waitForResult", timeout: 5000 }
      )
    )
  );

  // 5. State-only verification for picker: no screenshot, no probe
  steps.push(
    await step("verify-picker-open", () =>
      runTool(
        verifyArgs(
          [
            "bun",
            "scripts/agentic/verify-shot.ts",
            "--session",
            session,
            "--label",
            "picker-open",
            "--skip-screenshot",
            "--skip-probe",
            "--acp-picker-open",
          ],
          opts.target
        ),
        "verify-picker"
      )
    )
  );

  // 6. Accept with native key (with focus enforcement)
  const nativeKeyStep = await step(`native-${acceptKey}`, () =>
    runTool(
      nativeInputArgs("key", acceptKey, opts.surface),
      `native-${acceptKey}`
    )
  );
  steps.push(nativeKeyStep);

  if (nativeKeyStep.status !== "pass") {
    return {
      schemaVersion: SCHEMA_VERSION,
      recipe: `acp-${acceptKey}-accept`,
      status: "fail",
      steps,
      summary: `Native input failed at native-${acceptKey}: focus not confirmed or key not delivered`,
    };
  }

  // 7. Wait for key-specific acceptance proof (not generic acpItemAccepted)
  steps.push(
    await step("wait-accepted-via-key", () =>
      rpc(
        session,
        buildCmd(
          {
            type: "waitFor",
            requestId: `w-accepted-via-${acceptKey}`,
            condition: { type: "acpAcceptedViaKey", key: acceptKey },
            timeout: 3000,
            pollInterval: 25,
            trace: "onFailure",
          },
          opts.target
        ),
        { expect: "waitForResult", timeout: 5000 }
      )
    )
  );

  // 8. Final proof: screenshot + probe assertion (the only screenshot in the recipe)
  const finalVerifyBase = [
    "bun",
    "scripts/agentic/verify-shot.ts",
    "--session",
    session,
    "--label",
    `${acceptKey}-accepted`,
    "--acp-picker-closed",
    "--acp-item-accepted",
    "--acp-accepted-via",
    acceptKey,
    ...(opts.emitVision ? ["--emit-vision-crops"] : []),
  ];
  steps.push(
    await step("verify-accepted", () =>
      runTool(verifyArgs(finalVerifyBase, opts.target), "verify-accepted")
    )
  );

  const allPass = steps.every((s) => s.status === "pass");

  // Extract the verify-accepted step's proof bundle for top-level access
  const verifyStep = steps.find((s) => s.name === "verify-accepted");
  const proofBundle =
    opts.emitVision && verifyStep?.output ? verifyStep.output : undefined;

  return {
    schemaVersion: SCHEMA_VERSION,
    recipe: `acp-${acceptKey}-accept`,
    status: allPass
      ? "pass"
      : steps.some((s) => s.status === "error")
        ? "error"
        : "fail",
    steps,
    summary: allPass
      ? `ACP picker accepted via ${acceptKey}`
      : `Failed at: ${steps
          .filter((s) => s.status !== "pass")
          .map((s) => s.name)
          .join(", ")}`,
    ...(proofBundle ? { proofBundle } : {}),
  };
}

async function recipeAcpSetupRecovery(
  session: string,
  selectAgent?: string
): Promise<RecipeReceipt> {
  const steps: StepReceipt[] = [];

  // 1. Show window
  steps.push(
    await step("show", () => send(session, '{"type":"show"}'))
  );

  await Bun.sleep(300);

  // 2. Trigger ACP
  steps.push(
    await step("trigger-acp", () =>
      send(session, '{"type":"triggerBuiltin","name":"tab-ai"}')
    )
  );

  // 3. Wait for setup card to appear (or acpReady if no setup needed)
  const waitSetupStep = await step("wait-setup-visible", () =>
    rpc(
      session,
      JSON.stringify({
        type: "waitFor",
        requestId: "w-setup-visible",
        condition: { type: "acpSetupVisible" },
        timeout: 8000,
        pollInterval: 25,
        trace: "onFailure",
      }),
      { expect: "waitForResult", timeout: 10000 }
    )
  );
  steps.push(waitSetupStep);

  if (waitSetupStep.status !== "pass") {
    // Setup card never appeared — might already be ready or error
    const verifyStep = await step("verify-no-setup", () =>
      runTool(
        [
          "bun",
          "scripts/agentic/verify-shot.ts",
          "--session",
          session,
          "--label",
          "setup-not-found",
          "--skip-screenshot",
          "--skip-probe",
          "--acp-status",
          "setup",
        ],
        "verify-no-setup"
      )
    );
    steps.push(verifyStep);
    return {
      schemaVersion: SCHEMA_VERSION,
      recipe: "acp-setup-recovery",
      status: "fail",
      steps,
      summary:
        "Setup card did not appear — ACP may already be ready or failed to open",
    };
  }

  // 4. State-only verification of setup card
  steps.push(
    await step("verify-setup-visible", () =>
      runTool(
        [
          "bun",
          "scripts/agentic/verify-shot.ts",
          "--session",
          session,
          "--label",
          "setup",
          "--skip-screenshot",
          "--skip-probe",
          "--acp-setup-visible",
        ],
        "verify-setup"
      )
    )
  );

  // 5. If --select-agent provided, drive the setup recovery flow
  if (selectAgent) {
    // 5a. Open agent picker
    steps.push(
      await step("open-setup-agent-picker", () =>
        rpc(
          session,
          JSON.stringify({
            type: "performAcpSetupAction",
            requestId: "a-open-picker",
            action: "openAgentPicker",
          }),
          { expect: "acpSetupActionResult", timeout: 5000 }
        )
      )
    );

    // 5b. Wait for picker to open
    steps.push(
      await step("wait-agent-picker-open", () =>
        rpc(
          session,
          JSON.stringify({
            type: "waitFor",
            requestId: "w-agent-picker-open",
            condition: { type: "acpSetupAgentPickerOpen" },
            timeout: 3000,
            pollInterval: 25,
            trace: "onFailure",
          }),
          { expect: "waitForResult", timeout: 5000 }
        )
      )
    );

    // 5c. Select the agent
    steps.push(
      await step("select-setup-agent", () =>
        rpc(
          session,
          JSON.stringify({
            type: "performAcpSetupAction",
            requestId: "a-select-agent",
            action: "selectAgent",
            agentId: selectAgent,
          }),
          { expect: "acpSetupActionResult", timeout: 5000 }
        )
      )
    );

    // 5d. Wait for selected-agent confirmation
    steps.push(
      await step("wait-selected-agent", () =>
        rpc(
          session,
          JSON.stringify({
            type: "waitFor",
            requestId: "w-selected-agent",
            condition: { type: "acpSetupSelectedAgent", agentId: selectAgent },
            timeout: 3000,
            pollInterval: 25,
            trace: "onFailure",
          }),
          { expect: "waitForResult", timeout: 5000 }
        )
      )
    );

    // 5e. Wait for ACP to become ready after agent selection
    steps.push(
      await step("wait-ready", () =>
        rpc(
          session,
          JSON.stringify({
            type: "waitFor",
            requestId: "w-ready-after-select",
            condition: { type: "acpReady" },
            timeout: 8000,
            pollInterval: 25,
            trace: "onFailure",
          }),
          { expect: "waitForResult", timeout: 10000 }
        )
      )
    );
  }

  // 6. Final verification — assert expected ACP status based on flow
  const verifyArgs = [
    "bun",
    "scripts/agentic/verify-shot.ts",
    "--session",
    session,
    "--label",
    selectAgent ? "setup-recovered" : "setup-final",
    "--skip-probe",
    "--acp-status",
    selectAgent ? "idle" : "setup",
  ];
  steps.push(
    await step("verify-final", () =>
      runTool(verifyArgs, "verify-final")
    )
  );

  const allPass = steps.every((s) => s.status === "pass");

  // Extract final ACP state from the verify-final step for the receipt
  const verifyFinalStep = steps.find((s) => s.name === "verify-final");
  const finalState =
    verifyFinalStep?.output &&
    typeof verifyFinalStep.output === "object" &&
    !("raw" in (verifyFinalStep.output as Record<string, unknown>))
      ? (verifyFinalStep.output as Record<string, unknown>).state
      : null;
  const finalSetup =
    finalState && typeof finalState === "object"
      ? (finalState as Record<string, unknown>).setup
      : null;

  // Log recipe completion as single-line JSON on stderr
  console.error(
    JSON.stringify({
      event: "acp_setup_recovery_complete",
      finalStatus:
        finalState && typeof finalState === "object"
          ? (finalState as Record<string, unknown>).status
          : null,
      finalReasonCode:
        finalSetup && typeof finalSetup === "object"
          ? (finalSetup as Record<string, unknown>).reasonCode
          : null,
      selectedAgent: selectAgent ?? null,
    })
  );

  return {
    schemaVersion: SCHEMA_VERSION,
    recipe: "acp-setup-recovery",
    status: allPass
      ? "pass"
      : steps.some((s) => s.status === "error")
        ? "error"
        : "fail",
    steps,
    summary: allPass
      ? selectAgent
        ? `ACP setup recovered via ${selectAgent}`
        : "ACP setup card rendered"
      : `Failed at: ${steps
          .filter((s) => s.status !== "pass")
          .map((s) => s.name)
          .join(", ")}`,
  };
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const { recipe, session, key, vision, selectAgent, targetJson, surface } = parseArgs();

let result: RecipeReceipt;

switch (recipe) {
  case "preflight":
    result = await recipePreflight(session);
    break;

  case "acp-open":
    result = await recipeAcpOpen(session, { target: targetJson });
    break;

  case "acp-accept":
    result = await recipeAcpPickerAccept(session, key, {
      emitVision: vision,
      target: targetJson,
      surface,
    });
    break;

  case "acp-enter-accept":
    result = await recipeAcpPickerAccept(session, "enter", {
      emitVision: vision,
      target: targetJson,
      surface,
    });
    break;

  case "acp-tab-accept":
    result = await recipeAcpPickerAccept(session, "tab", {
      emitVision: vision,
      target: targetJson,
      surface,
    });
    break;

  case "acp-setup-recovery":
    result = await recipeAcpSetupRecovery(session, selectAgent);
    break;

  case "help":
  case "--help":
    console.log(`Usage: bun scripts/agentic/index.ts <recipe> [--session NAME] [--key enter|tab] [--vision]
  [--target-json '{"type":"kind","kind":"acpDetached","index":0}'] [--surface acp]
  [--select-agent ID]

Recipes:
  preflight          Check prerequisites (session, window, permissions)
  acp-open           Open ACP and verify ready state
  acp-accept         Full ACP picker accept; choose key with --key enter|tab
  acp-enter-accept   Compatibility alias for --key enter
  acp-tab-accept     Compatibility alias for --key tab
  acp-setup-recovery Recovery from ACP setup; select agent with --select-agent ID
  help               Show this help

Target threading:
  --target-json JSON   ACP window target for all RPCs (reused across all steps)
  --surface SURFACE    Automation surface for native input focus (main, acp, etc.)

Examples:
  bun scripts/agentic/index.ts acp-accept --session default --key enter
  bun scripts/agentic/index.ts acp-accept --session default --key tab --vision
  bun scripts/agentic/index.ts acp-accept --session default --key enter \\
    --target-json '{"type":"kind","kind":"acpDetached","index":0}' --surface acp --vision
  bun scripts/agentic/index.ts acp-setup-recovery --session default --select-agent opencode --json`);
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
process.exit(
  result!.status === "pass" ? 0 : result!.status === "error" ? 2 : 1
);
