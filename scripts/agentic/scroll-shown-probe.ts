#!/usr/bin/env bun
/**
 * scripts/agentic/scroll-shown-probe.ts — visible-window arrow-key scroll probe.
 *
 * The user-path symptom is "holding down arrow with fast key repeat feels
 * slow" — that path renders a frame per selection change, so the window must
 * be SHOWN to measure it. Uses a cheap waitFor stateMatch barrier (tiny
 * response) instead of getState (~250KB response) so the receipt measures the
 * app, not the transport.
 *
 * Phases:
 *   - hiddenBurst: N down keys + barrier, window hidden (key handling only)
 *   - shownBurst:  same with window shown (key handling + frame renders)
 *   - shownPerKey: serial key+barrier latency with window shown
 *   - sustain:     `--sustain-seconds` of continuous keying for CPU sampling
 *
 * Usage:
 *   bun scripts/agentic/scroll-shown-probe.ts --binary <path> \
 *     --scripts 400 --burst 200 --per-key 60 --sustain-seconds 0 --label red
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
const burstN = Number(argValue("--burst", "200"));
const perKeyN = Number(argValue("--per-key", "60"));
const sustainSeconds = Number(argValue("--sustain-seconds", "0"));
const settleSeconds = Number(argValue("--settle-seconds", "0"));
const label = argValue("--label", "unlabeled");

function summarize(durations: number[]): Json {
  const sorted = [...durations].sort((a, b) => a - b);
  const pick = (q: number) =>
    sorted[Math.min(sorted.length - 1, Math.floor(q * sorted.length))] ?? 0;
  const mean =
    durations.reduce((a, b) => a + b, 0) / Math.max(1, durations.length);
  return {
    count: durations.length,
    meanMs: Number(mean.toFixed(2)),
    p50Ms: Number(pick(0.5).toFixed(2)),
    p95Ms: Number(pick(0.95).toFixed(2)),
    maxMs: Number((sorted[sorted.length - 1] ?? 0).toFixed(2)),
  };
}

const sandboxRoot = `/tmp/sk-scroll-shown-${process.pid}-${Date.now().toString(36)}`;
const home = join(sandboxRoot, "home");
const scriptsDir = join(home, ".scriptkit", "plugins", "main", "scripts");
rmSync(sandboxRoot, { recursive: true, force: true });
mkdirSync(scriptsDir, { recursive: true });
for (let i = 0; i < scriptCount; i += 1) {
  const id = String(i).padStart(4, "0");
  writeFileSync(
    join(scriptsDir, `bench-script-${id}.ts`),
    `// Name: Bench Script ${id}\n// Description: scroll probe filler script ${id}\n\nexport {};\n`,
  );
}

const driver = await Driver.launch({
  sessionName: `scroll-shown-${label}`,
  binary,
  env: { HOME: home, SK_PATH: join(home, ".scriptkit") },
  defaultTimeoutMs: 60_000,
});

const receipt: Json = {
  schemaVersion: 1,
  benchmark: "main-menu-arrow-scroll-shown",
  label,
  binary,
  scriptCount,
  appPid: driver.pid ?? null,
};

async function selectedIndex(): Promise<number> {
  const state = await driver.getState();
  return Number(state.selectedIndex ?? 0);
}

/** Cheap serial barrier: waitFor stateMatch responds with a small payload. */
async function barrier(expectedIndex: number): Promise<void> {
  await driver.waitForState({ selectedIndex: expectedIndex });
}

async function burst(key: string, n: number): Promise<Json> {
  const before = await selectedIndex();
  const expected = key === "down" ? before + n : before - n;
  const start = performance.now();
  for (let i = 0; i < n; i += 1) driver.simulateKey(key);
  await barrier(expected);
  const totalMs = performance.now() - start;
  return {
    keys: n,
    totalMs: Number(totalMs.toFixed(1)),
    perKeyMs: Number((totalMs / n).toFixed(2)),
    keysPerSecond: Number((n / (totalMs / 1000)).toFixed(1)),
    selectedIndexBefore: before,
    selectedIndexAfter: await selectedIndex(),
  };
}

try {
  // Wait for the script corpus to be scanned in.
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
      `script corpus never showed up: visibleChoiceCount=${state.visibleChoiceCount}`,
    );
  }
  receipt.visibleChoiceCount = state.visibleChoiceCount;

  // Optional settle window so one-time background startup work (app scan,
  // icon extraction) does not pollute the scroll measurements.
  if (settleSeconds > 0) {
    await Bun.sleep(settleSeconds * 1000);
    receipt.settledSeconds = settleSeconds;
  }

  // Phase 1: hidden burst (key handling only, no frames).
  receipt.hiddenBurstDown = await burst("down", burstN);

  // Phase 2: show the window, then the same burst with frames rendering.
  driver.send({ type: "show" });
  await driver.waitForState({ windowVisible: true });
  await Bun.sleep(300); // let the first shown frame settle
  receipt.shownBurstDown = await burst("down", burstN);
  receipt.shownBurstUp = await burst("up", burstN);

  // Phase 3: serial per-key latency with cheap barrier, window shown.
  const perKeyDurations: number[] = [];
  const perKeySamples: { atMs: number; ms: number }[] = [];
  let index = await selectedIndex();
  for (let i = 0; i < perKeyN; i += 1) {
    const start = performance.now();
    driver.simulateKey("down");
    index += 1;
    await barrier(index);
    const ms = performance.now() - start;
    perKeyDurations.push(ms);
    perKeySamples.push({ atMs: Number(Date.now()), ms: Number(ms.toFixed(2)) });
  }
  receipt.shownPerKey = summarize(perKeyDurations);
  receipt.shownPerKeySlowest = perKeySamples
    .sort((a, b) => b.ms - a.ms)
    .slice(0, 5)
    .map((s) => ({ ...s, at: new Date(s.atMs).toISOString() }));

  // Phase 4: sustained keying for external CPU sampling.
  if (sustainSeconds > 0) {
    const deadline = performance.now() + sustainSeconds * 1000;
    let direction: "down" | "up" = "down";
    let sustained = 0;
    while (performance.now() < deadline) {
      const n = 100;
      await burst(direction, n);
      sustained += n;
      direction = direction === "down" ? "up" : "down";
    }
    receipt.sustainKeys = sustained;
  }

  receipt.sessionDir = driver.sessionDir;
} finally {
  await driver.close();
  rmSync(sandboxRoot, { recursive: true, force: true });
}

console.log(JSON.stringify(receipt, null, 2));
