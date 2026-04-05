#!/usr/bin/env bun
/**
 * scripts/agentic/verify-shot.ts
 *
 * Screenshot assertion helper for ACP agentic testing.
 * Captures a screenshot, reads the PNG, and returns a JSON pass/fail
 * receipt tied to explicit ACP state assertions and/or visual anchors.
 *
 * The reliable ACP verification order is:
 *   1. State receipt (getAcpState) — machine-readable proof
 *   2. Screenshot capture — visual proof (secondary confirmation)
 *   3. Note: actual image-read / pixel inspection is NOT performed automatically.
 *      The screenshot is captured and its metadata recorded, but a human or
 *      external vision tool must inspect the PNG to confirm visual correctness.
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
 *   --acp-context-ready         Assert contextReady is true
 *   --acp-no-selection          Assert hasSelection is false
 *   --acp-has-selection         Assert hasSelection is true
 *   --acp-no-permission         Assert hasPendingPermission is false
 *   --acp-has-permission        Assert hasPendingPermission is true
 *   --acp-visible-start N       Assert inputLayout.visibleStart equals N
 *   --acp-visible-end N         Assert inputLayout.visibleEnd equals N
 *   --acp-cursor-in-window N    Assert inputLayout.cursorInWindow equals N
 *   --skip-screenshot           Only run state assertions, skip capture
 *   --skip-state                Only capture screenshot, skip ACP state query
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

const SCHEMA_VERSION = 1;
const PROJECT_ROOT = resolve(import.meta.dir, "../..");

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface VerifyReceipt {
  schemaVersion: number;
  status: "pass" | "fail" | "error";
  label: string;
  timestamp: string;
  durationMs: number;
  stateReceipt: AcpStateResult | null;
  screenshotReceipt: ScreenshotResult | null;
  assertions: AssertionResult[];
  summary: string;
}

interface AcpStateResult {
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
  windowFrontmost: boolean | null;
  error: string | null;
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
  requestId: string
): Promise<AcpStateResult> {
  const sessionScript = join(PROJECT_ROOT, "scripts/agentic/session.sh");
  const cmd = JSON.stringify({
    type: "getAcpState",
    requestId,
  });

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
  outPath: string
): Promise<ScreenshotResult> {
  let captureMethod: "window.ts" | "captureWindow" | null = null;
  let windowFrontmost: boolean | null = null;

  // Use the window.ts helper for reliable capture
  const windowScript = join(PROJECT_ROOT, "scripts/agentic/window.ts");
  const proc = Bun.spawn(["bun", windowScript, "capture", outPath], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;

  if (code === 0) {
    captureMethod = "window.ts";
    // Parse window.ts envelope for frontmost status
    try {
      const envelope = JSON.parse(stdout);
      if (envelope?.data) {
        windowFrontmost = true; // window.ts focuses before capture
      }
    } catch {
      // couldn't parse — still captured
    }
  } else {
    // Fallback: use session-based captureWindow
    captureMethod = "captureWindow";
    windowFrontmost = null; // captureWindow doesn't guarantee frontmost
    const captureCmd = JSON.stringify({
      type: "captureWindow",
      title: "",
      path: outPath,
    });
    const { ok, stderr: sessErr } = await sendSessionCommand(
      session,
      captureCmd
    );
    await Bun.sleep(1000);

    if (!ok || !existsSync(outPath)) {
      return {
        captured: false,
        path: outPath,
        sizeBytes: null,
        width: null,
        height: null,
        captureMethod: "captureWindow",
        windowFrontmost: null,
        error: `Capture failed. window.ts: ${stderr.trim()}. session captureWindow: ${sessErr}`,
      };
    }
  }

  // Wait for file write
  await Bun.sleep(300);

  if (!existsSync(outPath)) {
    return {
      captured: false,
      path: outPath,
      sizeBytes: null,
      width: null,
      height: null,
      captureMethod,
      windowFrontmost,
      error: "Screenshot file not created after capture",
    };
  }

  const stats = statSync(outPath);
  const dims = await getImageDimensions(outPath);

  return {
    captured: true,
    path: outPath,
    sizeBytes: stats.size,
    width: dims.width,
    height: dims.height,
    captureMethod,
    windowFrontmost,
    error: null,
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

  return results;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const opts = parseArgs();

if (opts.help) {
  console.log(`Usage: bun scripts/agentic/verify-shot.ts --session NAME [options]

Captures a screenshot and verifies ACP state assertions. Returns a JSON receipt.

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
  --acp-context-ready         Assert contextReady is true
  --acp-no-selection          Assert hasSelection is false
  --acp-has-selection         Assert hasSelection is true
  --acp-no-permission         Assert hasPendingPermission is false
  --acp-has-permission        Assert hasPendingPermission is true
  --acp-visible-start N       Assert inputLayout.visibleStart equals N
  --acp-visible-end N         Assert inputLayout.visibleEnd equals N
  --acp-cursor-in-window N    Assert inputLayout.cursorInWindow equals N
  --skip-screenshot           Only run state assertions, skip capture
  --skip-state                Only capture screenshot, skip state query
  --request-id ID             Request ID for getAcpState (auto-generated)

Verification order (ACP golden path):
  1. State receipt first (getAcpState) — machine-readable proof
  2. Screenshot capture — secondary visual proof (metadata only; no automatic
     pixel inspection is performed — a human or vision tool must read the PNG)
  3. Assertions check ACP state fields

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
  stateResult = await queryAcpState(session, requestId);
}

// Step 2: Capture screenshot (unless skipped)
let screenshotResult: ScreenshotResult | null = null;
if (!skipScreenshot) {
  screenshotResult = await captureScreenshot(session, outPath);
}

// Step 3: Run assertions against state
const assertions = runAssertions(stateResult?.snapshot ?? null, opts);

// Build receipt
const allPassed = assertions.every((a) => a.passed);
const hasInfraError =
  (stateResult?.error && !skipState) ||
  (screenshotResult && !screenshotResult.captured && !skipScreenshot);

const receipt: VerifyReceipt = {
  schemaVersion: SCHEMA_VERSION,
  status: hasInfraError && assertions.length === 0 ? "error" : allPassed ? "pass" : "fail",
  label,
  timestamp: new Date().toISOString(),
  durationMs: Date.now() - startTime,
  stateReceipt: stateResult,
  screenshotReceipt: screenshotResult,
  assertions,
  summary: buildSummary(assertions, stateResult, screenshotResult),
};

console.log(JSON.stringify(receipt, null, 2));

if (hasInfraError && assertions.length === 0) {
  process.exit(2);
} else {
  process.exit(allPassed ? 0 : 1);
}

function buildSummary(
  assertions: AssertionResult[],
  state: AcpStateResult | null,
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
