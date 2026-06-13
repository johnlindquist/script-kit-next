#!/usr/bin/env bun
/**
 * Runtime proof: Day Page markdown is canonical, the brain index is derived
 * from that file, and the default Brain-profile Agent Chat stages recall from
 * the derived day-page doc. This probe must not seed brain.sqlite.
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/file-derived-recall/script-kit-gpui \
 *     bun scripts/agentic/brain-agent-chat-recall-probe.ts
 */
import { Database } from "bun:sqlite";
import { Driver } from "../devtools/driver.ts";
import { openDayPage } from "./day-page-open-helper.ts";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
} from "node:fs";
import { createHash } from "node:crypto";
import { homedir } from "node:os";
import { join } from "node:path";

const binary =
  process.env.PROBE_BINARY ??
  process.argv[2] ??
  "target-agent/artifacts/file-derived-recall/script-kit-gpui";
const timezone = process.env.SCRIPT_KIT_BRAIN_TZ || "America/Denver";
const runId = `file-derived-recall-${Date.now().toString(36)}`;
const uniqueTopic = `calico-lighthouse-${runId}`;
const uniqueAnswer = "49217";
const fact = `The ${uniqueTopic} handoff port is ${uniqueAnswer}.`;
const question = `What is the ${uniqueTopic} handoff port?`;

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

function sha256(text: string): string {
  return createHash("sha256").update(text).digest("hex");
}

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 30000,
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
  return waitFor(
    label,
    () =>
      agentChatState(driver).catch((error) => ({
        error: String(error),
      })),
    predicate,
    timeoutMs,
  );
}

function readDayPageRows(dbPath: string) {
  if (!existsSync(dbPath)) return [];
  const db = new Database(dbPath, { readonly: true });
  try {
    return db
      .query(
        `SELECT id, source_id, title, content, updated_at
         FROM brain_docs
         WHERE source = 'day_page' AND content LIKE ?1
         ORDER BY updated_at DESC, id DESC`,
      )
      .all(`%${uniqueTopic}%`) as Array<{
      id: number;
      source_id: string;
      title: string;
      content: string;
      updated_at: number;
    }>;
  } catch {
    return [];
  } finally {
    db.close();
  }
}

function readChatTurnRows(dbPath: string) {
  if (!existsSync(dbPath)) return [];
  const db = new Database(dbPath, { readonly: true });
  try {
    return db
      .query(
        `SELECT source_id, title, content
         FROM brain_docs
         WHERE source = 'chat_turn'
         ORDER BY updated_at DESC, id DESC`,
      )
      .all() as Array<{ source_id: string; title: string; content: string }>;
  } catch {
    return [];
  } finally {
    db.close();
  }
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

function appLogContains(driver: Driver, ...needles: string[]) {
  if (!existsSync(driver.logPath)) return false;
  const log = readFileSync(driver.logPath, "utf8");
  return needles.every((needle) => log.includes(needle));
}

const driver = await Driver.launch({
  sessionName: "brain-agent-chat-file-derived-recall",
  sandboxHome: true,
  binary,
  readyTimeoutMs: 15000,
  defaultTimeoutMs: 10000,
  env: {
    SCRIPT_KIT_BRAIN_TZ: timezone,
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});

const sandboxHome = join(driver.sessionDir, "home");
const skPath = join(sandboxHome, ".scriptkit");
const brainDbPath = join(skPath, "db", "brain.sqlite");
const localDate = localDateFor(new Date(), timezone);
const dayFile = join(skPath, "brain", "days", `${localDate}.md`);
copyAuthIntoSandbox(sandboxHome);

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "brain-agent-chat-recall-probe",
  classification: "blocked",
  seededDb: false,
  binary,
  timezone,
  sessionDir: driver.sessionDir,
  appLog: driver.logPath,
  canonicalDayFile: dayFile,
  derivedDbPath: brainDbPath,
  question,
};

try {
  const beforeRows = readDayPageRows(brainDbPath);
  receipt.preWriteDerivedRows = beforeRows.length;

  const dayState = await openDayPage(driver, runId);
  receipt.dayPageOpened = {
    promptType: dayState.promptType,
    windowVisible: dayState.windowVisible,
  };

  const setDayPage = await driver.batch(
    [{ type: "setInput", text: `# ${localDate}\n\n${fact}\n` }],
    { timeoutMs: 10000 },
  );
  receipt.setDayPage = setDayPage;
  driver.simulateKey("s", ["cmd"]);

  const dayContent = await waitFor(
    "canonical day page file",
    () => (existsSync(dayFile) ? readFileSync(dayFile, "utf8") : ""),
    (content) => content.includes(fact),
    10000,
  );
  const dayStat = statSync(dayFile);
  receipt.canonicalDayFileProof = {
    exists: true,
    bytes: dayContent.length,
    hash: sha256(dayContent),
    containsUniqueTopic: dayContent.includes(uniqueTopic),
    containsUniqueAnswer: dayContent.includes(uniqueAnswer),
    mtimeMs: dayStat.mtimeMs,
  };

  const indexedRows = await waitFor(
    "derived day_page brain doc",
    () => readDayPageRows(brainDbPath),
    (rows) => rows.some((row) => row.content.includes(fact)),
    20000,
  );
  const indexedRow = indexedRows.find((row) => row.content.includes(fact));
  receipt.derivedDayPageProof = {
    rows: indexedRows.length,
    sourceId: indexedRow?.source_id ?? null,
    title: indexedRow?.title ?? null,
    contentHasUniqueTopic: indexedRow?.content.includes(uniqueTopic) ?? false,
    contentHasUniqueAnswer: indexedRow?.content.includes(uniqueAnswer) ?? false,
    updatedAt: indexedRow?.updated_at ?? null,
  };

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
  receipt.setAgentChatInput = setInput;

  await waitFor(
    "brain recall staged log",
    () => appLogContains(driver, "agent_chat_brain_recall_staged", uniqueTopic),
    Boolean,
    30000,
  );
  receipt.brainRecallStaged = {
    logged: true,
    containsUniqueTopic: true,
  };

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
  const rows = readChatTurnRows(brainDbPath);
  const matchingTurn = rows.find((row) => row.content.includes(question));
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

  const screenshotPath = ".test-screenshots/brain-agent-chat-file-derived-recall.png";
  const screenshot = await driver
    .captureScreenshot({ savePath: screenshotPath, timeoutMs: 15000 })
    .catch((error) => ({ error: String(error) }));
  receipt.screenshot = screenshot.error == null ? screenshotPath : screenshot;

  if (beforeRows.length !== 0) {
    throw new Error("fresh sandbox derived DB already contained the unique day-page fact");
  }
  if (!indexedRow) {
    throw new Error("canonical day page was not mirrored into derived brain_docs");
  }
  if (!matchingTurn) {
    throw new Error("completed Agent Chat turn was not ingested into brain_docs");
  }
  if (!recallAnswerMatched) {
    throw new Error("assistant/turn evidence did not reflect file-derived recall");
  }

  receipt.classification = "completed";
} catch (error) {
  receipt.error = String(error);
  receipt.conversations = conversationDigest(skPath);
  receipt.dayPageRows = readDayPageRows(brainDbPath).map((row) => ({
    sourceId: row.source_id,
    title: row.title,
    contentHasUniqueTopic: row.content.includes(uniqueTopic),
    contentHasUniqueAnswer: row.content.includes(uniqueAnswer),
  }));
  receipt.chatTurnRows = readChatTurnRows(brainDbPath).map((row) => ({
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
