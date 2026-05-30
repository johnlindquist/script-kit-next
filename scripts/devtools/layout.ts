#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Rect = { x: number; y: number; width: number; height: number };

type Args = {
  session: string;
  target?: JsonObject;
  expectedSurfaceKind: string;
  timeoutMs: number;
  include: string[];
  limit: number;
  start: boolean;
  show: boolean;
  forwarded: string[];
};

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/layout.ts measure [target args] [--include nodes,regions,scroll,anchors,resize,overlaps] [--limit <n>]",
    "",
    "Target args match scripts/devtools/targets.ts inspect, e.g. --session <name> --main --strict --surface ScriptList --start --show.",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }
  if (argv[0] !== "measure") {
    console.error(usage());
    process.exit(2);
  }

  const args: Args = {
    session: "default",
    expectedSurfaceKind: "",
    timeoutMs: 8000,
    include: ["nodes", "regions", "scroll", "anchors", "resize", "overlaps"],
    limit: 200,
    start: false,
    show: false,
    forwarded: [],
  };

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
      args.expectedSurfaceKind = argv[++index] ?? "";
      args.forwarded.push(args.expectedSurfaceKind);
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
      args.forwarded.push(String(args.timeoutMs));
    } else if (arg === "--include") {
      args.include = String(argv[++index] ?? "")
        .split(",")
        .map((part) => part.trim())
        .filter(Boolean);
    } else if (arg === "--limit") {
      args.limit = Number(argv[++index] ?? args.limit);
    } else if (arg === "--start") {
      args.start = true;
    } else if (arg === "--show") {
      args.show = true;
    } else if (arg === "--strict") {
      // Forwarded only.
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
  return `devtools-layout-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(
  session: string,
  payload: JsonObject,
  expect: string,
  timeoutMs: number,
) {
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

function asArray(value: unknown): JsonObject[] {
  return Array.isArray(value)
    ? value.filter(
        (entry): entry is JsonObject =>
          typeof entry === "object" && entry !== null,
      )
    : [];
}

function asNumber(value: unknown, fallback = 0) {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

function rectFrom(value: unknown): Rect {
  const object =
    value && typeof value === "object" ? (value as JsonObject) : {};
  return {
    x: asNumber(object.x),
    y: asNumber(object.y),
    width: asNumber(object.width),
    height: asNumber(object.height),
  };
}

function optionalRectFrom(value: unknown): Rect | null {
  if (!value || typeof value !== "object") {
    return null;
  }
  return rectFrom(value);
}

function right(rect: Rect) {
  return rect.x + rect.width;
}

function bottom(rect: Rect) {
  return rect.y + rect.height;
}

function intersects(a: Rect, b: Rect) {
  return a.x < right(b) && right(a) > b.x && a.y < bottom(b) && bottom(a) > b.y;
}

function clippedBy(rect: Rect, viewport: Rect) {
  return (
    rect.x < viewport.x ||
    rect.y < viewport.y ||
    right(rect) > right(viewport) ||
    bottom(rect) > bottom(viewport)
  );
}

function hitMetrics(component: JsonObject, bounds: Rect) {
  const style =
    component.visualStyle && typeof component.visualStyle === "object"
      ? (component.visualStyle as JsonObject)
      : null;
  const hitBounds = optionalRectFrom(style?.hitBounds) ?? bounds;
  const visualBounds = optionalRectFrom(style?.visualBounds) ?? bounds;
  const exception =
    typeof style?.exception === "string" ? style.exception : null;
  return {
    hitBounds,
    visualBounds,
    hitWidth: hitBounds.width,
    hitHeight: hitBounds.height,
    visualWidth: visualBounds.width,
    visualHeight: visualBounds.height,
    minHitPass: hitBounds.width >= 28 && hitBounds.height >= 28,
    minVisualPass: visualBounds.width >= 20 && visualBounds.height >= 20,
    preferredHitPass: hitBounds.width >= 44 && hitBounds.height >= 44,
    exception,
  };
}

function center(rect: Rect) {
  return {
    x: rect.x + rect.width / 2,
    y: rect.y + rect.height / 2,
  };
}

function distance(a: { x: number; y: number }, b: { x: number; y: number }) {
  return Math.hypot(a.x - b.x, a.y - b.y);
}

function buttonCenterDistanceAssertions(nodes: Array<{
  name: unknown;
  type: unknown;
  parent: unknown;
  hitMetrics: ReturnType<typeof hitMetrics>;
}>) {
  const buttons = nodes.filter((node) => String(node.type ?? "") === "button");
  const failures = [];
  for (let left = 0; left < buttons.length; left += 1) {
    for (let rightIndex = left + 1; rightIndex < buttons.length; rightIndex += 1) {
      const a = buttons[left];
      const b = buttons[rightIndex];
      if (a.parent !== b.parent) continue;
      if (a.hitMetrics.exception || b.hitMetrics.exception) continue;
      const centerDistance = distance(
        center(a.hitMetrics.hitBounds),
        center(b.hitMetrics.hitBounds),
      );
      if (centerDistance < 60) {
        failures.push({
          a: a.name,
          b: b.name,
          centerDistance,
          required: 60,
          source: "apple-documented",
        });
      }
    }
  }
  return {
    source: "apple-documented",
    requiredCenterDistance: 60,
    failures,
  };
}

function analyzeLayout(layout: JsonObject, targetReceipt: JsonObject) {
  const info = (layout.info as JsonObject | undefined) ?? layout;
  const components = asArray(info.components);
  const targetBounds = rectFrom(
    (targetReceipt.resolvedTarget as JsonObject | undefined)?.bounds,
  );
  const viewportRect = {
    x: 0,
    y: 0,
    width: asNumber(info.windowWidth, targetBounds.width),
    height: asNumber(info.windowHeight, targetBounds.height),
  };
  const nodes = components.map((component) => {
    const bounds = rectFrom(component.bounds);
    return {
      name: component.name ?? null,
      type: component.type ?? null,
      bounds,
      depth: component.depth ?? null,
      parent: component.parent ?? null,
      children: component.children ?? [],
      explanation: component.explanation ?? null,
      visualStyle: component.visualStyle ?? null,
      hitMetrics: hitMetrics(component, bounds),
      clipped: clippedBy(bounds, viewportRect),
      raw: component,
    };
  });
  const overlaps = [];
  for (let left = 0; left < nodes.length; left += 1) {
    for (
      let rightIndex = left + 1;
      rightIndex < nodes.length;
      rightIndex += 1
    ) {
      const a = nodes[left];
      const b = nodes[rightIndex];
      const sameSiblingBand = a.depth === b.depth && a.parent === b.parent;
      if (
        sameSiblingBand &&
        a.name &&
        b.name &&
        intersects(a.bounds, b.bounds)
      ) {
        overlaps.push({ a: a.name, b: b.name });
      }
    }
  }
  const maxBottom = nodes.reduce(
    (current, node) => Math.max(current, bottom(node.bounds)),
    0,
  );
  const clippedNodeCount = nodes.filter((node) => node.clipped).length;
  const overlapCount = overlaps.length;
  const overflowY = maxBottom > viewportRect.height;
  const nodesWithVisualStyle = nodes.filter((node) => node.visualStyle != null);
  const controlsWithHitFailures = nodes.filter((node) => {
    const type = String(node.type ?? "");
    return (
      ["button", "input"].includes(type) &&
      !node.hitMetrics.minHitPass &&
      node.hitMetrics.exception == null
    );
  });
  const nativeMaterialSources = new Set([
    "NSGlassEffectView",
    "NSVisualEffectView",
    "nativeWindowBackdrop",
  ]);
  const contentNativeMaterialNodes = nodesWithVisualStyle.filter((node) => {
    const style = node.visualStyle as JsonObject;
    const materialSource = String(style.materialSource ?? "");
    return (
      style.chromeLayer === "content" &&
      nativeMaterialSources.has(materialSource)
    );
  });
  const glassLayerViolations = contentNativeMaterialNodes.map((node) => ({
    name: node.name,
    chromeLayer: (node.visualStyle as JsonObject).chromeLayer ?? null,
    materialSource: (node.visualStyle as JsonObject).materialSource ?? null,
  }));
  const buttonCenterDistance = buttonCenterDistanceAssertions(nodes);
  const hardcodedColorNodes = nodesWithVisualStyle.filter((node) => {
    const style = node.visualStyle as JsonObject;
    return style.usesSemanticThemeToken === false || style.colorSource === "hardcoded";
  });
  const cornerRadiusFailures = nodesWithVisualStyle.filter((node) => {
    const style = node.visualStyle as JsonObject;
    return style.cornerRadius == null && style.radius == null;
  });
  return {
    promptType: info.promptType ?? null,
    timestamp: info.timestamp ?? null,
    viewportRect,
    windowRect: targetBounds,
    regions: nodes.map((node) => ({
      name: node.name,
      type: node.type,
      bounds: node.bounds,
    })),
    nodes,
    overlaps,
    resizePressure: {
      windowCanGrow: null,
      overflowY,
      desiredContentHeight: maxBottom,
      measuredContentHeight: viewportRect.height,
      clippedNodeCount,
      overlapCount,
      pressureScore: clippedNodeCount + overlapCount + (overflowY ? 1 : 0),
    },
    visualAudit: {
      nodeCount: nodes.length,
      styledNodeCount: nodesWithVisualStyle.length,
      unstyledNodeCount: nodes.length - nodesWithVisualStyle.length,
      controlsWithHitFailures: controlsWithHitFailures.map((node) => ({
        name: node.name,
        type: node.type,
        hitMetrics: node.hitMetrics,
      })),
      contentGlassNodes: contentNativeMaterialNodes.map((node) => node.name),
      contentNativeMaterialNodes: contentNativeMaterialNodes.map((node) => node.name),
      glassLayerViolations,
      missingStyleNodeNames: nodes
        .filter((node) => node.visualStyle == null)
        .map((node) => node.name),
      chromeLayers: Object.fromEntries(
        Object.entries(
          nodesWithVisualStyle.reduce<Record<string, number>>((acc, node) => {
            const layer = String(
              (node.visualStyle as JsonObject).chromeLayer ?? "unknown",
            );
            acc[layer] = (acc[layer] ?? 0) + 1;
            return acc;
          }, {}),
        ).sort(([a], [b]) => a.localeCompare(b)),
      ),
      guidelineAssertions: {
        appleDocumented: {
          hitTargets: {
            source: "apple-documented",
            macosMinimumHitSize: { width: 28, height: 28 },
            macosMinimumVisualSize: { width: 20, height: 20 },
            failures: controlsWithHitFailures.map((node) => ({
              name: node.name,
              type: node.type,
              hitMetrics: node.hitMetrics,
            })),
          },
          buttonCenterDistance,
          materialLayering: {
            source: "apple-documented",
            contentGlassNodes: contentNativeMaterialNodes.map((node) => node.name),
            contentNativeMaterialNodes: contentNativeMaterialNodes.map((node) => node.name),
            glassLayerViolations,
          },
          colorAdaptivity: {
            source: "apple-documented",
            hardcodedColorNodes: hardcodedColorNodes.map((node) => node.name),
          },
          safeAreaLayout: {
            source: "apple-documented",
            clippedNodeCount,
            overflowY,
          },
          buttonShapeFamilies: {
            source: "apple-documented",
            exactAppleRadiusConstants: null,
          },
        },
        projectLocal: {
          cornerRadiusTokens: {
            source: "project-local",
            failures: cornerRadiusFailures.map((node) => node.name),
          },
          paddingTokens: {
            source: "project-local",
            minimumPanelPadding: 16,
            minimumCompactRowHorizontalPadding: 10,
          },
          spacingTokens: {
            source: "project-local",
            minimumFooterActionGap: 8,
          },
          windowBackdropPolicy: {
            source: "project-local",
            contentGlassNodeCount: contentNativeMaterialNodes.length,
          },
          themeTokenUsage: {
            source: "project-local",
            hardcodedColorNodes: hardcodedColorNodes.map((node) => node.name),
          },
          renderReadbackPixelThresholds: {
            source: "project-local",
            minimumNonBlackRatio: 0.01,
          },
        },
      },
    },
  };
}

function classify(
  targetReceipt: JsonObject,
  layoutEnvelope: JsonObject,
  analysis: ReturnType<typeof analyzeLayout>,
) {
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (layoutEnvelope.status === "error") {
    return "blocked-by-timeout";
  }
  if (analysis.nodes.length === 0) {
    return "blocked-by-missing-primitive";
  }
  return "ok";
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const targetReceipt = await run(
    ["bun", "scripts/devtools/targets.ts", "inspect", ...args.forwarded],
    "targets.inspect",
  );
  const selector = (targetReceipt.requestedTarget as JsonObject | undefined)
    ?.selector ??
    args.target ?? { type: "focused" };
  const layoutEnvelope = await rpc(
    args.session,
    {
      type: "getLayoutInfo",
      requestId: requestId("measure"),
      target: selector,
      options: {
        include: args.include,
        limit: args.limit,
      },
    },
    "layoutInfoResult",
    args.timeoutMs,
  );
  const layout = responseOf(layoutEnvelope);
  const analysis = analyzeLayout(layout, targetReceipt);
  const classification = classify(targetReceipt, layoutEnvelope, analysis);

  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        tool: "script-kit-devtools.layout",
        command: "layout.measure",
        classification,
        session: args.session,
        include: args.include,
        limit: args.limit,
        requestedTarget: targetReceipt.requestedTarget ?? { selector },
        target: targetReceipt.resolvedTarget ?? null,
        promptType: analysis.promptType,
        timestamp: analysis.timestamp,
        componentCount: analysis.nodes.length,
        window: {
          rect: analysis.windowRect,
          visible:
            (targetReceipt.resolvedTarget as JsonObject | undefined)?.visible ??
            null,
          focused:
            (targetReceipt.resolvedTarget as JsonObject | undefined)?.focused ??
            null,
        },
        viewport: {
          clientWidth: analysis.viewportRect.width,
          clientHeight: analysis.viewportRect.height,
          contentWidth: analysis.viewportRect.width,
          contentHeight: analysis.resizePressure.desiredContentHeight,
          scrollWidth: analysis.viewportRect.width,
          scrollHeight: analysis.resizePressure.desiredContentHeight,
          canScrollX: false,
          canScrollY: analysis.resizePressure.overflowY,
          scrollTop: null,
          maxScrollTop: Math.max(
            0,
            analysis.resizePressure.desiredContentHeight -
              analysis.viewportRect.height,
          ),
          overflowPolicyY: analysis.resizePressure.overflowY
            ? "auto"
            : "hidden",
        },
        pressure: {
          overflowY: analysis.resizePressure.overflowY,
          hiddenContentHeight: Math.max(
            0,
            analysis.resizePressure.desiredContentHeight -
              analysis.viewportRect.height,
          ),
          clippedNodeCount: analysis.resizePressure.clippedNodeCount,
          overlapCount: analysis.resizePressure.overlapCount,
          footerOverlapCount: analysis.overlaps.filter(
            (entry) =>
              String(entry.a).includes("Footer") ||
              String(entry.b).includes("Footer"),
          ).length,
          inputOverlapCount: analysis.overlaps.filter(
            (entry) =>
              String(entry.a).includes("Input") ||
              String(entry.b).includes("Input"),
          ).length,
          pressureScore: analysis.resizePressure.pressureScore,
        },
        viewportRect: analysis.viewportRect,
        windowRect: analysis.windowRect,
        regions: analysis.regions,
        nodes: analysis.nodes,
        overlaps: analysis.overlaps,
        resizePressure: analysis.resizePressure,
        visualAudit: analysis.visualAudit,
        handlerForm:
          (layout.info as JsonObject | undefined)?.handlerForm ??
          layout.handlerForm ??
          null,
        missingPrimitives: [
          analysis.nodes.length === 0 ? "layoutComponents" : "",
          layoutEnvelope.status === "error" ? "layoutInfoResult" : "",
          targetReceipt.classification !== "ok" ? "strictTargetIdentity" : "",
        ].filter(Boolean),
        warnings: [
          analysis.resizePressure.overflowY
            ? "content exceeds measured viewport height"
            : "",
          analysis.resizePressure.overlapCount > 0
            ? "layout components overlap"
            : "",
        ].filter(Boolean),
        errors: [targetReceipt, layoutEnvelope].filter(
          (value) => value.status === "error",
        ),
        rawLayout: layout,
      },
      null,
      2,
    ),
  );
}

await main();
