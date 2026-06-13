#!/usr/bin/env bun
/**
 * Negative runtime proof for the deleted Day Page inline Spine/Prompt Builder overlay.
 *
 * The main menu still owns Spine/context rows. Day/Today must not render a
 * local absolute overlay, expose Day spine rows through getElements, or show
 * Prompt Builder/Ready to send after editor focus/click-like interaction.
 */
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/day-page-no-spine/script-kit-gpui";

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

const forbiddenText = ["Prompt Builder", "Ready to send"];
const forbiddenIds = ["day-page-spine-list", "day_page_spine_row"];

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
  if (typeof json.semanticId === "string" || typeof json.id === "string") out.push(json);
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

function containsForbidden(value: unknown): string[] {
  const text = JSON.stringify(value);
  const hits: string[] = [];
  for (const forbidden of [...forbiddenText, ...forbiddenIds]) {
    if (text.includes(forbidden)) hits.push(forbidden);
  }
  return hits;
}

function spineRowsInDayElements(elements: Json): Json[] {
  return walkElements(elements).filter((el) => {
    const semanticId = typeof el.semanticId === "string" ? el.semanticId : "";
    const id = typeof el.id === "string" ? el.id : "";
    const role = typeof el.role === "string" ? el.role : "";
    return (
      id.includes("day-page-spine") ||
      semanticId.includes("day-page-spine") ||
      role === "day_page_spine_row"
    );
  });
}

async function setDayPageInput(driver: Driver, text: string, label: string) {
  const batch = (await driver.batch(
    [
      { type: "setInput", text },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: text },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check(`set_${label}`, batch.success === true, { batch });
  await Bun.sleep(100);
}

async function assertNoDayOverlay(driver: Driver, label: string) {
  const state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 260 },
    { timeoutMs: 5000 },
  )) as Json;
  const rows = spineRowsInDayElements(elements);
  const stateHits = containsForbidden(state);
  const elementHits = containsForbidden(elements);
  check(`no_day_spine_rows_${label}`, rows.length === 0, {
    rows: rows.slice(0, 12),
  });
  check(`no_prompt_builder_text_${label}`, stateHits.length === 0 && elementHits.length === 0, {
    stateHits,
    elementHits,
  });
  check(`still_day_page_${label}`, state.promptType === "dayPage", {
    promptType: state.promptType,
    inputValue: state.inputValue,
  });
}

const samples = [
  ["slash_rewrite", "/rew"],
  ["style_professional", ".pro"],
  ["capture_todo", ";to"],
  ["profile", "|"],
  ["cwd", ">d"],
  ["prompt_tail", "/rewrite summarize this folder"],
  ["markdown_link", "[release notes](https://example.com/release-notes)"],
] as const;

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-no-spine-overlay",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  const dayState = await openDayPage(driver, runId);
  check("opened_day_page", dayState.promptType === "dayPage", {
    promptType: dayState.promptType,
  });

  for (const [label, text] of samples) {
    await setDayPageInput(driver, text, label);
    await assertNoDayOverlay(driver, label);
    await driver.simulateKey("enter");
    await Bun.sleep(75);
    await assertNoDayOverlay(driver, `${label}_after_enter`);
  }

} catch (error) {
  check("probe_exception", false, {
    message: error instanceof Error ? error.message : String(error),
    stack: error instanceof Error ? error.stack : null,
  });
} finally {
  await driver.close();
}

const result = {
  tool: "day-page-spine-probe",
  classification: failures.length === 0 ? "completed" : "failed",
  pass: failures.length === 0,
  failures,
  receipts,
};

console.log(JSON.stringify(result, null, 2));
process.exit(failures.length === 0 ? 0 : 1);
