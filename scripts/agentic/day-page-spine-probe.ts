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

async function setDayPageInput(driver: Driver, text: string, label: string) {
  const batch = (await driver.batch(
    [
      { type: "setInput", text },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: text },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check(`batch_set_${label}`, batch.success === true, { batch });
  await Bun.sleep(150);
}

async function verifyFragmentCompletion(
  driver: Driver,
  label: string,
  text: string,
  expectedSemanticId: string | ((id: string) => boolean),
  expectedText: string | ((beforeId: string) => string),
) {
  await setDayPageInput(driver, text, label);
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 240 },
    { timeoutMs: 5000 },
  )) as Json;
  const flat = walkElements(elements);
  const row = flat.find((el) => {
    if (typeof el.semanticId !== "string") return false;
    return typeof expectedSemanticId === "string"
      ? el.semanticId === expectedSemanticId
      : expectedSemanticId(el.semanticId);
  });
  check(`row_visible_${label}`, Boolean(row), {
    expectedSemanticId: typeof expectedSemanticId === "string" ? expectedSemanticId : "predicate",
    selectedSemanticId: elements.selectedSemanticId ?? null,
    row: row ?? null,
    ids: flat.slice(0, 20).map((el) => el.semanticId ?? el.id),
  });
  check(`row_selected_${label}`, Boolean(row) && elements.selectedSemanticId === row?.semanticId, {
    selectedSemanticId: elements.selectedSemanticId ?? null,
    rowSemanticId: row?.semanticId ?? null,
  });

  await driver.simulateKey("enter");
  await Bun.sleep(200);
  const afterText = await editorText(driver);
  const expected =
    typeof expectedText === "string" ? expectedText : expectedText(String(row?.semanticId ?? ""));
  check(`enter_completes_${label}`, afterText === expected, { afterText, expected });
}

async function verifyModeExitRow(driver: Driver, label: string, text: string, expectedSemanticId: string) {
  await setDayPageInput(driver, text, label);
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 120 },
    { timeoutMs: 5000 },
  )) as Json;
  const flat = walkElements(elements);
  const row = flat.find((el) => el.semanticId === expectedSemanticId);
  check(`mode_exit_row_visible_${label}`, Boolean(row), {
    expectedSemanticId,
    selectedSemanticId: elements.selectedSemanticId ?? null,
    row: row ?? null,
    ids: flat.slice(0, 16).map((el) => el.semanticId ?? el.id),
  });
  check(
    `mode_exit_row_selected_${label}`,
    Boolean(row) && elements.selectedSemanticId === expectedSemanticId,
    { selectedSemanticId: elements.selectedSemanticId ?? null },
  );
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

  await verifyFragmentCompletion(
    driver,
    "context_builtin_selection",
    "@sel",
    "spine:@:builtin:selection",
    "@selection ",
  );

  await verifyFragmentCompletion(
    driver,
    "slash_rewrite",
    "/rew",
    "spine:/:rewrite",
    "/rewrite ",
  );

  await verifyFragmentCompletion(
    driver,
    "style_professional",
    ".pro",
    "spine:.:professional",
    ".professional ",
  );

  await verifyFragmentCompletion(
    driver,
    "capture_todo",
    ";to",
    "spine:;:todo",
    "todo; ",
  );

  await verifyFragmentCompletion(
    driver,
    "filter_script",
    ":type:s",
    "spine:::qualifier:type:script",
    ":type:script ",
  );

  await verifyFragmentCompletion(
    driver,
    "profile_first",
    "|",
    (id) => id.startsWith("spine:|:"),
    (id) => `|${id.replace("spine:|:", "")} `,
  );

  await verifyModeExitRow(driver, "tilde_file_search", "~readme", "spine:~:mode-exit");
  await verifyModeExitRow(driver, "bang_terminal", "!echo hi", "spine:!:mode-exit");
  await verifyModeExitRow(driver, "question_actions", "?", "spine:?:mode-exit");

  const plainNoCwdBatch = (await driver.batch(
    [
      { type: "setInput", text: "plain text before cwd" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: "plain text before cwd" },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("batch_set_day_page_plain_without_cwd", plainNoCwdBatch.success === true, {
    batch: plainNoCwdBatch,
  });

  await driver.simulateKey("enter", ["cmd"]);
  await Bun.sleep(250);
  const afterPlainNoCwdState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "cmd_enter_plain_without_cwd_stays_on_day_page",
    afterPlainNoCwdState.promptType === "dayPage" &&
      afterPlainNoCwdState.inputValue === "plain text before cwd",
    {
      promptType: afterPlainNoCwdState.promptType,
      inputValue: afterPlainNoCwdState.inputValue,
    },
  );

  const cwdBatch = (await driver.batch(
    [
      { type: "setInput", text: ">d" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: ">d" },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("batch_set_day_page_cwd_prompt", cwdBatch.success === true, {
    batch: cwdBatch,
  });

  const cwdElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const cwdFlat = walkElements(cwdElements);
  const cwdRow = cwdFlat.find(
    (el) => typeof el.semanticId === "string" && el.semanticId.startsWith("spine:>:dir:"),
  );
  check(
    "cwd_row_visible_and_selected",
    Boolean(cwdRow) && cwdElements.selectedSemanticId === cwdRow?.semanticId,
    {
      cwdRow: cwdRow ?? null,
      selectedSemanticId: cwdElements.selectedSemanticId ?? null,
    },
  );

  await driver.simulateKey("enter");
  const afterCwdText = await editorText(driver);
  check("enter_sets_cwd_and_strips_segment", afterCwdText === "", { afterCwdText });

  const unresolvedAfterCwdBatch = (await driver.batch(
    [
      { type: "setInput", text: "@clip" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: "@clip" },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("batch_set_day_page_unresolved_context_after_cwd", unresolvedAfterCwdBatch.success === true, {
    batch: unresolvedAfterCwdBatch,
  });

  await driver.simulateKey("enter", ["cmd"]);
  await Bun.sleep(250);
  const afterUnresolvedContextState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "cmd_enter_unresolved_context_after_cwd_stays_on_day_page",
    afterUnresolvedContextState.promptType === "dayPage" &&
      afterUnresolvedContextState.inputValue === "@clip",
    {
      promptType: afterUnresolvedContextState.promptType,
      inputValue: afterUnresolvedContextState.inputValue,
    },
  );

  const cwdPromptBatch = (await driver.batch(
    [
      { type: "setInput", text: "summarize this folder" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: "summarize this folder" },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("batch_set_day_page_cwd_plain_prompt", cwdPromptBatch.success === true, {
    batch: cwdPromptBatch,
  });

  await driver.simulateKey("enter", ["cmd"]);
  await Bun.sleep(750);
  const afterSubmitState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "cmd_enter_submits_day_page_prompt_to_agent_chat",
    afterSubmitState.promptType === "agentChatChat",
    {
      promptType: afterSubmitState.promptType,
      inputValue: afterSubmitState.inputValue,
    },
  );

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
