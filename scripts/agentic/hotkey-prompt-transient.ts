#!/usr/bin/env bun
import { createHash } from "node:crypto";
import { existsSync, readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = "hotkey-prompt-transient";
const timeoutMs = 8000;
let logPath = "";

function runSession(args: string[]): Json {
  const result = spawnSync(sessionScript, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
  });
  const stdout = result.stdout.trim();
  if (!stdout) {
    throw new Error(
      `session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`,
    );
  }
  const parsed = JSON.parse(stdout);
  if (result.status !== 0 || parsed.status === "error") {
    throw new Error(
      `session.sh ${args.join(" ")} failed: ${JSON.stringify(parsed)} stderr=${result.stderr.trim()}`,
    );
  }
  return parsed;
}

function send(command: Json): Json {
  return runSession([
    "send",
    session,
    JSON.stringify(command),
    "--await-parse",
    "--timeout",
    String(timeoutMs),
  ]);
}

function rpc(command: Json, expect: string): Json {
  const envelope = runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeoutMs),
  ]);
  return envelope.response;
}

function assert(condition: unknown, label: string, details?: unknown): void {
  if (!condition) {
    throw new Error(`${label}${details === undefined ? "" : `: ${JSON.stringify(details)}`}`);
  }
}

function fileOffset(path: string): number {
  try {
    return statSync(path).size;
  } catch {
    return 0;
  }
}

function configFingerprint(): string {
  const configPath = `${process.env.HOME}/.scriptkit/config.ts`;
  if (!existsSync(configPath)) return "missing";
  const bytes = readFileSync(configPath);
  return createHash("sha256").update(bytes).digest("hex");
}

function waitForSubmit(offset: number, id: string): Json {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    let text = "";
    try {
      text = readFileSync(logPath, "utf8").slice(offset);
    } catch {
      text = "";
    }
    for (const line of text.split(/\n+/)) {
      if (!line.trim()) continue;
      const jsonStart = line.indexOf("{");
      if (jsonStart === -1) continue;
      let parsed: Json;
      try {
        parsed = JSON.parse(line.slice(jsonStart));
      } catch {
        continue;
      }
      if (parsed.type === "submit" && parsed.id === id) return parsed;
    }
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 50);
  }
  throw new Error(`submit response for ${id} not found in ${logPath}`);
}

function openHotkey(id: string, placeholder: string): void {
  send({ type: "hotkey", id, placeholder, requestId: `${id}-open` });
  const state = rpc(
    { type: "getState", requestId: `${id}-state-open` },
    "stateResult",
  );
  assert(state.promptType === "hotkey", "hotkey promptType", state);
  assert(state.promptId === id, "hotkey prompt id", state);

  const elements = rpc(
    { type: "getElements", requestId: `${id}-elements-open`, limit: 20 },
    "elementsResult",
  );
  const semanticIds = (elements.elements ?? []).map((element: Json) => element.semanticId);
  assert(semanticIds.includes("panel:hotkey-capture"), "capture panel element", semanticIds);
  assert(semanticIds.includes("input:hotkey-shortcut"), "shortcut input element", semanticIds);
}

function proveCapture(): Json {
  const id = `hotkey-capture-${Date.now()}`;
  const beforeConfig = configFingerprint();
  const offset = fileOffset(logPath);
  openHotkey(id, "Press a keyboard shortcut");
  send({ type: "simulateKey", requestId: `${id}-cmd-shift-k`, key: "k", modifiers: ["cmd", "shift"] });
  const submit = waitForSubmit(offset, id);
  assert(typeof submit.value === "string", "capture submit value", submit);
  const value = JSON.parse(submit.value);
  assert(value.key === "K", "captured key", value);
  assert(value.command === true, "captured command modifier", value);
  assert(value.shift === true, "captured shift modifier", value);
  assert(value.option === false, "captured option modifier", value);
  assert(value.control === false, "captured control modifier", value);
  assert(value.shortcut.includes("K"), "captured shortcut display", value);
  assert(configFingerprint() === beforeConfig, "capture did not mutate config.ts");
  return { id, value };
}

function proveCancel(): Json {
  const id = `hotkey-cancel-${Date.now()}`;
  const beforeConfig = configFingerprint();
  const offset = fileOffset(logPath);
  openHotkey(id, "Cancel me");
  send({ type: "simulateKey", requestId: `${id}-escape`, key: "escape", modifiers: [] });
  const submit = waitForSubmit(offset, id);
  assert(submit.value === null, "cancel submits null", submit);
  assert(configFingerprint() === beforeConfig, "cancel did not mutate config.ts");
  return { id, cancelled: true };
}

function main(): void {
  const start = runSession(["start", session]);
  logPath = start.log;
  const receipts: Json[] = [];
  try {
    receipts.push(proveCapture());
    receipts.push(proveCancel());
  } finally {
    runSession(["stop", session]);
  }
  console.log(JSON.stringify({ status: "ok", receipts }, null, 2));
}

main();
