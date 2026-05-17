#!/usr/bin/env bun
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "builtin-command-text-matrix");
const limit = Number(argValue("--limit", "50"));
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "50"));
const outputDir = join(repoRoot, ".test-output", "builtin-command-text-matrix");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const sessionRoot = join(outputDir, "sessions");
const unsafeCommandIdParts = [
  "restart",
  "shut-down",
  "empty-trash",
  "force-quit",
  "stop-all-processes",
  "clear-suggested",
  "sleep",
  "log-out",
  "lock-screen",
  "quit-script-kit",
  "test-confirmation",
];

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
process.env.SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN = "1";

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function run(command: string, args: string[], options: { input?: string } = {}): string {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
    input: options.input,
  });
  if (result.status !== 0) {
    throw new Error(
      `${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
    );
  }
  return result.stdout;
}

function runSession(args: string[]): Json {
  const stdout = run(sessionScript, args).trim();
  if (!stdout) {
    throw new Error(`session.sh ${args.join(" ")} produced no stdout`);
  }
  const parsed = JSON.parse(stdout);
  if (parsed.status === "error") {
    throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}`);
  }
  return parsed;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  return runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeout),
  ]).response;
}

function send(command: Json): Json {
  return runSession([
    "send",
    session,
    JSON.stringify(command),
    "--await-parse",
    "--timeout",
    String(timeoutMs),
  ]);
}

function getState(tag: string): Json {
  return rpc(
    {
      type: "getState",
      requestId: `builtin-command-text-state-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
}

function waitForInput(input: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `builtin-command-text-wait-${Date.now()}`,
      condition: {
        type: "stateMatch",
        state: {
          promptType: "none",
          inputValue: input,
        },
      },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function selectedVisibleResult(preflight: Json): Json | undefined {
  const key = preflight.selectedResultKey;
  return (preflight.visibleResults ?? []).find((row: Json) => row.stableKey === key)
    ?? (preflight.visibleResults ?? [])[0];
}

function isSafeBuiltin(row: Json): boolean {
  const stableKey = String(row.stableKey ?? "");
  if (row.typeLabel !== "Built-in" || !stableKey.startsWith("builtin/")) {
    return false;
  }
  return !unsafeCommandIdParts.some((part) => stableKey.includes(part));
}

async function openCommandSource() {
  const input = "cmd:";
  send({ type: "show", requestId: `builtin-command-text-show-${Date.now()}` });
  send({ type: "setFilter", text: input, requestId: `builtin-command-text-set-${Date.now()}` });
  waitForInput(input);
  await Bun.sleep(Math.max(700, pollMs));
}

function currentSelectedState(tag: string): { state: Json; selected: Json; enterAction: Json } {
  const selectedState = getState(tag);
  const selected = selectedVisibleResult(selectedState.mainWindowPreflight ?? {});
  if (!selected) {
    throw new Error(`${tag}: no selected visible result`);
  }
  const enterAction = selectedState.mainWindowPreflight?.enterAction;
  if (!enterAction?.label) {
    throw new Error(`${selected.stableKey}: missing enterAction label`);
  }
  return { state: selectedState, selected, enterAction };
}

async function inspectSelected(selected: Json, enterAction: Json): Promise<Json> {
  send({ type: "simulateKey", key: "k", modifiers: ["cmd"], requestId: `builtin-command-text-actions-${Date.now()}` });
  let lastDialog: Json | undefined;
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const state = getState(`actions-${selected.stableKey.replace(/[^a-z0-9]+/gi, "-")}`);
    const dialog = state.actionsDialog;
    lastDialog = dialog;
    const primary = (dialog?.visibleActions ?? []).find((action: Json) => action.id === "run_script");
    if (dialog?.open && primary) {
      if (primary.label !== enterAction.label) {
        throw new Error(
          `${selected.stableKey}: primary action label ${primary.label} did not match enter label ${enterAction.label}`,
        );
      }
      if (primary.destructive) {
        throw new Error(`${selected.stableKey}: primary action unexpectedly marked destructive`);
      }
      send({ type: "simulateKey", key: "escape", modifiers: [], requestId: `builtin-command-text-escape-${Date.now()}` });
      await Bun.sleep(200);
      return {
        stableKey: selected.stableKey,
        visibleRank: selected.visibleRank,
        subject: enterAction.subject ?? null,
        enterLabel: enterAction.label,
        primaryActionLabel: primary.label,
        actionCount: dialog.visibleActions.length,
      };
    }
    await Bun.sleep(pollMs);
  }
  throw new Error(`${selected.stableKey}: timed out waiting for actions dialog, last=${JSON.stringify(lastDialog)}`);
}

async function moveDown() {
  send({ type: "simulateKey", key: "down", modifiers: [], requestId: `builtin-command-text-down-${Date.now()}` });
  await Bun.sleep(Math.max(120, pollMs));
}

async function main() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(kitDir, { recursive: true });
  runSession(["stop", session]);
  runSession(["start", session]);

  try {
    await openCommandSource();
    const inspected = [];
    const skippedUnsafe = [];
    const seen = new Set<string>();
    let attempts = 0;
    while (inspected.length < limit && attempts < 140) {
      attempts += 1;
      const { selected, enterAction } = currentSelectedState(`selected-${attempts}`);
      if (!seen.has(selected.stableKey)) {
        seen.add(selected.stableKey);
        if (isSafeBuiltin(selected)) {
          inspected.push(await inspectSelected(selected, enterAction));
        } else if (String(selected.stableKey ?? "").startsWith("builtin/")) {
          skippedUnsafe.push(selected.stableKey);
        }
      }
      await moveDown();
    }
    if (inspected.length < limit) {
      throw new Error(`Expected to inspect ${limit} safe built-ins, inspected ${inspected.length}`);
    }
    const receipt = {
      schemaVersion: 1,
      status: "pass",
      session,
      limit,
      inspectedCount: inspected.length,
      skippedUnsafeIdParts: unsafeCommandIdParts,
      skippedUnsafe,
      inspected,
    };
    const text = `${JSON.stringify(receipt, null, 2)}\n`;
    writeFileSync(join(outputDir, "receipt.json"), text);
    writeFileSync(join(repoRoot, ".test-output", "builtin-command-text-matrix.json"), text);
    process.stdout.write(text);
  } finally {
    runSession(["stop", session]);
  }
}

main().catch((error) => {
  const message = error instanceof Error ? error.stack ?? error.message : String(error);
  mkdirSync(join(repoRoot, ".test-output"), { recursive: true });
  writeFileSync(
    join(repoRoot, ".test-output", "builtin-command-text-matrix.json"),
    `${JSON.stringify({ schemaVersion: 1, status: "fail", session, error: message }, null, 2)}\n`,
  );
  process.stderr.write(`${message}\n`);
  process.exit(1);
});
