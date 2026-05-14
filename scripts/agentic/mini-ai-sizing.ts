#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const outDir = join(repoRoot, ".test-output", "mini-window-contract");
mkdirSync(outDir, { recursive: true });

const session = arg("--session", "mini-ai-sizing");
const timeoutMs = Number(arg("--timeout", "8000"));
const MINI_WIDTH = 480;
const FULL_WIDTH = 750;

function arg(name: string, fallback: string): string {
  const i = process.argv.indexOf(name);
  return i >= 0 ? process.argv[i + 1] ?? fallback : fallback;
}

function run(cmd: string[], input?: string): string {
  const proc = spawnSync(cmd[0], cmd.slice(1), {
    cwd: repoRoot,
    input,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (proc.status !== 0) {
    throw new Error(`${cmd.join(" ")} failed\n${proc.stdout}\n${proc.stderr}`);
  }
  return proc.stdout.trim();
}

function parseEnvelope(raw: string): Json {
  const lines = raw.trim().split(/\n/).filter(Boolean);
  for (let i = lines.length - 1; i >= 0; i -= 1) {
    try {
      return JSON.parse(lines[i]);
    } catch {
      // Keep scanning for the machine-readable session envelope.
    }
  }
  const miniAiMatch = raw.match(/"miniAi":(\{[^{}]*\})/);
  if (miniAiMatch) {
    return { status: "ok", response: { miniAi: JSON.parse(miniAiMatch[1]) } };
  }
  throw new Error(`No JSON envelope in output:\n${raw}`);
}

function rpc(payload: Json, expect: string): Json {
  const requestId = payload.requestId ?? `${payload.type}-${Date.now()}-${Math.random().toString(16).slice(2)}`;
  const raw = run(["bash", "scripts/agentic/session.sh", "rpc", session, JSON.stringify({ requestId, ...payload }), "--expect", expect, "--timeout", String(timeoutMs)]);
  const envelope = parseEnvelope(raw);
  if (envelope.status !== "ok") throw new Error(`${payload.type} failed: ${raw}`);
  return envelope.response;
}

function send(payload: Json): void {
  run(["bash", "scripts/agentic/session.sh", "send", session, JSON.stringify({ requestId: `${payload.type}-${Date.now()}`, ...payload }), "--await-parse", "--timeout", String(timeoutMs)]);
}

function state(tag: string): Json {
  return rpc({ type: "getState", requestId: `mini-ai-sizing-state-${tag}-${Date.now()}` }, "stateResult");
}

function windows(tag: string): Json {
  return rpc({ type: "listAutomationWindows", requestId: `mini-ai-sizing-windows-${tag}-${Date.now()}` }, "automationWindowListResult");
}

function waitHidden(): Json {
  return rpc(
    {
      type: "waitFor",
      condition: { type: "stateMatch", state: { windowVisible: false } },
      timeout: timeoutMs,
      pollInterval: 50,
    },
    "waitForResult",
  );
}

function waitPrompt(promptType: string): Json {
  return rpc(
    {
      type: "waitFor",
      condition: { type: "stateMatch", state: { promptType } },
      timeout: timeoutMs,
      pollInterval: 50,
    },
    "waitForResult",
  );
}

function mainWidth(tag: string): number {
  const list = windows(tag);
  const main = list.windows?.find((w: Json) => w.id === "main" || w.kind === "main");
  const width = Number(main?.bounds?.width);
  if (!Number.isFinite(width)) throw new Error(`${tag}: missing main bounds in ${JSON.stringify(list)}`);
  return width;
}

function assertNear(label: string, actual: number, expected: number): void {
  if (Math.abs(actual - expected) > 4) {
    throw new Error(`${label}: expected width ${expected}, got ${actual}`);
  }
}

async function openInlineChat(id: string): Promise<void> {
  send({
    type: "chat",
    id,
    requestId: id,
    placeholder: "Ask anything...",
    messages: [],
    actions: [],
    models: [],
    saveHistory: false,
    useBuiltinAi: false,
  });
  await Bun.sleep(300);
}

run(["bash", "scripts/agentic/session.sh", "start", session]);
send({ type: "show" });
send({ type: "triggerBuiltin", builtinId: "builtin/mini-main-window" });
await openInlineChat("inline-ai-sizing-mini");
const miniState = state("mini");
if (miniState.miniAi?.visible !== true || miniState.miniAi?.mainWindowMode !== "mini") {
  throw new Error(`mini ChatPrompt state mismatch: ${JSON.stringify(miniState.miniAi)}`);
}
assertNear("mini inline chat", mainWidth("mini"), MINI_WIDTH);

send({ type: "triggerBuiltin", builtinId: "builtin/choose-theme" });
await openInlineChat("inline-ai-sizing-full");
const fullState = state("full");
if (fullState.miniAi?.visible !== true || fullState.miniAi?.mainWindowMode !== "full") {
  throw new Error(`full ChatPrompt state mismatch: ${JSON.stringify(fullState.miniAi)}`);
}
assertNear("full inline chat", mainWidth("full"), FULL_WIDTH);

const hide = rpc({ type: "hide", requestId: `mini-ai-sizing-hide-${Date.now()}` }, "windowVisibilityAck");
const hidden = waitHidden();
if (!hidden.success) throw new Error(`cleanup failed: ${JSON.stringify(hidden)}`);

const receipt = { miniState: miniState.miniAi, fullState: fullState.miniAi, hide, hidden };
writeFileSync(join(outDir, "mini-ai-sizing.json"), JSON.stringify(receipt, null, 2));
console.log(JSON.stringify(receipt, null, 2));
