#!/usr/bin/env bun
import { spawnSync } from "bun";

const session = "tx-wait-for-acp-runtime-semantics";
const runId = Date.now();

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

function waitFor(
  requestId: string,
  condition: string | Record<string, unknown>,
  timeout = 4_000,
) {
  const response = rpc(
    {
      type: "waitFor",
      requestId: `${requestId}-${runId}`,
      condition,
      timeout,
      pollInterval: 25,
      trace: "on",
      target: { type: "kind", kind: "ai", index: 0 },
    },
    "waitForResult",
    timeout + 2_000,
  );
  assert(response.success === true, `${requestId} should satisfy ${JSON.stringify(condition)}`);
  assert(
    (response.trace?.commands?.[0]?.polls?.length ?? 0) >= 1,
    `${requestId} should record at least one poll`,
  );
  return response;
}

run(["start", session], "start");

try {
  send({ type: "show", requestId: `tx-acp-show-${runId}` });
  send({ type: "triggerBuiltin", name: "tab-ai", requestId: `tx-acp-open-${runId}` });

  waitFor("tx-acp-ready", { type: "acpReady" }, 8_000);

  rpc(
    {
      type: "resetAcpTestProbe",
      requestId: `tx-acp-reset-probe-${runId}`,
      target: { type: "kind", kind: "ai", index: 0 },
    },
    "acpTestProbeResult",
  );

  send({
    type: "setAcpInput",
    requestId: `tx-acp-set-slash-${runId}`,
    text: "/",
    submit: false,
  });

  waitFor("tx-acp-picker-open", { type: "acpPickerOpen" });
  waitFor("tx-acp-input-contains", { type: "acpInputContains", substring: "/" });

  const tabResult = rpc(
    {
      type: "simulateGpuiEvent",
      requestId: `tx-acp-tab-${runId}`,
      target: { type: "kind", kind: "main", index: 0 },
      event: { type: "keyDown", key: "tab", modifiers: [] },
    },
    "simulateGpuiEventResult",
  );
  assert(tabResult.success !== false, `tab key dispatch should succeed: ${JSON.stringify(tabResult)}`);

  waitFor("tx-acp-accepted-tab", { type: "acpAcceptedViaKey", key: "tab" }, 5_000);

  const probe = rpc(
    {
      type: "getAcpTestProbe",
      requestId: `tx-acp-probe-${runId}`,
      target: { type: "kind", kind: "ai", index: 0 },
    },
    "acpTestProbeResult",
  ).probe;
  const layout = probe?.inputLayout;
  if (layout) {
    waitFor("tx-acp-layout", {
      type: "acpInputLayoutMatch",
      visibleStart: layout.visibleStart,
      visibleEnd: layout.visibleEnd,
      cursorInWindow: layout.cursorInWindow,
    });
  }

  send({ type: "hide", requestId: `tx-acp-hide-${runId}` });
  const hidden = rpc(
    {
      type: "waitFor",
      requestId: `tx-acp-hidden-${runId}`,
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
      scenario: "tx_wait_for_acp_runtime_semantics",
      status: "pass",
      acceptedViaKey: "tab",
      inputLayoutMatched: Boolean(layout),
      hidden: true,
    }),
  );
} finally {
  run(["stop", session], "stop");
}
