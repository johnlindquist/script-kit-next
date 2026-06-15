#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const binary =
  process.env.PROBE_BINARY ?? "target-agent/artifacts/day-cmdp-focus/script-kit-gpui";

const receipt: Record<string, Json> = {};
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json = {}) {
  receipt[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function walk(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const child of node) walk(child, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") out.push(json);
  for (const value of Object.values(json)) walk(value, out);
  return out;
}

async function actionsState(driver: Driver): Promise<Json | null> {
  const state = (await driver.request(
    { type: "getState", target: { type: "kind", kind: "actionsDialog" } },
    { expect: "stateResult", timeoutMs: 3000 },
  ).catch(() => null)) as Json | null;
  return (state?.actionsDialog ?? null) as Json | null;
}

async function actionRows(driver: Driver): Promise<Json[]> {
  const elements = (await driver.getElements(
    { target: { type: "kind", kind: "actionsDialog" }, limit: 240 },
    { timeoutMs: 3000 },
  ).catch(() => ({ elements: [] }))) as Json;
  return walk(elements);
}

function selectedActionRow(rows: Json[]): Json | null {
  return rows.find((row) => row.selected === true && String(row.semanticId ?? row.id ?? "").startsWith("choice:")) ?? null;
}

async function mainFocus(driver: Driver): Promise<string | null> {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 80 },
    { timeoutMs: 3000 },
  )) as Json;
  return (elements.focusedSemanticId as string | undefined) ?? null;
}

const driver = await Driver.launch({
  binary,
  sandboxHome: true,
  sessionName: "day-cmdp-focus",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  const daysDir = join(driver.sessionDir, "home", ".scriptkit", "brain", "days");
  mkdirSync(daysDir, { recursive: true });
  writeFileSync(join(daysDir, "2026-06-01.md"), "monday seeded day\n");
  writeFileSync(join(daysDir, "2026-06-02.md"), "tuesday seeded day\n");

  await driver.batch([{ type: "setInput", text: "," }], { timeoutMs: 5000 });
  await Bun.sleep(250);
  driver.simulateKey("enter");
  await driver.waitForState({ windowVisible: true, promptType: "dayPage" }, { timeoutMs: 8000 });
  const opened = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("opened_day_page", opened.promptType === "dayPage", {
    promptType: opened.promptType,
    windowVisible: opened.windowVisible,
  });

  const seedText = "day cmdp focus seed";
  const setDay = (await driver.batch(
    [
      { type: "setInput", text: seedText },
      {
        type: "waitFor",
        condition: { type: "stateMatch", state: { promptType: "dayPage", inputValue: seedText } },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("day_editor_accepts_input", setDay.success === true, { setDay });

  driver.simulateKey("k", ["cmd"]);
  await Bun.sleep(700);
  const dayActions = await actionsState(driver);
  check("day_actions_open", dayActions !== null, { dayActionsOpen: dayActions !== null });
  driver.simulateKey("escape");
  await driver.waitForState({ promptType: "dayPage" }, { timeoutMs: 5000 });
  await Bun.sleep(250);
  const focusAfterActionsEscape = await mainFocus(driver);
  check("day_actions_escape_restores_editor_focus", focusAfterActionsEscape === "input:day-page-editor", {
    focusAfterActionsEscape,
  });

  driver.simulateKey("p", ["cmd"]);
  await Bun.sleep(700);
  const switcherBefore = await actionsState(driver);
  const rowsBefore = await actionRows(driver);
  const rowIdsBefore = rowsBefore.map((row) => `${row.semanticId ?? row.id ?? ""}|${row.text ?? ""}`);
  check("cmd_p_opens_notes_style_switcher", switcherBefore !== null && rowIdsBefore.some((id) => id.includes("2026-06")), {
    rowIdsBefore: rowIdsBefore.slice(0, 30),
  });

  driver.simulateKey("m");
  await Bun.sleep(250);
  const switcherAfterType = await actionsState(driver);
  const rowsAfterType = await actionRows(driver);
  const rowIdsAfterType = rowsAfterType.map((row) => `${row.semanticId ?? row.id ?? ""}|${row.text ?? ""}`);
  check("cmd_p_typing_filters_switcher", rowIdsAfterType.some((id) => id.toLowerCase().includes("monday")), {
    selectedActionId: switcherAfterType?.selectedActionId ?? null,
    rowIdsAfterType: rowIdsAfterType.slice(0, 30),
  });

  const selectedBeforeDown = switcherAfterType?.selectedActionId ?? null;
  const selectedRowBeforeDown = selectedActionRow(rowsAfterType);
  driver.simulateKey("down");
  await Bun.sleep(250);
  const switcherAfterDown = await actionsState(driver);
  const rowsAfterDown = await actionRows(driver);
  const selectedRowAfterDown = selectedActionRow(rowsAfterDown);
  const selectedBeforeUp = selectedRowAfterDown?.semanticId ?? selectedRowAfterDown?.id ?? null;
  driver.simulateKey("up");
  await Bun.sleep(250);
  const rowsAfterUp = await actionRows(driver);
  const selectedRowAfterUp = selectedActionRow(rowsAfterUp);
  check(
    "cmd_p_arrows_move_switcher_selection",
    switcherAfterDown !== null &&
      selectedRowBeforeDown !== null &&
      selectedRowAfterDown !== null &&
      selectedRowAfterUp !== null &&
      (selectedRowAfterDown.semanticId ?? selectedRowAfterDown.id) !==
        (selectedRowBeforeDown.semanticId ?? selectedRowBeforeDown.id) &&
      (selectedRowAfterUp.semanticId ?? selectedRowAfterUp.id) ===
        (selectedRowBeforeDown.semanticId ?? selectedRowBeforeDown.id),
    {
    selectedBeforeDown,
      selectedRowBeforeDown: selectedRowBeforeDown?.semanticId ?? selectedRowBeforeDown?.id ?? null,
      selectedAfterDown: switcherAfterDown?.selectedActionId ?? null,
      selectedRowAfterDown: selectedRowAfterDown?.semanticId ?? selectedRowAfterDown?.id ?? null,
      selectedBeforeUp,
      selectedRowAfterUp: selectedRowAfterUp?.semanticId ?? selectedRowAfterUp?.id ?? null,
    },
  );

  driver.simulateKey("escape");
  await driver.waitForState({ promptType: "dayPage" }, { timeoutMs: 5000 });
  await Bun.sleep(250);
  const focusAfterSwitcherEscape = await mainFocus(driver);
  const closedSwitcher = await actionsState(driver);
  check("cmd_p_escape_closes_and_restores_editor_focus", closedSwitcher === null && focusAfterSwitcherEscape === "input:day-page-editor", {
    closedSwitcher: closedSwitcher === null,
    focusAfterSwitcherEscape,
  });
} finally {
  await driver.close();
}

const output = {
  pass: failures.length === 0,
  failures,
  sessionDir: driver.sessionDir,
  receipt,
};

console.log(JSON.stringify(output, null, 2));
if (failures.length) process.exit(1);
