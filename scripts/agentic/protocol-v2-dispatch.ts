#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import { readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = "protocol-v2-dispatch";
const timeoutMs = 8000;
let logPath = "";

function runSession(args: string[]): Json {
  const result = spawnSync(sessionScript, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
    maxBuffer: 16 * 1024 * 1024,
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

function waitForLog(offset: number, predicate: (line: string) => boolean, label: string): string {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    let text = "";
    try {
      text = readFileSync(logPath, "utf8").slice(offset);
    } catch {
      text = "";
    }
    for (const line of text.split(/\n+/)) {
      if (line.trim() && predicate(line)) return line;
    }
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 50);
  }
  throw new Error(`${label} not found in ${logPath}`);
}

function waitForPromptType(promptType: string): Json {
  const deadline = Date.now() + timeoutMs;
  let lastState: Json | undefined;
  while (Date.now() < deadline) {
    lastState = rpc(
      {
        type: "getState",
        requestId: `state-${promptType}-${Date.now()}`,
        protocolVersion: 2,
      },
      "stateResult",
    );
    if (lastState.promptType === promptType) return lastState;
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 50);
  }
  throw new Error(`promptType ${promptType} not observed; last=${JSON.stringify(lastState)}`);
}

function main(): void {
  const start = runSession(["start", session]);
  logPath = start.log;
  const receipts: Json[] = [];
  try {
    const trigger = send({
      type: "triggerBuiltin",
      builtinId: "builtin/clipboard-history",
      requestId: "clipboard-v2",
      protocolVersion: 2,
    });
    assert(trigger.parseOutcome === "parsed", "v2 triggerBuiltin parsed", trigger);
    const clipboardState = waitForPromptType("clipboardHistory");
    receipts.push({
      command: "triggerBuiltin",
      parseOutcome: trigger.parseOutcome,
      commandType: trigger.commandType,
      promptType: clipboardState.promptType,
      surfaceKind: clipboardState.surfaceContract?.surfaceKind,
    });
    receipts.push({
      command: "getState",
      responseType: clipboardState.type,
      requestId: clipboardState.requestId,
      promptType: clipboardState.promptType,
    });

    const offset = fileOffset(logPath);
    const show = send({ type: "show", requestId: "show-v2", protocolVersion: 2 });
    assert(show.parseOutcome === "parsed", "v2 show parsed", show);
    const ackLine = waitForLog(
      offset,
      (line) =>
        line.includes('"type":"windowVisibilityAck"') &&
        line.includes('"requestId":"show-v2"') &&
        line.includes('"windowVisible":true'),
      "v2 show windowVisibilityAck",
    );
    receipts.push({
      command: "show",
      parseOutcome: show.parseOutcome,
      commandType: show.commandType,
      ack: JSON.parse(ackLine.slice(ackLine.indexOf("{"))),
    });
  } finally {
    runSession(["stop", session]);
  }
  console.log(JSON.stringify({ status: "ok", receipts }, null, 2));
}

main();
