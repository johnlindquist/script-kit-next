#!/usr/bin/env bun
/**
 * Runtime proof for Day Page shared Notes Markdown action parity.
 *
 * Proves Day Page exposes the shared Notes editor Markdown action catalog
 * through the existing main Actions menu and executes representative actions
 * without restoring the deprecated inline @ context popup.
 */

import { existsSync, readFileSync } from "node:fs";
import { mkdir } from "node:fs/promises";
import { dirname, join } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/day-notes-formatting-actions/script-kit-gpui";
const timezone = process.env.SCRIPT_KIT_BRAIN_TZ ?? "America/Denver";
const runId = `formatting-actions-${Date.now().toString(36)}`;

const SHARED_ACTIONS = [
  { id: "bold", title: "Bold", actionId: "day_page:format_bold", shortcut: "cmd+b" },
  { id: "italic", title: "Italic", actionId: "day_page:format_italic", shortcut: "cmd+i" },
  { id: "heading", title: "Heading", actionId: "day_page:format_heading" },
  { id: "list", title: "Bullet List", actionId: "day_page:format_list" },
  {
    id: "numbered-list",
    title: "Numbered List",
    actionId: "day_page:format_numbered_list",
  },
  { id: "code", title: "Inline Code", actionId: "day_page:format_code", shortcut: "cmd+e" },
  { id: "codeblock", title: "Code Block", actionId: "day_page:format_codeblock" },
  {
    id: "strikethrough",
    title: "Strikethrough",
    actionId: "day_page:format_strikethrough",
    shortcut: "cmd+shift+x",
  },
  { id: "checklist", title: "Checklist", actionId: "day_page:format_checklist" },
  { id: "link", title: "Link", actionId: "day_page:format_link" },
  { id: "rule", title: "Horizontal Rule", actionId: "day_page:format_rule" },
  { id: "blockquote", title: "Blockquote", actionId: "day_page:format_blockquote" },
];

const REPRESENTATIVE_ACTIONS = [
  { id: "heading", seed: "alpha", expect: (text: string) => text.startsWith("# alpha") },
  { id: "list", seed: "alpha", expect: (text: string) => text.startsWith("- alpha") },
  { id: "numbered-list", seed: "alpha", expect: (text: string) => text.startsWith("1. alpha") },
  { id: "checklist", seed: "alpha", expect: (text: string) => text.startsWith("- [ ] alpha") },
  { id: "link", seed: "alpha", expect: (text: string) => text.includes("[](url)") },
  { id: "codeblock", seed: "alpha", expect: (text: string) => text.includes("```\n\n```") },
  { id: "rule", seed: "alpha", expect: (text: string) => text.includes("---") },
  { id: "blockquote", seed: "alpha", expect: (text: string) => text.includes("> ") },
  { id: "bold", seed: "alpha", expect: (text: string) => text.includes("**") },
  { id: "italic", seed: "alpha", expect: (text: string) => text.includes("_") },
  { id: "code", seed: "alpha", expect: (text: string) => text.includes("`") },
  { id: "strikethrough", seed: "alpha", expect: (text: string) => text.includes("~~") },
];

const DEPRECATED_CONTEXT_IDS = [
  "day-page-inline-context-popup",
  "day-page-context-popup",
  "inline-context-popup",
];

const checks: Array<{ name: string; ok: boolean; detail: Json }> = [];
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json = {}) {
  checks.push({ name, ok, detail });
  if (!ok) failures.push(name);
}

function localDateSlug(zone: string): string {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone: zone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(new Date());
  const part = (type: string) => parts.find((item) => item.type === type)?.value ?? "";
  return `${part("year")}-${part("month")}-${part("day")}`;
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

function visibleActions(dialog: Json): Json[] {
  const rows = dialog.visibleActions;
  if (Array.isArray(rows)) return rows as Json[];
  const sample = (dialog.actions as Json | undefined)?.visibleSample;
  return Array.isArray(sample) ? (sample as Json[]) : [];
}

function rowActionId(row: Json): string {
  return String(row.id ?? row.actionId ?? row.value ?? "");
}

async function actionsDialogState(driver: Driver): Promise<Json> {
  if (!(await actionsWindowRegistered(driver).catch(() => false))) {
    const state = (await driver.getState({ timeoutMs: 5000 })) as Json;
    return (state.actionsDialog ?? {}) as Json;
  }
  const state = (await driver.request(
    { type: "getState", target: { type: "kind", kind: "actionsDialog" }, summaryOnly: true },
    { expect: "stateResult", timeoutMs: 5000 },
  )) as Json;
  return (state.actionsDialog ?? {}) as Json;
}

function isActionsWindow(win: Json): boolean {
  return (
    win.id === "actions-dialog" ||
    win.automationId === "actions-dialog" ||
    win.kind === "ActionsDialog" ||
    win.windowKind === "ActionsDialog" ||
    win.semanticSurface === "actionsDialog"
  );
}

async function actionsWindowRegistered(driver: Driver): Promise<boolean> {
  const windows = (await driver.listAutomationWindows({ timeoutMs: 3000 })) as Json;
  return ((windows.windows ?? []) as Json[]).some(isActionsWindow);
}

async function waitForActionsReady(driver: Driver): Promise<void> {
  for (let i = 0; i < 50; i += 1) {
    const state = (await driver.getState({ timeoutMs: 1000 }).catch(() => null)) as Json | null;
    const registered = await actionsWindowRegistered(driver).catch(() => false);
    const open = state?.promptType === "actionsDialog" || state?.actionsDialog?.open === true;
    if (open) {
      const target = registered ? { type: "kind", kind: "actionsDialog" } : { type: "main" };
      await driver.getElements({ target, limit: 20 }, { timeoutMs: 1000 }).catch(() => null);
      return;
    }
    await Bun.sleep(100);
  }
  throw new Error("ActionsDialog did not become automation-ready");
}

async function actionsElements(driver: Driver): Promise<Json[]> {
  const target = (await actionsWindowRegistered(driver).catch(() => false))
    ? { type: "kind", kind: "actionsDialog" }
    : { type: "main" };
  const elements = (await driver.getElements(
    { target, limit: 260 },
    { timeoutMs: 5000 },
  )) as Json;
  return walkElements(elements);
}

async function editorText(driver: Driver): Promise<string | null> {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 220 },
    { timeoutMs: 5000 },
  )) as Json;
  const editor = walkElements(elements).find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  return (editor?.value as string | undefined) ?? null;
}

async function setDayText(driver: Driver, text: string): Promise<Json> {
  return (await driver.batch([{ type: "setInput", text }], {
    timeoutMs: 6000,
  })) as Json;
}

async function openActions(driver: Driver) {
  const state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  if (state.promptType === "actionsDialog" && (await actionsWindowRegistered(driver))) {
    return;
  }
  if (state.actionsDialog?.open === true) {
    return;
  }
  if (state.promptType !== "dayPage") {
    await driver.waitForState({ promptType: "dayPage" }, { timeoutMs: 5000 });
  }
  await driver.simulateKey("k", ["cmd"]);
  try {
    await waitForActionsReady(driver);
  } catch (error) {
    await Bun.sleep(700);
    await driver.simulateKey("k", ["cmd"]);
    await waitForActionsReady(driver);
  }
}

async function filterActions(driver: Driver, text: string): Promise<Json> {
  const payload: Json = {
    type: "batch",
    requestId: `${runId}-filter-${text.replace(/[^a-z0-9]+/gi, "-")}-${Date.now()}`,
    commands: [{ type: "setInput", text }],
    options: { stopOnError: true, timeout: 5000 },
  };
  if (await actionsWindowRegistered(driver).catch(() => false)) {
    payload.target = { type: "kind", kind: "actionsDialog" };
  }
  return (await driver.request(payload, { expect: "batchResult", timeoutMs: 6000 })) as Json;
}

async function findActionRow(driver: Driver, action: { actionId: string; title: string }) {
  await waitForActionsReady(driver);
  const filter = await filterActions(driver, action.title);
  let last: Json = { filter, dialog: null, rows: [], row: null, element: null };
  for (let i = 0; i < 30; i += 1) {
    const dialog = await actionsDialogState(driver).catch(() => null);
    const rows = dialog ? visibleActions(dialog) : [];
    const row = rows.find((candidate) => rowActionId(candidate) === action.actionId) ?? null;
    const elements = await actionsElements(driver).catch(() => []);
    const element =
      elements.find((candidate) =>
        String(candidate.semanticId ?? "").endsWith(`:${action.actionId}`),
      ) ?? null;
    last = { filter, dialog, rows, row, element };
    if (row || element) return last;
    await Bun.sleep(100);
  }
  return last;
}

async function waitForActionsClosed(driver: Driver) {
  for (let i = 0; i < 50; i += 1) {
    const state = (await driver.getState({ timeoutMs: 1000 }).catch(() => null)) as Json | null;
    const registered = await actionsWindowRegistered(driver).catch(() => false);
    if (state?.promptType !== "actionsDialog" && state?.actionsDialog?.open !== true && !registered) {
      return;
    }
    await Bun.sleep(100);
  }
}

async function closeActionsIfOpen(driver: Driver) {
  const state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  if (
    state.promptType === "actionsDialog" ||
    state.actionsDialog?.open === true ||
    (await actionsWindowRegistered(driver))
  ) {
    await driver.simulateKey("escape");
    await waitForActionsClosed(driver);
    await Bun.sleep(700);
  }
}

async function activateAction(driver: Driver, action: { actionId: string; title: string }) {
  await openActions(driver);
  const found = await findActionRow(driver, action);
  let select: Json = { skipped: true, reason: "semantic id not exposed" };
  const semanticId = String(found.element?.semanticId ?? found.row?.semanticId ?? "");
  if (semanticId.startsWith("choice:")) {
    const selectPayload: Json = {
      type: "batch",
      requestId: `${runId}-select-${action.actionId}`,
      commands: [{ type: "selectBySemanticId", semanticId }],
      options: { stopOnError: true, timeout: 5000 },
    };
    if (await actionsWindowRegistered(driver).catch(() => false)) {
      selectPayload.target = { type: "kind", kind: "actionsDialog" };
    }
    select = (await driver.request(selectPayload, { expect: "batchResult", timeoutMs: 6000 })) as Json;
  }
  const target = (await actionsWindowRegistered(driver).catch(() => false))
    ? { type: "kind", kind: "actionsDialog" }
    : { type: "main" };
  const activate = (await driver.request(
    {
      type: "simulateGpuiEvent",
      requestId: `${runId}-activate-${action.actionId}`,
      target,
      event: { type: "keyDown", key: "enter", modifiers: [] },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  )) as Json;
  await driver.waitForState({ promptType: "dayPage" }, { timeoutMs: 5000 }).catch(() => null);
  await Bun.sleep(250);
  return { found, semanticId, select, activate };
}

function hasDeprecatedContextPopup(windows: Json, elements: Json[]): boolean {
  const haystack = [
    JSON.stringify(windows),
    ...elements.map((el) => String(el.semanticId ?? el.id ?? el.label ?? "")),
  ].join("\n");
  return DEPRECATED_CONTEXT_IDS.some((id) => haystack.includes(id));
}

await mkdir(".test-output", { recursive: true });

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-notes-formatting-action-parity",
  defaultTimeoutMs: 9000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_BRAIN_TZ: timezone,
    SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN: "1",
  },
});

const dayFile = join(
  driver.sessionDir,
  "home",
  ".scriptkit",
  "brain",
  "days",
  `${localDateSlug(timezone)}.md`,
);

try {
  const initial = await openDayPage(driver, runId);
  check("opened_day_page", initial.promptType === "dayPage", {
    promptType: initial.promptType ?? null,
    windowVisible: initial.windowVisible ?? null,
  });

  const rowReceipts: Json[] = [];
  await openActions(driver);
  for (const action of SHARED_ACTIONS) {
    const found = await findActionRow(driver, action);
    const visibleActionIds = found.rows.map(rowActionId);
    const ok = Boolean(found.row) || Boolean(found.element);
    check(`action_row_visible_${action.id}`, ok, {
      action,
      row: found.row,
      element: found.element,
      visibleActionIds,
      filter: found.filter,
    });
    if (action.shortcut && found.row) {
      check(`action_shortcut_${action.id}`, found.row.shortcut === action.shortcut, {
        expected: action.shortcut,
        actual: found.row.shortcut ?? null,
      });
    }
    rowReceipts.push({
      id: action.id,
      actionId: action.actionId,
      rowVisible: ok,
      semanticId: found.element?.semanticId ?? found.row?.semanticId ?? null,
    });
  }
  await closeActionsIfOpen(driver);

  const activationReceipts: Json[] = [];
  for (const spec of REPRESENTATIVE_ACTIONS) {
    const action = SHARED_ACTIONS.find((item) => item.id === spec.id)!;
    const seed = `${spec.seed}-${runId}-${spec.id}`;
    const set = await setDayText(driver, seed);
    check(`seed_${spec.id}`, set.success === true, { set });
    const before = await editorText(driver);
    const activation = await activateAction(driver, action);
    const after = (await editorText(driver)) ?? "";
    const ok = spec.expect(after) && after !== before;
    check(`action_executes_${spec.id}`, ok, {
      actionId: action.actionId,
      before,
      after,
      activation,
    });
    activationReceipts.push({ id: spec.id, actionId: action.actionId, before, after });
    await closeActionsIfOpen(driver);
  }

  const finalMarker = `day notes formatting parity ${runId}`;
  const finalSet = await setDayText(driver, finalMarker);
  check("final_marker_seeded", finalSet.success === true, { finalSet });
  await driver.simulateKey("s", ["cmd"]);
  let savedText = "";
  for (let i = 0; i < 40; i += 1) {
    await Bun.sleep(150);
    if (existsSync(dayFile)) {
      savedText = readFileSync(dayFile, "utf8");
      if (savedText.includes(finalMarker)) break;
    }
  }
  check("canonical_day_file_saved", savedText.includes(finalMarker), {
    dayFile,
    finalMarker,
    fileExists: existsSync(dayFile),
    savedChars: savedText.length,
  });

  const windows = (await driver.listAutomationWindows()) as Json;
  const mainElements = walkElements(
    (await driver.getElements({ target: { type: "main" }, limit: 260 }, { timeoutMs: 5000 })) as Json,
  );
  check("deprecated_inline_context_popup_absent", !hasDeprecatedContextPopup(windows, mainElements), {
    deprecatedIds: DEPRECATED_CONTEXT_IDS,
    windowCount: ((windows.windows ?? []) as Json[]).length,
  });

  const appLog = existsSync(driver.logPath) ? readFileSync(driver.logPath, "utf8") : "";
  const unknownWarningCount = (appLog.match(/unknown_warning_count=[1-9][0-9]*/g) ?? []).length;
  check("unknown_warning_count_zero", unknownWarningCount === 0, { unknownWarningCount });
  check("no_gpui_entity_double_lease", !appLog.includes("gpui_entity_double_lease"), {
    found: appLog.includes("gpui_entity_double_lease"),
  });
  check("no_runtime_panic", !appLog.includes("PANIC:"), {
    found: appLog.includes("PANIC:"),
  });

  const pass = failures.length === 0;
  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        tool: "day-notes-formatting-action-parity-probe",
        classification: "completed",
        pass,
        failures,
        screenshotProof: "not-used-semantic-devtools-only",
        protectedDirtyFiles: ["dev.sh", "scripts/agentic/ensure-pi-sidecar.sh"],
        sharedActions: SHARED_ACTIONS,
        rowReceipts,
        activationReceipts,
        checks,
        sessionDir: driver.sessionDir,
        appLog: driver.logPath,
        canonicalDayFile: dayFile,
      },
      null,
      2,
    ),
  );
  if (!pass) process.exitCode = 1;
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  check("probe_completed_without_exception", false, { error: message });
  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        tool: "day-notes-formatting-action-parity-probe",
        classification: "failed",
        pass: false,
        failures,
        screenshotProof: "not-used-semantic-devtools-only",
        protectedDirtyFiles: ["dev.sh", "scripts/agentic/ensure-pi-sidecar.sh"],
        checks,
        sessionDir: driver.sessionDir,
        appLog: driver.logPath,
        canonicalDayFile: dayFile,
      },
      null,
      2,
    ),
  );
  process.exitCode = 1;
} finally {
  await driver.close();
}
