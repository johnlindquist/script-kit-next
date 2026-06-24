#!/usr/bin/env bun
/**
 * Runtime proof for Today Cmd+P unified Notes switcher:
 * - Day Page Cmd+P lists day notes and regular notes with Notes row actions.
 * - Selecting a past day or regular note stays local in Day Page.
 * - Dirty Today/note content is saved before successful local switches.
 * - The floating Notes Window opens only through explicit Open in Notes Window.
 */
import { existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { randomUUID } from "node:crypto";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/today-notes-switcher/script-kit-gpui");

type Obj = Record<string, any>;

const runId = `today-notes-switcher-${Date.now()}`;
const receipt: Obj = {
  tool: "notes-day-page-switcher-probe",
  binary: BINARY,
  pass: false,
  failures: [] as string[],
  selectedRows: [] as Obj[],
  dirtySave: {},
};

const externalBrainDir = join("/tmp", `${runId}-notes-brain`);
const externalNotesDb = join("/tmp", `${runId}-notes.sqlite`);
const externalNotesDir = join(externalBrainDir, "notes");
rmSync(externalBrainDir, { recursive: true, force: true });
rmSync(externalNotesDb, { force: true });
mkdirSync(externalNotesDir, { recursive: true });

const noteId = randomUUID();
const secondNoteId = randomUUID();
const created = new Date().toISOString();
const noteTitle = `Switcher Probe Note ${runId}`;
const secondNoteTitle = `Switcher Probe Second ${runId}`;
const noteBody = `# ${noteTitle}\n\nregular note local body ${runId}\n`;
const secondNoteBody = `# ${secondNoteTitle}\n\nsecond note body ${runId}\n`;
writeFileSync(
  join(externalNotesDir, `switcher-probe-note-${runId}.md`),
  `---\nid: ${noteId}\ncreated: ${created}\nupdated: ${created}\n---\n\n${noteBody}`,
);
writeFileSync(
  join(externalNotesDir, `switcher-probe-second-${runId}.md`),
  `---\nid: ${secondNoteId}\ncreated: ${created}\nupdated: ${created}\n---\n\n${secondNoteBody}`,
);

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Obj) : {};
}

function check(name: string, ok: boolean, detail: Obj = {}) {
  receipt[name] = { ok, ...detail };
  if (!ok) receipt.failures.push(name);
}

function todayLocalDate(): string {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}-${String(
    now.getDate(),
  ).padStart(2, "0")}`;
}

function fileText(path: string): string | null {
  return existsSync(path) ? readFileSync(path, "utf8") : null;
}

function fingerprint(text: string): string {
  let hash = 0;
  for (let i = 0; i < text.length; i += 1) {
    hash = (hash * 31 + text.charCodeAt(i)) >>> 0;
  }
  return `${text.length}:${hash.toString(16)}`;
}

async function pollUntil(label: string, fn: () => Promise<boolean>, timeoutMs = 8000): Promise<boolean> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (await fn()) return true;
    await Bun.sleep(120);
  }
  receipt[`timeout_${label}`] = true;
  return false;
}

async function getState(driver: Driver): Promise<Obj> {
  return asObj(await driver.getState({ timeoutMs: 8000 }));
}

async function setDayPageInput(driver: Driver, text: string, label: string): Promise<Obj> {
  const batch = asObj(
    await driver.batch(
      [
        { type: "setInput", text },
        {
          type: "waitFor",
          condition: { type: "stateMatch", state: { promptType: "dayPage", inputValue: text } },
        },
      ],
      { timeoutMs: 8000 },
    ),
  );
  check(`set_day_page_input_${label}`, batch.success === true, { batch });
  return batch;
}

async function waitForInput(driver: Driver, text: string, label: string): Promise<Obj> {
  const ok = await pollUntil(label, async () => {
    const state = await getState(driver);
    return state.promptType === "dayPage" && state.inputValue === text;
  });
  const state = await getState(driver);
  check(`input_matches_${label}`, ok, { inputValue: state.inputValue });
  return state;
}

function isActionsWindow(win: Obj): boolean {
  return (
    win.id === "actions-dialog" ||
    win.automationId === "actions-dialog" ||
    win.kind === "ActionsDialog" ||
    win.windowKind === "ActionsDialog" ||
    win.semanticSurface === "actionsDialog"
  );
}

function isNotesWindow(win: Obj): boolean {
  const text = JSON.stringify(win).toLowerCase();
  return text.includes("notes") && !text.includes("actions-dialog");
}

async function listWindows(driver: Driver): Promise<Obj[]> {
  const result = asObj(await driver.listAutomationWindows({ timeoutMs: 5000 }));
  return Array.isArray(result.windows) ? result.windows.map(asObj) : [];
}

async function actionsWindowRegistered(driver: Driver): Promise<boolean> {
  return (await listWindows(driver)).some(isActionsWindow);
}

async function notesWindowOpen(driver: Driver): Promise<boolean> {
  return (await listWindows(driver)).some(isNotesWindow);
}

async function actionsDialogState(driver: Driver): Promise<Obj> {
  if (!(await actionsWindowRegistered(driver).catch(() => false))) {
    return asObj((await getState(driver)).actionsDialog);
  }
  const state = asObj(
    await driver.request(
      { type: "getState", target: { type: "kind", kind: "actionsDialog" }, summaryOnly: true },
      { expect: "stateResult", timeoutMs: 6000 },
    ),
  );
  return asObj(state.actionsDialog);
}

function visibleActions(dialog: Obj): Obj[] {
  if (Array.isArray(dialog.visibleActions)) return dialog.visibleActions.map(asObj);
  const sample = asObj(dialog.actions).visibleSample;
  return Array.isArray(sample) ? sample.map(asObj) : [];
}

function rowActionId(row: Obj): string {
  return String(row.id ?? row.actionId ?? row.value ?? "");
}

async function waitForActionsReady(driver: Driver): Promise<void> {
  for (let i = 0; i < 50; i += 1) {
    const state = await getState(driver).catch(() => null);
    const registered = await actionsWindowRegistered(driver).catch(() => false);
    if (state?.promptType === "actionsDialog" || state?.actionsDialog?.open === true || registered) {
      return;
    }
    await Bun.sleep(100);
  }
  throw new Error("ActionsDialog did not become automation-ready");
}

async function openCommandBar(driver: Driver, key: "p" | "k"): Promise<Obj> {
  await driver.simulateKey(key, ["cmd"]);
  await waitForActionsReady(driver);
  return actionsDialogState(driver);
}

async function filterActions(driver: Driver, text: string): Promise<void> {
  const payload: Obj = {
    type: "batch",
    requestId: `${runId}-filter-${text.replace(/[^a-z0-9]+/gi, "-")}-${Date.now()}`,
    commands: [{ type: "setInput", text }],
    options: { stopOnError: true, timeout: 5000 },
  };
  if (await actionsWindowRegistered(driver).catch(() => false)) {
    payload.target = { type: "kind", kind: "actionsDialog" };
  }
  await driver.request(payload, { expect: "batchResult", timeoutMs: 6000 });
}

async function findAction(driver: Driver, actionId: string | RegExp, filter: string): Promise<Obj> {
  await filterActions(driver, filter);
  for (let i = 0; i < 40; i += 1) {
    const dialog = await actionsDialogState(driver).catch(() => ({}));
    const row = visibleActions(dialog).find((candidate) => {
      const id = rowActionId(candidate);
      return typeof actionId === "string" ? id === actionId : actionId.test(id);
    });
    if (row) return { dialog, row };
    await Bun.sleep(100);
  }
  return { dialog: await actionsDialogState(driver).catch(() => ({})), row: null };
}

async function activateFilteredAction(
  driver: Driver,
  openKey: "p" | "k",
  actionId: string | RegExp,
  filter: string,
): Promise<Obj> {
  await openCommandBar(driver, openKey);
  const found = await findAction(driver, actionId, filter);
  check(`found_action_${filter.replace(/[^a-z0-9]+/gi, "_")}`, Boolean(found.row), {
    expected: String(actionId),
    row: asObj(found.row),
    visible: visibleActions(asObj(found.dialog)).slice(0, 8),
  });
  const semanticId = String(asObj(found.row).semanticId ?? "");
  if (semanticId.startsWith("choice:")) {
    const selectPayload: Obj = {
      type: "batch",
      requestId: `${runId}-select-${filter.replace(/[^a-z0-9]+/gi, "-")}`,
      commands: [{ type: "selectBySemanticId", semanticId }],
      options: { stopOnError: true, timeout: 5000 },
    };
    if (await actionsWindowRegistered(driver).catch(() => false)) {
      selectPayload.target = { type: "kind", kind: "actionsDialog" };
    }
    await driver.request(selectPayload, { expect: "batchResult", timeoutMs: 6000 });
  }
  const target = (await actionsWindowRegistered(driver).catch(() => false))
    ? { type: "kind", kind: "actionsDialog" }
    : { type: "main" };
  const activate = await driver.request(
    {
      type: "simulateGpuiEvent",
      requestId: `${runId}-activate-${filter.replace(/[^a-z0-9]+/gi, "-")}`,
      target,
      event: { type: "keyDown", key: "enter", modifiers: [] },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 6000 },
  );
  await Bun.sleep(450);
  return { ...found, semanticId, activate: asObj(activate) };
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "today-notes-switcher",
  sandboxHome: true,
  defaultTimeoutMs: 9000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_TEST_NOTES_BRAIN_PATH: externalBrainDir,
    SCRIPT_KIT_TEST_NOTES_DB_PATH: externalNotesDb,
  },
});

try {
  const home = join(driver.sessionDir, "home");
  const brainDir = join(home, ".scriptkit", "brain");
  const daysDir = join(brainDir, "days");
  mkdirSync(daysDir, { recursive: true });

  const today = todayLocalDate();
  const todayPath = join(daysDir, `${today}.md`);
  const pastDate = "2026-06-01";
  const pastDayText = `past day local switch target ${runId}\nsecond line\n`;
  writeFileSync(join(daysDir, `${pastDate}.md`), pastDayText);

  const opened = await openDayPage(driver, runId);
  check("opened_day_page", opened.promptType === "dayPage", { promptType: opened.promptType });
  check("notes_window_closed_initially", !(await notesWindowOpen(driver)), {
    windows: await listWindows(driver),
  });

  const dirtyToday = `dirty today before day switch ${runId}`;
  await setDayPageInput(driver, dirtyToday, "dirty_today");
  const dayAction = await activateFilteredAction(driver, "p", `note_day:${pastDate}`, pastDate);
  await waitForInput(driver, pastDayText, "past_day");
  const todayDisk = fileText(todayPath) ?? "";
  const noNotesAfterDay = !(await notesWindowOpen(driver));
  receipt.selectedRows.push({
    selectedRowKind: "day",
    selectedDate: pastDate,
    selectedActionId: rowActionId(asObj(dayAction.row)),
    mainWindowKind: "dayPage",
    dayPageBinding: "day",
  });
  receipt.dirtySave.todayToPastDay = {
    previousBinding: "today",
    saved: todayDisk === dirtyToday,
    savedContentFingerprint: fingerprint(todayDisk),
  };
  check("selecting_past_day_stays_in_day_page", (await getState(driver)).promptType === "dayPage", {
    action: dayAction,
  });
  check("dirty_today_saved_before_past_day_switch", todayDisk === dirtyToday, {
    todayPath,
    todayDisk,
  });
  check("notes_window_closed_after_day_selection", noNotesAfterDay, { windows: await listWindows(driver) });

  await driver.simulateKey("escape");
  await waitForInput(driver, dirtyToday, "return_to_today");

  const noteAction = await activateFilteredAction(driver, "p", `note_${noteId}`, noteTitle);
  await waitForInput(driver, noteBody, "regular_note");
  const noNotesAfterNote = !(await notesWindowOpen(driver));
  receipt.selectedRows.push({
    selectedRowKind: "note",
    selectedNoteId: noteId,
    selectedActionId: rowActionId(asObj(noteAction.row)),
    mainWindowKind: "dayPage",
    dayPageBinding: "note",
  });
  check("selecting_regular_note_stays_in_day_page", (await getState(driver)).promptType === "dayPage", {
    action: noteAction,
  });
  check("notes_window_closed_after_note_selection", noNotesAfterNote, { windows: await listWindows(driver) });

  const editedNote = `# ${noteTitle} Edited\n\nedited local note body ${runId}\n`;
  await setDayPageInput(driver, editedNote, "dirty_regular_note");
  const secondNoteAction = await activateFilteredAction(driver, "p", `note_${secondNoteId}`, secondNoteTitle);
  await waitForInput(driver, secondNoteBody, "second_regular_note");
  const noteFiles = existsSync(externalNotesDir)
    ? readdirSync(externalNotesDir).map((file) => join(externalNotesDir, file))
    : [];
  const savedNoteFiles = noteFiles.filter((path) => (fileText(path) ?? "").includes(`edited local note body ${runId}`));
  receipt.dirtySave.noteToNote = {
    previousBinding: "note",
    saved: savedNoteFiles.length === 1,
    savedContentFingerprint: fingerprint(savedNoteFiles.map((path) => fileText(path) ?? "").join("\n")),
  };
  check("dirty_regular_note_saved_before_local_switch", savedNoteFiles.length === 1, {
    savedNoteFiles,
    noteFiles,
    action: secondNoteAction,
  });

  receipt.notesWindowOpenBeforeExplicitAction = await notesWindowOpen(driver);
  const explicit = await activateFilteredAction(
    driver,
    "k",
    "day_page:open_in_notes_window",
    "Open in Notes Window",
  );
  const notesOpened = await pollUntil("notes-window-open", async () => notesWindowOpen(driver), 9000);
  receipt.notesWindowOpenAfterExplicitAction = notesOpened;
  check("notes_window_closed_before_explicit_action", receipt.notesWindowOpenBeforeExplicitAction === false, {
    windows: await listWindows(driver),
  });
  check("explicit_open_in_notes_window_opens_notes_window", notesOpened, {
    action: explicit,
    windows: await listWindows(driver),
  });

  receipt.mainWindowKind = "dayPage";
  receipt.dayPageBinding = "note";
  receipt.pass = receipt.failures.length === 0;
} finally {
  await driver.close().catch(() => {});
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
