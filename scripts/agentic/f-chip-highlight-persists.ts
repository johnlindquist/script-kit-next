#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;
type Chip = { text: string; range: [number, number]; role?: string };

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "f-chip-highlight-persists");
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "50"));
const outDir = join(repoRoot, ".test-output/f-chip-highlight-persists");
const receiptPath = join(outDir, "receipt.json");

process.env.HOME = join(outDir, "home");
process.env.SK_PATH = join(process.env.HOME, ".scriptkit");
process.env.SCRIPT_KIT_SESSION_DIR = join(outDir, "sessions");
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function run(command: string, args: string[], options: { allowFailure?: boolean } = {}): string {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.status !== 0 && !options.allowFailure) {
    throw new Error(`${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`);
  }
  return result.stdout;
}

function runSession(args: string[]): Json {
  const stdout = run("bash", [sessionScript, ...args]).trim();
  if (!stdout) throw new Error(`session.sh ${args.join(" ")} produced no stdout`);
  let parsed: Json | null = null;
  for (const line of stdout.split(/\r?\n/).reverse()) {
    const candidate = line.trim();
    if (!candidate.startsWith("{")) continue;
    try {
      parsed = JSON.parse(candidate);
      break;
    } catch {
      // session.sh can print diagnostics before the JSON envelope.
    }
  }
  if (!parsed) {
    throw new Error(`session.sh ${args.join(" ")} did not emit parseable JSON\nstdout=${stdout}`);
  }
  if (parsed.status === "error") {
    throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}`);
  }
  return parsed;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  const envelope = runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeout),
  ]);
  return envelope.response;
}

function send(command: Json): Json {
  return runSession(["send", session, JSON.stringify(command), "--await-parse", "--timeout", String(timeoutMs)]);
}

function showWindow(tag: string): Json {
  return rpc({ type: "show", requestId: `f-chip-show-${tag}-${Date.now()}` }, "windowVisibilityAck");
}

function hideWindow(tag: string): Json {
  return rpc({ type: "hide", requestId: `f-chip-hide-${tag}-${Date.now()}` }, "windowVisibilityAck");
}

function waitForInput(input: string, tag: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `f-chip-wait-input-${tag}-${Date.now()}`,
      condition: { type: "stateMatch", state: { inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function waitForWindowVisible(visible: boolean, tag: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `f-chip-wait-visible-${tag}-${Date.now()}`,
      condition: { type: "stateMatch", state: { windowVisible: visible } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function setFilter(text: string, tag: string): Json {
  const sent = send({ type: "setFilter", text, requestId: `f-chip-set-${tag}-${Date.now()}` });
  waitForInput(text, tag);
  return sent;
}

function getState(tag: string): Json {
  const response = rpc({ type: "getState", requestId: `f-chip-state-${tag}-${Date.now()}` }, "stateResult");
  if (response.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(response)}`);
  }
  return response;
}

function chips(state: Json): Chip[] {
  return state.filterInputDecorations?.chips ?? [];
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function assertSingleChip(state: Json, expectedText: string, expectedRange: [number, number]) {
  const actual = chips(state);
  assert(
    actual.length === 1,
    `expected exactly one input chip for ${JSON.stringify(state.inputValue)}, got ${JSON.stringify(actual)}`,
  );
  assert(actual[0].text === expectedText, `expected chip text ${expectedText}, got ${JSON.stringify(actual[0])}`);
  assert(
    JSON.stringify(actual[0].range) === JSON.stringify(expectedRange),
    `expected chip range ${JSON.stringify(expectedRange)}, got ${JSON.stringify(actual[0])}`,
  );
}

function assertNoChips(state: Json, label: string) {
  const actual = chips(state);
  assert(actual.length === 0, `${label}: expected no input chips, got ${JSON.stringify(actual)}`);
}

async function main() {
  mkdirSync(outDir, { recursive: true });
  const receipt: Json = {
    status: "running",
    session,
    protocol: ["show", "setFilter", "waitFor", "getState", "hide"],
    steps: [],
  };

  try {
    run("bash", [sessionScript, "stop", session], { allowFailure: true });
    receipt.start = runSession(["start", session]);
    showWindow("initial");

    setFilter("f: xy", "source-head");
    const sourceState = getState("source-head");
    assert(sourceState.inputValue === "f: xy", `expected inputValue f: xy, got ${sourceState.inputValue}`);
    assert(sourceState.filterInputDecorations?.text === "f: xy", "filterInputDecorations text did not mirror f: xy");
    assertSingleChip(sourceState, "f:", [0, 2]);
    receipt.steps.push({
      input: "f: xy",
      chips: chips(sourceState),
    });

    setFilter("~/", "home-path");
    const homeState = getState("home-path");
    assert(homeState.inputValue === "~/", `expected inputValue ~/, got ${homeState.inputValue}`);
    assert(homeState.filterInputDecorations?.text === "~/", "filterInputDecorations text did not mirror ~/");
    assertNoChips(homeState, "f: -> ~/");
    receipt.steps.push({
      input: "~/",
      chips: chips(homeState),
    });

    setFilter("/tmp", "absolute-path");
    const tmpState = getState("absolute-path");
    assert(tmpState.inputValue === "/tmp", `expected inputValue /tmp, got ${tmpState.inputValue}`);
    assert(tmpState.filterInputDecorations?.text === "/tmp", "filterInputDecorations text did not mirror /tmp");
    assertNoChips(tmpState, "f: -> /tmp");
    receipt.steps.push({
      input: "/tmp",
      chips: chips(tmpState),
    });

    hideWindow("final");
    const hiddenWait = waitForWindowVisible(false, "final");
    receipt.finalState = { windowVisible: false, waitFor: hiddenWait };
    receipt.status = "pass";
    writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  } catch (error) {
    receipt.status = "fail";
    receipt.error = error instanceof Error ? error.message : String(error);
    writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
    throw error;
  } finally {
    run("bash", [sessionScript, "stop", session], { allowFailure: true });
  }
}

await main();
