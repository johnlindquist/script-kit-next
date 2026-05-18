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
    appViewVariant: window.appViewVariant ?? null,
    parentAutomationId: window.parentAutomationId ?? window.parentWindowId ?? null,
    parentKind: window.parentKind ?? null,
  }));
}

function expectedSurfaceMatches(snapshot: JsonObject, expectedSurfaceKind: string) {
  if (!expectedSurfaceKind) {
    return true;
  }
  const candidates = [
    snapshot.windowKind,
    snapshot.kind,
    snapshot.surfaceKind,
    snapshot.semanticSurface,
    snapshot.appViewVariant,
    (snapshot.surfaceContract as JsonObject | undefined)?.surfaceKind,
    (snapshot.state as JsonObject | undefined)?.surfaceKind,
  ].filter(Boolean).map(String);
  return candidates.includes(expectedSurfaceKind);
}

function targetIdentity(args: Args, inspect: JsonObject, windows: JsonObject) {
  const snapshot = (inspect.snapshot as JsonObject | undefined) ?? inspect;
  const resolvedBounds = snapshot.resolvedBounds ?? snapshot.bounds ?? null;
  const windowId = snapshot.windowId ?? snapshot.id ?? null;
  const listedWindow = pickWindows(windows).find((window) => window.automationId === windowId) ?? {};
  const strictTargetMatch = Boolean(windowId) && expectedSurfaceMatches(snapshot, args.expectedSurfaceKind);
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
      strictTargetMatch,
      ambiguity,
    },
  };
}

function classification(args: Args, identity: ReturnType<typeof targetIdentity>, errors: JsonObject[]) {
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
      classification: errors.length ? "blocked-by-timeout" : "ok",
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
    errors: inspectErrors,
  }, null, 2));
}

await main();
