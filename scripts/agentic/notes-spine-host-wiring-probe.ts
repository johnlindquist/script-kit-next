#!/usr/bin/env bun
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ?? "target-agent/artifacts/notes-spine-host/script-kit-gpui";
const runId = `notes-spine-host-${Date.now().toString(36)}`;
const target = { type: "kind", kind: "notes", index: 0 };

type Check = { name: string; ok: boolean; detail: Json };
const checks: Check[] = [];
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json = {}) {
  checks.push({ name, ok, detail });
  if (!ok) failures.push(name);
}

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") out.push(json);
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

async function openNotes(driver: Driver) {
  driver.send({ type: "openNotes", requestId: `${runId}-open-notes` });
  await Bun.sleep(700);
}

async function notesState(driver: Driver): Promise<Json> {
  const result = (await driver.request(
    { type: "getState", target },
    { expect: "stateResult", timeoutMs: 5000 },
  )) as Json;
  return (result.notes ?? result) as Json;
}

async function notesElements(driver: Driver): Promise<Json[]> {
  const result = (await driver.getElements({ target, limit: 180 }, { timeoutMs: 5000 })) as Json;
  return walkElements(result);
}

async function editorValue(driver: Driver): Promise<string | null> {
  const elements = await notesElements(driver);
  const editor = elements.find(
    (el) => el.semanticId === "input:notes-editor" || el.id === "notes-editor",
  );
  return typeof editor?.value === "string" ? editor.value : null;
}

async function setNotesText(driver: Driver, text: string): Promise<Json> {
  return (await driver.request(
    {
      type: "batch",
      requestId: `${runId}-set-${text.replace(/[^a-z0-9]+/gi, "-")}-${Date.now()}`,
      target,
      commands: [{ type: "setInput", text }],
      options: { stopOnError: true, timeout: 5000 },
    },
    { expect: "batchResult", timeoutMs: 6000 },
  )) as Json;
}

async function waitForSpineRow(driver: Driver, expectedId: string): Promise<Json> {
  let last: Json = {};
  for (let i = 0; i < 40; i += 1) {
    const started = performance.now();
    const state = await notesState(driver);
    const elapsedMs = Math.round(performance.now() - started);
    const spine = (state.spine ?? {}) as Json;
    last = { state, spine, elapsedMs };
    const ids = Array.isArray(spine.rowSemanticIds) ? spine.rowSemanticIds.map(String) : [];
    if (spine.active === true && ids.includes(expectedId)) {
      return { state, spine, elapsedMs, ids };
    }
    await Bun.sleep(50);
  }
  throw new Error(`Timed out waiting for Notes spine row ${expectedId}: ${JSON.stringify(last)}`);
}

async function proveReplacement(
  driver: Driver,
  spec: { name: string; seed: string; rowId: string; expected: string },
) {
  const setResult = await setNotesText(driver, spec.seed);
  check(`${spec.name}.set_input`, setResult.success === true, { setResult });
  const receipt = await waitForSpineRow(driver, spec.rowId);
  check(`${spec.name}.row_visible`, true, {
    rowId: spec.rowId,
    selectedSemanticId: receipt.spine.selectedSemanticId ?? null,
    rowCount: receipt.spine.rowCount ?? null,
    elapsedMs: receipt.elapsedMs,
  });
  await driver.simulateKey("enter");
  await Bun.sleep(100);
  const value = await editorValue(driver);
  check(`${spec.name}.accepted`, value === spec.expected, {
    expected: spec.expected,
    actual: value,
  });
}

async function proveAtContextDoesNotOpenNotesLocalUi(driver: Driver) {
  await setNotesText(driver, "@note");
  await Bun.sleep(150);
  const state = await notesState(driver);
  check("at_context_no_notes_spine", (state.spine as Json | undefined)?.active !== true, {
    spine: state.spine ?? null,
  });
  check("at_context_no_note_switcher_auto_open", state.view?.showBrowsePanel !== true, {
    showBrowsePanel: state.view?.showBrowsePanel ?? null,
  });
}

async function proveAgentChatStillOpens(driver: Driver) {
  await setNotesText(driver, "Agent Chat handoff proof");
  await driver.simulateKey("enter", ["cmd"]);
  for (let i = 0; i < 50; i += 1) {
    const state = await notesState(driver);
    if (state.view?.surfaceMode === "AgentChat") {
      check("cmd_enter_opens_notes_agent_chat", true, {
        surfaceMode: state.view.surfaceMode,
      });
      return;
    }
    await Bun.sleep(100);
  }
  const state = await notesState(driver);
  check("cmd_enter_opens_notes_agent_chat", false, {
    surfaceMode: state.view?.surfaceMode ?? null,
  });
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "notes-spine-host-wiring",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  await openNotes(driver);

  await proveReplacement(driver, {
    name: "slash_rewrite",
    seed: "/rew",
    rowId: "spine:/:rewrite",
    expected: "/rewrite ",
  });
  await proveReplacement(driver, {
    name: "style_professional",
    seed: ".pro",
    rowId: "spine:.:professional",
    expected: ".professional ",
  });
  await proveReplacement(driver, {
    name: "capture_todo",
    seed: ";to",
    rowId: "spine:;:todo",
    expected: "todo; ",
  });
  await proveReplacement(driver, {
    name: "filter_script",
    seed: ":type:s",
    rowId: "spine:::qualifier:type:script",
    expected: ":type:script ",
  });

  await proveAtContextDoesNotOpenNotesLocalUi(driver);
  await proveAgentChatStillOpens(driver);

  const logText = await Bun.file(driver.logPath).text();
  check("no_runtime_panic", !/panicked at|thread 'main' panicked/i.test(logText), {
    logPath: driver.logPath,
  });
  check("no_gpui_entity_double_lease", !/already mutably borrowed|double lease/i.test(logText), {
    logPath: driver.logPath,
  });

  const maxStateMs = Math.max(
    0,
    ...checks
      .map((entry) => Number(entry.detail?.elapsedMs ?? 0))
      .filter((value) => Number.isFinite(value)),
  );
  check("state_receipts_instant", maxStateMs < 250, { maxStateMs });

  console.log(
    JSON.stringify(
      {
        status: failures.length === 0 ? "pass" : "fail",
        scenario: "notes-spine-host-wiring",
        failures,
        checks,
        sessionDir: driver.sessionDir,
        logPath: driver.logPath,
      },
      null,
      2,
    ),
  );
  if (failures.length > 0) process.exit(1);
} finally {
  await driver.close();
}
