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
import {
  assertTargetStable,
  listNativePeerWindows,
  promoteExactTarget,
  runTool as runTargetThreadTool,
  targetedRpc,
  type TargetThreadFailure,
  type TargetThreadIdentity,
} from "./target-thread";

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

/** Deterministic popup capture strategy. */
export interface PopupCaptureSummary {
  strategy: "parent_capture_with_crop" | "direct_window_capture" | "not_applicable";
  targetBounds?: { x: number; y: number; width: number; height: number } | null;
  semanticReceiptsArePrimary: boolean;
}

export interface ProofBundle {
  schemaVersion: 2;
  scenario: string;
  surfaceClass?: "main" | "attachedPopup" | "detached";
  resolvedTarget: {
    windowId: string;
    windowKind: string;
    title?: string | null;
    surfaceId?: string | null;
  };
  /** Routed input method used during the flow. */
  inputMethod?: "batch" | "simulateGpuiEvent" | "native";
  /** Dispatch path: exact_handle when target was an ID. */
  dispatchPath?: "exact_handle" | "window_role_fallback";
  /** Resolved window ID (same as resolvedTarget.windowId). */
  resolvedWindowId?: string;
  /** OS-level window ID (CGWindowID) when available from inspection. */
  osWindowId?: number | null;
  /** Popup capture strategy receipt. */
  popupCapture?: PopupCaptureSummary;
  /** Inspection metadata from inspectAutomationWindow. */
  inspect?: {
    screenshotWidth?: number | null;
    screenshotHeight?: number | null;
    warnings: string[];
  };
  steps: ProofBundleStep[];
  warnings: string[];
}

export interface HardScenarioReceipt {
  schemaVersion: 2;
  scenario:
    | "detached-acp-target-threading-stress"
    | "acp-prompt-popup-parity"
    | "notes-acp-delayed-action-origin-stress"
    | "file-portal-origin-roundtrip"
    | "permission-privacy-preflight"
    | "shortcut-recorder-focus-capture"
    | "template-prompt-automation-parity-stress"
    | "current-app-commands-frontmost-stress"
    | "actions-captured-subject-frame-stress"
    | "drop-prompt-native-drop-privacy-stress"
    | "path-prompt-filesystem-edge-stress"
    | "screenshot-identity-acp-context-stress"
    | "clipboard-history-portal-range-stress"
    | "browser-tabs-cache-identity-stress"
    | "scroll-selection-reanchor-stress";
  status: "pass" | "fail" | "error";
  targetThread?: {
    stable: boolean;
    initial?: TargetThreadIdentity;
    final?: TargetThreadIdentity;
    checkedSteps: string[];
    driftFailures: TargetThreadFailure[];
  };
  peerWindows?: Array<Record<string, unknown>>;
  popupCases?: Array<Record<string, unknown>>;
  origin?: Record<string, unknown>;
  portal?: Record<string, unknown>;
  permissions?: Record<string, unknown>;
  shortcut?: Record<string, unknown>;
  templatePrompt?: Record<string, unknown>;
  currentAppCommands?: Record<string, unknown>;
  actionsCapturedSubject?: Record<string, unknown>;
  dropPrompt?: Record<string, unknown>;
  pathPrompt?: Record<string, unknown>;
  screenshotIdentity?: Record<string, unknown>;
  clipboardPortal?: Record<string, unknown>;
  browserCache?: Record<string, unknown>;
  scrollSelection?: Record<string, unknown>;
  delayedAction?: Record<string, unknown>;
  usage: Record<string, unknown>;
  captureTarget?: Record<string, unknown> | null;
  steps: Array<Record<string, unknown>>;
  failure?: TargetThreadFailure | Record<string, unknown>;
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
  osWindowId?: number | null;
  popupSemantics?: {
    hasRealElements: boolean;
    panelOnly: boolean;
    warnings: string[];
    batchMutationAvailable: boolean;
  } | null;
  inspect?: Record<string, unknown> | null;
}

async function inspectAndPromoteTarget(opts: {
  session: string;
  kind: string;
  index: number;
  probes?: Array<{ x: number; y: number }>;
}): Promise<ResolvedTarget> {
  const cmd = [
    "bun",
    "scripts/agentic/automation-window.ts",
    "inspect",
    "--session",
    opts.session,
    "--kind",
    opts.kind,
    "--index",
    String(opts.index),
  ];
  for (const probe of opts.probes ?? []) {
    cmd.push("--probe", `${probe.x},${probe.y}`);
  }
  const result = await runTool(cmd, "inspect-and-promote-target");

  if (result.exitCode !== 0) {
    throw new Error(
      `Target inspection failed: ${result.stdout || result.stderr}`
    );
  }

  const parsed = JSON.parse(result.stdout);
  if (parsed.status !== "ok") {
    throw new Error(
      `Target inspection returned error: ${parsed.error?.message ?? "unknown"}`
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
    osWindowId: parsed.osWindowId ?? null,
  });

  return {
    targetJson,
    windowKind: parsed.inspect?.windowKind ?? parsed.windowKind ?? opts.kind,
    automationWindowId,
    title: parsed.inspect?.title ?? parsed.title ?? null,
    surfaceId: parsed.surfaceId ?? null,
    osWindowId: parsed.osWindowId ?? null,
    popupSemantics: parsed.popupSemantics ?? null,
    inspect: parsed.inspect ?? null,
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
  const resolved = await inspectAndPromoteTarget({
    session,
    kind: "acpDetached",
    index,
    probes: [{ x: 24, y: 24 }],
  });

  const bundle: ProofBundle = {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "detached-acp-exact-id",
    surfaceClass: "detached",
    resolvedTarget: {
      windowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
    },
    inputMethod: "simulateGpuiEvent",
    dispatchPath: "exact_handle",
    resolvedWindowId: resolved.automationWindowId,
    popupCapture: {
      strategy: "direct_window_capture",
      targetBounds: null,
      semanticReceiptsArePrimary: true,
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

    // Populate V2 inspect fields from the first successful inspection
    const resp = inspectBefore.response ?? inspectBefore;
    if (typeof (resp as Record<string, unknown>).osWindowId === "number") {
      bundle.osWindowId = (resp as Record<string, unknown>).osWindowId as number;
    }
    bundle.inspect = {
      screenshotWidth: (resp as Record<string, unknown>).screenshotWidth as number ?? null,
      screenshotHeight: (resp as Record<string, unknown>).screenshotHeight as number ?? null,
      warnings: ((resp as Record<string, unknown>).warnings as string[]) ?? [],
    };
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

export async function runPromptPopupExactIdScenario(
  session: string,
  index: number
): Promise<ProofBundle> {
  stderrLog("scenario.start", {
    scenario: "prompt-popup-exact-id",
    session,
    index,
  });

  // Step 1: Resolve the prompt popup target to an exact ID
  const resolved = await inspectAndPromoteTarget({
    session,
    kind: "promptPopup",
    index,
    probes: [{ x: 12, y: 12 }],
  });

  const bundle: ProofBundle = {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "prompt-popup-exact-id",
    surfaceClass: "attachedPopup",
    resolvedTarget: {
      windowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
    },
    inputMethod: "batch",
    dispatchPath: "exact_handle",
    resolvedWindowId: resolved.automationWindowId,
    popupCapture: {
      strategy: "parent_capture_with_crop",
      targetBounds: null, // Populated from inspect if available
      semanticReceiptsArePrimary: true,
    },
    steps: [],
    warnings: [],
  };

  pushProofStep(bundle, {
    type: "resolveTarget",
    at: new Date().toISOString(),
    request: { session, kind: "promptPopup", index },
    response: {
      automationWindowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
      targetJson: resolved.targetJson,
    },
  });

  // Step 2: Inspect the resolved popup window
  try {
    const inspectResult = await rpc(
      session,
      {
        type: "inspectAutomationWindow",
        requestId: "inspect-popup",
        target: resolved.targetJson,
        probes: [{ x: 12, y: 12 }],
      },
      "automationInspectResult",
      8000
    );

    pushProofStep(bundle, {
      type: "inspect",
      at: new Date().toISOString(),
      request: {
        type: "inspectAutomationWindow",
        requestId: "inspect-popup",
        target: resolved.targetJson,
        probes: [{ x: 12, y: 12 }],
      },
      response: inspectResult,
    });

    // Populate V2 fields from inspection
    const resp = inspectResult.response ?? inspectResult;
    if (typeof (resp as Record<string, unknown>).osWindowId === "number") {
      bundle.osWindowId = (resp as Record<string, unknown>).osWindowId as number;
    }
    bundle.inspect = {
      screenshotWidth: (resp as Record<string, unknown>).screenshotWidth as number ?? null,
      screenshotHeight: (resp as Record<string, unknown>).screenshotHeight as number ?? null,
      warnings: ((resp as Record<string, unknown>).warnings as string[]) ?? [],
    };

    // Populate targetBounds for attached popup crop strategy
    const tb = (resp as Record<string, unknown>).targetBoundsInScreenshot as
      | { x: number; y: number; width: number; height: number }
      | undefined;
    if (tb && bundle.popupCapture) {
      bundle.popupCapture.targetBounds = {
        x: tb.x,
        y: tb.y,
        width: tb.width,
        height: tb.height,
      };
    }
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    bundle.warnings.push(`inspect_popup_failed: ${msg}`);
    stderrLog("scenario.inspect_popup_failed", { error: msg });
  }

  // Step 3: Wait for popup to be ready (using waitFor with the exact target)
  try {
    const waitResult = await rpc(
      session,
      {
        type: "waitFor",
        requestId: "wait-popup-ready",
        target: resolved.targetJson,
        condition: { type: "windowVisible" },
        timeout: 3000,
        pollInterval: 25,
      },
      "waitForResult",
      5000
    );

    pushProofStep(bundle, {
      type: "waitFor",
      at: new Date().toISOString(),
      request: {
        type: "waitFor",
        requestId: "wait-popup-ready",
        target: resolved.targetJson,
        condition: { type: "windowVisible" },
      },
      response: waitResult,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    bundle.warnings.push(`wait_popup_ready_failed: ${msg}`);
    stderrLog("scenario.wait_popup_ready_failed", { error: msg });
  }

  stderrLog("scenario.complete", {
    scenario: "prompt-popup-exact-id",
    stepCount: bundle.steps.length,
    warningCount: bundle.warnings.length,
    hasTargetBounds: bundle.popupCapture?.targetBounds != null,
  });

  return bundle;
}

export async function runActionsDialogExactIdScenario(
  session: string,
  index: number
): Promise<ProofBundle> {
  stderrLog("scenario.start", {
    scenario: "actions-dialog-exact-id",
    session,
    index,
  });

  const resolved = await inspectAndPromoteTarget({
    session,
    kind: "actionsDialog",
    index,
    probes: [{ x: 12, y: 12 }],
  });

  const bundle: ProofBundle = {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "actions-dialog-exact-id",
    surfaceClass: "attachedPopup",
    resolvedTarget: {
      windowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
    },
    inputMethod: "batch",
    dispatchPath: "exact_handle",
    resolvedWindowId: resolved.automationWindowId,
    popupCapture: {
      strategy: "parent_capture_with_crop",
      targetBounds: null,
      semanticReceiptsArePrimary: true,
    },
    steps: [],
    warnings: [],
  };

  pushProofStep(bundle, {
    type: "resolveTarget",
    at: new Date().toISOString(),
    request: { session, kind: "actionsDialog", index },
    response: {
      automationWindowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
      popupSemantics: resolved.popupSemantics,
      targetJson: resolved.targetJson,
    },
  });

  try {
    const inspectResult = await rpc(
      session,
      {
        type: "inspectAutomationWindow",
        requestId: "inspect-actions-dialog",
        target: resolved.targetJson,
        probes: [{ x: 12, y: 12 }],
      },
      "automationInspectResult",
      8000
    );

    pushProofStep(bundle, {
      type: "inspect",
      at: new Date().toISOString(),
      request: {
        type: "inspectAutomationWindow",
        requestId: "inspect-actions-dialog",
        target: resolved.targetJson,
        probes: [{ x: 12, y: 12 }],
      },
      response: inspectResult,
    });

    const resp = inspectResult.response ?? inspectResult;
    if (typeof (resp as Record<string, unknown>).osWindowId === "number") {
      bundle.osWindowId = (resp as Record<string, unknown>).osWindowId as number;
    }
    bundle.inspect = {
      screenshotWidth: (resp as Record<string, unknown>).screenshotWidth as number ?? null,
      screenshotHeight: (resp as Record<string, unknown>).screenshotHeight as number ?? null,
      warnings: ((resp as Record<string, unknown>).warnings as string[]) ?? [],
    };
    const tb = (resp as Record<string, unknown>).targetBoundsInScreenshot as
      | { x: number; y: number; width: number; height: number }
      | undefined;
    if (tb && bundle.popupCapture) {
      bundle.popupCapture.targetBounds = tb;
    }
    if (!resolved.popupSemantics?.batchMutationAvailable) {
      bundle.warnings.push("actionsDialog_batch_unavailable");
    }
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    bundle.warnings.push(`inspect_actions_dialog_failed: ${msg}`);
    stderrLog("scenario.inspect_actions_dialog_failed", { error: msg });
  }

  try {
    const waitResult = await rpc(
      session,
      {
        type: "waitFor",
        requestId: "wait-actions-dialog-visible",
        target: resolved.targetJson,
        condition: { type: "windowVisible" },
        timeout: 3000,
        pollInterval: 25,
      },
      "waitForResult",
      5000
    );

    pushProofStep(bundle, {
      type: "waitFor",
      at: new Date().toISOString(),
      request: {
        type: "waitFor",
        requestId: "wait-actions-dialog-visible",
        target: resolved.targetJson,
        condition: { type: "windowVisible" },
      },
      response: waitResult,
    });
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    bundle.warnings.push(`wait_actions_dialog_ready_failed: ${msg}`);
    stderrLog("scenario.wait_actions_dialog_ready_failed", { error: msg });
  }

  stderrLog("scenario.complete", {
    scenario: "actions-dialog-exact-id",
    stepCount: bundle.steps.length,
    warningCount: bundle.warnings.length,
    hasTargetBounds: bundle.popupCapture?.targetBounds != null,
  });

  return bundle;
}

export async function runMainWindowExactIdScenario(
  session: string
): Promise<ProofBundle> {
  stderrLog("scenario.start", {
    scenario: "main-window-exact-id",
    session,
  });

  const resolved = await inspectAndPromoteTarget({
    session,
    kind: "main",
    index: 0,
    probes: [{ x: 24, y: 24 }],
  });

  const bundle: ProofBundle = {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "main-window-exact-id",
    surfaceClass: "main",
    resolvedTarget: {
      windowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
    },
    inputMethod: "batch",
    dispatchPath: "exact_handle",
    resolvedWindowId: resolved.automationWindowId,
    popupCapture: {
      strategy: "not_applicable",
      targetBounds: null,
      semanticReceiptsArePrimary: true,
    },
    steps: [],
    warnings: [],
  };

  pushProofStep(bundle, {
    type: "resolveTarget",
    at: new Date().toISOString(),
    request: { session, kind: "main", index: 0 },
    response: {
      automationWindowId: resolved.automationWindowId,
      windowKind: resolved.windowKind,
      title: resolved.title,
      surfaceId: resolved.surfaceId,
      targetJson: resolved.targetJson,
    },
  });

  try {
    const inspectResult = await rpc(
      session,
      {
        type: "inspectAutomationWindow",
        requestId: "inspect-main",
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
        requestId: "inspect-main",
        target: resolved.targetJson,
        probes: [{ x: 24, y: 24 }],
      },
      response: inspectResult,
    });

    const elementsResult = await rpc(
      session,
      {
        type: "getElements",
        requestId: "elements-main",
        target: resolved.targetJson,
      },
      "elementsResult",
      3000
    );

    pushProofStep(bundle, {
      type: "inspect",
      at: new Date().toISOString(),
      request: {
        type: "getElements",
        requestId: "elements-main",
        target: resolved.targetJson,
      },
      response: elementsResult,
    });

    const resp = inspectResult.response ?? inspectResult;
    if (typeof (resp as Record<string, unknown>).osWindowId === "number") {
      bundle.osWindowId = (resp as Record<string, unknown>).osWindowId as number;
    }
    bundle.inspect = {
      screenshotWidth: (resp as Record<string, unknown>).screenshotWidth as number ?? null,
      screenshotHeight: (resp as Record<string, unknown>).screenshotHeight as number ?? null,
      warnings: ((resp as Record<string, unknown>).warnings as string[]) ?? [],
    };
    const elementResponse = (elementsResult.response ?? elementsResult) as Record<string, unknown>;
    const elements = Array.isArray(elementResponse.elements) ? elementResponse.elements : [];
    if (elements.length === 0) {
      bundle.warnings.push("main_elements_empty");
    }
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    bundle.warnings.push(`inspect_main_failed: ${msg}`);
    stderrLog("scenario.inspect_main_failed", { error: msg });
  }

  stderrLog("scenario.complete", {
    scenario: "main-window-exact-id",
    stepCount: bundle.steps.length,
    warningCount: bundle.warnings.length,
  });

  return bundle;
}

function hardFailure(
  scenario: HardScenarioReceipt["scenario"],
  failure: TargetThreadFailure | Record<string, unknown>,
  steps: Array<Record<string, unknown>> = [],
): HardScenarioReceipt {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario,
    status: "fail",
    usage: {
      stateFirst: true,
      usedGetAcpState: false,
      usedGetAcpTestProbe: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
    },
    steps,
    failure,
    warnings: [],
  };
}

function failureFromError(
  error: unknown,
  fallbackCode: TargetThreadFailure["code"],
  stepName: string,
): TargetThreadFailure {
  const maybeFailure = (error as { failure?: TargetThreadFailure })?.failure;
  if (maybeFailure) return maybeFailure;
  return {
    code: fallbackCode,
    stepName,
    message: error instanceof Error ? error.message : String(error),
  };
}

export async function runDetachedAcpTargetThreadingStressScenario(opts: {
  session: string;
  kind: string;
  index: number;
  minTargets: number;
  key: "enter" | "tab";
  vision: boolean;
}): Promise<HardScenarioReceipt> {
  const scenario = "detached-acp-target-threading-stress";
  const steps: Array<Record<string, unknown>> = [];
  const driftFailures: TargetThreadFailure[] = [];
  const peerWindows = await listNativePeerWindows({ family: "acpDetached" });

  if (peerWindows.length < opts.minTargets) {
    return hardFailure(scenario, {
      code: "insufficient_target_count",
      stepName: "peer-window-count",
      message: `Expected at least ${opts.minTargets} ACP peer windows, found ${peerWindows.length}`,
      expected: { peerCount: opts.minTargets },
      actual: { peerCount: peerWindows.length, peerWindows },
    }, steps);
  }

  let identity: TargetThreadIdentity;
  try {
    identity = await promoteExactTarget({
      session: opts.session,
      kind: opts.kind,
      index: opts.index,
      expected: { windowKind: "acpDetached" },
    });
  } catch (error) {
    return hardFailure(
      scenario,
      failureFromError(error, "target_resolution_failed", "promote-exact-target"),
      steps,
    );
  }

  if (identity.surfaceId == null) {
    return hardFailure(scenario, {
      code: "missing_surface_id",
      stepName: "promote-exact-target",
      expected: { surfaceId: "non-null" } as Partial<TargetThreadIdentity>,
      actual: identity,
      message: "Detached ACP native input requires exact surfaceId from automation-window inspection",
    }, steps);
  }
  if (identity.osWindowId == null) {
    return hardFailure(scenario, {
      code: "missing_os_window_id",
      stepName: "promote-exact-target",
      expected: { osWindowId: 1 } as Partial<TargetThreadIdentity>,
      actual: identity,
      message: "Detached ACP strict capture requires osWindowId from automation-window inspection",
    }, steps);
  }

  const checkedSteps: string[] = [];
  const pushRpc = async (
    stepName: string,
    command: Record<string, unknown>,
    expect: string,
    timeout = 5000,
  ) => {
    const receipt = await targetedRpc({
      session: opts.session,
      identity,
      requestId: stepName,
      command,
      expect,
      timeout,
      stepName,
    });
    steps.push(receipt as unknown as Record<string, unknown>);
    checkedSteps.push(stepName);
    const stability = await assertTargetStable({
      session: opts.session,
      identity,
      stepName,
    });
    if (!stability.ok) driftFailures.push(stability.failure);
    return receipt;
  };

  await pushRpc("baseline-getAcpState", { type: "getAcpState" }, "acpStateResult");
  await pushRpc("reset-probe", { type: "resetAcpTestProbe" }, "acpTestProbeResult");

  const nativeType = await runTargetThreadTool(
    [
      "bun",
      "scripts/agentic/macos-input.ts",
      "type",
      "@",
      "--ensure-focus",
      "--session",
      opts.session,
      "--target",
      identity.surfaceId,
    ],
    "detached-stress-native-type",
  );
  steps.push({
    name: "native-type-at",
    status: nativeType.exitCode === 0 ? "pass" : "fail",
    output: nativeType.stdout ? JSON.parse(nativeType.stdout) : { stderr: nativeType.stderr },
  });
  checkedSteps.push("native-type-at");
  const afterType = await assertTargetStable({ session: opts.session, identity, stepName: "native-type-at" });
  if (!afterType.ok) driftFailures.push(afterType.failure);

  await pushRpc(
    "wait-picker-open",
    {
      type: "waitFor",
      condition: { type: "acpPickerOpen" },
      timeout: 3000,
      pollInterval: 25,
      trace: "onFailure",
    },
    "waitForResult",
    5000,
  );

  const nativeAccept = await runTargetThreadTool(
    [
      "bun",
      "scripts/agentic/macos-input.ts",
      "key",
      opts.key,
      "--ensure-focus",
      "--session",
      opts.session,
      "--target",
      identity.surfaceId,
    ],
    "detached-stress-native-accept",
  );
  steps.push({
    name: `native-accept-${opts.key}`,
    status: nativeAccept.exitCode === 0 ? "pass" : "fail",
    output: nativeAccept.stdout ? JSON.parse(nativeAccept.stdout) : { stderr: nativeAccept.stderr },
  });
  checkedSteps.push(`native-accept-${opts.key}`);

  await pushRpc(
    `wait-accepted-via-${opts.key}`,
    {
      type: "waitFor",
      condition: { type: "acpAcceptedViaKey", key: opts.key },
      timeout: 3000,
      pollInterval: 25,
      trace: "onFailure",
    },
    "waitForResult",
    5000,
  );

  await pushRpc("final-getAcpState", { type: "getAcpState" }, "acpStateResult");

  const verify = await runTargetThreadTool(
    [
      "bun",
      "scripts/agentic/verify-shot.ts",
      "--session",
      opts.session,
      "--label",
      "detached-target-threading-stress",
      "--target-json",
      JSON.stringify(identity.targetJson),
      "--capture-window-id",
      String(identity.osWindowId),
      "--acp-picker-closed",
      "--acp-item-accepted",
      "--acp-accepted-via",
      opts.key,
      ...(opts.vision ? ["--vision"] : []),
    ],
    "detached-stress-strict-capture",
  );
  const verifyOutput = verify.stdout ? JSON.parse(verify.stdout) : { stderr: verify.stderr };
  steps.push({
    name: "strict-capture",
    status: verify.exitCode === 0 ? "pass" : "fail",
    output: verifyOutput,
  });
  checkedSteps.push("strict-capture");

  const finalStable = await assertTargetStable({
    session: opts.session,
    identity,
    stepName: "final-before-return",
  });
  if (!finalStable.ok) driftFailures.push(finalStable.failure);

  const captureTarget = (verifyOutput as Record<string, unknown>).captureTarget as
    | Record<string, unknown>
    | undefined;
  const captureMismatch =
    captureTarget?.requestedWindowId != null &&
    captureTarget?.actualWindowId != null &&
    captureTarget.requestedWindowId !== captureTarget.actualWindowId;
  if (captureMismatch) {
    driftFailures.push({
      code: "target_identity_drift",
      stepName: "strict-capture",
      expected: { osWindowId: captureTarget.requestedWindowId as number },
      actual: { osWindowId: captureTarget.actualWindowId as number },
      message: "Strict capture returned a different native window ID",
    });
  }

  const stepFailed = steps.some((step) => step.status !== "pass");
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario,
    status: driftFailures.length === 0 && !stepFailed ? "pass" : "fail",
    targetThread: {
      stable: driftFailures.length === 0,
      initial: identity,
      final: finalStable.ok ? finalStable.identity : undefined,
      checkedSteps,
      driftFailures,
    },
    peerWindows,
    usage: {
      stateFirst: true,
      usedGetAcpState: true,
      usedGetAcpTestProbe: true,
      usedWaitFor: true,
      usedNativeInput: true,
      usedScreenshot: true,
      usedFixedSleepMs: 0,
    },
    captureTarget: captureTarget ?? null,
    steps,
    warnings: [],
  };
}

export async function runAcpPromptPopupParityScenario(opts: {
  session: string;
  families: string[];
}): Promise<HardScenarioReceipt> {
  const scenario = "acp-prompt-popup-parity";
  const popupMap: Record<string, { id: string; triggerText: string }> = {
    mention: { id: "acp-mention-popup", triggerText: "@" },
    "model-selector": { id: "acp-model-selector-popup", triggerText: "/" },
    "local-history": { id: "acp-history-popup", triggerText: "" },
  };
  const popupCases: Array<Record<string, unknown>> = [];
  const steps: Array<Record<string, unknown>> = [];
  const warnings: string[] = [];

  for (const family of opts.families) {
    const expected = popupMap[family];
    if (!expected) {
      popupCases.push({
        family,
        status: "fail",
        failure: {
          code: "wrong_popup_family",
          stepName: "family-parse",
          message: `Unknown popup family ${family}`,
        },
      });
      continue;
    }

    const setInput = await runTargetThreadTool(
      [
        "bash",
        "scripts/agentic/session.sh",
        "send",
        opts.session,
        JSON.stringify({
          type: "setAcpInput",
          text: expected.triggerText,
          requestId: `popup-parity-${family}-set-input`,
        }),
        "--await-parse",
      ],
      `popup-parity-trigger:${family}`,
    );
    steps.push({
      name: `trigger-${family}`,
      status: setInput.exitCode === 0 ? "pass" : "fail",
      output: setInput.stdout ? JSON.parse(setInput.stdout) : { stderr: setInput.stderr },
    });

    let identity: TargetThreadIdentity;
    try {
      identity = await promoteExactTarget({
        session: opts.session,
        kind: "promptPopup",
        index: 0,
        expected: { popupId: expected.id },
      });
    } catch (error) {
      const failure = failureFromError(error, "target_resolution_failed", `promote-${family}`);
      popupCases.push({ family, expectedPopupId: expected.id, status: "fail", failure });
      continue;
    }

    const visible = await targetedRpc({
      session: opts.session,
      identity,
      requestId: `popup-parity-${family}-visible`,
      command: {
        type: "waitFor",
        condition: { type: "windowVisible" },
        timeout: 3000,
        pollInterval: 25,
      },
      expect: "waitForResult",
      timeout: 5000,
      stepName: `wait-visible-${family}`,
    });
    const elements = await targetedRpc({
      session: opts.session,
      identity,
      requestId: `popup-parity-${family}-elements`,
      command: { type: "getElements", limit: 200 },
      expect: "elementsResult",
      timeout: 5000,
      stepName: `get-elements-${family}`,
    });
    const elementOutput = elements.output as Record<string, unknown>;
    const rows = Array.isArray(elementOutput.elements) ? elementOutput.elements : [];
    const rowAware = rows.length > 0;
    if (!rowAware) warnings.push(`${family}:popup_rows_missing`);

    const stable = await assertTargetStable({
      session: opts.session,
      identity,
      stepName: `final-${family}`,
    });

    popupCases.push({
      family,
      expectedPopupId: expected.id,
      trigger: { method: "protocol", command: "setAcpInput", text: expected.triggerText },
      targetThread: {
        stable: stable.ok,
        initial: identity,
        final: stable.ok ? stable.identity : undefined,
        driftFailures: stable.ok ? [] : [stable.failure],
      },
      visibleWait: visible.output,
      inspection: {
        windowKind: identity.windowKind,
        popupFamily: identity.popupFamily ?? family,
        popupId: identity.popupId ?? identity.automationWindowId,
      },
      elements: {
        type: "elementsResult",
        rowAware,
        rowCount: rows.length,
        rows,
      },
      rowAction: {
        mode: "inspect",
        status: rowAware ? "pass" : "fail",
      },
      status: stable.ok && rowAware && visible.status === "pass" ? "pass" : "fail",
    });
  }

  const allPass = popupCases.every((popupCase) => popupCase.status === "pass");
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario,
    status: allPass ? "pass" : "fail",
    popupCases,
    usage: {
      stateFirst: true,
      usedGetElements: true,
      usedWaitFor: true,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
    },
    steps,
    warnings,
  };
}

export async function runNotesAcpDelayedActionOriginStressScenario(opts: {
  session: string;
  drift: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "notes-acp-delayed-action-origin-stress",
    status: "fail",
    origin: {
      host: "notes",
      acpGeneration: null,
    },
    delayedAction: {
      outcome: "missingOriginGeneration",
      drift: {
        field: "acpGeneration",
        expected: "non-null",
        actual: null,
      },
    },
    usage: {
      stateFirst: true,
      usedGetAcpState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
    },
    steps: [
      {
        name: "notes-origin-receipt-preflight",
        status: "fail",
        output: {
          session: opts.session,
          drift: opts.drift,
          blockingGap:
            "Notes-hosted ACP delayed action origin/generation receipts are not yet exposed to the TypeScript harness.",
        },
      },
    ],
    failure: {
      code: "missing_origin_generation",
      stepName: "notes-origin-receipt-preflight",
      message:
        "The harness now fails closed until app-side Notes ACP origin/generation receipts exist.",
    },
    warnings: ["file_linear:notes_acp_origin_generation_receipts_missing"],
  };
}

function parseMaybeJson(text: string): Record<string, unknown> {
  if (!text.trim()) return {};
  try {
    return JSON.parse(text);
  } catch {
    return { raw: text };
  }
}

export async function runAcpPortalRoundTripOriginStressScenario(opts: {
  session: string;
  host: string;
  portal: string;
  selection?: string;
  query?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "file-portal-origin-roundtrip",
    status: "fail",
    origin: {
      host: opts.host,
      session: opts.session,
      acpGeneration: null,
      portalSessionId: null,
      returnTarget: null,
    },
    portal: {
      kind: opts.portal,
      selection: opts.selection ?? "file",
      query: opts.query ?? "AGENTS.md",
      roundTrip: "unverified",
      expectedReceipts: [
        "origin.host",
        "origin.acpGeneration",
        "portal.sessionId",
        "portal.returnTarget",
        "contextPart.uri",
      ],
    },
    usage: {
      stateFirst: true,
      usedGetAcpState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [
      {
        name: "portal-origin-receipt-preflight",
        status: "fail",
        output: {
          session: opts.session,
          host: opts.host,
          portal: opts.portal,
          blockingGap:
            "ACP portal round-trip receipts do not yet expose origin generation, portal session id, return target, and accepted context part URI to the TypeScript harness.",
        },
      },
    ],
    failure: {
      code: "missing_portal_round_trip_origin_receipt",
      stepName: "portal-origin-receipt-preflight",
      message:
        "The harness fails closed until ACP portal round-trip origin and context-part receipts exist.",
    },
    warnings: ["file_linear:acp_portal_round_trip_origin_receipts_missing"],
  };
}

export async function runPermissionPreflightReadonlyScenario(opts: {
  session: string;
  kinds?: string[];
}): Promise<HardScenarioReceipt> {
  const steps: Array<Record<string, unknown>> = [];

  const inputCheck = await runTool(
    ["bun", "scripts/agentic/macos-input.ts", "check"],
    "permission-preflight:macos-input-check"
  );
  const inputPayload = parseMaybeJson(inputCheck.stdout);
  steps.push({
    name: "macos-input-check",
    status: inputCheck.exitCode === 0 ? "pass" : "fail",
    output: inputPayload,
  });

  const windowStatus = await runTool(
    ["bun", "scripts/agentic/window.ts", "status"],
    "permission-preflight:window-status"
  );
  const windowPayload = parseMaybeJson(windowStatus.stdout);
  steps.push({
    name: "window-status",
    status: windowStatus.exitCode === 0 ? "pass" : "fail",
    output: windowPayload,
  });

  const permissions = {
    session: opts.session,
    kinds: opts.kinds ?? ["accessibility", "screen-recording", "microphone"],
    accessibility:
      ((inputPayload.data as Record<string, unknown> | undefined)?.accessibility as
        | boolean
        | undefined) ?? null,
    screenRecording: "notPrompted",
    microphone: "notPrompted",
    passiveOnly: true,
    openedSystemSettings: false,
    mutatedTcc: false,
    clickedSettings: false,
  };
  const allPass = steps.every((step) => step.status === "pass");

  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "permission-privacy-preflight",
    status: allPass ? "pass" : "fail",
    permissions,
    usage: {
      stateFirst: true,
      usedGetAcpState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      openedSystemSettings: false,
      mutatedUserData: false,
    },
    steps,
    failure: allPass
      ? undefined
      : {
          code: "permission_preflight_failed",
          stepName: "permission-privacy-preflight",
          message:
            "Read-only permission preflight failed without opening System Settings or mutating permissions.",
        },
    warnings: [],
  };
}

export async function runShortcutRecorderFocusCaptureStressScenario(opts: {
  session: string;
  chord: string;
  surface?: string;
  action?: string;
  sandboxConfig?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "shortcut-recorder-focus-capture",
    status: "fail",
    shortcut: {
      chord: opts.chord,
      surface: opts.surface ?? "shortcuts",
      action: opts.action ?? "test-agentic-shortcut",
      sandboxConfig: opts.sandboxConfig ?? false,
      recorderSurface: null,
      focusedAutomationWindowId: null,
      capturedShortcut: null,
      leakedGlobalHotkey: null,
    },
    usage: {
      stateFirst: true,
      usedGetAcpState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [
      {
        name: "shortcut-recorder-receipt-preflight",
        status: "fail",
        output: {
          session: opts.session,
          chord: opts.chord,
          surface: opts.surface ?? "shortcuts",
          action: opts.action ?? "test-agentic-shortcut",
          sandboxConfig: opts.sandboxConfig ?? false,
          blockingGap:
            "Shortcut recorder focus/capture receipts are not yet exposed to the TypeScript harness without writing config.ts.",
        },
      },
    ],
    failure: {
      code: "missing_shortcut_recorder_capture_receipt",
      stepName: "shortcut-recorder-receipt-preflight",
      message:
        "The harness fails closed until recorder focus, captured chord, and global-hotkey leak receipts exist.",
    },
    warnings: ["file_linear:shortcut_recorder_capture_receipts_missing"],
  };
}

export async function runTemplatePromptAutomationParityStressScenario(opts: {
  session: string;
  template?: string;
  field?: string;
  value?: string;
  forcedValue?: string;
}): Promise<HardScenarioReceipt> {
  const template = opts.template ?? "Hello {{name}}";
  const field = opts.field ?? "name";
  const value = opts.value ?? "Ada";
  const forcedValue = opts.forcedValue ?? "forced-template-result";
  const steps: Array<Record<string, unknown>> = [];

  const pushStep = (
    name: string,
    status: "pass" | "fail" | "error",
    output: unknown,
  ) => {
    steps.push({ name, status, output });
  };

  const fail = (
    code: TargetThreadFailure["code"],
    stepName: string,
    message: string,
    output?: unknown,
  ): HardScenarioReceipt => {
    if (output !== undefined) pushStep(stepName, "fail", output);
    return {
      schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
      scenario: "template-prompt-automation-parity-stress",
      status: "fail",
      templatePrompt: {
        session: opts.session,
        template,
        field,
        value,
        forcedValue,
        failureStep: stepName,
      },
      usage: {
        stateFirst: true,
        usedGetState: true,
        usedGetElements: true,
        usedWaitFor: false,
        usedBatch: true,
        usedSimulateKey: true,
        usedNativeInput: false,
        usedScreenshot: false,
        usedFixedSleepMs: 0,
        mutatedUserData: false,
      },
      steps,
      failure: { code, stepName, message },
      warnings: [],
    };
  };

  const send = async (payload: Record<string, unknown>, name: string) => {
    const result = await runTool(
      [
        "bash",
        "scripts/agentic/session.sh",
        "send",
        opts.session,
        JSON.stringify(payload),
        "--await-parse",
        "--timeout",
        "8000",
      ],
      `template-prompt:${name}`,
    );
    if (result.exitCode !== 0) {
      throw new Error(result.stdout || result.stderr || `${name} failed`);
    }
    return parseMaybeJson(result.stdout);
  };

  const extractResponse = (receipt: Record<string, unknown>) =>
    ((receipt.response as Record<string, unknown> | undefined) ?? receipt);

  try {
    const start = await runTool(
      ["bash", "scripts/agentic/session.sh", "start", opts.session],
      "template-prompt:session-start",
    );
    if (start.exitCode !== 0) {
      return fail(
        "missing_template_prompt_automation_receipt",
        "session-start",
        "TemplatePrompt parity could not start or resume the requested session.",
        parseMaybeJson(start.stdout || start.stderr),
      );
    }
    pushStep("session-start", "pass", parseMaybeJson(start.stdout));

    const submitId = `tpl-submit-${Date.now()}`;
    const opened = await send(
      { type: "template", id: submitId, template, requestId: `${submitId}-open` },
      "open-submit-template",
    );
    pushStep("open-submit-template", "pass", opened);

    const stateEnvelope = await rpc(
      opts.session,
      { type: "getState", requestId: `${submitId}-state` },
      "stateResult",
      8000,
    );
    const state = extractResponse(stateEnvelope);
    if (state.promptType !== "template") {
      return fail(
        "template_prompt_state_missing",
        "get-state",
        "Expected getState.promptType to be template.",
        stateEnvelope,
      );
    }
    pushStep("get-state", "pass", stateEnvelope);

    const elementsEnvelope = await rpc(
      opts.session,
      { type: "getElements", requestId: `${submitId}-elements`, limit: 80 },
      "elementsResult",
      8000,
    );
    const elementsResponse = extractResponse(elementsEnvelope);
    const elements = Array.isArray(elementsResponse.elements)
      ? elementsResponse.elements as Array<Record<string, unknown>>
      : [];
    const sourceRow = elements.find((element) => element.semanticId === "input:template-source");
    const fieldRowId = `input:template-${field}`;
    const fieldRow = elements.find((element) => element.semanticId === fieldRowId);
    if (!sourceRow || !fieldRow) {
      return fail(
        "template_prompt_elements_missing",
        "get-elements",
        "Expected template source and field rows in getElements.",
        { sourceRow: Boolean(sourceRow), fieldRowId, rowCount: elements.length, elementsEnvelope },
      );
    }
    pushStep("get-elements", "pass", { rowCount: elements.length, sourceRow, fieldRow });

    const fillEnvelope = await rpc(
      opts.session,
      {
        type: "batch",
        requestId: `${submitId}-fill`,
        commands: [{ type: "setInput", text: value }],
      },
      "batchResult",
      8000,
    );
    const fill = extractResponse(fillEnvelope);
    if (fill.success !== true) {
      return fail(
        "template_prompt_force_submit_failed",
        "batch-set-input",
        "TemplatePrompt batch.setInput failed.",
        fillEnvelope,
      );
    }
    pushStep("batch-set-input", "pass", fillEnvelope);

    const actionId = `tpl-actions-${Date.now()}`;
    await send(
      { type: "template", id: actionId, template, requestId: `${actionId}-open` },
      "open-actions-template",
    );
    await send(
      { type: "simulateKey", key: "k", modifiers: ["cmd"], requestId: `${actionId}-cmd-k` },
      "cmd-k-actions",
    );
    let actionsElementsEnvelope: Record<string, unknown> | null = null;
    try {
      actionsElementsEnvelope = await rpc(
        opts.session,
        {
          type: "getElements",
          requestId: `${actionId}-actions-elements`,
          target: { type: "kind", kind: "actionsDialog", index: 0 },
          limit: 80,
        },
        "elementsResult",
        8000,
      );
    } catch (error) {
      actionsElementsEnvelope = {
        error: error instanceof Error ? error.message : String(error),
      };
    }
    const actionsStateEnvelope = await rpc(
      opts.session,
      { type: "getState", requestId: `${actionId}-actions-state` },
      "stateResult",
      8000,
    );
    const actionsState = extractResponse(actionsStateEnvelope);
    const actionsElements = extractResponse(actionsElementsEnvelope ?? {});
    const actionRows = Array.isArray(actionsElements.elements)
      ? actionsElements.elements as Array<Record<string, unknown>>
      : [];
    const actionsOpened = Boolean(actionsState.activePopupContract) || actionRows.length > 0;
    if (!actionsOpened) {
      return fail(
        "template_prompt_actions_unavailable",
        "cmd-k-actions",
        "TemplatePrompt Cmd+K did not expose an active actions popup contract or actionsDialog elements.",
        { actionsStateEnvelope, actionsElementsEnvelope },
      );
    }
    pushStep("cmd-k-actions", "pass", { actionsStateEnvelope, actionsElementsEnvelope });
    await send(
      { type: "simulateKey", key: "escape", modifiers: [], requestId: `${actionId}-escape-actions` },
      "escape-actions",
    );

    const cancelId = `tpl-cancel-${Date.now()}`;
    await send(
      { type: "template", id: cancelId, template, requestId: `${cancelId}-open` },
      "open-cancel-template",
    );
    await send(
      { type: "simulateKey", key: "escape", modifiers: [], requestId: `${cancelId}-escape` },
      "escape-cancel",
    );
    pushStep("escape-cancel", "pass", { id: cancelId });

    const forceId = `tpl-force-${Date.now()}`;
    await send(
      { type: "template", id: forceId, template, requestId: `${forceId}-open` },
      "open-force-template",
    );
    const forceEnvelope = await rpc(
      opts.session,
      {
        type: "batch",
        requestId: `${forceId}-batch-force`,
        commands: [{ type: "forceSubmit", value: forcedValue }],
      },
      "batchResult",
      8000,
    );
    const force = extractResponse(forceEnvelope);
    const forceResult = Array.isArray(force.results)
      ? force.results[0] as Record<string, unknown> | undefined
      : undefined;
    if (force.success !== true || forceResult?.value !== forcedValue) {
      return fail(
        "template_prompt_force_submit_failed",
        "batch-force-submit",
        "TemplatePrompt batch.forceSubmit did not return the explicit provided value.",
        forceEnvelope,
      );
    }
    pushStep("batch-force-submit", "pass", forceEnvelope);

    return {
      schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
      scenario: "template-prompt-automation-parity-stress",
      status: "pass",
      templatePrompt: {
        session: opts.session,
        template,
        field,
        value,
        forcedValue,
        promptType: state.promptType,
        statePromptId: state.promptId ?? null,
        elementSemanticIds: elements.map((element) => element.semanticId).filter(Boolean),
        sourceRow,
        fieldRow,
        actionsHost: "TemplatePrompt",
        activePopupContract: actionsState.activePopupContract,
        actionsRowCount: actionRows.length,
        batchSetInput: { field, value, success: true },
        cancel: { viaEscape: true },
        batchForceSubmit: { providedValue: forcedValue, resolvedValue: forceResult.value },
      },
      usage: {
        stateFirst: true,
        usedGetState: true,
        usedGetElements: true,
        usedWaitFor: false,
        usedBatch: true,
        usedSimulateKey: true,
        usedNativeInput: false,
        usedScreenshot: false,
        usedFixedSleepMs: 0,
        mutatedUserData: false,
      },
      steps,
      warnings: [],
    };
  } catch (error) {
    return fail(
      "missing_template_prompt_automation_receipt",
      "template-prompt-runtime",
      error instanceof Error ? error.message : String(error),
    );
  }
}

export async function runCurrentAppCommandsFrontmostStressScenario(opts: {
  session: string;
  query?: string;
  alias?: string;
  expectedApp?: string;
}): Promise<HardScenarioReceipt> {
  const query = opts.query ?? "close tab";
  const alias = opts.alias ?? "Do in Current Command";

  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "current-app-commands-frontmost-stress",
    status: "fail",
    currentAppCommands: {
      session: opts.session,
      stableEntryId: "builtin/do-in-current-app",
      alias,
      query,
      expectedApp: opts.expectedApp ?? null,
      frontmostSnapshot: null,
      openedView: null,
      sharedFilterHelper: "current_app_commands_filtered_entries",
      stateVisibleChoiceCount: null,
      elementRowCount: null,
      rendererRowCount: null,
      staleAliasRejected: null,
      wrongAppExecutionBlocked: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [
      {
        name: "current-app-frontmost-receipt-preflight",
        status: "fail",
        output: {
          session: opts.session,
          alias,
          query,
          blockingGap:
            "CurrentAppCommandsView receipts do not yet expose a frontmost-app snapshot, alias normalization proof, shared filter counts, and wrong-app execution guard in one agentic recipe.",
        },
      },
    ],
    failure: {
      code: "missing_current_app_commands_frontmost_receipt",
      stepName: "current-app-frontmost-receipt-preflight",
      message:
        "The harness fails closed until Do in Current App can prove frontmost snapshot identity and shared filter semantics without executing against a stale app.",
    },
    warnings: ["file_linear:current_app_commands_frontmost_receipts_missing"],
  };
}

export async function runActionsCapturedSubjectFrameStressScenario(opts: {
  session: string;
  source?: string;
  action?: string;
  mutation?: string;
}): Promise<HardScenarioReceipt> {
  const source = opts.source ?? "root-file";
  const action = opts.action ?? "quick-look";
  const mutation = opts.mutation ?? "filter-selection-cache-frame";

  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "actions-captured-subject-frame-stress",
    status: "fail",
    actionsCapturedSubject: {
      session: opts.session,
      host: "MainList",
      source,
      action,
      mutation,
      subjectStableKey: null,
      pendingSubjectFrame: null,
      activePopupContract: null,
      executeSubjectStableKey: null,
      focusRestoredTo: null,
      reReadCurrentSelection: null,
      unknownRootIdNoop: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [
      {
        name: "actions-captured-subject-receipt-preflight",
        status: "fail",
        output: {
          session: opts.session,
          source,
          action,
          mutation,
          blockingGap:
            "ActionsDialog receipts do not yet expose the captured MainList subject, source-filter frame, execution subject, and focus-restore target as one stable proof.",
        },
      },
    ],
    failure: {
      code: "missing_actions_captured_subject_receipt",
      stepName: "actions-captured-subject-receipt-preflight",
      message:
        "The harness fails closed until root actions can prove execution uses the captured subject after filter/selection/cache/frame drift.",
    },
    warnings: ["file_linear:actions_captured_subject_receipts_missing"],
  };
}

export async function runDropPromptNativeDropPrivacyStressScenario(opts: {
  session: string;
  fileName?: string;
  size?: number;
}): Promise<HardScenarioReceipt> {
  const fileName = opts.fileName ?? "agentic-drop.txt";
  const size = opts.size ?? 12;
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "drop-prompt-native-drop-privacy-stress",
    status: "fail",
    dropPrompt: {
      session: opts.session,
      fileName,
      size,
      expectedState: "stateResult.drop.files[index,name,size]",
      expectedElements: "list:dropped-files + kind:dropped_file rows",
      forbiddenFields: ["path", "parentPath", "content", "mimeType", "modifiedTime"],
      nativeDropInjected: false,
      pathLeakDetected: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [
      {
        name: "drop-native-receipt-preflight",
        status: "fail",
        output: {
          session: opts.session,
          fileName,
          blockingGap:
            "DropPrompt automation has redacted state/elements receipts, but scripts/agentic does not yet have a deterministic native file-drop injection receipt.",
        },
      },
    ],
    failure: {
      code: "missing_drop_prompt_native_drop_receipt",
      stepName: "drop-native-receipt-preflight",
      message:
        "The harness fails closed until native DropPrompt file-drop injection can prove redacted automation receipts without leaking paths.",
    },
    warnings: ["file_linear:drop_prompt_native_drop_receipts_missing"],
  };
}

export async function runPathPromptFilesystemEdgeStressScenario(opts: {
  session: string;
}): Promise<HardScenarioReceipt> {
  const result = await runTool(
    ["bun", "scripts/agentic/path-prompt-fs-edges.ts"],
    "path-prompt-filesystem-edge-stress",
  );
  const output = parseMaybeJson(result.stdout);
  const passed = result.exitCode === 0 && (output as Record<string, unknown>).status === "ok";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "path-prompt-filesystem-edge-stress",
    status: passed ? "pass" : "fail",
    pathPrompt: {
      session: opts.session,
      helper: "scripts/agentic/path-prompt-fs-edges.ts",
      cases: ["missing", "empty", "file-start", "permission-denied"],
      statusKinds: ["missing", "empty", "loaded", "permissionDenied"],
      output,
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [
      {
        name: "path-prompt-fs-edges-helper",
        status: passed ? "pass" : "fail",
        output,
      },
    ],
    failure: passed
      ? undefined
      : {
          code: "path_prompt_filesystem_edge_failed",
          stepName: "path-prompt-fs-edges-helper",
          message: result.stderr || "PathPrompt filesystem edge helper failed.",
        },
    warnings: passed ? [] : ["path_prompt_filesystem_edge_helper_failed"],
  };
}

export async function runScreenshotIdentityAcpContextStressScenario(opts: {
  session: string;
  source?: string;
}): Promise<HardScenarioReceipt> {
  const source = opts.source ?? "tab-ai-screenshot";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "screenshot-identity-acp-context-stress",
    status: "fail",
    screenshotIdentity: {
      session: opts.session,
      source,
      stateField: "stateResult.screenshotIdentity",
      expectedIdentityShape: "bare screenshot filename",
      captureReceipt: null,
      acpContextPart: null,
      identityMatched: null,
      filesystemGrepUsed: false,
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [
      {
        name: "screenshot-identity-context-receipt-preflight",
        status: "fail",
        output: {
          session: opts.session,
          source,
          blockingGap:
            "ACP context automation does not yet expose one receipt tying capture identity, stateResult.screenshotIdentity, and accepted ACP context part identity together.",
        },
      },
    ],
    failure: {
      code: "missing_screenshot_identity_context_receipt",
      stepName: "screenshot-identity-context-receipt-preflight",
      message:
        "The harness fails closed until screenshot identity threading can be proven from state and ACP context receipts without grepping the filesystem.",
    },
    warnings: ["file_linear:screenshot_identity_context_receipts_missing"],
  };
}

export async function runClipboardHistoryPortalRangeStressScenario(opts: {
  session: string;
  portalId?: string;
  range?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "clipboard-history-portal-range-stress",
    status: "fail",
    clipboardPortal: {
      session: opts.session,
      portalId: opts.portalId ?? "kit://clipboard-history?id=agentic",
      range: opts.range ?? "composer:0..0",
      hostRefusalReceipt: null,
      roundTripUri: null,
      exactRangeReplacement: null,
      wrongHostAccepted: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [{
      name: "clipboard-portal-range-receipt-preflight",
      status: "fail",
      output: { blockingGap: "Clipboard portal host refusal, kit:// URI round-trip, and exact range replacement receipts are not exposed as one agentic proof." },
    }],
    failure: {
      code: "missing_clipboard_portal_range_receipt",
      stepName: "clipboard-portal-range-receipt-preflight",
      message: "The harness fails closed until clipboard-history portal range receipts exist.",
    },
    warnings: ["file_linear:clipboard_portal_range_receipts_missing"],
  };
}

export async function runBrowserTabsCacheIdentityStressScenario(opts: {
  session: string;
  source?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "browser-tabs-cache-identity-stress",
    status: "fail",
    browserCache: {
      session: opts.session,
      source: opts.source ?? "browser-tabs",
      cacheOnly: true,
      browserActivated: false,
      dedupeKey: null,
      sourceIdentity: null,
      staleCacheRejected: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [{
      name: "browser-cache-identity-receipt-preflight",
      status: "fail",
      output: { blockingGap: "Browser tabs/history cache identity and dedupe receipts are not exposed without activating the browser." },
    }],
    failure: {
      code: "missing_browser_cache_identity_receipt",
      stepName: "browser-cache-identity-receipt-preflight",
      message: "The harness fails closed until browser cache identity receipts exist.",
    },
    warnings: ["file_linear:browser_cache_identity_receipts_missing"],
  };
}

export async function runScrollSelectionReanchorStressScenario(opts: {
  session: string;
  surfaces?: string[];
}): Promise<HardScenarioReceipt> {
  const surfaces = opts.surfaces ?? ["clipboard", "browser-history", "current-app-commands", "file-search"];
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "scroll-selection-reanchor-stress",
    status: "fail",
    scrollSelection: {
      session: opts.session,
      surfaces,
      initialSelectedSemanticId: null,
      afterWheelSelectedSemanticId: null,
      afterDragSelectedSemanticId: null,
      visibleRowStillSelected: null,
      footerOcclusionSafe: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [{
      name: "scroll-selection-reanchor-receipt-preflight",
      status: "fail",
      output: { surfaces, blockingGap: "Cross-surface wheel/drag selection reanchor receipts are not exposed as one agentic proof." },
    }],
    failure: {
      code: "missing_scroll_selection_reanchor_receipt",
      stepName: "scroll-selection-reanchor-receipt-preflight",
      message: "The harness fails closed until scroll/drag reanchor receipts exist.",
    },
    warnings: ["file_linear:scroll_selection_reanchor_receipts_missing"],
  };
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

  const minTargetsIdx = args.indexOf("--min-targets");
  const minTargets =
    minTargetsIdx >= 0 && args[minTargetsIdx + 1]
      ? Number(args[minTargetsIdx + 1])
      : 2;
  const keyIdx = args.indexOf("--key");
  const key =
    keyIdx >= 0 && (args[keyIdx + 1] === "enter" || args[keyIdx + 1] === "tab")
      ? (args[keyIdx + 1] as "enter" | "tab")
      : "enter";
  const familiesIdx = args.indexOf("--families");
  const families =
    familiesIdx >= 0 && args[familiesIdx + 1]
      ? args[familiesIdx + 1].split(",").map((s) => s.trim()).filter(Boolean)
      : ["mention", "model-selector", "local-history"];
  const driftIdx = args.indexOf("--drift");
  const drift = driftIdx >= 0 && args[driftIdx + 1] ? args[driftIdx + 1] : "generation";
  const hostIdx = args.indexOf("--host");
  const originIdx = args.indexOf("--origin");
  const host =
    originIdx >= 0 && args[originIdx + 1]
      ? args[originIdx + 1]
      : hostIdx >= 0 && args[hostIdx + 1] ? args[hostIdx + 1] : "acp";
  const portalIdx = args.indexOf("--portal");
  const portal = portalIdx >= 0 && args[portalIdx + 1] ? args[portalIdx + 1] : "file-search";
  const selectionIdx = args.indexOf("--selection");
  const selection =
    selectionIdx >= 0 && args[selectionIdx + 1] ? args[selectionIdx + 1] : "file";
  const queryIdx = args.indexOf("--query");
  const query = queryIdx >= 0 && args[queryIdx + 1] ? args[queryIdx + 1] : "AGENTS.md";
  const kindsIdx = args.indexOf("--kinds");
  const kinds =
    kindsIdx >= 0 && args[kindsIdx + 1]
      ? args[kindsIdx + 1].split(",").map((s) => s.trim()).filter(Boolean)
      : ["accessibility", "screen-recording", "microphone"];
  const chordIdx = args.indexOf("--chord");
  const chord = chordIdx >= 0 && args[chordIdx + 1] ? args[chordIdx + 1] : "cmd+shift+7";
  const actionIdx = args.indexOf("--action");
  const action =
    actionIdx >= 0 && args[actionIdx + 1] ? args[actionIdx + 1] : "test-agentic-shortcut";
  const templateIdx = args.indexOf("--template");
  const template =
    templateIdx >= 0 && args[templateIdx + 1] ? args[templateIdx + 1] : "Hello {{name}}";
  const fieldIdx = args.indexOf("--field");
  const field = fieldIdx >= 0 && args[fieldIdx + 1] ? args[fieldIdx + 1] : "name";
  const valueIdx = args.indexOf("--value");
  const value = valueIdx >= 0 && args[valueIdx + 1] ? args[valueIdx + 1] : "Ada";
  const forcedValueIdx = args.indexOf("--forced-value");
  const forcedValue =
    forcedValueIdx >= 0 && args[forcedValueIdx + 1]
      ? args[forcedValueIdx + 1]
      : "forced-template-result";
  const aliasIdx = args.indexOf("--alias");
  const alias =
    aliasIdx >= 0 && args[aliasIdx + 1] ? args[aliasIdx + 1] : "Do in Current Command";
  const expectedAppIdx = args.indexOf("--expected-app");
  const expectedApp =
    expectedAppIdx >= 0 && args[expectedAppIdx + 1] ? args[expectedAppIdx + 1] : undefined;
  const sourceIdx = args.indexOf("--source");
  const source = sourceIdx >= 0 && args[sourceIdx + 1] ? args[sourceIdx + 1] : "root-file";
  const mutationIdx = args.indexOf("--mutation");
  const mutation =
    mutationIdx >= 0 && args[mutationIdx + 1]
      ? args[mutationIdx + 1]
      : "filter-selection-cache-frame";
  const surfaceIdx = args.indexOf("--surface");
  const surface = surfaceIdx >= 0 && args[surfaceIdx + 1] ? args[surfaceIdx + 1] : "shortcuts";
  const sandboxConfig = args.includes("--sandbox-config");
  const vision = args.includes("--vision");

  return {
    session,
    scenario,
    index,
    minTargets,
    key,
    families,
    drift,
    host,
    portal,
    selection,
    query,
    kinds,
    chord,
    action,
    template,
    field,
    value,
    forcedValue,
    alias,
    expectedApp,
    source,
    mutation,
    surface,
    sandboxConfig,
    vision,
  };
}

// Only run CLI when executed directly (not imported)
if (import.meta.main) {
  const {
    session,
    scenario,
    index,
    minTargets,
    key,
    families,
    drift,
    host,
    portal,
    selection,
    query,
    kinds,
    chord,
    action,
    template,
    field,
    value,
    forcedValue,
    alias,
    expectedApp,
    source,
    mutation,
    surface,
    sandboxConfig,
    vision,
  } = parseArgs();

  const AVAILABLE_SCENARIOS = [
    "main-window-exact-id",
    "actions-dialog-exact-id",
    "prompt-popup-exact-id",
    "detached-acp-exact-id",
    "detached-acp-target-threading-stress",
    "acp-prompt-popup-parity",
    "notes-acp-delayed-action-origin-stress",
    "file-portal-origin-roundtrip",
    "permission-privacy-preflight",
    "shortcut-recorder-focus-capture",
    "template-prompt-automation-parity-stress",
    "current-app-commands-frontmost-stress",
    "actions-captured-subject-frame-stress",
  ];

  if (!scenario) {
    process.stderr.write(
      JSON.stringify({
        event: "scenario.error",
        error: "Missing --scenario flag",
        available: AVAILABLE_SCENARIOS,
      }) + "\n"
    );
    process.exit(2);
  }

  switch (scenario) {
    case "main-window-exact-id": {
      const bundle = await runMainWindowExactIdScenario(session);
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.warnings.length > 0 ? 1 : 0);
      break;
    }

    case "actions-dialog-exact-id": {
      const bundle = await runActionsDialogExactIdScenario(session, index);
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.warnings.length > 0 ? 1 : 0);
      break;
    }

    case "detached-acp-exact-id": {
      const bundle = await runDetachedAcpExactIdScenario(session, index);
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.warnings.length > 0 ? 1 : 0);
      break;
    }

    case "prompt-popup-exact-id": {
      const bundle = await runPromptPopupExactIdScenario(session, index);
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.warnings.length > 0 ? 1 : 0);
      break;
    }

    case "detached-acp-target-threading-stress": {
      const bundle = await runDetachedAcpTargetThreadingStressScenario({
        session,
        kind: "acpDetached",
        index,
        minTargets,
        key,
        vision,
      });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    case "acp-prompt-popup-parity": {
      const bundle = await runAcpPromptPopupParityScenario({ session, families });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    case "notes-acp-delayed-action-origin-stress": {
      const bundle = await runNotesAcpDelayedActionOriginStressScenario({ session, drift });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    case "file-portal-origin-roundtrip": {
      const bundle = await runAcpPortalRoundTripOriginStressScenario({ session, host, portal, selection, query });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    case "permission-privacy-preflight": {
      const bundle = await runPermissionPreflightReadonlyScenario({ session, kinds });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    case "shortcut-recorder-focus-capture": {
      const bundle = await runShortcutRecorderFocusCaptureStressScenario({ session, chord, action, surface, sandboxConfig });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    case "template-prompt-automation-parity-stress": {
      const bundle = await runTemplatePromptAutomationParityStressScenario({ session, template, field, value, forcedValue });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    case "current-app-commands-frontmost-stress": {
      const bundle = await runCurrentAppCommandsFrontmostStressScenario({ session, query, alias, expectedApp });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    case "actions-captured-subject-frame-stress": {
      const bundle = await runActionsCapturedSubjectFrameStressScenario({ session, source, action, mutation });
      process.stdout.write(JSON.stringify(bundle, null, 2) + "\n");
      process.exit(bundle.status === "pass" ? 0 : 1);
      break;
    }

    default:
      process.stderr.write(
        JSON.stringify({
          event: "scenario.error",
          error: `Unknown scenario: ${scenario}`,
          available: AVAILABLE_SCENARIOS,
        }) + "\n"
      );
      process.exit(2);
  }
}
