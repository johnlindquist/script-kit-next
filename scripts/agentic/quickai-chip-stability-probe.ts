#!/usr/bin/env bun
/**
 * Visual stability proof for the header Tab chip: entering file navigation
 * (empty-input Tab → cwd picker) must not shift the chip horizontally.
 * Captures ScriptList-empty vs FileSearchView screenshots for pixel review.
 *
 * Run: QUICKAI_SHOT_DIR=<dir> bun scripts/agentic/quickai-chip-stability-probe.ts
 */
import { Driver } from "../devtools/driver.ts";

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/quickai-tab/script-kit-gpui";

const driver = await Driver.launch({
  sessionName: "quickai-chip-stability",
  binary,
  sandboxHome: true,
});

const receipt: Record<string, unknown> = { probe: "quickai-chip-stability", binary };

try {
  await driver.waitForSettle();
  const shotDir = process.env.QUICKAI_SHOT_DIR ?? driver.sessionDir;
  driver.send({ type: "show" });
  await driver.waitFor("windowVisible", { timeoutMs: 5000 }).catch(() => null);
  await driver.waitForSettle();

  const a = await driver.captureScreenshot({ savePath: `${shotDir}/stable-1-scriptlist.png` });
  driver.simulateKey("tab");
  await driver.waitForSettle();
  const b = await driver.captureScreenshot({ savePath: `${shotDir}/stable-2-filenav.png` });

  // Back out, type text, third shot: Quick AI chip state.
  driver.simulateKey("escape");
  await driver.waitForSettle();
  driver.simulateKey("escape");
  await driver.waitForSettle();
  await driver.setFilterAndWait("hello world");
  await driver.waitForSettle();
  const c = await driver.captureScreenshot({ savePath: `${shotDir}/stable-3-quickai.png` });

  receipt.screenshots = [
    `${shotDir}/stable-1-scriptlist.png`,
    `${shotDir}/stable-2-filenav.png`,
    `${shotDir}/stable-3-quickai.png`,
  ];
  receipt.errors = [a, b, c].map((r) => (r as any)?.error ?? null);
  receipt.pass = receipt.errors.every((e) => e === null);
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
