#!/usr/bin/env bun
// Escape-ladder contract for flow sessions (regression lock, 2026-07-10).
//
// A flow chat launched from the MAIN MENU must escape back the way the user
// came: Escape #1 → main menu (empty input, window visible), Escape #2 →
// window hidden. The session stays alive in the background. Detouring
// through the Conversation Desk (a surface the user never visited) reads as
// a swallowed Escape and is a ladder violation — the original report needed
// THREE escapes to hide the window.
//
// A flow chat entered from the DESK must escape back to the desk (one step),
// and the baseline "empty main menu + one Escape = hidden" must hold.
//
// Decision point under test: `flow_session_returns_to_desk` +
// `background_flow_session` in src/render_builtins/flow_ux.rs (unit-locked
// by the `flow_session_escape_origin` tests; this probe is the runtime
// proof through real GPUI key dispatch).
//
// Run: bun scripts/agentic/flow-session-escape-ladder-probe.ts
import { resolve } from "node:path";
import { Driver } from "../devtools/driver";

const repoRoot = resolve(import.meta.dir, "../..");
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

type Step = {
  step: string;
  promptType?: string;
  inputValue?: string;
  windowVisible?: boolean;
  activeSessions?: number;
};

const receipts: Step[] = [];
const failures: string[] = [];

function record(step: string, state: any): Step {
  const entry: Step = {
    step,
    promptType: state?.promptType,
    inputValue: state?.inputValue,
    windowVisible: state?.windowVisible,
    activeSessions:
      state?.flowUx?.activeSessions ?? state?.flowUx?.sessions?.length,
  };
  receipts.push(entry);
  return entry;
}

function expect(condition: boolean, message: string) {
  if (!condition) failures.push(message);
}

const d = await Driver.launch({
  sandboxHome: true,
  sessionName: "flow-escape-ladder",
  // Point flow discovery at this repo so flows/*.md are the corpus; no
  // turn is ever submitted, so no engine/credentials are needed.
  env: { SCRIPT_KIT_FLOW_UX_CWD: repoRoot },
});

async function pressEscape() {
  await d.simulateGpuiEvent({ type: "keyDown", key: "escape" });
  await sleep(150);
}

async function showFresh() {
  await d.request({ type: "show" });
  await d.waitForSettle();
}

async function openFlowSessionFromMainMenu(query: string) {
  await d.setFilterAndWait(query);
  // The roster lands async; wait until the flow row is the selected result
  // (flow rows carry their flow id, e.g. `project:scout`, as the selected
  // value once the roster has hoisted them above the fallbacks).
  let seen = false;
  for (let i = 0; i < 40 && !seen; i++) {
    const st = await d.getState();
    seen =
      typeof st?.selectedValue === "string" &&
      st.selectedValue.toLowerCase().includes(query.toLowerCase());
    if (!seen) await sleep(250);
  }
  expect(seen, `flow row for '${query}' never became the launcher selection`);
  await d.simulateGpuiEvent({ type: "keyDown", key: "enter" });
  await sleep(300);
}

/** Select the launcher row whose selectedValue matches, via real arrows. */
async function selectRowByValue(match: (value: string) => boolean) {
  for (let i = 0; i < 20; i++) {
    const st = await d.getState();
    if (typeof st?.selectedValue === "string" && match(st.selectedValue)) {
      return true;
    }
    await d.simulateGpuiEvent({ type: "keyDown", key: "down" });
    await sleep(80);
  }
  return false;
}

try {
  // ── Scenario 1: baseline — empty main menu hides on ONE Escape ──────
  await showFresh();
  await pressEscape();
  const baseline = record("baseline-escape", await d.getState());
  expect(
    baseline.windowVisible === false,
    "baseline: one Escape on the empty main menu must hide the window",
  );

  // ── Scenario 2: main-menu-launched flow chat ─────────────────────────
  await showFresh();
  await openFlowSessionFromMainMenu("scout");
  const session = record("main-menu-session-open", await d.getState());
  expect(
    session.promptType === "flowSession",
    `Enter on the flow row must open a flow session (got ${session.promptType})`,
  );

  await pressEscape();
  const backToMenu = record("main-menu-escape-1", await d.getState());
  expect(
    backToMenu.promptType === "none" && backToMenu.windowVisible === true,
    `Escape from a main-menu-launched flow session must land on the main menu, ` +
      `never the desk (got promptType=${backToMenu.promptType}, visible=${backToMenu.windowVisible})`,
  );
  expect(
    backToMenu.inputValue === "",
    `main menu input must be empty after escaping the session (got '${backToMenu.inputValue}')`,
  );
  expect(
    (backToMenu.activeSessions ?? 0) >= 1,
    "backgrounding must keep the session alive",
  );

  await pressEscape();
  const hidden = record("main-menu-escape-2", await d.getState());
  expect(
    hidden.windowVisible === false,
    "second Escape (empty main menu) must hide the window — an extra Escape here is the ladder violation",
  );

  // ── Scenario 3: desk-entered session escapes back to the desk ────────
  // The desk builtin is not in the triggerBuiltin registry; open it through
  // the real launcher path (the "Flows" builtin row).
  await showFresh();
  await d.setFilterAndWait("flows");
  const deskRowFound = await selectRowByValue((value) => value === "Flows");
  expect(deskRowFound, "the 'Flows' desk builtin row must be selectable");
  await d.simulateGpuiEvent({ type: "keyDown", key: "enter" });
  await sleep(300);
  const desk = record("desk-open", await d.getState());
  expect(
    desk.promptType === "flowUx",
    `the Flows builtin must open the desk (got ${desk.promptType})`,
  );

  // Reattach the scenario-2 session from the desk's Active rows (first row)
  // through the real Enter path.
  await d.simulateGpuiEvent({ type: "keyDown", key: "enter" });
  await sleep(300);
  const deskSession = record("desk-session-open", await d.getState());
  expect(
    deskSession.promptType === "flowSession",
    `Enter on the desk's first row must reattach the session (got ${deskSession.promptType})`,
  );

  await pressEscape();
  const backToDesk = record("desk-escape-1", await d.getState());
  expect(
    backToDesk.promptType === "flowUx" && backToDesk.windowVisible === true,
    `Escape from a desk-entered session must return to the desk (got ${backToDesk.promptType})`,
  );
} finally {
  await d.close();
}

const verdict = failures.length === 0 ? "green" : "red";
console.log(JSON.stringify({ verdict, failures, receipts }, null, 2));
if (failures.length > 0) process.exit(1);
