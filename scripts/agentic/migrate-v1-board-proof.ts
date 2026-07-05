#!/usr/bin/env bun
/**
 * Runtime proof for the "Migrate v1 Scripts" built-in board.
 *
 * Drives the real app through the real user path: launcher filter →
 * open built-in → scan report rows (from SK_MIGRATE_V1_DIR fixtures) →
 * getState surface receipt → screenshot → Escape back → hide.
 *
 *   bun scripts/agentic/migrate-v1-board-proof.ts [path/to/binary]
 */

import { join } from "node:path";
import { mkdirSync } from "node:fs";
import { Driver } from "../devtools/driver.ts";

const REPO = join(import.meta.dir, "..", "..");
const FIXTURES = join(REPO, "scripts", "migrate", "__tests__", "fixtures", "v1");
const SHOT_DIR = join(REPO, ".test-screenshots");
const SHOT = join(SHOT_DIR, "migrate-v1-board-report.png");

const binary = process.argv[2] ?? join(REPO, "target-agent", "artifacts", "migrate-board", "script-kit-gpui");

function includesAll(state: unknown, needles: string[]): string[] {
  const text = JSON.stringify(state);
  return needles.filter((n) => !text.includes(n));
}

async function pollState(
  driver: Driver,
  label: string,
  predicate: (state: unknown) => boolean,
  timeoutMs = 20_000,
): Promise<unknown> {
  const start = performance.now();
  let last: unknown = {};
  while (performance.now() - start < timeoutMs) {
    last = await driver.getState();
    if (predicate(last)) return last;
    await Bun.sleep(150);
  }
  throw new Error(`timeout waiting for: ${label}\nlast state: ${JSON.stringify(last).slice(0, 800)}`);
}

const receipt: Record<string, unknown> = { proof: "migrate-v1-board", binary };
mkdirSync(SHOT_DIR, { recursive: true });

const driver = await Driver.launch({
  sessionName: "migrate-board-proof",
  binary,
  sandboxHome: true,
  env: { SK_MIGRATE_V1_DIR: FIXTURES },
});

try {
  driver.send({ type: "show" });
  await driver.waitForSettle();

  // 1. Real user path: find the built-in in the launcher.
  await driver.setFilterAndWait("Migrate v1");
  const listState = await driver.getState();
  const missingEntry = includesAll(listState, ["Migrate v1 Scripts"]);
  receipt.launcherShowsEntry = missingEntry.length === 0;
  if (missingEntry.length > 0) throw new Error("launcher does not list 'Migrate v1 Scripts'");

  // 2. Open it; scan runs automatically against the fixtures dir.
  driver.simulateKey("enter");
  await pollState(
    driver,
    "migrateV1 surface active",
    (s) => JSON.stringify(s).includes('"migrateV1"'),
    8_000,
  );
  receipt.surface = "migrateV1";

  // getState only exposes counts + the selected row; the full row set lives
  // in the semantic element tree.
  const start = performance.now();
  let elements: unknown = {};
  while (performance.now() - start < 20_000) {
    elements = await driver.getElements();
    if (includesAll(elements, ["hello-world.ts", "widget-dashboard.ts"]).length === 0) break;
    await Bun.sleep(150);
  }
  const missingRows = includesAll(elements, ["hello-world.ts", "widget-dashboard.ts"]);
  if (missingRows.length > 0) {
    throw new Error(`scan rows missing from elements: ${missingRows.join(", ")}\nelements: ${JSON.stringify(elements).slice(0, 800)}`);
  }
  receipt.scanRowsVisible = ["hello-world.ts", "renamed-apis.ts", "save-note-db.ts", "widget-dashboard.ts"]
    .filter((f) => JSON.stringify(elements).includes(f));

  // 3. Screenshot receipt of the report.
  const shot = (await driver.captureScreenshot({ savePath: SHOT })) as { width?: number; error?: string };
  receipt.screenshot = shot.error ? `ERROR: ${shot.error}` : `${SHOT} (${shot.width}px wide)`;

  // 4. Escape returns to the launcher list. The board owns Escape via its
  // GPUI key listener, so use the real-dispatch path (legacy simulateKey
  // bypasses per-surface listeners for every builtin list view).
  await driver.simulateGpuiEvent({ type: "keyDown", key: "escape" });
  await pollState(driver, "back out of migrateV1", (s) => !JSON.stringify(s).includes('"migrateV1"'), 8_000);
  receipt.escapeReturns = true;

  // 5. Hide and prove the window is gone (always-on-top etiquette).
  driver.send({ type: "hide" });
  const hidden = await pollState(
    driver,
    "windowVisible false",
    (s) => JSON.stringify(s).includes('"windowVisible":false'),
    8_000,
  );
  receipt.windowVisible = (hidden as { windowVisible?: boolean }).windowVisible ?? false;
  receipt.result = "GREEN";
} catch (e) {
  receipt.result = "RED";
  receipt.error = String(e);
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.result !== "GREEN") process.exit(1);
