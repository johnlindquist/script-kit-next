/**
 * Proof probe: Enter in the Notes-hosted Agent Chat history popup (Cmd+P).
 *
 * Seeds agent_chat-history.jsonl into the sandbox kit dir, opens Notes,
 * switches to the Agent Chat surface (Cmd+Shift+A), and proves Enter selects
 * the highlighted history entry on BOTH keyboard paths:
 *
 *  Phase A — popup-key path: native osascript keystrokes land in the detached
 *  actions window (it is the OS key window). Filter to one entry, Enter,
 *  expect the popup to close and the selection to dispatch.
 *
 *  Phase B — parent-key path: reopen Cmd+P, then dispatch keys through the
 *  NOTES window's real GPUI pipeline via simulateGpuiEvent. This emulates the
 *  reported bug state (Notes window stayed key while the popup was open).
 *  The keyboard router must forward typing + Enter into the popup via
 *  route_key_to_detached_actions_window instead of leaking them into the
 *  composer.
 */
import { Driver } from "../devtools/driver";
import { join } from "path";

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
  sessionName: "notes-agent-chat-history-enter",
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

async function popupOpen(): Promise<boolean> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  return (result.windows ?? []).some((w: Json) => w.id === "actions-dialog");
}

async function notesGpuiKey(key: string, text?: string, mods: string[] = []) {
  const event: Json = { type: "keyDown", key };
  if (text) event.text = text;
  if (mods.length) event.modifiers = mods;
  await driver.request(
    {
      type: "simulateGpuiEvent",
      target: { type: "kind", kind: "notes", index: 0 },
      event,
    },
    { timeoutMs: 5000 },
  );
}

function brief(st: Json | null): Json {
  if (!st) return { missing: true };
  return {
    searchLen: st.search?.textLength,
    selectionActionId: st.selection?.actionId,
    totalCount: st.actions?.totalCount,
    filteredCount: st.actions?.filteredCount,
    routeId: st.route?.currentRouteId,
  };
}

const report: Json = {};

try {
  // Seed Agent Chat conversation history into the sandbox kit dir.
  const kitDir = join(driver.sessionDir, "home", ".scriptkit");
  const entries = [
    {
      timestamp: "2026-06-11T10:00:00Z",
      first_message: "First seeded conversation about rust lifetimes",
      message_count: 4,
      session_id: "seeded-session-one",
      title: "First seeded conversation",
      preview: "We talked about rust lifetimes",
      search_text: "first seeded conversation rust lifetimes",
    },
    {
      timestamp: "2026-06-11T11:00:00Z",
      first_message: "Second seeded conversation about gpui",
      message_count: 6,
      session_id: "seeded-session-two",
      title: "Second seeded conversation",
      preview: "We talked about gpui",
      search_text: "second seeded conversation gpui",
    },
  ];
  await Bun.write(
    join(kitDir, "agent_chat-history.jsonl"),
    entries.map((e) => JSON.stringify(e)).join("\n") + "\n",
  );

  driver.send({ type: "openNotes", requestId: "probe-open-notes" });
  await Bun.sleep(2000);
  await osa(
    `tell application "System Events" to set frontmost of (first process whose unix id is ${driver.pid}) to true`,
  );
  await Bun.sleep(600);

  // Switch the Notes window to the Agent Chat surface.
  await keystroke("a", ["command", "shift"]);
  await Bun.sleep(2500);

  // ── Phase A: popup is the key window, native keystrokes ──────────────
  await keystroke("p", ["command"]);
  await Bun.sleep(1200);
  report.a_popup_open = await popupOpen();
  report.a_dialog_open = brief(await dialogState());

  // Filter to the first entry only ("rust" matches its search_text).
  for (const ch of "rust") await keystroke(ch);
  await Bun.sleep(600);
  report.a_dialog_filtered = brief(await dialogState());

  await keyCode(36); // Enter
  await Bun.sleep(1500);
  report.a_popup_open_after_enter = await popupOpen();

  // ── Phase B: keys dispatched through the NOTES window pipeline ───────
  // Reopen via the same pipeline so the whole phase is independent of
  // which window the OS considers key after Phase A's session resume.
  await notesGpuiKey("p", undefined, ["cmd"]);
  await Bun.sleep(1200);
  report.b_popup_open = await popupOpen();
  report.b_dialog_open = brief(await dialogState());

  // Type "gpui" into the Notes window: must be routed into the popup
  // filter (searchLen rises, filteredCount drops to 1).
  for (const ch of "gpui") await notesGpuiKey(ch, ch);
  await Bun.sleep(600);
  report.b_dialog_filtered = brief(await dialogState());

  // Enter through the Notes window: must select the highlighted entry.
  await notesGpuiKey("enter");
  await Bun.sleep(1500);
  report.b_popup_open_after_enter = await popupOpen();

  const log = await Bun.file(`${driver.sessionDir}/app.log`).text();
  const interesting = [
    "ACTIONS_POPUP_PARENT_ROUTED",
    "notes_agent_chat_history_actions_requested",
    "notes_agent_chat_action_dispatched",
    "actions_dialog_activation",
  ];
  report.log_lines = log
    .split("\n")
    .filter((l) => interesting.some((m) => l.includes(m)))
    .slice(-20);

  report.pass =
    report.a_popup_open === true &&
    report.a_dialog_filtered?.filteredCount === 1 &&
    report.a_popup_open_after_enter === false &&
    report.b_popup_open === true &&
    report.b_dialog_filtered?.filteredCount === 1 &&
    report.b_popup_open_after_enter === false;
} finally {
  try {
    await keyCode(53);
  } catch {}
  await driver.close();
}

console.log(JSON.stringify(report, null, 2));
if (!report.pass) process.exit(1);
