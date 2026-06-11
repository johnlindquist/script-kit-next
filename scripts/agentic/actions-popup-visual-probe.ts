#!/usr/bin/env bun
/**
 * Visual snapshot of the main-window actions popup.
 *
 * Opens the launcher, toggles Cmd+K actions, and captures a screenshot of
 * the actions-dialog window plus its measured bounds. Used to verify the
 * compact-popup metrics (13px search/title fonts, no phantom icon-gap
 * indent on iconless rows).
 *
 *   bun scripts/agentic/actions-popup-visual-probe.ts
 */

import { mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/artifacts/notes-popup-fix/script-kit-gpui",
);
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/actions-popup-visual");

mkdirSync(OUT_DIR, { recursive: true });

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "actions-popup-visual",
  sandboxHome: true,
});

const report: Record<string, any> = {};

try {
  await driver.request({ type: "show" }, { timeoutMs: 5000 }).catch(() => {});
  await driver
    .waitForState({ windowFocused: true }, { timeoutMs: 5000 })
    .catch(() => {});
  await Bun.sleep(400);

  driver.simulateKey("k", ["cmd"]);
  await Bun.sleep(800);

  const windows = await driver.listAutomationWindows();
  const popup = ((windows.windows ?? []) as Array<Record<string, any>>).find(
    (w) => w.id === "actions-dialog",
  );
  report.popup_bounds = popup?.bounds ?? null;

  const screenshotPath = join(OUT_DIR, "actions-popup.png");
  const shot = await driver.captureScreenshot({
    target: { type: "kind", kind: "actionsDialog" },
    savePath: screenshotPath,
  });
  report.screenshot = shot.error ? { error: shot.error } : screenshotPath;
  report.pass = Boolean(popup) && !shot.error;
} finally {
  await driver.close();
}

console.log(JSON.stringify(report, null, 2));
if (!report.pass) process.exit(1);
