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
const outputDir = join(repoRoot, ".test-output", "root-all-passive-defaults-benchmark");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const dbDir = join(kitDir, "db");
const sessionRoot = join(outputDir, "sessions");
const recentDir = join(outputDir, "recent");
const chromeDir = join(homeDir, "Library/Application Support/Google/Chrome/Default");

const session = argValue("--session", "root-all-passive-defaults-benchmark");
const samples = Number(argValue("--samples", "15"));
const typingSamples = Number(argValue("--typing-samples", "10"));
const typingCadenceMs = Number(argValue("--typing-cadence", "20"));
const staleDelayMs = Number(argValue("--stale-delay", "650"));
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "5"));
const query = argValue("--query", `defaultpassive${Date.now()}`);
const staleFileQuery = `${query}-stale`;
const finalFileQuery = `${query}-final`;
const enforce = !process.argv.includes("--no-enforce");

let sessionStatus: Json | null = null;

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
  passthroughUnmatched: false,
  fixtures: [
    {
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
    },
    {
      query: staleFileQuery,
      delayMs: staleDelayMs,
      results: [
        {
          path: `/tmp/${staleFileQuery}-SHOULD-NOT-PUBLISH.txt`,
          name: `${staleFileQuery}-SHOULD-NOT-PUBLISH.txt`,
          fileType: "document",
          size: 42,
          modified: Date.now(),
        },
      ],
    },
    {
      query: finalFileQuery,
      delayMs: 0,
      results: [
        {
          path: `/tmp/${finalFileQuery}-file-result.txt`,
          name: `${finalFileQuery}-file-result.txt`,
          fileType: "document",
          size: 42,
          modified: Date.now(),
        },
      ],
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
    url: `https://example.invalid/${query}/tab`,
  },
]);
process.env.SCRIPT_KIT_AI_VAULT_TEST_PROVIDER = JSON.stringify([
  {
    provider: "codex",
    providerDisplayName: "Codex",
    sessionId: "defaults-benchmark-vault",
    sourceKind: "cli",
    safeTitle: `${query} vault session`,
    workspacePath: `/tmp/${query}-workspace`,
    model: "fixture-model",
    modifiedAt: new Date().toISOString(),
    matchedField: "title",
    stableKey: "ai-vault/codex/cli/defaults-benchmark-vault",
    score: 100,
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

function fileSize(path: string): number {
  try {
    return statSync(path).size;
  } catch {
    return 0;
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
  if (!sessionStatus?.pipe) {
    throw new Error("missing session pipe");
  }
  appendFileSync(sessionStatus.pipe, `${JSON.stringify(command)}\n`);
}

function directRpc(command: Json, expect: string, timeout = timeoutMs): Json {
  command.requestId ??= `root-all-passive-rpc-${Date.now()}`;
  const responses = String(sessionStatus?.responses ?? "");
  const responseOffset = fileSize(responses);
  const protocolResponses = String(sessionStatus?.protocolResponses ?? "");
  const protocolOffset = fileSize(protocolResponses);
  const logPath = String(sessionStatus?.log ?? "");
  const logOffset = fileSize(logPath);
  directWrite(command);
  const envelope = waitUntil(timeout, () => {
    for (const tail of [
      readFrom(responses, responseOffset),
      readFrom(protocolResponses, protocolOffset),
    ]) {
      for (const line of tail.split("\n")) {
        if (!line.trim()) continue;
        try {
          const parsed = JSON.parse(line);
          if (parsed.requestId === command.requestId) return parsed;
        } catch {
          continue;
        }
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
      } catch {
        continue;
      }
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
      requestId: `root-all-passive-wait-${Date.now()}`,
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
  return performance.now() - start;
}

function getState(tag: string): Json {
  return directRpc(
    { type: "getState", requestId: `root-all-passive-state-${tag}-${Date.now()}` },
    "stateResult",
  );
}

function getElements(tag: string): Json {
  return directRpc(
    { type: "getElements", requestId: `root-all-passive-elements-${tag}-${Date.now()}` },
    "elementsResult",
  );
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
  mkdirSync(recentDir, { recursive: true });
  mkdirSync(chromeDir, { recursive: true });
  mkdirSync(join(kitDir, "plugins", "main", "scripts"), { recursive: true });

  writeFileSync(
    join(kitDir, "plugins", "main", "scripts", `${query}.ts`),
    `// Name: ${query} script\nconsole.log("fixture");\n`,
  );

  const now = new Date().toISOString();
  const recentFilePath = join(recentDir, `${query}-recent-file.txt`);
  writeFileSync(recentFilePath, `${query} recent file body\n`);
  writeFileSync(
    join(kitDir, "frecency.json"),
    `${JSON.stringify({
      entries: {
        [`file/${recentFilePath}`]: {
          count: 3,
          last_used: Math.floor(Date.now() / 1000),
        },
      },
    })}\n`,
  );

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
VALUES ('22222222-2222-4222-8222-222222222222', '${query} note', '${query} note content', '${now}', '${now}', NULL, 0, 0);
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
INSERT INTO history VALUES (
  'clip-defaults-benchmark', '${query} clipboard text', 'fixture-hash', 'text',
  ${Date.now()}, 0, NULL, '${query} clipboard text', NULL, NULL, ${query.length + 15}
);
`,
  );

  writeFileSync(
    join(kitDir, "dictation-history.jsonl"),
    `${JSON.stringify({
      id: "dictation-defaults-benchmark",
      timestamp: now,
      transcript: `${query} dictation transcript`,
      preview: `${query} dictation transcript`,
      target: "Main Filter",
      audio_duration_ms: 1200,
    })}\n`,
  );

  writeFileSync(
    join(kitDir, "agent_chat-history.jsonl"),
    `${JSON.stringify({
      timestamp: now,
      first_message: `${query} conversation prompt`,
      message_count: 2,
      session_id: "agent_chat-defaults-benchmark",
      title: `${query} conversation prompt`,
      preview: `${query} conversation reply`,
      search_text: `${query} conversation prompt ${query} conversation reply`,
    })}\n`,
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
INSERT INTO urls VALUES (1, 'https://example.invalid/${query}/history', '${query} browser history', 7, 2, ${chromeTime});
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

function prefixes(input: string): string[] {
  const values: string[] = [];
  for (let index = 1; index <= input.length; index += 1) {
    values.push(input.slice(0, index));
  }
  return values;
}

const cases = [
  {
    kind: "global",
    input: () => query,
    expectedSourceName: "Browser Tabs",
    frameKey: "browserTabs",
  },
  {
    kind: "files",
    input: () => `f: ${query}`,
    expectedSourceName: "Files",
    frameKey: null,
  },
  {
    kind: "notes",
    input: () => `n: ${query}`,
    expectedSourceName: "Notes",
    frameKey: "notes",
  },
  {
    kind: "clipboard",
    input: () => `c: ${query}`,
    expectedSourceName: "Clipboard History",
    frameKey: "clipboardHistory",
  },
  {
    kind: "dictation",
    input: () => `d: ${query}`,
    expectedSourceName: "Dictation History",
    frameKey: "dictationHistory",
  },
  {
    kind: "conversations",
    input: () => `ai: ${query}`,
    expectedSourceName: "Agent Chat Conversations",
    frameKey: "agent_chatHistory",
  },
  {
    kind: "vault",
    input: () => `v: ${query}`,
    expectedSourceName: "AI Vault",
    frameKey: "aiVault",
  },
  {
    kind: "tabs",
    input: () => `t: ${query}`,
    expectedSourceName: "Browser Tabs",
    frameKey: "browserTabs",
  },
  {
    kind: "history",
    input: () => `h: ${query}`,
    expectedSourceName: "Browser History",
    frameKey: "browserHistory",
  },
];

function summarizePreflight(preflight: Json | null) {
  const rows = Array.isArray(preflight?.visibleResults) ? preflight.visibleResults : [];
  const passiveRows = rows.filter((row: Json) => row.role === "rootPassive");
  const sourceNames = [...new Set(rows.map((row: Json) => row.sourceName).filter(Boolean))];
  return {
    computedSearchText: preflight?.computedSearchText ?? null,
    sourceFilters: preflight?.sourceFilters ?? [],
    visibleResultCount: rows.length,
    passiveResultCount: passiveRows.length,
    sourceNames,
    selectedResultKey: preflight?.selectedResultKey ?? null,
    fingerprint: hash(rows.map((row: Json) => ({
      stableKey: row.stableKey ?? null,
      role: row.role ?? null,
      sourceName: row.sourceName ?? null,
      rank: row.visibleRank ?? null,
    }))),
    rootPassiveFrame: preflight?.rootPassiveFrame ?? null,
  };
}

function sourceSettled(preflight: Json | null, expectedSourceName: string, frameKey: string | null) {
  const rows = Array.isArray(preflight?.visibleResults) ? preflight.visibleResults : [];
  const matchingRows = rows.filter((row: Json) => row.sourceName === expectedSourceName);
  const frame = frameKey ? preflight?.rootPassiveFrame?.[frameKey] : null;
  const frameSettled = !frame || frame.refreshing === false;
  return {
    ok: matchingRows.length > 0 && frameSettled,
    matchingRows: matchingRows.length,
    frame: frame ?? null,
  };
}

function visibleRows(state: Json | null): Json[] {
  return Array.isArray(state?.mainWindowPreflight?.visibleResults)
    ? state.mainWindowPreflight.visibleResults
    : [];
}

function sourceRowsFor(state: Json | null, sourceName: string): Json[] {
  return visibleRows(state).filter((row: Json) => row.sourceName === sourceName);
}

function rowContains(row: Json, needle: string): boolean {
  return JSON.stringify({
    stableKey: row.stableKey ?? null,
    sourceName: row.sourceName ?? null,
    name: row.name ?? null,
    label: row.label ?? null,
    description: row.description ?? null,
    typeLabel: row.typeLabel ?? null,
  }).includes(needle);
}

function staleRowsFor(state: Json | null, staleNeedles: string[]): Json[] {
  return visibleRows(state).filter((row: Json) =>
    staleNeedles.some((needle) => rowContains(row, needle)),
  );
}

function waitForSourceResult(
  input: string,
  expectedComputedSearchText: string,
  kind: string,
  sampleIndex: number,
  expectedSourceName: string,
  frameKey: string | null,
) {
  const started = performance.now();
  let polls = 0;
  let lastState: Json | null = null;
  while (performance.now() - started < timeoutMs) {
    polls += 1;
    const state = getState(`${kind}-${sampleIndex}-settle-${polls}`);
    lastState = state;
    const preflight = state.mainWindowPreflight ?? null;
    const settled = sourceSettled(preflight, expectedSourceName, frameKey);
    if (preflight?.computedSearchText === expectedComputedSearchText && settled.ok) {
      return {
        state,
        polls,
        settled,
        elapsedMs: performance.now() - started,
        timedOut: false,
      };
    }
    if (
      preflight?.computedSearchText
      && preflight.computedSearchText !== expectedComputedSearchText
    ) {
      throw new Error(
        `${kind}: expected computedSearchText ${expectedComputedSearchText}, got ${preflight.computedSearchText} for input ${input}`,
      );
    }
    sleepSync(pollMs);
  }
  return {
    state: lastState,
    polls,
    settled: sourceSettled(lastState?.mainWindowPreflight ?? null, expectedSourceName, frameKey),
    elapsedMs: performance.now() - started,
    timedOut: true,
  };
}

function setFilterAndMeasureEcho(text: string, tag: string) {
  const parseMs = directSend({
    type: "setFilter",
    text,
    requestId: `root-input-echo-${tag}-${Date.now()}`,
  });
  const echoWaitMs = waitForInput(text);
  const inputEchoMs = parseMs + echoWaitMs;
  return {
    text,
    parseMs: Number(parseMs.toFixed(3)),
    inputEchoMs: Number(inputEchoMs.toFixed(3)),
  };
}

async function measure(testCase: (typeof cases)[number], sampleIndex: number) {
  const input = testCase.input();
  const kind = testCase.kind;
  directSend({ type: "setFilter", text: "", requestId: `root-all-passive-reset-${Date.now()}` });
  waitForInput("");

  const started = performance.now();
  const echo = setFilterAndMeasureEcho(input, `${kind}-${sampleIndex}`);
  const settled = waitForSourceResult(
    input,
    query,
    kind,
    sampleIndex,
    testCase.expectedSourceName,
    testCase.frameKey,
  );
  const state = settled.state ?? getState(`${kind}-${sampleIndex}`);
  const stateMs = performance.now() - started;
  const elements = getElements(`${kind}-${sampleIndex}`);
  const totalMs = performance.now() - started;
  const nodes = Array.isArray(elements.elements) ? elements.elements : [];
  return {
    kind,
    input,
    sampleIndex,
    timings: {
      parseMs: echo.parseMs,
      inputObservedMs: echo.inputEchoMs,
      searchSettledMs: Number((echo.inputEchoMs + settled.elapsedMs).toFixed(3)),
      stateMs: Number(stateMs.toFixed(3)),
      stateAndElementsMs: Number(totalMs.toFixed(3)),
    },
    searchSettled: {
      timedOut: settled.timedOut,
      polls: settled.polls,
      expectedSourceName: testCase.expectedSourceName,
      matchingRows: settled.settled.matchingRows,
      frame: settled.settled.frame,
    },
    preflight: summarizePreflight(state.mainWindowPreflight ?? null),
    elements: {
      count: nodes.length,
      selectedId: elements.selectedId ?? null,
      focusedId: elements.focusedId ?? null,
    },
  };
}

function measureRapidTypingCase(sampleIndex: number) {
  const target = query;
  const targetPrefixes = prefixes(target);
  const sequence = [
    ...targetPrefixes,
    ...targetPrefixes.slice(0, Math.max(1, targetPrefixes.length - 4)).reverse(),
    ...targetPrefixes,
  ];
  const echoes = [];
  let cadenceOverrunMaxMs = 0;

  directSend({ type: "setFilter", text: "", requestId: `typing-reset-${Date.now()}` });
  waitForInput("");

  for (const [index, text] of sequence.entries()) {
    const tickStarted = performance.now();
    echoes.push(setFilterAndMeasureEcho(text, `rapid-${sampleIndex}-${index}`));
    const elapsed = performance.now() - tickStarted;
    cadenceOverrunMaxMs = Math.max(cadenceOverrunMaxMs, elapsed - typingCadenceMs);
    if (elapsed < typingCadenceMs) sleepSync(typingCadenceMs - elapsed);
  }

  const finalSettle = waitForSourceResult(
    target,
    target,
    "global-rapid",
    sampleIndex,
    "Browser Tabs",
    "browserTabs",
  );

  return {
    kind: "global-rapid-typing",
    sampleIndex,
    cadenceMs: typingCadenceMs,
    cadenceOverrunMaxMs: Number(cadenceOverrunMaxMs.toFixed(3)),
    inputEcho: stats(echoes.map((entry) => entry.inputEchoMs)),
    parse: stats(echoes.map((entry) => entry.parseMs)),
    finalSourceSettledMs: Number(finalSettle.elapsedMs.toFixed(3)),
    finalPolls: finalSettle.polls,
    finalTimedOut: finalSettle.timedOut,
    echoes,
  };
}

function measureInFlightCancellation(sampleIndex: number) {
  const staleInput = `f: ${staleFileQuery}`;
  const finalInput = `f: ${finalFileQuery}`;

  directSend({ type: "setFilter", text: "", requestId: `cancel-reset-${Date.now()}` });
  waitForInput("");

  const staleEcho = setFilterAndMeasureEcho(staleInput, `cancel-stale-${sampleIndex}`);
  sleepSync(160);
  const finalEcho = setFilterAndMeasureEcho(finalInput, `cancel-final-${sampleIndex}`);
  const finalSettle = waitForSourceResult(
    finalInput,
    finalFileQuery,
    "files-cancel",
    sampleIndex,
    "Files",
    null,
  );

  sleepSync(staleDelayMs + 120);
  const postState = getState(`cancel-post-stale-${sampleIndex}`);
  const staleRows = staleRowsFor(postState, [staleFileQuery]);
  const finalRows = sourceRowsFor(postState, "Files").filter((row) =>
    rowContains(row, finalFileQuery),
  );

  return {
    kind: "files-inflight-cancellation",
    sampleIndex,
    staleEcho,
    finalEcho,
    finalSourceSettledMs: Number((finalEcho.inputEchoMs + finalSettle.elapsedMs).toFixed(3)),
    finalSettlePolls: finalSettle.polls,
    finalSettleTimedOut: finalSettle.timedOut,
    staleResultCount: staleRows.length,
    finalResultCount: finalRows.length,
    finalInputStillEchoed: postState.inputValue === finalInput,
    finalComputedSearchText: postState.mainWindowPreflight?.computedSearchText ?? null,
  };
}

async function main() {
  seedFixtures();
  runSession(["stop", session]);
  runSession(["start", session]);
  sessionStatus = runSession(["status", session]);

  directSend({ type: "setFilter", text: query, requestId: `root-all-passive-warm-${Date.now()}` });
  waitForInput(query);

  const receipts = [];
  for (let sampleIndex = 0; sampleIndex < samples; sampleIndex += 1) {
    for (const testCase of cases) {
      receipts.push(await measure(testCase, sampleIndex));
    }
  }
  const typingReceipts = [];
  const cancellationReceipts = [];
  for (let sampleIndex = 0; sampleIndex < typingSamples; sampleIndex += 1) {
    typingReceipts.push(measureRapidTypingCase(sampleIndex));
    cancellationReceipts.push(measureInFlightCancellation(sampleIndex));
  }

  const byKind: Json = {};
  for (const kind of cases.map((testCase) => testCase.kind)) {
    const matching = receipts.filter((receipt) => receipt.kind === kind);
    byKind[kind] = {
      searchSettled: stats(matching.map((receipt) => receipt.timings.searchSettledMs)),
      state: stats(matching.map((receipt) => receipt.timings.stateMs)),
      stateAndElements: stats(matching.map((receipt) => receipt.timings.stateAndElementsMs)),
      firstPreflight: matching[0]?.preflight ?? null,
      firstSearchSettled: matching[0]?.searchSettled ?? null,
    };
  }
  const fileSourceStats = byKind.files?.searchSettled ?? stats([]);
  const passiveKinds = cases
    .map((testCase) => testCase.kind)
    .filter((kind) => kind !== "files" && kind !== "global");
  const passiveSourceStats = stats(receipts
    .filter((receipt) => passiveKinds.includes(receipt.kind))
    .map((receipt) => receipt.timings.searchSettledMs));
  const typingEchoValues = typingReceipts.flatMap((receipt) =>
    receipt.echoes.map((echo: Json) => echo.inputEchoMs),
  );
  const typingParseValues = typingReceipts.flatMap((receipt) =>
    receipt.echoes.map((echo: Json) => echo.parseMs),
  );
  const cancellationFinalEchoValues = cancellationReceipts.map((receipt) =>
    receipt.finalEcho.inputEchoMs,
  );
  const cancellationStaleResultCount = cancellationReceipts.reduce(
    (total, receipt) => total + receipt.staleResultCount,
    0,
  );
  const cancellationFinalResultMin = Math.min(
    ...cancellationReceipts.map((receipt) => receipt.finalResultCount),
  );
  const finalInputRollbackCount = cancellationReceipts.filter(
    (receipt) => !receipt.finalInputStillEchoed,
  ).length;
  const computedSearchTextMismatchCount = cancellationReceipts.filter(
    (receipt) => receipt.finalComputedSearchText !== finalFileQuery,
  ).length;
  const timedOutCount = [
    ...receipts.filter((receipt) => receipt.searchSettled.timedOut),
    ...typingReceipts.filter((receipt) => receipt.finalTimedOut),
    ...cancellationReceipts.filter((receipt) => receipt.finalSettleTimedOut),
  ].length;
  const summary = {
    inputEcho: {
      typing: stats(typingEchoValues),
      parse: stats(typingParseValues),
      cancellationFinalEcho: stats(cancellationFinalEchoValues),
      cadenceMs: typingCadenceMs,
      cadenceOverrunMaxMs: Number(Math.max(
        0,
        ...typingReceipts.map((receipt) => receipt.cadenceOverrunMaxMs),
      ).toFixed(3)),
    },
    cancellation: {
      staleResultCount: cancellationStaleResultCount,
      finalResultMin: cancellationFinalResultMin,
      finalInputRollbackCount,
      computedSearchTextMismatchCount,
      staleDelayMs,
    },
    searchSettled: stats(receipts.map((receipt) => receipt.timings.searchSettledMs)),
    sourceSettledByKind: {
      files: fileSourceStats,
      passive: passiveSourceStats,
      byKind,
    },
    state: stats(receipts.map((receipt) => receipt.timings.stateMs)),
    stateAndElements: stats(receipts.map((receipt) => receipt.timings.stateAndElementsMs)),
    byKind,
  };
  const failures = [];
  if (timedOutCount > 0) failures.push(`timedOutCount > 0 (${timedOutCount})`);
  if (summary.inputEcho.typing.p50Ms > 25) failures.push("typing inputEcho p50 > 25ms");
  if (summary.inputEcho.typing.p95Ms > 75) failures.push("typing inputEcho p95 > 75ms");
  if (summary.inputEcho.typing.maxMs > 200) failures.push("typing inputEcho max > 200ms");
  if (summary.inputEcho.parse.p95Ms > 50) failures.push("typing parse p95 > 50ms");
  if (summary.inputEcho.cancellationFinalEcho.p95Ms > 100) {
    failures.push("overlappedInputEchoP95Ms > 100ms");
  }
  if (summary.inputEcho.cancellationFinalEcho.maxMs > 200) {
    failures.push("overlappedInputEchoMaxMs > 200ms");
  }
  if (summary.cancellation.staleResultCount !== 0) failures.push("staleResultCount != 0");
  if (summary.cancellation.finalResultMin < 1) failures.push("finalResultCount < 1");
  if (summary.cancellation.finalInputRollbackCount !== 0) {
    failures.push("finalInputStillEchoed regression");
  }
  if (summary.cancellation.computedSearchTextMismatchCount !== 0) {
    failures.push("computedSearchTextMismatchCount != 0");
  }
  if (summary.searchSettled.p95Ms > 250) failures.push("search-settled p95 > 250ms");
  if (fileSourceStats.p95Ms > 275) failures.push("files search-settled p95 > 275ms");
  if (passiveSourceStats.p95Ms > 175) failures.push("passive search-settled p95 > 175ms");
  if (summary.searchSettled.maxMs > 500) failures.push("search-settled max > 500ms");
  if (summary.state.p50Ms > 50) failures.push("state p50 > 50ms");
  if (summary.state.p95Ms > 250) failures.push("state p95 > 250ms");
  if (summary.stateAndElements.p95Ms > 300) failures.push("state+elements p95 > 300ms");

  const receipt = {
    schemaVersion: 1,
    status: failures.length === 0 ? "pass" : "fail",
    query,
    samples,
    typingSamples,
    thresholds: {
      typingInputEchoP50Ms: 25,
      typingInputEchoP95Ms: 75,
      typingInputEchoMaxMs: 200,
      typingParseP95Ms: 50,
      overlappedInputEchoP95Ms: 100,
      overlappedInputEchoMaxMs: 200,
      staleResultCount: 0,
      computedSearchTextMismatchCount: 0,
      searchSettledP95Ms: 250,
      filesSearchSettledP95Ms: 275,
      passiveSearchSettledP95Ms: 175,
      searchSettledMaxMs: 500,
      stateP50Ms: 50,
      stateP95Ms: 250,
      stateAndElementsP95Ms: 300,
      enforced: enforce,
      failures,
    },
    defaultConfigExercised: {
      configTsWritten: false,
      homeDir,
      kitDir,
    },
    session: {
      name: session,
      logPath: sessionStatus.log,
      responsesPath: sessionStatus.responses,
    },
    summary,
    receipts,
    typingReceipts,
    cancellationReceipts,
  };
  const outputPath = join(outputDir, "receipt.json");
  writeFileSync(outputPath, JSON.stringify(receipt, null, 2));
  process.stdout.write(`${JSON.stringify({ outputPath, status: receipt.status, summary }, null, 2)}\n`);
  runSession(["stop", session]);
  if (enforce && failures.length > 0) process.exit(1);
}

main().catch((error) => {
  try {
    runSession(["stop", session]);
  } catch {
    // Best-effort cleanup only.
  }
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
