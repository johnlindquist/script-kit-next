#!/usr/bin/env bun
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "source-chip-pagination-auto-proof");
const query = argValue("--query", `sourcechippage${Date.now()}`);
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = 50;
const outputDir = join(repoRoot, ".test-output", "source-chip-pagination-auto-proof");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const sessionRoot = join(outputDir, "sessions");

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
  query,
  delayMs: 0,
  results: Array.from({ length: 30 }, (_, index) => ({
    path: `/tmp/${query}-${String(index + 1).padStart(2, "0")}.mp4`,
    name: `${query}-${String(index + 1).padStart(2, "0")}.mp4`,
    fileType: "video",
    size: 1024 + index,
    modified: Date.now() - index,
  })),
});

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function runSession(args: string[]): Json {
  const result = spawnSync(sessionScript, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
  });
  const stdout = result.stdout.trim();
  if (!stdout) {
    throw new Error(`session.sh ${args.join(" ")} produced no stdout; stderr=${result.stderr.trim()}`);
  }
  const parsed = JSON.parse(stdout);
  if (result.status !== 0 || parsed.status === "error") {
    throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}\nstderr=${result.stderr.trim()}`);
  }
  return parsed;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  return runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeout),
  ]).response;
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

function waitForInput(input: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `source-chip-page-wait-input-${Date.now()}`,
      condition: { type: "stateMatch", state: { promptType: "none", inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function getElements(tag: string): Json {
  const response = rpc(
    {
      type: "getElements",
      requestId: `source-chip-page-elements-${tag}-${Date.now()}`,
    },
    "elementsResult",
  );
  if (response.type !== "elementsResult") {
    throw new Error(`expected elementsResult, got ${JSON.stringify(response)}`);
  }
  return response;
}

function fileRows(elements: Json): Json[] {
  return (elements.elements ?? []).filter(
    (element: Json) => element.role === "row" && element.source === "files" && element.selectable === true,
  );
}

function statusRows(elements: Json): Json[] {
  return (elements.elements ?? []).filter(
    (element: Json) => element.role === "status" && element.kind === "sourceStatus" && element.source === "files",
  );
}

function assertStatusMetadata(status: Json | undefined, label: string) {
  if (!status) {
    throw new Error(`${label}: missing Files source status metadata`);
  }
  if (Object.prototype.hasOwnProperty.call(status, "index")) {
    throw new Error(`${label}: source status must not occupy a list index ${JSON.stringify(status)}`);
  }
  if (status.selectable !== false) {
    throw new Error(`${label}: source status must be non-selectable ${JSON.stringify(status)}`);
  }
}

async function waitForFileRowsAtLeast(count: number): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let last: Json | null = null;
  while (Date.now() < deadline) {
    last = getElements(`wait-${count}`);
    if (fileRows(last).length >= count) {
      return last;
    }
    await Bun.sleep(pollMs);
  }
  throw new Error(`timed out waiting for ${count} file rows; last=${JSON.stringify(last)}`);
}

async function main() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(kitDir, { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {
  unifiedSearch: {
    notes: { enabled: false },
    clipboardHistory: { enabled: false },
    dictationHistory: { enabled: false },
    acpHistory: { enabled: false },
    browserTabs: { enabled: false },
    browserHistory: { enabled: false },
  },
};
`,
  );

  runSession(["stop", session]);
  runSession(["start", session]);

  try {
    const input = `f: ${query}`;
    send({ type: "setFilter", text: input, requestId: `source-chip-page-set-${Date.now()}` });
    waitForInput(input);
    const before = await waitForFileRowsAtLeast(12);
    const beforeRows = fileRows(before);
    const beforeStatus = statusRows(before);
    if (beforeRows.length !== 12) {
      throw new Error(`expected initial page to show exactly 12 files, got ${beforeRows.length}`);
    }
    if (beforeStatus.length !== 1 || beforeStatus[0].selectable !== false) {
      throw new Error(`expected one non-selectable Files status metadata entry before paging, got ${JSON.stringify(beforeStatus)}`);
    }
    assertStatusMetadata(beforeStatus[0], "before paging");

    for (let index = 0; index < 10; index += 1) {
      send({
        type: "simulateKey",
        key: "down",
        modifiers: [],
        requestId: `source-chip-page-down-${index}-${Date.now()}`,
      });
      await Bun.sleep(20);
    }

    const after = await waitForFileRowsAtLeast(24);
    const afterRows = fileRows(after);
    const afterStatus = statusRows(after);
    if (afterStatus.length !== 1 || afterStatus[0].selectable !== false) {
      throw new Error(`expected one non-selectable Files status metadata entry after paging, got ${JSON.stringify(afterStatus)}`);
    }
    assertStatusMetadata(afterStatus[0], "after paging");
    const selected = after.elements.find((element: Json) => element.selected);
    if (!selected || selected.role !== "row" || selected.source !== "files") {
      throw new Error(`expected selected row to stay on a Files row, got ${JSON.stringify(selected)}`);
    }

    const receipt = {
      schemaVersion: 1,
      status: "pass",
      session,
      query,
      input,
      before: {
        fileRows: beforeRows.length,
        status: beforeStatus[0],
      },
      after: {
        fileRows: afterRows.length,
        status: afterStatus[0],
        selected,
      },
    };
    writeFileSync(join(outputDir, "receipt.json"), `${JSON.stringify(receipt, null, 2)}\n`);
    process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
  } finally {
    runSession(["stop", session]);
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  process.exit(1);
});
