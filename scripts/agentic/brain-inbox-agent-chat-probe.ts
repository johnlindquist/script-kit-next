#!/usr/bin/env bun
/**
 * Runtime UX proof: Brain Inbox rows in the main menu → Agent Chat handoff.
 *
 * Seeds a sandbox brain.sqlite (SCRIPT_KIT_TEST_BRAIN_DB_PATH) with three open
 * inbox items, then drives the REAL user path with live Pi turns:
 *
 *   Iteration 1 — clipboard-sourced Commitment (top row, empty query, plain
 *   Enter). Expect: item resolves, Agent Chat opens, the follow-up prompt
 *   auto-submits, NO @cmd context chip, assistant replies.
 *
 *   Iteration 2 — capture-sourced Question (now the top row). Expect to learn
 *   whether the reused embedded chat CONTINUES the previous conversation
 *   (messageCount keeps growing) instead of starting fresh.
 *
 *   Iteration 3 — chat_turn-sourced Drift item with an unknown session id.
 *   This is the user-reported repro: the resume path enters via the
 *   non-suppressing entry, so we scrape for a stray `@cmd:` context chip and
 *   a "continue conversation" composer state (prompt parked, not submitted).
 *
 * Scrapes at every step: getAgentChatState (composer text, chips, counts,
 * status), getState (main filter input), getElements (top rows/selection),
 * and the saved conversation JSON for actual user/assistant message text.
 *
 * Usage: bun scripts/agentic/brain-inbox-agent-chat-probe.ts [binaryPath]
 */
import { Driver } from "../devtools/driver.ts";
import { Database } from "bun:sqlite";
import {
  cpSync,
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  rmSync,
} from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

const binary =
  process.argv[2] ?? "target-agent/artifacts/brain-inbox-probe/script-kit-gpui";

// ---------------------------------------------------------------- seed db
const seedDir = "/tmp/sk-brain-inbox-probe";
rmSync(seedDir, { recursive: true, force: true });
mkdirSync(seedDir, { recursive: true });
const dbPath = join(seedDir, "brain.sqlite");
const db = new Database(dbPath);
db.exec(`
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
const now = Math.floor(Date.now() / 1000);
const insertItem = db.prepare(
  `INSERT INTO brain_inbox (kind, title, detail, source, source_id, dedupe_hash, created_at)
   VALUES (?, ?, ?, ?, ?, ?, ?)`,
);
// Newest first in the launcher: iteration order = A, B, C.
insertItem.run(
  "commitment",
  "Send Alice the updated vibrancy demo build",
  "You told Alice you'd send the new build once the tint fix landed",
  "clipboard",
  "clip-demo-1",
  "hash-a",
  now - 60,
);
insertItem.run(
  "question",
  "Which backdrop saturation value shipped?",
  "Open question from yesterday's vibrancy research",
  "capture",
  "cap-demo-2",
  "hash-b",
  now - 120,
);
insertItem.run(
  "drift",
  "Perf numbers promised in chat never sent",
  "You promised benchmark numbers in an agent chat two days ago",
  "chat_turn",
  "fake-session-999#2",
  "hash-c",
  now - 180,
);
db.prepare(
  `INSERT INTO brain_docs (source, source_id, title, content, content_hash)
   VALUES (?, ?, ?, ?, ?)`,
).run(
  "clipboard",
  "clip-demo-1",
  "Message from Alice",
  "Alice: hey, can you send me the vibrancy demo build when the tint fix is in? I want to show it at standup tomorrow.",
  "hash-doc-1",
);
db.close();

// ---------------------------------------------------------------- launch
const driver = await Driver.launch({
  sessionName: "brain-inbox-probe",
  sandboxHome: true,
  binary,
  env: {
    SCRIPT_KIT_TEST_BRAIN_DB_PATH: dbPath,
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});
const sandboxHome = join(driver.sessionDir, "home");
const skPath = join(sandboxHome, ".scriptkit");

// Live Pi turns need real auth seeded into the sandbox HOME (launch wiped it).
for (const rel of [".pi/agent/auth.json", ".pi/agent/settings.json", ".codex/auth.json"]) {
  const src = join(homedir(), rel);
  if (existsSync(src)) {
    const dst = join(sandboxHome, rel);
    mkdirSync(join(dst, ".."), { recursive: true });
    cpSync(src, dst);
  }
}

const receipt: Record<string, unknown> = {
  schemaVersion: 1,
  tool: "brain-inbox-agent-chat-probe",
  binary,
  sessionDir: driver.sessionDir,
  classification: "blocked",
  iterations: [] as Array<Record<string, unknown>>,
};
const iterations = receipt.iterations as Array<Record<string, unknown>>;

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
    awaitingFirstAssistantText:
      s.awaitingFirstAssistantText ?? s.awaiting_first_assistant_text,
    retainedThreadCount: s.retainedThreadCount ?? s.retained_thread_count,
    uiVariant: s.uiVariant ?? s.ui_variant,
    warnings: s.warnings,
  };
}

async function waitForChat(
  predicate: (s: Record<string, unknown>) => boolean,
  label: string,
  timeoutMs = 90000,
): Promise<Record<string, unknown>> {
  const deadline = Date.now() + timeoutMs;
  let last: Record<string, unknown> = {};
  while (Date.now() < deadline) {
    last = await agentChatState();
    if (predicate(last)) return last;
    await sleep(400);
  }
  throw new Error(
    `timeout waiting for ${label}: ${JSON.stringify(chatStateDigest(last))}`,
  );
}

function num(s: Record<string, unknown>, key: string): number {
  return Number(
    s[key] ?? s[key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`)] ?? 0,
  );
}
function str(s: Record<string, unknown>, key: string): string {
  return String(
    s[key] ?? s[key.replace(/[A-Z]/g, (c) => `_${c.toLowerCase()}`)] ?? "",
  );
}

/** Top visible rows + selection from the main window semantic tree. */
async function mainMenuDigest() {
  const elements = (await driver.getElements()) as Record<string, unknown>;
  const nodes = (elements.elements ?? []) as Array<Record<string, unknown>>;
  const rows = nodes
    .filter((n) => n.type === "choice")
    .slice(0, 6)
    .map((n) => ({
      id: n.semanticId,
      text: n.text,
      kind: n.kind,
      selected: n.selected === true,
    }));
  const inboxRows = nodes.filter((n) => n.kind === "brain inbox");
  const state = (await driver.getState()) as Record<string, unknown>;
  const footer = (state.activeFooter ?? {}) as Record<string, unknown>;
  return {
    filter: String(state.inputValue ?? ""),
    selectedValue: state.selectedValue ?? null,
    selectedResultKey:
      ((state.mainWindowPreflight ?? {}) as Record<string, unknown>)
        .selectedResultKey ?? null,
    footerButtons: ((footer.buttons ?? []) as Array<Record<string, unknown>>).map(
      (b) => `${b.key} ${b.label}`,
    ),
    topRows: rows,
    inboxRowCount: inboxRows.length,
    inboxRows: inboxRows.map((n) => ({
      id: n.semanticId,
      text: n.text,
      subtitle: n.value,
      selected: n.selected === true,
    })),
  };
}

async function mainFilterText(): Promise<string> {
  const state = (await driver.getState()) as Record<string, unknown>;
  return String((state as Record<string, unknown>).inputValue ?? "");
}

/** Read saved conversation files for real user/assistant message text. */
function conversationDigest() {
  const dir = join(skPath, "agent_chat-conversations");
  if (!existsSync(dir)) return { saved: false, files: [] as unknown[] };
  const files = readdirSync(dir).filter((f) => f.endsWith(".json"));
  const convos = files.map((f) => {
    try {
      const raw = JSON.parse(readFileSync(join(dir, f), "utf8"));
      const messages = (raw.messages ?? raw) as Array<Record<string, unknown>>;
      const digest = Array.isArray(messages)
        ? messages.map((m) => ({
            role: m.role ?? m.sender ?? m.kind,
            text: String(m.content ?? m.text ?? JSON.stringify(m)).slice(0, 400),
          }))
        : raw;
      return { file: f, messages: digest };
    } catch (e) {
      return { file: f, error: String(e) };
    }
  });
  return { saved: true, files: convos };
}

async function screenshotBestEffort(savePath: string) {
  mkdirSync(".test-screenshots", { recursive: true });
  const shot = (await driver
    .captureScreenshot({ savePath, timeoutMs: 15000 })
    .catch((e) => ({ error: String(e) }))) as Record<string, unknown>;
  if (shot.error == null) return { ok: true, via: "captureScreenshot", savePath };
  const os = Bun.spawnSync(["screencapture", "-x", "-D", "1", savePath]);
  return {
    ok: os.exitCode === 0 && existsSync(savePath),
    via: "screencapture",
    inAppError: shot.error,
  };
}

async function backToMainMenu(label: string) {
  // Escape leaves the embedded agent chat back to the launcher.
  driver.simulateKey("escape", []);
  await sleep(600);
  // Nudge the filter to retrigger the brain-inbox snapshot refresh hooks.
  await driver.setFilterAndWait("zz").catch(() => {});
  await driver.setFilterAndWait("").catch(() => {});
  await sleep(400);
  const digest = await mainMenuDigest();
  const filter = await mainFilterText();
  return { label, filter, ...digest };
}

try {
  // ---- orientation: load the inbox snapshot via the filter-change hook.
  await driver.setFilterAndWait("zz");
  await driver.setFilterAndWait("");
  await sleep(800);

  const initialMenu = await mainMenuDigest();
  const initialFilter = await mainFilterText();
  receipt.initialMenu = { filter: initialFilter, ...initialMenu };
  const shotMain = await screenshotBestEffort(
    ".test-screenshots/brain-inbox-main-menu.png",
  );
  receipt.mainMenuScreenshot = shotMain;

  if (initialMenu.inboxRowCount === 0) {
    throw new Error(
      `brain inbox section never appeared: ${JSON.stringify(initialMenu)}`,
    );
  }

  // ================================================================ iter 1
  // Plain Enter on the top brain-inbox row (clipboard-sourced commitment).
  {
    const iter: Record<string, unknown> = { name: "iter1-clipboard-commitment" };
    iter.menuBefore = await mainMenuDigest();
    driver.simulateKey("enter", []);

    // Watch the handoff: collect early state (catches @cmd chips / drafts
    // before submit), then wait for the live turn to finish.
    await sleep(500);
    iter.chatEarly = chatStateDigest(await agentChatState());
    const settled = await waitForChat(
      (s) => str(s, "status") === "idle" && num(s, "messageCount") >= 2,
      "iter1 assistant reply",
      120000,
    );
    iter.chatSettled = chatStateDigest(settled);
    iter.screenshot = await screenshotBestEffort(
      ".test-screenshots/brain-inbox-iter1-chat.png",
    );
    iterations.push(iter);
  }

  // ---- back to launcher; inbox should have shrunk to 2 rows.
  const menuAfter1 = await backToMainMenu("after-iter1");
  iterations.push({ name: "menu-after-iter1", ...menuAfter1 });

  // ================================================================ iter 2
  // Enter on the (new) top row: capture-sourced question. The embedded chat
  // already holds conversation 1 — does this CONTINUE it?
  {
    const iter: Record<string, unknown> = { name: "iter2-capture-question" };
    iter.menuBefore = await mainMenuDigest();
    const before = chatStateDigest(await agentChatState());
    iter.chatBeforeEnter = before;
    driver.simulateKey("enter", []);
    await sleep(500);
    iter.chatEarly = chatStateDigest(await agentChatState());
    const baseline = Number(before.messageCount ?? 0);
    const settled = await waitForChat(
      (s) =>
        str(s, "status") === "idle" && num(s, "messageCount") >= baseline + 2,
      "iter2 assistant reply",
      120000,
    );
    iter.chatSettled = chatStateDigest(settled);
    iter.continuedSameConversation = num(settled, "messageCount") > 2;
    iter.screenshot = await screenshotBestEffort(
      ".test-screenshots/brain-inbox-iter2-chat.png",
    );
    iterations.push(iter);
  }

  const menuAfter2 = await backToMainMenu("after-iter2");
  iterations.push({ name: "menu-after-iter2", ...menuAfter2 });

  // ================================================================ iter 3
  // Enter on the chat_turn-sourced item with an unknown session id — the
  // user-reported repro. Resume fails → fallback parks the prompt in the
  // composer; the non-suppressing entry may stage an @cmd chip.
  {
    const iter: Record<string, unknown> = { name: "iter3-chat-turn-resume" };
    iter.menuBefore = await mainMenuDigest();
    driver.simulateKey("enter", []);
    await sleep(1200);
    const after = await agentChatState();
    iter.chatAfterEnter = chatStateDigest(after);
    iter.composerHasParkedPrompt = str(after, "inputText").includes(
      "Brain Inbox item",
    );
    iter.hasCmdChip =
      str(after, "contextSummary").includes("@cmd") ||
      num(after, "contextChipCount") > 0;
    // Give any async staging a moment, then re-scrape.
    await sleep(1500);
    const later = await agentChatState();
    iter.chatLater = chatStateDigest(later);
    iter.screenshot = await screenshotBestEffort(
      ".test-screenshots/brain-inbox-iter3-chat.png",
    );
    iterations.push(iter);
  }

  receipt.conversations = conversationDigest();
  receipt.classification = "completed";
} catch (error) {
  receipt.error = String(error);
  receipt.conversations = conversationDigest();
  receipt.classification = "blocked";
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
