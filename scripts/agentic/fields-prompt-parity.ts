#!/usr/bin/env bun
import { readFileSync, statSync } from "node:fs";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = "fields-prompt-parity";
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

function openFields(id: string, actions = false): void {
  send({
    type: "fields",
    id,
    fields: [
      { name: "name", label: "Name", placeholder: "Ada" },
      { name: "email", label: "Email", type: "email", placeholder: "ada@example.com" },
    ],
    actions: actions
      ? [
          {
            name: "Inspect",
            description: "Fields prompt parity action",
            value: "inspect",
            hasAction: false,
          },
        ]
      : undefined,
  });
  const state = getState(`${id}-opened`);
  assert(state.promptType === "fields", "fields prompt reports promptType fields", state);
  assert(state.promptId === id, "fields prompt id is preserved", state);
  assert(
    state.activeFooter?.activeSurface === "form_prompt",
    "fields prompt reuses the form prompt native footer host",
    state.activeFooter,
  );
}

function getState(tag: string): Json {
  const state = rpc(
    {
      type: "getState",
      requestId: `fields-state-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult") {
    throw new Error(`${tag}: expected stateResult, got ${JSON.stringify(state)}`);
  }
  return state;
}

function getElements(tag: string): Json {
  const elements = rpc(
    {
      type: "getElements",
      requestId: `fields-elements-${tag}-${Date.now()}`,
      limit: 80,
    },
    "elementsResult",
  );
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

function proveSubmitAndValidation(): Json {
  const id = `fields-submit-${Date.now()}`;
  openFields(id);
  let elements = getElements("submit-open");
  assert(elementById(elements, "list:fields-fields"), "fields list element exists", elements);
  assert(elementById(elements, "input:fields-name"), "name input element exists", elements);
  assert(elementById(elements, "input:fields-email"), "email input element exists", elements);

  rpc(
    {
      type: "batch",
      requestId: `fields-name-${Date.now()}`,
      commands: [{ type: "setInput", text: "Ada" }],
    },
    "batchResult",
  );
  rpc(
    {
      type: "batch",
      requestId: `fields-email-focus-invalid-${Date.now()}`,
      commands: [
        { type: "selectBySemanticId", semanticId: "input:fields-email", submit: false },
        { type: "setInput", text: "not-an-email" },
      ],
    },
    "batchResult",
  );

  send({ type: "simulateKey", requestId: `${id}-invalid-enter`, key: "enter", modifiers: [] });
  Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, 200);
  assert(getState("after-invalid").promptType === "fields", "invalid email keeps prompt open");

  rpc(
    {
      type: "batch",
      requestId: `fields-email-valid-${Date.now()}`,
      commands: [
        { type: "selectBySemanticId", semanticId: "input:fields-email", submit: false },
        { type: "setInput", text: "ada@example.com" },
      ],
    },
    "batchResult",
  );
  elements = getElements("before-valid-submit");
  assert(
    elementById(elements, "input:fields-email")?.value === "ada@example.com",
    "valid email value reached focused fields input",
    elements,
  );
  const submitOffset = logOffset();
  send({ type: "simulateKey", requestId: `${id}-valid-enter`, key: "enter", modifiers: [] });
  const submitLog = waitForLog(submitOffset, "Enter in FormPrompt - submitting form", "fields enter submit");
  return { id, submittedVia: "simulateKey Enter", submitLogMatched: submitLog.length > 0 };
}

function proveForceSubmit(): Json {
  const id = `fields-force-${Date.now()}`;
  openFields(id);
  const batch = rpc(
    {
      type: "batch",
      requestId: `fields-force-batch-${Date.now()}`,
      commands: [{ type: "forceSubmit", value: ["forced-name", "forced-email"] }],
    },
    "batchResult",
  );
  assert(batch.success === true, "fields forceSubmit batch succeeded", batch);
  assert(
    batch.results?.[0]?.value === "[\"forced-name\",\"forced-email\"]",
    "fields forceSubmit returned the explicit array JSON value",
    batch,
  );
  return { id, forced: batch.results[0].value };
}

function proveActions(): Json {
  const id = `fields-actions-${Date.now()}`;
  openFields(id, true);
  const before = getState("actions-before");
  const buttons = before.activeFooter?.buttons ?? [];
  assert(
    buttons.some((button: Json) => button.action === "actions" && button.enabled !== false),
    "fields prompt footer exposes enabled Actions button",
    before.activeFooter,
  );

  const offset = logOffset();
  send({ type: "simulateKey", key: "k", modifiers: ["cmd"], requestId: `${id}-cmd-k` });
  const log = waitForLog(offset, "actions popup receipt event=OpenRequested host=FormPrompt", "fields actions open receipt");
  send({ type: "simulateKey", key: "escape", modifiers: [], requestId: `${id}-escape-actions` });
  return { id, host: "FormPrompt", openedVia: "simulateKey Cmd+K", logMatched: log.length > 0 };
}

function proveCancel(): Json {
  const id = `fields-cancel-${Date.now()}`;
  openFields(id);
  const offset = logOffset();
  send({ type: "simulateKey", requestId: `${id}-escape`, key: "escape", modifiers: [] });
  const log = waitForLog(offset, "SimulateKey: Escape - cancel FormPrompt", "fields cancel log");
  return { id, cancelled: true, logMatched: log.length > 0 };
}

function main(): void {
  const start = runSession(["start", session]);
  logPath = start.log;
  const receipts: Json[] = [];
  try {
    receipts.push(proveSubmitAndValidation());
    receipts.push(proveForceSubmit());
    receipts.push(proveActions());
    receipts.push(proveCancel());
  } finally {
    send({ type: "simulateKey", key: "escape", modifiers: [], requestId: "fields-cleanup-escape" });
    runSession(["stop", session]);
  }

  console.log(JSON.stringify({ status: "ok", receipts }, null, 2));
}

main();
