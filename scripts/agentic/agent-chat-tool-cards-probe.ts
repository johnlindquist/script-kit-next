#!/usr/bin/env bun
/**
 * Runtime proof for structured Agent Chat tool-call cards.
 *
 * Loads the provider-free kitchen-sink fixture (whose Tool rows route through
 * the real tool-call event path) and captures a screenshot receipt showing:
 * - status badges (running/complete/failed glyph colors)
 * - kind glyph + tool name + mono args subject in card headers
 * - default-expanded diff body for the edit tool
 * - default-expanded failed bash card
 *
 * Usage: bun scripts/agentic/agent-chat-tool-cards-probe.ts [binaryPath]
 */
import { Driver } from "../devtools/driver.ts";
import { mkdirSync } from "node:fs";

const binary =
  process.argv[2] ?? "target-agent/artifacts/agent-chat-cards/script-kit-gpui";

const driver = await Driver.launch({
  sessionName: "tool-cards-probe",
  sandboxHome: true,
  binary,
  env: {
    // Debug-build NSPanel invariants mismatch in headless driver sessions
    // (collection/animation behavior differs without a real activation);
    // unrelated to transcript rendering under proof here.
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "agent-chat-tool-cards-probe",
  binary,
  classification: "blocked",
};

try {
  const openResult = await driver.request(
    { type: "openAgentChatKitchenSinkFixture" },
    { timeoutMs: 15000 },
  );
  receipt.openResult = openResult;

  const state = (await driver.getState()) as Record<string, unknown>;
  receipt.currentView = state.currentView ?? null;
  receipt.windowVisible = state.windowVisible ?? null;

  mkdirSync(".test-screenshots", { recursive: true });
  const shot = (await driver.captureScreenshot({
    savePath: ".test-screenshots/agent-chat-tool-cards.png",
    timeoutMs: 15000,
  })) as Record<string, unknown>;
  receipt.screenshot = {
    saved: ".test-screenshots/agent-chat-tool-cards.png",
    width: shot.width ?? null,
    height: shot.height ?? null,
    error: shot.error ?? null,
  };

  receipt.classification =
    shot.error == null && receipt.currentView != null ? "ok" : "blocked";
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
