#!/usr/bin/env bun
import { createHash } from "node:crypto";
import {
  appendFileSync,
  mkdirSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";
import { performance } from "node:perf_hooks";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const outputDir = join(repoRoot, ".test-output", "root-typing-lag-benchmark");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const dbDir = join(kitDir, "db");
const sessionRoot = join(outputDir, "sessions");
const chromeDir = join(homeDir, "Library/Application Support/Google/Chrome/Default");
const agentBinary = join(repoRoot, "target-agent", "pools", "agent-debug", "debug", "script-kit-gpui");

const session = argValue("--session", "root-typing-lag-benchmark");
const samples = Number(argValue("--samples", "6"));
const cadenceMs = Number(argValue("--cadence", "18"));
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "4"));
const stateProbeEvery = Number(argValue("--state-probe-every", "1"));
const enforce = process.argv.includes("--enforce");
const traceEnabled = !process.argv.includes("--no-trace");
const passiveRefreshOverlap = process.argv.includes("--passive-refresh-overlap");
const forceBrowserTabFailure = process.argv.includes("--force-browser-tabs-failure");
const scenarios = argValue("--scenarios", "amz,dictat,this is the f,Hae")
  .split(",")
  .map((scenario) => scenario.trim())
  .filter(Boolean);

let sessionStatus: Json | null = null;

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
if (!process.env.SCRIPT_KIT_GPUI_BINARY && fileExists(agentBinary)) {
  process.env.SCRIPT_KIT_GPUI_BINARY = agentBinary;
}
if (traceEnabled) process.env.SCRIPT_KIT_FILTER_PERF_LOG = "1";
delete process.env.SCRIPT_KIT_PREFLIGHT_DEEP_LOG;
process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
  passthroughUnmatched: false,
  fixtures: scenarios.map((query) => ({
    query,
    delayMs: 0,
    results: [
      {
        path: `/tmp/root-typing-${slug(query)}.txt`,
        name: `${query} file result.txt`,
        fileType: "document",
        size: 42,
        modified: Date.now(),
      },
    ],
  })),
});
process.env.SCRIPT_KIT_BROWSER_TABS_TEST_PROVIDER = JSON.stringify(
  forceBrowserTabFailure
    ? {
        delayMs: passiveRefreshOverlap ? 350 : 0,
        fail: true,
        error: "root typing benchmark forced browser tabs failure",
        tabs: [],
      }
    : {
        delayMs: passiveRefreshOverlap ? 350 : 0,
        tabs: scenarios.map((query, index) => ({
          browser_name: "Google Chrome",
          browser_bundle_id: "com.google.Chrome",
          window_index: 1,
          tab_index: index + 1,
          title: `${query} benchmark browser tab`,
          url: `https://example.invalid/${slug(query)}/tab`,
        })),
      },
);
process.env.SCRIPT_KIT_AI_VAULT_TEST_PROVIDER = JSON.stringify(
  scenarios.map((query) => ({
    provider: "codex",
    providerDisplayName: "Codex",
    sessionId: `root-typing-${slug(query)}`,
    sourceKind: "cli",
    safeTitle: `${query} vault session`,
    workspacePath: `/tmp/root-typing-${slug(query)}-workspace`,
    model: "fixture-model",
    modifiedAt: new Date().toISOString(),
    matchedField: "title",
    stableKey: `ai-vault/codex/cli/root-typing-${slug(query)}`,
    score: 100,
  })),
);

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function slug(value: string): string {
  return value.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "empty";
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
  if (!stdout) throw new Error(`session.sh ${args.join(" ")} produced no stdout`);
  const parsed = JSON.parse(stdout);
  if (parsed.status === "error") throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}`);
  return parsed;
}

function fileSize(path: string): number {
  try {
    return statSync(path).size;
  } catch {
    return 0;
  }
}

function fileExists(path: string): boolean {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
}

function readFrom(path: string, offset: number): string {
  try {
    return readFileSync(path).subarray(offset).toString("utf8");
  } catch {
    return "";
  }
}

function waitUntil<T>(timeout: number, fn: () => T | null): T {
  const deadline = performance.now() + timeout;
  const sleeper = new Int32Array(new SharedArrayBuffer(4));
  while (performance.now() < deadline) {
    const value = fn();
    if (value) return value;
    Atomics.wait(sleeper, 0, 0, pollMs);
  }
  throw new Error("timed out waiting for session response");
}

function directWrite(command: Json) {
  if (!sessionStatus?.pipe) throw new Error("missing session pipe");
  appendFileSync(sessionStatus.pipe, `${JSON.stringify(command)}\n`);
}

function directRpc(command: Json, expect: string, timeout = timeoutMs): Json {
  command.requestId ??= `root-typing-rpc-${Date.now()}`;
  const responses = String(sessionStatus?.responses ?? "");
  const responseOffset = fileSize(responses);
  const protocolResponses = String(sessionStatus?.protocolResponses ?? "");
  const protocolOffset = fileSize(protocolResponses);
  const logPath = String(sessionStatus?.log ?? "");
  const logOffset = fileSize(logPath);
  directWrite(command);
  const envelope = waitUntil(timeout, () => {
    for (const tail of [readFrom(responses, responseOffset), readFrom(protocolResponses, protocolOffset)]) {
      for (const line of tail.split("\n")) {
        if (!line.trim()) continue;
        try {
          const parsed = JSON.parse(line);
          if (parsed.requestId === command.requestId) return parsed;
        } catch {}
      }
    }
    const logTail = readFrom(logPath, logOffset);
    for (const line of logTail.split("\n")) {
      const jsonStart = line.indexOf("{");
      if (jsonStart < 0) continue;
      try {
        const parsed = JSON.parse(line.slice(jsonStart));
        if (parsed.requestId === command.requestId && parsed.type === expect) {
          return { status: "ok", responseType: expect, response: parsed };
        }
      } catch {}
    }
    return null;
  });
  if (envelope.status !== "ok" || envelope.responseType !== expect) {
    throw new Error(`unexpected direct rpc envelope: ${JSON.stringify(envelope)}`);
  }
  return envelope.response;
}

function directSend(command: Json): number {
  const logPath = String(sessionStatus?.log ?? "");
  const offset = fileSize(logPath);
  const start = performance.now();
  directWrite(command);
  waitUntil(timeoutMs, () => {
    const tail = readFrom(logPath, offset);
    if (tail.includes("event_type=stdin_command_parsed")) return true;
    if (tail.includes("event_type=stdin_parse_failed")) {
      throw new Error(`stdin parse failed: ${tail.slice(-500)}`);
    }
    return null;
  });
  return performance.now() - start;
}

function waitForInput(input: string): number {
  const start = performance.now();
  directRpc(
    {
      type: "waitFor",
      requestId: `root-typing-wait-${Date.now()}`,
      condition: { type: "stateMatch", state: { promptType: "none", inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
  return performance.now() - start;
}

function getState(tag: string): Json {
  return directRpc({ type: "getState", requestId: `root-typing-state-${tag}-${Date.now()}` }, "stateResult");
}

function sleepSync(ms: number) {
  if (ms <= 0) return;
  const sleeper = new Int32Array(new SharedArrayBuffer(4));
  Atomics.wait(sleeper, 0, 0, ms);
}

function sql(path: string, input: string) {
  run("sqlite3", [path], { input });
}

function seedFixtures() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(dbDir, { recursive: true });
  mkdirSync(chromeDir, { recursive: true });
  mkdirSync(join(kitDir, "plugins", "main", "scripts"), { recursive: true });

  const now = new Date().toISOString();
  for (const query of scenarios) {
    writeFileSync(
      join(kitDir, "plugins", "main", "scripts", `${slug(query)}.ts`),
      `// Name: ${query} script\nconsole.log("fixture");\n`,
    );
  }

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
${scenarios
  .map(
    (query, index) =>
      `INSERT INTO notes (id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order) VALUES ('${index}3333333-3333-4333-8333-333333333333', '${query} note', '${query} note content', '${now}', '${now}', NULL, 0, ${index});`,
  )
  .join("\n")}
INSERT INTO notes_fts(rowid, title, content) SELECT rowid, title, content FROM notes;
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
${scenarios
  .map(
    (query, index) =>
      `INSERT INTO history VALUES ('clip-root-typing-${index}', '${query} clipboard text', 'fixture-hash-${index}', 'text', ${Date.now() + index}, 0, NULL, '${query} clipboard text', NULL, NULL, ${query.length + 15});`,
  )
  .join("\n")}
`,
  );

  writeFileSync(
    join(kitDir, "dictation-history.jsonl"),
    scenarios
      .map((query) =>
        JSON.stringify({
          id: `dictation-root-typing-${slug(query)}`,
          timestamp: now,
          transcript: `${query} dictation transcript`,
          preview: `${query} dictation transcript`,
          target: "Main Filter",
          audio_duration_ms: 1200,
        }),
      )
      .join("\n") + "\n",
  );

  writeFileSync(
    join(kitDir, "acp-history.jsonl"),
    scenarios
      .map((query) =>
        JSON.stringify({
          timestamp: now,
          first_message: `${query} conversation prompt`,
          message_count: 2,
          session_id: `acp-root-typing-${slug(query)}`,
          title: `${query} conversation prompt`,
          preview: `${query} conversation reply`,
          search_text: `${query} conversation prompt ${query} conversation reply`,
        }),
      )
      .join("\n") + "\n",
  );

  const chromeTime = (Math.floor(Date.now() / 1000) + 11644473600) * 1000000;
  sql(
    join(chromeDir, "History"),
    `
CREATE TABLE urls (
  id INTEGER PRIMARY KEY,
  url TEXT NOT NULL,
  title TEXT,
  visit_count INTEGER NOT NULL DEFAULT 0,
  typed_count INTEGER NOT NULL DEFAULT 0,
  last_visit_time INTEGER NOT NULL DEFAULT 0
);
${scenarios
  .map(
    (query, index) =>
      `INSERT INTO urls VALUES (${index + 1}, 'https://example.invalid/${slug(query)}/history', '${query} browser history', 7, 2, ${chromeTime + index});`,
  )
  .join("\n")}
`,
  );
}

function percentile(values: number[], p: number): number {
  if (values.length === 0) return 0;
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.ceil((p / 100) * sorted.length) - 1);
  return Number(sorted[index].toFixed(3));
}

function stats(values: number[]) {
  return {
    count: values.length,
    p50Ms: percentile(values, 50),
    p95Ms: percentile(values, 95),
    maxMs: Number(Math.max(0, ...values).toFixed(3)),
  };
}

function hash(value: unknown): string {
  return createHash("sha256").update(JSON.stringify(value)).digest("hex").slice(0, 16);
}

function setFilter(text: string, tag: string) {
  const parseMs = directSend({ type: "setFilter", text, requestId: `root-typing-set-${tag}-${Date.now()}` });
  const echoWaitMs = waitForInput(text);
  return {
    text,
    parseMs: Number(parseMs.toFixed(3)),
    inputEchoMs: Number((parseMs + echoWaitMs).toFixed(3)),
  };
}

function typeScenario(query: string, sampleIndex: number) {
  setFilter("", `${slug(query)}-${sampleIndex}-clear`);
  const events = [];
  let current = "";
  let cadenceOverrunMaxMs = 0;
  for (let index = 0; index < query.length; index += 1) {
    current += query[index];
    const tickStarted = performance.now();
    const event = setFilter(current, `${slug(query)}-${sampleIndex}-${index}`);
    const echoElapsed = performance.now() - tickStarted;
    const state = stateProbeEvery > 0 && index % stateProbeEvery === 0 ? getState(`${slug(query)}-${sampleIndex}-${index}`) : null;
    const elapsed = performance.now() - tickStarted;
    cadenceOverrunMaxMs = Math.max(cadenceOverrunMaxMs, echoElapsed - cadenceMs);
    events.push({
      index,
      expected: current,
      expectedLength: current.length,
      ...event,
      computedMatchesInput: state ? state.mainWindowPreflight?.computedSearchText === current : null,
      visibleResultCount: state?.mainWindowPreflight?.visibleResults?.length ?? null,
      preflightFingerprint: state ? hash(state.mainWindowPreflight?.visibleResults ?? []) : null,
    });
    if (elapsed < cadenceMs) sleepSync(cadenceMs - elapsed);
  }
  return {
    kind: "typing",
    query,
    sampleIndex,
    cadenceOverrunMaxMs: Number(cadenceOverrunMaxMs.toFixed(3)),
    events,
  };
}

function duplicateEmptyInput(sampleIndex: number) {
  const first = setFilter("", `empty-${sampleIndex}-first`);
  const second = setFilter("", `empty-${sampleIndex}-second`);
  const state = getState(`empty-${sampleIndex}`);
  return {
    kind: "duplicate-empty",
    sampleIndex,
    first,
    second,
    inputValue: state.inputValue,
    computedSearchText: state.mainWindowPreflight?.computedSearchText ?? null,
  };
}

function maxLogLineBytes(log: string): number {
  return Math.max(
    0,
    ...log
      .split("\n")
      .filter((line) => {
        if (line.includes('"type":"stateResult"')) return false;
        if (line.includes('"type":"elementsResult"')) return false;
        if (line.includes('"type":"layoutInfoResult"')) return false;
        return true;
      })
      .map((line) => Buffer.byteLength(line)),
  );
}

function parsePerfLogs(logPath: string) {
  const log = readFileSync(logPath, "utf8");
  const numbers = (regex: RegExp) => [...log.matchAll(regex)].map((match) => Number(match[1]));
  const handlerDurations = numbers(/handle_filter_input_change took ([0-9.]+)ms/g);
  const applyDurations = numbers(/APPLY_FILTER_DONE in ([0-9.]+)ms/g);
  const groupDurations = numbers(/GROUP_DONE '?[^'\n]*'? in ([0-9.]+)ms/g);
  const searchDurations = numbers(/SEARCH_TOTAL[^:]*: sort=[0-9.]+ms total=([0-9.]+)ms/g);
  const refreshStarted = (log.match(/root_passive_snapshot_refresh_started/g) ?? []).length;
  const refreshFailed = (log.match(/root_passive_snapshot_refresh_failed/g) ?? []).length;
  const preflightDeepLineCount = (log.match(/visible_row_fingerprint":"(?:[^"]{512,})/g) ?? []).length;
  return {
    applyFilterDone: stats(applyDurations),
    groupDone: stats(groupDurations),
    searchTotal: stats(searchDurations),
    handlerSlow: stats(handlerDurations),
    handlerSlowCount: handlerDurations.length,
    browserTabsRefreshStartCount: refreshStarted,
    browserTabsRefreshFailedCount: refreshFailed,
    preflightDeepLineCount,
    maxLogLineBytes: maxLogLineBytes(log),
  };
}

async function main() {
  seedFixtures();
  runSession(["stop", session]);
  runSession(["start", session]);
  sessionStatus = runSession(["status", session]);

  setFilter(scenarios[0] ?? "warm", "warm");

  const typingReceipts = [];
  for (let sampleIndex = 0; sampleIndex < samples; sampleIndex += 1) {
    for (const query of scenarios) {
      typingReceipts.push(typeScenario(query, sampleIndex));
    }
  }

  const emptyReceipts = [];
  for (let sampleIndex = 0; sampleIndex < samples; sampleIndex += 1) {
    emptyReceipts.push(duplicateEmptyInput(sampleIndex));
  }

  const events = typingReceipts.flatMap((receipt) => receipt.events);
  const computedMismatchCount = events.filter((event) => event.computedMatchesInput === false).length;
  const emptyMismatchCount = emptyReceipts.filter(
    (receipt) => receipt.inputValue !== "" || receipt.computedSearchText !== "",
  ).length;
  const perfLogs = parsePerfLogs(String(sessionStatus.log));
  const summary = {
    typing: {
      inputEcho: stats(events.map((event) => event.inputEchoMs)),
      parse: stats(events.map((event) => event.parseMs)),
      cadenceMs,
      cadenceOverrunMaxMs: Number(Math.max(0, ...typingReceipts.map((receipt) => receipt.cadenceOverrunMaxMs)).toFixed(3)),
      computedMismatchCount,
    },
    duplicateEmpty: {
      inputEcho: stats(emptyReceipts.flatMap((receipt) => [receipt.first.inputEchoMs, receipt.second.inputEchoMs])),
      mismatchCount: emptyMismatchCount,
    },
    perfLogs,
  };

  const failures = [];
  if (summary.typing.inputEcho.p50Ms > 20) failures.push("typing inputEcho p50 > 20ms");
  if (summary.typing.inputEcho.p95Ms > 50) failures.push("typing inputEcho p95 > 50ms");
  if (summary.typing.inputEcho.maxMs > 150) failures.push("typing inputEcho max > 150ms");
  if (summary.typing.cadenceOverrunMaxMs > 75) failures.push("typing cadence overrun max > 75ms");
  if (summary.typing.computedMismatchCount !== 0) failures.push("computedSearchText mismatch");
  if (summary.duplicateEmpty.mismatchCount !== 0) failures.push("duplicate empty final mismatch");
  if (summary.perfLogs.handlerSlowCount !== 0) failures.push("handler slow logs present");
  if (summary.perfLogs.groupDone.p95Ms > 35) failures.push("GROUP_DONE p95 > 35ms");
  if (summary.perfLogs.searchTotal.p95Ms > 15) failures.push("SEARCH_TOTAL p95 > 15ms");
  if (summary.perfLogs.maxLogLineBytes > 2048) failures.push("max log line bytes > 2048");
  if (summary.perfLogs.preflightDeepLineCount !== 0) failures.push("deep preflight lines present");

  const receipt = {
    schemaVersion: 1,
    status: failures.length === 0 ? "pass" : "fail",
    scenarios,
    samples,
    cadenceMs,
    traceEnabled,
    passiveRefreshOverlap,
    forceBrowserTabFailure,
    enforce,
    thresholds: {
      inputEchoP50Ms: 20,
      inputEchoP95Ms: 50,
      inputEchoMaxMs: 150,
      cadenceOverrunMaxMs: 75,
      groupDoneP95Ms: 35,
      searchTotalP95Ms: 15,
      maxLogLineBytes: 2048,
      failures,
    },
    session: {
      name: session,
      logPath: sessionStatus.log,
      responsesPath: sessionStatus.responses,
    },
    summary,
    typingReceipts,
    emptyReceipts,
  };

  mkdirSync(outputDir, { recursive: true });
  writeFileSync(join(outputDir, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
  if (enforce && failures.length > 0) process.exit(1);
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
