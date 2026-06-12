#!/usr/bin/env bun
/**
 * Runtime proof for the Today → main-menu `@context` round trip (auto-swap):
 * - Typing into an `@file:` subsearch on the Day Page automatically swaps to
 *   the REAL main menu with the segment text as the launcher filter — the
 *   exact same @context selection UX the main menu uses (no Day Page popup).
 * - Escape while the search is pending cancels back to Today unchanged.
 * - Shrinking the input (deletion) does NOT re-trigger the swap.
 * - Accepting a file row in the main menu returns to Today with the resolved
 *   `@file:token` spliced into the originating line.
 */
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ?? "target-agent/artifacts/today/script-kit-gpui";

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
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

async function simulateMainHotkeyGesture(driver: Driver, phase: "down" | "up", label: string) {
  return driver.request(
    { type: "simulateMainHotkeyGesture", phase, requestId: `${runId}-${label}` },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  );
}

async function tapHotkey(driver: Driver, label: string) {
  await simulateMainHotkeyGesture(driver, "down", `${label}-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${label}-up`);
  await Bun.sleep(400);
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-round-trip",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

async function mainElements(limit = 240): Promise<Json[]> {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit },
    { timeoutMs: 5000 },
  )) as Json;
  return walkElements(elements);
}

async function selectedSemanticId(): Promise<string | null> {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 240 },
    { timeoutMs: 5000 },
  )) as Json;
  return (elements.selectedSemanticId as string | undefined) ?? null;
}

async function editorText(): Promise<string | null> {
  const flat = await mainElements();
  const editor = flat.find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  return (editor?.value as string | undefined) ?? null;
}

async function stateNow(): Promise<Json> {
  return (await driver.getState({ timeoutMs: 5000 })) as Json;
}

try {
  // --- Enter Day Page ---
  await simulateMainHotkeyGesture(driver, "down", "show-down");
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", "show-up");
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 8000 });
  await Bun.sleep(400);
  let state = await stateNow();
  if (state.promptType !== "dayPage") {
    await tapHotkey(driver, "toggle-day-page");
    state = await stateNow();
  }
  check("opened_day_page", state.promptType === "dayPage", { promptType: state.promptType });

  // --- Trigger: typing into an @file subsearch auto-swaps to the main menu ---
  await driver.batch([{ type: "setInput", text: "research @file:readme" }], { timeoutMs: 5000 });
  await Bun.sleep(700);
  const swapState = await stateNow();
  check(
    "typing_subsearch_auto_swaps_to_main_menu",
    swapState.promptType === "none" && swapState.inputValue === "@file:readme",
    { promptType: swapState.promptType, inputValue: swapState.inputValue },
  );

  // --- Cancel path: Escape returns to Today unchanged ---
  await driver.simulateKey("escape");
  await Bun.sleep(600);
  const cancelState = await stateNow();
  const cancelEditor = await editorText();
  check(
    "escape_cancels_back_to_today_unchanged",
    cancelState.promptType === "dayPage" && cancelEditor === "research @file:readme",
    { promptType: cancelState.promptType, cancelEditor },
  );

  // --- Deletion guard: shrinking inside the mention must NOT re-swap ---
  await driver.batch([{ type: "setInput", text: "research @file:readm" }], { timeoutMs: 5000 });
  await Bun.sleep(600);
  const shrinkState = await stateNow();
  const shrinkEditor = await editorText();
  check(
    "shrink_does_not_retrigger_swap",
    shrinkState.promptType === "dayPage" && shrinkEditor === "research @file:readm",
    { promptType: shrinkState.promptType, shrinkEditor },
  );

  // --- Accept path: grow again to re-trigger, then accept a file row ---
  await driver.batch([{ type: "setInput", text: "research @file:readme" }], { timeoutMs: 5000 });
  await Bun.sleep(700);
  const swapState2 = await stateNow();
  check(
    "second_swap_to_main_menu",
    swapState2.promptType === "none" && swapState2.inputValue === "@file:readme",
    { promptType: swapState2.promptType, inputValue: swapState2.inputValue },
  );

  // Wait for real file subsearch results (not just the "open full file
  // search" fallback row), then select and accept a file row.
  const isFallbackRow = (id: string) => id.includes("open-full-file-searc");
  let fileRowId: string | null = null;
  for (let attempt = 0; attempt < 20 && !fileRowId; attempt++) {
    const flat = await mainElements();
    const candidate = flat.find(
      (el) =>
        typeof el.semanticId === "string" &&
        (el.semanticId as string).startsWith("choice:") &&
        !isFallbackRow(el.semanticId as string),
    );
    if (candidate) fileRowId = candidate.semanticId as string;
    else await Bun.sleep(400);
  }
  check("main_menu_shows_file_results", Boolean(fileRowId), { fileRowId });
  for (let presses = 0; presses < 16; presses++) {
    const selected = await selectedSemanticId();
    if (selected && !isFallbackRow(selected)) break;
    await driver.simulateKey("down");
    await Bun.sleep(120);
  }
  const acceptedRow = await selectedSemanticId();
  check("file_row_selected", Boolean(acceptedRow) && !isFallbackRow(acceptedRow ?? ""), {
    acceptedRow,
  });
  await driver.simulateKey("enter");
  await Bun.sleep(800);

  const backState = await stateNow();
  const backEditor = await editorText();
  const tokenSpliced =
    typeof backEditor === "string" &&
    backEditor.startsWith("research @file:") &&
    backEditor !== "research @file:readme";
  check(
    "accept_returns_to_today_with_token",
    backState.promptType === "dayPage" && tokenSpliced,
    { promptType: backState.promptType, backEditor },
  );

  // --- Splice must not immediately re-open the search ---
  await Bun.sleep(700);
  const settleState = await stateNow();
  check("splice_does_not_retrigger_swap", settleState.promptType === "dayPage", {
    promptType: settleState.promptType,
  });

  const pass = failures.length === 0;
  console.log(
    JSON.stringify({ pass, failures, sessionDir: driver.sessionDir, receipts }, null, 2),
  );
  if (!pass) process.exitCode = 1;
} finally {
  await driver.close();
}
