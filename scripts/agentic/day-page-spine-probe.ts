#!/usr/bin/env bun
/**
 * Runtime proof for Day Page current-line spine behavior.
 *
 * Drives the real main-hotkey Day Page path, sets the active editor text through
 * the transaction input primitive, then accepts the shared @file row through
 * the same stdin simulateKey path used by the main spine probes.
 */
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/day-page-spine/script-kit-gpui";

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

async function simulateMainHotkeyGesture(
  driver: Driver,
  phase: "down" | "up",
  requestId: string,
) {
  return driver.request(
    { type: "simulateMainHotkeyGesture", phase, requestId },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  );
}

async function tapHotkey(driver: Driver, label: string) {
  await simulateMainHotkeyGesture(driver, "down", `${runId}-${label}-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${runId}-${label}-up`);
  await Bun.sleep(400);
}

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") {
    out.push(json);
  }
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

async function editorText(driver: Driver): Promise<string | null> {
  const elements = (await driver.getElements(
    { target: { type: "main" } },
    { timeoutMs: 5000 },
  )) as Json;
  const editor = walkElements(elements).find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  return (editor?.value as string | undefined) ?? null;
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-spine",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  await simulateMainHotkeyGesture(driver, "down", `${runId}-show-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${runId}-show-up`);
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 8000 });
  await Bun.sleep(400);

  let maybeDayState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  if (maybeDayState.promptType !== "dayPage") {
    await driver.setFilterAndWait("day page spine seed", { timeoutMs: 5000 });
    await tapHotkey(driver, "toggle-day-page");
    maybeDayState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  }

  check("opened_day_page", maybeDayState.promptType === "dayPage", {
    promptType: maybeDayState.promptType,
  });

  const batch = (await driver.batch(
    [
      { type: "setInput", text: "@fi" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: "@fi" },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("batch_set_day_page_input", batch.success === true, { batch });

  await Bun.sleep(250);
  const beforeText = await editorText(driver);
  const elementsBefore = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const flatBefore = walkElements(elementsBefore);
  const fileRow = flatBefore.find((el) => el.semanticId === "spine:@:subsearch:file");
  const spinePanel = flatBefore.find(
    (el) => el.semanticId === "list:day-page-spine-list" || el.id === "day-page-spine-list",
  );

  check("editor_contains_fragment", beforeText === "@fi", { beforeText });
  check("spine_panel_visible", Boolean(spinePanel), {
    ids: flatBefore.slice(0, 20).map((el) => el.semanticId ?? el.id),
  });
  check("files_row_visible", Boolean(fileRow), {
    fileRow: fileRow ?? null,
  });

  await driver.simulateKey("escape");
  await Bun.sleep(250);
  const dismissedElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const dismissedFlat = walkElements(dismissedElements);
  const dismissedPanel = dismissedFlat.find(
    (el) => el.semanticId === "list:day-page-spine-list" || el.id === "day-page-spine-list",
  );
  check("escape_dismisses_spine_elements", !dismissedPanel, {
    ids: dismissedFlat.slice(0, 12).map((el) => el.semanticId ?? el.id),
  });

  const restoreBatch = (await driver.batch(
    [
      { type: "setInput", text: "@fi" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: "@fi" },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("batch_restore_day_page_spine_input", restoreBatch.success === true, {
    batch: restoreBatch,
  });

  const restoredElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const restoredFlat = walkElements(restoredElements);
  check("spine_elements_restore_after_input_change", Boolean(
    restoredFlat.find((el) => el.semanticId === "spine:@:subsearch:file"),
  ), {
    selectedSemanticId: restoredElements.selectedSemanticId ?? null,
    ids: restoredFlat.slice(0, 12).map((el) => el.semanticId ?? el.id),
  });

  await Bun.sleep(250);
  await driver.simulateKey("enter");
  await Bun.sleep(250);
  const afterText = await editorText(driver);
  check("enter_completes_file_fragment", afterText === "@file:", { afterText });

  const fileBatch = (await driver.batch(
    [{ type: "setInput", text: "@file:" }],
    { timeoutMs: 5000 },
  )) as Json;
  check("batch_set_empty_file_subsearch", fileBatch.success === true, { batch: fileBatch });

  const immediateFileElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const immediateFlat = walkElements(immediateFileElements);
  check("empty_file_subsearch_immediate_elements", Boolean(
    immediateFlat.find((el) => el.semanticId === "list:day-page-spine-list"),
  ), {
    selectedSemanticId: immediateFileElements.selectedSemanticId ?? null,
    ids: immediateFlat.slice(0, 12).map((el) => el.semanticId ?? el.id),
  });
  check("empty_file_subsearch_unarmed_selection", !immediateFileElements.selectedSemanticId, {
    selectedSemanticId: immediateFileElements.selectedSemanticId ?? null,
  });

  await driver.simulateKey("down");
  await Bun.sleep(250);
  const armedFileElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  check("empty_file_subsearch_down_arms_selection", Boolean(armedFileElements.selectedSemanticId), {
    selectedSemanticId: armedFileElements.selectedSemanticId ?? null,
  });

  const submitBatch = (await driver.batch(
    [
      { type: "setInput", text: "@selection summarize this" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: "@selection summarize this" },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("batch_set_day_page_submit_prompt", submitBatch.success === true, {
    batch: submitBatch,
  });

  const submitElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const submitFlat = walkElements(submitElements);
  const readyRow = submitFlat.find((el) => el.semanticId === "spine:tail:ready");
  check("submit_prompt_ready_row_visible", Boolean(readyRow), {
    readyRow: readyRow ?? null,
    selectedSemanticId: submitElements.selectedSemanticId ?? null,
  });

  await driver.simulateKey("enter");
  await Bun.sleep(750);
  const afterSubmitState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("enter_submits_day_page_prompt_to_agent_chat", afterSubmitState.promptType === "agentChatChat", {
    promptType: afterSubmitState.promptType,
    inputValue: afterSubmitState.inputValue,
  });

  const pass = failures.length === 0;
  console.log(
    JSON.stringify(
      {
        pass,
        failures,
        sessionDir: driver.sessionDir,
        receipts,
      },
      null,
      2,
    ),
  );
  if (!pass) process.exitCode = 1;
} finally {
  await driver.close();
}
