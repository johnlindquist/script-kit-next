#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Args = {
  session: string;
  target?: JsonObject;
  limit: number;
  timeoutMs: number;
  forwarded: string[];
};

function usage() {
  return "Usage:\n  bun scripts/devtools/focus.ts inspect [target args] [--limit <n>]";
}

function parseArgs(argv: string[]): Args {
  if (argv[0] !== "inspect") {
    console.error(usage());
    process.exit(2);
  }
  const args: Args = { session: "default", limit: 100, timeoutMs: 8000, forwarded: [] };
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
    } else if (arg === "--limit") {
      args.limit = Number(argv[++index] ?? args.limit);
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
  return `devtools-focus-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run(["bash", "scripts/agentic/session.sh", "rpc", session, JSON.stringify(payload), "--expect", expect, "--timeout", String(timeoutMs)], String(payload.type ?? "rpc"));
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

function asArray(value: unknown): JsonObject[] {
  return Array.isArray(value) ? value.filter((entry): entry is JsonObject => typeof entry === "object" && entry !== null) : [];
}

function focusedNode(nodes: JsonObject[], focusedSemanticId: unknown) {
  const id = String(focusedSemanticId ?? "");
  return nodes.find((node) => node.semanticId === id || node.focused === true) ?? null;
}

function classify(targetReceipt: JsonObject, stateEnvelope: JsonObject, elementsEnvelope: JsonObject, focused: JsonObject | null) {
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (stateEnvelope.status === "error" || elementsEnvelope.status === "error") {
    return "blocked-by-timeout";
  }
  if (!focused) {
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
  const elementsEnvelope = await rpc(args.session, {
    type: "getElements",
    requestId: requestId("elements"),
    target: selector,
    limit: args.limit,
  }, "elementsResult", args.timeoutMs);
  const state = responseOf(stateEnvelope);
  const elements = responseOf(elementsEnvelope);
  const nodes = asArray(elements.elements);
  const focusedSemanticId = elements.focusedSemanticId ?? null;
  const selectedSemanticId = elements.selectedSemanticId ?? null;
  const focused = focusedNode(nodes, focusedSemanticId);
  const classification = classify(targetReceipt, stateEnvelope, elementsEnvelope, focused);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.focus",
    command: "focus.inspect",
    classification,
    session: args.session,
    requestedTarget: targetReceipt.requestedTarget ?? { selector },
    target: targetReceipt.resolvedTarget ?? null,
    windowFocused: state.isFocused ?? null,
    windowVisible: state.windowVisible ?? null,
    focusedSemanticId,
    selectedSemanticId,
    focusedNode: focused,
    selectedNode: nodes.find((node) => node.semanticId === selectedSemanticId) ?? null,
    activeFooter: state.activeFooter ?? null,
    submitDiagnostics: state.submitDiagnostics ?? null,
    keyboardOwner: {
      inputValue: state.inputValue ?? null,
      promptType: state.promptType ?? null,
      surfaceKind: (state.surfaceContract as JsonObject | undefined)?.surfaceKind ?? null,
      inputOwnership: (state.surfaceContract as JsonObject | undefined)?.inputOwnership ?? null,
      keyboardPolicy: (state.surfaceContract as JsonObject | undefined)?.keyboardPolicy ?? null,
    },
    missingPrimitives: [
      !focused ? "focusedSemanticId" : "",
      stateEnvelope.status === "error" ? "stateResult" : "",
      elementsEnvelope.status === "error" ? "elementsResult" : "",
      targetReceipt.classification !== "ok" ? "strictTargetIdentity" : "",
    ].filter(Boolean),
    warnings: [
      state.isFocused === false ? "window is visible but not focused" : "",
      focusedSemanticId == null ? "focused semantic id missing" : "",
    ].filter(Boolean),
    errors: [targetReceipt, stateEnvelope, elementsEnvelope].filter((value) => value.status === "error"),
  }, null, 2));
}

await main();
