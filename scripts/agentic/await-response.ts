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
 *   3 = parse error detected preemptively (stdin_parse_failed for this requestId)
 */

import { existsSync, readFileSync, statSync } from "fs";
import { join } from "path";

const SCHEMA_VERSION = 1;
const SESSION_DIR =
  process.env.SCRIPT_KIT_SESSION_DIR ?? "/tmp/sk-agentic-sessions";
const DEFAULT_TIMEOUT = 5000;
const POLL_INTERVAL = 50; // ms between log scans

// Preemptive parse-failure detection. Mirrors the shell-side post-hoc
// scan in session.sh cmd_rpc (lines ~579-608) but runs in parallel with
// the typed-response wait, short-circuiting before the full --timeout
// elapses. Charset and cid format are shared with the Rust-side
// extract_request_id_lenient (src/stdin_commands/mod.rs); charset-unsafe
// requestIds (e.g. `a+b`, `a\b`) fall through to the shell post-hoc
// unscoped-grep fallback. Exit code 3 signals "parse error detected
// preemptively" so cmd_rpc callers can distinguish from generic timeout.
const PARSE_ERROR_EXIT_CODE = 3;
const REQUEST_ID_CHARSET = /^[A-Za-z0-9_.:/-]+$/;
const ERROR_MSG_MAX_CHARS = 200;

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

/**
 * Scan the log file for a `stdin_parse_failed` tracing line scoped to
 * the given requestId. Returns [extracted error message, new offset] or
 * [null, new offset]. The caller gates this on REQUEST_ID_CHARSET so
 * charset-unsafe ids (which Rust routes to `cid=stdin:parse:<uuid>`) are
 * handled by the shell post-hoc fallback instead of racing unscoped here.
 *
 * Mirrors session.sh cmd_rpc's scoped grep (line ~599): the trailing
 * space after requestId prevents prefix matches (e.g. `p17-get` must not
 * match `p17-get-foo`). The error= suffix is trimmed of the serde
 * `at line N column M` tail and capped at ERROR_MSG_MAX_CHARS.
 */
function scanForParseFailure(
  logPath: string,
  requestId: string,
  startOffset: number,
): [string | null, number] {
  let size: number;
  try {
    size = statSync(logPath).size;
  } catch {
    return [null, startOffset];
  }

  if (size < startOffset) {
    startOffset = 0;
  }

  if (size <= startOffset) return [null, startOffset];

  const buf = readFileSync(logPath);
  const newBytes = buf.subarray(startOffset);
  const lastNewline = newBytes.lastIndexOf(0x0a);

  if (lastNewline < 0) {
    return [null, startOffset];
  }

  const completeContent = newBytes.subarray(0, lastNewline + 1).toString("utf8");
  const newOffset = startOffset + lastNewline + 1;

  const cidMarker = `cid=stdin:req:${requestId} `;
  for (const line of completeContent.split("\n")) {
    if (!line.includes(cidMarker)) continue;
    if (!line.includes("event_type=stdin_parse_failed")) continue;
    const errIdx = line.indexOf(" error=");
    if (errIdx < 0) continue;
    let errMsg = line.substring(errIdx + " error=".length);
    errMsg = errMsg.replace(/ at line \d+ column \d+.*$/, "");
    if (errMsg.length > ERROR_MSG_MAX_CHARS) {
      errMsg = errMsg.substring(0, ERROR_MSG_MAX_CHARS);
    }
    return [errMsg, newOffset];
  }
  return [null, newOffset];
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
let parseFailScanOffset = startOffset;
const charsetSafeRequestId = REQUEST_ID_CHARSET.test(requestId);

while (Date.now() < deadline) {
  // Preemptive parse-failure check: if the Rust listener rejected this
  // send at parse time, short-circuit with a parse_error envelope
  // instead of waiting the full --timeout for a typed response that
  // will never arrive. Gated on charset-safe requestIds because Rust
  // routes charset-unsafe ids to `cid=stdin:parse:<uuid>` which the
  // scoped grep cannot match — those fall through to the shell
  // cmd_rpc post-hoc unscoped-grep fallback (session.sh lines ~600-602).
  if (charsetSafeRequestId) {
    const [errMsg, newFailOffset] = scanForParseFailure(
      logPath,
      requestId,
      parseFailScanOffset,
    );
    parseFailScanOffset = newFailOffset;
    if (errMsg) {
      const parseErr = errorResult(
        sessionName,
        requestId,
        "parse_error",
        errMsg,
      );
      printResult(parseErr);
      process.exit(PARSE_ERROR_EXIT_CODE);
    }
  }

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
