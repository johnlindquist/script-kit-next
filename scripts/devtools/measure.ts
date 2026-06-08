#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type MeasureArgs = {
  inspectPath?: string;
  coveragePath?: string;
  surface?: string;
  markdown: boolean;
};

function parseArgs(argv: string[]): MeasureArgs {
  const args: MeasureArgs = { markdown: false };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--inspect" || arg === "--from") {
      args.inspectPath = argv[++index];
    } else if (arg === "--coverage") {
      args.coveragePath = argv[++index];
    } else if (arg === "--surface") {
      args.surface = argv[++index];
    } else if (arg === "--markdown") {
      args.markdown = true;
    }
  }
  return args;
}

async function readJson(path: string | undefined) {
  if (!path) {
    return null;
  }
  try {
    return JSON.parse(await Bun.file(path).text()) as JsonObject;
  } catch (error) {
    return {
      status: "error",
      path,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

function asArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : [];
}

function asObject(value: unknown): JsonObject {
  return value && typeof value === "object" && !Array.isArray(value) ? value as JsonObject : {};
}

function numberAt(value: JsonObject, key: string) {
  const current = value[key];
  return typeof current === "number" ? current : null;
}

function measurementSupport(inspect: JsonObject | null) {
  const elements = asObject(inspect?.elements);
  const layout = asObject(inspect?.layout);
  const screenshot = asObject(inspect?.screenshot);
  const target = asObject(inspect?.target);
  const layoutInfo = asObject(layout.info);
  const layoutComponents = asArray(layoutInfo.components).length > 0 ? asArray(layoutInfo.components) : asArray(layout.components);
  const elementRows = asArray(elements.elements);
  const missingFields = asArray(inspect?.missingFields).map(String);
  return {
    targetIdentity: target.windowId != null || target.windowKind != null,
    windowKind: target.windowKind ?? null,
    screenshotMetadata: numberAt(screenshot, "width") != null && numberAt(screenshot, "height") != null,
    semanticElements: elementRows.length > 0,
    layoutComponents: layoutComponents.length > 0,
    stateAvailable: !missingFields.includes("target_state"),
    elementCount: elementRows.length,
    layoutComponentCount: layoutComponents.length,
    screenshotSize: {
      width: numberAt(screenshot, "width"),
      height: numberAt(screenshot, "height"),
    },
  };
}

function targetMatchesSurface(surfaceId: string | undefined, windowKind: unknown) {
  if (!surfaceId) {
    return true;
  }
  const kind = String(windowKind ?? "").toLowerCase();
  if (surfaceId === "main") {
    return kind === "" || kind === "main";
  }
  if (surfaceId === "actions-dialog") {
    return kind.includes("action");
  }
  if (surfaceId === "notes" || surfaceId === "notes-agent_chat") {
    return kind.includes("notes");
  }
  if (surfaceId === "dictation" || surfaceId === "dictation-history") {
    return kind.includes("dictation");
  }
  return true;
}

function coverageSurface(coverage: JsonObject | null, requestedSurface?: string) {
  const surfaces = asArray(coverage?.surfaces).map(asObject);
  if (requestedSurface) {
    return surfaces.find((surface) => surface.id === requestedSurface) ?? null;
  }
  return surfaces[0] ?? null;
}

function plannedMeasurements(surface: JsonObject | null) {
  const missing = asArray(surface?.missingRuntimePrimitives).map(String);
  return {
    layout: {
      status: missing.some((item) => /layout|bounds|resize|anchor|overlap/i.test(item)) ? "partial" : "supported",
      missing: missing.filter((item) => /layout|bounds|resize|anchor|overlap/i.test(item)),
    },
    textFit: {
      status: missing.some((item) => /text|cursor|selection|preview|fingerprint|redacted/i.test(item)) ? "partial" : "unknown",
      missing: missing.filter((item) => /text|cursor|selection|preview|fingerprint|redacted/i.test(item)),
    },
    scroll: {
      status: missing.some((item) => /scroll/i.test(item)) ? "partial" : "unknown",
      missing: missing.filter((item) => /scroll/i.test(item)),
    },
    focus: {
      status: missing.some((item) => /focus|shortcut|hotkey|wrong-target/i.test(item)) ? "partial" : "unknown",
      missing: missing.filter((item) => /focus|shortcut|hotkey|wrong-target/i.test(item)),
    },
    media: {
      status: missing.some((item) => /microphone|media|model|recording|transcript|audio|delivery/i.test(item)) ? "missing" : "unknown",
      missing: missing.filter((item) => /microphone|media|model|recording|transcript|audio|delivery/i.test(item)),
    },
  };
}

function report(args: MeasureArgs, inspect: JsonObject | null, coverage: JsonObject | null) {
  const surface = coverageSurface(coverage, args.surface);
  const support = measurementSupport(inspect);
  const surfaceId = String(surface?.id ?? args.surface ?? "");
  const matchesSurface = targetMatchesSurface(surfaceId, support.windowKind);
  const planned = plannedMeasurements(surface);
  const missingRuntimePrimitives = asArray(surface?.missingRuntimePrimitives).map(String);
  const availableMeasurements = [
    support.targetIdentity ? "target.identity" : "",
    support.screenshotMetadata ? "screenshot.size" : "",
    support.semanticElements ? "semantic.elementCount" : "",
    support.layoutComponents ? "layout.componentCount" : "",
  ].filter(Boolean);

  return {
    schemaVersion: 1,
    tool: "script-kit-devtools.measure",
    status: inspect || coverage ? "ok" : "blocked",
    inputs: {
      inspectPath: args.inspectPath ?? null,
      coveragePath: args.coveragePath ?? null,
      requestedSurface: args.surface ?? null,
      targetMatchesSurface: matchesSurface,
    },
    surface: surface
      ? {
        id: surface.id ?? null,
        name: surface.name ?? null,
        status: surface.status ?? null,
      }
      : null,
    availableMeasurements,
    inspectSupport: support,
    targetSurfaceMatch: {
      matches: matchesSurface,
      requestedSurface: surfaceId || null,
      inspectedWindowKind: support.windowKind,
      warning: matchesSurface ? null : "inspect receipt target does not match requested surface; measurements are orientation only, not proof for that surface",
    },
    plannedMeasurements: planned,
    missingRuntimePrimitives,
    failClosed: missingRuntimePrimitives.length > 0 || !support.stateAvailable || !matchesSurface,
    recommendedNext: [
      missingRuntimePrimitives.includes("devtools.media.inspect") ? "devtools.media.inspect" : "",
      planned.layout.missing.length > 0 ? "target-scoped layout info" : "",
      planned.textFit.missing.length > 0 ? "text and selection bounds" : "",
      planned.scroll.missing.length > 0 ? "scroll anchor and viewport receipts" : "",
      planned.focus.missing.length > 0 ? "focus owner and shortcut registry receipts" : "",
    ].filter(Boolean),
  };
}

function markdown(result: ReturnType<typeof report>) {
  return [
    "# Script Kit DevTools Measure",
    "",
    `Status: ${result.status}`,
    `Surface: ${result.surface?.id ?? "unknown"}`,
    `Fail closed: ${String(result.failClosed)}`,
    "",
    "## Available Measurements",
    "",
    ...result.availableMeasurements.map((item) => `- ${item}`),
    "",
    "## Missing Runtime Primitives",
    "",
    ...result.missingRuntimePrimitives.map((item) => `- ${item}`),
  ].join("\n");
}

const args = parseArgs(Bun.argv.slice(2));
const [inspect, coverage] = await Promise.all([
  readJson(args.inspectPath),
  readJson(args.coveragePath),
]);
const result = report(args, inspect, coverage);
console.log(args.markdown ? markdown(result) : JSON.stringify(result, null, 2));
