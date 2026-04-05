#!/usr/bin/env bun
/**
 * scripts/agentic/session-state.ts
 *
 * Machine-readable session state reporter for Script Kit GPUI agentic testing.
 * Reports whether a named session is alive, where logs are, and pipe status.
 *
 * Usage:
 *   bun scripts/agentic/session-state.ts --session default
 *   bun scripts/agentic/session-state.ts --session default --json
 *   bun scripts/agentic/session-state.ts --list
 *
 * All output is JSON on stdout. Diagnostics go to stderr.
 */

import { existsSync, readdirSync, readFileSync, statSync } from "fs";
import { join } from "path";

const SCHEMA_VERSION = 1;
const SESSION_DIR =
  process.env.SCRIPT_KIT_SESSION_DIR ?? "/tmp/sk-agentic-sessions";

interface SessionState {
  schemaVersion: number;
  status: "ok" | "not_found" | "error";
  session: string;
  alive: boolean;
  pid: number | null;
  pipe: string | null;
  pipeWritable: boolean;
  log: string | null;
  logSizeBytes: number | null;
  logLastLine: string | null;
  responses: string | null;
  responsesCount: number | null;
  error?: { code: string; message: string };
}

interface SessionList {
  schemaVersion: number;
  status: "ok";
  sessions: SessionState[];
}

function isProcessAlive(pid: number): boolean {
  try {
    process.kill(pid, 0);
    return true;
  } catch {
    return false;
  }
}

function getLastLogLine(logPath: string): string | null {
  try {
    const content = readFileSync(logPath, "utf-8");
    const lines = content.trimEnd().split("\n");
    return lines[lines.length - 1] ?? null;
  } catch {
    return null;
  }
}

function getSessionState(name: string): SessionState {
  const sdir = join(SESSION_DIR, name);

  if (!existsSync(sdir)) {
    return {
      schemaVersion: SCHEMA_VERSION,
      status: "not_found",
      session: name,
      alive: false,
      pid: null,
      pipe: null,
      pipeWritable: false,
      log: null,
      logSizeBytes: null,
      logLastLine: null,
      responses: null,
      responsesCount: null,
    };
  }

  const pidPath = join(sdir, "pid");
  const inputFifo = join(sdir, "input");
  const logPath = join(sdir, "app.log");
  const responsesPath = join(sdir, "responses.ndjson");

  let pid: number | null = null;
  let alive = false;
  if (existsSync(pidPath)) {
    pid = parseInt(readFileSync(pidPath, "utf-8").trim(), 10);
    if (!isNaN(pid)) {
      alive = isProcessAlive(pid);
    } else {
      pid = null;
    }
  }

  let pipeWritable = false;
  try {
    const stat = statSync(inputFifo);
    pipeWritable = stat.isFIFO();
  } catch {
    // not found or not a FIFO
  }

  let logSizeBytes: number | null = null;
  let logLastLine: string | null = null;
  if (existsSync(logPath)) {
    try {
      logSizeBytes = statSync(logPath).size;
      logLastLine = getLastLogLine(logPath);
    } catch {
      // permission or read error
    }
  }

  let responsesCount: number | null = null;
  if (existsSync(responsesPath)) {
    try {
      const content = readFileSync(responsesPath, "utf-8").trim();
      responsesCount = content ? content.split("\n").length : 0;
    } catch {
      // read error
    }
  }

  return {
    schemaVersion: SCHEMA_VERSION,
    status: "ok",
    session: name,
    alive,
    pid,
    pipe: existsSync(inputFifo) ? inputFifo : null,
    pipeWritable,
    log: existsSync(logPath) ? logPath : null,
    logSizeBytes,
    logLastLine,
    responses: responsesPath,
    responsesCount,
  };
}

function listSessions(): SessionList {
  const sessions: SessionState[] = [];

  if (existsSync(SESSION_DIR)) {
    try {
      for (const entry of readdirSync(SESSION_DIR)) {
        const sdir = join(SESSION_DIR, entry);
        try {
          if (statSync(sdir).isDirectory()) {
            sessions.push(getSessionState(entry));
          }
        } catch {
          // skip unreadable entries
        }
      }
    } catch {
      // directory not readable
    }
  }

  return {
    schemaVersion: SCHEMA_VERSION,
    status: "ok",
    sessions,
  };
}

// --- CLI -------------------------------------------------------------------

const args = process.argv.slice(2);

if (args.includes("--list")) {
  console.log(JSON.stringify(listSessions(), null, 2));
  process.exit(0);
}

const sessionIdx = args.indexOf("--session");
const sessionName =
  sessionIdx >= 0 && args[sessionIdx + 1] ? args[sessionIdx + 1] : "default";

const state = getSessionState(sessionName);
console.log(JSON.stringify(state, null, 2));
process.exit(state.status === "ok" && state.alive ? 0 : 1);
