/**
 * Runtime proof for two Notes-window contracts:
 *
 * 1. Escape focus restoration — closing the Cmd+K actions popup or the
 *    Cmd+P note switcher with a REAL Escape key must return keyboard focus
 *    to the notes editor: typing immediately afterwards lands in the note
 *    body (editor textLength grows), regardless of whether AppKit made the
 *    detached popup or the Notes window key.
 *
 * 2. Reset Window Position — after dragging the Notes window elsewhere
 *    (via AX set position), running the new "Reset Window Position" action
 *    from the Cmd+K menu must restore the default top-right placement the
 *    window opened with.
 *
 * Pass criteria in the printed report:
 *   - s1_actions_escape_focus.editorGrewAfterEscape === true
 *   - s2_switcher_escape_focus.editorGrewAfterEscape === true
 *   - s3_reset_position.returnedToDefault === true
 */
import { Driver } from "../devtools/driver";

const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/notes-popups/script-kit-gpui";

function osa(script: string) {
  return Bun.$`osascript -e ${script}`.quiet();
}

async function keystroke(text: string, mods: string[] = []) {
  const using = mods.length
    ? ` using {${mods.map((m) => `${m} down`).join(", ")}}`
    : "";
  await osa(`tell application "System Events" to keystroke "${text}"${using}`);
}

async function keyCode(code: number) {
  await osa(`tell application "System Events" to key code ${code}`);
}

type Json = Record<string, any>;

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "notes-escape-focus-reset-position",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

async function notesState(): Promise<Json | null> {
  try {
    const result = (await driver.request(
      { type: "getState", target: { type: "kind", kind: "notes" } },
      { expect: "stateResult", timeoutMs: 3000 },
    )) as Json;
    return result.notes ?? result;
  } catch {
    return null;
  }
}

async function editorLen(): Promise<number | null> {
  const st = await notesState();
  return st?.editor?.textLength ?? null;
}

async function popupOpen(): Promise<boolean> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  return (result.windows ?? []).some((w: Json) => w.id === "actions-dialog");
}

async function notesBounds(): Promise<Json | null> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  return (result.windows ?? []).find((w: Json) => w.id === "notes")?.bounds ?? null;
}

async function logEventCount(needle: string): Promise<number> {
  try {
    const log = await Bun.file(`${driver.sessionDir}/app.log`).text();
    return log.split(needle).length - 1;
  } catch {
    return -1;
  }
}

const report: Json = {};

// Both the first-open placement and the reset action anchor to the display
// the mouse cursor is on. Pin the mouse to one spot for the whole probe so
// "reset returns to the open-time default" is a deterministic assertion.
// (cliclick needs `=` prefixes for negative coordinates.)
const mousePos = (await Bun.$`cliclick p`.text()).trim();
const [mx, my] = mousePos.split(",");
const pinMouse = () => Bun.$`cliclick m:=${mx},=${my}`.quiet();

try {
  await pinMouse();
  driver.send({ type: "openNotes", requestId: "probe-open-notes" });
  await Bun.sleep(2000);
  await osa(
    `tell application "System Events" to set frontmost of (first process whose unix id is ${driver.pid}) to true`,
  );
  await Bun.sleep(600);

  const defaultBounds = await notesBounds();
  report.default_bounds = defaultBounds;

  // Sanity: editor receives typing before any popup interaction.
  const len0 = await editorLen();
  await keystroke("abc");
  await Bun.sleep(500);
  const len1 = await editorLen();
  report.editor_sanity = { len0, len1, editorFocusedAtStart: len1 === (len0 ?? 0) + 3 };

  // --- Scenario 1: Cmd+K actions popup, REAL Escape, then type ---
  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  const s1Open = await popupOpen();
  await keyCode(53); // escape
  await Bun.sleep(900);
  const s1Closed = !(await popupOpen());
  const s1LenBefore = await editorLen();
  await keystroke("xyz");
  await Bun.sleep(500);
  const s1LenAfter = await editorLen();
  report.s1_actions_escape_focus = {
    popupOpened: s1Open,
    popupClosedViaEscape: s1Closed,
    editorLenBeforeTyping: s1LenBefore,
    editorLenAfterTyping: s1LenAfter,
    editorGrewAfterEscape:
      s1LenBefore !== null && s1LenAfter === (s1LenBefore ?? 0) + 3,
  };

  // --- Scenario 2: Cmd+P note switcher, REAL Escape, then type ---
  await keystroke("p", ["command"]);
  await Bun.sleep(900);
  const s2Open = await popupOpen();
  await keyCode(53); // escape
  await Bun.sleep(900);
  const s2Closed = !(await popupOpen());
  const s2LenBefore = await editorLen();
  await keystroke("qrs");
  await Bun.sleep(500);
  const s2LenAfter = await editorLen();
  report.s2_switcher_escape_focus = {
    popupOpened: s2Open,
    popupClosedViaEscape: s2Closed,
    editorLenBeforeTyping: s2LenBefore,
    editorLenAfterTyping: s2LenAfter,
    editorGrewAfterEscape:
      s2LenBefore !== null && s2LenAfter === (s2LenBefore ?? 0) + 3,
  };

  // --- Scenario 3: drag the window away via AX, run Reset Window Position ---
  await osa(
    `tell application "System Events" to tell (first process whose unix id is ${driver.pid}) to set position of window "Notes" to {120, 480}`,
  );
  await Bun.sleep(800);
  const movedBounds = await notesBounds();
  // AX move doesn't flow through the automation registry; the registry may
  // still report the old frame. The real proof is the FINAL frame below.
  report.s3_moved_bounds = movedBounds;

  await pinMouse();
  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  for (const ch of "reset") {
    await keystroke(ch);
    await Bun.sleep(250);
  }
  await keyCode(36); // return -> Reset Window Position
  await Bun.sleep(1200);

  const resetBounds = await notesBounds();
  const close = (a: number | undefined, b: number | undefined) =>
    a !== undefined && b !== undefined && Math.abs(a - b) <= 4;
  report.s3_reset_position = {
    resetBounds,
    defaultBounds,
    resetEventInLog: (await logEventCount("notes_window_position_reset")) > 0,
    returnedToDefault:
      close(resetBounds?.x, defaultBounds?.x) &&
      close(resetBounds?.y, defaultBounds?.y) &&
      close(resetBounds?.width, defaultBounds?.width),
  };

  report.on_close_restore_events = await logEventCount(
    "notes_detached_popup_closed_externally",
  );
} finally {
  try {
    await keyCode(53);
  } catch {}
  await driver.close();
}

console.log(JSON.stringify(report, null, 2));
