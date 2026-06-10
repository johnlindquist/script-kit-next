#!/usr/bin/env bun
/**
 * Runtime proof for Agent Chat "Rewind & Edit Message" (Feature D, Pi fork).
 *
 * Drives a real Pi-backed Agent Chat through two genuine LLM turns, then
 * rewinds to the second user message via the Cmd+K drill-down:
 * 1. openAi → live thread (fork_point_count = 0)
 * 2. submit "Reply with exactly: OK"  → fork_point_count = 1
 * 3. submit "Reply with exactly: DONE" → fork_point_count = 2
 * 4. Cmd+K → "Rewind & Edit Message" drill-down (screenshots)
 * 5. Enter on the preselected latest message → transcript truncates to the
 *    first exchange, composer prefills the second message's text.
 *
 * Auth: copies the dev machine's ~/.pi/agent/{auth,settings}.json and
 * ~/.codex/auth.json into the sandbox HOME so Pi can run real model turns
 * without touching real Script Kit state.
 *
 * Usage: bun scripts/agentic/agent-chat-rewind-probe.ts [binaryPath]
 */
import { Driver } from "../devtools/driver.ts";
import { mkdirSync, copyFileSync, existsSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

const binary =
  process.argv[2] ?? "target-agent/artifacts/agent-chat-threads/script-kit-gpui";

const driver = await Driver.launch({
  sessionName: "agent-chat-rewind-probe",
  sandboxHome: true,
  binary,
  defaultTimeoutMs: 15000,
  env: {
    // Debug-build NSPanel invariants mismatch in headless driver sessions;
    // unrelated to the rewind flow under proof here.
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});

// Seed Pi/Codex auth into the sandbox HOME before the first agent spawn so
// real (tiny) model turns work without the user's real Script Kit state.
const sandboxHome = join(driver.sessionDir, "home");
for (const [src, dst] of [
  [join(homedir(), ".pi/agent/auth.json"), join(sandboxHome, ".pi/agent/auth.json")],
  [join(homedir(), ".pi/agent/settings.json"), join(sandboxHome, ".pi/agent/settings.json")],
  [join(homedir(), ".codex/auth.json"), join(sandboxHome, ".codex/auth.json")],
] as const) {
  if (existsSync(src)) {
    mkdirSync(join(dst, ".."), { recursive: true });
    copyFileSync(src, dst);
  }
}

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "agent-chat-rewind-probe",
  binary,
  classification: "blocked",
  checks: [] as Array<Record<string, unknown>>,
};
const checks = receipt.checks as Array<Record<string, unknown>>;

function check(name: string, pass: boolean, detail: unknown) {
  checks.push({ name, pass, detail });
}

// In-app capture (CGWindowListCreateImageFromArray) silently loses its
// Screen Recording grant whenever the debug binary is rebuilt (new cdhash).
// Fall back to OS-level `screencapture` — the app floats above everything,
// so a display grab still shows the dialog under proof.
async function captureDialog(savePath: string): Promise<{ ok: boolean; via: string; detail: unknown }> {
  const shot = (await driver.captureScreenshot({
    target: { type: "kind", kind: "actionsDialog" },
    savePath,
    timeoutMs: 15000,
  })) as Record<string, unknown>;
  if (shot.error == null) return { ok: true, via: "captureScreenshot", detail: savePath };
  const os = Bun.spawnSync(["screencapture", "-x", "-D", "1", savePath]);
  const wrote = os.exitCode === 0 && existsSync(savePath);
  return {
    ok: wrote,
    via: "screencapture",
    detail: wrote ? { savePath, inAppError: shot.error } : { inAppError: shot.error, osExit: os.exitCode },
  };
}

async function agentChatState(): Promise<Record<string, unknown>> {
  const result = (await driver.request(
    { type: "getAgentChatState" },
    { timeoutMs: 10000 },
  )) as Record<string, unknown>;
  return (result.state ?? result) as Record<string, unknown>;
}

function num(state: Record<string, unknown>, key: string): number {
  return Number(state[key] ?? state[key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`)] ?? 0);
}

async function waitForAgentChatState(
  predicate: (s: Record<string, unknown>) => boolean,
  label: string,
  timeoutMs = 60000,
): Promise<Record<string, unknown>> {
  const deadline = Date.now() + timeoutMs;
  let last: Record<string, unknown> = {};
  while (Date.now() < deadline) {
    last = await agentChatState();
    if (predicate(last)) return last;
    await new Promise((r) => setTimeout(r, 400));
  }
  throw new Error(
    `timeout waiting for ${label}: status=${last.status} messages=${last.messageCount ?? last.message_count} forkPoints=${last.forkPointCount ?? last.fork_point_count} input=${JSON.stringify(last.inputText ?? last.input_text)}`,
  );
}

async function submitTurn(text: string, expectedForkPoints: number) {
  await driver.request({ type: "setAgentChatInput", text }, { timeoutMs: 10000 });
  driver.simulateKey("enter", []);
  return waitForAgentChatState(
    (s) =>
      s.status === "idle" &&
      num(s, "forkPointCount") === expectedForkPoints &&
      String(s.inputText ?? s.input_text ?? "") === "",
    `turn "${text}" finished with ${expectedForkPoints} fork point(s)`,
    90000,
  );
}

try {
  // 1. Open a live Agent Chat. Let the Pi connection finish initializing
  // before submitting — a prompt sent within ~100ms of openAi races the
  // sidecar's session setup and the turn's set_model handshake times out.
  driver.send({ type: "openAi" });
  const initial = await waitForAgentChatState(
    (s) => s.status === "idle" && s.contextReady === true,
    "live agent chat",
    20000,
  );
  await new Promise((r) => setTimeout(r, 1500));
  check("opens with no fork points", num(initial, "forkPointCount") === 0, initial.forkPointCount);

  // 2-3. Two real turns; each finished turn refreshes the rewind checkpoints.
  const afterFirst = await submitTurn("Reply with exactly: OK", 1);
  check("first turn yields one fork point", true, {
    messages: num(afterFirst, "messageCount"),
    forkPoints: num(afterFirst, "forkPointCount"),
  });
  const afterSecond = await submitTurn("Reply with exactly: DONE", 2);
  const messagesBeforeRewind = num(afterSecond, "messageCount");
  check("second turn yields two fork points", true, {
    messages: messagesBeforeRewind,
    forkPoints: 2,
  });

  // 4. Cmd+K → Rewind & Edit drill-down.
  driver.simulateKey("k", ["cmd"]);
  await new Promise((r) => setTimeout(r, 700));
  mkdirSync(".test-screenshots", { recursive: true });
  const rootShot = await captureDialog(".test-screenshots/agent-chat-rewind-root.png");
  check("root actions screenshot captured", rootShot.ok, rootShot);

  for (const ch of "rewind") driver.simulateKey(ch, []);
  await new Promise((r) => setTimeout(r, 400));
  driver.simulateKey("enter", []); // drill into the fork picker
  await new Promise((r) => setTimeout(r, 700));
  const pickerShot = await captureDialog(".test-screenshots/agent-chat-rewind-picker.png");
  check("fork picker screenshot captured", pickerShot.ok, pickerShot);

  // 5. Accept the preselected latest message → fork.
  driver.simulateKey("enter", []);
  const afterRewind = await waitForAgentChatState(
    (s) =>
      String(s.inputText ?? s.input_text ?? "").includes("Reply with exactly: DONE") &&
      num(s, "messageCount") < messagesBeforeRewind,
    "transcript truncated and composer prefilled",
    30000,
  );
  check(
    "composer prefilled with the rewound message",
    String(afterRewind.inputText ?? afterRewind.input_text).includes("Reply with exactly: DONE"),
    afterRewind.inputText ?? afterRewind.input_text,
  );
  check(
    "transcript truncated to the first exchange",
    num(afterRewind, "messageCount") < messagesBeforeRewind,
    { before: messagesBeforeRewind, after: num(afterRewind, "messageCount") },
  );

  // Fork point list refreshes from the rebuilt session (one user message left).
  const refreshed = await waitForAgentChatState(
    (s) => num(s, "forkPointCount") === 1,
    "fork points refreshed after rewind",
    20000,
  );
  check("fork points refreshed to pre-rewind history", true, num(refreshed, "forkPointCount"));

  receipt.classification = checks.every((c) => c.pass) ? "ok" : "reproduced-failure";
} catch (error) {
  receipt.error = String(error);
  receipt.classification = "blocked";
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.classification !== "ok") process.exit(1);
