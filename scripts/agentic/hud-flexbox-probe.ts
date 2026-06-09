#!/usr/bin/env bun
/**
 * HUD sizing proof probe.
 *
 * Drives a real user path that shows a HUD (actions popup → Copy Deep
 * Link → "Copied: scriptkit://…" HUD), reads the HUD's registered
 * automation bounds (the measured window size), and captures a targeted
 * screenshot through the new `hud` automation window kind.
 *
 *   bun scripts/agentic/hud-flexbox-probe.ts
 */

import { mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/pools/agent-debug/debug/script-kit-gpui",
);
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/hud-flexbox-probe");

mkdirSync(OUT_DIR, { recursive: true });

async function waitForHudWindow(driver: Driver, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const windows = await driver.listAutomationWindows();
    const list = (windows.windows ?? []) as Array<Record<string, any>>;
    const hud = list.find((w) => w.kind === "hud");
    if (hud) return hud;
    await Bun.sleep(50);
  }
  throw new Error("HUD window never appeared in automation registry");
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "hud-probe",
  sandboxHome: true,
});

try {
  await driver.request({ type: "show" }, { timeoutMs: 5000 }).catch(() => {});
  await driver
    .waitForState({ windowFocused: true }, { timeoutMs: 5000 })
    .catch(() => {});
  await Bun.sleep(300);

  // Open actions popup and run "Copy Deep Link" (clipboard copy → HUD).
  driver.simulateKey("k", ["cmd"]);
  await Bun.sleep(500);
  for (const ch of "deep") driver.simulateKey(ch, []);
  await Bun.sleep(300);
  driver.simulateKey("enter", []);

  const hud = await waitForHudWindow(driver);
  await Bun.sleep(400); // settle first paint before capture

  const screenshotPath = join(OUT_DIR, "hud-window.png");
  const shot = await driver.captureScreenshot({
    target: { type: "kind", kind: "hud" },
    savePath: screenshotPath,
  });

  // Sanity: main window capture in the same session (isolates HUD-specific
  // capture failures from environment-wide permission problems).
  const mainShotPath = join(OUT_DIR, "main-window.png");
  const mainShot = await driver.captureScreenshot({
    target: { type: "kind", kind: "main" },
    savePath: mainShotPath,
  });

  console.log(
    JSON.stringify(
      {
        schemaVersion: 2,
        hudAutomationWindow: hud,
        screenshot: shot.error ? { error: shot.error } : screenshotPath,
        mainScreenshot: mainShot.error ? { error: mainShot.error } : mainShotPath,
        sessionDir: driver.sessionDir,
      },
      null,
      2,
    ),
  );
} finally {
  await driver.close();
}
