#!/usr/bin/env bun
/**
 * Runtime proof for Day Page editor cursor parity with Notes.
 *
 * Proves production paths that automation setInput used to mask:
 * - saved Day Page document load/rebind places the caret at the editor end
 * - launcher carry-over into Day Page places the caret at the appended end
 * - repeated main-hotkey tap toggles still work through the same session
 */
import { Driver, type Json } from "../devtools/driver";
import { tapMainHotkey } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/today/script-kit-gpui";

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

async function tapHotkey(driver: Driver, label: string) {
  await tapMainHotkey(driver, runId, label);
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

async function dayPageEditorReceipt(driver: Driver, label: string) {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 80 },
    { timeoutMs: 5000 },
  )) as Json;
  const editor = walkElements(elements).find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  const selection = String(editor?.source ?? "").split(":").map(Number);
  const selectionStart = Number.isFinite(selection[0]) ? selection[0] : null;
  const selectionEnd = Number.isFinite(selection[1]) ? selection[1] : null;
  const value = typeof editor?.value === "string" ? editor.value : "";
  const valueLength =
    typeof editor?.sourceName === "string" && /^\d+$/.test(editor.sourceName)
      ? Number(editor.sourceName)
      : value.length;
  const receipt = {
    label,
    selectedSemanticId: elements.selectedSemanticId ?? null,
    focusedSemanticId: elements.focusedSemanticId ?? null,
    semanticId: editor?.semanticId ?? null,
    focused: editor?.focused ?? null,
    role: editor?.role ?? null,
    kind: editor?.kind ?? null,
    value,
    valueLength,
    selectionStart,
    selectionEnd,
  };
  return receipt;
}

function assertCursorAtEnd(label: string, receipt: Json, expectedValue: string) {
  check(`${label}_value`, receipt.value === expectedValue, {
    actual: receipt.value,
    expected: expectedValue,
  });
  check(
    `${label}_cursor_at_end`,
    receipt.selectionStart === expectedValue.length &&
      receipt.selectionEnd === expectedValue.length &&
      receipt.valueLength === expectedValue.length,
    {
      selectionStart: receipt.selectionStart,
      selectionEnd: receipt.selectionEnd,
      valueLength: receipt.valueLength,
      expectedLength: expectedValue.length,
    },
  );
  check(`${label}_editor_focused`, receipt.focused === true, {
    focused: receipt.focused,
    focusedSemanticId: receipt.focusedSemanticId,
  });
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-cursor-parity",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  await tapHotkey(driver, "show-launcher");
  await driver.waitForState({ windowVisible: true, promptType: "none" }, { timeoutMs: 8000 });

  await driver.setFilterAndWait("seed day page", { timeoutMs: 5000 });
  await tapHotkey(driver, "open-day-page");
  let state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("opened_day_page", state.promptType === "dayPage", { promptType: state.promptType });

  const savedContent = "alpha\n\n";
  const seedBatch = (await driver.batch(
    [
      { type: "setInput", text: savedContent },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: savedContent },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("seed_content_batch", seedBatch.success === true, { batch: seedBatch });

  await driver.simulateKey("s", ["cmd"]);
  await Bun.sleep(200);

  await tapHotkey(driver, "back-to-launcher-after-save");
  state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("back_to_launcher_after_save", state.promptType === "none", {
    promptType: state.promptType,
    inputValue: state.inputValue ?? null,
  });

  await tapHotkey(driver, "reload-day-page");
  state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("reloaded_day_page", state.promptType === "dayPage", { promptType: state.promptType });
  const reloadReceipt = await dayPageEditorReceipt(driver, "reload");
  assertCursorAtEnd("reload", reloadReceipt, savedContent);

  await tapHotkey(driver, "back-to-launcher-before-carry");
  await driver.setFilterAndWait("carry me to today", { timeoutMs: 5000 });
  await tapHotkey(driver, "carry-to-day-page");
  state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("carry_opened_day_page", state.promptType === "dayPage", {
    promptType: state.promptType,
  });
  const carriedValue = `${savedContent}carry me to today`;
  const carryReceipt = await dayPageEditorReceipt(driver, "carry");
  assertCursorAtEnd("carry", carryReceipt, carriedValue);

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
