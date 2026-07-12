#!/usr/bin/env bun
// Scenario C: with a live backgrounded session, Enter on the flow IDENTITY
// row (the "Flows" section row, not the pinned "Active Flows" session row)
// must resume the live conversation, not mint a blank new one.
//
// Run: bun scripts/agentic/flow-identity-row-reattach-probe.ts
import { resolve } from "node:path";
import { Driver } from "../devtools/driver";

const repoRoot = resolve(import.meta.dir, "../..");
const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

const receipts: any[] = [];
const failures: string[] = [];

function sessions(state: any): any[] {
  return state?.flowUx?.sessions ?? [];
}

function record(step: string, state: any) {
  const entry = {
    step,
    promptType: state?.promptType,
    inputValue: state?.inputValue,
    selectedValue: state?.selectedValue,
    selectedIndex: state?.selectedIndex,
    windowVisible: state?.windowVisible,
    sessions: sessions(state).map((s: any) => ({
      sessionId: s.sessionId,
      flowId: s.flowId,
      live: s.live,
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
  sessionName: "flow-identity-reattach",
  ...(binary ? { binary } : {}),
  env: { SCRIPT_KIT_FLOW_UX_CWD: repoRoot },
});

async function pressKey(key: string) {
  await d.simulateGpuiEvent({ type: "keyDown", key });
  await sleep(120);
}

try {
  await d.request({ type: "show" });
  await d.waitForSettle();

  // Open a session on the scout flow via typed query.
  await d.setFilterAndWait("scout");
  let seen = false;
  for (let i = 0; i < 40 && !seen; i++) {
    const st = await d.getState();
    seen =
      typeof st?.selectedValue === "string" &&
      st.selectedValue.toLowerCase().includes("scout");
    if (!seen) await sleep(250);
  }
  expect(seen, "flow row for 'scout' never became the launcher selection");
  await pressKey("enter");
  await sleep(300);
  const opened = record("session-open", await d.getState());
  expect(opened.promptType === "flowSession", `expected flowSession, got ${opened.promptType}`);
  const s1id = sessions(await d.getState())[0]?.sessionId;

  await pressKey("escape");
  await d.waitForSettle();
  record("escape-to-menu", await d.getState());

  // Typed query: the pinned Active Flows session row ("Scout") is selected;
  // the flow IDENTITY row for the same flow sits below it in the results.
  // Arrow DOWN until we find that second "Scout" row.
  await d.setFilterAndWait("scout");
  await d.waitForSettle();
  const firstIdx = (await d.getState())?.selectedIndex;
  let identityFound = false;
  for (let i = 0; i < 12; i++) {
    await pressKey("down");
    const st = await d.getState();
    if (
      typeof st?.selectedValue === "string" &&
      st.selectedValue === "Scout" &&
      st.selectedIndex !== firstIdx
    ) {
      identityFound = true;
      record("identity-row-selected", st);
      break;
    }
  }
  expect(identityFound, "never found a second 'Scout' row (flow identity row) below the pin");

  await pressKey("enter");
  await sleep(300);
  const after = record("after-enter-identity-row", await d.getState());
  expect(
    after.promptType === "flowSession",
    `Enter on the identity row must open a flow session (got ${after.promptType})`,
  );
  const sess = sessions(await d.getState());
  const liveIds = sess.filter((s: any) => s.live).map((s: any) => s.sessionId);
  expect(
    liveIds.length === 1 && liveIds[0] === s1id,
    `REPRODUCED: identity-row Enter minted a blank session instead of resuming ` +
      `(live sessions ${JSON.stringify(liveIds)}, expected [${s1id}])`,
  );
} finally {
  await d.close();
}

const verdict = failures.length === 0 ? "green" : "red";
console.log(JSON.stringify({ verdict, failures, receipts }, null, 2));
if (failures.length > 0) process.exit(1);
