#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "root-source-filter-stability");
const query = argValue("--query", "zzqxfilters");
const timeoutMs = Number(argValue("--timeout", "10000"));
const pollMs = Number(argValue("--poll", "25"));
const outputDir = join(repoRoot, ".test-output", "root-source-filter-stability");

process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
  query,
  delayMs: 250,
  results: [
    {
      path: `/tmp/${query}-late-provider-result.txt`,
      name: `${query}-late-provider-result.txt`,
      fileType: "document",
      size: 42,
      modified: 1,
    },
  ],
});

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
    throw new Error(
      `session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`,
    );
  }
  const parsed = JSON.parse(stdout);
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
      requestId: `root-source-filter-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(state)}`);
  }
  return state;
}

function waitForInput(input: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `root-source-filter-wait-${Date.now()}`,
      condition: {
        type: "stateMatch",
        state: {
          promptType: "none",
          inputValue: input,
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
  return preflight;
}

function comparable(state: Json, label: string): Json {
  const preflight = requirePreflight(state, label);
  return {
    inputValue: state.inputValue,
    computedSearchText: preflight.computedSearchText,
    sourceFilters: preflight.sourceFilters,
    selectedResultKey: preflight.selectedResultKey ?? null,
    selectedResultRole: preflight.selectedResultRole,
    visibleResultKeyFingerprint: preflight.visibleResultKeyFingerprint,
    visibleRowFingerprint: preflight.visibleRowFingerprint,
    visibleResultCount: preflight.visibleResultCount,
    visibleResults: preflight.visibleResults,
    rootFileSearch: state.rootFileSearch,
  };
}

function assertFilesOnly(frame: Json, input: string) {
  if (frame.computedSearchText !== query) {
    throw new Error(`${input}: expected computedSearchText ${query}, got ${frame.computedSearchText}`);
  }
  if (JSON.stringify(frame.sourceFilters) !== JSON.stringify(["files"])) {
    throw new Error(`${input}: expected sourceFilters [files], got ${JSON.stringify(frame.sourceFilters)}`);
  }
  if (frame.rootFileSearch?.query !== query) {
    throw new Error(`${input}: provider query was not stripped: ${JSON.stringify(frame.rootFileSearch)}`);
  }
  for (const result of frame.visibleResults ?? []) {
    if (result.role !== "rootFile") {
      throw new Error(`${input}: disallowed visible result ${JSON.stringify(result)}`);
    }
  }
}

function assertSameFrame(before: Json, after: Json, label: string) {
  const stableBefore = {
    visibleResultKeyFingerprint: before.visibleResultKeyFingerprint,
    visibleRowFingerprint: before.visibleRowFingerprint,
    visibleResultCount: before.visibleResultCount,
    visibleResults: before.visibleResults,
  };
  const stableAfter = {
    visibleResultKeyFingerprint: after.visibleResultKeyFingerprint,
    visibleRowFingerprint: after.visibleRowFingerprint,
    visibleResultCount: after.visibleResultCount,
    visibleResults: after.visibleResults,
  };
  if (JSON.stringify(stableBefore) !== JSON.stringify(stableAfter)) {
    throw new Error(
      `${label}: visible frame changed for same source-filter input\nbefore=${JSON.stringify(stableBefore)}\nafter=${JSON.stringify(stableAfter)}`,
    );
  }
}

async function runCase(input: string): Promise<Json> {
  send({
    type: "setFilter",
    text: input,
    requestId: `root-source-filter-set-${Date.now()}`,
  });
  waitForInput(input);

  const beforeState = getState(`before-${input}`);
  const baseline = comparable(beforeState, `before-${input}`);
  assertFilesOnly(baseline, input);

  const samples: Json[] = [];
  const deadline = Date.now() + timeoutMs;
  let settled: Json | null = null;
  while (Date.now() < deadline) {
    await Bun.sleep(Math.max(25, pollMs));
    const state = getState(`sample-${samples.length}`);
    const frame = comparable(state, `sample-${samples.length}`);
    assertFilesOnly(frame, input);
    assertSameFrame(baseline, frame, `${input} sample ${samples.length}`);
    samples.push(frame);
    if (frame.rootFileSearch?.loading === false) {
      settled = frame;
      break;
    }
  }
  if (!settled) {
    throw new Error(`${input}: root file provider did not settle`);
  }

  return {
    input,
    baseline,
    samples,
    settled,
  };
}

async function warmProvider() {
  send({
    type: "setFilter",
    text: query,
    requestId: `root-source-filter-warm-${Date.now()}`,
  });
  waitForInput(query);
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const state = getState("warm");
    const status = state.rootFileSearch;
    if (
      status?.query === query &&
      status?.mode === "GlobalQuery" &&
      status?.loading === false &&
      (status?.cacheResultCount ?? 0) > 0
    ) {
      return status;
    }
    await Bun.sleep(Math.max(25, pollMs));
  }
  throw new Error(`provider did not warm for ${query}`);
}

async function main() {
  mkdirSync(outputDir, { recursive: true });
  runSession(["stop", session]);
  runSession(["start", session]);
  const warmed = await warmProvider();

  const cases = [];
  for (const input of [`:f ${query}`, `${query} :f`, `${query} :files`]) {
    cases.push(await runCase(input));
  }

  const receipt = {
    schemaVersion: 1,
    status: "pass",
    session,
    query,
    warmed,
    cases,
  };
  const receiptPath = join(outputDir, "receipt.json");
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
