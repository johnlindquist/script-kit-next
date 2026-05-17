#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Args = {
  session: string;
  target?: JsonObject;
  strict: boolean;
  expectedSurfaceKind: string;
  limit: number;
  timeoutMs: number;
  start: boolean;
  show: boolean;
  forwarded: string[];
};

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/elements.ts snapshot [target args] [--limit <n>]",
    "",
    "Target args match scripts/devtools/targets.ts inspect, e.g. --session <name> --main --strict --surface ScriptList --start --show.",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv[0] !== "snapshot") {
    console.error(usage());
    process.exit(2);
  }

  const args: Args = {
    session: "default",
    strict: false,
    expectedSurfaceKind: "",
    limit: 200,
    timeoutMs: 8000,
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
    } else if (arg === "--strict") {
      args.strict = true;
    } else if (arg === "--limit") {
      args.limit = Number(argv[++index] ?? args.limit);
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
      args.forwarded.push(String(args.timeoutMs));
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
  return `devtools-elements-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
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

function asArray(value: unknown): JsonObject[] {
  return Array.isArray(value) ? value.filter((entry): entry is JsonObject => typeof entry === "object" && entry !== null) : [];
}

function nodeLabel(node: JsonObject) {
  return node.text ?? node.value ?? node.semanticId ?? null;
}

function snapshot(nodes: JsonObject[]) {
  const ids = nodes.map((node) => String(node.semanticId ?? "")).filter(Boolean);
  const seen = new Set<string>();
  const duplicateSemanticIds = ids.filter((id) => {
    if (seen.has(id)) {
      return true;
    }
    seen.add(id);
    return false;
  });

  return {
    nodes: nodes.map((node) => ({
      semanticId: node.semanticId ?? null,
      role: node.role ?? node.type ?? null,
      type: node.type ?? null,
      label: nodeLabel(node),
      text: node.text ?? null,
      value: node.value ?? null,
      selected: node.selected ?? null,
      focused: node.focused ?? null,
      index: node.index ?? null,
      owner: node.sourceName ?? node.source ?? null,
      actions: [],
      bounds: node.bounds ?? null,
      raw: node,
    })),
    duplicateSemanticIds: [...new Set(duplicateSemanticIds)],
    missingSemanticIdCount: nodes.length - ids.length,
    missingBoundsCount: nodes.filter((node) => node.bounds == null).length,
  };
}

function classify(targetReceipt: JsonObject, elementsEnvelope: JsonObject, elementSnapshot: ReturnType<typeof snapshot>) {
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (elementsEnvelope.status === "error") {
    return "blocked-by-timeout";
  }
  if (elementSnapshot.missingSemanticIdCount > 0 || elementSnapshot.duplicateSemanticIds.length > 0) {
    return "blocked-by-missing-primitive";
  }
  return "ok";
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const targetReceipt = await run(["bun", "scripts/devtools/targets.ts", "inspect", ...args.forwarded], "targets.inspect");
  const selector = (targetReceipt.requestedTarget as JsonObject | undefined)?.selector ?? args.target ?? { type: "focused" };
  const elementsEnvelope = await rpc(args.session, {
    type: "getElements",
    requestId: requestId("snapshot"),
    target: selector,
    limit: args.limit,
  }, "elementsResult", args.timeoutMs);
  const elements = responseOf(elementsEnvelope);
  const nodes = asArray(elements.elements);
  const elementSnapshot = snapshot(nodes);
  const classification = classify(targetReceipt, elementsEnvelope, elementSnapshot);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.elements",
    command: "elements.snapshot",
    classification,
    session: args.session,
    requestedTarget: targetReceipt.requestedTarget ?? { selector },
    target: targetReceipt.resolvedTarget ?? null,
    semanticSurface: {
      surfaceKind: (targetReceipt.resolvedTarget as JsonObject | undefined)?.surfaceKind ?? null,
      appViewVariant: (targetReceipt.resolvedTarget as JsonObject | undefined)?.appViewVariant ?? null,
    },
    totalCount: elements.totalCount ?? nodes.length,
    returnedCount: nodes.length,
    truncated: elements.truncated ?? false,
    focusedSemanticId: elements.focusedSemanticId ?? null,
    selectedSemanticId: elements.selectedSemanticId ?? null,
    duplicateSemanticIds: elementSnapshot.duplicateSemanticIds,
    missingPrimitives: [
      elementSnapshot.missingSemanticIdCount > 0 ? "semanticId" : "",
      elementSnapshot.missingBoundsCount > 0 ? "elementBounds" : "",
      elementsEnvelope.status === "error" ? "elementsResult" : "",
      targetReceipt.classification !== "ok" ? "strictTargetIdentity" : "",
    ].filter(Boolean),
    nodes: elementSnapshot.nodes,
    warnings: [
      ...(Array.isArray(elements.warnings) ? elements.warnings : []),
      elementSnapshot.missingBoundsCount > 0 ? "getElements does not expose bounds yet; use devtools.layout.measure for geometry." : "",
    ].filter(Boolean),
    errors: [targetReceipt, elementsEnvelope].filter((value) => value.status === "error"),
  }, null, 2));
}

await main();
