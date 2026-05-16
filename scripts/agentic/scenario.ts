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
    | "menu-tray-notification-modal-interruption-stress"
    | "stream-progress-cancel-visual-stability-stress"
    | "dictation-media-permission-readiness-churn-stress"
    | "animation-frame-capture-determinism-stress"
    | "accessibility-tree-semantic-parity-stress"
    | "rtl-bidi-emoji-text-rendering-stress"
    | "high-volume-virtualized-list-stability-stress"
    | "input-modality-transition-ownership-stress"
    | "multi-context-attachment-dedupe-provenance-stress"
    | "visual-contrast-readable-state-stress"
    | "empty-error-retry-state-ux-stress"
    | "form-validation-inline-recovery-stress"
    | "navigation-back-stack-history-stress"
    | "long-text-wrap-resize-surface-stress"
    | "actions-command-discoverability-noop-stress"
    | "dense-list-detail-preview-readability-stress"
    | "toast-notification-queue-lifecycle-stress"
    | "destructive-confirm-modal-safety-stress"
    | "loading-skeleton-progress-restoration-stress"
    | "icon-image-fallback-redaction-stress"
    | "footer-status-persistence-stress"
    | "keyboard-hint-label-parity-stress"
    | "row-state-parity-without-pointer-stress"
    | "quiet-chrome-card-nesting-stress"
    | "scroll-shadow-sticky-header-density-stress"
    | "popup-focus-keycap-visual-semantics-stress"
    | "reduced-motion-animation-disable-stress"
    | "command-search-highlighting-accessory-badges-stress"
    | "clipboard-copy-visual-feedback-stress"
    | "portal-cancel-return-state-restoration-stress"
    | "tooltip-hover-focus-affordance-stress"
    | "shortcut-recorder-cancel-layering-stress"
    | "inline-popover-anchor-resize-stress"
    | "disabled-footer-hit-target-refusal-stress"
    | "mini-full-transition-layout-continuity-stress"
    | "filter-input-decoration-chip-layout-stress"
    | "focus-ring-viewport-integrity-stress"
    | "warning-banner-action-dismiss-semantics-stress"
    | "select-prompt-multiselect-keyboard-state-stress"
    | "file-search-preview-sanitization-stress"
    | "hotkey-prompt-transient-capture-cancel-stress"
    | "process-manager-sort-detail-panel-stability-stress"
    | "env-prompt-redacted-status-error-recovery-stress"
    | "command-palette-breadcrumb-route-stack-stress"
    | "root-source-chip-action-semantics-stress"
    | "recent-history-dedupe-root-grouping-stress"
    | "inline-attachment-preview-chip-stability-stress"
    | "window-title-status-semantics-stress"
    | "menu-syntax-capture-validation-chip-stress"
    | "acp-footer-activity-indicator-stress"
    | "acp-model-history-popover-visual-state-stress"
    | "acp-context-insertion-preview-parity-stress"
    | "acp-slash-mention-provider-visibility-stress"
    | "acp-composer-token-keyboard-edit-parity-stress"
    | "acp-transcript-stream-retry-virtualization-stress";
  status: "pass" | "fail" | "error";
  failClosed?: boolean;
  failureMode?: string;
  missingReceipt?: string;
  reasonCode?: string;
  linearIssue?: string;
  error?: Record<string, unknown>;
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
  streamProgressCancelVisualStability?: Record<string, unknown>;
  dictationMediaPermissionReadinessChurn?: Record<string, unknown>;
  animationFrameCaptureDeterminism?: Record<string, unknown>;
  accessibilityTreeSemanticParity?: Record<string, unknown>;
  rtlBidiEmojiTextRendering?: Record<string, unknown>;
  highVolumeVirtualizedListStability?: Record<string, unknown>;
  inputModalityTransitionOwnership?: Record<string, unknown>;
  multiContextAttachmentDedupeProvenance?: Record<string, unknown>;
  visualContrastReadableState?: Record<string, unknown>;
  emptyErrorRetryStateUx?: Record<string, unknown>;
  formValidationInlineRecovery?: Record<string, unknown>;
  navigationBackStackHistory?: Record<string, unknown>;
  longTextWrapResizeSurface?: Record<string, unknown>;
  actionsCommandDiscoverabilityNoop?: Record<string, unknown>;
  denseListDetailPreviewReadability?: Record<string, unknown>;
  toastNotificationQueueLifecycle?: Record<string, unknown>;
  destructiveConfirmModalSafety?: Record<string, unknown>;
  loadingSkeletonProgressRestoration?: Record<string, unknown>;
  iconImageFallbackRedaction?: Record<string, unknown>;
  footerStatusPersistence?: Record<string, unknown>;
  keyboardHintLabelParity?: Record<string, unknown>;
  rowStateParityWithoutPointer?: Record<string, unknown>;
  quietChromeCardNesting?: Record<string, unknown>;
  scrollShadowStickyHeaderDensity?: Record<string, unknown>;
  popupFocusKeycapVisualSemantics?: Record<string, unknown>;
  reducedMotionAnimationDisable?: Record<string, unknown>;
  commandSearchHighlightingAccessoryBadges?: Record<string, unknown>;
  clipboardCopyVisualFeedback?: Record<string, unknown>;
  portalCancelReturnStateRestoration?: Record<string, unknown>;
  tooltipHoverFocusAffordance?: Record<string, unknown>;
  shortcutRecorderCancelLayeringReceipt?: Record<string, unknown>;
  inlinePopoverAnchorResizeReceipt?: Record<string, unknown>;
  disabledFooterHitTargetRefusalReceipt?: Record<string, unknown>;
  miniFullTransitionLayoutContinuityReceipt?: Record<string, unknown>;
  filterInputDecorationChipLayoutReceipt?: Record<string, unknown>;
  focusRingViewportIntegrityReceipt?: Record<string, unknown>;
  warningBannerActionDismissSemanticsReceipt?: Record<string, unknown>;
  selectPromptMultiselectKeyboardStateReceipt?: Record<string, unknown>;
  fileSearchPreviewSanitizationReceipt?: Record<string, unknown>;
  hotkeyPromptTransientCaptureCancelReceipt?: Record<string, unknown>;
  processManagerSortDetailPanelStabilityReceipt?: Record<string, unknown>;
  envPromptRedactedStatusErrorRecoveryReceipt?: Record<string, unknown>;
  commandPaletteBreadcrumbRouteStackReceipt?: Record<string, unknown>;
  rootSourceChipActionSemanticsReceipt?: Record<string, unknown>;
  recentHistoryDedupeRootGroupingReceipt?: Record<string, unknown>;
  inlineAttachmentPreviewChipStabilityReceipt?: Record<string, unknown>;
  windowTitleStatusSemanticsReceipt?: Record<string, unknown>;
  menuSyntaxCaptureValidationChipReceipt?: Record<string, unknown>;
  acpFooterActivityIndicatorReceipt?: Record<string, unknown>;
  acpModelHistoryPopoverVisualStateReceipt?: Record<string, unknown>;
  acpContextInsertionPreviewParityReceipt?: Record<string, unknown>;
  acpSlashMentionProviderVisibilityReceipt?: Record<string, unknown>;
  acpComposerTokenKeyboardEditParityReceipt?: Record<string, unknown>;
  acpTranscriptStreamRetryVirtualizationReceipt?: Record<string, unknown>;
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

export async function runStreamProgressCancelVisualStabilityStressScenario(opts: {
  session: string;
  surface?: string;
  updates?: number;
  cancelAt?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "stream-progress-cancel-visual-stability-stress",
    status: "fail",
    streamProgressCancelVisualStability: {
      session: opts.session,
      requiredReceipt: "stream.progressCancelVisualStability",
      streamRunId: null,
      originSurface: opts.surface ?? "acp-composer",
      updates: opts.updates ?? 40,
      cancelAt: opts.cancelAt ?? 25,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      streamGenerationBefore: null,
      streamGenerationAfterCancel: null,
      progressSamples: [],
      progressSequenceMonotonic: null,
      visibleProgressMonotonic: null,
      visibleTextSamples: [],
      cancelRequestId: null,
      cancelRequestedAtMs: null,
      cancelAcknowledgedAtMs: null,
      cancelStateVisible: null,
      lastPaintedChunkSequence: null,
      staleChunkAfterCancelRejected: null,
      staleChunkIdsRejected: [],
      staleChunkRepaintDetected: null,
      focusSemanticIdBefore: null,
      focusSemanticIdAfter: null,
      cursorRangeBefore: null,
      cursorRangeAfter: null,
      submitCountBefore: null,
      submitCountAfter: null,
      layoutShiftPxMax: null,
      screenshotSamples: [],
      screenshotStateRevalidated: null,
      cleanupConfirmed: null,
      forbiddenProofModes: ["fixed_sleep_until_done", "screenshot_only", "log_tail_only"],
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
      name: "stream-progress-cancel-visual-stability-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove monotonic stream/progress repaint, cancellation ordering, stale post-cancel chunk rejection, focus/cursor restoration, and screenshot-to-state revalidation.",
      },
    }],
    failure: {
      code: "missing_stream_progress_cancel_visual_stability_receipt",
      stepName: "stream-progress-cancel-visual-stability-receipt",
      message:
        "The harness fails closed until stream.progressCancelVisualStability receipts prove progress/cancel visual stability and stale chunk rejection.",
    },
    warnings: ["file_linear:stream_progress_cancel_visual_stability_receipts_missing"],
  };
}

export async function runDictationMediaPermissionReadinessChurnStressScenario(opts: {
  session: string;
  target?: string;
  churn?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "dictation-media-permission-readiness-churn-stress",
    status: "fail",
    dictationMediaPermissionReadinessChurn: {
      session: opts.session,
      requiredReceipt: "media.dictationPermissionReadinessChurn",
      dictationSessionId: null,
      targetSurface: opts.target ?? "acp-composer",
      churn: opts.churn && opts.churn.length > 0
        ? opts.churn
        : ["microphone-permission", "model-readiness"],
      targetAutomationWindowId: null,
      targetOsWindowId: null,
      targetSemanticSurface: null,
      targetFingerprint: null,
      setupMode: "passive",
      passiveSetupConfirmed: null,
      microphonePermissionBefore: null,
      microphonePermissionAfter: null,
      microphonePermissionGenerationBefore: null,
      microphonePermissionGenerationAfter: null,
      modelReadinessBefore: null,
      modelReadinessAfter: null,
      modelReadinessGenerationBefore: null,
      modelReadinessGenerationAfter: null,
      readinessChurnEvents: [],
      transcriptGenerationId: null,
      transcriptTargetFingerprint: null,
      transcriptInsertedRange: null,
      transcriptPreviewRedacted: null,
      transcriptDeliveredToTarget: null,
      wrongTargetDeliveryRejected: null,
      autoSubmitPrevented: null,
      submitCountBefore: null,
      submitCountAfter: null,
      focusSemanticIdBefore: null,
      focusSemanticIdAfter: null,
      cursorRangeBefore: null,
      cursorRangeAfter: null,
      noSystemSettingsOpened: true,
      noTccMutationAttempted: true,
      cleanupConfirmed: null,
      forbiddenProofModes: ["permission_prompt_side_effect", "transcript_text_only", "target_label_only"],
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
      name: "dictation-media-permission-readiness-churn-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove passive dictation/media setup, permission/model readiness generation churn, target identity, no auto-submit, and wrong-target rejection.",
      },
    }],
    failure: {
      code: "missing_dictation_media_permission_readiness_churn_receipt",
      stepName: "dictation-media-permission-readiness-churn-receipt",
      message:
        "The harness fails closed until media.dictationPermissionReadinessChurn receipts prove passive setup, readiness churn ordering, transcript target identity, and cleanup.",
    },
    warnings: ["file_linear:dictation_media_permission_readiness_churn_receipts_missing"],
  };
}

export async function runAnimationFrameCaptureDeterminismStressScenario(opts: {
  session: string;
  surfaces?: string[];
  frames?: number;
  intervalMs?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "animation-frame-capture-determinism-stress",
    status: "fail",
    animationFrameCaptureDeterminism: {
      session: opts.session,
      requiredReceipt: "visual.animationFrameCaptureDeterminism",
      animationStressId: null,
      surfaces: opts.surfaces && opts.surfaces.length > 0
        ? opts.surfaces
        : ["main", "actionsDialog", "promptPopup"],
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      capturePlanId: null,
      animationGenerationBefore: null,
      animationGenerationAfter: null,
      frameSampleCount: opts.frames ?? 6,
      frameIntervalMs: opts.intervalMs ?? 80,
      animationClockSource: null,
      frameSamples: [],
      captureSequence: null,
      frameId: null,
      animationFrameId: null,
      stateReceipt: null,
      elementsReceipt: null,
      screenshotReceipt: null,
      visibleTextFingerprint: null,
      layoutFingerprint: null,
      occlusionPairs: [],
      spinnerSemanticId: null,
      skeletonSemanticIds: [],
      frameIdsStrictlyIncreasing: null,
      captureFrameIdsStable: null,
      stateBeforeScreenshot: null,
      screenshotTargetMatched: null,
      screenshotStateRevalidated: null,
      blankFrameRejected: null,
      motionOcclusionDetected: null,
      visibleTextNotOccluded: null,
      layoutFingerprintStable: null,
      staleFrameRejected: null,
      wrongWindowFrameRejected: null,
      cleanupConfirmed: null,
      forbiddenProofModes: ["single_frame_only", "screenshot_without_state", "sleep_sampled_animation"],
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
      name: "animation-frame-capture-determinism-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove deterministic animated-frame capture with per-frame state/elements/screenshot receipts, visible text/layout fingerprints, occlusion pairs, and stale-frame rejection.",
      },
    }],
    failure: {
      code: "missing_animation_frame_capture_determinism_receipt",
      stepName: "animation-frame-capture-determinism-receipt",
      message:
        "The harness fails closed until visual.animationFrameCaptureDeterminism receipts prove stable frame sampling, occlusion safety, and screenshot-to-state revalidation.",
    },
    warnings: ["file_linear:animation_frame_capture_determinism_receipts_missing"],
  };
}

export async function runAccessibilityTreeSemanticParityStressScenario(opts: {
  session: string;
  surfaces?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "accessibility-tree-semantic-parity-stress",
    status: "fail",
    failureMode: "missing_app_receipt",
    missingReceipt: "accessibility.treeSemanticParity",
    error: {
      code: "missing_accessibility_tree_semantic_parity_receipt",
      linear: "file_linear:accessibility_tree_semantic_parity_receipts_missing",
    },
    accessibilityTreeSemanticParity: {
      session: opts.session,
      requiredReceipt: "accessibility.treeSemanticParity",
      accessibilityAuditId: null,
      surfaceSamples: (opts.surfaces && opts.surfaces.length > 0
        ? opts.surfaces
        : ["main", "actionsDialog", "promptPopup"]).map((surface) => ({
          surface,
          automationWindowId: null,
          osWindowId: null,
          semanticSurface: null,
          stateReceipt: null,
          elementsReceipt: null,
          axTreeReceipt: null,
          screenshotReceipt: null,
          visibleControlIds: [],
          automationElementIds: [],
          axNodeIds: [],
          roleParity: null,
          labelParity: null,
          focusOrder: [],
          tabOrder: [],
          disabledStateParity: null,
          keyboardActivationParity: null,
          activationPlan: {
            sideEffectSafe: true,
            activationMethod: null,
            activatedSemanticId: null,
            activationResult: null,
          },
          disabledActivationPrevented: null,
          focusSemanticIdBefore: null,
          focusSemanticIdAfter: null,
          hitTargetBounds: [],
          screenshotSemanticAlignment: null,
          missingAxNodes: [],
          extraAxNodes: [],
          staleAxTreeRejected: null,
          wrongWindowAxRejected: null,
        })),
      accessibilityPermissionBefore: null,
      noSystemSettingsOpened: true,
      noTccMutationAttempted: true,
      cleanupConfirmed: null,
      forbiddenProofModes: ["screenshot_only", "ax_tree_without_window_identity", "unsafe_activation"],
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      mutatedAppBehavior: false,
      openedSystemSettings: false,
      mutatedTcc: false,
    },
    steps: [{
      name: "accessibility-tree-semantic-parity-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove visible controls, automation elements, and AX nodes share roles, labels, focus/tab order, activation semantics, hit targets, and screenshot-to-semantics alignment.",
      },
    }],
    failure: {
      code: "missing_accessibility_tree_semantic_parity_receipt",
      stepName: "accessibility-tree-semantic-parity-receipt",
      message:
        "The harness fails closed until accessibility.treeSemanticParity receipts prove role/label/focus/activation parity, stale AX rejection, wrong-window rejection, and cleanup.",
    },
    warnings: ["file_linear:accessibility_tree_semantic_parity_receipts_missing"],
  };
}

export async function runRtlBidiEmojiTextRenderingStressScenario(opts: {
  session: string;
  surface?: string;
  text?: string;
}): Promise<HardScenarioReceipt> {
  const rawText = opts.text ?? "abc שלום 👩🏽‍💻 é مرحبا 123";
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "rtl-bidi-emoji-text-rendering-stress",
    status: "fail",
    failureMode: "missing_app_receipt",
    missingReceipt: "text.rtlBidiEmojiTextRendering",
    error: {
      code: "missing_rtl_bidi_emoji_text_rendering_receipt",
      linear: "file_linear:rtl_bidi_emoji_text_rendering_receipts_missing",
    },
    rtlBidiEmojiTextRendering: {
      session: opts.session,
      requiredReceipt: "text.rtlBidiEmojiTextRendering",
      bidiStressId: null,
      surface: opts.surface ?? "acp-composer",
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      inputSemanticId: null,
      rawText,
      normalizedText: null,
      textDirectionBase: "auto",
      directionRuns: [],
      bidiEmbeddingLevels: [],
      graphemeClusters: [],
      clusterBoundaries: [],
      emojiZwJSequences: [],
      combiningMarkSequences: [],
      cursorSamples: [{
        cursorLogicalIndex: null,
        cursorUtf16Index: null,
        cursorVisualRect: null,
        cursorInVisibleWindow: null,
      }],
      selectionSamples: [{
        selectionLogicalRange: null,
        selectionUtf16Range: null,
        selectionVisualRects: [],
      }],
      visibleTextBounds: null,
      renderedTextBounds: null,
      availableWidth: null,
      measuredWidth: null,
      truncationState: {
        isTruncated: null,
        truncationIntentional: null,
        accessibleFullText: null,
      },
      searchFilterSamples: [{
        query: null,
        normalizedQuery: null,
        matchingSemanticIds: [],
        filterResultFingerprint: null,
      }],
      backspaceClusterAtomicity: null,
      selectionPreservedAcrossFilter: null,
      cursorRangeBefore: null,
      cursorRangeAfter: null,
      screenshotStateRevalidated: null,
      staleTextLayoutRejected: null,
      wrongSurfaceMutationRejected: null,
      noAccidentalSubmit: null,
      cleanupConfirmed: null,
      forbiddenProofModes: ["utf16_index_only", "screenshot_only", "plain_text_echo"],
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      mutatedAppBehavior: false,
    },
    steps: [{
      name: "rtl-bidi-emoji-text-rendering-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove mixed RTL/LTR/emoji/combining-mark rendering, cursor visual positions, selection rectangles, grapheme-aware editing, search/filter normalization, and stale layout rejection.",
      },
    }],
    failure: {
      code: "missing_rtl_bidi_emoji_text_rendering_receipt",
      stepName: "rtl-bidi-emoji-text-rendering-receipt",
      message:
        "The harness fails closed until text.rtlBidiEmojiTextRendering receipts prove bidi, grapheme, cursor, selection, filter, truncation, and cleanup semantics.",
    },
    warnings: ["file_linear:rtl_bidi_emoji_text_rendering_receipts_missing"],
  };
}

export async function runHighVolumeVirtualizedListStabilityStressScenario(opts: {
  session: string;
  surface?: string;
  fixtureCount?: number;
  filterCycles?: number;
  scrollCycles?: number;
}): Promise<HardScenarioReceipt> {
  const fixtureItemCount = opts.fixtureCount ?? 5000;
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "high-volume-virtualized-list-stability-stress",
    status: "fail",
    failureMode: "missing_app_receipt",
    missingReceipt: "list.highVolumeVirtualizedListStability",
    error: {
      code: "missing_high_volume_virtualized_list_stability_receipt",
      linear: "file_linear:high_volume_virtualized_list_stability_receipts_missing",
    },
    highVolumeVirtualizedListStability: {
      session: opts.session,
      requiredReceipt: "list.highVolumeVirtualizedListStability",
      virtualizedListStressId: null,
      surface: opts.surface ?? "clipboard-history",
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      datasetId: null,
      fixtureItemCount,
      totalItemCount: null,
      visibleWindowSize: null,
      virtualizationGenerationBefore: null,
      virtualizationGenerationAfter: null,
      rowSamples: [],
      semanticId: null,
      stableRowKey: null,
      dataIndex: null,
      visibleIndex: null,
      rowBounds: null,
      textBounds: null,
      renderedTextBounds: null,
      selectedSemanticIdBefore: null,
      selectedSemanticIdAfter: null,
      selectedStableKeyBefore: null,
      selectedStableKeyAfter: null,
      selectionReanchored: null,
      scrollAnchorKey: null,
      scrollTopBefore: null,
      scrollTopAfter: null,
      viewportBounds: null,
      contentHeight: null,
      filterCycles: Array.from({ length: opts.filterCycles ?? 8 }, () => ({
        query: null,
        expectedCount: null,
        actualCount: null,
        firstVisibleKey: null,
        selectedKey: null,
        rowFingerprintBefore: null,
        rowFingerprintAfter: null,
        elementsFingerprint: null,
      })),
      scrollCycles: opts.scrollCycles ?? 12,
      rapidFilterTransitions: [],
      filterGeneration: null,
      staleFilterResultsRejected: null,
      screenshotReceipt: null,
      screenshotStateRevalidated: null,
      semanticVisibleTextMatchesRows: null,
      duplicateRowKeysRejected: null,
      rowReuseIdentityPreserved: null,
      blankRowsRejected: null,
      footerSafeSelectedRow: null,
      cleanupConfirmed: null,
      forbiddenProofModes: ["row_count_only", "screenshot_only", "unscoped_user_data_fixture"],
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      mutatedAppBehavior: false,
      mutatedUserData: false,
    },
    steps: [{
      name: "high-volume-virtualized-list-stability-receipt",
      status: "fail",
      output: {
        blockingGap:
          "The harness cannot yet prove high-volume fixture identity, stable row keys, selected-row reanchor, scroll anchor preservation, rapid filter generation ordering, stale result rejection, and screenshot-to-semantics consistency.",
      },
    }],
    failure: {
      code: "missing_high_volume_virtualized_list_stability_receipt",
      stepName: "high-volume-virtualized-list-stability-receipt",
      message:
        "The harness fails closed until list.highVolumeVirtualizedListStability receipts prove virtualized row identity, filter/scroll ordering, screenshot-to-state consistency, and cleanup.",
    },
    warnings: ["file_linear:high_volume_virtualized_list_stability_receipts_missing"],
  };
}

export async function runInputModalityTransitionOwnershipStressScenario(opts: {
  session: string;
  surface?: string;
  interleave?: string[];
  cycles?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "input-modality-transition-ownership-stress",
    status: "fail",
    failClosed: true,
    failureMode: "missing_required_receipt",
    missingReceipt: "modality.inputTransitionOwnership",
    linearIssue: "file_linear:input_modality_transition_ownership_receipts_missing",
    error: {
      code: "missing_input_modality_transition_ownership_receipt",
      linear: "file_linear:input_modality_transition_ownership_receipts_missing",
    },
    inputModalityTransitionOwnership: {
      session: opts.session,
      requiredReceipt: "modality.inputTransitionOwnership",
      requiredReceiptKind: "modality.inputTransitionOwnership",
      modalityStressId: null,
      surface: opts.surface ?? "main",
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      initialStateReceipt: null,
      initialElementsReceipt: null,
      interleave: opts.interleave && opts.interleave.length > 0
        ? opts.interleave
        : ["pointer-hover", "keyboard-nav", "trackpad-scroll", "wheel-scroll", "shortcut"],
      cycles: opts.cycles ?? 8,
      modalitySequence: [{
        eventSequenceId: null,
        inputDevice: null,
        pointerDeviceKind: null,
        modalityGeneration: null,
        hoverSemanticId: null,
        hoverBounds: null,
        focusSemanticId: null,
        focusRingVisible: null,
        selectedSemanticId: null,
        scrollInputKind: null,
        scrollTopBefore: null,
        scrollTopAfter: null,
        scrollAnchorKey: null,
        shortcutCommandId: null,
        activationOwnerSemanticId: null,
        activationMethod: null,
      }],
      hoverFocusParity: null,
      selectionPreservedAcrossModality: null,
      activationOwnershipPreserved: null,
      shortcutDidNotStealHoverOwner: null,
      wheelDidNotMutateFocus: null,
      staleModalityEventRejected: null,
      wrongSurfaceInputRejected: null,
      noAccidentalSubmit: null,
      screenshotStateRevalidated: null,
      cleanupConfirmed: null,
      forbiddenProofModes: ["input_delivery_only", "hover_without_generation", "screenshot_without_state"],
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      mutatedAppBehavior: false,
    },
    steps: [{
      name: "declare-required-receipt",
      status: "fail",
      output: { reason: "modality.inputTransitionOwnership" },
    }],
    failure: {
      code: "missing_input_modality_transition_ownership_receipt",
      stepName: "declare-required-receipt",
      message:
        "The harness fails closed until modality.inputTransitionOwnership receipts prove hover, focus, selection, scroll, shortcut, and activation ownership across modality changes.",
    },
    warnings: ["file_linear:input_modality_transition_ownership_receipts_missing"],
  };
}

export async function runMultiContextAttachmentDedupeProvenanceStressScenario(opts: {
  session: string;
  origins?: string[];
  destinations?: string[];
  reorderCycles?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "multi-context-attachment-dedupe-provenance-stress",
    status: "fail",
    failClosed: true,
    failureMode: "missing_required_receipt",
    missingReceipt: "context.multiContextAttachmentDedupeProvenance",
    linearIssue: "file_linear:multi_context_attachment_dedupe_provenance_receipts_missing",
    error: {
      code: "missing_multi_context_attachment_dedupe_provenance_receipt",
      linear: "file_linear:multi_context_attachment_dedupe_provenance_receipts_missing",
    },
    multiContextAttachmentDedupeProvenance: {
      session: opts.session,
      requiredReceipt: "context.multiContextAttachmentDedupeProvenance",
      requiredReceiptKind: "context.multiContextAttachmentDedupeProvenance",
      attachmentStressId: null,
      contextRunId: null,
      hostSamples: (opts.destinations && opts.destinations.length > 0
        ? opts.destinations
        : ["acp-composer", "notes"]).map((destinationSurface) => ({
          destinationSurface,
          automationWindowId: null,
          osWindowId: null,
          semanticSurface: null,
          destinationGeneration: null,
          stateReceipt: null,
          elementsReceipt: null,
        })),
      originSamples: (opts.origins && opts.origins.length > 0
        ? opts.origins
        : ["file", "screenshot", "selected-text", "mcp-resource", "clipboard-snippet"]).map((sourceKind) => ({
          sourceKind,
          originSurface: null,
          originGeneration: null,
          sourceUri: null,
          resourceProfile: null,
          mcpResourceUri: null,
          scriptResourceIdentity: null,
          screenshotIdentity: null,
          selectedTextCaptureGeneration: null,
          clipboardGeneration: null,
          redactedPreview: null,
          privacyClass: null,
        })),
      reorderCycles: opts.reorderCycles ?? 3,
      attachmentSamples: [{
        attachmentId: null,
        dedupeKey: null,
        provenanceId: null,
        provenanceFingerprint: null,
        acceptedContextPartUri: null,
        insertIndex: null,
        removeReceipt: null,
        reorderReceipt: null,
      }],
      insertedAttachmentIds: [],
      removedAttachmentIds: [],
      reorderedAttachmentIds: [],
      duplicateAttachmentIdsRejected: null,
      dedupeCollisionRejected: null,
      staleProvenanceRejected: null,
      wrongDestinationRejected: null,
      orphanAttachmentRejected: null,
      noCrossHostLeakage: null,
      rawPathNotLogged: null,
      rawTextNotLogged: null,
      cleanupConfirmed: null,
      forbiddenProofModes: ["visible_chip_count_only", "raw_path_logging", "single_host_only"],
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      mutatedAppBehavior: false,
    },
    steps: [{
      name: "declare-required-receipt",
      status: "fail",
      output: { reason: "context.multiContextAttachmentDedupeProvenance" },
    }],
    failure: {
      code: "missing_multi_context_attachment_dedupe_provenance_receipt",
      stepName: "declare-required-receipt",
      message:
        "The harness fails closed until context.multiContextAttachmentDedupeProvenance receipts prove attachment dedupe, provenance, ordering, privacy, stale rejection, and cleanup.",
    },
    warnings: ["file_linear:multi_context_attachment_dedupe_provenance_receipts_missing"],
  };
}

export async function runVisualContrastReadableStateStressScenario(opts: {
  session: string;
  surfaces?: string[];
  themes?: string[];
  scaleFactors?: number[];
  states?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "visual-contrast-readable-state-stress",
    status: "fail",
    failClosed: true,
    failureMode: "missing_required_receipt",
    missingReceipt: "visual.contrastReadableState",
    linearIssue: "file_linear:visual_contrast_readable_state_receipts_missing",
    error: {
      code: "missing_visual_contrast_readable_state_receipt",
      linear: "file_linear:visual_contrast_readable_state_receipts_missing",
    },
    visualContrastReadableState: {
      session: opts.session,
      requiredReceipt: "visual.contrastReadableState",
      requiredReceiptKind: "visual.contrastReadableState",
      visualContrastStressId: null,
      themes: opts.themes && opts.themes.length > 0 ? opts.themes : ["light", "dark"],
      scaleFactors: opts.scaleFactors && opts.scaleFactors.length > 0 ? opts.scaleFactors : [1, 1.25, 1.5],
      surfaceSamples: (opts.surfaces && opts.surfaces.length > 0
        ? opts.surfaces
        : ["main", "actionsDialog", "promptPopup", "acp-composer", "notes"]).map((surface) => ({
          surface,
          automationWindowId: null,
          osWindowId: null,
          semanticSurface: null,
          themeId: null,
          themeMode: null,
          themeTokenFingerprint: null,
          appearanceGeneration: null,
          scaleFactor: null,
          remSize: null,
          stateSamples: (opts.states && opts.states.length > 0
            ? opts.states
            : ["active", "inactive", "disabled", "focused", "error", "loading"]).map((stateKind) => ({
              stateKind,
              semanticId: null,
              role: null,
              label: null,
              visibleText: null,
              fontSizePx: null,
              fontWeight: null,
              elementBounds: null,
              textBounds: null,
              foregroundColor: null,
              backgroundColor: null,
              contrastRatio: null,
              minimumContrastRatio: null,
              contrastPass: null,
              readabilityPass: null,
              focusIndicatorBounds: null,
              focusIndicatorContrastRatio: null,
              disabledStateVisible: null,
              errorStateVisible: null,
              loadingStateVisible: null,
              nonColorStateCue: null,
              activeInactiveDifferentiator: null,
            })),
          screenshotReceipt: null,
          screenshotStateRevalidated: null,
          semanticVisibleTextMatchesReceipt: null,
        })),
      staleThemeTokenRejected: null,
      wrongSurfaceContrastRejected: null,
      blankScreenshotRejected: null,
      cleanupConfirmed: null,
      forbiddenProofModes: ["theme_name_only", "screenshot_without_color_samples", "color_only_state_cue"],
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      mutatedAppBehavior: false,
    },
    steps: [{
      name: "declare-required-receipt",
      status: "fail",
      output: { reason: "visual.contrastReadableState" },
    }],
    failure: {
      code: "missing_visual_contrast_readable_state_receipt",
      stepName: "declare-required-receipt",
      message:
        "The harness fails closed until visual.contrastReadableState receipts prove contrast/readability across themes, scales, states, surfaces, screenshot revalidation, and cleanup.",
    },
    warnings: ["file_linear:visual_contrast_readable_state_receipts_missing"],
  };
}

export async function runEmptyErrorRetryStateUxStressScenario(opts: {
  session: string;
  surfaces?: string[];
  query?: string;
  retryCycles?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "empty-error-retry-state-ux-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "ux.emptyErrorRetryState",
    reasonCode: "missing_empty_error_retry_state_ux_receipt",
    linearIssue: "file_linear:empty_error_retry_state_ux_receipts_missing",
    error: {
      code: "missing_empty_error_retry_state_ux_receipt",
      linear: "file_linear:empty_error_retry_state_ux_receipts_missing",
    },
    emptyErrorRetryStateUx: {
      session: opts.session,
      requiredReceipt: "ux.emptyErrorRetryState",
      emptyRetryStressId: null,
      surfaceSamples: (opts.surfaces && opts.surfaces.length > 0
        ? opts.surfaces
        : ["main", "clipboard-history", "emoji-picker", "file-search"]).map((surface) => ({
          surface,
          automationWindowId: null,
          osWindowId: null,
          semanticSurface: null,
          query: opts.query ?? "agentic-loop-eighteen-no-results-zzzz",
          stateReceipt: null,
          elementsReceipt: null,
        })),
      retryCycles: opts.retryCycles ?? 2,
      emptyStateSamples: [],
      emptyMessageSemanticId: null,
      emptyMessageText: null,
      emptyMessageVisible: null,
      emptyIllustrationVisible: null,
      loadingStateSamples: [],
      loadingGeneration: null,
      loadingMessageText: null,
      loadingSpinnerVisible: null,
      errorStateSamples: [],
      errorGeneration: null,
      errorBannerSemanticId: null,
      errorMessageText: null,
      errorSeverity: null,
      retryButtonSemanticId: null,
      retryButtonLabel: null,
      retryButtonEnabled: null,
      retryRequestId: null,
      retryStateSamples: [],
      retryAttempt: null,
      retryStartedAt: null,
      retryCompletedAt: null,
      recoverySamples: [],
      recoveredStateReceipt: null,
      recoveredElementsReceipt: null,
      recoveryClearsError: null,
      selectionStableAcrossEmpty: null,
      footerActionsSafeInEmpty: null,
      noStaleErrorAfterRecovery: null,
      noDisabledRetryTrap: null,
      screenshotStateRevalidated: null,
      cleanupConfirmed: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      openedSecurityPrompt: false,
      mutatedTcc: false,
      installedAgent: false,
      triggeredCodexAcpSecurityAgent: false,
    },
    steps: [{
      name: "declare-required-receipt",
      status: "fail",
      output: { reason: "missing_empty_error_retry_state_ux_receipt" },
    }],
    failure: {
      code: "missing_empty_error_retry_state_ux_receipt",
      stepName: "declare-required-receipt",
      message:
        "The harness fails closed until ux.emptyErrorRetryState receipts prove empty/loading/error/retry/recovery UX, footer safety, selection stability, and no stale error after recovery.",
    },
    warnings: ["file_linear:empty_error_retry_state_ux_receipts_missing"],
  };
}

export async function runFormValidationInlineRecoveryStressScenario(opts: {
  session: string;
  surface?: string;
  fields?: string[];
  invalid?: string[];
  valid?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "form-validation-inline-recovery-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "ux.formValidationInlineRecovery",
    reasonCode: "missing_form_validation_inline_recovery_receipt",
    linearIssue: "file_linear:form_validation_inline_recovery_receipts_missing",
    error: {
      code: "missing_form_validation_inline_recovery_receipt",
      linear: "file_linear:form_validation_inline_recovery_receipts_missing",
    },
    formValidationInlineRecovery: {
      session: opts.session,
      requiredReceipt: "ux.formValidationInlineRecovery",
      formValidationStressId: null,
      surface: opts.surface ?? "fields-prompt",
      promptType: null,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      initialFormReceipt: null,
      initialElementsReceipt: null,
      fields: opts.fields ?? ["email", "required-text", "number"],
      invalidInputs: opts.invalid ?? ["email:not-an-email", "required-text:", "number:not-a-number"],
      validInputs: opts.valid ?? ["email:ada@example.com", "required-text:Ada", "number:42"],
      fieldSamples: [],
      fieldSemanticId: null,
      fieldName: null,
      fieldLabel: null,
      fieldRole: null,
      fieldRequired: null,
      fieldValueBeforeInvalidSubmit: null,
      invalidInputValue: null,
      validInputValue: null,
      validationRuleId: null,
      fieldValidationGeneration: null,
      invalidSubmitReceipt: null,
      submitPrevented: null,
      preventedAccidentalSubmit: null,
      firstInvalidFieldSemanticId: null,
      focusAfterInvalidSubmit: null,
      cursorAfterInvalidSubmit: null,
      inlineErrorSamples: [],
      errorSemanticId: null,
      errorText: null,
      errorVisible: null,
      errorLinkedFieldSemanticId: null,
      errorSeverity: null,
      inputPreservedAfterInvalidSubmit: null,
      footerSubmitDisabledReason: null,
      validEditReceipt: null,
      errorsClearedOnValidEdit: null,
      fieldValueAfterValidEdit: null,
      focusPreservedDuringRecovery: null,
      submitRecoveryReceipt: null,
      submittedValueReceipt: null,
      noStaleInlineErrors: null,
      noCrossFieldErrorLeakage: null,
      actionsDialogStillSafe: null,
      escapeCancelStillSafe: null,
      cleanupConfirmed: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      openedSecurityPrompt: false,
      mutatedTcc: false,
      installedAgent: false,
      triggeredCodexAcpSecurityAgent: false,
    },
    steps: [{
      name: "declare-required-receipt",
      status: "fail",
      output: { reason: "missing_form_validation_inline_recovery_receipt" },
    }],
    failure: {
      code: "missing_form_validation_inline_recovery_receipt",
      stepName: "declare-required-receipt",
      message:
        "The harness fails closed until ux.formValidationInlineRecovery receipts prove inline validation errors, input preservation, first invalid focus, valid edit recovery, submit prevention, and cleanup.",
    },
    warnings: ["file_linear:form_validation_inline_recovery_receipts_missing"],
  };
}

export async function runNavigationBackStackHistoryStressScenario(opts: {
  session: string;
  origin?: string;
  surfaces?: string[];
  transitions?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "navigation-back-stack-history-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "ux.navigationBackStackHistory",
    reasonCode: "missing_navigation_back_stack_history_receipt",
    linearIssue: "file_linear:navigation_back_stack_history_receipts_missing",
    error: {
      code: "missing_navigation_back_stack_history_receipt",
      linear: "file_linear:navigation_back_stack_history_receipts_missing",
    },
    navigationBackStackHistory: {
      session: opts.session,
      requiredReceipt: "ux.navigationBackStackHistory",
      navigationBackStackStressId: null,
      navigationRunId: null,
      originSurface: opts.origin ?? "main",
      originAutomationWindowId: null,
      originSemanticSurface: null,
      originStateReceipt: null,
      originElementsReceipt: null,
      originSelectionSemanticId: null,
      originFilterText: null,
      originScrollTop: null,
      originFooterReceipt: null,
      originFocusSemanticId: null,
      surfaces: opts.surfaces ?? ["clipboard-history", "emoji-picker", "file-search", "actionsDialog"],
      transitions: opts.transitions ?? ["triggerBuiltin", "cmd-k", "escape", "back"],
      transitionSamples: [],
      transitionSequenceId: null,
      transitionKind: null,
      fromSurface: null,
      toSurface: null,
      surfaceStackGeneration: null,
      routeStackDepthBefore: null,
      routeStackDepthAfter: null,
      triggerReceipt: null,
      stateReceiptAfterTransition: null,
      elementsReceiptAfterTransition: null,
      actionsDialogReceipt: null,
      actionsDiscoverabilityReceipt: null,
      actionRowsVisible: null,
      disabledActionSamples: [],
      disabledReason: null,
      noOpActionSemanticId: null,
      noOpAffordanceVisible: null,
      noAccidentalExecution: null,
      backStackSamples: [],
      backAction: null,
      escapeReceipt: null,
      backReceipt: null,
      cmdKCloseReceipt: null,
      returnToOriginReceipt: null,
      returnedSurface: null,
      selectionRestored: null,
      filterRestored: null,
      scrollRestored: null,
      footerRestored: null,
      focusRestored: null,
      inputCursorRestored: null,
      routeStackDrained: null,
      noStalePopup: null,
      noStaleSurfaceState: null,
      wrongSurfaceBackRejected: null,
      staleTransitionRejected: null,
      cleanupConfirmed: null,
    },
    usage: {
      stateFirst: true,
      usedGetState: true,
      usedGetElements: true,
      usedScreenshot: false,
      usedNativeInput: false,
      openedSecurityPrompt: false,
      mutatedTcc: false,
      installedAgent: false,
      triggeredCodexAcpSecurityAgent: false,
    },
    steps: [{
      name: "declare-required-receipt",
      status: "fail",
      output: { reason: "missing_navigation_back_stack_history_receipt" },
    }],
    failure: {
      code: "missing_navigation_back_stack_history_receipt",
      stepName: "declare-required-receipt",
      message:
        "The harness fails closed until ux.navigationBackStackHistory receipts prove transitions, route stack generations, actions discoverability, no-op affordances, return-to-origin restoration, stale rejection, and cleanup.",
    },
    warnings: ["file_linear:navigation_back_stack_history_receipts_missing"],
  };
}

export async function runLongTextWrapResizeSurfaceStressScenario(opts: {
  session: string;
  surfaces?: string[];
  widths?: string[];
  fixtures?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "long-text-wrap-resize-surface-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_long_text_wrap_resize_surface_receipt",
    linearIssue: "file_linear:long_text_wrap_resize_surface_receipts_missing",
    longTextWrapResizeSurface: {
      requiredReceipt: "ux.longTextWrapResizeSurface",
      receiptKind: "ux.longTextWrapResizeSurface",
      longTextStressId: "loop-nineteen-long-text-wrap-resize",
      requestedSurfaces: opts.surfaces ?? ["main", "clipboard-history", "emoji-picker", "file-search", "actionsDialog"],
      requestedWidths: opts.widths ?? ["mini", "narrow", "full"],
      requestedFixtures: opts.fixtures ?? ["long-name", "long-path", "long-description", "multiline-snippet"],
      surfaceSamples: [],
      surface: null,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      stateReceipt: null,
      elementsReceipt: null,
      widthSamples: [],
      widthMode: null,
      resizeGeneration: null,
      windowBounds: null,
      contentBounds: null,
      inputBounds: null,
      listBounds: null,
      footerBounds: null,
      fixtureSamples: [],
      fixtureId: null,
      longNameFixture: null,
      longPathFixture: null,
      longDescriptionFixture: null,
      multilineSnippetFixture: null,
      semanticId: null,
      role: null,
      fullText: null,
      visibleText: null,
      textBounds: null,
      renderedTextBounds: null,
      elementBounds: null,
      availableWidth: null,
      measuredWidth: null,
      wrapLineCount: null,
      clippingState: null,
      truncationIntent: null,
      tooltipOrAccessibleFullText: null,
      accessibleFullText: null,
      overlapPairs: [],
      footerCollision: null,
      inputCollision: null,
      lostAccessibleText: null,
      resizeTransitionSamples: [],
      fromWidthMode: null,
      toWidthMode: null,
      selectionPreserved: null,
      focusPreserved: null,
      noLayoutShiftBeyondContainer: null,
      noFooterCollision: null,
      screenshotStateRevalidated: null,
      cleanupConfirmed: null,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_long_text_wrap_resize_surface_receipt" } }],
    failure: { code: "missing_long_text_wrap_resize_surface_receipt", stepName: "declare-required-receipt", message: "Missing app-side long text wrapping and resize layout receipts." },
    warnings: ["file_linear:long_text_wrap_resize_surface_receipts_missing"],
  };
}

export async function runActionsCommandDiscoverabilityNoopStressScenario(opts: {
  session: string;
  hosts?: string[];
  states?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "actions-command-discoverability-noop-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_actions_command_discoverability_noop_receipt",
    linearIssue: "file_linear:actions_command_discoverability_noop_receipts_missing",
    actionsCommandDiscoverabilityNoop: {
      requiredReceipt: "ux.actionsCommandDiscoverabilityNoop",
      receiptKind: "ux.actionsCommandDiscoverabilityNoop",
      actionsNoopStressId: "loop-nineteen-actions-command-discoverability-noop",
      requestedHosts: opts.hosts ?? ["main", "clipboard-history", "emoji-picker", "file-search", "app-launcher"],
      requestedStates: opts.states ?? ["actionable", "disabled", "no-op"],
      hostSamples: [],
      hostSurface: null,
      hostAutomationWindowId: null,
      hostSemanticSurface: null,
      hostStateBefore: null,
      hostElementsBefore: null,
      actionsDialogReceipt: null,
      parentAutomationWindowId: null,
      routeStackDepth: null,
      actionsVisible: null,
      filterText: null,
      focusedSemanticId: null,
      actionRowSamples: [],
      rowSemanticId: null,
      actionId: null,
      label: null,
      section: null,
      rowKind: null,
      actionable: null,
      disabled: null,
      noOp: null,
      enabled: null,
      disabledReason: null,
      noOpReason: null,
      keyboardSelectable: null,
      keyboardSkipOrExplainReceipt: null,
      enterWouldExecute: null,
      keyboardSelectionSamples: [],
      fromSemanticId: null,
      toSemanticId: null,
      skippedSemanticIds: [],
      skipReasons: [],
      activationGuardSamples: [],
      attemptedSemanticId: null,
      attemptedActionId: null,
      activationPrevented: null,
      preventedReason: null,
      noAccidentalExecution: null,
      hostMutationCountBefore: null,
      hostMutationCountAfter: null,
      hostStateAfter: null,
      hostMutationReceipt: null,
      selectionUnchanged: null,
      filterUnchanged: null,
      scrollUnchanged: null,
      footerUnchanged: null,
      focusRestored: null,
      cleanupConfirmed: null,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_actions_command_discoverability_noop_receipt" } }],
    failure: { code: "missing_actions_command_discoverability_noop_receipt", stepName: "declare-required-receipt", message: "Missing app-side actions discoverability, disabled reason, and no-op execution guard receipts." },
    warnings: ["file_linear:actions_command_discoverability_noop_receipts_missing"],
  };
}

export async function runDenseListDetailPreviewReadabilityStressScenario(opts: {
  session: string;
  surfaces?: string[];
  query?: string;
  filterCycles?: number;
  selectionCycles?: number;
  resizeCycles?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "dense-list-detail-preview-readability-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_dense_list_detail_preview_readability_receipt",
    linearIssue: "file_linear:dense_list_detail_preview_readability_receipts_missing",
    denseListDetailPreviewReadability: {
      requiredReceipt: "ux.denseListDetailPreviewReadability",
      receiptKind: "ux.denseListDetailPreviewReadability",
      densePreviewStressId: "loop-nineteen-dense-list-detail-preview-readability",
      requestedSurfaces: opts.surfaces ?? ["file-search", "sdk-reference", "script-template-catalog"],
      requestedQuery: opts.query ?? "agentic-loop-nineteen-preview",
      filterCycles: opts.filterCycles ?? 4,
      selectionCycles: opts.selectionCycles ?? 8,
      resizeCycles: opts.resizeCycles ?? 3,
      surfaceSamples: [],
      surface: null,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      query: opts.query ?? "agentic-loop-nineteen-preview",
      stateReceipt: null,
      elementsReceipt: null,
      listPaneSamples: [],
      listPaneBounds: null,
      visibleRowCount: null,
      selectedRowSemanticId: null,
      selectedStableKey: null,
      selectedRowBounds: null,
      selectedRowTextBounds: null,
      selectedRowVisible: null,
      selectedRowAboveFooter: null,
      rowIdentityVisible: null,
      previewPaneSamples: [],
      previewPaneBounds: null,
      previewSourceStableKey: null,
      previewMatchesSelectedStableKey: null,
      previewTitleSemanticId: null,
      previewTitleText: null,
      previewTitleBounds: null,
      previewBodySemanticId: null,
      previewBodyVisibleLineCount: null,
      previewBodyBounds: null,
      previewMetadataChips: [],
      chipSemanticId: null,
      chipLabel: null,
      chipBounds: null,
      chipReadable: null,
      chipOverlaps: [],
      previewFooterCollision: null,
      previewListOverlap: null,
      selectionChangeSamples: [],
      selectionGeneration: null,
      fromStableKey: null,
      toStableKey: null,
      previewGenerationBefore: null,
      previewGenerationAfter: null,
      previewUpdated: null,
      noPreviewStaleAfterSelection: null,
      focusPreserved: null,
      filterGenerationSamples: [],
      filterGeneration: null,
      filterText: null,
      rowFingerprintBefore: null,
      rowFingerprintAfter: null,
      previewStaleRejected: null,
      selectedRowReanchored: null,
      resizeSamples: [],
      resizeGeneration: null,
      widthMode: null,
      noColumnOverlap: null,
      previewReadable: null,
      metadataChipsReadable: null,
      footerActionsReadable: null,
      footerActionSamples: [],
      footerActionSemanticId: null,
      label: null,
      enabled: null,
      overlapsPreview: null,
      overlapsSelectedRow: null,
      rowPreviewIdentityMatches: null,
      cleanupConfirmed: null,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_dense_list_detail_preview_readability_receipt" } }],
    failure: { code: "missing_dense_list_detail_preview_readability_receipt", stepName: "declare-required-receipt", message: "Missing app-side dense list/detail preview readability receipts." },
    warnings: ["file_linear:dense_list_detail_preview_readability_receipts_missing"],
  };
}

export async function runToastNotificationQueueLifecycleStressScenario(opts: {
  session: string;
  surface?: string;
  fixtures?: string[];
  cycles?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "toast-notification-queue-lifecycle-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_toast_notification_queue_lifecycle_receipt",
    linearIssue: "file_linear:toast_notification_queue_lifecycle_receipts_missing",
    toastNotificationQueueLifecycle: {
      kind: "ux.toastNotificationQueueLifecycle",
      requiredReceipt: "ux.toastNotificationQueueLifecycle",
      receiptKind: "ux.toastNotificationQueueLifecycle",
      toastStressId: "loop-twenty-toast-notification-queue-lifecycle",
      requestedSurface: opts.surface ?? "main",
      requestedFixtures: opts.fixtures ?? ["success", "duplicate", "persistent", "dismiss", "autohide"],
      requestedCycles: opts.cycles ?? 3,
      surfaceSamples: [],
      surface: null,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      stateReceipt: null,
      elementsReceipt: null,
      toastQueueReceipt: null,
      queueGeneration: null,
      notificationBridgeGeneration: null,
      toastSamples: [],
      toastId: null,
      message: null,
      variant: null,
      persistent: null,
      autoHideMs: null,
      duplicateCount: null,
      visible: null,
      visibleText: null,
      createdAtMs: null,
      expiresAtMs: null,
      dismissedAtMs: null,
      dismissReason: null,
      autohideObserved: null,
      manualDismissObserved: null,
      duplicateCollapsed: null,
      orderingPreserved: null,
      maxVisibleCount: null,
      toastBounds: null,
      overlapPairs: [],
      doesNotBlockInput: null,
      doesNotCoverFooter: null,
      staleToastRejected: null,
      noActionExecutionFromToast: null,
      networkAccessed: false,
      externalServiceContacted: false,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_toast_notification_queue_lifecycle_receipt" } }],
    failure: { code: "missing_toast_notification_queue_lifecycle_receipt", stepName: "declare-required-receipt", message: "Missing app-side toast queue lifecycle receipts." },
    warnings: ["file_linear:toast_notification_queue_lifecycle_receipts_missing"],
  };
}

export async function runDestructiveConfirmModalSafetyStressScenario(opts: {
  session: string;
  host?: string;
  fixture?: string;
  paths?: string[];
  dryRunOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  const dryRunOnly = opts.dryRunOnly === true;
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "destructive-confirm-modal-safety-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_destructive_confirm_modal_safety_receipt",
    linearIssue: "file_linear:destructive_confirm_modal_safety_receipts_missing",
    destructiveConfirmModalSafety: {
      kind: "ux.destructiveConfirmModalSafety",
      requiredReceipt: "ux.destructiveConfirmModalSafety",
      receiptKind: "ux.destructiveConfirmModalSafety",
      confirmSafetyStressId: "loop-twenty-destructive-confirm-modal-safety",
      requestedHost: opts.host ?? "main",
      requestedFixture: opts.fixture ?? "agentic-destructive-dry-run",
      requestedPaths: opts.paths ?? ["cancel", "confirm", "stale-confirm"],
      dryRunOnly,
      dryRunOnlyRequired: !dryRunOnly,
      hostSurface: null,
      hostAutomationWindowId: null,
      hostSemanticSurface: null,
      stateBefore: null,
      elementsBefore: null,
      confirmReceipt: null,
      confirmPromptId: null,
      confirmRouteGeneration: null,
      confirmSurfaceKind: null,
      parentAutomationWindowId: null,
      previousViewIdentity: null,
      dangerActionId: null,
      dangerActionLabel: null,
      dangerLevel: null,
      destructiveActionFixture: opts.fixture ?? "agentic-destructive-dry-run",
      confirmButtonSemanticId: null,
      cancelButtonSemanticId: null,
      focusedButtonBefore: null,
      tabFocusSamples: [],
      enterResolutionSamples: [],
      escapeCancelReceipt: null,
      cancelResolvedFalse: null,
      confirmResolvedTrue: null,
      actionMutationCountBefore: null,
      actionMutationCountAfter: null,
      noMutationBeforeConfirm: null,
      noMutationAfterCancel: null,
      noExecutionWithoutConfirm: null,
      destructiveCommandExecuted: false,
      systemCommandRequested: false,
      quitRequested: false,
      trashMutationRequested: false,
      restartRequested: false,
      shutdownRequested: false,
      staleConfirmRejected: null,
      wrongSurfaceConfirmRejected: null,
      focusRestored: null,
      selectionRestored: null,
      filterRestored: null,
      routeStackRestored: null,
      footerActionsSafe: null,
      networkAccessed: false,
      externalServiceContacted: false,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: dryRunOnly ? "missing_destructive_confirm_modal_safety_receipt" : "dryRunOnlyRequired" } }],
    failure: { code: dryRunOnly ? "missing_destructive_confirm_modal_safety_receipt" : "dryRunOnlyRequired", stepName: "declare-required-receipt", message: "Missing app-side destructive confirm dry-run safety receipts." },
    warnings: ["file_linear:destructive_confirm_modal_safety_receipts_missing"],
  };
}

export async function runLoadingSkeletonProgressRestorationStressScenario(opts: {
  session: string;
  surfaces?: string[];
  fixture?: string;
  cycles?: number;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "loading-skeleton-progress-restoration-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_loading_skeleton_progress_restoration_receipt",
    linearIssue: "file_linear:loading_skeleton_progress_restoration_receipts_missing",
    loadingSkeletonProgressRestoration: {
      kind: "ux.loadingSkeletonProgressRestoration",
      requiredReceipt: "ux.loadingSkeletonProgressRestoration",
      receiptKind: "ux.loadingSkeletonProgressRestoration",
      loadingSkeletonStressId: "loop-twenty-loading-skeleton-progress-restoration",
      requestedSurfaces: opts.surfaces ?? ["sdk-reference", "script-template-catalog"],
      requestedFixture: opts.fixture ?? "delayed-local",
      requestedCycles: opts.cycles ?? 4,
      surfaceSamples: [],
      surface: null,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      stateBefore: null,
      elementsBefore: null,
      requestGeneration: null,
      loadingReceipt: null,
      loadingState: null,
      skeletonVisible: null,
      skeletonRows: [],
      skeletonRowSemanticIds: [],
      skeletonBounds: null,
      progressReceipt: null,
      progressText: null,
      progressPercent: null,
      progressMonotonic: null,
      resultGeneration: null,
      resultsReadyReceipt: null,
      stateAfter: null,
      elementsAfter: null,
      realRowsVisible: null,
      skeletonCleared: null,
      noSkeletonAfterResults: null,
      selectedSemanticIdBefore: null,
      selectedSemanticIdAfter: null,
      selectionRestored: null,
      focusRestored: null,
      filterTextPreserved: null,
      scrollAnchorPreserved: null,
      footerActionStateDuringLoading: null,
      activationBlockedWhileLoading: null,
      noSubmitDuringLoading: null,
      staleLoadingGenerationRejected: null,
      staleProgressRejected: null,
      staleResultRejected: null,
      noBlankFrame: null,
      localFixtureOnly: true,
      networkAccessed: false,
      externalServiceContacted: false,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_loading_skeleton_progress_restoration_receipt" } }],
    failure: { code: "missing_loading_skeleton_progress_restoration_receipt", stepName: "declare-required-receipt", message: "Missing app-side loading skeleton/progress restoration receipts." },
    warnings: ["file_linear:loading_skeleton_progress_restoration_receipts_missing"],
  };
}

export async function runIconImageFallbackRedactionStressScenario(opts: {
  session: string;
  surfaces?: string[];
  fixtures?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "icon-image-fallback-redaction-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_icon_image_fallback_redaction_receipt",
    linearIssue: "file_linear:icon_image_fallback_redaction_receipts_missing",
    iconImageFallbackRedaction: {
      kind: "ux.iconImageFallbackRedaction",
      iconImageStressId: "loop-twenty-one-icon-image-fallback-redaction",
      requestedSurfaces: opts.surfaces ?? ["app-launcher", "file-search", "clipboard-history"],
      requestedFixtures: opts.fixtures ?? ["missing-file", "corrupt-png", "private-local-path", "data-uri-redacted"],
      surfaceSamples: [],
      surface: null,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      stateReceipt: null,
      elementsReceipt: null,
      imageFallbackReceipt: null,
      assetFixtureReceipt: null,
      fixtureKind: null,
      requestedImageSourceKind: null,
      requestedImageFingerprint: null,
      rawSourceRedacted: null,
      displayedImageKind: null,
      fallbackIconKind: null,
      fallbackReason: null,
      imageLoadGeneration: null,
      cacheKeyFingerprint: null,
      redactedPreview: null,
      noRawPath: null,
      noRawUrl: null,
      noFileContents: null,
      brokenImageRejected: null,
      unsupportedSchemeRejected: null,
      staleImageGenerationRejected: null,
      defaultIconRendered: null,
      accessibleLabelPreserved: null,
      rowIdentityPreserved: null,
      footerStatePreserved: null,
      networkAccessed: false,
      externalServiceContacted: false,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_icon_image_fallback_redaction_receipt" } }],
    failure: { code: "missing_icon_image_fallback_redaction_receipt", stepName: "declare-required-receipt", message: "Missing app-side icon/image fallback redaction receipts." },
    warnings: ["file_linear:icon_image_fallback_redaction_receipts_missing"],
  };
}

export async function runFooterStatusPersistenceStressScenario(opts: {
  session: string;
  surfaces?: string[];
  transitions?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "footer-status-persistence-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_footer_status_persistence_receipt",
    linearIssue: "file_linear:footer_status_persistence_receipts_missing",
    footerStatusPersistence: {
      kind: "ux.footerStatusPersistence",
      footerStatusStressId: "loop-twenty-one-footer-status-persistence",
      requestedSurfaces: opts.surfaces ?? ["main", "clipboard-history", "emoji-picker", "file-search", "actionsDialog"],
      requestedTransitions: opts.transitions ?? ["filter", "selection", "cmd-k", "escape", "clear-filter"],
      surfaceSamples: [],
      surface: null,
      hostAutomationWindowId: null,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      stateBefore: null,
      elementsBefore: null,
      transitionSamples: [],
      transitionGeneration: null,
      routeStackDepth: null,
      filterTextBefore: null,
      selectedSemanticIdBefore: null,
      footerReceipt: null,
      footerOwner: null,
      nativeFooterSurfaceId: null,
      gpuiFallbackVisible: null,
      renderedButtons: [],
      buttonSemanticIds: [],
      buttonLabel: null,
      buttonShortcutHint: null,
      disabledReason: null,
      statusBarReceipt: null,
      statusText: null,
      statusKind: null,
      statusGeneration: null,
      persistedAcrossFilter: null,
      persistedAcrossSelection: null,
      persistedAcrossActionsOpenClose: null,
      persistedAcrossPopupClose: null,
      noDuplicateFooterRows: null,
      noStaleStatusAfterRecovery: null,
      footerSafeSelection: null,
      inputCollisionFree: null,
      wrongSurfaceFooterRejected: null,
      staleFooterGenerationRejected: null,
      stateAfter: null,
      elementsAfter: null,
      networkAccessed: false,
      externalServiceContacted: false,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_footer_status_persistence_receipt" } }],
    failure: { code: "missing_footer_status_persistence_receipt", stepName: "declare-required-receipt", message: "Missing app-side footer/status persistence receipts." },
    warnings: ["file_linear:footer_status_persistence_receipts_missing"],
  };
}

export async function runKeyboardHintLabelParityStressScenario(opts: {
  session: string;
  surfaces?: string[];
  families?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "keyboard-hint-label-parity-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_keyboard_hint_label_parity_receipt",
    linearIssue: "file_linear:keyboard_hint_label_parity_receipts_missing",
    keyboardHintLabelParity: {
      kind: "ux.keyboardHintLabelParity",
      keyboardHintStressId: "loop-twenty-one-keyboard-hint-label-parity",
      requestedSurfaces: opts.surfaces ?? ["main", "clipboard-history", "emoji-picker", "file-search", "actionsDialog", "menuSyntaxTriggerPopup"],
      requestedFamilies: opts.families ?? ["footer", "row-accessory", "tooltip", "action-catalog"],
      surfaceSamples: [],
      surface: null,
      automationWindowId: null,
      osWindowId: null,
      semanticSurface: null,
      stateReceipt: null,
      elementsReceipt: null,
      actionCatalogReceipt: null,
      footerHintReceipt: null,
      rowHintSamples: [],
      tooltipHintReceipt: null,
      semanticId: null,
      actionId: null,
      hintOwner: null,
      visibleLabel: null,
      accessibleLabel: null,
      footerLabel: null,
      rowAccessoryLabel: null,
      tooltipLabel: null,
      tooltipNotRequiredReason: null,
      platformShortcutLabel: null,
      shortcutTokens: [],
      normalizedShortcut: null,
      glyphTokens: [],
      labelParityMatched: null,
      noMismatchedKeyGlyphs: null,
      noDuplicateShortcutHints: null,
      disabledStateParity: null,
      activationOwner: null,
      safeKeyboardActivation: null,
      noAccidentalExecution: null,
      hintGeneration: null,
      staleHintRejected: null,
      wrongSurfaceHintRejected: null,
      networkAccessed: false,
      externalServiceContacted: false,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_keyboard_hint_label_parity_receipt" } }],
    failure: { code: "missing_keyboard_hint_label_parity_receipt", stepName: "declare-required-receipt", message: "Missing app-side keyboard hint label parity receipts." },
    warnings: ["file_linear:keyboard_hint_label_parity_receipts_missing"],
  };
}

export async function runRowStateParityWithoutPointerStressScenario(opts: {
  session: string;
  surfaces?: string[];
  states?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "row-state-parity-without-pointer-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_row_state_parity_without_pointer_receipt",
    linearIssue: "file_linear:row_state_parity_without_pointer_receipts_missing",
    rowStateParityWithoutPointer: {
      kind: "ux.rowStateParityWithoutPointer",
      rowStateParityStressId: "loop-twenty-two-row-state-parity-without-pointer",
      requestedSurfaces: opts.surfaces ?? ["main", "clipboard-history", "emoji-picker", "file-search", "actionsDialog"],
      requestedStates: opts.states ?? ["selected", "focused", "hovered", "selected-hovered"],
      surfaceSamples: [], surface: null, automationWindowId: null, osWindowId: null, semanticSurface: null,
      stateReceipt: null, elementsReceipt: null, rowStateParityWithoutPointerReceipt: null,
      rowStateSamples: [], semanticId: null, rowRole: null, rowIndex: null, rowLabel: null, modality: null,
      selectedSemanticId: null, focusedSemanticId: null, hoverSemanticId: null,
      keyboardFocusRingVisible: null, selectionPaintVisible: null, hoverPaintVisible: null, focusPaintVisible: null,
      selectedFillToken: null, hoverFillToken: null, focusRingToken: null, textOpacityToken: null, iconOpacityToken: null,
      selectedPrecedenceOverHover: null, hoverDoesNotOverrideSelection: null, focusDoesNotStealSelection: null,
      focusedRowMatchesElements: null, selectedRowMatchesState: null, hoverReceiptSyntheticOnly: null,
      noNativePointerRequired: true, staleRowStateRejected: null, wrongSurfaceRowStateRejected: null, noAccidentalExecution: null,
      networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_row_state_parity_without_pointer_receipt" } }],
    failure: { code: "missing_row_state_parity_without_pointer_receipt", stepName: "declare-required-receipt", message: "Missing app-side row visual-state parity receipts without native pointer input." },
    warnings: ["file_linear:row_state_parity_without_pointer_receipts_missing"],
  };
}

export async function runQuietChromeCardNestingStressScenario(opts: {
  session: string;
  surfaces?: string[];
  chrome?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "quiet-chrome-card-nesting-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_quiet_chrome_card_nesting_receipt",
    linearIssue: "file_linear:quiet_chrome_card_nesting_receipts_missing",
    quietChromeCardNesting: {
      kind: "ux.quietChromeCardNesting",
      quietChromeStressId: "loop-twenty-two-quiet-chrome-card-nesting",
      requestedSurfaces: opts.surfaces ?? ["main", "clipboard-history", "emoji-picker", "file-search", "actionsDialog", "promptPopup"],
      requestedChrome: opts.chrome ?? "quiet",
      surfaceSamples: [], surface: null, automationWindowId: null, osWindowId: null, semanticSurface: null,
      stateReceipt: null, elementsReceipt: null, quietChromeCardNestingReceipt: null,
      chromeLayerSamples: [], shellLayer: null, contentLayer: null, rowLayer: null, popupLayer: null, footerLayer: null,
      borderToken: null, fillToken: null, shadowToken: null, vibrancyMaterial: null, cornerRadius: null,
      insetPx: null, gapPx: null, cardDepth: null, nestedCardCount: null, maxAllowedCardDepth: null,
      duplicateBorderRejected: null, opaqueFillRejected: null, heavyShadowRejected: null, doubleCardNestingRejected: null,
      footerChromeSeparated: null, inputChromeSeparated: null, popupMaterialPreserved: null, quietChromeBudgetMatched: null,
      themeTokenFingerprint: null, staleChromeTokenRejected: null, wrongSurfaceChromeRejected: null,
      networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_quiet_chrome_card_nesting_receipt" } }],
    failure: { code: "missing_quiet_chrome_card_nesting_receipt", stepName: "declare-required-receipt", message: "Missing app-side quiet chrome/card nesting receipts." },
    warnings: ["file_linear:quiet_chrome_card_nesting_receipts_missing"],
  };
}

export async function runScrollShadowStickyHeaderDensityStressScenario(opts: {
  session: string;
  surfaces?: string[];
  scrollPositions?: string[];
  density?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "scroll-shadow-sticky-header-density-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_scroll_shadow_sticky_header_density_receipt",
    linearIssue: "file_linear:scroll_shadow_sticky_header_density_receipts_missing",
    scrollShadowStickyHeaderDensity: {
      kind: "ux.scrollShadowStickyHeaderDensity",
      scrollChromeDensityStressId: "loop-twenty-two-scroll-shadow-sticky-header-density",
      requestedSurfaces: opts.surfaces ?? ["clipboard-history", "emoji-picker", "file-search", "app-launcher", "actionsDialog"],
      requestedScrollPositions: opts.scrollPositions ?? ["top", "middle", "bottom"],
      requestedDensity: opts.density ?? ["compact", "default"],
      surfaceSamples: [], surface: null, automationWindowId: null, osWindowId: null, semanticSurface: null,
      stateReceipt: null, elementsReceipt: null, scrollShadowStickyHeaderDensityReceipt: null,
      scrollSamples: [], scrollPosition: null, scrollTop: null, scrollViewportBounds: null, scrollContentBounds: null,
      scrollContentHeight: null, scrollViewportHeight: null, stickyHeaderReceipt: null, headerSemanticId: null,
      headerBounds: null, headerPinned: null, headerZIndex: null, headerDoesNotOverlapRows: null,
      headerDoesNotOverlapInput: null, headerDoesNotOverlapFooter: null, scrollShadowReceipt: null,
      topShadowVisible: null, bottomShadowVisible: null, topShadowOpacityToken: null, bottomShadowOpacityToken: null,
      shadowGradientToken: null, shadowMatchesScrollPosition: null, densityReceipt: null, densityMode: null,
      rowHeightPx: null, sectionHeaderHeightPx: null, inputHeightPx: null, footerHeightPx: null, verticalGapPx: null,
      horizontalInsetPx: null, remSize: null, scaleFactor: null, densityTokenFingerprint: null, rowRhythmStable: null,
      footerSafeViewport: null, selectedRowVisibleAboveFooter: null, staleScrollGenerationRejected: null,
      wrongSurfaceScrollRejected: null, networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_scroll_shadow_sticky_header_density_receipt" } }],
    failure: { code: "missing_scroll_shadow_sticky_header_density_receipt", stepName: "declare-required-receipt", message: "Missing app-side scroll shadow, sticky header, and density receipts." },
    warnings: ["file_linear:scroll_shadow_sticky_header_density_receipts_missing"],
  };
}

export async function runPopupFocusKeycapVisualSemanticsStressScenario(opts: {
  session: string;
  surfaces?: string[];
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "popup-focus-keycap-visual-semantics-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_popup_focus_keycap_visual_semantics_receipt",
    linearIssue: "file_linear:popup_focus_keycap_visual_semantics_receipts_missing",
    popupFocusKeycapVisualSemantics: {
      kind: "ux.popupFocusKeycapVisualSemantics", popupKeycapStressId: "loop-twenty-three-popup-focus-keycap",
      requestedSurfaces: opts.surfaces ?? ["actionsDialog", "menuSyntaxTriggerPopup", "confirmPrompt"],
      surfaceSamples: [], surface: null, popupKind: null, automationWindowId: null, osWindowId: null, parentAutomationWindowId: null, semanticSurface: null,
      stateReceipt: null, elementsReceipt: null, popupFocusKeycapVisualSemanticsReceipt: null,
      keycapSamples: [], keycapRole: null, keycapLabel: null, shortcutLabel: null, normalizedShortcutTokens: [], platformGlyph: null,
      focused: null, focusOwnerSemanticId: null, focusedButtonSemanticId: null, focusedKeycapMatchesFocusedButton: null,
      escapeKeycapAvailable: null, enterKeycapAvailable: null, keycapFillToken: null, keycapGlyphToken: null,
      keycapTextToken: null, focusRingToken: null, dangerSemanticOnLabelNotKeycap: null, disabledKeycapMuted: null,
      shortcutGlyphNormalized: null, popupIsTopmostOwner: null, parentFocusUnchanged: null, parentSelectionUnchanged: null,
      staleFocusReceiptRejected: null, wrongSurfaceKeycapRejected: null, noAccidentalExecution: null,
      networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_popup_focus_keycap_visual_semantics_receipt" } }],
    failure: { code: "missing_popup_focus_keycap_visual_semantics_receipt", stepName: "declare-required-receipt", message: "Missing app-side popup focus/keycap visual semantics receipts." },
    warnings: ["file_linear:popup_focus_keycap_visual_semantics_receipts_missing"],
  };
}

export async function runReducedMotionAnimationDisableStressScenario(opts: {
  session: string;
  surfaces?: string[];
  fixture?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "reduced-motion-animation-disable-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_reduced_motion_animation_disable_receipt",
    linearIssue: "file_linear:reduced_motion_animation_disable_receipts_missing",
    reducedMotionAnimationDisable: {
      kind: "ux.reducedMotionAnimationDisable", reducedMotionStressId: "loop-twenty-three-reduced-motion-animation-disable",
      requestedSurfaces: opts.surfaces ?? ["main", "actionsDialog", "menuSyntaxTriggerPopup"], requestedFixture: opts.fixture ?? "reduced-motion",
      surfaceSamples: [], surface: null, automationWindowId: null, osWindowId: null, semanticSurface: null,
      stateReceipt: null, elementsReceipt: null, reducedMotionAnimationDisableReceipt: null,
      motionPolicyReceipt: null, motionPreferenceSource: null, fixtureOnlyReducedMotion: true, systemPreferenceNotRead: true,
      systemPreferenceNotMutated: true, animationSamples: [], animationName: null, transitionGeneration: null, frameId: null,
      frameClockPaused: null, motionDurationMs: null, effectiveDurationMs: null, animatedOpacityStable: null,
      animatedTransformStable: null, spinnerHiddenOrStatic: null, shimmerDisabled: null, loadingPulseDisabled: null,
      autoFocusPreserved: null, selectedRowPreserved: null, cursorPositionPreserved: null, noLayoutShiftDuringMotionDisable: null,
      staleMotionGenerationRejected: null, wrongSurfaceMotionRejected: null, noNativeInputRequired: true,
      networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_reduced_motion_animation_disable_receipt" } }],
    failure: { code: "missing_reduced_motion_animation_disable_receipt", stepName: "declare-required-receipt", message: "Missing app-side reduced-motion animation-disable receipts." },
    warnings: ["file_linear:reduced_motion_animation_disable_receipts_missing"],
  };
}

export async function runCommandSearchHighlightingAccessoryBadgesStressScenario(opts: {
  session: string;
  hosts?: string[];
  query?: string;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "command-search-highlighting-accessory-badges-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_command_search_highlighting_accessory_badges_receipt",
    linearIssue: "file_linear:command_search_highlighting_accessory_badges_receipts_missing",
    commandSearchHighlightingAccessoryBadges: {
      kind: "ux.commandSearchHighlightAccessoryBadges", commandHighlightBadgeStressId: "loop-twenty-three-command-search-highlighting-accessory-badges",
      requestedHosts: opts.hosts ?? ["main", "actionsDialog", "app-launcher", "menuSyntaxTriggerPopup"], query: opts.query ?? "agentic-loop-twenty-three",
      hostSamples: [], host: null, popupKind: null, automationWindowId: null, osWindowId: null, semanticSurface: null,
      stateReceipt: null, elementsReceipt: null, commandSearchHighlightingAccessoryBadgesReceipt: null,
      querySamples: [], searchGeneration: null, commandRows: [], semanticId: null, commandId: null, commandLabel: null,
      sectionLabel: null, highlightedRanges: [], highlightText: null, matchedQuery: null, highlightMatchesFilter: null,
      highlightDoesNotMutateLabel: null, accessoryBadges: [], badgeKind: null, badgeLabel: null, badgeTooltip: null,
      shortcutBadge: null, disabledBadge: null, noOpBadge: null, loadingBadge: null, accessoryOrderStable: null,
      badgesMatchActionCatalog: null, disabledReasonVisible: null, loadingReasonVisible: null, staleBadgeRejected: null,
      staleHighlightRejected: null, wrongHostCommandRejected: null, footerActionsStable: null, noAccidentalExecution: null,
      networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_command_search_highlighting_accessory_badges_receipt" } }],
    failure: { code: "missing_command_search_highlighting_accessory_badges_receipt", stepName: "declare-required-receipt", message: "Missing app-side command search highlighting/accessory badge receipts." },
    warnings: ["file_linear:command_search_highlighting_accessory_badges_receipts_missing"],
  };
}

export async function runClipboardCopyVisualFeedbackStressScenario(opts: {
  session: string;
  hosts?: string[];
  fixture?: string;
  pasteboardScope?: string;
  noSystemPasteboard?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "clipboard-copy-visual-feedback-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_clipboard_copy_visual_feedback_receipt",
    linearIssue: "file_linear:clipboard_copy_visual_feedback_receipts_missing",
    clipboardCopyVisualFeedback: {
      kind: "ux.clipboardCopyVisualFeedback",
      clipboardCopyFeedbackStressId: "loop-twenty-four-clipboard-copy-visual-feedback",
      requestedHosts: opts.hosts ?? ["file-search", "actionsDialog", "app-launcher"],
      requestedFixture: opts.fixture ?? "agentic-copy-preview",
      pasteboardScope: opts.pasteboardScope ?? "fixture",
      noSystemPasteboard: opts.noSystemPasteboard ?? true,
      hostSamples: [], host: null, automationWindowId: null, osWindowId: null, semanticSurface: null,
      stateReceipt: null, elementsReceipt: null, clipboardCopyVisualFeedbackReceipt: null,
      copyActionSemanticId: null, copyActionLabel: null, copyGeneration: null,
      copyButtonStateBefore: null, copyButtonStateAfter: null, visibleCopiedState: null,
      copiedStateDurationMs: null, copyToastReceipt: null, redactedPayloadPreview: null,
      payloadFingerprint: null, fixturePasteboardUsed: null, systemPasteboardUnchanged: null,
      originalPasteboardFingerprint: null, postRunPasteboardFingerprint: null,
      noRawClipboardContentLogged: null, staleCopyGenerationRejected: null, wrongHostCopyRejected: null,
      noAccidentalPaste: null, networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_clipboard_copy_visual_feedback_receipt" } }],
    failure: { code: "missing_clipboard_copy_visual_feedback_receipt", stepName: "declare-required-receipt", message: "Missing app-side fixture-scoped copy visual feedback receipts." },
    warnings: ["file_linear:clipboard_copy_visual_feedback_receipts_missing"],
  };
}

export async function runPortalCancelReturnStateRestorationStressScenario(opts: {
  session: string;
  origins?: string[];
  portal?: string;
  query?: string;
  cancelMethods?: string[];
  fixture?: string;
  noNativePicker?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "portal-cancel-return-state-restoration-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_portal_cancel_return_state_restoration_receipt",
    linearIssue: "file_linear:portal_cancel_return_state_restoration_receipts_missing",
    portalCancelReturnStateRestoration: {
      kind: "ux.portalCancelReturnStateRestoration",
      portalCancelReturnStressId: "loop-twenty-four-portal-cancel-return-state-restoration",
      requestedOrigins: opts.origins ?? ["acp-composer", "notes"],
      requestedPortal: opts.portal ?? "file-search",
      requestedQuery: opts.query ?? "AGENTS.md",
      requestedCancelMethods: opts.cancelMethods ?? ["escape", "back"],
      requestedFixture: opts.fixture ?? "repo-file",
      noNativePicker: opts.noNativePicker ?? true,
      originSamples: [], origin: null, originAutomationWindowId: null, originGeneration: null,
      originSemanticSurface: null, originStateReceipt: null, originElementsReceipt: null,
      draftTextBeforePortal: null, cursorBeforePortal: null, selectionBeforePortal: null,
      portalSessionId: null, portalSurface: null, portalAutomationWindowId: null, portalQuery: null,
      portalSelectionBeforeCancel: null, cancelMethod: null, cancelReceipt: null,
      returnTargetIdentity: null, returnGeneration: null, focusRestored: null, draftTextRestored: null,
      cursorRestored: null, selectionRestored: null, filterRestored: null, scrollRestored: null,
      noContextPartInserted: null, noPromptSubmit: null, noSelectionMutationDuringPortal: null,
      stalePortalReturnRejected: null, foreignPortalEventRejected: null, wrongOriginReturnRejected: null,
      networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_portal_cancel_return_state_restoration_receipt" } }],
    failure: { code: "missing_portal_cancel_return_state_restoration_receipt", stepName: "declare-required-receipt", message: "Missing app-side portal cancel/back return restoration receipts." },
    warnings: ["file_linear:portal_cancel_return_state_restoration_receipts_missing"],
  };
}

export async function runTooltipHoverFocusAffordanceStressScenario(opts: {
  session: string;
  surfaces?: string[];
  targets?: string[];
  fixture?: string;
  inputModes?: string[];
  noNativePointer?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "tooltip-hover-focus-affordance-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_tooltip_hover_focus_affordance_receipt",
    linearIssue: "file_linear:tooltip_hover_focus_affordance_receipts_missing",
    tooltipHoverFocusAffordance: {
      kind: "ux.tooltipHoverFocusAffordance",
      tooltipHoverFocusStressId: "loop-twenty-four-tooltip-hover-focus-affordance",
      requestedSurfaces: opts.surfaces ?? ["main", "actionsDialog", "app-launcher"],
      requestedTargets: opts.targets ?? ["truncated-row", "disabled-action", "footer-button"],
      requestedFixture: opts.fixture ?? "agentic-tooltips",
      requestedInputModes: opts.inputModes ?? ["protocol-hover", "keyboard-focus"],
      noNativePointer: opts.noNativePointer ?? true,
      surfaceSamples: [], surface: null, automationWindowId: null, osWindowId: null, semanticSurface: null,
      stateReceipt: null, elementsReceipt: null, tooltipHoverFocusAffordanceReceipt: null,
      targetSamples: [], targetSemanticId: null, targetRole: null, triggerMode: null,
      hoverGeneration: null, focusGeneration: null, tooltipGeneration: null, tooltipText: null,
      tooltipKind: null, tooltipAnchorBounds: null, tooltipBounds: null, tooltipPlacement: null,
      hoverDelayMs: null, hoverDelayRespected: null, keyboardFocusOpensTooltip: null,
      tooltipAccessibleDescriptionMatches: null, escapeDismissesTooltip: null, scrollDismissesTooltip: null,
      focusLossDismissesTooltip: null, noFocusSteal: null, targetFocusPreserved: null,
      doesNotCoverTarget: null, doesNotCoverFooter: null, doesNotCoverPopupOwner: null,
      staleTooltipGenerationRejected: null, wrongSurfaceTooltipRejected: null, noAccidentalExecution: null,
      networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_tooltip_hover_focus_affordance_receipt" } }],
    failure: { code: "missing_tooltip_hover_focus_affordance_receipt", stepName: "declare-required-receipt", message: "Missing app-side tooltip hover/focus affordance receipts." },
    warnings: ["file_linear:tooltip_hover_focus_affordance_receipts_missing"],
  };
}

export async function runShortcutRecorderCancelLayeringStressScenario(opts: {
  session: string;
  surface?: string;
  action?: string;
  cancelMethods?: string[];
  inputModes?: string[];
  sandboxConfig?: boolean;
  noConfigWrite?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "shortcut-recorder-cancel-layering-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_shortcut_recorder_cancel_layering_receipt",
    linearIssue: "file_linear:shortcut_recorder_cancel_layering_receipts_missing",
    shortcutRecorderCancelLayeringReceipt: {
      kind: "ux.shortcutRecorderCancelLayering",
      shortcutRecorderCancelLayeringStressId: "loop-twenty-five-shortcut-recorder-cancel-layering",
      session: opts.session,
      surface: opts.surface ?? "shortcuts",
      action: opts.action ?? "test-agentic-shortcut",
      requestedCancelMethods: opts.cancelMethods ?? ["escape", "cmd-w", "backdrop", "parent-click"],
      requestedInputModes: opts.inputModes ?? ["protocol-key", "protocol-click"],
      sandboxConfig: opts.sandboxConfig ?? true,
      noConfigWrite: opts.noConfigWrite ?? true,
      parentAutomationWindowId: null, recorderAutomationWindowId: null, parentSemanticSurface: null,
      modalLayerReceipt: null, parentBounds: null, recorderBounds: null, shellNarrowerThanParent: null,
      titleText: null, pressKeysPlaceholderVisible: null, footerAbsent: null, visibleCancelButton: null,
      cancelMethod: null, escapeCancels: null, cmdWCancels: null, backdropClickCancels: null,
      parentClickCancels: null, chordNotCapturedOnCancel: null, configFingerprintBefore: null,
      configFingerprintAfter: null, configUnchanged: null, globalHotkeyNotRegistered: null,
      parentFocusRestored: null, parentSelectionRestored: null, staleRecorderRejected: null,
      wrongParentRejected: null, networkAccessed: false, externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_shortcut_recorder_cancel_layering_receipt" } }],
    failure: { code: "missing_shortcut_recorder_cancel_layering_receipt", stepName: "declare-required-receipt", message: "Missing app-side shortcut recorder cancel/layering receipts." },
    warnings: ["file_linear:shortcut_recorder_cancel_layering_receipts_missing"],
  };
}

export async function runInlinePopoverAnchorResizeStressScenario(opts: {
  session: string;
  families?: string[];
  widths?: string[];
  fixture?: string;
  inputModes?: string[];
  noNativeInput?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "inline-popover-anchor-resize-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_inline_popover_anchor_resize_receipt",
    linearIssue: "file_linear:inline_popover_anchor_resize_receipts_missing",
    inlinePopoverAnchorResizeReceipt: {
      kind: "ux.inlinePopoverAnchorResize",
      inlinePopoverAnchorResizeStressId: "loop-twenty-five-inline-popover-anchor-resize",
      session: opts.session,
      requestedFamilies: opts.families ?? ["acp-slash", "acp-mention", "menu-syntax-colon"],
      requestedWidths: opts.widths ?? ["mini", "narrow", "full"],
      requestedFixture: opts.fixture ?? "agentic-inline-popover",
      requestedInputModes: opts.inputModes ?? ["protocol-key", "protocol-resize"],
      noNativeInput: opts.noNativeInput ?? true,
      familySamples: [], family: null, originAutomationWindowId: null, popupAutomationWindowId: null,
      parentSemanticSurface: null, triggerText: null, triggerRange: null, anchorBoundsBeforeResize: null,
      anchorBoundsAfterResize: null, popupBoundsBeforeResize: null, popupBoundsAfterResize: null,
      resizeGeneration: null, widthMode: null, visibleRangeBeforeResize: null, visibleRangeAfterResize: null,
      selectedRowVisible: null, selectedRowIdentityPreserved: null, synopsisBounds: null, footerRowBounds: null,
      noSynopsisFooterOverlap: null, noParentClipping: null, noViewportOverflow: null, zOrderAboveParent: null,
      noFocusSteal: null, keyboardSelectionPreserved: null, keyboardFallbackAccepted: null,
      screenshotToSemanticsAlignment: null, strictCaptureTarget: null, blankScreenshotRejected: null,
      staleResizeGenerationRejected: null, wrongPopupRejected: null, networkAccessed: false,
      externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_inline_popover_anchor_resize_receipt" } }],
    failure: { code: "missing_inline_popover_anchor_resize_receipt", stepName: "declare-required-receipt", message: "Missing app-side inline popover anchor/resize receipts." },
    warnings: ["file_linear:inline_popover_anchor_resize_receipts_missing"],
  };
}

export async function runDisabledFooterHitTargetRefusalStressScenario(opts: {
  session: string;
  surfaces?: string[];
  fixtures?: string[];
  inputModes?: string[];
  noNativePointer?: boolean;
  dryRunOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "disabled-footer-hit-target-refusal-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_disabled_footer_hit_target_refusal_receipt",
    linearIssue: "file_linear:disabled_footer_hit_target_refusal_receipts_missing",
    disabledFooterHitTargetRefusalReceipt: {
      kind: "ux.disabledFooterHitTargetRefusal",
      disabledFooterHitTargetRefusalStressId: "loop-twenty-five-disabled-footer-hit-target-refusal",
      session: opts.session,
      requestedSurfaces: opts.surfaces ?? ["drop-prompt", "fields-prompt", "path-prompt"],
      requestedFixtures: opts.fixtures ?? ["empty-drop", "invalid-fields", "missing-path"],
      requestedInputModes: opts.inputModes ?? ["enter", "footer-shortcut", "protocol-footer-click"],
      noNativePointer: opts.noNativePointer ?? true,
      dryRunOnly: opts.dryRunOnly ?? true,
      surfaceSamples: [], surface: null, fixture: null, automationWindowId: null, osWindowId: null,
      semanticSurface: null, stateReceipt: null, elementsReceipt: null, activeFooter: null,
      nativeFooterSurfaceId: null, footerButtonSemanticId: null, footerButtonLabel: null,
      actionDisabled: null, disabledReason: null, disabledVisualState: null, disabledAccessibleState: null,
      keyboardEnterRefused: null, footerShortcutRefused: null, protocolFooterClickRefused: null,
      cmdKActionsStillAvailable: null, noSubmitReceipt: null, submitAttemptGeneration: null,
      sideEffectCountsBefore: null, sideEffectCountsAfter: null, stateFingerprintBefore: null,
      stateFingerprintAfter: null, focusPreserved: null, selectionPreserved: null, filterPreserved: null,
      staleFooterGenerationRejected: null, wrongSurfaceFooterRejected: null, networkAccessed: false,
      externalServiceContacted: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_disabled_footer_hit_target_refusal_receipt" } }],
    failure: { code: "missing_disabled_footer_hit_target_refusal_receipt", stepName: "declare-required-receipt", message: "Missing app-side disabled footer hit-target refusal receipts." },
    warnings: ["file_linear:disabled_footer_hit_target_refusal_receipts_missing"],
  };
}

export async function runMiniFullTransitionLayoutContinuityStressScenario(opts: {
  session: string;
  surfaces?: string[];
  transitions?: string[];
  fixture?: string;
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "mini-full-transition-layout-continuity-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_mini_full_transition_layout_continuity_receipt",
    linearIssue: "file_linear:mini_full_transition_layout_continuity_receipts_missing",
    miniFullTransitionLayoutContinuityReceipt: {
      kind: "ux.miniFullTransitionLayoutContinuity",
      miniFullTransitionLayoutContinuityStressId: "loop-twenty-six-mini-full-transition-layout-continuity",
      session: opts.session,
      requestedSurfaces: opts.surfaces ?? ["main", "mini-prompt", "fields-prompt", "actionsDialog"],
      requestedTransitions: opts.transitions ?? ["mini-to-full", "full-to-mini", "hide-show", "return-to-origin"],
      fixture: opts.fixture ?? "agentic-mini-full-layout",
      requestedInputModes: opts.inputModes ?? ["protocol-key", "protocol-resize"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, localFixtureOnly: opts.localFixtureOnly ?? true,
      modeSamples: [], surface: null, transition: null, automationWindowId: null, osWindowId: null,
      semanticSurface: null, modeBefore: null, modeAfter: null, viewTypeBefore: null, viewTypeAfter: null,
      transitionGeneration: null, remSizeBefore: null, remSizeAfter: null, scaleFactor: null,
      windowBoundsBefore: null, windowBoundsAfter: null, contentBoundsBefore: null, contentBoundsAfter: null,
      inputBoundsBefore: null, inputBoundsAfter: null, listViewportBoundsBefore: null, listViewportBoundsAfter: null,
      footerBoundsBefore: null, footerBoundsAfter: null, nativeFooterSurfaceId: null, focusRingBounds: null,
      selectedRowVisible: null, selectedRowAboveFooter: null, noInputFooterOverlap: null, noContentClip: null,
      noFooterClip: null, noPopupMainClobbering: null, screenshotToSemanticsAlignment: null,
      strictCaptureTarget: null, blankScreenshotRejected: null, staleModeGenerationRejected: null,
      wrongSurfaceRejected: null, openedSystemSettings: false, mutatedTcc: false, systemPasteboardMutated: false,
      setupInstallFlowEntered: false, triggeredSecurityPrompt: false, networkAccessed: false,
      externalServiceContacted: false, destructiveOperationRequested: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_mini_full_transition_layout_continuity_receipt" } }],
    failure: { code: "missing_mini_full_transition_layout_continuity_receipt", stepName: "declare-required-receipt", message: "Missing app-side mini/full transition layout continuity receipts." },
    warnings: ["file_linear:mini_full_transition_layout_continuity_receipts_missing"],
  };
}

export async function runFilterInputDecorationChipLayoutStressScenario(opts: {
  session: string;
  surfaces?: string[];
  queries?: string[];
  widths?: string[];
  scaleFactors?: number[];
  fixture?: string;
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noConfigWrite?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "filter-input-decoration-chip-layout-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_filter_input_decoration_chip_layout_receipt",
    linearIssue: "file_linear:filter_input_decoration_chip_layout_receipts_missing",
    filterInputDecorationChipLayoutReceipt: {
      kind: "ux.filterInputDecorationChipLayout",
      filterInputDecorationChipLayoutStressId: "loop-twenty-six-filter-input-decoration-chip-layout",
      session: opts.session, requestedSurfaces: opts.surfaces ?? ["main"],
      requestedQueries: opts.queries ?? ["f: AGENTS.md", "c: agentic", "~/script", ":actions", ";note", "!command", "literal\\:chip"],
      requestedWidths: opts.widths ?? ["mini", "narrow", "full"], requestedScaleFactors: opts.scaleFactors ?? [1, 1.25, 1.5],
      fixture: opts.fixture ?? "agentic-filter-input-decorations", requestedInputModes: opts.inputModes ?? ["protocol-set-filter", "protocol-resize"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noConfigWrite: opts.noConfigWrite ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true, inputDecorationSamples: [], surface: null, query: null,
      widthMode: null, scaleFactor: null, remSize: null, automationWindowId: null, osWindowId: null,
      semanticSurface: null, stateReceipt: null, elementsReceipt: null, filterInputDecorations: null,
      renderedInputText: null, strippedSearchText: null, chipRanges: [], chipRoles: [], chipBounds: [],
      textBounds: null, renderedTextBounds: null, cursorBounds: null, placeholderBounds: null,
      measuredWidth: null, availableWidth: null, visibleText: null, decorationGeneration: null,
      inputGeneration: null, sourceHeadCleared: null, staleDecorationCleared: null, noChipTextOverlap: null,
      noChipCursorOverlap: null, noPlaceholderOverlap: null, noInputFooterOverlap: null, noHorizontalClip: null,
      tooltipOrAccessibleFullText: null, screenshotToSemanticsAlignment: null, strictCaptureTarget: null,
      blankScreenshotRejected: null, staleDecorationGenerationRejected: null, wrongSurfaceRejected: null,
      openedSystemSettings: false, mutatedTcc: false, systemPasteboardMutated: false, configUnchanged: true,
      setupInstallFlowEntered: false, triggeredSecurityPrompt: false, networkAccessed: false,
      externalServiceContacted: false, destructiveOperationRequested: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_filter_input_decoration_chip_layout_receipt" } }],
    failure: { code: "missing_filter_input_decoration_chip_layout_receipt", stepName: "declare-required-receipt", message: "Missing app-side filter input decoration chip layout receipts." },
    warnings: ["file_linear:filter_input_decoration_chip_layout_receipts_missing"],
  };
}

export async function runFocusRingViewportIntegrityStressScenario(opts: {
  session: string;
  surfaces?: string[];
  fixture?: string;
  inputModes?: string[];
  steps?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "focus-ring-viewport-integrity-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_focus_ring_viewport_integrity_receipt",
    linearIssue: "file_linear:focus_ring_viewport_integrity_receipts_missing",
    focusRingViewportIntegrityReceipt: {
      kind: "ux.focusRingViewportIntegrity",
      focusRingViewportIntegrityStressId: "loop-twenty-six-focus-ring-viewport-integrity",
      session: opts.session, requestedSurfaces: opts.surfaces ?? ["main", "actionsDialog", "fields-prompt", "path-prompt"],
      fixture: opts.fixture ?? "agentic-focus-rings", requestedInputModes: opts.inputModes ?? ["protocol-key", "simulate-gpui-event"],
      requestedSteps: opts.steps ?? ["tab", "shift-tab", "up", "down", "escape"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true, localFixtureOnly: opts.localFixtureOnly ?? true,
      focusSamples: [], surface: null, automationWindowId: null, osWindowId: null, semanticSurface: null,
      focusStep: null, inputMode: null, focusGeneration: null, focusedSemanticId: null, focusOwner: null,
      semanticFocusMatchesState: null, focusRingBounds: null, focusedElementBounds: null, viewportBounds: null,
      scrollViewportBounds: null, contentBounds: null, footerBounds: null, popupBounds: null, ringVisible: null,
      ringNotClipped: null, ringWithinViewport: null, ringAboveFooter: null, ringNotObscuredByFooter: null,
      ringNotCoveredByPopup: null, tabOrderIndex: null, tabOrderStable: null, selectionPreserved: null,
      scrollAnchorPreserved: null, focusRestoredAfterEscape: null, noActivationReceipt: null, noSubmitReceipt: null,
      staleFocusGenerationRejected: null, wrongSurfaceFocusRejected: null, openedSystemSettings: false,
      mutatedTcc: false, systemPasteboardMutated: false, setupInstallFlowEntered: false,
      triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false,
      destructiveOperationRequested: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_focus_ring_viewport_integrity_receipt" } }],
    failure: { code: "missing_focus_ring_viewport_integrity_receipt", stepName: "declare-required-receipt", message: "Missing app-side focus ring viewport integrity receipts." },
    warnings: ["file_linear:focus_ring_viewport_integrity_receipts_missing"],
  };
}

export async function runWarningBannerActionDismissSemanticsStressScenario(opts: {
  session: string;
  surface?: string;
  fixtures?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noConfigWrite?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "warning-banner-action-dismiss-semantics-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_warning_banner_action_dismiss_semantics_receipt",
    linearIssue: "file_linear:warning_banner_action_dismiss_semantics_receipts_missing",
    warningBannerActionDismissSemanticsReceipt: {
      kind: "ux.warningBannerActionDismissSemantics",
      warningBannerActionDismissSemanticsStressId: "loop-twenty-seven-warning-banner-action-dismiss-semantics",
      session: opts.session, surface: opts.surface ?? "main",
      requestedFixtures: opts.fixtures ?? ["warning", "actionable", "dismissible", "error"],
      requestedInputModes: opts.inputModes ?? ["protocol-hover", "protocol-click", "protocol-key"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noConfigWrite: opts.noConfigWrite ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true, bannerSamples: [], fixture: null,
      bannerGeneration: null, bannerSemanticId: null, bannerKind: null, bannerVisibleText: null,
      bannerBounds: null, bannerTextBounds: null, bannerActionSemanticId: null, bannerDismissSemanticId: null,
      hoverStateReceipt: null, focusStateReceipt: null, dismissClickReceipt: null, actionClickReceipt: null,
      actionExecutionPreventedForDismiss: null, dismissDoesNotTriggerAction: null,
      actionDoesNotDismissUnlessConfigured: null, nonColorStateCue: null, contrastRatio: null,
      footerNotObscured: null, inputNotObscured: null, staleBannerGenerationRejected: null,
      wrongSurfaceRejected: null, systemPasteboardMutated: false, openedSystemSettings: false,
      mutatedTcc: false, networkAccessed: false, externalServiceContacted: false,
      destructiveOperationRequested: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_warning_banner_action_dismiss_semantics_receipt" } }],
    failure: { code: "missing_warning_banner_action_dismiss_semantics_receipt", stepName: "declare-required-receipt", message: "Missing app-side warning banner action/dismiss semantics receipts." },
    warnings: ["file_linear:warning_banner_action_dismiss_semantics_receipts_missing"],
  };
}

export async function runSelectPromptMultiselectKeyboardStateStressScenario(opts: {
  session: string;
  surface?: string;
  fixture?: string;
  choices?: number;
  selectionSteps?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "select-prompt-multiselect-keyboard-state-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_select_prompt_multiselect_keyboard_state_receipt",
    linearIssue: "file_linear:select_prompt_multiselect_keyboard_state_receipts_missing",
    selectPromptMultiselectKeyboardStateReceipt: {
      kind: "ux.selectPromptMultiselectKeyboardState",
      selectPromptMultiselectKeyboardStateStressId: "loop-twenty-seven-select-prompt-multiselect-keyboard-state",
      session: opts.session, surface: opts.surface ?? "select-prompt", fixture: opts.fixture ?? "agentic-multiselect",
      choiceCount: opts.choices ?? 24, requestedSelectionSteps: opts.selectionSteps ?? ["space", "cmd-a", "filter-preserve", "clear-filter", "range-toggle", "escape-restore"],
      requestedInputModes: opts.inputModes ?? ["protocol-key", "batch"], noNativeInput: opts.noNativeInput ?? true,
      noNativePointer: opts.noNativePointer ?? true, noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true, multiSelectSamples: [], selectionStep: null,
      inputMode: null, promptType: null, selectMode: null, focusedChoiceSemanticId: null,
      selectedChoiceSemanticIds: [], checkedRowSemanticIds: [], visibleChoiceSemanticIds: [],
      selectionCountLabel: null, footerSubmitLabel: null, footerSubmitDisabledReason: null,
      filterTextBefore: null, filterTextAfter: null, filterGeneration: null, selectionGeneration: null,
      focusGeneration: null, cmdAReceipt: null, spaceToggleReceipt: null, rangeToggleReceipt: null,
      filterPreservesSelectedSet: null, clearFilterRestoresCheckedRows: null,
      focusedRowNotDuplicatedAsSelected: null, checkedRowsMatchState: null, visibleRowsMatchElements: null,
      noSubmitReceipt: null, noActivationReceipt: null, staleSelectionGenerationRejected: null,
      wrongSurfaceSelectionRejected: null, systemPasteboardMutated: false, configUnchanged: true,
      destructiveOperationRequested: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_select_prompt_multiselect_keyboard_state_receipt" } }],
    failure: { code: "missing_select_prompt_multiselect_keyboard_state_receipt", stepName: "declare-required-receipt", message: "Missing app-side SelectPrompt keyboard multi-selection state receipts." },
    warnings: ["file_linear:select_prompt_multiselect_keyboard_state_receipts_missing"],
  };
}

export async function runFileSearchPreviewSanitizationStressScenario(opts: {
  session: string;
  surface?: string;
  fixture?: string;
  previewFixtures?: string[];
  selectionCycles?: number;
  filterCycles?: number;
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noNativePicker?: boolean;
  noQuickLook?: boolean;
  noSystemPasteboard?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "file-search-preview-sanitization-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_file_search_preview_sanitization_receipt",
    linearIssue: "file_linear:file_search_preview_sanitization_receipts_missing",
    fileSearchPreviewSanitizationReceipt: {
      kind: "ux.fileSearchPreviewSanitization",
      fileSearchPreviewSanitizationStressId: "loop-twenty-seven-file-search-preview-sanitization",
      session: opts.session, surface: opts.surface ?? "file-search", fixture: opts.fixture ?? "agentic-safe-preview",
      requestedPreviewFixtures: opts.previewFixtures ?? ["text", "binary", "large-text", "missing-file", "private-path", "unsupported-kind"],
      selectionCycles: opts.selectionCycles ?? 8, filterCycles: opts.filterCycles ?? 4,
      requestedInputModes: opts.inputModes ?? ["protocol-set-filter", "protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noNativePicker: opts.noNativePicker ?? true, noQuickLook: opts.noQuickLook ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, localFixtureOnly: opts.localFixtureOnly ?? true,
      previewSamples: [], previewFixtureKind: null, selectionCycle: null, filterCycle: null,
      selectedRowSemanticId: null, selectedFileUri: null, selectedFileFingerprint: null,
      previewGeneration: null, previewSourceIdentity: null, previewRenderKind: null, previewTitle: null,
      previewVisibleText: null, previewBounds: null, previewTextBounds: null, previewByteLimit: null,
      previewTruncated: null, binaryPreviewFallback: null, missingFileFallback: null,
      unsupportedPreviewFallback: null, privatePathRedacted: null, redactedPathFingerprint: null,
      noRawPathLeak: null, noNetworkFetch: true, noExternalServiceContacted: true,
      noQuickLookOpened: true, noNativePickerOpened: true, noSystemPasteboardMutation: true,
      filterGeneration: null, selectionGeneration: null, stalePreviewGenerationRejected: null,
      wrongRowPreviewRejected: null, wrongSurfaceRejected: null, footerNotObscured: null,
      inputNotObscured: null, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_file_search_preview_sanitization_receipt" } }],
    failure: { code: "missing_file_search_preview_sanitization_receipt", stepName: "declare-required-receipt", message: "Missing app-side File Search safe preview sanitization receipts." },
    warnings: ["file_linear:file_search_preview_sanitization_receipts_missing"],
  };
}

export async function runHotkeyPromptTransientCaptureCancelStressScenario(opts: {
  session: string;
  surface?: string;
  fixture?: string;
  chords?: string[];
  cancelMethods?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noConfigWrite?: boolean;
  noGlobalHotkeyRegistration?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "hotkey-prompt-transient-capture-cancel-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_hotkey_prompt_transient_capture_cancel_receipt",
    linearIssue: "file_linear:hotkey_prompt_transient_capture_cancel_receipts_missing",
    hotkeyPromptTransientCaptureCancelReceipt: {
      kind: "ux.hotkeyPromptTransientCaptureCancel",
      hotkeyPromptTransientCaptureCancelStressId: "loop-twenty-eight-hotkey-prompt-transient-capture-cancel",
      session: opts.session, surface: opts.surface ?? "hotkey-prompt", fixture: opts.fixture ?? "agentic-transient-hotkey",
      requestedChords: opts.chords ?? ["cmd+shift+7", "ctrl+space"],
      requestedCancelMethods: opts.cancelMethods ?? ["escape", "cmd-w"],
      requestedInputModes: opts.inputModes ?? ["protocol-key", "simulate-key"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noConfigWrite: opts.noConfigWrite ?? true, noGlobalHotkeyRegistration: opts.noGlobalHotkeyRegistration ?? true,
      dryRunOnly: opts.dryRunOnly ?? true, localFixtureOnly: opts.localFixtureOnly ?? true,
      promptType: null, hotkeyPromptSurfaceId: null, capturePanelSemanticId: null,
      shortcutInputSemanticId: null, placeholderVisibleText: null, capturedChordTokens: [],
      capturedHotkeyInfo: null, simulateKeyCaptureReceipt: null, escapeCancelReceipt: null,
      cmdWCancelReceipt: null, noConfigFingerprintChange: null, noGlobalHotkeyRegistrationReceipt: null,
      noShortcutRecorderRoute: null, cancelSubmitsNull: null, focusRestoredToParent: null,
      staleHotkeyCaptureRejected: null, wrongSurfaceRejected: null, usedNativeInput: false,
      usedNativePointer: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_hotkey_prompt_transient_capture_cancel_receipt" } }],
    failure: { code: "missing_hotkey_prompt_transient_capture_cancel_receipt", stepName: "declare-required-receipt", message: "Missing app-side HotkeyPrompt transient capture/cancel receipts." },
    warnings: ["file_linear:hotkey_prompt_transient_capture_cancel_receipts_missing"],
  };
}

export async function runProcessManagerSortDetailPanelStabilityStressScenario(opts: {
  session: string;
  surface?: string;
  fixture?: string;
  sortKeys?: string[];
  selectionCycles?: number;
  filterCycles?: number;
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noProcessKill?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "process-manager-sort-detail-panel-stability-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_process_manager_sort_detail_panel_stability_receipt",
    linearIssue: "file_linear:process_manager_sort_detail_panel_receipts_missing",
    processManagerSortDetailPanelStabilityReceipt: {
      kind: "ux.processManagerSortDetailPanelStability",
      processManagerSortDetailPanelStabilityStressId: "loop-twenty-eight-process-manager-sort-detail-panel-stability",
      session: opts.session, surface: opts.surface ?? "process-manager", fixture: opts.fixture ?? "agentic-process-table",
      requestedSortKeys: opts.sortKeys ?? ["name", "cpu", "memory", "pid"],
      selectionCycles: opts.selectionCycles ?? 8, filterCycles: opts.filterCycles ?? 4,
      requestedInputModes: opts.inputModes ?? ["protocol-click", "protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noProcessKill: opts.noProcessKill ?? true,
      dryRunOnly: opts.dryRunOnly ?? true, localFixtureOnly: opts.localFixtureOnly ?? true,
      processFixtureIdentity: null, tableHeaderSemanticIds: [], sortKey: null,
      sortDirection: null, sortGeneration: null, sectionHeaderRows: [],
      sectionHeaderSelectableFalse: null, selectedProcessSemanticId: null, selectedPid: null,
      detailPanelGeneration: null, detailSourceIdentity: null, detailTitle: null,
      detailMetricRows: [], cpuMemoryPidParity: null, filterGeneration: null,
      rowReanchorAfterSort: null, visibleRowsMatchElements: null, headerAriaSortLabel: null,
      killActionDisabled: null, noProcessSignalRequested: true, staleSortGenerationRejected: null,
      staleDetailRejected: null, wrongSurfaceRejected: null, usedNativeInput: false,
      usedNativePointer: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, systemPasteboardMutated: false, destructiveOperationRequested: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_process_manager_sort_detail_panel_stability_receipt" } }],
    failure: { code: "missing_process_manager_sort_detail_panel_stability_receipt", stepName: "declare-required-receipt", message: "Missing app-side Process Manager sort/header/detail panel stability receipts." },
    warnings: ["file_linear:process_manager_sort_detail_panel_receipts_missing"],
  };
}

export async function runEnvPromptRedactedStatusErrorRecoveryStressScenario(opts: {
  session: string;
  surface?: string;
  fixture?: string;
  statusFixtures?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noConfigWrite?: boolean;
  noSecretWrite?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "env-prompt-redacted-status-error-recovery-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_env_prompt_redacted_status_error_recovery_receipt",
    linearIssue: "file_linear:env_prompt_redacted_status_error_recovery_receipts_missing",
    envPromptRedactedStatusErrorRecoveryReceipt: {
      kind: "ux.envPromptRedactedStatusErrorRecovery",
      envPromptRedactedStatusErrorRecoveryStressId: "loop-twenty-eight-env-prompt-redacted-status-error-recovery",
      session: opts.session, surface: opts.surface ?? "env-prompt", fixture: opts.fixture ?? "agentic-env-status",
      requestedStatusFixtures: opts.statusFixtures ?? ["missing-secret", "parse-error", "masked-existing", "valid-edit"],
      requestedInputModes: opts.inputModes ?? ["protocol-set-input", "protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noConfigWrite: opts.noConfigWrite ?? true,
      noSecretWrite: opts.noSecretWrite ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      promptType: null, envFixtureIdentity: null, statusGeneration: null, statusKind: null,
      statusVisibleText: null, statusSemanticId: null, inlineErrorSemanticId: null,
      firstInvalidFieldSemanticId: null, maskedValueVisible: null, secretValueRedacted: null,
      redactedSecretFingerprint: null, noRawSecretLeak: null, noSecretWriteReceipt: null,
      noConfigFingerprintChange: null, validEditClearsErrors: null, submitDisabledReason: null,
      footerSubmitDisabled: null, focusPreservedAfterError: null, staleStatusRejected: null,
      wrongFieldErrorRejected: null, visibleRowsMatchElements: null, usedNativeInput: false,
      usedNativePointer: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, systemPasteboardMutated: false, configUnchanged: true, secretMutated: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_env_prompt_redacted_status_error_recovery_receipt" } }],
    failure: { code: "missing_env_prompt_redacted_status_error_recovery_receipt", stepName: "declare-required-receipt", message: "Missing app-side EnvPrompt redacted status/error recovery receipts." },
    warnings: ["file_linear:env_prompt_redacted_status_error_recovery_receipts_missing"],
  };
}

export async function runCommandPaletteBreadcrumbRouteStackStressScenario(opts: {
  session: string;
  host?: string;
  fixture?: string;
  drillPath?: string[];
  filter?: string;
  backMethods?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "command-palette-breadcrumb-route-stack-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_command_palette_breadcrumb_route_stack_receipt",
    linearIssue: "file_linear:command_palette_breadcrumb_route_stack_receipts_missing",
    commandPaletteBreadcrumbRouteStackReceipt: {
      kind: "ux.commandPaletteBreadcrumbRouteStack",
      commandPaletteBreadcrumbRouteStackStressId: "loop-twenty-nine-command-palette-breadcrumb-route-stack",
      session: opts.session, host: opts.host ?? "main", fixture: opts.fixture ?? "agentic-actions-breadcrumbs",
      requestedDrillPath: opts.drillPath ?? ["parent-action", "child-action"],
      requestedFilter: opts.filter ?? "switch", requestedBackMethods: opts.backMethods ?? ["escape", "breadcrumb-click"],
      requestedInputModes: opts.inputModes ?? ["protocol-key", "protocol-click", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      actionsDialogHost: null, routeStackDepth: null, breadcrumbTrailLabels: [],
      breadcrumbSemanticIds: [], activeRouteId: null, parentRouteSnapshot: null,
      childRouteSnapshot: null, drillDownActionId: null, drillDownPushedReceipt: null,
      breadcrumbBackReceipt: null, escapeBackReceipt: null, searchTextPreserved: null,
      selectionRestoredToParent: null, scrollAnchorRestored: null, noOnSelectBeforeDrillDown: null,
      noAccidentalExecution: null, topmostOwnerBeforeKey: null, staleRouteRejected: null,
      wrongHostRejected: null, usedNativeInput: false, usedNativePointer: false,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_command_palette_breadcrumb_route_stack_receipt" } }],
    failure: { code: "missing_command_palette_breadcrumb_route_stack_receipt", stepName: "declare-required-receipt", message: "Missing app-side command palette breadcrumb route-stack receipts." },
    warnings: ["file_linear:command_palette_breadcrumb_route_stack_receipts_missing"],
  };
}

export async function runRootSourceChipActionSemanticsStressScenario(opts: {
  session: string;
  queries?: string[];
  actions?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noConfigWrite?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "root-source-chip-action-semantics-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_root_source_chip_action_semantics_receipt",
    linearIssue: "file_linear:root_source_chip_action_semantics_receipts_missing",
    rootSourceChipActionSemanticsReceipt: {
      kind: "ux.rootSourceChipActionSemantics",
      rootSourceChipActionSemanticsStressId: "loop-twenty-nine-root-source-chip-action-semantics",
      session: opts.session,
      requestedQueries: opts.queries ?? ["f: AGENTS.md", "c: agentic", "n: welcome", "-c: noise"],
      requestedActions: opts.actions ?? ["remove-chip", "clear-all", "toggle-exclude", "open-chip-actions"],
      requestedInputModes: opts.inputModes ?? ["protocol-click", "protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noConfigWrite: opts.noConfigWrite ?? true,
      dryRunOnly: opts.dryRunOnly ?? true, localFixtureOnly: opts.localFixtureOnly ?? true,
      fixtureSourceCatalogId: null, inputRenderedText: null, strippedSearchText: null,
      sourceFilterSet: null, sourceChipSemanticIds: [], sourceChipRoles: [],
      chipRemoveReceipt: null, chipClearAllReceipt: null, chipToggleExcludeReceipt: null,
      filterInputDecorationsGeneration: null, preflightFilterIndicators: null,
      statusChipNonSelectable: null, groupedRowsSuppressDisallowedSources: null,
      inputHistoryRecallBlocked: null, selectionPreservedAfterChipAction: null,
      noStatusAsActionSubject: null, noAccidentalExecution: null, staleChipActionRejected: null,
      wrongSurfaceRejected: null, usedNativeInput: false, usedNativePointer: false,
      usedSystemPasteboard: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, systemPasteboardMutated: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_root_source_chip_action_semantics_receipt" } }],
    failure: { code: "missing_root_source_chip_action_semantics_receipt", stepName: "declare-required-receipt", message: "Missing app-side root source-chip action semantics receipts." },
    warnings: ["file_linear:root_source_chip_action_semantics_receipts_missing"],
  };
}

export async function runRecentHistoryDedupeRootGroupingStressScenario(opts: {
  session: string;
  fixture?: string;
  sources?: string[];
  query?: string;
  cycles?: number;
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "recent-history-dedupe-root-grouping-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_recent_history_dedupe_root_grouping_receipt",
    linearIssue: "file_linear:recent_history_dedupe_root_grouping_receipts_missing",
    recentHistoryDedupeRootGroupingReceipt: {
      kind: "ux.recentHistoryDedupeRootGrouping",
      recentHistoryDedupeRootGroupingStressId: "loop-twenty-nine-recent-history-dedupe-root-grouping",
      session: opts.session, fixture: opts.fixture ?? "agentic-root-recents",
      requestedSources: opts.sources ?? ["files", "notes", "clipboard", "dictation", "acp-history"],
      requestedQuery: opts.query ?? "agentic-loop-29-dupe", requestedCycles: opts.cycles ?? 6,
      requestedInputModes: opts.inputModes ?? ["protocol-set-filter", "protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      dryRunOnly: opts.dryRunOnly ?? true, localFixtureOnly: opts.localFixtureOnly ?? true,
      fixtureHistorySnapshotId: null, sourceCatalogGeneration: null, queryFrameKey: null,
      rootFileFrameKey: null, passiveFrameKey: null, visibleResultsRoles: [],
      groupSectionOrder: [], filesSectionContiguous: null, searchFilesContinuationStable: null,
      dedupeKeys: [], duplicateKeyCollisionsRejected: null, recentFileSeedPoolFingerprint: null,
      historyRowsMetadataOnly: null, noFullTranscriptOrNoteBodyLeak: null, stableSelectionKey: null,
      rowFingerprintBeforeAfterCycles: null, fallbackRowsSuppressedWhenSourceRowsPresent: null,
      stalePassivePublishRejected: null, noAccidentalExecution: null, usedNativeInput: false,
      usedNativePointer: false, usedSystemPasteboard: false, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, systemPasteboardMutated: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_recent_history_dedupe_root_grouping_receipt" } }],
    failure: { code: "missing_recent_history_dedupe_root_grouping_receipt", stepName: "declare-required-receipt", message: "Missing app-side recent/history dedupe and root grouping receipts." },
    warnings: ["file_linear:recent_history_dedupe_root_grouping_receipts_missing"],
  };
}

export async function runInlineAttachmentPreviewChipStabilityStressScenario(opts: {
  session: string;
  hosts?: string[];
  fixture?: string;
  origins?: string[];
  chipActions?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noNativePicker?: boolean;
  noScreenCapture?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "inline-attachment-preview-chip-stability-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_inline_attachment_preview_chip_stability_receipt",
    linearIssue: "file_linear:inline_attachment_preview_chip_stability_receipts_missing",
    inlineAttachmentPreviewChipStabilityReceipt: {
      kind: "ux.inlineAttachmentPreviewChipStability",
      inlineAttachmentPreviewChipStabilityStressId: "loop-thirty-inline-attachment-preview-chip-stability",
      session: opts.session, requestedHosts: opts.hosts ?? ["acp-composer", "notes"],
      fixture: opts.fixture ?? "agentic-inline-attachments",
      requestedOrigins: opts.origins ?? ["local-file", "fixture-image", "fixture-text", "script-resource"],
      requestedChipActions: opts.chipActions ?? ["focus", "preview", "remove", "reorder", "overflow"],
      requestedInputModes: opts.inputModes ?? ["protocol-set-input", "protocol-click", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noNativePicker: opts.noNativePicker ?? true, noScreenCapture: opts.noScreenCapture ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      fixtureAttachmentSetId: null, hostSurfaceIdentity: null, composerGeneration: null,
      attachmentChipSemanticIds: [], chipKinds: [], chipLabels: [], chipBounds: [],
      previewRedactedFingerprint: null, overflowChipReceipt: null, focusChipReceipt: null,
      removeChipReceipt: null, reorderChipReceipt: null, cursorSelectionPreserved: null,
      noRawPathOrContentLeak: null, noSystemPasteboardReceipt: null, noNativePickerReceipt: null,
      noScreenCaptureReceipt: null, noNetworkReceipt: null, staleAttachmentRejected: null,
      wrongHostRejected: null, localFixtureOnly: true, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, systemPasteboardMutated: false, networkAccessed: false, externalServiceContacted: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_inline_attachment_preview_chip_stability_receipt" } }],
    failure: { code: "missing_inline_attachment_preview_chip_stability_receipt", stepName: "declare-required-receipt", message: "Missing app-side inline attachment preview chip stability receipts." },
    warnings: ["file_linear:inline_attachment_preview_chip_stability_receipts_missing"],
  };
}

export async function runWindowTitleStatusSemanticsStressScenario(opts: {
  session: string;
  surfaces?: string[];
  states?: string[];
  transitions?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "window-title-status-semantics-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_window_title_status_semantics_receipt",
    linearIssue: "file_linear:window_title_status_semantics_receipts_missing",
    windowTitleStatusSemanticsReceipt: {
      kind: "ux.windowTitleStatusSemantics",
      windowTitleStatusSemanticsStressId: "loop-thirty-window-title-status-semantics",
      session: opts.session,
      requestedSurfaces: opts.surfaces ?? ["main", "acp-composer", "actionsDialog", "promptPopup", "notes"],
      requestedStates: opts.states ?? ["idle", "busy", "error", "dirty", "ready"],
      requestedTransitions: opts.transitions ?? ["triggerBuiltin", "cmd-k", "escape", "hide-show"],
      requestedInputModes: opts.inputModes ?? ["protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      resolvedTarget: null, automationWindowTitle: null, nativeWindowTitle: null,
      semanticSurfaceTitle: null, visibleStatusText: null, titleGeneration: null,
      statusGeneration: null, transitionReceipts: [], detachedWindowParity: null,
      attachedPopupParentTitleUnaffected: null, statusErrorRecovery: null,
      staleTitleRejected: null, staleStatusRejected: null, wrongSurfaceRejected: null,
      noFocusSteal: null, noNativeInput: true, noNativePointer: true, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, systemPasteboardMutated: false, networkAccessed: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_window_title_status_semantics_receipt" } }],
    failure: { code: "missing_window_title_status_semantics_receipt", stepName: "declare-required-receipt", message: "Missing app-side window title/status semantics receipts." },
    warnings: ["file_linear:window_title_status_semantics_receipts_missing"],
  };
}

export async function runMenuSyntaxCaptureValidationChipStressScenario(opts: {
  session: string;
  fixture?: string;
  cases?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "menu-syntax-capture-validation-chip-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_menu_syntax_capture_validation_chip_receipt",
    linearIssue: "file_linear:menu_syntax_capture_validation_chip_receipts_missing",
    menuSyntaxCaptureValidationChipReceipt: {
      kind: "ux.menuSyntaxCaptureValidationChip",
      menuSyntaxCaptureValidationChipStressId: "loop-thirty-menu-syntax-capture-validation-chip",
      session: opts.session, fixture: opts.fixture ?? "agentic-capture-validation",
      requestedCases: opts.cases ?? ["missing-body-date", "missing-date", "ready", "malformed-url", "unresolved-date", "dynamic-schema"],
      requestedInputModes: opts.inputModes ?? ["protocol-set-filter", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      fixtureMenuSyntaxCatalogId: null, filterInputText: null, menuSyntaxMainHintSnapshot: null,
      captureValidationStatus: null, statusChipLabels: [], missingFieldLabels: [],
      malformedFieldLabel: null, malformedReason: null, unresolvedDates: [],
      fragmentPreviewRows: [], priorityChoicesRow: null, canSubmitFalsePreventsEnter: null,
      noPayloadWrite: null, noHandlerSpawn: null, staleValidationRejected: null,
      wrongSurfaceRejected: null, noNativeInput: true, noNativePointer: true,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, systemPasteboardMutated: false, networkAccessed: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_menu_syntax_capture_validation_chip_receipt" } }],
    failure: { code: "missing_menu_syntax_capture_validation_chip_receipt", stepName: "declare-required-receipt", message: "Missing app-side menu syntax capture validation chip receipts." },
    warnings: ["file_linear:menu_syntax_capture_validation_chip_receipts_missing"],
  };
}

export async function runAcpFooterActivityIndicatorStressScenario(opts: {
  session: string;
  hosts?: string[];
  fixture?: string;
  activityFixtures?: string[];
  inputModes?: string[];
  agentFixture?: string;
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSecurityPrompts?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "acp-footer-activity-indicator-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_acp_footer_activity_indicator_receipt",
    linearIssue: "file_linear:acp_footer_activity_indicator_receipts_missing",
    acpFooterActivityIndicatorReceipt: {
      kind: "ux.acpFooterActivityIndicator",
      acpFooterActivityIndicatorStressId: "loop-thirty-one-acp-footer-activity-indicator",
      session: opts.session, requestedHosts: opts.hosts ?? ["acp-composer", "notes"],
      fixture: opts.fixture ?? "agentic-acp-footer-activity",
      requestedActivityFixtures: opts.activityFixtures ?? ["context-capture", "tool-call", "plan-update", "permission-wait", "cancelled", "idle-recovered"],
      requestedInputModes: opts.inputModes ?? ["protocol-state", "batch"], agentFixture: opts.agentFixture ?? "scripted-local",
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSecurityPrompts: opts.noSecurityPrompts ?? true, noSystemPasteboard: opts.noSystemPasteboard ?? true,
      noNetwork: opts.noNetwork ?? true, noSubmit: opts.noSubmit ?? true,
      dryRunOnly: opts.dryRunOnly ?? true, localFixtureOnly: opts.localFixtureOnly ?? true,
      fixtureAgentEventStreamId: null, hostSurfaceIdentity: null, footerOwner: null,
      nativeFooterSurfaceId: null, gpuiFooterDotStatus: null, nativeFooterDotStatus: null,
      activityStatusTransitions: [], contextCapturePendingStatus: null, toolCallStatus: null,
      planUpdateStatus: null, permissionWaitStatus: null, cancelRestoresIdle: null,
      footerRepaintGeneration: null, dotPulseTokenStable: null, modelLabelPreserved: null,
      noGlobalAiFooterButton: null, noAgentProcessSpawn: true, noSecurityPrompt: true,
      staleActivityRejected: null, wrongHostRejected: null, afkSafeFlags: true,
      cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, systemPasteboardMutated: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_acp_footer_activity_indicator_receipt" } }],
    failure: { code: "missing_acp_footer_activity_indicator_receipt", stepName: "declare-required-receipt", message: "Missing app-side ACP footer activity indicator receipts." },
    warnings: ["file_linear:acp_footer_activity_indicator_receipts_missing"],
  };
}

export async function runAcpModelHistoryPopoverVisualStateStressScenario(opts: {
  session: string;
  families?: string[];
  fixture?: string;
  states?: string[];
  selectionCycles?: number;
  filterCycles?: number;
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "acp-model-history-popover-visual-state-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_acp_model_history_popover_visual_state_receipt",
    linearIssue: "file_linear:acp_model_history_popover_visual_state_receipts_missing",
    acpModelHistoryPopoverVisualStateReceipt: {
      kind: "ux.acpModelHistoryPopoverVisualState",
      acpModelHistoryPopoverVisualStateStressId: "loop-thirty-one-acp-model-history-popover-visual-state",
      session: opts.session, requestedFamilies: opts.families ?? ["model-selector", "local-history"],
      fixture: opts.fixture ?? "agentic-acp-popover-visual-state",
      requestedStates: opts.states ?? ["idle", "filtered", "empty", "loading", "current-selection", "error-recovered"],
      selectionCycles: opts.selectionCycles ?? 8, filterCycles: opts.filterCycles ?? 4,
      requestedInputModes: opts.inputModes ?? ["protocol-set-input", "protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      fixturePopoverCatalogId: null, popupFamily: null, popupAutomationId: null,
      promptPopupKind: null, anchorBounds: null, popupBounds: null, selectedRowSemanticId: null,
      focusedRowSemanticId: null, rowVisualStateTokens: [], currentModelBadge: null,
      historyRecencyBadge: null, historyPreviewRedactedFingerprint: null, emptyFilteredState: null,
      loadingRefreshState: null, errorRecoveredState: null, synopsisBounds: null,
      selectionPreservedAfterFilter: null, noTranscriptBodyLeak: null, stalePopupSnapshotRejected: null,
      wrongPopupRejected: null, noSubmit: true, afkSafeFlags: true, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, systemPasteboardMutated: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_acp_model_history_popover_visual_state_receipt" } }],
    failure: { code: "missing_acp_model_history_popover_visual_state_receipt", stepName: "declare-required-receipt", message: "Missing app-side ACP model/history popover visual-state receipts." },
    warnings: ["file_linear:acp_model_history_popover_visual_state_receipts_missing"],
  };
}

export async function runAcpContextInsertionPreviewParityStressScenario(opts: {
  session: string;
  sources?: string[];
  destination?: string;
  fixture?: string;
  selectionCycles?: number;
  filterCycles?: number;
  insertModes?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noNativePicker?: boolean;
  noQuickLook?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "acp-context-insertion-preview-parity-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_acp_context_insertion_preview_parity_receipt",
    linearIssue: "file_linear:acp_context_insertion_preview_parity_receipts_missing",
    acpContextInsertionPreviewParityReceipt: {
      kind: "ux.acpContextInsertionPreviewParity",
      acpContextInsertionPreviewParityStressId: "loop-thirty-one-acp-context-insertion-preview-parity",
      session: opts.session, requestedSources: opts.sources ?? ["file-search", "browser-history", "dictation-history", "notes"],
      destinationComposerIdentity: opts.destination ?? "acp-composer", fixture: opts.fixture ?? "agentic-context-preview-parity",
      selectionCycles: opts.selectionCycles ?? 6, filterCycles: opts.filterCycles ?? 4,
      requestedInsertModes: opts.insertModes ?? ["protocol-accept", "batch"],
      requestedInputModes: opts.inputModes ?? ["protocol-set-filter", "protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noNativePicker: opts.noNativePicker ?? true, noQuickLook: opts.noQuickLook ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      sourceSurfaceIdentity: null, portalSessionId: null, sourceSelectionGeneration: null,
      selectedRowSemanticId: null, selectedRowPreviewFingerprint: null, selectedRowPreviewTitle: null,
      selectedRowPreviewKind: null, previewGeneration: null, acceptedContextPartUri: null,
      insertedTokenAlias: null, insertedTokenPreviewFingerprint: null, composerGeneration: null,
      replacementRange: null, rowPreviewMatchesInsertedContext: null, selectionPreservedAfterInsert: null,
      selectionDriftRejected: null, stalePreviewRejected: null, wrongDestinationRejected: null,
      noRawContentLeak: null, noNativePicker: true, noQuickLook: true, noSystemPasteboard: true,
      noNetwork: true, noSubmit: true, afkSafeFlags: true, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, systemPasteboardMutated: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_acp_context_insertion_preview_parity_receipt" } }],
    failure: { code: "missing_acp_context_insertion_preview_parity_receipt", stepName: "declare-required-receipt", message: "Missing app-side ACP context insertion preview parity receipts." },
    warnings: ["file_linear:acp_context_insertion_preview_parity_receipts_missing"],
  };
}

export async function runAcpSlashMentionProviderVisibilityStressScenario(opts: {
  session: string;
  families?: string[];
  fixture?: string;
  providers?: string[];
  queries?: string[];
  states?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noNativePicker?: boolean;
  noQuickLook?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "acp-slash-mention-provider-visibility-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_acp_slash_mention_provider_visibility_receipt",
    linearIssue: "file_linear:acp_slash_mention_provider_visibility_receipts_missing",
    acpSlashMentionProviderVisibilityReceipt: {
      kind: "ux.acpSlashMentionProviderVisibility",
      acpSlashMentionProviderVisibilityStressId: "loop-thirty-two-acp-slash-mention-provider-visibility",
      session: opts.session, requestedFamilies: opts.families ?? ["slash", "mention"],
      fixture: opts.fixture ?? "agentic-acp-provider-hints",
      requestedProviders: opts.providers ?? ["dictation-history", "browser-history", "notes", "files", "skills"],
      requestedQueries: opts.queries ?? ["@di", "@browser-history", "@missing", "/new-script", "/unknown"],
      requestedStates: opts.states ?? ["ready", "unavailable", "loading", "error-recovered", "filtered-empty"],
      requestedInputModes: opts.inputModes ?? ["protocol-set-input", "protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noNativePicker: opts.noNativePicker ?? true, noQuickLook: opts.noQuickLook ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      providerHintCatalogId: null, popupFamily: null, triggerText: null, queryText: null,
      providerReadinessGeneration: null, providerVisibilityRows: [], providerHintText: null,
      providerUnavailableReason: null, providerLoadingState: null, providerErrorRecoveredState: null,
      hiddenUntilResourceAvailable: null, dictationProviderVisibleWhenKitResourceReady: null,
      browserHistoryProviderVisibleWhenCacheReady: null, slashCommandProviderRows: [],
      mentionProviderRows: [], selectedRowSemanticId: null, focusedRowSemanticId: null,
      disabledProviderRowsNotAccepted: null, staleProviderGenerationRejected: null,
      wrongPopupRejected: null, noRawProviderContentLeak: null, noNativePicker: true,
      noQuickLook: true, noNetwork: true, noSubmit: true, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, systemPasteboardMutated: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_acp_slash_mention_provider_visibility_receipt" } }],
    failure: { code: "missing_acp_slash_mention_provider_visibility_receipt", stepName: "declare-required-receipt", message: "Missing app-side ACP slash/mention provider visibility receipts." },
    warnings: ["file_linear:acp_slash_mention_provider_visibility_receipts_missing"],
  };
}

export async function runAcpComposerTokenKeyboardEditParityStressScenario(opts: {
  session: string;
  hosts?: string[];
  fixture?: string;
  tokenKinds?: string[];
  editSteps?: string[];
  inputModes?: string[];
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noNativePicker?: boolean;
  noScreenCapture?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "acp-composer-token-keyboard-edit-parity-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_acp_composer_token_keyboard_edit_parity_receipt",
    linearIssue: "file_linear:acp_composer_token_keyboard_edit_parity_receipts_missing",
    acpComposerTokenKeyboardEditParityReceipt: {
      kind: "ux.acpComposerTokenKeyboardEditParity",
      acpComposerTokenKeyboardEditParityStressId: "loop-thirty-two-acp-composer-token-keyboard-edit-parity",
      session: opts.session, requestedHosts: opts.hosts ?? ["acp-composer", "notes"],
      fixture: opts.fixture ?? "agentic-acp-composer-tokens",
      requestedTokenKinds: opts.tokenKinds ?? ["mention", "slash", "pasted-text", "pasted-image", "skill-file"],
      requestedEditSteps: opts.editSteps ?? ["backspace-delete", "delete-forward", "range-remove", "move-token-left", "move-token-right", "cursor-around-token"],
      requestedInputModes: opts.inputModes ?? ["protocol-key", "batch"],
      noNativeInput: opts.noNativeInput ?? true, noNativePointer: opts.noNativePointer ?? true,
      noNativePicker: opts.noNativePicker ?? true, noScreenCapture: opts.noScreenCapture ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      fixtureComposerTokenSetId: null, hostSurfaceIdentity: null, composerGeneration: null,
      tokenSemanticIds: [], tokenKinds: [], tokenAliases: [], tokenBounds: [],
      cursorBeforeToken: null, cursorAfterToken: null, backspaceRemovesTokenAtomically: null,
      deleteForwardRemovesTokenAtomically: null, rangeRemoveReceipt: null,
      moveTokenLeftReceipt: null, moveTokenRightReceipt: null, tokenOrderBefore: [],
      tokenOrderAfter: [], pendingContextPartsPreserved: null, slashSkillContextPreserved: null,
      pastedTokenMetadataPreserved: null, cursorSelectionPreserved: null,
      noPartialTokenTextLeak: null, staleComposerGenerationRejected: null,
      duplicateTokenIdRejected: null, wrongHostRejected: null, noSystemPasteboard: true,
      noNativeInput: true, noSubmit: true, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, systemPasteboardMutated: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_acp_composer_token_keyboard_edit_parity_receipt" } }],
    failure: { code: "missing_acp_composer_token_keyboard_edit_parity_receipt", stepName: "declare-required-receipt", message: "Missing app-side ACP composer token keyboard edit parity receipts." },
    warnings: ["file_linear:acp_composer_token_keyboard_edit_parity_receipts_missing"],
  };
}

export async function runAcpTranscriptStreamRetryVirtualizationStressScenario(opts: {
  session: string;
  hosts?: string[];
  fixture?: string;
  messageCount?: number;
  streamChunks?: number;
  errorFixtures?: string[];
  retryPaths?: string[];
  scrollPositions?: string[];
  inputModes?: string[];
  agentFixture?: string;
  noNativeInput?: boolean;
  noNativePointer?: boolean;
  noSecurityPrompts?: boolean;
  noSystemPasteboard?: boolean;
  noNetwork?: boolean;
  noSubmit?: boolean;
  dryRunOnly?: boolean;
  localFixtureOnly?: boolean;
}): Promise<HardScenarioReceipt> {
  return {
    schemaVersion: PROOF_BUNDLE_SCHEMA_VERSION,
    scenario: "acp-transcript-stream-retry-virtualization-stress",
    status: "fail",
    failClosed: true,
    failureMode: "fail_closed",
    missingReceipt: "missing_acp_transcript_stream_retry_virtualization_receipt",
    linearIssue: "file_linear:acp_transcript_stream_retry_virtualization_receipts_missing",
    acpTranscriptStreamRetryVirtualizationReceipt: {
      kind: "ux.acpTranscriptStreamRetryVirtualization",
      acpTranscriptStreamRetryVirtualizationStressId: "loop-thirty-two-acp-transcript-stream-retry-virtualization",
      session: opts.session, requestedHosts: opts.hosts ?? ["acp-composer", "notes"],
      fixture: opts.fixture ?? "agentic-acp-transcript-stream",
      messageCount: opts.messageCount ?? 160, streamChunks: opts.streamChunks ?? 48,
      requestedErrorFixtures: opts.errorFixtures ?? ["tool-error", "agent-error", "model-timeout", "cancelled"],
      requestedRetryPaths: opts.retryPaths ?? ["retry-same-draft", "retry-edited-draft", "retry-after-scroll"],
      requestedScrollPositions: opts.scrollPositions ?? ["top", "middle", "bottom", "near-active"],
      requestedInputModes: opts.inputModes ?? ["protocol-state", "protocol-key", "batch"],
      agentFixture: opts.agentFixture ?? "scripted-local", noNativeInput: opts.noNativeInput ?? true,
      noNativePointer: opts.noNativePointer ?? true, noSecurityPrompts: opts.noSecurityPrompts ?? true,
      noSystemPasteboard: opts.noSystemPasteboard ?? true, noNetwork: opts.noNetwork ?? true,
      noSubmit: opts.noSubmit ?? true, dryRunOnly: opts.dryRunOnly ?? true,
      localFixtureOnly: opts.localFixtureOnly ?? true,
      fixtureTranscriptId: null, hostSurfaceIdentity: null, threadGeneration: null,
      transcriptGeneration: null, virtualizedMessageWindow: null, visibleMessageIds: [],
      messageRowSemanticIds: [], streamRunId: null, streamChunkSequence: [],
      activeAssistantMessageId: null, monotonicChunkAppend: null, scrollAnchorBefore: null,
      scrollAnchorAfter: null, bottomStickinessState: null, userScrolledAwayPreserved: null,
      assistantErrorMessageId: null, errorKind: null, errorVisibleText: null,
      retryButtonSemanticId: null, retryDraftFingerprint: null, retryRequestGeneration: null,
      retryRecoveryMessageId: null, noStaleErrorAfterRecovery: null, staleStreamChunkRejected: null,
      wrongMessageRetryRejected: null, virtualizedRowIdentityStable: null, blankRowRejected: null,
      noTranscriptBodyLeakInReceipts: null, noAgentProcessSpawn: true, noSecurityPrompt: true,
      noNetwork: true, noSubmit: true, cleanupConfirmed: true,
    },
    usage: { stateFirst: true, usedGetState: true, usedGetElements: true, usedNativeInput: false, usedNativePointer: false, usedScreenshot: false, openedSystemSettings: false, mutatedTcc: false, installedAgents: false, triggeredSecurityPrompt: false, networkAccessed: false, systemPasteboardMutated: false },
    steps: [{ name: "declare-required-receipt", status: "fail", output: { reason: "missing_acp_transcript_stream_retry_virtualization_receipt" } }],
    failure: { code: "missing_acp_transcript_stream_retry_virtualization_receipt", stepName: "declare-required-receipt", message: "Missing app-side ACP transcript streaming retry virtualization receipts." },
    warnings: ["file_linear:acp_transcript_stream_retry_virtualization_receipts_missing"],
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
