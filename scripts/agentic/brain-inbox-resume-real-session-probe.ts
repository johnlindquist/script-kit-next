#!/usr/bin/env bun
/**
 * Phase 2 of the Brain Inbox → Agent Chat UX proof: what happens when a
 * chat_turn-sourced inbox item resumes a REAL saved conversation?
 *
 * Hypothesis from source (selection_fallback.rs:723 →
 * render_builtins/agent_chat_history.rs:501): on resume success the
 * follow-up prompt built by response_prompt_for_inbox_item is passed as
 * `first_message` but only used when resume FAILS — so the user lands in the
 * old conversation with an empty composer (prompt silently dropped), plus a
 * stray staged Selection/@cmd context chip from the non-suppressing entry.
 *
 * Seeds a saved conversation fixture (from a prior live probe run) into the
 * sandbox, files one chat_turn inbox item referencing it, presses Enter on
 * the row, and scrapes the resulting chat state.
 *
 * Usage: bun scripts/agentic/brain-inbox-resume-real-session-probe.ts \
 *          [binaryPath] [fixtureConversationJson]
 */
import { Driver } from "../devtools/driver.ts";
import { Database } from "bun:sqlite";
import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { basename, join } from "node:path";
import { homedir } from "node:os";

const binary =
  process.argv[2] ?? "target-agent/artifacts/brain-inbox-probe/script-kit-gpui";
const fixture =
  process.argv[3] ??
  "/tmp/sk-driver-sessions/brain-inbox-probe-32372-1-mq8tva78/home/.scriptkit/agent_chat-conversations/warm:ea53befa-2398-4e9f-a78a-12b365cbdb5b.json";
if (!existsSync(fixture)) {
  console.error(`fixture conversation not found: ${fixture}`);
  process.exit(2);
}
const sessionId = basename(fixture).replace(/\.json$/, "");

// ---------------------------------------------------------------- seed db
const seedDir = "/tmp/sk-brain-inbox-resume-probe";
rmSync(seedDir, { recursive: true, force: true });
mkdirSync(seedDir, { recursive: true });
const dbPath = join(seedDir, "brain.sqlite");
const db = new Database(dbPath);
db.exec(`
CREATE TABLE IF NOT EXISTS brain_inbox (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    kind TEXT NOT NULL,
    title TEXT NOT NULL,
    detail TEXT NOT NULL DEFAULT '',
    source TEXT NOT NULL DEFAULT '',
    source_id TEXT NOT NULL DEFAULT '',
    dedupe_hash TEXT NOT NULL UNIQUE,
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    resolved_at INTEGER
);
CREATE INDEX IF NOT EXISTS idx_brain_inbox_open
    ON brain_inbox(resolved_at, created_at DESC);
`);
db.prepare(
  `INSERT INTO brain_inbox (kind, title, detail, source, source_id, dedupe_hash, created_at)
   VALUES (?, ?, ?, ?, ?, ?, ?)`,
).run(
  "commitment",
  "Promised Alice the demo build in chat",
  "Commitment made in an agent chat conversation",
  "chat_turn",
  `${sessionId}#1`,
  "hash-resume-1",
  Math.floor(Date.now() / 1000) - 60,
);
db.close();

// ---------------------------------------------------------------- launch
const driver = await Driver.launch({
  sessionName: "brain-inbox-resume-probe",
  sandboxHome: true,
  binary,
  env: {
    SCRIPT_KIT_TEST_BRAIN_DB_PATH: dbPath,
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});
const sandboxHome = join(driver.sessionDir, "home");
const skPath = join(sandboxHome, ".scriptkit");

for (const rel of [".pi/agent/auth.json", ".pi/agent/settings.json", ".codex/auth.json"]) {
  const src = join(homedir(), rel);
  if (existsSync(src)) {
    const dst = join(sandboxHome, rel);
    mkdirSync(join(dst, ".."), { recursive: true });
    cpSync(src, dst);
  }
}
// Seed the saved conversation the inbox item points at.
const convDir = join(skPath, "agent_chat-conversations");
mkdirSync(convDir, { recursive: true });
cpSync(fixture, join(convDir, basename(fixture)));

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "brain-inbox-resume-real-session-probe",
  binary,
  sessionId,
  sessionDir: driver.sessionDir,
  classification: "blocked",
};

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function agentChatState(): Promise<Record<string, unknown>> {
  const result = (await driver.request(
    { type: "getAgentChatState" },
    { timeoutMs: 10000 },
  )) as Record<string, unknown>;
  return (result.state ?? result) as Record<string, unknown>;
}

function chatStateDigest(s: Record<string, unknown>) {
  return {
    status: s.status,
    inputText: s.inputText ?? s.input_text,
    messageCount: s.messageCount ?? s.message_count,
    contextChipCount: s.contextChipCount ?? s.context_chip_count,
    contextSummary: s.contextSummary ?? s.context_summary,
    retainedThreadCount: s.retainedThreadCount ?? s.retained_thread_count,
    warnings: s.warnings,
  };
}

try {
  await driver.setFilterAndWait("zz");
  await driver.setFilterAndWait("");
  await sleep(800);

  const state = (await driver.getState()) as Record<string, unknown>;
  const preflight = (state.mainWindowPreflight ?? {}) as Record<string, unknown>;
  const footer = (state.activeFooter ?? {}) as Record<string, unknown>;
  receipt.menuBefore = {
    selectedValue: state.selectedValue,
    selectedResultKey: preflight.selectedResultKey,
    footerButtons: ((footer.buttons ?? []) as Array<Record<string, unknown>>).map(
      (b) => `${b.key} ${b.label}`,
    ),
  };
  if (preflight.selectedResultKey !== "brain-inbox/1") {
    throw new Error(`inbox row not selected: ${JSON.stringify(receipt.menuBefore)}`);
  }

  driver.simulateKey("enter", []);
  await sleep(1500);
  const after = await agentChatState();
  receipt.chatAfterEnter = chatStateDigest(after);

  // Green expectation: 2 resumed messages + auto-submitted follow-up turn
  // (user + assistant) = 4, idle, clean composer.
  const deadline = Date.now() + 120000;
  let settled: Record<string, unknown> = after;
  while (Date.now() < deadline) {
    settled = await agentChatState();
    if (
      String(settled.status) === "idle" &&
      Number(settled.messageCount ?? settled.message_count ?? 0) >= 4
    ) {
      break;
    }
    await sleep(500);
  }
  receipt.chatSettled = chatStateDigest(settled);

  const input = String(settled.inputText ?? settled.input_text ?? "");
  const messageCount = Number(
    settled.messageCount ?? settled.message_count ?? 0,
  );
  receipt.findings = {
    resumedOldConversation: messageCount >= 2,
    followUpAutoSubmitted:
      messageCount >= 4 && String(settled.status) === "idle",
    composerClean: input.trim().length === 0 && !input.includes("@cmd"),
    strayContextChips: String(
      settled.contextSummary ?? settled.context_summary ?? "",
    ),
  };
  const findings = receipt.findings as Record<string, unknown>;
  receipt.classification =
    findings.resumedOldConversation === true &&
    findings.followUpAutoSubmitted === true &&
    findings.composerClean === true
      ? "fixed"
      : "reproduced-failure";
} catch (error) {
  receipt.error = String(error);
  receipt.classification = "blocked";
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
