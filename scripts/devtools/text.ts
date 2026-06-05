#!/usr/bin/env bun

export {};

type JsonObject = Record<string, unknown>;

type Args = {
  session: string;
  target?: JsonObject;
  limit: number;
  timeoutMs: number;
  forwarded: string[];
};

function usage() {
  return "Usage:\n  bun scripts/devtools/text.ts measure [target args] [--limit <n>]";
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
  const args: Args = { session: "default", limit: 120, timeoutMs: 8000, forwarded: [] };
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
  return `devtools-text-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
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

function textOf(node: JsonObject) {
  const text = node.text ?? node.value ?? "";
  return typeof text === "string" ? text : String(text);
}

function fingerprint(value: string) {
  let hash = 2166136261;
  for (let index = 0; index < value.length; index += 1) {
    hash ^= value.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return (hash >>> 0).toString(16).padStart(8, "0");
}

function textRows(nodes: JsonObject[]) {
  return nodes
    .map((node) => {
      const text = textOf(node);
      return {
        semanticId: node.semanticId ?? null,
        role: node.role ?? node.type ?? null,
        text,
        textLength: text.length,
        lineCount: text.length ? text.split(/\r\n|\r|\n/).length : 0,
        selected: node.selected ?? null,
        focused: node.focused ?? null,
        fingerprint: fingerprint(text),
      };
    })
    .filter((row) => row.textLength > 0);
}

function classify(targetReceipt: JsonObject, stateEnvelope: JsonObject, elementsEnvelope: JsonObject, rows: ReturnType<typeof textRows>) {
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (stateEnvelope.status === "error" || elementsEnvelope.status === "error") {
    return "blocked-by-timeout";
  }
  if (rows.length === 0) {
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
  const rows = textRows(nodes);
  const inputValue = typeof state.inputValue === "string" ? state.inputValue : "";
  const selectedValue = typeof state.selectedValue === "string" ? state.selectedValue : "";
  const footerButtons = asArray((state.activeFooter as JsonObject | undefined)?.buttons);
  const footerTexts = footerButtons.map((button) => ({
    action: button.action ?? null,
    key: button.key ?? null,
    label: button.label ?? null,
    labelLength: typeof button.label === "string" ? button.label.length : null,
  }));
  const classification = classify(targetReceipt, stateEnvelope, elementsEnvelope, rows);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.text",
    command: "text.measure",
    classification,
    session: args.session,
    requestedTarget: targetReceipt.requestedTarget ?? { selector },
    target: targetReceipt.resolvedTarget ?? null,
    textSummary: {
      inputValue,
      inputLength: inputValue.length,
      inputFingerprint: fingerprint(inputValue),
      selectedValue,
      selectedLength: selectedValue.length,
      selectedFingerprint: fingerprint(selectedValue),
      textNodeCount: rows.length,
      longestTextLength: rows.reduce((max, row) => Math.max(max, row.textLength), 0),
    },
    rows,
    footerTexts,
    missingPrimitives: [
      rows.length === 0 ? "textNodes" : "",
      rows.some((row) => row.semanticId == null) ? "semanticId" : "",
      rows.length > 0 ? "textBounds" : "",
      stateEnvelope.status === "error" ? "stateResult" : "",
      elementsEnvelope.status === "error" ? "elementsResult" : "",
      targetReceipt.classification !== "ok" ? "strictTargetIdentity" : "",
    ].filter(Boolean),
    warnings: [
      "Text bounds are not exposed by getElements yet; pair this with devtools.layout.measure for geometry.",
    ],
    errors: [targetReceipt, stateEnvelope, elementsEnvelope].filter((value) => value.status === "error"),
  }, null, 2));
}

await main();
