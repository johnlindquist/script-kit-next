#!/usr/bin/env bun
/**
 * Runtime proof for the complete procedural background-effect catalog.
 *
 * Drives the real launcher entries (these commands are not in the direct
 * triggerBuiltin registry), verifies every persisted slug in cycle order,
 * then sends real GPUI mouse moves and confirms the active effect receives
 * the shared focus signal. Screenshot hashes are intentionally not a gate:
 * another same-title Script Kit process can win OS-level window capture.
 */
import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { Driver } from "../devtools/driver.ts";

const EFFECT_SLUGS = [
  "aurora",
  "plasma",
  "starfield",
  "lava-lamp",
  "nebula",
  "rain",
  "waves",
  "fireflies",
  "hue-drift",
  "grain",
  "scanlines",
  "dot-grid",
  "caustics",
  "matrix",
  "breath",
  "confetti",
  "silk",
  "dunes",
  "moonwater",
  "petals",
  "zen-garden",
  "ink-wash",
  "marble",
  "tree-rings",
  "sea-glass",
  "koi-pond",
  "bamboo",
  "candlelight",
  "jellyfish",
  "lotus",
  "soft-prism",
] as const;

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/bg-effects/script-kit-gpui";
const driver = await Driver.launch({
  binary,
  sessionName: `bg-effects-proof-${process.pid}`,
  sandboxHome: true,
});

const receipt: Record<string, unknown> = {
  binary,
  expectedCount: EFFECT_SLUGS.length,
  cycle: [] as unknown[],
};
const cycle = receipt.cycle as unknown[];

try {
  driver.send({ type: "show" });
  await driver.waitForSettle();

  const configPath = join(driver.sessionDir, "home", ".scriptkit", "config.ts");
  const readPersistedSlug = (): string | null => {
    if (!existsSync(configPath)) return null;
    const text = readFileSync(configPath, "utf8");
    // The generated template contains commented examples before the live
    // `effects` assignment, so the effective value is the final match.
    const matches = [...text.matchAll(/background["']?\s*:\s*["']([^"']+)["']/g)];
    return matches.at(-1)?.[1] ?? null;
  };

  const waitForSlug = async (expected: string): Promise<string | null> => {
    const deadline = Date.now() + 4_000;
    let actual = readPersistedSlug();
    while (actual !== expected && Date.now() < deadline) {
      await Bun.sleep(50);
      actual = readPersistedSlug();
    }
    return actual;
  };

  const executeLauncherEntry = async (name: string) => {
    await driver.setFilterAndWait(name);
    driver.simulateKey("enter");
  };

  await executeLauncherEntry("Background Effect: Off");
  const offSlug = await waitForSlug("off");
  receipt.off = { expected: "off", actual: offSlug, ok: offSlug === "off" };

  for (const [index, expected] of EFFECT_SLUGS.entries()) {
    // Builtin execution can clear or replace launcher state, so resolve the
    // real entry again before every Enter instead of assuming selection sticks.
    await executeLauncherEntry("Background Effect: Next");
    const actual = await waitForSlug(expected);
    cycle.push({ id: index + 1, expected, actual, ok: actual === expected });
    if (actual !== expected) break;
  }

  const focusDispatches = [];
  for (const [x, y] of [
    [90, 90],
    [520, 180],
    [260, 390],
  ]) {
    focusDispatches.push(
      await driver.simulateGpuiEvent(
        { type: "mouseMove", x, y },
        { target: { type: "kind", kind: "main" }, timeoutMs: 5_000 },
      ),
    );
    await Bun.sleep(550);
  }

  const focusLogs = await driver.getLogs({ contains: "source=mouse", limit: 20 });
  const persistLogs = await driver.getLogs({ contains: "persist background effect", limit: 20 });
  const finalState = await driver.getState();
  const focusEntries = (focusLogs.entries ?? []) as unknown[];
  const persistEntries = (persistLogs.entries ?? []) as unknown[];
  receipt.focus = {
    dispatches: focusDispatches,
    logCount: focusEntries.length,
    entries: focusEntries,
  };
  receipt.final = {
    persistedSlug: readPersistedSlug(),
    appAlive: driver.alive,
    stateType: finalState.type,
    persistFailures: persistEntries.length,
  };

  const completeCycle =
    cycle.length === EFFECT_SLUGS.length &&
    cycle.every((entry: any) => entry.ok === true);
  receipt.ok =
    offSlug === "off" &&
    completeCycle &&
    focusEntries.length > 0 &&
    driver.alive &&
    persistEntries.length === 0;
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
