#!/usr/bin/env bun
import { copyFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "root-source-filter-stability");
const query = argValue("--query", "zzqxfilters");
const timeoutMs = Number(argValue("--timeout", "10000"));
const pollMs = Number(argValue("--poll", "25"));
const outputDir = join(repoRoot, ".test-output", "root-source-filter-stability");
const sessionRoot = process.env.SCRIPT_KIT_SESSION_DIR ?? "/tmp/sk-agentic-sessions";

process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
  query,
  delayMs: 250,
  results: [
    {
      path: `/tmp/${query}-late-provider-result.txt`,
      name: `${query}-late-provider-result.txt`,
      fileType: "document",
      size: 42,
      modified: 1,
    },
  ],
});

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function runSession(args: string[]): Json {
  const result = spawnSync(sessionScript, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
  });
  const stdout = result.stdout.trim();
  if (!stdout) {
    throw new Error(
      `session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`,
    );
  }
  const parsed = JSON.parse(stdout);
  if (result.status !== 0 || parsed.status === "error") {
    throw new Error(
      `session.sh ${args.join(" ")} failed: ${JSON.stringify(parsed)} stderr=${result.stderr.trim()}`,
    );
  }
  return parsed;
}

function sessionLogPath(sessionName: string): string {
  return join(sessionRoot, sessionName, "app.log");
}

function sessionResponsesPath(sessionName: string): string {
  return join(sessionRoot, sessionName, "responses.ndjson");
}

function rpcFor(sessionName: string, command: Json, expect: string, timeout = timeoutMs): Json {
  const envelope = runSession([
    "rpc",
    sessionName,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeout),
  ]);
  return envelope.response;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  return rpcFor(session, command, expect, timeout);
}

function sendFor(sessionName: string, command: Json): Json {
  return runSession([
    "send",
    sessionName,
    JSON.stringify(command),
    "--await-parse",
    "--timeout",
    String(timeoutMs),
  ]);
}

function send(command: Json): Json {
  return sendFor(session, command);
}

function getState(tag: string): Json {
  const state = rpc(
    {
      type: "getState",
      requestId: `root-source-filter-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(state)}`);
  }
  return state;
}

function waitForInputFor(sessionName: string, input: string): Json {
  return rpcFor(
    sessionName,
    {
      type: "waitFor",
      requestId: `root-source-filter-wait-${Date.now()}`,
      condition: {
        type: "stateMatch",
        state: {
          promptType: "none",
          inputValue: input,
        },
      },
      timeout: timeoutMs,
      pollInterval: Math.max(25, Math.min(pollMs, 250)),
    },
    "waitForResult",
  );
}

function waitForInput(input: string): Json {
  return waitForInputFor(session, input);
}

function requirePreflight(state: Json, label: string): Json {
  const preflight = state.mainWindowPreflight;
  if (!preflight) {
    throw new Error(`${label}: missing mainWindowPreflight`);
  }
  return preflight;
}

function comparable(state: Json, label: string): Json {
  const preflight = requirePreflight(state, label);
  return {
    inputValue: state.inputValue,
    computedSearchText: preflight.computedSearchText,
    sourceFilters: preflight.sourceFilters,
    filterIndicators: preflight.filterIndicators,
    selectedResultKey: preflight.selectedResultKey ?? null,
    selectedResultRole: preflight.selectedResultRole,
    visibleResultKeyFingerprint: preflight.visibleResultKeyFingerprint,
    visibleRowFingerprint: preflight.visibleRowFingerprint,
    visibleResultCount: preflight.visibleResultCount,
    visibleResults: preflight.visibleResults,
    rootFileSearch: state.rootFileSearch,
  };
}

function assertFilesOnly(frame: Json, input: string) {
  if (frame.computedSearchText !== query) {
    throw new Error(`${input}: expected computedSearchText ${query}, got ${frame.computedSearchText}`);
  }
  if (JSON.stringify(frame.sourceFilters) !== JSON.stringify(["files"])) {
    throw new Error(`${input}: expected sourceFilters [files], got ${JSON.stringify(frame.sourceFilters)}`);
  }
  const indicators = frame.filterIndicators ?? [];
  const hasFilesIndicator = indicators.some((indicator: Json) => {
    return indicator.id === "files" && indicator.head === "files" && indicator.negated === false;
  });
  if (!hasFilesIndicator) {
    throw new Error(`${input}: expected files filter indicator, got ${JSON.stringify(indicators)}`);
  }
  if (frame.rootFileSearch?.query !== query) {
    throw new Error(`${input}: provider query was not stripped: ${JSON.stringify(frame.rootFileSearch)}`);
  }
  for (const result of frame.visibleResults ?? []) {
    if (result.role !== "rootFile") {
      throw new Error(`${input}: disallowed visible result ${JSON.stringify(result)}`);
    }
  }
}

function assertLiveFilesOnly(frame: Json, input: string, expectedSearchText: string) {
  if (frame.computedSearchText !== expectedSearchText) {
    throw new Error(
      `${input}: expected computedSearchText ${expectedSearchText}, got ${frame.computedSearchText}`,
    );
  }
  if (JSON.stringify(frame.sourceFilters) !== JSON.stringify(["files"])) {
    throw new Error(`${input}: expected sourceFilters [files], got ${JSON.stringify(frame.sourceFilters)}`);
  }
  const indicators = frame.filterIndicators ?? [];
  const hasFilesIndicator = indicators.some((indicator: Json) => {
    return indicator.id === "files" && indicator.head === "files" && indicator.negated === false;
  });
  if (!hasFilesIndicator) {
    throw new Error(`${input}: expected files filter indicator, got ${JSON.stringify(indicators)}`);
  }
  if (frame.rootFileSearch?.query !== expectedSearchText) {
    throw new Error(`${input}: provider query was not stripped: ${JSON.stringify(frame.rootFileSearch)}`);
  }
  for (const result of frame.visibleResults ?? []) {
    if (result.role !== "rootFile") {
      throw new Error(`${input}: disallowed visible result ${JSON.stringify(result)}`);
    }
  }
}

function assertNoPowerUi(frame: Json, input: string) {
  if ("menuSyntaxMainHint" in frame && frame.menuSyntaxMainHint != null) {
    throw new Error(`${input}: source filter exposed menuSyntaxMainHint`);
  }
}

function assertSameFrame(before: Json, after: Json, label: string) {
  const stableBefore = {
    visibleResultKeyFingerprint: before.visibleResultKeyFingerprint,
    visibleRowFingerprint: before.visibleRowFingerprint,
    visibleResultCount: before.visibleResultCount,
    visibleResults: before.visibleResults,
  };
  const stableAfter = {
    visibleResultKeyFingerprint: after.visibleResultKeyFingerprint,
    visibleRowFingerprint: after.visibleRowFingerprint,
    visibleResultCount: after.visibleResultCount,
    visibleResults: after.visibleResults,
  };
  if (JSON.stringify(stableBefore) !== JSON.stringify(stableAfter)) {
    throw new Error(
      `${label}: visible frame changed for same source-filter input\nbefore=${JSON.stringify(stableBefore)}\nafter=${JSON.stringify(stableAfter)}`,
    );
  }
}

async function runCase(input: string): Promise<Json> {
  send({
    type: "setFilter",
    text: input,
    requestId: `root-source-filter-set-${Date.now()}`,
  });
  waitForInput(input);

  const beforeState = getState(`before-${input}`);
  const baseline = comparable(beforeState, `before-${input}`);
  assertNoPowerUi(beforeState, input);
  assertFilesOnly(baseline, input);

  const samples: Json[] = [];
  const deadline = Date.now() + timeoutMs;
  let settled: Json | null = null;
  while (Date.now() < deadline) {
    await Bun.sleep(Math.max(25, pollMs));
    const state = getState(`sample-${samples.length}`);
    const frame = comparable(state, `sample-${samples.length}`);
    assertNoPowerUi(state, input);
    assertFilesOnly(frame, input);
    assertSameFrame(baseline, frame, `${input} sample ${samples.length}`);
    samples.push(frame);
    if (frame.rootFileSearch?.loading === false) {
      settled = frame;
      break;
    }
  }
  if (!settled) {
    throw new Error(`${input}: root file provider did not settle`);
  }

  return {
    input,
    baseline,
    samples,
    settled,
  };
}

async function runLivePngCase(): Promise<Json> {
  const input = "png files:";
  const expectedSearchText = "png";
  send({
    type: "setFilter",
    text: input,
    requestId: `root-source-filter-live-png-${Date.now()}`,
  });
  waitForInput(input);

  const deadline = Date.now() + timeoutMs;
  let lastFrame: Json | null = null;
  while (Date.now() < deadline) {
    await Bun.sleep(Math.max(25, pollMs));
    const state = getState(`live-png-${Date.now()}`);
    assertNoPowerUi(state, input);
    const frame = comparable(state, "live-png");
    assertLiveFilesOnly(frame, input, expectedSearchText);
    lastFrame = frame;

    const pngResult = (frame.visibleResults ?? []).find((result: Json) => {
      const haystack = [
        result.stableKey,
        result.title,
        result.subtitle,
        frame.rootFileSearch?.query,
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase();
      return result.role === "rootFile" && haystack.includes(".png");
    });
    if (pngResult) {
      return {
        input,
        frame,
        pngResult,
      };
    }
  }

  throw new Error(
    `${input}: did not observe a visible PNG root file result; lastFrame=${JSON.stringify(lastFrame)}`,
  );
}

function assert_source_session_did_not_open_power_picker() {
  const logPath = sessionLogPath(session);
  const log = existsSync(logPath) ? readFileSync(logPath, "utf8") : "";
  if (log.includes("menu_syntax_trigger_picker_render")) {
    throw new Error("source-filter session rendered the menu syntax trigger picker");
  }
  if (existsSync(logPath)) {
    copyFileSync(logPath, join(outputDir, "app.log"));
  }
  const responsesPath = sessionResponsesPath(session);
  if (existsSync(responsesPath)) {
    copyFileSync(responsesPath, join(outputDir, "responses.ndjson"));
  }
}

function prove_semicolon_still_opens_capture_picker(): Json {
  const semicolonSession = `${session}-semicolon`;
  runSession(["stop", semicolonSession]);
  runSession(["start", semicolonSession]);
  sendFor(semicolonSession, {
    type: "setFilter",
    text: ";",
    requestId: `root-source-filter-semicolon-${Date.now()}`,
  });
  waitForInputFor(semicolonSession, ";");
  const logPath = sessionLogPath(semicolonSession);
  const log = existsSync(logPath) ? readFileSync(logPath, "utf8") : "";
  if (!log.includes("menu_syntax_trigger_picker_render")) {
    throw new Error("semicolon did not render the menu syntax trigger picker");
  }
  copyFileSync(logPath, join(outputDir, "semicolon-app.log"));
  const responsesPath = sessionResponsesPath(semicolonSession);
  if (existsSync(responsesPath)) {
    copyFileSync(responsesPath, join(outputDir, "semicolon-responses.ndjson"));
  }
  runSession(["stop", semicolonSession]);
  return {
    session: semicolonSession,
    pickerRenderLogged: true,
  };
}

async function warmProvider() {
  send({
    type: "setFilter",
    text: query,
    requestId: `root-source-filter-warm-${Date.now()}`,
  });
  waitForInput(query);
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const state = getState("warm");
    const status = state.rootFileSearch;
    if (
      status?.query === query &&
      status?.mode === "GlobalQuery" &&
      status?.loading === false &&
      (status?.cacheResultCount ?? 0) > 0
    ) {
      return status;
    }
    await Bun.sleep(Math.max(25, pollMs));
  }
  throw new Error(`provider did not warm for ${query}`);
}

async function main() {
  mkdirSync(outputDir, { recursive: true });
  runSession(["stop", session]);
  runSession(["start", session]);
  const warmed = await warmProvider();

  const cases = [];
  for (const input of [`f: ${query}`, `${query} f:`, `${query} files:`]) {
    cases.push(await runCase(input));
  }
  const livePngCase = await runLivePngCase();
  assert_source_session_did_not_open_power_picker();
  const semicolonProof = prove_semicolon_still_opens_capture_picker();
  runSession(["stop", session]);

  const receipt = {
    schemaVersion: 1,
    status: "pass",
    session,
    query,
    warmed,
    powerUi: {
      sourceFilterPickerRenderLogged: false,
      sourceFilterHintObserved: false,
      semicolonProof,
    },
    cases,
    livePngCase,
  };
  const receiptPath = join(outputDir, "receipt.json");
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
