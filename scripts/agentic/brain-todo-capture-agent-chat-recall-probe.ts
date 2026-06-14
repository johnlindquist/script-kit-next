#!/usr/bin/env bun
/**
 * Runtime proof: launcher ;todo capture reaches canonical Day qmd, Brain
 * recall/resources, and Agent Chat recall staging alongside a related Note.
 */
import { Database } from "bun:sqlite";
import { createHash, randomUUID } from "node:crypto";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { createServer } from "node:net";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

type BrainRow = {
  id: number;
  source: string;
  source_id: string;
  title: string;
  content: string;
  updated_at: number;
};

type Check = { name: string; pass: boolean; detail?: Json };

const binary =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/brain-todo-capture/script-kit-gpui";
const timezone = process.env.SCRIPT_KIT_BRAIN_TZ || "America/Denver";
const dueDate = "2026-06-14";
const runId = `brain-todo-${Date.now().toString(36)}`;
const todoToken = `TODO-${Math.random().toString(36).slice(2, 8).toUpperCase()}`;
const noteToken = `NOTE-${Math.random().toString(36).slice(2, 8).toUpperCase()}`;
const topic = `amber-task-${runId}`;
const agentOnlyTopic = `unprimed-qmd-${runId}`;
const agentOnlyTodoToken = `AGENT-DAY-${Math.random().toString(36).slice(2, 8).toUpperCase()}`;
const agentOnlyNoteToken = `AGENT-NOTE-${Math.random().toString(36).slice(2, 8).toUpperCase()}`;
const agentOnlyFragmentToken = `AGENT-FRAG-${Math.random().toString(36).slice(2, 8).toUpperCase()}`;
const todoBody = `qmd todo parity ${topic} ${todoToken}`;
const todoInput = `;todo ${todoBody} #brain due:${dueDate}`;
const expectedTaskTail = `${todoBody} #brain due:${dueDate}`;
const todoComposerInput = `todo; ${expectedTaskTail}`;
const noteFact = `Related note for ${topic} carries ${noteToken}.`;
const agentOnlyQuestion = `Which qmd Day, Note, and Fragment tokens are stored for ${agentOnlyTopic}?`;
const outPath = ".test-output/brain-todo-capture-agent-chat-recall-probe.json";

const checks: Check[] = [];
const failures: string[] = [];

function check(name: string, pass: boolean, detail: Json = {}) {
  checks.push({ name, pass, detail });
  if (!pass) failures.push(name);
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 30_000,
  intervalMs = 250,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  let last: T | undefined;
  while (Date.now() < deadline) {
    last = await read();
    if (accept(last)) return last;
    await sleep(intervalMs);
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
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

async function findFreePort(): Promise<number> {
  return await new Promise((resolve, reject) => {
    const server = createServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address ? address.port : 0;
      server.close((error) => (error ? reject(error) : resolve(port)));
    });
  });
}

function readJson(path: string): Json {
  return JSON.parse(readFileSync(path, "utf8")) as Json;
}

function writeFile(path: string, content: string) {
  mkdirSync(dirname(path), { recursive: true });
  writeFileSync(path, content, "utf8");
}

function isoNow() {
  return new Date().toISOString().replace(/\.\d{3}Z$/, "Z");
}

function seedCanonicalDayTask(dayFile: string) {
  const line = `23:58 - [ ] qmd unprimed day fact ${agentOnlyTopic} ${agentOnlyTodoToken}`;
  const existing = existsSync(dayFile) ? readFileSync(dayFile, "utf8") : "";
  const prefix = existing.endsWith("\n") || existing.length === 0 ? "" : "\n";
  writeFile(dayFile, `${existing}${prefix}${line}\n`);
  return { line };
}

function seedCanonicalNote(notesDir: string) {
  const id = randomUUID();
  const slug = `${runId}-agent-only-note`;
  const created = isoNow();
  const body = [
    `# ${agentOnlyTopic} note`,
    "",
    `QMD unprimed note fact ${agentOnlyTopic} ${agentOnlyNoteToken}.`,
    "",
  ].join("\n");
  const raw = [
    "---",
    `id: ${id}`,
    `created: ${created}`,
    `updated: ${created}`,
    `source: scriptkit://agent-chat-proof/${slug}`,
    "---",
    "",
    body,
  ].join("\n");
  const path = join(notesDir, `${slug}.md`);
  writeFile(path, raw);
  return { id, slug, path, body };
}

function seedCanonicalFragment(brainBase: string) {
  const id = randomUUID();
  const fragmentId = `${runId}-agent-only-fragment`;
  const created = isoNow();
  const body = `QMD unprimed fragment fact ${agentOnlyTopic} ${agentOnlyFragmentToken}.\n`;
  const raw = [
    "---",
    `id: ${id}`,
    `created: ${created}`,
    `updated: ${created}`,
    `source: scriptkit://agent-chat-proof/${fragmentId}`,
    "---",
    "",
    body,
  ].join("\n");
  const path = join(brainBase, "fragments", `${fragmentId}.md`);
  writeFile(path, raw);
  return { id, fragmentId, path, body };
}

async function mcp(serverJsonPath: string, method: string, params: Json): Promise<Json> {
  const discovery = readJson(serverJsonPath);
  const endpoint = String(discovery.url ?? "").endsWith("/rpc")
    ? String(discovery.url)
    : `${String(discovery.url ?? "").replace(/\/$/, "")}/rpc`;
  const token = String(discovery.token ?? "");
  if (!endpoint || !token) {
    throw new Error(`invalid MCP discovery at ${serverJsonPath}`);
  }
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      authorization: `Bearer ${token}`,
      "content-type": "application/json",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: `${runId}-${method}-${Date.now()}`,
      method,
      params,
    }),
  });
  const body = (await response.json()) as Json;
  if (!response.ok || body.error) {
    throw new Error(`MCP ${method} failed: ${JSON.stringify(body)}`);
  }
  return body.result as Json;
}

async function readResource(serverJsonPath: string, uri: string) {
  const result = await mcp(serverJsonPath, "resources/read", { uri });
  const first = result.contents?.[0] as Json | undefined;
  if (!first || typeof first.text !== "string") {
    throw new Error(`resources/read returned no text for ${uri}: ${JSON.stringify(result)}`);
  }
  return {
    mimeType: String(first.mimeType ?? ""),
    text: first.text,
  };
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

function logAfter(log: string, offset: number) {
  return log.slice(Math.min(offset, log.length));
}

function parseNumber(line: string | null, key: string): number | null {
  const value = new RegExp(`${key}=([0-9]+(?:\\.[0-9]+)?)`).exec(line ?? "")?.[1];
  return value == null ? null : Number(value);
}

function containsAnyNeedle(text: string, needles: string[]) {
  return needles.filter((needle) => text.includes(needle));
}

function taskLineRegex() {
  return new RegExp(
    `^\\d\\d:\\d\\d - \\[ \\] ${escapeRegex(expectedTaskTail)}$`,
    "m",
  );
}

function escapeRegex(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

let driver: Driver | null = null;
let driverClosed = false;

try {
  const mcpPort = await findFreePort();
  driver = await Driver.launch({
    binary,
    sessionName: "brain-todo-capture-agent-chat-recall",
    sandboxHome: true,
    readyTimeoutMs: 15_000,
    defaultTimeoutMs: 10_000,
    env: {
      MCP_PORT: String(mcpPort),
      SCRIPT_KIT_BRAIN_TZ: timezone,
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    },
  });

  const localDate = localDateFor(new Date(), timezone);
  const sandboxHome = join(driver.sessionDir, "home");
  const skPath = join(sandboxHome, ".scriptkit");
  const brainBase = join(skPath, "brain");
  const dayFile = join(brainBase, "days", `${localDate}.md`);
  const notesDir = join(brainBase, "notes");
  const brainDbPath = join(skPath, "db", "brain.sqlite");
  const serverJsonPath = join(skPath, "server.json");
  const blockedMetadataNeedles = ["/Users/", ".scriptkit/db/brain.sqlite", sandboxHome];
  copyAuthIntoSandbox(sandboxHome);

  await waitFor("MCP server.json", () => existsSync(serverJsonPath), Boolean, 12_000);

  await driver.setFilterAndWait(todoInput);
  await sleep(250);
  const beforeSubmit = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("launcher_todo_capture_ready", beforeSubmit.promptType === "none", {
    inputValue: beforeSubmit.inputValue,
    promptType: beforeSubmit.promptType,
    visibleChoiceCount: beforeSubmit.visibleChoiceCount,
  });
  await driver.simulateKey("enter");
  type TodoFirstEnterState = {
    state: Json;
    dayContent: string;
  };
  const afterFirstEnter = await waitFor<TodoFirstEnterState>(
    "postfix todo composer or canonical day task line after first Enter",
    async () => {
      const state = (await driver!
        .getState({ timeoutMs: 5000 })
        .catch((error) => ({ error: String(error) }))) as Json;
      return {
        state,
        dayContent: existsSync(dayFile) ? readFileSync(dayFile, "utf8") : "",
      };
    },
    (value) =>
      taskLineRegex().test(value.dayContent) ||
      String(value.state.inputValue ?? "") === todoComposerInput,
    8_000,
  );
  const firstEnterAlreadySaved = taskLineRegex().test(afterFirstEnter.dayContent);
  check(
    "launcher_todo_first_enter_reaches_capture_submit_path",
    firstEnterAlreadySaved ||
      String(afterFirstEnter.state.inputValue ?? "") === todoComposerInput,
    {
      firstEnterAlreadySaved,
      inputValue: afterFirstEnter.state.inputValue,
      expectedComposerInput: todoComposerInput,
      promptType: afterFirstEnter.state.promptType,
      visibleChoiceCount: afterFirstEnter.state.visibleChoiceCount,
    },
  );
  if (!firstEnterAlreadySaved) {
    await driver.simulateKey("enter");
  }
  const dayContent = firstEnterAlreadySaved
    ? afterFirstEnter.dayContent
    : await waitFor(
        "canonical day task line",
        () => (existsSync(dayFile) ? readFileSync(dayFile, "utf8") : ""),
        (content) => taskLineRegex().test(content),
        12_000,
      );
  const matchedTaskLine = taskLineRegex().exec(dayContent)?.[0] ?? null;
  check("canonical_day_task_line_written", matchedTaskLine != null, {
    canonicalDayFile: dayFile,
    matchedTaskLine,
    mtimeMs: statSync(dayFile).mtimeMs,
  });

  const dayState = await openDayPage(driver, runId);
  check("opened_day_page_after_todo_capture", dayState.promptType === "dayPage", {
    promptType: dayState.promptType,
    inputValueIncludesTask: String(dayState.inputValue ?? "").includes(expectedTaskTail),
  });
  const dayElements = (await driver.getElements(
    { target: { type: "main" }, limit: 160 },
    { timeoutMs: 5000 },
  )) as Json;
  const dayEditor = JSON.stringify(dayElements).includes("day-page-editor");
  const editorStyleText = JSON.stringify(dayElements);
  check("day_editor_shows_task_line", String(dayState.inputValue ?? "").includes(expectedTaskTail), {
    inputValueLength: String(dayState.inputValue ?? "").length,
    dayEditor,
  });
  check("day_editor_uses_shared_notes_editor_style", editorStyleText.includes("notes_editor"), {
    dayEditor,
    styleExcerpt: editorStyleText.slice(0, 1000),
  });

  driver.send({ type: "openNotes", requestId: `${runId}-open-notes` });
  await waitFor(
    "notes target ready",
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
  const noteText = `# ${topic} related note\n\n${noteFact}\n`;
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
  const canonicalNoteFile = noteFiles[0];
  check("canonical_note_file_written", Boolean(canonicalNoteFile), {
    canonicalNoteFile,
  });

  const recallUri = `kit://brain/recall?q=${encodeURIComponent(todoToken)}&format=json`;
  const recallResource = await readResource(serverJsonPath, recallUri);
  const recallJson = JSON.parse(recallResource.text) as Json;
  const dayHit = (recallJson.hits ?? []).find((hit: Json) => hit.source === "day_page");
  const recallDetail = {
    uri: recallUri,
    mimeType: recallResource.mimeType,
    schemaVersion: recallJson.schemaVersion,
    source: dayHit?.source ?? null,
    sourceId: dayHit?.sourceId ?? null,
    citationUri: dayHit?.citationUri ?? null,
    canonicalPath: dayHit?.canonicalPath ?? null,
    lineStart: dayHit?.lineStart ?? null,
    lineEnd: dayHit?.lineEnd ?? null,
  };
  check(
    "recall_json_reports_day_canonical_path",
    recallResource.mimeType === "application/json" &&
      recallDetail.schemaVersion === 1 &&
      recallDetail.source === "day_page" &&
      recallDetail.sourceId === localDate &&
      recallDetail.citationUri === `brain://day_page/${localDate}` &&
      recallDetail.canonicalPath === `brain/days/${localDate}.md` &&
      Number.isInteger(recallDetail.lineStart) &&
      Number.isInteger(recallDetail.lineEnd),
    recallDetail,
  );

  const noteRecallUri = `kit://brain/recall?q=${encodeURIComponent(noteToken)}&format=json`;
  const noteRecallResource = await readResource(serverJsonPath, noteRecallUri);
  const noteRecallJson = JSON.parse(noteRecallResource.text) as Json;
  const noteHit = (noteRecallJson.hits ?? []).find((hit: Json) => hit.source === "note");
  check(
    "recall_json_reports_note_canonical_path",
    noteRecallResource.mimeType === "application/json" &&
      noteHit?.source === "note" &&
      typeof noteHit?.sourceId === "string" &&
      String(noteHit?.canonicalPath ?? "").startsWith("brain/notes/") &&
      Number.isInteger(noteHit?.lineStart) &&
      Number.isInteger(noteHit?.lineEnd),
    {
      uri: noteRecallUri,
      source: noteHit?.source ?? null,
      sourceId: noteHit?.sourceId ?? null,
      canonicalPath: noteHit?.canonicalPath ?? null,
      citationUri: noteHit?.citationUri ?? null,
      lineStart: noteHit?.lineStart ?? null,
      lineEnd: noteHit?.lineEnd ?? null,
    },
  );

  const metadataText = [recallResource.text, noteRecallResource.text]
    .map((text) => text.replace(/"content":"[^"]*"/g, '"content":"<omitted>"'))
    .join("\n");
  check("no_private_storage_paths_in_recall_metadata", containsAnyNeedle(metadataText, blockedMetadataNeedles).length === 0, {
    leakedNeedles: containsAnyNeedle(metadataText, blockedMetadataNeedles),
  });

  const agentDay = seedCanonicalDayTask(dayFile);
  const agentNote = seedCanonicalNote(notesDir);
  const agentFragment = seedCanonicalFragment(brainBase);
  check("agent_only_canonical_day_task_seeded", existsSync(dayFile) && readFileSync(dayFile, "utf8").includes(agentOnlyTodoToken), {
    canonicalDayFile: dayFile,
    line: agentDay.line,
  });
  check("agent_only_canonical_note_seeded", existsSync(agentNote.path), {
    canonicalNoteFile: agentNote.path,
    sourceId: agentNote.id,
  });
  check("agent_only_canonical_fragment_seeded", existsSync(agentFragment.path), {
    canonicalFragmentFile: agentFragment.path,
    sourceId: agentFragment.fragmentId,
  });
  const preSubmitRows = readRows(brainDbPath, agentOnlyTopic);
  check("agent_only_tokens_not_preprimed_in_brain_db", preSubmitRows.length === 0, {
    rows: preSubmitRows.map((row) => ({
      source: row.source,
      sourceId: row.source_id,
      title: row.title,
    })),
  });

  driver.send({ type: "openAi", requestId: `${runId}-open-ai` });
  const opened = await waitFor(
    "Agent Chat open",
    () => agentChatState(driver).catch((error) => ({ error: String(error) })),
    (state) => !("error" in state) && stateString(state, "status") !== "setup",
    30_000,
  );
  check("agent_chat_opened", true, {
    status: stateString(opened, "status"),
    uiVariant: opened.uiVariant ?? opened.ui_variant,
    messageCount: stateNumber(opened, "messageCount"),
  });
  const appLogBeforeSubmit = existsSync(driver.logPath) ? readFileSync(driver.logPath, "utf8") : "";
  const appLogSubmitOffset = appLogBeforeSubmit.length;
  const submit = await driver.request(
    { type: "setAgentChatInput", text: agentOnlyQuestion, submit: true },
    { timeoutMs: 10_000 },
  );
  check("submitted_agent_chat_question", (submit as Json).type !== "error", { submit });

  const rows = await waitFor(
    "derived agent-only day note and fragment rows",
    () => readRows(brainDbPath, agentOnlyTopic),
    (values) =>
      values.some((row) => row.source === "day_page" && row.content.includes(agentOnlyTodoToken)) &&
      values.some((row) => row.source === "note" && row.content.includes(agentOnlyNoteToken)) &&
      values.some((row) => row.source === "fragment" && row.content.includes(agentOnlyFragmentToken)),
    20_000,
  );
  const dayRow = rows.find((row) => row.source === "day_page" && row.content.includes(agentOnlyTodoToken));
  const noteRow = rows.find((row) => row.source === "note" && row.content.includes(agentOnlyNoteToken));
  const fragmentRow = rows.find((row) => row.source === "fragment" && row.content.includes(agentOnlyFragmentToken));
  check("brain_docs_contain_day_todo", Boolean(dayRow), {
    sourceId: dayRow?.source_id ?? null,
    title: dayRow?.title ?? null,
  });
  check("brain_docs_contain_note_fact", Boolean(noteRow), {
    sourceId: noteRow?.source_id ?? null,
    title: noteRow?.title ?? null,
  });
  check("brain_docs_contain_fragment_fact", Boolean(fragmentRow), {
    sourceId: fragmentRow?.source_id ?? null,
    title: fragmentRow?.title ?? null,
  });

  const appLog = await waitFor(
    "post-submit brain recall logs",
    () => (existsSync(driver!.logPath) ? logAfter(readFileSync(driver!.logPath, "utf8"), appLogSubmitOffset) : ""),
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
  const fragmentFingerprint =
    fragmentRow == null ? "" : `fragment:${fragmentRow.source_id}:${sha16(fragmentRow.content)}`;
  check("recall_file_sources_synced", recallSyncElapsedMs != null && failedSourcesEmpty, {
    syncLine,
    recallSyncElapsedMs,
  });
  check("recall_sources_include_day_page", builtLine?.includes("day_page") === true, {
    builtLine,
  });
  check("recall_sources_include_note", builtLine?.includes("note") === true, {
    builtLine,
  });
  check("recall_sources_include_fragment", builtLine?.includes("fragment") === true, {
    builtLine,
  });
  check("final_user_content_has_day_todo", builtLine?.includes(dayFingerprint) === true, {
    dayFingerprint,
    builtLine,
  });
  check("final_user_content_has_note_fact", builtLine?.includes(noteFingerprint) === true, {
    noteFingerprint,
    builtLine,
  });
  check("final_user_content_has_fragment_fact", builtLine?.includes(fragmentFingerprint) === true, {
    fragmentFingerprint,
    builtLine,
  });
  check("agent_chat_brain_recall_staged", Boolean(stagedLine), { stagedLine });
  const unknownWarningCount =
    (appLog.match(/unknown_warning_count=[1-9][0-9]*/g) ?? []).length;
  check("unknown_warning_count_zero", unknownWarningCount === 0, { unknownWarningCount });

  await driver.close();
  driverClosed = true;
  check("driver_closed", true, {});

  const pass = failures.length === 0 && checks.every((item) => item.pass);
  const receipt = {
    schemaVersion: 1,
    tool: "brain-todo-capture-agent-chat-recall-probe",
    classification: pass ? "completed" : "failed",
    pass,
    failures,
    runId,
    binary,
    timezone,
    localDate,
    todoInput,
    expectedTaskTail,
    canonicalDayFile: dayFile,
    canonicalNoteFile,
    sessionDir: driver.sessionDir,
    appLog: driver.logPath,
    screenshotProof: "not-used-semantic-devtools-only",
    checks,
  };
  mkdirSync(".test-output", { recursive: true });
  Bun.write(outPath, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
  if (!pass) process.exit(1);
} catch (error) {
  if (driver && !driverClosed) {
    await driver.close().catch(() => {});
    driverClosed = true;
  }
  const receipt = {
    schemaVersion: 1,
    tool: "brain-todo-capture-agent-chat-recall-probe",
    classification: "error",
    pass: false,
    failures: ["probe_completed_without_exception"],
    error: error instanceof Error ? error.message : String(error),
    binary,
    sessionDir: driver?.sessionDir ?? null,
    appLog: driver?.logPath ?? null,
    checks,
  };
  mkdirSync(".test-output", { recursive: true });
  Bun.write(outPath, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
  process.exit(1);
}
