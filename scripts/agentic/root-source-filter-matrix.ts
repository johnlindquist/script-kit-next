#!/usr/bin/env bun
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "root-source-filter-matrix");
const query = argValue("--query", `codexsource${Date.now()}`);
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "50"));
const outputDir = join(repoRoot, ".test-output", "root-source-filter-matrix");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const dbDir = join(kitDir, "db");
const sessionRoot = join(outputDir, "sessions");

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
  query,
  delayMs: 0,
  results: [
    {
      path: `/tmp/${query}-file-result.txt`,
      name: `${query}-file-result.txt`,
      fileType: "document",
      size: 42,
      modified: Date.now(),
    },
  ],
});
process.env.SCRIPT_KIT_BROWSER_TABS_TEST_PROVIDER = JSON.stringify([
  {
    browser_name: "Google Chrome",
    browser_bundle_id: "com.google.Chrome",
    window_index: 1,
    tab_index: 1,
    title: `${query} browser tab`,
    url: `https://example.com/${query}/tab`,
  },
]);

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function run(command: string, args: string[], options: { input?: string } = {}): string {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
    input: options.input,
  });
  if (result.status !== 0) {
    throw new Error(
      `${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
    );
  }
  return result.stdout;
}

function runSession(args: string[]): Json {
  const stdout = run(sessionScript, args).trim();
  if (!stdout) {
    throw new Error(`session.sh ${args.join(" ")} produced no stdout`);
  }
  const parsed = JSON.parse(stdout);
  if (parsed.status === "error") {
    throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}`);
  }
  return parsed;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  const envelope = runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeout),
  ]);
  return envelope.response;
}

function send(command: Json): Json {
  return runSession([
    "send",
    session,
    JSON.stringify(command),
    "--await-parse",
    "--timeout",
    String(timeoutMs),
  ]);
}

function waitForInput(input: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `source-filter-matrix-wait-${Date.now()}`,
      condition: {
        type: "stateMatch",
        state: {
          promptType: "none",
          inputValue: input,
        },
      },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function getState(tag: string): Json {
  return rpc(
    {
      type: "getState",
      requestId: `source-filter-matrix-state-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
}

function sql(path: string, input: string) {
  run("sqlite3", [path], { input });
}

function seedFixtures() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(dbDir, { recursive: true });
  mkdirSync(join(kitDir, "plugins", "main", "scripts"), { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {
  unifiedSearch: {
    files: { enabled: false, globalSearch: false, recentFiles: false, directoryBrowse: false },
    notes: { enabled: false },
    clipboardHistory: { enabled: false },
    dictationHistory: { enabled: false },
    acpHistory: { enabled: false },
    browserTabs: { enabled: false },
    browserHistory: { enabled: false },
  },
};
`,
  );

  const now = new Date().toISOString();
  const noteId = "11111111-1111-4111-8111-111111111111";
  sql(
    join(dbDir, "notes.sqlite"),
    `
CREATE TABLE notes (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  is_pinned INTEGER NOT NULL DEFAULT 0,
  sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE VIRTUAL TABLE notes_fts USING fts5(title, content, content='notes', content_rowid='rowid');
INSERT INTO notes (id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order)
VALUES ('${noteId}', '${query} note title', '${query} note body', '${now}', '${now}', NULL, 0, 0);
INSERT INTO notes_fts(rowid, title, content)
SELECT rowid, title, content FROM notes WHERE id = '${noteId}';
`,
  );

  sql(
    join(dbDir, "clipboard-history.sqlite"),
    `
CREATE TABLE history (
  id TEXT PRIMARY KEY,
  content TEXT NOT NULL,
  content_hash TEXT,
  content_type TEXT NOT NULL DEFAULT 'text',
  timestamp INTEGER NOT NULL,
  pinned INTEGER DEFAULT 0,
  ocr_text TEXT,
  text_preview TEXT,
  image_width INTEGER,
  image_height INTEGER,
  byte_size INTEGER
);
INSERT INTO history (
  id, content, content_hash, content_type, timestamp, pinned, ocr_text, text_preview, image_width, image_height, byte_size
) VALUES (
  'clip-source-filter', '${query} clipboard text', 'fixture-hash', 'text', ${Date.now()}, 0, NULL, '${query} clipboard text', NULL, NULL, ${query.length + 15}
);
`,
  );

  writeFileSync(
    join(kitDir, "dictation-history.jsonl"),
    `${JSON.stringify({
      id: "dictation-source-filter",
      timestamp: now,
      transcript: `${query} dictation transcript`,
      preview: `${query} dictation transcript`,
      target: "Main Filter",
      audio_duration_ms: 1200,
    })}\n`,
  );

  writeFileSync(
    join(kitDir, "acp-history.jsonl"),
    `${JSON.stringify({
      timestamp: now,
      first_message: `${query} conversation prompt`,
      message_count: 2,
      session_id: "acp-source-filter",
      title: `${query} conversation prompt`,
      preview: `${query} conversation reply`,
      search_text: `${query} conversation prompt ${query} conversation reply`,
    })}\n`,
  );

  const historyDir = join(homeDir, "Library/Application Support/Google/Chrome/Default");
  mkdirSync(historyDir, { recursive: true });
  const chromeTime = (Math.floor(Date.now() / 1000) + 11644473600) * 1000000;
  sql(
    join(historyDir, "History"),
    `
CREATE TABLE urls (
  id INTEGER PRIMARY KEY,
  url TEXT NOT NULL,
  title TEXT,
  visit_count INTEGER NOT NULL DEFAULT 0,
  typed_count INTEGER NOT NULL DEFAULT 0,
  last_visit_time INTEGER NOT NULL DEFAULT 0
);
INSERT INTO urls (id, url, title, visit_count, typed_count, last_visit_time)
VALUES (1, 'https://example.com/${query}/history', '${query} browser history', 7, 2, ${chromeTime});
`,
  );
}

const cases = [
  {
    heads: ["f:", "files:"],
    expectedFilters: ["files"],
    role: "rootFile",
    sourceName: "Files",
    stableKeyIncludes: `${query}-file-result.txt`,
  },
  {
    heads: ["n:", "notes:"],
    expectedFilters: ["notes"],
    role: "rootPassive",
    sourceName: "Notes",
    stableKey: "note/11111111-1111-4111-8111-111111111111",
  },
  {
    heads: ["c:", "clipboard:"],
    expectedFilters: ["clipboard"],
    role: "rootPassive",
    sourceName: "Clipboard History",
    stableKey: "clipboard-history/clip-source-filter",
  },
  {
    heads: ["d:", "dictation:"],
    expectedFilters: ["dictation"],
    role: "rootPassive",
    sourceName: "Dictation History",
    stableKey: "dictation-history/dictation-source-filter",
  },
  {
    heads: ["ai:", "conversations:"],
    expectedFilters: ["conversations"],
    role: "rootPassive",
    sourceName: "AI Conversations",
    stableKey: "acp-history/acp-source-filter",
  },
  {
    heads: ["h:", "history:"],
    expectedFilters: ["history"],
    role: "rootPassive",
    sourceName: "Browser History",
    stableKeyIncludes: "browser-history/",
  },
  {
    heads: ["t:", "tabs:"],
    expectedFilters: ["tabs"],
    role: "rootPassive",
    sourceName: "Browser Tabs",
    stableKeyIncludes: `browser-tab/com.google.Chrome/1/1/https://example.com/${query}/tab`,
  },
];

function assertFrame(state: Json, input: string, spec: Json) {
  const preflight = state.mainWindowPreflight;
  if (!preflight) {
    throw new Error(`${input}: missing mainWindowPreflight in ${JSON.stringify(state)}`);
  }
  if (preflight.computedSearchText !== query) {
    throw new Error(`${input}: expected computedSearchText ${query}, got ${preflight.computedSearchText}`);
  }
  if (JSON.stringify(preflight.sourceFilters) !== JSON.stringify(spec.expectedFilters)) {
    throw new Error(`${input}: expected filters ${JSON.stringify(spec.expectedFilters)}, got ${JSON.stringify(preflight.sourceFilters)}`);
  }
  const visible = preflight.visibleResults ?? [];
  const matches = visible.filter((row: Json) => {
    if (row.role !== spec.role || row.sourceName !== spec.sourceName) {
      return false;
    }
    if (spec.stableKey) {
      return row.stableKey === spec.stableKey;
    }
    return String(row.stableKey ?? "").includes(spec.stableKeyIncludes);
  });
  if (matches.length !== 1) {
    throw new Error(`${input}: expected one ${spec.sourceName} row, got ${JSON.stringify(visible)}`);
  }
  const disallowed = visible.filter((row: Json) => row.role !== spec.role || row.sourceName !== spec.sourceName);
  if (disallowed.length > 0) {
    throw new Error(`${input}: disallowed rows survived ${JSON.stringify(disallowed)}`);
  }
}

async function runCase(head: string, spec: Json): Promise<Json> {
  const input = `${head} ${query}`;
  send({ type: "setFilter", text: input, requestId: `source-filter-matrix-set-${Date.now()}` });
  waitForInput(input);

  let lastFrame: Json | null = null;
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const state = getState(head.replace(/[^a-z0-9]+/gi, "-"));
    lastFrame = state.mainWindowPreflight;
    try {
      assertFrame(state, input, spec);
      return { input, preflight: state.mainWindowPreflight };
    } catch (error) {
      await Bun.sleep(pollMs);
      if (Date.now() >= deadline) {
        throw error;
      }
    }
  }
  throw new Error(`${input}: timed out, lastFrame=${JSON.stringify(lastFrame)}`);
}

async function main() {
  seedFixtures();
  runSession(["stop", session]);
  runSession(["start", session]);

  try {
    const results: Json[] = [];
    for (const spec of cases) {
      for (const head of spec.heads) {
        results.push(await runCase(head, spec));
        send({ type: "setFilter", text: "", requestId: `source-filter-matrix-reset-${Date.now()}` });
        waitForInput("");
      }
    }

    const logPath = join(sessionRoot, session, "app.log");
    const responsesPath = join(sessionRoot, session, "responses.ndjson");
    const receipt = {
      schemaVersion: 1,
      status: "pass",
      session,
      query,
      homeDir,
      cases: results,
      logExcerpt: readFileSync(logPath, "utf8").split("\n").slice(-80),
      responsesPath,
    };
    writeFileSync(join(outputDir, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
    process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
  } finally {
    runSession(["stop", session]);
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
