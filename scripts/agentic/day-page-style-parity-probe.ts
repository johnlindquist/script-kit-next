#!/usr/bin/env bun
/**
 * Runtime proof for Today editor style parity with Notes.
 *
 * Proves the editor-core contract, not window-shell parity:
 * - Notes and Day Page expose the same shared NotesEditor style owner/path.
 * - Padding/font/theme surface tokens match through semantic metadata.
 * - Day Page remains the main-window Day Page surface and does not inherit
 *   Notes window titlebar/footer semantics.
 */
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/today-style-parity/script-kit-gpui";

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") {
    out.push(json);
  }
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

async function simulateMainHotkeyGesture(
  driver: Driver,
  phase: "down" | "up",
  requestId: string,
) {
  return driver.request(
    { type: "simulateMainHotkeyGesture", phase, requestId },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  );
}

async function tapHotkey(driver: Driver, label: string) {
  await simulateMainHotkeyGesture(driver, "down", `${runId}-${label}-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${runId}-${label}-up`);
  await Bun.sleep(420);
}

function findSemantic(elements: Json, semanticId: string): Json | null {
  return walkElements(elements).find((el) => el.semanticId === semanticId) ?? null;
}

function comparableStyle(style: unknown): Json | null {
  if (!style || typeof style !== "object") return null;
  const raw = style as Json;
  return {
    owner: raw.owner ?? null,
    inputRenderPath: raw.inputRenderPath ?? null,
    surfaceBackgroundRgb: raw.surfaceBackgroundRgb ?? null,
    occlusionRgba: raw.occlusionRgba ?? null,
    paddingX: raw.paddingX ?? null,
    paddingY: raw.paddingY ?? null,
    fontFamilySource: raw.fontFamilySource ?? null,
    textSizeSource: raw.textSizeSource ?? null,
  };
}

function assertStyleEqual(notesStyle: Json | null, dayStyle: Json | null) {
  const fields = [
    "owner",
    "inputRenderPath",
    "surfaceBackgroundRgb",
    "occlusionRgba",
    "paddingX",
    "paddingY",
    "fontFamilySource",
    "textSizeSource",
  ];
  for (const field of fields) {
    check(`style_${field}_matches`, notesStyle?.[field] === dayStyle?.[field], {
      notes: notesStyle?.[field] ?? null,
      dayPage: dayStyle?.[field] ?? null,
    });
  }
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-style-parity",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  driver.send({ type: "openNotes", requestId: `${runId}-open-notes` });
  await Bun.sleep(600);

  const notesElements = (await driver.getElements(
    { target: { type: "kind", kind: "notes", index: 0 }, limit: 80 },
    { timeoutMs: 8000 },
  )) as Json;
  const notesEditor = findSemantic(notesElements, "input:notes-editor");
  const notesPanel = findSemantic(notesElements, "panel:notes-window");
  const notesStyle = comparableStyle(notesEditor?.style);

  check("notes_editor_present", Boolean(notesEditor), {
    focusedSemanticId: notesElements.focusedSemanticId ?? null,
    notesEditor,
  });
  check("notes_panel_present", Boolean(notesPanel), { notesPanel });
  check("notes_style_present", Boolean(notesStyle), { notesStyle });

  await tapHotkey(driver, "show-launcher");
  await driver.waitForState({ windowVisible: true, promptType: "none" }, { timeoutMs: 8000 });
  await tapHotkey(driver, "open-day-page");
  const dayState = (await driver.getState({ timeoutMs: 8000 })) as Json;
  check("day_page_stays_main_surface", dayState.promptType === "dayPage", {
    promptType: dayState.promptType ?? null,
    windowVisible: dayState.windowVisible ?? null,
  });

  const dayElements = (await driver.getElements(
    { target: { type: "main" }, limit: 160 },
    { timeoutMs: 8000 },
  )) as Json;
  const dayEditor = findSemantic(dayElements, "input:day-page-editor");
  const dayPanel = findSemantic(dayElements, "panel:day-page");
  const dayStyle = comparableStyle(dayEditor?.style);
  const dayFlat = walkElements(dayElements);

  check("day_page_editor_present", Boolean(dayEditor), {
    focusedSemanticId: dayElements.focusedSemanticId ?? null,
    dayEditor,
  });
  check("day_page_panel_present", Boolean(dayPanel), { dayPanel });
  check("day_page_style_present", Boolean(dayStyle), { dayStyle });
  assertStyleEqual(notesStyle, dayStyle);

  check(
    "day_page_does_not_import_notes_chrome",
    !dayFlat.some((el) =>
      ["notes-titlebar", "notes-footer", "panel:notes-window", "input:notes-editor"].includes(
        String(el.semanticId ?? el.id ?? ""),
      ),
    ),
    { semanticIds: dayFlat.map((el) => el.semanticId ?? el.id).filter(Boolean).slice(0, 80) },
  );

  // "/rew" keeps the inline spine list on the Day Page; `@` fragments now
  // swap to the main menu (round-trip contract) so they can't style-probe.
  const setSpine = (await driver.batch(
    [
      { type: "setInput", text: "/rew" },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: "/rew" },
        },
      },
    ],
    { timeoutMs: 8000 },
  )) as Json;
  check("spine_input_batch", setSpine.success === true, { batch: setSpine });
  const spineElements = (await driver.getElements(
    { target: { type: "main" }, limit: 160 },
    { timeoutMs: 8000 },
  )) as Json;
  const spineEditor = findSemantic(spineElements, "input:day-page-editor");
  const spineStyle = comparableStyle(spineEditor?.style);
  check("spine_keeps_editor_style", spineStyle?.occlusionRgba === dayStyle?.occlusionRgba, {
    spineStyle,
    dayStyle,
    selectedSemanticId: spineElements.selectedSemanticId ?? null,
  });

  const pass = failures.length === 0;
  console.log(
    JSON.stringify(
      {
        pass,
        failures,
        sessionDir: driver.sessionDir,
        receipts,
      },
      null,
      2,
    ),
  );
  if (!pass) process.exitCode = 1;
} finally {
  await driver.close();
}
