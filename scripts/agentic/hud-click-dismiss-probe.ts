#!/usr/bin/env bun
/**
 * HUD click-to-dismiss proof probe.
 *
 * Drives a real user path that shows a HUD (actions popup → Copy Deep
 * Link → "Copied: scriptkit://…" HUD), clicks the HUD pill, and asserts
 * the HUD window disappears well before its natural 2000ms expiry. This
 * proves both halves of the dismissal fix: HUD windows accept mouse
 * events (click_through=false) and a click routes through the tracked
 * dismiss path that actually closes the window.
 *
 *   bun scripts/agentic/hud-click-dismiss-probe.ts
 */

import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/artifacts/notes-popup-fix/script-kit-gpui",
);

async function hudWindow(driver: Driver): Promise<Record<string, any> | null> {
  const windows = await driver.listAutomationWindows();
  const list = (windows.windows ?? []) as Array<Record<string, any>>;
  return list.find((w) => w.kind === "hud") ?? null;
}

async function waitForHudWindow(driver: Driver, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const hud = await hudWindow(driver);
    if (hud) return hud;
    await Bun.sleep(50);
  }
  throw new Error("HUD window never appeared in automation registry");
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "hud-click-dismiss",
  sandboxHome: true,
});

const report: Record<string, any> = {};

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
  report.hud_appeared = true;
  report.hud_bounds = hud.bounds;
  await Bun.sleep(300); // settle first paint

  // Click the center of the HUD pill.
  const cx = Math.round(hud.bounds.x + hud.bounds.width / 2);
  const cy = Math.round(hud.bounds.y + hud.bounds.height / 2);
  const clickedAt = Date.now();
  // cliclick parses a bare leading "-" as a relative offset; absolute
  // negative coordinates (secondary displays) need the "=" prefix.
  await Bun.$`cliclick c:=${cx},=${cy}`.quiet();
  report.clicked = { x: cx, y: cy };

  // The HUD's natural expiry is 2000ms after show; we clicked ~1100ms
  // before that. If the click dismissed it, it disappears within a few
  // frames; poll up to 900ms so a pass can only come from the click path.
  let dismissedAfterMs: number | null = null;
  const deadline = clickedAt + 900;
  while (Date.now() < deadline) {
    if (!(await hudWindow(driver))) {
      dismissedAfterMs = Date.now() - clickedAt;
      break;
    }
    await Bun.sleep(50);
  }
  report.dismissed_after_click_ms = dismissedAfterMs;
  report.pass = dismissedAfterMs !== null;
} finally {
  await driver.close();
}

console.log(JSON.stringify(report, null, 2));
if (!report.pass) process.exit(1);
