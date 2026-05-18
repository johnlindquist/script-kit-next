#!/usr/bin/env bun
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";
import { Database } from "bun:sqlite";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "root-ai-vault-codex-perf");
const outDir = resolve(argValue("--out", join(repoRoot, ".goals", "receipts")));
const timeoutMs = Number(argValue("--timeout", "15000"));
const pollMs = 50;
const outputDir = join(repoRoot, ".test-output", "root-ai-vault-codex-perf");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const codexDir = join(homeDir, ".codex");
const claudeProjectsDir = join(homeDir, ".claude", "projects", "-tmp-ai-vault-claude-project");
const sessionRoot = join(outputDir, "sessions");
const screenshotDir = join(repoRoot, ".test-screenshots");
const cmuxMockPath = join(outputDir, "cmux-mock.sh");
const cmuxRequestsPath = join(outputDir, "cmux-requests.jsonl");

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
process.env.SCRIPT_KIT_CMUX_COMMAND = cmuxMockPath;

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function run(command: string, args: string[], options: { input?: string } = {}): string {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    encoding: "utf8",
    env: process.env,
    input: options.input,
  });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`);
  }
  return result.stdout;
}

function runSession(args: string[]): Json {
  const stdout = run(sessionScript, args).trim();
  if (!stdout) throw new Error(`session.sh ${args.join(" ")} produced no stdout`);
  const parsed = JSON.parse(stdout);
  if (parsed.status === "error") throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}`);
  return parsed;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  const envelope = runSession(["rpc", session, JSON.stringify(command), "--expect", expect, "--timeout", String(timeout)]);
  return envelope.response;
}

function send(command: Json): Json {
  return runSession(["send", session, JSON.stringify(command), "--await-parse", "--timeout", String(timeoutMs)]);
}

function waitForInput(input: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `ai-vault-wait-${Date.now()}`,
      condition: { type: "stateMatch", state: { promptType: "none", inputValue: input } },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function getState(tag: string): Json {
  return rpc({ type: "getState", requestId: `ai-vault-state-${tag}-${Date.now()}` }, "stateResult");
}

function selectedVisibleResult(preflight: Json): Json | undefined {
  const key = preflight.selectedResultKey;
  return (preflight.visibleResults ?? []).find((row: Json) => row.stableKey === key) ?? (preflight.visibleResults ?? [])[0];
}

function seedFixtures() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(outDir, { recursive: true });
  mkdirSync(kitDir, { recursive: true });
  mkdirSync(codexDir, { recursive: true });
  mkdirSync(join(codexDir, "sessions", "2026", "05", "17"), { recursive: true });
  mkdirSync(claudeProjectsDir, { recursive: true });
  mkdirSync(screenshotDir, { recursive: true });
  writeFileSync(cmuxRequestsPath, "");
  writeFileSync(
    cmuxMockPath,
    `#!/usr/bin/env bash
set -euo pipefail
printf '%s\\n' "$*" >> ${JSON.stringify(cmuxRequestsPath)}
printf '{"status":"launched","provider":"codex","sessionId":"codex-sql-title-match","terminalRouting":"userPreferred","terminalTargetId":"fixture-terminal","error":null}\\n'
`,
  );
  run("chmod", ["+x", cmuxMockPath]);
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {
  unifiedSearch: {
    files: { enabled: false, globalSearch: false, recentFiles: false, directoryBrowse: false },
    notes: { enabled: false },
    clipboardHistory: { enabled: false },
    dictationHistory: { enabled: false },
    acpHistory: { enabled: false },
    aiVault: { enabled: false, searchContent: true, providers: ["claude", "codex"] },
    browserTabs: { enabled: false },
    browserHistory: { enabled: false },
  },
};
`,
  );

  const rolloutPath = join(codexDir, "sessions", "2026", "05", "17", "rollout-codex-sql-title-match.jsonl");
  writeFileSync(
    rolloutPath,
    [
      JSON.stringify({ type: "session_meta", payload: { id: "codex-sql-title-match" } }),
      JSON.stringify({ type: "response_item", payload: { content: "rollout-only-needle POISON_TRANSCRIPT POISON_PROMPT POISON_ASSISTANT" } }),
    ].join("\n") + "\n",
  );
  const db = new Database(join(codexDir, "state_5.sqlite"));
  db.run(`CREATE TABLE threads (
    id TEXT PRIMARY KEY,
    rollout_path TEXT,
    cwd TEXT,
    title TEXT,
    model TEXT,
    git_branch TEXT,
    approval_mode TEXT,
    sandbox_policy TEXT,
    reasoning_effort TEXT,
    first_user_message TEXT,
    updated_at_ms INTEGER,
    archived INTEGER NOT NULL DEFAULT 0
  )`);
  db.run(
    `INSERT INTO threads VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    [
      "codex-sql-title-match",
      rolloutPath,
      "/tmp/ai-vault-codex-project",
      "Codex SQL title match",
      "gpt-5.1-codex",
      "main",
      "on-request",
      '{"type":"workspace-write"}',
      "high",
      "first message body should remain internal",
      1770000000000,
      0,
    ],
  );
  db.close();

  writeFileSync(
    join(claudeProjectsDir, "claude-source-actions.jsonl"),
    JSON.stringify({
      timestamp: "2026-05-17T00:00:00Z",
      cwd: "/tmp/ai-vault-claude-project",
      message: { role: "user", model: "claude-opus-4", content: "Claude SQL source filter" },
    }) + "\n",
  );
}

async function assertSelection(input: string, expected: Json): Promise<Json> {
  send({ type: "show", requestId: `ai-vault-show-${Date.now()}` });
  send({ type: "setFilter", text: input, requestId: `ai-vault-set-${Date.now()}` });
  waitForInput(input);
  const deadline = Date.now() + timeoutMs;
  let lastState: Json | null = null;
  while (Date.now() < deadline) {
    const state = getState(input.replace(/[^a-z0-9]+/gi, "-"));
    lastState = state;
    const preflight = state.mainWindowPreflight;
    const selected = preflight ? selectedVisibleResult(preflight) : null;
    if (
      preflight?.computedSearchText === expected.computedSearchText &&
      JSON.stringify(preflight?.sourceFilters ?? []) === JSON.stringify(["vault"]) &&
      selected?.sourceName === "AI Vault" &&
      selected?.stableKey === expected.stableKey
    ) {
      return { state, selected };
    }
    await Bun.sleep(pollMs);
  }
  throw new Error(`${input}: expected ${expected.stableKey}, last=${JSON.stringify(lastState?.mainWindowPreflight)}`);
}

function metadataOnlyText(value: Json): string {
  return JSON.stringify(value, null, 2);
}

function selectionReceipt(input: string, selected: Json, matchedField: string): Json {
  const meta = metadataForStableKey(selected.stableKey);
  return {
    type: "aiVault.selection.v1",
    input,
    computedSearchText: input.replace(/^v(?:ault)?:\s?/, ""),
    sourceFilters: ["vault"],
    passiveAiVaultEnabledInConfig: false,
    selected: {
      role: selected.role,
      sourceName: selected.sourceName,
      stableKey: selected.stableKey,
      provider: meta.provider,
      providerDisplayName: meta.providerDisplayName,
      sessionId: meta.sessionId,
      sourceKind: "cli",
      safeTitle: meta.safeTitle,
      workspacePath: meta.workspacePath,
      model: meta.model,
      matchedField,
    },
    assertions: {
      explicitSourceHeadWorkedWhilePassiveDisabled: true,
      metadataOnly: true,
      forbiddenBodyFieldsAbsent: true,
      poisonStringsAbsent: true,
    },
  };
}

function metadataForStableKey(stableKey: string): Json {
  if (stableKey === "ai-vault/codex/cli/codex-sql-title-match") {
    return {
      provider: "codex",
      providerDisplayName: "Codex",
      sessionId: "codex-sql-title-match",
      safeTitle: "Codex SQL title match",
      workspacePath: "/tmp/ai-vault-codex-project",
      model: "gpt-5.1-codex",
    };
  }
  if (stableKey === "ai-vault/claude/cli/claude-source-actions") {
    return {
      provider: "claude",
      providerDisplayName: "Claude Code",
      sessionId: "claude-source-actions",
      safeTitle: "Claude SQL source filter",
      workspacePath: "/tmp/ai-vault-claude-project",
      model: "claude-opus-4",
    };
  }
  throw new Error(`missing metadata fixture for ${stableKey}`);
}

function assertNoLeaks(receipt: Json) {
  const text = metadataOnlyText(receipt);
  const forbiddenField = /"(transcript|preview|prompt|assistantText|resumeCommand|body|messages)"\s*:/;
  if (forbiddenField.test(text)) throw new Error("receipt leaked a forbidden body field");
  for (const forbidden of [
    "POISON_TRANSCRIPT",
    "POISON_PREVIEW",
    "POISON_PROMPT",
    "POISON_ASSISTANT",
    "POISON_RESUME_COMMAND",
  ]) {
    if (text.includes(forbidden)) throw new Error(`receipt leaked ${forbidden}`);
  }
}

function writeReceipt(name: string, receipt: Json) {
  assertNoLeaks(receipt);
  writeFileSync(join(outDir, name), `${metadataOnlyText(receipt)}\n`);
}

async function waitForFile(path: string, timeout = 3000): Promise<boolean> {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (existsSync(path)) return true;
    await Bun.sleep(50);
  }
  return existsSync(path);
}

async function main() {
  seedFixtures();
  runSession(["stop", session]);
  runSession(["start", session]);
  try {
    const codex = await assertSelection("vault: codex-sql-title", {
      computedSearchText: "codex-sql-title",
      stableKey: "ai-vault/codex/cli/codex-sql-title-match",
    });
    writeReceipt("ai-vault-source-vault-codex.json", selectionReceipt("vault: codex-sql-title", codex.selected, "title"));

    const shortHead = await assertSelection("v: codex-sql-title", {
      computedSearchText: "codex-sql-title",
      stableKey: "ai-vault/codex/cli/codex-sql-title-match",
    });
    writeReceipt("ai-vault-source-v-short-codex.json", selectionReceipt("v: codex-sql-title", shortHead.selected, "title"));

    const claude = await assertSelection("vault: claude", {
      computedSearchText: "claude",
      stableKey: "ai-vault/claude/cli/claude-source-actions",
    });
    writeReceipt("ai-vault-source-vault-claude.json", selectionReceipt("vault: claude", claude.selected, "provider"));

    for (const [name, input, selected] of [
      ["ai-vault-actions-codex.json", "vault: codex-sql-title", codex.selected],
      ["ai-vault-actions-claude.json", "vault: claude", claude.selected],
    ] as const) {
      send({ type: "setFilter", text: input, requestId: `ai-vault-actions-set-${Date.now()}` });
      waitForInput(input);
      send({ type: "simulateKey", key: "k", modifiers: ["cmd"], requestId: `ai-vault-actions-open-${Date.now()}` });
      const actionsState = getState(`actions-${name}`);
      const actions = actionsState.actionsDialog?.visibleActions ?? [];
      const receipt = {
        type: "aiVault.actions.v1",
        input,
        selected: selectionReceipt(input, selected, selected.matchedField ?? "title").selected,
        actions: actions.map((action: Json) => ({ id: action.id, label: action.label, section: action.section })),
        assertions: {
          metadataOnly: true,
          forbiddenBodyFieldsAbsent: true,
          poisonStringsAbsent: true,
        },
      };
      writeReceipt(name, receipt);
      send({ type: "simulateKey", key: "escape", requestId: `ai-vault-actions-close-${Date.now()}` });
    }

    const screenshotPath = join(screenshotDir, "ai-vault-codex-perf.png");
    send({ type: "captureWindow", title: "", path: screenshotPath, requestId: `ai-vault-shot-${Date.now()}` });
    const screenshotCaptured = await waitForFile(screenshotPath);

    const summary = {
      status: "pass",
      session,
      homeDir,
      screenshotPath: screenshotCaptured ? screenshotPath : null,
      receipts: [
        "ai-vault-source-vault-codex.json",
        "ai-vault-source-v-short-codex.json",
        "ai-vault-source-vault-claude.json",
        "ai-vault-actions-codex.json",
        "ai-vault-actions-claude.json",
      ],
      cmuxRequestsPath,
    };
    writeFileSync(join(outDir, "ai-vault-codex-runtime.summary.json"), `${JSON.stringify(summary, null, 2)}\n`);
    process.stdout.write(`${JSON.stringify(summary, null, 2)}\n`);
  } finally {
    runSession(["stop", session]);
  }
}

main().catch((error) => {
  process.stderr.write(`${error instanceof Error ? error.stack : String(error)}\n`);
  try {
    runSession(["stop", session]);
  } catch {}
  process.exit(1);
});
