#!/usr/bin/env bun
import { readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = "template-prompt-parity";
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
  let parsed: Json;
  try {
    parsed = JSON.parse(stdout);
  } catch (error) {
    throw new Error(`invalid JSON from session.sh: ${stdout}\n${String(error)}`);
  }
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

function openTemplate(id: string, template: string): void {
  send({ type: "template", id, template, requestId: `${id}-open` });
  const state = getState(`${id}-opened`);
  if (state.promptType !== "template" || state.promptId !== id) {
    throw new Error(`${id}: expected template state, got ${JSON.stringify(state)}`);
  }
}

function getState(tag: string): Json {
  const state = rpc(
    {
      type: "getState",
      requestId: `tpl-state-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult") {
    throw new Error(`${tag}: expected stateResult, got ${JSON.stringify(state)}`);
  }
  return state;
}

function getElements(tag: string, target?: Json): Json {
  const command: Json = {
    type: "getElements",
    requestId: `tpl-elements-${tag}-${Date.now()}`,
    limit: 80,
  };
  if (target) command.target = target;
  const elements = rpc(command, "elementsResult");
  if (elements.type !== "elementsResult") {
    throw new Error(`${tag}: expected elementsResult, got ${JSON.stringify(elements)}`);
  }
  return elements;
}

function elementById(elements: Json, semanticId: string): Json | undefined {
  return (elements.elements ?? []).find((element: Json) => element.semanticId === semanticId);
}

function assert(condition: unknown, label: string, details?: unknown): void {
  if (!condition) {
    throw new Error(`${label}${details === undefined ? "" : `: ${JSON.stringify(details)}`}`);
  }
}

function logOffset(): number {
  if (!logPath) return 0;
  try {
    return statSync(logPath).size;
  } catch {
    return 0;
  }
}

function waitForLog(offset: number, needle: string, label: string): string {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    let text = "";
    try {
      text = readFileSync(logPath, "utf8").slice(offset);
    } catch {
      text = "";
    }
    if (text.includes(needle)) return text;
    Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 50);
  }
  throw new Error(`${label}: did not find ${JSON.stringify(needle)} in ${logPath}`);
}

function proveSubmit(): Json {
  const id = `tpl-submit-${Date.now()}`;
  openTemplate(id, "Hello {{name}}");
  let elements = getElements("submit-open");
  assert(elementById(elements, "input:template-source"), "template source element exists", elements);
  assert(elementById(elements, "input:template-name"), "template name element exists", elements);

  rpc(
    {
      type: "batch",
      requestId: `tpl-fill-${Date.now()}`,
      commands: [{ type: "setInput", text: "Ada" }],
    },
    "batchResult",
  );
  elements = getElements("submit-filled");
  assert(
    elementById(elements, "input:template-name")?.value === "Ada",
    "template field updated through batch.setInput",
    elements,
  );

  const offset = logOffset();
  send({ type: "simulateKey", requestId: `${id}-enter`, key: "enter", modifiers: [] });
  waitForLog(offset, "SimulateKey: Enter - submit TemplatePrompt", "TemplatePrompt Enter log");
  return { id, submittedVia: "simulateKey Enter", field: "Ada" };
}

function proveCancel(): Json {
  const id = `tpl-cancel-${Date.now()}`;
  openTemplate(id, "Cancel {{name}}");
  const offset = logOffset();
  send({ type: "simulateKey", requestId: `${id}-escape`, key: "escape", modifiers: [] });
  waitForLog(offset, "SimulateKey: Escape - cancel TemplatePrompt", "TemplatePrompt Escape log");
  return { id, cancelled: true };
}

function proveForceSubmit(): Json {
  const id = `tpl-force-${Date.now()}`;
  openTemplate(id, "Ignored {{name}}");
  const batch = rpc(
    {
      type: "batch",
      requestId: `tpl-force-batch-${Date.now()}`,
      commands: [{ type: "forceSubmit", value: "forced-template-result" }],
    },
    "batchResult",
  );
  assert(batch.success === true, "forceSubmit batch succeeded", batch);
  assert(
    batch.results?.[0]?.value === "forced-template-result",
    "forceSubmit returned explicit value",
    batch,
  );
  return { id, forced: batch.results[0].value };
}

function proveActions(): Json {
  const id = `tpl-actions-${Date.now()}`;
  openTemplate(id, "Actions {{name}}");
  const before = getState("actions-before");
  const buttons = before.activeFooter?.buttons ?? [];
  assert(
    buttons.some((button: Json) => button.action === "actions" && button.enabled !== false),
    "TemplatePrompt footer exposes enabled Actions button",
    before.activeFooter,
  );

  const offset = logOffset();
  send({ type: "simulateKey", key: "k", modifiers: ["cmd"], requestId: `${id}-cmd-k` });
  const log = waitForLog(offset, "actions popup receipt event=OpenRequested host=TemplatePrompt", "TemplatePrompt actions open receipt");
  assert(
    log.includes("ActionsDialog created with"),
    "TemplatePrompt actions dialog created shared actions",
    log,
  );
  send({ type: "simulateKey", key: "escape", modifiers: [], requestId: `${id}-escape-actions` });
  return { id, host: "TemplatePrompt", openedVia: "simulateKey Cmd+K" };
}

function main(): void {
  const start = runSession(["start", session]);
  logPath = start.log;
  const receipts: Json[] = [];
  try {
    receipts.push(proveSubmit());
    receipts.push(proveCancel());
    receipts.push(proveForceSubmit());
    receipts.push(proveActions());
  } finally {
    send({ type: "simulateKey", key: "escape", modifiers: [], requestId: "tpl-cleanup-escape" });
    runSession(["stop", session]);
  }

  console.log(JSON.stringify({ status: "ok", receipts }, null, 2));
}

main();
