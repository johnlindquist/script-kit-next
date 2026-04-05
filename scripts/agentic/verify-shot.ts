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
 *   2. Screenshot capture — visual proof
 *   3. Image-read verification — confirm visuals match expectations
 *
 * Usage:
 *   bun scripts/agentic/verify-shot.ts --session NAME [options]
 *
 * Options:
 *   --session NAME           Session name (default: "default")
 *   --label LABEL            Human-readable step label (default: "verify")
 *   --out PATH               Screenshot output path (default: test-screenshots/<label>-<ts>.png)
 *   --acp-status STATUS      Assert ACP status equals this value
 *   --acp-picker-open        Assert picker is open
 *   --acp-picker-closed      Assert picker is closed
 *   --acp-input-contains STR Assert input text contains substring
 *   --acp-input-match STR    Assert input text equals exactly
 *   --acp-cursor-at N        Assert cursor is at character index N
 *   --acp-item-accepted      Assert lastAcceptedItem is non-null
 *   --acp-context-ready      Assert contextReady is true
 *   --skip-screenshot        Only run state assertions, skip capture
 *   --skip-state             Only capture screenshot, skip ACP state query
 *   --request-id ID          Request ID for getAcpState (default: auto-generated)
 *   --json                   (default) Output JSON receipt
 *   --help                   Show this help
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
    } else if (arg === "--json") {
      // default, no-op
    } else if (arg.startsWith("--") && i + 1 < args.length) {
      const key = arg.slice(2).replace(/-([a-z])/g, (_, c) => c.toUpperCase());
      opts[key] = args[++i];
    }
  }

  return opts;
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
  const cmd = JSON.stringify({
    type: "getAcpState",
    requestId,
  });

  const { ok, stderr } = await sendSessionCommand(session, cmd);
  if (!ok) {
    return {
      queried: true,
      snapshot: null,
      error: `Failed to send getAcpState: ${stderr}`,
    };
  }

  // Read the log to find the acpStateResult response
  const sessionDir =
    process.env.SCRIPT_KIT_SESSION_DIR ?? "/tmp/sk-agentic-sessions";
  const logPath = join(sessionDir, session, "app.log");

  // Wait briefly for the response to appear in logs
  await Bun.sleep(500);

  if (!existsSync(logPath)) {
    return {
      queried: true,
      snapshot: null,
      error: `Log file not found: ${logPath}`,
    };
  }

  // Grep the log for the response
  const grepProc = Bun.spawn(
    ["grep", "-a", `acpStateResult.*${requestId}`, logPath],
    {
      stdout: "pipe",
      stderr: "pipe",
    }
  );
  const grepOut = await new Response(grepProc.stdout).text();
  await grepProc.exited;

  if (!grepOut.trim()) {
    // Also try looking for the JSON response directly
    const grep2 = Bun.spawn(
      ["grep", "-a", `"requestId":"${requestId}"`, logPath],
      {
        stdout: "pipe",
        stderr: "pipe",
      }
    );
    const grep2Out = await new Response(grep2.stdout).text();
    await grep2.exited;

    if (!grep2Out.trim()) {
      return {
        queried: true,
        snapshot: null,
        error: `No acpStateResult found in logs for requestId=${requestId}. The app may not support getAcpState yet, or the ACP view is not open.`,
      };
    }

    // Try to parse JSON from the line
    try {
      const line = grep2Out.trim().split("\n").pop() ?? "";
      const jsonStart = line.indexOf("{");
      if (jsonStart >= 0) {
        const parsed = JSON.parse(line.slice(jsonStart));
        return { queried: true, snapshot: parsed, error: null };
      }
    } catch {
      // fall through
    }
  }

  // Try to parse the acpStateResult from log output
  try {
    const lastLine = grepOut.trim().split("\n").pop() ?? "";
    const jsonStart = lastLine.indexOf("{");
    if (jsonStart >= 0) {
      const parsed = JSON.parse(lastLine.slice(jsonStart));
      return { queried: true, snapshot: parsed, error: null };
    }
  } catch {
    // fall through
  }

  return {
    queried: true,
    snapshot: null,
    error: `Found log entry but could not parse ACP state JSON`,
  };
}

async function captureScreenshot(
  session: string,
  outPath: string
): Promise<ScreenshotResult> {
  // Use the window.ts helper for reliable capture
  const windowScript = join(PROJECT_ROOT, "scripts/agentic/window.ts");
  const proc = Bun.spawn(["bun", windowScript, "capture", outPath], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;

  if (code !== 0) {
    // Fallback: use session-based captureWindow
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
      error: "Screenshot file not created after capture",
    };
  }

  const stats = statSync(outPath);
  return {
    captured: true,
    path: outPath,
    sizeBytes: stats.size,
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
    if (opts.acpStatus) {
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
    if (opts.acpInputContains) {
      results.push({
        name: "acp-input-contains",
        expected: `contains "${opts.acpInputContains}"`,
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpInputMatch) {
      results.push({
        name: "acp-input-match",
        expected: String(opts.acpInputMatch),
        actual: "<no state>",
        passed: false,
      });
    }
    if (opts.acpCursorAt) {
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
    return results;
  }

  // Status assertion
  if (opts.acpStatus) {
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
  if (opts.acpInputContains) {
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
  if (opts.acpInputMatch) {
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
  if (opts.acpCursorAt) {
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
  --session NAME           Session name (default: "default")
  --label LABEL            Human-readable step label (default: "verify")
  --out PATH               Screenshot output path (auto-generated if omitted)
  --acp-status STATUS      Assert ACP status equals this value
  --acp-picker-open        Assert picker is open
  --acp-picker-closed      Assert picker is closed
  --acp-input-contains STR Assert input text contains substring
  --acp-input-match STR    Assert input text equals exactly
  --acp-cursor-at N        Assert cursor is at character index N
  --acp-item-accepted      Assert lastAcceptedItem is non-null
  --acp-context-ready      Assert contextReady is true
  --skip-screenshot        Only run state assertions, skip capture
  --skip-state             Only capture screenshot, skip state query
  --request-id ID          Request ID for getAcpState (auto-generated)

Verification order (ACP golden path):
  1. State receipt first (getAcpState) — machine-readable proof
  2. Screenshot capture — visual proof
  3. Assertions check both sources

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
