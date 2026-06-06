#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Args = {
  command: "open-detached-placeholder" | "open-kitchen-sink";
  session: string;
  start: boolean;
  show: boolean;
  timeoutMs: number;
};

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/acp.ts open-detached-placeholder [--session <name>] [--start] [--show] [--timeout <ms>]",
    "  bun scripts/devtools/acp.ts open-kitchen-sink [--session <name>] [--start] [--show] [--timeout <ms>]",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }
  if (argv[0] !== "open-detached-placeholder" && argv[0] !== "open-kitchen-sink") {
    console.error(usage());
    process.exit(2);
  }
  const args: Args = {
    command: argv[0],
    session: "default",
    start: false,
    show: false,
    timeoutMs: 8000,
  };
  for (let index = 1; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--session") {
      args.session = argv[++index] ?? args.session;
    } else if (arg === "--start") {
      args.start = true;
    } else if (arg === "--show") {
      args.show = true;
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
    }
  }
  return args;
}

async function inspectEmbeddedAcpTarget(args: Args) {
  let targetReceipt: JsonObject | null = null;
  for (let attempt = 0; attempt < 6; attempt += 1) {
    targetReceipt = await run([
      "bun",
      "scripts/devtools/targets.ts",
      "inspect",
      "--session",
      args.session,
      "--target-kind",
      "main",
      "--surface",
      "AcpChat",
      "--strict",
      "--timeout",
      String(args.timeoutMs),
    ], "targets.inspect.embeddedAcp");
    if (targetReceipt.classification === "ok") {
      break;
    }
    await Bun.sleep(150);
  }
  return targetReceipt;
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
    return { status: "error", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim(), parsedError: parsed };
  }
  return parsed ?? { status: "ok", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
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

async function inspectDetachedTarget(args: Args) {
  let targetReceipt: JsonObject | null = null;
  for (let attempt = 0; attempt < 6; attempt += 1) {
    targetReceipt = await run([
      "bun",
      "scripts/devtools/targets.ts",
      "inspect",
      "--session",
      args.session,
      "--target-kind",
      "acpDetached",
      "--surface",
      "AcpChat",
      "--strict",
      "--timeout",
      String(args.timeoutMs),
    ], "targets.inspect.acpDetached");
    if (targetReceipt.classification === "ok") {
      break;
    }
    await Bun.sleep(150);
  }
  return targetReceipt;
}

async function openDetachedPlaceholder(args: Args) {
  await maybeStartAndShow(args);
  const requestId = `devtools-acp-detached-fixture-${Date.now()}`;
  const openReceipt = await run([
    "bash",
    "scripts/agentic/session.sh",
    "send",
    args.session,
    JSON.stringify({
      type: "openAcpDetachedFixture",
      requestId,
    }),
    "--await-parse",
    "--timeout",
    String(args.timeoutMs),
  ], "openAcpDetachedFixture");
  const targetReceipt = await inspectDetachedTarget(args);
  const resolvedTarget = targetReceipt?.resolvedTarget as JsonObject | undefined;
  const classification = openReceipt.status === "error"
    ? "blocked-by-timeout"
    : targetReceipt?.classification === "ok"
      ? "ok"
      : "blocked-by-target-ambiguity";

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.acp",
    command: "acp.openDetachedPlaceholder",
    classification,
    session: args.session,
    requestId,
    safety: {
      providerRequired: false,
      liveThreadRequired: false,
      fixtureOnly: true,
    },
    target: resolvedTarget ?? null,
    resolvedTarget: resolvedTarget ?? null,
    openReceipt,
    targetReceipt,
    errors: [openReceipt, targetReceipt].filter((receipt) => receipt?.status === "error"),
  }, null, 2));
}

async function openKitchenSink(args: Args) {
  await maybeStartAndShow(args);
  const requestId = `devtools-acp-kitchen-sink-${Date.now()}`;
  const openReceipt = await run([
    "bash",
    "scripts/agentic/session.sh",
    "rpc",
    args.session,
    JSON.stringify({
      type: "openAgentChatKitchenSinkFixture",
      requestId,
    }),
    "--timeout",
    String(args.timeoutMs),
  ], "openAgentChatKitchenSinkFixture");
  const targetReceipt = await inspectEmbeddedAcpTarget(args);
  const resolvedTarget = targetReceipt?.resolvedTarget as JsonObject | undefined;
  const classification = openReceipt.status === "error"
    ? "blocked-by-timeout"
    : targetReceipt?.classification === "ok"
      ? "ok"
      : "blocked-by-target-ambiguity";

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.acp",
    command: "acp.openKitchenSink",
    classification,
    session: args.session,
    requestId,
    safety: {
      providerRequired: false,
      liveThreadRequired: false,
      fixtureOnly: true,
    },
    target: resolvedTarget ?? null,
    resolvedTarget: resolvedTarget ?? null,
    openReceipt,
    targetReceipt,
    errors: [openReceipt, targetReceipt].filter((receipt) => receipt?.status === "error"),
  }, null, 2));
}

const args = parseArgs(Bun.argv.slice(2));
if (args.command === "open-kitchen-sink") {
  await openKitchenSink(args);
} else {
  await openDetachedPlaceholder(args);
}
