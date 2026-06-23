#!/usr/bin/env bun
/**
 * Runtime proof for passive root Brain prefix cleanup.
 *
 * Seeds a sandbox brain.sqlite and drives the launcher through DevTools. The
 * implicit root path should ignore filler-only natural-language prefixes, while
 * meaningful passive queries and explicit `brain:` queries remain searchable.
 */
import { Database } from "bun:sqlite";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const session = argValue("--session", `brain-prefix-search-cleanup-${process.pid}`);
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "50"));
const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/brain-prefix-search-cleanup/script-kit-gpui";
const outputDir = join(repoRoot, ".test-output", "brain-prefix-search-cleanup", session);
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const dbDir = join(kitDir, "db");
const brainDbPath = join(outputDir, "seed", "brain.sqlite");
const receiptPath = join(outputDir, "receipt.json");

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function seedFixtures() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(dbDir, { recursive: true });
  mkdirSync(join(outputDir, "seed"), { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {
  unifiedSearch: {
    files: { enabled: false, globalSearch: false, recentFiles: false, directoryBrowse: false },
    notes: { enabled: false },
    clipboardHistory: { enabled: false },
    dictationHistory: { enabled: false },
    agent_chatHistory: { enabled: false },
    aiVault: { enabled: false },
    browserTabs: { enabled: false },
    browserHistory: { enabled: false },
    brainInbox: { enabled: false },
    brain: { enabled: true, maxResults: 4, minQueryChars: 3 },
  },
};
`,
  );

  const db = new Database(brainDbPath);
  db.exec(`
CREATE TABLE IF NOT EXISTS brain_docs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source TEXT NOT NULL,
    source_id TEXT NOT NULL,
    title TEXT NOT NULL DEFAULT '',
    content TEXT NOT NULL DEFAULT '',
    canonical_path TEXT,
    content_hash TEXT NOT NULL DEFAULT '',
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    UNIQUE(source, source_id)
);
CREATE INDEX IF NOT EXISTS idx_brain_docs_source ON brain_docs(source);
CREATE INDEX IF NOT EXISTS idx_brain_docs_updated ON brain_docs(updated_at DESC);
CREATE VIRTUAL TABLE IF NOT EXISTS brain_docs_fts USING fts5(
    title, content, content='brain_docs', content_rowid='id',
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
`);
  const insert = db.prepare(
    `INSERT INTO brain_docs (source, source_id, title, content, canonical_path, content_hash, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)`,
  );
  const now = Math.floor(Date.now() / 1000);
  insert.run(
    "note",
    "script-crashing",
    "Script crashing diagnosis",
    "The script crashing trace mentions root brain passive search cleanup.",
    "brain/notes/script-crashing.md",
    "hash-script-crashing",
    now,
  );
  insert.run(
    "note",
    "brain-works",
    "Brain works fixture",
    "brain works when the query has real recall intent.",
    "brain/notes/brain-works.md",
    "hash-brain-works",
    now - 1,
  );
  insert.run(
    "note",
    "unicode",
    "🚀 猫 recall fixture",
    "Single emoji and one CJK character should still search Brain.",
    "brain/notes/unicode.md",
    "hash-unicode",
    now - 2,
  );
  insert.run(
    "note",
    "decoy-filler",
    "Why is this on any decoy",
    "This document intentionally contains Why is t, Why is thi, Why is this o, and What is this anyway.",
    "brain/notes/decoy-filler.md",
    "hash-decoy",
    now - 3,
  );
  db.close();
}

function rowsFrom(state: Json): Json[] {
  return Array.isArray(state?.mainWindowPreflight?.visibleResults)
    ? state.mainWindowPreflight.visibleResults
    : [];
}

function brainRows(rows: Json[]): Json[] {
  return rows.filter((row) => {
    return row.role === "rootPassive" && row.sourceName === "From Your Brain";
  });
}

function rowText(row: Json): string {
  return JSON.stringify(row);
}

async function settledState(driver: Driver, input: string): Promise<Json> {
  await driver.setFilterAndWait(input, { timeoutMs });
  let last = await driver.getState({ timeoutMs });
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    await Bun.sleep(pollMs);
    const next = await driver.getState({ timeoutMs });
    const frame = next.mainWindowPreflight ?? {};
    if (
      frame.computedSearchText === input ||
      String(input).startsWith("brain:") ||
      next.inputValue === input
    ) {
      last = next;
      if (!frame.rootPassiveFrame?.brain?.refreshing) {
        return next;
      }
    }
  }
  return last;
}

async function runCase(driver: Driver, spec: Json): Promise<Json> {
  const state = await settledState(driver, spec.input);
  const rows = rowsFrom(state);
  const brain = brainRows(rows);
  const result = {
    input: spec.input,
    expectedBrain: spec.expectedBrain,
    expectedTitleIncludes: spec.expectedTitleIncludes ?? null,
    inputValue: state.inputValue,
    computedSearchText: state.mainWindowPreflight?.computedSearchText ?? null,
    sourceFilters: state.mainWindowPreflight?.sourceFilters ?? null,
    visibleResultCount: state.mainWindowPreflight?.visibleResultCount ?? rows.length,
    brainRows: brain.map((row) => ({
      title: row.title ?? row.label ?? row.name ?? null,
      sourceName: row.sourceName ?? null,
      stableKey: row.stableKey ?? null,
      row,
    })),
    rowSample: rows.slice(0, 8),
  };

  if (spec.expectedBrain === false && brain.length > 0) {
    throw new Error(`${spec.input}: expected no Brain rows, got ${JSON.stringify(result)}`);
  }
  if (spec.expectedBrain === true && brain.length === 0) {
    throw new Error(`${spec.input}: expected Brain rows, got ${JSON.stringify(result)}`);
  }
  if (spec.expectedTitleIncludes) {
    const hit = brain.some((row) => rowText(row).includes(spec.expectedTitleIncludes));
    if (!hit) {
      throw new Error(
        `${spec.input}: expected Brain row containing ${spec.expectedTitleIncludes}, got ${JSON.stringify(result)}`,
      );
    }
  }
  return result;
}

seedFixtures();
const driver = await Driver.launch({
  sessionName: session,
  sandboxHome: true,
  binary,
  env: {
    HOME: homeDir,
    SK_PATH: kitDir,
    SCRIPT_KIT_TEST_BRAIN_DB_PATH: brainDbPath,
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
  },
  readyTimeoutMs: 15000,
});

const receipt: Json = {
  schemaVersion: 1,
  tool: "brain-prefix-search-cleanup",
  binary,
  brainDbPath,
  sessionDir: driver.sessionDir,
  cases: [] as Json[],
};

try {
  await driver.waitForState({ promptType: "none" }, { timeoutMs });
  const fillerPrefixCases = ["What is this anyway?", "Why is this on any"].flatMap((phrase) =>
    Array.from({ length: phrase.length - 2 }, (_, index) => ({
      input: phrase.slice(0, index + 3),
      expectedBrain: false,
    })),
  );
  const cases = [
    { input: "Why is t", expectedBrain: false },
    { input: "Why is thi", expectedBrain: false },
    { input: "Why is this o", expectedBrain: false },
    { input: "Why is this on any", expectedBrain: false },
    { input: "What is this anyway?", expectedBrain: false },
    ...fillerPrefixCases,
    {
      input: "Why is this script crashing",
      expectedBrain: true,
      expectedTitleIncludes: "brain/note/script-crashing",
    },
    { input: "Why is t", expectedBrain: false },
    { input: "brain works", expectedBrain: true, expectedTitleIncludes: "brain/note/brain-works" },
    { input: "🚀", expectedBrain: true, expectedTitleIncludes: "brain/note/unicode" },
    { input: "猫", expectedBrain: true, expectedTitleIncludes: "brain/note/unicode" },
    { input: "brain:", expectedBrain: true },
    { input: "brain: Why is t", expectedBrain: true, expectedTitleIncludes: "brain/note/decoy-filler" },
  ];
  for (const spec of cases) {
    receipt.cases.push(await runCase(driver, spec));
  }
  receipt.classification = "pass";
} catch (error) {
  receipt.classification = "fail";
  receipt.error = error instanceof Error ? error.stack ?? error.message : String(error);
  throw error;
} finally {
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  await driver.close();
  console.log(JSON.stringify({ classification: receipt.classification, receiptPath }, null, 2));
}
