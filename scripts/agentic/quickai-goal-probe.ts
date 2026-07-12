#!/usr/bin/env bun
/**
 * Runtime proof for the 2026-07-10 Quick AI goal set:
 *
 *  1. Tab on an EMPTY composer opens the `>` cwd picker (main-menu parity:
 *     Tab = cwd chip-as-button) — receipt: inputText ">", spine kind
 *     "projectCwd", log `agent_chat_tab_empty_composer_opened_cwd_picker`.
 *  2. `-` opens the flow search; Tab-accepting the highlighted flow leaves a
 *     compact `-name` token and stages the flow markdown as context —
 *     receipt: spine kind "flow", selectable rows, token in inputText,
 *     contextChipCount bump, log `agent_chat_flow_search_staged_flow`.
 *  3. `@` context picker renders a SINGLE-line section separator (the
 *     title+subtitle double separator is dead) — receipt: screenshot for
 *     visual review.
 *  4. Plain Up/Down on an empty composer cycles the persisted prompt
 *     history (newest first, Down past newest restores empty).
 *
 * Run from the repo root (flow roster needs the repo flows/):
 *   SCRIPT_KIT_FLOW_UX_CWD=$PWD bun scripts/agentic/quickai-goal-probe.ts
 */
import { Driver } from "../devtools/driver.ts";

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/quickai-goal/script-kit-gpui";

const receipt: Record<string, unknown> = { probe: "quickai-goal", binary };
const failures: string[] = [];

const driver = await Driver.launch({
  sessionName: "quickai-goal-probe",
  binary,
  sandboxHome: true,
  seedAgentAuth: true,
});

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

async function chatState(): Promise<Record<string, unknown>> {
  const result = (await driver.request(
    { type: "getAgentChatState" },
    { timeoutMs: 10000 },
  )) as Record<string, unknown>;
  return (result.state ?? result) as Record<string, unknown>;
}

async function waitForChat(
  predicate: (s: Record<string, unknown>) => boolean,
  label: string,
  timeoutMs = 15000,
): Promise<Record<string, unknown>> {
  const deadline = Date.now() + timeoutMs;
  let last: Record<string, unknown> = {};
  while (Date.now() < deadline) {
    try {
      last = await chatState();
      if (predicate(last)) return last;
    } catch {
      // view not active yet
    }
    await sleep(250);
  }
  failures.push(`timeout: ${label}`);
  return last;
}

function setInput(text: string, submit = false) {
  return driver.request(
    { type: "setAgentChatInput", text, submit },
    { timeoutMs: 10000 },
  );
}

function pressKey(key: string) {
  return driver.simulateGpuiEvent(
    { type: "keyDown", key },
    { target: { type: "kind", kind: "main" } },
  );
}

async function logsContain(needle: string): Promise<boolean> {
  const logs = await driver.request(
    { type: "getLogs", limit: 500 },
    { timeoutMs: 10000 },
  );
  return JSON.stringify(logs).includes(needle);
}

function spine(s: Record<string, unknown>): Record<string, unknown> {
  return (s.spine ?? {}) as Record<string, unknown>;
}

try {
  await driver.waitForSettle();
  driver.send({ type: "show" });
  await driver.waitForState({ windowVisible: true });

  // ── Guard: `-` in the MAIN MENU filter must stay a plain query (the
  // flow search is composer-only; main-menu flows are ordinary rows) ──
  await driver.setFilterAndWait("-");
  const mainElements = JSON.stringify(
    await driver.request({ type: "getElements" }, { timeoutMs: 10000 }),
  );
  receipt.mainMenuDash = {
    spineFlowSectionAbsent: !mainElements.includes("spine-section-flows"),
  };
  if (mainElements.includes("spine-section-flows")) {
    failures.push("main menu '-' swapped to spine Flows section");
  }
  await driver.setFilterAndWait("");

  // ── Enter Quick AI via the real launcher path: text + Tab ──
  await driver.setFilterAndWait("quickai goal probe entry");
  await pressKey("tab");
  const entered = await waitForChat(
    (s) => typeof s.inputText === "string",
    "quick-ai-entered",
  );
  receipt.entryUiVariant = entered.uiVariant;

  // ── Feature 4: Up/Down cycles submitted prompt history ──
  await setInput("first probe prompt", true);
  await sleep(400);
  await setInput("second probe prompt", true);
  await sleep(400);
  await setInput("", false);
  await sleep(200);

  await pressKey("up");
  let s = await waitForChat(
    (st) => st.inputText === "second probe prompt",
    "up-recalls-newest",
  );
  const up1 = s.inputText;
  await pressKey("up");
  s = await waitForChat(
    (st) => st.inputText === "first probe prompt",
    "up-recalls-older",
  );
  const up2 = s.inputText;
  await pressKey("down");
  s = await waitForChat(
    (st) => st.inputText === "second probe prompt",
    "down-steps-newer",
  );
  const down1 = s.inputText;
  await pressKey("down");
  s = await waitForChat((st) => st.inputText === "", "down-restores-empty");
  const down2 = s.inputText;
  receipt.promptHistory = { up1, up2, down1, down2 };

  // ── Feature 1: Tab on empty composer opens the cwd picker ──
  await pressKey("tab");
  s = await waitForChat(
    (st) => spine(st).activeSegmentKind === "projectCwd",
    "tab-empty-opens-cwd-picker",
  );
  receipt.tabEmpty = {
    inputText: s.inputText,
    segmentKind: spine(s).activeSegmentKind,
    ownsList: spine(s).ownsList,
    logEvent: await logsContain("agent_chat_tab_empty_composer_opened_cwd_picker"),
  };
  await pressKey("escape");
  await setInput("", false);
  await sleep(200);

  // ── Feature 2: `-` opens the flow search; Tab stages the flow ──
  await setInput("-", false);
  s = await waitForChat(
    (st) =>
      spine(st).activeSegmentKind === "flow" &&
      Number(spine(st).selectableRowCount ?? 0) > 0,
    "flow-search-has-rows",
    20000,
  );
  // Roster may still be warming on first paint: poke the segment once.
  if (Number(spine(s).selectableRowCount ?? 0) === 0) {
    await setInput("", false);
    await sleep(2000);
    await setInput("-", false);
    s = await waitForChat(
      (st) => Number(spine(st).selectableRowCount ?? 0) > 0,
      "flow-search-has-rows-after-warm",
      20000,
    );
  }
  const chipCountBefore = Number(s.contextChipCount ?? 0);
  receipt.flowSearch = {
    segmentKind: spine(s).activeSegmentKind,
    selectableRowCount: spine(s).selectableRowCount,
  };
  await pressKey("tab");
  s = await waitForChat(
    (st) =>
      typeof st.inputText === "string" &&
      (st.inputText as string).startsWith("-") &&
      (st.inputText as string).trim().length > 1,
    "flow-accept-inserts-token",
  );
  receipt.flowAccept = {
    inputText: s.inputText,
    contextChipCountBefore: chipCountBefore,
    contextChipCountAfter: s.contextChipCount,
    logEvent: await logsContain("agent_chat_flow_search_staged_flow"),
  };
  await setInput("", false);
  await sleep(200);

  // ── Feature 3: `@` picker single-line separator (screenshot) ──
  await setInput("@", false);
  await waitForChat(
    (st) => spine(st).activeSegmentKind === "contextMention",
    "context-picker-open",
  );
  await sleep(400);
  const shotPath = `${driver.sessionDir}/quickai-goal-at-picker.png`;
  const shot = await driver.captureScreenshot(shotPath).catch((e) => ({
    error: String(e),
  }));
  receipt.atPickerScreenshot = { path: shotPath, result: shot };

  receipt.failures = failures;
  receipt.ok = failures.length === 0;
} finally {
  try {
    await pressKey("escape");
    driver.send({ type: "hide" });
  } catch {}
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
process.exit(failures.length === 0 ? 0 : 1);
