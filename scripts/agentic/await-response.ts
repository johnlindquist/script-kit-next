#!/usr/bin/env bun
/**
 * scripts/agentic/await-response.ts
 *
 * Tail the session log for a JSON response matching a given requestId
 * and optional response type. Returns structured JSON on stdout.
 *
 * Usage:
 *   bun await-response.ts --session default --request-id acp1 --expect acpStateResult --timeout 3000
 *
 * Exit codes:
 *   0 = response found
 *   1 = timeout
 *   2 = infrastructure error (missing session, bad args)
 */

import { existsSync, readFileSync, statSync } from "fs";
import { join } from "path";

const SCHEMA_VERSION = 1;
const SESSION_DIR =
  process.env.SCRIPT_KIT_SESSION_DIR ?? "/tmp/sk-agentic-sessions";
const DEFAULT_TIMEOUT = 5000;
const POLL_INTERVAL = 50; // ms between log scans

// --- types -----------------------------------------------------------------

interface RpcResponse {
  schemaVersion: number;
  status: "ok";
  session: string;
  requestId: string;
  responseType: string;
  response: unknown;
}

interface RpcError {
  schemaVersion: number;
  status: "error";
  session: string;
  requestId: string;
  error: { code: string; message: string };
}

type RpcResult = RpcResponse | RpcError;

function printResult(result: RpcResult): void {
  process.stdout.write(`${JSON.stringify(result)}\n`);
}

// --- helpers ---------------------------------------------------------------

function errorResult(
  session: string,
  requestId: string,
  code: string,
  message: string,
): RpcError {
  return {
    schemaVersion: SCHEMA_VERSION,
    status: "error",
    session,
    requestId,
    error: { code, message },
  };
}

/**
 * Scan the log file from `startOffset` for a JSON line containing the
 * given requestId and optionally the expected response type.
 * Returns [parsed JSON, new file offset] or [null, new offset].
 */
function scanLog(
  logPath: string,
  requestId: string,
  expect: string | null,
  startOffset: number,
): [unknown | null, string | null, number] {
  let size: number;
  try {
    size = statSync(logPath).size;
  } catch {
    return [null, null, startOffset];
  }

  if (size < startOffset) {
    startOffset = 0;
  }

  if (size <= startOffset) return [null, null, startOffset];

  const buf = readFileSync(logPath);
  const newBytes = buf.subarray(startOffset);
  const lastNewline = newBytes.lastIndexOf(0x0a);

  if (lastNewline < 0) {
    return [null, null, startOffset];
  }

  const completeContent = newBytes.subarray(0, lastNewline + 1).toString("utf8");
  const newOffset = startOffset + lastNewline + 1;

  for (const line of completeContent.split("\n")) {
    const trimmed = line.trim();
    if (!trimmed || !trimmed.startsWith("{")) continue;
    if (!trimmed.includes(requestId)) continue;

    try {
      const parsed = JSON.parse(trimmed);
      if (parsed.requestId !== requestId) continue;

      const responseType = parsed.type ?? parsed.responseType ?? null;
      if (expect && responseType !== expect) continue;

      return [parsed, responseType, newOffset];
    } catch {
      // not valid JSON, skip
    }
  }

  return [null, null, newOffset];
}

// --- main ------------------------------------------------------------------

const args = process.argv.slice(2);

function getArg(flag: string): string | null {
  const idx = args.indexOf(flag);
  return idx >= 0 && args[idx + 1] ? args[idx + 1] : null;
}

function parseNonNegativeInt(value: string | null, fallback: number): number {
  if (value == null) {
    return fallback;
  }

  const parsed = parseInt(value, 10);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : fallback;
}

const sessionName = getArg("--session") ?? "default";
const requestId = getArg("--request-id");
const expect = getArg("--expect");
const timeout = parseNonNegativeInt(getArg("--timeout"), DEFAULT_TIMEOUT);
const startOffset = parseNonNegativeInt(getArg("--start-offset"), 0);

if (!requestId) {
  const result = errorResult(
    sessionName,
    "",
    "missing_request_id",
    "Usage: await-response.ts --request-id ID [--expect TYPE] [--timeout MS]",
  );
  printResult(result);
  process.exit(2);
}

const sdir = join(SESSION_DIR, sessionName);
const logPath = join(sdir, "app.log");

if (!existsSync(sdir)) {
  const result = errorResult(
    sessionName,
    requestId,
    "no_session",
    `Session '${sessionName}' not found at ${sdir}`,
  );
  printResult(result);
  process.exit(2);
}

// Poll for new content
const deadline = Date.now() + timeout;
let scanOffset = startOffset;

while (Date.now() < deadline) {
  const [found, responseType, newOffset] = scanLog(
    logPath,
    requestId,
    expect,
    scanOffset,
  );
  scanOffset = newOffset;

  if (found) {
    const result: RpcResponse = {
      schemaVersion: SCHEMA_VERSION,
      status: "ok",
      session: sessionName,
      requestId,
      responseType: responseType ?? "unknown",
      response: found,
    };
    printResult(result);
    process.exit(0);
  }

  await Bun.sleep(POLL_INTERVAL);
}

// Timeout
const result = errorResult(
  sessionName,
  requestId,
  "timeout",
  `No response matching requestId '${requestId}'${expect ? ` and type '${expect}'` : ""} within ${timeout}ms`,
);
printResult(result);
process.exit(1);
