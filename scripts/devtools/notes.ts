#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Args = {
  command: "inspect" | "resize-compare";
  session: string;
  open: boolean;
  openActions: boolean;
  start: boolean;
  sandbox: boolean;
  sandboxPath: string;
  cleanup: boolean;
  timeoutMs: number;
  limit: number;
  shortLineCount: number;
  tallLineCount: number;
};

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/notes.ts inspect [--session <name>] [--open] [--open-actions] [--start] [--limit <n>]",
    "  bun scripts/devtools/notes.ts resize-compare --session <name> --start --sandbox [--short-lines <n>] [--tall-lines <n>]",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }
  if (argv[0] !== "inspect" && argv[0] !== "resize-compare") {
    console.error(usage());
    process.exit(2);
  }
  const args: Args = {
    command: argv[0],
    session: "default",
    open: false,
    openActions: false,
    start: false,
    sandbox: false,
    sandboxPath: "",
    cleanup: true,
    timeoutMs: 8000,
    limit: 80,
    shortLineCount: 2,
    tallLineCount: 80,
  };
  for (let index = 1; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--session") {
      args.session = argv[++index] ?? args.session;
    } else if (arg === "--open") {
      args.open = true;
    } else if (arg === "--open-actions") {
      args.openActions = true;
    } else if (arg === "--start") {
      args.start = true;
    } else if (arg === "--sandbox") {
      args.sandbox = true;
    } else if (arg === "--sandbox-path") {
      args.sandboxPath = argv[++index] ?? "";
      args.sandbox = true;
    } else if (arg === "--no-cleanup") {
      args.cleanup = false;
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
    } else if (arg === "--limit") {
      args.limit = Number(argv[++index] ?? args.limit);
    } else if (arg === "--short-lines") {
      args.shortLineCount = Number(argv[++index] ?? args.shortLineCount);
    } else if (arg === "--tall-lines") {
      args.tallLineCount = Number(argv[++index] ?? args.tallLineCount);
    }
  }
  return args;
}

async function run(command: string[], label: string): Promise<JsonObject> {
  const proc = Bun.spawn(command, { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  if (exitCode !== 0) {
    return { status: "error", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
  }
  try {
    return JSON.parse(stdout);
  } catch {
    return { status: "ok", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
  }
}

async function maybeOpenNotes(args: Args) {
  if (args.start) {
    await run(["bash", "scripts/agentic/session.sh", "start", args.session], "session-start");
  }
  if (!args.open) {
    return null;
  }
  return run([
    "bash",
    "scripts/agentic/session.sh",
    "send",
    args.session,
    JSON.stringify({ type: "openNotes", requestId: `devtools-notes-open-${Date.now()}` }),
  ], "open-notes");
}

async function maybeOpenActions(args: Args) {
  if (!args.openActions) {
    return null;
  }
  const receipt = await run([
    "bash",
    "scripts/agentic/session.sh",
    "rpc",
    args.session,
    JSON.stringify({
      type: "batch",
      requestId: `devtools-notes-open-actions-${Date.now()}`,
      target: { type: "kind", kind: "notes" },
      commands: [{ type: "openActions" }],
      options: { stopOnError: true, rollbackOnError: false, timeout: args.timeoutMs },
      trace: "on",
    }),
    "--expect",
    "batchResult",
    "--timeout",
    String(args.timeoutMs),
  ], "open-actions");
  await Bun.sleep(250);
  return receipt;
}

function notesRuntimeState(envelope: JsonObject): JsonObject {
  return ((responseOf(envelope).notes as JsonObject | undefined) ?? {});
}

function notesActionOpen(value: JsonObject): boolean {
  const commandBars = (value.commandBars as JsonObject | undefined) ?? {};
  const actions = (commandBars.actions as JsonObject | undefined) ?? {};
  return actions.open === true || ((actions.dialog as JsonObject | undefined)?.open === true);
}

function notesActiveScope(value: JsonObject): unknown {
  const shortcutRegistry = (value.shortcutRegistry as JsonObject | undefined) ?? {};
  return shortcutRegistry.activeScope ?? null;
}

function notesFocusGeneration(value: JsonObject): number | null {
  const focusTransitions = (value.focusTransitions as JsonObject | undefined) ?? {};
  return typeof focusTransitions.generation === "number" ? focusTransitions.generation : null;
}

function hasDraftSnapshot(value: JsonObject): boolean {
  const draftSnapshot = value.draftSnapshot as JsonObject | undefined;
  const draft = draftSnapshot?.draft as JsonObject | undefined;
  return (
    typeof draft?.bodyFingerprint === "string" &&
    typeof draft?.bodyByteLength === "number" &&
    draft?.selectionUnit === "utf8ByteOffset" &&
    draftSnapshot?.contentReturned === false
  );
}

function missingCoveragePrimitives(coverage: ReturnType<typeof notesCoverage>, runtimeNotes: JsonObject): string[] {
  const missing = coverage.missingRuntimePrimitives.map(String);
  if (hasDraftSnapshot(runtimeNotes)) {
    return missing.filter((primitive) => primitive !== "draft snapshot fingerprint");
  }
  return missing;
}

function shortcutSnapshot(value: JsonObject) {
  return {
    actionsOpen: notesActionOpen(value),
    activeScope: notesActiveScope(value),
    focusTransitionGeneration: notesFocusGeneration(value),
  };
}

function buildShortcutActivationReceipt(sendReceipt: JsonObject | null, beforeEnvelope: JsonObject | null, afterEnvelope: JsonObject | null) {
  if (!sendReceipt || !beforeEnvelope || !afterEnvelope) {
    return null;
  }
  const before = notesRuntimeState(beforeEnvelope);
  const after = notesRuntimeState(afterEnvelope);
  const beforeSnapshot = shortcutSnapshot(before);
  const afterSnapshot = shortcutSnapshot(after);
  const delivered = sendReceipt.status === "ok" || sendReceipt.sent === true;
  const opened = afterSnapshot.actionsOpen === true;
  const focusGenerationAdvanced =
    typeof beforeSnapshot.focusTransitionGeneration === "number" &&
    typeof afterSnapshot.focusTransitionGeneration === "number" &&
    afterSnapshot.focusTransitionGeneration > beforeSnapshot.focusTransitionGeneration;

  return {
    schemaVersion: 1,
    shortcut: "Cmd+K",
    channel: "protocol.batch.openActions",
    requestedOwner: "notes.actions",
    delivered,
    before: beforeSnapshot,
    after: afterSnapshot,
    assertions: {
      sendReceiptOk: delivered,
      actionsPanelOpened: opened,
      activeScopeBecameActionsPanel: afterSnapshot.activeScope === "actionsPanel",
      focusTransitionAdvanced: focusGenerationAdvanced,
    },
    classification: delivered && opened ? "ok" : "reproduced",
    failure: delivered && !opened
      ? "target-scoped batch openActions did not open the Notes actions command bar"
      : null,
    receipts: {
      send: sendReceipt,
      beforeState: beforeEnvelope,
      afterState: afterEnvelope,
    },
  };
}

async function waitForNotesTarget(args: Args) {
  const deadline = Date.now() + args.timeoutMs;
  let last: JsonObject = {};
  while (Date.now() < deadline) {
    last = await run([
      "bun",
      "scripts/devtools/targets.ts",
      "inspect",
      "--session",
      args.session,
      "--target-kind",
      "notes",
      "--strict",
    ], "targets.inspect");
    if (last.classification === "ok") {
      return last;
    }
    await Bun.sleep(100);
  }
  return last;
}

function requestId(prefix: string) {
  return `devtools-notes-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run([
    "bash",
    "scripts/agentic/session.sh",
    "rpc",
    session,
    JSON.stringify(payload),
    "--expect",
    expect,
    "--timeout",
    String(timeoutMs),
  ], String(payload.type ?? "rpc"));
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

function asArray(value: unknown): JsonObject[] {
  return Array.isArray(value) ? value.filter((entry): entry is JsonObject => typeof entry === "object" && entry !== null) : [];
}

function asObject(value: unknown): JsonObject {
  return value && typeof value === "object" ? value as JsonObject : {};
}

function asNumber(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function notesLines(count: number, prefix = "DevTools notes resize line"): string {
  return Array.from({ length: count }, (_, index) =>
    `${prefix} ${String(index + 1).padStart(2, "0")}`
  ).join("\n");
}

function notesCoverage(coverage: JsonObject) {
  const notes = asArray(coverage.surfaces).find((surface) => surface.id === "notes") ?? {};
  return {
    status: notes.status ?? null,
    sourceFiles: notes.sourceFiles ?? [],
    features: notes.features ?? [],
    shortcuts: notes.shortcuts ?? [],
    supportedNow: notes.supportedNow ?? [],
    missingRuntimePrimitives: notes.missingRuntimePrimitives ?? [],
    regressionRecipeRole: notes.regressionRecipeRole ?? null,
  };
}

function textFingerprint(value: string) {
  let hash = 2166136261;
  for (const char of value) {
    hash ^= char.charCodeAt(0);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function notesState(elements: JsonObject, focus: JsonObject, text: JsonObject, runtimeState: JsonObject, layout: JsonObject) {
  const nodes = asArray(elements.nodes);
  const editor = nodes.find((node) => node.semanticId === "input:notes-editor") ?? null;
  const editorText = String(editor?.label ?? editor?.text ?? editor?.value ?? "");
  const runtimeNotes = (runtimeState.notes as JsonObject | undefined) ?? {};
  const runtimeEditor = (runtimeNotes.editor as JsonObject | undefined) ?? {};
  const layoutRegions = asArray(layout.regions);
  const editorRegion = layoutRegions.find((region) => {
    const name = String(region.name ?? "");
    return name === "NotesEditor" || name === "NotesPreview" || name === "NotesEmbeddedAcp";
  }) ?? null;
  return {
    panelPresent: nodes.some((node) => node.semanticId === "panel:notes-window"),
    editorPresent: Boolean(editor),
    editorFocused: focus.focusedSemanticId === "input:notes-editor" || editor?.focused === true,
    focusedSemanticId: focus.focusedSemanticId ?? elements.focusedSemanticId ?? null,
    activeNoteId: runtimeNotes.activeNoteId ?? null,
    dirtyState: runtimeNotes.dirtyState ?? null,
    editorTextLength: runtimeEditor.textLength ?? editorText.length,
    editorFingerprint: runtimeEditor.textFingerprint ?? (editorText ? textFingerprint(editorText) : null),
    selectionRange: runtimeEditor.selectionRange ?? null,
    cursorLine: runtimeEditor.cursorLine ?? null,
    draftSnapshot: runtimeNotes.draftSnapshot ?? null,
    editorAnchor: runtimeNotes.editorAnchor ?? null,
    previewAnchor: runtimeNotes.previewAnchor ?? null,
    view: runtimeNotes.view ?? null,
    commandBars: runtimeNotes.commandBars ?? null,
    shortcutRegistry: runtimeNotes.shortcutRegistry ?? null,
    focusTransitions: runtimeNotes.focusTransitions ?? null,
    layout: {
      classification: layout.classification ?? null,
      componentCount: layout.componentCount ?? null,
      editorRegion: editorRegion ? { name: editorRegion.name ?? null, bounds: editorRegion.bounds ?? null } : null,
      pressure: layout.pressure ?? null,
      viewport: layout.viewport ?? null,
    },
    storage: runtimeNotes.storage ?? null,
    counts: runtimeNotes.counts ?? null,
    redacted: runtimeNotes.redacted ?? null,
    footerTexts: text.footerTexts ?? [],
  };
}

function classify(target: JsonObject, elements: JsonObject, coverage: ReturnType<typeof notesCoverage>) {
  if (target.classification !== "ok") {
    return target.classification ?? "blocked-by-target-ambiguity";
  }
  if (elements.classification !== "ok") {
    return elements.classification ?? "blocked-by-missing-primitive";
  }
  if (asArray(elements.nodes).length === 0) {
    return "blocked-by-missing-primitive";
  }
  if (coverage.missingRuntimePrimitives.length > 0) {
    return "blocked-by-missing-primitive";
  }
  return "ok";
}

function unsafeResizeReceipt(args: Args, reason: string, extra: JsonObject = {}) {
  return {
    schemaVersion: 1,
    tool: "script-kit-devtools.notes",
    command: "notes.resize-compare",
    classification: "blocked-by-unsafe-operation",
    session: args.session,
    safety: {
      mutatesNotesEditor: true,
      requiresSandbox: true,
      sandboxRequested: args.sandbox,
      startedByTool: args.start,
    },
    reason,
    ...extra,
  };
}

async function sessionStatus(session: string) {
  return run(["bash", "scripts/agentic/session.sh", "status", session], "session-status");
}

async function stopSession(session: string) {
  return run(["bash", "scripts/agentic/session.sh", "stop", session], "session-stop");
}

async function getNotesState(args: Args, notesTargetId: string, label: string) {
  return rpc(args.session, {
    type: "getState",
    requestId: requestId(label),
    target: { type: "id", id: notesTargetId },
  }, "stateResult", args.timeoutMs);
}

async function inspectNotesTarget(args: Args) {
  return run([
    "bun",
    "scripts/devtools/targets.ts",
    "inspect",
    "--session",
    args.session,
    "--target-kind",
    "notes",
    "--strict",
  ], "targets.inspect");
}

function targetHeight(target: JsonObject): number | null {
  const bounds = asObject((target.resolvedTarget as JsonObject | undefined)?.bounds);
  return asNumber(bounds.height);
}

function targetWidth(target: JsonObject): number | null {
  const bounds = asObject((target.resolvedTarget as JsonObject | undefined)?.bounds);
  return asNumber(bounds.width);
}

async function setNotesInput(args: Args, text: string, label: string) {
  const receipt = await rpc(args.session, {
    type: "batch",
    requestId: requestId(label),
    target: { type: "kind", kind: "notes", index: 0 },
    commands: [{ type: "setInput", text }],
    options: { stopOnError: true, rollbackOnError: false, timeout: args.timeoutMs },
    trace: "on",
  }, "batchResult", args.timeoutMs);
  await Bun.sleep(200);
  return receipt;
}

async function runResizeCompare(args: Args) {
  if (!args.sandbox) {
    console.log(JSON.stringify(unsafeResizeReceipt(args, "notes.resize-compare mutates the Notes editor and requires --sandbox."), null, 2));
    return;
  }
  if (!args.start) {
    console.log(JSON.stringify(unsafeResizeReceipt(args, "notes.resize-compare requires --start so the app can be launched with SCRIPT_KIT_TEST_NOTES_DB_PATH."), null, 2));
    return;
  }

  const beforeStatus = await sessionStatus(args.session);
  if (beforeStatus.alive === true) {
    console.log(JSON.stringify(unsafeResizeReceipt(args, "Refusing to reuse a running session for sandboxed Notes mutation. Use a fresh session name or stop it first.", { beforeStatus }), null, 2));
    return;
  }

  const sandboxPath = args.sandboxPath || `/tmp/sk-devtools-notes-${args.session}-${Date.now()}/notes.sqlite`;
  process.env.SCRIPT_KIT_TEST_NOTES_DB_PATH = sandboxPath;

  const startedByTool = true;
  let cleanupReceipt: JsonObject | null = null;
  const receipts: JsonObject = { beforeStatus };

  try {
    const openReceipt = await maybeOpenNotes({ ...args, open: true });
    receipts.open = openReceipt;
    const target = await waitForNotesTarget(args);
    receipts.target = target;
    const notesTargetId = String((target.resolvedTarget as JsonObject | undefined)?.automationId ?? "notes");
    const beforeStateEnvelope = await getNotesState(args, notesTargetId, "resize-before-state");
    receipts.beforeState = beforeStateEnvelope;
    const beforeRuntime = notesRuntimeState(beforeStateEnvelope);
    const storage = asObject(beforeRuntime.storage);
    if (storage.testSandbox !== true) {
      console.log(JSON.stringify(unsafeResizeReceipt(args, "Resolved Notes runtime is not using the sandbox notes store; refusing setInput mutation.", {
        sandboxPath,
        storage,
        receipts,
      }), null, 2));
      return;
    }

    const beforeTarget = await inspectNotesTarget(args);
    receipts.beforeTarget = beforeTarget;
    const tallText = notesLines(args.tallLineCount);
    const growBatch = await setNotesInput(args, tallText, "resize-grow");
    const afterGrowTarget = await inspectNotesTarget(args);
    const afterGrowState = await getNotesState(args, notesTargetId, "resize-after-grow-state");
    receipts.growBatch = growBatch;
    receipts.afterGrowTarget = afterGrowTarget;
    receipts.afterGrowState = afterGrowState;

    const shortText = notesLines(args.shortLineCount, "DevTools notes restored line");
    const shrinkBatch = await setNotesInput(args, shortText, "resize-shrink");
    const afterShrinkTarget = await inspectNotesTarget(args);
    const afterShrinkState = await getNotesState(args, notesTargetId, "resize-after-shrink-state");
    receipts.shrinkBatch = shrinkBatch;
    receipts.afterShrinkTarget = afterShrinkTarget;
    receipts.afterShrinkState = afterShrinkState;

    const beforeHeight = targetHeight(beforeTarget);
    const afterGrowHeight = targetHeight(afterGrowTarget);
    const afterShrinkHeight = targetHeight(afterShrinkTarget);
    const beforeWidth = targetWidth(beforeTarget);
    const afterGrowWidth = targetWidth(afterGrowTarget);
    const afterShrinkWidth = targetWidth(afterShrinkTarget);
    const growDeltaPx = beforeHeight != null && afterGrowHeight != null ? afterGrowHeight - beforeHeight : null;
    const shrinkDeltaPx = afterGrowHeight != null && afterShrinkHeight != null ? afterGrowHeight - afterShrinkHeight : null;
    const heightGrewForTallContent = growDeltaPx != null && growDeltaPx > 0;
    const heightShrankForShortContent = shrinkDeltaPx != null && shrinkDeltaPx > 0;
    const widthStable = beforeWidth != null
      && afterGrowWidth != null
      && afterShrinkWidth != null
      && Math.abs(afterGrowWidth - beforeWidth) <= 1
      && Math.abs(afterShrinkWidth - beforeWidth) <= 1;
    const growBatchSucceeded = responseOf(growBatch).success === true;
    const shrinkBatchSucceeded = responseOf(shrinkBatch).success === true;
    const beforeView = asObject(beforeRuntime.view);
    const afterGrowView = asObject(notesRuntimeState(afterGrowState).view);
    const afterShrinkView = asObject(notesRuntimeState(afterShrinkState).view);
    const autoSizingStayedEnabled =
      beforeView.autoSizingEnabled === true &&
      afterGrowView.autoSizingEnabled === true &&
      afterShrinkView.autoSizingEnabled === true;
    const fixed =
      growBatchSucceeded &&
      shrinkBatchSucceeded &&
      heightGrewForTallContent &&
      heightShrankForShortContent &&
      widthStable &&
      autoSizingStayedEnabled;

    console.log(JSON.stringify({
      schemaVersion: 1,
      tool: "script-kit-devtools.notes",
      command: "notes.resize-compare",
      classification: fixed ? "ok" : "reproduced",
      session: args.session,
      target: afterShrinkTarget.resolvedTarget ?? null,
      safety: {
        mutatesNotesEditor: true,
        sandboxRequired: true,
        sandboxConfirmed: true,
        sandboxNotesDbPath: sandboxPath,
        usedNativeInput: false,
        usedNativePointer: false,
        usedScreenshot: false,
        openedSystemSettings: false,
        mutatedTcc: false,
      },
      resizeCompare: {
        shortLineCount: args.shortLineCount,
        tallLineCount: args.tallLineCount,
        beforeHeight,
        afterGrowHeight,
        afterShrinkHeight,
        growDeltaPx,
        shrinkDeltaPx,
        beforeWidth,
        afterGrowWidth,
        afterShrinkWidth,
        heightGrewForTallContent,
        heightShrankForShortContent,
        widthStable,
        growBatchSucceeded,
        shrinkBatchSucceeded,
        autoSizingStayedEnabled,
        beforeView,
        afterGrowView,
        afterShrinkView,
      },
      assertions: {
        sandboxConfirmed: true,
        growBatchSucceeded,
        shrinkBatchSucceeded,
        heightGrewForTallContent,
        heightShrankForShortContent,
        widthStable,
        autoSizingStayedEnabled,
      },
      missingPrimitives: fixed ? [
        "editor and preview scroll anchors",
        "ACP embedded origin receipts",
        "portal session provenance",
        "Notes native shortcut activation parity receipt",
      ] : ["auto-resize before/after compare"],
      receipts,
      cleanup: null,
    }, null, 2));
  } finally {
    if (args.cleanup && startedByTool) {
      cleanupReceipt = await stopSession(args.session);
      void cleanupReceipt;
    }
  }
}

async function runInspect(args: Args) {
  const openReceipt = await maybeOpenNotes(args);
  const target = await waitForNotesTarget(args);
  const notesTargetId = String((target.resolvedTarget as JsonObject | undefined)?.automationId ?? "notes");
  const shortcutBeforeEnvelope = args.openActions ? await rpc(args.session, {
    type: "getState",
    requestId: requestId("shortcut-before"),
    target: { type: "id", id: notesTargetId },
  }, "stateResult", args.timeoutMs) : null;
  const openActionsReceipt = await maybeOpenActions(args);
  const shortcutAfterEnvelope = args.openActions ? await rpc(args.session, {
    type: "getState",
    requestId: requestId("shortcut-after"),
    target: { type: "id", id: notesTargetId },
  }, "stateResult", args.timeoutMs) : null;
  const shortcutActivation = buildShortcutActivationReceipt(
    openActionsReceipt,
    shortcutBeforeEnvelope,
    shortcutAfterEnvelope,
  );
  const targetArgs = ["--session", args.session, "--target-id", notesTargetId, "--strict"];
  const elements = await run(["bun", "scripts/devtools/elements.ts", "snapshot", ...targetArgs, "--limit", String(args.limit)], "elements.snapshot");
  const focus = await run(["bun", "scripts/devtools/focus.ts", "inspect", ...targetArgs], "focus.inspect");
  const text = await run(["bun", "scripts/devtools/text.ts", "measure", ...targetArgs, "--limit", String(args.limit)], "text.measure");
  const layout = await run(["bun", "scripts/devtools/layout.ts", "measure", ...targetArgs, "--include", "nodes,regions,scroll,resize,overlaps"], "layout.measure");
  const stateEnvelope = await rpc(args.session, {
    type: "getState",
    requestId: requestId("state"),
    target: { type: "id", id: notesTargetId },
  }, "stateResult", args.timeoutMs);
  const runtimeState = responseOf(stateEnvelope);
  const coverageRaw = await run(["bun", "scripts/devtools/coverage.ts", "--surface", "notes"], "coverage.notes");
  const coverage = notesCoverage(coverageRaw);
  const state = notesState(elements, focus, text, runtimeState, layout);
  const runtimeNotes = (runtimeState.notes as JsonObject | undefined) ?? {};
  const missing = [
    ...new Set([
      ...(Array.isArray(elements.missingPrimitives) ? elements.missingPrimitives.map(String) : []),
      ...(Array.isArray(text.missingPrimitives) ? text.missingPrimitives.map(String) : []),
      ...missingCoveragePrimitives(coverage, runtimeNotes),
      state.activeNoteId == null ? "active note id" : "",
      state.dirtyState == null ? "dirty state" : "",
      state.selectionRange == null ? "cursor and selection ranges" : "",
      state.draftSnapshot == null ? "draft snapshot fingerprint" : "",
      state.layout.editorRegion == null ? "notes editor layout region" : "",
      state.storage == null ? "note store generation and sandbox identity" : "",
      state.commandBars == null ? "notes command bar runtime state" : "",
      state.shortcutRegistry == null ? "notes shortcut registry" : "",
      state.focusTransitions == null ? "notes focus owner transition timeline" : "",
    ].filter(Boolean)),
  ];

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.notes",
    command: "notes.inspect",
    classification: classify(target, elements, coverage),
    session: args.session,
    openReceipt,
    openActionsReceipt,
    shortcutActivation,
    target: target.resolvedTarget ?? null,
    requestedTarget: target.requestedTarget ?? { selector: { type: "kind", kind: "notes" } },
    notesState: state,
    runtimeState,
    coverage,
    receipts: {
      target,
      elements,
      focus,
      text,
      layout,
      state: stateEnvelope,
    },
    missingPrimitives: missing,
    warnings: [
      ...(Array.isArray(elements.warnings) ? elements.warnings : []),
      ...(Array.isArray(focus.warnings) ? focus.warnings : []),
      ...(Array.isArray(text.warnings) ? text.warnings : []),
      missing.length > 0 ? `Notes inspection remains fail-closed until missing primitives are available: ${missing.join(", ")}.` : "",
    ].filter(Boolean),
    errors: [target, elements, focus, text].filter((receipt) => receipt.status === "error"),
  }, null, 2));
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  if (args.command === "resize-compare") {
    await runResizeCompare(args);
    return;
  }
  await runInspect(args);
}

await main();
