#!/usr/bin/env bun
/**
 * scripts/agentic/window.ts
 *
 * Window discovery, focus, activation, and frontmost verification for
 * Script Kit GPUI agentic testing.
 *
 * Usage:
 *   bun scripts/agentic/window.ts find [--title SUBSTR] [--json]
 *   bun scripts/agentic/window.ts focus [--title SUBSTR] [--json]
 *   bun scripts/agentic/window.ts status [--json]
 *   bun scripts/agentic/window.ts capture PATH [--title SUBSTR] [--json]
 *
 * All structured output is JSON on stdout. Diagnostics go to stderr.
 */

import { existsSync, statSync } from "fs";

const SCHEMA_VERSION = 1;
const APP_NAME = "Script Kit";
const DEFAULT_TITLE_SUBSTR = "";

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
  name: string;
  index: number;
  bounds: { x: number; y: number; width: number; height: number } | null;
  frontmost: boolean;
  visible: boolean;
}

interface FindResult {
  windows: WindowInfo[];
  appRunning: boolean;
}

interface FocusResult {
  activated: boolean;
  frontmost: boolean;
  keyWindow: boolean;
}

interface StatusResult {
  appRunning: boolean;
  frontmost: boolean;
  visibleWindows: number;
  windows: WindowInfo[];
}

interface CaptureResult {
  path: string;
  sizeBytes: number;
  width: number | null;
  height: number | null;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function envelope<T>(
  command: string,
  data: T
): Envelope<T> {
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
    // Check for common permission errors
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
// Commands
// ---------------------------------------------------------------------------

async function findWindows(titleSubstr: string): Promise<FindResult> {
  // Check if app process is running
  let appRunning = false;
  try {
    const ps = await runCommand(
      ["pgrep", "-f", "script-kit-gpui"],
      "pgrep"
    );
    appRunning = ps.trim().length > 0;
  } catch {
    appRunning = false;
  }

  // Get window list from osascript
  const windows: WindowInfo[] = [];
  try {
    const script = `
tell application "System Events"
  set appProcs to every process whose name contains "Script Kit"
  set out to ""
  repeat with p in appProcs
    set wList to every window of p
    set idx to 0
    repeat with w in wList
      set wName to name of w
      set wPos to position of w
      set wSize to size of w
      set wVisible to true
      try
        set wVisible to (value of attribute "AXMinimized" of w) is false
      end try
      set out to out & wName & "|||" & idx & "|||" & (item 1 of wPos) & "," & (item 2 of wPos) & "," & (item 1 of wSize) & "," & (item 2 of wSize) & "|||" & wVisible & linefeed
      set idx to idx + 1
    end repeat
  end repeat
  return out
end tell`;
    const raw = await runOsascript(script);
    // Check frontmost status
    let frontmostApp = "";
    try {
      frontmostApp = await runOsascript(
        'tell application "System Events" to get name of first process whose frontmost is true'
      );
    } catch {
      // ignore
    }

    for (const line of raw.split("\n")) {
      if (!line.trim()) continue;
      const parts = line.split("|||");
      if (parts.length < 4) continue;
      const name = parts[0].trim();
      const index = parseInt(parts[1].trim(), 10);
      const boundsStr = parts[2].trim().split(",").map(Number);
      const visible = parts[3].trim() === "true";
      const bounds =
        boundsStr.length === 4
          ? {
              x: boundsStr[0],
              y: boundsStr[1],
              width: boundsStr[2],
              height: boundsStr[3],
            }
          : null;

      // Apply title filter
      if (titleSubstr && !name.toLowerCase().includes(titleSubstr.toLowerCase())) {
        continue;
      }

      windows.push({
        name,
        index,
        bounds,
        frontmost: frontmostApp.includes("Script Kit"),
        visible,
      });
    }
  } catch (e: any) {
    if (e.code === "APP_NOT_RUNNING") {
      return { windows: [], appRunning: false };
    }
    // Accessibility denied — still report it
    if (e.code === "ACCESSIBILITY_DENIED") {
      throw e;
    }
    // No windows found is ok
  }

  return { windows, appRunning };
}

async function focusWindow(titleSubstr: string): Promise<FocusResult> {
  // First activate via osascript
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
    throw Object.assign(new Error(`Failed to activate: ${e.message}`), {
      code: "ACTIVATION_FAILED",
    });
  }

  // Brief wait for activation
  await Bun.sleep(300);

  // Verify frontmost
  let frontmost = false;
  let keyWindow = false;
  try {
    const frontApp = await runOsascript(
      'tell application "System Events" to get name of first process whose frontmost is true'
    );
    frontmost = frontApp.includes("Script Kit");
  } catch {
    // couldn't verify
  }

  // Check key window via AX
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
    keyWindow = keyResult === "key";
  } catch {
    // AX query can fail - still report partial result
  }

  return { activated: true, frontmost, keyWindow };
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
  titleSubstr: string
): Promise<CaptureResult> {
  // Ensure window is focused first
  const focusResult = await focusWindow(titleSubstr);
  if (!focusResult.frontmost) {
    console.error(
      "[window.ts] Warning: window may not be frontmost after focus attempt"
    );
  }

  await Bun.sleep(200);

  // Use screencapture to capture the frontmost window
  const proc = Bun.spawn(["screencapture", "-l", await getWindowId(titleSubstr), path], {
    stdout: "pipe",
    stderr: "pipe",
  });
  const stderr = await new Response(proc.stderr).text();
  const code = await proc.exited;

  if (code !== 0) {
    // Fallback: use screencapture -w (interactive) won't work headless
    // Try full-window capture of frontmost
    const proc2 = Bun.spawn(["screencapture", "-o", "-x", path], {
      stdout: "pipe",
      stderr: "pipe",
    });
    const stderr2 = await new Response(proc2.stderr).text();
    const code2 = await proc2.exited;
    if (code2 !== 0) {
      throw Object.assign(
        new Error(`screencapture failed: ${stderr2.trim() || stderr.trim()}`),
        { code: "CAPTURE_FAILED" }
      );
    }
  }

  // Verify file was created
  if (!existsSync(path)) {
    throw Object.assign(new Error(`Screenshot file not created at ${path}`), {
      code: "CAPTURE_FAILED",
    });
  }

  const stats = statSync(path);
  // Try to get dimensions via sips
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

  return { path, sizeBytes: stats.size, width, height };
}

async function getWindowId(titleSubstr: string): Promise<string> {
  // Get the CGWindowID for screencapture -l
  try {
    const script = `
tell application "System Events"
  set appProcs to every process whose name contains "Script Kit"
  if (count of appProcs) > 0 then
    set p to item 1 of appProcs
    set wList to every window of p
    repeat with w in wList
      set wName to name of w
      ${titleSubstr ? `if wName contains "${titleSubstr}" then` : ""}
        -- Get window ID via attribute
        try
          set wid to value of attribute "AXIdentifier" of w
          return wid
        on error
          return ""
        end try
      ${titleSubstr ? "end if" : ""}
    end repeat
  end if
  return ""
end tell`;
    const wid = await runOsascript(script);
    if (wid && wid !== "") return wid;
  } catch {
    // fall through
  }

  // Fallback: use CGWindowListCopyWindowInfo to find the window ID
  try {
    const script = `
do shell script "python3 -c \\"
import Quartz
import json
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
        print(wid)
        break
\\""`;
    const wid = await runOsascript(script);
    if (wid && wid.trim() !== "") return wid.trim();
  } catch {
    // fall through
  }

  // Last resort — empty means screencapture will fail, caller handles fallback
  return "0";
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
    case "find": {
      const result = await findWindows(titleSubstr);
      emit(envelope("find", result));
      process.exit(result.windows.length > 0 ? 0 : 1);
      break;
    }

    case "focus": {
      const result = await focusWindow(titleSubstr);
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
      if (!capturePath) {
        emit(
          errorEnvelope(
            "capture",
            "MISSING_PATH",
            "Usage: window.ts capture <path> [--title SUBSTR]"
          )
        );
        process.exit(1);
        break;
      }
      const result = await captureWindow(capturePath, titleSubstr);
      emit(envelope("capture", result));
      process.exit(0);
      break;
    }

    case "help":
    case "--help": {
      console.log(`Usage: bun scripts/agentic/window.ts <command> [options]

Commands:
  find    [--title SUBSTR]           Find Script Kit windows
  focus   [--title SUBSTR]           Activate and focus window
  status                             App running + window status
  capture PATH [--title SUBSTR]      Focus + screencapture window

Options:
  --title SUBSTR    Filter windows by title substring

All commands output JSON. Exit 0 = success, 1 = not found or failed.

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

  // Add remediation hints for permission errors
  if (code === "ACCESSIBILITY_DENIED") {
    message +=
      "\n\nRemediation: System Preferences → Privacy & Security → Accessibility → enable your terminal/IDE app.";
  }

  emit(errorEnvelope(command, code, message));
  process.exit(1);
}
