#!/usr/bin/env bun
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "root-source-filter-lazy-scroll");
const query = argValue("--query", "sc");
const timeoutMs = Number(argValue("--timeout", "20000"));
const pollMs = 50;
const outputDir = join(repoRoot, ".test-output", "root-source-filter-lazy-scroll");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const recentDir = join(outputDir, "recent-files");
const sessionRoot = join(outputDir, "sessions");

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
  query,
  delayMs: 900,
  results: Array.from({ length: 30 }, (_, index) => ({
    path: `/tmp/${query}-provider-${String(index + 1).padStart(2, "0")}.txt`,
    name: `${query}-provider-${String(index + 1).padStart(2, "0")}.txt`,
    fileType: "document",
    size: 2048 + index,
    modified: Date.now() - index,
  })),
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
    throw new Error(`session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`);
  }
  const parsed = JSON.parse(stdout);
  if (result.status !== 0 || parsed.status === "error") {
    throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}\nstderr=${result.stderr.trim()}`);
  }
  return parsed;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  return runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeout),
  ]).response;
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
      requestId: `root-source-lazy-wait-input-${Date.now()}`,
      condition: { type: "stateMatch", state: { promptType: "none", inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function getState(tag: string): Json {
  const response = rpc(
    { type: "getState", requestId: `root-source-lazy-state-${tag}-${Date.now()}` },
    "stateResult",
  );
  if (response.type !== "stateResult") {
    throw new Error(`expected stateResult, got ${JSON.stringify(response)}`);
  }
  return response;
}

function getElements(tag: string): Json {
  const response = rpc(
    { type: "getElements", requestId: `root-source-lazy-elements-${tag}-${Date.now()}` },
    "elementsResult",
  );
  if (response.type !== "elementsResult") {
    throw new Error(`expected elementsResult, got ${JSON.stringify(response)}`);
  }
  return response;
}

function showWindow(tag: string): Json {
  const response = rpc(
    { type: "show", requestId: `root-source-lazy-show-${tag}-${Date.now()}` },
    "windowVisibilityAck",
  );
  return response;
}

function waitForWindowVisible(tag: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `root-source-lazy-visible-${tag}-${Date.now()}`,
      condition: { type: "stateMatch", state: { windowVisible: true } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function fileRows(elements: Json): Json[] {
  return (elements.elements ?? []).filter(
    (element: Json) => element.role === "row" && element.source === "files" && element.selectable === true,
  );
}

function statusRows(elements: Json): Json[] {
  return (elements.elements ?? []).filter(
    (element: Json) => element.role === "status" && element.kind === "sourceStatus" && element.source === "files",
  );
}

function assertStatusMetadata(status: Json | undefined, label: string) {
  if (!status) {
    throw new Error(`${label}: missing Files source status metadata`);
  }
  if (Object.prototype.hasOwnProperty.call(status, "index")) {
    throw new Error(`${label}: source status must not occupy a list index ${JSON.stringify(status)}`);
  }
  if (status.selectable !== false) {
    throw new Error(`${label}: source status must be non-selectable ${JSON.stringify(status)}`);
  }
}

function requirePreflight(state: Json, label: string): Json {
  if (!state.mainWindowPreflight) {
    throw new Error(`${label}: missing mainWindowPreflight`);
  }
  return state.mainWindowPreflight;
}

function requireScroll(state: Json, label: string): Json {
  const scroll = state.mainListScroll;
  if (!scroll) {
    throw new Error(`${label}: missing mainListScroll`);
  }
  if (scroll.scrollTop < 0) {
    throw new Error(`${label}: negative scrollTop ${JSON.stringify(scroll)}`);
  }
  if (scroll.scrollTop > scroll.maxScrollTop + 1) {
    throw new Error(`${label}: scrollTop exceeded maxScrollTop ${JSON.stringify(scroll)}`);
  }
  if (scroll.selectedRowVisible !== true) {
    throw new Error(`${label}: selected row is not visible ${JSON.stringify(scroll)}`);
  }
  if (scroll.selectedRowAboveFooter !== true) {
    throw new Error(`${label}: selected row is not above footer ${JSON.stringify(scroll)}`);
  }
  if (scroll.selectedRowBottom > scroll.safeViewportHeight + 1) {
    throw new Error(`${label}: selected row bottom is under footer ${JSON.stringify(scroll)}`);
  }
  return scroll;
}

async function waitForFileRowsAtLeast(count: number): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let last: Json | null = null;
  while (Date.now() < deadline) {
    last = getElements(`wait-${count}`);
    if (fileRows(last).length >= count) {
      return last;
    }
    await Bun.sleep(pollMs);
  }
  throw new Error(`timed out waiting for ${count} file rows; last=${JSON.stringify(last)}`);
}

async function waitForProviderSettled(input: string): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let last: Json | null = null;
  while (Date.now() < deadline) {
    last = getState("settle");
    if (
      last.inputValue === input &&
      last.rootFileSearch?.query === query &&
      last.rootFileSearch?.mode === "GlobalQuery" &&
      last.rootFileSearch?.loading === false &&
      last.mainListScroll &&
      last.mainListScroll.viewportHeight > 0 &&
      last.mainListScroll.safeViewportHeight > 0 &&
      last.mainListScroll.selectedRowVisible === true &&
      last.mainListScroll.selectedRowAboveFooter === true
    ) {
      return last;
    }
    await Bun.sleep(pollMs);
  }
  throw new Error(`timed out waiting for provider settle; last=${JSON.stringify(last)}`);
}

async function waitForMeasuredScrollState(input: string, tag: string): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let last: Json | null = null;
  while (Date.now() < deadline) {
    last = getState(tag);
    if (
      last.inputValue === input &&
      last.mainListScroll &&
      last.mainListScroll.viewportHeight > 0 &&
      last.mainListScroll.safeViewportHeight > 0
    ) {
      return last;
    }
    await Bun.sleep(pollMs);
  }
  throw new Error(`${input}: timed out waiting for measured mainListScroll; last=${JSON.stringify(last)}`);
}

async function waitForSelectedRowVisible(input: string, tag: string): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let last: Json | null = null;
  while (Date.now() < deadline) {
    last = await waitForMeasuredScrollState(input, tag);
    const scroll = last.mainListScroll;
    if (scroll.selectedRowVisible === true && scroll.selectedRowAboveFooter === true) {
      return last;
    }
    await Bun.sleep(pollMs);
  }
  throw new Error(`${input}: timed out waiting for selected row to be visible; last=${JSON.stringify(last)}`);
}

async function pressDown(times: number) {
  for (let index = 0; index < times; index += 1) {
    send({
      type: "simulateKey",
      key: "down",
      modifiers: [],
      requestId: `root-source-lazy-down-${index}-${Date.now()}`,
    });
    await Bun.sleep(20);
  }
}

function assertFilesSourceState(state: Json, input: string, expectedSearchText: string) {
  if (state.inputValue !== input) {
    throw new Error(`expected input ${input}, got ${state.inputValue}`);
  }
  const preflight = requirePreflight(state, input);
  if (preflight.computedSearchText !== expectedSearchText) {
    throw new Error(`${input}: expected computedSearchText ${expectedSearchText}, got ${preflight.computedSearchText}`);
  }
  if (JSON.stringify(preflight.sourceFilters) !== JSON.stringify(["files"])) {
    throw new Error(`${input}: expected sourceFilters [files], got ${JSON.stringify(preflight.sourceFilters)}`);
  }
  if (preflight.selectedResultRole !== "rootFile") {
    throw new Error(`${input}: selected row should be rootFile, got ${preflight.selectedResultRole}`);
  }
  if (!String(preflight.selectedResultKey ?? "").startsWith("file/")) {
    throw new Error(`${input}: selected key should be file/, got ${preflight.selectedResultKey}`);
  }
  for (const result of preflight.visibleResults ?? []) {
    if (result.role !== "rootFile") {
      throw new Error(`${input}: non-Files result appeared ${JSON.stringify(result)}`);
    }
  }
  requireScroll(state, input);
}

async function runLazyCase(input: string, expectedSearchText: string) {
  showWindow(input);
  waitForWindowVisible(input);
  send({ type: "setFilter", text: input, requestId: `root-source-lazy-set-${Date.now()}` });
  waitForInput(input);
  const beforeElements = await waitForFileRowsAtLeast(12);
  if (fileRows(beforeElements).length !== 12) {
    throw new Error(`${input}: expected initial Files page of 12 rows`);
  }
  if (statusRows(beforeElements).length !== 1) {
    throw new Error(`${input}: expected one source status metadata entry before paging`);
  }
  assertStatusMetadata(statusRows(beforeElements)[0], `${input}: before`);

  await pressDown(10);
  const afterPageElements = await waitForFileRowsAtLeast(24);
  const afterPageState = await waitForSelectedRowVisible(input, `after-page-${input}`);
  assertFilesSourceState(afterPageState, input, expectedSearchText);
  const selectedKeyAfterPage = requirePreflight(afterPageState, input).selectedResultKey;

  const settledState = expectedSearchText
    ? await waitForProviderSettled(input)
    : await waitForSelectedRowVisible(input, `settled-${input}`);
  assertFilesSourceState(settledState, input, expectedSearchText);
  const selectedKeyAfterSettle = requirePreflight(settledState, input).selectedResultKey;
  if (selectedKeyAfterSettle !== selectedKeyAfterPage) {
    throw new Error(
      `${input}: selected file key changed across provider settle/page reveal: ${selectedKeyAfterPage} -> ${selectedKeyAfterSettle}`,
    );
  }

  return {
    input,
    before: {
      fileRows: fileRows(beforeElements).length,
      status: statusRows(beforeElements)[0],
    },
    afterPage: {
      fileRows: fileRows(afterPageElements).length,
      selectedKey: selectedKeyAfterPage,
      scroll: afterPageState.mainListScroll,
    },
    settled: {
      selectedKey: selectedKeyAfterSettle,
      rootFileSearch: settledState.rootFileSearch,
      scroll: settledState.mainListScroll,
      visibleRowFingerprint: requirePreflight(settledState, input).visibleRowFingerprint,
    },
  };
}

async function main() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(kitDir, { recursive: true });
  mkdirSync(recentDir, { recursive: true });

  const entries: Record<string, Json> = {};
  for (let index = 0; index < 30; index += 1) {
    const path = join(recentDir, `${query}-recent-${String(index + 1).padStart(2, "0")}.txt`);
    writeFileSync(path, `${query} recent ${index}\n`);
    entries[`file/${path}`] = {
      count: 30 - index,
      last_used: Math.floor(Date.now() / 1000) - index,
    };
  }
  writeFileSync(join(kitDir, "frecency.json"), `${JSON.stringify({ entries })}\n`);
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {
  unifiedSearch: {
    notes: { enabled: false },
    clipboardHistory: { enabled: false },
    dictationHistory: { enabled: false },
    agent_chatHistory: { enabled: false },
    browserTabs: { enabled: false },
    browserHistory: { enabled: false },
  },
};
`,
  );

  runSession(["stop", session]);
  runSession(["start", session]);

  let passed = false;
  try {
    const browse = await runLazyCase("f:", "");
    const filtered = await runLazyCase(`f:${query}`, query);
    const receipt = {
      schemaVersion: 1,
      status: "pass",
      session,
      query,
      browse,
      filtered,
    };
    writeFileSync(join(outputDir, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
    process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
    passed = true;
  } finally {
    if (passed || process.env.SCRIPT_KIT_AGENTIC_KEEP_SESSION_ON_FAILURE !== "1") {
      runSession(["stop", session]);
    }
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
