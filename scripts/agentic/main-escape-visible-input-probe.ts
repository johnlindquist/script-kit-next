#!/usr/bin/env bun
/**
 * Runtime proof for ScriptList Escape behavior:
 * - visible input text clears on the first Escape
 * - empty visible input closes on the next Escape
 * - the real "quit" builtin confirmation path does not swallow an extra Escape
 *   after popup cancel + filter clear
 * - filterInputDiagnostics exposes canonical/computed/raw visual state
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/main-escape-visible-input/script-kit-gpui \
 *     bun scripts/agentic/main-escape-visible-input-probe.ts
 */
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/main-escape-visible-input/script-kit-gpui";
const OUT_PATH = process.env.PROBE_OUT ?? "";
const READY_TIMEOUT_MS = Number(process.env.PROBE_READY_TIMEOUT_MS ?? 30_000);

const receipts: Record<string, Json> = {};
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function diagnostics(state: Json): Json {
  return (state.filterInputDiagnostics ?? {}) as Json;
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function confirmWindows(driver: Driver): Promise<Json[]> {
  const receipt = await driver.listAutomationWindows({ timeoutMs: 5_000 });
  const windows = Array.isArray(receipt.windows) ? receipt.windows as Json[] : [];
  return windows.filter((window) =>
    String(window.id ?? "") === "confirm-popup"
    || String(window.semanticSurface ?? "") === "confirmDialog"
  );
}

async function waitForConfirmOpen(driver: Driver, label: string) {
  const deadline = Date.now() + 8_000;
  let last: Json[] = [];
  while (Date.now() < deadline) {
    last = await confirmWindows(driver);
    if (last.length > 0) return { ok: true, windows: last };
    await sleep(50);
  }
  return { ok: false, windows: last, label };
}

async function waitForConfirmClosed(driver: Driver, label: string) {
  const deadline = Date.now() + 8_000;
  let last: Json[] = [];
  while (Date.now() < deadline) {
    last = await confirmWindows(driver);
    if (last.length === 0) return { ok: true, windows: last };
    await sleep(50);
  }
  return { ok: false, windows: last, label };
}

async function stateDigest(driver: Driver): Promise<Json> {
  const state = await driver.getState({ timeoutMs: 5_000 });
  const contract = (state.surfaceContract ?? {}) as Json;
  return {
    windowVisible: state.windowVisible,
    promptType: state.promptType,
    inputValue: state.inputValue,
    surface: contract.surface ?? contract.kind ?? contract.name,
    diagnostics: diagnostics(state),
  };
}

async function relevantLogs(driver: Driver): Promise<Json> {
  const logs = await driver.getLogs({ limit: 500 }, { timeoutMs: 5_000 });
  const entries = Array.isArray(logs.logs) ? logs.logs as Json[] : [];
  return entries.filter((entry) => {
    const message = String(entry.message ?? "");
    const target = String(entry.target ?? "");
    return target.includes("confirm")
      || message.includes("SimulateKey: Escape")
      || message.includes("ESC -")
      || message.includes("Capture Escape")
      || message.includes("clear_filter")
      || message.includes("confirm_cancel")
      || message.includes("Resetting to script list");
  }) as Json;
}

async function proveQuitConfirmEscapePath(driver: Driver) {
  await driver.request({ type: "show" }, { timeoutMs: 8_000 });
  await driver.waitForState(
    { windowVisible: true, promptType: "none" },
    { timeoutMs: 8_000 },
  );

  await driver.setFilterAndWait("quit", { timeoutMs: 8_000 });
  const beforeEnter = await stateDigest(driver);
  driver.simulateKey("enter", []);
  const confirmOpen = await waitForConfirmOpen(driver, "quit-confirm-open");

  const trace: Json[] = [{ step: "beforeEnter", state: beforeEnter }, { step: "afterEnter", confirmOpen }];
  if (!confirmOpen.ok) {
    return {
      ok: false,
      classification: "blocked-confirm-did-not-open",
      trace,
      logs: await relevantLogs(driver),
    };
  }

  driver.simulateKey("escape", []);
  const confirmClosed = await waitForConfirmClosed(driver, "escape-1-confirm-cancel");
  await sleep(250);
  const afterEscape1 = await stateDigest(driver);
  trace.push({
    step: "escape1",
    expected: "confirm popup closes",
    confirmClosed,
    state: afterEscape1,
  });

  driver.simulateKey("escape", []);
  await driver.waitForState(
    { windowVisible: true, promptType: "none", inputValue: "" },
    { timeoutMs: 8_000, pollIntervalMs: 10 },
  );
  const afterEscape2 = await stateDigest(driver);
  trace.push({
    step: "escape2",
    expected: "visible filter clears",
    state: afterEscape2,
  });

  driver.simulateKey("escape", []);
  let hiddenAfterThird = false;
  const thirdDeadline = Date.now() + 2_000;
  while (Date.now() < thirdDeadline) {
    const state = await stateDigest(driver);
    if (state.windowVisible === false) {
      hiddenAfterThird = true;
      break;
    }
    await sleep(50);
  }
  const afterEscape3 = await stateDigest(driver);
  trace.push({
    step: "escape3",
    expected: "main window hides",
    state: afterEscape3,
  });

  let afterEscape4: Json | null = null;
  if (!hiddenAfterThird) {
    driver.simulateKey("escape", []);
    await driver.waitForState(
      { windowVisible: false },
      { timeoutMs: 8_000, pollIntervalMs: 10 },
    );
    afterEscape4 = await stateDigest(driver);
    trace.push({
      step: "escape4",
      expected: "red-path extra Escape hides",
      state: afterEscape4,
    });
  }

  return {
    ok: hiddenAfterThird,
    classification: hiddenAfterThird ? "green" : "red-extra-escape-swallowed",
    trace,
    escapesAfterConfirmCancelToHide: hiddenAfterThird ? 3 : 4,
    afterEscape4,
    logs: await relevantLogs(driver),
  };
}

let driver: Driver | null = null;

try {
  driver = await Driver.launch({
    binary: BINARY,
    sandboxHome: true,
    sessionName: "main-escape-visible-input-probe",
    readyTimeoutMs: READY_TIMEOUT_MS,
    env: {
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_STARTUP_PROFILE: "dev-fast",
      SCRIPT_KIT_DISABLE_AGENT_CHAT_HOT_PREWARM: "1",
      SCRIPT_KIT_DISABLE_AUTOMATIC_UPDATE_CHECK: "1",
    },
  });

  driver.send({ type: "show", requestId: "main-escape-show" });
  await driver.waitForState(
    { windowVisible: true, promptType: "none" },
    { timeoutMs: 8000 },
  );

  const input = `escape-probe-${Date.now()}`;
  await driver.setFilterAndWait(input, { timeoutMs: 8000 });
  const before = await driver.getState({ timeoutMs: 5000 });
  const beforeDiag = diagnostics(before);
  check("diagnostics_match_visible_input", before.inputValue === input
    && beforeDiag.canonicalFilterText === input
    && beforeDiag.computedFilterText === input
    && beforeDiag.rawVisualInputValue === input
    && beforeDiag.pendingFilterSync === false, {
    inputValue: before.inputValue,
    diagnostics: beforeDiag,
  });

  driver.simulateKey("escape");
  await driver.waitForState(
    { windowVisible: true, promptType: "none", inputValue: "" },
    { timeoutMs: 8000, pollIntervalMs: 10 },
  );
  const afterFirst = await driver.getState({ timeoutMs: 5000 });
  const afterFirstDiag = diagnostics(afterFirst);
  check("first_escape_clears_visible_input", afterFirst.inputValue === ""
    && afterFirst.windowVisible === true
    && afterFirstDiag.canonicalFilterText === ""
    && afterFirstDiag.computedFilterText === ""
    && afterFirstDiag.rawVisualInputValue === "", {
    inputValue: afterFirst.inputValue,
    windowVisible: afterFirst.windowVisible,
    diagnostics: afterFirstDiag,
  });

  driver.simulateKey("escape");
  await driver.waitForState(
    { windowVisible: false },
    { timeoutMs: 8000, pollIntervalMs: 10 },
  );
  const afterSecond = await driver.getState({ timeoutMs: 5000 });
  check("second_escape_closes_empty_main", afterSecond.windowVisible === false, {
    windowVisible: afterSecond.windowVisible,
    inputValue: afterSecond.inputValue,
    diagnostics: diagnostics(afterSecond),
  });

  receipts.quitConfirmEscapePath = await proveQuitConfirmEscapePath(driver);
  check(
    "quit_confirm_escape3_hides_main",
    (receipts.quitConfirmEscapePath as Json).classification === "green",
    receipts.quitConfirmEscapePath,
  );

  const output = {
    ok: failures.length === 0,
    failures,
    binary: BINARY,
    sessionDir: driver.sessionDir,
    appLog: driver.logPath,
    receipts,
  };
  if (OUT_PATH) {
    const outDir = dirname(OUT_PATH);
    if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true });
    writeFileSync(OUT_PATH, `${JSON.stringify(output, null, 2)}\n`);
  }
  console.log(JSON.stringify(output, null, 2));
  await driver.close();
  process.exit(failures.length === 0 ? 0 : 1);
} catch (error) {
  console.error(error);
  if (driver) await driver.close().catch(() => {});
  process.exit(1);
}
