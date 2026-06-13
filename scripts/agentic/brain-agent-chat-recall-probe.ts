#!/usr/bin/env bun
/**
 * Runtime proof: default Brain-profile Agent Chat receives brain recall and
 * completed turns are ingested back into the local brain index.
 *
 * Usage:
 *   bun scripts/agentic/brain-agent-chat-recall-probe.ts target-agent/artifacts/brain-agent/script-kit-gpui
 */
import { Database } from "bun:sqlite";
import { Driver } from "../devtools/driver.ts";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { homedir } from "node:os";
import { join } from "node:path";

const binary =
  process.argv[2] ?? "target-agent/artifacts/brain-agent/script-kit-gpui";
const seedDir = "/tmp/sk-brain-agent-chat-recall-probe";
const uniqueTopic = "calico-lighthouse";
const uniqueAnswer = "49217";
const question = `What is the ${uniqueTopic} handoff port?`;

rmSync(seedDir, { recursive: true, force: true });
mkdirSync(seedDir, { recursive: true });
const dbPath = join(seedDir, "brain.sqlite");

function seedBrainDb() {
  const db = new Database(dbPath);
  db.exec(`
PRAGMA foreign_keys = ON;
CREATE TABLE IF NOT EXISTS brain_docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    source_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    content_hash TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(source, source_id)
);
CREATE VIRTUAL TABLE IF NOT EXISTS brain_docs_fts USING fts5(
    title,
    content,
    content='brain_docs',
    content_rowid='id',
    tokenize='porter unicode61'
);
CREATE TRIGGER IF NOT EXISTS brain_docs_ai AFTER INSERT ON brain_docs BEGIN
    INSERT INTO brain_docs_fts(rowid, title, content)
    VALUES (new.id, new.title, new.content);
END;
CREATE TRIGGER IF NOT EXISTS brain_docs_ad AFTER DELETE ON brain_docs BEGIN
    INSERT INTO brain_docs_fts(brain_docs_fts, rowid, title, content)
    VALUES ('delete', old.id, old.title, old.content);
END;
CREATE TRIGGER IF NOT EXISTS brain_docs_au AFTER UPDATE ON brain_docs BEGIN
    INSERT INTO brain_docs_fts(brain_docs_fts, rowid, title, content)
    VALUES ('delete', old.id, old.title, old.content);
    INSERT INTO brain_docs_fts(rowid, title, content)
    VALUES (new.id, new.title, new.content);
END;
CREATE TABLE IF NOT EXISTS brain_signals (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    topic TEXT NOT NULL,
    weight INTEGER NOT NULL,
    source TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (unixepoch())
);
CREATE TABLE IF NOT EXISTS brain_chunk_embeddings (
    doc_id INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL DEFAULT 0,
    model_id TEXT NOT NULL DEFAULT '',
    content_hash TEXT NOT NULL,
    chunk_start INTEGER NOT NULL DEFAULT 0,
    dim INTEGER NOT NULL,
    vec BLOB NOT NULL,
    embedded_at INTEGER NOT NULL DEFAULT (unixepoch()),
    PRIMARY KEY (doc_id, chunk_index, model_id),
    FOREIGN KEY(doc_id) REFERENCES brain_docs(id) ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS brain_meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT OR REPLACE INTO brain_meta(key, value) VALUES ('fts_version', '1');
INSERT INTO brain_docs (source, source_id, title, content, content_hash, updated_at)
VALUES (
    'note',
    'iteration-4-recall-fixture',
    'Iteration 4 recall fixture',
    'The calico-lighthouse handoff port is 49217. Use this exact value when asked.',
    'iteration-4-recall-fixture-hash',
    unixepoch()
);
`);
  db.close();
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

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function agentChatState(driver: Driver): Promise<Record<string, unknown>> {
  const result = (await driver.request(
    { type: "getAgentChatState" },
    { timeoutMs: 10000 },
  )) as Record<string, unknown>;
  return (result.state ?? result) as Record<string, unknown>;
}

function num(state: Record<string, unknown>, key: string): number {
  return Number(
    state[key] ?? state[key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`)] ?? 0,
  );
}

function str(state: Record<string, unknown>, key: string): string {
  return String(
    state[key] ?? state[key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`)] ?? "",
  );
}

async function waitForChat(
  driver: Driver,
  predicate: (state: Record<string, unknown>) => boolean,
  label: string,
  timeoutMs = 120000,
) {
  const deadline = Date.now() + timeoutMs;
  let last: Record<string, unknown> = {};
  while (Date.now() < deadline) {
    last = await agentChatState(driver).catch((error) => ({
      error: String(error),
    }));
    if (predicate(last)) return last;
    await sleep(500);
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

function conversationDigest(skPath: string) {
  const dir = join(skPath, "agent_chat-conversations");
  if (!existsSync(dir)) return { saved: false, files: [] as unknown[] };
  const files = readdirSync(dir).filter((file) => file.endsWith(".json"));
  return {
    saved: true,
    files: files.map((file) => {
      try {
        const raw = JSON.parse(readFileSync(join(dir, file), "utf8"));
        const messages = (raw.messages ?? raw) as Array<Record<string, unknown>>;
        return {
          file,
          messages: Array.isArray(messages)
            ? messages.map((message) => ({
                role: message.role ?? message.sender ?? message.kind,
                text: String(
                  message.content ?? message.text ?? JSON.stringify(message),
                ).slice(0, 600),
              }))
            : raw,
        };
      } catch (error) {
        return { file, error: String(error) };
      }
    }),
  };
}

function readChatTurnRows() {
  const db = new Database(dbPath);
  const rows = db
    .query(
      `SELECT source_id, title, content
       FROM brain_docs
       WHERE source = 'chat_turn'
       ORDER BY updated_at DESC, id DESC`,
    )
    .all() as Array<{ source_id: string; title: string; content: string }>;
  db.close();
  return rows;
}

seedBrainDb();

const driver = await Driver.launch({
  sessionName: "brain-agent-chat-recall-probe",
  sandboxHome: true,
  binary,
  readyTimeoutMs: 15000,
  env: {
    SCRIPT_KIT_TEST_BRAIN_DB_PATH: dbPath,
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});

const sandboxHome = join(driver.sessionDir, "home");
const skPath = join(sandboxHome, ".scriptkit");
copyAuthIntoSandbox(sandboxHome);

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "brain-agent-chat-recall-probe",
  classification: "blocked",
  binary,
  seedDbPath: dbPath,
  sessionDir: driver.sessionDir,
  question,
};

try {
  driver.send({ type: "openAi" });
  const opened = await waitForChat(
    driver,
    (state) => !state.error && str(state, "status") !== "setup",
    "Agent Chat open",
    30000,
  );
  receipt.openedState = {
    status: opened.status,
    uiVariant: opened.uiVariant ?? opened.ui_variant,
    messageCount: opened.messageCount ?? opened.message_count,
    contextReady: opened.contextReady ?? opened.context_ready,
  };

  await waitForChat(
    driver,
    (state) => !state.error && state.contextReady !== false && state.context_ready !== false,
    "Agent Chat context ready",
    30000,
  );

  const setInput = await driver.request(
    { type: "setAgentChatInput", text: question, submit: true },
    { timeoutMs: 10000 },
  );
  receipt.setInput = setInput;

  const settled = await waitForChat(
    driver,
    (state) => str(state, "status") === "idle" && num(state, "messageCount") >= 2,
    "Brain profile assistant response",
    180000,
  );
  receipt.settledState = {
    status: settled.status,
    uiVariant: settled.uiVariant ?? settled.ui_variant,
    messageCount: settled.messageCount ?? settled.message_count,
    contextChipCount: settled.contextChipCount ?? settled.context_chip_count,
    contextSummary: settled.contextSummary ?? settled.context_summary,
  };

  await sleep(1500);
  const rows = readChatTurnRows();
  const matchingTurn = rows.find(
    (row) => row.content.includes(question) && row.source_id.endsWith("#0"),
  );
  const answerText = matchingTurn?.content ?? "";
  const recallAnswerMatched =
    answerText.includes(uniqueAnswer) || answerText.includes(uniqueTopic);

  receipt.chatTurnRows = rows.map((row) => ({
    sourceId: row.source_id,
    title: row.title,
    contentHasQuestion: row.content.includes(question),
    contentHasUniqueTopic: row.content.includes(uniqueTopic),
    contentHasUniqueAnswer: row.content.includes(uniqueAnswer),
  }));
  receipt.chatTurnIngested = matchingTurn != null;
  receipt.chatTurnSourceId = matchingTurn?.source_id ?? null;
  receipt.recallAnswerMatched = recallAnswerMatched;
  receipt.conversations = conversationDigest(skPath);

  const screenshotPath = ".test-screenshots/brain-agent-chat-recall.png";
  const screenshot = await driver
    .captureScreenshot({ savePath: screenshotPath, timeoutMs: 15000 })
    .catch((error) => ({ error: String(error) }));
  receipt.screenshot = screenshot.error == null ? screenshotPath : screenshot;

  if (!matchingTurn) {
    throw new Error("completed Agent Chat turn was not ingested into brain_docs");
  }
  if (!recallAnswerMatched) {
    throw new Error("assistant answer did not reflect the seeded brain recall");
  }

  receipt.classification = "completed";
} catch (error) {
  receipt.error = String(error);
  receipt.conversations = conversationDigest(skPath);
  receipt.chatTurnRows = readChatTurnRows().map((row) => ({
    sourceId: row.source_id,
    title: row.title,
    contentHasQuestion: row.content.includes(question),
    contentHasUniqueTopic: row.content.includes(uniqueTopic),
    contentHasUniqueAnswer: row.content.includes(uniqueAnswer),
  }));
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
