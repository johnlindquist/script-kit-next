#!/usr/bin/env bun
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "colon-filter-picker-flow");
const timeoutMs = Number(argValue("--timeout", "18000"));
const pollMs = Number(argValue("--poll", "50"));
const outDir = join(repoRoot, ".test-output/colon-filter-picker-flow");
const receiptPath = join(outDir, "receipt.json");
const homeDir = join(outDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const agentBinary = join(repoRoot, "target-agent/pools/agent-debug/debug/script-kit-gpui");

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = join(outDir, "sessions");
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "14000";
if (!process.env.SCRIPT_KIT_GPUI_BINARY && existsSync(agentBinary)) {
  process.env.SCRIPT_KIT_GPUI_BINARY = agentBinary;
}

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function run(command: string, args: string[], options: { allowFailure?: boolean } = {}): string {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
    maxBuffer: 32 * 1024 * 1024,
  });
  if (result.status !== 0 && !options.allowFailure) {
    throw new Error(`${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`);
  }
  return result.stdout;
}

function runSession(args: string[]): Json {
  const stdout = run("bash", [sessionScript, ...args]).trim();
  if (!stdout) throw new Error(`session.sh ${args.join(" ")} produced no stdout`);
  const jsonLine = stdout
    .split(/\r?\n/)
    .reverse()
    .find((line) => line.trim().startsWith("{"));
  const parsed = JSON.parse(jsonLine ?? "{}");
  if (parsed.status === "error") throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}`);
  return parsed;
}

function send(command: Json): Json {
  return runSession(["send", session, JSON.stringify(command), "--await-parse", "--timeout", String(timeoutMs)]);
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

function waitForInput(input: string, tag: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `cfp-wait-${tag}-${Date.now()}`,
      condition: { type: "stateMatch", state: { promptType: "none", inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function getState(tag: string): Json {
  return rpc({ type: "getState", requestId: `cfp-state-${tag}-${Date.now()}` }, "stateResult");
}

function showWindow(tag: string): Json {
  return rpc({ type: "show", requestId: `cfp-show-${tag}-${Date.now()}` }, "windowVisibilityAck");
}

function waitForWindowVisible(visible: boolean, tag: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `cfp-wait-visible-${tag}-${Date.now()}`,
      condition: { type: "stateMatch", state: { windowVisible: visible } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function getElements(tag: string): Json {
  return rpc({ type: "getElements", requestId: `cfp-elements-${tag}-${Date.now()}`, limit: 100 }, "elementsResult");
}

function listAutomationWindows(tag: string): Json {
  return rpc(
    { type: "listAutomationWindows", requestId: `cfp-windows-${tag}-${Date.now()}` },
    "automationWindowListResult",
  );
}

function batch(commands: Json[], tag: string): Json {
  return rpc(
    {
      type: "batch",
      requestId: `cfp-batch-${tag}-${Date.now()}`,
      commands,
      trace: "on",
    },
    "batchResult",
  );
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function choiceRows(elements: Json): Json[] {
  return (elements.elements ?? []).filter((element: Json) => element.type === "choice");
}

function deprecatedDetachedPickerOpen(windows: Json): boolean {
  return (windows.windows ?? []).some((window: Json) => {
    if (window.visible !== true || String(window.kind ?? "").toLowerCase() !== "promptpopup") {
      return false;
    }
    const identity = [
      window.id,
      window.title,
      window.semanticSurface,
      window.surfaceId,
    ]
      .filter(Boolean)
      .join(" ")
      .toLowerCase();
    return identity.includes("menu") && identity.includes("syntax");
  });
}

function rowValues(elements: Json): string[] {
  return choiceRows(elements).map((row) => String(row.value ?? row.text ?? row.title ?? ""));
}

function findChoice(elements: Json, value: string): Json {
  const row = choiceRows(elements).find((element: Json) => element.value === value || element.text === value);
  assert(row, `missing choice row for ${value}; values=${JSON.stringify(rowValues(elements))}`);
  assert(typeof row.semanticId === "string", `${value} row missing semanticId`);
  return row;
}

async function waitForMainPickerValues(label: string, values: string[]): Promise<Json> {
  const started = Date.now();
  let lastElements: Json | null = null;
  let lastError = "";
  while (Date.now() - started < timeoutMs) {
    const elements = getElements(`${label}-settle`);
    lastElements = elements;
    try {
      assertMainPicker(elements, label);
      for (const value of values) {
        findChoice(elements, value);
      }
      return elements;
    } catch (error) {
      lastError = String(error?.message ?? error);
      await Bun.sleep(pollMs);
    }
  }
  throw new Error(
    `${label}: timed out waiting for picker values ${JSON.stringify(values)}; lastError=${lastError}; values=${JSON.stringify(
      lastElements ? rowValues(lastElements) : [],
    )}`,
  );
}

function assertMainPicker(elements: Json, label: string) {
  const list = (elements.elements ?? []).find((element: Json) => element.semanticId === "list:menu-syntax-trigger-picker");
  assert(
    list,
    `${label}: missing main-list menu syntax trigger picker element; semanticIds=${JSON.stringify(
      (elements.elements ?? []).map((element: Json) => element.semanticId),
    )}`,
  );
  const rows = choiceRows(elements);
  assert(rows.length > 0, `${label}: expected picker choice rows`);
  for (const row of rows) {
    assert(row.kind === "menuSyntaxTriggerPicker" || row.source === "menuSyntaxTriggerPicker", `${label}: row ${row.semanticId} not marked as menuSyntaxTriggerPicker`);
  }
}

function assertNoDetachedPopup(windows: Json, label: string) {
  assert(!deprecatedDetachedPickerOpen(windows), `${label}: detached menu-syntax prompt popup is visible`);
}

function seedShortcutFixtures() {
  mkdirSync(join(kitDir, "plugins", "main", "scripts"), { recursive: true });
  mkdirSync(join(kitDir, "plugins", "main", "scriptlets"), { recursive: true });
  writeFileSync(join(kitDir, "config.ts"), "export default {};\n");
  writeFileSync(
    join(kitDir, "plugins", "main", "scripts", "shortcut-fixture.ts"),
    [
      "// Name: Shortcut Fixture",
      "// Description: Deterministic has:shortcut script row",
      "// Shortcut: cmd shift 1",
      "await arg('Shortcut Fixture')",
      "",
    ].join("\n"),
  );
  writeFileSync(
    join(kitDir, "plugins", "main", "scriptlets", "shortcut-fixtures.md"),
    [
      "# Snippets",
      "",
      "## Run Snippet",
      "<!-- shortcut: cmd shift 2 -->",
      "<!-- description: Deterministic has:shortcut scriptlet row -->",
      "```ts",
      "await arg('Run Snippet')",
      "```",
      "",
    ].join("\n"),
  );
}

async function main() {
  run("bash", [sessionScript, "stop", session], { allowFailure: true });
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });
  mkdirSync(process.env.SCRIPT_KIT_SESSION_DIR!, { recursive: true });
  seedShortcutFixtures();

  const receipt: Json = {
    schemaVersion: 1,
    status: "running",
    session,
    binary: process.env.SCRIPT_KIT_GPUI_BINARY,
    checks: {},
    protocol: ["show", "setFilter", "waitFor", "getState", "getElements", "batch.selectBySemanticId", "listAutomationWindows"],
  };

  try {
    receipt.start = runSession(["start", session]);
    receipt.show = showWindow("initial");
    receipt.visible = waitForWindowVisible(true, "initial");

    send({ type: "setFilter", text: ":", requestId: "cfp-set-colon" });
    waitForInput(":", "colon");
    const colonState = getState("colon");
    const colonElements = await waitForMainPickerValues("colon", ["has:", "type:"]);
    const colonWindows = listAutomationWindows("colon");
    assertNoDetachedPopup(colonWindows, "colon");
    const hasRow = findChoice(colonElements, "has:");
    receipt.checks.colon = { state: colonState, elements: colonElements, windows: colonWindows, hasSemanticId: hasRow.semanticId };

    const selectHas = batch([{ type: "selectBySemanticId", semanticId: hasRow.semanticId, submit: true }], "select-has");
    assert(selectHas.success === true, `select has: failed: ${JSON.stringify(selectHas)}`);
    waitForInput("has:", "has-head");
    const hasState = getState("has-head");
    const hasElements = await waitForMainPickerValues("has-head", [
      "has:shortcut",
      "has:alias",
      "has:menuSyntax",
    ]);
    const hasWindows = listAutomationWindows("has-head");
    assertNoDetachedPopup(hasWindows, "has-head");
    assert(hasState.menuSyntaxMainHint?.activeHead === "has:", `has-head activeHead mismatch: ${JSON.stringify(hasState.menuSyntaxMainHint)}`);
    receipt.checks.hasHead = { selectHas, state: hasState, elements: hasElements, windows: hasWindows };

    send({ type: "setFilter", text: "has:sh", requestId: "cfp-set-has-sh" });
    waitForInput("has:sh", "has-sh");
    const hasShState = getState("has-sh");
    const hasShElements = await waitForMainPickerValues("has-sh", ["has:shortcut"]);
    const hasShWindows = listAutomationWindows("has-sh");
    assertNoDetachedPopup(hasShWindows, "has-sh");
    const hasShValues = rowValues(hasShElements);
    assert(hasShValues.includes("has:shortcut"), `has:sh missing has:shortcut: ${JSON.stringify(hasShValues)}`);
    assert(!hasShValues.includes("has:alias"), `has:sh should not include has:alias: ${JSON.stringify(hasShValues)}`);
    const shortcutRow = findChoice(hasShElements, "has:shortcut");
    receipt.checks.hasSh = { state: hasShState, elements: hasShElements, windows: hasShWindows, shortcutSemanticId: shortcutRow.semanticId };

    const selectShortcut = batch([{ type: "selectBySemanticId", semanticId: shortcutRow.semanticId, submit: true }], "select-shortcut");
    assert(selectShortcut.success === true, `select has:shortcut failed: ${JSON.stringify(selectShortcut)}`);
    waitForInput("has:shortcut", "terminal");
    await Bun.sleep(50);
    const terminalState = getState("terminal");
    const terminalElements = getElements("terminal");
    const terminalWindows = listAutomationWindows("terminal");
    assertNoDetachedPopup(terminalWindows, "terminal");
    assert(!terminalState.menuSyntaxMainHint, `terminal has:shortcut still exposes menuSyntaxMainHint: ${JSON.stringify(terminalState.menuSyntaxMainHint)}`);
    assert((terminalState.visibleChoiceCount ?? 0) > 0, `terminal expected filtered results, got ${terminalState.visibleChoiceCount}`);
    assert(choiceRows(terminalElements).length === terminalState.visibleChoiceCount, "terminal getElements/count mismatch");
    assert(!(terminalElements.elements ?? []).some((element: Json) => element.semanticId === "list:menu-syntax-trigger-picker"), "terminal still exposes trigger picker list");
    receipt.checks.terminal = { selectShortcut, state: terminalState, elements: terminalElements, windows: terminalWindows };

    receipt.status = "pass";
  } finally {
    run("bash", [sessionScript, "stop", session], { allowFailure: true });
    receipt.stopped = run("bash", [sessionScript, "status", session], { allowFailure: true }).trim();
    mkdirSync(outDir, { recursive: true });
    writeFileSync(receiptPath, JSON.stringify(receipt, null, 2));
  }

  console.log(JSON.stringify({ status: receipt.status, receiptPath: receiptPath.replace(`${repoRoot}/`, "") }, null, 2));
}

main().catch((error) => {
  mkdirSync(outDir, { recursive: true });
  writeFileSync(receiptPath, JSON.stringify({ schemaVersion: 1, status: "fail", error: String(error?.stack ?? error) }, null, 2));
  console.error(error);
  process.exit(1);
});
