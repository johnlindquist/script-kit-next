#!/usr/bin/env bun
/**
 * Runtime UX proof: Escape must hide the window on the FIRST press once the
 * user is back on an empty main menu after an Agent Chat round trip.
 *
 * User-reported repro (2026-07-03):
 *   a. Enter on the first "brain" inbox item → embedded Agent Chat opens.
 *   b. Escape → back to the main menu (input empty).
 *   c. Escape → expected: window hides. Bug: a stale `opened_from_main_menu`
 *      makes this press run a no-op go_back_or_close ("ESC - returning to
 *      main menu (opened from main menu)" while already on ScriptList), so a
 *      THIRD Escape was needed.
 *
 * Green = exactly ONE Escape hides the window after returning to the menu.
 *
 * Usage: bun scripts/agentic/main-menu-escape-after-agent-chat-probe.ts [binaryPath]
 */
import { Driver } from "../devtools/driver.ts";
import { Database } from "bun:sqlite";
import { cpSync, existsSync, mkdirSync, rmSync } from "node:fs";
import { join } from "node:path";
import { homedir } from "node:os";

const binary =
  process.argv[2] ?? "target-agent/artifacts/escape-fix/script-kit-gpui";

// ---------------------------------------------------------------- seed db
// One chat_turn-sourced item with an unknown session id: the resume fails
// fast and parks the follow-up prompt in the composer — Agent Chat opens
// without a live streaming turn, so Escape is never consumed by
// cancel-streaming and the proof stays deterministic without Pi auth.
const seedDir = "/tmp/sk-escape-fix-probe";
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
db.prepare(
  `INSERT INTO brain_inbox (kind, title, detail, source, source_id, dedupe_hash, created_at)
   VALUES (?, ?, ?, ?, ?, ?, ?)`,
).run(
  "drift",
  "Perf numbers promised in chat never sent",
  "You promised benchmark numbers in an agent chat two days ago",
  "chat_turn",
  "fake-session-999#2",
  "hash-escape-probe",
  Math.floor(Date.now() / 1000) - 60,
);
db.close();

// ---------------------------------------------------------------- launch
const driver = await Driver.launch({
  sessionName: "escape-fix-probe",
  sandboxHome: true,
  binary,
  env: {
    SCRIPT_KIT_TEST_BRAIN_DB_PATH: dbPath,
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
});
const sandboxHome = join(driver.sessionDir, "home");
// Auth seeding keeps the Agent Chat surface on its normal path instead of a
// setup card (the probe never waits for a live reply).
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
  tool: "main-menu-escape-after-agent-chat-probe",
  binary,
  sessionDir: driver.sessionDir,
  classification: "blocked",
};

function sleep(ms: number) {
  return new Promise((r) => setTimeout(r, ms));
}

async function digest() {
  const state = (await driver.getState()) as Record<string, unknown>;
  const contract = (state.surfaceContract ?? {}) as Record<string, unknown>;
  return {
    windowVisible: state.windowVisible,
    promptType: String(state.promptType ?? ""),
    surface: String(contract.surface ?? contract.kind ?? contract.name ?? ""),
    inputValue: String(state.inputValue ?? ""),
  };
}

async function waitFor(
  predicate: (d: Awaited<ReturnType<typeof digest>>) => boolean,
  label: string,
  timeoutMs = 15000,
) {
  const deadline = Date.now() + timeoutMs;
  let last = await digest();
  while (Date.now() < deadline) {
    if (predicate(last)) return last;
    await sleep(250);
    last = await digest();
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

try {
  // The escape → hide contract only exists for a visible window.
  await driver.request({ type: "show" });
  await waitFor((d) => d.windowVisible === true, "window visible");

  // Load the brain-inbox snapshot via the filter-change hook.
  await driver.setFilterAndWait("zz");
  await driver.setFilterAndWait("");
  await sleep(800);
  receipt.menuBefore = await digest();

  // a. Enter on the top brain item → embedded Agent Chat.
  const menuPromptType = (receipt.menuBefore as { promptType: string }).promptType;
  driver.simulateKey("enter", []);
  const inChat = await waitFor(
    (d) => d.promptType !== menuPromptType,
    "agent chat open",
    20000,
  );
  receipt.afterEnter = inChat;
  await driver.waitForSettle().catch(() => {});

  // b. Escape until we're back on the main menu with an empty input.
  let escapesToReturn = 0;
  for (; escapesToReturn < 4; ) {
    driver.simulateKey("escape", []);
    escapesToReturn++;
    await sleep(500);
    const d = await digest();
    if (
      d.windowVisible !== false &&
      d.promptType === (receipt.menuBefore as { promptType: string }).promptType &&
      d.inputValue === ""
    ) {
      break;
    }
    if (d.windowVisible === false) {
      throw new Error(
        `window hid while escaping out of agent chat (escape #${escapesToReturn}): ${JSON.stringify(d)}`,
      );
    }
  }
  receipt.escapesToReturnToMenu = escapesToReturn;
  receipt.menuAfterReturn = await digest();

  // c. THE assertion: exactly one more Escape hides the window.
  driver.simulateKey("escape", []);
  let extraEscapes = 1;
  let hidden = false;
  const deadline = Date.now() + 4000;
  while (Date.now() < deadline) {
    const d = await digest();
    if (d.windowVisible === false) {
      hidden = true;
      break;
    }
    await sleep(250);
  }
  if (!hidden) {
    // Red path: count how many MORE escapes it takes (the bug needed one).
    for (; extraEscapes < 4 && !hidden; ) {
      driver.simulateKey("escape", []);
      extraEscapes++;
      await sleep(600);
      const d = await digest();
      if (d.windowVisible === false) hidden = true;
    }
  }
  receipt.finalState = await digest();
  receipt.windowHidden = hidden;
  receipt.escapesNeededOnEmptyMenu = extraEscapes;
  receipt.classification =
    hidden && extraEscapes === 1 ? "green" : hidden ? "red-extra-escapes" : "red-never-hid";
} catch (error) {
  receipt.error = String(error);
  receipt.classification = "blocked";
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
process.exit(receipt.classification === "green" ? 0 : 1);
