#!/usr/bin/env bun

import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const repoRoot = resolve(import.meta.dir, "../..");
const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(repoRoot, "target-agent/artifacts/type-filter-editing/script-kit-gpui");
const outDir = join(repoRoot, ".test-output/type-filter-editing-proof");
const sessionDir = join("/tmp", `sk-type-filter-editing-${process.pid}`);
const homeDir = join(sessionDir, "home");
const kitDir = join(homeDir, ".scriptkit");

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function choiceRows(elements: Json): Json[] {
  return (elements.elements ?? []).filter((element: Json) => element.type === "choice");
}

function findRow(elements: Json, value: string): Json {
  const rows = choiceRows(elements);
  const row = rows.find((element) => element.value === value || element.text === value);
  assert(
    row,
    `missing row ${value}; rows=${JSON.stringify(
      rows.map((element) => ({
        semanticId: element.semanticId,
        text: element.text,
        value: element.value,
        kind: element.kind,
      })),
    )}`,
  );
  assert(typeof row.semanticId === "string", `row ${value} missing semanticId`);
  return row;
}

function seedScript() {
  mkdirSync(join(kitDir, "plugins", "main", "scripts"), { recursive: true });
  writeFileSync(join(kitDir, "config.ts"), "export default {};\n");
  writeFileSync(
    join(kitDir, "plugins", "main", "scripts", "type-filter-proof.ts"),
    [
      "// Name: Type Filter Proof Script",
      "// Description: Deterministic type:script proof row",
      "await arg('Type Filter Proof Script')",
      "",
    ].join("\n"),
  );
}

async function gpuiKey(driver: Driver, key: string, text?: string): Promise<Json> {
  const event: Json = { type: "keyDown", key, modifiers: [] };
  if (text !== undefined) event.text = text;
  return driver.simulateGpuiEvent(event, {
    target: { type: "kind", kind: "main" },
    timeoutMs: 5_000,
  });
}

async function waitForInput(driver: Driver, input: string, timeoutMs = 4_000): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let state = await driver.getState({ timeoutMs: 5_000 });
  while (state.inputValue !== input && Date.now() < deadline) {
    await Bun.sleep(50);
    state = await driver.getState({ timeoutMs: 5_000 });
  }
  return state;
}

function logSlice(logPath: string, offset: number): string {
  return readFileSync(logPath, "utf8").slice(offset);
}

async function main() {
  rmSync(outDir, { recursive: true, force: true });
  rmSync(sessionDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });
  mkdirSync(kitDir, { recursive: true });
  seedScript();

  const receipt: Json = {
    schemaVersion: 1,
    status: "running",
    binary,
    outDir,
    checks: {},
  };

  const driver = await Driver.launch({
    binary,
    sessionName: "type-filter-editing-proof",
    sessionDir,
    sandboxHome: false,
    env: {
      HOME: homeDir,
      SK_PATH: kitDir,
      SCRIPT_KIT_SHORTCUT_DEBUG: "1",
    },
    readyTimeoutMs: 15_000,
    defaultTimeoutMs: 8_000,
  });

  try {
    await driver.setFilterAndWait("type:", { timeoutMs: 8_000 });
    const pickerElements = await driver.getElements({ limit: 120 }, { timeoutMs: 8_000 });
    assert(
      (pickerElements.elements ?? []).some(
        (element: Json) => element.semanticId === "list:menu-syntax-trigger-picker",
      ),
      "type: should show the menu-syntax trigger picker",
    );
    const scriptsRow = findRow(pickerElements, "type:script");
    receipt.checks.picker = { scriptsRow };

    const pickerOpenOffset = readFileSync(driver.logPath, "utf8").length;
    const pickerSResult = await gpuiKey(driver, "s", "s");
    const pickerAfterSState = await waitForInput(driver, "type:s", 2_000);
    const pickerAfterSLog = logSlice(driver.logPath, pickerOpenOffset);
    assert(
      pickerAfterSState.inputValue === "type:s",
      `With the picker open, s should edit the input; got ${pickerAfterSState.inputValue}; logs=\n${pickerAfterSLog}`,
    );
    assert(
      !pickerAfterSLog.includes("Shortcut route blocked key=s: menu-syntax owns main list"),
      `With the picker open, s was still blocked by shortcut routing:\n${pickerAfterSLog}`,
    );

    const pickerBackspaceOffset = readFileSync(driver.logPath, "utf8").length;
    const pickerBackspaceResult = await gpuiKey(driver, "backspace");
    const pickerAfterBackspaceState = await waitForInput(driver, "type:", 2_000);
    const pickerAfterBackspaceLog = logSlice(driver.logPath, pickerBackspaceOffset);
    assert(
      pickerAfterBackspaceState.inputValue === "type:",
      `With the picker open, Backspace should edit the input; got ${pickerAfterBackspaceState.inputValue}; logs=\n${pickerAfterBackspaceLog}`,
    );
    assert(
      !pickerAfterBackspaceLog.includes("Shortcut route blocked key=backspace: menu-syntax owns main list"),
      `With the picker open, Backspace was still blocked by shortcut routing:\n${pickerAfterBackspaceLog}`,
    );

    const pickerEscapeResult = await gpuiKey(driver, "escape");
    const pickerAfterEscapeState = await waitForInput(driver, "", 2_000);
    assert(
      pickerAfterEscapeState.inputValue === "",
      `With the picker open, Escape should clear the filter; got ${pickerAfterEscapeState.inputValue}`,
    );
    await driver.setFilterAndWait("type:", { timeoutMs: 8_000 });

    const selectResult = await driver.batch(
      [{ type: "selectBySemanticId", semanticId: scriptsRow.semanticId, submit: true }],
      { timeoutMs: 8_000 },
    );
    assert(selectResult.success === true, `select Scripts row failed: ${JSON.stringify(selectResult)}`);
    const acceptedState = await waitForInput(driver, "type:script", 8_000);
    assert(acceptedState.inputValue === "type:script", `expected type:script, got ${acceptedState.inputValue}`);
    const acceptedElements = await driver.getElements({ limit: 120 }, { timeoutMs: 8_000 });
    assert(
      !(acceptedElements.elements ?? []).some(
        (element: Json) => element.semanticId === "list:menu-syntax-trigger-picker",
      ),
      "accepted type:script should close the trigger picker",
    );
    assert((acceptedState.visibleChoiceCount ?? 0) > 0, "type:script should show filtered script results");
    receipt.checks.accepted = {
      state: {
        inputValue: acceptedState.inputValue,
        visibleChoiceCount: acceptedState.visibleChoiceCount,
        promptType: acceptedState.promptType,
      },
      firstChoices: choiceRows(acceptedElements).slice(0, 5).map((row) => ({
        text: row.text,
        value: row.value,
        kind: row.kind,
      })),
      pickerOpenEditing: {
        pickerSResult,
        pickerAfterSState: { inputValue: pickerAfterSState.inputValue },
        pickerBackspaceResult,
        pickerAfterBackspaceState: { inputValue: pickerAfterBackspaceState.inputValue },
        pickerEscapeResult,
        pickerAfterEscapeState: { inputValue: pickerAfterEscapeState.inputValue },
      },
    };

    const beforeKeysOffset = readFileSync(driver.logPath, "utf8").length;

    const sResult = await gpuiKey(driver, "s", "s");
    const afterSState = await waitForInput(driver, "type:scripts", 2_000);
    const afterSSlice = logSlice(driver.logPath, beforeKeysOffset);
    assert(
      !afterSSlice.includes("Shortcut route blocked key=s: menu-syntax owns main list"),
      `s was still blocked by shortcut routing:\n${afterSSlice}`,
    );
    assert(
      afterSSlice.includes("Shortcut route passed through key=s")
        || afterSSlice.includes("Displayed shortcut s passed through")
        || afterSState.inputValue === "type:scripts",
      `missing pass-through evidence for s; input=${afterSState.inputValue}; logs=\n${afterSSlice}`,
    );
    assert(
      afterSState.inputValue === "type:scripts",
      `GPUI text key should append after type:script; got ${afterSState.inputValue}; logs=\n${afterSSlice}`,
    );

    const backspaceOffset = readFileSync(driver.logPath, "utf8").length;
    const backspaceResult = await gpuiKey(driver, "backspace");
    const afterBackspaceState = await waitForInput(driver, "type:script", 2_000);
    const afterBackspaceSlice = logSlice(driver.logPath, backspaceOffset);
    assert(
      !afterBackspaceSlice.includes("Shortcut route blocked key=backspace: menu-syntax owns main list"),
      `backspace was still blocked by shortcut routing:\n${afterBackspaceSlice}`,
    );
    assert(
      afterBackspaceState.inputValue === "type:script",
      `Backspace should edit the input after type:scripts; got ${afterBackspaceState.inputValue}; logs=\n${afterBackspaceSlice}`,
    );

    receipt.checks.editingKeys = {
      sResult,
      afterSState: { inputValue: afterSState.inputValue },
      afterSLog: afterSSlice
        .split(/\r?\n/)
        .filter((line) => line.includes("Shortcut route") || line.includes("Displayed shortcut")),
      backspaceResult,
      afterBackspaceState: { inputValue: afterBackspaceState.inputValue },
      afterBackspaceLog: afterBackspaceSlice
        .split(/\r?\n/)
        .filter((line) => line.includes("Shortcut route") || line.includes("Displayed shortcut")),
    };

    receipt.status = "pass";
  } finally {
    await driver.close();
    receipt.logPath = driver.logPath;
    writeFileSync(join(outDir, "receipt.json"), JSON.stringify(receipt, null, 2));
  }

  console.log(JSON.stringify({ status: receipt.status, receiptPath: ".test-output/type-filter-editing-proof/receipt.json" }, null, 2));
}

main().catch((error) => {
  mkdirSync(outDir, { recursive: true });
  writeFileSync(
    join(outDir, "receipt.json"),
    JSON.stringify({ schemaVersion: 1, status: "fail", error: String(error?.stack ?? error) }, null, 2),
  );
  console.error(error);
  process.exit(1);
});
