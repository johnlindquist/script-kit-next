#!/usr/bin/env bun
/**
 * Runtime proof for Day Page and Notes shared Markdown editor parity.
 *
 * Proves the same user-facing editor runtime is observable from both windows:
 * - shared component/style owner and render path
 * - shared Markdown highlighter registration/query fingerprints
 * - editable Markdown does not inject markdown_inline per inline node
 * - long Markdown is accepted by both editors
 * - editor scroll metrics are exposed and getElements remains fast
 */

import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/day-notes-editor-parity/script-kit-gpui";
const runId = `editor-runtime-${Date.now().toString(36)}`;
const OUT_PATH = ".test-output/day-notes-preview-renderer-parity-probe.json";

const checks: Array<{ name: string; ok: boolean; detail: Json }> = [];
const failures: string[] = [];
const timings: Record<string, number[]> = { notesGetElementsMs: [], dayGetElementsMs: [] };

function check(name: string, ok: boolean, detail: Json = {}) {
  checks.push({ name, ok, detail });
  if (!ok) failures.push(name);
}

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") out.push(json);
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

function findSemantic(elements: Json, semanticId: string): Json | null {
  return walkElements(elements).find((el) => el.semanticId === semanticId) ?? null;
}

function ids(elements: Json): string[] {
  return walkElements(elements)
    .map((el) => String(el.semanticId ?? el.id ?? ""))
    .filter(Boolean);
}

function localOverlayIds(elements: Json): string[] {
  return ids(elements).filter((id) => {
    const lower = id.toLowerCase();
    return (
      lower.includes("day-page-spine") ||
      lower.includes("day-spine") ||
      lower.includes("ready-to-send") ||
      lower.includes("prompt-builder") ||
      lower === "notes-spine-list"
    );
  });
}

function editorRuntime(editor: Json | null): Json | null {
  const runtime = editor?.style?.editorRuntime;
  return runtime && typeof runtime === "object" ? (runtime as Json) : null;
}

function scrollMetrics(runtime: Json | null): Json | null {
  const metrics = runtime?.editorScrollMetrics;
  return metrics && typeof metrics === "object" ? (metrics as Json) : null;
}

function markdownLinkHighlightRanges(runtime: Json | null): Json | null {
  const ranges = runtime?.markdownLinkHighlightRanges;
  return ranges && typeof ranges === "object" ? (ranges as Json) : null;
}

function p95(values: number[]): number {
  if (values.length === 0) return Number.POSITIVE_INFINITY;
  const sorted = [...values].sort((a, b) => a - b);
  return sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * 0.95) - 1)];
}

async function timedGetElements(
  driver: Driver,
  label: "notesGetElementsMs" | "dayGetElementsMs",
  request: Json,
): Promise<Json> {
  const start = performance.now();
  const result = (await driver.getElements(request, { timeoutMs: 8000 })) as Json;
  timings[label].push(Math.round(performance.now() - start));
  return result;
}

async function gpuiKey(
  driver: Driver,
  key: string,
  target?: Json,
  modifiers: string[] = [],
): Promise<Json> {
  const event: Json = { type: "keyDown", key, modifiers };
  const payload: Json = {
    type: "simulateGpuiEvent",
    requestId: `${runId}-${key}-${Math.random().toString(36).slice(2)}`,
    event,
  };
  if (target) payload.target = target;
  return driver.request(payload, { expect: "simulateGpuiEventResult", timeoutMs: 5000 });
}

async function notesState(driver: Driver): Promise<Json> {
  const result = (await driver.request(
    { type: "getState", target: { type: "kind", kind: "notes", index: 0 } },
    { expect: "stateResult", timeoutMs: 5000 },
  )) as Json;
  return (result.notes ?? result) as Json;
}

async function currentState(driver: Driver): Promise<Json> {
  return (await driver.getState({ timeoutMs: 5000 })) as Json;
}

async function waitForDayReadMode(driver: Driver, readMode: boolean): Promise<Json> {
  for (let i = 0; i < 40; i += 1) {
    const state = await currentState(driver);
    if (state.dayPage?.readMode === readMode) return state;
    await Bun.sleep(100);
  }
  return currentState(driver);
}

function isActionsWindow(win: Json): boolean {
  return (
    win.id === "actions-dialog" ||
    win.automationId === "actions-dialog" ||
    win.kind === "ActionsDialog" ||
    win.windowKind === "ActionsDialog" ||
    win.semanticSurface === "actionsDialog"
  );
}

async function actionsWindowRegistered(driver: Driver): Promise<boolean> {
  const windows = (await driver.listAutomationWindows({ timeoutMs: 3000 })) as Json;
  return ((windows.windows ?? []) as Json[]).some(isActionsWindow);
}

async function waitForActionsReady(driver: Driver): Promise<void> {
  for (let i = 0; i < 50; i += 1) {
    const state = (await driver.getState({ timeoutMs: 1000 }).catch(() => null)) as Json | null;
    const registered = await actionsWindowRegistered(driver).catch(() => false);
    if (registered || state?.promptType === "actionsDialog" || state?.actionsDialog?.open === true) {
      return;
    }
    await Bun.sleep(100);
  }
  throw new Error("ActionsDialog did not become automation-ready");
}

async function waitForActionsReopenDebounce(driver: Driver): Promise<void> {
  for (let i = 0; i < 20; i += 1) {
    if (!(await actionsWindowRegistered(driver).catch(() => false))) break;
    await Bun.sleep(50);
  }
  // AppImpl intentionally suppresses immediate Actions reopens to avoid a
  // footer-click close/reopen race. Keep the probe on the product path.
  await Bun.sleep(350);
}

function visibleActions(dialog: Json): Json[] {
  const rows = dialog.visibleActions;
  if (Array.isArray(rows)) return rows as Json[];
  const sample = (dialog.actions as Json | undefined)?.visibleSample;
  return Array.isArray(sample) ? (sample as Json[]) : [];
}

function rowActionId(row: Json): string {
  return String(row.id ?? row.actionId ?? row.value ?? "");
}

async function actionsDialogState(driver: Driver): Promise<Json> {
  if (await actionsWindowRegistered(driver).catch(() => false)) {
    const state = (await driver.request(
      { type: "getState", target: { type: "kind", kind: "actionsDialog" }, summaryOnly: true },
      { expect: "stateResult", timeoutMs: 5000 },
    )) as Json;
    return (state.actionsDialog ?? {}) as Json;
  }
  const state = await currentState(driver);
  return (state.actionsDialog ?? {}) as Json;
}

async function actionsElements(driver: Driver): Promise<Json[]> {
  const target = (await actionsWindowRegistered(driver).catch(() => false))
    ? { type: "kind", kind: "actionsDialog" }
    : { type: "main" };
  const elements = (await driver.getElements({ target, limit: 260 }, { timeoutMs: 5000 })) as Json;
  return walkElements(elements);
}

async function findActionSemanticId(driver: Driver, actionId: string): Promise<string> {
  for (let i = 0; i < 30; i += 1) {
    const dialog = await actionsDialogState(driver).catch(() => null);
    const rows = dialog ? visibleActions(dialog) : [];
    const row = rows.find((candidate) => rowActionId(candidate) === actionId);
    const elements = await actionsElements(driver).catch(() => []);
    const element = elements.find((candidate) =>
      String(candidate.semanticId ?? "").endsWith(`:${actionId}`),
    );
    const semanticId = String(element?.semanticId ?? row?.semanticId ?? "");
    if (semanticId.startsWith("choice:")) return semanticId;
    await Bun.sleep(100);
  }
  return "";
}

async function toggleDayReadModeViaActions(driver: Driver, label: string): Promise<Json> {
  await waitForActionsReopenDebounce(driver);
  const open = (await driver.batch([{ type: "openActions" }], { timeoutMs: 7000 })) as Json;
  check(`open_actions_for_${label.replace(/\s+/g, "_").toLowerCase()}`, open.success === true, {
    batch: open,
  });
  await waitForActionsReady(driver);
  const target = (await actionsWindowRegistered(driver).catch(() => false))
    ? { type: "kind", kind: "actionsDialog" }
    : { type: "main" };
  const filter = (await driver.request(
    {
      type: "batch",
      requestId: `${runId}-filter-${label.replace(/\s+/g, "-")}`,
      target,
      commands: [{ type: "setInput", text: label }],
      options: { stopOnError: true, timeout: 7000 },
    },
    { expect: "batchResult", timeoutMs: 8000 },
  )) as Json;
  check(`filter_actions_for_${label.replace(/\s+/g, "_").toLowerCase()}`, filter.success === true, {
    batch: filter,
  });
  const semanticId = await findActionSemanticId(driver, "day_page:toggle_read_mode");
  check(`find_read_mode_action_${label.replace(/\s+/g, "_").toLowerCase()}`, semanticId.startsWith("choice:"), {
    semanticId,
  });
  const select = (await driver.request(
    {
      type: "batch",
      requestId: `${runId}-select-read-mode-${label.replace(/\s+/g, "-")}`,
      target,
      commands: semanticId.startsWith("choice:")
        ? [{ type: "selectBySemanticId", semanticId }]
        : [{ type: "selectByValue", value: "day_page:toggle_read_mode" }],
      options: { stopOnError: true, timeout: 7000 },
    },
    { expect: "batchResult", timeoutMs: 8000 },
  )) as Json;
  check(`select_read_mode_action_${label.replace(/\s+/g, "_").toLowerCase()}`, select.success === true, {
    batch: select,
  });
  const activateTarget = (await actionsWindowRegistered(driver).catch(() => false))
    ? { type: "kind", kind: "actionsDialog" }
    : { type: "main" };
  const activate = (await driver.request(
    {
      type: "simulateGpuiEvent",
      requestId: `${runId}-activate-read-mode-${label.replace(/\s+/g, "-")}`,
      target: activateTarget,
      event: { type: "keyDown", key: "enter", modifiers: [] },
    },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  )) as Json;
  check(`activate_read_mode_action_${label.replace(/\s+/g, "_").toLowerCase()}`, activate.ok !== false, {
    activate,
  });
  return { filter, select, activate };
}

function buildLongMarkdown(): string {
  const lines = [
    "---",
    `title: Runtime parity ${runId}`,
    "tags:",
    "  - parity",
    "  - markdown",
    "---",
    "",
    `# Runtime parity ${runId}`,
    "",
    "- bullet with [Script Kit](https://scriptkit.com)",
    "- inline `code` should highlight without inline parser injection",
    "",
    "> blockquote for markdown highlighter coverage",
    "",
    "```ts",
    "const surface = 'shared notes editor';",
    "console.log(surface);",
    "```",
    "",
    "```rust",
    "let surface = \"shared notes editor\";",
    "println!(\"{surface}\");",
    "```",
    "",
  ];
  for (let i = 0; i < 150; i += 1) {
    lines.push(
      `${i + 1}. Scroll line ${i + 1} for ${runId} with **bold**, _italic_, and \`code\`.`,
    );
  }
  return `${lines.join("\n")}\n`;
}

const longMarkdown = buildLongMarkdown();
const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-notes-editor-runtime-parity",
  readyTimeoutMs: 60_000,
  defaultTimeoutMs: 9000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_BRAIN_TZ: process.env.SCRIPT_KIT_BRAIN_TZ ?? "America/Denver",
    SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN: "1",
  },
});

try {
  check("long_markdown_fits_stdin_limit", new TextEncoder().encode(longMarkdown).length < 16_384, {
    bytes: new TextEncoder().encode(longMarkdown).length,
  });

  driver.send({ type: "openNotes", requestId: `${runId}-open-notes` });
  await Bun.sleep(700);

  const setNotes = (await driver.request(
    {
      type: "batch",
      requestId: `${runId}-set-notes-long-markdown`,
      target: { type: "kind", kind: "notes", index: 0 },
      commands: [{ type: "setInput", text: longMarkdown }],
      options: { stopOnError: true, timeout: 10_000 },
    },
    { expect: "batchResult", timeoutMs: 11_000 },
  )) as Json;
  check("set_notes_long_markdown", setNotes.success === true, { batch: setNotes });

  let notesElements = await timedGetElements(driver, "notesGetElementsMs", {
    target: { type: "kind", kind: "notes", index: 0 },
    limit: 100,
  });
  const notesEditor = findSemantic(notesElements, "input:notes-editor");
  const notesRuntime = editorRuntime(notesEditor);
  const notesScrollBefore = scrollMetrics(notesRuntime);

  check("notes_editor_present", Boolean(notesEditor), { focusedSemanticId: notesElements.focusedSemanticId ?? null });
  check("notes_long_markdown_echo_matches", notesEditor?.value === longMarkdown, {
    expectedChars: longMarkdown.length,
    actualChars: typeof notesEditor?.value === "string" ? notesEditor.value.length : null,
  });
  check("notes_scroll_metrics_available", notesScrollBefore?.available === true, { scroll: notesScrollBefore });
  check("notes_editor_viewport_not_collapsed", Number(notesScrollBefore?.clientHeight ?? 0) >= 100, {
    scroll: notesScrollBefore,
  });
  check("notes_scroll_can_scroll_y", Number(notesScrollBefore?.maxScrollTop ?? 0) > 0 || notesScrollBefore?.canScrollY === true, {
    scroll: notesScrollBefore,
  });
  const notesLinkHighlights = markdownLinkHighlightRanges(notesRuntime);
  check("notes_markdown_link_highlight_ranges_present", Number(notesLinkHighlights?.count ?? 0) > 0, {
    ranges: notesLinkHighlights,
  });

  await gpuiKey(driver, "pageup", { type: "kind", kind: "notes", index: 0 });
  await Bun.sleep(180);
  notesElements = await timedGetElements(driver, "notesGetElementsMs", {
    target: { type: "kind", kind: "notes", index: 0 },
    limit: 100,
  });
  const notesScrollAfter = scrollMetrics(editorRuntime(findSemantic(notesElements, "input:notes-editor")));
  check(
    "notes_scroll_position_changed",
    Number(notesScrollAfter?.scrollTop ?? 0) !== Number(notesScrollBefore?.scrollTop ?? 0),
    { before: notesScrollBefore, after: notesScrollAfter },
  );

  await gpuiKey(driver, "p", { type: "kind", kind: "notes", index: 0 }, ["cmd", "shift"]);
  await Bun.sleep(250);
  const previewState = await notesState(driver);
  const previewAnchor = (previewState.previewAnchor ?? {}) as Json;
  check(
    "notes_preview_enabled",
    previewState.view?.previewEnabled === true || previewAnchor.previewEnabled === true,
    { view: previewState.view ?? null, previewAnchor },
  );
  check("notes_preview_anchor_available", previewAnchor.available === true, { previewAnchor });
  check("notes_preview_uses_shared_owner", previewAnchor.owner === "components.notes_editor", {
    owner: previewAnchor.owner ?? null,
    previewAnchor,
  });
  check(
    "notes_preview_uses_shared_render_path",
    previewAnchor.renderPath === "components.notes_editor.render_preview",
    { renderPath: previewAnchor.renderPath ?? null, previewAnchor },
  );
  check(
    "notes_preview_scroll_metrics_available",
    previewAnchor.scrollMetricsAvailable === true && previewAnchor.scroll?.available === true,
    { previewAnchor },
  );
  check(
    "notes_preview_scroll_can_scroll_y",
    Number(previewAnchor.scroll?.maxScrollTop ?? 0) > 0 || previewAnchor.scroll?.canScrollY === true,
    { scroll: previewAnchor.scroll ?? null },
  );

  const dayState = await openDayPage(driver, runId);
  check("opened_day_page", dayState.promptType === "dayPage", {
    promptType: dayState.promptType ?? null,
    windowVisible: dayState.windowVisible ?? null,
  });

  const setDay = (await driver.batch([{ type: "setInput", text: longMarkdown }], {
    timeoutMs: 12_000,
  })) as Json;
  check("set_day_long_markdown", setDay.success === true, { batch: setDay });

  let dayElements = await timedGetElements(driver, "dayGetElementsMs", {
    target: { type: "main" },
    limit: 200,
  });
  check("day_no_local_preview_or_spine_overlay", localOverlayIds(dayElements).length === 0, {
    localOverlayIds: localOverlayIds(dayElements),
  });
  const dayEditor = findSemantic(dayElements, "input:day-page-editor");
  const dayRuntime = editorRuntime(dayEditor);
  const dayScrollBefore = scrollMetrics(dayRuntime);

  check("day_editor_present", Boolean(dayEditor), { focusedSemanticId: dayElements.focusedSemanticId ?? null });
  check("day_long_markdown_echo_matches", dayEditor?.value === longMarkdown, {
    expectedChars: longMarkdown.length,
    actualChars: typeof dayEditor?.value === "string" ? dayEditor.value.length : null,
  });
  check("day_scroll_metrics_available", dayScrollBefore?.available === true, { scroll: dayScrollBefore });
  check("day_editor_viewport_not_collapsed", Number(dayScrollBefore?.clientHeight ?? 0) >= 100, {
    scroll: dayScrollBefore,
  });
  check("day_scroll_can_scroll_y", Number(dayScrollBefore?.maxScrollTop ?? 0) > 0 || dayScrollBefore?.canScrollY === true, {
    scroll: dayScrollBefore,
  });
  const dayLinkHighlights = markdownLinkHighlightRanges(dayRuntime);
  check("day_markdown_link_highlight_ranges_present", Number(dayLinkHighlights?.count ?? 0) > 0, {
    ranges: dayLinkHighlights,
  });

  await gpuiKey(driver, "pageup");
  await Bun.sleep(180);
  dayElements = await timedGetElements(driver, "dayGetElementsMs", {
    target: { type: "main" },
    limit: 200,
  });
  const dayScrollAfter = scrollMetrics(editorRuntime(findSemantic(dayElements, "input:day-page-editor")));
  check(
    "day_scroll_position_changed",
    Number(dayScrollAfter?.scrollTop ?? 0) !== Number(dayScrollBefore?.scrollTop ?? 0),
    { before: dayScrollBefore, after: dayScrollAfter },
  );

  const notesStyle = notesEditor?.style ?? null;
  const dayStyle = dayEditor?.style ?? null;
  check("shared_style_owner_matches", notesStyle?.owner === dayStyle?.owner && dayStyle?.owner === "components.notes_editor", {
    notes: notesStyle?.owner ?? null,
    day: dayStyle?.owner ?? null,
  });
  check(
    "shared_render_path_matches",
    notesStyle?.inputRenderPath === dayStyle?.inputRenderPath &&
      dayStyle?.inputRenderPath === "components.notes_editor.render_input",
    { notes: notesStyle?.inputRenderPath ?? null, day: dayStyle?.inputRenderPath ?? null },
  );
  check("syntax_language_matches", notesRuntime?.language === "markdown" && dayRuntime?.language === "markdown", {
    notes: notesRuntime?.language ?? null,
    day: dayRuntime?.language ?? null,
  });
  check("markdown_highlighter_registered", notesRuntime?.markdownRegistered === true && dayRuntime?.markdownRegistered === true, {
    notes: notesRuntime?.markdownRegistered ?? null,
    day: dayRuntime?.markdownRegistered ?? null,
  });
  check(
    "markdown_inline_highlighter_registered",
    notesRuntime?.markdownInlineRegistered === true && dayRuntime?.markdownInlineRegistered === true,
    { notes: notesRuntime?.markdownInlineRegistered ?? null, day: dayRuntime?.markdownInlineRegistered ?? null },
  );
  check(
    "markdown_query_fingerprint_matches",
    notesRuntime?.highlightQueryFingerprint === dayRuntime?.highlightQueryFingerprint &&
      notesRuntime?.injectionQueryFingerprint === dayRuntime?.injectionQueryFingerprint &&
      notesRuntime?.inlineHighlightQueryFingerprint === dayRuntime?.inlineHighlightQueryFingerprint,
    {
      notes: {
        highlight: notesRuntime?.highlightQueryFingerprint ?? null,
        injection: notesRuntime?.injectionQueryFingerprint ?? null,
        inline: notesRuntime?.inlineHighlightQueryFingerprint ?? null,
      },
      day: {
        highlight: dayRuntime?.highlightQueryFingerprint ?? null,
        injection: dayRuntime?.injectionQueryFingerprint ?? null,
        inline: dayRuntime?.inlineHighlightQueryFingerprint ?? null,
      },
    },
  );
  check(
    "markdown_injection_languages_match",
    JSON.stringify(notesRuntime?.injectionLanguages ?? null) === JSON.stringify(["html", "toml", "yaml"]) &&
      JSON.stringify(notesRuntime?.injectionLanguages ?? null) === JSON.stringify(dayRuntime?.injectionLanguages ?? null),
    { notes: notesRuntime?.injectionLanguages ?? null, day: dayRuntime?.injectionLanguages ?? null },
  );
  check(
    "inline_markdown_injection_disabled",
    notesRuntime?.inlineMarkdownInjectionDisabled === true && dayRuntime?.inlineMarkdownInjectionDisabled === true,
    {
      notes: notesRuntime?.inlineMarkdownInjectionDisabled ?? null,
      day: dayRuntime?.inlineMarkdownInjectionDisabled ?? null,
    },
  );

  const readModeMarkdown = [
    `# Day read mode ${runId}`,
    "",
    "- [ ] unchecked task",
    "- [x] checked task",
    "",
    "Shared preview renderer should mount inside AppView::DayPage.",
    "",
  ].join("\n");
  const setDayTasks = (await driver.batch([{ type: "setInput", text: readModeMarkdown }], {
    timeoutMs: 8000,
  })) as Json;
  check("set_day_task_markdown_for_read_mode", setDayTasks.success === true, { batch: setDayTasks });

  await toggleDayReadModeViaActions(driver, "Preview Markdown");
  const readModeState = await waitForDayReadMode(driver, true);
  const dayPage = (readModeState.dayPage ?? {}) as Json;
  const dayPreviewAnchor = (dayPage.previewAnchor ?? {}) as Json;
  const dayTaskStats = (dayPage.taskStats ?? {}) as Json;
  check("day_read_mode_enabled", dayPage.readMode === true && dayPage.mode === "read", {
    mode: dayPage.mode ?? null,
    readMode: dayPage.readMode ?? null,
  });
  check("day_read_mode_uses_shared_preview_owner", dayPreviewAnchor.owner === "components.notes_editor", {
    previewAnchor: dayPreviewAnchor,
  });
  check(
    "day_read_mode_uses_shared_preview_render_path",
    dayPreviewAnchor.renderPath === "components.notes_editor.render_preview",
    { previewAnchor: dayPreviewAnchor },
  );
  check(
    "day_read_mode_preview_scroll_receipt_available",
    dayPreviewAnchor.scrollMetricsAvailable === true && dayPreviewAnchor.scroll?.available === true,
    { previewAnchor: dayPreviewAnchor },
  );
  check("day_read_mode_task_receipt_counts", dayTaskStats.total === 2 && dayTaskStats.checked === 1 && dayTaskStats.unchecked === 1, {
    taskStats: dayTaskStats,
  });
  const dayReadElements = await timedGetElements(driver, "dayGetElementsMs", {
    target: { type: "main" },
    limit: 220,
  });
  const dayReadPreview = findSemantic(dayReadElements, "day-page-read-preview");
  check("day_read_preview_element_exposed", Boolean(dayReadPreview), { preview: dayReadPreview });

  await toggleDayReadModeViaActions(driver, "Edit Markdown");
  const editModeState = await waitForDayReadMode(driver, false);
  check("day_read_mode_toggle_returns_to_edit", editModeState.dayPage?.readMode === false && editModeState.dayPage?.mode === "edit", {
    dayPage: editModeState.dayPage ?? null,
  });

  const notesP95 = p95(timings.notesGetElementsMs);
  const dayP95 = p95(timings.dayGetElementsMs);
  const dayRatio = dayP95 / Math.max(1, notesP95);
  check("scroll_responsiveness_under_budget", notesP95 <= 250 && dayP95 <= 250 && (dayP95 <= 25 || dayRatio <= 2.0), {
    notesP95,
    dayP95,
    ratio: Number(dayRatio.toFixed(2)),
    timings,
  });

  const appLog = existsSync(driver.logPath) ? readFileSync(driver.logPath, "utf8") : "";
  const unknownWarningCount = (appLog.match(/unknown_warning_count=[1-9][0-9]*/g) ?? []).length;
  check("unknown_warning_count_zero", unknownWarningCount === 0, { unknownWarningCount });

  const pass = failures.length === 0;
  const receipt = {
    schemaVersion: 1,
    tool: "day-notes-preview-renderer-parity-probe",
    classification: pass ? "completed" : "failed",
    pass,
    failures,
    binary: BINARY,
    screenshotProof: "not-used-semantic-devtools-only",
    protectedDirtyFiles: ["CLAUDE.md", "dev.sh", "scripts/agentic/ensure-pi-sidecar.sh"],
    checks,
    timings,
    sessionDir: driver.sessionDir,
    appLog: driver.logPath,
  };
  mkdirSync(".test-output", { recursive: true });
  await Bun.write(OUT_PATH, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
  if (!pass) process.exitCode = 1;
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  check("probe_completed_without_exception", false, { error: message });
  const receipt = {
    schemaVersion: 1,
    tool: "day-notes-preview-renderer-parity-probe",
    classification: "failed",
    pass: false,
    failures,
    binary: BINARY,
    screenshotProof: "not-used-semantic-devtools-only",
    protectedDirtyFiles: ["CLAUDE.md", "dev.sh", "scripts/agentic/ensure-pi-sidecar.sh"],
    checks,
    timings,
    sessionDir: driver.sessionDir,
    appLog: driver.logPath,
  };
  mkdirSync(".test-output", { recursive: true });
  await Bun.write(OUT_PATH, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
  process.exitCode = 1;
} finally {
  await driver.close();
}
