#!/usr/bin/env bun
/**
 * scripts/agentic/input-ambiguity-probe.ts
 *
 * Runtime proof for the 2026-06-09 input-ambiguity decisions (see
 * .notes/20260609-input-ambiguity-decisions.md):
 *
 *   A1  exact alias match pins the aliased command at index 0; typing an
 *       alias (even with a trailing space) never auto-executes.
 *   A2  Tab opens the cwd picker ONLY when the main input is empty.
 *   A7  ghost predictions are disabled; backquote types a literal char.
 *   A8  Up-arrow history recall fires when input is empty at top of list,
 *       continues deeper into history once navigating, and never fires
 *       mid-query.
 *   A5  multi-line Cmd+V on the script list routes to Agent Chat.
 *
 * Key events that live in GPUI interceptors (Tab/Up/Cmd+V) are driven with
 * `simulateGpuiEvent`, which dispatches through window.dispatch_keystroke —
 * the legacy `simulateKey` stdin surface bypasses interceptors and would
 * silently test the wrong path.
 *
 * The launch is two-phase: first-run scaffolding rebuilds ~/.scriptkit in the
 * sandbox, wiping pre-seeded files, so we launch once to scaffold, close,
 * seed aliases.json + input_history.json, then relaunch for the real probe.
 *
 * Usage: bun scripts/agentic/input-ambiguity-probe.ts
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/input-ambiguity/script-kit-gpui");
const OUT_DIR = join(
  PROJECT_ROOT,
  `.test-output/input-ambiguity/${process.pid}`,
);
const SESSION_DIR = `/tmp/sk-input-ambiguity-${process.pid}`;

// ScriptList reports promptType "none" in stateResult.
const SCRIPT_LIST = "none";

interface StepResult {
  step: string;
  pass: boolean;
  details: Json;
}
const results: StepResult[] = [];
function record(step: string, pass: boolean, details: Json = {}) {
  results.push({ step, pass, details });
}

async function shot(driver: Driver, name: string): Promise<string | null> {
  const savePath = join(OUT_DIR, name);
  const res = await driver.captureScreenshot({
    savePath,
    target: { type: "kind", kind: "main" },
  });
  if (res.error) {
    record(`screenshot-${name}`, false, { error: res.error });
    return null;
  }
  return savePath;
}

function gpuiKey(
  driver: Driver,
  key: string,
  modifiers: string[] = [],
  text?: string,
): Promise<Json> {
  const event: Json = { type: "keyDown", key, modifiers };
  if (text !== undefined) event.text = text;
  return driver.request(
    { type: "simulateGpuiEvent", target: { type: "kind", kind: "main" }, event },
    { expect: "simulateGpuiEventResult" },
  );
}

async function waitUntil(
  driver: Driver,
  predicate: (state: Json) => boolean,
  timeoutMs = 4000,
): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let state = await driver.getState();
  while (!predicate(state) && Date.now() < deadline) {
    await Bun.sleep(100);
    state = await driver.getState();
  }
  return state;
}

async function main() {
  mkdirSync(OUT_DIR, { recursive: true });

  // --- phase 0: scaffold the sandbox, then seed -----------------------------
  const scaffold = await Driver.launch({
    binary: BINARY,
    sessionName: "input-ambiguity-scaffold",
    sessionDir: SESSION_DIR,
    sandboxHome: true,
    defaultTimeoutMs: 8000,
  });
  await scaffold.getState();
  await scaffold.close();

  const sk = join(SESSION_DIR, "home", ".scriptkit");
  // Alias override: "zz" → Clipboard History builtin. "zz" fuzzy-matches
  // nothing, so the pin must use the synthetic-fallback path too.
  writeFileSync(
    join(sk, "aliases.json"),
    JSON.stringify({ "builtin/clipboard-history": "zz" }),
  );
  // Input history, most recent first.
  writeFileSync(
    join(sk, "input_history.json"),
    JSON.stringify({
      entries: ["recent query", "older query"],
      selected_results: {},
    }),
  );

  // --- phase 1: the real probe ----------------------------------------------
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: "input-ambiguity",
    sessionDir: join(SESSION_DIR, "run"),
    sandboxHome: false,
    env: { HOME: join(SESSION_DIR, "home"), SK_PATH: sk },
    defaultTimeoutMs: 8000,
  });

  // Save and later restore the user clipboard around the Cmd+V proof.
  const savedClipboard = await Bun.$`pbpaste`.text().catch(() => "");

  try {
    driver.send({ type: "show" });
    await driver.waitForState({ windowVisible: true });

    // === A1: alias pin ======================================================
    await driver.setFilterAndWait("zz");
    await Bun.sleep(200);
    const a1Elements = await driver.getElements();
    const a1State = await driver.getState();
    const flat = JSON.stringify(a1Elements);
    record("a1-alias-pins-clipboard-history-first", flat.includes("Clipboard History"), {
      selectedIndex: a1State.selectedIndex,
      promptType: a1State.promptType,
      visibleChoiceCount: a1State.visibleChoiceCount,
      treeMentionsClipboardHistory: flat.includes("Clipboard History"),
    });
    record(
      "a1-selected-index-0",
      a1State.selectedIndex === 0 && a1State.promptType === SCRIPT_LIST,
      { selectedIndex: a1State.selectedIndex, promptType: a1State.promptType },
    );
    const a1Shot = await shot(driver, "a1-alias-pin.png");

    // Trailing space must NOT auto-execute (view stays scriptList).
    await driver.setFilterAndWait("zz ");
    await Bun.sleep(200);
    const a1Space = await driver.getState();
    record("a1-trailing-space-does-not-execute", a1Space.promptType === SCRIPT_LIST, {
      promptType: a1Space.promptType,
      inputValue: a1Space.inputValue,
    });

    // === A2: Tab → cwd picker only when input empty =========================
    await driver.setFilterAndWait("");
    // With text typed, Tab must NOT open the picker.
    await driver.setFilterAndWait("clip");
    await gpuiKey(driver, "tab");
    await Bun.sleep(200);
    const a2Typed = await driver.getState();
    record(
      "a2-tab-with-text-stays-on-script-list",
      a2Typed.promptType === SCRIPT_LIST && a2Typed.inputValue === "clip",
      { promptType: a2Typed.promptType, inputValue: a2Typed.inputValue },
    );

    // With empty input, Tab opens the cwd picker (FileSearch surface, input "~/").
    await driver.setFilterAndWait("");
    await gpuiKey(driver, "tab");
    const a2Empty = await waitUntil(
      driver,
      (s) => s.promptType === "fileSearch" && s.inputValue === "~/",
    );
    record(
      "a2-tab-empty-opens-cwd-picker",
      a2Empty.promptType === "fileSearch" && a2Empty.inputValue === "~/",
      { promptType: a2Empty.promptType, inputValue: a2Empty.inputValue },
    );
    const a2Shot = await shot(driver, "a2-cwd-picker.png");
    // Escape back to the script list.
    await gpuiKey(driver, "escape");
    await waitUntil(driver, (s) => s.promptType === SCRIPT_LIST);
    await driver.setFilterAndWait("");

    // === A7: backquote types a literal char, no ghost accept ================
    await gpuiKey(driver, "`", [], "`");
    await Bun.sleep(200);
    const a7State = await driver.getState();
    record("a7-backquote-types-literal-char", a7State.inputValue === "`", {
      inputValue: a7State.inputValue,
      promptType: a7State.promptType,
    });
    await driver.setFilterAndWait("");

    // === A8: history recall ================================================
    // Mid-query first (before any history navigation exists): Up must NOT
    // recall history — the typed text stays. NOTE: this must run before the
    // recall tests because protocol setFilter bypasses the typing path that
    // resets history navigation (handle_filter_input_change), so a stale
    // history index from an earlier recall would leak into this step.
    await driver.setFilterAndWait("abc");
    await gpuiKey(driver, "up");
    await Bun.sleep(300);
    const a8Typed = await driver.getState();
    record("a8-up-mid-query-does-not-recall", a8Typed.inputValue === "abc", {
      inputValue: a8Typed.inputValue,
    });
    await driver.setFilterAndWait("");

    // Empty input at top → Up recalls most recent entry.
    await gpuiKey(driver, "up");
    const a8First = await waitUntil(driver, (s) => s.inputValue !== "");
    record("a8-up-on-empty-recalls-recent", a8First.inputValue === "recent query", {
      inputValue: a8First.inputValue,
    });
    // Up again → continues deeper into history (the continuation fix).
    // Consecutive Ups are coalesced until the recalled filter has rendered
    // (key-repeat guard, cleared by a render ack). A live window acks within
    // one frame; the headless probe window renders lazily, so press Up
    // user-style until the guard clears (bounded retries).
    let a8Second: Json = {};
    for (let attempt = 0; attempt < 5; attempt++) {
      await gpuiKey(driver, "up");
      a8Second = await waitUntil(
        driver,
        (s) => s.inputValue === "older query",
        600,
      );
      if (a8Second.inputValue === "older query") break;
    }
    record("a8-up-again-continues-history", a8Second.inputValue === "older query", {
      inputValue: a8Second.inputValue,
    });
    const a8Shot = await shot(driver, "a8-history-recall.png");
    await driver.setFilterAndWait("");

    // === A5: multi-line Cmd+V routes to Agent Chat ==========================
    await Bun.$`printf 'line one\nline two\nline three\n' | pbcopy`;
    await gpuiKey(driver, "v", ["cmd"]);
    const a5State = await waitUntil(driver, (s) => s.promptType !== SCRIPT_LIST);
    const pastedIntoFilter =
      typeof a5State.inputValue === "string" && a5State.inputValue.includes("line one");
    record(
      "a5-multiline-paste-routes-to-agent-chat",
      a5State.promptType !== SCRIPT_LIST && !pastedIntoFilter,
      { promptType: a5State.promptType, inputValue: a5State.inputValue },
    );
    const a5Shot = await shot(driver, "a5-agent-handoff.png");

    // --- receipt -------------------------------------------------------------
    const pass = results.every((r) => r.pass);
    console.log(
      JSON.stringify(
        {
          probe: "input-ambiguity",
          binary: BINARY,
          sessionDir: driver.sessionDir,
          outDir: OUT_DIR,
          screenshots: { a1Shot, a2Shot, a8Shot, a5Shot },
          pass,
          results,
        },
        null,
        2,
      ),
    );
    process.exitCode = pass ? 0 : 1;
  } finally {
    try {
      if (savedClipboard.length > 0) {
        const p = Bun.spawn(["pbcopy"], { stdin: "pipe" });
        p.stdin.write(savedClipboard);
        p.stdin.end();
        await p.exited;
      }
    } catch {
      // clipboard restore is best-effort
    }
    await driver.close();
  }
}

main();
