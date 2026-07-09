#!/usr/bin/env bun

import { mkdirSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/agent-chat-auth-recovery/script-kit-gpui";
const screenshotPath = resolve(
  process.env.PROBE_SCREENSHOT ??
    ".test-screenshots/agent-chat-auth-recovery.png",
);
const receiptPath = resolve(
  process.env.PROBE_RECEIPT ??
    ".test-output/agent-chat-auth-recovery-probe.json",
);

function collectStrings(value: unknown, output: string[] = []): string[] {
  if (typeof value === "string") {
    output.push(value);
  } else if (Array.isArray(value)) {
    for (const item of value) collectStrings(item, output);
  } else if (value && typeof value === "object") {
    for (const item of Object.values(value)) collectStrings(item, output);
  }
  return output;
}

mkdirSync(resolve(".test-screenshots"), { recursive: true });
mkdirSync(resolve(".test-output"), { recursive: true });

const receipt: Json = {
  tool: "agent-chat-auth-recovery-probe",
  binary,
  pass: false,
  failures: [],
};

const driver = await Driver.launch({
  binary,
  sandboxHome: true,
  sessionName: "agent-chat-auth-recovery",
  defaultTimeoutMs: 15_000,
});

try {
  driver.send({ type: "openAiWithMockData" });
  await Bun.sleep(300);
  receipt.open = await driver.getState({ timeoutMs: 15_000 });
  receipt.fixture = await driver.request(
    {
      type: "setAgentChatTestFixture",
      phase: "error",
      userText: "Please continue.",
      assistantText:
        'Provider error: openai-codex: OpenAI API error (HTTP 429): {"error":{"type":"usage_limit_reached","message":"The usage limit has been reached","plan_type":"free"}}',
    },
    { expect: "externalCommandResult", timeoutMs: 15_000 },
  );

  await Bun.sleep(300);
  const windows = (await driver.listAutomationWindows({ timeoutMs: 15_000 })) as {
    windows?: Json[];
  };
  receipt.windows = windows;
  const mainWindow = windows.windows?.find(
    (window) => window.semanticSurface === "agentChatChat" && window.visible === true,
  );
  const target = mainWindow?.id
    ? { type: "id", id: mainWindow.id }
    : { type: "main" };
  const elements = await driver.getElements(
    { target, limit: 400 },
    { timeoutMs: 15_000 },
  );
  const strings = collectStrings(elements);
  receipt.elementStrings = strings.filter((value) =>
    /agent.chat-callout|usage limit|sign in|switch account|retry/i.test(value),
  );
  receipt.semanticCollection = {
    friendlyTitle: strings.some((value) =>
      /Account usage limit reached/i.test(value),
    ),
    signInAgain: strings.some((value) =>
      /agent.chat-callout-sign-in|Sign in again/i.test(value),
    ),
    switchAccount: strings.some((value) =>
      /agent.chat-callout-switch-account|Switch account/i.test(value),
    ),
    retry: strings.some((value) =>
      /agent.chat-callout-retry|^Retry$/i.test(value),
    ),
    rawJsonHidden: !strings.some((value) => value.includes("plan_type")),
    note:
      "Agent Chat child controls are not currently enumerated by getElements; the screenshot is the visual receipt.",
  };

  const agentState = await driver.request(
    { type: "getAgentChatState", target: { type: "id", id: "main" } },
    { expect: "agentChatStateResult", timeoutMs: 15_000 },
  );
  receipt.agentState = agentState;

  const screenshot = await driver.captureScreenshot({
    target,
    savePath: screenshotPath,
    timeoutMs: 15_000,
  });
  receipt.screenshot = {
    path: screenshotPath,
    width: screenshot.width ?? null,
    height: screenshot.height ?? null,
    error: screenshot.error ?? null,
  };

  const failures = receipt.failures as Json[];
  if (agentState.status !== "error")
    failures.push({ name: "agent_chat_error_state", agentState });
  if (agentState.messageCount !== 2)
    failures.push({ name: "agent_chat_error_message_count", agentState });
  if (screenshot.error) failures.push({ name: "screenshot", error: screenshot.error });
  receipt.pass = failures.length === 0;
} catch (error) {
  (receipt.failures as Json[]).push({
    name: "probe_exception",
    error: error instanceof Error ? error.message : String(error),
  });
} finally {
  await driver.close();
}

writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
console.log(JSON.stringify(receipt, null, 2));
process.exit(receipt.pass ? 0 : 1);
