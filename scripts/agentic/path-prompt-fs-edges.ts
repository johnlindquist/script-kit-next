#!/usr/bin/env bun
import { chmodSync, existsSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { spawnSync } from "node:child_process";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = "path-prompt-fs-edges";
const timeoutMs = 8000;
const fixtureRoot = join(tmpdir(), `sk-path-joh-61-${process.pid}`);

function runSession(args: string[]): Json {
  const result = spawnSync(sessionScript, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
  });
  const stdout = result.stdout.trim();
  if (!stdout) {
    throw new Error(
      `session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`,
    );
  }
  let parsed: Json;
  try {
    parsed = JSON.parse(stdout);
  } catch (error) {
    throw new Error(`invalid JSON from session.sh: ${stdout}\n${String(error)}`);
  }
  if (result.status !== 0 || parsed.status === "error") {
    throw new Error(
      `session.sh ${args.join(" ")} failed: ${JSON.stringify(parsed)} stderr=${result.stderr.trim()}`,
    );
  }
  return parsed;
}

function send(command: Json): Json {
  return runSession([
    "send",
    session,
    JSON.stringify(command),
    "--await-parse",
    "--timeout",
    String(timeoutMs),
  ]);
}

function rpc(command: Json, expect: string): Json {
  const envelope = runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeoutMs),
  ]);
  return envelope.response;
}

function getState(tag: string): Json {
  const state = rpc(
    {
      type: "getState",
      requestId: `path-fs-state-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
  if (state.type !== "stateResult" || state.promptType !== "path") {
    throw new Error(`${tag}: expected path stateResult, got ${JSON.stringify(state)}`);
  }
  if (!state.path || typeof state.path !== "object") {
    throw new Error(`${tag}: missing state.path receipt: ${JSON.stringify(state)}`);
  }
  return state;
}

function getElements(tag: string): Json {
  const elements = rpc(
    {
      type: "getElements",
      requestId: `path-fs-elements-${tag}-${Date.now()}`,
      limit: 50,
    },
    "elementsResult",
  );
  if (elements.type !== "elementsResult") {
    throw new Error(`${tag}: expected elementsResult, got ${JSON.stringify(elements)}`);
  }
  return elements;
}

function pathStatusKind(elements: Json): string | undefined {
  return (elements.elements ?? []).find((element: Json) => element.kind === "path_status")
    ?.statusKind;
}

function assertEqual(actual: unknown, expected: unknown, label: string): void {
  if (actual !== expected) {
    throw new Error(`${label}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

function openPath(id: string, startPath: string): void {
  send({ type: "path", id, startPath, hint: `JOH-61 ${id}` });
}

function proveCase(tag: string, startPath: string, expected: Json): Json {
  openPath(`path-fs-${tag}`, startPath);
  const state = getState(tag);
  const elements = getElements(tag);

  for (const [key, value] of Object.entries(expected)) {
    assertEqual(state.path[key], value, `${tag}: state.path.${key}`);
  }
  assertEqual(pathStatusKind(elements), state.path.status.kind, `${tag}: elements statusKind`);
  return { tag, path: state.path, statusKind: pathStatusKind(elements) };
}

function main(): void {
  rmSync(fixtureRoot, { recursive: true, force: true });
  mkdirSync(fixtureRoot, { recursive: true });
  const empty = join(fixtureRoot, "empty");
  const missing = join(fixtureRoot, "missing");
  const denied = join(fixtureRoot, "denied");
  const fileStart = join(fixtureRoot, "file-start.txt");
  mkdirSync(empty);
  mkdirSync(denied);
  writeFileSync(fileStart, "file start");

  runSession(["start", session]);

  const receipts: Json[] = [];
  try {
    receipts.push(
      proveCase("missing", missing, {
        loadStatus: "error",
        loadErrorKind: "missing",
        statusMessage: "Path not found.",
        visibleEntryCount: 0,
      }),
    );
    receipts.push(
      proveCase("empty", empty, {
        loadStatus: "empty",
        emptyKind: "emptyDirectory",
        statusMessage: "This folder is empty.",
        visibleEntryCount: 0,
      }),
    );
    receipts.push(
      proveCase("file-start", fileStart, {
        startPathKind: "file",
        loadStatus: "loaded",
        selectedPath: fileStart,
      }),
    );

    let deniedReceipt: Json;
    if (typeof process.getuid === "function" && process.getuid() === 0) {
      deniedReceipt = {
        tag: "permission-denied",
        skippedRuntime: true,
        reason: "running as root",
      };
    } else {
      chmodSync(denied, 0o000);
      try {
        deniedReceipt = proveCase("permission-denied", denied, {
          loadStatus: "error",
          loadErrorKind: "permissionDenied",
          statusMessage: "Permission denied.",
          visibleEntryCount: 0,
        });
      } finally {
        chmodSync(denied, 0o755);
      }
    }
    receipts.push(deniedReceipt);
  } finally {
    send({ type: "simulateKey", key: "escape" });
    runSession(["stop", session]);
    if (existsSync(denied)) {
      chmodSync(denied, 0o755);
    }
    rmSync(fixtureRoot, { recursive: true, force: true });
  }

  console.log(JSON.stringify({ status: "ok", receipts }, null, 2));
}

main();
