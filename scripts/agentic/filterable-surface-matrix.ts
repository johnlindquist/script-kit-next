#!/usr/bin/env bun
/**
 * State-first verification matrix for filterable launcher surfaces.
 *
 * The matrix is intentionally data-first: each migrated surface declares the
 * entry command, promptType, list semantic id, and filter text that must keep
 * `getState.visibleChoiceCount` aligned with `getElements`.
 *
 * Usage:
 *   bun scripts/agentic/filterable-surface-matrix.ts --list
 *   bun scripts/agentic/filterable-surface-matrix.ts --session aurp06-filterable
 *   bun scripts/agentic/filterable-surface-matrix.ts --case current-app-commands-visible-rows
 */

import { resolve } from "path";

const SCHEMA_VERSION = 1;
const PROJECT_ROOT = resolve(import.meta.dir, "../..");

type JsonObject = Record<string, unknown>;

interface FilterableSurfaceMatrixEntry {
  id: string;
  surface: string;
  promptType: string;
  listSemanticId: string;
  entryCommand: JsonObject;
  filterText: string;
  expectedElementChromeCount: number;
}

interface RpcEnvelope {
  status: "ok" | "error";
  response?: JsonObject;
  error?: { code?: string; message?: string };
}

interface StepReceipt {
  name: string;
  command: JsonObject;
  response: JsonObject;
}

interface CountObservation {
  choiceCount: number;
  visibleChoiceCount: number;
  listCount: number;
  elementsTotalCount: number;
  selectedValue: string | null;
}

// @lat: [[lat.md/automation#Automation#Operational Rules]]
export const FILTERABLE_SURFACE_MATRIX: FilterableSurfaceMatrixEntry[] = [
  {
    id: "current-app-commands-visible-rows",
    surface: "currentAppCommands",
    promptType: "currentAppCommands",
    listSemanticId: "list:menu-commands",
    entryCommand: { type: "triggerBuiltin", name: "current-app-commands" },
    filterText: "workspace",
    expectedElementChromeCount: 2,
  },
  {
    id: "clipboard-history-visible-rows",
    surface: "clipboardHistory",
    promptType: "clipboardHistory",
    listSemanticId: "list:clipboard-history",
    entryCommand: { type: "triggerBuiltin", name: "clipboard-history" },
    filterText: "__aurp11_no_clipboard_match__",
    expectedElementChromeCount: 2,
  },
  {
    id: "emoji-picker-visible-rows",
    surface: "emojiPicker",
    promptType: "emojiPicker",
    listSemanticId: "list:emoji-results",
    entryCommand: { type: "triggerBuiltin", name: "emoji" },
    filterText: "heart",
    expectedElementChromeCount: 2,
  },
  {
    id: "app-launcher-visible-rows",
    surface: "appLauncher",
    promptType: "appLauncher",
    listSemanticId: "list:apps",
    entryCommand: { type: "triggerBuiltin", name: "apps" },
    filterText: "__aurp16_no_app_match__",
    expectedElementChromeCount: 2,
  },
  {
    id: "design-gallery-visible-rows",
    surface: "designGallery",
    promptType: "designGallery",
    listSemanticId: "list:design-gallery",
    entryCommand: { type: "triggerBuiltin", name: "design-gallery" },
    filterText: "icon",
    expectedElementChromeCount: 2,
  },
  {
    id: "process-manager-visible-rows",
    surface: "processManager",
    promptType: "processManager",
    listSemanticId: "list:processes",
    entryCommand: { type: "triggerBuiltin", name: "process-manager" },
    filterText: "__aurp16_no_process_match__",
    expectedElementChromeCount: 2,
  },
];

function hasFlag(flag: string): boolean {
  return process.argv.includes(flag);
}

function argValue(flag: string, fallback: string): string {
  const index = process.argv.indexOf(flag);
  if (index < 0) {
    return fallback;
  }
  return process.argv[index + 1] ?? fallback;
}

function selectedCases(caseId: string): FilterableSurfaceMatrixEntry[] {
  if (caseId === "all") {
    return FILTERABLE_SURFACE_MATRIX;
  }
  const entry = FILTERABLE_SURFACE_MATRIX.find((candidate) => candidate.id === caseId);
  if (!entry) {
    throw new Error(`Unknown filterable surface matrix case: ${caseId}`);
  }
  return [entry];
}

async function runTool(cmd: string[], label: string): Promise<JsonObject> {
  const proc = Bun.spawn(cmd, {
    cwd: PROJECT_ROOT,
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = (await new Response(proc.stdout).text()).trim();
  const stderr = (await new Response(proc.stderr).text()).trim();
  const exitCode = await proc.exited;
  if (exitCode !== 0) {
    throw new Error(`${label} failed: ${stdout || stderr || `exit ${exitCode}`}`);
  }
  try {
    return JSON.parse(stdout);
  } catch (error) {
    throw new Error(`${label} returned non-JSON output: ${stdout}`);
  }
}

async function sessionStart(session: string): Promise<JsonObject> {
  return runTool(["bash", "scripts/agentic/session.sh", "start", session], "session.start");
}

async function sessionStop(session: string): Promise<JsonObject> {
  return runTool(["bash", "scripts/agentic/session.sh", "stop", session], "session.stop");
}

async function sendAndAwaitParse(
  session: string,
  command: JsonObject,
  timeoutMs: number,
): Promise<JsonObject> {
  const receipt = await runTool(
    [
      "bash",
      "scripts/agentic/session.sh",
      "send",
      session,
      JSON.stringify(command),
      "--await-parse",
      "--timeout",
      String(timeoutMs),
    ],
    `send.${String(command.type)}`,
  );
  if (receipt.parseOutcome !== "parsed") {
    throw new Error(
      `Command ${String(command.type)} did not parse: ${JSON.stringify(receipt)}`,
    );
  }
  return receipt;
}

async function rpc(
  session: string,
  command: JsonObject,
  expect: string,
  timeoutMs: number,
): Promise<JsonObject> {
  const envelope = (await runTool(
    [
      "bash",
      "scripts/agentic/session.sh",
      "rpc",
      session,
      JSON.stringify(command),
      "--expect",
      expect,
      "--timeout",
      String(timeoutMs),
    ],
    `rpc.${String(command.type)}`,
  )) as RpcEnvelope;
  if (envelope.status !== "ok" || !envelope.response) {
    throw new Error(`RPC ${String(command.type)} failed: ${JSON.stringify(envelope)}`);
  }
  return envelope.response;
}

function numberField(source: JsonObject, key: string): number {
  const value = source[key];
  if (typeof value !== "number") {
    throw new Error(`Expected numeric ${key}, got ${JSON.stringify(value)}`);
  }
  return value;
}

function stringField(source: JsonObject, key: string): string {
  const value = source[key];
  if (typeof value !== "string") {
    throw new Error(`Expected string ${key}, got ${JSON.stringify(value)}`);
  }
  return value;
}

function elementsFrom(response: JsonObject): JsonObject[] {
  const elements = response.elements;
  if (!Array.isArray(elements)) {
    throw new Error("elementsResult response must contain an elements array");
  }
  return elements as JsonObject[];
}

function listCountFromElements(response: JsonObject, listSemanticId: string): number {
  const list = elementsFrom(response).find(
    (element) => element.semanticId === listSemanticId,
  );
  if (!list) {
    throw new Error(`Missing list element ${listSemanticId}`);
  }
  const text = stringField(list, "text");
  const match = text.match(/^(\d+) items?$/);
  if (!match) {
    throw new Error(`List ${listSemanticId} text is not an item count: ${text}`);
  }
  return Number(match[1]);
}

function observeCounts(
  entry: FilterableSurfaceMatrixEntry,
  state: JsonObject,
  elements: JsonObject,
): CountObservation {
  const promptType = stringField(state, "promptType");
  if (promptType !== entry.promptType) {
    throw new Error(
      `${entry.id}: expected promptType ${entry.promptType}, got ${promptType}`,
    );
  }

  const choiceCount = numberField(state, "choiceCount");
  const visibleChoiceCount = numberField(state, "visibleChoiceCount");
  const listCount = listCountFromElements(elements, entry.listSemanticId);
  const elementsTotalCount = numberField(elements, "totalCount");

  if (visibleChoiceCount > choiceCount) {
    throw new Error(
      `${entry.id}: visibleChoiceCount ${visibleChoiceCount} exceeds choiceCount ${choiceCount}`,
    );
  }
  if (listCount !== visibleChoiceCount) {
    throw new Error(
      `${entry.id}: list count ${listCount} differs from visibleChoiceCount ${visibleChoiceCount}`,
    );
  }
  if (elementsTotalCount !== visibleChoiceCount + entry.expectedElementChromeCount) {
    throw new Error(
      `${entry.id}: elements totalCount ${elementsTotalCount} does not equal visible rows plus chrome`,
    );
  }

  return {
    choiceCount,
    visibleChoiceCount,
    listCount,
    elementsTotalCount,
    selectedValue:
      typeof state.selectedValue === "string" ? state.selectedValue : null,
  };
}

async function runEntry(
  session: string,
  entry: FilterableSurfaceMatrixEntry,
  timeoutMs: number,
): Promise<JsonObject> {
  const steps: StepReceipt[] = [];

  await sendAndAwaitParse(session, entry.entryCommand, timeoutMs);
  await sendAndAwaitParse(session, { type: "setFilter", text: "" }, timeoutMs);

  const emptyStateCommand = {
    type: "getState",
    requestId: `${entry.id}-empty-state`,
  };
  const emptyElementsCommand = {
    type: "getElements",
    requestId: `${entry.id}-empty-elements`,
    limit: 200,
  };
  const emptyState = await rpc(session, emptyStateCommand, "stateResult", timeoutMs);
  const emptyElements = await rpc(
    session,
    emptyElementsCommand,
    "elementsResult",
    timeoutMs,
  );
  steps.push({ name: "empty.getState", command: emptyStateCommand, response: emptyState });
  steps.push({
    name: "empty.getElements",
    command: emptyElementsCommand,
    response: emptyElements,
  });

  const setFilterCommand = { type: "setFilter", text: entry.filterText };
  await sendAndAwaitParse(session, setFilterCommand, timeoutMs);

  const filteredStateCommand = {
    type: "getState",
    requestId: `${entry.id}-filtered-state`,
  };
  const filteredElementsCommand = {
    type: "getElements",
    requestId: `${entry.id}-filtered-elements`,
    limit: 200,
  };
  const filteredState = await rpc(
    session,
    filteredStateCommand,
    "stateResult",
    timeoutMs,
  );
  const filteredElements = await rpc(
    session,
    filteredElementsCommand,
    "elementsResult",
    timeoutMs,
  );
  steps.push({
    name: "filtered.getState",
    command: filteredStateCommand,
    response: filteredState,
  });
  steps.push({
    name: "filtered.getElements",
    command: filteredElementsCommand,
    response: filteredElements,
  });

  const empty = observeCounts(entry, emptyState, emptyElements);
  const filtered = observeCounts(entry, filteredState, filteredElements);

  return {
    id: entry.id,
    status: "pass",
    surface: entry.surface,
    observations: { empty, filtered },
    steps,
  };
}

async function main(): Promise<void> {
  if (hasFlag("--list")) {
    process.stdout.write(
      `${JSON.stringify({
        schemaVersion: SCHEMA_VERSION,
        matrix: FILTERABLE_SURFACE_MATRIX,
      })}\n`,
    );
    return;
  }

  const session = argValue("--session", "filterable-surface-matrix");
  const timeoutMs = Number(argValue("--timeout", "5000"));
  const caseId = argValue("--case", "all");
  const keepSession = hasFlag("--keep-session");
  const cases = selectedCases(caseId);
  let startedSession = false;

  try {
    await sessionStart(session);
    startedSession = true;
    const caseReceipts = [];
    for (const entry of cases) {
      caseReceipts.push(await runEntry(session, entry, timeoutMs));
    }
    if (!keepSession) {
      await sessionStop(session);
    }
    process.stdout.write(
      `${JSON.stringify({
        schemaVersion: SCHEMA_VERSION,
        status: "pass",
        session,
        startedSession,
        cases: caseReceipts,
      })}\n`,
    );
  } catch (error) {
    if (startedSession && !keepSession) {
      await sessionStop(session).catch(() => undefined);
    }
    process.stdout.write(
      `${JSON.stringify({
        schemaVersion: SCHEMA_VERSION,
        status: "fail",
        session,
        error: error instanceof Error ? error.message : String(error),
      })}\n`,
    );
    process.exit(1);
  }
}

await main();
