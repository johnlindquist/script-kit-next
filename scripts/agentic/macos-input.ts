#!/usr/bin/env bun
/**
 * scripts/agentic/macos-input.ts
 *
 * Native macOS keyboard and mouse automation for Script Kit GPUI agentic testing.
 * Uses cliclick for input delivery and osascript for activation fallback.
 *
 * Focus enforcement delegates to window.ts — this script does NOT maintain a
 * separate activation path.
 *
 * Usage:
 *   bun scripts/agentic/macos-input.ts key <keyname> [--modifiers cmd,shift,...] [--ensure-focus] [--focus-title SUBSTR] [--session NAME] [--target SURFACE] [--json]
 *   bun scripts/agentic/macos-input.ts type <text> [--ensure-focus] [--focus-title SUBSTR] [--session NAME] [--target SURFACE] [--json]
 *   bun scripts/agentic/macos-input.ts click <x> <y> [--ensure-focus] [--focus-title SUBSTR] [--session NAME] [--target SURFACE] [--json]
 *   bun scripts/agentic/macos-input.ts sequence <json-array> [--ensure-focus] [--focus-title SUBSTR] [--session NAME] [--target SURFACE] [--json]
 *   bun scripts/agentic/macos-input.ts check [--json]
 *
 * All structured output is JSON on stdout. Diagnostics go to stderr as NDJSON.
 */

import { existsSync } from "fs";

const SCHEMA_VERSION = 2;
const WINDOW_TS_PATH = new URL("./window.ts", import.meta.url).pathname;
const DEFAULT_FOCUS_TITLE = "Script Kit";
const DEFAULT_FOCUS_RETRY = 3;
const DEFAULT_FOCUS_SETTLE_MS = 200;

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

interface KeyResult {
  key: string;
  modifiers: string[];
  delivered: boolean;
  method: "cliclick" | "osascript";
  focusEnforced: boolean;
  /** Automation surface targeted, if --target was specified. */
  target?: string;
  /** Session used for focus, if --session was specified. */
  session?: string;
}

interface TypeResult {
  text: string;
  delivered: boolean;
  method: "cliclick" | "osascript";
  focusEnforced: boolean;
  target?: string;
  session?: string;
}

interface ClickResult {
  x: number;
  y: number;
  delivered: boolean;
  focusEnforced: boolean;
  target?: string;
  session?: string;
}

interface SequenceStep {
  action: "key" | "type" | "click" | "sleep";
  key?: string;
  modifiers?: string[];
  text?: string;
  x?: number;
  y?: number;
  ms?: number;
}

interface SequenceResult {
  steps: number;
  completed: number;
  focusEnforced: boolean;
  results: Array<KeyResult | TypeResult | ClickResult | { sleptMs: number }>;
}

interface CheckResult {
  cliclick: boolean;
  cliclickPath: string | null;
  osascript: boolean;
  accessibility: boolean | "unknown";
  windowTsAvailable: boolean;
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

// cliclick key name mapping
const KEY_MAP: Record<string, string> = {
  enter: "return",
  return: "return",
  tab: "tab",
  escape: "esc",
  esc: "esc",
  space: "space",
  delete: "delete",
  backspace: "delete",
  up: "arrow-up",
  down: "arrow-down",
  left: "arrow-left",
  right: "arrow-right",
  home: "home",
  end: "end",
  pageup: "page-up",
  pagedown: "page-down",
  f1: "f1",
  f2: "f2",
  f3: "f3",
  f4: "f4",
  f5: "f5",
  f6: "f6",
  f7: "f7",
  f8: "f8",
  f9: "f9",
  f10: "f10",
  f11: "f11",
  f12: "f12",
};

// osascript key code mapping for keys that need it
const OSASCRIPT_KEY_CODES: Record<string, number> = {
  enter: 36,
  return: 36,
  tab: 48,
  escape: 53,
  esc: 53,
  space: 49,
  delete: 51,
  backspace: 51,
  up: 126,
  down: 125,
  left: 123,
  right: 124,
};

function findCliclick(): string | null {
  const paths = ["/opt/homebrew/bin/cliclick", "/usr/local/bin/cliclick"];
  for (const p of paths) {
    if (existsSync(p)) return p;
  }
  return null;
}

async function runProcess(
  cmd: string[],
  label: string
): Promise<{ stdout: string; stderr: string; exitCode: number }> {
  const proc = Bun.spawn(cmd, {
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const exitCode = await proc.exited;
  return { stdout: stdout.trim(), stderr: stderr.trim(), exitCode };
}

// ---------------------------------------------------------------------------
// Session-aware focus: show window via session.sh then verify with window.ts
// ---------------------------------------------------------------------------

const SESSION_SH_PATH = new URL("../../scripts/agentic/session.sh", import.meta.url).pathname;

/**
 * Show the app window via session.sh send, then delegate to window.ts focus.
 * This is more reliable than OS-level activation alone because it uses the
 * app's own show protocol to ensure the window is visible before focusing.
 */
async function ensureFocusViaSession(
  sessionName: string,
  targetSurface: string | null,
  retry: number = DEFAULT_FOCUS_RETRY,
  settleMs: number = DEFAULT_FOCUS_SETTLE_MS
): Promise<{ frontmost: boolean; focused: boolean; surfaceId: string | null }> {
  stderrLog("session_focus_start", { sessionName, targetSurface, retry, settleMs });

  // 1. Send show command via session to ensure window is visible
  const { exitCode: showCode, stderr: showErr } = await runProcess(
    ["bash", SESSION_SH_PATH, "send", sessionName, '{"type":"show"}'],
    "session-show"
  );
  if (showCode !== 0) {
    stderrLog("session_show_failed", { exitCode: showCode, stderr: showErr.slice(0, 200) });
  }

  // Brief settle for window to appear
  await Bun.sleep(150);

  // 2. If target surface specified, resolve via window.ts list to find the right title
  let focusTitle = DEFAULT_FOCUS_TITLE;
  let resolvedSurfaceId: string | null = null;

  if (targetSurface) {
    const { stdout: listOut } = await runProcess(
      ["bun", WINDOW_TS_PATH, "list"],
      "window-list"
    );
    try {
      const listResult = JSON.parse(listOut);
      const surfaces = listResult?.data?.surfaces ?? [];
      const match = surfaces.find(
        (s: any) => s.surfaceId === targetSurface || s.surfaceId.startsWith(targetSurface + ":")
      );
      if (match) {
        resolvedSurfaceId = match.surfaceId;
        // Use title for focused targeting if available
        if (match.title) {
          focusTitle = match.title;
        }
      } else {
        stderrLog("session_focus_target_not_found", {
          targetSurface,
          availableSurfaces: surfaces.map((s: any) => s.surfaceId),
        });
      }
    } catch {
      stderrLog("session_focus_list_parse_error", { stdout: listOut.slice(0, 200) });
    }
  }

  // 3. Delegate actual focus to window.ts
  const focusResult = await ensureFocusViaWindowTs(focusTitle, retry, settleMs);

  // Emit structured session focus resolution event for autonomous verification
  stderrLog("session_focus_resolved", {
    sessionName,
    targetSurface,
    resolvedSurfaceId,
    focusTitle,
    frontmost: focusResult.frontmost,
    focused: focusResult.focused,
  });

  return { ...focusResult, surfaceId: resolvedSurfaceId };
}

// ---------------------------------------------------------------------------
// Focus delegation to window.ts
// ---------------------------------------------------------------------------

/**
 * Delegate focus enforcement to window.ts. Returns true if the window was
 * confirmed frontmost, false otherwise. Throws on hard errors.
 */
async function ensureFocusViaWindowTs(
  focusTitle: string,
  retry: number = DEFAULT_FOCUS_RETRY,
  settleMs: number = DEFAULT_FOCUS_SETTLE_MS
): Promise<{ frontmost: boolean; focused: boolean }> {
  stderrLog("focus_delegate_start", { focusTitle, retry, settleMs });

  const { stdout, stderr, exitCode } = await runProcess(
    [
      "bun",
      WINDOW_TS_PATH,
      "focus",
      "--title",
      focusTitle,
      "--retry",
      String(retry),
      "--settle-ms",
      String(settleMs),
    ],
    "window.ts-focus"
  );

  // Parse the window.ts JSON envelope
  let parsed: any = null;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    stderrLog("focus_delegate_parse_error", { stdout: stdout.slice(0, 200) });
  }

  const frontmost = parsed?.data?.frontmost ?? false;
  const focused = parsed?.data?.focused ?? false;

  stderrLog("focus_delegate_result", {
    exitCode,
    frontmost,
    focused,
    windowId: parsed?.data?.windowId ?? 0,
  });

  if (exitCode !== 0 && !frontmost) {
    // Non-fatal: we tried but may not be frontmost
    stderrLog("focus_delegate_warning", {
      message: "window.ts focus did not confirm frontmost",
    });
  }

  return { frontmost, focused };
}

// ---------------------------------------------------------------------------
// Input delivery
// ---------------------------------------------------------------------------

async function sendKey(
  key: string,
  modifiers: string[],
  focusEnforced: boolean
): Promise<KeyResult> {
  const cliclick = findCliclick();
  const keyLower = key.toLowerCase();

  // Try cliclick first for simple keys
  if (cliclick) {
    const cliclickKey = KEY_MAP[keyLower] ?? key;
    const args: string[] = [];

    if (KEY_MAP[keyLower]) {
      if (modifiers.length > 0) {
        // cliclick doesn't support modifiers with kp, fall through to osascript
      } else {
        args.push(`kp:${cliclickKey}`);
      }
    } else if (key.length === 1 && modifiers.length === 0) {
      args.push(`t:${key}`);
    }

    if (args.length > 0) {
      stderrLog("input_key_attempt", { key, modifiers, method: "cliclick" });
      const { exitCode, stderr } = await runProcess(
        [cliclick, "-w", "10", ...args],
        "cliclick-key"
      );
      if (exitCode === 0) {
        return { key, modifiers, delivered: true, method: "cliclick", focusEnforced };
      }
      stderrLog("input_key_cliclick_failed", { key, stderr });
    }
  }

  // Fallback: osascript
  const keyCode = OSASCRIPT_KEY_CODES[keyLower];
  let script: string;

  if (keyCode !== undefined) {
    const modParts: string[] = [];
    for (const m of modifiers) {
      switch (m.toLowerCase()) {
        case "cmd":
        case "command":
          modParts.push("command down");
          break;
        case "shift":
          modParts.push("shift down");
          break;
        case "alt":
        case "option":
          modParts.push("option down");
          break;
        case "ctrl":
        case "control":
          modParts.push("control down");
          break;
      }
    }
    const using = modParts.length > 0 ? ` using {${modParts.join(", ")}}` : "";
    script = `tell application "System Events" to key code ${keyCode}${using}`;
  } else if (key.length === 1) {
    const modParts: string[] = [];
    for (const m of modifiers) {
      switch (m.toLowerCase()) {
        case "cmd":
        case "command":
          modParts.push("command down");
          break;
        case "shift":
          modParts.push("shift down");
          break;
        case "alt":
        case "option":
          modParts.push("option down");
          break;
        case "ctrl":
        case "control":
          modParts.push("control down");
          break;
      }
    }
    const using = modParts.length > 0 ? ` using {${modParts.join(", ")}}` : "";
    script = `tell application "System Events" to keystroke "${key}"${using}`;
  } else {
    throw Object.assign(
      new Error(`Unknown key: ${key}. Use a single character or named key.`),
      { code: "UNKNOWN_KEY" }
    );
  }

  stderrLog("input_key_attempt", { key, modifiers, method: "osascript" });
  const { exitCode, stderr } = await runProcess(
    ["osascript", "-e", script],
    "osascript-key"
  );

  if (exitCode !== 0) {
    if (
      stderr.includes("not allowed assistive access") ||
      stderr.includes("(-1743)")
    ) {
      throw Object.assign(new Error(stderr), { code: "ACCESSIBILITY_DENIED" });
    }
    throw Object.assign(new Error(`Key send failed: ${stderr}`), {
      code: "KEY_FAILED",
    });
  }

  return { key, modifiers, delivered: true, method: "osascript", focusEnforced };
}

async function sendType(text: string, focusEnforced: boolean): Promise<TypeResult> {
  const cliclick = findCliclick();

  if (cliclick) {
    stderrLog("input_type_attempt", { textLen: text.length, method: "cliclick" });
    const { exitCode, stderr } = await runProcess(
      [cliclick, "-w", "10", `t:${text}`],
      "cliclick-type"
    );
    if (exitCode === 0) {
      return { text, delivered: true, method: "cliclick", focusEnforced };
    }
    stderrLog("input_type_cliclick_failed", { stderr });
  }

  // Fallback: osascript keystroke
  const escaped = text.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  stderrLog("input_type_attempt", { textLen: text.length, method: "osascript" });
  const { exitCode, stderr } = await runProcess(
    [
      "osascript",
      "-e",
      `tell application "System Events" to keystroke "${escaped}"`,
    ],
    "osascript-type"
  );

  if (exitCode !== 0) {
    if (
      stderr.includes("not allowed assistive access") ||
      stderr.includes("(-1743)")
    ) {
      throw Object.assign(new Error(stderr), { code: "ACCESSIBILITY_DENIED" });
    }
    throw Object.assign(new Error(`Type failed: ${stderr}`), {
      code: "TYPE_FAILED",
    });
  }

  return { text, delivered: true, method: "osascript", focusEnforced };
}

async function sendClick(x: number, y: number, focusEnforced: boolean): Promise<ClickResult> {
  const cliclick = findCliclick();

  if (cliclick) {
    stderrLog("input_click_attempt", { x, y, method: "cliclick" });
    const { exitCode, stderr } = await runProcess(
      [cliclick, `c:${x},${y}`],
      "cliclick-click"
    );
    if (exitCode === 0) {
      return { x, y, delivered: true, focusEnforced };
    }
    stderrLog("input_click_cliclick_failed", { stderr });
  }

  stderrLog("input_click_attempt", { x, y, method: "osascript" });
  const { exitCode, stderr } = await runProcess(
    [
      "osascript",
      "-e",
      `tell application "System Events"
  click at {${x}, ${y}}
end tell`,
    ],
    "osascript-click"
  );

  if (exitCode !== 0) {
    throw Object.assign(new Error(`Click failed: ${stderr}`), {
      code: "CLICK_FAILED",
    });
  }

  return { x, y, delivered: true, focusEnforced };
}

async function runSequence(
  steps: SequenceStep[],
  focusEnforced: boolean
): Promise<SequenceResult> {
  const results: SequenceResult["results"] = [];
  let completed = 0;

  for (const step of steps) {
    switch (step.action) {
      case "key":
        results.push(await sendKey(step.key ?? "enter", step.modifiers ?? [], focusEnforced));
        completed++;
        break;
      case "type":
        results.push(await sendType(step.text ?? "", focusEnforced));
        completed++;
        break;
      case "click":
        results.push(await sendClick(step.x ?? 0, step.y ?? 0, focusEnforced));
        completed++;
        break;
      case "sleep":
        await Bun.sleep(step.ms ?? 100);
        results.push({ sleptMs: step.ms ?? 100 });
        completed++;
        break;
    }
  }

  return { steps: steps.length, completed, focusEnforced, results };
}

// ---------------------------------------------------------------------------
// Preflight checks
// ---------------------------------------------------------------------------

async function checkPrerequisites(): Promise<CheckResult> {
  const cliclickPath = findCliclick();
  const cliclick = cliclickPath !== null;
  const osascript = existsSync("/usr/bin/osascript");
  const windowTsAvailable = existsSync(WINDOW_TS_PATH);

  let accessibility: boolean | "unknown" = "unknown";
  try {
    const { exitCode, stderr } = await runProcess(
      [
        "osascript",
        "-e",
        'tell application "System Events" to get name of first process whose frontmost is true',
      ],
      "ax-check"
    );
    if (exitCode === 0) {
      accessibility = true;
    } else if (
      stderr.includes("not allowed assistive access") ||
      stderr.includes("(-1743)")
    ) {
      accessibility = false;
    }
  } catch {
    // unknown
  }

  return { cliclick, cliclickPath, osascript, accessibility, windowTsAvailable };
}

// ---------------------------------------------------------------------------
// CLI argument parsing
// ---------------------------------------------------------------------------

function hasFlag(args: string[], flag: string): boolean {
  return args.includes(flag);
}

function getStringArg(args: string[], flag: string, fallback: string): string {
  const idx = args.indexOf(flag);
  if (idx >= 0 && args[idx + 1]) return args[idx + 1];
  return fallback;
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const args = process.argv.slice(2);
const command = args[0] ?? "help";

// Global focus flags
const ensureFocus = hasFlag(args, "--ensure-focus");
const focusTitle = getStringArg(args, "--focus-title", DEFAULT_FOCUS_TITLE);
const sessionName = getStringArg(args, "--session", "");
const targetSurface = getStringArg(args, "--target", "");

function emit(data: Envelope<any>) {
  console.log(JSON.stringify(data, null, 2));
}

// Run focus enforcement once before any command if requested
let focusEnforced = false;
let resolvedTarget: string | null = null;
if (ensureFocus && command !== "check" && command !== "help" && command !== "--help") {
  let frontmost = false;
  let focused = false;

  if (sessionName) {
    // Session-aware focus: show via session, then verify with window.ts
    const result = await ensureFocusViaSession(
      sessionName,
      targetSurface || null
    );
    frontmost = result.frontmost;
    focused = result.focused;
    resolvedTarget = result.surfaceId;
  } else {
    // Legacy focus: directly via window.ts
    const result = await ensureFocusViaWindowTs(focusTitle);
    frontmost = result.frontmost;
    focused = result.focused;
  }

  focusEnforced = true;
  if (!frontmost) {
    stderrLog("focus_enforcement_failed", {
      frontmost,
      focused,
      focusTitle,
      sessionName: sessionName || undefined,
      targetSurface: targetSurface || undefined,
    });
    emit({
      schemaVersion: SCHEMA_VERSION,
      status: "error",
      command,
      error: {
        code: "FOCUS_NOT_CONFIRMED",
        message: sessionName
          ? `Script Kit window was not frontmost after session-based focus (session: ${sessionName}${targetSurface ? `, target: ${targetSurface}` : ""})`
          : "Script Kit window was not frontmost after focus enforcement",
      },
      data: {
        frontmost,
        focused,
        focusTitle,
        ...(sessionName ? { session: sessionName } : {}),
        ...(targetSurface ? { target: targetSurface } : {}),
        ...(resolvedTarget ? { resolvedTarget } : {}),
      },
    } as Envelope<any>);
    process.exit(1);
  }
}

try {
  switch (command) {
    case "key": {
      const keyName = args[1];
      if (!keyName) {
        emit(
          errorEnvelope(
            "key",
            "MISSING_KEY",
            "Usage: macos-input.ts key <keyname> [--modifiers cmd,shift] [--ensure-focus] [--session NAME] [--target SURFACE]"
          )
        );
        process.exit(1);
        break;
      }
      const modIdx = args.indexOf("--modifiers");
      const modifiers =
        modIdx >= 0 && args[modIdx + 1]
          ? args[modIdx + 1].split(",").map((m) => m.trim())
          : [];
      const result = await sendKey(keyName, modifiers, focusEnforced);
      if (sessionName) result.session = sessionName;
      if (resolvedTarget) result.target = resolvedTarget;
      emit(envelope("key", result));
      process.exit(result.delivered ? 0 : 1);
      break;
    }

    case "type": {
      const text = args[1];
      if (!text) {
        emit(
          errorEnvelope(
            "type",
            "MISSING_TEXT",
            "Usage: macos-input.ts type <text> [--ensure-focus] [--session NAME] [--target SURFACE]"
          )
        );
        process.exit(1);
        break;
      }
      const result = await sendType(text, focusEnforced);
      if (sessionName) result.session = sessionName;
      if (resolvedTarget) result.target = resolvedTarget;
      emit(envelope("type", result));
      process.exit(result.delivered ? 0 : 1);
      break;
    }

    case "click": {
      const x = parseInt(args[1], 10);
      const y = parseInt(args[2], 10);
      if (isNaN(x) || isNaN(y)) {
        emit(
          errorEnvelope(
            "click",
            "MISSING_COORDS",
            "Usage: macos-input.ts click <x> <y> [--ensure-focus] [--session NAME] [--target SURFACE]"
          )
        );
        process.exit(1);
        break;
      }
      const result = await sendClick(x, y, focusEnforced);
      if (sessionName) result.session = sessionName;
      if (resolvedTarget) result.target = resolvedTarget;
      emit(envelope("click", result));
      process.exit(result.delivered ? 0 : 1);
      break;
    }

    case "sequence": {
      const jsonStr = args[1];
      if (!jsonStr) {
        emit(
          errorEnvelope(
            "sequence",
            "MISSING_STEPS",
            'Usage: macos-input.ts sequence \'[{"action":"key","key":"tab"},{"action":"sleep","ms":200},{"action":"key","key":"enter"}]\' [--ensure-focus]'
          )
        );
        process.exit(1);
        break;
      }
      let steps: SequenceStep[];
      try {
        steps = JSON.parse(jsonStr);
      } catch (e) {
        emit(
          errorEnvelope(
            "sequence",
            "INVALID_JSON",
            `Failed to parse sequence JSON: ${e}`
          )
        );
        process.exit(1);
        break;
      }
      const result = await runSequence(steps, focusEnforced);
      emit(envelope("sequence", result));
      process.exit(result.completed === result.steps ? 0 : 1);
      break;
    }

    case "check": {
      const prereqs = await checkPrerequisites();
      emit(envelope("check", prereqs));
      if (!prereqs.cliclick) {
        stderrLog("check_warning", { message: "cliclick not found. Install: brew install cliclick" });
      }
      if (prereqs.accessibility === false) {
        stderrLog("check_warning", {
          message: "Accessibility denied. Fix: System Preferences → Privacy & Security → Accessibility → enable terminal/IDE",
        });
      }
      if (!prereqs.windowTsAvailable) {
        stderrLog("check_warning", { message: "window.ts not found — focus delegation unavailable" });
      }
      process.exit(
        prereqs.cliclick && prereqs.accessibility !== false ? 0 : 1
      );
      break;
    }

    case "help":
    case "--help": {
      console.log(`Usage: bun scripts/agentic/macos-input.ts <command> [options]

Commands:
  key <name> [--modifiers cmd,shift,alt,ctrl]   Send a keystroke
  type <text>                                     Type text string
  click <x> <y>                                   Click at screen coordinates
  sequence '<json-array>'                         Run a sequence of actions
  check                                           Verify prerequisites

Focus enforcement:
  --ensure-focus                 Focus Script Kit window before input (delegates to window.ts)
  --focus-title SUBSTR           Title substring for focus target (default: "${DEFAULT_FOCUS_TITLE}")
  --session NAME                 Use session.sh to show window before focusing (more reliable)
  --target SURFACE               Target a specific automation surface (main, acp, actions, notes, ai)
                                 Requires --ensure-focus. Resolves via window.ts list.

Named keys: enter, tab, escape, space, delete, backspace,
            up, down, left, right, home, end, pageup, pagedown,
            f1-f12

Sequence JSON format:
  [{"action":"key","key":"tab"},
   {"action":"sleep","ms":200},
   {"action":"type","text":"hello"},
   {"action":"key","key":"enter"},
   {"action":"click","x":100,"y":200}]

Output:
  Schema version ${SCHEMA_VERSION} JSON envelopes on stdout.
  Structured NDJSON diagnostics on stderr.
  Exit 0 = delivered, 1 = failed or missing prerequisites.

Focus delegation:
  When --ensure-focus is set, focus is enforced via window.ts (retry 3, settle 200ms).
  When --session is also set, the window is first shown via session.sh send before focusing.
  When --target is set, the target automation surface is resolved via window.ts list.
  The focusEnforced, session, and target fields in the response confirm the focus path used.

Permission requirements:
  - Accessibility: System Preferences → Privacy & Security → Accessibility
  - cliclick: brew install cliclick`);
      process.exit(0);
      break;
    }

    default:
      emit(
        errorEnvelope(
          "unknown",
          "UNKNOWN_COMMAND",
          `Unknown command: ${command}`
        )
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
