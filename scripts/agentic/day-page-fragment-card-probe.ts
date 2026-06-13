#!/usr/bin/env bun
/**
 * Runtime proof: long clipboard re-copy creates a markdown-backed Today
 * fragment card, opens the fragment inline, and returns to the day page.
 *
 *   PROBE_BINARY=target-agent/artifacts/today-fragment-card/script-kit-gpui \
 *     bun scripts/agentic/day-page-fragment-card-probe.ts
 */

import { existsSync, readFileSync } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/today-fragment-card/script-kit-gpui";

const runId = `fragment-card-${Date.now().toString(36)}`;
const EXCERPT_TOKEN = `EXCERPT-${runId}`;
const FULL_TOKEN = `FULL-PAYLOAD-${runId}`;
const PRIVACY_SEPARATOR = `separator-${runId}`;
const FRAGMENT_CARD_ID = "day-page-fragment-card-0";
const SEDIMENT_LAYER_ID = "day-page-sediment-layer";
const FRAGMENT_BACK_ID = "day-page-fragment-back";
const DEPRECATED_CONTEXT_IDS = [
  "day-page-inline-context-popup",
  "day-page-context-popup",
  "inline-context-popup",
  "context-picker",
];

const receipts: Record<string, Json> = {};
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function todayLocalDate() {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

function longPayload() {
  const words = [
    "clipboard",
    "fragment",
    "proof",
    EXCERPT_TOKEN,
    "keeps",
    "the",
    "beginning",
    "visible",
    "inside",
    "the",
    "day",
    "page",
    "card",
  ];
  for (let index = 0; index < 260; index += 1) {
    words.push(`bodyword${index}`);
  }
  words.push(FULL_TOKEN);
  return words.join(" ");
}

function countOccurrences(haystack: string, needle: string) {
  if (!needle) return 0;
  return haystack.split(needle).length - 1;
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

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 10_000,
  intervalMs = 150,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  let last: T | undefined;
  while (Date.now() < deadline) {
    last = await read();
    if (accept(last)) return last;
    await Bun.sleep(intervalMs);
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

async function copyText(text: string) {
  await Bun.$`printf ${text} | pbcopy`.quiet();
}

async function readMarkdownFiles(dir: string) {
  const names = await readdir(dir).catch(() => [] as string[]);
  const files: Array<{ path: string; content: string }> = [];
  for (const name of names.filter((name) => name.endsWith(".md"))) {
    const path = join(dir, name);
    files.push({ path, content: await readFile(path, "utf8") });
  }
  return files;
}

async function editorText(driver: Driver): Promise<string | null> {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 240 },
    { timeoutMs: 5000 },
  )) as Json;
  const editor = walkElements(elements).find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  return (editor?.value as string | undefined) ?? null;
}

async function mainElements(driver: Driver) {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 260 },
    { timeoutMs: 5000 },
  )) as Json;
  return { raw: elements, flat: walkElements(elements) };
}

async function actionsDialogState(driver: Driver): Promise<Json> {
  const state = (await driver.request(
    { type: "getState", target: { type: "kind", kind: "actionsDialog" }, summaryOnly: true },
    { expect: "stateResult", timeoutMs: 5000 },
  )) as Json;
  return (state.actionsDialog ?? {}) as Json;
}

function visibleActions(dialog: Json): Json[] {
  const rows = dialog.visibleActions;
  if (Array.isArray(rows)) return rows as Json[];
  const sample = (dialog.actions as Json | undefined)?.visibleSample;
  return Array.isArray(sample) ? (sample as Json[]) : [];
}

function actionRowId(row: Json) {
  return row.actionId ?? row.id ?? row.value ?? row.semanticId;
}

async function mainWindowBounds(driver: Driver): Promise<Json | null> {
  const windows = (await driver.listAutomationWindows({ timeoutMs: 5000 })) as Json;
  const list = (windows.windows ?? []) as Json[];
  return (list.find((w) => w.id === "main" || w.automationId === "main")?.bounds ?? null) as
    | Json
    | null;
}

async function gpuiEvent(driver: Driver, event: Json) {
  return (await driver.request(
    { type: "simulateGpuiEvent", target: { type: "kind", kind: "main" }, event },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  )) as Json;
}

async function clickFragmentCard(driver: Driver) {
  const select = await driver
    .request(
      {
        type: "batch",
        target: { type: "main" },
        commands: [{ type: "selectBySemanticId", semanticId: FRAGMENT_CARD_ID, submit: true }],
        options: { stopOnError: true, timeout: 5000 },
      },
      { expect: "batchResult", timeoutMs: 6000 },
    )
    .catch((error) => ({ error: String(error) })) as Json;
  await Bun.sleep(700);
  let afterSelect = (await editorText(driver)) ?? "";
  if (afterSelect.includes(FULL_TOKEN)) {
    return { method: "selectBySemanticId", select };
  }

  const bounds = await mainWindowBounds(driver);
  const click = {
    x: Math.round(Number(bounds?.width ?? 750) / 2),
    y: 78,
  };
  const down = await gpuiEvent(driver, { type: "mouseDown", ...click });
  const up = await gpuiEvent(driver, { type: "mouseUp", ...click });
  await Bun.sleep(900);
  afterSelect = (await editorText(driver)) ?? "";
  return { method: "coordinate", select, click, down, up, openedAfterClick: afterSelect.includes(FULL_TOKEN) };
}

function hasDeprecatedContextPopup(windows: Json, elements: Json[]) {
  const automationWindows = ((windows.windows ?? []) as Json[]).map((w) =>
    [w.automationId, w.semanticSurface, w.windowKind, w.title].filter(Boolean).join("|"),
  );
  const elementIds = elements.map((el) =>
    [el.semanticId, el.id, el.text, el.value].filter(Boolean).join("|"),
  );
  const combined = [...automationWindows, ...elementIds];
  return combined.some((value) =>
    DEPRECATED_CONTEXT_IDS.some((needle) => value.includes(needle)),
  );
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "day-page-fragment-card",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_BRAIN_TZ: process.env.SCRIPT_KIT_BRAIN_TZ ?? "America/Denver",
  },
});

const sandboxHome = driver.sandboxHome ?? `${driver.sessionDir}/home`;
const skPath = join(sandboxHome, ".scriptkit");
const todayFile = join(skPath, "brain", "days", `${todayLocalDate()}.md`);
const fragmentsDir = join(skPath, "brain", "fragments");
const payload = longPayload();
const protectedBefore = {
  dev: Bun.spawnSync(["git", "diff", "--", "dev.sh"], {
    cwd: process.cwd(),
    stdout: "pipe",
  }).stdout.toString(),
  pi: Bun.spawnSync(["git", "diff", "--", "scripts/agentic/ensure-pi-sidecar.sh"], {
    cwd: process.cwd(),
    stdout: "pipe",
  }).stdout.toString(),
};

try {
  const opened = await openDayPage(driver, runId);
  check("opened_day_page", opened.promptType === "dayPage", {
    promptType: opened.promptType,
    windowVisible: opened.windowVisible,
  });

  await copyText(payload);
  await Bun.sleep(500);
  await copyText(PRIVACY_SEPARATOR);
  await Bun.sleep(700);
  await copyText(payload);

  const fragmentFiles = await waitFor(
    "long recopy fragment file",
    () => readMarkdownFiles(fragmentsDir),
    (files) => files.some((file) => file.content.includes(FULL_TOKEN)),
    12_000,
  );
  const matchingFragments = fragmentFiles.filter((file) => file.content.includes(FULL_TOKEN));
  const fragmentContainsFullPayload = matchingFragments.some((file) => file.content.includes(payload));
  const fragmentContainsSourceFrontmatter = matchingFragments.some((file) =>
    file.content.includes("source: scriptkit://clipboard/"),
  );
  check(
    "long_recopy_created_fragment",
    matchingFragments.length > 0 && fragmentContainsFullPayload && fragmentContainsSourceFrontmatter,
    {
      fragmentFiles: matchingFragments.map((file) => file.path),
      fragmentContainsFullPayload,
      fragmentContainsSourceFrontmatter,
    },
  );

  const dayContent = await waitFor(
    "day fragment reference",
    () => (existsSync(todayFile) ? readFileSync(todayFile, "utf8") : ""),
    (content) => content.includes("../fragments/") && content.includes(EXCERPT_TOKEN),
    12_000,
  );
  check(
    "day_page_contains_fragment_reference_card_markdown",
    dayContent.includes("../fragments/") &&
      dayContent.includes(EXCERPT_TOKEN) &&
      !dayContent.includes(FULL_TOKEN) &&
      !dayContent.includes(payload),
    {
      containsFragmentLink: dayContent.includes("../fragments/"),
      containsExcerpt: dayContent.includes(EXCERPT_TOKEN),
      fullPayloadOccurrencesInDayFile: countOccurrences(dayContent, payload),
      fullTokenOccurrencesInDayFile: countOccurrences(dayContent, FULL_TOKEN),
    },
  );

  const visible = await waitFor(
    "fragment card visible",
    () => mainElements(driver),
    ({ flat }) =>
      flat.some((el) => el.semanticId === SEDIMENT_LAYER_ID || el.id === SEDIMENT_LAYER_ID) &&
      flat.some((el) => el.semanticId === FRAGMENT_CARD_ID || el.id === FRAGMENT_CARD_ID),
    10_000,
  );
  const card = visible.flat.find(
    (el) => el.semanticId === FRAGMENT_CARD_ID || el.id === FRAGMENT_CARD_ID,
  );
  const layer = visible.flat.find(
    (el) => el.semanticId === SEDIMENT_LAYER_ID || el.id === SEDIMENT_LAYER_ID,
  );
  const combinedVisibleText = visible.flat
    .map((el) => [el.text, el.value].filter(Boolean).join(" "))
    .join(" ");
  check("fragment_card_visible", Boolean(card && layer), {
    semanticId: FRAGMENT_CARD_ID,
    sedimentLayerVisible: Boolean(layer),
    excerptVisible: combinedVisibleText.includes(EXCERPT_TOKEN),
    provenanceVisible: combinedVisibleText.includes("Clipboard"),
    card: card ?? null,
  });

  const openReceipt = await clickFragmentCard(driver);
  await waitFor(
    "fragment editor content",
    () => editorText(driver),
    (text) => Boolean(text?.includes(FULL_TOKEN)),
    10_000,
  );
  const fragmentEditor = (await editorText(driver)) ?? "";
  const afterOpenElements = await mainElements(driver);
  const afterOpenState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const backBar = afterOpenElements.flat.find(
    (el) => el.semanticId === FRAGMENT_BACK_ID || el.id === FRAGMENT_BACK_ID,
  );
  const openedPath = matchingFragments[0]?.path ?? "";
  check("fragment_card_opened", afterOpenState.promptType === "dayPage" && fragmentEditor.includes(FULL_TOKEN), {
    promptType: afterOpenState.promptType,
    editorContainsFullPayload: fragmentEditor.includes(FULL_TOKEN),
    editorContainsDayReference: fragmentEditor.includes("../fragments/"),
    backBarVisible: Boolean(backBar),
    openedPathEndsWith: openedPath.endsWith(".md"),
    openReceipt,
  });

  await driver.simulateKey("k", ["cmd"]);
  await Bun.sleep(800);
  for (const ch of "back today") {
    await driver.simulateKey(ch === " " ? "space" : ch);
    await Bun.sleep(35);
  }
  await Bun.sleep(500);
  const dialog = await actionsDialogState(driver);
  const rows = visibleActions(dialog);
  const backAction = rows.find((row) => actionRowId(row) === "day_page:back_to_today");
  check("back_to_today_action_visible", Boolean(backAction), {
    actionId: "day_page:back_to_today",
    row: backAction ?? null,
    visibleActionIds: rows.map(actionRowId),
  });
  await driver.simulateKey("enter");
  await Bun.sleep(1000);
  const returnedText = (await editorText(driver)) ?? "";
  const returnedState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const returnedElements = await mainElements(driver);
  const fragmentCardVisibleAgain = returnedElements.flat.some(
    (el) => el.semanticId === FRAGMENT_CARD_ID || el.id === FRAGMENT_CARD_ID,
  );
  check(
    "back_to_today_action_returns",
    returnedState.promptType === "dayPage" &&
      returnedText.includes("../fragments/") &&
      fragmentCardVisibleAgain,
    {
      promptType: returnedState.promptType,
      editorContainsDayReference: returnedText.includes("../fragments/"),
      fragmentCardVisibleAgain,
    },
  );

  const windows = (await driver.listAutomationWindows()) as Json;
  check(
    "deprecated_inline_context_popup_absent",
    !hasDeprecatedContextPopup(windows, returnedElements.flat),
    {
      automationWindowCount: ((windows.windows ?? []) as Json[]).length,
    },
  );

  const appLog = existsSync(driver.logPath) ? readFileSync(driver.logPath, "utf8") : "";
  const unknownWarningCount = (appLog.match(/unknown_warning_count=[1-9][0-9]*/g) ?? []).length;
  check("unknown_warning_count_zero", unknownWarningCount === 0, { unknownWarningCount });
} catch (error) {
  check("probe_completed_without_exception", false, { error: String(error) });
} finally {
  await driver.close().catch(() => {});
}

const protectedAfter = {
  dev: Bun.spawnSync(["git", "diff", "--", "dev.sh"], {
    cwd: process.cwd(),
    stdout: "pipe",
  }).stdout.toString(),
  pi: Bun.spawnSync(["git", "diff", "--", "scripts/agentic/ensure-pi-sidecar.sh"], {
    cwd: process.cwd(),
    stdout: "pipe",
  }).stdout.toString(),
};
const protectedDirtyFilesUnchanged =
  protectedBefore.dev === protectedAfter.dev && protectedBefore.pi === protectedAfter.pi;
check("protected_dirty_files_unchanged", protectedDirtyFilesUnchanged, {
  protectedDirtyFiles: ["dev.sh", "scripts/agentic/ensure-pi-sidecar.sh"],
  protectedDirtyFilesUnchanged,
});

const report: Json = {
  schemaVersion: 1,
  tool: "day-page-fragment-card-probe",
  classification: failures.length === 0 ? "completed" : "failed",
  pass: failures.length === 0,
  failures,
  binary: BINARY,
  runId,
  sessionDir: driver.sessionDir,
  appLog: driver.logPath,
  sandboxHome,
  todayFile,
  fragmentsDir,
  excerptToken: EXCERPT_TOKEN,
  fullToken: FULL_TOKEN,
  ...receipts,
};

console.log(JSON.stringify(report, null, 2));
if (failures.length > 0) {
  process.exit(1);
}
