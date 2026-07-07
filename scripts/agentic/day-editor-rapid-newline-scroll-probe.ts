#!/usr/bin/env bun
// Runtime proof for deterministic editor auto-scroll during rapid multi-line
// edits (Enter/Backspace bursts) in the Day Page editor.
//
// Guards the fixes for:
// - vendored input cursor tracking moving one line per changed frame (drift)
// - content-dependent top/bottom margin flipping while typing
// - forced bottom-scroll retries yanking the viewport after user edits
//
// Red before the fix: after a burst of Enters the viewport lags several lines
// behind the cursor (not at bottom), and a Backspace burst leaves inconsistent
// scroll positions run to run.
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const binary =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/day-editor-rapid-newline/script-kit-gpui";

const receipt: Record<string, Json> = {};
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json = {}) {
  receipt[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function walk(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const child of node) walk(child, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") out.push(json);
  for (const value of Object.values(json)) walk(value, out);
  return out;
}

function findSemantic(elements: Json, semanticId: string): Json | null {
  return walk(elements).find((el) => el.semanticId === semanticId) ?? null;
}

function scrollMetrics(editor: Json | null): Json | null {
  const runtime = editor?.style?.editorRuntime;
  const metrics =
    runtime && typeof runtime === "object" ? (runtime as Json).editorScrollMetrics : null;
  return metrics && typeof metrics === "object" ? (metrics as Json) : null;
}

function isAtBottom(metrics: Json | null): boolean {
  const scrollTop = Number(metrics?.scrollTop ?? -1);
  const liveScrollTop = Number(metrics?.liveScrollTop ?? scrollTop);
  const maxScrollTop = Number(metrics?.maxScrollTop ?? -1);
  return maxScrollTop >= 0 && Math.max(scrollTop, liveScrollTop) >= maxScrollTop - 6;
}

async function mainElements(driver: Driver): Promise<Json> {
  return (await driver.getElements(
    { target: { type: "main" }, limit: 160 },
    { timeoutMs: 5000 },
  )) as Json;
}

async function editorMetrics(driver: Driver): Promise<Json | null> {
  const elements = await mainElements(driver);
  return scrollMetrics(findSemantic(elements, "input:day-page-editor"));
}

async function gpuiKey(driver: Driver, key: string, modifiers: string[] = []): Promise<Json> {
  return driver.request(
    {
      type: "simulateGpuiEvent",
      event: { type: "keyDown", key, modifiers },
      target: { type: "main" },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  );
}

// Rapid burst: no settle between keys, so multiple edits land between frames.
async function burst(driver: Driver, key: string, count: number) {
  for (let i = 0; i < count; i += 1) {
    await gpuiKey(driver, key);
  }
}

function longDayText(): string {
  const lines = ["# Rapid newline probe", ""];
  for (let i = 1; i <= 120; i += 1) {
    lines.push(`- probe line ${i.toString().padStart(3, "0")} overflows the Day editor`);
  }
  lines.push("", "final probe line");
  return `${lines.join("\n")}\n`;
}

function todayLocalDate(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

const driver = await Driver.launch({
  binary,
  sandboxHome: true,
  sessionName: "day-editor-rapid-newline",
  readyTimeoutMs: 30000,
  defaultTimeoutMs: 9000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_BRAIN_TZ: "America/Denver",
  },
});

try {
  const daysFile = `${driver.sessionDir}/home/.scriptkit/brain/days/${todayLocalDate()}.md`;
  mkdirSync(dirname(daysFile), { recursive: true });
  writeFileSync(daysFile, longDayText());

  const openedState = await openDayPage(driver, "rapid-newline");
  check("opened_day_page", openedState.promptType === "dayPage", {
    promptType: openedState.promptType ?? null,
  });
  await Bun.sleep(900); // let load-time bottom-scroll retries settle
  let elements = await mainElements(driver);
  const editor = findSemantic(elements, "input:day-page-editor");
  check("opened_day_page_focused_editor", elements.focusedSemanticId === "input:day-page-editor", {
    focusedSemanticId: elements.focusedSemanticId ?? null,
  });
  const openedMetrics = scrollMetrics(editor);
  check("opened_at_bottom", isAtBottom(openedMetrics), { metrics: openedMetrics });
  const baselineScrollHeight = Number(openedMetrics?.scrollHeight ?? 0);

  // 1. Rapid Enter burst at the end: viewport must track the cursor to the
  //    bottom in the same settle window, not lag one line per keystroke.
  await burst(driver, "enter", 25);
  await Bun.sleep(400);
  const afterEnters = await editorMetrics(driver);
  check("rapid_enters_grow_content", Number(afterEnters?.scrollHeight ?? 0) > baselineScrollHeight, {
    baselineScrollHeight,
    scrollHeight: afterEnters?.scrollHeight ?? null,
  });
  check("rapid_enters_track_cursor_to_bottom", isAtBottom(afterEnters), { metrics: afterEnters });

  // 2. Rapid Backspace burst: content shrinks back; the viewport must clamp
  //    to the new bottom without overscroll and stay on the cursor.
  await burst(driver, "backspace", 25);
  await Bun.sleep(400);
  const afterBackspaces = await editorMetrics(driver);
  const shrunkHeight = Number(afterBackspaces?.scrollHeight ?? -1);
  check("rapid_backspaces_shrink_content", Math.abs(shrunkHeight - baselineScrollHeight) <= 2, {
    baselineScrollHeight,
    scrollHeight: shrunkHeight,
  });
  check("rapid_backspaces_stay_at_bottom", isAtBottom(afterBackspaces), {
    metrics: afterBackspaces,
  });
  const overscroll =
    Number(afterBackspaces?.liveScrollTop ?? 0) - Number(afterBackspaces?.maxScrollTop ?? 0);
  check("rapid_backspaces_do_not_overscroll", overscroll <= 1, { overscroll });

  // 3. Mid-document edits: with the cursor visible mid-viewport, newline
  //    bursts must not jump the viewport (deterministic margin, no yank).
  await burst(driver, "pageup", 3);
  await Bun.sleep(300);
  const midBefore = await editorMetrics(driver);
  check("pageup_left_bottom", !isAtBottom(midBefore), { metrics: midBefore });
  await burst(driver, "enter", 5);
  await Bun.sleep(400);
  const midAfterEnters = await editorMetrics(driver);
  const clientHeight = Number(midBefore?.clientHeight ?? 0);
  const midDelta = Math.abs(
    Number(midAfterEnters?.scrollTop ?? 0) - Number(midBefore?.scrollTop ?? 0),
  );
  check("mid_document_enters_keep_viewport_stable", midDelta <= clientHeight / 4, {
    before: midBefore?.scrollTop ?? null,
    after: midAfterEnters?.scrollTop ?? null,
    midDelta,
    clientHeight,
  });
  await burst(driver, "backspace", 5);
  await Bun.sleep(400);
  const midAfterBackspaces = await editorMetrics(driver);
  const midDeltaBack = Math.abs(
    Number(midAfterBackspaces?.scrollTop ?? 0) - Number(midBefore?.scrollTop ?? 0),
  );
  check("mid_document_backspaces_keep_viewport_stable", midDeltaBack <= clientHeight / 4, {
    before: midBefore?.scrollTop ?? null,
    after: midAfterBackspaces?.scrollTop ?? null,
    midDeltaBack,
    clientHeight,
  });

  // 4. Repeatability: a second Enter+Backspace round must land on the same
  //    scroll position as the first (the "inconsistent" complaint).
  await gpuiKey(driver, "down", ["cmd"]); // MoveToEnd
  await Bun.sleep(300);
  await burst(driver, "enter", 10);
  await Bun.sleep(400);
  const roundOne = await editorMetrics(driver);
  await burst(driver, "backspace", 10);
  await Bun.sleep(400);
  await burst(driver, "enter", 10);
  await Bun.sleep(400);
  const roundTwo = await editorMetrics(driver);
  const roundDelta = Math.abs(
    Number(roundTwo?.scrollTop ?? 0) - Number(roundOne?.scrollTop ?? 0),
  );
  check("repeat_rounds_land_identically", roundDelta <= 2, {
    roundOne: roundOne?.scrollTop ?? null,
    roundTwo: roundTwo?.scrollTop ?? null,
    roundDelta,
  });
  check("repeat_rounds_end_at_bottom", isAtBottom(roundTwo), { metrics: roundTwo });
} finally {
  await driver.close();
}

const output = {
  pass: failures.length === 0,
  failures,
  binary,
  sessionDir: driver.sessionDir,
  appLog: driver.logPath,
  receipt,
};

console.log(JSON.stringify(output, null, 2));
if (failures.length) process.exit(1);
