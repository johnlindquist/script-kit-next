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
    dayPage: state.dayPage,
  };
}

async function getState(driver: Driver): Promise<Obj> {
  return asObj(await driver.getState({ timeoutMs: 8000 }));
}

async function getStateSummary(driver: Driver): Promise<Obj> {
  return stateSummary(await getState(driver));
}

async function dayPageKitResourcePreview(driver: Driver): Promise<Obj> {
  return asObj(asObj((await getState(driver)).dayPage).kitResourcePreview);
}

async function dayPageElementsText(driver: Driver): Promise<string> {
  const elements = await driver.getElements(
    { target: { type: "kind", kind: "main" } },
    { timeoutMs: 8000 },
  );
  return JSON.stringify(elements);
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

async function clickDayPageEditorFirstLine(driver: Driver) {
  const results = await driver.simulateGpuiClick(72, 104, {
    target: { type: "kind", kind: "main" },
    timeoutMs: 8000,
  });
  await Bun.sleep(150);
  return results.map(asObj);
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

  const kitPreviewSeed = asObj(await setDayPageInput(driver, "[scripts](kit://scripts)"));
  check("kit_scripts_link_seeded", kitPreviewSeed.success === true, { batch: kitPreviewSeed });
  await driver.simulateKey(".", ["cmd"]);
  const kitPreviewOpened = await pollUntil("kit-scripts-preview-open", async () => {
    const preview = await dayPageKitResourcePreview(driver);
    return (
      preview.active === true &&
      preview.uri === "kit://scripts" &&
      preview.readOnly === true
    );
  });
  const kitPreviewState = await dayPageKitResourcePreview(driver);
  check("kit_scripts_cmd_dot_opens_read_only_preview", kitPreviewOpened, {
    preview: kitPreviewState,
    state: await getStateSummary(driver),
    confirmWindows: await confirmWindows(driver),
  });
  await driver.simulateKey("escape");
  const kitPreviewClosed = await pollUntil("kit-scripts-preview-closed", async () => {
    const preview = await dayPageKitResourcePreview(driver);
    return preview.active === false;
  });
  check("escape_closes_kit_scripts_preview_and_returns_day_page", kitPreviewClosed, {
    state: await getStateSummary(driver),
    preview: await dayPageKitResourcePreview(driver),
  });

  const unsupportedKitSeed = asObj(await setDayPageInput(driver, "[context](kit://context)"));
  check("unsupported_kit_context_seeded", unsupportedKitSeed.success === true, {
    batch: unsupportedKitSeed,
  });
  await driver.simulateKey(".", ["cmd"]);
  const unsupportedModal = await waitForPromptType(
    driver,
    "confirmPrompt",
    "unsupported-kit-context-modal-open",
  );
  check("unsupported_kit_context_opens_helpful_modal", unsupportedModal.opened, {
    ...unsupportedModal,
    elements: await dayPageElementsText(driver),
  });
  const unsupportedClose = await closeConfirmWithEscape(driver, "unsupported-kit-context");
  check(
    "escape_closes_unsupported_kit_context_modal_and_returns_day_page",
    unsupportedClose.closed && unsupportedClose.state.promptType === "dayPage",
    unsupportedClose,
  );

  const mousePlainSeed = asObj(await setDayPageInput(driver, "mouse plain text"));
  check("mouse_plain_text_seeded", mousePlainSeed.success === true, { batch: mousePlainSeed });
  const mousePlainClick = await clickDayPageEditorFirstLine(driver);
  check("mouse_plain_text_click_dispatches", mousePlainClick.every((result) => result.success), {
    click: mousePlainClick,
  });
  await Bun.sleep(200);
  const mousePlainState = await getStateSummary(driver);
  check("mouse_plain_text_click_stays_on_day_page", mousePlainState.promptType === "dayPage", {
    state: mousePlainState,
    confirmWindows: await confirmWindows(driver),
  });

  const mouseRunLink = "scriptkit://run/nonexistent-day-page-mouse-proof-script";
  const mouseRunSeed = asObj(await setDayPageInput(driver, mouseRunLink));
  check("mouse_run_link_seeded", mouseRunSeed.success === true, { batch: mouseRunSeed });
  const mouseRunClick = await clickDayPageEditorFirstLine(driver);
  check("mouse_run_link_click_dispatches", mouseRunClick.every((result) => result.success), {
    click: mouseRunClick,
  });
  const mouseRunConfirm = await waitForPromptType(
    driver,
    "confirmPrompt",
    "mouse-run-confirm-open",
  );
  check("mouse_run_link_opens_confirm", mouseRunConfirm.opened, mouseRunConfirm);
  const mouseRunClose = await closeConfirmWithEscape(driver, "mouse-run");
  check(
    "escape_closes_mouse_run_confirm_and_returns_day_page",
    mouseRunClose.closed && mouseRunClose.state.promptType === "dayPage",
    mouseRunClose,
  );

  const mouseUnknownSpineLink = "scriptkit://spine/not-a-source/value";
  const mouseUnknownSeed = asObj(await setDayPageInput(driver, mouseUnknownSpineLink));
  check("mouse_unknown_spine_link_seeded", mouseUnknownSeed.success === true, {
    batch: mouseUnknownSeed,
  });
  const mouseUnknownClick = await clickDayPageEditorFirstLine(driver);
  check("mouse_unknown_spine_click_dispatches", mouseUnknownClick.every((result) => result.success), {
    click: mouseUnknownClick,
  });
  const mouseUnknownModal = await waitForPromptType(
    driver,
    "confirmPrompt",
    "mouse-unknown-spine-modal-open",
  );
  check("mouse_unknown_spine_opens_context_modal", mouseUnknownModal.opened, mouseUnknownModal);
  const mouseUnknownClose = await closeConfirmWithEscape(driver, "mouse-unknown-spine");
  check(
    "escape_closes_mouse_unknown_spine_modal_and_returns_day_page",
    mouseUnknownClose.closed && mouseUnknownClose.state.promptType === "dayPage",
    mouseUnknownClose,
  );

  const mouseKitSeed = asObj(await setDayPageInput(driver, "[scripts](kit://scripts)"));
  check("mouse_kit_scripts_link_seeded", mouseKitSeed.success === true, { batch: mouseKitSeed });
  const mouseKitClick = await clickDayPageEditorFirstLine(driver);
  check("mouse_kit_scripts_click_dispatches", mouseKitClick.every((result) => result.success), {
    click: mouseKitClick,
  });
  const mouseKitPreviewOpened = await pollUntil("mouse-kit-scripts-preview-open", async () => {
    const preview = await dayPageKitResourcePreview(driver);
    return (
      preview.active === true &&
      preview.uri === "kit://scripts" &&
      preview.readOnly === true
    );
  });
  check("mouse_kit_scripts_opens_read_only_preview", mouseKitPreviewOpened, {
    preview: await dayPageKitResourcePreview(driver),
    state: await getStateSummary(driver),
  });
  await driver.simulateKey("escape");
  const mouseKitPreviewClosed = await pollUntil("mouse-kit-scripts-preview-closed", async () => {
    const preview = await dayPageKitResourcePreview(driver);
    return preview.active === false;
  });
  check("escape_closes_mouse_kit_scripts_preview", mouseKitPreviewClosed, {
    state: await getStateSummary(driver),
    preview: await dayPageKitResourcePreview(driver),
  });

  receipt.sessionDir = driver.sessionDir;
  receipt.pass = receipt.failures.length === 0;
  console.log(JSON.stringify(receipt, null, 2));
  if (!receipt.pass) process.exitCode = 1;
} finally {
  await driver.close();
}
