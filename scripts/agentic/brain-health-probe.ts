#!/usr/bin/env bun
/**
 * Runtime smoke proof: the kit://brain status resource exposes the BrainHealth
 * block the indexer records after every cycle. The round-trip contract itself
 * is covered by brain unit tests (health_snapshot_round_trips_all_fields,
 * health_error_snapshot_is_replaced_by_later_ok,
 * brain_status_resource_reports_health); this probe proves the block is
 * readable end-to-end from a live app over MCP resources/read.
 *
 * The indexer's first cycle waits 20s after launch (no env hook shortens it),
 * so this probe waits bounded (~35s) for the health block to populate.
 *
 *   PROBE_BINARY=target-agent/artifacts/brain-health/script-kit-gpui \
 *     bun scripts/agentic/brain-health-probe.ts
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { join } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const BINARY = process.env.PROBE_BINARY ?? "target-agent/artifacts/brain-health/script-kit-gpui";
const runId = `brain-health-${Date.now().toString(36)}`;
const outPath = ".test-output/brain-health-probe.json";

type Check = { name: string; pass: boolean; detail?: Json };

const checks: Check[] = [];
const failures: string[] = [];

function check(name: string, pass: boolean, detail: Json = {}) {
  checks.push({ name, pass, detail });
  if (!pass) failures.push(name);
}

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 10_000,
  intervalMs = 100,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  let last: T | undefined;
  while (Date.now() < deadline) {
    last = await read();
    if (accept(last)) return last;
    await Bun.sleep(intervalMs);
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

function readJson(path: string): Json {
  return JSON.parse(readFileSync(path, "utf8")) as Json;
}

async function mcp(serverJsonPath: string, method: string, params: Json): Promise<Json> {
  const discovery = readJson(serverJsonPath);
  const endpoint = String(discovery.url ?? "").endsWith("/rpc")
    ? String(discovery.url)
    : `${String(discovery.url ?? "").replace(/\/$/, "")}/rpc`;
  const token = String(discovery.token ?? "");
  if (!endpoint || !token) {
    throw new Error(`invalid MCP discovery at ${serverJsonPath}`);
  }
  const response = await fetch(endpoint, {
    method: "POST",
    headers: {
      authorization: `Bearer ${token}`,
      "content-type": "application/json",
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: `${runId}-${method}-${Date.now()}`,
      method,
      params,
    }),
  });
  const body = (await response.json()) as Json;
  if (!response.ok || body.error) {
    throw new Error(`MCP ${method} failed: ${JSON.stringify(body)}`);
  }
  return body.result as Json;
}

async function readStatusJson(serverJsonPath: string): Promise<Json> {
  const result = await mcp(serverJsonPath, "resources/read", { uri: "kit://brain" });
  const first = result.contents?.[0] as Json | undefined;
  if (!first || typeof first.text !== "string") {
    throw new Error(`resources/read returned no text for kit://brain: ${JSON.stringify(result)}`);
  }
  return JSON.parse(first.text) as Json;
}

async function findFreePort(): Promise<number> {
  return await new Promise((resolve, reject) => {
    const server = createServer();
    server.on("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      const port = typeof address === "object" && address ? address.port : 0;
      server.close((error) => {
        if (error) {
          reject(error);
        } else {
          resolve(port);
        }
      });
    });
  });
}

let driver: Driver | null = null;
let driverClosed = false;

try {
  const mcpPort = await findFreePort();
  driver = await Driver.launch({
    binary: BINARY,
    sessionName: "brain-health",
    sandboxHome: true,
    defaultTimeoutMs: 8000,
    env: {
      MCP_PORT: String(mcpPort),
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_BRAIN_TZ: "UTC",
    },
  });

  const sandboxHome = join(driver.sessionDir, "home");
  const skPath = join(sandboxHome, ".scriptkit");
  const serverJsonPath = join(skPath, "server.json");

  await waitFor("MCP server.json", () => existsSync(serverJsonPath), Boolean, 12_000);

  // Before the first cycle the health block is null; that itself proves the
  // key exists in the payload without a snapshot.
  const initialStatus = await readStatusJson(serverJsonPath);
  check("status_payload_has_health_key", "health" in initialStatus, {
    healthType: initialStatus.health === null ? "null" : typeof initialStatus.health,
  });

  // The indexer's first cycle fires ~20s after launch and records a snapshot.
  const statusWithHealth = await waitFor(
    "health block populated by first index cycle",
    () => readStatusJson(serverJsonPath),
    (status) =>
      status.health !== null &&
      typeof status.health === "object" &&
      typeof (status.health as Json).recall_mode === "string",
    35_000,
    1_000,
  );

  const health = statusWithHealth.health as Json;
  const recallMode = String(health.recall_mode ?? "");
  check(
    "health_block_reports_recall_mode",
    recallMode === "semantic" || recallMode === "lexical-only",
    { recallMode },
  );
  check(
    "health_block_reports_cycle_outcome",
    typeof health.last_cycle_ok === "boolean" &&
      typeof health.last_cycle_finished_unix === "number" &&
      typeof health.embedder_alive === "boolean" &&
      typeof health.docs_total === "number" &&
      typeof health.docs_pending_embedding === "number",
    {
      lastCycleOk: health.last_cycle_ok ?? null,
      lastCycleFinishedUnix: health.last_cycle_finished_unix ?? null,
      embedderAlive: health.embedder_alive ?? null,
      docsTotal: health.docs_total ?? null,
      docsPending: health.docs_pending_embedding ?? null,
    },
  );

  check(
    "runtime_log_has_no_panic",
    !/panicked|gpui_entity_double_lease/i.test(readFileSync(driver.logPath, "utf8")),
    { appLog: driver.logPath },
  );

  await driver.close();
  driverClosed = true;
  check("driver_closed", true, {});

  const pass = failures.length === 0 && checks.every((item) => item.pass);
  const receipt = {
    schemaVersion: 1,
    tool: "brain-health-probe",
    classification: pass ? "completed" : "failed",
    pass,
    failures,
    binary: BINARY,
    sandboxHome,
    serverJson: serverJsonPath,
    sessionDir: driver.sessionDir,
    appLog: driver.logPath,
    health,
    checks,
  };
  mkdirSync(".test-output", { recursive: true });
  writeFileSync(outPath, `${JSON.stringify(receipt, null, 2)}\n`, "utf8");
  console.log(JSON.stringify(receipt, null, 2));
  if (!pass) process.exit(1);
} catch (error) {
  if (driver && !driverClosed) {
    await driver.close().catch(() => {});
    driverClosed = true;
  }
  const receipt = {
    schemaVersion: 1,
    tool: "brain-health-probe",
    classification: "error",
    pass: false,
    failures: ["probe_completed_without_exception"],
    error: error instanceof Error ? error.message : String(error),
    binary: BINARY,
    sessionDir: driver?.sessionDir ?? null,
    appLog: driver?.logPath ?? null,
    checks,
  };
  mkdirSync(".test-output", { recursive: true });
  writeFileSync(outPath, `${JSON.stringify(receipt, null, 2)}\n`, "utf8");
  console.log(JSON.stringify(receipt, null, 2));
  process.exit(1);
}
