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

const session = argValue("--session", "root-search-frame-stability");
const query = argValue("--query", "zzqxframeproof");
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
  let parsed: Json;
  try {
    parsed = JSON.parse(stdout);
  } catch (error) {
    throw new Error(`invalid JSON from session.sh: ${stdout}\n${String(error)}`);
  }
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
  if (envelope.status !== "ok") {
    throw new Error(`RPC failed: ${JSON.stringify(envelope)}`);
  }
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
      requestId: `root-frame-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(state)}`);
  }
  return state;
}

function waitForInput(): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `root-frame-wait-input-${Date.now()}`,
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
    throw new Error(`${label}: missing mainWindowPreflight in getState receipt`);
  }
  for (const field of [
    "selectedResultKey",
    "visibleResultKeyFingerprint",
    "enterAction",
  ]) {
    if (!(field in preflight)) {
      throw new Error(`${label}: mainWindowPreflight missing ${field}`);
    }
  }
  return preflight;
}

function comparable(preflight: Json): Json {
  return {
    selectedIndex: preflight.selectedIndex,
    selectedResultKey: preflight.selectedResultKey ?? null,
    visibleResultKeyFingerprint: preflight.visibleResultKeyFingerprint,
    enterAction: preflight.enterAction,
  };
}

async function waitForRootFileSettled(): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let last = getState("settle-start");
  while (Date.now() < deadline) {
    const status = last.rootFileSearch;
    if (
      status &&
      status.query === query &&
      status.mode === "GlobalQuery" &&
      status.loading === false
    ) {
      return last;
    }
    await Bun.sleep(Math.max(25, pollMs));
    last = getState("settle-poll");
  }
  throw new Error(
    `root file search did not settle for ${JSON.stringify(query)}; last=${JSON.stringify(
      last.rootFileSearch,
    )}`,
  );
}

async function main() {
  if (!existsSync(sessionScript)) {
    throw new Error(`missing ${sessionScript}`);
  }

  runSession(["start", session]);
  send({
    type: "setFilter",
    text: query,
    requestId: `root-frame-set-${Date.now()}`,
  });
  waitForInput();

  const before = getState("before");
  const beforePreflight = requirePreflight(before, "before");
  const beforeComparable = comparable(beforePreflight);

  if (before.rootFileSearch?.query !== query) {
    throw new Error(
      `root file search did not track query ${JSON.stringify(query)}: ${JSON.stringify(
        before.rootFileSearch,
      )}`,
    );
  }
  if (before.rootFileSearch?.mode !== "GlobalQuery") {
    throw new Error(
      `expected GlobalQuery root file mode, got ${JSON.stringify(before.rootFileSearch)}`,
    );
  }

  const after = await waitForRootFileSettled();
  const afterPreflight = requirePreflight(after, "after");
  const afterComparable = comparable(afterPreflight);

  const stable =
    JSON.stringify(beforeComparable) === JSON.stringify(afterComparable);
  const receipt = {
    schemaVersion: 1,
    status: stable ? "pass" : "fail",
    session,
    query,
    before: {
      inputValue: before.inputValue,
      selectedValue: before.selectedValue ?? null,
      rootFileSearch: before.rootFileSearch,
      mainWindowPreflight: beforeComparable,
    },
    after: {
      inputValue: after.inputValue,
      selectedValue: after.selectedValue ?? null,
      rootFileSearch: after.rootFileSearch,
      mainWindowPreflight: afterComparable,
    },
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
