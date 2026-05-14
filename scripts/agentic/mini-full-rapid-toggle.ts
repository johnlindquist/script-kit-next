#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "path";

type Json = Record<string, any>;
const repoRoot = resolve(import.meta.dir, "../..");
const outDir = join(repoRoot, ".test-output", "mini-window-contract");
mkdirSync(outDir, { recursive: true });
const session = arg("--session", "mini-full-rapid-toggle");
const timeoutMs = Number(arg("--timeout", "8000"));
function arg(name: string, fallback: string): string {
  const i = process.argv.indexOf(name);
  return i >= 0 ? process.argv[i + 1] ?? fallback : fallback;
}
function run(cmd: string[]): string {
  const proc = spawnSync(cmd[0], cmd.slice(1), {
    cwd: repoRoot,
    encoding: "utf8",
    maxBuffer: 64 * 1024 * 1024,
  });
  if (proc.status !== 0) throw new Error(`${cmd.join(" ")} failed\n${proc.stdout}\n${proc.stderr}`);
  return proc.stdout.trim();
}
function parseEnvelope(raw: string): Json {
  const lines = raw.trim().split(/\n/).filter(Boolean);
  for (let i = lines.length - 1; i >= 0; i -= 1) {
    try {
      return JSON.parse(lines[i]);
    } catch {
      // Keep scanning for session.sh JSON.
    }
  }
  const miniAiMatch = raw.match(/"miniAi":(\{[^{}]*\})/);
  if (miniAiMatch) {
    return { status: "ok", response: { miniAi: JSON.parse(miniAiMatch[1]) } };
  }
  throw new Error(`No JSON envelope in output:\n${raw}`);
}
function rpc(payload: Json, expect: string): Json {
  const raw = run(["bash", "scripts/agentic/session.sh", "rpc", session, JSON.stringify({ requestId: `${payload.type}-${Date.now()}`, ...payload }), "--expect", expect, "--timeout", String(timeoutMs)]);
  const envelope = parseEnvelope(raw);
  if (envelope.status !== "ok") throw new Error(raw);
  return envelope.response;
}
function send(payload: Json): void {
  run(["bash", "scripts/agentic/session.sh", "send", session, JSON.stringify({ requestId: `${payload.type}-${Date.now()}`, ...payload }), "--await-parse", "--timeout", String(timeoutMs)]);
}
function state(tag: string): Json {
  return rpc({ type: "getState", requestId: `rapid-toggle-state-${tag}-${Date.now()}` }, "stateResult");
}
function waitHidden(): Json {
  return rpc({ type: "waitFor", condition: { type: "stateMatch", state: { windowVisible: false } }, timeout: timeoutMs, pollInterval: 50 }, "waitForResult");
}
function waitPrompt(promptType: string): Json {
  return rpc({ type: "waitFor", condition: { type: "stateMatch", state: { promptType } }, timeout: timeoutMs, pollInterval: 50 }, "waitForResult");
}

run(["bash", "scripts/agentic/session.sh", "start", session]);
send({ type: "show" });
for (let i = 0; i < 10; i += 1) {
  send({ type: "triggerBuiltin", builtinId: i % 2 === 0 ? "builtin/choose-theme" : "builtin/mini-main-window" });
}
send({
  type: "chat",
  id: "inline-ai-rapid-toggle",
  requestId: "inline-ai-rapid-toggle",
  placeholder: "Ask anything...",
  messages: [],
  actions: [],
  models: [],
  saveHistory: false,
  useBuiltinAi: false,
});
await Bun.sleep(300);
const final = state("final");
if (final.miniAi?.visible !== true || final.miniAi?.mainWindowMode !== "mini") {
  throw new Error(`expected final coherent mini ChatPrompt state: ${JSON.stringify(final.miniAi)}`);
}
const hide = rpc({ type: "hide" }, "windowVisibilityAck");
const hidden = waitHidden();
if (!hidden.success) throw new Error(`cleanup failed: ${JSON.stringify(hidden)}`);
const receipt = { final: final.miniAi, hide, hidden };
writeFileSync(join(outDir, "mini-full-rapid-toggle.json"), JSON.stringify(receipt, null, 2));
console.log(JSON.stringify(receipt, null, 2));
