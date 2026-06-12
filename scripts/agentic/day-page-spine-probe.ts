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

  // ============================ `@` mentions =============================
  // The Day Page renders NO `@` selector of its own. Typing into any `@`
  // mention swaps to the REAL main menu (the launcher's own context UX) with
  // the segment text as the filter; accept returns to Today with the token,
  // Escape cancels back unchanged.

  // Typing "@fi" auto-swaps to the launcher. Clear first: the day-page entry
  // gesture carries the launcher filter into the editor, and the growth
  // detector compares whole-content lengths on protocol setInput.
  await driver.batch([{ type: "setInput", text: "" }], { timeoutMs: 5000 });
  await Bun.sleep(300);
  await driver.batch([{ type: "setInput", text: "@fi" }], { timeoutMs: 5000 });
  await Bun.sleep(700);
  const swapFiState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "typing_at_mention_swaps_to_main_menu",
    swapFiState.promptType === "none" && swapFiState.inputValue === "@fi",
    { promptType: swapFiState.promptType, inputValue: swapFiState.inputValue },
  );

  // The launcher (not the Day Page) shows the shared context rows.
  const launcherElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const launcherFlat = walkElements(launcherElements);
  check("launcher_shows_files_row", Boolean(
    launcherFlat.find(
      (el) =>
        typeof el.semanticId === "string" &&
        (el.semanticId === "spine:@:subsearch:file" ||
          ((el.semanticId as string).startsWith("choice:") &&
            (el.semanticId as string).includes("file"))),
    ),
  ), {
    selectedSemanticId: launcherElements.selectedSemanticId ?? null,
    ids: launcherFlat.slice(0, 12).map((el) => el.semanticId ?? el.id),
  });

  // Enter completes "@fi" → "@file:" colon mode INSIDE the launcher; the
  // round trip stays pending (no bounce back to Today yet).
  await driver.simulateKey("enter");
  await Bun.sleep(600);
  const afterCompleteState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "launcher_enter_completes_colon_mode_in_launcher",
    afterCompleteState.promptType === "none" && afterCompleteState.inputValue === "@file:",
    {
      promptType: afterCompleteState.promptType,
      inputValue: afterCompleteState.inputValue,
    },
  );

  // Escape cancels the round trip back to Today with the original segment.
  await driver.simulateKey("escape");
  await Bun.sleep(500);
  const afterCancelState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const afterCancelText = await editorText(driver);
  check(
    "escape_returns_to_day_page_with_fragment",
    afterCancelState.promptType === "dayPage" && afterCancelText === "@fi",
    { promptType: afterCancelState.promptType, afterCancelText },
  );

  // No Day Page-local spine panel may render for `@` (cursor is inside the
  // mention after the cancel restore).
  const afterCancelElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const afterCancelFlat = walkElements(afterCancelElements);
  check("no_inline_at_selector_on_day_page", !afterCancelFlat.find(
    (el) => el.semanticId === "list:day-page-spine-list",
  ), {
    ids: afterCancelFlat.slice(0, 12).map((el) => el.semanticId ?? el.id),
  });

  // Builtin acceptance: "@sel" swaps, Enter on the launcher's Selection row
  // resolves immediately and returns to Today with "@selection ".
  await driver.batch([{ type: "setInput", text: "" }], { timeoutMs: 5000 });
  await Bun.sleep(300);
  await driver.batch([{ type: "setInput", text: "@sel" }], { timeoutMs: 5000 });
  await Bun.sleep(700);
  const swapSelState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "builtin_mention_swaps_to_main_menu",
    swapSelState.promptType === "none" && swapSelState.inputValue === "@sel",
    { promptType: swapSelState.promptType, inputValue: swapSelState.inputValue },
  );
  const selElements = (await driver.getElements(
    { target: { type: "main" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  check(
    "launcher_selection_row_selected",
    typeof selElements.selectedSemanticId === "string" &&
      (selElements.selectedSemanticId as string).includes("selection"),
    { selectedSemanticId: selElements.selectedSemanticId ?? null },
  );
  await driver.simulateKey("enter");
  await Bun.sleep(600);
  const afterBuiltinState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const afterBuiltinText = await editorText(driver);
  check(
    "builtin_accept_returns_to_today_with_token",
    afterBuiltinState.promptType === "dayPage" && afterBuiltinText === "@selection ",
    { promptType: afterBuiltinState.promptType, afterBuiltinText },
  );

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

  // ("@sel" builtin completion is covered by the round-trip flow above —
  // the Day Page no longer completes `@` fragments inline.)

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

  // Reaching "@clip" without triggering the auto-swap: GROW with the cursor
  // landing in tail free text, then SHRINK back to the bare mention
  // (deletions never trigger the round trip).
  const unresolvedGrowBatch = (await driver.batch(
    [
      { type: "setInput", text: "@clip extra" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: "@clip extra" },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
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
  check(
    "batch_set_day_page_unresolved_context_after_cwd",
    unresolvedGrowBatch.success === true && unresolvedAfterCwdBatch.success === true,
    { grow: unresolvedGrowBatch, shrink: unresolvedAfterCwdBatch },
  );

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
