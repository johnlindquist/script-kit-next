/**
 * Runtime proof: day pages are discoverable in the Notes Cmd+P switcher and
 * picking one hands off to the MAIN window's Day Page surface.
 *
 * Contract under test (src/notes/day_page_rows.rs + window/panels.rs +
 * ScriptListApp::open_day_page_in_main_window_hook):
 *   1. Cmd+P in Notes lists `daypage_YYYY-MM-DD` rows in a "Day Pages"
 *      section, read-through from `~/.scriptkit/brain/days/*.md` (sandboxed
 *      here), newest first.
 *   2. Narrowing to a day row and pressing Enter closes the switcher and
 *      opens that exact day — not today — in the main window's Day Page
 *      (promptType "dayPage", editor showing the seeded file content).
 *
 * Protocol-only (simulateGpuiEvent), so the proof runs hidden-window safe.
 * Pass criteria in the printed report: every `checks[*].pass` is true.
 */
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/today/script-kit-gpui";

function walkElements(json: any, out: Json[] = []): Json[] {
  if (!json || typeof json !== "object") return out;
  if (Array.isArray(json)) {
    for (const item of json) walkElements(item, out);
    return out;
  }
  if (typeof json.semanticId === "string" || typeof json.id === "string") {
    out.push(json);
  }
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

const checks: { name: string; pass: boolean; detail?: Json }[] = [];
function check(name: string, pass: boolean, detail?: Json) {
  checks.push({ name, pass, detail });
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "notes-day-page-switcher",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

function gpuiKey(
  kind: string,
  key: string,
  modifiers: string[] = [],
  text?: string,
) {
  const event: Json = { type: "keyDown", key, modifiers };
  if (text !== undefined) event.text = text;
  return driver.request(
    { type: "simulateGpuiEvent", target: { type: "kind", kind }, event },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  );
}

try {
  // Seed two day files in the sandbox brain. A fixed PAST date proves the
  // pick binds that day rather than falling back to today.
  const daysDir = join(driver.sessionDir, "home", ".scriptkit", "brain", "days");
  mkdirSync(daysDir, { recursive: true });
  const seededDate = "2026-06-01";
  const seededLine = "switcher probe alpha entry";
  writeFileSync(join(daysDir, `${seededDate}.md`), `${seededLine}\nsecond line\n`);
  const today = new Date();
  const todayStr = [
    today.getFullYear(),
    String(today.getMonth() + 1).padStart(2, "0"),
    String(today.getDate()).padStart(2, "0"),
  ].join("-");
  writeFileSync(join(daysDir, `${todayStr}.md`), "today seed entry\n");

  driver.send({ type: "openNotes", requestId: "probe-open-notes" });
  await Bun.sleep(2000);

  // --- 1. Cmd+P opens the switcher and lists day page rows ---
  await gpuiKey("notes", "p", ["cmd"]);
  await Bun.sleep(900);

  const windows = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  check(
    "switcher_window_open",
    (windows.windows ?? []).some((w: Json) => w.id === "actions-dialog"),
    { windowIds: (windows.windows ?? []).map((w: Json) => w.id) },
  );

  const dialogElements = (await driver.request(
    { type: "getElements", target: { type: "kind", kind: "actionsDialog" } },
    { expect: "elementsResult", timeoutMs: 5000 },
  )) as Json;
  const nodes = walkElements(dialogElements);
  const dayRows = nodes.filter(
    (el) =>
      String(el.semanticId ?? el.id ?? "").includes("daypage_") ||
      /^\d{4}-\d{2}-\d{2} · |^Today · /.test(String(el.label ?? "")),
  );
  check("switcher_lists_day_page_rows", dayRows.length >= 2, {
    dayRowCount: dayRows.length,
    sample: dayRows.slice(0, 3).map((el) => ({
      id: el.semanticId ?? el.id,
      label: el.label,
    })),
    allNodes: nodes.slice(0, 25).map((el) => ({
      id: el.semanticId ?? el.id,
      label: el.label,
    })),
  });

  // --- 2. Select the seeded PAST day deterministically and accept ---
  // (Fuzzy filter text like "06-01" also subsequence-matches "2026-06-12",
  // so target the row by semantic id instead of racing the ranking.)
  const pastRow = dayRows.find((el) =>
    String(el.semanticId ?? el.id ?? "").includes(`daypage_${seededDate}`),
  );
  const pastRowId = String(pastRow?.semanticId ?? pastRow?.id ?? "");
  check("seeded_past_day_row_present", pastRowId.length > 0, { pastRowId });
  await driver.request(
    {
      type: "batch",
      target: { type: "kind", kind: "actionsDialog" },
      commands: [{ type: "selectBySemanticId", semanticId: pastRowId }],
    },
    { expect: "batchResult", timeoutMs: 5000 },
  );
  await Bun.sleep(300);
  await gpuiKey("actionsDialog", "enter");
  await Bun.sleep(1500);

  const mainState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("main_window_shows_day_page", mainState.promptType === "dayPage", {
    promptType: mainState.promptType,
  });

  const mainElements = (await driver.getElements(
    { target: { type: "main" } },
    { timeoutMs: 5000 },
  )) as Json;
  const editor = walkElements(mainElements).find(
    (el) =>
      el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  const editorValue = String(editor?.value ?? "");
  check("day_page_editor_shows_seeded_day", editorValue.includes(seededLine), {
    editorValuePrefix: editorValue.slice(0, 60),
  });

  const log = await Bun.file(`${driver.sessionDir}/app.log`).text();
  check(
    "handoff_event_logged_opened",
    log.includes("notes_note_switcher_day_page_handoff") &&
      /notes_note_switcher_day_page_handoff.*opened=true/.test(log),
    {
      browsePanelLogLines: log
        .split("\n")
        .filter((line) => /open_browse_panel|day_page_handoff/i.test(line))
        .slice(-5),
    },
  );
} finally {
  const failed = checks.filter((c) => !c.pass);
  console.log(
    JSON.stringify(
      { pass: failed.length === 0, checks, binary: BINARY },
      null,
      2,
    ),
  );
  await driver.close();
}
