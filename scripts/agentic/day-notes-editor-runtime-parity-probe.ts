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

import { existsSync, readFileSync } from "node:fs";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/day-notes-editor-parity/script-kit-gpui";
const runId = `editor-runtime-${Date.now().toString(36)}`;

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

function editorRuntime(editor: Json | null): Json | null {
  const runtime = editor?.style?.editorRuntime;
  return runtime && typeof runtime === "object" ? (runtime as Json) : null;
}

function scrollMetrics(runtime: Json | null): Json | null {
  const metrics = runtime?.editorScrollMetrics;
  return metrics && typeof metrics === "object" ? (metrics as Json) : null;
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

async function gpuiKey(driver: Driver, key: string, target?: Json): Promise<Json> {
  const event: Json = { type: "keyDown", key, modifiers: [] };
  const payload: Json = {
    type: "simulateGpuiEvent",
    requestId: `${runId}-${key}-${Math.random().toString(36).slice(2)}`,
    event,
  };
  if (target) payload.target = target;
  return driver.request(payload, { expect: "simulateGpuiEventResult", timeoutMs: 5000 });
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
  defaultTimeoutMs: 9000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_BRAIN_TZ: process.env.SCRIPT_KIT_BRAIN_TZ ?? "America/Denver",
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
  check("notes_scroll_can_scroll_y", Number(notesScrollBefore?.maxScrollTop ?? 0) > 0 || notesScrollBefore?.canScrollY === true, {
    scroll: notesScrollBefore,
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
  const dayEditor = findSemantic(dayElements, "input:day-page-editor");
  const dayRuntime = editorRuntime(dayEditor);
  const dayScrollBefore = scrollMetrics(dayRuntime);

  check("day_editor_present", Boolean(dayEditor), { focusedSemanticId: dayElements.focusedSemanticId ?? null });
  check("day_long_markdown_echo_matches", dayEditor?.value === longMarkdown, {
    expectedChars: longMarkdown.length,
    actualChars: typeof dayEditor?.value === "string" ? dayEditor.value.length : null,
  });
  check("day_scroll_metrics_available", dayScrollBefore?.available === true, { scroll: dayScrollBefore });
  check("day_scroll_can_scroll_y", Number(dayScrollBefore?.maxScrollTop ?? 0) > 0 || dayScrollBefore?.canScrollY === true, {
    scroll: dayScrollBefore,
  });

  await gpuiKey(driver, "pageup", { type: "main" });
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
      dayStyle?.inputRenderPath === "components.notes_editor.render_input_state",
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

  const notesP95 = p95(timings.notesGetElementsMs);
  const dayP95 = p95(timings.dayGetElementsMs);
  check("scroll_responsiveness_under_budget", notesP95 <= 250 && dayP95 <= 250 && dayP95 / Math.max(1, notesP95) <= 2.0, {
    notesP95,
    dayP95,
    ratio: Number((dayP95 / Math.max(1, notesP95)).toFixed(2)),
    timings,
  });

  const appLog = existsSync(driver.logPath) ? readFileSync(driver.logPath, "utf8") : "";
  const unknownWarningCount = (appLog.match(/unknown_warning_count=[1-9][0-9]*/g) ?? []).length;
  check("unknown_warning_count_zero", unknownWarningCount === 0, { unknownWarningCount });

  const pass = failures.length === 0;
  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        tool: "day-notes-editor-runtime-parity-probe",
        classification: "completed",
        pass,
        failures,
        screenshotProof: "not-used-semantic-devtools-only",
        protectedDirtyFiles: ["dev.sh", "scripts/agentic/ensure-pi-sidecar.sh"],
        checks,
        timings,
        sessionDir: driver.sessionDir,
      },
      null,
      2,
    ),
  );
  if (!pass) process.exitCode = 1;
} catch (error) {
  const message = error instanceof Error ? error.message : String(error);
  check("probe_completed_without_exception", false, { error: message });
  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        tool: "day-notes-editor-runtime-parity-probe",
        classification: "failed",
        pass: false,
        failures,
        screenshotProof: "not-used-semantic-devtools-only",
        protectedDirtyFiles: ["dev.sh", "scripts/agentic/ensure-pi-sidecar.sh"],
        checks,
        timings,
        sessionDir: driver.sessionDir,
      },
      null,
      2,
    ),
  );
  process.exitCode = 1;
} finally {
  await driver.close();
}
