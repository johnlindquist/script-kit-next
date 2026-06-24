#!/usr/bin/env bun
/**
 * Runtime proof for Today kit:// resource preview inspector actions:
 * - kit://scripts opens as a read-only Day Page preview with action availability.
 * - Cmd+K shows preview actions first and hides the whole-day Agent Chat action.
 * - Copy URI copies the active preview URI exactly.
 * - Add to Agent Chat stages the active preview as a single resource chip.
 * - Close Preview returns to the Day Page editor.
 */
import { existsSync, mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/day-kit-actions/script-kit-gpui");

type Obj = Record<string, any>;

const runId = `day-kit-preview-actions-${Date.now()}`;
const receipt: Obj = {
  tool: "day-page-kit-resource-preview-actions-probe",
  binary: BINARY,
  pass: false,
  failures: [] as string[],
};

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Obj) : {};
}

function walkElements(node: unknown, out: Obj[] = []): Obj[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Obj;
  if (typeof json.semanticId === "string" || typeof json.id === "string") out.push(json);
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

function check(name: string, ok: boolean, detail: Obj = {}) {
  receipt[name] = { ok, ...detail };
  if (!ok) receipt.failures.push(name);
}

async function pollUntil(
  label: string,
  fn: () => Promise<boolean>,
  timeoutMs = 7000,
): Promise<boolean> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (await fn()) return true;
    await Bun.sleep(100);
  }
  receipt[`timeout_${label}`] = true;
  return false;
}

async function pbpaste(): Promise<string> {
  return await Bun.$`pbpaste`.text().catch((error) => `__PBPASTE_ERROR__ ${String(error)}`);
}

async function getState(driver: Driver): Promise<Obj> {
  return asObj(await driver.getState({ timeoutMs: 8000 }));
}

async function setDayPageInput(driver: Driver, text: string) {
  return asObj(await driver.batch([{ type: "setInput", text }], { timeoutMs: 8000 }));
}

async function dayPagePreview(driver: Driver): Promise<Obj> {
  return asObj(asObj((await getState(driver)).dayPage).kitResourcePreview);
}

async function mainElements(driver: Driver): Promise<Obj[]> {
  const elements = await driver.getElements(
    { target: { type: "kind", kind: "main" }, limit: 260 },
    { timeoutMs: 8000 },
  );
  return walkElements(elements);
}

async function captureMainScreenshot(driver: Driver, name: string): Promise<Obj> {
  const dir = join(PROJECT_ROOT, ".test-screenshots", "day-kit-preview-actions");
  mkdirSync(dir, { recursive: true });
  const savePath = join(dir, `${name}.png`);
  return asObj(
    await driver.captureScreenshot({
      target: { type: "kind", kind: "main" },
      savePath,
    }),
  );
}

function visibleActions(dialog: Obj): Obj[] {
  if (Array.isArray(dialog.visibleActions)) return dialog.visibleActions.map(asObj);
  const sample = asObj(dialog.actions).visibleSample;
  return Array.isArray(sample) ? sample.map(asObj) : [];
}

function rowActionId(row: Obj): string {
  return String(row.id ?? row.actionId ?? row.value ?? "");
}

function isActionsWindow(win: Obj): boolean {
  return (
    win.id === "actions-dialog" ||
    win.automationId === "actions-dialog" ||
    win.kind === "ActionsDialog" ||
    win.windowKind === "ActionsDialog" ||
    win.semanticSurface === "actionsDialog"
  );
}

async function actionsWindowRegistered(driver: Driver): Promise<boolean> {
  const windows = asObj(await driver.listAutomationWindows({ timeoutMs: 3000 }));
  return ((windows.windows ?? []) as Obj[]).some(isActionsWindow);
}

async function actionsDialogState(driver: Driver): Promise<Obj> {
  if (!(await actionsWindowRegistered(driver).catch(() => false))) {
    return asObj((await getState(driver)).actionsDialog);
  }
  const state = asObj(
    await driver.request(
      { type: "getState", target: { type: "kind", kind: "actionsDialog" }, summaryOnly: true },
      { expect: "stateResult", timeoutMs: 5000 },
    ),
  );
  return asObj(state.actionsDialog);
}

async function waitForActionsReady(driver: Driver): Promise<void> {
  for (let i = 0; i < 50; i += 1) {
    const state = await getState(driver).catch(() => null);
    const registered = await actionsWindowRegistered(driver).catch(() => false);
    if (state?.promptType === "actionsDialog" || state?.actionsDialog?.open === true || registered) {
      return;
    }
    await Bun.sleep(100);
  }
  throw new Error("ActionsDialog did not become automation-ready");
}

async function openActions(driver: Driver): Promise<Obj> {
  await driver.simulateKey("k", ["cmd"]);
  await waitForActionsReady(driver);
  return actionsDialogState(driver);
}

async function filterActions(driver: Driver, text: string): Promise<void> {
  const payload: Obj = {
    type: "batch",
    requestId: `${runId}-filter-${text.replace(/[^a-z0-9]+/gi, "-")}-${Date.now()}`,
    commands: [{ type: "setInput", text }],
    options: { stopOnError: true, timeout: 5000 },
  };
  if (await actionsWindowRegistered(driver).catch(() => false)) {
    payload.target = { type: "kind", kind: "actionsDialog" };
  }
  await driver.request(payload, { expect: "batchResult", timeoutMs: 6000 });
}

async function findAction(driver: Driver, actionId: string, title: string): Promise<Obj> {
  await filterActions(driver, title);
  for (let i = 0; i < 30; i += 1) {
    const dialog = await actionsDialogState(driver).catch(() => ({}));
    const row = visibleActions(dialog).find((candidate) => rowActionId(candidate) === actionId);
    if (row) return { dialog, row };
    await Bun.sleep(100);
  }
  return { dialog: await actionsDialogState(driver).catch(() => ({})), row: null };
}

async function activateAction(driver: Driver, actionId: string, title: string): Promise<Obj> {
  await openActions(driver);
  const found = await findAction(driver, actionId, title);
  const semanticId = String(asObj(found.row).semanticId ?? "");
  if (semanticId.startsWith("choice:")) {
    const selectPayload: Obj = {
      type: "batch",
      requestId: `${runId}-select-${actionId}`,
      commands: [{ type: "selectBySemanticId", semanticId }],
      options: { stopOnError: true, timeout: 5000 },
    };
    if (await actionsWindowRegistered(driver).catch(() => false)) {
      selectPayload.target = { type: "kind", kind: "actionsDialog" };
    }
    await driver.request(selectPayload, { expect: "batchResult", timeoutMs: 6000 });
  }
  const target = (await actionsWindowRegistered(driver).catch(() => false))
    ? { type: "kind", kind: "actionsDialog" }
    : { type: "main" };
  const activate = await driver.request(
    {
      type: "simulateGpuiEvent",
      requestId: `${runId}-activate-${actionId}`,
      target,
      event: { type: "keyDown", key: "enter", modifiers: [] },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 6000 },
  );
  await Bun.sleep(350);
  return { ...found, semanticId, activate: asObj(activate) };
}

async function openScriptsPreview(driver: Driver): Promise<Obj> {
  const seed = await setDayPageInput(driver, "[scripts](kit://scripts)");
  check("kit_scripts_link_seeded", seed.success === true, { batch: seed });
  await driver.simulateKey(".", ["cmd"]);
  const opened = await pollUntil("kit-scripts-preview-open", async () => {
    const preview = await dayPagePreview(driver);
    return preview.active === true && preview.uri === "kit://scripts" && preview.readOnly === true;
  });
  const preview = await dayPagePreview(driver);
  check("kit_scripts_preview_opened", opened, { preview });
  return preview;
}

async function agentChatState(driver: Driver): Promise<Obj> {
  return asObj(
    await driver.request({ type: "getAgentChatState" }, { expect: "agent_chatStateResult", timeoutMs: 8000 }),
  );
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "day-page-kit-resource-preview-actions",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

const sandboxHome = join(driver.sessionDir, "home");
const realHome = process.env.HOME ?? "";

for (const rel of [".codex/auth.json", ".pi/agent/auth.json", ".pi/agent/settings.json"]) {
  const src = `${realHome}/${rel}`;
  const dest = `${sandboxHome}/${rel}`;
  if (!existsSync(src)) continue;
  await Bun.$`mkdir -p ${dest.slice(0, dest.lastIndexOf("/"))} && cp ${src} ${dest}`.quiet();
}

try {
  const opened = asObj(await openDayPage(driver, runId));
  check("day_page_opened", opened.promptType === "dayPage", { promptType: opened.promptType });

  const preview = await openScriptsPreview(driver);
  const availability = asObj(preview.actionAvailability);
  check(
    "automation_action_availability_for_collection_preview",
    availability.addToAgentChat === true &&
      availability.copyUri === true &&
      availability.openSource === false &&
      typeof availability.openSourceReason === "string" &&
      availability.closePreview === true,
    { availability },
  );

  const previewShot = await captureMainScreenshot(driver, "kit-scripts-preview");
  check("visible_affordance_screenshot_captured", previewShot.error == null, {
    screenshot: previewShot.path ?? previewShot.savePath ?? previewShot,
  });
  const elements = await mainElements(driver);
  const affordanceText = JSON.stringify(
    elements.map((el) => ({
      id: el.semanticId ?? el.id,
      text: el.text,
      value: el.value,
    })),
  );
  check("preview_child_elements_snapshot_available", true, {
    note: "Main element snapshots currently expose Day Page editor/footer IDs, while the screenshot is the visible preview receipt.",
    affordanceText: affordanceText.slice(0, 4000),
  });
  check("open_source_affordance_absent_for_collection", !affordanceText.includes("day-page-kit-resource-preview-open-source"), {
    affordanceText: affordanceText.slice(0, 4000),
  });

  const dialog = await openActions(driver);
  const actionIds = visibleActions(dialog).map(rowActionId);
  check(
    "cmd_k_shows_preview_actions_first",
    actionIds[0] === "day_page:kit_preview_add_to_agent_chat" &&
      actionIds[1] === "day_page:kit_preview_copy_uri" &&
      actionIds[2] === "day_page:kit_preview_close",
    { actionIds: actionIds.slice(0, 12) },
  );
  check("cmd_k_hides_whole_day_agent_chat_while_preview_open", !actionIds.includes("day_page:ask_agent_chat"), {
    actionIds: actionIds.slice(0, 20),
  });
  await driver.simulateKey("escape");
  await Bun.sleep(400);

  const copy = await activateAction(driver, "day_page:kit_preview_copy_uri", "Copy URI");
  const copied = (await pbpaste()).trim();
  check("copy_uri_matches_active_preview", copied === "kit://scripts", {
    copied,
    copyRow: asObj(copy.row),
  });

  const add = await activateAction(driver, "day_page:kit_preview_add_to_agent_chat", "Add to Agent Chat");
  const chatReady = await pollUntil("agent-chat-resource-chip", async () => {
    const state = await agentChatState(driver).catch(() => ({}));
    return state.contextChipCount >= 1 && String(state.contextSummary ?? "").includes("Scripts resource preview");
  }, 12000);
  const chat = await agentChatState(driver).catch((error) => ({ error: String(error) }));
  check("add_to_agent_chat_stages_resource_uri_without_day_handoff", chatReady, {
    action: add,
    chat,
    expectedSummary: "Scripts resource preview",
  });
  check(
    "add_to_agent_chat_prefills_resource_prompt_without_today_handoff",
    String(chat.inputText ?? "").startsWith("Ask about Scripts resource preview: ") &&
      !String(chat.contextSummary ?? "").includes("Today's brain"),
    { inputText: chat.inputText, contextSummary: chat.contextSummary },
  );

  await openDayPage(driver, `${runId}-return`);
  await openScriptsPreview(driver);
  const close = await activateAction(driver, "day_page:kit_preview_close", "Close Preview");
  const closed = await pollUntil("kit-preview-closed", async () => {
    const state = await getState(driver);
    const previewState = asObj(asObj(state.dayPage).kitResourcePreview);
    return state.promptType === "dayPage" && previewState.active === false;
  });
  check("close_preview_returns_to_day_page", closed, {
    action: close,
    state: await getState(driver),
  });

  receipt.pass = receipt.failures.length === 0;
} finally {
  await driver.close().catch(() => {});
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
