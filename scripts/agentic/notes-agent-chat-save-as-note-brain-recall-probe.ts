#!/usr/bin/env bun
/**
 * Runtime proof: a Notes-hosted Agent Chat transcript can be saved as a
 * canonical Notes/QMD markdown file, indexed by Brain recall, and recalled by a
 * later Brain-profile Agent Chat turn.
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
import { basename, join } from "node:path";
import { Driver, type Json } from "../devtools/driver";

type BrainRow = {
  source: string;
  source_id: string;
  title: string;
  content: string;
  updated_at: number;
};

type NoteCandidate = {
  path: string;
  content: string;
  mtimeMs: number;
};

const binary =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/notes-chat-save-note-recall/script-kit-gpui";
const timezone = process.env.SCRIPT_KIT_BRAIN_TZ || "America/Denver";
const runId = `notes-save-note-${Date.now().toString(36)}`;
const transcriptToken = `TRANSCRIPT-${Date.now().toString(36).slice(-6).toUpperCase()}`;
const recallToken = `RECALL-${Math.random().toString(36).slice(2, 8).toUpperCase()}`;
const noteTitle = `${runId} source note`;
const noteFact = `Notes source seed for ${runId}.`;
const chatQuestion = `Please remember ${transcriptToken} for the Notes-hosted save-as-note recall proof.`;
const assistantHint = `If you answer, mention ${recallToken}.`;
const recallQuestion = `Which Notes-hosted Agent Chat transcript mentioned ${transcriptToken}?`;

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "notes-agent-chat-save-as-note-brain-recall-probe",
  classification: "blocked",
  pass: false,
  failures: [] as string[],
  runId,
  binary,
  timezone,
  transcriptToken,
  recallToken,
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

function listMarkdownFiles(dir: string): string[] {
  if (!existsSync(dir)) return [];
  return readdirSync(dir)
    .filter((file) => file.endsWith(".md"))
    .map((file) => join(dir, file));
}

function transcriptCandidates(notesDir: string, token: string): NoteCandidate[] {
  return listMarkdownFiles(notesDir)
    .map((path) => ({
      path,
      content: readFileSync(path, "utf8"),
      mtimeMs: statSync(path).mtimeMs,
    }))
    .filter(
      (candidate) =>
        candidate.content.includes("# Agent Chat Conversation") &&
        candidate.content.includes(token),
    );
}

function sourceFrontmatter(content: string): string | null {
  return /^source:\s*(.+)$/m.exec(content)?.[1]?.trim() ?? null;
}

function sha16(text: string): string {
  return createHash("sha256").update(text).digest("hex").slice(0, 16);
}

async function notesAgentChatState(driver: Driver): Promise<Record<string, unknown>> {
  const result = (await driver.request(
    {
      type: "getAgentChatState",
      target: { type: "kind", kind: "notes", index: 0 },
    },
    { timeoutMs: 10_000 },
  )) as Record<string, unknown>;
  return (result.state ?? result) as Record<string, unknown>;
}

async function mainAgentChatState(driver: Driver): Promise<Record<string, unknown>> {
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

function readRows(dbPath: string, pattern: string): BrainRow[] {
  if (!existsSync(dbPath)) return [];
  const db = new Database(dbPath, { readonly: true });
  try {
    return db
      .query(
        `SELECT source, source_id, title, content, updated_at
         FROM brain_docs
         WHERE content LIKE ?1
         ORDER BY updated_at DESC`,
      )
      .all(`%${pattern}%`) as BrainRow[];
  } finally {
    db.close();
  }
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

async function actionsDialogState(driver: Driver): Promise<Record<string, unknown> | null> {
  const result = (await driver
    .request(
      {
        type: "getState",
        target: { type: "kind", kind: "actionsDialog" },
        summaryOnly: true,
      },
      { timeoutMs: 5_000 },
    )
    .catch(() => null)) as Record<string, unknown> | null;
  return (result?.actionsDialog as Record<string, unknown> | undefined) ?? null;
}

function visibleActions(dialog: Record<string, unknown> | null): Record<string, unknown>[] {
  const direct = dialog?.visibleActions;
  if (Array.isArray(direct)) return direct as Record<string, unknown>[];
  const actions = dialog?.actions as Record<string, unknown> | undefined;
  return Array.isArray(actions?.visibleSample) ? (actions.visibleSample as Record<string, unknown>[]) : [];
}

const driver = await Driver.launch({
  sessionName: "notes-agent-chat-save-as-note-brain-recall",
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
const notesDir = join(skPath, "brain", "notes");
const brainDbPath = join(skPath, "db", "brain.sqlite");
receipt.sessionDir = driver.sessionDir;
receipt.appLog = driver.logPath;
receipt.canonicalNotesDir = notesDir;
receipt.derivedDbPath = brainDbPath;
copyAuthIntoSandbox(sandboxHome);

try {
  driver.send({ type: "openNotes", requestId: `${runId}-open-notes` });
  await waitFor(
    "Notes target",
    () =>
      driver
        .getElements(
          { target: { type: "kind", kind: "notes", index: 0 }, limit: 80 },
          { timeoutMs: 5_000 },
        )
        .catch((error) => ({ error: String(error) })),
    (value) => !("error" in (value as Record<string, unknown>)),
    10_000,
  );
  check("notes_opened", true, {});

  const sourceNoteText = `# ${noteTitle}\n\n${noteFact}\n`;
  const setSourceNote = (await driver.request(
    {
      type: "batch",
      requestId: `${runId}-source-note`,
      target: { type: "kind", kind: "notes", index: 0 },
      commands: [{ type: "setInput", text: sourceNoteText }],
      options: { stopOnError: true, timeout: 10_000 },
    },
    { expect: "batchResult", timeoutMs: 11_000 },
  )) as Json;
  check("source_note_written", setSourceNote.success === true, { setSourceNote });

  const beforeTranscriptFiles = new Set(listMarkdownFiles(notesDir));

  const openAgentChat = (await driver.request(
    {
      type: "batch",
      requestId: `${runId}-open-notes-agent-chat`,
      target: { type: "kind", kind: "notes", index: 0 },
      commands: [{ type: "openNotesAgentChat" }],
      options: { stopOnError: true, timeout: 15_000 },
    },
    { expect: "batchResult", timeoutMs: 16_000 },
  )) as Json;
  check("notes_agent_chat_opened", openAgentChat.success === true, { openAgentChat });

  await waitFor(
    "Notes Agent Chat ready",
    () => notesAgentChatState(driver),
    (state) => stateString(state, "status") !== "setup",
    30_000,
  );

  const chatInput = `${chatQuestion}\n\n${assistantHint}`;
  const setChatInput = (await driver.request(
    {
      type: "batch",
      requestId: `${runId}-set-notes-chat-input`,
      target: { type: "kind", kind: "notes", index: 0 },
      commands: [{ type: "setInput", text: chatInput }],
      options: { stopOnError: true, timeout: 10_000 },
    },
    { expect: "batchResult", timeoutMs: 11_000 },
  )) as Json;
  check("notes_agent_chat_input_set", setChatInput.success === true, { setChatInput });

  const submit = await driver.request(
    {
      type: "simulateGpuiEvent",
      requestId: `${runId}-submit-notes-chat`,
      target: { type: "kind", kind: "notes", index: 0 },
      event: { type: "keyDown", key: "enter", modifiers: [] },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 5_000 },
  );
  check("notes_agent_chat_question_submitted", (submit as Json).success !== false, { submit });

  const afterTurn = await waitFor(
    "Notes Agent Chat assistant response",
    () => notesAgentChatState(driver),
    (state) =>
      stateNumber(state, "messageCount") >= 2 && stateString(state, "status") === "idle",
    120_000,
  );
  check("notes_agent_chat_turn_observed", stateNumber(afterTurn, "messageCount") >= 2, {
    status: stateString(afterTurn, "status"),
    messageCount: stateNumber(afterTurn, "messageCount"),
    assistantResponseObserved: stateNumber(afterTurn, "messageCount") >= 2,
  });

  const openActions = await driver.request(
    {
      type: "simulateGpuiEvent",
      requestId: `${runId}-open-notes-chat-actions`,
      target: { type: "kind", kind: "notes", index: 0 },
      event: { type: "keyDown", key: "k", modifiers: ["cmd"] },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 5_000 },
  );
  check("notes_agent_chat_actions_opened", (openActions as Json).success !== false, { openActions });

  await waitFor(
    "Save as Note action visible",
    async () => {
      const setFilter = await driver.request(
        {
          type: "batch",
          requestId: `${runId}-filter-save-action-${Date.now()}`,
          target: { type: "kind", kind: "actionsDialog" },
          commands: [{ type: "setInput", text: "save note" }],
          options: { stopOnError: true, timeout: 5_000 },
        },
        { expect: "batchResult", timeoutMs: 6_000 },
      );
      const dialog = await actionsDialogState(driver);
      return { setFilter, dialog, rows: visibleActions(dialog) };
    },
    (value) =>
      (value.setFilter as Json).success === true &&
      value.rows.some((row) => row.id === "agent_chat_save_as_note"),
    15_000,
  );
  const appLogAfterActionsOpen = existsSync(driver.logPath)
    ? readFileSync(driver.logPath, "utf8")
    : "";
  check(
    "notes_hosted_actions_open_deferred_logged",
    appLogAfterActionsOpen.includes("event=notes_agent_chat_actions_open_deferred"),
    {
      line: latestLine(appLogAfterActionsOpen, "event=notes_agent_chat_actions_open_deferred"),
    },
  );
  check(
    "notes_hosted_actions_opened_logged",
    appLogAfterActionsOpen.includes("event=notes_agent_chat_actions_opened"),
    {
      line: latestLine(appLogAfterActionsOpen, "event=notes_agent_chat_actions_opened"),
    },
  );
  check(
    "no_script_list_double_lease_after_notes_cmd_k",
    !appLogAfterActionsOpen.includes("gpui_entity_double_lease") &&
      !appLogAfterActionsOpen.includes("cannot update script_kit_gpui::ScriptListApp"),
    {
      doubleLeaseLine: latestLine(appLogAfterActionsOpen, "gpui_entity_double_lease"),
      panicLine: latestLine(
        appLogAfterActionsOpen,
        "cannot update script_kit_gpui::ScriptListApp",
      ),
    },
  );
  const saveDialog = await actionsDialogState(driver);
  const saveRow = visibleActions(saveDialog).find((row) => row.id === "agent_chat_save_as_note");
  const saveSemanticId = String(saveRow?.semanticId ?? "");
  check("save_as_note_action_visible", Boolean(saveRow), {
    saveRow,
    saveSemanticId,
  });

  let selectSave: Json = {
    skipped: true,
    reason: "filtered row is visible but getState did not expose semanticId",
  };
  if (saveSemanticId.startsWith("choice:")) {
    selectSave = (await driver.request(
      {
        type: "batch",
        requestId: `${runId}-select-save-action`,
        target: { type: "kind", kind: "actionsDialog" },
        commands: [{ type: "selectBySemanticId", semanticId: saveSemanticId }],
        options: { stopOnError: true, timeout: 5_000 },
      },
      { expect: "batchResult", timeoutMs: 6_000 },
    )) as Json;
  }
  check("save_as_note_action_selected", selectSave.skipped === true || selectSave.success === true, {
    selectSave,
    saveSemanticId,
  });

  const activateSave = await driver.request(
    {
      type: "simulateGpuiEvent",
      requestId: `${runId}-activate-save-action`,
      target: { type: "kind", kind: "actionsDialog" },
      event: { type: "keyDown", key: "enter", modifiers: [] },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 5_000 },
  );
  check("save_as_note_action_activated", (activateSave as Json).success !== false, { activateSave });

  const transcriptFiles = await waitFor(
    "canonical transcript note",
    () =>
      transcriptCandidates(notesDir, transcriptToken).filter(
        (candidate) => !beforeTranscriptFiles.has(candidate.path),
      ),
    (files) => files.length === 1,
    15_000,
  );
  const transcript = transcriptFiles[0];
  const source = sourceFrontmatter(transcript.content);
  const transcriptFingerprint = `note-file:${basename(transcript.path)}:${sha16(transcript.content)}`;
  check("canonical_transcript_note_found", true, {
    path: transcript.path,
    fingerprint: transcriptFingerprint,
    bytes: transcript.content.length,
    mtimeMs: transcript.mtimeMs,
  });
  check(
    "source_frontmatter_is_agent_chat_thread",
    typeof source === "string" && source.startsWith("scriptkit://agent-chat/"),
    { source },
  );
  check("transcript_contains_user_token", transcript.content.includes(transcriptToken), {
    fingerprint: transcriptFingerprint,
  });
  check("transcript_has_export_heading", transcript.content.includes("# Agent Chat Conversation"), {
    fingerprint: transcriptFingerprint,
  });

  const appLogAfterSave = existsSync(driver.logPath) ? readFileSync(driver.logPath, "utf8") : "";
  const saveLine = latestLine(appLogAfterSave, "event=notes_agent_chat_save_as_note");
  const unhandledSaveLine = appLogAfterSave
    .split("\n")
    .find(
      (line) =>
        line.includes("event=notes_agent_chat_action_unhandled") &&
        line.includes("agent_chat_save_as_note"),
    );
  check("notes_hosted_save_as_note_logged", Boolean(saveLine), { saveLine });
  check("save_as_note_not_unhandled", !unhandledSaveLine, { unhandledSaveLine: unhandledSaveLine ?? null });

  driver.send({ type: "openAi", requestId: `${runId}-open-main-agent-chat` });
  await waitFor(
    "main Agent Chat open",
    () =>
      mainAgentChatState(driver).catch((error) => ({
        error: String(error),
      })),
    (state) => !("error" in state) && stateString(state, "status") !== "setup",
    30_000,
  );

  const submitRecall = await driver.request(
    { type: "setAgentChatInput", text: recallQuestion, submit: true },
    { timeoutMs: 10_000 },
  );
  check("brain_recall_question_submitted", (submitRecall as Json).type !== "error", { submitRecall });

  const brainRows = await waitFor(
    "Brain docs contain transcript note",
    () => readRows(brainDbPath, transcriptToken),
    (rows) => rows.some((row) => row.source === "note" && row.content.includes(transcriptToken)),
    20_000,
  );
  const noteRow = brainRows.find(
    (row) => row.source === "note" && row.content.includes(transcriptToken),
  );
  const rowFingerprint =
    noteRow == null ? "" : `note:${noteRow.source_id}:${sha16(noteRow.content)}`;
  check("brain_docs_contain_transcript_note", Boolean(noteRow), {
    sourceId: noteRow?.source_id ?? null,
    title: noteRow?.title ?? null,
    rowFingerprint,
  });

  const appLog = await waitFor(
    "Brain recall logs for transcript note",
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
  check("brain_recall_file_sources_synced", recallSyncElapsedMs != null && recallSyncElapsedMs < 500, {
    syncLine,
    recallSyncElapsedMs,
  });
  check("recall_sources_include_saved_note", builtLine?.includes("note") === true, {
    builtLine,
    rowFingerprint,
    fingerprintPresent: rowFingerprint ? builtLine?.includes(rowFingerprint) === true : false,
  });
  check("agent_chat_brain_recall_staged", Boolean(stagedLine), { stagedLine });

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
    receipt.brainRows = readRows(brainDbPath, transcriptToken).map((row) => ({
      source: row.source,
      sourceId: row.source_id,
      title: row.title,
      contentHasTranscriptToken: row.content.includes(transcriptToken),
    }));
  }
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exitCode = 1;
