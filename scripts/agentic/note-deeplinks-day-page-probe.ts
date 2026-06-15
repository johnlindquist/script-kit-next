#!/usr/bin/env bun
/**
 * Runtime proof for Day Page deeplink activation:
 * - Cmd+. on a run deeplink opens a confirm popup
 * - Escape closes the confirm and returns to Day Page
 * - Cmd+. on a spine deeplink opens a non-silent context modal
 */
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/note-deeplinks-iter3/script-kit-gpui");

type Obj = Record<string, any>;

const runId = `note-deeplinks-day-${Date.now()}`;
const receipt: Obj = {
  tool: "note-deeplinks-day-page-probe",
  binary: BINARY,
  pass: false,
  failures: [] as string[],
};

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Obj) : {};
}

function check(name: string, ok: boolean, detail: Obj = {}) {
  receipt[name] = { ok, ...detail };
  if (!ok) receipt.failures.push(name);
}

async function pollUntil(
  label: string,
  fn: () => Promise<boolean>,
  timeoutMs = 6000,
): Promise<boolean> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (await fn()) return true;
    await Bun.sleep(100);
  }
  receipt[`timeout_${label}`] = true;
  return false;
}

async function setDayPageInput(driver: Driver, text: string) {
  return driver.batch([{ type: "setInput", text }], { timeoutMs: 8000 });
}

async function confirmWindows(driver: Driver): Promise<Obj[]> {
  const res = asObj(await driver.listAutomationWindows({ timeoutMs: 8000 }));
  const windows = (res.windows as Json[] | undefined) ?? [];
  return windows
    .map(asObj)
    .filter((window) => String(window.semanticSurface) === "confirmDialog");
}

function stateSummary(state: Obj): Obj {
  return {
    promptType: state.promptType,
    windowVisible: state.windowVisible,
    inputValue: state.inputValue,
  };
}

async function getStateSummary(driver: Driver): Promise<Obj> {
  return stateSummary(asObj(await driver.getState({ timeoutMs: 8000 })));
}

async function waitForPromptType(driver: Driver, promptType: string, label: string) {
  const opened = await pollUntil(label, async () => {
    const state = await getStateSummary(driver);
    return state.promptType === promptType;
  });
  return {
    opened,
    state: await getStateSummary(driver),
    confirmWindows: await confirmWindows(driver),
  };
}

async function closeConfirmWithEscape(driver: Driver, label: string) {
  await driver.simulateKey("escape");
  const closed = await pollUntil(`${label}-confirm-closed`, async () => {
    const state = await getStateSummary(driver);
    return state.promptType === "dayPage";
  });
  return {
    closed,
    confirmWindows: await confirmWindows(driver),
    state: await getStateSummary(driver),
  };
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "note-deeplinks-day-page",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  const opened = asObj(await openDayPage(driver, runId));
  check("day_page_opened", opened.promptType === "dayPage", {
    promptType: opened.promptType,
  });

  const runLink = "scriptkit://run/nonexistent-day-page-proof-script";
  const runSeed = asObj(await setDayPageInput(driver, runLink));
  check("run_link_seeded", runSeed.success === true, { batch: runSeed });
  await driver.simulateKey(".", ["cmd"]);
  const runConfirm = await waitForPromptType(driver, "confirmPrompt", "run-confirm-open");
  check("cmd_dot_on_run_link_opens_confirm", runConfirm.opened, runConfirm);
  const runClose = await closeConfirmWithEscape(driver, "run");
  check(
    "escape_closes_run_confirm_and_returns_day_page",
    runClose.closed && runClose.state.promptType === "dayPage",
    runClose,
  );

  const spineLink = "scriptkit://spine/notes/day-page-proof";
  const spineSeed = asObj(await setDayPageInput(driver, spineLink));
  check("spine_link_seeded", spineSeed.success === true, { batch: spineSeed });
  await driver.simulateKey(".", ["cmd"]);
  const spineModal = await waitForPromptType(driver, "confirmPrompt", "spine-modal-open");
  check("cmd_dot_on_spine_link_opens_context_modal", spineModal.opened, spineModal);
  const spineClose = await closeConfirmWithEscape(driver, "spine");
  check(
    "escape_closes_spine_modal_and_returns_day_page",
    spineClose.closed && spineClose.state.promptType === "dayPage",
    spineClose,
  );

  receipt.sessionDir = driver.sessionDir;
  receipt.pass = receipt.failures.length === 0;
  console.log(JSON.stringify(receipt, null, 2));
  if (!receipt.pass) process.exitCode = 1;
} finally {
  await driver.close();
}
