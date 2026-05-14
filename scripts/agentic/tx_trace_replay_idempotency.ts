#!/usr/bin/env bun
import { spawnSync } from "bun";

const session = "tx-trace-replay-idempotency";
const runId = Date.now();
const requestId = `tx-idem-${runId}`;

function run(args: string[], label: string): string {
  const proc = spawnSync({
    cmd: ["bash", "scripts/agentic/session.sh", ...args],
    cwd: import.meta.dir + "/../..",
    stdout: "pipe",
    stderr: "pipe",
  });
  const stdout = new TextDecoder().decode(proc.stdout).trim();
  const stderr = new TextDecoder().decode(proc.stderr).trim();
  if (proc.exitCode !== 0) {
    throw new Error(`${label} failed: ${stdout || stderr}`);
  }
  return stdout;
}

function rpc(payload: Record<string, unknown>, expect: string, timeout = 8_000) {
  const stdout = run(
    [
      "rpc",
      session,
      JSON.stringify(payload),
      "--expect",
      expect,
      "--timeout",
      String(timeout),
    ],
    `rpc:${payload.type}`,
  );
  const envelope = JSON.parse(stdout);
  if (envelope.status !== "ok") {
    throw new Error(`rpc error: ${stdout}`);
  }
  return envelope.response;
}

function send(payload: Record<string, unknown>) {
  run(
    [
      "send",
      session,
      JSON.stringify(payload),
      "--await-parse",
      "--timeout",
      "5000",
    ],
    `send:${payload.type}`,
  );
}

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function commandFingerprint(trace: Record<string, unknown>): string {
  return String(trace.commandFingerprint ?? trace.command_fingerprint ?? "");
}

function schemaVersion(trace: Record<string, unknown>): number {
  return Number(trace.schemaVersion ?? trace.schema_version ?? 0);
}

run(["start", session], "start");

try {
  send({ type: "show", requestId: "tx-idem-show" });

  const commands = [
    { type: "setInput", text: "" },
    { type: "waitFor", condition: "inputEmpty", timeout: 1000, pollInterval: 25 },
  ];

  const first = rpc(
    {
      type: "batch",
      requestId,
      commands,
      trace: "on",
      target: { type: "kind", kind: "main", index: 0 },
    },
    "batchResult",
  );
  assert(first.success === true, "first transaction should succeed");
  const firstFingerprint = commandFingerprint(first.trace ?? {});
  assert(schemaVersion(first.trace ?? {}) === 1, "trace must carry schemaVersion 1");
  assert(
    firstFingerprint.length > 0,
    `trace must carry a stable commandFingerprint: ${JSON.stringify(first.trace)}`,
  );
  assert(
    first.trace?.commands?.[0]?.commandPayload?.type === "setInput",
    "trace must carry command payloads",
  );

  const replay = rpc(
    {
      type: "batch",
      requestId,
      commands,
      trace: "on",
      target: { type: "kind", kind: "main", index: 0 },
    },
    "batchResult",
  );
  assert(replay.success === true, "same requestId and payload should replay");
  assert(
    commandFingerprint(replay.trace ?? {}) === firstFingerprint,
    "replay should return the original fingerprint",
  );

  const conflict = rpc(
    {
      type: "batch",
      requestId,
      commands: [{ type: "setInput", text: "different-payload" }],
      trace: "on",
      target: { type: "kind", kind: "main", index: 0 },
    },
    "batchResult",
  );
  assert(conflict.success === false, "same requestId with different payload should fail");
  assert(
    String(conflict.error?.message ?? conflict.results?.[0]?.error?.message ?? "").includes(
      "different transaction payload",
    ) || conflict.failedAt === 0,
    "conflict should surface a stable different-payload failure",
  );

  send({ type: "hide", requestId: "tx-idem-hide" });
  const hidden = rpc(
    {
      type: "waitFor",
      requestId: `tx-idem-hidden-${runId}`,
      condition: { type: "stateMatch", state: { windowVisible: false } },
      timeout: 1000,
      pollInterval: 25,
      trace: "on",
      target: { type: "kind", kind: "main", index: 0 },
    },
    "waitForResult",
  );
  assert(hidden.success === true, "app should be hidden at scenario end");

  console.log(
    JSON.stringify({
      schemaVersion: 1,
      scenario: "tx_trace_replay_idempotency",
      status: "pass",
      fingerprint: firstFingerprint,
      replayed: true,
      hidden: true,
    }),
  );
} finally {
  run(["stop", session], "stop");
}
