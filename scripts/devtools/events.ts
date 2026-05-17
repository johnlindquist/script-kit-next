#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Command = "tail" | "record";

type Args = {
  command: Command;
  session: string;
  limit: number;
  contains: string;
  start: boolean;
  show: boolean;
  timeoutMs: number;
  childCommand: string[];
};

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/events.ts tail [--session <name>] [--limit <n>] [--contains <text>]",
    "  bun scripts/devtools/events.ts record [--session <name>] [--start] [--show] -- <devtools command...>",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }
  const command = argv[0] as Command | undefined;
  if (!command || !["tail", "record"].includes(command)) {
    console.error(usage());
    process.exit(2);
  }
  const separator = argv.indexOf("--");
  const options = separator >= 0 ? argv.slice(1, separator) : argv.slice(1);
  const args: Args = {
    command,
    session: "default",
    limit: 80,
    contains: "",
    start: false,
    show: false,
    timeoutMs: 8000,
    childCommand: separator >= 0 ? argv.slice(separator + 1) : [],
  };

  for (let index = 0; index < options.length; index += 1) {
    const arg = options[index];
    if (arg === "--session") {
      args.session = options[++index] ?? args.session;
    } else if (arg === "--limit") {
      args.limit = Number(options[++index] ?? args.limit);
    } else if (arg === "--contains") {
      args.contains = options[++index] ?? "";
    } else if (arg === "--start") {
      args.start = true;
    } else if (arg === "--show") {
      args.show = true;
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(options[++index] ?? args.timeoutMs);
    }
  }

  if (command === "record" && args.childCommand.length === 0) {
    console.error(usage());
    process.exit(2);
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
    return { status: "ok", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
  }
}

async function fileSize(path: string) {
  try {
    return (await Bun.file(path).stat()).size;
  } catch {
    return 0;
  }
}

async function readLines(path: string, offset: number, limit: number, contains = "") {
  try {
    const text = (await Bun.file(path).text()).slice(offset);
    const lines = text.split(/\r?\n/).filter(Boolean);
    return lines
      .filter((line) => !contains || line.includes(contains))
      .slice(-limit)
      .map((line, index) => parseLine(line, index));
  } catch {
    return [];
  }
}

function parseLine(line: string, index: number) {
  const parsed = tryJson(line);
  const eventType = /event_type=([^ ]+)/.exec(line)?.[1] ?? /\bevent=([^ ]+)/.exec(line)?.[1] ?? null;
  const correlationId = /cid=([^ ]+)/.exec(line)?.[1] ?? null;
  const commandType = /command_type=([^ ]+)/.exec(line)?.[1] ?? null;
  const compactLevel = /^\d+(?:\.\d+)?\|([iwedt])\|/.exec(line)?.[1] ?? null;
  const level = /\b(INFO|WARN|ERROR|DEBUG|TRACE)\b/.exec(line)?.[1]?.toLowerCase() ?? compactLevelName(compactLevel);
  return {
    index,
    kind: parsed ? "json" : "log",
    eventType,
    correlationId,
    commandType,
    level,
    line,
    parsed,
  };
}

function compactLevelName(level: string | null) {
  if (level === "i") return "info";
  if (level === "w") return "warn";
  if (level === "e") return "error";
  if (level === "d") return "debug";
  if (level === "t") return "trace";
  return null;
}

function tryJson(line: string) {
  try {
    return JSON.parse(line) as JsonObject;
  } catch {
    return null;
  }
}

function countEvents(lines: ReturnType<typeof parseLine>[]) {
  const byType: Record<string, number> = {};
  const warnings: string[] = [];
  for (const entry of lines) {
    const key = entry.eventType ?? entry.commandType ?? entry.level ?? entry.kind;
    byType[key] = (byType[key] ?? 0) + 1;
    if (entry.level === "warn" || entry.level === "error" || entry.line.includes("WARN") || entry.line.includes("ERROR")) {
      warnings.push(entry.line);
    }
  }
  return { byType, warningCount: warnings.length, warnings: warnings.slice(-10) };
}

async function sessionStatus(args: Args) {
  if (args.start) {
    await run(["bash", "scripts/agentic/session.sh", "start", args.session], "session-start");
  }
  if (args.show) {
    await run(["bash", "scripts/agentic/session.sh", "send", args.session, JSON.stringify({ type: "show" }), "--await-parse", "--timeout", String(args.timeoutMs)], "session-show");
  }
  return run(["bash", "scripts/agentic/session.sh", "status", args.session], "session-status");
}

function classify(status: JsonObject, child: JsonObject | null, appLines: unknown[], responseLines: unknown[]) {
  if (status.status !== "ok" || status.healthy === false) {
    return "blocked-by-target-ambiguity";
  }
  if (child?.status === "error") {
    return "blocked-by-timeout";
  }
  if (appLines.length === 0 && responseLines.length === 0) {
    return "blocked-by-missing-primitive";
  }
  return "ok";
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const status = await sessionStatus(args);
  const logPath = String(status.log ?? "");
  const responsesPath = String(status.responses ?? "");
  const startedAt = new Date().toISOString();
  const logOffset = args.command === "record" ? await fileSize(logPath) : 0;
  const responsesOffset = args.command === "record" ? await fileSize(responsesPath) : 0;

  let child: JsonObject | null = null;
  if (args.command === "record") {
    child = await run(args.childCommand, "recorded-command");
  }

  const appEvents = await readLines(logPath, logOffset, args.limit, args.contains);
  const responseEvents = await readLines(responsesPath, responsesOffset, args.limit, args.contains);
  const endedAt = new Date().toISOString();
  const summary = {
    appLog: countEvents(appEvents),
    responses: countEvents(responseEvents),
  };

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.events",
    command: args.command === "record" ? "events.record" : "events.tail",
    classification: classify(status, child, appEvents, responseEvents),
    session: args.session,
    startedAt,
    endedAt,
    source: {
      logPath,
      responsesPath,
      logOffset,
      responsesOffset,
      limit: args.limit,
      contains: args.contains || null,
    },
    recordedCommand: args.command === "record" ? args.childCommand : null,
    actionReceipt: child,
    eventSummary: summary,
    events: {
      appLog: appEvents,
      responses: responseEvents,
    },
    warnings: [
      ...summary.appLog.warnings,
      ...summary.responses.warnings,
      appEvents.length === 0 && responseEvents.length === 0 ? "no session events matched the requested span" : "",
    ].filter(Boolean),
    errors: [
      status.status !== "ok" ? status : null,
      child?.status === "error" ? child : null,
    ].filter(Boolean),
  }, null, 2));
}

await main();
