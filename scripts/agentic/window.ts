#!/usr/bin/env bun
/**
 * scripts/agentic/window.ts
 *
 * Window discovery, focus, activation, and screenshot capture for
 * Script Kit GPUI agentic testing. Single authority for window focus
 * and capture — other scripts (macos-input.ts) delegate here.
 *
 * Usage:
 *   bun scripts/agentic/window.ts find   [--title SUBSTR] [--json]
 *   bun scripts/agentic/window.ts focus  [--title SUBSTR] [--retry N] [--settle-ms MS] [--json]
 *   bun scripts/agentic/window.ts status [--json]
 *   bun scripts/agentic/window.ts capture PATH [--title SUBSTR] [--retry N] [--activate-first] [--settle-ms MS] [--window-id N] [--json]
 *
 * All structured output is JSON on stdout. Diagnostics go to stderr as NDJSON.
 */

import { existsSync, statSync } from "fs";

const SCHEMA_VERSION = 2;
const APP_NAME = "Script Kit";
const DEFAULT_TITLE_SUBSTR = "";
const DEFAULT_RETRY = 1;
const DEFAULT_SETTLE_MS = 300;
const DEFAULT_CAPTURE_RETRY = 1;
const DEFAULT_CAPTURE_SETTLE_MS = 200;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface Envelope<T> {
  schemaVersion: number;
  status: "ok" | "error";
  command: string;
  data?: T;
  error?: { code: string; message: string };
}

interface WindowInfo {
  windowId: number;
  ownerName: string;
  title: string;
  layer: number;
  bounds: { x: number; y: number; width: number; height: number } | null;
  frontmost: boolean;
  focused: boolean;
  visible: boolean;
}

interface FindResult {
  windows: WindowInfo[];
  appRunning: boolean;
}

interface FocusResult {
  activated: boolean;
  frontmost: boolean;
  focused: boolean;
  attempts: number;
  settleMs: number;
  windowId: number;
}

interface StatusResult {
  appRunning: boolean;
  frontmost: boolean;
  visibleWindows: number;
  windows: WindowInfo[];
}

/** Stable automation-surface identity for ACP-relevant windows. */
interface AutomationSurface {
  /** Stable identity: "main", "acp", "actions", "notes", "ai", or window title. */
  surfaceId: string;
  /** Window kind hint: "main", "popup", "panel", or "unknown". */
  kind: string;
  windowId: number;
  title: string;
  frontmost: boolean;
  focused: boolean;
  visible: boolean;
  bounds: { x: number; y: number; width: number; height: number } | null;
}

interface ListResult {
  surfaces: AutomationSurface[];
  appRunning: boolean;
  focusedSurfaceId: string | null;
}

interface CaptureResult {
  path: string;
  windowId: number;
  attempts: number;
  method: "quartz" | "screencapture";
  frontmost: boolean;
  focused: boolean;
  sizeBytes: number;
  width: number | null;
  height: number | null;
}

// ---------------------------------------------------------------------------
// Structured stderr logging
// ---------------------------------------------------------------------------

function stderrLog(event: string, fields: Record<string, unknown> = {}): void {
  console.error(JSON.stringify({ event, ts: new Date().toISOString(), ...fields }));
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function envelope<T>(command: string, data: T): Envelope<T> {
  return { schemaVersion: SCHEMA_VERSION, status: "ok", command, data };
}

function errorEnvelope(
  command: string,
  code: string,
  message: string
): Envelope<never> {
  return {
    schemaVersion: SCHEMA_VERSION,
    status: "error",
    command,
    error: { code, message },
  };
}

async function runOsascript(script: string): Promise<string> {
  const proc = Bun.spawn(["osascript", "-e", script], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;
  if (code !== 0) {
    const msg = stderr.trim() || `osascript exit ${code}`;
    if (
      msg.includes("not allowed assistive access") ||
      msg.includes("is not allowed to send keystrokes") ||
      msg.includes("(-1743)")
    ) {
      throw Object.assign(new Error(msg), { code: "ACCESSIBILITY_DENIED" });
    }
    if (msg.includes("(-600)") || msg.includes("Application isn't running")) {
      throw Object.assign(new Error(msg), { code: "APP_NOT_RUNNING" });
    }
    throw Object.assign(new Error(msg), { code: "OSASCRIPT_ERROR" });
  }
  return stdout.trim();
}

async function runCommand(
  cmd: string[],
  label: string
): Promise<string> {
  const proc = Bun.spawn(cmd, {
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;
  if (code !== 0) {
    throw Object.assign(new Error(stderr.trim() || `${label} exit ${code}`), {
      code: "COMMAND_FAILED",
    });
  }
  return stdout.trim();
}

// ---------------------------------------------------------------------------
// Window ID resolution (Quartz CGWindowListCopyWindowInfo)
// ---------------------------------------------------------------------------

/**
 * Resolve the CGWindowID for a Script Kit window.
 * Returns the numeric ID (>0) or 0 if no window found.
 * Prefers Quartz enumeration for reliable IDs that screencapture -l accepts.
 */
async function resolveWindowId(titleSubstr: string): Promise<{ windowId: number; method: "quartz" | "ax" }> {
  // Primary: Quartz CGWindowListCopyWindowInfo
  try {
    const script = `
do shell script "python3 -c \\"
import Quartz, json
windows = Quartz.CGWindowListCopyWindowInfo(Quartz.kCGWindowListOptionOnScreenOnly, Quartz.kCGNullWindowID)
for w in windows:
    owner = w.get('kCGWindowOwnerName', '')
    name = w.get('kCGWindowName', '')
    wid = w.get('kCGWindowNumber', 0)
    layer = w.get('kCGWindowLayer', -1)
    bounds = w.get('kCGWindowBounds', {})
    width = bounds.get('Width', 0)
    height = bounds.get('Height', 0)
    if 'Script Kit' in owner and width > 100 and height > 100:
        print(json.dumps({'wid': wid, 'name': name, 'owner': owner, 'layer': layer}))
\\""`;
    const raw = await runOsascript(script);
    const lines = raw
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);
    const lowerTitle = titleSubstr.toLowerCase();
    for (const line of lines) {
      const parsed = JSON.parse(line) as {
        wid?: number;
        name?: string;
        owner?: string;
      };
      const name = parsed.name ?? "";
      const owner = parsed.owner ?? "";
      const matchesTitle =
        lowerTitle.length === 0 ||
        name.toLowerCase().includes(lowerTitle) ||
        owner.toLowerCase().includes(lowerTitle);
      if (matchesTitle && parsed.wid && parsed.wid > 0) {
        return { windowId: parsed.wid, method: "quartz" };
      }
    }
  } catch {
    // fall through to AX
  }

  // Fallback: AXIdentifier
  try {
    const script = `
tell application "System Events"
    set appProcs to every process whose name contains "Script Kit"
  if (count of appProcs) > 0 then
    set p to item 1 of appProcs
    set wList to every window of p
    repeat with w in wList
      set wName to name of w
      try
        set wid to value of attribute "AXIdentifier" of w
        return (wid as text) & "|||" & wName
      on error
        return ""
      end try
    end repeat
  end if
  return ""
end tell`;
    const wid = await runOsascript(script);
    if (wid) {
      const [idText, name = ""] = wid.split("|||");
      const parsed = parseInt(idText, 10);
      const lowerTitle = titleSubstr.toLowerCase();
      const matchesTitle =
        lowerTitle.length === 0 || name.toLowerCase().includes(lowerTitle);
      if (matchesTitle && parsed > 0) {
        return { windowId: parsed, method: "ax" };
      }
    }
  } catch {
    // fall through
  }

  return { windowId: 0, method: "quartz" };
}

// ---------------------------------------------------------------------------
// Quartz-based window enumeration (richer than System Events)
// ---------------------------------------------------------------------------

async function enumerateQuartzWindows(titleSubstr: string): Promise<WindowInfo[]> {
  try {
    const script = `
do shell script "python3 -c \\"
import Quartz, json
windows = Quartz.CGWindowListCopyWindowInfo(Quartz.kCGWindowListOptionOnScreenOnly, Quartz.kCGNullWindowID)
results = []
for w in windows:
    owner = w.get('kCGWindowOwnerName', '')
    name = w.get('kCGWindowName', '')
    wid = w.get('kCGWindowNumber', 0)
    layer = w.get('kCGWindowLayer', -1)
    bounds = w.get('kCGWindowBounds', {})
    if 'Script Kit' in owner:
        results.append({'wid': wid, 'owner': owner, 'name': name, 'layer': layer, 'x': int(bounds.get('X', 0)), 'y': int(bounds.get('Y', 0)), 'w': int(bounds.get('Width', 0)), 'h': int(bounds.get('Height', 0))})
print(json.dumps(results))
\\""`;
    const raw = await runOsascript(script);
    const parsed = JSON.parse(raw.trim());
    const lowerTitle = titleSubstr.toLowerCase();
    return (parsed as any[])
      .filter((w: any) => {
        if (lowerTitle.length === 0) return true;
        const name = String(w.name ?? "").toLowerCase();
        const owner = String(w.owner ?? "").toLowerCase();
        return name.includes(lowerTitle) || owner.includes(lowerTitle);
      })
      .map((w: any) => ({
        windowId: w.wid,
        ownerName: w.owner,
        title: w.name,
        layer: w.layer,
        bounds: { x: w.x, y: w.y, width: w.w, height: w.h },
        frontmost: false, // filled in below
        focused: false,
        visible: w.w > 0 && w.h > 0,
      }));
  } catch {
    return [];
  }
}

// ---------------------------------------------------------------------------
// Frontmost / focused checks
// ---------------------------------------------------------------------------

async function checkFrontmost(): Promise<{ frontmost: boolean; focused: boolean }> {
  let frontmost = false;
  let focused = false;
  try {
    const frontApp = await runOsascript(
      'tell application "System Events" to get name of first process whose frontmost is true'
    );
    frontmost = frontApp.includes("Script Kit");
  } catch {
    // couldn't verify
  }
  try {
    const keyResult = await runOsascript(`
tell application "System Events"
  set appProcs to every process whose name contains "Script Kit"
  if (count of appProcs) > 0 then
    set p to item 1 of appProcs
    set focWin to value of attribute "AXFocusedWindow" of p
    if focWin is not missing value then
      return "key"
    else
      return "no-key"
    end if
  end if
end tell`);
    focused = keyResult === "key";
  } catch {
    // AX query can fail
  }
  return { frontmost, focused };
}

// ---------------------------------------------------------------------------
// Commands
// ---------------------------------------------------------------------------

async function findWindows(titleSubstr: string): Promise<FindResult> {
  let appRunning = false;
  try {
    const ps = await runCommand(["pgrep", "-f", "script-kit-gpui"], "pgrep");
    appRunning = ps.trim().length > 0;
  } catch {
    appRunning = false;
  }

  const windows = await enumerateQuartzWindows(titleSubstr);

  // Fill in frontmost status
  const { frontmost, focused } = await checkFrontmost();
  if (windows.length > 0) {
    // First window inherits frontmost/focused (Quartz returns front-to-back order)
    windows[0].frontmost = frontmost;
    windows[0].focused = focused;
  }

  return { windows, appRunning: appRunning || windows.length > 0 };
}

async function focusWindow(
  titleSubstr: string,
  retryCount: number,
  settleMs: number
): Promise<FocusResult> {
  let lastFrontmost = false;
  let lastFocused = false;
  let windowId = 0;

  for (let attempt = 1; attempt <= retryCount; attempt++) {
    stderrLog("window_focus_attempt", { attempt, retryCount, titleSubstr });

    try {
      await runOsascript(`
tell application "System Events"
  set appProcs to every process whose name contains "Script Kit"
  if (count of appProcs) > 0 then
    set frontmost of item 1 of appProcs to true
  else
    error "Script Kit process not found"
  end if
end tell`);
    } catch (e: any) {
      if (e.code === "ACCESSIBILITY_DENIED") throw e;
      stderrLog("window_focus_activate_failed", { attempt, error: e.message });
      if (attempt < retryCount) {
        await Bun.sleep(settleMs);
        continue;
      }
      throw Object.assign(new Error(`Failed to activate after ${retryCount} attempts: ${e.message}`), {
        code: "ACTIVATION_FAILED",
      });
    }

    // Settle delay
    await Bun.sleep(settleMs);

    // Check state after settle
    const state = await checkFrontmost();
    lastFrontmost = state.frontmost;
    lastFocused = state.focused;

    // Resolve window ID
    const resolved = await resolveWindowId(titleSubstr);
    windowId = resolved.windowId;

    stderrLog("window_focus_result", {
      attempt,
      frontmost: lastFrontmost,
      focused: lastFocused,
      windowId,
    });

    if (lastFrontmost) break;
  }

  return {
    activated: true,
    frontmost: lastFrontmost,
    focused: lastFocused,
    attempts: retryCount,
    settleMs,
    windowId,
  };
}

/**
 * Classify a Script Kit window into a stable automation surface identity.
 * Uses title heuristics to assign surface IDs that agents can target.
 */
function classifyWindow(w: WindowInfo): AutomationSurface {
  const titleLower = (w.title ?? "").toLowerCase();
  let surfaceId = "main";
  let kind = "popup";

  if (titleLower.includes("acp") || titleLower.includes("chat")) {
    surfaceId = "acp";
    kind = "panel";
  } else if (titleLower.includes("actions") || titleLower.includes("⌘k")) {
    surfaceId = "actions";
    kind = "popup";
  } else if (titleLower.includes("notes")) {
    surfaceId = "notes";
    kind = "panel";
  } else if (titleLower.includes("ai")) {
    surfaceId = "ai";
    kind = "panel";
  } else if (w.layer > 100) {
    // PopUp windows at elevated levels are likely main or detached surfaces
    surfaceId = "main";
    kind = "popup";
  }

  return {
    surfaceId,
    kind,
    windowId: w.windowId,
    title: w.title,
    frontmost: w.frontmost,
    focused: w.focused,
    visible: w.visible,
    bounds: w.bounds,
  };
}

async function listSurfaces(titleSubstr: string): Promise<ListResult> {
  const findResult = await findWindows(titleSubstr);
  const surfaces = findResult.windows.map(classifyWindow);

  // Deduplicate by surfaceId — keep the first (frontmost) window per ID
  const seen = new Set<string>();
  const deduped: AutomationSurface[] = [];
  for (const s of surfaces) {
    // If two windows have the same surfaceId, disambiguate with index
    const key = seen.has(s.surfaceId) ? `${s.surfaceId}:${s.windowId}` : s.surfaceId;
    if (seen.has(s.surfaceId)) {
      s.surfaceId = key;
    }
    seen.add(s.surfaceId);
    deduped.push(s);
  }

  const focusedSurface = deduped.find((s) => s.focused || s.frontmost);
  return {
    surfaces: deduped,
    appRunning: findResult.appRunning,
    focusedSurfaceId: focusedSurface?.surfaceId ?? null,
  };
}

async function getStatus(): Promise<StatusResult> {
  const findResult = await findWindows("");
  return {
    appRunning: findResult.appRunning,
    frontmost: findResult.windows.some((w) => w.frontmost),
    visibleWindows: findResult.windows.filter((w) => w.visible).length,
    windows: findResult.windows,
  };
}

async function captureWindow(
  path: string,
  titleSubstr: string,
  retryCount: number,
  activateFirst: boolean,
  settleMs: number,
  exactWindowId?: number
): Promise<CaptureResult> {
  let windowId = exactWindowId ?? 0;
  let captureMethod: "quartz" | "screencapture" = "screencapture";
  let attempts = 0;
  let lastFrontmost = false;
  let lastFocused = false;

  for (let attempt = 1; attempt <= retryCount; attempt++) {
    attempts = attempt;
    stderrLog("window_capture_attempt", { attempt, retryCount, path, titleSubstr, activateFirst, exactWindowId: exactWindowId ?? null });

    // Optionally focus first
    if (activateFirst || attempt > 1) {
      const focusResult = await focusWindow(titleSubstr, 1, settleMs);
      lastFrontmost = focusResult.frontmost;
      lastFocused = focusResult.focused;
      // Prefer exact window ID if provided; fall back to focus-resolved ID
      if (!exactWindowId) {
        windowId = focusResult.windowId;
      }
    } else {
      // Just resolve ID without activating
      if (!exactWindowId) {
        const resolved = await resolveWindowId(titleSubstr);
        windowId = resolved.windowId;
      }
      const state = await checkFrontmost();
      lastFrontmost = state.frontmost;
      lastFocused = state.focused;
    }

    // Try screencapture with window ID
    if (windowId > 0) {
      const proc = Bun.spawn(["screencapture", "-l", String(windowId), path], {
        stdout: "pipe",
        stderr: "pipe",
      });
      const stderr = await new Response(proc.stderr).text();
      const code = await proc.exited;

      if (code === 0 && existsSync(path)) {
        captureMethod = "quartz";
        stderrLog("window_capture_success", { attempt, windowId, method: "quartz", path });
        break;
      }
      stderrLog("window_capture_screencapture_l_failed", { attempt, windowId, exitCode: code, stderr: stderr.trim() });
    }

    if (exactWindowId && exactWindowId > 0) {
      if (attempt === retryCount) {
        throw Object.assign(
          new Error(
            `Targeted window capture failed for exact window ID ${exactWindowId} after ${retryCount} attempts`
          ),
          { code: "CAPTURE_FAILED" }
        );
      }
      await Bun.sleep(settleMs);
      continue;
    }

    // Fallback: full-screen capture (non-interactive)
    if (attempt === retryCount) {
      const proc2 = Bun.spawn(["screencapture", "-o", "-x", path], {
        stdout: "pipe",
        stderr: "pipe",
      });
      const stderr2 = await new Response(proc2.stderr).text();
      const code2 = await proc2.exited;
      if (code2 === 0 && existsSync(path)) {
        captureMethod = "screencapture";
        stderrLog("window_capture_fallback_success", { attempt, method: "screencapture", path });
        break;
      }
      throw Object.assign(
        new Error(`screencapture failed after ${retryCount} attempts: ${stderr2.trim()}`),
        { code: "CAPTURE_FAILED" }
      );
    }

    // Settle before retry
    await Bun.sleep(settleMs);
  }

  // Verify file
  if (!existsSync(path)) {
    throw Object.assign(new Error(`Screenshot file not created at ${path}`), {
      code: "CAPTURE_FAILED",
    });
  }

  const stats = statSync(path);
  let width: number | null = null;
  let height: number | null = null;
  try {
    const sipsOut = await runCommand(
      ["sips", "-g", "pixelWidth", "-g", "pixelHeight", path],
      "sips"
    );
    const wMatch = sipsOut.match(/pixelWidth:\s*(\d+)/);
    const hMatch = sipsOut.match(/pixelHeight:\s*(\d+)/);
    if (wMatch) width = parseInt(wMatch[1], 10);
    if (hMatch) height = parseInt(hMatch[1], 10);
  } catch {
    // dimensions unknown
  }

  // Do not return windowId: 0 for a claimed successful capture
  // If we used the fallback method, windowId may be 0 — that's still valid
  // because the fallback captures the full screen, not a specific window.
  // But we report it honestly.

  return {
    path,
    windowId,
    attempts,
    method: captureMethod,
    frontmost: lastFrontmost,
    focused: lastFocused,
    sizeBytes: stats.size,
    width,
    height,
  };
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

function parseIntArg(args: string[], flag: string, fallback: number): number {
  const idx = args.indexOf(flag);
  if (idx >= 0 && args[idx + 1]) {
    const v = parseInt(args[idx + 1], 10);
    return isNaN(v) ? fallback : v;
  }
  return fallback;
}

function hasFlag(args: string[], flag: string): boolean {
  return args.includes(flag);
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const args = process.argv.slice(2);
const command = args[0] ?? "status";
const titleIdx = args.indexOf("--title");
const titleSubstr =
  titleIdx >= 0 && args[titleIdx + 1] ? args[titleIdx + 1] : DEFAULT_TITLE_SUBSTR;

function emit(data: Envelope<any>) {
  console.log(JSON.stringify(data, null, 2));
}

try {
  switch (command) {
    case "list": {
      const result = await listSurfaces(titleSubstr);
      emit(envelope("list", result));
      process.exit(result.surfaces.length > 0 ? 0 : 1);
      break;
    }

    case "find": {
      const result = await findWindows(titleSubstr);
      emit(envelope("find", result));
      process.exit(result.windows.length > 0 ? 0 : 1);
      break;
    }

    case "focus": {
      const retry = parseIntArg(args, "--retry", DEFAULT_RETRY);
      const settleMs = parseIntArg(args, "--settle-ms", DEFAULT_SETTLE_MS);
      const result = await focusWindow(titleSubstr, retry, settleMs);
      emit(envelope("focus", result));
      process.exit(result.frontmost ? 0 : 1);
      break;
    }

    case "status": {
      const result = await getStatus();
      emit(envelope("status", result));
      process.exit(result.appRunning ? 0 : 1);
      break;
    }

    case "capture": {
      const capturePath = args[1];
      if (!capturePath || capturePath.startsWith("--")) {
        emit(
          errorEnvelope(
            "capture",
            "MISSING_PATH",
            "Usage: window.ts capture <path> [--title SUBSTR] [--retry N] [--activate-first] [--settle-ms MS] [--window-id N]"
          )
        );
        process.exit(1);
        break;
      }
      const retry = parseIntArg(args, "--retry", DEFAULT_CAPTURE_RETRY);
      const activateFirst = hasFlag(args, "--activate-first");
      const settleMs = parseIntArg(args, "--settle-ms", DEFAULT_CAPTURE_SETTLE_MS);
      const exactWindowId = parseIntArg(args, "--window-id", 0);
      const result = await captureWindow(capturePath, titleSubstr, retry, activateFirst, settleMs, exactWindowId > 0 ? exactWindowId : undefined);
      emit(envelope("capture", result));
      process.exit(0);
      break;
    }

    case "help":
    case "--help": {
      console.log(`Usage: bun scripts/agentic/window.ts <command> [options]

Commands:
  list    [--title SUBSTR]                                 List automation surfaces with stable IDs
  find    [--title SUBSTR]                                 Find Script Kit windows (raw Quartz)
  focus   [--title SUBSTR] [--retry N] [--settle-ms MS]    Activate and focus window
  status                                                    App running + window status
  capture PATH [--title SUBSTR] [--retry N]                Focus + screencapture window
          [--activate-first] [--settle-ms MS]

Options:
  --title SUBSTR       Filter windows by title substring
  --retry N            Number of focus/capture attempts (default: 1 for focus, 1 for capture)
  --settle-ms MS       Delay after activation before checking state (default: 300 focus, 200 capture)
  --activate-first     Activate window before capture attempt (default: only on retry)
  --json               (accepted for compatibility, output is always JSON)

Output:
  Schema version ${SCHEMA_VERSION} JSON envelopes on stdout.
  Structured NDJSON diagnostics on stderr.
  Exit 0 = success, 1 = not found or failed.

Window ID resolution:
  Uses Quartz CGWindowListCopyWindowInfo (preferred) with AXIdentifier fallback.
  screencapture -l <windowId> for targeted capture; full-screen fallback if ID unavailable.

Permission requirements:
  - System Events access (Automation permission in Privacy & Security)
  - Screen Recording permission for 'capture' command

Remediation:
  System Preferences → Privacy & Security → Automation → allow Terminal/IDE
  System Preferences → Privacy & Security → Screen Recording → allow Terminal/IDE`);
      process.exit(0);
      break;
    }

    default:
      emit(
        errorEnvelope("unknown", "UNKNOWN_COMMAND", `Unknown command: ${command}`)
      );
      process.exit(1);
  }
} catch (e: any) {
  const code = e.code ?? "UNKNOWN_ERROR";
  let message = e.message ?? String(e);

  if (code === "ACCESSIBILITY_DENIED") {
    message +=
      "\n\nRemediation: System Preferences → Privacy & Security → Accessibility → enable your terminal/IDE app.";
  }

  emit(errorEnvelope(command, code, message));
  process.exit(1);
}
