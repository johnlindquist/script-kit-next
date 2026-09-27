#!/usr/bin/env bun
// Polish-fix verification probe: agent chat footer stability.
// Receipts: one JSON summary + screenshots under .test-screenshots/polish-verify/.
import { Driver } from "../devtools/driver.ts";
import { mkdirSync } from "node:fs";

const SHOT_DIR = ".test-screenshots/polish-verify";
mkdirSync(SHOT_DIR, { recursive: true });

const receipt: Record<string, unknown> = {};
const driver = await Driver.launch({ sandboxHome: true, sessionName: "polish-verify" });
try {
  await driver.waitForSettle();

  // ── Agent Chat surface: footer leading slot + actions ──
  // (was driven by the removed kitchen-sink fixture command; a plain Agent
  // Chat open exercises the same current-surface footer chrome)
  await driver
    .request({ type: "openAi" })
    .catch((e: unknown) => (receipt.chatOpenError = String(e)));
  await driver.waitForSettle();
  const chatState = await driver.request({ type: "getAgentChatState" }).catch((e: unknown) => ({ error: String(e) }));
  const chatElements = await driver
    .getElements({ target: { type: "kind", kind: "agentChat" } })
    .catch((e: unknown) => ({ error: String(e) }));
  const chatText = JSON.stringify(chatElements) + JSON.stringify(chatState);
  receipt.agentChat = {
    footerHasActions: chatText.includes("⌘K") || chatText.toLowerCase().includes("actions"),
    footerHasLeadingRun: chatText.includes("Send") || chatText.includes("Stop"),
  };
  await driver
    .captureScreenshot({ savePath: `${SHOT_DIR}/agent-chat.png` })
    .catch((e: unknown) => (receipt.chatShotError = String(e)));

  receipt.ok = true;
} catch (e) {
  receipt.ok = false;
  receipt.fatal = String(e);
} finally {
  await driver.close();
}
console.log(JSON.stringify(receipt, null, 2));
