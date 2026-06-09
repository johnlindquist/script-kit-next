#!/usr/bin/env bun
/**
 * Red/green proof probe for the Actions popup flexbox refactor.
 *
 * Opens the real actions popup via the user path (Cmd+K from the focused
 * main window), captures targeted screenshots of the popup window in both
 * populated and empty-filter states, and records layout receipts
 * (window bounds vs dialog container bounds) so the refactor can prove the
 * interior keeps matching the window's computed size.
 *
 *   bun scripts/agentic/actions-flexbox-compare.ts red
 *   bun scripts/agentic/actions-flexbox-compare.ts green
 */

import { mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/pools/agent-debug/debug/script-kit-gpui",
);
const phase = process.argv[2] ?? "red";
const OUT_DIR = join(
  PROJECT_ROOT,
  `.test-screenshots/actions-flexbox-compare/${phase}`,
);

const ACTIONS_TARGET = { type: "kind", kind: "actionsDialog" };

async function waitForActionsWindow(driver: Driver, timeoutMs = 5000) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const windows = await driver.listAutomationWindows();
    const list = (windows.windows ?? []) as Array<Record<string, any>>;
    const popup = list.find((w) => w.kind === "actionsDialog");
    if (popup) return popup;
    await Bun.sleep(50);
  }
  throw new Error("actions popup window never appeared in automation registry");
}

function boundsReceipt(layout: Record<string, any>) {
  return {
    windowBounds: layout.windowBounds ?? null,
    components: (layout.components ?? layout.layoutComponents ?? null),
  };
}

mkdirSync(OUT_DIR, { recursive: true });

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: `actions-flexbox-${phase}`,
  sandboxHome: true,
});

try {
  await driver.request({ type: "show" }, { timeoutMs: 5000 }).catch(() => {});
  await driver
    .waitForState({ windowFocused: true }, { timeoutMs: 5000 })
    .catch(() => {});
  await Bun.sleep(300);

  // User path: Cmd+K opens the actions popup.
  driver.simulateKey("k", ["cmd"]);
  const popupWindow = await waitForActionsWindow(driver);
  await Bun.sleep(400); // settle resize + first paint

  // Populated state: screenshot + layout receipt.
  const populatedShot = join(OUT_DIR, "actions-populated.png");
  const shotResult = await driver.captureScreenshot({
    target: ACTIONS_TARGET,
    savePath: populatedShot,
  });
  if (shotResult.error) throw new Error(`screenshot failed: ${shotResult.error}`);
  const populatedLayout = await driver.getLayoutInfo({ target: ACTIONS_TARGET });
  const populatedState = await driver.request(
    { type: "getState", target: ACTIONS_TARGET },
    { timeoutMs: 5000 },
  );

  // Empty state: type a filter that matches nothing.
  for (const ch of "zzzqqq") driver.simulateKey(ch, []);
  await Bun.sleep(400);
  const emptyShot = join(OUT_DIR, "actions-empty.png");
  const emptyShotResult = await driver.captureScreenshot({
    target: ACTIONS_TARGET,
    savePath: emptyShot,
  });
  if (emptyShotResult.error) {
    throw new Error(`empty screenshot failed: ${emptyShotResult.error}`);
  }
  const emptyLayout = await driver.getLayoutInfo({ target: ACTIONS_TARGET });

  const receipt = {
    schemaVersion: 1,
    phase,
    primitiveStack: [
      "show",
      "waitForState(windowFocused)",
      "simulateKey(cmd+k)",
      "listAutomationWindows",
      "captureScreenshot(target=actionsDialog)",
      "getLayoutInfo(target=actionsDialog)",
      "getState(target=actionsDialog)",
    ],
    popupWindow: {
      automationId: popupWindow.automationId ?? null,
      kind: popupWindow.kind ?? null,
      title: popupWindow.title ?? null,
      bounds: popupWindow.bounds ?? null,
    },
    populated: {
      screenshot: populatedShot,
      layout: boundsReceipt(populatedLayout),
      visibleCount:
        populatedState.visibleCount ?? populatedState.filteredCount ?? null,
    },
    empty: {
      screenshot: emptyShot,
      layout: boundsReceipt(emptyLayout),
    },
    sessionDir: driver.sessionDir,
  };
  await Bun.write(join(OUT_DIR, "receipt.json"), JSON.stringify(receipt, null, 2));
  console.log(JSON.stringify(receipt, null, 2));
} finally {
  await driver.close();
}
