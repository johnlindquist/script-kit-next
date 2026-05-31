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

function asObject(value: unknown): JsonObject {
  return value && typeof value === "object" && !Array.isArray(value) ? value as JsonObject : {};
}

function arrayOf(value: unknown): JsonObject[] {
  return Array.isArray(value) ? value.filter((entry): entry is JsonObject => typeof entry === "object" && entry !== null) : [];
}

function assertFileToken(inputText: string, fileLabel: string) {
  const bad = inputText.match(/@(md|ts|rs|js|py):/);
  const good = inputText.includes(`@file:${fileLabel}`) || inputText.includes(`@file:"${fileLabel}"`);
  return { good, bad: bad?.[0] ?? null, inputText };
}

function acpTargetScore(window: JsonObject) {
  const kind = String(window.windowKind ?? "").toLowerCase();
  const semanticSurface = String(window.semanticSurface ?? window.surfaceKind ?? "").toLowerCase();
  const automationId = String(window.automationId ?? "");
  if (kind === "ai" || automationId === "ai") return 100;
  if (semanticSurface === "acpchat") return 90;
  if (kind === "acpdetached") return 80;
  return 0;
}

export function resolveAcpTargetFromList(targetsReceipt: JsonObject) {
  const targets = arrayOf(targetsReceipt.targets)
    .map((window) => ({ window, score: acpTargetScore(window) }))
    .filter(({ score, window }) => score > 0 && typeof window.automationId === "string")
    .sort((left, right) => right.score - left.score);
  const selected = targets[0]?.window ?? null;
  return {
    target: selected ? { type: "id", id: String(selected.automationId) } : null,
    selected,
    candidates: targets.map(({ window, score }) => ({ score, ...window })),
  };
}

async function waitForAcpTarget(args: Args) {
  const deadline = Date.now() + args.timeoutMs;
  let lastTargets: JsonObject = {};
  let lastResolution = resolveAcpTargetFromList(lastTargets);
  while (Date.now() < deadline) {
    lastTargets = await run(["bun", "scripts/devtools/targets.ts", "list", "--session", args.session], "targets.list");
    lastResolution = resolveAcpTargetFromList(lastTargets);
    if (lastResolution.target) {
      return { targetsReceipt: lastTargets, targetResolution: lastResolution };
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  return { targetsReceipt: lastTargets, targetResolution: lastResolution };
}

async function waitForMainTarget(args: Args) {
  const deadline = Date.now() + args.timeoutMs;
  let lastTargets: JsonObject = {};
  while (Date.now() < deadline) {
    lastTargets = await run(["bun", "scripts/devtools/targets.ts", "list", "--session", args.session], "targets.list");
    const main = arrayOf(lastTargets.targets).find((window) => window.automationId === "main");
    if (main) {
      return { targetsReceipt: lastTargets, main };
    }
    await new Promise((resolve) => setTimeout(resolve, 250));
  }
  return { targetsReceipt: lastTargets, main: null };
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  // Use flat /tmp/sk-agentic-sessions/<session>/ (not .../${SESSION}/${SESSION}/).
  if (!process.env.SCRIPT_KIT_SESSION_DIR) {
    process.env.SCRIPT_KIT_SESSION_DIR = "/tmp/sk-agentic-sessions";
  }

  const setupReceipts: JsonObject = {};
  if (args.start) {
    setupReceipts.sessionStart = await run(["bash", "scripts/agentic/session.sh", "start", args.session], "session-start");
    for (let attempt = 0; attempt < 30; attempt += 1) {
      const status = await run(
        ["bash", "scripts/agentic/session.sh", "status", args.session],
        "session-status",
      );
      setupReceipts.lastSessionStatus = status;
      if (status.healthy === true && status.alive === true) {
        break;
      }
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
    setupReceipts.show = await run(
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
    setupReceipts.mainReady = await waitForMainTarget(args);
    setupReceipts.openAi = await run(
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
  } else {
    setupReceipts.openAi = await run(
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
  }

  let { targetsReceipt, targetResolution } = await waitForAcpTarget(args);
  const firstTargetResolution = targetResolution;
  if (!targetResolution.target) {
    setupReceipts.openAiRetry = await run(
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
      "openAi-retry",
    );
    ({ targetsReceipt, targetResolution } = await waitForAcpTarget(args));
  }
  const target = targetResolution.target;
  if (!target) {
    console.log(JSON.stringify({
      schemaVersion: 1,
      tool: "script-kit-devtools.acp-mention",
      command: "verify",
      session: args.session,
      classification: "blocked-by-target-ambiguity",
      reason: "noAcpTargetAfterOpenAi",
      fileLabel: args.fileLabel,
      setupReceipts,
      firstTargetResolution,
      targetResolution,
      targetsReceipt,
      cleanup: { command: `bash scripts/agentic/session.sh stop ${args.session}` },
    }, null, 2));
    process.exit(1);
  }

  // Picker is two-level: `@` → `@file` category → basename row (e.g. CLAUDE.md).
  const batchPayload = {
    type: "batch",
    requestId: `devtools-acp-mention-${Date.now()}`,
    target,
    commands: [
      { type: "setInput", text: "@" },
      { type: "waitFor", condition: { type: "acpPickerOpen" }, timeout: 8000, pollInterval: 25 },
      { type: "selectByValue", value: "@file", submit: true },
      { type: "waitFor", condition: { type: "acpPickerOpen" }, timeout: 8000, pollInterval: 25 },
      { type: "selectByValue", value: args.fileLabel, submit: true },
      { type: "waitFor", condition: { type: "acpItemAccepted" }, timeout: 8000, pollInterval: 25 },
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
  const batchFailure = asObject(batch.failure);

  const classification =
    batch.success === true && tokenCheck.good && !tokenCheck.bad
      ? "ok"
      : batch.success === true
        ? "reproduced"
        : String(batchFailure.message ?? "").includes("target resolution")
          ? "blocked-by-target-ambiguity"
          : "blocked-by-missing-primitive";

  const report = {
    schemaVersion: 1,
    tool: "script-kit-devtools.acp-mention",
    command: "verify",
    session: args.session,
    classification,
    fileLabel: args.fileLabel,
    setupReceipts,
    firstTargetResolution,
    targetResolution,
    batch,
    acpState: {
      inputText,
      lastAcceptedItem: state.lastAcceptedItem ?? null,
      picker: state.picker ?? null,
      resolvedTarget: state.resolvedTarget ?? null,
    },
    tokenCheck,
    cleanup: { command: `bash scripts/agentic/session.sh stop ${args.session}` },
  };

  console.log(JSON.stringify(report, null, 2));
  if (classification !== "ok") {
    process.exit(1);
  }
}

if (import.meta.main) {
  main().catch((error) => {
  console.error(JSON.stringify({ status: "error", message: String(error) }, null, 2));
  process.exit(1);
  });
}
