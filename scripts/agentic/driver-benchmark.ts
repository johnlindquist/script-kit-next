#!/usr/bin/env bun
/**
 * scripts/agentic/driver-benchmark.ts — throughput proof for the persistent
 * driver (scripts/devtools/driver.ts) vs the legacy per-command path
 * (session.sh rpc/send subprocess + FIFO forwarder + 50ms response polling).
 *
 * Each scenario is the same logical filter exercise on the main menu:
 *   1. setFilter("<query-i>")
 *   2. waitFor stateMatch { inputValue }
 *   3. getState
 *   4. setFilter("")
 *   5. waitFor stateMatch { inputValue: "" }
 *
 * Usage:
 *   bun scripts/agentic/driver-benchmark.ts                 # driver x100, legacy x5
 *   bun scripts/agentic/driver-benchmark.ts --driver-n 200 --legacy-n 0
 */

import { spawnSync } from "node:child_process";
import { mkdirSync, rmSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const SESSION_SCRIPT = join(PROJECT_ROOT, "scripts/agentic/session.sh");

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1]
    ? process.argv[index + 1]
    : fallback;
}

const driverN = Number(argValue("--driver-n", "100"));
const legacyN = Number(argValue("--legacy-n", "5"));

interface PhaseResult {
  scenarios: number;
  totalMs: number;
  perScenarioMs: { mean: number; p50: number; p95: number; min: number; max: number };
  scenariosPerSecond: number;
  setupMs: number;
}

function summarize(durations: number[], totalMs: number, setupMs: number): PhaseResult {
  const sorted = [...durations].sort((a, b) => a - b);
  const pick = (q: number) =>
    sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))] ?? 0;
  const mean = durations.reduce((a, b) => a + b, 0) / Math.max(1, durations.length);
  return {
    scenarios: durations.length,
    totalMs: Math.round(totalMs),
    perScenarioMs: {
      mean: Number(mean.toFixed(1)),
      p50: Number(pick(0.5).toFixed(1)),
      p95: Number(pick(0.95).toFixed(1)),
      min: Number((sorted[0] ?? 0).toFixed(1)),
      max: Number((sorted[sorted.length - 1] ?? 0).toFixed(1)),
    },
    scenariosPerSecond: Number((durations.length / (totalMs / 1000)).toFixed(1)),
    setupMs: Math.round(setupMs),
  };
}

// --- driver phases -------------------------------------------------------------

async function runDriverPhases(
  n: number,
): Promise<{ perCommand: PhaseResult; batch: PhaseResult }> {
  const setupStart = performance.now();
  const driver = await Driver.launch({
    sessionName: `driver-benchmark-${process.pid}`,
    sandboxHome: true,
  });
  // First command after ready absorbs remaining init cost; do it outside
  // the measured loops so scenarios reflect steady-state throughput.
  await driver.getState();
  const setupMs = performance.now() - setupStart;

  try {
    // Phase A: one round trip per step (3 request/response cycles + 2 sends).
    const perCommandDurations: number[] = [];
    const perCommandStart = performance.now();
    for (let index = 0; index < n; index += 1) {
      const start = performance.now();
      const query = `bench-${index}`;
      await driver.setFilterAndWait(query);
      const state = await driver.getState();
      if (state.inputValue !== query) {
        throw new Error(
          `scenario ${index}: expected inputValue '${query}', got '${state.inputValue}'`,
        );
      }
      await driver.setFilterAndWait("");
      perCommandDurations.push(performance.now() - start);
    }
    const perCommand = summarize(
      perCommandDurations,
      performance.now() - perCommandStart,
      setupMs,
    );

    // Phase B: the whole scenario as ONE server-side batch round trip.
    // The waitFor stateMatch steps double as the input-value verification
    // the per-command phase did via getState.
    const batchDurations: number[] = [];
    const batchStart = performance.now();
    for (let index = 0; index < n; index += 1) {
      const start = performance.now();
      const query = `batch-${index}`;
      const result = await driver.batch([
        { type: "setInput", text: query },
        {
          type: "waitFor",
          condition: { type: "stateMatch", state: { inputValue: query } },
          timeout: 5000,
          pollInterval: 5,
        },
        { type: "setInput", text: "" },
        {
          type: "waitFor",
          condition: { type: "stateMatch", state: { inputValue: "" } },
          timeout: 5000,
          pollInterval: 5,
        },
      ]);
      if (result.type !== "batchResult" || result.success !== true) {
        throw new Error(
          `batch scenario ${index} failed: ${JSON.stringify(result)}`,
        );
      }
      batchDurations.push(performance.now() - start);
    }
    const batch = summarize(
      batchDurations,
      performance.now() - batchStart,
      0,
    );
    return { perCommand, batch };
  } finally {
    await driver.close();
  }
}

// --- legacy phase -------------------------------------------------------------

// pid-namespaced so parallel benchmark runs never share sessions or sandboxes
const LEGACY_SESSION = `driver-benchmark-legacy-${process.pid}`;
const legacyRoot = join(
  PROJECT_ROOT,
  ".test-output",
  "driver-benchmark-legacy",
  String(process.pid),
);
const legacyEnv: Record<string, string> = {
  ...(process.env as Record<string, string>),
  HOME: join(legacyRoot, "home"),
  SK_PATH: join(legacyRoot, "home", ".scriptkit"),
  SCRIPT_KIT_SESSION_DIR: join(legacyRoot, "sessions"),
  SCRIPT_KIT_SESSION_READY_TIMEOUT_MS: "10000",
};

function legacySession(args: string[]): Json {
  const result = spawnSync(SESSION_SCRIPT, args, {
    cwd: PROJECT_ROOT,
    encoding: "utf8",
    env: legacyEnv,
  });
  const stdout = (result.stdout ?? "").trim();
  if (result.status !== 0 || !stdout) {
    throw new Error(
      `session.sh ${args[0]} failed (exit ${result.status})\nstdout=${stdout}\nstderr=${result.stderr}`,
    );
  }
  const parsed = JSON.parse(stdout);
  if (parsed.status === "error") {
    throw new Error(`session.sh ${args[0]} error: ${stdout}`);
  }
  return parsed;
}

function legacyRpc(command: Json, expect: string): Json {
  return legacySession([
    "rpc",
    LEGACY_SESSION,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    "10000",
  ]);
}

function legacySend(command: Json): void {
  legacySession(["send", LEGACY_SESSION, JSON.stringify(command)]);
}

function legacyWaitForInput(input: string, tag: string): void {
  legacyRpc(
    {
      type: "waitFor",
      requestId: `bench-legacy-wait-${tag}`,
      condition: { type: "stateMatch", state: { inputValue: input } },
      timeout: 10000,
      pollInterval: 25,
    },
    "waitForResult",
  );
}

function runLegacyPhase(n: number): PhaseResult {
  rmSync(legacyRoot, { recursive: true, force: true });
  mkdirSync(join(legacyRoot, "home", ".scriptkit"), { recursive: true });

  const setupStart = performance.now();
  legacySession(["start", LEGACY_SESSION]);
  const setupMs = performance.now() - setupStart;

  const durations: number[] = [];
  const loopStart = performance.now();
  try {
    for (let index = 0; index < n; index += 1) {
      const start = performance.now();
      const query = `bench-${index}`;
      legacySend({ type: "setFilter", text: query });
      legacyWaitForInput(query, `set-${index}`);
      const state = legacyRpc(
        { type: "getState", requestId: `bench-legacy-state-${index}` },
        "stateResult",
      );
      const inputValue = (state.response as Json | undefined)?.inputValue;
      if (inputValue !== query) {
        throw new Error(
          `legacy scenario ${index}: expected inputValue '${query}', got '${inputValue}'`,
        );
      }
      legacySend({ type: "setFilter", text: "" });
      legacyWaitForInput("", `reset-${index}`);
      durations.push(performance.now() - start);
    }
  } finally {
    try {
      legacySession(["stop", LEGACY_SESSION]);
    } catch {
      // best-effort cleanup
    }
  }
  return summarize(durations, performance.now() - loopStart, setupMs);
}

// --- main -----------------------------------------------------------------------

const receipt: Json = {
  schemaVersion: 1,
  benchmark: "driver-vs-legacy-filter-scenario",
  stepsPerScenario: 5,
};

if (driverN > 0) {
  console.error(
    `[driver-benchmark] running driver phases (${driverN} scenarios each, per-command + batch)...`,
  );
  const { perCommand, batch } = await runDriverPhases(driverN);
  receipt.driver = perCommand;
  receipt.driverBatch = batch;
}
if (legacyN > 0) {
  console.error(`[driver-benchmark] running legacy phase (${legacyN} scenarios)...`);
  receipt.legacy = runLegacyPhase(legacyN);
}
if (receipt.driver && receipt.legacy) {
  receipt.speedupPerCommand = Number(
    (receipt.legacy.perScenarioMs.mean / receipt.driver.perScenarioMs.mean).toFixed(1),
  );
}
if (receipt.driverBatch && receipt.legacy) {
  receipt.speedupBatch = Number(
    (receipt.legacy.perScenarioMs.mean / receipt.driverBatch.perScenarioMs.mean).toFixed(1),
  );
}

console.log(JSON.stringify(receipt, null, 2));
