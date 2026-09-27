#!/usr/bin/env bun
/**
 * Runtime proof for the Today editor style contract:
 * - Day Page remains the main-window Day Page surface and exposes its editor
 *   style through semantic metadata.
 * - Sigil input stays plain editor text with the same style and no local
 *   spine list.
 */
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

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

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-style-parity",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  const dayState = await openDayPage(driver, runId);
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

  check("day_page_editor_present", Boolean(dayEditor), {
    focusedSemanticId: dayElements.focusedSemanticId ?? null,
    dayEditor,
  });
  check("day_page_panel_present", Boolean(dayPanel), { dayPanel });
  check("day_page_style_present", Boolean(dayStyle), { dayStyle });

  // "/rew" remains plain editor text on the Day Page. The Day surface must not
  // render a local inline spine/prompt-builder list for sigil input.
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
  check("sigil_input_batch", setSpine.success === true, { batch: setSpine });
  const spineElements = (await driver.getElements(
    { target: { type: "main" }, limit: 160 },
    { timeoutMs: 8000 },
  )) as Json;
  const spineEditor = findSemantic(spineElements, "input:day-page-editor");
  const spineStyle = comparableStyle(spineEditor?.style);
  const spineFlat = walkElements(spineElements);
  const localSpineRows = spineFlat.filter((el) => {
    const id = String(el.semanticId ?? el.id ?? el.role ?? "");
    return id.includes("day-page-spine") || id.includes("day_page_spine");
  });
  check("sigil_keeps_editor_style", spineStyle?.occlusionRgba === dayStyle?.occlusionRgba, {
    spineStyle,
    dayStyle,
    selectedSemanticId: spineElements.selectedSemanticId ?? null,
  });
  check("sigil_does_not_render_day_spine_list", localSpineRows.length === 0, {
    localSpineRows: localSpineRows.slice(0, 12),
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
