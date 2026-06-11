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

async function editorLen(): Promise<number | null> {
  const st = await notesState();
  return st?.editor?.textLength ?? null;
}

async function notesCount(): Promise<number | null> {
  const st = await notesState();
  return st?.counts?.notes ?? null;
}

async function routedLogCount(): Promise<number> {
  try {
    const log = await Bun.file(`${driver.sessionDir}/app.log`).text();
    return log.split("notes_popup_key_routed").length - 1;
  } catch {
    return -1;
  }
}

async function popupOpen(): Promise<boolean> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  return (result.windows ?? []).some((w: Json) => w.id === "actions-dialog");
}

async function shot(name: string): Promise<string | null> {
  try {
    const result = (await driver.request(
      { type: "captureScreenshot", target: { type: "kind", kind: "notes" } },
      { expect: "screenshotResult", timeoutMs: 5000 },
    )) as Json;
    if (result.data) {
      const dest = `.test-screenshots/notes-enter-${name}.png`;
      await Bun.$`mkdir -p .test-screenshots`.quiet();
      await Bun.write(dest, Buffer.from(result.data, "base64"));
      return dest;
    }
    return result.error ?? null;
  } catch (e) {
    return `error: ${e}`;
  }
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
  // Seed the (new, empty) note body so the find-bar match counter becomes a
  // legible focus proof in s2: querying "zzz" should flip it from 0/0 to 1/1.
  await keystroke("zzz match target");
  await Bun.sleep(600);
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
  const editorLenBeforeEnter = await editorLen();
  await keyCode(36); // return
  await Bun.sleep(1200);
  report.s2_after_enter = {
    popupOpen: await popupOpen(),
    shot: await shot("s2-find-via-popup"),
  };
  // Focus proof: type into whatever is focused. If the find input has focus
  // (the fix), "zzz" lands there and the note body length is unchanged. If
  // focus was stolen back to the editor (the bug), the body grows by 3.
  await keystroke("zzz");
  await Bun.sleep(600);
  const editorLenAfterTyping = await editorLen();
  report.s2_focus_proof = {
    editorLenBeforeEnter,
    editorLenAfterTyping,
    typedTextWentToNoteBody:
      editorLenBeforeEnter !== null &&
      editorLenAfterTyping !== null &&
      editorLenAfterTyping !== editorLenBeforeEnter,
    shot: await shot("s2-find-typed-via-popup"),
  };
  // Close any find UI that opened so s3 starts clean.
  await keyCode(53);
  await Bun.sleep(500);
  report.s2_escape_left_notes_open = await notesState() !== null;

  // Live Cmd+F comparison: the action should match this exact result.
  await keystroke("f", ["command"]);
  await Bun.sleep(900);
  const editorLenBeforeCmdFTyping = await editorLen();
  await keystroke("zzz");
  await Bun.sleep(600);
  report.s2b_cmd_f = {
    shot: await shot("s2b-find-typed-via-cmd-f"),
    editorLenBeforeCmdFTyping,
    editorLenAfterTyping: await editorLen(),
  };
  await keyCode(53);
  await Bun.sleep(500);

  // --- Scenario 3: Cmd+K, press row shortcut cmd+d (Duplicate Note) ---
  const s3NotesBefore = await notesCount();
  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  await keystroke("d", ["command"]);
  await Bun.sleep(1200);
  report.s3_after_cmd_d = {
    popupOpen: await popupOpen(),
    notesBefore: s3NotesBefore,
    notesAfter: await notesCount(),
  };

  // --- Scenario 4: router-path attempt. Open the popup via protocol
  // automation (toggle_notes_popup_for_automation) instead of a real Cmd+K,
  // then send a REAL cmd+d. If AppKit leaves the Notes window key, the key
  // flows through NotesApp::handle_key_down's popup branch — the path that
  // previously had no row-shortcut matching. notes_popup_key_routed log
  // lines tell us which path actually fired.
  const s4RoutedBefore = await routedLogCount();
  const s4NotesBefore = await notesCount();
  driver.send({
    type: "simulateKey",
    key: "k",
    modifiers: ["cmd"],
    target: { type: "kind", kind: "notes" },
  } as any);
  await Bun.sleep(1200);
  report.s4_popup_open_via_automation = await popupOpen();
  await keystroke("d", ["command"]);
  await Bun.sleep(1200);
  report.s4_router_path = {
    popupOpenAfter: await popupOpen(),
    notesBefore: s4NotesBefore,
    notesAfter: await notesCount(),
    routedLogLinesBefore: s4RoutedBefore,
    routedLogLinesAfter: await routedLogCount(),
  };
} finally {
  try {
    await keyCode(53);
  } catch {}
  await driver.close();
}

console.log(JSON.stringify(report, null, 2));
