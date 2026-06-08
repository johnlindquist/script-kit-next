#!/usr/bin/env bun

/**
 * DevTools proof: Agent Chat @file mention picker accepts a file row as @file:<basename.ext>.
 *
 * Usage:
 *   bun scripts/devtools/agent_chat-mention.ts verify --session <name> [--start] [--file CLAUDE.md]
 *   bun scripts/devtools/agent_chat-mention.ts bench --session <name> [--start] [--iterations 8] [--output <path>]
 */

import { mkdir } from "node:fs/promises";
import { dirname } from "node:path";

type JsonObject = Record<string, unknown>;

type Args = {
  command: "verify" | "bench";
  session: string;
  start: boolean;
  fileLabel: string;
  timeoutMs: number;
  iterations: number;
  discardWarmup: number;
  outputPath: string;
};

type ScenarioTargetKind = "main" | "agent_chat";

type BenchScenario = {
  name: string;
  targetKind: ScenarioTargetKind;
  values: string[];
};

type BenchStep = {
  label: string;
  input: string;
  success: boolean;
  traceTotalElapsedMs: number | null;
  commandElapsedMs: number | null;
  wallMs: number;
  effectiveElapsedMs: number | null;
  inputTextAfter: string;
  visibleCountAfter: number | null;
  spine: AgentChatSpineReceipt | null;
  spineProof: SpineProof;
  target: JsonObject | null;
  error?: unknown;
};

type BenchScenarioResult = {
  name: string;
  targetKind: ScenarioTargetKind;
  target: JsonObject | null;
  steps: BenchStep[];
  summary: BenchSummary;
};

type BenchSummary = {
  sampleCount: number;
  p50Ms: number | null;
  p95Ms: number | null;
  maxMs: number | null;
  over16Count: number;
  over32Count: number;
  missingTimingCount: number;
  missingTraceTimingCount: number;
  missingSpineCount: number;
  failedSpineProofCount: number;
  failedStepCount: number;
};

type AgentChatSpineReceipt = {
  ownsList: boolean;
  activeSegmentKind: string;
  subsearchSource?: string;
  rowCount: number;
  selectableRowCount: number;
  selectedIndex: number;
  rowFingerprint?: string;
  selectedRowFingerprint?: string;
  refreshElapsedMs: number;
};

type SpineProof = {
  required: boolean;
  ok: boolean;
  reason?: string;
};

const ACCEPTANCE = {
  maxAllowedP95RatioVsMainMenu: 1.25,
  maxAllowedAbsoluteP95Ms: 16,
  maxAllowedSingleStepMs: 32,
};

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/agent_chat-mention.ts verify --session <name> [--start] [--file <basename>]",
    "  bun scripts/devtools/agent_chat-mention.ts bench --session <name> [--start] [--iterations 8] [--discard-warmup 2] [--output <path>]",
    "",
    "Opens Agent Chat, types @, selects a file row via batch selectByValue, and asserts",
    "the composer contains @file:<basename> (not @md:/@ts: extension prefixes).",
    "",
    "The bench command compares main menu search typing against Agent Chat @ context typing",
    "with transaction-trace timings and strict target receipts.",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv[0] !== "verify" && argv[0] !== "bench") {
    console.error(usage());
    process.exit(2);
  }
  const args: Args = {
    command: argv[0],
    session: "default",
    start: false,
    fileLabel: "CLAUDE.md",
    timeoutMs: 20000,
    iterations: 8,
    discardWarmup: 2,
    outputPath: "",
  };
  for (let i = 1; i < argv.length; i += 1) {
    const arg = argv[i];
    if (arg === "--session") args.session = argv[++i] ?? args.session;
    else if (arg === "--start") args.start = true;
    else if (arg === "--file") args.fileLabel = argv[++i] ?? args.fileLabel;
    else if (arg === "--timeout") args.timeoutMs = Number(argv[++i] ?? args.timeoutMs);
    else if (arg === "--iterations") args.iterations = Number(argv[++i] ?? args.iterations);
    else if (arg === "--discard-warmup") args.discardWarmup = Number(argv[++i] ?? args.discardWarmup);
    else if (arg === "--output") args.outputPath = argv[++i] ?? "";
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

function numberOrNull(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function stringOrEmpty(value: unknown) {
  return typeof value === "string" ? value : "";
}

function spineOf(state: JsonObject): AgentChatSpineReceipt | null {
  const spine = asObject(state.spine);
  if (Object.keys(spine).length === 0) return null;
  return {
    ownsList: spine.ownsList === true,
    activeSegmentKind: stringOrEmpty(spine.activeSegmentKind),
    subsearchSource: typeof spine.subsearchSource === "string" ? spine.subsearchSource : undefined,
    rowCount: Number(spine.rowCount),
    selectableRowCount: Number(spine.selectableRowCount),
    selectedIndex: Number(spine.selectedIndex),
    rowFingerprint: typeof spine.rowFingerprint === "string" ? spine.rowFingerprint : undefined,
    selectedRowFingerprint: typeof spine.selectedRowFingerprint === "string" ? spine.selectedRowFingerprint : undefined,
    refreshElapsedMs: Number(spine.refreshElapsedMs),
  };
}

function agent_chatSpineRequiredForStep(scenario: BenchScenario, input: string) {
  return scenario.targetKind === "agent_chat" && input.length > 0 && input.startsWith("@");
}

function finiteNumber(value: unknown) {
  return typeof value === "number" && Number.isFinite(value);
}

function proveAgentChatSpineStep(
  scenario: BenchScenario,
  input: string,
  spine: AgentChatSpineReceipt | null,
): SpineProof {
  const required = agent_chatSpineRequiredForStep(scenario, input);
  if (!required) return { required, ok: true };
  if (!spine) return { required, ok: false, reason: "missing-spine" };
  if (!spine.ownsList) return { required, ok: false, reason: "spine-does-not-own-list" };
  if (!finiteNumber(spine.rowCount) || spine.rowCount < 1) {
    return { required, ok: false, reason: "invalid-row-count" };
  }
  if (!finiteNumber(spine.selectableRowCount) || spine.selectableRowCount < 0) {
    return { required, ok: false, reason: "invalid-selectable-row-count" };
  }
  if (!finiteNumber(spine.selectedIndex) || spine.selectedIndex < 0) {
    return { required, ok: false, reason: "invalid-selected-index" };
  }
  if (spine.selectableRowCount > 0 && spine.selectedIndex >= spine.selectableRowCount) {
    return { required, ok: false, reason: "selected-index-out-of-range" };
  }
  if (!spine.rowFingerprint) {
    return { required, ok: false, reason: "missing-row-fingerprint" };
  }
  if (!finiteNumber(spine.refreshElapsedMs) || spine.refreshElapsedMs < 0) {
    return { required, ok: false, reason: "invalid-refresh-elapsed" };
  }
  if (scenario.name === "agent_chat-file-subsearch" && spine.subsearchSource !== "file") {
    return { required, ok: false, reason: "missing-file-subsearch-source" };
  }
  if (scenario.name === "agent_chat-clipboard-subsearch" && spine.subsearchSource !== "clipboard") {
    return { required, ok: false, reason: "missing-clipboard-subsearch-source" };
  }
  if (scenario.name === "agent_chat-context-root" && spine.activeSegmentKind !== "contextMention") {
    return { required, ok: false, reason: "unexpected-context-root-kind" };
  }
  return { required, ok: true };
}

function assertFileToken(inputText: string, fileLabel: string) {
  const bad = inputText.match(/@(md|ts|rs|js|py):/);
  const good = inputText.includes(`@file:${fileLabel}`) || inputText.includes(`@file:"${fileLabel}"`);
  return { good, bad: bad?.[0] ?? null, inputText };
}

function agent_chatTargetScore(window: JsonObject) {
  const kind = String(window.windowKind ?? "").toLowerCase();
  const semanticSurface = String(window.semanticSurface ?? window.surfaceKind ?? "").toLowerCase();
  const automationId = String(window.automationId ?? "");
  if (kind === "ai" || automationId === "ai") return 100;
  if (semanticSurface === "agentchatchat") return 90;
  if (kind === "agentchatdetached") return 80;
  return 0;
}

export function resolveAgentChatTargetFromList(targetsReceipt: JsonObject) {
  const targets = arrayOf(targetsReceipt.targets)
    .map((window) => ({ window, score: agent_chatTargetScore(window) }))
    .filter(({ score, window }) => score > 0 && typeof window.automationId === "string")
    .sort((left, right) => right.score - left.score);
  const selected = targets[0]?.window ?? null;
  return {
    target: selected ? { type: "id", id: String(selected.automationId) } : null,
    selected,
    candidates: targets.map(({ window, score }) => ({ score, ...window })),
  };
}

async function waitForAgentChatTarget(args: Args) {
  const deadline = Date.now() + args.timeoutMs;
  let lastTargets: JsonObject = {};
  let lastResolution = resolveAgentChatTargetFromList(lastTargets);
  while (Date.now() < deadline) {
    lastTargets = await run(["bun", "scripts/devtools/targets.ts", "list", "--session", args.session], "targets.list");
    lastResolution = resolveAgentChatTargetFromList(lastTargets);
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

function sortedNumbers(values: number[]) {
  return [...values].sort((a, b) => a - b);
}

function percentile(values: number[], percentileValue: number) {
  if (values.length === 0) return null;
  const sorted = sortedNumbers(values);
  const index = Math.min(
    sorted.length - 1,
    Math.max(0, Math.ceil((percentileValue / 100) * sorted.length) - 1),
  );
  return sorted[index];
}

function commandElapsed(batch: JsonObject) {
  const results = arrayOf(batch.results);
  const setInput = results.find((entry) => entry.command === "setInput") ?? results[0];
  return numberOrNull(setInput?.elapsed);
}

function traceElapsed(batch: JsonObject) {
  const trace = asObject(batch.trace);
  return numberOrNull(batch.totalElapsed)
    ?? numberOrNull(batch.total_elapsed)
    ?? numberOrNull(trace.totalElapsedMs)
    ?? numberOrNull(trace.total_elapsed_ms);
}

export function summarizeBenchSteps(steps: BenchStep[], discardWarmup = 0): BenchSummary {
  const sampled = steps.slice(Math.max(0, discardWarmup));
  const timings = sampled
    .map((step) => step.effectiveElapsedMs)
    .filter((value): value is number => typeof value === "number" && Number.isFinite(value));
  return {
    sampleCount: sampled.length,
    p50Ms: percentile(timings, 50),
    p95Ms: percentile(timings, 95),
    maxMs: timings.length > 0 ? Math.max(...timings) : null,
    over16Count: timings.filter((value) => value > 16).length,
    over32Count: timings.filter((value) => value > 32).length,
    missingTimingCount: sampled.length - timings.length,
    missingTraceTimingCount: sampled.filter((step) => step.traceTotalElapsedMs == null && step.commandElapsedMs == null).length,
    missingSpineCount: sampled.filter((step) => step.spineProof.required && !step.spine).length,
    failedSpineProofCount: sampled.filter((step) => step.spineProof.required && !step.spineProof.ok).length,
    failedStepCount: sampled.filter((step) => !step.success).length,
  };
}

function p95Of(scenarios: BenchScenarioResult[], name: string) {
  return scenarios.find((scenario) => scenario.name === name)?.summary.p95Ms ?? null;
}

export function classifyBenchReceipt(scenarios: BenchScenarioResult[]) {
  if (scenarios.some((scenario) => !scenario.target)) {
    return "blocked-by-target-ambiguity";
  }
  if (scenarios.some((scenario) => scenario.summary.missingTimingCount > 0)) {
    return "blocked-by-missing-primitive";
  }
  if (scenarios.some((scenario) => scenario.summary.failedSpineProofCount > 0)) {
    return "blocked-by-missing-primitive";
  }
  if (scenarios.some((scenario) => scenario.summary.failedStepCount > 0)) {
    return "blocked-by-missing-primitive";
  }
  const mainMenuP95 = p95Of(scenarios, "main-menu-search-baseline");
  if (mainMenuP95 == null || mainMenuP95 <= 0) {
    return "blocked-by-missing-primitive";
  }

  const threshold = Math.max(
    ACCEPTANCE.maxAllowedAbsoluteP95Ms,
    mainMenuP95 * ACCEPTANCE.maxAllowedP95RatioVsMainMenu,
  );
  const agent_chatScenarioNames = [
    "agent_chat-context-root",
    "agent_chat-file-subsearch",
    "agent_chat-clipboard-subsearch",
  ];
  const slow = agent_chatScenarioNames.some((name) => {
    const p95 = p95Of(scenarios, name);
    return p95 == null || p95 > threshold;
  });
  const tooSlowStep = scenarios.some((scenario) => {
    if (!agent_chatScenarioNames.includes(scenario.name)) return false;
    return (scenario.summary.maxMs ?? 0) > ACCEPTANCE.maxAllowedSingleStepMs;
  });
  return slow || tooSlowStep ? "reproduced" : "fixed";
}

function comparisonFor(scenarios: BenchScenarioResult[]) {
  const mainMenuP95 = p95Of(scenarios, "main-menu-search-baseline");
  const agent_chatP95Values = ["agent_chat-context-root", "agent_chat-file-subsearch", "agent_chat-clipboard-subsearch"]
    .map((name) => p95Of(scenarios, name))
    .filter((value): value is number => value != null);
  const agent_chatAtSteadyStateP95Ms = agent_chatP95Values.length > 0 ? Math.max(...agent_chatP95Values) : null;
  return {
    mainMenuP95Ms: mainMenuP95,
    agent_chatAtSteadyStateP95Ms,
    ratio: mainMenuP95 && agent_chatAtSteadyStateP95Ms != null
      ? Number((agent_chatAtSteadyStateP95Ms / mainMenuP95).toFixed(3))
      : null,
  };
}

async function ensureSessionDefaults() {
  if (!process.env.SCRIPT_KIT_SESSION_DIR) {
    process.env.SCRIPT_KIT_SESSION_DIR = "/tmp/sk-agentic-sessions";
  }
}

async function startAndShowMain(args: Args) {
  const setupReceipts: JsonObject = {};
  if (args.start) {
    setupReceipts.sessionStart = await run(["bash", "scripts/agentic/session.sh", "start", args.session], "session-start");
    for (let attempt = 0; attempt < 30; attempt += 1) {
      const status = await run(
        ["bash", "scripts/agentic/session.sh", "status", args.session],
        "session-status",
      );
      setupReceipts.lastSessionStatus = status;
      if (status.healthy === true && status.alive === true) break;
      await new Promise((resolve) => setTimeout(resolve, 1000));
    }
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
  return setupReceipts;
}

async function openAgentChat(args: Args) {
  return run(
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

async function runSetInputStep(
  args: Args,
  scenario: BenchScenario,
  target: JsonObject,
  value: string,
) {
  const requestId = `devtools-agent_chat-context-bench-${scenario.name}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
  const waitCondition = scenario.targetKind === "agent_chat"
    ? { type: "agent_chatInputMatch", text: value }
    : { type: "stateMatch", state: { inputValue: value } };
  const payload = {
    type: "batch",
    requestId,
    target,
    commands: [
      { type: "setInput", text: value },
      { type: "waitFor", condition: waitCondition, timeout: 5000, pollInterval: 10 },
    ],
    options: { stopOnError: true, rollbackOnError: false, timeout: args.timeoutMs },
    trace: "on",
  };
  const started = Date.now();
  const batchEnvelope = await rpc(args.session, payload, "batchResult", args.timeoutMs);
  const wallMs = Date.now() - started;
  const batch = responseOf(batchEnvelope);
  const traceTotalElapsedMs = traceElapsed(batch);
  const commandElapsedMs = commandElapsed(batch);
  const effectiveElapsedMs = Math.max(
    ...[traceTotalElapsedMs, commandElapsedMs, wallMs]
      .filter((value): value is number => typeof value === "number" && Number.isFinite(value)),
  );
  const stateEnvelope = await rpc(
    args.session,
    scenario.targetKind === "agent_chat"
      ? { type: "getAgentChatState", requestId: `${requestId}-state`, target }
      : { type: "getState", requestId: `${requestId}-state`, target, summaryOnly: true },
    scenario.targetKind === "agent_chat" ? "agent_chatStateResult" : "stateResult",
    8000,
  );
  const state = responseOf(stateEnvelope);
  const inputTextAfter = scenario.targetKind === "agent_chat"
    ? stringOrEmpty(state.inputText)
    : stringOrEmpty(state.inputValue);
  const spine = scenario.targetKind === "agent_chat" ? spineOf(state) : null;
  const spineProof = proveAgentChatSpineStep(scenario, value, spine);
  const visibleCountAfter = scenario.targetKind === "agent_chat"
    ? numberOrNull(spine?.rowCount) ?? numberOrNull(asObject(state.picker).itemCount) ?? numberOrNull(asObject(state.picker).rowCount)
    : numberOrNull(state.choiceCount) ?? numberOrNull(state.visibleCount);
  return {
    label: value.length === 0 ? "clear" : `set ${value}`,
    input: value,
    success: batch.success === true && inputTextAfter === value && spineProof.ok,
    traceTotalElapsedMs,
    commandElapsedMs,
    wallMs,
    effectiveElapsedMs,
    inputTextAfter,
    visibleCountAfter,
    spine,
    spineProof,
    target,
    error: batch.success === true
      ? spineProof.ok ? undefined : { reason: spineProof.reason, spine }
      : batch.failure ?? batch.error ?? batchEnvelope,
  } satisfies BenchStep;
}

async function runScenario(
  args: Args,
  scenario: BenchScenario,
  target: JsonObject,
) {
  const steps: BenchStep[] = [];
  const values = Array.from({ length: Math.max(1, args.iterations) }, () => scenario.values).flat();
  for (const value of values) {
    steps.push(await runSetInputStep(args, scenario, target, value));
  }
  return {
    name: scenario.name,
    targetKind: scenario.targetKind,
    target,
    steps,
    summary: summarizeBenchSteps(steps, args.discardWarmup * scenario.values.length),
  } satisfies BenchScenarioResult;
}

async function writeOutput(path: string, receipt: JsonObject) {
  if (!path) return;
  await mkdir(dirname(path), { recursive: true });
  await Bun.write(path, JSON.stringify(receipt, null, 2));
}

async function runBench(args: Args) {
  await ensureSessionDefaults();
  const setupReceipts = await startAndShowMain(args);
  const mainTarget = { type: "id", id: "main" };
  const scenarios: BenchScenario[] = [
    {
      name: "main-menu-search-baseline",
      targetKind: "main",
      values: ["", "s", "se", "sel", "se", "s", ""],
    },
    {
      name: "main-menu-spine-at-baseline",
      targetKind: "main",
      values: ["", "@", "@s", "@se", "@sel", "@se", "@s", "@", ""],
    },
  ];
  const scenarioResults: BenchScenarioResult[] = [];
  for (const scenario of scenarios) {
    scenarioResults.push(await runScenario(args, scenario, mainTarget));
  }

  setupReceipts.openAi = await openAgentChat(args);
  let { targetsReceipt, targetResolution } = await waitForAgentChatTarget(args);
  const firstTargetResolution = targetResolution;
  if (!targetResolution.target) {
    setupReceipts.openAiRetry = await openAgentChat(args);
    ({ targetsReceipt, targetResolution } = await waitForAgentChatTarget(args));
  }

  const agent_chatTarget = targetResolution.target;
  if (agent_chatTarget) {
    for (const scenario of [
      {
        name: "agent_chat-context-root",
        targetKind: "agent_chat" as const,
        values: ["", "@", "@s", "@se", "@sel", "@se", "@s", "@", ""],
      },
      {
        name: "agent_chat-file-subsearch",
        targetKind: "agent_chat" as const,
        values: ["", "@file:", "@file:r", "@file:re", "@file:rea", "@file:re", "@file:r", "@file:", ""],
      },
      {
        name: "agent_chat-clipboard-subsearch",
        targetKind: "agent_chat" as const,
        values: ["", "@clipboard:", "@clipboard:t", "@clipboard:te", "@clipboard:t", "@clipboard:", ""],
      },
    ]) {
      scenarioResults.push(await runScenario(args, scenario, agent_chatTarget));
    }
  } else {
    for (const name of ["agent_chat-context-root", "agent_chat-file-subsearch", "agent_chat-clipboard-subsearch"]) {
      scenarioResults.push({
        name,
        targetKind: "agent_chat",
        target: null,
        steps: [],
        summary: summarizeBenchSteps([], 0),
      });
    }
  }

  const classification = classifyBenchReceipt(scenarioResults);
  const receipt = {
    schemaVersion: 1,
    tool: "script-kit-devtools.at-context-typing-bench",
    command: "bench",
    classification,
    session: args.session,
    acceptance: ACCEPTANCE,
    iterations: args.iterations,
    discardWarmup: args.discardWarmup,
    setupReceipts,
    firstTargetResolution,
    targetResolution,
    targetsReceipt,
    scenarios: scenarioResults,
    comparison: comparisonFor(scenarioResults),
    cleanup: { command: `bash scripts/agentic/session.sh stop ${args.session}` },
  };
  await writeOutput(args.outputPath, receipt);
  console.log(JSON.stringify(receipt, null, 2));
  if (classification.startsWith("blocked-")) {
    process.exit(1);
  }
}

async function runVerify(args: Args) {
  await ensureSessionDefaults();

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

  let { targetsReceipt, targetResolution } = await waitForAgentChatTarget(args);
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
    ({ targetsReceipt, targetResolution } = await waitForAgentChatTarget(args));
  }
  const target = targetResolution.target;
  if (!target) {
    console.log(JSON.stringify({
      schemaVersion: 1,
      tool: "script-kit-devtools.agent_chat-mention",
      command: "verify",
      session: args.session,
      classification: "blocked-by-target-ambiguity",
      reason: "noAgentChatTargetAfterOpenAi",
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
    requestId: `devtools-agent_chat-mention-${Date.now()}`,
    target,
    commands: [
      { type: "setInput", text: "@" },
      { type: "waitFor", condition: { type: "agent_chatPickerOpen" }, timeout: 8000, pollInterval: 25 },
      { type: "selectByValue", value: "@file", submit: true },
      { type: "waitFor", condition: { type: "agent_chatPickerOpen" }, timeout: 8000, pollInterval: 25 },
      { type: "selectByValue", value: args.fileLabel, submit: true },
      { type: "waitFor", condition: { type: "agent_chatItemAccepted" }, timeout: 8000, pollInterval: 25 },
    ],
    options: { stopOnError: true, rollbackOnError: false, timeout: args.timeoutMs },
    trace: "on",
  };

  const batchEnvelope = await rpc(args.session, batchPayload, "batchResult", args.timeoutMs);
  const batch = responseOf(batchEnvelope);
  const stateEnvelope = await rpc(
    args.session,
    { type: "getAgentChatState", requestId: `devtools-agent_chat-mention-state-${Date.now()}`, target },
    "agent_chatStateResult",
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
    tool: "script-kit-devtools.agent_chat-mention",
    command: "verify",
    session: args.session,
    classification,
    fileLabel: args.fileLabel,
    setupReceipts,
    firstTargetResolution,
    targetResolution,
    batch,
    agent_chatState: {
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

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  if (args.command === "bench") {
    await runBench(args);
  } else {
    await runVerify(args);
  }
}

if (import.meta.main) {
  main().catch((error) => {
  console.error(JSON.stringify({ status: "error", message: String(error) }, null, 2));
  process.exit(1);
  });
}
