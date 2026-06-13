#!/usr/bin/env bun
/**
 * Runtime proof: fresh Day Page + Notes markdown reaches Brain-profile Agent
 * Chat recall without waiting for the background indexer.
 */
import { Database } from "bun:sqlite";
import { createHash } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
} from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const binary =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/brain-day-notes-recall/script-kit-gpui";
const timezone = process.env.SCRIPT_KIT_BRAIN_TZ || "America/Denver";
const runId = `brain-day-notes-${Date.now().toString(36)}`;
const dayGate = `DAY-${Date.now().toString(36).slice(-5).toUpperCase()}`;
const noteGate = `NOTE-${Math.random().toString(36).slice(2, 7).toUpperCase()}`;
const topic = `violet-bridge-${runId}`;
const dayFact = `The ${topic} day release gate is ${dayGate}.`;
const noteFact = `The ${topic} note release gate is ${noteGate}.`;
const question = `What are the day and note release gates for ${topic}?`;

type BrainRow = {
  id: number;
  source: string;
  source_id: string;
  title: string;
  content: string;
  updated_at: number;
};

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "brain-day-notes-agent-chat-recall-probe",
  classification: "blocked",
  pass: false,
  failures: [] as string[],
  runId,
  binary,
  timezone,
  topic,
  dayGate,
  noteGate,
  question,
};

function fail(name: string, detail: unknown) {
  (receipt.failures as string[]).push(name);
  receipt[name] = detail;
}

function check(name: string, ok: boolean, detail: unknown = {}) {
  receipt[name] = { ok, ...(typeof detail === "object" && detail ? (detail as object) : { detail }) };
  if (!ok) fail(`failed_${name}`, detail);
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function localDateFor(date: Date, timeZone: string): string {
  const parts = new Intl.DateTimeFormat("en-US", {
    timeZone,
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).formatToParts(date);
  const part = (type: string) =>
    parts.find((entry) => entry.type === type)?.value ?? "";
  return `${part("year")}-${part("month")}-${part("day")}`;
}

function sha16(text: string): string {
  return createHash("sha256").update(text).digest("hex").slice(0, 16);
}

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 30_000,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  let last: T | undefined;
  while (Date.now() < deadline) {
    last = await read();
    if (accept(last)) return last;
    await sleep(250);
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

function copyAuthIntoSandbox(sandboxHome: string) {
  for (const rel of [
    ".pi/agent/auth.json",
    ".pi/agent/settings.json",
    ".codex/auth.json",
  ]) {
    const src = join(homedir(), rel);
    if (!existsSync(src)) continue;
    const dst = join(sandboxHome, rel);
    mkdirSync(join(dst, ".."), { recursive: true });
    cpSync(src, dst);
  }
}

function readRows(dbPath: string, pattern: string): BrainRow[] {
  if (!existsSync(dbPath)) return [];
  const db = new Database(dbPath, { readonly: true });
  try {
    return db
      .query(
        `SELECT id, source, source_id, title, content, updated_at
         FROM brain_docs
         WHERE content LIKE ?1
         ORDER BY updated_at DESC, id DESC`,
      )
      .all(`%${pattern}%`) as BrainRow[];
  } finally {
    db.close();
  }
}

function noteFilesContaining(notesDir: string, text: string) {
  if (!existsSync(notesDir)) return [] as string[];
  return readdirSync(notesDir)
    .filter((file) => file.endsWith(".md"))
    .map((file) => join(notesDir, file))
    .filter((path) => readFileSync(path, "utf8").includes(text));
}

async function agentChatState(driver: Driver): Promise<Record<string, unknown>> {
  const result = (await driver.request(
    { type: "getAgentChatState" },
    { timeoutMs: 10_000 },
  )) as Record<string, unknown>;
  return (result.state ?? result) as Record<string, unknown>;
}

function stateString(state: Record<string, unknown>, key: string): string {
  return String(
    state[key] ?? state[key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`)] ?? "",
  );
}

function stateNumber(state: Record<string, unknown>, key: string): number {
  return Number(
    state[key] ?? state[key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`)] ?? 0,
  );
}

function latestLine(log: string, needle: string): string | null {
  return log
    .split("\n")
    .filter((line) => line.includes(needle))
    .at(-1) ?? null;
}

function parseNumber(line: string | null, key: string): number | null {
  const value = new RegExp(`${key}=([0-9]+(?:\\.[0-9]+)?)`).exec(line ?? "")?.[1];
  return value == null ? null : Number(value);
}

const localDate = localDateFor(new Date(), timezone);
const driver = await Driver.launch({
  sessionName: "brain-day-notes-agent-chat-recall",
  sandboxHome: true,
  binary,
  readyTimeoutMs: 15_000,
  defaultTimeoutMs: 10_000,
  env: {
    SCRIPT_KIT_BRAIN_TZ: timezone,
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});

const sandboxHome = join(driver.sessionDir, "home");
const skPath = join(sandboxHome, ".scriptkit");
const brainBase = join(skPath, "brain");
const brainDbPath = join(skPath, "db", "brain.sqlite");
const dayFile = join(brainBase, "days", `${localDate}.md`);
const notesDir = join(brainBase, "notes");
receipt.sessionDir = driver.sessionDir;
receipt.appLog = driver.logPath;
receipt.canonicalDayFile = dayFile;
receipt.canonicalNotesDir = notesDir;
receipt.derivedDbPath = brainDbPath;
copyAuthIntoSandbox(sandboxHome);

try {
  const dayState = await openDayPage(driver, runId);
  check("opened_day_page", dayState.promptType === "dayPage", {
    promptType: dayState.promptType,
    windowVisible: dayState.windowVisible,
  });

  const dayText = `# ${localDate}\n\n${dayFact}\n`;
  const setDay = (await driver.batch(
    [{ type: "setInput", text: dayText }],
    { timeoutMs: 10_000 },
  )) as Json;
  check("set_day_page_fact", setDay.success === true, { setDay });
  driver.simulateKey("s", ["cmd"]);

  const dayContent = await waitFor(
    "canonical day file",
    () => (existsSync(dayFile) ? readFileSync(dayFile, "utf8") : ""),
    (content) => content.includes(dayFact),
    10_000,
  );
  check("day_file_contains_day_fact", true, {
    bytes: dayContent.length,
    mtimeMs: statSync(dayFile).mtimeMs,
  });

  driver.send({ type: "openNotes", requestId: `${runId}-open-notes` });
  await waitFor(
    "notes window elements",
    () =>
      driver
        .getElements(
          { target: { type: "kind", kind: "notes", index: 0 }, limit: 80 },
          { timeoutMs: 5000 },
        )
        .catch((error) => ({ error: String(error) })),
    (value) => !("error" in (value as Record<string, unknown>)),
    10_000,
  );

  const noteText = `# ${topic} note\n\n${noteFact}\n`;
  const setNote = (await driver.request(
    {
      type: "batch",
      requestId: `${runId}-set-note`,
      target: { type: "kind", kind: "notes", index: 0 },
      commands: [{ type: "setInput", text: noteText }],
      options: { stopOnError: true, timeout: 10_000 },
    },
    { expect: "batchResult", timeoutMs: 11_000 },
  )) as Json;
  check("set_notes_fact", setNote.success === true, { setNote });
  await driver.request(
    {
      type: "simulateGpuiEvent",
      requestId: `${runId}-save-note`,
      target: { type: "kind", kind: "notes", index: 0 },
      event: { type: "keyDown", key: "s", modifiers: ["cmd"] },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  );

  const noteFiles = await waitFor(
    "canonical note file",
    () => noteFilesContaining(notesDir, noteFact),
    (files) => files.length > 0,
    15_000,
  );
  check("note_file_contains_note_fact", noteFiles.length > 0, { noteFiles });

  driver.send({ type: "openAi", requestId: `${runId}-open-ai` });
  const opened = await waitFor(
    "Agent Chat open",
    () =>
      agentChatState(driver).catch((error) => ({
        error: String(error),
      })),
    (state) => !("error" in state) && stateString(state, "status") !== "setup",
    30_000,
  );
  check("agent_chat_opened", true, {
    status: stateString(opened, "status"),
    uiVariant: opened.uiVariant ?? opened.ui_variant,
    messageCount: stateNumber(opened, "messageCount"),
  });

  const submit = await driver.request(
    { type: "setAgentChatInput", text: question, submit: true },
    { timeoutMs: 10_000 },
  );
  check("submitted_agent_chat_question", (submit as Json).type !== "error", { submit });

  const rows = await waitFor(
    "derived day and note rows",
    () => readRows(brainDbPath, topic),
    (values) =>
      values.some((row) => row.source === "day_page" && row.content.includes(dayFact)) &&
      values.some((row) => row.source === "note" && row.content.includes(noteFact)),
    20_000,
  );
  const dayRow = rows.find((row) => row.source === "day_page" && row.content.includes(dayFact));
  const noteRow = rows.find((row) => row.source === "note" && row.content.includes(noteFact));
  check("brain_docs_contain_day_fact", Boolean(dayRow), {
    sourceId: dayRow?.source_id ?? null,
    title: dayRow?.title ?? null,
  });
  check("brain_docs_contain_note_fact", Boolean(noteRow), {
    sourceId: noteRow?.source_id ?? null,
    title: noteRow?.title ?? null,
  });

  const appLog = await waitFor(
    "brain recall logs",
    () => (existsSync(driver.logPath) ? readFileSync(driver.logPath, "utf8") : ""),
    (log) =>
      log.includes("event=brain_recall_file_sources_synced") &&
      log.includes("event=brain_recall_context_built") &&
      log.includes("event=agent_chat_brain_recall_staged"),
    30_000,
  );
  const syncLine = latestLine(appLog, "event=brain_recall_file_sources_synced");
  const builtLine = latestLine(appLog, "event=brain_recall_context_built");
  const stagedLine = latestLine(appLog, "event=agent_chat_brain_recall_staged");
  const recallSyncElapsedMs = parseNumber(syncLine, "elapsed_ms");
  const failedSourcesEmpty =
    syncLine?.includes("failed_sources=[]") || syncLine?.includes("failed_sources = []");
  const dayFingerprint =
    dayRow == null ? "" : `day_page:${dayRow.source_id}:${sha16(dayRow.content)}`;
  const noteFingerprint =
    noteRow == null ? "" : `note:${noteRow.source_id}:${sha16(noteRow.content)}`;
  const recallSourcesIncludeDayPage =
    builtLine?.includes('"day_page"') === true || builtLine?.includes("day_page") === true;
  const recallSourcesIncludeNote =
    builtLine?.includes('"note"') === true || builtLine?.includes("note") === true;
  const finalUserContentHasDayFact = builtLine?.includes(dayFingerprint) === true;
  const finalUserContentHasNoteFact = builtLine?.includes(noteFingerprint) === true;

  check("recall_file_sources_synced", recallSyncElapsedMs != null && failedSourcesEmpty, {
    syncLine,
    recallSyncElapsedMs,
  });
  check("recall_sync_under_soft_budget", (recallSyncElapsedMs ?? Infinity) < 500, {
    recallSyncElapsedMs,
  });
  check("recall_sources_include_day_page", recallSourcesIncludeDayPage, { builtLine });
  check("recall_sources_include_note", recallSourcesIncludeNote, { builtLine });
  check("final_user_content_has_day_fact", finalUserContentHasDayFact, {
    dayFingerprint,
    builtLine,
  });
  check("final_user_content_has_note_fact", finalUserContentHasNoteFact, {
    noteFingerprint,
    builtLine,
  });
  check("agent_chat_brain_recall_staged", Boolean(stagedLine), { stagedLine });

  const stateAfterRecall = await agentChatState(driver);
  check("agent_chat_reached_chat_surface", stateString(stateAfterRecall, "status") !== "setup", {
    status: stateString(stateAfterRecall, "status"),
    messageCount: stateNumber(stateAfterRecall, "messageCount"),
  });

  const unknownWarningCount =
    (appLog.match(/unknown_warning_count=[1-9][0-9]*/g) ?? []).length;
  check("unknown_warning_count_zero", unknownWarningCount === 0, { unknownWarningCount });

  if ((receipt.failures as string[]).length === 0) {
    receipt.classification = "completed";
    receipt.pass = true;
  }
} catch (error) {
  receipt.error = String(error);
  if (existsSync(brainDbPath)) {
    receipt.brainRows = readRows(brainDbPath, topic).map((row) => ({
      source: row.source,
      sourceId: row.source_id,
      title: row.title,
      contentHasDayFact: row.content.includes(dayFact),
      contentHasNoteFact: row.content.includes(noteFact),
    }));
  }
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exitCode = 1;
