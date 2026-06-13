#!/usr/bin/env bun
/**
 * Runtime proof for Day Page @context parity:
 * - typing @con in the Day Page editor swaps to the main menu context list
 * - no deprecated Day Page inline @ popup/list is used for @context
 * - accepting the main-menu row returns to the same Day Page line with @here
 * - Cmd+Enter submits that line to Agent Chat with a resolved context part
 */
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/day-page-context/script-kit-gpui";

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") {
    out.push(json);
  }
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

function gpuiKey(
  driver: Driver,
  key: string,
  modifiers: string[] = [],
  text?: string,
): Promise<Json> {
  const event: Json = { type: "keyDown", key, modifiers };
  if (text !== undefined) event.text = text;
  return driver.request(
    { type: "simulateGpuiEvent", target: { type: "kind", kind: "main" }, event },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  );
}

async function typeText(driver: Driver, text: string) {
  for (const ch of text) {
    const key = ch === " " ? "space" : ch;
    const result = await gpuiKey(driver, key, [], ch);
    if (result.success !== true) {
      check(`typed_${ch}_dispatch`, false, { result });
      return false;
    }
    await Bun.sleep(35);
  }
  return true;
}

async function editorText(driver: Driver): Promise<string | null> {
  const elements = (await driver.getElements(
    { target: { type: "main" }, limit: 160 },
    { timeoutMs: 5000 },
  )) as Json;
  const editor = walkElements(elements).find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  return (editor?.value as string | undefined) ?? null;
}

function firstMatchingLog(log: string, needle: string): string | null {
  return log.split("\n").find((line) => line.includes(needle)) ?? null;
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-context-roundtrip",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

// Seed only the auth/config files needed for Agent Chat submission in the
// isolated sandbox. Missing files are reported by the Agent Chat assertions.
const sandboxHome = `${driver.sessionDir}/home`;
const realHome = process.env.HOME ?? "";
for (const rel of [
  ".codex/auth.json",
  ".pi/agent/auth.json",
  ".pi/agent/settings.json",
]) {
  const src = `${realHome}/${rel}`;
  const dest = `${sandboxHome}/${rel}`;
  try {
    await Bun.$`mkdir -p ${dest.slice(0, dest.lastIndexOf("/"))} && cp ${src} ${dest}`.quiet();
  } catch {
    // The submit step will fail honestly if auth is unavailable.
  }
}

try {
  const initial = await openDayPage(driver, runId);
  check("opened_day_page", initial.promptType === "dayPage", {
    promptType: initial.promptType,
  });

  const prefix = `Day Page context roundtrip ${runId} `;
  const seed = (await driver.batch(
    [
      { type: "setInput", text: prefix },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: prefix },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("seeded_day_page_prefix", seed.success === true, { batch: seed });

  const typed = await typeText(driver, "@con");
  check("typed_context_alias_through_gpui", typed, {});

  const menuWait = (await driver.waitFor(
    {
      type: "stateMatch",
      state: { promptType: "none", inputValue: "@con" },
    },
    { timeoutMs: 5000 },
  )) as Json;
  const menuState = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("typing_swapped_to_main_menu", menuState.promptType === "none", {
    promptType: menuState.promptType,
    inputValue: menuState.inputValue ?? null,
    wait: menuWait,
  });
  check("main_menu_filter_is_context_alias", menuState.inputValue === "@con", {
    inputValue: menuState.inputValue ?? null,
  });

  const menuElements = (await driver.getElements(
    { target: { type: "main" }, limit: 220 },
    { timeoutMs: 5000 },
  )) as Json;
  const menuFlat = walkElements(menuElements);
  const contextRow = menuFlat.find(
    (el) =>
      el.semanticId === "spine:@:builtin:here" ||
      el.semanticId === "choice:0:what-i-m-looking-at" ||
      String(el.semanticId ?? "").includes("what-i-m-looking-at") ||
      (typeof el.label === "string" && el.label.includes("What I")),
  );
  const inlinePopupIds = menuFlat
    .map((el) => String(el.semanticId ?? el.id ?? ""))
    .filter((id) => id.includes("day-page-spine") || id.includes("context-picker"));
  check("main_menu_context_row_visible", Boolean(contextRow), {
    selectedSemanticId: menuElements.selectedSemanticId ?? null,
    contextRow: contextRow ?? null,
    sampleIds: menuFlat.map((el) => el.semanticId ?? el.id).filter(Boolean).slice(0, 40),
  });
  check("deprecated_inline_context_popup_absent", inlinePopupIds.length === 0, {
    inlinePopupIds,
  });

  driver.simulateKey("enter");
  await Bun.sleep(900);
  const afterAccept = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const completedLine = `${prefix}@here `;
  check("accept_returns_to_day_page", afterAccept.promptType === "dayPage", {
    promptType: afterAccept.promptType,
    inputValue: afterAccept.inputValue ?? null,
  });
  const afterAcceptText = await editorText(driver);
  check("accepted_context_spliced_into_original_line", afterAcceptText === completedLine, {
    afterAcceptText,
    expected: completedLine,
  });

  driver.simulateKey("enter", ["cmd"]);
  await Bun.sleep(3000);
  const afterSubmit = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const appLog = await Bun.file(`${driver.sessionDir}/app.log`).text();
  const startedLine = firstMatchingLog(appLog, "event=day_page_cmd_enter_handoff_started");
  const submitLine = firstMatchingLog(
    appLog,
    "event=agent_chat_reused_entry_intent_with_host_context_submitted source=day_page_line_handoff",
  );
  const startedContextCount = Number(/context_token_count=(\d+)/.exec(startedLine ?? "")?.[1] ?? -1);
  const startedAliasCount = Number(/alias_count=(\d+)/.exec(startedLine ?? "")?.[1] ?? -1);
  const contextPartCount = Number(/context_part_count=(\d+)/.exec(submitLine ?? "")?.[1] ?? -1);
  const unknownWarningCount = Number(
    /unknown_warning_count=(\d+)/.exec(submitLine ?? "")?.[1] ?? -1,
  );

  check("cmd_enter_opens_agent_chat", afterSubmit.promptType === "agentChatChat", {
    promptType: afterSubmit.promptType,
  });
  check("day_page_handoff_logged_context_token", startedContextCount > 0, {
    startedContextCount,
    startedAliasCount,
    startedLine,
  });
  check("agent_chat_received_context_part", contextPartCount > 0, {
    contextPartCount,
    unknownWarningCount,
    submitLine,
  });
  check("agent_chat_no_unknown_context_warnings", unknownWarningCount === 0, {
    unknownWarningCount,
    submitLine,
  });

  const pass = failures.length === 0;
  console.log(
    JSON.stringify(
      {
        pass,
        failures,
        sessionDir: driver.sessionDir,
        screenshotProof: "not-used-semantic-devtools-only",
        receipts,
      },
      null,
      2,
    ),
  );
  if (!pass) process.exitCode = 1;
} finally {
  await driver.close();
}
