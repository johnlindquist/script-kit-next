#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Args = {
  command: "list" | "inspect";
  session: string;
  target?: JsonObject;
  strict: boolean;
  expectedSurfaceKind: string;
  timeoutMs: number;
  hiDpi: boolean;
  start: boolean;
  show: boolean;
};

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/targets.ts list [--session <name>] [--start] [--show]",
    "  bun scripts/devtools/targets.ts inspect --target-id <id>|--target-kind <kind>|--main|--focused [--surface <SurfaceKind>] [--strict]",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  const args: Args = {
    command: argv[0] === "inspect" ? "inspect" : "list",
    session: "default",
    strict: false,
    expectedSurfaceKind: "",
    timeoutMs: 8000,
    hiDpi: false,
    start: false,
    show: false,
  };

  const startIndex = argv[0] === "inspect" || argv[0] === "list" ? 1 : 0;
  for (let index = startIndex; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--session") {
      args.session = argv[++index] ?? args.session;
    } else if (arg === "--target-id") {
      args.target = { type: "id", id: argv[++index] ?? "" };
    } else if (arg === "--target-kind") {
      const kind = argv[++index] ?? "main";
      args.target = { type: "kind", kind };
    } else if (arg === "--target-index") {
      if (!args.target || args.target.type !== "kind") {
        throw new Error("--target-index requires --target-kind first");
      }
      args.target.index = Number(argv[++index] ?? 0);
    } else if (arg === "--target-title") {
      args.target = { type: "titleContains", text: argv[++index] ?? "" };
    } else if (arg === "--focused") {
      args.target = { type: "focused" };
    } else if (arg === "--target-json") {
      try {
        args.target = JSON.parse(argv[++index] ?? "{}");
      } catch (error) {
        throw new Error(`Invalid --target-json: ${error}`);
      }
    } else if (arg === "--main") {
      args.target = { type: "main" };
    } else if (arg === "--surface") {
      args.expectedSurfaceKind = argv[++index] ?? "";
    } else if (arg === "--strict") {
      args.strict = true;
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
    } else if (arg === "--hi-dpi") {
      args.hiDpi = true;
    } else if (arg === "--start") {
      args.start = true;
    } else if (arg === "--show") {
      args.show = true;
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
  return `devtools-targets-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run(
    [
      "bash",
      "scripts/agentic/session.sh",
      "rpc",
      session,
      JSON.stringify(payload),
      "--expect",
      expect,
      "--timeout",
      String(timeoutMs),
    ],
    String(payload.type ?? "rpc"),
  );
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

function arrayOf(value: unknown): JsonObject[] {
  return Array.isArray(value) ? value.filter((entry): entry is JsonObject => typeof entry === "object" && entry !== null) : [];
}

function stableWindowKind(value: unknown) {
  if (value === "actionsDialog") return "ActionsDialog";
  if (value === "promptPopup") return "PromptPopup";
  if (value === "acpDetached") return "AcpDetached";
  if (value === "dictation") return "Dictation";
  if (value === "miniAi") return "MiniAi";
  if (value === "ai") return "Ai";
  if (value === "main") return "Main";
  if (value === "notes") return "Notes";
  return value ?? null;
}

function pickWindows(windows: JsonObject) {
  return arrayOf(windows.windows ?? windows.automationWindows ?? windows.targets).map((window, index) => ({
    index,
    automationId: window.id ?? window.windowId ?? window.automationId ?? null,
    windowKind: stableWindowKind(window.kind ?? window.windowKind),
    title: window.title ?? null,
    visible: window.visible ?? null,
    focused: window.focused ?? null,
    bounds: window.bounds ?? window.resolvedBounds ?? null,
    surfaceKind: window.surfaceKind ?? null,
    semanticSurface: window.semanticSurface ?? null,
    appViewVariant: window.appViewVariant ?? null,
    parentAutomationId: window.parentAutomationId ?? window.parentWindowId ?? null,
    parentKind: window.parentKind ?? null,
    pid: window.pid ?? null,
  }));
}

type SurfaceCandidate = { field: string; value: string };
type ActualSurfaceCandidate = {
  automationId: unknown;
  windowKind: unknown;
  surfaceKind: unknown;
  semanticSurface: unknown;
  appViewVariant: unknown;
};

function surfaceCandidates(snapshot: JsonObject, listedWindow: JsonObject): SurfaceCandidate[] {
  return [
    ["snapshot.windowKind", snapshot.windowKind],
    ["snapshot.kind", snapshot.kind],
    ["snapshot.surfaceKind", snapshot.surfaceKind],
    ["snapshot.semanticSurface", snapshot.semanticSurface],
    ["snapshot.appViewVariant", snapshot.appViewVariant],
    ["snapshot.surfaceContract.surfaceKind", (snapshot.surfaceContract as JsonObject | undefined)?.surfaceKind],
    ["snapshot.state.surfaceKind", (snapshot.state as JsonObject | undefined)?.surfaceKind],
    ["listedWindow.windowKind", listedWindow.windowKind],
    ["listedWindow.semanticSurface", listedWindow.semanticSurface],
    ["listedWindow.surfaceKind", listedWindow.surfaceKind],
    ["listedWindow.appViewVariant", listedWindow.appViewVariant],
  ]
    .filter((entry): entry is [string, string] => typeof entry[1] === "string" && entry[1].length > 0)
    .map(([field, value]) => ({ field, value }));
}

function acceptedSurfaceValues(expectedSurfaceKind: string): Set<string> {
  const values = new Set<string>([expectedSurfaceKind]);
  // ACP detached windows expose their UI contract through automation
  // semanticSurface while their window kind remains AcpDetached.
  if (expectedSurfaceKind === "AcpChat") {
    values.add("acpChat");
  }
  return values;
}

function surfaceMatch(snapshot: JsonObject, listedWindow: JsonObject, expectedSurfaceKind: string) {
  if (!expectedSurfaceKind) {
    return {
      ok: true,
      expectedSurfaceKind: null,
      acceptedValues: [] as string[],
      matchedCandidate: null as SurfaceCandidate | null,
      candidates: [] as SurfaceCandidate[],
      actualValues: [] as string[],
      mismatchReason: null,
    };
  }
  const candidates = surfaceCandidates(snapshot, listedWindow);
  const actualValues = [...new Set(candidates.map((candidate) => candidate.value))];
  const acceptedValues = acceptedSurfaceValues(expectedSurfaceKind);
  const matchedCandidate = candidates.find((candidate) => acceptedValues.has(candidate.value)) ?? null;
  const ok = matchedCandidate != null;
  return {
    ok,
    expectedSurfaceKind,
    acceptedValues: [...acceptedValues],
    matchedCandidate,
    candidates,
    actualValues,
    mismatchReason: ok ? null : "expected-surface-not-found",
  };
}

function actualSurfaceCandidate(windowId: unknown, listedWindow: JsonObject): ActualSurfaceCandidate {
  return {
    automationId: windowId,
    windowKind: listedWindow.windowKind ?? null,
    surfaceKind: listedWindow.surfaceKind ?? null,
    semanticSurface: listedWindow.semanticSurface ?? null,
    appViewVariant: listedWindow.appViewVariant ?? null,
  };
}

function targetIdentity(args: Args, inspect: JsonObject, windows: JsonObject) {
  const snapshot = (inspect.snapshot as JsonObject | undefined) ?? inspect;
  const resolvedBounds = snapshot.resolvedBounds ?? snapshot.bounds ?? null;
  const windowId = snapshot.windowId ?? snapshot.id ?? null;
  const listedWindow = pickWindows(windows).find((window) => window.automationId === windowId) ?? {};
  const match = surfaceMatch(snapshot, listedWindow, args.expectedSurfaceKind);
  const strictTargetMatch = Boolean(windowId) && match.ok;
  const ambiguity = args.strict && !windowId ? pickWindows(windows) : [];

  return {
    requestedTarget: {
      selector: args.target ?? { type: "focused" },
      strict: args.strict,
      expectedSurfaceKind: args.expectedSurfaceKind || null,
    },
    resolvedTarget: {
      automationId: windowId,
      stableTargetId: windowId,
      targetKind: snapshot.windowKind ?? snapshot.kind ?? null,
      hostKind: snapshot.hostKind ?? null,
      parentAutomationId: snapshot.parentAutomationId ?? snapshot.parentWindowId ?? listedWindow.parentAutomationId ?? null,
      openerAutomationId: snapshot.openerAutomationId ?? null,
      surfaceKind: snapshot.surfaceKind ?? null,
      semanticSurface: snapshot.semanticSurface ?? listedWindow.semanticSurface ?? null,
      appViewVariant: snapshot.appViewVariant ?? null,
      nativeFooterSurface: snapshot.nativeFooterSurface ?? null,
      surfaceFamily: snapshot.surfaceFamily ?? null,
      routeId: snapshot.routeId ?? null,
      routeStack: snapshot.routeStack ?? [],
      targetGeneration: snapshot.targetGeneration ?? null,
      surfaceGeneration: snapshot.surfaceGeneration ?? null,
      dataGeneration: snapshot.dataGeneration ?? null,
      bounds: resolvedBounds,
      screenId: snapshot.screenId ?? null,
      zOrder: snapshot.zOrder ?? null,
      visible: snapshot.visible ?? null,
      frontmost: snapshot.frontmost ?? null,
      focused: snapshot.focused ?? null,
      screenshotIdentity: {
        width: snapshot.screenshotWidth ?? snapshot.screenshot_width ?? null,
        height: snapshot.screenshotHeight ?? snapshot.screenshot_height ?? null,
        targetBoundsInScreenshot: snapshot.targetBoundsInScreenshot ?? null,
        nonBlankRatio: snapshot.nonBlankRatio ?? null,
      },
      pid: snapshot.pid ?? listedWindow.pid ?? null,
      strictTargetMatch,
      strictTargetMismatch: args.strict && !strictTargetMatch
        ? {
          expectedSurfaceKind: args.expectedSurfaceKind || null,
          automationId: windowId,
          surfaceCandidates: match.candidates,
          actualCandidates: [actualSurfaceCandidate(windowId, listedWindow)],
          actualValues: match.actualValues,
          mismatchReason: match.mismatchReason,
        }
        : null,
      ambiguity,
    },
  };
}

const SESSION_LIFECYCLE_CODES = new Set([
  "session_dead",
  "forwarder_dead",
  "no_session",
  "app_process_dead_before_send",
  "app_process_dead_before_rpc",
  "forwarder_dead_before_send",
  "forwarder_dead_before_rpc",
]);

function errorCode(error: JsonObject): string {
  const direct = (error.error as JsonObject | undefined)?.code;
  if (typeof direct === "string") return direct;
  const parsed = error.parsedError as JsonObject | undefined | null;
  const parsedCode = (parsed?.error as JsonObject | undefined)?.code;
  return typeof parsedCode === "string" ? parsedCode : "";
}

function hasSessionLifecycleError(errors: JsonObject[]) {
  return errors.some((error) => SESSION_LIFECYCLE_CODES.has(errorCode(error)));
}

function lifecycleDetails(errors: JsonObject[]) {
  return errors
    .map((error) => {
      const parsed = (error.parsedError as JsonObject | undefined | null) ?? error;
      const code = errorCode(error);
      if (!SESSION_LIFECYCLE_CODES.has(code)) return null;
      return {
        label: error.label ?? null,
        code,
        lifecycle: parsed.lifecycle ?? error.lifecycle ?? null,
        keepActionsWindowOpen: parsed.keepActionsWindowOpen ?? null,
        sessionLifecycle: parsed.sessionLifecycle ?? null,
        message: (parsed.error as JsonObject | undefined)?.message ?? (error.error as JsonObject | undefined)?.message ?? null,
      };
    })
    .filter(Boolean);
}

function lifecycleCodes(errors: JsonObject[]) {
  return lifecycleDetails(errors)
    .map((detail) => (detail as JsonObject).code)
    .filter((code): code is string => typeof code === "string");
}

function primaryLifecycleDetails(errors: JsonObject[]) {
  return (lifecycleDetails(errors)[0] as JsonObject | undefined) ?? null;
}

function primarySessionLifecycle(errors: JsonObject[]) {
  const details = primaryLifecycleDetails(errors);
  return (details?.sessionLifecycle as JsonObject | undefined) ?? null;
}

function primaryParsedError(errors: JsonObject[]) {
  const lifecycleError = errors.find((error) => SESSION_LIFECYCLE_CODES.has(errorCode(error)));
  return (lifecycleError?.parsedError as JsonObject | undefined) ?? null;
}

function classification(args: Args, identity: ReturnType<typeof targetIdentity>, errors: JsonObject[]) {
  if (hasSessionLifecycleError(errors)) {
    return "blocked-by-session-lifecycle";
  }
  if (errors.length > 0) {
    return "blocked-by-timeout";
  }
  if (args.strict && !identity.resolvedTarget.automationId) {
    return "blocked-by-target-ambiguity";
  }
  if (args.strict && !identity.resolvedTarget.strictTargetMatch) {
    return "blocked-by-target-ambiguity";
  }
  return "ok";
}

async function maybeStartAndShow(args: Args) {
  if (args.start) {
    await run(["bash", "scripts/agentic/session.sh", "start", args.session], "session-start");
  }
  if (args.show) {
    await run([
      "bash",
      "scripts/agentic/session.sh",
      "send",
      args.session,
      JSON.stringify({ type: "show" }),
      "--await-parse",
      "--timeout",
      String(args.timeoutMs),
    ], "session-show");
  }
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  await maybeStartAndShow(args);

  const windowsEnvelope = await rpc(args.session, {
    type: "listAutomationWindows",
    requestId: requestId("list"),
  }, "automationWindowListResult", args.timeoutMs);
  const windows = responseOf(windowsEnvelope);
  const errors = [windowsEnvelope].filter((value) => value.status === "error");

  if (args.command === "list") {
    console.log(JSON.stringify({
      schemaVersion: 1,
      tool: "script-kit-devtools.targets",
      command: "targets.list",
      session: args.session,
      classification: hasSessionLifecycleError(errors) ? "blocked-by-session-lifecycle" : errors.length ? "blocked-by-timeout" : "ok",
      lifecycleCodes: lifecycleCodes(errors),
      lifecycleDetails: primaryLifecycleDetails(errors),
      sessionLifecycle: primarySessionLifecycle(errors),
      parsedError: primaryParsedError(errors),
      targetCount: pickWindows(windows).length,
      targets: pickWindows(windows),
      errors,
    }, null, 2));
    return;
  }

  const target = args.target ?? { type: "focused" };
  const inspectEnvelope = await rpc(args.session, {
    type: "inspectAutomationWindow",
    requestId: requestId("inspect"),
    target,
    hiDpi: args.hiDpi,
    probes: [],
  }, "automationInspectResult", args.timeoutMs);
  const inspect = responseOf(inspectEnvelope);
  const inspectErrors = [...errors, inspectEnvelope].filter((value) => value.status === "error");
  const identity = targetIdentity(args, inspect, windows);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.targets",
    command: "targets.inspect",
    session: args.session,
    classification: classification(args, identity, inspectErrors),
    ...identity,
    windows: pickWindows(windows),
    rawInspect: inspect,
    lifecycleCodes: lifecycleCodes(inspectErrors),
    lifecycleDetails: primaryLifecycleDetails(inspectErrors),
    sessionLifecycle: primarySessionLifecycle(inspectErrors),
    parsedError: primaryParsedError(inspectErrors),
    errors: inspectErrors,
  }, null, 2));
}

await main();
