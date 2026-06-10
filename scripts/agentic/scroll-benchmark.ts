#!/usr/bin/env bun
/**
 * scripts/agentic/scroll-benchmark.ts — main-menu arrow-key scroll throughput
 * benchmark for the "holding down arrow with fast key repeat feels slow" report.
 *
 * Populates a sandbox HOME with N dummy scripts so the main list is large,
 * then measures (via the persistent protocol driver):
 *   - perKey:   simulateKey("down") + getState round trip per key (serial)
 *   - burstDown: M down keys written back-to-back, one getState barrier at the
 *                end — closest analog of OS key repeat backlog pressure
 *   - burstUp:   same burst going back up
 *
 * Usage:
 *   bun scripts/agentic/scroll-benchmark.ts \
 *     --binary target-agent/pools/scrollperf/debug/script-kit-gpui \
 *     --scripts 400 --per-key 100 --burst 200 --label red
 */

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1]
    ? process.argv[index + 1]
    : fallback;
}

const binary = resolve(
  PROJECT_ROOT,
  argValue("--binary", "target/debug/script-kit-gpui"),
);
const scriptCount = Number(argValue("--scripts", "400"));
const perKeyN = Number(argValue("--per-key", "100"));
const burstN = Number(argValue("--burst", "200"));
const label = argValue("--label", "unlabeled");

function summarize(durations: number[]): Json {
  const sorted = [...durations].sort((a, b) => a - b);
  const pick = (q: number) =>
    sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))] ?? 0;
  const mean = durations.reduce((a, b) => a + b, 0) / Math.max(1, durations.length);
  return {
    count: durations.length,
    meanMs: Number(mean.toFixed(2)),
    p50Ms: Number(pick(0.5).toFixed(2)),
    p95Ms: Number(pick(0.95).toFixed(2)),
    maxMs: Number((sorted[sorted.length - 1] ?? 0).toFixed(2)),
  };
}

// --- sandbox with a large script corpus ---------------------------------------

const sandboxRoot = `/tmp/sk-scroll-bench-${process.pid}-${Date.now().toString(36)}`;
const home = join(sandboxRoot, "home");
const scriptsDir = join(home, ".scriptkit", "plugins", "main", "scripts");
rmSync(sandboxRoot, { recursive: true, force: true });
mkdirSync(scriptsDir, { recursive: true });
for (let i = 0; i < scriptCount; i += 1) {
  const id = String(i).padStart(4, "0");
  writeFileSync(
    join(scriptsDir, `bench-script-${id}.ts`),
    `// Name: Bench Script ${id}\n// Description: scroll benchmark filler script ${id}\n\nexport {};\n`,
  );
}

// --- run ------------------------------------------------------------------------

const driver = await Driver.launch({
  sessionName: `scroll-bench-${label}`,
  binary,
  env: { HOME: home, SK_PATH: join(home, ".scriptkit") },
  defaultTimeoutMs: 60_000,
});

const receipt: Json = {
  schemaVersion: 1,
  benchmark: "main-menu-arrow-scroll",
  label,
  binary,
  scriptCount,
};

try {
  // Wait until the script scan has populated the list with our corpus.
  const scanDeadline = performance.now() + 60_000;
  let state = await driver.getState();
  while (
    Number(state.visibleChoiceCount ?? 0) < scriptCount &&
    performance.now() < scanDeadline
  ) {
    await Bun.sleep(250);
    state = await driver.getState();
  }
  if (Number(state.visibleChoiceCount ?? 0) < scriptCount) {
    throw new Error(
      `script corpus never showed up: visibleChoiceCount=${state.visibleChoiceCount} < ${scriptCount}`,
    );
  }
  receipt.visibleChoiceCount = state.visibleChoiceCount;
  receipt.startSelectedIndex = state.selectedIndex;

  // Phase A: per-key latency (simulateKey + getState barrier each key).
  // stdin is processed serially, so the getState response proves the key
  // (selection move + scheduled notify) was handled.
  const perKeyDurations: number[] = [];
  for (let i = 0; i < perKeyN; i += 1) {
    const start = performance.now();
    driver.simulateKey("down");
    await driver.getState();
    perKeyDurations.push(performance.now() - start);
  }
  const afterPerKey = await driver.getState();
  receipt.perKey = {
    ...summarize(perKeyDurations),
    selectedIndexAfter: afterPerKey.selectedIndex,
  };

  // Phase B: burst of down keys (key-repeat backlog analog).
  const burstStartState = await driver.getState();
  const burstStart = performance.now();
  for (let i = 0; i < burstN; i += 1) {
    driver.simulateKey("down");
  }
  const burstEndState = await driver.getState();
  const burstMs = performance.now() - burstStart;
  receipt.burstDown = {
    keys: burstN,
    totalMs: Number(burstMs.toFixed(1)),
    perKeyMs: Number((burstMs / burstN).toFixed(2)),
    keysPerSecond: Number((burstN / (burstMs / 1000)).toFixed(1)),
    selectedIndexBefore: burstStartState.selectedIndex,
    selectedIndexAfter: burstEndState.selectedIndex,
  };

  // Phase C: burst of up keys back toward the top.
  const upStartState = await driver.getState();
  const upStart = performance.now();
  for (let i = 0; i < burstN; i += 1) {
    driver.simulateKey("up");
  }
  const upEndState = await driver.getState();
  const upMs = performance.now() - upStart;
  receipt.burstUp = {
    keys: burstN,
    totalMs: Number(upMs.toFixed(1)),
    perKeyMs: Number((upMs / burstN).toFixed(2)),
    keysPerSecond: Number((burstN / (upMs / 1000)).toFixed(1)),
    selectedIndexBefore: upStartState.selectedIndex,
    selectedIndexAfter: upEndState.selectedIndex,
  };

  receipt.sessionDir = driver.sessionDir;
} finally {
  await driver.close();
  rmSync(sandboxRoot, { recursive: true, force: true });
}

console.log(JSON.stringify(receipt, null, 2));
