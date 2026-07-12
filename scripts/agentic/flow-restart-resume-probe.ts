#!/usr/bin/env bun
// Cross-restart flow conversation resume (bug fix proof, 2026-07-10).
//
// User report: an app restart (dev.sh rebuild) wiped the in-memory flow
// sessions, so Enter on the flow's main-menu row landed in a BLANK composer
// instead of the user's active conversation. Fix: every committed turn
// persists a snapshot under $SK_PATH/flows/conversations/<flow-id>.json and
// resume_or_start_flow_session restores it when no live session exists.
//
// This probe seeds a persisted snapshot (playing the previous app run),
// launches a fresh app, hits Enter on the flow row through the real user
// path, and asserts the session opens with the restored turns instead of a
// blank Threadline.
//
// Run: bun scripts/agentic/flow-restart-resume-probe.ts
import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
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
    selectedValue: state?.selectedValue,
    windowVisible: state?.windowVisible,
    sessions: sessions(state).map((s: any) => ({
      sessionId: s.sessionId,
      flowId: s.flowId,
      live: s.live,
      turns: s.turns,
      state: s.state,
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
  sessionName: "flow-restart-resume",
  ...(binary ? { binary } : {}),
  env: { SCRIPT_KIT_FLOW_UX_CWD: repoRoot },
});

try {
  // Seed the snapshot a previous app run would have written for the
  // repo-roster flow `project:scout` (file name is the sanitized flow id).
  const store = join(d.sessionDir, "home", ".scriptkit", "flows", "conversations");
  mkdirSync(store, { recursive: true });
  writeFileSync(
    join(store, "project-scout.json"),
    JSON.stringify({
      flow_id: "project:scout",
      saved_at: "2026-07-10T20:00:00Z",
      turns: [
        { user: "where does escape routing live?", assistant: "src/app_impl — the escape ladder." },
        { user: "and the flow desk?", assistant: "src/render_builtins/flow_ux.rs owns it." },
      ],
    }),
  );

  await d.request({ type: "show" });
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
  expect(seen, "flow row for 'scout' never became the launcher selection");
  record("row-selected", await d.getState());

  await d.simulateGpuiEvent({ type: "keyDown", key: "enter" });
  await sleep(400);
  const after = record("after-enter", await d.getState());
  expect(
    after.promptType === "flowSession",
    `Enter on the flow row must open a flow session (got ${after.promptType})`,
  );
  const sess = sessions(await d.getState());
  expect(sess.length === 1, `expected exactly 1 session (got ${sess.length})`);
  expect(
    sess[0]?.turns === 2,
    `REPRODUCED (blank conversation): restored session must carry the 2 persisted ` +
      `turns (got turns=${sess[0]?.turns})`,
  );
  expect(sess[0]?.live === true, "restored session must be live");
} finally {
  await d.close();
}

const verdict = failures.length === 0 ? "green" : "red";
console.log(JSON.stringify({ verdict, failures, receipts }, null, 2));
if (failures.length > 0) process.exit(1);
