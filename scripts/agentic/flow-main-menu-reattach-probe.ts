#!/usr/bin/env bun
// Main-menu flow reattach contract (bug repro, 2026-07-10).
//
// User report: with an active flow conversation in the background, hitting
// Enter on the flow's row in the main menu should reattach to that ACTIVE
// conversation — instead it opens a blank new one.
//
// Two rows can represent the flow on the main menu:
//   A. the "Active Flows" session row (prepend_root_flow_sessions_section)
//      — carries session_id, Enter must reattach.
//   B. the flow identity row in the "Flows" section — today Enter always
//      calls start_flow_session (a fresh blank Threadline).
//
// Red proof: after backgrounding session S1, Enter on the selected main-menu
// flow row must NOT grow flowUx.sessions — a second session id is the blank
// conversation the user saw.
//
// Run: bun scripts/agentic/flow-main-menu-reattach-probe.ts
import { resolve } from "node:path";
import { Driver } from "../devtools/driver";

const repoRoot = resolve(import.meta.dir, "../..");
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

type Sess = { sessionId: number; flowId: string; live: boolean; turns: number };

const receipts: any[] = [];
const failures: string[] = [];

function sessions(state: any): Sess[] {
  return (state?.flowUx?.sessions ?? []) as Sess[];
}

function record(step: string, state: any) {
  const entry = {
    step,
    promptType: state?.promptType,
    inputValue: state?.inputValue,
    selectedValue: state?.selectedValue,
    selectedName: state?.selectedName,
    selectedIndex: state?.selectedIndex,
    windowVisible: state?.windowVisible,
    sessions: sessions(state).map((s) => ({
      sessionId: s.sessionId,
      flowId: s.flowId,
      live: s.live,
      turns: s.turns,
    })),
  };
  receipts.push(entry);
  return entry;
}

function expect(condition: boolean, message: string) {
  if (!condition) failures.push(message);
}

const binary = process.env.SCRIPT_KIT_GPUI_BINARY;
const d = await Driver.launch({
  sandboxHome: true,
  sessionName: "flow-main-menu-reattach",
  ...(binary ? { binary } : {}),
  env: { SCRIPT_KIT_FLOW_UX_CWD: repoRoot },
});

async function pressKey(key: string) {
  await d.simulateGpuiEvent({ type: "keyDown", key });
  await sleep(150);
}

async function showFresh() {
  await d.request({ type: "show" });
  await d.waitForSettle();
}

async function openFlowSessionFromMainMenu(query: string) {
  await d.setFilterAndWait(query);
  let seen = false;
  for (let i = 0; i < 40 && !seen; i++) {
    const st = await d.getState();
    seen =
      typeof st?.selectedValue === "string" &&
      st.selectedValue.toLowerCase().includes(query.toLowerCase());
    if (!seen) await sleep(250);
  }
  expect(seen, `flow row for '${query}' never became the launcher selection`);
  await pressKey("enter");
  await sleep(300);
}

try {
  // ── Setup: start one flow conversation, then background it ────────────
  await showFresh();
  await openFlowSessionFromMainMenu("scout");
  const opened = record("session-open", await d.getState());
  expect(
    opened.promptType === "flowSession",
    `Enter on the flow row must open a flow session (got ${opened.promptType})`,
  );
  const s1 = sessions(await d.getState());
  expect(s1.length === 1, `expected exactly 1 session after open (got ${s1.length})`);
  const s1id = s1[0]?.sessionId;

  await pressKey("escape");
  const backToMenu = record("escape-to-menu", await d.getState());
  expect(
    backToMenu.promptType === "none" && backToMenu.windowVisible === true,
    `Escape must land on the main menu (got ${backToMenu.promptType})`,
  );
  expect(
    (backToMenu.sessions?.length ?? 0) === 1 && backToMenu.sessions[0]?.live,
    "the backgrounded session must stay live",
  );

  // ── Scenario A: empty-query selected row (Active Flows pin) ───────────
  await d.waitForSettle();
  const menuState = record("menu-empty-query", await d.getState());
  await pressKey("enter");
  await sleep(300);
  const afterEnterA = record("after-enter-selected-row", await d.getState());
  expect(
    afterEnterA.promptType === "flowSession",
    `Enter on the selected row must open a flow session (got ${afterEnterA.promptType})`,
  );
  const sessA = sessions(await d.getState());
  expect(
    sessA.length === 1 && sessA[0]?.sessionId === s1id,
    `Scenario A REPRODUCED: Enter minted a new session instead of reattaching ` +
      `(sessions now ${JSON.stringify(sessA.map((s) => s.sessionId))}, expected [${s1id}])`,
  );

  // ── Scenario B: typed-query flow identity row ─────────────────────────
  await pressKey("escape");
  await d.waitForSettle();
  await d.setFilterAndWait("scout");
  let seen = false;
  for (let i = 0; i < 40 && !seen; i++) {
    const st = await d.getState();
    seen =
      typeof st?.selectedValue === "string" &&
      st.selectedValue.toLowerCase().includes("scout");
    if (!seen) await sleep(250);
  }
  expect(seen, "typed query 'scout' never selected the flow row");
  record("menu-typed-query", await d.getState());
  await pressKey("enter");
  await sleep(300);
  const afterEnterB = record("after-enter-typed-row", await d.getState());
  expect(
    afterEnterB.promptType === "flowSession",
    `Enter on the typed flow row must open a flow session (got ${afterEnterB.promptType})`,
  );
  const sessB = sessions(await d.getState());
  const liveIds = sessB.filter((s) => s.live).map((s) => s.sessionId);
  expect(
    liveIds.length === 1 && liveIds[0] === s1id,
    `Scenario B REPRODUCED: typed-query Enter minted a new session instead of ` +
      `reattaching (live sessions ${JSON.stringify(liveIds)}, expected [${s1id}])`,
  );
} finally {
  await d.close();
}

const verdict = failures.length === 0 ? "green" : "red";
console.log(JSON.stringify({ verdict, failures, receipts }, null, 2));
if (failures.length > 0) process.exit(1);
