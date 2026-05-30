#!/usr/bin/env bun
import { outputSummary, summarizeText, tryParseJson as summarizeJsonText } from "./lib/receipt-output";

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
  outputPath: string;
  previewBytes: number;
  inlineFullOutput: boolean;
  externalSession: string;
  externalOutputLog: string;
  wrapperTimeoutMs: number;
};

const actionOutputFields = [
  "stdoutSummary",
  "stderrSummary",
  "omittedBytes",
  "fingerprint",
  "stdoutJson",
  "artifactPath",
];

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/events.ts tail [--session <name>] [--limit <n>] [--contains <text>]",
    "  bun scripts/devtools/events.ts record [--session <name>] [--start] [--show] [--output <path>] [--preview-bytes <n>] [--inline-full-output] [--external-session <slug>] [--external-output-log <path>] [--wrapper-timeout-ms <n>] -- <devtools command...>",
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
    outputPath: "",
    previewBytes: 2048,
    inlineFullOutput: false,
    externalSession: "",
    externalOutputLog: "",
    wrapperTimeoutMs: 0,
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
    } else if (arg === "--output") {
      args.outputPath = options[++index] ?? "";
    } else if (arg === "--preview-bytes") {
      args.previewBytes = Number(options[++index] ?? args.previewBytes);
    } else if (arg === "--inline-full-output") {
      args.inlineFullOutput = true;
    } else if (arg === "--external-session") {
      args.externalSession = options[++index] ?? "";
      if (!args.wrapperTimeoutMs) args.wrapperTimeoutMs = 120000;
    } else if (arg === "--external-output-log") {
      args.externalOutputLog = options[++index] ?? "";
    } else if (arg === "--wrapper-timeout-ms") {
      args.wrapperTimeoutMs = Number(options[++index] ?? 0);
    }
  }

  if (command === "record" && args.childCommand.length === 0) {
    console.error(usage());
    process.exit(2);
  }

  return args;
}

async function runCommand(command: string[], label: string): Promise<{ receipt: JsonObject; stdout: string; stderr: string; exitCode: number }> {
  const proc = Bun.spawn(command, { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  if (exitCode !== 0) {
    return {
      receipt: { status: "error", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim() },
      stdout,
      stderr,
      exitCode,
    };
  }
  try {
    return { receipt: JSON.parse(stdout), stdout, stderr, exitCode };
  } catch {
    return {
      receipt: { status: "ok", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim() },
      stdout,
      stderr,
      exitCode,
    };
  }
}

async function run(command: string[], label: string): Promise<JsonObject> {
  return (await runCommand(command, label)).receipt;
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

function lineText(lines: Array<{ line?: string }>) {
  return lines.map((entry) => String(entry.line ?? "")).join("\n");
}

function classifyFailureSignals(child: JsonObject | null, appEvents: ReturnType<typeof parseLine>[], responseEvents: ReturnType<typeof parseLine>[]) {
  const text = [
    JSON.stringify(child ?? {}),
    lineText(appEvents),
    lineText(responseEvents),
  ].join("\n");
  const openAiSeen = /command_type=openAi|"type":"openAi"|openAi/.test(text);
  const acpNotAcp = /notAcp|not_acp|status.?[:=].?notAcp/i.test(text);
  const piWarmSeen = /pi warm|warm pi|prepare.*pi|acquire.*pi|Pi Text|openAi/i.test(text);
  const responseTimeout = /response_timeout|timeout|parseOutcome.?[:=].?timeout/i.test(text);
  return {
    openAi: {
      seen: openAiSeen,
      parsedEventSeen: /event_type=stdin_command_parsed.*command_type=openAi/.test(text),
    },
    acp: {
      statusNotAcpSeen: acpNotAcp,
      nextReceipt: "getAcpState(target ai or focused AcpChat target)",
    },
    piWarm: {
      signalsSeen: piWarmSeen,
      nextReceipt: "events.record around openAi plus getAcpState after transition",
    },
    timeout: {
      responseTimeoutSeen: responseTimeout,
    },
    likelyOwners: [
      acpNotAcp ? "acp-chat-core" : "",
      piWarmSeen ? "sdk-script-execution" : "",
      openAiSeen ? "protocol-automation" : "",
    ].filter(Boolean),
    recommendedNext: [
      openAiSeen && acpNotAcp ? "Prove target identity first: targets.inspect --target-kind acpDetached|main and getAcpState." : "",
      piWarmSeen ? "Separate warm Pi acquire failure from ACP routing failure in the receipt." : "",
      responseTimeout ? "Use compact output artifact path; do not rely on transcript tail." : "",
    ].filter(Boolean),
  };
}

function compactActionReceipt(receipt: JsonObject | null, inlineFullOutput: boolean) {
  if (!receipt || inlineFullOutput) return receipt;
  const safety = (receipt.safety as JsonObject | undefined) ?? {};
  const submitGate = (receipt.submitGate as JsonObject | undefined) ?? {};
  const targetBefore = (receipt.targetBefore as JsonObject | undefined) ?? null;
  const targetAfter = (receipt.targetAfter as JsonObject | undefined) ?? null;
  return {
    schemaVersion: receipt.schemaVersion ?? null,
    tool: receipt.tool ?? null,
    command: receipt.command ?? null,
    classification: receipt.classification ?? null,
    session: receipt.session ?? null,
    proofIntent: receipt.proofIntent ?? null,
    preflightOnly: receipt.preflightOnly ?? null,
    actionKind: receipt.actionKind ?? null,
    blockedAction: receipt.blockedAction ?? null,
    submitGate: {
      gateName: submitGate.gateName ?? null,
      key: submitGate.key ?? null,
      modifiers: submitGate.modifiers ?? [],
      allowSubmit: submitGate.allowSubmit ?? null,
      allowSubmitReason: submitGate.allowSubmitReason ?? null,
      submitIntent: submitGate.submitIntent ?? null,
      selectedSemanticId: submitGate.selectedSemanticId ?? null,
      selectedActionId: submitGate.selectedActionId ?? null,
      target: submitGate.target ?? null,
    },
    safety: {
      channel: safety.channel ?? null,
      destructive: safety.destructive ?? null,
      submitAllowed: safety.submitAllowed ?? null,
      submitIntent: safety.submitIntent ?? null,
      allowSubmitReason: safety.allowSubmitReason ?? null,
      nativeEscalation: safety.nativeEscalation ?? null,
      submitAttempted: safety.submitAttempted ?? null,
      errors: safety.errors ?? [],
      warnings: safety.warnings ?? [],
    },
    submitLifecycle: receipt.submitLifecycle ?? null,
    postActionLifecycle: receipt.postActionLifecycle ?? null,
    actionReceipt: receipt.actionReceipt ?? null,
    targetBefore: targetBefore
      ? {
        automationId: targetBefore.automationId ?? null,
        targetKind: targetBefore.targetKind ?? null,
        surfaceKind: targetBefore.surfaceKind ?? null,
        appViewVariant: targetBefore.appViewVariant ?? null,
        nativeFooterSurface: targetBefore.nativeFooterSurface ?? null,
        strictTargetMatch: targetBefore.strictTargetMatch ?? null,
      }
      : null,
    targetAfter: targetAfter
      ? {
        automationId: targetAfter.automationId ?? null,
        targetKind: targetAfter.targetKind ?? null,
        surfaceKind: targetAfter.surfaceKind ?? null,
        appViewVariant: targetAfter.appViewVariant ?? null,
        nativeFooterSurface: targetAfter.nativeFooterSurface ?? null,
        strictTargetMatch: targetAfter.strictTargetMatch ?? null,
      }
      : null,
    rawActionReceiptOmitted: true,
  };
}

function parsedSummary(parsed: JsonObject | null) {
  if (!parsed) return null;
  return {
    type: parsed.type ?? null,
    requestId: parsed.requestId ?? null,
    tool: parsed.tool ?? null,
    command: parsed.command ?? null,
    classification: parsed.classification ?? null,
    status: parsed.status ?? null,
    json: summarizeJsonText(JSON.stringify(parsed)),
  };
}

function compactEventEntries(entries: ReturnType<typeof parseLine>[], previewBytes: number, inlineFullOutput: boolean) {
  if (inlineFullOutput) return entries;
  return entries.map((entry) => ({
    index: entry.index,
    kind: entry.kind,
    eventType: entry.eventType,
    correlationId: entry.correlationId,
    commandType: entry.commandType,
    level: entry.level,
    line: summarizeText(entry.line, previewBytes).preview,
    lineSummary: summarizeText(entry.line, previewBytes),
    parsed: parsedSummary(entry.parsed),
  }));
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
  if (child?.status === "error" && String((child.error as JsonObject | undefined)?.code ?? "").includes("external_wrapper_timeout")) {
    return "blocked-by-external-wrapper-timeout";
  }
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
  let actionOutput: Awaited<ReturnType<typeof outputSummary>> | null = null;
  if (args.command === "record") {
    const childRun = await runCommand(args.childCommand, "recorded-command");
    child = childRun.receipt;
    actionOutput = await outputSummary("recorded-command", childRun.stdout, childRun.stderr, {
      outputPath: args.outputPath || null,
      previewBytes: args.previewBytes,
      inlineFullOutput: args.inlineFullOutput,
    });
  }

  const appEvents = await readLines(logPath, logOffset, args.limit, args.contains);
  const responseEvents = await readLines(responsesPath, responsesOffset, args.limit, args.contains);
  const endedAt = new Date().toISOString();
  const summary = {
    appLog: countEvents(appEvents),
    responses: countEvents(responseEvents),
  };
  const externalSession = args.externalSession
    ? {
      slug: args.externalSession,
      outputLogPath: args.externalOutputLog || `~/.oracle/sessions/${args.externalSession}/output.log`,
      wrapperTimeoutMs: args.wrapperTimeoutMs || 120000,
      wrapperTimeoutIsNotBrowserSessionFailure: true,
      reattachInstruction: "read output.log and preserve slug/path in the report",
    }
    : null;

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
    outputPolicy: {
      previewBytes: args.previewBytes,
      inlineFullOutput: args.inlineFullOutput,
      artifactPath: actionOutput?.artifactPath ?? null,
      actionOutputFields,
    },
    actionOutput,
    failureSignals: classifyFailureSignals(child, appEvents, responseEvents),
    externalSession,
    recordedCommand: args.command === "record" ? args.childCommand : null,
    actionReceipt: compactActionReceipt(child, args.inlineFullOutput),
    eventSummary: summary,
    events: {
      appLog: compactEventEntries(appEvents, args.previewBytes, args.inlineFullOutput),
      responses: compactEventEntries(responseEvents, args.previewBytes, args.inlineFullOutput),
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
