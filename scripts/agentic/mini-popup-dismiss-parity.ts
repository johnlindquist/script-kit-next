#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "path";

type Json = Record<string, any>;
const repoRoot = resolve(import.meta.dir, "../..");
const outDir = join(repoRoot, ".test-output", "mini-window-contract");
mkdirSync(outDir, { recursive: true });
const session = arg("--session", "mini-popup-dismiss-parity");
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
  return rpc({ type: "getState", requestId: `popup-dismiss-state-${tag}-${Date.now()}` }, "stateResult");
}
function rawState(tag: string): string {
  return run([
    "bash",
    "scripts/agentic/session.sh",
    "rpc",
    session,
    JSON.stringify({
      type: "getState",
      requestId: `popup-dismiss-state-${tag}-${Date.now()}`,
    }),
    "--expect",
    "stateResult",
    "--timeout",
    String(timeoutMs),
  ]);
}
function waitHidden(): Json {
  return rpc({ type: "waitFor", condition: { type: "stateMatch", state: { windowVisible: false } }, timeout: timeoutMs, pollInterval: 50 }, "waitForResult");
}
async function openActionsAndDismiss(tag: string, entry: Json): Promise<Json> {
  send(entry);
  await Bun.sleep(150);
  send({ type: "simulateKey", key: "k", modifiers: ["cmd"] });
  await Bun.sleep(250);
  const open = rawState(`${tag}-open`);
  if (!open.includes('"activePopupContract"')) throw new Error(`${tag}: actions did not open`);
  send({ type: "simulateKey", key: "escape", modifiers: [] });
  await Bun.sleep(250);
  const closed = rawState(`${tag}-closed`);
  if (closed.includes('"activePopupContract"')) throw new Error(`${tag}: actions did not close`);
  return { open: true, closed: true };
}

run(["bash", "scripts/agentic/session.sh", "start", session]);
send({ type: "show" });
const mini = await openActionsAndDismiss("mini", { type: "triggerBuiltin", builtinId: "builtin/mini-main-window" });
const full = await openActionsAndDismiss("full", { type: "triggerBuiltin", builtinId: "builtin/choose-theme" });
const hide = rpc({ type: "hide" }, "windowVisibilityAck");
const hidden = waitHidden();
if (!hidden.success) throw new Error(`cleanup failed: ${JSON.stringify(hidden)}`);
const receipt = { mini, full, hide, hidden };
writeFileSync(join(outDir, "mini-popup-dismiss-parity.json"), JSON.stringify(receipt, null, 2));
console.log(JSON.stringify(receipt, null, 2));
