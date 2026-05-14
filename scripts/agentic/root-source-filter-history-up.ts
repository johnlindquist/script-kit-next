#!/usr/bin/env bun
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "root-source-filter-history-up");
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = 50;
const outputDir = join(repoRoot, ".test-output", "root-source-filter-history-up");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const sessionRoot = join(outputDir, "sessions");
const historyEntry = "history recall sentinel";

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";

const sourceHeads = [
  "f:", "files:",
  "n:", "notes:",
  "c:", "clipboard:",
  "d:", "dictation:",
  "ai:", "conversations:",
  "h:", "history:",
  "t:", "tabs:",
  "a:", "apps:",
  "s:", "scripts:",
  "cmd:", "commands:",
  "v:", "vault:",
  "w:", "windows:",
  "p:", "processes:",
];

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function runSession(args: string[]): Json {
  const result = spawnSync(sessionScript, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
  });
  const stdout = result.stdout.trim();
  if (!stdout) {
    throw new Error(`session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`);
  }
  const parsed = JSON.parse(stdout);
  if (result.status !== 0 || parsed.status === "error") {
    throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}\nstderr=${result.stderr.trim()}`);
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

function showWindow(tag: string): void {
  rpc({ type: "show", requestId: `source-history-show-${tag}-${Date.now()}` }, "windowVisibilityAck");
}

function waitForInput(input: string, tag: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `source-history-wait-${tag}-${Date.now()}`,
      condition: { type: "stateMatch", state: { promptType: "none", inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function getState(tag: string): Json {
  const response = rpc(
    { type: "getState", requestId: `source-history-state-${tag}-${Date.now()}` },
    "stateResult",
  );
  if (response.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(response)}`);
  }
  return response;
}

function pressUp(tag: string): void {
  const response = rpc(
    {
      type: "simulateGpuiEvent",
      requestId: `source-history-up-${tag}-${Date.now()}`,
      target: { type: "main" },
      event: { type: "keyDown", key: "up", modifiers: [] },
    },
    "simulateGpuiEventResult",
  );
  if (response.success !== true) {
    throw new Error(`${tag}: simulateGpuiEvent Up failed ${JSON.stringify(response)}`);
  }
}

async function assertInputStableAfterUp(input: string, tag: string): Promise<Json> {
  send({ type: "setFilter", text: input, requestId: `source-history-set-${tag}-${Date.now()}` });
  waitForInput(input, tag);
  const before = getState(`before-${tag}`);
  pressUp(tag);
  await Bun.sleep(150);
  const after = getState(`after-${tag}`);
  if (after.inputValue !== input) {
    throw new Error(`${input}: Up should keep source-filter input stable, got ${JSON.stringify(after.inputValue)}`);
  }
  if (after.inputValue === historyEntry) {
    throw new Error(`${input}: Up recalled launcher history instead of staying in source-filter mode`);
  }
  return {
    input,
    beforeSelectedIndex: before.selectedIndex,
    afterSelectedIndex: after.selectedIndex,
    visibleChoiceCount: after.visibleChoiceCount,
    sourceFilters: after.mainWindowPreflight?.sourceFilters ?? [],
  };
}

async function main() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(kitDir, { recursive: true });
  mkdirSync(sessionRoot, { recursive: true });
  writeFileSync(
    join(kitDir, "input_history.json"),
    JSON.stringify({ entries: [historyEntry], selected_results: {} }, null, 2),
  );

  runSession(["start", session]);
  try {
    showWindow("initial");
    waitForInput("", "initial");
    pressUp("ordinary");
    waitForInput(historyEntry, "ordinary-history");

    const results: Json[] = [];
    for (const head of sourceHeads) {
      const input = `${head} source`;
      results.push(await assertInputStableAfterUp(input, head.replace(/[^a-z0-9]+/gi, "-")));
    }

    send({ type: "setFilter", text: "", requestId: `source-history-reset-${Date.now()}` });
    console.log(JSON.stringify({ ok: true, historyEntry, sourceHeads, results }, null, 2));
  } finally {
    runSession(["stop", session]);
  }
}

main().catch((error) => {
  try {
    runSession(["stop", session]);
  } catch {
    // best-effort cleanup
  }
  console.error(error instanceof Error ? error.stack : error);
  process.exit(1);
});
