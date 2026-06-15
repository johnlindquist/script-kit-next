#!/usr/bin/env bun
/**
 * Runtime proof for Notes deeplink activation:
 * - Cmd+. with no link preserves the focus-mode fallback
 * - Cmd+. on a run deeplink opens a confirm popup instead of silently executing
 */
import { join, resolve } from "node:path";
import { mkdir, readFile } from "node:fs/promises";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/note-deeplinks-iter1/script-kit-gpui");

type Obj = Record<string, any>;

const receipt: Obj = {
  tool: "note-deeplinks-notes-probe",
  binary: BINARY,
  pass: false,
  failures: [] as string[],
};
const runExecReceiptPath = join(
  process.env.TMPDIR ?? "/tmp",
  `notes-run-exec-${Date.now()}.jsonl`,
);

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Obj)
    : {};
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

async function notesState(driver: Driver): Promise<Obj> {
  const res = await driver.request(
    { type: "getState", target: { type: "kind", kind: "notes" } },
    { expect: "stateResult", timeoutMs: 8000 },
  );
  return asObj(asObj(res).notes);
}

async function notesView(driver: Driver): Promise<Obj> {
  return asObj((await notesState(driver)).view);
}

async function notesKitResourcePreview(driver: Driver): Promise<Obj> {
  return asObj((await notesState(driver)).kitResourcePreview);
}

async function notesElementsText(driver: Driver): Promise<string> {
  const elements = await driver.getElements(
    { target: { type: "kind", kind: "notes" } },
    { timeoutMs: 8000 },
  );
  return JSON.stringify(elements);
}

async function openNotes(driver: Driver) {
  driver.send({ type: "openNotes" });
  return pollUntil("notes-open", async () => {
    const res = asObj(await driver.listAutomationWindows({ timeoutMs: 8000 }));
    const windows = (res.windows as Json[] | undefined) ?? [];
    return windows.map(asObj).some((window) => String(window.kind) === "notes");
  });
}

async function setNotesInput(driver: Driver, text: string) {
  return driver.request(
    {
      type: "batch",
      target: { type: "kind", kind: "notes" },
      commands: [{ type: "setInput", text }],
      options: { stopOnError: true, timeout: 8000 },
    },
    { expect: "batchResult", timeoutMs: 9000 },
  );
}

function legacyTargetedKey(driver: Driver, key: string, modifiers: string[] = []) {
  driver.send({
    type: "simulateKey",
    target: { type: "kind", kind: "notes" },
    key,
    modifiers,
  });
}

async function cmdDot(driver: Driver) {
  legacyTargetedKey(driver, ".", ["cmd"]);
  await Bun.sleep(100);
}

async function escape(driver: Driver) {
  legacyTargetedKey(driver, "escape");
  await Bun.sleep(100);
}

async function enter(driver: Driver) {
  legacyTargetedKey(driver, "enter");
  await Bun.sleep(100);
}

async function clickNotesEditorFirstLine(driver: Driver) {
  const results = await driver.simulateGpuiClick(32, 64, {
    target: { type: "kind", kind: "notes" },
    timeoutMs: 8000,
  });
  await Bun.sleep(150);
  return results.map(asObj);
}

async function readReceipt(path: string): Promise<string> {
  try {
    return await readFile(path, "utf8");
  } catch {
    return "";
  }
}

async function confirmWindows(driver: Driver): Promise<Obj[]> {
  const res = asObj(await driver.listAutomationWindows({ timeoutMs: 8000 }));
  const windows = (res.windows as Json[] | undefined) ?? [];
  return windows
    .map(asObj)
    .filter((window) => String(window.semanticSurface) === "confirmDialog");
}

async function closeConfirmWithEscape(driver: Driver, label: string) {
  await escape(driver).catch(() => legacyTargetedKey(driver, "escape"));
  const closed = await pollUntil(`${label}-confirm-closed`, async () => {
    const windows = await confirmWindows(driver);
    return windows.length === 0;
  });
  return {
    closed,
    confirmWindows: await confirmWindows(driver),
  };
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "note-deeplinks-notes",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_TEST_NOTES_RUN_EXEC_RECEIPT: runExecReceiptPath,
  },
});

try {
  const opened = await openNotes(driver);
  check("notes_opened", opened);

  const plain = await setNotesInput(driver, "plain text with no deeplink");
  check("plain_text_seeded", plain.success === true, { batch: plain });
  await cmdDot(driver);
  const focusModeEnabled = await pollUntil("focus-mode-enabled", async () => {
    const view = await notesView(driver);
    return view.focusMode === true;
  });
  check("cmd_dot_without_link_toggles_focus_mode", focusModeEnabled, {
    view: await notesView(driver),
  });

  // Leave focus mode so the run-link leg starts from the normal editor state.
  await cmdDot(driver);
  await pollUntil("focus-mode-disabled", async () => {
    const view = await notesView(driver);
    return view.focusMode === false;
  });

  const runLink = "scriptkit://run/nonexistent-proof-script";
  const runSeed = await setNotesInput(driver, runLink);
  check("run_link_seeded", runSeed.success === true, { batch: runSeed });
  await cmdDot(driver);

  const confirmOpened = await pollUntil("run-confirm-open", async () => {
    const windows = await confirmWindows(driver);
    return windows.length > 0;
  });
  const confirmSnapshot = await confirmWindows(driver);
  check("cmd_dot_on_run_link_opens_confirm", confirmOpened, {
    confirmWindows: confirmSnapshot,
  });
  check("run_link_did_not_toggle_focus_mode", (await notesView(driver)).focusMode !== true, {
    view: await notesView(driver),
  });

  const runClose = await closeConfirmWithEscape(driver, "run");
  check("escape_closes_run_confirm", runClose.closed, {
    confirmWindows: runClose.confirmWindows,
  });
  const afterCancelReceipt = await readReceipt(runExecReceiptPath);
  check("run_link_escape_did_not_execute", afterCancelReceipt.trim() === "", {
    receipt: afterCancelReceipt,
  });

  const safeRunLink = "scriptkit://commands/builtin/refresh-scripts";
  const safeRunSeed = await setNotesInput(driver, safeRunLink);
  check("safe_run_link_seeded", safeRunSeed.success === true, { batch: safeRunSeed });
  await cmdDot(driver);
  const safeConfirmOpened = await pollUntil("safe-run-confirm-open", async () => {
    const windows = await confirmWindows(driver);
    return windows.length > 0;
  });
  check("cmd_dot_on_safe_run_link_opens_confirm", safeConfirmOpened, {
    confirmWindows: await confirmWindows(driver),
  });
  const beforeConfirmReceipt = await readReceipt(runExecReceiptPath);
  check("safe_run_link_not_executed_before_confirm", beforeConfirmReceipt.trim() === "", {
    receipt: beforeConfirmReceipt,
  });
  await enter(driver);
  const safeRunExecuted = await pollUntil("safe-run-executed-after-confirm", async () => {
    const receiptText = await readReceipt(runExecReceiptPath);
    return receiptText.includes('"commandId":"builtin/refresh-scripts"');
  });
  const afterConfirmReceipt = await readReceipt(runExecReceiptPath);
  check("safe_run_link_executes_after_confirm", safeRunExecuted, {
    receipt: afterConfirmReceipt,
  });
  const safeConfirmClosed = await pollUntil("safe-run-confirm-closed", async () => {
    const windows = await confirmWindows(driver);
    return windows.length === 0;
  });
  check("safe_run_confirm_closes_cleanly", safeConfirmClosed, {
    confirmWindows: await confirmWindows(driver),
  });

  const missingParent = join(driver.sessionDir, "existing-parent");
  await mkdir(missingParent, { recursive: true });
  const missingFile = join(missingParent, "missing-deeplink-target.txt");
  const missingSeed = await setNotesInput(driver, missingFile);
  check("missing_file_link_seeded", missingSeed.success === true, { batch: missingSeed });
  await cmdDot(driver);
  const missingModalOpened = await pollUntil("missing-file-modal-open", async () => {
    const windows = await confirmWindows(driver);
    return windows.some((window) => String(window.title) === "Can't open this link");
  });
  const missingModalSnapshot = await confirmWindows(driver);
  check("missing_file_opens_helpful_modal", missingModalOpened, {
    confirmWindows: missingModalSnapshot,
  });
  const missingClose = await closeConfirmWithEscape(driver, "missing-file");
  check("escape_closes_missing_file_modal", missingClose.closed, {
    confirmWindows: missingClose.confirmWindows,
    view: await notesView(driver),
  });

  const kitPreviewLink = "[scripts](kit://scripts)";
  const kitPreviewSeed = await setNotesInput(driver, kitPreviewLink);
  check("kit_scripts_link_seeded", kitPreviewSeed.success === true, { batch: kitPreviewSeed });
  await cmdDot(driver);
  const kitPreviewOpened = await pollUntil("kit-scripts-preview-open", async () => {
    const preview = await notesKitResourcePreview(driver);
    return (
      preview.active === true &&
      preview.uri === "kit://scripts" &&
      preview.readOnly === true
    );
  });
  const kitPreviewState = await notesKitResourcePreview(driver);
  check("kit_scripts_cmd_dot_opens_read_only_preview", kitPreviewOpened, {
    preview: kitPreviewState,
    confirmWindows: await confirmWindows(driver),
  });
  await escape(driver);
  const kitPreviewClosed = await pollUntil("kit-scripts-preview-closed", async () => {
    const preview = await notesKitResourcePreview(driver);
    return preview.active === false;
  });
  check("escape_closes_kit_scripts_preview", kitPreviewClosed, {
    view: await notesView(driver),
    preview: await notesKitResourcePreview(driver),
  });

  const unsupportedKitSeed = await setNotesInput(driver, "[context](kit://context)");
  check("unsupported_kit_context_seeded", unsupportedKitSeed.success === true, {
    batch: unsupportedKitSeed,
  });
  await cmdDot(driver);
  const unsupportedModalOpened = await pollUntil("unsupported-kit-context-modal-open", async () => {
    const windows = await confirmWindows(driver);
    const text = await notesElementsText(driver);
    return (
      windows.some((window) => String(window.title) === "Can't open this link") &&
      text.includes("kit://context")
    );
  });
  check("unsupported_kit_context_opens_helpful_modal", unsupportedModalOpened, {
    confirmWindows: await confirmWindows(driver),
    elements: await notesElementsText(driver),
  });
  const unsupportedClose = await closeConfirmWithEscape(driver, "unsupported-kit-context");
  check("escape_closes_unsupported_kit_context_modal", unsupportedClose.closed, {
    confirmWindows: unsupportedClose.confirmWindows,
    view: await notesView(driver),
  });

  const mousePlain = await setNotesInput(driver, "mouse plain text");
  check("mouse_plain_text_seeded", mousePlain.success === true, { batch: mousePlain });
  const mousePlainClick = await clickNotesEditorFirstLine(driver);
  check("mouse_plain_text_click_dispatches", mousePlainClick.every((result) => result.success), {
    click: mousePlainClick,
  });
  await Bun.sleep(200);
  check("mouse_plain_text_click_does_not_open_modal", (await confirmWindows(driver)).length === 0, {
    confirmWindows: await confirmWindows(driver),
    view: await notesView(driver),
  });

  const mouseRunLink = "scriptkit://run/nonexistent-mouse-proof-script";
  const mouseRunSeed = await setNotesInput(driver, mouseRunLink);
  check("mouse_run_link_seeded", mouseRunSeed.success === true, { batch: mouseRunSeed });
  const beforeMouseRunReceipt = await readReceipt(runExecReceiptPath);
  const mouseRunClick = await clickNotesEditorFirstLine(driver);
  check("mouse_run_link_click_dispatches", mouseRunClick.every((result) => result.success), {
    click: mouseRunClick,
  });
  const mouseRunConfirmOpened = await pollUntil("mouse-run-confirm-open", async () => {
    const windows = await confirmWindows(driver);
    return windows.some((window) => String(window.title) === "Run Script Kit command?");
  });
  check("mouse_run_link_opens_confirm", mouseRunConfirmOpened, {
    confirmWindows: await confirmWindows(driver),
  });
  const afterMouseRunReceipt = await readReceipt(runExecReceiptPath);
  check("mouse_run_link_not_executed_before_confirm", afterMouseRunReceipt === beforeMouseRunReceipt, {
    before: beforeMouseRunReceipt,
    after: afterMouseRunReceipt,
  });
  const mouseRunClose = await closeConfirmWithEscape(driver, "mouse-run");
  check("escape_closes_mouse_run_confirm", mouseRunClose.closed, {
    confirmWindows: mouseRunClose.confirmWindows,
    view: await notesView(driver),
  });

  const mouseMissingParent = join(driver.sessionDir, "mouse-existing-parent");
  await mkdir(mouseMissingParent, { recursive: true });
  const mouseMissingFile = join(mouseMissingParent, "missing-mouse-deeplink-target.txt");
  const mouseMissingSeed = await setNotesInput(driver, mouseMissingFile);
  check("mouse_missing_file_link_seeded", mouseMissingSeed.success === true, {
    batch: mouseMissingSeed,
  });
  const mouseMissingClick = await clickNotesEditorFirstLine(driver);
  check("mouse_missing_file_click_dispatches", mouseMissingClick.every((result) => result.success), {
    click: mouseMissingClick,
  });
  const mouseMissingModalOpened = await pollUntil("mouse-missing-file-modal-open", async () => {
    const windows = await confirmWindows(driver);
    return windows.some((window) => String(window.title) === "Can't open this link");
  });
  check("mouse_missing_file_opens_helpful_modal", mouseMissingModalOpened, {
    confirmWindows: await confirmWindows(driver),
  });
  const mouseMissingClose = await closeConfirmWithEscape(driver, "mouse-missing-file");
  check("escape_closes_mouse_missing_file_modal", mouseMissingClose.closed, {
    confirmWindows: mouseMissingClose.confirmWindows,
    view: await notesView(driver),
  });

  const mouseKitSeed = await setNotesInput(driver, "[scripts](kit://scripts)");
  check("mouse_kit_scripts_link_seeded", mouseKitSeed.success === true, { batch: mouseKitSeed });
  const mouseKitClick = await clickNotesEditorFirstLine(driver);
  check("mouse_kit_scripts_click_dispatches", mouseKitClick.every((result) => result.success), {
    click: mouseKitClick,
  });
  const mouseKitPreviewOpened = await pollUntil("mouse-kit-scripts-preview-open", async () => {
    const preview = await notesKitResourcePreview(driver);
    return (
      preview.active === true &&
      preview.uri === "kit://scripts" &&
      preview.readOnly === true
    );
  });
  check("mouse_kit_scripts_opens_read_only_preview", mouseKitPreviewOpened, {
    preview: await notesKitResourcePreview(driver),
  });
  await escape(driver);
  const mouseKitPreviewClosed = await pollUntil("mouse-kit-scripts-preview-closed", async () => {
    const preview = await notesKitResourcePreview(driver);
    return preview.active === false;
  });
  check("escape_closes_mouse_kit_scripts_preview", mouseKitPreviewClosed, {
    view: await notesView(driver),
    preview: await notesKitResourcePreview(driver),
  });

  receipt.sessionDir = driver.sessionDir;
  receipt.pass = receipt.failures.length === 0;
  console.log(JSON.stringify(receipt, null, 2));
  if (!receipt.pass) process.exitCode = 1;
} finally {
  await driver.close();
}
