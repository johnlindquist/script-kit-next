#!/usr/bin/env bun
/**
 * Confirm popup sizing proof probe.
 *
 * Opens the dev style tool's confirm-modal kitchen-sink fixture (a real
 * confirm popup window with a wrapping multi-line body), reads the
 * popup's registered automation bounds (the measured window size), and
 * captures a targeted screenshot.
 *
 *   bun scripts/agentic/confirm-flexbox-probe.ts
 */

import { mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/pools/agent-debug/debug/script-kit-gpui",
);
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/confirm-flexbox-probe");

mkdirSync(OUT_DIR, { recursive: true });

async function waitForWindow(
  driver: Driver,
  predicate: (w: Record<string, any>) => boolean,
  label: string,
  timeoutMs = 8000,
) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const windows = await driver.listAutomationWindows();
    const list = (windows.windows ?? []) as Array<Record<string, any>>;
    const match = list.find(predicate);
    if (match) return match;
    await Bun.sleep(80);
  }
  throw new Error(`${label} never appeared in automation registry`);
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "confirm-probe",
  sandboxHome: true,
  env: { SCRIPT_KIT_STYLE_DEVTOOLS: "1" },
});

try {
  await driver.request({ type: "show" }, { timeoutMs: 5000 }).catch(() => {});
  await Bun.sleep(500);

  await waitForWindow(
    driver,
    (w) => w.kind === "devStyleTool",
    "dev style tool window",
  );

  const batch = await driver.request(
    {
      type: "batch",
      commands: [
        {
          type: "selectBySemanticId",
          semanticId: "button:dev-style-tool-open-confirm-modal-kitchen-sink",
          submit: true,
        },
      ],
      options: { stopOnError: true, timeout: 8000 },
      target: { type: "kind", kind: "devStyleTool" },
    },
    { expect: "batchResult", timeoutMs: 10000 },
  );

  // The kitchen-sink fixture opens the in-main-window confirm prompt
  // (AppView::ConfirmPrompt). Wait for the main window to rekey its
  // semantic surface, then capture it: this surface renders the shared
  // confirm shell with the flexbox-native action pills.
  const confirm = await waitForWindow(
    driver,
    (w) => w.id === "main" && w.semanticSurface === "confirmPrompt",
    "main-window confirm prompt surface",
  );
  await Bun.sleep(400); // settle first paint before capture

  const screenshotPath = join(OUT_DIR, "confirm-prompt-main.png");
  const shot = await driver.captureScreenshot({
    target: { type: "kind", kind: "main" },
    savePath: screenshotPath,
  });

  console.log(
    JSON.stringify(
      {
        schemaVersion: 2,
        batchSuccess: (batch as Record<string, any>).success,
        confirmSurfaceWindow: confirm,
        screenshot: shot.error ? { error: shot.error } : screenshotPath,
        sessionDir: driver.sessionDir,
      },
      null,
      2,
    ),
  );
} finally {
  await driver.close();
}
