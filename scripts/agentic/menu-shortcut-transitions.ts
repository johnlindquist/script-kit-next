#!/usr/bin/env bun
import { existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "menu-shortcut-transitions");
const timeoutMs = Number(argValue("--timeout", "16000"));
const pollMs = Number(argValue("--poll", "50"));
const outDir = join(repoRoot, ".test-output/menu-shortcut-transitions");
const receiptPath = join(outDir, "receipt.json");
const homeDir = join(outDir, "home");
const kitDir = join(homeDir, ".scriptkit");

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = join(outDir, "sessions");
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "12000";

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
  const parsed = JSON.parse(stdout.split(/\r?\n/).reverse().find((line) => line.trim().startsWith("{")) ?? "{}");
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
      requestId: `mst-wait-${tag}-${Date.now()}`,
      condition: { type: "stateMatch", state: { promptType: "none", inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function getState(tag: string): Json {
  return rpc({ type: "getState", requestId: `mst-state-${tag}-${Date.now()}` }, "stateResult");
}

function getElements(tag: string, target?: Json): Json {
  return rpc(
    {
      type: "getElements",
      requestId: `mst-elements-${tag}-${Date.now()}`,
      ...(target ? { target } : {}),
      limit: 100,
    },
    "elementsResult",
  );
}

function inspectAutomationWindow(tag: string, target: Json): Json {
  return rpc(
    {
      type: "inspectAutomationWindow",
      requestId: `mst-inspect-${tag}-${Date.now()}`,
      target,
    },
    "automationInspectResult",
  );
}

function listAutomationWindows(tag: string): Json {
  return rpc(
    { type: "listAutomationWindows", requestId: `mst-windows-${tag}-${Date.now()}` },
    "automationWindowListResult",
  );
}

function batch(commands: Json[], tag: string, target?: Json): Json {
  return rpc(
    {
      type: "batch",
      requestId: `mst-batch-${tag}-${Date.now()}`,
      ...(target ? { target } : {}),
      commands,
      trace: "on",
    },
    "batchResult",
  );
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function popupOpen(windows: Json): boolean {
  return (windows.windows ?? []).some((window: Json) => window.id === "menu-syntax-trigger-popup" && window.visible === true);
}

function choiceRows(elements: Json): Json[] {
  return (elements.elements ?? []).filter((element: Json) => element.type === "choice");
}

function assertShortcutResults(state: Json, elements: Json, label: string) {
  assert(state.inputValue === "has:shortcut", `${label}: expected has:shortcut input, got ${state.inputValue}`);
  assert((state.visibleChoiceCount ?? 0) > 0, `${label}: expected visibleChoiceCount > 0, got ${state.visibleChoiceCount}`);
  const rows = choiceRows(elements);
  assert(rows.length === state.visibleChoiceCount, `${label}: getElements rows ${rows.length} != visibleChoiceCount ${state.visibleChoiceCount}`);
  assert(!state.menuSyntaxMainHint, `${label}: complete has:shortcut should not expose empty menuSyntaxMainHint`);
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
    checks: {},
    protocol: ["setFilter", "batch", "simulateKey", "waitFor", "getState", "getElements", "inspectAutomationWindow"],
  };

  try {
    receipt.start = runSession(["start", session]);

    send({ type: "setFilter", text: "has:shortc", requestId: "mst-set-partial" });
    waitForInput("has:shortc", "partial");
    const partialState = getState("partial");
    const partialWindows = listAutomationWindows("partial");
    assert(popupOpen(partialWindows), `partial has:shortc did not open popup: ${JSON.stringify(partialWindows.windows ?? [])}`);
    const partialPopup = getElements("partial-popup", { type: "id", id: "menu-syntax-trigger-popup" });
    const partialInspect = inspectAutomationWindow("partial-popup", { type: "id", id: "menu-syntax-trigger-popup" });
    assert(JSON.stringify(partialPopup).includes("has:shortcut"), "partial popup elements missing has:shortcut");
    assert(partialState.menuSyntaxMainHint?.activeHead === "has:", `partial activeHead mismatch: ${JSON.stringify(partialState.menuSyntaxMainHint)}`);
    receipt.checks.partial = { state: partialState, windows: partialWindows, popup: partialPopup, inspect: partialInspect };

    send({ type: "simulateKey", key: "enter", modifiers: [], requestId: "mst-accept-enter" });
    waitForInput("has:shortcut", "accepted");
    await Bun.sleep(25);
    const acceptedState = getState("accepted");
    const acceptedElements = getElements("accepted");
    const acceptedWindows = listAutomationWindows("accepted");
    assert(!popupOpen(acceptedWindows), "popup stayed open after Enter accept");
    assertShortcutResults(acceptedState, acceptedElements, "accepted");
    receipt.checks.accepted = { state: acceptedState, elements: acceptedElements, windows: acceptedWindows };

    await Bun.sleep(50);
    const tickState = getState("tick");
    const tickElements = getElements("tick");
    const tickWindows = listAutomationWindows("tick");
    assert(!popupOpen(tickWindows), "popup reopened one render tick after Enter");
    assertShortcutResults(tickState, tickElements, "tick");
    receipt.checks.tick = { state: tickState, elements: tickElements, windows: tickWindows };

    send({ type: "setFilter", text: "has:shortcut ", requestId: "mst-space" });
    waitForInput("has:shortcut ", "space");
    const spaceWindows = listAutomationWindows("space");
    const spaceState = getState("space");
    const spaceElements = getElements("space");
    assert(!popupOpen(spaceWindows), "popup reopened after trailing Space");
    assert((spaceState.visibleChoiceCount ?? 0) > 0, `space: expected rows, got ${spaceState.visibleChoiceCount}`);
    assert(choiceRows(spaceElements).length === spaceState.visibleChoiceCount, "space: elements/count mismatch");
    receipt.checks.space = { state: spaceState, elements: spaceElements, windows: spaceWindows };

    batch(
      [
        { type: "setInput", text: "has:shortcut" },
        {
          type: "waitFor",
          condition: { type: "stateMatch", state: { promptType: "none", inputValue: "has:shortcut" } },
          timeout: timeoutMs,
          pollInterval: pollMs,
        },
      ],
      "batch-reset",
    );
    const batchState = getState("batch-reset");
    const batchElements = getElements("batch-reset");
    assertShortcutResults(batchState, batchElements, "batch-reset");
    receipt.checks.batchReset = { state: batchState, elements: batchElements };

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
