/**
 * Repro probe: Enter + row shortcuts in the Notes Cmd+K actions panel.
 *
 * User report: pressing Enter on a focused action in the notes popups does
 * nothing; Find in Note doesn't focus the search input; other row shortcuts
 * may be dead. Drives REAL native keys and captures notes + dialog state
 * around each interaction.
 */
import { Driver } from "../devtools/driver";

const BINARY = "target-agent/artifacts/notes-popup-fix/script-kit-gpui";

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
  sessionName: "notes-actions-enter-probe",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

async function dialogState(): Promise<Json | null> {
  try {
    const result = (await driver.request(
      { type: "getState", target: { type: "kind", kind: "actionsDialog" } },
      { expect: "stateResult", timeoutMs: 3000 },
    )) as Json;
    return result.actionsDialog ?? null;
  } catch {
    return null;
  }
}

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

async function popupOpen(): Promise<boolean> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  return (result.windows ?? []).some((w: Json) => w.id === "actions-dialog");
}

const report: Json = {};

try {
  driver.send({ type: "openNotes", requestId: "probe-open-notes" });
  await Bun.sleep(2000);
  await osa(
    `tell application "System Events" to set frontmost of (first process whose unix id is ${driver.pid}) to true`,
  );
  await Bun.sleep(600);

  await keystroke("alpha probe note");
  await Bun.sleep(500);

  const notesBefore = await notesState();
  report.notes_before = {
    keys: notesBefore ? Object.keys(notesBefore) : null,
    raw: notesBefore,
  };

  // --- Scenario 1: Cmd+K, Enter on the default-selected action ---
  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  const dlg = await dialogState();
  report.s1_dialog_after_open = {
    popupOpen: await popupOpen(),
    selection: dlg?.selection,
    searchLen: dlg?.search?.textLength,
  };

  await keyCode(36); // return
  await Bun.sleep(1200);
  report.s1_after_enter = {
    popupOpen: await popupOpen(),
    notes: await notesState(),
  };
  // NOTE: do NOT press Escape between scenarios — with no popup open it
  // closes the whole Notes window and surfaces the main launcher, silently
  // retargeting the rest of the probe at the MAIN actions popup.

  // --- Scenario 2: Cmd+K, filter "find", Enter -> search bar focused ---
  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  const s2Open = await dialogState();
  report.s2_selection_per_char = [
    { after: "open", selection: s2Open?.selection },
  ];
  for (const ch of "find") {
    await keystroke(ch);
    await Bun.sleep(400);
    const st = await dialogState();
    report.s2_selection_per_char.push({
      after: ch,
      searchLen: st?.search?.textLength,
      selection: st?.selection,
    });
  }
  const dlgFind = await dialogState();
  report.s2_dialog_filtered = {
    selection: dlgFind?.selection,
    searchLen: dlgFind?.search?.textLength,
  };
  await keyCode(36); // return
  await Bun.sleep(1200);
  report.s2_after_enter = {
    popupOpen: await popupOpen(),
    notes: await notesState(),
  };

  // --- Scenario 3: Cmd+K, press row shortcut cmd+d (Duplicate Note) ---
  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  await keystroke("d", ["command"]);
  await Bun.sleep(1200);
  report.s3_after_cmd_d = {
    popupOpen: await popupOpen(),
    notes: await notesState(),
  };
} finally {
  try {
    await keyCode(53);
  } catch {}
  await driver.close();
}

console.log(JSON.stringify(report, null, 2));
