#!/usr/bin/env bun
/**
 * Runtime proof for the background shader-effects feature.
 *
 * Drives the real app: enables an effect via the "Background Effect: Next"
 * builtin, proves the effect layer renders (screenshot deltas + animation
 * frames), cycles again, verifies persistence into the sandbox config.ts,
 * then turns it off. Prints one JSON receipt.
 */
import { join } from "node:path";
import { readFileSync, existsSync } from "node:fs";
import { Driver } from "../devtools/driver.ts";

const OUT_DIR = process.env.BG_FX_OUT ?? "/tmp/bg-effects-proof";
await Bun.$`mkdir -p ${OUT_DIR}`.quiet();

const receipt: Record<string, unknown> = { steps: [] as unknown[] };
const steps = receipt.steps as unknown[];

function sha(buf: Buffer): string {
  return new Bun.CryptoHasher("sha256").update(buf).digest("hex").slice(0, 16);
}

const driver = await Driver.launch({
  binary: "target-agent/artifacts/bg-effects/script-kit-gpui",
  sessionName: "bg-effects-proof",
  sandboxHome: true,
});

try {
  driver.send({ type: "show" });
  await driver.waitForSettle();

  const shot = async (name: string) => {
    const path = join(OUT_DIR, `${name}.png`);
    const res = await driver.captureScreenshot({ savePath: path });
    if (res.error) throw new Error(`screenshot ${name}: ${res.error}`);
    return { path, hash: sha(readFileSync(path)), width: res.width, height: res.height };
  };

  const baseline = await shot("0-baseline");
  steps.push({ step: "baseline", ...baseline });

  // Enable: filter to the builtin and hit Enter.
  await driver.setFilterAndWait("Background Effect: Next");
  const state1 = await driver.getState();
  driver.simulateKey("enter");
  await Bun.sleep(700);
  const effect1a = await shot("1-effect-aurora");
  await Bun.sleep(150);
  const effect1b = await shot("2-effect-aurora-later-frame");
  steps.push({
    step: "enable-first-effect",
    filterMatchedInput: state1.inputValue,
    effectVsBaselineDiffers: effect1a.hash !== baseline.hash,
    animationFramesDiffer: effect1a.hash !== effect1b.hash,
    shots: [effect1a, effect1b],
  });

  // Cycle: Enter again on the same entry.
  driver.simulateKey("enter");
  await Bun.sleep(700);
  const effect2 = await shot("3-effect-plasma");
  steps.push({
    step: "cycle-second-effect",
    differsFromFirstEffect: effect2.hash !== effect1b.hash,
    shot: effect2,
  });

  // Persistence: the sandbox config.ts should now carry the effects group.
  const configPath = join(driver.sessionDir, "home", ".scriptkit", "config.ts");
  const configText = existsSync(configPath) ? readFileSync(configPath, "utf8") : "";
  const effectsLine = configText
    .split("\n")
    .find((line) => line.includes("effects"));
  steps.push({
    step: "persistence",
    configPath,
    configHasEffects: /effects/.test(configText),
    configHasSlug: /plasma/.test(configText),
    effectsLine: effectsLine?.trim() ?? null,
  });

  // Off.
  await driver.setFilterAndWait("Background Effect: Off");
  driver.simulateKey("enter");
  await Bun.sleep(700);
  const off = await shot("4-effect-off");
  steps.push({ step: "off", shot: off });

  // No persist errors in the app log ring.
  const logs = await driver.getLogs({ contains: "background effect", limit: 20 });
  steps.push({
    step: "logs",
    persistFailures: (logs.entries ?? []).filter((e: any) =>
      /Failed to persist/.test(e.message ?? ""),
    ).length,
  });

  receipt.ok =
    effect1a.hash !== baseline.hash &&
    effect1a.hash !== effect1b.hash &&
    effect2.hash !== effect1b.hash &&
    /plasma/.test(configText);
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
