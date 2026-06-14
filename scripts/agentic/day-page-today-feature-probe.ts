#!/usr/bin/env bun
/**
 * Runtime proof for the Today feature contract additions:
 * - Notes-parity autosave: typed text lands on disk without Cmd+S.
 * - Today contextual Cmd+K actions (day_page:* rows) render AND execute.
 * - Cmd+P past-day switcher: lists seeded days, filters, swaps, returns.
 * - Markdown formatting shortcuts (Cmd+B) shared with Notes.
 */
import { join } from "node:path";
import { mkdirSync, writeFileSync, readFileSync, readdirSync, existsSync } from "node:fs";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ?? "target-agent/artifacts/today/script-kit-gpui";

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

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-today-feature",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

const sandboxHome = join(driver.sessionDir, "home");
const daysDir = join(sandboxHome, ".scriptkit", "brain", "days");
const realHome = process.env.HOME ?? "";

// The handoff action submits into a live Agent Chat thread. Seed only the
// small auth files the sandbox needs; never copy all of ~/.codex.
for (const rel of [
  ".codex/auth.json",
  ".pi/agent/auth.json",
  ".pi/agent/settings.json",
]) {
  const src = `${realHome}/${rel}`;
  const dest = `${sandboxHome}/${rel}`;
  try {
    await Bun.$`mkdir -p ${dest.slice(0, dest.lastIndexOf("/"))} && cp ${src} ${dest}`
      .quiet();
  } catch {
    // Missing auth file is reported by the handoff checks if Agent Chat opens in setup mode.
  }
}

async function mainElements(limit = 240): Promise<Json[]> {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit },
    { timeoutMs: 5000 },
  )) as Json;
  return walkElements(elements);
}

async function editorText(): Promise<string | null> {
  const flat = await mainElements();
  const editor = flat.find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  return (editor?.value as string | undefined) ?? null;
}

async function setDayPageInput(text: string, label: string) {
  const batch = (await driver.batch(
    [
      { type: "setInput", text },
      {
        type: "waitFor",
        condition: { type: "stateMatch", state: { promptType: "dayPage", inputValue: text } },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check(`batch_set_${label}`, batch.success === true, { batch });
}

function todayLocalDate(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

try {
  // --- Enter Day Page through the real gesture path ---
  let state = await openDayPage(driver, runId);
  check("opened_day_page", state.promptType === "dayPage", { promptType: state.promptType });

  // --- Autosave proof: type, wait past the debounce, read disk ---
  const autosaveText = "autosave proof line";
  await setDayPageInput(autosaveText, "autosave_text");
  await Bun.sleep(1200);
  const todayFile = join(daysDir, `${todayLocalDate()}.md`);
  const diskContent = existsSync(todayFile) ? readFileSync(todayFile, "utf8") : null;
  check("autosave_persists_to_disk_without_cmd_s", diskContent === autosaveText, {
    todayFile,
    diskContent,
    daysDirListing: existsSync(daysDir) ? readdirSync(daysDir) : null,
  });

  // --- Markdown formatting parity: Cmd+B inserts ** ** at cursor ---
  await driver.simulateKey("b", ["cmd"]);
  await Bun.sleep(250);
  const afterBold = await editorText();
  check("cmd_b_inserts_bold_markers", (afterBold ?? "").includes("****"), { afterBold });
  await setDayPageInput(autosaveText, "reset_after_bold");

  // --- Today contextual actions: Cmd+K shows day_page rows ---
  await driver.simulateKey("k", ["cmd"]);
  await Bun.sleep(900);
  const dialogState = (await driver.request(
    { type: "getState", target: { type: "kind", kind: "actionsDialog" } },
    { expect: "stateResult", timeoutMs: 4000 },
  )) as Json;
  const dialog = (dialogState.actionsDialog ?? null) as Json | null;
  check("actions_dialog_opened_on_day_page", dialog !== null, {
    hasDialog: dialog !== null,
  });
  const dialogElements = (await driver.getElements(
    { target: { type: "kind", kind: "actionsDialog" }, limit: 200 },
    { timeoutMs: 5000 },
  )) as Json;
  const dialogFlat = walkElements(dialogElements);
  const dialogIds = dialogFlat
    .map((el) => `${el.semanticId ?? el.id ?? ""}|${el.text ?? ""}|${el.value ?? ""}`)
    .slice(0, 60);
  const hasOpenPastDay = dialogFlat.some((el) =>
    [el.semanticId, el.id, el.text, el.value].some(
      (v) => typeof v === "string" && (v.includes("Open Past Day") || v.includes("day_page:open_past_day")),
    ),
  );
  const hasSaveToday = dialogFlat.some((el) =>
    [el.semanticId, el.id, el.text, el.value].some(
      (v) => typeof v === "string" && (v.includes("Save Today") || v.includes("day_page:save")),
    ),
  );
  const hasDeprecatedAgentRow = dialogFlat.some((el) =>
    [el.semanticId, el.id, el.text, el.value].some(
      (v) => typeof v === "string" && /day_page:.*handoff|agent chat/i.test(v),
    ),
  );
  const hasPromptHandoffRows = dialogFlat.some((el) =>
    [el.semanticId, el.id, el.text, el.value].some(
      (v) =>
        typeof v === "string" &&
        (v.includes("prompt-action/") ||
          v.includes("prompt-target/") ||
          v.includes("Export Prompt") ||
          v.includes("Send Prompt")),
    ),
  );
  check("today_actions_rows_visible", hasOpenPastDay && hasSaveToday && !hasDeprecatedAgentRow && !hasPromptHandoffRows, {
    dialogIds,
    hasOpenPastDay,
    hasSaveToday,
    hasDeprecatedAgentRow,
    hasPromptHandoffRows,
  });

  await driver.simulateKey("escape");
  await Bun.sleep(500);

  // --- Past-day switcher: seed a past day, Cmd+P, filter, swap ---
  mkdirSync(daysDir, { recursive: true });
  const pastDate = "2026-06-01";
  const pastContent = "past day seeded content";
  writeFileSync(join(daysDir, `${pastDate}.md`), pastContent);

  await driver.simulateKey("p", ["cmd"]);
  await Bun.sleep(500);
  const switcherFlat = await mainElements();
  const switcherList = switcherFlat.find(
    (el) => typeof el.semanticId === "string" && el.semanticId.startsWith("list:day-page-day-switch"),
  );
  const pastRow = switcherFlat.find((el) => el.semanticId === `day-switcher-${pastDate}`);
  check("cmd_p_opens_day_switcher", Boolean(switcherList), {
    ids: switcherFlat.slice(0, 24).map((el) => el.semanticId ?? el.id),
  });
  check("switcher_lists_seeded_past_day", Boolean(pastRow), { pastRow: pastRow ?? null });

  // Filter typing narrows to the past day, then Enter swaps.
  for (const ch of "06-01") {
    await driver.simulateKey(ch === "-" ? "-" : ch);
    await Bun.sleep(60);
  }
  await Bun.sleep(300);
  const filteredFlat = await mainElements();
  const filteredRows = filteredFlat.filter(
    (el) => typeof el.semanticId === "string" && el.semanticId.startsWith("day-switcher-"),
  );
  check(
    "switcher_query_filters_rows",
    filteredRows.length === 1 && filteredRows[0]?.semanticId === `day-switcher-${pastDate}`,
    { rows: filteredRows.map((el) => el.semanticId) },
  );

  await driver.simulateKey("enter");
  await Bun.sleep(500);
  const pastEditor = await editorText();
  check("enter_swaps_to_past_day", pastEditor === pastContent, { pastEditor });

  // Escape returns to today (with today's autosaved content).
  await driver.simulateKey("escape");
  await Bun.sleep(500);
  const backEditor = await editorText();
  const backState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "escape_returns_to_today",
    backEditor === autosaveText && backState.promptType === "dayPage",
    { backEditor, promptType: backState.promptType },
  );

  // --- Execute a Today action end-to-end: Open Past Day… via Cmd+K row ---
  await driver.simulateKey("k", ["cmd"]);
  await Bun.sleep(900);
  // Filter the dialog search to the row, then Enter executes it.
  for (const ch of "open past") {
    await driver.simulateKey(ch === " " ? "space" : ch);
    await Bun.sleep(40);
  }
  await Bun.sleep(400);
  await driver.simulateKey("enter");
  await Bun.sleep(700);
  const afterActionFlat = await mainElements();
  const switcherViaAction = afterActionFlat.find(
    (el) => typeof el.semanticId === "string" && el.semanticId.startsWith("list:day-page-day-switch"),
  );
  check("cmdk_open_past_day_action_executes", Boolean(switcherViaAction), {
    ids: afterActionFlat.slice(0, 24).map((el) => el.semanticId ?? el.id),
  });
  await driver.simulateKey("escape");
  await Bun.sleep(300);

  // --- External refresh: a clean editor picks up on-disk changes ---
  // Settle to a known clean state first (autosave flush), then mutate the
  // day file externally and nudge a render via a benign key press.
  const externalSeed = "external base line";
  await setDayPageInput(externalSeed, "external_base");
  await Bun.sleep(1200);
  const externalText = "external base line\nappended outside the app";
  writeFileSync(todayFile, externalText);
  await driver.simulateKey("right");
  await Bun.sleep(600);
  const refreshedEditor = await editorText();
  check("external_disk_change_refreshes_editor", refreshedEditor === externalText, {
    refreshedEditor,
    externalText,
  });

  const pass = failures.length === 0;
  console.log(
    JSON.stringify({ pass, failures, sessionDir: driver.sessionDir, receipts }, null, 2),
  );
  if (!pass) process.exitCode = 1;
} finally {
  await driver.close();
}
