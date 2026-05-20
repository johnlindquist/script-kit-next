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
const outputDir = join(repoRoot, ".test-output", "root-delete-key-benchmark");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const dbDir = join(kitDir, "db");
const sessionRoot = join(outputDir, "sessions");
const chromeDir = join(homeDir, "Library/Application Support/Google/Chrome/Default");

const session = argValue("--session", "root-delete-key-benchmark");
const samples = Number(argValue("--samples", "8"));
const deleteCount = Number(argValue("--delete-count", "24"));
const cadenceMs = Number(argValue("--cadence", "18"));
const burstSamples = Number(argValue("--burst-samples", "5"));
const stateProbeEvery = Number(argValue("--state-probe-every", "0"));
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "4"));
const query = argValue("--query", `amazon-delete-${Date.now()}`);
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
    sessionId: "delete-key-benchmark-vault",
    sourceKind: "cli",
    safeTitle: `${query} vault session`,
    workspacePath: `/tmp/${query}-workspace`,
    model: "fixture-model",
    modifiedAt: new Date().toISOString(),
    matchedField: "title",
    stableKey: "ai-vault/codex/cli/delete-key-benchmark-vault",
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
  if (!stdout) throw new Error(`session.sh ${args.join(" ")} produced no stdout`);
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
  if (!sessionStatus?.pipe) throw new Error("missing session pipe");
  appendFileSync(sessionStatus.pipe, `${JSON.stringify(command)}\n`);
}

function directRpc(command: Json, expect: string, timeout = timeoutMs): Json {
  command.requestId ??= `root-delete-rpc-${Date.now()}`;
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
      requestId: `root-delete-wait-${Date.now()}`,
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
    { type: "getState", requestId: `root-delete-state-${tag}-${Date.now()}` },
    "stateResult",
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
  mkdirSync(chromeDir, { recursive: true });
  mkdirSync(join(kitDir, "plugins", "main", "scripts"), { recursive: true });

  writeFileSync(
    join(kitDir, "plugins", "main", "scripts", `${query}.ts`),
    `// Name: ${query} script\nconsole.log("fixture");\n`,
  );

  const now = new Date().toISOString();
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
VALUES ('33333333-3333-4333-8333-333333333333', '${query} note', '${query} note content', '${now}', '${now}', NULL, 0, 0);
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
  'clip-delete-key-benchmark', '${query} clipboard text', 'fixture-hash', 'text',
  ${Date.now()}, 0, NULL, '${query} clipboard text', NULL, NULL, ${query.length + 15}
);
`,
  );

  writeFileSync(
    join(kitDir, "dictation-history.jsonl"),
    `${JSON.stringify({
      id: "dictation-delete-key-benchmark",
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
      session_id: "acp-delete-key-benchmark",
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

function setFilter(text: string, tag: string) {
  const parseMs = directSend({
    type: "setFilter",
    text,
    requestId: `root-delete-set-${tag}-${Date.now()}`,
  });
  const echoWaitMs = waitForInput(text);
  return {
    text,
    parseMs: Number(parseMs.toFixed(3)),
    inputEchoMs: Number((parseMs + echoWaitMs).toFixed(3)),
  };
}

function sendBackspace(tag: string) {
  const start = performance.now();
  const response = directRpc(
    {
      type: "simulateGpuiEvent",
      requestId: `root-delete-key-${tag}-${Date.now()}`,
      target: { type: "main" },
      event: { type: "keyDown", key: "backspace", modifiers: [] },
    },
    "simulateGpuiEventResult",
  );
  const dispatchMs = performance.now() - start;
  if (response.success !== true) {
    throw new Error(`backspace dispatch failed: ${JSON.stringify(response)}`);
  }
  return Number(dispatchMs.toFixed(3));
}

function measureDeleteEcho(sampleIndex: number) {
  const initial = `${query}-sample-${sampleIndex}`;
  setFilter(initial, `sample-${sampleIndex}-start`);

  const events = [];
  let current = initial;
  let cadenceOverrunMaxMs = 0;
  const count = Math.min(deleteCount, initial.length);
  for (let index = 0; index < count; index += 1) {
    const expected = current.slice(0, -1);
    const tickStarted = performance.now();
    const dispatchMs = sendBackspace(`${sampleIndex}-${index}`);
    const echoWaitMs = waitForInput(expected);
    const inputEchoMs = performance.now() - tickStarted;
    const shouldProbeState = stateProbeEvery > 0 && index % stateProbeEvery === 0;
    const state = shouldProbeState ? getState(`${sampleIndex}-${index}`) : null;
    const computedMatchesInput = state
      ? state.mainWindowPreflight?.computedSearchText === expected
      : null;
    events.push({
      index,
      expectedLength: expected.length,
      dispatchMs,
      echoWaitMs: Number(echoWaitMs.toFixed(3)),
      inputEchoMs: Number(inputEchoMs.toFixed(3)),
      computedMatchesInput,
      visibleResultCount: state?.mainWindowPreflight?.visibleResults?.length ?? null,
      preflightFingerprint: state ? hash(state.mainWindowPreflight?.visibleResults ?? []) : null,
    });
    current = expected;
    const elapsed = performance.now() - tickStarted;
    cadenceOverrunMaxMs = Math.max(cadenceOverrunMaxMs, elapsed - cadenceMs);
    if (elapsed < cadenceMs) sleepSync(cadenceMs - elapsed);
  }
  return {
    kind: "delete-echo",
    sampleIndex,
    initial,
    final: current,
    cadenceOverrunMaxMs: Number(cadenceOverrunMaxMs.toFixed(3)),
    events,
  };
}

function measureDeleteBurst(sampleIndex: number) {
  const initial = `${query}-burst-${sampleIndex}`;
  setFilter(initial, `burst-${sampleIndex}-start`);
  const count = Math.min(deleteCount, initial.length);
  const expected = initial.slice(0, initial.length - count);
  const dispatches = [];
  const start = performance.now();
  for (let index = 0; index < count; index += 1) {
    dispatches.push(sendBackspace(`burst-${sampleIndex}-${index}`));
  }
  const dispatchTotalMs = performance.now() - start;
  const echoWaitMs = waitForInput(expected);
  const totalMs = performance.now() - start;
  const state = getState(`burst-${sampleIndex}-final`);
  return {
    kind: "delete-burst",
    sampleIndex,
    initial,
    expected,
    dispatchTotalMs: Number(dispatchTotalMs.toFixed(3)),
    echoWaitMs: Number(echoWaitMs.toFixed(3)),
    totalMs: Number(totalMs.toFixed(3)),
    dispatch: stats(dispatches),
    finalInput: state.inputValue,
    computedSearchText: state.mainWindowPreflight?.computedSearchText ?? null,
    visibleResultCount: state.mainWindowPreflight?.visibleResults?.length ?? null,
  };
}

function parsePerfLogs(logPath: string) {
  const log = readFileSync(logPath, "utf8");
  const applyDurations = [...log.matchAll(/APPLY_FILTER_DONE in ([0-9.]+)ms/g)].map((match) =>
    Number(match[1]),
  );
  const handlerDurations = [...log.matchAll(/handle_filter_input_change took ([0-9.]+)ms/g)].map(
    (match) => Number(match[1]),
  );
  const groupDurations = [...log.matchAll(/GROUP_DONE in ([0-9.]+)ms/g)].map((match) =>
    Number(match[1]),
  );
  return {
    applyFilterDone: stats(applyDurations),
    handlerSlow: stats(handlerDurations),
    groupDone: stats(groupDurations),
    handlerSlowCount: handlerDurations.length,
  };
}

async function main() {
  seedFixtures();
  runSession(["stop", session]);
  runSession(["start", session]);
  sessionStatus = runSession(["status", session]);

  setFilter(query, "warm");

  const echoReceipts = [];
  for (let sampleIndex = 0; sampleIndex < samples; sampleIndex += 1) {
    echoReceipts.push(measureDeleteEcho(sampleIndex));
  }
  const burstReceipts = [];
  for (let sampleIndex = 0; sampleIndex < burstSamples; sampleIndex += 1) {
    burstReceipts.push(measureDeleteBurst(sampleIndex));
  }

  const echoEvents = echoReceipts.flatMap((receipt) => receipt.events);
  const computedMismatchCount = echoEvents.filter(
    (event) => event.computedMatchesInput === false,
  ).length;
  const burstFinalMismatchCount = burstReceipts.filter(
    (receipt) =>
      receipt.finalInput !== receipt.expected || receipt.computedSearchText !== receipt.expected,
  ).length;
  const summary = {
    deleteEcho: {
      inputEcho: stats(echoEvents.map((event) => event.inputEchoMs)),
      dispatch: stats(echoEvents.map((event) => event.dispatchMs)),
      echoWait: stats(echoEvents.map((event) => event.echoWaitMs)),
      cadenceMs,
      cadenceOverrunMaxMs: Number(Math.max(
        0,
        ...echoReceipts.map((receipt) => receipt.cadenceOverrunMaxMs),
      ).toFixed(3)),
      computedMismatchCount,
    },
    deleteBurst: {
      total: stats(burstReceipts.map((receipt) => receipt.totalMs)),
      dispatchTotal: stats(burstReceipts.map((receipt) => receipt.dispatchTotalMs)),
      echoWait: stats(burstReceipts.map((receipt) => receipt.echoWaitMs)),
      finalMismatchCount: burstFinalMismatchCount,
    },
    perfLogs: parsePerfLogs(String(sessionStatus.log)),
  };

  const failures = [];
  if (summary.deleteEcho.inputEcho.p50Ms > 25) failures.push("delete inputEcho p50 > 25ms");
  if (summary.deleteEcho.inputEcho.p95Ms > 75) failures.push("delete inputEcho p95 > 75ms");
  if (summary.deleteEcho.inputEcho.maxMs > 200) failures.push("delete inputEcho max > 200ms");
  if (summary.deleteEcho.cadenceOverrunMaxMs > 75) {
    failures.push("delete cadence overrun max > 75ms");
  }
  if (summary.deleteEcho.computedMismatchCount !== 0) {
    failures.push("delete computedSearchText mismatch");
  }
  if (summary.deleteBurst.total.p95Ms > 500) failures.push("delete burst total p95 > 500ms");
  if (summary.deleteBurst.finalMismatchCount !== 0) failures.push("delete burst final mismatch");
  if (summary.perfLogs.handlerSlowCount > 0) failures.push("handler slow logs present");

  const receipt = {
    schemaVersion: 1,
    status: failures.length === 0 ? "pass" : "fail",
    query,
    samples,
    deleteCount,
      burstSamples,
      stateProbeEvery,
      thresholds: {
      deleteInputEchoP50Ms: 25,
      deleteInputEchoP95Ms: 75,
      deleteInputEchoMaxMs: 200,
      cadenceOverrunMaxMs: 75,
      deleteBurstTotalP95Ms: 500,
      handlerSlowCount: 0,
      enforced: enforce,
      failures,
    },
    session: {
      name: session,
      logPath: sessionStatus.log,
      responsesPath: sessionStatus.responses,
    },
    summary,
    echoReceipts,
    burstReceipts,
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
