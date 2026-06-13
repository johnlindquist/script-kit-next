#!/usr/bin/env bun
/**
 * Runtime proof: clipboard sediment enters Today only through approved lanes.
 *
 * Historical filename is kept for existing checklists. The expected behavior:
 * - first single-token URL copy writes one kept URL line to today's markdown
 * - same URL copied again on the same local day dedupes that day line
 * - a single non-URL copy does not silently enter day pages, fragments, or brain_docs
 * - the explicit Today "Insert Clipboard Text" action inserts the current clipboard
 * - no deprecated post-copy popup or inline @ context popup returns
 *
 *   PROBE_BINARY=target-agent/artifacts/today-clipboard-sediment/script-kit-gpui \
 *     bun scripts/agentic/clipboard-post-copy-menu-probe.ts
 */

import { Database } from "bun:sqlite";
import { existsSync, readFileSync } from "node:fs";
import { readdir, readFile } from "node:fs/promises";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/today-clipboard-sediment/script-kit-gpui";

const POPUP_AUTOMATION_ID = "clipboard-post-copy-menu";
const DEPRECATED_CONTEXT_IDS = [
  "day-page-inline-context-popup",
  "day-page-context-popup",
  "inline-context-popup",
  "context-spine-popup",
];
const runId = `clipboard-sediment-${Date.now().toString(36)}`;
const PROBE_URL = `https://example.com/script-kit-${runId}`;
const PRIVACY_TOKEN = `PRIVATE-NONURL-${runId}`;
const EXPLICIT_TOKEN = `EXPLICIT-CLIPBOARD-${runId}`;
const BASE_TODAY_TEXT = `clipboard sediment base ${runId}`;

type ClipboardState = {
  id: string;
  copy_count: number;
  brain_kept: number;
  brain_tier: number;
  kept_url_day: string | null;
} | null;

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

async function findPostCopyPopup(driver: Driver) {
  const windows = await driver.listAutomationWindows();
  const list = (windows.windows ?? []) as Array<Record<string, any>>;
  return (
    list.find((w) => w.automationId === POPUP_AUTOMATION_ID) ??
    list.find((w) => w.semanticSurface === "clipboardPostCopyMenu") ??
    null
  );
}

async function readDayFiles(brainDaysDir: string) {
  const names = await readdir(brainDaysDir).catch(() => [] as string[]);
  const files: Array<{ path: string; content: string }> = [];
  for (const name of names.filter((name) => name.endsWith(".md"))) {
    const path = join(brainDaysDir, name);
    files.push({ path, content: await readFile(path, "utf8") });
  }
  return files;
}

async function readTreeFiles(root: string) {
  const names = await readdir(root).catch(() => [] as string[]);
  const files: Array<{ path: string; content: string }> = [];
  for (const name of names.filter((name) => name.endsWith(".md"))) {
    const path = join(root, name);
    files.push({ path, content: await readFile(path, "utf8") });
  }
  return files;
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

function readClipboardState(dbPath: string, content: string): ClipboardState {
  if (!existsSync(dbPath)) return null;
  const db = new Database(dbPath, { readonly: true });
  try {
    return db
      .query(
        `SELECT id, copy_count, brain_kept, brain_tier, kept_url_day
         FROM history
         WHERE content = ?1
         ORDER BY timestamp DESC
         LIMIT 1`,
      )
      .get(content) as ClipboardState;
  } catch {
    return null;
  } finally {
    db.close();
  }
}

function brainDocsContain(dbPath: string, token: string) {
  if (!existsSync(dbPath)) return false;
  const db = new Database(dbPath, { readonly: true });
  try {
    const row = db
      .query(
        `SELECT 1
         FROM brain_docs
         WHERE title LIKE ?1 OR content LIKE ?1 OR source_id LIKE ?1
         LIMIT 1`,
      )
      .get(`%${token}%`) as Record<string, unknown> | null;
    return Boolean(row);
  } catch {
    return false;
  } finally {
    db.close();
  }
}

async function editorText(driver: Driver): Promise<string | null> {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 180 },
    { timeoutMs: 5000 },
  )) as Json;
  const editor = walkElements(elements).find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  return (editor?.value as string | undefined) ?? null;
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
    { timeoutMs: 6000 },
  )) as Json;
  check(`batch_set_${label}`, batch.success === true, { batch });
  return batch;
}

async function actionsElements(driver: Driver) {
  const elements = (await driver.getElements(
    { target: { type: "kind", kind: "actionsDialog" }, limit: 220 },
    { timeoutMs: 5000 },
  )) as Json;
  return walkElements(elements);
}

function hasDeprecatedContextPopup(windows: Json, flatElements: Json[]) {
  const automationWindows = ((windows.windows ?? []) as Json[]).map((w) =>
    [
      w.automationId,
      w.semanticSurface,
      w.windowKind,
      w.title,
    ]
      .filter(Boolean)
      .join("|"),
  );
  const elementIds = flatElements.map((el) =>
    [el.semanticId, el.id, el.text, el.value].filter(Boolean).join("|"),
  );
  const combined = [...automationWindows, ...elementIds];
  return combined.some((value) =>
    DEPRECATED_CONTEXT_IDS.some((needle) => value.includes(needle)),
  );
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "clipboard-sediment-privacy-dedupe",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_BRAIN_TZ: process.env.SCRIPT_KIT_BRAIN_TZ ?? "America/Denver",
  },
});

const sandboxHome = driver.sandboxHome ?? `${driver.sessionDir}/home`;
const skPath = join(sandboxHome, ".scriptkit");
const canonicalDaysDir = join(skPath, "brain", "days");
const fragmentsDir = join(skPath, "brain", "fragments");
const clipboardDbPath = join(skPath, "db", "clipboard-history.sqlite");
const brainDbPath = join(skPath, "db", "brain.sqlite");
const todayFile = join(canonicalDaysDir, `${todayLocalDate()}.md`);
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

  await copyText(PROBE_URL);
  const firstUrlFiles = await waitFor(
    "first URL sediment",
    () => readDayFiles(canonicalDaysDir),
    (files) => files.some((file) => file.content.includes(PROBE_URL)),
    12_000,
  );
  const firstJoined = firstUrlFiles.map((file) => file.content).join("\n");
  const firstUrlOccurrences = countOccurrences(firstJoined, PROBE_URL);
  const firstUrlState = await waitFor(
    "first URL clipboard state",
    () => readClipboardState(clipboardDbPath, PROBE_URL),
    (state) => Boolean(state?.brain_kept && state?.kept_url_day),
    8_000,
  );
  check("url_sediment_first_copy", firstUrlOccurrences === 1 && Boolean(firstUrlState), {
    urlOccurrences: firstUrlOccurrences,
    brainKept: Boolean(firstUrlState?.brain_kept),
    keptUrlDay: firstUrlState?.kept_url_day ?? null,
    copyCount: firstUrlState?.copy_count ?? null,
  });

  const separatorToken = `separator-${runId}`;
  await copyText(separatorToken);
  await waitFor(
    "separator clipboard state",
    () => readClipboardState(clipboardDbPath, separatorToken),
    (state) => Boolean(state),
    8_000,
  );
  await copyText(PROBE_URL);
  const repeatedUrlState = await waitFor(
    "repeated URL copy count",
    () => readClipboardState(clipboardDbPath, PROBE_URL),
    (state) => (state?.copy_count ?? 0) >= 2,
    8_000,
  );
  await Bun.sleep(800);
  const afterRepeatFiles = await readDayFiles(canonicalDaysDir);
  const afterRepeatJoined = afterRepeatFiles.map((file) => file.content).join("\n");
  const urlOccurrencesAfterRepeat = countOccurrences(afterRepeatJoined, PROBE_URL);
  check("url_recopy_dedupes_same_day", urlOccurrencesAfterRepeat === 1, {
    urlOccurrencesAfterRepeat,
    copyCountAtLeast: repeatedUrlState?.copy_count ?? 0,
    keptUrlDay: repeatedUrlState?.kept_url_day ?? null,
  });

  await copyText(PRIVACY_TOKEN);
  const privacyState = await waitFor(
    "single non-url clipboard state",
    () => readClipboardState(clipboardDbPath, PRIVACY_TOKEN),
    (state) => Boolean(state),
    8_000,
  );
  await Bun.sleep(1000);
  const dayContainsToken = (await readDayFiles(canonicalDaysDir)).some((file) =>
    file.content.includes(PRIVACY_TOKEN),
  );
  const fragmentsContainToken = (await readTreeFiles(fragmentsDir)).some((file) =>
    file.content.includes(PRIVACY_TOKEN),
  );
  const brainDocsContainToken = brainDocsContain(brainDbPath, PRIVACY_TOKEN);
  check(
    "single_non_url_privacy_not_auto_inserted",
    !dayContainsToken &&
      !fragmentsContainToken &&
      !brainDocsContainToken &&
      privacyState?.brain_kept === 0 &&
      privacyState?.copy_count === 1,
    {
      dayContainsToken,
      fragmentsContainToken,
      brainDocsContainToken,
      brainKept: privacyState?.brain_kept ?? null,
      copyCount: privacyState?.copy_count ?? null,
    },
  );

  await setDayPageInput(driver, BASE_TODAY_TEXT, "base_today_text");
  await Bun.sleep(1200);
  await copyText(EXPLICIT_TOKEN);
  await waitFor(
    "explicit clipboard state",
    () => readClipboardState(clipboardDbPath, EXPLICIT_TOKEN),
    (state) => Boolean(state),
    8_000,
  );

  await driver.simulateKey("k", ["cmd"]);
  await Bun.sleep(800);
  for (const ch of "insert clipboard") {
    await driver.simulateKey(ch === " " ? "space" : ch);
    await Bun.sleep(35);
  }
  await Bun.sleep(500);
  const insertRows = await actionsElements(driver);
  const insertRow = insertRows.find((el) =>
    [el.semanticId, el.id, el.text, el.value].some(
      (value) =>
        typeof value === "string" &&
        (value.includes("day_page:insert_clipboard") ||
          value.includes("Insert Clipboard Text")),
    ),
  );
  check("explicit_insert_action_visible", Boolean(insertRow), {
    actionId: "day_page:insert_clipboard",
    row: insertRow ?? null,
  });

  await driver.simulateKey("enter");
  await Bun.sleep(1200);
  const editorAfterInsert = await editorText(driver);
  await waitFor(
    "explicit token autosave",
    () => (existsSync(todayFile) ? readFileSync(todayFile, "utf8") : ""),
    (content) => content.includes(EXPLICIT_TOKEN),
    10_000,
  );
  const dayFileAfterInsert = existsSync(todayFile) ? readFileSync(todayFile, "utf8") : "";
  const explicitTokenOccurrences = countOccurrences(dayFileAfterInsert, EXPLICIT_TOKEN);
  check(
    "explicit_insert_action_executes",
    Boolean(editorAfterInsert?.includes(EXPLICIT_TOKEN)) &&
      dayFileAfterInsert.includes(EXPLICIT_TOKEN) &&
      explicitTokenOccurrences === 1,
    {
      editorContainsExplicitToken: Boolean(editorAfterInsert?.includes(EXPLICIT_TOKEN)),
      dayFileContainsExplicitToken: dayFileAfterInsert.includes(EXPLICIT_TOKEN),
      explicitTokenOccurrences,
    },
  );

  const popup = await findPostCopyPopup(driver);
  check("post_copy_popup_absent", !popup, {
    postCopyPopupPresent: Boolean(popup),
    postCopyPopup: popup,
  });

  const windows = (await driver.listAutomationWindows()) as Json;
  check("deprecated_inline_context_popup_absent", !hasDeprecatedContextPopup(windows, insertRows), {
    automationWindowCount: ((windows.windows ?? []) as Json[]).length,
  });

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
  protectedDirtyFilesUnchanged,
});

const report: Json = {
  schemaVersion: 1,
  tool: "clipboard-sediment-privacy-dedupe-probe",
  classification: failures.length === 0 ? "completed" : "failed",
  pass: failures.length === 0,
  failures,
  binary: BINARY,
  runId,
  sessionDir: driver.sessionDir,
  appLog: driver.logPath,
  sandboxHome,
  canonicalDaysDir,
  fragmentsDir,
  clipboardDbPath,
  derivedDbPath: brainDbPath,
  todayFile,
  probeUrl: PROBE_URL,
  privacyToken: PRIVACY_TOKEN,
  explicitToken: EXPLICIT_TOKEN,
  ...receipts,
};

console.log(JSON.stringify(report, null, 2));
if (failures.length > 0) {
  process.exit(1);
}
