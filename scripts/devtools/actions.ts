#!/usr/bin/env bun

import { classifyTransportError } from "./lib/transport-errors.ts";

type JsonObject = Record<string, unknown>;
type Rect = { x: number; y: number; width: number; height: number };

type Args = {
  session: string;
  target?: JsonObject;
  openTarget?: JsonObject;
  timeoutMs: number;
  open: boolean;
  start: boolean;
  keepOpen: boolean;
  inspectTargetForwarded: string[];
  hasExplicitInspectTarget: boolean;
  openTargetForwarded: string[];
  hasExplicitOpenTarget: boolean;
};

const DEFAULT_INSPECT_TARGET = ["--target-kind", "actionsDialog", "--strict", "--surface", "ActionsDialog"];
const DEFAULT_OPEN_TARGET = ["--show", "--main", "--strict", "--surface", "ScriptList"];

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/actions.ts inspect [--session <name>] [--start] [--keep-open] [--open] [--open-target-kind <kind>] [target args]",
    "",
    "Target args match scripts/devtools/targets.ts inspect. Defaults to --target-kind actionsDialog --strict --surface ActionsDialog.",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }
  if (argv[0] !== "inspect") {
    console.error(usage());
    process.exit(2);
  }

  const args: Args = {
    session: "default",
    timeoutMs: 8000,
    open: false,
    start: false,
    keepOpen: false,
    inspectTargetForwarded: [],
    hasExplicitInspectTarget: false,
    openTargetForwarded: [],
    hasExplicitOpenTarget: false,
  };
  for (let index = 1; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--session") {
      args.session = argv[++index] ?? args.session;
    } else if (arg === "--open") {
      args.open = true;
    } else if (arg === "--start") {
      args.start = true;
    } else if (arg === "--keep-open") {
      args.keepOpen = true;
    } else if (arg === "--open-target-id") {
      args.openTarget = { type: "id", id: argv[++index] ?? "" };
      args.hasExplicitOpenTarget = true;
      args.openTargetForwarded.push("--target-id", String(args.openTarget.id ?? ""));
    } else if (arg === "--open-target-kind") {
      const kind = argv[++index] ?? "main";
      args.openTarget = { type: "kind", kind };
      args.hasExplicitOpenTarget = true;
      args.openTargetForwarded.push("--target-kind", kind);
    } else if (arg === "--open-target-index") {
      const value = Number(argv[++index] ?? 0);
      if (!args.openTarget || args.openTarget.type !== "kind") {
        throw new Error("--open-target-index requires --open-target-kind first");
      }
      args.openTarget.index = value;
      args.hasExplicitOpenTarget = true;
      args.openTargetForwarded.push("--target-index", String(value));
    } else if (arg === "--open-target-title") {
      args.openTarget = { type: "titleContains", text: argv[++index] ?? "" };
      args.hasExplicitOpenTarget = true;
      args.openTargetForwarded.push("--target-title", String(args.openTarget.text ?? ""));
    } else if (arg === "--open-focused") {
      args.openTarget = { type: "focused" };
      args.hasExplicitOpenTarget = true;
      args.openTargetForwarded.push("--focused");
    } else if (arg === "--open-main") {
      args.openTarget = { type: "main" };
      args.hasExplicitOpenTarget = true;
      args.openTargetForwarded.push("--main");
    } else if (arg === "--target-id") {
      args.target = { type: "id", id: argv[++index] ?? "" };
      args.hasExplicitInspectTarget = true;
      args.inspectTargetForwarded.push("--target-id", String(args.target.id ?? ""));
    } else if (arg === "--target-kind") {
      const kind = argv[++index] ?? "actionsDialog";
      args.target = { type: "kind", kind };
      args.hasExplicitInspectTarget = true;
      args.inspectTargetForwarded.push("--target-kind", kind);
    } else if (arg === "--target-index") {
      const value = Number(argv[++index] ?? 0);
      if (!args.target || args.target.type !== "kind") {
        throw new Error("--target-index requires --target-kind first");
      }
      args.target.index = value;
      args.hasExplicitInspectTarget = true;
      args.inspectTargetForwarded.push("--target-index", String(value));
    } else if (arg === "--target-title") {
      args.target = { type: "titleContains", text: argv[++index] ?? "" };
      args.hasExplicitInspectTarget = true;
      args.inspectTargetForwarded.push("--target-title", String(args.target.text ?? ""));
    } else if (arg === "--focused") {
      args.target = { type: "focused" };
      args.hasExplicitInspectTarget = true;
      args.inspectTargetForwarded.push("--focused");
    } else if (arg === "--main") {
      args.target = { type: "main" };
      args.hasExplicitInspectTarget = true;
      args.inspectTargetForwarded.push("--main");
    } else if (arg === "--strict") {
      args.hasExplicitInspectTarget = true;
      args.inspectTargetForwarded.push("--strict");
    } else if (arg === "--surface") {
      args.hasExplicitInspectTarget = true;
      args.inspectTargetForwarded.push("--surface", argv[++index] ?? "");
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
    }
  }
  return args;
}

function inspectForwarded(args: Args): string[] {
  return [
    "--session",
    args.session,
    ...(args.hasExplicitInspectTarget ? args.inspectTargetForwarded : DEFAULT_INSPECT_TARGET),
    "--timeout",
    String(args.timeoutMs),
  ];
}

function openForwarded(args: Args): string[] {
  return [
    "--session",
    args.session,
    ...(args.hasExplicitOpenTarget ? args.openTargetForwarded : DEFAULT_OPEN_TARGET),
    "--timeout",
    String(args.timeoutMs),
  ];
}

async function run(command: string[], label: string, env?: Record<string, string>): Promise<JsonObject> {
  const proc = Bun.spawn(command, { stdout: "pipe", stderr: "pipe", env: env ? { ...process.env, ...env } : process.env });
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
  return `devtools-actions-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run(["bash", "scripts/agentic/session.sh", "rpc", session, JSON.stringify(payload), "--expect", expect, "--timeout", String(timeoutMs)], String(payload.type ?? "rpc"));
}

async function maybeStartSession(args: Args) {
  if (!args.start) return null;
  const env = args.keepOpen ? { SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN: "1" } : undefined;
  return run(["bash", "scripts/agentic/session.sh", "start", args.session], "session.start", env);
}

async function maybeOpenParentTarget(args: Args) {
  const normalizedOpenKind =
    typeof args.openTarget?.kind === "string" ? args.openTarget.kind.toLowerCase() : "";
  if (!args.open || normalizedOpenKind !== "notes") return null;
  const receipt = await run([
    "bash",
    "scripts/agentic/session.sh",
    "send",
    args.session,
    JSON.stringify({ type: "openNotes", requestId: requestId("open-notes-parent") }),
    "--await-parse",
    "--timeout",
    String(args.timeoutMs),
  ], "parent.open.notes");
  await Bun.sleep(300);
  return receipt;
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

function asArray(value: unknown): JsonObject[] {
  return Array.isArray(value) ? value.filter((entry): entry is JsonObject => typeof entry === "object" && entry !== null) : [];
}

function asNumber(value: unknown, fallback = 0) {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function rectFrom(value: unknown): Rect | null {
  if (!value || typeof value !== "object") return null;
  const object = value as JsonObject;
  return {
    x: asNumber(object.x),
    y: asNumber(object.y),
    width: asNumber(object.width),
    height: asNumber(object.height),
  };
}

function edgeClipping(child: Rect | null, parent: Rect | null) {
  if (!child || !parent) {
    return { available: false, top: null, right: null, bottom: null, left: null };
  }
  return {
    available: true,
    top: child.y < parent.y,
    right: child.x + child.width > parent.x + parent.width,
    bottom: child.y + child.height > parent.y + parent.height,
    left: child.x < parent.x,
  };
}

function shortcutTokens(shortcut: unknown) {
  if (typeof shortcut !== "string" || shortcut.trim().length === 0) return [];
  return shortcut
    .split(/(?=[⌘⇧⌃⌥↑↓←→↵⌫⇥⎋␣])|\+/)
    .map((token) => token.trim())
    .filter(Boolean);
}

function visibleActionRows(actionsDialog: JsonObject | null) {
  const actions = (actionsDialog?.actions as JsonObject | undefined) ?? {};
  const rows = asArray(actionsDialog?.visibleActions).length > 0
    ? asArray(actionsDialog?.visibleActions)
    : asArray(actions.visibleSample);
  return rows.map((action, index) => {
    const shortcut = action.shortcut ?? null;
    return {
      index,
      id: action.id ?? null,
      label: action.label ?? action.title ?? null,
      section: action.section ?? null,
      shortcut,
      shortcutTokens: shortcutTokens(shortcut),
      destructive: action.destructive ?? false,
      enabled: action.enabled ?? null,
      disabledReason: action.actionDisabled ?? action.disabledReason ?? null,
    };
  });
}

function groupSections(rows: ReturnType<typeof visibleActionRows>) {
  const sections = new Map<string, { title: string; rowCount: number; firstIndex: number; lastIndex: number }>();
  rows.forEach((row, index) => {
    const title = typeof row.section === "string" && row.section ? row.section : "default";
    const existing = sections.get(title);
    if (existing) {
      existing.rowCount += 1;
      existing.lastIndex = index;
    } else {
      sections.set(title, { title, rowCount: 1, firstIndex: index, lastIndex: index });
    }
  });
  return [...sections.values()];
}

function findParentTarget(target: JsonObject, windows: JsonObject[]) {
  const parentId = target.parentAutomationId ?? target.openerAutomationId;
  return windows.find((window) => window.automationId === parentId) ?? null;
}

async function maybeOpenActions(args: Args) {
  if (!args.open) return null;
  const openCommand = [
      "bun",
      "scripts/devtools/act.ts",
      "open-actions",
      ...openForwarded(args),
    ];
  return run([
    ...openCommand,
  ], "actions.open");
}

function isActionsDialogTarget(receipt: JsonObject) {
  const target = receipt.resolvedTarget as JsonObject | undefined;
  return receipt.classification === "ok"
    && target?.automationId === "actions-dialog"
    && target?.targetKind === "ActionsDialog";
}

async function waitForActionsDialogTarget(args: Args, forwarded: string[]) {
  const startedAt = Date.now();
  const deadline = startedAt + args.timeoutMs;
  const attempts: JsonObject[] = [];

  while (Date.now() < deadline) {
    const receipt = await run(
      ["bun", "scripts/devtools/targets.ts", "inspect", ...forwarded],
      "targets.inspect.actionsDialog.ready",
    );
    attempts.push({
      elapsedMs: Date.now() - startedAt,
      classification: receipt.classification ?? null,
      requestedTarget: receipt.requestedTarget ?? null,
      resolvedTarget: receipt.resolvedTarget ?? null,
      errors: receipt.errors ?? [],
    });
    if (isActionsDialogTarget(receipt)) {
      return { status: "ok", receipt, attempts };
    }
    await Bun.sleep(50);
  }

  return {
    status: "error",
    classification: "blocked-by-target-ambiguity",
    reason: "actionsDialog target did not become observable after openActions",
    attempts,
  };
}

function classify(targetReceipt: JsonObject, stateEnvelope: JsonObject, missing: string[]) {
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (stateEnvelope.status === "error") {
    return classifyTransportError(stateEnvelope);
  }
  if (missing.length > 0) {
    return "blocked-by-missing-primitive";
  }
  return "ok";
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const forwarded = inspectForwarded(args);
  const startReceipt = await maybeStartSession(args);
  const parentOpenReceipt = await maybeOpenParentTarget(args);
  const openReceipt = await maybeOpenActions(args);
  const targetReadiness = args.open
    ? await waitForActionsDialogTarget(args, forwarded)
    : null;
  const targetReceipt = targetReadiness?.status === "ok"
    ? (targetReadiness.receipt as JsonObject)
    : await run(["bun", "scripts/devtools/targets.ts", "inspect", ...forwarded], "targets.inspect");

  const targetsList = await run(
    ["bun", "scripts/devtools/targets.ts", "list", "--session", args.session, "--timeout", String(args.timeoutMs)],
    "targets.list",
  );
  const selector = (targetReceipt.requestedTarget as JsonObject | undefined)?.selector ?? args.target ?? { type: "kind", kind: "actionsDialog" };
  const stateEnvelope = await rpc(
    args.session,
    { type: "getState", requestId: requestId("state"), target: selector, summaryOnly: true },
    "stateResult",
    args.timeoutMs,
  );
  const elements = await run(["bun", "scripts/devtools/elements.ts", "snapshot", ...forwarded], "elements.snapshot");
  const layout = await run(["bun", "scripts/devtools/layout.ts", "measure", ...forwarded], "layout.measure");
  const keyboard = await run(["bun", "scripts/devtools/keyboard.ts", "inspect", ...forwarded], "keyboard.inspect");

  const state = responseOf(stateEnvelope);
  const actionsDialog = (state.actionsDialog as JsonObject | undefined) ?? null;
  const attachedPopup = (actionsDialog?.attachedPopup as JsonObject | undefined) ?? null;
  const attachedGeometry = (attachedPopup?.geometry as JsonObject | undefined) ?? {};
  const rowGeometry = (actionsDialog?.rowGeometry as JsonObject | undefined) ?? null;
  const runtimeAudit = (actionsDialog?.runtimeAudit as JsonObject | undefined) ?? null;
  const runtimeAuditViolations = asArray(actionsDialog?.runtimeAuditViolations);
  const runtimeAuditStatus = typeof actionsDialog?.runtimeAuditStatus === "string"
    ? actionsDialog.runtimeAuditStatus
    : runtimeAudit
      ? runtimeAuditViolations.length === 0 ? "ok" : "violation"
      : "unavailable";
  const target = (targetReceipt.resolvedTarget as JsonObject | undefined) ?? {};
  const windows = asArray(targetsList.targets ?? targetsList.windows);
  const parent = findParentTarget(target, windows);
  const popupRect = rectFrom(attachedGeometry.popupRect) ?? rectFrom(target.bounds);
  const parentRect = rectFrom(attachedGeometry.parentRect) ?? rectFrom(parent?.bounds);
  const anchorRect = rectFrom(attachedGeometry.anchorRect);
  const rows = visibleActionRows(actionsDialog);
  const sections = groupSections(rows);
  const clipping = edgeClipping(popupRect, parentRect);
  const dialogRoute = (actionsDialog?.route as JsonObject | undefined) ?? {};
  const targetRouteStack = asArray(target.routeStack);
  const routeStack = targetRouteStack.length > 0 ? targetRouteStack : asArray(dialogRoute.stack);
  const routeStateAvailable = Boolean(actionsDialog && Object.prototype.hasOwnProperty.call(actionsDialog, "route"));
  const shortcutRows = rows.filter((row) => row.shortcut);
  const runtimeShortcutLayout = (rowGeometry?.shortcutLayout as JsonObject | undefined) ?? null;
  const destructiveRows = rows.filter((row) => row.destructive);
  const disabledRows = rows.filter((row) => row.enabled === false || row.disabledReason);
  const disabledReasonBoundsRequired = disabledRows.length > 0;
  const sectionBoundsAvailable = rowGeometry?.sectionBoundsAvailable === true;
  const hoverRowAvailable = rowGeometry?.hoverRowAvailable === true;
  const shortcutBoundsAvailable = rowGeometry?.shortcutBoundsAvailable === true;
  const disabledReasonBoundsAvailable = rowGeometry?.disabledReasonBoundsAvailable === true;
  const missing = [
    actionsDialog ? "" : "actionsDialogState",
    routeStateAvailable ? "" : "route stack",
    popupRect ? "" : "popup rect",
    parentRect ? "" : "parent target rect",
    anchorRect ? "" : "anchor rect",
    sectionBoundsAvailable ? "" : "section bounds",
    hoverRowAvailable ? "" : "hover row",
    shortcutBoundsAvailable ? "" : "shortcut layout bounds",
    disabledReasonBoundsRequired && !disabledReasonBoundsAvailable ? "disabled reason bounds" : "",
  ].filter(Boolean);
  const classification = classify(targetReceipt, stateEnvelope, missing);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.actions",
    command: "actions.inspect",
    classification,
    session: args.session,
    requestedTarget: targetReceipt.requestedTarget ?? { selector },
    target,
    parentTarget: parent,
    startReceipt,
    parentOpenReceipt,
    openReceipt,
    targetReadiness,
    popupState: {
      open: Boolean(actionsDialog),
      host: actionsDialog?.host ?? null,
      contextTitle: actionsDialog?.contextTitle ?? null,
      contextStableKey: actionsDialog?.contextStableKey ?? null,
      contextSource: actionsDialog?.contextSource ?? null,
      selectedActionId: ((actionsDialog?.selection as JsonObject | undefined)?.actionId ?? actionsDialog?.selectedActionId) ?? null,
    },
    attachedPopup,
    routeStack,
    routeId: target.routeId ?? dialogRoute.currentRouteId ?? null,
    rowGeometry,
    chromeContract: {
      source: "actionsDialog.automationState.runtimeAudit",
      status: runtimeAuditStatus,
      audit: runtimeAudit,
      violations: runtimeAuditViolations,
    },
    geometry: {
      layoutPrimitive: "getLayoutInfo(actionsDialog)",
      popupRect,
      parentRect,
      anchorRect,
      clippingEdges: clipping,
      placement: {
        parentAutomationId: attachedPopup?.parentAutomationId ?? target.parentAutomationId ?? null,
        parentWindowId: target.parentWindowId ?? attachedPopup?.parentAutomationId ?? target.parentAutomationId ?? null,
        openerAutomationId: target.openerAutomationId ?? null,
        screenId: target.screenId ?? attachedGeometry.displayId ?? attachedPopup?.displayId ?? null,
        pinnedEdge: attachedGeometry.pinnedEdge ?? null,
        position: attachedGeometry.position ?? attachedPopup?.position ?? null,
        generation: attachedPopup?.generation ?? null,
        stale: attachedPopup?.stale ?? null,
      },
      layoutPressure: (layout.resizePressure as JsonObject | undefined) ?? null,
      overlapPairs: layout.overlaps ?? [],
    },
    sections: {
      count: sectionBoundsAvailable ? asArray(rowGeometry?.sections).length : sections.length,
      rows: sectionBoundsAvailable ? asArray(rowGeometry?.sections) : sections,
      boundsAvailable: sectionBoundsAvailable,
    },
    actions: {
      visibleCount: rows.length,
      rows,
      destructiveRows,
      disabledRows,
    },
    shortcutLayout: {
      primitive: "runtime shortcut layout bounds",
      rowCount: shortcutBoundsAvailable ? asArray(runtimeShortcutLayout?.rows).length : shortcutRows.length,
      rows: shortcutBoundsAvailable ? asArray(runtimeShortcutLayout?.rows) : shortcutRows.map((row) => ({
        id: row.id,
        label: row.label,
        shortcut: row.shortcut,
        shortcutTokens: row.shortcutTokens,
        tokenCount: row.shortcutTokens.length,
        bounds: null,
      })),
      boundsAvailable: shortcutBoundsAvailable,
      runtimeBoundsAvailable: shortcutBoundsAvailable,
      measurementSource: runtimeShortcutLayout?.measurementSource ?? null,
      stopReason: runtimeShortcutLayout?.stopReason ?? (shortcutBoundsAvailable ? null : "runtime shortcut layout unavailable"),
    },
    receipts: {
      target: targetReceipt,
      state: stateEnvelope,
      elements,
      layout,
      keyboard,
    },
    missingPrimitives: missing,
    recommendedNext: [
      disabledReasonBoundsRequired && !disabledReasonBoundsAvailable
        ? "Expose disabled reason text bounds when a visible ActionsDialog route renders disabled explanations."
        : "",
      "Add safe click/hover receipts for action rows before native pointer escalation.",
    ].filter(Boolean),
    proofMode: {
      keepOpen: args.keepOpen,
      keepOpenEnv: args.keepOpen ? "SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN=1" : null,
      openTarget: args.openTarget ?? null,
    },
    warnings: [
      missing.length > 0 ? "Actions popup inspection is fail-closed until anchor/section/shortcut geometry is first-class." : "",
      clipping.available && (clipping.top || clipping.right || clipping.bottom || clipping.left) ? "popup rect clips outside parent rect" : "",
    ].filter(Boolean),
    errors: [targetReceipt, stateEnvelope, elements, layout, keyboard].filter((receipt) => receipt.status === "error"),
  }, null, 2));
}

await main();
