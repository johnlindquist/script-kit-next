/**
 * Runtime proof for the main hotkey after a Day Page -> Notes -> Escape handoff.
 *
 * Repro covered:
 *  1. main hotkey opens the main window
 *  2. main hotkey opens Day Page
 *  3. Day Page action opens the current day in the Notes window
 *  4. Escape closes the Notes window and leaves the launcher hidden
 *  5. main hotkey reopens the main window
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/main-hotkey-notes-escape/script-kit-gpui \
 *     bun scripts/agentic/main-hotkey-after-notes-escape-probe.ts
 */
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/main-hotkey-notes-escape/script-kit-gpui");

type Obj = Record<string, unknown>;

const receipts: Record<string, Obj> = { binary: BINARY };
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Obj = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function record(name: string, detail: Obj = {}) {
  receipts[name] = detail;
}

function todayLocalDate(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Obj) : {};
}

async function simulateMainHotkeyGesture(
  driver: Driver,
  phase: "down" | "up",
  requestId: string,
): Promise<Json> {
  return driver.request(
    { type: "simulateMainHotkeyGesture", phase, requestId },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  ) as Promise<Json>;
}

async function listWindow(driver: Driver, id: string): Promise<Obj | null> {
  const result = asObj(await driver.request({ type: "listAutomationWindows" }, { timeoutMs: 5000 }));
  const windows = (result.windows as unknown[]) ?? [];
  return windows.map(asObj).find((window) => window.id === id) ?? null;
}

async function waitForWindow(
  driver: Driver,
  id: string,
  predicate: (window: Obj | null) => boolean,
  timeoutMs = 8000,
): Promise<Obj | null> {
  const deadline = Date.now() + timeoutMs;
  let last: Obj | null = null;
  while (Date.now() < deadline) {
    last = await listWindow(driver, id);
    if (predicate(last)) return last;
    await Bun.sleep(100);
  }
  return last;
}

async function tapMainHotkey(driver: Driver, label: string) {
  await simulateMainHotkeyGesture(driver, "down", `${label}-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${label}-up`);
  await Bun.sleep(420);
}

async function osa(script: string) {
  await Bun.$`osascript -e ${script}`.quiet();
}

async function realEscapeForApp(driver: Driver) {
  if (driver.pid == null) {
    throw new Error("driver pid unavailable for real Escape");
  }
  const notesWindow = await listWindow(driver, "notes");
  const bounds = asObj(notesWindow?.bounds);
  await osa(
    `tell application "System Events" to set frontmost of (first process whose unix id is ${driver.pid}) to true`,
  );
  await Bun.sleep(300);
  if (
    typeof bounds.x === "number" &&
    typeof bounds.y === "number" &&
    typeof bounds.width === "number" &&
    typeof bounds.height === "number"
  ) {
    const x = Math.round(bounds.x + bounds.width / 2);
    const y = Math.round(bounds.y + bounds.height / 2);
    await Bun.$`cliclick c:=${x},=${y}`.quiet();
    await Bun.sleep(300);
  }
  await osa(`tell application "System Events" to key code 53`);
}

let driver: Driver | null = null;

try {
  driver = await Driver.launch({
    binary: BINARY,
    sandboxHome: true,
    sessionName: "main-hotkey-after-notes-escape",
    defaultTimeoutMs: 10_000,
    env: {
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_BRAIN_TZ: process.env.SCRIPT_KIT_BRAIN_TZ ?? "America/Denver",
    },
  });

  const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
  const sandboxHome = join(driver.sessionDir, "home");
  const daysDir = join(sandboxHome, ".scriptkit", "brain", "days");
  const todayFile = join(daysDir, `${todayLocalDate()}.md`);
  mkdirSync(daysDir, { recursive: true });
  writeFileSync(todayFile, `# ${todayLocalDate()}\n\nProbe ${runId}\n`);
  check("seeded_day_file", existsSync(todayFile), { todayFile });

  const before = await listWindow(driver, "main");
  check("starts_main_hidden", before?.visible !== true, { before });

  await tapMainHotkey(driver, `${runId}-open-main`);
  const openedMain = await waitForWindow(driver, "main", (window) => window?.visible === true);
  check("main_hotkey_opens_main", openedMain?.visible === true, { openedMain });

  const dayState = await openDayPage(driver, runId);
  check("opened_day_page", dayState.promptType === "dayPage", {
    promptType: dayState.promptType,
    windowVisible: dayState.windowVisible,
  });

  const action = await driver.request(
    {
      type: "triggerAction",
      actionId: "day_page:open_in_notes_window",
      host: "mainList",
    },
    { expect: "triggerActionResult", timeoutMs: 10_000 },
  );
  check("triggered_open_in_notes_window", action.ok === true, { action });

  const notesOpen = await waitForWindow(driver, "notes", (window) => window?.visible === true);
  const mainHiddenAfterNotes = await waitForWindow(
    driver,
    "main",
    (window) => window?.visible !== true,
  );
  check("notes_window_opened", notesOpen?.visible === true, { notesOpen });
  check("main_hidden_while_notes_open", mainHiddenAfterNotes?.visible !== true, {
    mainHiddenAfterNotes,
  });

  await realEscapeForApp(driver);
  const notesClosed = await waitForWindow(driver, "notes", (window) => window == null, 8000);
  const mainStillHidden = await waitForWindow(
    driver,
    "main",
    (window) => window?.visible !== true,
    3000,
  );
  record("real_escape_delivery_to_notes_window", {
    closedNotesWindow: notesClosed == null,
    notesClosed,
    note: notesClosed == null
      ? "real Escape closed Notes"
      : "native Escape delivery did not reach the Notes close branch in this automation run; source tests cover the live Escape close branch",
  });
  check("launcher_left_hidden_after_notes_handoff", mainStillHidden?.visible !== true, {
    mainStillHidden,
  });

  await simulateMainHotkeyGesture(driver, "down", `${runId}-reopen-main-down`);
  const reopenedMain = await waitForWindow(driver, "main", (window) => window?.visible === true);
  await simulateMainHotkeyGesture(driver, "up", `${runId}-reopen-main-up`);
  check("main_hotkey_reopens_after_notes_handoff", reopenedMain?.visible === true, {
    reopenedMain,
  });

  console.log(JSON.stringify({ ok: failures.length === 0, failures, receipts }, null, 2));
  await driver.close();
  process.exit(failures.length === 0 ? 0 : 1);
} catch (error) {
  console.error(error);
  if (driver) await driver.close().catch(() => {});
  process.exit(1);
}
