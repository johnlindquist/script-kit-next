#!/usr/bin/env bun
/**
 * scripts/agentic/verify-shot.ts
 *
 * ACP proof bundle: state receipt + test probe + screenshot + vision prompts.
 *
 * The reliable ACP verification order is:
 *   1. State receipt (getAcpState) — machine-readable proof
 *   2. Probe receipt (getAcpTestProbe) — key-route / picker-acceptance telemetry
 *   3. Screenshot capture — visual proof (secondary confirmation)
 *   4. Vision checks — structured prompts for external image readers
 *   Note: actual image-read / pixel inspection is NOT performed automatically.
 *
 * Usage:
 *   bun scripts/agentic/verify-shot.ts --session NAME [options]
 *
 * Options:
 *   --session NAME              Session name (default: "default")
 *   --label LABEL               Human-readable step label (default: "verify")
 *   --out PATH                  Screenshot output path (default: test-screenshots/<label>-<ts>.png)
 *   --acp-status STATUS         Assert ACP status equals this value
 *   --acp-picker-open           Assert picker is open
 *   --acp-picker-closed         Assert picker is closed
 *   --acp-input-contains STR    Assert input text contains substring
 *   --acp-input-match STR       Assert input text equals exactly
 *   --acp-cursor-at N           Assert cursor is at character index N
 *   --acp-item-accepted         Assert lastAcceptedItem is non-null
 *   --acp-accepted-label STR    Assert lastAcceptedItem.label equals STR
 *   --acp-accepted-trigger STR  Assert lastAcceptedItem.trigger equals STR (@ or /)
 *   --acp-accepted-via KEY      Assert probe acceptedItems[last].acceptedViaKey equals KEY (enter|tab)
 *   --acp-cursor-after-accepted N  Assert probe acceptedItems[last].cursorAfter equals N
 *   --acp-context-ready         Assert contextReady is true
 *   --acp-no-selection          Assert hasSelection is false
 *   --acp-has-selection         Assert hasSelection is true
 *   --acp-no-permission         Assert hasPendingPermission is false
 *   --acp-has-permission        Assert hasPendingPermission is true
 *   --acp-visible-start N       Assert inputLayout.visibleStart equals N
 *   --acp-visible-end N         Assert inputLayout.visibleEnd equals N
 *   --acp-cursor-in-window N    Assert inputLayout.cursorInWindow equals N
 *   --acp-setup-visible         Assert setup card is present (status == "setup")
 *   --acp-setup-reason CODE     Assert setup.reasonCode equals CODE
 *   --acp-setup-primary-action A  Assert setup.primaryAction equals A
 *   --acp-setup-selected-agent ID Assert setup.selectedAgentId equals ID
 *   --acp-setup-agent-picker-open Assert setup.agentPickerOpen is true
 *   --probe-tail N              Number of probe events to request (default: 20)
 *   --vision                    Emit vision checks with mustReview prompts and requiresVisionReview
 *   --emit-vision-crops         Alias for --vision
 *   --skip-screenshot           Only run state assertions, skip capture
 *   --skip-state                Only capture screenshot, skip ACP state query
 *   --skip-probe                Skip ACP test probe query
 *   --target-json JSON           ACP window target for getAcpState/getAcpTestProbe RPCs
 *                               (same AutomationWindowTarget shape as the Rust protocol)
 *   --capture-window-id N        Exact native window ID for screencapture
 *                               (the inspected osWindowId, not automationWindowId)
 *   --request-id ID             Request ID for getAcpState (default: auto-generated)
 *   --json                      (default) Output JSON receipt
 *   --help                      Show this help
 *
 * Exit codes:
 *   0 = all assertions passed
 *   1 = one or more assertions failed
 *   2 = infrastructure error (session dead, capture failed, etc.)
 */

import { existsSync, mkdirSync, statSync } from "fs";
import { join, resolve } from "path";

const SCHEMA_VERSION = 3;
const PROJECT_ROOT = resolve(import.meta.dir, "../..");

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface CaptureTarget {
  requestedWindowId: number | null;
  actualWindowId: number | null;
}

type CaptureRouting =
  | "cli-window-id"
  | "inspection-os-window-id"
  | "runtime-capture-window"
  | "generic-frontmost";

interface InspectionReceipt {
  automationWindowId: string;
  windowKind: string;
  title?: string | null;
  osWindowId?: number | null;
  targetBoundsInScreenshot?: {
    x: number;
    y: number;
    width: number;
    height: number;
  } | null;
  screenshotWidth?: number | null;
  screenshotHeight?: number | null;
  pixelProbes: Array<{ x: number; y: number; r: number; g: number; b: number; a: number }>;
  warnings: string[];
}

interface CapturePlan {
  routing: CaptureRouting;
  requestedAutomationWindowId: string | null;
  requestedOsWindowId: number | null;
  inspectionOsWindowId: number | null;
  showIssuedBeforeCapture: boolean;
}

/**
 * Deterministic popup capture strategy receipt.
 *
 * For attached popups (ActionsDialog, PromptPopup), the screenshot captures
 * the parent (main) window. The `targetBounds` field specifies the exact
 * crop region within the parent screenshot where the popup lives.
 *
 * For detached windows (AcpDetached, Notes), the screenshot is the window
 * itself at origin (0, 0).
 */
interface PopupCaptureReceipt {
  /** The capture strategy used. */
  strategy: "parent_capture_with_crop" | "direct_window_capture" | "not_applicable";
  /** The automation window kind that determined the strategy. */
  windowKind: string | null;
  /** Crop bounds within the screenshot (null for detached or when geometry unavailable). */
  targetBounds: { x: number; y: number; width: number; height: number } | null;
  /** Whether semantic receipts (not screenshots) are the primary verification oracle. */
  semanticReceiptsArePrimary: boolean;
}

interface VerifyReceipt {
  schemaVersion: number;
  status: "pass" | "fail" | "error";
  label: string;
  timestamp: string;
  durationMs: number;
  requiresVisionReview: boolean;
  // Exact target identity — populated from inspection when --target-json is used
  resolvedTarget: {
    windowId: string;
    windowKind: string;
    title?: string | null;
    surfaceId?: string | null;
  } | null;
  /** Routed input method used during the flow that produced this proof bundle. */
  inputMethod?: "batch" | "simulateGpuiEvent" | "native";
  /** Dispatch path for routed input: exact_handle when target was an ID, window_role_fallback otherwise. */
  dispatchPath?: "exact_handle" | "window_role_fallback";
  /** Resolved window ID from the automation target (same as resolvedTarget.windowId). */
  resolvedWindowId?: string;
  /** OS-level window ID (CGWindowID) when available from inspection. */
  osWindowId?: number | null;
  inspectionOsWindowId?: number | null;
  captureRouting?: CaptureRouting;
  requestedAutomationWindowId?: string | null;
  requestedOsWindowId?: number | null;
  showIssuedBeforeCapture?: boolean;
  // Stable proof bundle fields (canonical names for machine consumption)
  state: Record<string, unknown> | null;
  probe: Record<string, unknown> | null;
  screenshot: {
    path: string | null;
    captureMethod: string | null;
    windowCaptureMethod: string | null;
    windowId: number | null;
  } | null;
  captureTarget: CaptureTarget | null;
  inspection: InspectionReceipt | null;
  /** Deterministic popup capture strategy receipt. */
  popupCapture: PopupCaptureReceipt | null;
  visionCrops: VisionCheck[];
  // Detailed receipts (full diagnostics)
  stateReceipt: AcpStateResult | null;
  probeReceipt: ProbeResult | null;
  screenshotReceipt: ScreenshotResult | null;
  visionChecks: VisionCheck[];
  assertions: AssertionResult[];
  summary: string;
}

interface AcpStateResult {
  queried: boolean;
  snapshot: Record<string, unknown> | null;
  error: string | null;
}

interface ProbeResult {
  queried: boolean;
  snapshot: Record<string, unknown> | null;
  error: string | null;
}

interface ScreenshotResult {
  captured: boolean;
  path: string | null;
  sizeBytes: number | null;
  width: number | null;
  height: number | null;
  captureMethod: "window.ts" | "captureWindow" | null;
  windowCaptureMethod: "quartz" | "screencapture" | null;
  windowFrontmost: boolean | null;
  windowFocused: boolean | null;
  windowId: number | null;
  error: string | null;
}

interface VisionCheck {
  name: string;
  path: string;
  question: string;
  crop: { x: number; y: number; width: number; height: number } | null;
  expectedAnswer?: string | null;
  mustReview: boolean;
  failureMessage: string;
}

interface AssertionResult {
  name: string;
  expected: string;
  actual: string;
  passed: boolean;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function parseArgs() {
  const args = process.argv.slice(2);
  const opts: Record<string, string | boolean> = {};

  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--help" || arg === "-h") {
      opts.help = true;
    } else if (arg === "--skip-screenshot") {
      opts.skipScreenshot = true;
    } else if (arg === "--skip-state") {
      opts.skipState = true;
    } else if (arg === "--acp-picker-open") {
      opts.acpPickerOpen = true;
    } else if (arg === "--acp-picker-closed") {
      opts.acpPickerClosed = true;
    } else if (arg === "--acp-item-accepted") {
      opts.acpItemAccepted = true;
    } else if (arg === "--acp-context-ready") {
      opts.acpContextReady = true;
    } else if (arg === "--acp-no-selection") {
      opts.acpNoSelection = true;
    } else if (arg === "--acp-has-selection") {
      opts.acpHasSelection = true;
    } else if (arg === "--acp-no-permission") {
      opts.acpNoPermission = true;
    } else if (arg === "--acp-has-permission") {
      opts.acpHasPermission = true;
    } else if (arg === "--acp-setup-visible") {
      opts.acpSetupVisible = true;
    } else if (arg === "--acp-setup-agent-picker-open") {
      opts.acpSetupAgentPickerOpen = true;
    } else if (arg === "--emit-vision-crops" || arg === "--vision") {
      opts.emitVisionCrops = true;
    } else if (arg === "--skip-probe") {
      opts.skipProbe = true;
    } else if (arg === "--json") {
      // default, no-op
    } else if (arg.startsWith("--") && i + 1 < args.length) {
      const key = arg.slice(2).replace(/-([a-z])/g, (_, c) => c.toUpperCase());
      opts[key] = args[++i];
    }
  }

  return opts;
}

function hasOpt(
  opts: Record<string, string | boolean>,
  key: string
): boolean {
  return Object.prototype.hasOwnProperty.call(opts, key);
}

async function sendSessionCommand(
  session: string,
  cmd: string
): Promise<{ ok: boolean; stdout: string; stderr: string }> {
  const sessionScript = join(PROJECT_ROOT, "scripts/agentic/session.sh");
  const proc = Bun.spawn(["bash", sessionScript, "send", session, cmd], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;
  return { ok: code === 0, stdout: stdout.trim(), stderr: stderr.trim() };
}

async function queryAcpState(
  session: string,
  requestId: string,
  target?: Record<string, unknown>
): Promise<AcpStateResult> {
  const sessionScript = join(PROJECT_ROOT, "scripts/agentic/session.sh");
  const payload: Record<string, unknown> = {
    type: "getAcpState",
    requestId,
  };
  if (target) {
    payload.target = target;
  }
  const cmd = JSON.stringify(payload);

  const proc = Bun.spawn(
    [
      "bash",
      sessionScript,
      "rpc",
      session,
      cmd,
      "--expect",
      "acpStateResult",
      "--timeout",
      "3000",
    ],
    {
      stdout: "pipe",
      stderr: "pipe",
    }
  );
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;

  let parsed: Record<string, unknown> | null = null;
  try {
    parsed = JSON.parse(stdout) as Record<string, unknown>;
  } catch {
    parsed = null;
  }

  if (code !== 0) {
    const errorMessage =
      parsed &&
      typeof parsed.error === "object" &&
      parsed.error != null &&
      typeof (parsed.error as Record<string, unknown>).message === "string"
        ? String((parsed.error as Record<string, unknown>).message)
        : stderr.trim() || stdout.trim() || "RPC failed";
    return {
      queried: true,
      snapshot: null,
      error: `Failed to query getAcpState: ${errorMessage}`,
    };
  }

  const response = parsed?.response;
  if (!response || typeof response !== "object") {
    return {
      queried: true,
      snapshot: null,
      error: "RPC completed but did not return an acpStateResult payload",
    };
  }

  if ((response as Record<string, unknown>).type !== "acpStateResult") {
    return {
      queried: true,
      snapshot: null,
      error: "RPC returned an unexpected response type",
    };
  }

  return {
    queried: true,
    snapshot: response as Record<string, unknown>,
    error: null,
  };
}

async function queryAcpTestProbe(
  session: string,
  requestId: string,
  tail: number,
  target?: Record<string, unknown>
): Promise<ProbeResult> {
  const sessionScript = join(PROJECT_ROOT, "scripts/agentic/session.sh");
  const payload: Record<string, unknown> = {
    type: "getAcpTestProbe",
    requestId,
    tail,
  };
  if (target) {
    payload.target = target;
  }
  const cmd = JSON.stringify(payload);

  const proc = Bun.spawn(
    [
      "bash",
      sessionScript,
      "rpc",
      session,
      cmd,
      "--expect",
      "acpTestProbeResult",
      "--timeout",
      "3000",
    ],
    {
      stdout: "pipe",
      stderr: "pipe",
    }
  );
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;

  let parsed: Record<string, unknown> | null = null;
  try {
    parsed = JSON.parse(stdout) as Record<string, unknown>;
  } catch {
    parsed = null;
  }

  if (code !== 0) {
    const errorMessage =
      parsed &&
      typeof parsed.error === "object" &&
      parsed.error != null &&
      typeof (parsed.error as Record<string, unknown>).message === "string"
        ? String((parsed.error as Record<string, unknown>).message)
        : stderr.trim() || stdout.trim() || "RPC failed";
    return {
      queried: true,
      snapshot: null,
      error: `Failed to query getAcpTestProbe: ${errorMessage}`,
    };
  }

  const response = parsed?.response;
  if (!response || typeof response !== "object") {
    return {
      queried: true,
      snapshot: null,
      error: "RPC completed but did not return an acpTestProbeResult payload",
    };
  }

  if ((response as Record<string, unknown>).type !== "acpTestProbeResult") {
    return {
      queried: true,
      snapshot: null,
      error: "RPC returned an unexpected response type",
    };
  }

  return {
    queried: true,
    snapshot: response as Record<string, unknown>,
    error: null,
  };
}

async function queryInspection(
  session: string,
  requestId: string,
  target?: Record<string, unknown>
): Promise<InspectionReceipt | null> {
  const sessionScript = join(PROJECT_ROOT, "scripts/agentic/session.sh");
  const payload: Record<string, unknown> = {
    type: "inspectAutomationWindow",
    requestId,
  };
  if (target) {
    payload.target = target;
  }
  const cmd = JSON.stringify(payload);

  const proc = Bun.spawn(
    [
      "bash",
      sessionScript,
      "rpc",
      session,
      cmd,
      "--expect",
      "automationInspectResult",
      "--timeout",
      "5000",
    ],
    {
      stdout: "pipe",
      stderr: "pipe",
    }
  );
  const stdout = await new Response(proc.stdout).text();
  const code = await proc.exited;

  if (code !== 0) {
    return null;
  }

  let parsed: Record<string, unknown> | null = null;
  try {
    parsed = JSON.parse(stdout) as Record<string, unknown>;
  } catch {
    return null;
  }

  const response = (parsed?.response ?? parsed) as Record<string, unknown> | null;
  if (!response) {
    return null;
  }

  return {
    automationWindowId: String(response.windowId ?? ""),
    windowKind: String(response.windowKind ?? ""),
    title: (response.title as string) ?? null,
    osWindowId: (response.osWindowId as number) ?? null,
    targetBoundsInScreenshot:
      (response.targetBoundsInScreenshot as InspectionReceipt["targetBoundsInScreenshot"]) ?? null,
    screenshotWidth: (response.screenshotWidth as number) ?? null,
    screenshotHeight: (response.screenshotHeight as number) ?? null,
    pixelProbes: (response.pixelProbes as InspectionReceipt["pixelProbes"]) ?? [],
    warnings: (response.warnings as string[]) ?? [],
  };
}

function diag(event: string, data: Record<string, unknown> = {}): void {
  console.error(JSON.stringify({ event, ...data }));
}

function hasAcpAssertions(opts: Record<string, string | boolean>): boolean {
  return [
    "acpStatus",
    "acpPickerOpen",
    "acpPickerClosed",
    "acpInputContains",
    "acpInputMatch",
    "acpCursorAt",
    "acpItemAccepted",
    "acpAcceptedLabel",
    "acpAcceptedTrigger",
    "acpAcceptedVia",
    "acpCursorAfterAccepted",
    "acpContextReady",
    "acpNoSelection",
    "acpHasSelection",
    "acpNoPermission",
    "acpHasPermission",
    "acpVisibleStart",
    "acpVisibleEnd",
    "acpCursorInWindow",
  ].some((key) => hasOpt(opts, key));
}

function shouldQueryProbe(
  opts: Record<string, string | boolean>,
  skipProbe: boolean
): boolean {
  return (
    !skipProbe &&
    (hasOpt(opts, "acpAcceptedVia") || hasOpt(opts, "acpCursorAfterAccepted"))
  );
}

async function getImageDimensions(
  filePath: string
): Promise<{ width: number | null; height: number | null }> {
  try {
    const proc = Bun.spawn(
      ["sips", "-g", "pixelWidth", "-g", "pixelHeight", filePath],
      { stdout: "pipe", stderr: "pipe" }
    );
    const out = await new Response(proc.stdout).text();
    await proc.exited;
    const wMatch = out.match(/pixelWidth:\s*(\d+)/);
    const hMatch = out.match(/pixelHeight:\s*(\d+)/);
    return {
      width: wMatch ? parseInt(wMatch[1], 10) : null,
      height: hMatch ? parseInt(hMatch[1], 10) : null,
    };
  } catch {
    return { width: null, height: null };
  }
}

async function captureScreenshot(
  session: string,
  outPath: string,
  label: string,
  opts: Record<string, string | boolean>,
  inspection: InspectionReceipt | null,
  targetJson?: Record<string, unknown>,
  captureWindowId?: number
): Promise<{ result: ScreenshotResult; plan: CapturePlan }> {
  let captureMethod: "window.ts" | "captureWindow" | null = null;
  let windowCaptureMethod: "quartz" | "screencapture" | null = null;
  let windowFrontmost: boolean | null = null;
  let windowFocused: boolean | null = null;
  let windowId: number | null = null;
  const strictWindowProof = hasAcpAssertions(opts);
  const inspectionOsWindowId =
    typeof inspection?.osWindowId === "number" && inspection.osWindowId > 0
      ? inspection.osWindowId
      : null;
  const requestedAutomationWindowId = inspection?.automationWindowId ?? null;
  const requestedOsWindowId =
    typeof captureWindowId === "number" && captureWindowId > 0
      ? captureWindowId
      : inspectionOsWindowId;
  let captureRouting: CaptureRouting =
    requestedOsWindowId != null
      ? captureWindowId != null
        ? "cli-window-id"
        : "inspection-os-window-id"
      : "generic-frontmost";
  let showIssuedBeforeCapture = false;

  if (
    targetJson &&
    captureWindowId != null &&
    inspectionOsWindowId != null &&
    captureWindowId !== inspectionOsWindowId
  ) {
    return {
      result: {
        captured: false,
        path: outPath,
        sizeBytes: null,
        width: null,
        height: null,
        captureMethod: null,
        windowCaptureMethod: null,
        windowFrontmost: null,
        windowFocused: null,
        windowId: null,
        error: `Capture ID mismatch: cli --capture-window-id=${captureWindowId} but inspection resolved osWindowId=${inspectionOsWindowId}`,
      },
      plan: {
        routing: captureRouting,
        requestedAutomationWindowId,
        requestedOsWindowId,
        inspectionOsWindowId,
        showIssuedBeforeCapture,
      },
    };
  }

  if (targetJson && requestedOsWindowId == null) {
    return {
      result: {
        captured: false,
        path: outPath,
        sizeBytes: null,
        width: null,
        height: null,
        captureMethod: null,
        windowCaptureMethod: null,
        windowFrontmost: null,
        windowFocused: null,
        windowId: null,
        error:
          "Targeted screenshot capture requires a native osWindowId, but inspection did not return one",
      },
      plan: {
        routing: captureRouting,
        requestedAutomationWindowId,
        requestedOsWindowId,
        inspectionOsWindowId,
        showIssuedBeforeCapture,
      },
    };
  }

  const showCommand = await sendSessionCommand(session, JSON.stringify({ type: "show" }));
  if (!showCommand.ok) {
    return {
      result: {
        captured: false,
        path: outPath,
        sizeBytes: null,
        width: null,
        height: null,
        captureMethod: null,
        windowCaptureMethod: null,
        windowFrontmost: null,
        windowFocused: null,
        windowId: null,
        error: `Failed to send show before capture: ${showCommand.stderr || showCommand.stdout || "unknown error"}`,
      },
      plan: {
        routing: captureRouting,
        requestedAutomationWindowId,
        requestedOsWindowId,
        inspectionOsWindowId,
        showIssuedBeforeCapture,
      },
    };
  }
  showIssuedBeforeCapture = true;
  await Bun.sleep(500);

  // Use the window.ts helper for reliable capture
  const windowScript = join(PROJECT_ROOT, "scripts/agentic/window.ts");
  const captureArgs = [
    "bun",
    windowScript,
    "capture",
    outPath,
    "--activate-first",
    "--retry",
    "2",
    "--settle-ms",
    "200",
  ];
  // Thread exact window ID when provided
  if (requestedOsWindowId && requestedOsWindowId > 0) {
    captureArgs.push("--window-id", String(requestedOsWindowId));
  }
  const proc = Bun.spawn(
    captureArgs,
    {
      stdout: "pipe",
      stderr: "pipe",
    }
  );
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;

  if (code === 0) {
    captureMethod = "window.ts";
    try {
      const envelope = JSON.parse(stdout) as {
        data?: {
          method?: "quartz" | "screencapture";
          frontmost?: boolean;
          focused?: boolean;
          windowId?: number;
        };
      };
      windowCaptureMethod = envelope?.data?.method ?? null;
      windowFrontmost = envelope?.data?.frontmost ?? null;
      windowFocused = envelope?.data?.focused ?? null;
      windowId = envelope?.data?.windowId ?? null;
    } catch {
      // leave parsed fields null
    }
  } else if (!strictWindowProof) {
    captureRouting = "runtime-capture-window";
    // Fallback: use session-based captureWindow (only when no ACP assertions).
    // The runtime captureWindow handler now routes through the resolver-driven
    // capture path (capture_window_by_title_via_resolver), which translates the
    // title to an AutomationWindowTarget and emits the same structured log
    // sequence as the protocol captureScreenshot path:
    //   automation.capture_screenshot.title_compatibility
    //   automation.capture_screenshot.targeted
    //   automation.capture_screenshot.candidate_selected (or ambiguous_candidate)
    // An empty title resolves to AutomationWindowTarget::Main.
    captureMethod = "captureWindow";
    const captureCmd = JSON.stringify({
      type: "captureWindow",
      title: inspection?.title ?? "",
      path: outPath,
    });
    const { ok, stderr: sessErr } = await sendSessionCommand(
      session,
      captureCmd
    );
    await Bun.sleep(1000);

    if (!ok || !existsSync(outPath)) {
      return {
        result: {
          captured: false,
          path: outPath,
          sizeBytes: null,
          width: null,
          height: null,
          captureMethod,
          windowCaptureMethod,
          windowFrontmost,
          windowFocused,
          windowId,
          error: `Capture failed. window.ts: ${stderr.trim()}. session captureWindow: ${sessErr}`,
        },
        plan: {
          routing: captureRouting,
          requestedAutomationWindowId,
          requestedOsWindowId,
          inspectionOsWindowId,
          showIssuedBeforeCapture,
        },
      };
    }
  } else {
    // Strict mode: window.ts failed and we have ACP assertions — do not fall back
    return {
      result: {
        captured: false,
        path: outPath,
        sizeBytes: null,
        width: null,
        height: null,
        captureMethod: "window.ts",
        windowCaptureMethod,
        windowFrontmost,
        windowFocused,
        windowId,
        error: `Strict window capture failed: ${stderr.trim() || stdout.trim() || "window.ts capture failed"}`,
      },
      plan: {
        routing: captureRouting,
        requestedAutomationWindowId,
        requestedOsWindowId,
        inspectionOsWindowId,
        showIssuedBeforeCapture,
      },
    };
  }

  diag("verify_shot_capture_receipt", {
    label,
    captureMethod,
    windowCaptureMethod,
    windowFrontmost,
    windowFocused,
    windowId,
  });

  // Strict window proof: require quartz method, frontmost, and valid windowId
  if (
    strictWindowProof &&
    (windowCaptureMethod !== "quartz" ||
      windowFrontmost !== true ||
      windowId == null ||
      windowId <= 0)
  ) {
    return {
      result: {
        captured: false,
        path: outPath,
        sizeBytes: null,
        width: null,
        height: null,
        captureMethod,
        windowCaptureMethod,
        windowFrontmost,
        windowFocused,
        windowId,
        error: `Strict window capture required quartz/frontmost/windowId; got method=${windowCaptureMethod ?? "null"} frontmost=${String(windowFrontmost)} windowId=${String(windowId)}`,
      },
      plan: {
        routing: captureRouting,
        requestedAutomationWindowId,
        requestedOsWindowId,
        inspectionOsWindowId,
        showIssuedBeforeCapture,
      },
    };
  }

  if (
    strictWindowProof &&
    requestedOsWindowId != null &&
    windowId != null &&
    windowId !== requestedOsWindowId
  ) {
    return {
      result: {
        captured: false,
        path: outPath,
        sizeBytes: null,
        width: null,
        height: null,
        captureMethod,
        windowCaptureMethod,
        windowFrontmost,
        windowFocused,
        windowId,
        error: `Strict window capture targeted osWindowId=${requestedOsWindowId} but captured windowId=${windowId}`,
      },
      plan: {
        routing: captureRouting,
        requestedAutomationWindowId,
        requestedOsWindowId,
        inspectionOsWindowId,
        showIssuedBeforeCapture,
      },
    };
  }

  // Wait for file write
  await Bun.sleep(300);

  if (!existsSync(outPath)) {
    return {
      result: {
        captured: false,
        path: outPath,
        sizeBytes: null,
        width: null,
        height: null,
        captureMethod,
        windowCaptureMethod,
        windowFrontmost,
        windowFocused,
        windowId,
        error: "Screenshot file not created after capture",
      },
      plan: {
        routing: captureRouting,
        requestedAutomationWindowId,
        requestedOsWindowId,
        inspectionOsWindowId,
        showIssuedBeforeCapture,
      },
    };
  }

  const stats = statSync(outPath);
  const dims = await getImageDimensions(outPath);

  return {
    result: {
      captured: true,
      path: outPath,
      sizeBytes: stats.size,
      width: dims.width,
      height: dims.height,
      captureMethod,
      windowCaptureMethod,
      windowFrontmost,
      windowFocused,
      windowId,
      error: null,
    },
    plan: {
      routing: captureRouting,
      requestedAutomationWindowId,
      requestedOsWindowId,
      inspectionOsWindowId,
      showIssuedBeforeCapture,
    },
  };
}

function runAssertions(
  snapshot: Record<string, unknown> | null,
  opts: Record<string, string | boolean>
): AssertionResult[] {
  const results: AssertionResult[] = [];

  if (!snapshot) {
    // If state was expected but missing, every assertion fails
    if (hasOpt(opts, "acpStatus")) {
      results.push({
        name: "acp-status",
        expected: String(opts.acpStatus),
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpPickerOpen) {
      results.push({
        name: "acp-picker-open",
        expected: "true",
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpPickerClosed) {
      results.push({
        name: "acp-picker-closed",
        expected: "true",
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpInputContains")) {
      results.push({
        name: "acp-input-contains",
        expected: `contains "${opts.acpInputContains}"`,
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpInputMatch")) {
      results.push({
        name: "acp-input-match",
        expected: String(opts.acpInputMatch),
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpCursorAt")) {
      results.push({
        name: "acp-cursor-at",
        expected: String(opts.acpCursorAt),
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpItemAccepted) {
      results.push({
        name: "acp-item-accepted",
        expected: "non-null",
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpContextReady) {
      results.push({
        name: "acp-context-ready",
        expected: "true",
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpAcceptedLabel")) {
      results.push({
        name: "acp-accepted-label",
        expected: String(opts.acpAcceptedLabel),
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpAcceptedTrigger")) {
      results.push({
        name: "acp-accepted-trigger",
        expected: String(opts.acpAcceptedTrigger),
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpNoSelection) {
      results.push({
        name: "acp-no-selection",
        expected: "false",
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpHasSelection) {
      results.push({
        name: "acp-has-selection",
        expected: "true",
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpNoPermission) {
      results.push({
        name: "acp-no-permission",
        expected: "false",
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpHasPermission) {
      results.push({
        name: "acp-has-permission",
        expected: "true",
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpVisibleStart")) {
      results.push({
        name: "acp-visible-start",
        expected: String(opts.acpVisibleStart),
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpVisibleEnd")) {
      results.push({
        name: "acp-visible-end",
        expected: String(opts.acpVisibleEnd),
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpCursorInWindow")) {
      results.push({
        name: "acp-cursor-in-window",
        expected: String(opts.acpCursorInWindow),
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpSetupVisible) {
      results.push({
        name: "acp-setup-visible",
        expected: "true",
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpSetupReason")) {
      results.push({
        name: "acp-setup-reason",
        expected: String(opts.acpSetupReason),
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpSetupPrimaryAction")) {
      results.push({
        name: "acp-setup-primary-action",
        expected: String(opts.acpSetupPrimaryAction),
        actual: "<no state>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpSetupSelectedAgent")) {
      results.push({
        name: "acp-setup-selected-agent",
        expected: String(opts.acpSetupSelectedAgent),
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpSetupAgentPickerOpen) {
      results.push({
        name: "acp-setup-agent-picker-open",
        expected: "true",
        actual: "<no state>",
        passed: false,
      });
    }
    return results;
  }

  // Status assertion
  if (hasOpt(opts, "acpStatus")) {
    const actual = String(snapshot.status ?? "<missing>");
    results.push({
      name: "acp-status",
      expected: String(opts.acpStatus),
      actual,
      passed: actual === String(opts.acpStatus),
    });
  }

  // Picker open assertion
  if (opts.acpPickerOpen) {
    const picker = snapshot.picker as Record<string, unknown> | null;
    const actual = picker ? String(picker.open ?? false) : "false";
    results.push({
      name: "acp-picker-open",
      expected: "true",
      actual,
      passed: actual === "true",
    });
  }

  // Picker closed assertion
  if (opts.acpPickerClosed) {
    const picker = snapshot.picker as Record<string, unknown> | null;
    const isOpen = picker ? picker.open === true : false;
    results.push({
      name: "acp-picker-closed",
      expected: "true",
      actual: String(!isOpen),
      passed: !isOpen,
    });
  }

  // Input contains assertion
  if (hasOpt(opts, "acpInputContains")) {
    const inputText = String(snapshot.inputText ?? "");
    const substring = String(opts.acpInputContains);
    results.push({
      name: "acp-input-contains",
      expected: `contains "${substring}"`,
      actual: `"${inputText}"`,
      passed: inputText.includes(substring),
    });
  }

  // Input match assertion
  if (hasOpt(opts, "acpInputMatch")) {
    const inputText = String(snapshot.inputText ?? "");
    const expected = String(opts.acpInputMatch);
    results.push({
      name: "acp-input-match",
      expected: `"${expected}"`,
      actual: `"${inputText}"`,
      passed: inputText === expected,
    });
  }

  // Cursor position assertion
  if (hasOpt(opts, "acpCursorAt")) {
    const cursorIndex = Number(snapshot.cursorIndex ?? -1);
    const expected = Number(opts.acpCursorAt);
    results.push({
      name: "acp-cursor-at",
      expected: String(expected),
      actual: String(cursorIndex),
      passed: cursorIndex === expected,
    });
  }

  // Item accepted assertion
  if (opts.acpItemAccepted) {
    const item = snapshot.lastAcceptedItem;
    results.push({
      name: "acp-item-accepted",
      expected: "non-null",
      actual: item ? "present" : "null",
      passed: item != null,
    });
  }

  // Context ready assertion
  if (opts.acpContextReady) {
    const ready = snapshot.contextReady === true;
    results.push({
      name: "acp-context-ready",
      expected: "true",
      actual: String(ready),
      passed: ready,
    });
  }

  // Accepted item label assertion
  if (hasOpt(opts, "acpAcceptedLabel")) {
    const item = snapshot.lastAcceptedItem as Record<string, unknown> | null;
    const actual = item ? String(item.label ?? "<missing>") : "<no item>";
    const expected = String(opts.acpAcceptedLabel);
    results.push({
      name: "acp-accepted-label",
      expected,
      actual,
      passed: actual === expected,
    });
  }

  // Accepted item trigger assertion
  if (hasOpt(opts, "acpAcceptedTrigger")) {
    const item = snapshot.lastAcceptedItem as Record<string, unknown> | null;
    const actual = item ? String(item.trigger ?? "<missing>") : "<no item>";
    const expected = String(opts.acpAcceptedTrigger);
    results.push({
      name: "acp-accepted-trigger",
      expected,
      actual,
      passed: actual === expected,
    });
  }

  // Selection assertions
  if (opts.acpNoSelection) {
    const hasSel = snapshot.hasSelection === true;
    results.push({
      name: "acp-no-selection",
      expected: "false",
      actual: String(hasSel),
      passed: !hasSel,
    });
  }

  if (opts.acpHasSelection) {
    const hasSel = snapshot.hasSelection === true;
    results.push({
      name: "acp-has-selection",
      expected: "true",
      actual: String(hasSel),
      passed: hasSel,
    });
  }

  // Permission assertions
  if (opts.acpNoPermission) {
    const hasPerm = snapshot.hasPendingPermission === true;
    results.push({
      name: "acp-no-permission",
      expected: "false",
      actual: String(hasPerm),
      passed: !hasPerm,
    });
  }

  if (opts.acpHasPermission) {
    const hasPerm = snapshot.hasPendingPermission === true;
    results.push({
      name: "acp-has-permission",
      expected: "true",
      actual: String(hasPerm),
      passed: hasPerm,
    });
  }

  // Input layout assertions
  const layout = snapshot.inputLayout as Record<string, unknown> | null;

  if (hasOpt(opts, "acpVisibleStart")) {
    const expected = Number(opts.acpVisibleStart);
    const actual = layout ? Number(layout.visibleStart ?? -1) : -1;
    results.push({
      name: "acp-visible-start",
      expected: String(expected),
      actual: layout ? String(actual) : "<no layout>",
      passed: actual === expected,
    });
  }

  if (hasOpt(opts, "acpVisibleEnd")) {
    const expected = Number(opts.acpVisibleEnd);
    const actual = layout ? Number(layout.visibleEnd ?? -1) : -1;
    results.push({
      name: "acp-visible-end",
      expected: String(expected),
      actual: layout ? String(actual) : "<no layout>",
      passed: actual === expected,
    });
  }

  if (hasOpt(opts, "acpCursorInWindow")) {
    const expected = Number(opts.acpCursorInWindow);
    const actual = layout ? Number(layout.cursorInWindow ?? -1) : -1;
    results.push({
      name: "acp-cursor-in-window",
      expected: String(expected),
      actual: layout ? String(actual) : "<no layout>",
      passed: actual === expected,
    });
  }

  // ACP setup assertions
  const setup = snapshot.setup as Record<string, unknown> | null;

  if (opts.acpSetupVisible) {
    results.push({
      name: "acp-setup-visible",
      expected: "true",
      actual: setup ? "true" : "false",
      passed: setup != null,
    });
  }

  if (hasOpt(opts, "acpSetupReason")) {
    const expected = String(opts.acpSetupReason);
    const actual = setup ? String(setup.reasonCode ?? "<missing>") : "<no setup>";
    results.push({
      name: "acp-setup-reason",
      expected,
      actual,
      passed: actual === expected,
    });
  }

  if (hasOpt(opts, "acpSetupPrimaryAction")) {
    const expected = String(opts.acpSetupPrimaryAction);
    const actual = setup ? String(setup.primaryAction ?? "<missing>") : "<no setup>";
    results.push({
      name: "acp-setup-primary-action",
      expected,
      actual,
      passed: actual === expected,
    });
  }

  if (hasOpt(opts, "acpSetupSelectedAgent")) {
    const expected = String(opts.acpSetupSelectedAgent);
    const actual = setup ? String(setup.selectedAgentId ?? "<none>") : "<no setup>";
    results.push({
      name: "acp-setup-selected-agent",
      expected,
      actual,
      passed: actual === expected,
    });
  }

  if (opts.acpSetupAgentPickerOpen) {
    const actual = setup ? String(setup.agentPickerOpen ?? false) : "false";
    results.push({
      name: "acp-setup-agent-picker-open",
      expected: "true",
      actual,
      passed: actual === "true",
    });
  }

  return results;
}

function runProbeAssertions(
  probeSnapshot: Record<string, unknown> | null,
  opts: Record<string, string | boolean>
): AssertionResult[] {
  const results: AssertionResult[] = [];

  const needsProbe =
    hasOpt(opts, "acpAcceptedVia") || hasOpt(opts, "acpCursorAfterAccepted");
  if (!needsProbe) return results;

  if (!probeSnapshot) {
    if (hasOpt(opts, "acpAcceptedVia")) {
      results.push({
        name: "acp-accepted-via",
        expected: String(opts.acpAcceptedVia),
        actual: "<no probe>",
        passed: false,
      });
    }
    if (hasOpt(opts, "acpCursorAfterAccepted")) {
      results.push({
        name: "acp-cursor-after-accepted",
        expected: String(opts.acpCursorAfterAccepted),
        actual: "<no probe>",
        passed: false,
      });
    }
    return results;
  }

  // Extract the last accepted item from the probe
  const acceptedItems = probeSnapshot.acceptedItems as
    | Record<string, unknown>[]
    | undefined;
  const lastAccepted =
    acceptedItems && acceptedItems.length > 0
      ? acceptedItems[acceptedItems.length - 1]
      : null;

  if (hasOpt(opts, "acpAcceptedVia")) {
    const expected = String(opts.acpAcceptedVia);
    const actual = lastAccepted
      ? String(lastAccepted.acceptedViaKey ?? "<missing>")
      : "<no accepted items>";
    results.push({
      name: "acp-accepted-via",
      expected,
      actual,
      passed: actual === expected,
    });
  }

  if (hasOpt(opts, "acpCursorAfterAccepted")) {
    const expected = Number(opts.acpCursorAfterAccepted);
    const actual = lastAccepted
      ? Number(lastAccepted.cursorAfter ?? -1)
      : -1;
    results.push({
      name: "acp-cursor-after-accepted",
      expected: String(expected),
      actual: lastAccepted ? String(actual) : "<no accepted items>",
      passed: actual === expected,
    });
  }

  return results;
}

function buildVisionChecks(
  screenshotResult: ScreenshotResult | null,
  opts: Record<string, string | boolean>
): VisionCheck[] {
  if (!opts.emitVisionCrops) return [];
  if (!screenshotResult?.captured || !screenshotResult.path) return [];

  const checks: VisionCheck[] = [];
  const imgWidth = screenshotResult.width ?? 998;
  const imgHeight = screenshotResult.height ?? 712;

  // Composer line check — bottom region of the window where input lives
  const composerHeight = 56;
  const composerY = Math.max(0, imgHeight - composerHeight - 40);

  // Caret placement after insertion — mustReview because state alone
  // cannot prove the caret is *visually* positioned correctly
  if (hasOpt(opts, "acpAcceptedVia") || opts.acpItemAccepted) {
    checks.push({
      name: "composer-caret",
      path: screenshotResult.path,
      question:
        "Is the caret (blinking cursor) immediately after the accepted picker text with no gap or misalignment?",
      crop: {
        x: 0,
        y: composerY,
        width: imgWidth,
        height: composerHeight,
      },
      expectedAnswer: "yes",
      mustReview: true,
      failureMessage:
        "Caret is not visually aligned after ACP insertion. The cursor may have jumped or the inserted text may be clipped.",
    });
  }

  // Picker dismissal check — mustReview because a stale picker overlay
  // can linger visually even when state reports it closed
  if (opts.acpPickerClosed || hasOpt(opts, "acpAcceptedVia")) {
    checks.push({
      name: "picker-dismissed",
      path: screenshotResult.path,
      question:
        "Is the inline mention/slash picker dropdown fully dismissed (no floating list visible)?",
      crop: {
        x: 0,
        y: Math.max(0, composerY - 200),
        width: imgWidth,
        height: 200,
      },
      expectedAnswer: "yes",
      mustReview: true,
      failureMessage:
        "Picker overlay is still visually present despite state reporting it closed. Possible render stale frame.",
    });
  }

  // Single-line composer stability — mustReview because layout metrics
  // can report correct values while the visual input jumps or clips
  if (hasOpt(opts, "acpAcceptedVia") || opts.acpItemAccepted) {
    checks.push({
      name: "single-line-stability",
      path: screenshotResult.path,
      question:
        "Does the single-line composer remain visually stable — no clipped leading text, no vertical shift, no layout jump?",
      crop: {
        x: 0,
        y: composerY,
        width: imgWidth,
        height: composerHeight + 20,
      },
      expectedAnswer: "yes",
      mustReview: true,
      failureMessage:
        "Single-line composer shifted, clipped, or jumped after picker acceptance or cursor movement.",
    });
  }

  // Picker visibility check (for picker-open assertions)
  if (opts.acpPickerOpen) {
    checks.push({
      name: "picker-visible",
      path: screenshotResult.path,
      question:
        "Is the inline mention/slash picker dropdown visible with selectable rows?",
      crop: {
        x: 0,
        y: Math.max(0, composerY - 200),
        width: imgWidth,
        height: 200,
      },
      expectedAnswer: "yes",
      mustReview: true,
      failureMessage:
        "Picker dropdown is not visible despite state reporting it open.",
    });
  }

  return checks;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const opts = parseArgs();

if (opts.help) {
  const jsonFlag = process.argv.includes("--json");
  if (jsonFlag) {
    const helpJson = {
      schemaVersion: 1,
      script: "verify-shot",
      commands: [
        { name: "run", description: "Collect state/probe/screenshot proof", flags: ["--session", "--label", "--out", "--target-json", "--capture-window-id", "--vision", "--skip-screenshot", "--skip-state", "--skip-probe", "--json"] },
      ],
      contracts: [
        "popup-capture-receipts",
        "detached-proof-contract",
      ],
      receipts: [
        "popupCapture.strategy",
        "popupCapture.targetBounds",
        "popupCapture.semanticReceiptsArePrimary",
      ],
    };
    console.log(JSON.stringify(helpJson, null, 2));
    process.exit(0);
  }
  console.log(`Usage: bun scripts/agentic/verify-shot.ts --session NAME [options]

ACP proof bundle: state receipt + test probe + screenshot + vision prompts.

Options:
  --session NAME              Session name (default: "default")
  --label LABEL               Human-readable step label (default: "verify")
  --out PATH                  Screenshot output path (auto-generated if omitted)
  --acp-status STATUS         Assert ACP status equals this value
  --acp-picker-open           Assert picker is open
  --acp-picker-closed         Assert picker is closed
  --acp-input-contains STR    Assert input text contains substring
  --acp-input-match STR       Assert input text equals exactly
  --acp-cursor-at N           Assert cursor is at character index N
  --acp-item-accepted         Assert lastAcceptedItem is non-null
  --acp-accepted-label STR    Assert lastAcceptedItem.label equals STR
  --acp-accepted-trigger STR  Assert lastAcceptedItem.trigger equals STR (@ or /)
  --acp-accepted-via KEY      Assert probe acceptedItems[last].acceptedViaKey (enter|tab)
  --acp-cursor-after-accepted N  Assert probe acceptedItems[last].cursorAfter equals N
  --acp-context-ready         Assert contextReady is true
  --acp-no-selection          Assert hasSelection is false
  --acp-has-selection         Assert hasSelection is true
  --acp-no-permission         Assert hasPendingPermission is false
  --acp-has-permission        Assert hasPendingPermission is true
  --acp-visible-start N       Assert inputLayout.visibleStart equals N
  --acp-visible-end N         Assert inputLayout.visibleEnd equals N
  --acp-cursor-in-window N    Assert inputLayout.cursorInWindow equals N
  --acp-setup-visible         Assert setup card is present (status == "setup")
  --acp-setup-reason CODE     Assert setup.reasonCode equals CODE
  --acp-setup-primary-action A  Assert setup.primaryAction equals A
  --acp-setup-selected-agent ID Assert setup.selectedAgentId equals ID
  --acp-setup-agent-picker-open Assert setup.agentPickerOpen is true
  --probe-tail N              Number of probe events to request (default: 20)
  --vision                    Emit vision checks with mustReview prompts and requiresVisionReview
  --emit-vision-crops         Alias for --vision
  --skip-screenshot           Only run state assertions, skip capture
  --skip-state                Only capture screenshot, skip state query
  --skip-probe                Skip ACP test probe query
  --target-json JSON          ACP window target for getAcpState/getAcpTestProbe RPCs
  --capture-window-id N       Exact native window ID for screencapture
                              (the inspected osWindowId, not automationWindowId)
  --request-id ID             Request ID for getAcpState (auto-generated)

Verification order (ACP golden path):
  1. State receipt (getAcpState) — machine-readable proof
  2. Probe receipt (getAcpTestProbe) — key-route/picker-acceptance telemetry
  3. Screenshot capture — visual proof (metadata only; no automatic
     pixel inspection — a human or vision tool must read the PNG)
  4. Vision checks — structured prompts for external image readers
  5. Assertions check state + probe fields

Exit 0 = all assertions pass. Exit 1 = assertion failure. Exit 2 = infra error.`);
  process.exit(0);
}

const startTime = Date.now();
const session = String(opts.session ?? "default");
const label = String(opts.label ?? "verify");
const requestId =
  String(opts.requestId ?? `verify-${label}-${Date.now()}`);
const skipScreenshot = opts.skipScreenshot === true;
const skipState = opts.skipState === true;
const skipProbe = opts.skipProbe === true;
const probeTail = Number(opts.probeTail ?? 20);
const captureWindowId = typeof opts.captureWindowId === "string"
  ? (() => {
      const parsed = parseInt(opts.captureWindowId, 10);
      return Number.isFinite(parsed) && parsed > 0 ? parsed : undefined;
    })()
  : undefined;

// Parse --target-json for ACP window targeting
let targetJson: Record<string, unknown> | undefined;
if (typeof opts.targetJson === "string") {
  try {
    targetJson = JSON.parse(opts.targetJson) as Record<string, unknown>;
  } catch (error) {
    const reason = error instanceof Error ? error.message : String(error);
    console.error(`Invalid --target-json: ${reason}`);
    process.exit(2);
  }
}

// Determine output path
let outPath: string;
if (opts.out) {
  outPath = resolve(String(opts.out));
} else {
  const screenshotDir = join(PROJECT_ROOT, "test-screenshots");
  if (!existsSync(screenshotDir)) {
    mkdirSync(screenshotDir, { recursive: true });
  }
  outPath = join(
    screenshotDir,
    `${label}-${Date.now()}.png`
  );
}

// Step 1: Query ACP state (unless skipped)
let stateResult: AcpStateResult | null = null;
if (!skipState) {
  stateResult = await queryAcpState(session, requestId, targetJson);
}

// Step 2: Query ACP test probe only when probe assertions are requested
let probeResult: ProbeResult | null = null;
const needsProbe = shouldQueryProbe(opts, skipProbe);
if (needsProbe) {
  const probeRequestId = `${requestId}-probe`;
  probeResult = await queryAcpTestProbe(session, probeRequestId, probeTail, targetJson);
  diag("verify_shot_probe_loaded", {
    label,
    queried: probeResult.queried,
    hasSnapshot: probeResult.snapshot != null,
    acceptedVia: hasOpt(opts, "acpAcceptedVia")
      ? String(opts.acpAcceptedVia)
      : null,
    cursorAfterAccepted: hasOpt(opts, "acpCursorAfterAccepted")
      ? String(opts.acpCursorAfterAccepted)
      : null,
    error: probeResult.error ?? null,
  });
}

// Step 2b: Query inspection when --target-json or --vision is present
let inspection: InspectionReceipt | null = null;
if (targetJson || opts.emitVisionCrops) {
  const inspectRequestId = `${requestId}-inspect`;
  inspection = await queryInspection(session, inspectRequestId, targetJson);
  diag("verify_shot_inspection_loaded", {
    label,
    automationWindowId: inspection?.automationWindowId ?? null,
    windowKind: inspection?.windowKind ?? null,
    osWindowId: inspection?.osWindowId ?? null,
    probeCount: inspection?.pixelProbes.length ?? 0,
    warningCount: inspection?.warnings.length ?? 0,
  });
}

// Step 3: Capture screenshot (unless skipped)
let screenshotResult: ScreenshotResult | null = null;
let capturePlan: CapturePlan | null = null;
if (!skipScreenshot) {
  const captureOutcome = await captureScreenshot(
    session,
    outPath,
    label,
    opts,
    inspection,
    targetJson,
    captureWindowId
  );
  screenshotResult = captureOutcome.result;
  capturePlan = captureOutcome.plan;
}

// Step 4: Run assertions against state + probe
const stateAssertions = runAssertions(stateResult?.snapshot ?? null, opts);
const probeAssertions = runProbeAssertions(
  probeResult?.snapshot ?? null,
  opts
);
const assertions = [...stateAssertions, ...probeAssertions];

// Step 5: Build vision checks
const visionChecks = buildVisionChecks(screenshotResult, opts);

// Log assertion evaluation
for (const a of assertions) {
  diag("verify_shot_assertion", {
    label,
    assertion: a.name,
    expected: a.expected,
    actual: a.actual,
    passed: a.passed,
  });
}

// Build receipt
const allPassed = assertions.every((a) => a.passed);
const hasInfraError =
  (stateResult?.error && !skipState) ||
  (probeResult?.error && needsProbe) ||
  (screenshotResult && !screenshotResult.captured && !skipScreenshot);

const hasMustReviewItems = visionChecks.some((v) => v.mustReview);

// Compute deterministic popup capture strategy receipt
let popupCapture: PopupCaptureReceipt | null = null;
if (inspection) {
  const wk = inspection.windowKind;
  const attachedPopupKinds = ["ActionsDialog", "PromptPopup", "actionsDialog", "promptPopup"];
  const detachedKinds = ["AcpDetached", "Notes", "acpDetached", "notes"];
  const isAttached = attachedPopupKinds.includes(wk);
  const isDetached = detachedKinds.includes(wk);

  // Parse targetBoundsInScreenshot from inspection if available
  let targetBounds: PopupCaptureReceipt["targetBounds"] = null;
  if (inspection.targetBoundsInScreenshot) {
    const tb = inspection.targetBoundsInScreenshot as {
      x: number; y: number; width: number; height: number;
    };
    targetBounds = { x: tb.x, y: tb.y, width: tb.width, height: tb.height };
  }

  if (isAttached) {
    if (!targetBounds) {
      // Fail loud: attached popup without crop bounds is not a valid proof.
      // The Rust layer should have already failed, but enforce here too.
      diag("automation.capture_screenshot.parent_crop_failed", {
        windowKind: wk,
        automationWindowId: inspection.automationWindowId,
        reason: "attached popup has no targetBoundsInScreenshot — parent identity or popup bounds missing",
      });
      popupCapture = {
        strategy: "parent_capture_with_crop",
        windowKind: wk,
        targetBounds: null,
        semanticReceiptsArePrimary: true,
      };
      // Mark the receipt as an error since we cannot verify this popup.
      // The receipt will show strategy=parent_capture_with_crop but null bounds,
      // and the overall status will be "error" because hasInfraError will be set.
      if (!stateResult?.error && !screenshotResult?.error) {
        // Force infrastructure error when crop bounds are missing for attached popup
        if (screenshotResult) {
          screenshotResult.error = `Attached popup ${wk} captured without crop bounds — cannot verify popup region`;
          screenshotResult.captured = false;
        }
      }
    } else {
      diag("automation.capture_screenshot.parent_crop_selected", {
        windowKind: wk,
        automationWindowId: inspection.automationWindowId,
        targetBounds,
      });
      popupCapture = {
        strategy: "parent_capture_with_crop",
        windowKind: wk,
        targetBounds,
        semanticReceiptsArePrimary: true,
      };
    }
  } else if (isDetached) {
    popupCapture = {
      strategy: "direct_window_capture",
      windowKind: wk,
      targetBounds: null,
      semanticReceiptsArePrimary: true,
    };
  } else {
    popupCapture = {
      strategy: "not_applicable",
      windowKind: wk,
      targetBounds: null,
      semanticReceiptsArePrimary: false,
    };
  }
}

// Build resolvedTarget from inspection when available
const resolvedTarget: VerifyReceipt["resolvedTarget"] = inspection
  ? {
      windowId: inspection.automationWindowId,
      windowKind: inspection.windowKind,
      title: inspection.title ?? null,
      surfaceId: null,
    }
  : null;

// Infer dispatch path from target-json shape
const dispatchPath: VerifyReceipt["dispatchPath"] = targetJson
  ? (targetJson as { type: string }).type === "id"
    ? "exact_handle"
    : "window_role_fallback"
  : undefined;

const receipt: VerifyReceipt = {
  schemaVersion: SCHEMA_VERSION,
  status: hasInfraError ? "error" : allPassed ? "pass" : "fail",
  label,
  timestamp: new Date().toISOString(),
  durationMs: Date.now() - startTime,
  requiresVisionReview: hasMustReviewItems,
  resolvedTarget,
  ...(dispatchPath ? { dispatchPath } : {}),
  ...(resolvedTarget ? { resolvedWindowId: resolvedTarget.windowId } : {}),
  ...(inspection?.osWindowId != null ? { osWindowId: inspection.osWindowId } : {}),
  ...(capturePlan?.inspectionOsWindowId != null
    ? { inspectionOsWindowId: capturePlan.inspectionOsWindowId }
    : {}),
  ...(capturePlan?.routing ? { captureRouting: capturePlan.routing } : {}),
  ...(capturePlan ? { requestedAutomationWindowId: capturePlan.requestedAutomationWindowId } : {}),
  ...(capturePlan ? { requestedOsWindowId: capturePlan.requestedOsWindowId } : {}),
  ...(capturePlan ? { showIssuedBeforeCapture: capturePlan.showIssuedBeforeCapture } : {}),
  // Stable proof bundle fields
  state: stateResult?.snapshot ?? null,
  probe: probeResult?.snapshot ?? null,
  screenshot: screenshotResult
    ? {
        path: screenshotResult.path,
        captureMethod: screenshotResult.captureMethod,
        windowCaptureMethod: screenshotResult.windowCaptureMethod,
        windowId: screenshotResult.windowId,
      }
    : null,
  captureTarget: screenshotResult
    ? {
        requestedWindowId: capturePlan?.requestedOsWindowId ?? null,
        actualWindowId: screenshotResult.windowId,
      }
    : null,
  inspection,
  popupCapture,
  visionCrops: visionChecks,
  // Detailed receipts
  stateReceipt: stateResult,
  probeReceipt: probeResult,
  screenshotReceipt: screenshotResult,
  visionChecks,
  assertions,
  summary: buildSummary(assertions, stateResult, probeResult, screenshotResult),
};

console.log(JSON.stringify(receipt, null, 2));

if (hasInfraError) {
  process.exit(2);
} else {
  process.exit(allPassed ? 0 : 1);
}

function buildSummary(
  assertions: AssertionResult[],
  state: AcpStateResult | null,
  probe: ProbeResult | null,
  screenshot: ScreenshotResult | null
): string {
  const parts: string[] = [];

  if (state) {
    if (state.error) {
      parts.push(`state: ERROR (${state.error})`);
    } else {
      parts.push("state: queried");
    }
  }

  if (probe) {
    if (probe.error) {
      parts.push(`probe: ERROR (${probe.error})`);
    } else {
      parts.push("probe: queried");
    }
  }

  if (screenshot) {
    if (screenshot.captured) {
      parts.push(`screenshot: ${screenshot.path} (${screenshot.sizeBytes}B)`);
    } else {
      parts.push(`screenshot: FAILED (${screenshot.error})`);
    }
  }

  if (assertions.length > 0) {
    const passed = assertions.filter((a) => a.passed).length;
    const failed = assertions.filter((a) => !a.passed).length;
    parts.push(`assertions: ${passed} passed, ${failed} failed`);
    for (const a of assertions.filter((a) => !a.passed)) {
      parts.push(`  FAIL ${a.name}: expected ${a.expected}, got ${a.actual}`);
    }
  }

  return parts.join(" | ");
}
