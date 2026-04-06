#!/usr/bin/env bun
/**
 * scripts/agentic/scenario.ts
 *
 * Replayable agentic scenarios that produce machine-readable proof bundles.
 * Each scenario resolves one exact target once, reuses it for every step,
 * and records the exact windowId/surfaceId in the emitted proof bundle.
 *
 * Proof bundles are the regression substrate for cross-window automation:
 * target resolution, inspect snapshots, GPUI events, and waits captured
 * in one structured receipt.
 *
 * Usage (standalone):
 *   bun scripts/agentic/scenario.ts --session default --scenario detached-acp-exact-id --index 0
 *
 * Output:
 *   stdout: JSON proof bundle (schemaVersion 2)
 *   stderr: structured step-level logs (NDJSON)
 */

import { resolve } from "path";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const PROOF_BUNDLE_SCHEMA_VERSION = 2;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface ProofBundleStep {
  type: "resolveTarget" | "inspect" | "simulateGpuiEvent" | "waitFor";
  at: string;
  request: Record<string, unknown>;
  response: Record<string, unknown>;
}

export interface ProofBundle {
  schemaVersion: 2;
  scenario: string;
  resolvedTarget: {
    windowId: string;
    windowKind: string;
    title?: string | null;
    surfaceId?: string | null;
  };
  steps: ProofBundleStep[];
  warnings: string[];
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function stderrLog(event: string, fields: Record<string, unknown> = {}): void {
  process.stderr.write(
    JSON.stringify({ event, ts: new Date().toISOString(), ...fields }) + "\n"
  );
}

export function pushProofStep(
  bundle: ProofBundle,
  step: ProofBundleStep
): void {
  bundle.steps.push(step);
  stderrLog("proof_bundle.step_written", {
    scenario: bundle.scenario,
    stepType: step.type,
    windowId: bundle.resolvedTarget.windowId,
  });
}

async function runTool(
  cmd: string[],
  label: string
): Promise<{ exitCode: number; stdout: string; stderr: string }> {
  const proc = Bun.spawn(cmd, {
    stdout: "pipe",
    stderr: "pipe",
    cwd: PROJECT_ROOT,
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const exitCode = await proc.exited;
  stderrLog("tool_complete", { label, exitCode });
  return { exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
}

async function rpc(
  session: string,
  payload: Record<string, unknown>,
  expect: string,
  timeoutMs: number = 5000
): Promise<Record<string, unknown>> {
  const result = await runTool(
    [
      "bash",
      "scripts/agentic/session.sh",
      "rpc",
      session,
      JSON.stringify(payload),
      "--expect",
      expect,
      "--timeout",
      String(timeoutMs),
    ],
    `rpc:${payload.type}`
  );
  if (result.exitCode !== 0) {
    throw new Error(
      result.stdout || result.stderr || `RPC failed with exit code ${result.exitCode}`
    );
  }
  return JSON.parse(result.stdout);
}

// ---------------------------------------------------------------------------
// Target resolution via automation-window.ts
// ---------------------------------------------------------------------------

interface ResolvedTarget {
  targetJson: { type: "id"; id: string };
  windowKind: string;
  automationWindowId: string;
  title: string | null;
  surfaceId: string | null;
}

async function resolveAutomationWindow(opts: {
  session: string;
  kind: string;
  index: number;
}): Promise<ResolvedTarget> {
  const result = await runTool(
    [
      "bun",
      "scripts/agentic/automation-window.ts",
      "resolve",
      "--session",
      opts.session,
      "--kind",
      opts.kind,
      "--index",
      String(opts.index),
    ],
    "resolve-target"
  );

  if (result.exitCode !== 0) {
    throw new Error(
      `Target resolution failed: ${result.stdout || result.stderr}`
    );
  }

  const parsed = JSON.parse(result.stdout);
  if (parsed.status !== "ok") {
    throw new Error(
      `Target resolution returned error: ${parsed.error?.message ?? "unknown"}`
    );
  }

  const automationWindowId = parsed.automationWindowId
    ? String(parsed.automationWindowId)
    : "";
  if (!automationWindowId) {
    throw new Error("Target resolution returned an empty automationWindowId");
  }

  // Promote to exact-id target for all subsequent RPCs
  const targetJson: { type: "id"; id: string } = {
    type: "id",
    id: automationWindowId,
  };

  stderrLog("agentic.promote_exact_target", {
    fromKind: opts.kind,
    fromIndex: opts.index,
    promotedTargetJson: targetJson,
    automationWindowId,
    surfaceId: parsed.surfaceId ?? null,
  });

  return {
    targetJson,
    windowKind: parsed.windowKind ?? opts.kind,
    automationWindowId,
    title: parsed.title ?? null,
    surfaceId: parsed.surfaceId ?? null,
  };
}

// ---------------------------------------------------------------------------
// Scenarios
// ---------------------------------------------------------------------------

export async function runDetachedAcpExactIdScenario(
  session: string,
  index: number
): Promise<ProofBundle> {
  stderrLog("scenario.start", {
    scenario: "detached-acp-exact-id",
    session,
    index,
  });

  // Step 1: Resolve the detached ACP target to an exact ID
  const resolved = await resolveAutomationWindow({
    session,
    kind: "acpDetached",
    index,
  });

  const bundle: ProofBundle = {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "detached-acp-exact-id",
    resolvedTarget: {
      windowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
    },
    steps: [],
    warnings: [],
  };

  pushProofStep(bundle, {
    type: "resolveTarget",
    at: new Date().toISOString(),
    request: { session, kind: "acpDetached", index },
    response: {
      automationWindowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
      targetJson: resolved.targetJson,
    },
  });

  // Step 2: Inspect the resolved window (before any interaction)
  try {
    const inspectBefore = await rpc(
      session,
      {
        type: "inspectAutomationWindow",
        requestId: "inspect-before",
        target: resolved.targetJson,
        probes: [{ x: 24, y: 24 }],
      },
      "automationInspectResult",
      8000
    );

    pushProofStep(bundle, {
      type: "inspect",
      at: new Date().toISOString(),
      request: {
        type: "inspectAutomationWindow",
        requestId: "inspect-before",
        target: resolved.targetJson,
        probes: [{ x: 24, y: 24 }],
      },
      response: inspectBefore,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    bundle.warnings.push(`inspect_before_failed: ${msg}`);
    stderrLog("scenario.inspect_before_failed", { error: msg });
  }

  // Step 3: Simulate a GPUI event (Cmd+K) to the exact target
  try {
    const eventResult = await rpc(
      session,
      {
        type: "simulateGpuiEvent",
        requestId: "gpui-cmd-k",
        target: resolved.targetJson,
        event: { type: "keyDown", key: "k", modifiers: ["cmd"] },
      },
      "simulateGpuiEventResult",
      5000
    );

    pushProofStep(bundle, {
      type: "simulateGpuiEvent",
      at: new Date().toISOString(),
      request: {
        type: "simulateGpuiEvent",
        requestId: "gpui-cmd-k",
        target: resolved.targetJson,
        event: { type: "keyDown", key: "k", modifiers: ["cmd"] },
      },
      response: eventResult,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    bundle.warnings.push(`gpui_event_failed: ${msg}`);
    stderrLog("scenario.gpui_event_failed", { error: msg });
  }

  // Step 4: Inspect the window again (after interaction)
  try {
    const inspectAfter = await rpc(
      session,
      {
        type: "inspectAutomationWindow",
        requestId: "inspect-after",
        target: resolved.targetJson,
        probes: [{ x: 24, y: 24 }],
      },
      "automationInspectResult",
      8000
    );

    pushProofStep(bundle, {
      type: "inspect",
      at: new Date().toISOString(),
      request: {
        type: "inspectAutomationWindow",
        requestId: "inspect-after",
        target: resolved.targetJson,
        probes: [{ x: 24, y: 24 }],
      },
      response: inspectAfter,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    bundle.warnings.push(`inspect_after_failed: ${msg}`);
    stderrLog("scenario.inspect_after_failed", { error: msg });
  }

  stderrLog("scenario.complete", {
    scenario: "detached-acp-exact-id",
    stepCount: bundle.steps.length,
    warningCount: bundle.warnings.length,
  });

  return bundle;
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);

  const sessionIdx = args.indexOf("--session");
  const session =
    sessionIdx >= 0 && args[sessionIdx + 1] ? args[sessionIdx + 1] : "default";

  const scenarioIdx = args.indexOf("--scenario");
  const scenario =
    scenarioIdx >= 0 && args[scenarioIdx + 1] ? args[scenarioIdx + 1] : "";

  const indexIdx = args.indexOf("--index");
  const rawIndex = indexIdx >= 0 ? args[indexIdx + 1] : undefined;
  if (rawIndex != null) {
    const parsedIndex = Number(rawIndex);
    if (!Number.isInteger(parsedIndex) || parsedIndex < 0) {
      throw new Error(
        `Invalid --index: expected non-negative integer, got ${rawIndex}`
      );
    }
  }
  const index = rawIndex != null ? Number(rawIndex) : 0;

  return { session, scenario, index };
}

// Only run CLI when executed directly (not imported)
if (import.meta.main) {
  const { session, scenario, index } = parseArgs();

  if (!scenario) {
    process.stderr.write(
      JSON.stringify({
        event: "scenario.error",
        error: "Missing --scenario flag",
        available: ["detached-acp-exact-id"],
      }) + "\n"
    );
    process.exit(2);
  }

  switch (scenario) {
    case "detached-acp-exact-id": {
      const bundle = await runDetachedAcpExactIdScenario(session, index);
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.warnings.length > 0 ? 1 : 0);
      break;
    }

    default:
      process.stderr.write(
        JSON.stringify({
          event: "scenario.error",
          error: `Unknown scenario: ${scenario}`,
          available: ["detached-acp-exact-id"],
        }) + "\n"
      );
      process.exit(2);
  }
}
