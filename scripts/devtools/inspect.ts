#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type InspectArgs = {
  session: string;
  target?: JsonObject;
  limit: number;
  timeoutMs: number;
  hiDpi: boolean;
  start: boolean;
  show: boolean;
};

function parseArgs(argv: string[]): InspectArgs {
  const args: InspectArgs = {
    session: "default",
    limit: 200,
    timeoutMs: 8000,
    hiDpi: false,
    start: false,
    show: false,
  };

  for (let index = 0; index < argv.length; index += 1) {
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
    } else if (arg === "--limit") {
      args.limit = Number(argv[++index] ?? args.limit);
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
    } else if (arg === "--hi-dpi") {
      args.hiDpi = true;
    } else if (arg === "--start") {
      args.start = true;
    } else if (arg === "--show") {
      args.show = true;
    }
  }

  return args;
}

async function run(command: string[], label: string): Promise<JsonObject> {
  const proc = Bun.spawn(command, {
    stdout: "pipe",
    stderr: "pipe",
  });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);

  if (exitCode !== 0) {
    return {
      status: "error",
      label,
      exitCode,
      stdout: stdout.trim(),
      stderr: stderr.trim(),
    };
  }

  try {
    return JSON.parse(stdout);
  } catch {
    return {
      status: "error",
      label,
      exitCode,
      stdout: stdout.trim(),
      stderr: stderr.trim(),
      error: "invalid_json_output",
    };
  }
}

function requestId(prefix: string) {
  return `devtools-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
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

function statePromptType(state: JsonObject) {
  if (state.status === "error") {
    return "tool_error";
  }
  return String(state.promptType ?? "");
}

function pickNumber(value: JsonObject, ...names: string[]) {
  for (const name of names) {
    const current = value[name];
    if (typeof current === "number") {
      return current;
    }
  }
  return null;
}

function pickArray(value: JsonObject, ...names: string[]) {
  for (const name of names) {
    const current = value[name];
    if (Array.isArray(current)) {
      return current;
    }
  }
  return [];
}

function warningsFrom(...values: Array<JsonObject | undefined>) {
  const warnings = values.flatMap((value) => {
    const warnings = value?.warnings;
    return Array.isArray(warnings) ? warnings.map(String) : [];
  });
  const errors = values
    .filter((value) => value?.status === "error")
    .map((value) => `${String(value?.label ?? "rpc")}: ${String(value?.error ?? value?.stderr ?? "error")}`);
  return [...warnings, ...errors];
}

function rpcErrors(...values: Array<JsonObject | undefined>) {
  return values
    .filter((value) => value?.status === "error")
    .map((value) => ({
      label: value?.label ?? "rpc",
      exitCode: value?.exitCode ?? null,
      error: value?.error ?? null,
      stderr: value?.stderr ?? null,
    }));
}

function missingFields(report: {
  state: JsonObject;
  elements: JsonObject;
  layout: JsonObject;
  inspect: JsonObject;
}) {
  const missing: string[] = [];
  const stateType = statePromptType(report.state);
  if (stateType === "tool_error" || stateType === "unsupported" || stateType === "target_resolution_failed") {
    missing.push("target_state");
  }
  if (!Array.isArray(report.elements.elements) || report.elements.elements.length === 0) {
    missing.push("semantic_elements");
  }
  if (report.inspect.semanticQuality && report.inspect.semanticQuality !== "full") {
    missing.push("full_semantic_elements");
  }
  const layoutComponents = (report.layout.info as JsonObject | undefined)?.components
    ?? report.layout.components;
  if (!Array.isArray(layoutComponents) || layoutComponents.length === 0) {
    missing.push("target_layout_info");
  }
  const screenshotWidth = pickNumber(report.inspect, "screenshotWidth", "screenshot_width");
  const screenshotHeight = pickNumber(report.inspect, "screenshotHeight", "screenshot_height");
  if (screenshotWidth == null || screenshotHeight == null) {
    missing.push("screenshot_metadata");
  }
  return [...new Set(missing)];
}

function capabilities(report: {
  state: JsonObject;
  elements: JsonObject;
  layout: JsonObject;
  inspect: JsonObject;
}) {
  const layoutComponents = (report.layout.info as JsonObject | undefined)?.components
    ?? report.layout.components;
  return {
    state: !["tool_error", "unsupported", "target_resolution_failed"].includes(statePromptType(report.state)),
    elements: Array.isArray(report.elements.elements) && report.elements.elements.length > 0,
    fullSemantics: report.inspect.semanticQuality === "full",
    layout: Array.isArray(layoutComponents) && layoutComponents.length > 0,
    screenshotMetadata: pickNumber(report.inspect, "screenshotWidth", "screenshot_width") != null
      && pickNumber(report.inspect, "screenshotHeight", "screenshot_height") != null,
    pixelProbes: Array.isArray(report.inspect.pixelProbes) || Array.isArray(report.inspect.pixel_probes),
    hitPoints: pickArray(report.inspect, "suggestedHitPoints", "suggested_hit_points").length > 0,
  };
}

function recommendedNext(missing: string[]) {
  const next = ["devtools.measure"];
  if (missing.includes("target_state")) {
    next.push("inspectAutomationWindow", "getElements");
  }
  if (missing.includes("target_layout_info")) {
    next.push("add-target-scoped-layout-info");
  }
  if (missing.includes("full_semantic_elements")) {
    next.push("add-surface-element-collector");
  }
  if (missing.includes("screenshot_metadata")) {
    next.push("verify-shot-strict-window");
  }
  return [...new Set(next)];
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
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

  const target = args.target ?? { type: "focused" };
  const windowsEnvelope = await rpc(args.session, {
    type: "listAutomationWindows",
    requestId: requestId("windows"),
  }, "automationWindowListResult", args.timeoutMs);
  const inspectEnvelope = await rpc(args.session, {
    type: "inspectAutomationWindow",
    requestId: requestId("inspect"),
    target,
    hiDpi: args.hiDpi,
    probes: [],
  }, "automationInspectResult", args.timeoutMs);
  const stateEnvelope = await rpc(args.session, {
    type: "getState",
    requestId: requestId("state"),
    target,
  }, "stateResult", args.timeoutMs);
  const elementsEnvelope = await rpc(args.session, {
    type: "getElements",
    requestId: requestId("elements"),
    target,
    limit: args.limit,
  }, "elementsResult", args.timeoutMs);
  const layoutEnvelope = await rpc(args.session, {
    type: "getLayoutInfo",
    requestId: requestId("layout"),
    target,
  }, "layoutInfoResult", args.timeoutMs);

  const windows = responseOf(windowsEnvelope);
  const inspect = responseOf(inspectEnvelope).snapshot as JsonObject | undefined ?? responseOf(inspectEnvelope);
  const state = responseOf(stateEnvelope);
  const elements = responseOf(elementsEnvelope);
  const layout = responseOf(layoutEnvelope);
  const report = { state, elements, layout, inspect };
  const missing = missingFields(report);
  const errors = rpcErrors(windowsEnvelope, inspectEnvelope, stateEnvelope, elementsEnvelope, layoutEnvelope);
  const screenshotWidth = pickNumber(inspect, "screenshotWidth", "screenshot_width");
  const screenshotHeight = pickNumber(inspect, "screenshotHeight", "screenshot_height");
  const suggestedHitPoints = pickArray(inspect, "suggestedHitPoints", "suggested_hit_points");
  const pixelProbes = pickArray(inspect, "pixelProbes", "pixel_probes");

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.inspect",
    session: args.session,
    requestedTarget: target,
    status: errors.length === 0 ? "ok" : "degraded",
    errors,
    windows,
    target: {
      windowId: inspect.windowId ?? null,
      windowKind: inspect.windowKind ?? null,
      title: inspect.title ?? null,
      resolvedBounds: inspect.resolvedBounds ?? null,
      semanticQuality: inspect.semanticQuality ?? null,
      targetBoundsInScreenshot: inspect.targetBoundsInScreenshot ?? null,
      surfaceHitPoint: inspect.surfaceHitPoint ?? null,
      suggestedHitPoints,
    },
    capabilities: capabilities(report),
    missingFields: missing,
    recommendedNext: recommendedNext(missing),
    warnings: warningsFrom(inspect, elements, layout, state),
    state,
    elements,
    layout,
    screenshot: {
      width: screenshotWidth,
      height: screenshotHeight,
      osWindowId: inspect.osWindowId ?? null,
      pixelProbes,
    },
  }, null, 2));
}

await main();
