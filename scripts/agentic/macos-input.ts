#!/usr/bin/env bun
/**
 * scripts/agentic/macos-input.ts
 *
 * Native macOS keyboard and mouse automation for Script Kit GPUI agentic testing.
 * Uses cliclick for input delivery and osascript for activation fallback.
 *
 * Usage:
 *   bun scripts/agentic/macos-input.ts key <keyname> [--modifiers cmd,shift,...] [--json]
 *   bun scripts/agentic/macos-input.ts type <text> [--json]
 *   bun scripts/agentic/macos-input.ts click <x> <y> [--json]
 *   bun scripts/agentic/macos-input.ts sequence <json-array> [--json]
 *
 * All structured output is JSON on stdout. Diagnostics go to stderr.
 */

import { existsSync } from "fs";

const SCHEMA_VERSION = 1;

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
}

interface TypeResult {
  text: string;
  delivered: boolean;
  method: "cliclick" | "osascript";
}

interface ClickResult {
  x: number;
  y: number;
  delivered: boolean;
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
  results: Array<KeyResult | TypeResult | ClickResult | { sleptMs: number }>;
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
  const paths = [
    "/opt/homebrew/bin/cliclick",
    "/usr/local/bin/cliclick",
  ];
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

async function verifyWindowFocused(): Promise<boolean> {
  try {
    const { stdout, exitCode } = await runProcess(
      [
        "osascript",
        "-e",
        'tell application "System Events" to get name of first process whose frontmost is true',
      ],
      "check-frontmost"
    );
    return exitCode === 0 && stdout.includes("Script Kit");
  } catch {
    return false;
  }
}

async function focusAppIfNeeded(): Promise<void> {
  const focused = await verifyWindowFocused();
  if (!focused) {
    console.error("[macos-input.ts] Script Kit not frontmost, activating...");
    await runProcess(
      [
        "osascript",
        "-e",
        `tell application "System Events"
  set appProcs to every process whose name contains "Script Kit"
  if (count of appProcs) > 0 then
    set frontmost of item 1 of appProcs to true
  end if
end tell`,
      ],
      "activate"
    );
    await Bun.sleep(300);
  }
}

// ---------------------------------------------------------------------------
// Input delivery
// ---------------------------------------------------------------------------

async function sendKey(
  key: string,
  modifiers: string[]
): Promise<KeyResult> {
  await focusAppIfNeeded();

  const cliclick = findCliclick();
  const keyLower = key.toLowerCase();

  // Try cliclick first for simple keys
  if (cliclick) {
    const cliclickKey = KEY_MAP[keyLower] ?? key;

    // Build cliclick key press command
    // cliclick kp:<key> for special keys, t:<char> for characters
    const args: string[] = [];

    if (KEY_MAP[keyLower]) {
      // Special key
      if (modifiers.length > 0) {
        // cliclick doesn't support modifiers with kp, fall through to osascript
      } else {
        args.push(`kp:${cliclickKey}`);
      }
    } else if (key.length === 1 && modifiers.length === 0) {
      // Single character — use type
      args.push(`t:${key}`);
    }

    if (args.length > 0) {
      const { exitCode, stderr } = await runProcess(
        [cliclick, "-w", "10", ...args],
        "cliclick-key"
      );
      if (exitCode === 0) {
        return {
          key,
          modifiers,
          delivered: true,
          method: "cliclick",
        };
      }
      console.error(`[macos-input.ts] cliclick failed: ${stderr}`);
    }
  }

  // Fallback: osascript
  const keyCode = OSASCRIPT_KEY_CODES[keyLower];
  let script: string;

  if (keyCode !== undefined) {
    // Use key code for special keys
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
    const using =
      modParts.length > 0 ? ` using {${modParts.join(", ")}}` : "";
    script = `tell application "System Events" to key code ${keyCode}${using}`;
  } else if (key.length === 1) {
    // Single character keystroke
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
    const using =
      modParts.length > 0 ? ` using {${modParts.join(", ")}}` : "";
    script = `tell application "System Events" to keystroke "${key}"${using}`;
  } else {
    throw Object.assign(
      new Error(`Unknown key: ${key}. Use a single character or named key.`),
      { code: "UNKNOWN_KEY" }
    );
  }

  const { exitCode, stderr } = await runProcess(
    ["osascript", "-e", script],
    "osascript-key"
  );

  if (exitCode !== 0) {
    if (
      stderr.includes("not allowed assistive access") ||
      stderr.includes("(-1743)")
    ) {
      throw Object.assign(new Error(stderr), {
        code: "ACCESSIBILITY_DENIED",
      });
    }
    throw Object.assign(new Error(`Key send failed: ${stderr}`), {
      code: "KEY_FAILED",
    });
  }

  return {
    key,
    modifiers,
    delivered: true,
    method: "osascript",
  };
}

async function sendType(text: string): Promise<TypeResult> {
  await focusAppIfNeeded();

  const cliclick = findCliclick();

  // cliclick t: handles arbitrary text well
  if (cliclick) {
    const { exitCode, stderr } = await runProcess(
      [cliclick, "-w", "10", `t:${text}`],
      "cliclick-type"
    );
    if (exitCode === 0) {
      return { text, delivered: true, method: "cliclick" };
    }
    console.error(`[macos-input.ts] cliclick type failed: ${stderr}`);
  }

  // Fallback: osascript keystroke (escaping for AppleScript)
  const escaped = text.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
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
      throw Object.assign(new Error(stderr), {
        code: "ACCESSIBILITY_DENIED",
      });
    }
    throw Object.assign(new Error(`Type failed: ${stderr}`), {
      code: "TYPE_FAILED",
    });
  }

  return { text, delivered: true, method: "osascript" };
}

async function sendClick(x: number, y: number): Promise<ClickResult> {
  await focusAppIfNeeded();

  const cliclick = findCliclick();

  if (cliclick) {
    const { exitCode, stderr } = await runProcess(
      [cliclick, `c:${x},${y}`],
      "cliclick-click"
    );
    if (exitCode === 0) {
      return { x, y, delivered: true };
    }
    console.error(`[macos-input.ts] cliclick click failed: ${stderr}`);
  }

  // Fallback: osascript click
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

  return { x, y, delivered: true };
}

async function runSequence(steps: SequenceStep[]): Promise<SequenceResult> {
  const results: SequenceResult["results"] = [];
  let completed = 0;

  for (const step of steps) {
    switch (step.action) {
      case "key":
        results.push(await sendKey(step.key ?? "enter", step.modifiers ?? []));
        completed++;
        break;
      case "type":
        results.push(await sendType(step.text ?? ""));
        completed++;
        break;
      case "click":
        results.push(await sendClick(step.x ?? 0, step.y ?? 0));
        completed++;
        break;
      case "sleep":
        await Bun.sleep(step.ms ?? 100);
        results.push({ sleptMs: step.ms ?? 100 });
        completed++;
        break;
    }
  }

  return { steps: steps.length, completed, results };
}

// ---------------------------------------------------------------------------
// Preflight checks
// ---------------------------------------------------------------------------

async function checkPrerequisites(): Promise<{
  cliclick: boolean;
  cliclickPath: string | null;
  osascript: boolean;
  accessibility: boolean | "unknown";
}> {
  const cliclickPath = findCliclick();
  const cliclick = cliclickPath !== null;
  const osascript = existsSync("/usr/bin/osascript");

  // Test accessibility by trying a no-op osascript
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

  return { cliclick, cliclickPath, osascript, accessibility };
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

const args = process.argv.slice(2);
const command = args[0] ?? "help";

function emit(data: Envelope<any>) {
  console.log(JSON.stringify(data, null, 2));
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
            "Usage: macos-input.ts key <keyname> [--modifiers cmd,shift]"
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
      const result = await sendKey(keyName, modifiers);
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
            "Usage: macos-input.ts type <text>"
          )
        );
        process.exit(1);
        break;
      }
      const result = await sendType(text);
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
            "Usage: macos-input.ts click <x> <y>"
          )
        );
        process.exit(1);
        break;
      }
      const result = await sendClick(x, y);
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
            'Usage: macos-input.ts sequence \'[{"action":"key","key":"tab"},{"action":"sleep","ms":200},{"action":"key","key":"enter"}]\''
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
      const result = await runSequence(steps);
      emit(envelope("sequence", result));
      process.exit(result.completed === result.steps ? 0 : 1);
      break;
    }

    case "check": {
      const prereqs = await checkPrerequisites();
      emit(envelope("check", prereqs));
      if (!prereqs.cliclick) {
        console.error(
          "[macos-input.ts] cliclick not found. Install: brew install cliclick"
        );
      }
      if (prereqs.accessibility === false) {
        console.error(
          "[macos-input.ts] Accessibility denied. Fix: System Preferences → Privacy & Security → Accessibility → enable terminal/IDE"
        );
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

Named keys: enter, tab, escape, space, delete, backspace,
            up, down, left, right, home, end, pageup, pagedown,
            f1-f12

Sequence JSON format:
  [{"action":"key","key":"tab"},
   {"action":"sleep","ms":200},
   {"action":"type","text":"hello"},
   {"action":"key","key":"enter"},
   {"action":"click","x":100,"y":200}]

The target window is auto-focused before input. Uses cliclick when
available, falls back to osascript.

Permission requirements:
  - Accessibility: System Preferences → Privacy & Security → Accessibility
  - cliclick: brew install cliclick

Exit 0 = delivered, 1 = failed or missing prerequisites.`);
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
