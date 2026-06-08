#!/usr/bin/env bun
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { dirname, join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "menu-syntax-agentic-testing-scenarios");
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "50"));
const outDir = join(repoRoot, ".test-output/menu-syntax-agentic-testing-scenarios");
const receiptPath = join(outDir, "receipt.json");
const screenshotRel = ".test-screenshots/menu-syntax-help-context-aware-has-sh.png";
const screenshotPath = join(repoRoot, screenshotRel);
const baselineCopy = join(outDir, "baseline/menu-syntax-help-context-aware-has-sh.png");
const helpFooterText = ["Open Menu Syntax", " help"].join("");
const TYPE_FILTER_VALUES = [
  "script",
  "scriptlet",
  "skill",
  "agent",
  "builtin",
  "app",
  "window",
  "file",
  "note",
  "clipboard",
  "clipboard-history",
  "dictation",
  "dictation-history",
  "browser-tab",
  "browser-history",
  "agent_chat-history",
  "fallback",
  "issue",
];

process.env.HOME = join(outDir, "home");
process.env.SK_PATH = join(process.env.HOME, ".scriptkit");
process.env.SCRIPT_KIT_SESSION_DIR = join(outDir, "sessions");
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";

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
  let parsed: Json | null = null;
  for (const line of stdout.split(/\r?\n/).reverse()) {
    const candidate = line.trim();
    if (!candidate.startsWith("{")) continue;
    try {
      parsed = JSON.parse(candidate);
      break;
    } catch {
      // session.sh can emit diagnostic lines before the JSON payload; keep
      // scanning backward for the response envelope.
    }
  }
  if (!parsed) {
    throw new Error(`session.sh ${args.join(" ")} did not emit parseable JSON\nstdout=${stdout}`);
  }
  if (parsed.status === "error") {
    throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}`);
  }
  return parsed;
}

function cleanupIsolatedProcesses() {
  run("pkill", ["-f", outDir], { allowFailure: true });
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

function waitForInput(input: string, requestId: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId,
      condition: { type: "stateMatch", state: { promptType: "none", inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function waitForWindowVisible(visible: boolean, requestId: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId,
      condition: { type: "stateMatch", state: { windowVisible: visible } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function showWindow(requestId: string): Json {
  return rpc({ type: "show", requestId }, "windowVisibilityAck");
}

function getState(tag: string): Json {
  return rpc({ type: "getState", requestId: `ms-${tag}-state-${Date.now()}` }, "stateResult");
}

function listWindows(tag: string): Json {
  return rpc({ type: "listAutomationWindows", requestId: `ms-${tag}-windows-${Date.now()}` }, "automationWindowListResult");
}

function getPopupElements(tag: string): Json {
  return rpc(
    {
      type: "getElements",
      requestId: `ms-${tag}-popup-elements-${Date.now()}`,
      target: { type: "id", id: "menu-syntax-trigger-popup" },
      limit: 100,
    },
    "elementsResult",
  );
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function assertEq(actual: unknown, expected: unknown, message: string) {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function assertArrayIncludes(values: string[], expected: string, label: string) {
  assert(values.includes(expected), `${label}: missing ${expected}; got ${JSON.stringify(values)}`);
}

function assertNoText(value: unknown, forbidden: string[], label: string) {
  const text = JSON.stringify(value);
  for (const needle of forbidden) {
    assert(!text.includes(needle), `${label}: leaked forbidden text ${needle}`);
  }
}

function popupOpen(windows: Json): boolean {
  return (windows.windows ?? []).some((w: Json) => w.id === "menu-syntax-trigger-popup" && w.visible === true);
}

function waitForPopupOpen(tag: string): Json {
  const deadline = Date.now() + timeoutMs;
  let last: Json | null = null;
  while (Date.now() < deadline) {
    last = listWindows(`${tag}-popup-poll`);
    if (popupOpen(last)) return last;
    Bun.sleepSync(pollMs);
  }
  throw new Error(`${tag}: menu syntax popup did not open; last windows=${JSON.stringify(last?.windows ?? [])}`);
}

function popupRows(elements: Json): Array<{ text: string; value: string; role: string; kind: string; sourceName: string }> {
  const warnings = elements.warnings ?? [];
  assert(!warnings.some((w: string) => String(w).startsWith("panel_only_")), `popup elements degraded: ${warnings.join(", ")}`);
  return (elements.elements ?? [])
    .filter((el: Json) => el.type === "choice")
    .map((el: Json) => ({
      text: el.text ?? "",
      value: el.value ?? "",
      role: el.role ?? "",
      kind: el.kind ?? "",
      sourceName: el.sourceName ?? "",
    }));
}

function rowHaystack(rows: Array<Record<string, string>>): string {
  return rows.map((row) => Object.values(row).join("\n")).join("\n");
}

function assertPopup(input: string, tag: string, spec: Json): Json {
  let shownForPopup = false;
  let windows: Json;
  try {
    windows = waitForPopupOpen(tag);
  } catch {
    shownForPopup = true;
    showWindow(`ms-${tag}-popup-show`);
    send({ type: "setFilter", text: input, requestId: `ms-${tag}-popup-reset` });
    waitForInput(input, `ms-${tag}-popup-reset-wait`);
    windows = waitForPopupOpen(tag);
  }
  const elements = getPopupElements(tag);
  const rows = popupRows(elements);
  const haystack = rowHaystack(rows);
  for (const token of spec.mustContainTokens ?? []) {
    assert(haystack.includes(token), `${input}: popup rows missing ${token}; rows=${JSON.stringify(rows)}`);
  }
  if (spec.preferredTitle) {
    assert(haystack.includes(spec.preferredTitle), `${input}: popup rows missing preferred title ${spec.preferredTitle}`);
  }
  assert(!haystack.includes(helpFooterText), `${input}: popup rows contained removed help footer`);
  for (const text of spec.mustNotContainText ?? []) {
    assert(!haystack.includes(text), `${input}: popup rows contained ${text}`);
  }
  if (shownForPopup) {
    send({ type: "hide", requestId: `ms-${tag}-popup-hide` });
    waitForWindowVisible(false, `ms-${tag}-popup-hide-wait`);
  }
  return { windows, elements, rows };
}

function assertHint(input: string, state: Json, expected: Json, checks: (hint: Json) => void): Json {
  assertEq(state.inputValue, input, `${input}: inputValue`);
  const hint = state.menuSyntaxMainHint;
  assert(hint, `${input}: missing menuSyntaxMainHint`);
  for (const [key, value] of Object.entries(expected)) {
    assertEq(hint[key], value, `${input}: hint.${key}`);
  }
  checks(hint);
  return hint;
}

function driveScenario(spec: Json): Json {
  send({ type: "setFilter", text: spec.input, requestId: `ms-${spec.tag}-set` });
  waitForInput(spec.input, `ms-${spec.tag}-wait`);
  const state = getState(spec.tag);
  const hint = assertHint(spec.input, state, spec.expected, spec.check);
  const popup = spec.popup ? assertPopup(spec.input, spec.tag, spec.popup) : null;
  return {
    scenario: spec.tag,
    input: spec.input,
    protocol: ["setFilter", "waitFor", "getState"],
    stateResult: { inputValue: state.inputValue, menuSyntaxMainHint: hint },
    popup,
  };
}

function menuSyntaxHintBodyText(hint: Json): string {
  return [
    hint?.title,
    hint?.subtitle,
    hint?.primaryHint,
    hint?.secondaryHint,
    hint?.example,
    ...(hint?.examples ?? []),
    ...(hint?.rows ?? []).flatMap((row: Json) => [
      row?.label,
      row?.value,
      ...(row?.chips ?? []).map((chip: Json) => chip?.label),
    ]),
    ...(hint?.fragmentPreview?.rows ?? []).flatMap((row: Json) => [
      row?.label,
      row?.value,
      row?.source,
    ]),
  ]
    .filter((part) => typeof part === "string" && part.trim().length > 0)
    .join("\n");
}

function assertAdvancedQueryGuideHintBody(raw: string, tag: string): Json {
  send({ type: "setFilter", text: raw, requestId: `ms-${tag}-set` });
  waitForInput(raw, `ms-${tag}-wait`);
  const state = getState(tag);
  const hint = state.menuSyntaxMainHint;
  assert(hint, `${raw}: expected menuSyntaxMainHint`);
  assertEq(hint.kind, "AdvancedQueryGuide", `${raw}: menuSyntaxMainHint.kind`);
  assert(
    Array.isArray(hint.rows) && hint.rows.length > 0,
    `${raw}: expected non-empty MenuSyntaxMainHintSnapshot.rows, got ${JSON.stringify(hint)}`,
  );
  const body = menuSyntaxHintBodyText(hint);
  assert(body.trim().length > 0, `${raw}: expected non-empty MenuSyntaxMainHintSnapshot body`);
  return { input: raw, kind: hint.kind, rows: hint.rows, body };
}

function runTypeFilterAppliedHintScenario(): Json {
  showWindow("ms-type-filter-hints-show");
  const receipts: Json[] = [];
  for (const value of TYPE_FILTER_VALUES) {
    receipts.push(assertAdvancedQueryGuideHintBody(`:type:${value} review`, `type-filter-${value}`));
  }
  for (const value of TYPE_FILTER_VALUES) {
    receipts.push(assertAdvancedQueryGuideHintBody(`:kind:${value} review`, `kind-filter-${value}`));
  }
  receipts.push(assertAdvancedQueryGuideHintBody(":-type:app review", "negated-type-app"));
  return {
    scenario: "type-filter-applied-hint-body-non-empty",
    protocol: ["show", "setFilter", "waitFor", "getState"],
    values: TYPE_FILTER_VALUES,
    receipts,
  };
}

function captureScreenshot(): Json {
  mkdirSync(dirname(baselineCopy), { recursive: true });
  assert(existsSync(screenshotPath), `missing screenshot baseline ${screenshotRel}`);
  writeFileSync(baselineCopy, readFileSync(screenshotPath));

  run("bash", [sessionScript, "stop", session], { allowFailure: true });
  runSession(["start", session]);
  showWindow("ms-screenshot-show");
  Bun.sleepSync(750);
  send({ type: "setFilter", text: "has:sh", requestId: "ms-screenshot-has-sh-set" });
  waitForInput("has:sh", "ms-screenshot-has-sh-wait");
  const state = getState("screenshot-has-sh");
  const hint = assertHint(
    "has:sh",
    state,
    {
      kind: "AdvancedQueryEmpty",
      rawFilterText: "has:sh",
      activeHead: "has:",
      activeHeadValuePartial: "sh",
    },
    (h) => assertEq(JSON.stringify(h.examples ?? []), JSON.stringify(["has:shortcut"]), "has:sh screenshot examples"),
  );

  const hintText = JSON.stringify(hint);
  assert(hintText.includes("has:shortcut"), "pre-capture hint receipt missing has:shortcut");
  assertNoText(hint, [":#work", ":tag:work", ":type:script deploy", helpFooterText], "pre-capture hint receipt");

  send({ type: "setFilter", text: "has:sh", requestId: "ms-screenshot-has-sh-selection-reset" });
  waitForInput("has:sh", "ms-screenshot-has-sh-selection-reset-wait");
  showWindow("ms-screenshot-final-show");
  send({ type: "setFilter", text: "has:sh", requestId: "ms-screenshot-final-has-sh-set" });
  waitForInput("has:sh", "ms-screenshot-final-has-sh-wait");
  waitForPopupOpen("screenshot-has-sh");
  send({ type: "simulateKey", key: "Escape", requestId: "ms-screenshot-close-popup" });
  waitForInput("has:sh", "ms-screenshot-close-popup-wait");
  assert(!popupOpen(listWindows("screenshot-after-popup-close")), "screenshot setup did not close trigger popup");
  Bun.sleepSync(1500);
  const before = readFileSync(baselineCopy);
  const captures: Json[] = [];
  let matched = false;
  for (let attempt = 1; attempt <= 10; attempt += 1) {
    const capture = send({
      type: "captureWindow",
      title: "",
      path: screenshotPath,
      requestId: `ms-screenshot-capture-${attempt}`,
    });
    captures.push(capture);
    Bun.sleepSync(250);
    assert(existsSync(screenshotPath), `capture did not write ${screenshotRel}`);
    const after = readFileSync(screenshotPath);
    if (before.equals(after)) {
      matched = true;
      break;
    }
  }
  assert(matched, `screenshot drifted from existing baseline: ${screenshotRel}`);

  return {
    path: screenshotRel,
    baselineCopy: baselineCopy.replace(`${repoRoot}/`, ""),
    baselineDiff: "pass",
    contains: ["has:shortcut"],
    notContains: [":#work", ":tag:work", ":type:script deploy"],
    targetCheck: "menuSyntaxMainHint pre-capture receipt",
    captures,
  };
}

async function main() {
  cleanupIsolatedProcesses();
  rmSync(outDir, { recursive: true, force: true });
  mkdirSync(process.env.SK_PATH!, { recursive: true });
  mkdirSync(process.env.SCRIPT_KIT_SESSION_DIR!, { recursive: true });

  const receipt: Json = {
    schemaVersion: 1,
    status: "running",
    session,
    scenarios: [],
    escapeProof: {},
    screenshot: {
      path: screenshotRel,
      baselineDiff: "not-run",
      contains: ["has:shortcut"],
      notContains: [":#work", ":tag:work", ":type:script deploy"],
    },
    negativeGreps: {},
  };

  try {
    run("bash", [sessionScript, "stop", session], { allowFailure: true });
    receipt.start = runSession(["start", session]);

    const scenarios = [
      {
        tag: "has-bare",
        input: "has:",
        expected: {
          kind: "AdvancedQueryEmpty",
          rawFilterText: "has:",
          activeHead: "has:",
          title: "Filter by metadata field",
          primaryHint: "Choose a field from the list or finish typing a metadata key.",
        },
        check: (hint: Json) => {
          assert(!hint.activeHeadValuePartial, "has: activeHeadValuePartial should be absent or empty");
          for (const example of hint.examples ?? []) assert(String(example).startsWith("has:"), `has: non-has example ${example}`);
          for (const example of ["has:shortcut", "has:alias", "has:menuSyntax"]) assertArrayIncludes(hint.examples ?? [], example, "has: examples");
          assertNoText(hint.examples, [":#work", ":tag:work", ":type:script deploy"], "has: examples");
        },
        popup: {
          mustContainTokens: ["has:shortcut", "has:alias", "has:menuSyntax"],
          mustNotContainText: [],
        },
      },
      {
        tag: "has-sh",
        input: "has:sh",
        expected: {
          kind: "AdvancedQueryEmpty",
          rawFilterText: "has:sh",
          activeHead: "has:",
          activeHeadValuePartial: "sh",
          title: "Filter by metadata field",
          primaryHint: "Choose a field from the list or finish typing a metadata key.",
        },
        check: (hint: Json) => {
          assertEq(JSON.stringify(hint.examples ?? []), JSON.stringify(["has:shortcut"]), "has:sh examples");
          assert((hint.rows ?? []).some((r: Json) => r.label === "has:shortcut"), "has:sh missing has:shortcut row");
          assertNoText(hint, ["#work", "tag:work", "type:script deploy"], "has:sh hint");
        },
        popup: {
          mustContainTokens: ["has:shortcut"],
          mustNotContainText: [":#work", ":tag:work", ":type:script deploy"],
        },
      },
      {
        tag: "c-zzz",
        input: "c:zzz",
        expected: {
          kind: "AdvancedQueryEmpty",
          rawFilterText: "c:zzz",
          activeHead: "c:",
          activeHeadValuePartial: "zzz",
          title: "No clipboard entries match `zzz`.",
          primaryHint: "Press Esc to clear the filter.",
        },
        check: (hint: Json) => {
          assertEq(JSON.stringify(hint.examples ?? []), JSON.stringify(["c: order id"]), "c:zzz examples");
          assertNoText(hint.examples, [":#work", ":tag:work", ":type:script deploy"], "c:zzz examples");
        },
      },
      {
        tag: "type-scriptlet",
        input: ":type:scriptlet zzz",
        expected: {
          kind: "AdvancedQueryGuide",
          rawFilterText: ":type:scriptlet zzz",
          activeHead: ":type:",
          activeHeadValuePartial: "scriptlet",
          title: "Filtering to scriptlets",
          primaryHint: "Keep typing to narrow results, or remove a filter to widen.",
        },
        check: (hint: Json) => {
          assert((hint.rows ?? []).length > 0, "type:scriptlet applied hint rows must be non-empty");
          assert((hint.rows ?? []).some((r: Json) => r.label === "Filters"), "type:scriptlet missing Filters row");
          assert((hint.rows ?? []).some((r: Json) => r.label === "Search words" && r.value === "zzz"), "type:scriptlet missing search words row");
          assertNoText(hint.examples, [":#work", ":tag:work"], "type:scriptlet examples");
        },
      },
      {
        tag: "daily",
        input: ";daily",
        expected: {
          kind: "CapturePickerCompanion",
          rawFilterText: ";daily",
          title: "Daily note",
        },
        check: (hint: Json) => {
          assertEq(hint.modeChip?.label, "; capture", ";daily mode chip");
          assert((hint.rows ?? []).some((r: Json) => r.label === "Selected" && r.value === ";note"), ";daily selected row");
          assert((hint.rows ?? []).some((r: Json) => r.label === "Target" && String(r.value).includes("today")), ";daily target row");
          assert((hint.examples ?? []).every((e: string) => e.startsWith(";note ")), ";daily examples must be ;note examples");
        },
        popup: {
          mustContainTokens: [";note"],
          preferredTitle: "Daily note",
          mustNotContainText: [],
        },
      },
      {
        tag: "colon",
        input: ":",
        expected: {
          kind: "AdvancedQueryGuide",
          rawFilterText: ":",
          title: "Refine launcher search",
          subtitle: "Use `:` to add filters, then type the words you want to match.",
        },
        check: (hint: Json) => {
          assertEq(hint.modeChip?.label, ": refine", "colon mode chip");
          assertEq(hint.statusChip?.label, "guide", "colon status chip");
          for (const example of [":type:script deploy", ":#work type:script", ":-type:app triage", ":shortcut:any"]) {
            assertArrayIncludes(hint.examples ?? [], example, "colon examples");
          }
        },
        popup: {
          mustContainTokens: ["type:script", "type:scriptlet", "shortcut:any", "tag:", "has:"],
          mustNotContainText: [],
        },
      },
    ];

    receipt.scenarios = scenarios.map(driveScenario);
    receipt.scenarios.push(runTypeFilterAppliedHintScenario());

    showWindow("ms-escape-show");
    send({ type: "setFilter", text: "has:sh", requestId: "ms-escape-setup-set" });
    waitForInput("has:sh", "ms-escape-setup-wait");
    assert(popupOpen(listWindows("escape-setup")), "escape setup did not open popup");
    send({ type: "simulateKey", key: "Escape", requestId: "ms-escape-popup-close" });
    waitForInput("has:sh", "ms-escape-popup-close-wait");
    const afterPopupCloseState = getState("escape-popup-close");
    const afterPopupCloseWindows = listWindows("escape-popup-close");
    assert(!popupOpen(afterPopupCloseWindows), "first Escape did not close popup");
    assertEq(afterPopupCloseState.menuSyntaxMainHint?.activeHead, "has:", "escape step 1 activeHead");
    assertEq(afterPopupCloseState.menuSyntaxMainHint?.activeHeadValuePartial, "sh", "escape step 1 activeHeadValuePartial");

    send({ type: "simulateKey", key: "Escape", requestId: "ms-escape-clear-filter" });
    waitForInput("", "ms-escape-clear-filter-wait");
    waitForWindowVisible(true, "ms-escape-clear-filter-visible-wait");

    send({ type: "simulateKey", key: "Escape", requestId: "ms-escape-hide-window" });
    waitForWindowVisible(false, "ms-escape-hide-window-wait");
    receipt.escapeProof = {
      popupClose: { inputValue: afterPopupCloseState.inputValue, popupOpen: false },
      clearFilter: { inputValue: "", windowVisible: true },
      hideWindow: { windowVisible: false },
    };

    receipt.screenshot = captureScreenshot();

    const outText = existsSync(outDir)
      ? run(
          "rg",
          ["--fixed-strings", "--text", helpFooterText, outDir, "--glob", "!home/**"],
          { allowFailure: true },
        )
      : "";
    assert(outText.trim() === "", "removed help footer text appeared in proof output");
    const sourceText = run("git", ["grep", "-n", helpFooterText, "--", "src/menu_syntax", "src/app_impl", "src/render_script_list", "scripts/agentic"], { allowFailure: true });
    assert(sourceText.trim() === "", "removed help footer text appeared in guarded source paths");
    receipt.negativeGreps = {
      openMenuSyntaxHelpInHarnessLogs: "pass",
      openMenuSyntaxHelpInGuardedSource: "pass",
    };

    receipt.status = "pass";
    writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  } catch (error) {
    receipt.status = "fail";
    receipt.error = error instanceof Error ? error.message : String(error);
    mkdirSync(outDir, { recursive: true });
    writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
    throw error;
  } finally {
    run("bash", [sessionScript, "stop", session], { allowFailure: true });
    cleanupIsolatedProcesses();
  }
}

await main();
