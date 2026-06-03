#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Rect = {
  x: number;
  y: number;
  width: number;
  height: number;
};

type Args = {
  session: string;
  target?: JsonObject;
  timeoutMs: number;
  forwarded: string[];
};

function usage() {
  return "Usage:\n  bun scripts/devtools/scroll.ts inspect [target args]";
}

function parseArgs(argv: string[]): Args {
  if (argv[0] !== "inspect") {
    console.error(usage());
    process.exit(2);
  }
  const args: Args = { session: "default", timeoutMs: 8000, forwarded: [] };
  for (let index = 1; index < argv.length; index += 1) {
    const arg = argv[index];
    args.forwarded.push(arg);
    if (arg === "--session") {
      args.session = argv[++index] ?? args.session;
      args.forwarded.push(args.session);
    } else if (arg === "--target-id") {
      args.target = { type: "id", id: argv[++index] ?? "" };
      args.forwarded.push(String(args.target.id ?? ""));
    } else if (arg === "--target-kind") {
      const kind = argv[++index] ?? "main";
      args.target = { type: "kind", kind };
      args.forwarded.push(kind);
    } else if (arg === "--target-index") {
      const value = Number(argv[++index] ?? 0);
      if (!args.target || args.target.type !== "kind") {
        throw new Error("--target-index requires --target-kind first");
      }
      args.target.index = value;
      args.forwarded.push(String(value));
    } else if (arg === "--target-title") {
      args.target = { type: "titleContains", text: argv[++index] ?? "" };
      args.forwarded.push(String(args.target.text ?? ""));
    } else if (arg === "--focused") {
      args.target = { type: "focused" };
    } else if (arg === "--main") {
      args.target = { type: "main" };
    } else if (arg === "--surface") {
      args.forwarded.push(argv[++index] ?? "");
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
      args.forwarded.push(String(args.timeoutMs));
    } else if (arg === "--strict" || arg === "--start" || arg === "--show") {
      // Forwarded only.
    } else if (arg === "--help" || arg === "-h") {
      console.log(usage());
      process.exit(0);
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
    return { status: "error", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim(), error: "invalid_json_output" };
  }
}

function requestId(prefix: string) {
  return `devtools-scroll-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run(["bash", "scripts/agentic/session.sh", "rpc", session, JSON.stringify(payload), "--expect", expect, "--timeout", String(timeoutMs)], String(payload.type ?? "rpc"));
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

function asObject(value: unknown): JsonObject {
  return value && typeof value === "object" && !Array.isArray(value) ? value as JsonObject : {};
}

function asNumber(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function asArray(value: unknown): JsonObject[] {
  return Array.isArray(value)
    ? value.filter((entry): entry is JsonObject => entry != null && typeof entry === "object")
    : [];
}

function rectFrom(value: unknown): Rect | null {
  const rect = asObject(value);
  const x = asNumber(rect.x);
  const y = asNumber(rect.y);
  const width = asNumber(rect.width);
  const height = asNumber(rect.height);
  return x == null || y == null || width == null || height == null
    ? null
    : { x, y, width, height };
}

export function notesScrollFromState(state: JsonObject) {
  const notes = asObject(state.notes);
  const view = asObject(notes.view);
  const editorAnchor = asObject(notes.editorAnchor);
  const previewAnchor = asObject(notes.previewAnchor);
  const previewEnabled = previewAnchor.previewEnabled === true || view.previewEnabled === true;
  const owner = previewEnabled ? "notes.preview" : "notes.editor";
  const anchor = previewEnabled ? previewAnchor : editorAnchor;
  const scroll = asObject(anchor.scroll);
  const scrollHeight = asNumber(scroll.scrollHeight);
  const clientHeight = asNumber(scroll.clientHeight);
  return {
    owner,
    activeNoteId: notes.activeNoteId ?? null,
    generation: asObject(notes.generations).state ?? null,
    scrollTop: asNumber(scroll.scrollTop),
    contentHeight: scrollHeight,
    viewportHeight: clientHeight,
    safeViewportHeight: clientHeight,
    maxScrollTop: scrollHeight != null && clientHeight != null ? Math.max(0, scrollHeight - clientHeight) : null,
    selectedIndex: null,
    selectedRowTop: null,
    selectedRowBottom: null,
    selectedRowVisible: null,
    selectedRowAboveFooter: null,
    itemCount: asObject(notes.counts).noteCount ?? null,
    anchor,
  };
}

function mainListScrollFromState(state: JsonObject) {
  return asObject(state.mainListScroll);
}

async function layoutMeasurement(args: Args): Promise<JsonObject | null> {
  const layoutReceipt = await run(
    ["bun", "scripts/devtools/layout.ts", "measure", ...args.forwarded],
    "layout.measure",
  );
  return layoutReceipt.status === "error" ? null : layoutReceipt;
}

function layoutNodeBounds(layoutReceipt: JsonObject, name: string): Rect | null {
  const nodes = asArray(layoutReceipt.nodes);
  const node = nodes.find((entry) => entry.name === name);
  return rectFrom(node?.bounds);
}

function normalizeScriptListScrollMeasurement(scroll: JsonObject, layoutReceipt: JsonObject | null) {
  const listStateViewportHeight = asNumber(scroll.viewportHeight);
  const listStateSafeViewportHeight = asNumber(scroll.safeViewportHeight);
  const rawSelectedRowVisible = scroll.selectedRowVisible;
  const rawSelectedRowAboveFooter = scroll.selectedRowAboveFooter;

  if (listStateViewportHeight == null || listStateViewportHeight > 0) {
    return {
      scroll,
      classification: null,
      missingPrimitive: null,
      listStateViewportHeight,
      effectiveViewportHeight: listStateViewportHeight,
      effectiveSafeViewportHeight: listStateSafeViewportHeight,
      viewportMeasurementSource: "listState",
      viewportMeasurementWarning: null,
      selectedRowVisible: rawSelectedRowVisible ?? null,
      selectedRowAboveFooter: rawSelectedRowAboveFooter ?? null,
    };
  }

  const listBounds = layoutReceipt ? layoutNodeBounds(layoutReceipt, "ScriptList") : null;
  const selectedIndex = asNumber(scroll.selectedIndex);
  const selectedRowBounds =
    selectedIndex == null || !layoutReceipt
      ? null
      : layoutNodeBounds(layoutReceipt, `ListItem[${selectedIndex}]`);
  const footerBounds = layoutReceipt ? layoutNodeBounds(layoutReceipt, "MainViewFooter") : null;

  if (!listBounds || !selectedRowBounds) {
    return {
      scroll: {
        ...scroll,
        selectedRowVisible: null,
        selectedRowAboveFooter: null,
      },
      classification: "blocked-by-missing-primitive",
      missingPrimitive: "selectedRowBounds",
      listStateViewportHeight,
      effectiveViewportHeight: listBounds?.height ?? null,
      effectiveSafeViewportHeight: null,
      viewportMeasurementSource: listBounds ? "layout" : "listState",
      viewportMeasurementWarning: "listStateViewportUnmeasured",
      selectedRowVisible: null,
      selectedRowAboveFooter: null,
    };
  }

  const effectiveViewportHeight = listBounds.height;
  const selectedRowVisible =
    selectedRowBounds.y >= listBounds.y
    && selectedRowBounds.y + selectedRowBounds.height <= listBounds.y + listBounds.height;
  const effectiveSafeViewportHeight = footerBounds
    ? Math.max(0, footerBounds.y - listBounds.y)
    : effectiveViewportHeight;
  const selectedRowAboveFooter =
    selectedRowBounds.y + selectedRowBounds.height <= listBounds.y + effectiveSafeViewportHeight;

  return {
    scroll: {
      ...scroll,
      viewportHeight: effectiveViewportHeight,
      safeViewportHeight: effectiveSafeViewportHeight,
      selectedRowTop: selectedRowBounds.y - listBounds.y,
      selectedRowBottom: selectedRowBounds.y + selectedRowBounds.height - listBounds.y,
      selectedRowVisible,
      selectedRowAboveFooter,
    },
    classification: null,
    missingPrimitive: null,
    listStateViewportHeight,
    effectiveViewportHeight,
    effectiveSafeViewportHeight,
    viewportMeasurementSource: "layout",
    viewportMeasurementWarning: "listStateViewportUnmeasured",
    selectedRowVisible,
    selectedRowAboveFooter,
  };
}

function classify(
  targetReceipt: JsonObject,
  stateEnvelope: JsonObject,
  scroll: JsonObject,
  measurementClassification?: string | null,
) {
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (stateEnvelope.status === "error") {
    return "blocked-by-timeout";
  }
  if (measurementClassification) {
    return measurementClassification;
  }
  if (Object.keys(scroll).length === 0 || scroll.scrollTop == null || scroll.viewportHeight == null) {
    return "blocked-by-missing-primitive";
  }
  return "ok";
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const targetReceipt = await run(["bun", "scripts/devtools/targets.ts", "inspect", ...args.forwarded], "targets.inspect");
  const selector = (targetReceipt.requestedTarget as JsonObject | undefined)?.selector ?? args.target ?? { type: "focused" };
  const stateEnvelope = await rpc(args.session, {
    type: "getState",
    requestId: requestId("state"),
    target: selector,
    summaryOnly: true,
  }, "stateResult", args.timeoutMs);
  const state = responseOf(stateEnvelope);
  const resolved = asObject(targetReceipt.resolvedTarget);
  const resolvedKind = String(resolved.targetKind ?? "").toLowerCase();
  const resolvedSurface = String(resolved.semanticSurface ?? resolved.surfaceKind ?? "").toLowerCase();
  const isNotesTarget = resolvedKind === "notes" || resolvedSurface === "notes";
  const rawScroll = isNotesTarget ? notesScrollFromState(state) : mainListScrollFromState(state);
  const normalized = isNotesTarget
    ? {
        scroll: rawScroll,
        classification: null,
        missingPrimitive: null,
        listStateViewportHeight: asNumber(rawScroll.viewportHeight),
        effectiveViewportHeight: asNumber(rawScroll.viewportHeight),
        effectiveSafeViewportHeight: asNumber(rawScroll.safeViewportHeight),
        viewportMeasurementSource: "listState",
        viewportMeasurementWarning: null,
        selectedRowVisible: rawScroll.selectedRowVisible ?? null,
        selectedRowAboveFooter: rawScroll.selectedRowAboveFooter ?? null,
      }
    : normalizeScriptListScrollMeasurement(
        rawScroll,
        asNumber(rawScroll.viewportHeight) != null && (asNumber(rawScroll.viewportHeight) ?? 0) <= 0
          ? await layoutMeasurement(args)
          : null,
      );
  const scroll = normalized.scroll;
  const contentHeight = asNumber(scroll.contentHeight);
  const viewportHeight = asNumber(scroll.viewportHeight);
  const maxScrollTop = asNumber(scroll.maxScrollTop);
  const scrollTop = asNumber(scroll.scrollTop);
  const canScrollY = maxScrollTop != null ? maxScrollTop > 0 : contentHeight != null && viewportHeight != null ? contentHeight > viewportHeight : null;
  const classification = classify(targetReceipt, stateEnvelope, scroll, normalized.classification);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.scroll",
    command: "scroll.inspect",
    classification,
    session: args.session,
    requestedTarget: targetReceipt.requestedTarget ?? { selector },
    target: targetReceipt.resolvedTarget ?? null,
    scroll: {
      scrollTop,
      scrollTopItem: scroll.scrollTopItem ?? null,
      contentHeight,
      viewportHeight,
      listStateViewportHeight: normalized.listStateViewportHeight,
      effectiveViewportHeight: normalized.effectiveViewportHeight,
      viewportMeasurementSource: normalized.viewportMeasurementSource,
      viewportMeasurementWarning: normalized.viewportMeasurementWarning,
      safeViewportHeight: scroll.safeViewportHeight ?? null,
      effectiveSafeViewportHeight: normalized.effectiveSafeViewportHeight,
      footerHeight: scroll.footerHeight ?? null,
      maxScrollTop,
      canScrollY,
      selectedIndex: scroll.selectedIndex ?? state.selectedIndex ?? null,
      selectedRowTop: scroll.selectedRowTop ?? null,
      selectedRowBottom: scroll.selectedRowBottom ?? null,
      selectedRowVisible: scroll.selectedRowVisible ?? null,
      selectedRowAboveFooter: scroll.selectedRowAboveFooter ?? null,
      itemCount: scroll.itemCount ?? state.visibleChoiceCount ?? null,
      owner: scroll.owner ?? (isNotesTarget ? "notes.unknown" : "main.list"),
      activeNoteId: scroll.activeNoteId ?? null,
      generation: scroll.generation ?? null,
    },
    missingPrimitive: normalized.missingPrimitive,
    resizePressure: {
      overflowY: canScrollY,
      hiddenContentHeight: contentHeight != null && viewportHeight != null ? Math.max(0, contentHeight - viewportHeight) : null,
      selectedRowOccluded: scroll.selectedRowVisible === false || scroll.selectedRowAboveFooter === false,
    },
    missingPrimitives: [
      Object.keys(scroll).length === 0 || scroll.scrollTop == null || scroll.viewportHeight == null
        ? isNotesTarget ? "notesScrollMetrics" : "mainListScroll"
        : "",
      stateEnvelope.status === "error" ? "stateResult" : "",
      targetReceipt.classification !== "ok" ? "strictTargetIdentity" : "",
      normalized.missingPrimitive ?? "",
    ].filter(Boolean),
    errors: [targetReceipt, stateEnvelope].filter((value) => value.status === "error"),
    state,
  }, null, 2));
}

if (import.meta.main) {
  await main();
}
