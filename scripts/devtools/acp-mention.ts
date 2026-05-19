#!/usr/bin/env bun

/**
 * DevTools proof: ACP @file mention picker accepts a file row as @file:<basename.ext>.
 *
 * Usage:
 *   bun scripts/devtools/acp-mention.ts verify --session <name> [--start] [--file CLAUDE.md]
 */

type JsonObject = Record<string, unknown>;

type Args = {
  command: "verify";
  session: string;
  start: boolean;
  fileLabel: string;
  timeoutMs: number;
};

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/acp-mention.ts verify --session <name> [--start] [--file <basename>]",
    "",
    "Opens Agent Chat, types @, selects a file row via batch selectByValue, and asserts",
    "the composer contains @file:<basename> (not @md:/@ts: extension prefixes).",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv[0] !== "verify") {
    console.error(usage());
    process.exit(2);
  }
  const args: Args = {
    command: "verify",
    session: "default",
    start: false,
    fileLabel: "CLAUDE.md",
    timeoutMs: 20000,
  };
  for (let i = 1; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--session") args.session = argv[++i] ?? args.session;
    else if (arg === "--start") args.start = true;
    else if (arg === "--file") args.fileLabel = argv[++i] ?? args.fileLabel;
    else if (arg === "--timeout") args.timeoutMs = Number(argv[++i] ?? args.timeoutMs);
    else if (arg === "--help" || arg === "-h") {
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
    return JSON.parse(stdout) as JsonObject;
  } catch {
    return { status: "error", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim(), error: "invalid_json_output" };
  }
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

function assertFileToken(inputText: string, fileLabel: string) {
  const bad = inputText.match(/@(md|ts|rs|js|py):/);
  const good = inputText.includes(`@file:${fileLabel}`) || inputText.includes(`@file:"${fileLabel}"`);
  return { good, bad: bad?.[0] ?? null, inputText };
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  if (args.start) {
    await run(["bash", "scripts/agentic/session.sh", "start", args.session], "session-start");
    await run(
      [
        "bash",
        "scripts/agentic/session.sh",
        "send",
        args.session,
        JSON.stringify({ type: "show" }),
        "--await-parse",
        "--timeout",
        String(args.timeoutMs),
      ],
      "session-show",
    );
  }

  await run(
    [
      "bash",
      "scripts/agentic/session.sh",
      "send",
      args.session,
      JSON.stringify({ type: "openAi" }),
      "--await-parse",
      "--timeout",
      String(args.timeoutMs),
    ],
    "openAi",
  );

  const target = { type: "id", id: "ai" };
  const batchPayload = {
    type: "batch",
    requestId: `devtools-acp-mention-${Date.now()}`,
    target,
    commands: [
      { type: "setInput", text: "@" },
      { type: "waitFor", condition: { type: "acpPickerOpen" }, timeout: 5000, pollInterval: 25 },
      { type: "selectByValue", value: args.fileLabel, submit: true },
      { type: "waitFor", condition: { type: "acpItemAccepted" }, timeout: 5000, pollInterval: 25 },
    ],
    options: { stopOnError: true, rollbackOnError: false, timeout: args.timeoutMs },
    trace: "on",
  };

  const batchEnvelope = await rpc(args.session, batchPayload, "batchResult", args.timeoutMs);
  const batch = responseOf(batchEnvelope);
  const stateEnvelope = await rpc(
    args.session,
    { type: "getAcpState", requestId: `devtools-acp-mention-state-${Date.now()}`, target },
    "acpStateResult",
    8000,
  );
  const state = responseOf(stateEnvelope);
  const inputText = String(state.inputText ?? "");
  const tokenCheck = assertFileToken(inputText, args.fileLabel);

  const classification =
    batch.success === true && tokenCheck.good && !tokenCheck.bad ? "fixed" : "blocked-by-missing-primitive";

  const report = {
    schemaVersion: 1,
    tool: "script-kit-devtools.acp-mention",
    command: "verify",
    session: args.session,
    classification,
    fileLabel: args.fileLabel,
    batch,
    acpState: {
      inputText,
      lastAcceptedItem: state.lastAcceptedItem ?? null,
      picker: state.picker ?? null,
    },
    tokenCheck,
    cleanup: { command: `bash scripts/agentic/session.sh stop ${args.session}` },
  };

  console.log(JSON.stringify(report, null, 2));
  if (classification !== "fixed") {
    process.exit(1);
  }
}

main().catch((error) => {
  console.error(JSON.stringify({ status: "error", message: String(error) }, null, 2));
  process.exit(1);
});
