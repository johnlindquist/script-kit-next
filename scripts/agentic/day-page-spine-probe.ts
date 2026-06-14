#!/usr/bin/env bun
/**
 * Negative runtime proof for the deleted Day Page inline Spine overlay.
 *
 * The main menu still owns Spine/context rows. Day/Today must not render a
 * local absolute overlay, expose Day spine rows through getElements, or show
 * stale assistant-panel affordances after editor focus/click-like interaction.
 */
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";
import { mkdirSync, writeFileSync } from "node:fs";

const BINARY = process.env.PROBE_BINARY ?? process.env.SCRIPT_KIT_GPUI_BINARY;
if (!BINARY) {
  throw new Error(
    "day-page-spine-probe requires PROBE_BINARY or SCRIPT_KIT_GPUI_BINARY so it cannot run a stale artifact",
  );
}

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;
const OUT_PATH = ".test-output/day-page-no-spine-probe.json";

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

function spineRowsInDayElements(elements: Json): Json[] {
  return walkElements(elements).filter((el) => {
    const semanticId = typeof el.semanticId === "string" ? el.semanticId : "";
    const id = typeof el.id === "string" ? el.id : "";
    const role = typeof el.role === "string" ? el.role : "";
    const haystack = `${semanticId} ${id} ${role}`.toLowerCase();
    return (
      haystack.includes("day") &&
      haystack.includes("spine") &&
      !haystack.includes("handoff")
    );
  });
}

function promptBuilderTextInElements(elements: Json): Json[] {
  return walkElements(elements).filter((el) => {
    const text = typeof el.text === "string" ? el.text : "";
    const title = typeof el.title === "string" ? el.title : "";
    const value = typeof el.value === "string" ? el.value : "";
    const label = typeof el.label === "string" ? el.label : "";
    const haystack = `${text} ${title} ${value} ${label}`.toLowerCase();
    return haystack.includes("prompt builder") || haystack.includes("ready to send");
  });
}

function forbiddenPopupWindows(windowsResult: Json): Json[] {
  const windows = Array.isArray(windowsResult.windows) ? (windowsResult.windows as Json[]) : [];
  return windows.filter((entry) => {
    const id = typeof entry.id === "string" ? entry.id : "";
    const kind = typeof entry.kind === "string" ? entry.kind : "";
    const title = typeof entry.title === "string" ? entry.title : "";
    const semanticSurface =
      typeof entry.semanticSurface === "string" ? entry.semanticSurface : "";
    const haystack = `${id} ${kind} ${title} ${semanticSurface}`.toLowerCase();
    return (
      haystack.includes("inline-agent") ||
      haystack.includes("inlineagent") ||
      haystack.includes("prompt builder") ||
      haystack.includes("ready to send") ||
      kind.toLowerCase() === "miniai"
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
  const windows = (await driver.listAutomationWindows({ timeoutMs: 5000 })) as Json;
  const rows = spineRowsInDayElements(elements);
  const promptBuilderText = promptBuilderTextInElements(elements);
  const forbiddenWindows = forbiddenPopupWindows(windows);
  check(`no_day_spine_rows_${label}`, rows.length === 0, {
    rows: rows.slice(0, 12),
  });
  check(`no_prompt_builder_text_${label}`, promptBuilderText.length === 0, {
    promptBuilderText: promptBuilderText.slice(0, 12),
  });
  check(`no_forbidden_popup_windows_${label}`, forbiddenWindows.length === 0, {
    forbiddenWindows,
    windows: Array.isArray(windows.windows) ? windows.windows : [],
  });
  check(`still_day_page_${label}`, state.promptType === "dayPage", {
    promptType: state.promptType,
    inputValue: state.inputValue,
  });
  const activeFooter = (state.activeFooter ?? {}) as Json;
  const footerButtons = Array.isArray(activeFooter.buttons)
    ? (activeFooter.buttons as Json[])
    : [];
  const agentButtons = footerButtons.filter((button) => {
    const action = typeof button.action === "string" ? button.action.toLowerCase() : "";
    const label = typeof button.label === "string" ? button.label.toLowerCase() : "";
    return action === "ai" || label === "agent";
  });
  check(`no_day_agent_footer_button_${label}`, agentButtons.length === 0, {
    activeFooter,
    agentButtons,
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

const contextSamples = [["context_file", "@file"]] as const;

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

  for (const [label, text] of contextSamples) {
    await setDayPageInput(driver, text, label);
    const windows = (await driver.listAutomationWindows({ timeoutMs: 5000 })) as Json;
    const forbiddenWindows = forbiddenPopupWindows(windows);
    check(`no_forbidden_popup_windows_${label}_context_round_trip`, forbiddenWindows.length === 0, {
      forbiddenWindows,
      windows: Array.isArray(windows.windows) ? windows.windows : [],
    });
    const elements = (await driver.getElements(
      { target: { type: "main" }, limit: 300 },
      { timeoutMs: 5000 },
    )) as Json;
    const promptBuilderText = promptBuilderTextInElements(elements);
    check(`no_prompt_builder_text_${label}_context_round_trip`, promptBuilderText.length === 0, {
      promptBuilderText: promptBuilderText.slice(0, 12),
    });
  }

  await driver.batch([{ type: "setInput", text: "" }], { timeoutMs: 5000 });
  await Bun.sleep(100);
  driver.simulateKey("@");
  await Bun.sleep(50);
  driver.simulateKey("f");
  await Bun.sleep(250);
  const typedContextState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const typedContextWindows = (await driver.listAutomationWindows({ timeoutMs: 5000 })) as Json;
  const typedForbiddenWindows = forbiddenPopupWindows(typedContextWindows);
  check("typed_context_no_forbidden_popup_windows", typedForbiddenWindows.length === 0, {
    promptType: typedContextState.promptType,
    inputValue: typedContextState.inputValue,
    forbiddenWindows: typedForbiddenWindows,
    windows: Array.isArray(typedContextWindows.windows) ? typedContextWindows.windows : [],
  });
  const typedContextElements = (await driver.getElements(
    { target: { type: "main" }, limit: 300 },
    { timeoutMs: 5000 },
  )) as Json;
  const typedPromptBuilderText = promptBuilderTextInElements(typedContextElements);
  check("typed_context_no_prompt_builder_text", typedPromptBuilderText.length === 0, {
    promptType: typedContextState.promptType,
    inputValue: typedContextState.inputValue,
    promptBuilderText: typedPromptBuilderText.slice(0, 12),
  });

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

mkdirSync(".test-output", { recursive: true });
writeFileSync(OUT_PATH, JSON.stringify(result, null, 2));
console.log(JSON.stringify(result, null, 2));
process.exit(failures.length === 0 ? 0 : 1);
