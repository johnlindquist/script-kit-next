#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

const session = argValue("--session", "root-passive-frame-stability");
const query = argValue("--query", "zzqxpassiveproof");
const timeoutMs = Number(argValue("--timeout", "8000"));
const pollMs = Number(argValue("--poll", "100"));

function runSession(args: string[]): Json {
  const result = spawnSync(sessionScript, args, {
    cwd: repoRoot,
    encoding: "utf8",
  });
  const stdout = result.stdout.trim();
  if (!stdout) {
    throw new Error(
      `session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`,
    );
  }
  const parsed = JSON.parse(stdout) as Json;
  if (result.status !== 0 || parsed.status === "error") {
    throw new Error(
      `session.sh ${args.join(" ")} failed: ${JSON.stringify(parsed)} stderr=${result.stderr.trim()}`,
    );
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
  const state = rpc(
    {
      type: "getState",
      requestId: `root-passive-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(state)}`);
  }
  return state;
}

function waitForInput(): void {
  rpc(
    {
      type: "waitFor",
      requestId: `root-passive-wait-input-${Date.now()}`,
      condition: {
        type: "stateMatch",
        state: {
          promptType: "none",
          inputValue: query,
        },
      },
      timeout: timeoutMs,
      pollInterval: Math.max(25, Math.min(pollMs, 250)),
    },
    "waitForResult",
  );
}

function requirePreflight(state: Json, label: string): Json {
  const preflight = state.mainWindowPreflight;
  if (!preflight) {
    throw new Error(`${label}: missing mainWindowPreflight`);
  }
  if (!preflight.rootPassiveFrame) {
    throw new Error(`${label}: missing mainWindowPreflight.rootPassiveFrame`);
  }
  return preflight;
}

function comparable(preflight: Json): Json {
  return {
    selectedIndex: preflight.selectedIndex,
    selectedResultKey: preflight.selectedResultKey ?? null,
    visibleResultKeyFingerprint: preflight.visibleResultKeyFingerprint,
    enterAction: preflight.enterAction,
    rootPassiveFrame: preflight.rootPassiveFrame,
  };
}

function refreshing(preflight: Json): boolean {
  const frame = preflight.rootPassiveFrame;
  return Boolean(frame?.browserTabs?.refreshing || frame?.browserHistory?.refreshing);
}

function assertPassiveRolesDoNotPrecedePrimary(preflight: Json, label: string): void {
  const rows = preflight.visibleResults ?? [];
  const firstPrimary = rows.find((row: Json) => row.role === "primary");
  const firstPassive = rows.find((row: Json) => row.role === "rootPassive");
  if (!firstPrimary || !firstPassive) {
    return;
  }
  if (firstPassive.visibleRank <= firstPrimary.visibleRank) {
    throw new Error(
      `${label}: rootPassive row appeared before primary row: ${JSON.stringify(rows)}`,
    );
  }
}

async function waitForPassiveSettled(): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let last = getState("settle-start");
  while (Date.now() < deadline) {
    const preflight = requirePreflight(last, "settle");
    if (!refreshing(preflight)) {
      return last;
    }
    await Bun.sleep(Math.max(25, pollMs));
    last = getState("settle-poll");
  }
  throw new Error("passive browser snapshots did not settle before timeout");
}

async function main() {
  if (!existsSync(sessionScript)) {
    throw new Error(`missing ${sessionScript}`);
  }

  runSession(["start", session]);
  send({
    type: "setFilter",
    text: query,
    requestId: `root-passive-set-${Date.now()}`,
  });
  waitForInput();

  const before = getState("before");
  const beforePreflight = requirePreflight(before, "before");
  assertPassiveRolesDoNotPrecedePrimary(beforePreflight, "before");
  const beforeComparable = comparable(beforePreflight);
  const after = await waitForPassiveSettled();
  const afterPreflight = requirePreflight(after, "after");
  assertPassiveRolesDoNotPrecedePrimary(afterPreflight, "after");
  const afterComparable = comparable(afterPreflight);

  const stable =
    JSON.stringify(beforeComparable) === JSON.stringify(afterComparable);
  const receipt = {
    schemaVersion: 1,
    status: stable ? "pass" : "fail",
    session,
    query,
    before: beforeComparable,
    after: afterComparable,
  };

  process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
  if (!stable) {
    process.exit(1);
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
