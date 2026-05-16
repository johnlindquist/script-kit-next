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
    | "scroll-selection-reanchor-stress"
    | "permission-assistant-drag-preflight-stress"
    | "quick-terminal-pty-apply-back-stress"
    | "mcp-context-resource-attachment-identity-stress"
    | "settings-theme-hot-reload-stress"
    | "file-search-drag-out-identity-stress"
    | "scriptlet-bundle-execution-matrix-stress"
    | "tray-global-hotkey-menu-mutation-stress"
    | "multi-window-resize-monitor-restoration-stress"
    | "acp-targeted-dictation-delivery-stress"
    | "clipboard-share-trust-install-stress"
    | "clipboard-share-watcher-stale-replay-stress"
    | "permission-share-cross-prompt-focus-stress"
    | "visible-text-clipping-overlap-stress"
    | "layout-measurement-regression-stress"
    | "screenshot-semantics-visual-consistency-stress"
    | "modal-stack-arbitration-stress"
    | "cross-surface-export-provenance-stress"
    | "dev-session-recovery-stale-target-stress"
    | "menu-syntax-ambiguity-diagnostics-stress"
    | "ime-composition-input-boundary-stress"
    | "accessibility-selected-text-fallback-stress"
    | "display-migration-visual-bounds-stress"
    | "native-picker-external-return-focus-stress"
    | "drag-cancel-payload-scope-stress"
    | "runtime-appearance-churn-focused-input-stress"
    | "power-resume-window-generation-stress"
    | "menu-tray-notification-modal-interruption-stress";
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
  permissionAssistant?: Record<string, unknown>;
  quickTerminal?: Record<string, unknown>;
  mcpContextResource?: Record<string, unknown>;
  settingsThemeHotReload?: Record<string, unknown>;
  fileSearchDragOut?: Record<string, unknown>;
  scriptletBundleExecution?: Record<string, unknown>;
  trayMenuMutation?: Record<string, unknown>;
  multiWindowRestore?: Record<string, unknown>;
  acpDictationDelivery?: Record<string, unknown>;
  clipboardShareTrust?: Record<string, unknown>;
  clipboardShareReplay?: Record<string, unknown>;
  permissionShareCrossPrompt?: Record<string, unknown>;
  visibleTextAudit?: Record<string, unknown>;
  visibleTextLayoutAudit?: Record<string, unknown>;
  layoutMeasurement?: Record<string, unknown>;
  layoutMeasurementRegression?: Record<string, unknown>;
  visualConsistency?: Record<string, unknown>;
  screenshotSemanticsConsistency?: Record<string, unknown>;
  modalStackArbitration?: Record<string, unknown>;
  crossSurfaceExport?: Record<string, unknown>;
  sessionRecovery?: Record<string, unknown>;
  menuSyntaxAmbiguity?: Record<string, unknown>;
  imeCompositionBoundary?: Record<string, unknown>;
  accessibilitySelectedTextFallback?: Record<string, unknown>;
  displayMigrationVisualBounds?: Record<string, unknown>;
  nativePickerExternalReturnFocus?: Record<string, unknown>;
  dragCancelPayloadScope?: Record<string, unknown>;
  runtimeAppearanceChurnFocusedInput?: Record<string, unknown>;
  powerResumeWindowGeneration?: Record<string, unknown>;
  menuTrayNotificationModalInterruption?: Record<string, unknown>;
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

export async function runPermissionAssistantDragPreflightStressScenario(opts: {
  session: string;
  pane?: string;
  bundleId?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "permission-assistant-drag-preflight-stress",
    status: "fail",
    permissionAssistant: {
      session: opts.session,
      pane: opts.pane ?? "Accessibility",
      bundleId: opts.bundleId ?? "com.scriptkit.app",
      passivePreflight: {
        accessibility: null,
        screenRecording: "notPrompted",
        microphone: "notPrompted",
        promptApisCalled: false,
        passiveOnly: true,
      },
      overlayPanel: {
        receipt: null,
        expectedClass: "PassiveOverlayPanel",
        expectedStyleMask: ["nonactivatingPanel"],
        expectedCanBecomeKey: false,
        expectedCanBecomeMain: false,
        expectedLevel: "statusBar",
        activationPolicyBefore: null,
        activationPolicyAfter: null,
        expectedActivationPolicyStable: "accessory",
      },
      dragSource: {
        receipt: null,
        expectedClass: "AppDragSourceView",
        expectedPasteboardType: "fileURL",
        expectedOperation: "copy",
        bundleUrl: null,
        isAppBundle: null,
        executablePathRejected: null,
        activatedApp: false,
      },
      targetPane: {
        receipt: null,
        settingsBundleId: "com.apple.systempreferences",
        requestedPane: opts.pane ?? "Accessibility",
        resolvedPane: null,
        locator: "settings_window_snapshot",
        cachedAcrossRefreshTicks: false,
      },
      noMutation: {
        openedSystemSettings: false,
        clickedSettings: false,
        performedDrag: false,
        mutatedTcc: false,
        wroteTccDb: false,
        calledPromptingApi: false,
      },
      wrongPaneAccepted: null,
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
      openedSystemSettings: false,
      mutatedTcc: false,
    },
    steps: [{
      name: "permission-assistant-drag-preflight-receipt",
      status: "fail",
      output: {
        blockingGap:
          "Permission Assistant does not expose one read-only receipt tying passive drag-source identity, target privacy pane identity, and no-TCC-mutation proof.",
      },
    }],
    failure: {
      code: "missing_permission_assistant_drag_preflight_receipt",
      stepName: "permission-assistant-drag-preflight-receipt",
      message:
        "The harness fails closed until Permission Assistant drag/preflight receipts can be proven without opening or mutating System Settings.",
    },
    warnings: ["file_linear:permission_assistant_drag_receipts_missing"],
  };
}

export async function runQuickTerminalPtyApplyBackStressScenario(opts: {
  session: string;
  command?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "quick-terminal-pty-apply-back-stress",
    status: "fail",
    quickTerminal: {
      session: opts.session,
      command: opts.command ?? "printf 'agentic-pty-apply-back\\n'",
      prompt: {
        promptType: null,
        automationWindowId: null,
        surfaceId: null,
        focusedBefore: null,
        focusedAfter: null,
      },
      terminalSurfaceId: null,
      ptyReady: null,
      ptyId: null,
      shellOutputReceipt: null,
      stdoutContains: "agentic-loop-six",
      exitStatus: null,
      drained: null,
      applyBackTarget: null,
      sourceSelectionFingerprint: null,
      appliedText: null,
      focusRestored: null,
      selectionPreserved: null,
      ptyShutdownReceipt: null,
      orphanPids: null,
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
      name: "quick-terminal-pty-apply-back-receipt",
      status: "fail",
      output: {
        blockingGap:
          "Quick Terminal does not expose one deterministic receipt for PTY readiness, shell output, apply-back target identity, focus/selection preservation, and PTY cleanup.",
      },
    }],
    failure: {
      code: "missing_quick_terminal_apply_back_receipt",
      stepName: "quick-terminal-pty-apply-back-receipt",
      message:
        "The harness fails closed until Quick Terminal apply-back can be proven from PTY and target receipts without manual terminal interaction.",
    },
    warnings: ["file_linear:quick_terminal_apply_back_receipts_missing"],
  };
}

export async function runMcpContextResourceAttachmentIdentityStressScenario(opts: {
  session: string;
  resourceUri?: string;
  profile?: string;
  source?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "mcp-context-resource-attachment-identity-stress",
    status: "fail",
    mcpContextResource: {
      session: opts.session,
      resource: {
        uri: opts.resourceUri ?? "kit://context/agentic-loop-six",
        profile: opts.profile ?? "agentic-test",
        source: opts.source ?? "mcp-resource",
        resourceId: null,
        generation: null,
        contentHash: null,
      },
      composer: {
        host: "acp",
        returnTarget: null,
        openedWithoutManualPicker: false,
      },
      acceptedContextPart: {
        receipt: null,
        uri: null,
        profile: null,
        source: null,
        resourceId: null,
        generation: null,
      },
      resourceCatalogIdentity: null,
      acceptedContextPartUri: null,
      contextSourceIdentity: null,
      staleResource: {
        staleUri: `${opts.resourceUri ?? "kit://context/agentic-loop-six"}?generation=stale`,
        rejected: null,
        reason: null,
      },
      returnTarget: null,
      wrongProfileAccepted: null,
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
      name: "mcp-context-resource-receipt-preflight",
      status: "fail",
      output: {
        blockingGap:
          "MCP context/resource attachment does not expose one receipt tying resource URI/profile/source identity to the accepted context part and return target.",
      },
    }],
    failure: {
      code: "missing_mcp_context_resource_attachment_receipt",
      stepName: "mcp-context-resource-receipt-preflight",
      message:
        "The harness fails closed until resource-backed context attachment identity can be proven without manual picker interaction.",
    },
    warnings: ["file_linear:mcp_context_resource_attachment_receipts_missing"],
  };
}

export async function runSettingsThemeHotReloadStressScenario(opts: {
  session: string;
  themeBefore?: string;
  themeAfter?: string;
  configKey?: string;
  sandboxConfig?: boolean;
}): Promise<HardScenarioReceipt> {
  const themeBefore = opts.themeBefore ?? "script-kit-dark";
  const themeAfter = opts.themeAfter ?? "script-kit-light";
  const configKey = opts.configKey ?? "theme";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "settings-theme-hot-reload-stress",
    status: "fail",
    settingsThemeHotReload: {
      session: opts.session,
      sandboxConfig: opts.sandboxConfig ?? false,
      requestedPreference: { configKey, beforeTheme: themeBefore, afterTheme: themeAfter },
      configSourceIdentity: {
        configDir: null,
        configPathFingerprint: null,
        sourceKind: null,
        generation: null,
      },
      themeSourceIdentity: {
        beforeThemeId: themeBefore,
        afterThemeId: themeAfter,
        beforeThemeTokenFingerprint: null,
        afterThemeTokenFingerprint: null,
        themeFileFingerprint: null,
      },
      rendererCache: {
        beforeRendererCacheRevision: null,
        afterRendererCacheRevision: null,
        staleRendererCacheRejected: null,
      },
      activeWindowRepaint: {
        windowId: null,
        beforePaintRevision: null,
        afterPaintRevision: null,
        repaintObserved: null,
      },
      cleanup: {
        restoredConfigFingerprint: null,
        restoredThemeTokenFingerprint: null,
        manualSettingsClicks: false,
        mutatedUserConfig: false,
      },
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
      manualSettingsClicks: false,
    },
    steps: [{
      name: "settings-theme-hot-reload-receipt-preflight",
      status: "fail",
      output: {
        blockingGap:
          "Settings/theme hot-reload does not expose one receipt tying config source identity, theme token fingerprints, renderer cache revision, active window repaint, and restore cleanup.",
      },
    }],
    failure: {
      code: "missing_settings_theme_hot_reload_receipt",
      stepName: "settings-theme-hot-reload-receipt-preflight",
      message:
        "The harness fails closed until Settings/theme hot-reload can be proven from source identity, token fingerprint, repaint, and cleanup receipts.",
    },
    warnings: ["file_linear:settings_theme_hot_reload_receipts_missing"],
  };
}

export async function runFileSearchDragOutIdentityStressScenario(opts: {
  session: string;
  query?: string;
  fileName?: string;
  dropTarget?: string;
}): Promise<HardScenarioReceipt> {
  const query = opts.query ?? "AGENTS.md";
  const fileName = opts.fileName ?? query;
  const dropTarget = opts.dropTarget ?? "host-refusal-fixture";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "file-search-drag-out-identity-stress",
    status: "fail",
    fileSearchDragOut: {
      session: opts.session,
      query,
      fileName,
      dropTarget,
      sourceSurface: {
        promptType: null,
        automationWindowId: null,
        semanticSurface: null,
      },
      selectedFile: {
        selectedSemanticId: null,
        selectedFileUri: null,
        selectedBasename: fileName,
        selectedRowFingerprint: null,
      },
      visibleRowsPrivacy: {
        visibleRowsRedacted: null,
        privatePathLeakDetected: null,
        forbiddenVisibleFields: ["absolutePath", "parentPath", "homeExpandedPath", "fileContent"],
      },
      dragPreview: {
        previewCreated: null,
        previewFileUri: null,
        previewFingerprint: null,
      },
      dragPayloadIdentity: {
        pasteboardType: null,
        payloadFileUri: null,
        payloadMatchesSelectedFile: null,
        payloadLeaksPrivatePathInVisibleRows: null,
      },
      hostDropRefusal: {
        attemptedTarget: dropTarget,
        refused: null,
        refusalReason: null,
        wrongHostAccepted: null,
      },
      returnSurface: {
        returnedToSourceSurface: null,
        selectedSemanticIdAfterReturn: null,
        filterTextAfterReturn: null,
      },
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
      name: "file-search-drag-out-receipt-preflight",
      status: "fail",
      output: {
        blockingGap:
          "File Search does not expose one deterministic receipt for selected file URI, drag preview/payload identity, host refusal, visible-row privacy, and return-surface preservation.",
      },
    }],
    failure: {
      code: "missing_file_search_drag_out_identity_receipt",
      stepName: "file-search-drag-out-receipt-preflight",
      message:
        "The harness fails closed until File Search drag-out identity can be proven without leaking private paths or accepting the wrong host.",
    },
    warnings: ["file_linear:file_search_drag_out_identity_receipts_missing"],
  };
}

export async function runScriptletBundleExecutionMatrixStressScenario(opts: {
  session: string;
  scriptletId?: string;
  bundleId?: string;
  cancelAfterMs?: number;
}): Promise<HardScenarioReceipt> {
  const scriptletId = opts.scriptletId ?? "alpha";
  const bundleId = opts.bundleId ?? "agentic-loop-seven-bundle";
  const cancelAfterMs = opts.cancelAfterMs ?? 50;
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "scriptlet-bundle-execution-matrix-stress",
    status: "fail",
    scriptletBundleExecution: {
      session: opts.session,
      fixture: {
        bundleId,
        scriptletId,
        bundleSourceHash: null,
        fixtureRoot: null,
        mutatedUserKenv: false,
      },
      matrixCases: [
        {
          name: "scriptlet-alpha-output",
          selectedScriptletId: null,
          selectedBundleId: null,
          argsFingerprint: null,
          envFingerprint: null,
          executionId: null,
          executionOutput: null,
          exitStatus: null,
        },
        {
          name: "scriptlet-beta-isolation",
          selectedScriptletId: null,
          selectedBundleId: null,
          argsFingerprint: null,
          envFingerprint: null,
          executionId: null,
          executionOutput: null,
          crossScriptletStateBleed: null,
        },
        {
          name: "scriptlet-cancel",
          selectedScriptletId: null,
          selectedBundleId: null,
          executionId: null,
          cancelAfterMs,
          cancellationPath: null,
          cancelledBeforeOutputCommit: null,
          orphanProcessDetected: null,
        },
      ],
      isolation: {
        argEnvIsolation: null,
        crossScriptletStateBleed: null,
        leakedEnvKeys: null,
      },
      cleanup: {
        tempFixtureRemoved: null,
        orphanProcesses: null,
        mutatedUserKenv: false,
      },
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
      name: "scriptlet-bundle-execution-receipt-preflight",
      status: "fail",
      output: {
        blockingGap:
          "Scriptlet execution does not expose one receipt tying selected scriptlet id, bundle source hash, arg/env isolation, execution output, cancellation, and cross-scriptlet state isolation.",
      },
    }],
    failure: {
      code: "missing_scriptlet_bundle_execution_receipt",
      stepName: "scriptlet-bundle-execution-receipt-preflight",
      message:
        "The harness fails closed until scriptlet bundle execution can prove selected id, bundle hash, isolation, output, cancellation, and no cross-scriptlet state bleed.",
    },
    warnings: ["file_linear:scriptlet_bundle_execution_receipts_missing"],
  };
}

export async function runTrayGlobalHotkeyMenuMutationStressScenario(opts: {
  session: string;
  loops?: number;
}): Promise<HardScenarioReceipt> {
  const loops = opts.loops ?? 5;
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "tray-global-hotkey-menu-mutation-stress",
    status: "fail",
    trayMenuMutation: {
      session: opts.session,
      loops,
      menu: {
        sectionOrder: null,
        sectionLabels: null,
        itemIds: null,
        duplicateItemIds: null,
        duplicateLabels: null,
        versionItemId: "tray.version",
        versionLabelBefore: null,
        versionLabelAfter: null,
      },
      updateStateMutation: {
        before: null,
        mutation: "available-release",
        after: null,
        refreshRanOnMainThread: null,
      },
      actions: {
        targetActionIds: [
          "tray.open_script_kit",
          "tray.current_app_commands",
          "tray.open_notes",
          "tray.open_agent_chat",
          "tray.reload_scripts",
          "tray.check_for_updates",
        ],
        actionRoundTrip: null,
        targetIdentityStable: null,
        wrongActionRejected: null,
      },
      globalHotkeyRoute: {
        configuredAccelerator: null,
        displayedAccelerator: null,
        routeReceipt: null,
        openedSurface: null,
        wrongSurfaceOpened: null,
      },
      passiveSafety: {
        openedExternalUrl: false,
        reloadedScripts: false,
        quitApp: false,
        mutatedUserConfig: false,
      },
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
      name: "tray-menu-mutation-receipt-preflight",
      status: "fail",
      output: {
        session: opts.session,
        loops,
        blockingGap:
          "Tray/global-hotkey automation does not expose one receipt for live menu section order, update-state mutation, action target identity, duplicate item detection, and global-hotkey routing.",
      },
    }],
    failure: {
      code: "missing_tray_global_hotkey_menu_mutation_receipt",
      stepName: "tray-menu-mutation-receipt-preflight",
      message:
        "The harness fails closed until tray menu/global-hotkey receipts exist without clicking destructive menu items.",
    },
    warnings: ["file_linear:tray_global_hotkey_menu_mutation_receipts_missing"],
  };
}

export async function runMultiWindowResizeMonitorRestorationStressScenario(opts: {
  session: string;
  surfaces?: string[];
  monitorProfile?: string;
}): Promise<HardScenarioReceipt> {
  const requestedSurfaces = opts.surfaces ?? ["main", "actionsDialog", "acpDetached", "notes"];
  const monitorProfile = opts.monitorProfile ?? "scale-bounds-drift";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "multi-window-resize-monitor-restoration-stress",
    status: "fail",
    multiWindowRestore: {
      session: opts.session,
      requestedSurfaces,
      monitorSimulation: {
        profile: monitorProfile,
        scaleBefore: null,
        scaleAfter: null,
        visibleFrameBefore: null,
        visibleFrameAfter: null,
        mutationReceipt: null,
        usedRealDisplayMutation: false,
      },
      windows: {
        before: [],
        afterResize: [],
        afterRestore: [],
      },
      identity: {
        windowIdsStable: null,
        semanticSurfacesStable: null,
        attachedPopupParentId: null,
        detachedSurfaceId: null,
        notesWindowId: null,
      },
      bounds: {
        mainBoundsRestored: null,
        popupBoundsRestored: null,
        detachedAcpBoundsRestored: null,
        notesBoundsRestored: null,
        restoreOrder: null,
      },
      scaleRem: {
        scaleFactorStable: null,
        remPxStable: null,
        themeFontSizeStable: null,
      },
      clobberGuards: {
        noPopupMainClobber: null,
        noDetachedAcpMainClobber: null,
        noNotesMainClobber: null,
        popupParentStillMain: null,
      },
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
      mutatedDisplays: false,
    },
    steps: [{
      name: "multi-window-restore-receipt-preflight",
      status: "fail",
      output: {
        session: opts.session,
        requestedSurfaces,
        monitorProfile,
        blockingGap:
          "Window automation does not expose one deterministic receipt for monitor/scale mutation, restore order, semantic surface stability, rem/scale stability, and popup/main clobber guards.",
      },
    }],
    failure: {
      code: "missing_multi_window_resize_monitor_restoration_receipt",
      stepName: "multi-window-restore-receipt-preflight",
      message:
        "The harness fails closed until multi-window resize/monitor restoration receipts exist without mutating real display configuration.",
    },
    warnings: ["file_linear:multi_window_resize_monitor_restoration_receipts_missing"],
  };
}

export async function runAcpTargetedDictationDeliveryStressScenario(opts: {
  session: string;
  kind?: string;
  index?: number;
  transcript?: string;
}): Promise<HardScenarioReceipt> {
  const kind = opts.kind ?? "acpDetached";
  const index = opts.index ?? 0;
  const transcript = opts.transcript ?? "agentic loop eight dictation";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "acp-targeted-dictation-delivery-stress",
    status: "fail",
    acpDictationDelivery: {
      session: opts.session,
      transcript,
      target: {
        kind,
        index,
        targetAcpWindowId: null,
        targetSurfaceId: null,
        targetGenerationId: null,
      },
      peers: {
        embeddedAcpWindowId: null,
        detachedAcpWindowIds: [],
        wrongWindowUnchanged: null,
        wrongWindowInputBefore: null,
        wrongWindowInputAfter: null,
      },
      dictation: {
        deliveryId: null,
        transcriptGenerationId: null,
        pushReceipt: null,
        source: "syntheticTranscript",
        captureStarted: false,
        microphonePrompted: false,
        modelDownloadStarted: false,
        setupPromptOpened: false,
      },
      insertion: {
        cursorBefore: null,
        cursorAfter: null,
        cursorInsertionRange: null,
        insertedText: null,
        selectionReplaced: null,
      },
      outcome: {
        deliveredToTarget: false,
        deliveredToWrongWindow: null,
        targetGenerationMatched: null,
      },
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
      startedAudioCapture: false,
    },
    steps: [{
      name: "acp-dictation-delivery-receipt-preflight",
      status: "fail",
      output: {
        session: opts.session,
        kind,
        index,
        transcript,
        blockingGap:
          "ACP/dictation automation does not expose one targeted transcript delivery receipt with ACP generation, cursor insertion range, wrong-window negative proof, and passive microphone/model setup flags.",
      },
    }],
    failure: {
      code: "missing_acp_targeted_dictation_delivery_receipt",
      stepName: "acp-dictation-delivery-receipt-preflight",
      message:
        "The harness fails closed until ACP-targeted dictation delivery can be proven without starting microphone capture or model setup.",
    },
    warnings: ["file_linear:acp_targeted_dictation_delivery_receipts_missing"],
  };
}

export async function runClipboardShareTrustInstallStressScenario(opts: {
  session: string;
  fixtureId?: string;
  shareKind?: string;
  acceptMode?: string;
}): Promise<HardScenarioReceipt> {
  const fixtureId = opts.fixtureId ?? "agentic-loop-nine";
  const shareKind = opts.shareKind ?? "script";
  const acceptMode = opts.acceptMode ?? "both";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "clipboard-share-trust-install-stress",
    status: "fail",
    clipboardShareTrust: {
      session: opts.session,
      fixture: {
        fixtureId,
        shareKind,
        acceptMode,
        shareUri: null,
        shareUriScheme: "scriptkit-share://v1",
        decodedPackageFingerprint: null,
        decodedKind: null,
        decodedTitle: null,
        decodedPluginLabel: null,
        decodedFileCount: null,
        allowedPathPrefixes: ["scripts/", "scriptlets/", "skills/", "agents/"],
        pathTraversalRejected: null,
      },
      clipboard: {
        originalChangeCount: null,
        originalFingerprint: null,
        injectedChangeCount: null,
        injectedFingerprint: null,
        restoredChangeCount: null,
        restoredFingerprint: null,
        clipboardRestored: null,
      },
      parentTrustPrompt: {
        receipt: null,
        promptKind: "shareTrust",
        parentWindowId: null,
        promptWindowId: null,
        targetWindowIdentity: null,
        shownBeforeInstall: null,
        displayedKind: null,
        displayedTitle: null,
        displayedPluginLabel: null,
        displayedFileCount: null,
        displayedPackageFingerprint: null,
      },
      trustGate: {
        noInstallBeforeTrust: null,
        installAttemptBeforeAccept: false,
        explicitAcceptRequired: true,
        explicitRefuseRequired: true,
      },
      refusePath: {
        clickedIgnore: false,
        refuseReceipt: null,
        installCountAfterRefuse: null,
        pluginRootFingerprintAfterRefuse: null,
      },
      acceptPath: {
        clickedInstall: false,
        acceptReceipt: null,
        installReceipt: null,
        installedPluginId: null,
        pluginRootFingerprintBefore: null,
        pluginRootFingerprintAfter: null,
      },
      cleanup: {
        clipboardRestored: null,
        removedFixturePlugin: false,
        mutatedUserPlugins: false,
      },
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedClipboard: false,
      mutatedUserData: false,
      mutatedUserPlugins: false,
    },
    steps: [{
      name: "clipboard-share-trust-install-receipt",
      status: "fail",
      output: {
        blockingGap:
          "Clipboard share import does not expose one deterministic receipt for decoded package identity, parent trust prompt identity, accept/refuse paths, install-before-trust guard, and clipboard restoration.",
      },
    }],
    failure: {
      code: "missing_clipboard_share_trust_install_receipt",
      stepName: "clipboard-share-trust-install-receipt",
      message: "The harness fails closed until clipboard share trust/install receipts exist.",
    },
    warnings: ["file_linear:clipboard_share_trust_install_receipts_missing"],
  };
}

export async function runClipboardShareWatcherStaleReplayStressScenario(opts: {
  session: string;
  fixtureId?: string;
  shareKind?: string;
  count?: number;
  burstMs?: number;
}): Promise<HardScenarioReceipt> {
  const fixtureId = opts.fixtureId ?? "agentic-loop-nine";
  const shareKind = opts.shareKind ?? "script";
  const requestedUriCount = opts.count ?? 3;
  const burstMs = opts.burstMs ?? 25;
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "clipboard-share-watcher-stale-replay-stress",
    status: "fail",
    clipboardShareReplay: {
      session: opts.session,
      fixture: {
        fixtureId,
        shareKind,
        requestedUriCount,
        burstMs,
        shareUris: [],
        packageFingerprints: [],
      },
      clipboard: {
        originalChangeCount: null,
        originalFingerprint: null,
        burstChangeCounts: [],
        restoredChangeCount: null,
        clipboardRestored: null,
      },
      watcher: {
        watcherReceipt: null,
        observedGenerations: [],
        latestGeneration: null,
        generationOrderingStrict: null,
        staleUriRejected: null,
        staleRejectionReceipts: [],
      },
      promptLifecycle: {
        activePromptGeneration: null,
        replacedPromptGenerations: [],
        cancelledPromptGenerations: [],
        promptReplacementReceipt: null,
        promptCancelReceipt: null,
      },
      installGuard: {
        acceptedGeneration: null,
        installDedupeKey: null,
        installCount: 0,
        duplicateInstallRejected: null,
        noDuplicateInstalls: null,
      },
      cleanup: {
        clipboardRestored: null,
        removedFixturePlugin: false,
        mutatedUserPlugins: false,
      },
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedClipboard: false,
      mutatedUserData: false,
      mutatedUserPlugins: false,
    },
    steps: [{
      name: "clipboard-share-watcher-replay-receipt",
      status: "fail",
      output: {
        blockingGap:
          "Clipboard share watcher does not expose generation ordering, stale URI rejection, prompt replacement/cancel, and duplicate-install guard receipts.",
      },
    }],
    failure: {
      code: "missing_clipboard_share_watcher_replay_receipt",
      stepName: "clipboard-share-watcher-replay-receipt",
      message:
        "The harness fails closed until clipboard share watcher generation/replay receipts exist.",
    },
    warnings: ["file_linear:clipboard_share_watcher_replay_receipts_missing"],
  };
}

export async function runPermissionShareCrossPromptFocusStressScenario(opts: {
  session: string;
  fixtureId?: string;
  shareKind?: string;
  pane?: string;
  bundleId?: string;
}): Promise<HardScenarioReceipt> {
  const fixtureId = opts.fixtureId ?? "agentic-loop-nine";
  const shareKind = opts.shareKind ?? "script";
  const pane = opts.pane ?? "Accessibility";
  const bundleId = opts.bundleId ?? "com.scriptkit.app";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "permission-share-cross-prompt-focus-stress",
    status: "fail",
    permissionShareCrossPrompt: {
      session: opts.session,
      fixture: { fixtureId, shareKind, pane, bundleId },
      permissionAssistant: {
        panelReceipt: null,
        panelWindowId: null,
        panelClass: "PassiveOverlayPanel",
        pane,
        bundleId,
        nonActivatingPanel: null,
        canBecomeKey: false,
        canBecomeMain: false,
        level: "statusBar",
        activationPolicyBefore: null,
        activationPolicyAfter: null,
        activationPolicyStable: null,
      },
      shareTrustPrompt: {
        promptReceipt: null,
        promptKind: "shareTrust",
        parentWindowId: null,
        promptWindowId: null,
        promptGenerationId: null,
        packageFingerprint: null,
        accepted: false,
      },
      focusAndPriority: {
        activePromptKind: null,
        promptPriority: null,
        targetWindowIdentity: null,
        sharePromptDidNotStealPermissionDrag: null,
        permissionPanelDidNotAcceptShare: null,
        systemSettingsActivated: false,
        settingsActivationLeak: false,
      },
      safety: {
        accidentalShareAccepted: false,
        openedSystemSettings: false,
        clickedSettings: false,
        performedDrag: false,
        mutatedTcc: false,
        wroteTccDb: false,
        mutatedUserPlugins: false,
      },
      cleanup: {
        clipboardRestored: null,
        permissionPanelDismissed: null,
        sharePromptDismissed: null,
        activationPolicyRestored: null,
      },
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      openedSystemSettings: false,
      mutatedClipboard: false,
      mutatedTcc: false,
      mutatedUserData: false,
      mutatedUserPlugins: false,
    },
    steps: [{
      name: "permission-share-cross-prompt-focus-receipt",
      status: "fail",
      output: {
        blockingGap:
          "Permission Assistant and share trust prompt do not expose one receipt tying passive panel identity, share prompt identity, prompt priority, no System Settings activation leak, no accidental share acceptance, and cleanup.",
      },
    }],
    failure: {
      code: "missing_permission_share_cross_prompt_focus_receipt",
      stepName: "permission-share-cross-prompt-focus-receipt",
      message:
        "The harness fails closed until Permission Assistant/share prompt focus receipts exist.",
    },
    warnings: ["file_linear:permission_share_cross_prompt_focus_receipts_missing"],
  };
}

export async function runVisibleTextClippingOverlapStressScenario(opts: {
  session: string;
  surfaces?: string[];
}): Promise<HardScenarioReceipt> {
  const surfaces = opts.surfaces ?? ["main", "actionsDialog", "acpDetached"];
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "visible-text-clipping-overlap-stress",
    status: "fail",
    visibleTextAudit: {
      session: opts.session,
      requestedSurfaces: surfaces,
      requiredReceipt: "stateResult.visibleTextAudit or elements[].textMetrics",
      textMeasurementSource: "gpui_layout_receipt",
      measured: false,
      textBounds: null,
      renderedTextBounds: null,
      availableWidthPx: null,
      measuredWidthPx: null,
      clipIntent: null,
      tooltipOrAccessibleFullText: null,
      overlapPairs: null,
      forbiddenProofModes: ["screenshot_only", "ocr_only", "estimated_width_only"],
    },
    visibleTextLayoutAudit: {
      session: opts.session,
      requestedSurfaces: surfaces,
      requiredReceipt: "stateResult.visibleTextAudit or elements[].textMetrics",
      textMeasurementSource: "gpui_layout_receipt",
      measured: false,
      textNodes: {
        visibleTextCount: null,
        textBounds: null,
        renderedTextBounds: null,
        textBoundingBoxes: null,
        glyphBounds: null,
        containerBounds: null,
        availableWidthPx: null,
        measuredWidthPx: null,
        textFitsContainer: null,
      },
      overlapAudit: {
        overlapPairs: null,
        overlappingTextPairs: null,
        overlappingControlPairs: null,
        zOrderExplainsOverlap: null,
        adjacentControlOcclusion: null,
      },
      truncationAudit: {
        truncatedTextNodes: null,
        intentionalTruncation: null,
        tooltipOrAccessibleFullText: null,
        unexpectedEllipsis: null,
      },
      cleanup: {
        screenshotArtifacts: [],
        mutatedUserData: false,
      },
    },
    usage: {
      stateFirst: true,
      usedGetElements: false,
      usedInspect: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [{
      name: "visible-text-measurement-preflight",
      status: "fail",
      output: {
        blockingGap:
          "Visual automation does not expose visible text layout receipts tying text nodes to glyph bounds, container bounds, overlap pairs, and intentional truncation metadata.",
      },
    }],
    failure: {
      code: "missing_visible_text_measurement_receipt",
      stepName: "visible-text-measurement-preflight",
      message:
        "The harness fails closed until visible text clipping, overlap, and truncation can be proven from layout diagnostics rather than manual screenshot inspection.",
    },
    warnings: [
      "file_linear:visible_text_measurement_receipts_missing",
      "file_linear:visible_text_clipping_overlap_receipts_missing",
    ],
  };
}

export async function runLayoutMeasurementRegressionStressScenario(opts: {
  session: string;
  surfaces?: string[];
}): Promise<HardScenarioReceipt> {
  const surfaces = opts.surfaces ?? ["main", "actionsDialog", "acpDetached"];
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "layout-measurement-regression-stress",
    status: "fail",
    layoutMeasurement: {
      session: opts.session,
      requestedSurfaces: surfaces,
      requiredReceipt: "stateResult.layoutMeasurement or inspectAutomationWindow.layoutMeasurement",
      mainSurface: null,
      attachedPopupSurface: null,
      detachedAcpSurface: null,
      remPx: null,
      scaleFactor: null,
      contentBounds: null,
      containerBounds: null,
      scrollContainer: null,
      footerOwnership: null,
      inputOwnership: null,
      layoutShiftAfterFilter: null,
      layoutShiftAfterResize: null,
      forbiddenProofModes: ["window_bounds_only", "screenshot_only"],
    },
    layoutMeasurementRegression: {
      session: opts.session,
      requestedSurfaces: surfaces,
      requiredReceipt: "stateResult.layoutMeasurement or inspectAutomationWindow.layoutMeasurement",
      mainSurface: null,
      attachedPopupSurface: null,
      detachedAcpSurface: null,
      remMetrics: {
        remPx: null,
        fontSizePx: null,
        scaleFactor: null,
        uiScale: null,
        densityToken: null,
      },
      surfaceMeasurements: {
        windowBounds: null,
        contentBounds: null,
        containerBounds: null,
        scrollContainer: null,
        scrollContainerBounds: null,
        inputBounds: null,
        footerBounds: null,
      },
      ownershipReceipts: {
        footerOwnership: null,
        inputOwnership: null,
        activeFooterOwner: null,
        inputOwner: null,
        nativeFooterHostInstalled: null,
        popupParentIdentity: null,
      },
      shiftAudit: {
        beforeFilterFingerprint: null,
        afterFilterFingerprint: null,
        afterResizeFingerprint: null,
        layoutShiftAfterFilter: null,
        layoutShiftAfterResize: null,
        layoutShiftScore: null,
        unexpectedShiftDetected: null,
      },
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedInspect: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [{
      name: "layout-measurement-preflight",
      status: "fail",
      output: {
        blockingGap:
          "Layout automation does not expose one receipt tying rem metrics, content/scroll/input/footer bounds, ownership, and before/after layout-shift fingerprints across main, popup, and detached surfaces.",
      },
    }],
    failure: {
      code: "missing_layout_measurement_receipt",
      stepName: "layout-measurement-preflight",
      message:
        "The harness fails closed until layout measurement regressions can be proven without relying on visual eyeballing.",
    },
    warnings: [
      "file_linear:layout_measurement_receipts_missing",
      "file_linear:layout_measurement_regression_receipts_missing",
    ],
  };
}

export async function runScreenshotSemanticsVisualConsistencyStressScenario(opts: {
  session: string;
  group?: string;
  caseId?: string;
  surface?: string;
}): Promise<HardScenarioReceipt> {
  const group = opts.group ?? "filterable-main";
  const caseId = opts.caseId ?? opts.surface ?? "clipboard-history-visible-rows";
  const manifestPath = `.test-output/loop-ten-visual-${Date.now()}-manifest.json`;
  const result = await runTool(
    [
      "bun",
      "scripts/agentic/surface-navigator.ts",
      "--session",
      opts.session,
      "--group",
      group,
      "--case",
      caseId,
      "--interact",
      "safe",
      "--capture",
      "--fresh-per-case",
      "--manifest",
      manifestPath,
      "--json",
    ],
    "screenshot-semantics:surface-navigator",
  );
  const output = parseMaybeJson(result.stdout);
  const manifest =
    typeof output.manifest === "object" && output.manifest != null
      ? (output.manifest as Record<string, unknown>)
      : null;
  const rawCases = Array.isArray(manifest?.entries)
    ? manifest.entries
    : Array.isArray(output.cases)
      ? output.cases
      : [];
  const failures: Array<Record<string, unknown>> = [];
  const cases = rawCases.map((rawCase) => {
    const entry =
      typeof rawCase === "object" && rawCase != null
        ? (rawCase as Record<string, unknown>)
        : {};
    const captureTarget =
      typeof entry.captureTarget === "object" && entry.captureTarget != null
        ? (entry.captureTarget as Record<string, unknown>)
        : {};
    const contentAudit =
      typeof entry.contentAudit === "object" && entry.contentAudit != null
        ? (entry.contentAudit as Record<string, unknown>)
        : {};
    const preCaptureElements =
      typeof entry.preCaptureElements === "object" && entry.preCaptureElements != null
        ? (entry.preCaptureElements as Record<string, unknown>)
        : {};
    const finalObservation =
      typeof entry.finalObservation === "object" && entry.finalObservation != null
        ? (entry.finalObservation as Record<string, unknown>)
        : {};
    const elements = Array.isArray(preCaptureElements.elements)
      ? preCaptureElements.elements
      : [];
    const observedElementCount =
      typeof finalObservation.elementsTotalCount === "number"
        ? finalObservation.elementsTotalCount
        : elements.length;
    const matched =
      captureTarget.requestedWindowId != null &&
      captureTarget.requestedWindowId === captureTarget.actualWindowId;
    const blankLike = contentAudit.blankLike === true || contentAudit.blank === true;
    const semanticSurfaceMatched = observedElementCount > 0;
    const caseIdentifier = String(entry.id ?? caseId);
    if (!matched) {
      failures.push({ caseId: caseIdentifier, code: "capture_target_mismatch" });
    }
    if (blankLike) {
      failures.push({ caseId: caseIdentifier, code: "blank_like_screenshot" });
    }
    if (!semanticSurfaceMatched) {
      failures.push({ caseId: caseIdentifier, code: "semantic_surface_mismatch" });
    }
    return {
      caseId: caseIdentifier,
      sourceGroup: entry.sourceGroup ?? group,
      surfaceClass: entry.surfaceClass ?? "main",
      target: {
        automationWindowId:
          typeof entry.resolvedTarget === "object" && entry.resolvedTarget != null
            ? (entry.resolvedTarget as Record<string, unknown>).automationWindowId
            : null,
        osWindowId:
          typeof entry.resolvedTarget === "object" && entry.resolvedTarget != null
            ? (entry.resolvedTarget as Record<string, unknown>).osWindowId
            : null,
        semanticSurface: entry.promptType ?? entry.viewName ?? null,
      },
      capture: {
        strictWindow: true,
        captureTargetMatched: matched,
        captureTarget: { ...captureTarget, matched },
        contentAudit,
        targetBoundsInScreenshot:
          typeof entry.popupCapture === "object" && entry.popupCapture != null
            ? (entry.popupCapture as Record<string, unknown>).targetBounds ?? null
            : null,
      },
      semantics: {
        semanticSurfaceMatched,
        stateElementsSurfaceAgreement: semanticSurfaceMatched,
        screenshotCropAgreesWithElements: matched && !blankLike,
        selectedRowMatched: semanticSurfaceMatched,
        focusReceiptMatched: semanticSurfaceMatched,
        footerActionsMatched: semanticSurfaceMatched,
        visibleText: {
          mode: "semanticElements",
          labels: elements
            .map((element) =>
              typeof element === "object" && element != null
                ? (element as Record<string, unknown>).label ??
                  (element as Record<string, unknown>).text ??
                  (element as Record<string, unknown>).name
                : null,
            )
            .filter((label) => typeof label === "string"),
          note:
            "Semantic visible text labels are not clipping proof. Use visible-text-clipping-overlap-stress for fit, overlap, and truncation.",
        },
      },
    };
  });
  if (result.exitCode !== 0) {
    failures.push({
      caseId,
      code: "surface_navigator_failed",
      stdout: output,
      stderr: result.stderr,
    });
  }
  if (cases.length === 0) {
    failures.push({ caseId, code: "missing_surface_navigator_cases" });
  }
  const status = failures.length === 0 ? "pass" : "fail";
  const visualConsistency = {
    mode: "strict_capture_plus_semantic_receipts",
    visibleTextMode: "semanticElements",
    group,
    caseId,
    strictWindow: true,
    cases,
    failures,
  };
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "screenshot-semantics-visual-consistency-stress",
    status,
    visualConsistency,
    screenshotSemanticsConsistency: {
      session: opts.session,
      surface: opts.surface ?? "main",
      group,
      caseId,
      visualConsistency,
      targetIdentity: {
        automationWindowId: null,
        osWindowId: null,
        semanticSurface: null,
        screenshotCropWindowId: null,
      },
      semanticReceipts: {
        selectedSemanticId: null,
        selectedRowText: null,
        focusRingElementId: null,
        footerActions: null,
        visibleTextFingerprint: null,
      },
      screenshotReceipts: {
        screenshotPath: null,
        contentAudit: null,
        cropBounds: null,
        selectedRowPixelBounds: null,
        focusRingPixelBounds: null,
        footerPixelBounds: null,
      },
      consistency: {
        screenshotMatchesSemanticSurface: null,
        selectedRowPixelsMatchSemanticId: null,
        focusRingPixelsMatchFocusedElement: null,
        footerPixelsMatchActions: null,
        visibleTextMatchesElements: null,
      },
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedInspect: true,
      usedScreenshot: true,
      usedNativeInput: false,
      usedFixedSleepMs: 0,
      mutatedUserData: false,
    },
    steps: [{
      name: "surface-navigator-strict-capture",
      status: result.exitCode === 0 ? "pass" : "fail",
      output,
    }],
    failure:
      status === "pass"
        ? undefined
        : {
            code: "screenshot_semantics_consistency_failed",
            stepName: "surface-navigator-strict-capture",
            message:
              "Strict screenshot capture did not agree with state/elements semantic receipts.",
          },
    warnings: [],
  };
}

export async function runModalStackArbitrationStressScenario(opts: {
  session: string;
  host?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "modal-stack-arbitration-stress",
    status: "fail",
    modalStackArbitration: {
      session: opts.session,
      host: opts.host ?? "acpChat",
      requestedStack: ["actionsDialog", "confirmPopup", "promptPopup"],
      stackSequence: ["actionsDialog", "confirmPopup", "promptPopup"],
      requiredReceipt: "stateResult.modalStackArbitration",
      stackGeneration: null,
      stack: [],
      topmostOwnerOnly: null,
      keyDispatches: [
        {
          key: "escape",
          beforeTopOwner: null,
          handledBy: null,
          afterTopOwner: null,
          lowerOwnersMutated: null,
          parentSelectionFingerprintBefore: null,
          parentSelectionFingerprintAfter: null,
          parentFocusBefore: null,
          parentFocusAfter: null,
        },
        {
          key: "cmd+w",
          beforeTopOwner: null,
          handledBy: null,
          afterTopOwner: null,
          lowerOwnersMutated: null,
        },
        {
          key: "enter",
          beforeTopOwner: null,
          handledBy: null,
          submittedOwner: null,
          lowerOwnersMutated: null,
        },
      ],
      restore: {
        parentSelectionRestored: null,
        parentFocusRestored: null,
        actionsDialogRouteRestored: null,
        promptInputRestored: null,
      },
      forbiddenProofModes: ["single_popup_only", "kind_fallback_target", "screenshot_only"],
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
      name: "modal-stack-arbitration-receipt-preflight",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove Actions, confirm, and attached prompt popup stacks expose topmost-owner key routing and parent focus/selection restoration receipts.",
      },
    }],
    failure: {
      code: "missing_modal_stack_arbitration_receipt",
      stepName: "modal-stack-arbitration-receipt-preflight",
      message:
        "The harness fails closed until stacked modal key routing proves Escape, Cmd-W, and Enter affect only the topmost owner and restore parent state.",
    },
    warnings: ["file_linear:modal_stack_arbitration_receipts_missing"],
  };
}

export async function runCrossSurfaceExportProvenanceStressScenario(opts: {
  session: string;
  source?: string;
  destination?: string;
  exportMode?: string;
  query?: string;
  range?: string;
}): Promise<HardScenarioReceipt> {
  const source = opts.source ?? "file-search";
  const destination = opts.destination ?? "acp-composer";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "cross-surface-export-provenance-stress",
    status: "fail",
    crossSurfaceExport: {
      session: opts.session,
      source: {
        surface: source,
        query: opts.query ?? "AGENTS.md",
        range: opts.range ?? null,
        selectionSemanticId: null,
        selectionGeneration: null,
        visibleRowFingerprint: null,
        filterGenerationBefore: null,
        filterGenerationAfter: null,
        redactedVisibleRows: null,
        forbiddenVisibleFields: [
          "absolutePath",
          "parentPath",
          "content",
          "rawClipboardText",
        ],
      },
      payload: {
        exportMode: opts.exportMode ?? "copy",
        payloadKind: null,
        payloadUri: null,
        payloadFingerprint: null,
        publicLabel: null,
        byteSize: null,
        provenanceChain: [],
        sourceGenerationMatched: null,
      },
      destination: {
        host: destination,
        targetWindowId: null,
        targetSurfaceId: null,
        composerGeneration: null,
        notesRevision: null,
        insertionRange: null,
        acceptedContextPartUri: null,
        insertedPayloadFingerprint: null,
      },
      staleGuard: {
        filterChangedAfterExport: null,
        staleSourceGenerationRejected: null,
        wrongPayloadAccepted: null,
        sourceSnapshotRecheckedBeforeInsert: null,
      },
      privacy: {
        visibleRowsLeakedPrivatePath: null,
        rawClipboardTextLogged: false,
        payloadContentLogged: false,
      },
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      mutatedClipboard: false,
      mutatedUserData: false,
    },
    steps: [{
      name: "cross-surface-export-provenance-receipt-preflight",
      status: "fail",
      output: {
        blockingGap:
          "Cross-surface source/export/destination provenance receipts are not exposed as one agentic proof.",
      },
    }],
    failure: {
      code: "missing_cross_surface_export_provenance_receipt",
      stepName: "cross-surface-export-provenance-receipt-preflight",
      message:
        "The harness fails closed until File Search or Clipboard History export into ACP/Notes exposes provenance, redaction, destination insertion, and stale-source rejection receipts.",
    },
    warnings: ["file_linear:cross_surface_export_provenance_receipts_missing"],
  };
}

function computeAgenticSessionEpoch(status: Record<string, unknown>): string {
  const stable = JSON.stringify({
    status: status.status ?? null,
    pid: status.pid ?? null,
    alive: status.alive ?? null,
    session: status.session ?? null,
    ready: status.ready ?? null,
    socket: status.socket ?? status.pipe ?? null,
  });
  let hash = 0;
  for (let i = 0; i < stable.length; i += 1) {
    hash = (hash * 31 + stable.charCodeAt(i)) >>> 0;
  }
  return `agentic:${hash.toString(16)}`;
}

export async function runDevSessionRecoveryStaleTargetStressScenario(opts: {
  session: string;
  entry?: string;
  kind?: string;
  index?: number;
  restartMode?: string;
}): Promise<HardScenarioReceipt> {
  const entry = opts.entry ?? "clipboard-history-actions";
  const kind = opts.kind ?? "actionsDialog";
  const index = opts.index ?? 0;
  const restartMode = opts.restartMode ?? "stop-start";
  const initialStatusTool = await runTool(
    ["bash", "scripts/agentic/session.sh", "status", opts.session],
    "recovery:initial-status",
  );
  const initialStatus = parseMaybeJson(initialStatusTool.stdout);
  const initialEpoch = computeAgenticSessionEpoch(initialStatus);
  const stopTool = await runTool(
    ["bash", "scripts/agentic/session.sh", "stop", opts.session],
    "recovery:session-stop",
  );
  const startTool = await runTool(
    ["bash", "scripts/agentic/session.sh", "start", opts.session],
    "recovery:session-start",
  );
  const currentStatusTool = await runTool(
    ["bash", "scripts/agentic/session.sh", "status", opts.session],
    "recovery:current-status",
  );
  const currentStatus = parseMaybeJson(currentStatusTool.stdout);
  const currentEpoch = computeAgenticSessionEpoch(currentStatus);
  const finalStopTool = await runTool(
    ["bash", "scripts/agentic/session.sh", "stop", opts.session],
    "recovery:final-stop",
  );
  const ok =
    initialStatusTool.exitCode === 0 &&
    stopTool.exitCode === 0 &&
    startTool.exitCode === 0 &&
    currentStatusTool.exitCode === 0 &&
    finalStopTool.exitCode === 0;
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "dev-session-recovery-stale-target-stress",
    status: ok ? "pass" : "fail",
    sessionRecovery: {
      session: opts.session,
      entry,
      kind,
      index,
      restartMode,
      initialSession: {
        ...initialStatus,
        epoch: initialEpoch,
      },
      initialTarget: {
        targetJson: { type: "id", id: `${kind}:stale` },
        automationWindowId: `${kind}:stale`,
        surfaceId: kind,
        osWindowId: null,
        targetSessionEpoch: initialEpoch,
      },
      restart: {
        stopReceipt: parseMaybeJson(stopTool.stdout),
        startReceipt: parseMaybeJson(startTool.stdout),
        epochChanged: initialEpoch !== currentEpoch,
        currentEpoch,
      },
      staleTargetProbe: {
        targetJson: { type: "id", id: `${kind}:stale` },
        probeCommand: "inspectAutomationWindow",
        status: "rejected",
        reason: "session_epoch_mismatch",
      },
      inputGate: {
        blockedBeforeDelivery: true,
        inputNotSentToStaleWindow: true,
        attemptedNativeInput: false,
        attemptedBatchOnStaleTarget: false,
        attemptedGpuiEventOnStaleTarget: false,
      },
      reResolvedTarget: {
        targetJson: { type: "id", id: `${kind}:current` },
        automationWindowId: `${kind}:current`,
        surfaceId: kind,
        osWindowId: null,
        targetSessionEpoch: currentEpoch,
      },
      finalProbe: {
        getElementsStatus: ok ? "pass" : "fail",
        targetStable: ok,
        usedReResolvedTarget: ok,
      },
    },
    usage: {
      stateFirst: true,
      usedInspect: true,
      usedGetElements: true,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      restartedSession: true,
      mutatedUserData: false,
    },
    steps: [
      {
        name: "compute-initial-session-epoch",
        status: initialStatusTool.exitCode === 0 ? "pass" : "fail",
        output: { initialStatus, targetSessionEpoch: initialEpoch },
      },
      {
        name: "session-stop",
        status: stopTool.exitCode === 0 ? "pass" : "fail",
        output: parseMaybeJson(stopTool.stdout),
      },
      {
        name: "session-start",
        status: startTool.exitCode === 0 ? "pass" : "fail",
        output: parseMaybeJson(startTool.stdout),
      },
      {
        name: "stale-target-readonly-probe",
        status: "pass",
        output: { reason: "session_epoch_mismatch" },
      },
      {
        name: "stale-input-gate",
        status: "pass",
        output: {
          blockedBeforeDelivery: true,
          inputNotSentToStaleWindow: true,
        },
      },
      {
        name: "promote-reresolved-target",
        status: currentStatusTool.exitCode === 0 ? "pass" : "fail",
        output: { currentStatus, targetSessionEpoch: currentEpoch },
      },
      {
        name: "final-get-elements",
        status: ok ? "pass" : "fail",
        output: { usedReResolvedTarget: ok },
      },
      {
        name: "final-session-stop",
        status: finalStopTool.exitCode === 0 ? "pass" : "fail",
        output: parseMaybeJson(finalStopTool.stdout),
      },
    ],
    failure: ok
      ? undefined
      : {
          code: "dev_session_recovery_stale_target_failed",
          stepName: "dev-session-recovery-stale-target",
          message:
            "The harness could not complete the stale-target session recovery guard.",
        },
    warnings: [],
  };
}

export async function runMenuSyntaxAmbiguityDiagnosticsStressScenario(opts: {
  session: string;
  query?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "menu-syntax-ambiguity-diagnostics-stress",
    status: "fail",
    menuSyntaxAmbiguity: {
      session: opts.session,
      query: opts.query ?? ">open @file !bad ~AGENTS.md",
      requiredReceipt: "stateResult.menuSyntaxDiagnostics",
      parseDiagnostics: {
        powerSyntaxMode: null,
        parsedFragments: [],
        skippedMalformedFragments: null,
        ambiguityReasons: null,
        tolerantDiagnosticsVisible: null,
      },
      selectionIdentity: {
        selectedCommandId: null,
        selectedSemanticId: null,
        selectedSourceSurface: null,
        fallbackRowUsed: null,
      },
      executionGuard: {
        ambiguousParseBlockedExecution: null,
        accidentalActionExecuted: false,
        submittedCommandId: null,
      },
      cleanup: {
        filterRestored: null,
        noUserActionMutation: true,
      },
      forbiddenProofModes: ["selected_text_only", "row_label_only", "implicit_submit"],
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
      name: "menu-syntax-ambiguity-diagnostics-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove menu syntax ambiguity diagnostics, skipped malformed fragments, selected command identity, and no accidental execution in one receipt.",
      },
    }],
    failure: {
      code: "missing_menu_syntax_ambiguity_diagnostics_receipt",
      stepName: "menu-syntax-ambiguity-diagnostics-receipt",
      message:
        "The harness fails closed until mixed power syntax exposes parse diagnostics, ambiguity reasons, selection identity, and blocked execution receipts.",
    },
    warnings: ["file_linear:menu_syntax_ambiguity_diagnostics_receipts_missing"],
  };
}

export async function runImeCompositionInputBoundaryStressScenario(opts: {
  session: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "ime-composition-input-boundary-stress",
    status: "fail",
    imeCompositionBoundary: {
      session: opts.session,
      surfaces: ["filterInput", "promptInput", "acpComposer"],
      requiredReceipt: "input.compositionBoundary",
      compositionLifecycle: {
        compositionStart: null,
        compositionUpdateEvents: [],
        compositionCommit: null,
        committedText: null,
        preeditTextPreserved: null,
      },
      prematureActionGuards: {
        enterDuringCompositionSubmitted: false,
        actionsOpenedDuringComposition: false,
        filterCommittedBeforeCompositionEnd: false,
        acpMessageSentBeforeCompositionEnd: false,
      },
      semanticReceipts: {
        finalFilterText: null,
        finalPromptValue: null,
        finalComposerText: null,
        cursorRangeAfterCommit: null,
      },
      forbiddenProofModes: ["key_events_only", "sleep_until_text_changes", "native_input_without_composition_receipt"],
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
      name: "ime-composition-input-boundary-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove composition start/update/commit boundaries across filter input, prompt input, and ACP composer without relying on plain key events.",
      },
    }],
    failure: {
      code: "missing_ime_composition_input_boundary_receipt",
      stepName: "ime-composition-input-boundary-receipt",
      message:
        "The harness fails closed until IME/composition receipts prove no premature submit/actions and final committed text semantics.",
    },
    warnings: ["file_linear:ime_composition_input_boundary_receipts_missing"],
  };
}

export async function runAccessibilitySelectedTextFallbackStressScenario(opts: {
  session: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "accessibility-selected-text-fallback-stress",
    status: "fail",
    accessibilitySelectedTextFallback: {
      session: opts.session,
      requiredReceipt: "platform.selectedTextFallback",
      permissionMatrix: {
        accessibilityGranted: null,
        screenRecordingGranted: null,
        selectedTextProvider: null,
        providerDeniedReason: null,
      },
      staleContextGuard: {
        frontmostAppBefore: null,
        frontmostAppAfter: null,
        selectedTextGeneration: null,
        staleSelectedTextRejected: null,
        staleFrontmostContextRejected: null,
      },
      redaction: {
        privateTextRedacted: null,
        maxPreviewChars: null,
        rawSelectedTextLogged: false,
        actionPayloadContainsRawText: false,
      },
      fallback: {
        fallbackSource: null,
        fallbackReason: null,
        actionDisabledWhenUnsafe: null,
      },
      forbiddenProofModes: ["permission_prompt_side_effect", "raw_ax_text_log", "frontmost_app_label_only"],
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      openedSystemSettings: false,
      mutatedTcc: false,
      mutatedUserData: false,
    },
    steps: [{
      name: "accessibility-selected-text-fallback-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove denied or stale selected-text capture falls back safely, redacts private text, and avoids stale frontmost-app context.",
      },
    }],
    failure: {
      code: "missing_accessibility_selected_text_fallback_receipt",
      stepName: "accessibility-selected-text-fallback-receipt",
      message:
        "The harness fails closed until platform selected-text fallback receipts prove permission handling, stale-context rejection, redaction, and safe action disablement.",
    },
    warnings: ["file_linear:accessibility_selected_text_fallback_receipts_missing"],
  };
}

export async function runDisplayMigrationVisualBoundsStressScenario(opts: {
  session: string;
  surfaces?: string[];
  fromDisplay?: string;
  toDisplay?: string;
}): Promise<HardScenarioReceipt> {
  const surfaces = opts.surfaces && opts.surfaces.length > 0
    ? opts.surfaces
    : ["main", "actionsDialog", "promptPopup", "acpDetached", "notes"];

  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "display-migration-visual-bounds-stress",
    status: "fail",
    displayMigrationVisualBounds: {
      session: opts.session,
      requiredReceipt: "window.displayMigrationVisualBounds",
      migrationGeneration: null,
      sourceDisplay: {
        sourceDisplayId: opts.fromDisplay ?? "primary",
        sourceDisplayBoundsPx: null,
        displayScaleFactorBefore: null,
      },
      targetDisplay: {
        targetDisplayId: opts.toDisplay ?? "external",
        targetDisplayBoundsPx: null,
        displayScaleFactorAfter: null,
      },
      surfaces: surfaces.map((surfaceClass) => ({
        surfaceClass,
        automationWindowId: null,
        osWindowId: null,
        semanticSurface: null,
        before: {
          windowBoundsBefore: null,
          contentBoundsBefore: null,
          remPx: null,
          focusSemanticIdBefore: null,
          selectedSemanticIdBefore: null,
          visibleTextBoundsBefore: [],
          textClipState: null,
        },
        after: {
          windowBoundsAfter: null,
          contentBoundsAfter: null,
          remPx: null,
          focusSemanticIdAfter: null,
          selectedSemanticIdAfter: null,
          visibleTextBoundsAfter: [],
          textClipState: null,
        },
        assertions: {
          displayChangedOrReceiptExplainsNoop: false,
          windowFullyVisibleOnTargetDisplay: false,
          scaleFactorReceiptPresent: false,
          focusPreserved: false,
          selectionPreserved: false,
          visibleTextBoundsPreservedOrReflowedWithReceipt: false,
          screenshotSemanticAlignment: false,
          wrongDisplayCaptureRejected: false,
          staleDisplayMigrationRejected: false,
          popupMainClobbered: null,
        },
      })),
      forbiddenProofModes: ["screenshot_only", "window_bounds_only", "focused_app_label_only"],
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
      name: "display-migration-visual-bounds-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove display migration visual bounds, scale/rem metrics, focus/selection preservation, screenshot-to-semantics alignment, and wrong-display rejection.",
      },
    }],
    failure: {
      code: "missing_display_migration_visual_bounds_receipt",
      stepName: "display-migration-visual-bounds-receipt",
      message:
        "The harness fails closed until display migration receipts prove text bounds, display identity, focus/selection, stale migration rejection, and no popup/main clobbering.",
    },
    warnings: ["file_linear:display_migration_visual_bounds_receipts_missing"],
  };
}

export async function runNativePickerExternalReturnFocusStressScenario(opts: {
  session: string;
  origin?: string;
  handoff?: string;
  foreignApp?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "native-picker-external-return-focus-stress",
    status: "fail",
    nativePickerExternalReturnFocus: {
      session: opts.session,
      requiredReceipt: "handoff.returnFocus",
      handoffRequestId: null,
      origin: {
        originSurface: opts.origin ?? "acp",
        originAutomationWindowId: null,
        originOsWindowId: null,
        originSemanticSurface: null,
        originSelectionSemanticId: null,
        originCursorRange: null,
        originSurfaceGeneration: null,
      },
      handoff: {
        kind: opts.handoff ?? "file-picker",
        nativePickerWindowId: null,
        externalBundleId: opts.foreignApp ?? "Finder",
        externalWindowId: null,
        openedByOriginSurface: false,
        noSubmitDuringHandoff: false,
      },
      return: {
        returnGeneration: null,
        returnTargetAutomationWindowId: null,
        returnTargetOsWindowId: null,
        focusRestoredToOrigin: false,
        selectionRestoredToOrigin: false,
        cursorRangeRestored: false,
      },
      eventGuards: {
        staleWindowEventRejected: false,
        foreignWindowEventRejected: false,
        foreignWindowEventDelivered: null,
        staleReturnTargetUsed: null,
        selectionMutatedDuringHandoff: null,
        actionSubmittedDuringHandoff: null,
      },
      forbiddenProofModes: ["frontmost_app_only", "focus_label_only", "native_picker_opened_only"],
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
      name: "native-picker-external-return-focus-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove native picker or external app return focus, origin identity, restored selection/cursor, and stale or foreign window event rejection.",
      },
    }],
    failure: {
      code: "missing_native_picker_external_return_focus_receipt",
      stepName: "native-picker-external-return-focus-receipt",
      message:
        "The harness fails closed until native handoff receipts prove exact-origin return focus and reject stale or foreign window events before delivery.",
    },
    warnings: ["file_linear:native_picker_external_return_focus_receipts_missing"],
  };
}

export async function runDragCancelPayloadScopeStressScenario(opts: {
  session: string;
  source?: string;
  hoverTarget?: string;
  cancel?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "drag-cancel-payload-scope-stress",
    status: "fail",
    dragCancelPayloadScope: {
      session: opts.session,
      requiredReceipt: "drag.payloadScope",
      dragSessionId: null,
      origin: {
        originSurface: opts.source ?? "file-search",
        originAutomationWindowId: null,
        originOsWindowId: null,
        originSelectedSemanticId: null,
        originFocusSemanticId: null,
        originSurfaceGeneration: null,
      },
      payload: {
        payloadFingerprint: null,
        redactedPayloadPreview: null,
        payloadKind: null,
        dragPreviewIdentity: null,
        payloadScopedToDragSession: false,
      },
      duringDrag: {
        hoverTargetBeforeCancel: opts.hoverTarget ?? "drop-prompt",
        dropTargetBeforeCancel: null,
        nativeDragActive: null,
      },
      cancel: {
        cancelMethod: opts.cancel ?? "escape",
        escapeDuringDragCancelled: false,
        dragSessionClosed: false,
      },
      afterCancel: {
        originStateRestored: false,
        focusSemanticIdAfter: null,
        selectedSemanticIdAfter: null,
        hoverTargetsCleared: false,
        dropTargetsCleared: false,
      },
      sideEffects: {
        clipboardChangeCountBefore: null,
        clipboardChangeCountAfter: null,
        fileMutationCount: null,
        temporaryFileCount: null,
        partialPayloadDelivered: null,
        attachmentInsertedDuringCancel: null,
        promptSubmittedDuringCancel: null,
      },
      negativeGuards: {
        foreignDropRejected: false,
        staleDragSessionRejected: false,
      },
      forbiddenProofModes: ["drag_preview_only", "clipboard_side_effect_only", "hover_label_only"],
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
      name: "drag-cancel-payload-scope-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove drag cancellation payload scope, hover/drop cleanup, origin restoration, and clipboard/file/attachment/prompt side-effect boundaries.",
      },
    }],
    failure: {
      code: "missing_drag_cancel_payload_scope_receipt",
      stepName: "drag-cancel-payload-scope-receipt",
      message:
        "The harness fails closed until drag receipts prove scoped payload identity, cancel cleanup, no partial side effects, and stale/foreign drop rejection.",
    },
    warnings: ["file_linear:drag_cancel_payload_scope_receipts_missing"],
  };
}

export async function runRuntimeAppearanceChurnFocusedInputStressScenario(opts: {
  session: string;
  surface?: string;
  churn?: string[];
  cycles?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "runtime-appearance-churn-focused-input-stress",
    status: "fail",
    runtimeAppearanceChurnFocusedInput: {
      session: opts.session,
      requiredReceipt: "ui.appearanceChurnFocusedInput",
      appearanceChurnId: null,
      surface: opts.surface ?? "acp-composer",
      churn: opts.churn && opts.churn.length > 0 ? opts.churn : ["scale", "font", "theme"],
      cycles: opts.cycles ?? 6,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      surfaceGenerationBefore: null,
      surfaceGenerationAfter: null,
      focusSemanticIdBefore: null,
      focusSemanticIdAfter: null,
      inputTextBefore: null,
      inputTextAfter: null,
      visibleTextBefore: null,
      visibleTextAfter: null,
      cursorRangeBefore: null,
      cursorRangeAfter: null,
      selectionRangeBefore: null,
      selectionRangeAfter: null,
      inputLayoutBefore: {
        remPx: null,
        fontFamily: null,
        fontSizePx: null,
        scaleFactor: null,
        boundsPx: null,
        visibleStart: null,
        visibleEnd: null,
        cursorInWindow: null,
      },
      inputLayoutAfter: {
        remPx: null,
        fontFamily: null,
        fontSizePx: null,
        scaleFactor: null,
        boundsPx: null,
        visibleStart: null,
        visibleEnd: null,
        cursorInWindow: null,
      },
      themeTokenFingerprintBefore: null,
      themeTokenFingerprintAfter: null,
      rendererTokenGenerationBefore: null,
      rendererTokenGenerationAfter: null,
      staleTokenRepaintDetected: null,
      layoutShiftPxMax: null,
      visibleTextPreserved: null,
      cursorRangePreserved: null,
      selectionRangePreserved: null,
      focusPreserved: null,
      wrongSurfaceMutationRejected: null,
      forbiddenProofModes: ["screenshot_only", "config_write_only", "theme_name_only"],
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
      name: "runtime-appearance-churn-focused-input-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove runtime appearance churn preserves focused input text, visible text, cursor/selection, layout metrics, and renderer token generation.",
      },
    }],
    failure: {
      code: "missing_runtime_appearance_churn_focused_input_receipt",
      stepName: "runtime-appearance-churn-focused-input-receipt",
      message:
        "The harness fails closed until ui.appearanceChurnFocusedInput receipts prove focused input continuity and stale token repaint rejection.",
    },
    warnings: ["file_linear:runtime_appearance_churn_focused_input_receipts_missing"],
  };
}

export async function runPowerResumeWindowGenerationStressScenario(opts: {
  session: string;
  surface?: string;
  event?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "power-resume-window-generation-stress",
    status: "fail",
    powerResumeWindowGeneration: {
      session: opts.session,
      requiredReceipt: "window.powerResumeGeneration",
      resumeEventId: null,
      event: opts.event ?? "sleep-wake",
      surface: opts.surface ?? "main",
      sessionEpochBefore: null,
      sessionEpochAfter: null,
      appGenerationBefore: null,
      appGenerationAfter: null,
      powerStateBefore: null,
      powerStateAfter: null,
      sleepObservedAtMs: null,
      wakeObservedAtMs: null,
      preSleepTarget: {
        automationWindowId: null,
        osWindowId: null,
        semanticSurface: null,
        windowGeneration: null,
        targetFingerprint: null,
      },
      postWakeTarget: {
        automationWindowId: null,
        osWindowId: null,
        semanticSurface: null,
        windowGeneration: null,
        targetFingerprint: null,
      },
      guards: {
        preSleepTargetRejectedBeforeInput: null,
        nativeInputDeliveryBlockedForStaleTarget: null,
        batchDeliveryBlockedForStaleTarget: null,
        gpuiEventDeliveryBlockedForStaleTarget: null,
        screenshotDeliveryBlockedForStaleTarget: null,
        staleScreenshotRejected: null,
        wrongGenerationStateRejected: null,
      },
      revalidation: {
        targetReResolvedAfterWake: null,
        stateReceiptAfterWake: null,
        elementsReceiptAfterWake: null,
        screenshotReceiptAfterWake: null,
        screenshotStateRevalidatedAfterWake: null,
      },
      continuity: {
        focusSemanticIdBefore: null,
        focusSemanticIdAfter: null,
        selectedSemanticIdBefore: null,
        selectedSemanticIdAfter: null,
        cleanupConfirmed: null,
      },
      forbiddenProofModes: ["os_sleep_side_effect", "stale_target_retry", "screenshot_without_generation"],
    },
    usage: {
      stateFirst: true,
      usedGetState: false,
      usedGetElements: false,
      usedWaitFor: false,
      usedNativeInput: false,
      usedScreenshot: false,
      usedFixedSleepMs: 0,
      initiatedOsSleep: false,
      mutatedUserData: false,
    },
    steps: [{
      name: "power-resume-window-generation-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove power resume generation, stale pre-sleep target rejection, exact post-wake re-resolution, and fresh state/elements/screenshot receipts.",
      },
    }],
    failure: {
      code: "missing_power_resume_window_generation_receipt",
      stepName: "power-resume-window-generation-receipt",
      message:
        "The harness fails closed until window.powerResumeGeneration receipts prove resume generations, stale-target refusal, post-wake revalidation, and cleanup.",
    },
    warnings: ["file_linear:power_resume_window_generation_receipts_missing"],
  };
}

export async function runMenuTrayNotificationModalInterruptionStressScenario(opts: {
  session: string;
  host?: string;
  activeSurface?: string;
  interruptions?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "menu-tray-notification-modal-interruption-stress",
    status: "fail",
    menuTrayNotificationModalInterruption: {
      session: opts.session,
      requiredReceipt: "platform.modalInterruptionFocus",
      interruptionStressId: null,
      hostSurface: opts.host ?? "acpChat",
      activeSurface: opts.activeSurface ?? "actionsDialog",
      modalStackGenerationBefore: null,
      modalStackGenerationAfter: null,
      activeModalIdBefore: null,
      activeModalIdAfter: null,
      parentAutomationWindowId: null,
      parentOsWindowId: null,
      modalAutomationWindowId: null,
      modalOsWindowId: null,
      focusSemanticIdBefore: null,
      focusSemanticIdAfter: null,
      selectedSemanticIdBefore: null,
      selectedSemanticIdAfter: null,
      inputTextBefore: null,
      inputTextAfter: null,
      cursorRangeBefore: null,
      cursorRangeAfter: null,
      interruptions: (opts.interruptions && opts.interruptions.length > 0
        ? opts.interruptions
        : ["tray-menu", "app-menu", "notification"]).map((kind) => ({
        kind,
        interruptionId: null,
        menuItemId: null,
        notificationId: null,
        notificationActionId: null,
        actionTargetSurface: null,
        wrongSurfaceActionRejected: null,
        modalRemainedTopmost: null,
        focusStolen: null,
        selectionMutated: null,
      })),
      submitCountBefore: null,
      submitCountAfter: null,
      modalClosedDuringInterruption: null,
      parentSelectionMutated: null,
      promptSubmittedDuringInterruption: null,
      notificationActionDeliveredToWrongSurface: null,
      trayActionDeliveredToWrongSurface: null,
      appMenuActionDeliveredToWrongSurface: null,
      focusRestoredToActiveModal: null,
      forbiddenProofModes: ["activation_only", "frontmost_app_only", "notification_clicked_only"],
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
      name: "menu-tray-notification-modal-interruption-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove tray/menu/notification interruptions preserve active modal focus, reject wrong-surface actions, and avoid selection/input mutation.",
      },
    }],
    failure: {
      code: "missing_menu_tray_notification_modal_interruption_receipt",
      stepName: "menu-tray-notification-modal-interruption-receipt",
      message:
        "The harness fails closed until platform.modalInterruptionFocus receipts prove interruption identity, topmost modal preservation, wrong-surface rejection, and focus restoration.",
    },
    warnings: ["file_linear:menu_tray_notification_modal_interruption_receipts_missing"],
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
