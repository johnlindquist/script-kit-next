#!/usr/bin/env bun
import { mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { join, resolve } from "node:path";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const sessionScript = join(repoRoot, "scripts/agentic/session.sh");
const session = argValue("--session", "root-source-actions-matrix");
const query = argValue("--query", `codexactions${Date.now()}`);
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "50"));
const keepSession = process.argv.includes("--keep-session");
const outputDir = join(repoRoot, ".test-output", "root-source-actions-matrix");
const homeDir = join(outputDir, "home");
const kitDir = join(homeDir, ".scriptkit");
const dbDir = join(kitDir, "db");
const sessionRoot = join(outputDir, "sessions");
const recentDir = join(outputDir, "recent");
const scriptsDir = join(kitDir, "plugins", "main", "scripts");
const fixtureAppPath = join(outputDir, "Fixture Actions.app");
const cmuxMockPath = join(outputDir, "cmux-mock.sh");
const cmuxRequestsPath = join(outputDir, "cmux-requests.jsonl");
const aiVaultPoisonStrings = [
  "POISON_TRANSCRIPT",
  "POISON_PREVIEW",
  "POISON_PROMPT",
  "POISON_ASSISTANT",
  "POISON_RESUME_COMMAND",
];
const aiVaultSensitiveRequestFields = [
  "transcript",
  "preview",
  "prompt",
  "assistantText",
  "resumeCommand",
];
// AI Vault Codex/Claude local-provider actions are covered by
// root-ai-vault-codex-perf.ts: codex-sql-title-match, claude-source-actions.

process.env.HOME = homeDir;
process.env.SK_PATH = kitDir;
process.env.SCRIPT_KIT_SESSION_DIR = sessionRoot;
process.env.SCRIPT_KIT_SESSION_READY_TIMEOUT_MS = "10000";
process.env.SCRIPT_KIT_AGENTIC_KEEP_ACTIONS_WINDOW_OPEN = "1";
process.env.SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER = JSON.stringify({
  query,
  delayMs: 0,
  results: [
    {
      path: `/tmp/${query}-file-result.txt`,
      name: `${query}-file-result.txt`,
      fileType: "document",
      size: 42,
      modified: Date.now(),
    },
  ],
});
process.env.SCRIPT_KIT_BROWSER_TABS_TEST_PROVIDER = JSON.stringify([
  {
    browser_name: "Google Chrome",
    browser_bundle_id: "com.google.Chrome",
    window_index: 1,
    tab_index: 1,
    title: `${query} browser tab`,
    url: `https://example.com/${query}/tab`,
  },
]);
process.env.SCRIPT_KIT_WINDOW_SEARCH_TEST_PROVIDER = JSON.stringify([
  {
    id: 4242,
    app: "Script Fixture",
    title: `${query} script window`,
    pid: 4242,
    bounds: { x: 20, y: 20, width: 1280, height: 720 },
  },
]);
process.env.SCRIPT_KIT_AI_VAULT_TEST_PROVIDER = JSON.stringify([
  {
    provider: "hermes-agent",
    providerDisplayName: "Hermes Agent",
    sessionId: "vault-source-actions",
    sourceKind: "cli",
    safeTitle: `${query} vault session`,
    workspacePath: `/tmp/${query}-workspace`,
    model: "fixture-model",
    modifiedAt: new Date().toISOString(),
    matchedField: "title",
    stableKey: "ai-vault/hermes-agent/cli/vault-source-actions",
    score: 100,
    transcript: aiVaultPoisonStrings[0],
    preview: aiVaultPoisonStrings[1],
    prompt: aiVaultPoisonStrings[2],
    assistantText: aiVaultPoisonStrings[3],
    resumeCommand: aiVaultPoisonStrings[4],
  },
]);
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
    throw new Error(
      `${command} ${args.join(" ")} failed\nstdout=${result.stdout}\nstderr=${result.stderr}`,
    );
  }
  return result.stdout;
}

function runSession(args: string[]): Json {
  const stdout = run(sessionScript, args).trim();
  if (!stdout) {
    throw new Error(`session.sh ${args.join(" ")} produced no stdout`);
  }
  const parsed = JSON.parse(stdout);
  if (parsed.status === "error") {
    throw new Error(`session.sh ${args.join(" ")} failed: ${stdout}`);
  }
  return parsed;
}

function rpc(command: Json, expect: string, timeout = timeoutMs): Json {
  const envelope = runSession([
    "rpc",
    session,
    JSON.stringify(command),
    "--expect",
    expect,
    "--timeout",
    String(timeout),
  ]);
  return envelope.response;
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

function getState(tag: string): Json {
  return rpc(
    {
      type: "getState",
      requestId: `root-source-actions-state-${tag}-${Date.now()}`,
    },
    "stateResult",
  );
}

function waitForInput(input: string): Json {
  return rpc(
    {
      type: "waitFor",
      requestId: `root-source-actions-wait-${Date.now()}`,
      condition: {
        type: "stateMatch",
        state: {
          promptType: "none",
          inputValue: input,
        },
      },
      timeout: timeoutMs,
      pollInterval: pollMs,
    },
    "waitForResult",
  );
}

function sql(path: string, input: string) {
  run("sqlite3", [path], { input });
}

function seedFixtures() {
  rmSync(outputDir, { recursive: true, force: true });
  mkdirSync(dbDir, { recursive: true });
  mkdirSync(recentDir, { recursive: true });
  mkdirSync(scriptsDir, { recursive: true });
  writeFileSync(cmuxRequestsPath, "");
  mkdirSync(fixtureAppPath, { recursive: true });
  writeFileSync(join(fixtureAppPath, "fixture.txt"), query);
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {
  unifiedSearch: {
    files: { enabled: false, globalSearch: false, recentFiles: false, directoryBrowse: false },
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
  writeFileSync(
    cmuxMockPath,
    `#!/usr/bin/env bash
set -euo pipefail
printf '%s\\n' "$*" >> ${JSON.stringify(cmuxRequestsPath)}
if [[ "\${1:-}" == "ai-vault" && "\${2:-}" == "resume" ]]; then
  printf '{"status":"launched","provider":"hermes-agent","sessionId":"vault-source-actions","terminalRouting":"userPreferred","terminalTargetId":"fixture-terminal","error":null}\\n'
else
  printf '{"status":"opened","provider":"hermes-agent","sessionId":"vault-source-actions","terminalRouting":"reveal","terminalTargetId":null,"error":null}\\n'
fi
`,
  );
  run("chmod", ["+x", cmuxMockPath]);

  const recentFilePath = join(recentDir, `${query}-recent-file.txt`);
  writeFileSync(recentFilePath, `${query} recent file body\n`);
  writeFileSync(
    join(kitDir, "frecency.json"),
    `${JSON.stringify({
      entries: {
        [`file/${recentFilePath}`]: {
          count: 3,
          last_used: Math.floor(Date.now() / 1000),
        },
      },
    })}\n`,
  );

  writeFileSync(
    join(scriptsDir, `${query}-build.ts`),
    `// Name: ${query} Build Script
// Description: Fixture script for root source action matrix.
await div("fixture");
`,
  );

  const now = new Date().toISOString();
  const noteId = "22222222-2222-4222-8222-222222222222";
  sql(
    join(dbDir, "notes.sqlite"),
    `
CREATE TABLE notes (
  id TEXT PRIMARY KEY,
  title TEXT NOT NULL DEFAULT '',
  content TEXT NOT NULL DEFAULT '',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  deleted_at TEXT,
  is_pinned INTEGER NOT NULL DEFAULT 0,
  sort_order INTEGER NOT NULL DEFAULT 0
);
CREATE VIRTUAL TABLE notes_fts USING fts5(title, content, content='notes', content_rowid='rowid');
INSERT INTO notes (id, title, content, created_at, updated_at, deleted_at, is_pinned, sort_order)
VALUES ('${noteId}', '${query} note title', 'Starter content for root source actions', '${now}', '${now}', NULL, 0, 0);
INSERT INTO notes_fts(rowid, title, content)
SELECT rowid, title, content FROM notes WHERE id = '${noteId}';
`,
  );

  sql(
    join(dbDir, "clipboard-history.sqlite"),
    `
CREATE TABLE history (
  id TEXT PRIMARY KEY,
  content TEXT NOT NULL,
  content_hash TEXT,
  content_type TEXT NOT NULL DEFAULT 'text',
  timestamp INTEGER NOT NULL,
  pinned INTEGER DEFAULT 0,
  ocr_text TEXT,
  text_preview TEXT,
  image_width INTEGER,
  image_height INTEGER,
  byte_size INTEGER
);
INSERT INTO history (
  id, content, content_hash, content_type, timestamp, pinned, ocr_text, text_preview, image_width, image_height, byte_size
) VALUES (
  'clip-source-actions', '${query} skip clipboard text', 'fixture-actions-hash', 'text', ${Date.now()}, 0, NULL, '${query} skip clipboard text', NULL, NULL, ${query.length + 20}
);
`,
  );

  writeFileSync(
    join(kitDir, "dictation-history.jsonl"),
    `${JSON.stringify({
      id: "dictation-source-actions",
      timestamp: now,
      transcript: `${query} dictation transcript`,
      preview: `${query} dictation transcript`,
      target: "Main Filter",
      audio_duration_ms: 1200,
    })}\n`,
  );

  writeFileSync(
    join(kitDir, "acp-history.jsonl"),
    `${JSON.stringify({
      timestamp: now,
      first_message: `${query} conversation prompt`,
      message_count: 2,
      session_id: "acp-source-actions",
      title: `${query} conversation prompt`,
      preview: `${query} conversation reply`,
      search_text: `${query} conversation prompt ${query} conversation reply`,
    })}\n`,
  );

  const historyDir = join(homeDir, "Library/Application Support/Google/Chrome/Default");
  mkdirSync(historyDir, { recursive: true });
  const chromeTime = (Math.floor(Date.now() / 1000) + 11644473600) * 1000000;
  sql(
    join(historyDir, "History"),
    `
CREATE TABLE urls (
  id INTEGER PRIMARY KEY,
  url TEXT NOT NULL,
  title TEXT,
  visit_count INTEGER NOT NULL DEFAULT 0,
  typed_count INTEGER NOT NULL DEFAULT 0,
  last_visit_time INTEGER NOT NULL DEFAULT 0
);
INSERT INTO urls (id, url, title, visit_count, typed_count, last_visit_time)
VALUES (1, 'https://example.com/${query}/history', '${query} browser history', 7, 2, ${chromeTime});
`,
  );

  sql(
    join(dbDir, "apps.sqlite"),
    `
CREATE TABLE apps (
  bundle_id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  path TEXT NOT NULL UNIQUE,
  icon_blob BLOB,
  mtime INTEGER NOT NULL,
  last_seen INTEGER NOT NULL
);
CREATE INDEX idx_apps_path ON apps(path);
INSERT INTO apps (bundle_id, name, path, icon_blob, mtime, last_seen)
VALUES ('com.example.fixture-actions-calendar', 'Calendar Fixture', '${fixtureAppPath.replaceAll("'", "''")}', NULL, ${Date.now()}, ${Date.now()});
`,
  );
}

const cases: Json[] = [
  {
    id: "files",
    sourceHead: "f:",
    query,
    expectedFilters: ["files"],
    role: "rootFile",
    typeLabel: "File",
    sourceName: "Files",
    stableKeyIncludes: query,
    expectedActions: [
      ["root_file_open", "Open File"],
      ["root_file_reveal_in_finder", "Reveal in Finder"],
      ["root_file_copy_path", "Copy Path"],
      ["root_file_copy_name", "Copy Name"],
      ["root_file_quick_look", "Quick Look"],
    ],
  },
  {
    id: "notes",
    sourceHead: "n:",
    query: "not",
    attachedQuery: "not",
    expectedFilters: ["notes"],
    role: "rootPassive",
    typeLabel: "Note",
    sourceName: "Notes",
    stableKey: "note/22222222-2222-4222-8222-222222222222",
    expectedActions: [
      ["root_note_open", "Open Note"],
      ["root_note_copy_title", "Copy Note Title"],
      ["root_note_copy_id", "Copy Note ID"],
    ],
  },
  {
    id: "clipboard",
    sourceHead: "c:",
    query: "skip",
    attachedQuery: "skip",
    expectedFilters: ["clipboard"],
    role: "rootPassive",
    typeLabel: "Clipboard",
    sourceName: "Clipboard History",
    stableKey: "clipboard-history/clip-source-actions",
    expectedActions: [
      ["root_clipboard_paste", "Paste Clipboard"],
      ["root_clipboard_copy", "Copy to Clipboard"],
      ["root_clipboard_attach_to_ai", "Attach to Agent Chat"],
      ["root_clipboard_pin", "Pin"],
      ["root_clipboard_quick_look", "Quick Look"],
      ["root_clipboard_delete", "Delete Clipboard Entry"],
    ],
    destructiveActions: ["root_clipboard_delete"],
  },
  {
    id: "tabs",
    sourceHead: "t:",
    query,
    expectedFilters: ["tabs"],
    role: "rootPassive",
    typeLabel: "Browser Tab",
    sourceName: "Browser Tabs",
    stableKeyIncludes: `browser-tab/com.google.Chrome/1/1/https://example.com/${query}/tab`,
    expectedActions: [
      ["root_browser_tab_switch", "Switch to Tab"],
      ["root_browser_tab_copy_url", "Copy URL"],
      ["root_browser_tab_copy_title", "Copy Title"],
      ["root_browser_tab_copy_title_url", "Copy Title and URL"],
      ["root_browser_tab_open_url", "Open URL in Browser"],
    ],
  },
  {
    id: "history",
    sourceHead: "h:",
    query: `https://example.com/${query}`,
    attachedQuery: `https://example.com/${query}`,
    expectedFilters: ["history"],
    role: "rootPassive",
    typeLabel: "Browser History",
    sourceName: "Browser History",
    stableKeyIncludes: "browser-history/",
    expectedActions: [
      ["root_browser_history_open", "Open Page"],
      ["root_browser_history_copy_url", "Copy URL"],
      ["root_browser_history_copy_title", "Copy Title"],
      ["root_browser_history_copy_title_url", "Copy Title and URL"],
    ],
  },
  {
    id: "conversations",
    sourceHead: "ai:",
    query: "conversation",
    expectedFilters: ["conversations"],
    role: "rootPassive",
    typeLabel: "Agent Chat Conversation",
    sourceName: "Agent Chat Conversations",
    stableKey: "acp-history/acp-source-actions",
    expectedActions: [
      ["root_acp_history_resume", "Resume Conversation"],
      ["root_acp_history_copy_title", "Copy Conversation Title"],
      ["root_acp_history_copy_session_id", "Copy Session ID"],
      ["root_acp_history_copy_preview", "Copy Preview"],
    ],
  },
  {
    id: "vault",
    sourceHead: "v:",
    query: "vault",
    expectedFilters: ["vault"],
    role: "rootPassive",
    typeLabel: "Vault Conversation",
    sourceName: "AI Vault",
    stableKey: "ai-vault/hermes-agent/cli/vault-source-actions",
    expectedActions: [
      ["root_ai_vault_resume_preferred_terminal", "Resume in Preferred Terminal"],
      ["root_ai_vault_resume_new_terminal", "Resume in New Terminal"],
      ["root_ai_vault_copy_session_id", "Copy Session ID"],
      ["root_ai_vault_copy_provider", "Copy Provider"],
      ["root_ai_vault_copy_workspace_path", "Copy Workspace Path"],
      ["root_ai_vault_copy_title", "Copy Title"],
      ["root_ai_vault_reveal_in_cmux", "Reveal in cmux"],
    ],
  },
  {
    id: "dictation",
    sourceHead: "d:",
    query: "dictation",
    expectedFilters: ["dictation"],
    role: "rootPassive",
    typeLabel: "Dictation",
    sourceName: "Dictation History",
    stableKey: "dictation-history/dictation-source-actions",
    expectedActions: [
      ["root_dictation_paste", "Paste Dictation"],
      ["root_dictation_copy_transcript", "Copy Transcript"],
      ["root_dictation_attach_to_ai", "Attach to Agent Chat"],
      ["root_dictation_create_note", "Create Note from Transcript"],
      ["root_dictation_delete", "Delete Dictation"],
    ],
    destructiveActions: ["root_dictation_delete"],
  },
  {
    id: "scripts",
    sourceHead: "s:",
    query,
    expectedFilters: ["scripts"],
    role: "primary",
    typeLabel: "Script",
    stableKeyIncludes: `${query} Build Script`,
    expectedActions: [
      ["run_script", "Run"],
      ["toggle_info", "Show Info"],
      ["add_shortcut", "Add Keyboard Shortcut"],
      ["add_alias", "Add Alias"],
      ["toggle_favorite", "Add to Favorites"],
      ["edit_script", "Edit Script"],
      ["view_logs", "Show Logs"],
      ["reveal_in_finder", "Open in Finder"],
      ["file:open_in_quick_terminal", "Open in Quick Terminal"],
      ["copy_path", "Copy Path"],
      ["copy_content", "Copy Content"],
      ["copy_deeplink", "Share"],
      ["delete_script", "Delete Script?"],
      ["reload_scripts", "Reload Scripts"],
      ["settings", "Open Settings"],
    ],
    destructiveActions: ["delete_script"],
  },
  {
    id: "commands",
    sourceHead: "cmd:",
    query: "settings",
    expectedFilters: ["commands"],
    role: "primary",
    typeLabel: "Built-in",
    stableKey: "builtin/bluetooth-settings",
    expectedActions: [
      ["run_script", "Open Bluetooth Settings"],
      ["toggle_info", "Show Info"],
      ["add_shortcut", "Add Keyboard Shortcut"],
      ["add_alias", "Add Alias"],
      ["copy_deeplink", "Copy Deep Link"],
      ["reload_scripts", "Reload Scripts"],
      ["settings", "Open Settings"],
    ],
  },
  {
    id: "apps",
    sourceHead: "a:",
    query: "calendar",
    expectedFilters: ["apps"],
    role: "primary",
    typeLabel: "App",
    stableKey: "app/com.example.fixture-actions-calendar",
    expectedActions: [
      ["run_script", "Launch"],
      ["toggle_info", "Show Info"],
      ["add_shortcut", "Add Keyboard Shortcut"],
      ["add_alias", "Add Alias"],
      ["toggle_favorite", "Add to Favorites"],
      ["reveal_in_finder", "Show in Finder"],
      ["file:open_in_quick_terminal", "Open in Quick Terminal"],
      ["show_info_in_finder", "Show Info in Finder"],
      ["show_package_contents", "Show Package Contents"],
      ["copy_name", "Copy Name"],
      ["copy_path", "Copy Path"],
      ["copy_bundle_id", "Copy Bundle Identifier"],
      ["quit_app", "Quit Application"],
      ["restart_app", "Restart Application"],
      ["copy_deeplink", "Copy Deep Link"],
      ["force_quit_app", "Force Quit Application"],
      ["reload_scripts", "Reload Scripts"],
      ["settings", "Open Settings"],
    ],
    destructiveActions: ["force_quit_app"],
  },
  {
    id: "windows",
    sourceHead: "w:",
    query,
    expectedFilters: ["windows"],
    role: "primary",
    typeLabel: "Window",
    stableKey: `window:Script Fixture:${query} script window`,
    expectedActions: [
      ["root_window_switch", "Switch to Window"],
      ["root_window_copy_title", "Copy Window Title"],
      ["root_window_copy_app_name", "Copy App Name"],
      ["root_window_copy_descriptor", "Copy Window Descriptor"],
    ],
  },
];

function selectedVisibleResult(preflight: Json): Json | undefined {
  const key = preflight.selectedResultKey;
  return (preflight.visibleResults ?? []).find((row: Json) => row.stableKey === key)
    ?? (preflight.visibleResults ?? [])[0];
}

function assertSelectedFrame(state: Json, input: string, spec: Json, expectedSearchText: string): Json {
  const preflight = state.mainWindowPreflight;
  if (!preflight) {
    throw new Error(`${input}: missing mainWindowPreflight in ${JSON.stringify(state)}`);
  }
  if (preflight.computedSearchText !== expectedSearchText) {
    throw new Error(`${input}: expected computedSearchText ${expectedSearchText}, got ${preflight.computedSearchText}`);
  }
  if (JSON.stringify(preflight.sourceFilters) !== JSON.stringify(spec.expectedFilters)) {
    throw new Error(`${input}: expected filters ${JSON.stringify(spec.expectedFilters)}, got ${JSON.stringify(preflight.sourceFilters)}`);
  }
  const selected = selectedVisibleResult(preflight);
  if (!selected) {
    throw new Error(`${input}: no selected visible result in ${JSON.stringify(preflight)}`);
  }
  if (selected.role !== spec.role) {
    throw new Error(`${input}: expected selected role ${spec.role}, got ${selected.role} in ${JSON.stringify(selected)}`);
  }
  if (spec.typeLabel && selected.typeLabel !== spec.typeLabel) {
    throw new Error(`${input}: expected selected typeLabel ${spec.typeLabel}, got ${selected.typeLabel} in ${JSON.stringify(selected)}`);
  }
  if (spec.sourceName && selected.sourceName !== spec.sourceName) {
    throw new Error(`${input}: expected selected sourceName ${spec.sourceName}, got ${selected.sourceName} in ${JSON.stringify(selected)}`);
  }
  if (spec.stableKey && selected.stableKey !== spec.stableKey) {
    throw new Error(`${input}: expected stableKey ${spec.stableKey}, got ${selected.stableKey}`);
  }
  if (spec.stableKeyIncludes && !String(selected.stableKey ?? "").includes(spec.stableKeyIncludes)) {
    throw new Error(`${input}: expected stableKey to include ${spec.stableKeyIncludes}, got ${selected.stableKey}`);
  }
  return selected;
}

async function waitForSelected(input: string, spec: Json, expectedSearchText: string): Promise<{ state: Json; selected: Json }> {
  await Bun.sleep(Math.max(700, pollMs));
  const state = getState(`${spec.id}-selected`);
  const selected = assertSelectedFrame(state, input, spec, expectedSearchText);
  return { state, selected };
}

function assertActionsDialog(state: Json, spec: Json, selected: Json): Json {
  const dialog = state.actionsDialog;
  if (!dialog?.open) {
    throw new Error(`${spec.id}: expected actionsDialog.open, got ${JSON.stringify(dialog)}`);
  }
  if (dialog.host !== "MainList") {
    throw new Error(`${spec.id}: expected MainList host, got ${dialog.host}`);
  }
  if (selected.stableKey && dialog.contextStableKey && dialog.contextStableKey !== selected.stableKey) {
    throw new Error(`${spec.id}: contextStableKey ${dialog.contextStableKey} did not match selected ${selected.stableKey}`);
  }
  const actions = dialog.visibleActions ?? [];
  if (spec.requireAnyActions && actions.length === 0) {
    throw new Error(`${spec.id}: expected existing action owner to expose at least one action`);
  }
  for (const [id, label] of spec.expectedActions ?? []) {
    const action = actions.find((candidate: Json) => candidate.id === id);
    if (!action) {
      throw new Error(`${spec.id}: missing action ${id} in ${JSON.stringify(actions)}`);
    }
    if (action.label !== label) {
      throw new Error(`${spec.id}: action ${id} expected label ${label}, got ${action.label}`);
    }
  }
  for (const id of spec.destructiveActions ?? []) {
    const action = actions.find((candidate: Json) => candidate.id === id);
    if (!action?.destructive || !["Danger", "Destructive"].includes(action.section)) {
      throw new Error(`${spec.id}: expected ${id} to be destructive in Danger/Destructive, got ${JSON.stringify(action)}`);
    }
  }
  return dialog;
}

async function openActionsAndAssert(spec: Json, input: string, expectedSearchText: string): Promise<Json> {
  send({ type: "show", requestId: `${spec.id}-show-${Date.now()}` });
  send({ type: "setFilter", text: input, requestId: `${spec.id}-set-${Date.now()}` });
  waitForInput(input);
  const { state: selectedState, selected } = await waitForSelected(input, spec, expectedSearchText);

  send({ type: "simulateKey", key: "k", modifiers: ["cmd"], requestId: `${spec.id}-cmd-k-${Date.now()}` });
  let lastState: Json | null = null;
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    const state = getState(`${spec.id}-actions`);
    lastState = state;
    try {
      const dialog = assertActionsDialog(state, spec, selected);
      send({ type: "simulateKey", key: "escape", modifiers: [], requestId: `${spec.id}-escape-${Date.now()}` });
      await Bun.sleep(350);
      return {
        sourceHead: spec.sourceHead,
        query: input,
        selected: {
          role: selected.role,
          typeLabel: selected.typeLabel,
          sourceName: selected.sourceName ?? null,
          stableKey: selected.stableKey ?? null,
        },
        enterAction: selectedState.mainWindowPreflight?.enterAction ?? null,
        actions: dialog.visibleActions,
      };
    } catch (error) {
      await Bun.sleep(pollMs);
      if (Date.now() >= deadline) {
        throw error;
      }
    }
  }
  throw new Error(`${spec.id}: timed out waiting for actions dialog, lastState=${JSON.stringify(lastState)}`);
}

async function runCase(spec: Json): Promise<Json> {
  const textQuery = spec.query ?? query;
  const input = `${spec.sourceHead} ${textQuery}`;
  const normal = await openActionsAndAssert(spec, input, textQuery);
  send({ type: "setFilter", text: "", requestId: `${spec.id}-reset-${Date.now()}` });
  waitForInput("");

  if (!spec.attachedQuery) {
    return { ...normal, variants: [{ kind: "spaced", ...normal }] };
  }

  const attachedInput = `${spec.sourceHead}${spec.attachedQuery}`;
  const attached = await openActionsAndAssert(spec, attachedInput, spec.attachedQuery);
  send({ type: "setFilter", text: "", requestId: `${spec.id}-attached-reset-${Date.now()}` });
  waitForInput("");
  return { ...normal, variants: [{ kind: "spaced", ...normal }, { kind: "attached", ...attached }] };
}

async function main() {
  seedFixtures();
  runSession(["stop", session]);
  runSession(["start", session]);

  try {
    const results: Json[] = [];
    for (const spec of cases) {
      results.push(await runCase(spec));
    }

    const logPath = join(sessionRoot, session, "app.log");
    const responsesPath = join(sessionRoot, session, "responses.ndjson");
    const receipt = {
      schemaVersion: 1,
      status: "pass",
      session,
      query,
      homeDir,
      cases: results,
      logExcerpt: readFileSync(logPath, "utf8").split("\n").slice(-120),
      responsesPath,
      cmuxRequestsPrivacy: (() => {
        const cmuxRequests = readFileSync(cmuxRequestsPath, "utf8");
        return {
          containsTranscript: cmuxRequests.includes("transcript"),
          containsPreview: cmuxRequests.includes("preview"),
          containsPrompt: cmuxRequests.includes("prompt"),
          containsAssistantText: cmuxRequests.includes("assistantText"),
          containsResumeCommand: cmuxRequests.includes("resumeCommand"),
        };
      })(),
    };
    const receiptText = `${JSON.stringify(receipt, null, 2)}\n`;
    const cmuxRequests = readFileSync(cmuxRequestsPath, "utf8");
    const leakHaystack = `${receiptText}\n${readFileSync(logPath, "utf8")}\n${readFileSync(responsesPath, "utf8")}\n${cmuxRequests}`;
    const leakedPoison = aiVaultPoisonStrings.filter((value) => leakHaystack.includes(value));
    if (leakedPoison.length > 0) {
      throw new Error(`AI Vault receipt leaked poison metadata: ${leakedPoison.join(", ")}`);
    }
    const leakedRequestFields = aiVaultSensitiveRequestFields.filter((value) => cmuxRequests.includes(value));
    if (leakedRequestFields.length > 0) {
      throw new Error(`AI Vault cmux request leaked sensitive fields: ${leakedRequestFields.join(", ")}`);
    }
    writeFileSync(join(outputDir, "receipt.json"), receiptText);
    writeFileSync(join(repoRoot, ".test-output", "root-source-actions-matrix.json"), receiptText);
    process.stdout.write(receiptText);
  } finally {
    if (!keepSession) {
      runSession(["stop", session]);
    }
  }
}

main().catch((error) => {
  const message = error instanceof Error ? error.stack ?? error.message : String(error);
  mkdirSync(join(repoRoot, ".test-output"), { recursive: true });
  writeFileSync(
    join(repoRoot, ".test-output", "root-source-actions-matrix.json"),
    `${JSON.stringify(
      {
        schemaVersion: 1,
        status: "fail",
        session,
        query,
        error: message,
      },
      null,
      2,
    )}\n`,
  );
  process.stderr.write(`${message}\n`);
  process.exit(1);
});
