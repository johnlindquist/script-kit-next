#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Args = {
  session: string;
  target?: JsonObject;
  timeoutMs: number;
  forwarded: string[];
};

function usage() {
  return "Usage:\n  bun scripts/devtools/scroll.ts inspect [target args]";
}

function parseArgs(argv: string[]): Args {
  if (argv[0] !== "inspect") {
    console.error(usage());
    process.exit(2);
  }
  const args: Args = { session: "default", timeoutMs: 8000, forwarded: [] };
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
  return `devtools-scroll-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run(["bash", "scripts/agentic/session.sh", "rpc", session, JSON.stringify(payload), "--expect", expect, "--timeout", String(timeoutMs)], String(payload.type ?? "rpc"));
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

function asObject(value: unknown): JsonObject {
  return value && typeof value === "object" && !Array.isArray(value) ? value as JsonObject : {};
}

function asNumber(value: unknown) {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function classify(targetReceipt: JsonObject, stateEnvelope: JsonObject, scroll: JsonObject) {
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (stateEnvelope.status === "error") {
    return "blocked-by-timeout";
  }
  if (Object.keys(scroll).length === 0) {
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
  const state = responseOf(stateEnvelope);
  const scroll = asObject(state.mainListScroll);
  const contentHeight = asNumber(scroll.contentHeight);
  const viewportHeight = asNumber(scroll.viewportHeight);
  const maxScrollTop = asNumber(scroll.maxScrollTop);
  const scrollTop = asNumber(scroll.scrollTop);
  const canScrollY = maxScrollTop != null ? maxScrollTop > 0 : contentHeight != null && viewportHeight != null ? contentHeight > viewportHeight : null;
  const classification = classify(targetReceipt, stateEnvelope, scroll);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.scroll",
    command: "scroll.inspect",
    classification,
    session: args.session,
    requestedTarget: targetReceipt.requestedTarget ?? { selector },
    target: targetReceipt.resolvedTarget ?? null,
    scroll: {
      scrollTop,
      scrollTopItem: scroll.scrollTopItem ?? null,
      contentHeight,
      viewportHeight,
      safeViewportHeight: scroll.safeViewportHeight ?? null,
      footerHeight: scroll.footerHeight ?? null,
      maxScrollTop,
      canScrollY,
      selectedIndex: scroll.selectedIndex ?? state.selectedIndex ?? null,
      selectedRowTop: scroll.selectedRowTop ?? null,
      selectedRowBottom: scroll.selectedRowBottom ?? null,
      selectedRowVisible: scroll.selectedRowVisible ?? null,
      selectedRowAboveFooter: scroll.selectedRowAboveFooter ?? null,
      itemCount: scroll.itemCount ?? state.visibleChoiceCount ?? null,
    },
    resizePressure: {
      overflowY: canScrollY,
      hiddenContentHeight: contentHeight != null && viewportHeight != null ? Math.max(0, contentHeight - viewportHeight) : null,
      selectedRowOccluded: scroll.selectedRowVisible === false || scroll.selectedRowAboveFooter === false,
    },
    missingPrimitives: [
      Object.keys(scroll).length === 0 ? "mainListScroll" : "",
      stateEnvelope.status === "error" ? "stateResult" : "",
      targetReceipt.classification !== "ok" ? "strictTargetIdentity" : "",
    ].filter(Boolean),
    errors: [targetReceipt, stateEnvelope].filter((value) => value.status === "error"),
    state,
  }, null, 2));
}

await main();
