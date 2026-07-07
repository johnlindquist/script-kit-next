#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const binary =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/day-editor-bottom-scroll/script-kit-gpui";

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

function editorRuntime(editor: Json | null): Json | null {
  const runtime = editor?.style?.editorRuntime;
  return runtime && typeof runtime === "object" ? (runtime as Json) : null;
}

function scrollMetrics(editor: Json | null): Json | null {
  const metrics = editorRuntime(editor)?.editorScrollMetrics;
  return metrics && typeof metrics === "object" ? (metrics as Json) : null;
}

function isAtBottom(metrics: Json | null): boolean {
  const scrollTop = Number(metrics?.scrollTop ?? -1);
  const liveScrollTop = Number(metrics?.liveScrollTop ?? scrollTop);
  const maxScrollTop = Number(metrics?.maxScrollTop ?? -1);
  return maxScrollTop >= 0 && Math.max(scrollTop, liveScrollTop) >= maxScrollTop - 6;
}

function num(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function automationWindows(windows: Json): Json[] {
  const list = windows.windows;
  return Array.isArray(list) ? (list as Json[]) : [];
}

function boundsOf(windowInfo: Json | null | undefined): Json | null {
  const bounds = windowInfo?.bounds;
  return bounds && typeof bounds === "object" ? (bounds as Json) : null;
}

function footerClearanceProof(metrics: Json | null, windows: Json): Json {
  const minimumMainFooterClearance = 40;
  const list = automationWindows(windows);
  const main = list.find((win) => win.id === "main") ?? null;
  const footer = list.find((win) => win.id === "footer-overlay") ?? null;
  const mainBounds = boundsOf(main);
  const footerBounds = boundsOf(footer);
  const mainHeight = num(mainBounds?.height);
  const footerHeight = num(footerBounds?.height);
  const clientHeight = num(metrics?.clientHeight);
  const reservedOutsideEditor =
    mainHeight != null && clientHeight != null ? mainHeight - clientHeight : null;
  const requiredFooterHeight = footerHeight ?? minimumMainFooterClearance;
  const ok =
    reservedOutsideEditor != null &&
    reservedOutsideEditor >= requiredFooterHeight;
  return {
    ok,
    mainBounds,
    footerBounds,
    clientHeight,
    reservedOutsideEditor,
    requiredFooterHeight,
    measurementMode: footerHeight == null ? "main-window-minus-editor-client-height" : "footer-window-bounds",
    windowIds: list.map((win) => win.id),
  };
}

async function mainElements(driver: Driver): Promise<Json> {
  return (await driver.getElements(
    { target: { type: "main" }, limit: 160 },
    { timeoutMs: 5000 },
  )) as Json;
}

// Open via the hotkey hold gesture (shared helper). The old "setInput ','
// then Enter" opener silently launched whatever script matched "," in the
// sandbox (e.g. the todo example via alias fuzzy match) — the day page never
// opened and waitForState's failed result was swallowed.
async function openDayWithComma(driver: Driver) {
  await openDayPage(driver, "day-editor-bottom-focus");
  await Bun.sleep(600);
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

function longDayText(): string {
  const lines = ["# Bottom focus probe", ""];
  for (let i = 1; i <= 140; i += 1) {
    lines.push(`- probe line ${i.toString().padStart(3, "0")} should overflow the Day editor`);
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
  sessionName: "day-editor-bottom-focus",
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

  await openDayWithComma(driver);
  let elements = await mainElements(driver);
  let editor = findSemantic(elements, "input:day-page-editor");
  let metrics = scrollMetrics(editor);
  check("opened_day_page_focused_editor", elements.focusedSemanticId === "input:day-page-editor", {
    focusedSemanticId: elements.focusedSemanticId ?? null,
  });
  check("opened_day_page_loaded_long_content", String(editor?.value ?? "").includes("final probe line"), {
    valueLength: typeof editor?.value === "string" ? editor.value.length : null,
  });
  check("opened_day_page_scrolls_to_bottom", isAtBottom(metrics), { metrics });
  const openedWindows = (await driver.listAutomationWindows({ timeoutMs: 5000 })) as Json;
  const openedClearance = footerClearanceProof(metrics, openedWindows);
  check("opened_day_page_reserves_footer_clearance", openedClearance.ok === true, openedClearance);
  await Bun.sleep(1500);
  elements = await mainElements(driver);
  const settledOpenMetrics = scrollMetrics(findSemantic(elements, "input:day-page-editor"));
  check("opened_day_page_stays_at_bottom_after_settle", isAtBottom(settledOpenMetrics), {
    metrics: settledOpenMetrics,
  });

  await gpuiKey(driver, "pageup");
  await Bun.sleep(300);
  elements = await mainElements(driver);
  const pageUpMetrics = scrollMetrics(findSemantic(elements, "input:day-page-editor"));
  check("pageup_moves_away_from_bottom", !isAtBottom(pageUpMetrics), { metrics: pageUpMetrics });

  driver.simulateKey("k", ["cmd"]);
  await Bun.sleep(600);
  driver.simulateKey("escape");
  await driver.waitForState({ promptType: "dayPage" }, { timeoutMs: 5000 });
  await Bun.sleep(700);
  elements = await mainElements(driver);
  editor = findSemantic(elements, "input:day-page-editor");
  metrics = scrollMetrics(editor);
  check("actions_escape_refocuses_day_editor", elements.focusedSemanticId === "input:day-page-editor", {
    focusedSemanticId: elements.focusedSemanticId ?? null,
  });
  check("actions_escape_scrolls_back_to_bottom", isAtBottom(metrics), { metrics });
  const restoredWindows = (await driver.listAutomationWindows({ timeoutMs: 5000 })) as Json;
  const restoredClearance = footerClearanceProof(metrics, restoredWindows);
  check("actions_escape_preserves_footer_clearance", restoredClearance.ok === true, restoredClearance);
  await Bun.sleep(1500);
  elements = await mainElements(driver);
  const settledActionsMetrics = scrollMetrics(findSemantic(elements, "input:day-page-editor"));
  check("actions_escape_stays_at_bottom_after_settle", isAtBottom(settledActionsMetrics), {
    metrics: settledActionsMetrics,
  });

  await gpuiKey(driver, "pageup");
  await Bun.sleep(250);
  driver.simulateKey("escape");
  await Bun.sleep(500);
  await openDayWithComma(driver);
  elements = await mainElements(driver);
  editor = findSemantic(elements, "input:day-page-editor");
  metrics = scrollMetrics(editor);
  check("reopened_day_page_refocuses_editor", elements.focusedSemanticId === "input:day-page-editor", {
    focusedSemanticId: elements.focusedSemanticId ?? null,
  });
  check("reopened_day_page_scrolls_to_bottom", isAtBottom(metrics), { metrics });
  await Bun.sleep(1500);
  elements = await mainElements(driver);
  const settledReopenMetrics = scrollMetrics(findSemantic(elements, "input:day-page-editor"));
  check("reopened_day_page_stays_at_bottom_after_settle", isAtBottom(settledReopenMetrics), {
    metrics: settledReopenMetrics,
  });
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
