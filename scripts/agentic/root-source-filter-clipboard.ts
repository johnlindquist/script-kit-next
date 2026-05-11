#!/usr/bin/env bun
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "root-source-filter-clipboard");
const query = argValue("--query", `codexclip${Date.now()}`);
const outputDir = join(repoRoot, ".test-output", "root-source-filter-clipboard");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const dbDir = join(kitDir, "db");
const dbPath = join(dbDir, "clipboard-history.sqlite");
const sessionRoot = join(outputDir, "sessions");
const timeoutMs = Number(argValue("--timeout", "10000"));
const pollMs = Number(argValue("--poll", "50"));

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";

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
      requestId: `clipboard-source-filter-wait-${Date.now()}`,
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
      requestId: `clipboard-source-filter-state-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
}

function seedClipboardDb() {
  mkdirSync(dbDir, { recursive: true });
  mkdirSync(join(kitDir, "plugins", "main", "scripts"), { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    "export default { unifiedSearch: { clipboardHistory: { enabled: false } } };\n",
  );

  const now = Date.now();
  const sql = `
CREATE TABLE IF NOT EXISTS history (
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
DELETE FROM history;
INSERT INTO history (
  id,
  content,
  content_hash,
  content_type,
  timestamp,
  pinned,
  ocr_text,
  text_preview,
  image_width,
  image_height,
  byte_size
) VALUES (
  'clip-source-filter',
  '${query} seeded clipboard text',
  'fixture-hash',
  'text',
  ${now},
  0,
  NULL,
  '${query} seeded clipboard text',
  NULL,
  NULL,
  ${query.length + 22}
);
`;
  run("sqlite3", [dbPath], { input: sql });
}

async function warmClipboardCache() {
  const input = `c: ${query}`;
  send({ type: "setFilter", text: input, requestId: `clipboard-filter-warm-${Date.now()}` });
  waitForInput(input);
  await Bun.sleep(350);
  send({ type: "setFilter", text: "", requestId: `clipboard-filter-reset-${Date.now()}` });
  waitForInput("");
}

function assertClipboardFrame(frame: Json, input: string) {
  const preflight = frame.mainWindowPreflight;
  if (!preflight) {
    throw new Error(`${input}: missing mainWindowPreflight`);
  }
  if (preflight.computedSearchText !== query) {
    throw new Error(
      `${input}: expected computedSearchText ${query}, got ${preflight.computedSearchText}`,
    );
  }
  if (JSON.stringify(preflight.sourceFilters) !== JSON.stringify(["clipboard"])) {
    throw new Error(
      `${input}: expected clipboard source filter, got ${JSON.stringify(preflight.sourceFilters)}`,
    );
  }
  const visible = preflight.visibleResults ?? [];
  const clipboardRows = visible.filter(
    (row: Json) =>
      row.role === "rootPassive" &&
      row.sourceName === "Clipboard History" &&
      row.stableKey === "clipboard-history/clip-source-filter",
  );
  if (clipboardRows.length !== 1) {
    throw new Error(`${input}: expected one seeded clipboard row, got ${JSON.stringify(visible)}`);
  }
  const disallowed = visible.filter(
    (row: Json) => row.role !== "rootPassive" || row.sourceName !== "Clipboard History",
  );
  if (disallowed.length > 0) {
    throw new Error(`${input}: disallowed rows survived ${JSON.stringify(disallowed)}`);
  }
}

async function runCase(input: string): Promise<Json> {
  send({ type: "setFilter", text: input, requestId: `clipboard-filter-set-${Date.now()}` });
  waitForInput(input);

  let lastFrame: Json | null = null;
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const state = getState(input.replace(/[^a-z0-9]+/gi, "-"));
    lastFrame = state.mainWindowPreflight;
    try {
      assertClipboardFrame(state, input);
      return {
        input,
        preflight: state.mainWindowPreflight,
      };
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
  mkdirSync(outputDir, { recursive: true });
  seedClipboardDb();
  runSession(["stop", session]);
  runSession(["start", session]);

  try {
    await warmClipboardCache();
    const cases = [
      await runCase(`c: ${query}`),
      await runCase(`clipboard: ${query}`),
    ];
    const logPath = join(sessionRoot, session, "app.log");
    const responsesPath = join(sessionRoot, session, "responses.ndjson");
    const receipt = {
      schemaVersion: 1,
      status: "pass",
      session,
      query,
      homeDir,
      dbPath,
      cases,
      logExcerpt: readFileSync(logPath, "utf8").split("\n").slice(-40),
      responsesPath,
    };
    writeFileSync(join(outputDir, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
    process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
  } finally {
    runSession(["stop", session]);
  }
}

main().catch((error) => {
  try {
    runSession(["stop", session]);
  } catch {
    // Best-effort cleanup after a failed start.
  }
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
