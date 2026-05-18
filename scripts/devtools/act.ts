#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type ActionKind = "set-input" | "select" | "key" | "open-actions";

type Args = {
  actionKind: ActionKind;
  session: string;
  target?: JsonObject;
  text: string;
  semanticId: string;
  key: string;
  modifiers: string[];
  allowSubmit: boolean;
  strict: boolean;
  expectedSurfaceKind: string;
  timeoutMs: number;
  start: boolean;
  show: boolean;
  forwarded: string[];
};

type SubmitLifecycleState =
  | { state: "not-submit"; reason: string }
  | { state: "blocked-before-dispatch"; reason: string; selectedSemanticId?: string | null }
  | { state: "dispatched"; actionId?: string | null }
  | { state: "source-live"; actionId?: string | null }
  | { state: "source-closed-parent-live"; actionId?: string | null; parentTarget?: JsonObject | null }
  | { state: "failed"; reason: string; actionId?: string | null; sourceAfter?: JsonObject | null; parentAfter?: JsonObject | null };

const blockedSubmitKeys = new Set(["enter", "return"]);
const allowedKeys = new Set([
  "arrowup",
  "arrowdown",
  "arrowleft",
  "arrowright",
  "up",
  "down",
  "left",
  "right",
  "tab",
  "escape",
  "esc",
  "enter",
  "return",
  "k",
  "p",
  "w",
]);

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/act.ts set-input --text <value> [target args]",
    "  bun scripts/devtools/act.ts set-input --value <value> [target args]  # alias for --text",
    "  bun scripts/devtools/act.ts select --semantic-id <id> [--allow-submit] [target args]",
    "  bun scripts/devtools/act.ts key --key <name> [--modifiers cmd,shift] [--allow-submit] [target args]",
    "  bun scripts/devtools/act.ts open-actions [target args]",
    "",
    "Target args match scripts/devtools/targets.ts inspect, e.g. --session <name> --main --strict --surface ScriptList.",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }

  const command = argv[0] as ActionKind | undefined;
  if (!command || !["set-input", "select", "key", "open-actions"].includes(command)) {
    console.error(usage());
    process.exit(2);
  }

  const args: Args = {
    actionKind: command,
    session: "default",
    text: "",
    semanticId: "",
    key: "",
    modifiers: [],
    allowSubmit: false,
    strict: false,
    expectedSurfaceKind: "",
    timeoutMs: 8000,
    start: false,
    show: false,
    forwarded: [],
  };

  for (let index = 1; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--session") {
      args.session = argv[++index] ?? args.session;
      args.forwarded.push("--session", args.session);
    } else if (arg === "--target-id") {
      args.target = { type: "id", id: argv[++index] ?? "" };
      args.forwarded.push("--target-id", String(args.target.id ?? ""));
    } else if (arg === "--target-kind") {
      const kind = argv[++index] ?? "main";
      args.target = { type: "kind", kind };
      args.forwarded.push("--target-kind", kind);
    } else if (arg === "--target-index") {
      const value = Number(argv[++index] ?? 0);
      if (!args.target || args.target.type !== "kind") {
        throw new Error("--target-index requires --target-kind first");
      }
      args.target.index = value;
      args.forwarded.push("--target-index", String(value));
    } else if (arg === "--target-title") {
      args.target = { type: "titleContains", text: argv[++index] ?? "" };
      args.forwarded.push("--target-title", String(args.target.text ?? ""));
    } else if (arg === "--focused") {
      args.target = { type: "focused" };
      args.forwarded.push("--focused");
    } else if (arg === "--main") {
      args.target = { type: "main" };
      args.forwarded.push("--main");
    } else if (arg === "--surface") {
      args.expectedSurfaceKind = argv[++index] ?? "";
      args.forwarded.push("--surface", args.expectedSurfaceKind);
    } else if (arg === "--strict") {
      args.strict = true;
      args.forwarded.push("--strict");
    } else if (arg === "--text" || arg === "--value") {
      args.text = argv[++index] ?? "";
    } else if (arg === "--semantic-id") {
      args.semanticId = argv[++index] ?? "";
    } else if (arg === "--key") {
      args.key = argv[++index] ?? "";
    } else if (arg === "--modifiers") {
      args.modifiers = (argv[++index] ?? "")
        .split(",")
        .map((modifier) => modifier.trim().toLowerCase())
        .filter(Boolean);
    } else if (arg === "--allow-submit") {
      args.allowSubmit = true;
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
      args.forwarded.push("--timeout", String(args.timeoutMs));
    } else if (arg === "--start") {
      args.start = true;
      args.forwarded.push("--start");
    } else if (arg === "--show") {
      args.show = true;
      args.forwarded.push("--show");
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
  let parsed: JsonObject | null = null;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    parsed = null;
  }
  if (exitCode !== 0) {
    return {
      status: "error",
      label,
      exitCode,
      stdout: stdout.trim(),
      stderr: stderr.trim(),
      parsedError: parsed,
      lifecycle: parsed?.lifecycle ?? null,
    };
  }
  if (parsed) {
    return parsed;
  }
  return { status: "error", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim(), error: "invalid_json_output" };
}

function requestId(prefix: string) {
  return `devtools-act-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run(["bash", "scripts/agentic/session.sh", "rpc", session, JSON.stringify(payload), "--expect", expect, "--timeout", String(timeoutMs)], String(payload.type ?? "rpc"));
}

async function send(session: string, payload: JsonObject, timeoutMs: number) {
  return run(["bash", "scripts/agentic/session.sh", "send", session, JSON.stringify(payload), "--await-parse", "--timeout", String(timeoutMs)], String(payload.type ?? "send"));
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

function targetSelector(targetReceipt: JsonObject, fallback?: JsonObject) {
  return (targetReceipt.requestedTarget as JsonObject | undefined)?.selector ?? fallback ?? { type: "focused" };
}

async function focusReceipt(args: Args, label: string) {
  return run(["bun", "scripts/devtools/focus.ts", "inspect", ...args.forwarded], label);
}

async function scrollReceipt(args: Args, label: string) {
  return run(["bun", "scripts/devtools/scroll.ts", "inspect", ...args.forwarded], label);
}

function safety(args: Args) {
  const warnings: string[] = [];
  const errors: string[] = [];
  const normalizedKey = args.key.toLowerCase();

  if (args.actionKind === "set-input" && args.text.length === 0) {
    errors.push("set-input requires --text or --value");
  }
  if (args.actionKind === "select" && !args.semanticId) {
    errors.push("select requires --semantic-id");
  }
  if (args.actionKind === "key" && !args.key) {
    errors.push("key requires --key");
  }
  if (args.actionKind === "key" && !allowedKeys.has(normalizedKey)) {
    errors.push(`key '${args.key}' is not in the safe DevTools key allowlist`);
  }
  if (args.actionKind === "key" && blockedSubmitKeys.has(normalizedKey) && !args.allowSubmit) {
    errors.push("submit-like key requires --allow-submit");
  }
  if (args.actionKind === "key" && args.modifiers.includes("cmd") && blockedSubmitKeys.has(normalizedKey) && !args.allowSubmit) {
    errors.push("cmd+enter requires --allow-submit");
  }
  if (args.actionKind === "select" && args.allowSubmit) {
    warnings.push("selection will submit because --allow-submit was passed");
  }
  if (args.actionKind === "key" && (normalizedKey === "escape" || normalizedKey === "esc")) {
    warnings.push("escape can close or dismiss UI; receipt will classify by post-action target state");
  }

  return {
    channel: args.actionKind === "key" ? "simulateKey" : "batch",
    destructive: args.allowSubmit,
    submitAllowed: args.allowSubmit,
    nativeEscalation: false,
    errors,
    warnings,
    submitAttempted: false,
    submitPreflightSelectedSemanticId: null as string | null,
  };
}

function actionPayload(args: Args, selector: JsonObject) {
  if (args.actionKind === "set-input") {
    return {
      type: "batch",
      requestId: requestId("set-input"),
      target: selector,
      commands: [{ type: "setInput", text: args.text }],
      options: { stopOnError: true, rollbackOnError: false, timeout: args.timeoutMs },
      trace: "on",
    };
  }
  if (args.actionKind === "select") {
    return {
      type: "batch",
      requestId: requestId("select"),
      target: selector,
      commands: [{ type: "selectBySemanticId", semanticId: args.semanticId, submit: args.allowSubmit }],
      options: { stopOnError: true, rollbackOnError: false, timeout: args.timeoutMs },
      trace: "on",
    };
  }
  if (args.actionKind === "open-actions") {
    return {
      type: "batch",
      requestId: requestId("open-actions"),
      target: selector,
      commands: [{ type: "openActions" }],
      options: { stopOnError: true, rollbackOnError: false, timeout: args.timeoutMs },
      trace: "on",
    };
  }
  return {
    type: "simulateKey",
    requestId: requestId("key"),
    target: selector,
    key: args.key,
    modifiers: args.modifiers,
  };
}

function expectedResponse(args: Args) {
  return args.actionKind === "key" ? "stdin_command_parsed" : "batchResult";
}

function visibleResult(before: JsonObject, after: JsonObject, beforeScroll: JsonObject, afterScroll: JsonObject) {
  return {
    focusChanged: before.focusedSemanticId !== after.focusedSemanticId,
    selectionChanged: before.selectedSemanticId !== after.selectedSemanticId,
    inputChanged: (before.keyboardOwner as JsonObject | undefined)?.inputValue !== (after.keyboardOwner as JsonObject | undefined)?.inputValue,
    windowVisibleBefore: before.windowVisible ?? null,
    windowVisibleAfter: after.windowVisible ?? null,
    scrollChanged: JSON.stringify(beforeScroll.scroll ?? null) !== JSON.stringify(afterScroll.scroll ?? null),
  };
}

function actionFailed(actionReceipt: JsonObject) {
  if (actionReceipt.status === "error") return true;
  if (actionReceipt.success === false) return true;
  if (actionReceipt.parseOutcome === "timeout") return true;
  if (actionReceipt.parseOutcome === "parseError") return true;
  if (Array.isArray(actionReceipt.results)) {
    return actionReceipt.results.some((result) => {
      return typeof result === "object" && result !== null && (result as JsonObject).success === false;
    });
  }
  return false;
}

const lifecycleCodes = new Set([
  "session_dead",
  "forwarder_dead",
  "no_session",
  "app_process_dead_before_send",
  "app_process_dead_before_rpc",
  "forwarder_dead_before_send",
  "forwarder_dead_before_rpc",
]);

function receiptErrorCode(receipt: JsonObject): string {
  const direct = (receipt.error as JsonObject | undefined)?.code;
  if (typeof direct === "string") return direct;
  const parsed = receipt.parsedError as JsonObject | undefined | null;
  const parsedCode = (parsed?.error as JsonObject | undefined)?.code;
  return typeof parsedCode === "string" ? parsedCode : "";
}

function isLifecycleClassification(receipt: JsonObject) {
  return receipt.classification === "blocked-by-session-lifecycle" || lifecycleCodes.has(receiptErrorCode(receipt));
}

function isSubmitLike(args: Args) {
  const normalizedKey = args.key.toLowerCase();
  return args.allowSubmit && (
    (args.actionKind === "key" && blockedSubmitKeys.has(normalizedKey))
    || (args.actionKind === "select")
  );
}

function selectedActionIdFromSemanticId(value: unknown) {
  if (typeof value !== "string") return null;
  const match = value.match(/^choice:\d+:(.+)$/);
  return match?.[1] ?? null;
}

function isActionsDialogTargetReceipt(receipt: JsonObject) {
  const resolved = receipt.resolvedTarget as JsonObject | undefined;
  return resolved?.targetKind === "ActionsDialog"
    || resolved?.surfaceKind === "ActionsDialog"
    || resolved?.appViewVariant === "ActionsDialog";
}

function submitPreflight(args: Args, targetReceipt: JsonObject, before: JsonObject): SubmitLifecycleState {
  if (!isSubmitLike(args)) {
    return { state: "not-submit", reason: "action is not submit-like" };
  }
  const selectedSemanticId = before.selectedSemanticId ?? null;
  if (!isActionsDialogTargetReceipt(targetReceipt)) {
    return { state: "blocked-before-dispatch", reason: "submit requires ActionsDialog target", selectedSemanticId: selectedSemanticId as string | null };
  }
  const actionId = selectedActionIdFromSemanticId(selectedSemanticId);
  if (!actionId) {
    return { state: "blocked-before-dispatch", reason: "submit requires selected ActionsDialog choice:* row", selectedSemanticId: selectedSemanticId as string | null };
  }
  return { state: "dispatched", actionId };
}

async function inspectParentAfterSubmit(args: Args, targetReceipt: JsonObject) {
  const resolved = targetReceipt.resolvedTarget as JsonObject | undefined;
  const parentAutomationId = resolved?.parentAutomationId;
  const parentArgs = parentAutomationId
    ? ["--target-id", String(parentAutomationId)]
    : ["--main", "--surface", "ScriptList"];
  return run([
    "bun",
    "scripts/devtools/targets.ts",
    "inspect",
    "--session",
    args.session,
    "--strict",
    "--timeout",
    String(args.timeoutMs),
    ...parentArgs,
  ], "targets.inspect.parent-after-submit");
}

function resolveSubmitLifecycleAfterAction(
  preflight: SubmitLifecycleState,
  sourceAfter: JsonObject,
  parentAfter: JsonObject | null,
): SubmitLifecycleState {
  if (preflight.state !== "dispatched") return preflight;
  if (sourceAfter.classification === "ok") {
    return { state: "source-live", actionId: preflight.actionId };
  }
  if (parentAfter?.classification === "ok") {
    return {
      state: "source-closed-parent-live",
      actionId: preflight.actionId,
      parentTarget: parentAfter.resolvedTarget as JsonObject | undefined ?? null,
    };
  }
  const sourceLifecycle = isLifecycleClassification(sourceAfter);
  const parentLifecycle = parentAfter ? isLifecycleClassification(parentAfter) : false;
  return {
    state: "failed",
    reason: sourceLifecycle || parentLifecycle ? "blocked-by-session-lifecycle" : "post-submit target not inspectable",
    actionId: preflight.actionId,
    sourceAfter,
    parentAfter,
  };
}

function classify(
  targetReceipt: JsonObject,
  guard: ReturnType<typeof safety>,
  actionReceipt: JsonObject,
  after: JsonObject,
  submitLifecycle: SubmitLifecycleState,
) {
  if (guard.errors.length > 0) {
    return "blocked-by-unsafe-operation";
  }
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (submitLifecycle.state === "blocked-before-dispatch") {
    return "blocked-by-unsafe-operation";
  }
  if (submitLifecycle.state === "source-live" || submitLifecycle.state === "source-closed-parent-live") {
    return "ok";
  }
  if (submitLifecycle.state === "failed" && submitLifecycle.reason === "blocked-by-session-lifecycle") {
    return "blocked-by-session-lifecycle";
  }
  if (isLifecycleClassification(actionReceipt) || isLifecycleClassification(after)) {
    return "blocked-by-session-lifecycle";
  }
  if (actionFailed(actionReceipt)) {
    return "blocked-by-timeout";
  }
  if (after.classification && after.classification !== "ok") {
    return after.classification;
  }
  return "ok";
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const targetReceipt = await run(["bun", "scripts/devtools/targets.ts", "inspect", ...args.forwarded], "targets.inspect");
  const selector = targetSelector(targetReceipt, args.target);
  const guard = safety(args);

  const before = await focusReceipt(args, "focus.before");
  const beforeScroll = await scrollReceipt(args, "scroll.before");
  const startedAt = new Date().toISOString();
  const preflight = submitPreflight(args, targetReceipt, before);
  const guardWithPreflight = {
    ...guard,
    submitAttempted: isSubmitLike(args) && preflight.state !== "blocked-before-dispatch",
    submitPreflightSelectedSemanticId: (before.selectedSemanticId as string | undefined) ?? null,
    errors: preflight.state === "blocked-before-dispatch"
      ? [...guard.errors, preflight.reason]
      : guard.errors,
  };

  let actionEnvelope: JsonObject = { status: "blocked", reason: "blocked-by-unsafe-operation" };
  if (guardWithPreflight.errors.length === 0 && targetReceipt.classification === "ok") {
    actionEnvelope = args.actionKind === "key"
      ? await send(args.session, actionPayload(args, selector), args.timeoutMs)
      : await rpc(args.session, actionPayload(args, selector), expectedResponse(args), args.timeoutMs);
  }

  const after = await focusReceipt(args, "focus.after");
  const afterScroll = await scrollReceipt(args, "scroll.after");
  const endedAt = new Date().toISOString();
  const actionReceipt = responseOf(actionEnvelope);
  const parentAfterSubmit = isSubmitLike(args) ? await inspectParentAfterSubmit(args, targetReceipt) : null;
  const submitLifecycle = resolveSubmitLifecycleAfterAction(preflight, after, parentAfterSubmit);
  const classification = classify(targetReceipt, guardWithPreflight, actionReceipt, after, submitLifecycle);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.act",
    command: `act.${args.actionKind}`,
    classification,
    session: args.session,
    startedAt,
    endedAt,
    actionId: requestId(args.actionKind),
    actionKind: args.actionKind,
    requestedTarget: targetReceipt.requestedTarget ?? { selector },
    targetBefore: before.target ?? targetReceipt.resolvedTarget ?? null,
    input: {
      text: args.actionKind === "set-input" ? args.text : null,
      semanticId: args.actionKind === "select" ? args.semanticId : null,
      key: args.actionKind === "key" ? args.key : null,
      modifiers: args.actionKind === "key" ? args.modifiers : [],
    },
    safety: guardWithPreflight,
    submitLifecycle,
    expected: {
      protocolResponse: expectedResponse(args),
      submitAllowed: args.allowSubmit,
      noNativeEscalation: true,
      prePostReceipts: ["focus.inspect", "scroll.inspect"],
    },
    actionReceipt,
    targetAfter: after.target ?? null,
    visibleResult: visibleResult(before, after, beforeScroll, afterScroll),
    before: { focus: before, scroll: beforeScroll },
    after: { focus: after, scroll: afterScroll },
    warnings: [
      ...guardWithPreflight.warnings,
      ...(Array.isArray(before.warnings) ? before.warnings : []),
      ...(Array.isArray(after.warnings) ? after.warnings : []),
    ],
    errors: [
      ...guardWithPreflight.errors.map((error) => ({ error })),
      targetReceipt.classification !== "ok" ? targetReceipt : null,
      actionFailed(actionReceipt) ? actionReceipt : null,
    ].filter(Boolean),
  }, null, 2));
}

await main();
