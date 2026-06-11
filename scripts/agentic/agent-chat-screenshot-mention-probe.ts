#!/usr/bin/env bun
/**
 * Runtime proof for the Agent Chat `@screenshot` mention.
 *
 * Red (pre-fix): submitting "@screenshot ..." logged
 * `agent_chat_context_special_block_capture_failed` with
 * "CGWindowListCreateImageFromArray returned null" — the legacy CGWindowList
 * capture APIs were obsoleted in macOS 15 and return null, so the chip never
 * became an image block.
 *
 * Green (post-fix): the same user path logs
 * `agent_chat_inline_screenshot_attachment_captured` with non-zero
 * width/height/bytes, proving the ScreenCaptureKit backend captured the
 * active desktop and attached it as a real image block.
 *
 * Usage: bun scripts/agentic/agent-chat-screenshot-mention-probe.ts [binaryPath]
 */
import { Driver } from "../devtools/driver.ts";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const binary =
  process.argv[2] ?? "target-agent/artifacts/sck-screenshot/script-kit-gpui";

const driver = await Driver.launch({
  sessionName: "agent-chat-screenshot-mention-probe",
  sandboxHome: true,
  binary,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    // Worktree/artifact builds embed a dev pi-sidecar path that may not
    // exist; resolve the repo-local sidecar explicitly.
    SCRIPT_KIT_PI_BINARY: join(
      import.meta.dir,
      "../../target/pi-sidecar/pi",
    ),
  },
});

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "agent-chat-screenshot-mention-probe",
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

function appLog(): string {
  try {
    return readFileSync(join(driver.sessionDir, "app.log"), "utf8");
  } catch {
    return "";
  }
}

async function waitForLog(
  needle: string,
  label: string,
  timeoutMs = 25000,
  fromOffset = 0,
): Promise<string | null> {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const log = appLog().slice(fromOffset);
    const line = log
      .split("\n")
      .find((entry) => entry.includes(needle));
    if (line) return line;
    await new Promise((r) => setTimeout(r, 300));
  }
  return null;
}

try {
  // 1. Open a live Agent Chat through the real user entry path.
  driver.send({ type: "openAi" });
  await waitForAgentChatState(
    (s) => typeof s.status === "string" && s.status !== "setup",
    "live agent chat",
    30000,
  );
  // Settle: composer/model wiring races the first submit right after open.
  await new Promise((r) => setTimeout(r, 1500));

  // 2. Type the @screenshot mention into the composer. setAgentChatInput and
  // legacy simulateKey do NOT run sync_inline_mentions (only the real GPUI
  // input pipeline does), so set all but the last character via stdin, then
  // type the final character through simulateGpuiEvent — the real-dispatch
  // path — to arm the mention as a pending context part.
  const offsetBeforeTyping = appLog().length;
  await driver.request(
    { type: "setAgentChatInput", text: "@screenshot what is on my scree" },
    { timeoutMs: 10000 },
  );
  await driver.request(
    {
      type: "simulateGpuiEvent",
      target: { type: "kind", kind: "main" },
      event: { type: "keyDown", key: "n", text: "n" },
    },
    { timeoutMs: 10000 },
  );
  const drafted = await waitForAgentChatState(
    (s) =>
      String(s.inputText ?? s.input_text ?? "") ===
      "@screenshot what is on my screen",
    "@screenshot draft in composer",
  );
  check(
    "@screenshot typed into composer",
    true,
    drafted.inputText ?? drafted.input_text,
  );
  const mentionArmed = await waitForLog(
    "agent_chat_pending_context_armed",
    "pending context armed by inline mention",
    10000,
    offsetBeforeTyping,
  );
  check("inline @screenshot mention armed", mentionArmed != null, mentionArmed);

  // 3. Submit. The screenshot chip resolves during submit, before any
  // model/auth dependency, so the capture event is observable even if the
  // sandboxed Pi thread cannot authenticate.
  driver.simulateKey("enter", []);

  const captured = await waitForLog(
    "agent_chat_inline_screenshot_attachment_captured",
    "screenshot attachment captured",
    30000,
  );
  const failed = appLog()
    .split("\n")
    .find((line) => line.includes("agent_chat_context_special_block_capture_failed"));

  check("screenshot capture event logged", captured != null, captured);
  check("no capture-failed event", failed == null, failed ?? null);

  const specialBlock = await waitForLog(
    "agent_chat_context_part_resolved_to_special_block",
    "chip resolved to image block",
    5000,
  );
  check(
    "chip resolved to special image block (not text fallback)",
    specialBlock != null,
    specialBlock,
  );

  if (captured) {
    const width = Number(/width=(\d+)/.exec(captured)?.[1] ?? 0);
    const height = Number(/height=(\d+)/.exec(captured)?.[1] ?? 0);
    const bytes = Number(/bytes=(\d+)/.exec(captured)?.[1] ?? 0);
    check("captured dimensions are plausible", width >= 800 && height >= 600, {
      width,
      height,
    });
    check("captured PNG is non-trivial", bytes > 50_000, { bytes });
    receipt.capture = { width, height, bytes };
  }

  receipt.classification = checks.every((c) => c.pass)
    ? "fixed"
    : "reproduced-failure";
} catch (error) {
  receipt.error = String(error);
  receipt.classification = "blocked";
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.classification !== "fixed") process.exit(1);
