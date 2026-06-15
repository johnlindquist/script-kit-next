/**
 * Runtime proof: Notes Cmd+P and Day Page Cmd+P share the same Notes switcher
 * row language for day files, while selecting a day row from Notes stays in
 * the Notes editor instead of opening the main-window Day Page surface.
 *
 * Contract under test:
 *   1. Cmd+P in Notes opens the shared note switcher.
 *   2. Day files under ~/.scriptkit/brain/days/*.md appear as switcher rows.
 *   3. Selecting a day row from Notes loads that day file into the Notes editor.
 *   4. The main window remains outside promptType "dayPage"; Day Page is only
 *      reached from the main window or explicit actions.
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
  sessionName: "notes-day-page-switcher-shared",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

function gpuiKey(kind: string, key: string, modifiers: string[] = [], text?: string) {
  const event: Json = { type: "keyDown", key, modifiers };
  if (text !== undefined) event.text = text;
  return driver.request(
    { type: "simulateGpuiEvent", target: { type: "kind", kind }, event },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  );
}

async function actionDialogElements() {
  const dialogElements = (await driver.request(
    { type: "getElements", target: { type: "kind", kind: "actionsDialog" } },
    { expect: "elementsResult", timeoutMs: 5000 },
  )) as Json;
  return walkElements(dialogElements);
}

async function notesEditorText(): Promise<string | null> {
  const elements = (await driver.getElements(
    { target: { type: "kind", kind: "notes", index: 0 }, limit: 80 },
    { timeoutMs: 5000 },
  )) as Json;
  const nodes = walkElements(elements);
  const editor = nodes.find((node) => node.semanticId === "input:notes-editor");
  return typeof editor?.value === "string" ? editor.value : null;
}

try {
  const daysDir = join(driver.sessionDir, "home", ".scriptkit", "brain", "days");
  mkdirSync(daysDir, { recursive: true });
  const seededDate = "2026-06-01";
  const seededLine = "switcher probe alpha entry";
  const seededContent = `${seededLine}\nsecond line\n`;
  writeFileSync(join(daysDir, `${seededDate}.md`), seededContent);

  driver.send({ type: "openNotes", requestId: "probe-open-notes" });
  await Bun.sleep(2000);

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

  const initialNodes = await actionDialogElements();
  const seededRow = initialNodes.find((el) =>
    JSON.stringify({
      id: el.semanticId ?? el.id,
      label: el.label ?? el.text,
      value: el.value,
    }).includes(seededDate),
  );
  check("switcher_lists_seeded_day_note", Boolean(seededRow), {
    seededDate,
    sample: initialNodes.slice(0, 25).map((el) => ({
      id: el.semanticId ?? el.id,
      label: el.label ?? el.text,
      value: el.value,
    })),
  });

  for (const ch of seededDate) {
    await gpuiKey("notes", ch, [], ch);
    await Bun.sleep(40);
  }
  await Bun.sleep(400);
  const filteredNodes = await actionDialogElements();
  check(
    "switcher_filter_keeps_seeded_day_note",
    filteredNodes.some((el) =>
      JSON.stringify({
        id: el.semanticId ?? el.id,
        label: el.label ?? el.text,
        value: el.value,
      }).includes(seededDate),
    ),
    {
      seededDate,
      filtered: filteredNodes.slice(0, 20).map((el) => ({
        id: el.semanticId ?? el.id,
        label: el.label ?? el.text,
        value: el.value,
      })),
    },
  );

  await gpuiKey("notes", "enter");
  await Bun.sleep(800);
  const editorText = await notesEditorText();
  check("enter_loads_day_note_in_notes_editor", editorText === seededContent, {
    editorText,
    seededContent,
  });

  const mainState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("main_window_not_day_page", mainState.promptType !== "dayPage", {
    promptType: mainState.promptType,
  });
} finally {
  const failed = checks.filter((c) => !c.pass);
  console.log(JSON.stringify({ pass: failed.length === 0, checks, binary: BINARY }, null, 2));
  await driver.close();
}
