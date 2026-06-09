#!/usr/bin/env bun
/**
 * Mention-picker anchor smoke probe: opens the detached Agent Chat fixture,
 * types text plus "@" into the composer, and records the picker popup's
 * registered bounds so measured-text anchoring can be verified against the
 * composer geometry. Captures a parent-crop screenshot when possible.
 *
 *   bun scripts/agentic/mention-anchor-probe.ts
 */

import { mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/pools/agent-debug/debug/script-kit-gpui",
);
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/mention-anchor-probe");
mkdirSync(OUT_DIR, { recursive: true });

async function windowsOfKind(driver: Driver, kind: string) {
  const windows = await driver.listAutomationWindows();
  const list = (windows.windows ?? []) as Array<Record<string, any>>;
  return list.filter((w) => w.kind === kind);
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "mention-anchor",
  sandboxHome: true,
});

try {
  await driver
    .request({ type: "openAgentChatDetachedFixture" }, { timeoutMs: 8000 })
    .catch(() => {});
  await Bun.sleep(1500);

  const chatWindows = await windowsOfKind(driver, "agentChatDetached");
  if (chatWindows.length === 0) {
    throw new Error("agent chat detached window never registered");
  }

  // Type a prefix then the mention trigger, dispatched through GPUI's real
  // input pipeline targeted at the detached chat window.
  const target = { type: "kind", kind: "agentChatDetached" };
  const typeKey = (key: string, text: string) =>
    driver.request(
      {
        type: "simulateGpuiEvent",
        target,
        event: { type: "keyDown", key, modifiers: [], text },
      },
      { timeoutMs: 3000 },
    );
  for (const ch of "hello ") {
    await typeKey(ch === " " ? "space" : ch, ch);
    await Bun.sleep(40);
  }
  await typeKey("@", "@");
  await Bun.sleep(800);

  const pickers = await windowsOfKind(driver, "promptPopup");
  const shotPath = join(OUT_DIR, "mention-picker.png");
  let screenshot: unknown = null;
  if (pickers.length > 0) {
    const shot = await driver.captureScreenshot({
      target: { type: "id", id: pickers[0].id },
      savePath: shotPath,
    });
    screenshot = shot.error ? { error: shot.error } : shotPath;
  }

  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        chatWindow: chatWindows[0],
        pickerWindows: pickers,
        screenshot,
        sessionDir: driver.sessionDir,
      },
      null,
      2,
    ),
  );
} finally {
  await driver.close();
}
