#!/usr/bin/env bun
/**
 * Runtime proof for Agent Chat multi-thread sessions (Feature C).
 *
 * Drives the real user path against a live Pi-backed Agent Chat:
 * 1. openAi → live thread A (retained_thread_count = 0)
 * 2. type a draft into A's composer
 * 3. Cmd+N → fresh thread B on its own Pi connection; A is retained
 *    (retained_thread_count = 1, composer empty)
 * 4. Cmd+K → screenshot the actions dialog "Threads" section
 *    (New Thread ⌘N + "Switch to: …" row)
 * 5. dialog search "switch" + Enter → back on thread A
 *    (draft restored, B retained, retained_thread_count still 1)
 *
 * Usage: bun scripts/agentic/agent-chat-threads-probe.ts [binaryPath]
 */
import { Driver } from "../devtools/driver.ts";
import { mkdirSync } from "node:fs";

const binary =
  process.argv[2] ?? "target-agent/artifacts/agent-chat-threads/script-kit-gpui";

const driver = await Driver.launch({
  sessionName: "agent-chat-threads-probe",
  sandboxHome: true,
  binary,
  env: {
    // Debug-build NSPanel invariants mismatch in headless driver sessions;
    // unrelated to thread-pool behavior under proof here.
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "agent-chat-threads-probe",
  binary,
  classification: "blocked",
  checks: [] as Array<Record<string, unknown>>,
};
const checks = receipt.checks as Array<Record<string, unknown>>;

function check(name: string, pass: boolean, detail: unknown) {
  checks.push({ name, pass, detail });
}

async function agentChatState(): Promise<Record<string, unknown>> {
  const result = (await driver.request(
    { type: "getAgentChatState" },
    { timeoutMs: 10000 },
  )) as Record<string, unknown>;
  return (result.state ?? result) as Record<string, unknown>;
}

async function waitForAgentChatState(
  predicate: (s: Record<string, unknown>) => boolean,
  label: string,
  timeoutMs = 20000,
): Promise<Record<string, unknown>> {
  const deadline = Date.now() + timeoutMs;
  let last: Record<string, unknown> = {};
  while (Date.now() < deadline) {
    last = await agentChatState();
    if (predicate(last)) return last;
    await new Promise((r) => setTimeout(r, 250));
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

try {
  // 1. Open a live Agent Chat (thread A). openAi emits no command receipt,
  // so fire-and-forget and poll getAgentChatState for the live thread.
  driver.send({ type: "openAi" });
  const initial = await waitForAgentChatState(
    (s) => typeof s.status === "string" && s.status !== "setup",
    "live agent chat",
  );
  check(
    "opens with empty thread pool",
    initial.retained_thread_count === 0 ||
      initial.retainedThreadCount === 0,
    initial.retainedThreadCount ?? initial.retained_thread_count,
  );

  // 2. Draft text in thread A's composer.
  await driver.request(
    { type: "setAgentChatInput", text: "alpha draft for thread A" },
    { timeoutMs: 10000 },
  );
  const drafted = await waitForAgentChatState(
    (s) => String(s.inputText ?? s.input_text ?? "").includes("alpha draft"),
    "draft text in composer",
  );
  check("draft typed into thread A", true, drafted.inputText ?? drafted.input_text);

  // 3. Cmd+N → new thread B, thread A retained.
  driver.simulateKey("n", ["cmd"]);
  const afterNew = await waitForAgentChatState(
    (s) => Number(s.retainedThreadCount ?? s.retained_thread_count ?? 0) === 1,
    "retained_thread_count == 1 after Cmd+N",
    30000,
  );
  const freshInput = String(afterNew.inputText ?? afterNew.input_text ?? "");
  check("thread A retained after Cmd+N", true, afterNew.retainedThreadCount ?? afterNew.retained_thread_count);
  check("new thread composer is empty", freshInput === "", freshInput);

  // 4. Cmd+K → Threads section screenshot.
  driver.simulateKey("k", ["cmd"]);
  await new Promise((r) => setTimeout(r, 700));
  mkdirSync(".test-screenshots", { recursive: true });
  const shot = (await driver.captureScreenshot({
    target: { type: "kind", kind: "actionsDialog" },
    savePath: ".test-screenshots/agent-chat-threads-actions.png",
    timeoutMs: 15000,
  })) as Record<string, unknown>;
  check(
    "actions dialog screenshot captured",
    shot.error == null,
    { saved: ".test-screenshots/agent-chat-threads-actions.png", error: shot.error ?? null },
  );

  // 5. Search "switch" in the dialog and accept → back on thread A.
  for (const ch of "switch") driver.simulateKey(ch, []);
  await new Promise((r) => setTimeout(r, 500));
  const dialogShot = (await driver.captureScreenshot({
    target: { type: "kind", kind: "actionsDialog" },
    savePath: ".test-screenshots/agent-chat-threads-switch-filtered.png",
    timeoutMs: 15000,
  })) as Record<string, unknown>;
  check(
    "switch row filtered screenshot captured",
    dialogShot.error == null,
    { saved: ".test-screenshots/agent-chat-threads-switch-filtered.png" },
  );
  driver.simulateKey("enter", []);
  const afterSwitch = await waitForAgentChatState(
    (s) =>
      String(s.inputText ?? s.input_text ?? "").includes("alpha draft") &&
      Number(s.retainedThreadCount ?? s.retained_thread_count ?? 0) === 1,
    "thread A active again with B retained",
    20000,
  );
  check("switched back to thread A (draft restored)", true, afterSwitch.inputText ?? afterSwitch.input_text);
  check(
    "thread B retained after switch",
    Number(afterSwitch.retainedThreadCount ?? afterSwitch.retained_thread_count) === 1,
    afterSwitch.retainedThreadCount ?? afterSwitch.retained_thread_count,
  );

  receipt.classification = checks.every((c) => c.pass) ? "ok" : "reproduced-failure";
} catch (error) {
  receipt.error = String(error);
  receipt.classification = "blocked";
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.classification !== "ok") process.exit(1);
