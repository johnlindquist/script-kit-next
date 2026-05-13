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
const outputDir = join(repoRoot, ".test-output", "instant-browser-sources");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const sessionRoot = join(outputDir, "sessions");
const dbDir = join(homeDir, "Library/Application Support/Google/Chrome/Default");

const session = argValue("--session", "instant-browser-sources");
const samples = Number(argValue("--samples", "30"));
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "5"));
const label = argValue("--label", "optimized");
const singleQuery = optionalArgValue("--query");
const enforceThresholds = label !== "baseline" && !process.argv.includes("--no-enforce");
const fixtureToken = "github";
let sessionStatus: Json | null = null;

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
process.env.SCRIPT_KIT_BROWSER_TABS_TEST_PROVIDER = JSON.stringify([
  {
    browser_name: "Google Chrome",
    browser_bundle_id: "com.google.Chrome",
    window_index: 1,
    tab_index: 1,
    title: "Fixture browser tab",
    url: `https://example.invalid/${fixtureToken}/tab`,
  },
]);

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function optionalArgValue(name: string): string | null {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : null;
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
  if (sessionStatus) {
    return directRpc(command, expect, timeout);
  }
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

function send(command: Json): { receipt: Json; elapsedMs: number } {
  if (sessionStatus) {
    return directSend(command);
  }
  const start = performance.now();
  const receipt = runSession([
    "send",
    session,
    JSON.stringify(command),
    "--await-parse",
    "--timeout",
    String(timeoutMs),
  ]);
  return { receipt, elapsedMs: performance.now() - start };
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

function waitUntil<T>(timeout: number, poll: number, fn: () => T | null): T {
  const deadline = performance.now() + timeout;
  const sleeper = new Int32Array(new SharedArrayBuffer(4));
  while (performance.now() < deadline) {
    const value = fn();
    if (value) {
      return value;
    }
    Atomics.wait(sleeper, 0, 0, poll);
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
  if (!command.requestId) {
    command.requestId = `browser-source-latency-rpc-${Date.now()}`;
  }
  const responses = String(sessionStatus?.responses ?? "");
  const responseOffset = fileSize(responses);
  const logPath = String(sessionStatus?.log ?? "");
  const logOffset = fileSize(logPath);
  directWrite(command);
  const envelope = waitUntil(timeout, pollMs, () => {
    const tail = readFrom(responses, responseOffset);
    for (const line of tail.split("\n")) {
      if (!line.trim()) continue;
      let parsed: Json;
      try {
        parsed = JSON.parse(line);
      } catch {
        continue;
      }
      if (parsed.requestId === command.requestId) {
        return parsed;
      }
    }
    const logTail = readFrom(logPath, logOffset);
    for (const line of logTail.split("\n")) {
      const jsonStart = line.indexOf("{");
      if (jsonStart < 0) continue;
      let parsed: Json;
      try {
        parsed = JSON.parse(line.slice(jsonStart));
      } catch {
        continue;
      }
      if (parsed.requestId === command.requestId && parsed.type === expect) {
        return { status: "ok", responseType: expect, response: parsed };
      }
    }
    return null;
  });
  if (envelope.status !== "ok" || envelope.responseType !== expect) {
    throw new Error(`unexpected direct rpc envelope: ${JSON.stringify(envelope)}`);
  }
  return envelope.response;
}

function directSend(command: Json): { receipt: Json; elapsedMs: number } {
  const logPath = String(sessionStatus?.log ?? "");
  const offset = fileSize(logPath);
  const start = performance.now();
  directWrite(command);
  waitUntil(timeoutMs, pollMs, () => {
    const tail = readFrom(logPath, offset);
    if (tail.includes("event_type=stdin_command_parsed")) {
      return true;
    }
    if (tail.includes("event_type=stdin_parse_failed")) {
      throw new Error(`stdin parse failed after direct send: ${tail.slice(-500)}`);
    }
    return null;
  });
  return {
    receipt: {
      status: "ok",
      parseOutcome: "parsed",
      commandType: command.type,
    },
    elapsedMs: performance.now() - start,
  };
}

function waitForInput(input: string): { receipt: Json; elapsedMs: number } {
  const start = performance.now();
  const receipt = rpc(
    {
      type: "waitFor",
      requestId: `browser-source-latency-wait-${Date.now()}`,
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
  return { receipt, elapsedMs: performance.now() - start };
}

function getState(tag: string): Json {
  const state = rpc(
    {
      type: "getState",
      requestId: `browser-source-latency-state-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(state)}`);
  }
  return state;
}

function getElements(tag: string): Json {
  const elements = rpc(
    {
      type: "getElements",
      requestId: `browser-source-latency-elements-${tag}-${Date.now()}`,
    },
    "elementsResult",
  );
  if (elements.type !== "elementsResult") {
    throw new Error(`expected elementsResult, got ${JSON.stringify(elements)}`);
  }
  return elements;
}

function sql(path: string, input: string) {
  run("sqlite3", [path], { input });
}

function seedFixtureHome() {
  rmSync(homeDir, { recursive: true, force: true });
  rmSync(sessionRoot, { recursive: true, force: true });
  mkdirSync(kitDir, { recursive: true });
  mkdirSync(join(kitDir, "plugins", "main", "scripts"), { recursive: true });
  mkdirSync(dbDir, { recursive: true });

  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {
  unifiedSearch: {
    browserTabs: {
      enabled: true,
      minQueryChars: 0,
      maxResults: 6,
      scanLimit: 500,
      searchUrls: true,
      cacheTtlMs: 30000,
      providers: ["chrome"]
    },
    browserHistory: {
      enabled: true,
      minQueryChars: 0,
      maxResults: 6,
      scanLimit: 500,
      searchUrls: true,
      maxAgeDays: 90,
      cacheTtlMs: 30000,
      providers: ["chrome"]
    }
  }
};
`,
  );

  writeFileSync(
    join(kitDir, "plugins", "main", "scripts", "github.ts"),
    `// Name: GitHub Fixture\nconsole.log("fixture");\n`,
  );

  const chromeTime = (Math.floor(Date.now() / 1000) + 11644473600) * 1000000;
  sql(
    join(dbDir, "History"),
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
VALUES
  (1, 'https://example.invalid/${fixtureToken}/history', 'Fixture browser history', 7, 2, ${chromeTime}),
  (2, 'https://example.invalid/other/history', 'Other browser history', 3, 1, ${chromeTime - 1000000});
`,
  );
}

function hash(input: unknown): string {
  return createHash("sha256").update(JSON.stringify(input)).digest("hex").slice(0, 16);
}

function percentile(values: number[], p: number): number {
  if (values.length === 0) {
    return 0;
  }
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

function queryKind(input: string): string {
  if (input.startsWith("tabs:")) return "tabs-long";
  if (input.startsWith("t:")) return "tabs-short";
  if (input.startsWith("history:")) return "history-long";
  if (input.startsWith("h:")) return "history-short";
  return "collision";
}

function sanitizePreflight(preflight: Json | null): Json | null {
  if (!preflight) {
    return null;
  }
  const visibleResults = Array.isArray(preflight.visibleResults)
    ? preflight.visibleResults.map((row: Json) => ({
        visibleRank: row.visibleRank,
        groupedIndex: row.groupedIndex,
        stableKeyHash: row.stableKey ? hash(row.stableKey) : null,
        role: row.role,
        actionKind: row.actionKind,
        typeLabel: row.typeLabel,
        sourceName: row.sourceName ?? null,
      }))
    : [];
  const frame = preflight.rootPassiveFrame;
  return {
    filterTextLength: typeof preflight.filterText === "string" ? preflight.filterText.length : 0,
    computedSearchTextLength:
      typeof preflight.computedSearchText === "string" ? preflight.computedSearchText.length : 0,
    sourceFilters: preflight.sourceFilters ?? [],
    selectedIndex: preflight.selectedIndex,
    selectedResultKeyHash: preflight.selectedResultKey ? hash(preflight.selectedResultKey) : null,
    selectedResultRole: preflight.selectedResultRole,
    enterAction: preflight.enterAction
      ? {
          kind: preflight.enterAction.kind,
          typeLabel: preflight.enterAction.typeLabel,
          sourceName: preflight.enterAction.sourceName ?? null,
        }
      : null,
    visibleResultKeyFingerprintHash: hash(preflight.visibleResultKeyFingerprint ?? ""),
    visibleRowFingerprintHash: hash(preflight.visibleRowFingerprint ?? ""),
    visibleResultCount: preflight.visibleResultCount ?? visibleResults.length,
    visibleResults,
    rootPassiveFrame: frame
      ? {
          queryLength: typeof frame.query === "string" ? frame.query.length : 0,
          sourceFilters: frame.sourceFilters ?? [],
          browserTabs: frame.browserTabs,
          browserHistory: frame.browserHistory,
        }
      : null,
  };
}

function elementsReceipt(elements: Json): Json {
  const all = Array.isArray(elements.elements) ? elements.elements : [];
  const rows = all.filter((element: Json) => element.role === "row" || element.kind === "row");
  return {
    totalCount: all.length,
    rowCount: rows.length,
    fingerprintHash: hash(
      all.map((element: Json) => ({
        role: element.role ?? null,
        kind: element.kind ?? null,
        index: element.index ?? null,
      })),
    ),
  };
}

function sourceRowStatus(preflight: Json | null) {
  const rows = Array.isArray(preflight?.visibleResults) ? preflight.visibleResults : [];
  const browserRows = rows.filter(
    (row: Json) => row.sourceName === "Browser Tabs" || row.sourceName === "Browser History",
  );
  const primaryRows = rows.filter((row: Json) => row.role === "primary");
  return {
    firstBrowserRowRank:
      browserRows.length > 0 ? Math.min(...browserRows.map((row: Json) => row.visibleRank)) : null,
    browserRowCount: browserRows.length,
    primaryRowCount: primaryRows.length,
    passiveBelowPrimary:
      browserRows.length === 0 || primaryRows.length === 0
        ? true
        : Math.min(...browserRows.map((row: Json) => row.visibleRank)) >
          Math.min(...primaryRows.map((row: Json) => row.visibleRank)),
    sourceNames: rows.map((row: Json) => row.sourceName ?? null).filter(Boolean),
    roles: rows.map((row: Json) => row.role),
  };
}

async function waitForBrowserSnapshotsSettled() {
  const deadline = Date.now() + timeoutMs;
  let last = getState("settle-start");
  while (Date.now() < deadline) {
    const frame = last.mainWindowPreflight?.rootPassiveFrame;
    if (
      frame &&
      !frame.browserTabs?.refreshing &&
      !frame.browserHistory?.refreshing
    ) {
      return sanitizePreflight(last.mainWindowPreflight);
    }
    await Bun.sleep(pollMs);
    last = getState("settle-poll");
  }
  throw new Error("browser passive snapshots did not settle before timeout");
}

async function measureInput(input: string, sampleIndex: number): Promise<Json> {
  send({
    type: "setFilter",
    text: "",
    requestId: `browser-source-latency-reset-${Date.now()}`,
  });
  waitForInput("");

  const sentAt = performance.now();
  const parse = send({
    type: "setFilter",
    text: input,
    requestId: `browser-source-latency-set-${Date.now()}`,
  });
  const inputWait = waitForInput(input);
  const state = getState(`${queryKind(input)}-${sampleIndex}`);
  const preflightAt = performance.now();
  const elements = getElements(`${queryKind(input)}-${sampleIndex}`);
  const preflight = state.mainWindowPreflight ?? null;
  const sanitized = sanitizePreflight(preflight);

  return {
    sampleIndex,
    queryKind: queryKind(input),
    queryLength: input.length,
    timings: {
      setFilterToParseMs: Number(parse.elapsedMs.toFixed(3)),
      setFilterToInputObservedMs: Number((parse.elapsedMs + inputWait.elapsedMs).toFixed(3)),
      setFilterToPreflightMs: Number((preflightAt - sentAt).toFixed(3)),
    },
    parseReceipt: {
      status: parse.receipt.status,
      parseOutcome: parse.receipt.parseOutcome,
      commandType: parse.receipt.commandType,
    },
    inputObserved: inputWait.receipt.type === "waitForResult",
    firstMatchingPreflight: sanitized,
    firstBrowserRow: sourceRowStatus(preflight),
    elements: elementsReceipt(elements),
  };
}

function parseRustPerf(logPath: string): Json {
  let content = "";
  try {
    content = readFileSync(logPath, "utf8");
  } catch {
    return { logPath, samples: [], stats: {} };
  }
  const samples = content
    .split("\n")
    .filter((line) => line.includes("root_browser_source_perf"))
    .map((line) => {
      const fields: Json = {};
      for (const match of line.matchAll(/([a-zA-Z_]+)=("[^"]*"|[^ ]+)/g)) {
        const key = match[1];
        const value = match[2];
        fields[key] = value.startsWith('"') ? value.slice(1, -1) : value;
      }
      return {
        source: fields.source ?? null,
        phase: fields.phase ?? null,
        reason: fields.reason ?? null,
        elapsedMs: Number(fields.elapsed_ms ?? fields.elapsedMs ?? 0),
        queryLength: Number(fields.query_len ?? fields.queryLength ?? 0),
        cachedCount: Number(fields.cached_count ?? fields.cachedCount ?? 0),
        hitCount: Number(fields.hit_count ?? fields.hitCount ?? 0),
        scanLimit: Number(fields.scan_limit ?? fields.scanLimit ?? 0),
        maxResults: Number(fields.max_results ?? fields.maxResults ?? 0),
      };
    })
    .filter((sample) => Number.isFinite(sample.elapsedMs));

  const foreground = samples.filter((sample) => sample.phase === "foreground_filter");
  const bySource: Json = {};
  for (const source of ["browser_tabs", "browser_history", "root_passive_frame", "grouping"]) {
    bySource[source] = stats(
      foreground.filter((sample) => sample.source === source).map((sample) => sample.elapsedMs),
    );
  }
  return {
    logPath,
    sampleCount: samples.length,
    foreground: stats(foreground.map((sample) => sample.elapsedMs)),
    bySource,
    samples: samples.slice(-100),
  };
}

function writeSummary(current: Json) {
  const baselinePath = join(outputDir, "baseline.json");
  const optimizedPath = join(outputDir, "optimized.json");
  const baseline = label === "baseline" ? current : tryReadJson(baselinePath);
  const optimized = label === "optimized" ? current : tryReadJson(optimizedPath);
  const lines = [
    "# Instant Browser Sources",
    "",
    `Current label: ${label}`,
    `Current status: ${current.status}`,
    "",
    "| Receipt | setFilter->preflight p50 | setFilter->preflight p95 | Rust foreground p50 | Rust foreground p95 | Rust foreground max |",
    "| --- | ---: | ---: | ---: | ---: | ---: |",
  ];
  for (const [name, receipt] of [
    ["baseline", baseline],
    ["optimized", optimized],
  ] as [string, Json | null][]) {
    if (!receipt) {
      lines.push(`| ${name} | n/a | n/a | n/a | n/a | n/a |`);
      continue;
    }
    lines.push(
      `| ${name} | ${receipt.summary.setFilterToPreflight.p50Ms} | ${receipt.summary.setFilterToPreflight.p95Ms} | ${receipt.rustPerf.foreground.p50Ms} | ${receipt.rustPerf.foreground.p95Ms} | ${receipt.rustPerf.foreground.maxMs} |`,
    );
  }
  writeFileSync(join(outputDir, "summary.md"), `${lines.join("\n")}\n`);
  writeFileSync(join(outputDir, "summary.json"), JSON.stringify({ baseline, optimized }, null, 2));
}

function tryReadJson(path: string): Json | null {
  try {
    return JSON.parse(readFileSync(path, "utf8"));
  } catch {
    return null;
  }
}

async function main() {
  mkdirSync(outputDir, { recursive: true });
  seedFixtureHome();
  runSession(["start", session]);
  const status = runSession(["status", session]);

  send({
    type: "setFilter",
    text: fixtureToken,
    requestId: `browser-source-latency-warm-${Date.now()}`,
  });
  waitForInput(fixtureToken);
  const warmup = await waitForBrowserSnapshotsSettled();
  sessionStatus = status;

  const queries = singleQuery
    ? [singleQuery]
    : ["t: ", `tabs: ${fixtureToken}`, "h: ", `history: ${fixtureToken}`, fixtureToken];
  const receipts: Json[] = [];
  for (let i = 0; i < samples; i += 1) {
    for (const input of queries) {
      receipts.push(await measureInput(input, i));
    }
  }

  const setFilterToPreflight = stats(
    receipts.map((receipt) => receipt.timings.setFilterToPreflightMs),
  );
  const setFilterToInputObserved = stats(
    receipts.map((receipt) => receipt.timings.setFilterToInputObservedMs),
  );
  const byQueryKind: Json = {};
  for (const kind of [...new Set(receipts.map((receipt) => receipt.queryKind))]) {
    byQueryKind[kind] = stats(
      receipts
        .filter((receipt) => receipt.queryKind === kind)
        .map((receipt) => receipt.timings.setFilterToPreflightMs),
    );
  }

  const rustPerf = parseRustPerf(status.log);
  const thresholdFailures = [];
  if (setFilterToPreflight.p50Ms > 50) thresholdFailures.push("setFilterToPreflight p50 > 50ms");
  if (setFilterToPreflight.p95Ms > 100) thresholdFailures.push("setFilterToPreflight p95 > 100ms");
  if ((rustPerf.foreground?.p50Ms ?? 0) > 5) thresholdFailures.push("Rust foreground p50 > 5ms");
  if ((rustPerf.foreground?.p95Ms ?? 0) > 16) thresholdFailures.push("Rust foreground p95 > 16ms");
  if ((rustPerf.foreground?.maxMs ?? 0) > 25) thresholdFailures.push("Rust foreground max > 25ms");

  const receipt = {
    schemaVersion: 1,
    label,
    status: thresholdFailures.length === 0 ? "pass" : "fail",
    thresholds: {
      setFilterToPreflightP50Ms: 50,
      setFilterToPreflightP95Ms: 100,
      rustForegroundP50Ms: 5,
      rustForegroundP95Ms: 16,
      rustForegroundMaxMs: 25,
      enforced: enforceThresholds,
      failures: thresholdFailures,
    },
    session: {
      name: session,
      logPath: status.log,
    },
    effectiveConfig: {
      browserTabsEnabled: true,
      browserHistoryEnabled: true,
      cacheTtlMs: 30000,
      scanLimit: 500,
    },
    warmup,
    summary: {
      sampleCount: receipts.length,
      setFilterToPreflight,
      setFilterToInputObserved,
      byQueryKind,
    },
    rustPerf,
    receipts,
  };

  const outputPath = join(outputDir, `${label}.json`);
  writeFileSync(outputPath, JSON.stringify(receipt, null, 2));
  writeSummary(receipt);
  process.stdout.write(`${JSON.stringify({ outputPath, status: receipt.status, summary: receipt.summary, rustPerf }, null, 2)}\n`);

  if (enforceThresholds && thresholdFailures.length > 0) {
    process.exit(1);
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
