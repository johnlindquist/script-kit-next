#!/usr/bin/env bun
/**
 * Runtime proof for Script List-backed Agent Chat attachment portals.
 *
 * For script, scriptlet, and skill portals this drives the same user path:
 * type a focused `@kind:query` token, press Cmd+. to open its portal, then
 * press Escape. Physical GPUI dispatch and the SimulateKey mirror must both
 * restore the Agent Chat host without hiding the main window or clearing the
 * composer token.
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/attachment-portal-escape/script-kit-gpui \
 *     bun scripts/agentic/attachment-portal-escape-probe.ts
 */
import { Driver, type Json } from "../devtools/driver.ts";

const BINARY =
  process.env.PROBE_BINARY ??
  process.argv[2] ??
  "target-agent/artifacts/attachment-portal-escape/script-kit-gpui";

type EscapeRoute = "physical_bubble" | "simulate_key";

type PortalCase = {
  name: string;
  token: string;
  query: string;
  kind: string;
  escapeRoute: EscapeRoute;
};

const CASES: PortalCase[] = [
  {
    name: "script_search_physical_escape",
    token: "@script:portal-escape-script",
    query: "portal-escape-script",
    kind: "ScriptSearch",
    escapeRoute: "physical_bubble",
  },
  {
    name: "scriptlet_search_simulated_escape",
    token: "@scriptlet:portal-escape-scriptlet",
    query: "portal-escape-scriptlet",
    kind: "ScriptletSearch",
    escapeRoute: "simulate_key",
  },
  {
    name: "skill_search_physical_escape",
    token: "@skill:portal-escape-skill",
    query: "portal-escape-skill",
    kind: "SkillSearch",
    escapeRoute: "physical_bubble",
  },
];

function topStateDigest(state: Json): Json {
  return {
    promptType: state.promptType,
    surfaceKind: state.surfaceContract?.surfaceKind,
    inputValue: state.inputValue,
    windowVisible: state.windowVisible,
    isFocused: state.isFocused,
    filterInputDiagnostics: state.filterInputDiagnostics,
  };
}

function agentChatDigest(state: Json): Json {
  return {
    status: state.status,
    inputText: state.inputText,
    cursorIndex: state.cursorIndex,
    contextChipCount: state.contextChipCount,
    spine: state.spine,
  };
}

function elementDigest(result: Json): Json {
  const elements = Array.isArray(result.elements) ? result.elements as Json[] : [];
  return {
    focusedSemanticId: result.focusedSemanticId,
    totalCount: result.totalCount,
    inputs: elements
      .filter((element) => element.type === "input")
      .map((element) => ({
        semanticId: element.semanticId,
        value: element.value,
        focused: element.focused,
      })),
    lists: elements
      .filter((element) => element.type === "list")
      .map((element) => ({
        semanticId: element.semanticId,
        text: element.text,
      })),
  };
}

async function getAgentChatState(driver: Driver): Promise<Json> {
  return driver.request({ type: "getAgentChatState" }, { timeoutMs: 10_000 });
}

async function pollState(
  driver: Driver,
  predicate: (state: Json) => boolean,
  timeoutMs = 8_000,
): Promise<{ matched: boolean; state: Json }> {
  const deadline = Date.now() + timeoutMs;
  let state = await driver.getState({ timeoutMs: 5_000 });
  while (!predicate(state) && Date.now() < deadline) {
    await Bun.sleep(25);
    state = await driver.getState({ timeoutMs: 5_000 });
  }
  return { matched: predicate(state), state };
}

async function pollAgentChatInput(
  driver: Driver,
  expected: string,
  timeoutMs = 8_000,
): Promise<{ matched: boolean; state: Json }> {
  const deadline = Date.now() + timeoutMs;
  let state = await getAgentChatState(driver);
  while (state.inputText !== expected && Date.now() < deadline) {
    await Bun.sleep(25);
    state = await getAgentChatState(driver);
  }
  return { matched: state.inputText === expected, state };
}

async function portalLogs(driver: Driver): Promise<Json[]> {
  const response = await driver.getLogs(
    { contains: "portal", limit: 500 },
    { timeoutMs: 5_000 },
  );
  return Array.isArray(response.entries) ? response.entries as Json[] : [];
}

function relevantLogMessages(entries: Json[]): string[] {
  return entries
    .map((entry) => String(entry.message ?? ""))
    .filter((message) =>
      message.includes("portal")
      || message.includes("attachment_portal")
    );
}

function countMessages(messages: string[], predicate: (message: string) => boolean): number {
  return messages.filter(predicate).length;
}

async function pollPortalEvidence(
  driver: Driver,
  beforeEntries: Json[],
  portalCase: PortalCase,
  timeoutMs = 2_000,
): Promise<{ matched: boolean; messages: string[] }> {
  const before = relevantLogMessages(beforeEntries);
  const opened = (message: string) =>
    message.includes("event=attachment_portal_opened")
    && message.includes(`kind=${portalCase.kind}`);
  const consumed = (message: string) =>
    message.includes("event=script_list_attachment_portal_escape_consumed")
    && message.includes(`routing_path=${portalCase.escapeRoute}`);
  const cancelled = (message: string) =>
    message.includes("event=attachment_portal_cancelled")
    && message.includes(`kind=Some(${portalCase.kind})`);
  const sessionCancelled = (message: string) =>
    message.includes("event=agent_chat_portal_session_cancelled")
    && message.includes(`kind=${portalCase.kind}`);
  const baseline = {
    opened: countMessages(before, opened),
    consumed: countMessages(before, consumed),
    cancelled: countMessages(before, cancelled),
    sessionCancelled: countMessages(before, sessionCancelled),
  };

  const deadline = Date.now() + timeoutMs;
  let messages: string[] = [];
  let matched = false;
  while (!matched && Date.now() < deadline) {
    messages = relevantLogMessages(await portalLogs(driver));
    matched = countMessages(messages, opened) > baseline.opened
      && countMessages(messages, consumed) > baseline.consumed
      && countMessages(messages, cancelled) > baseline.cancelled
      && countMessages(messages, sessionCancelled) > baseline.sessionCancelled;
    if (!matched) await Bun.sleep(25);
  }

  return {
    matched,
    messages: messages.filter((message) =>
      message.includes(portalCase.kind)
      || message.includes(portalCase.query)
      || consumed(message)
    ),
  };
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "attachment-portal-escape-probe",
  readyTimeoutMs: 30_000,
  defaultTimeoutMs: 10_000,
  env: {
    SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    SCRIPT_KIT_STARTUP_PROFILE: "dev-fast",
    SCRIPT_KIT_DISABLE_AGENT_CHAT_HOT_PREWARM: "1",
    SCRIPT_KIT_DISABLE_AUTOMATIC_UPDATE_CHECK: "1",
  },
});

const receipt: Json = {
  schemaVersion: 1,
  probe: "attachment-portal-escape",
  binary: BINARY,
  classification: "blocked",
  cases: [],
  failures: [],
};
const caseReceipts = receipt.cases as Json[];
const failures = receipt.failures as string[];

function check(caseName: string, name: string, pass: boolean, detail: Json): Json {
  if (!pass) failures.push(`${caseName}:${name}`);
  return { name, pass, detail };
}

try {
  for (const portalCase of CASES) {
    const checks: Json[] = [];
    const logsBefore = await portalLogs(driver);

    const openFixture = await driver.request(
      { type: "openAgentChatKitchenSinkFixture" },
      { timeoutMs: 10_000 },
    );
    const hostReady = await pollState(
      driver,
      (state) =>
        state.promptType === "agentChatChat"
        && state.surfaceContract?.surfaceKind === "AgentChat"
        && state.windowVisible === true,
    );
    checks.push(check(
      portalCase.name,
      "agent_chat_host_ready",
      openFixture.ok === true && hostReady.matched,
      { openFixture, state: topStateDigest(hostReady.state) },
    ));

    const setInput = await driver.request(
      { type: "setAgentChatInput", text: portalCase.token },
      { timeoutMs: 10_000 },
    );
    const composerReady = await pollAgentChatInput(driver, portalCase.token);
    const hostElements = await driver.getElements(
      { target: { type: "kind", kind: "main" } },
      { timeoutMs: 5_000 },
    );
    const hostElementReceipt = elementDigest(hostElements);
    const hostComposer = (hostElementReceipt.inputs as Json[])
      .find((input) => input.semanticId === "input:agent-chat-composer");
    checks.push(check(
      portalCase.name,
      "focused_token_ready",
      setInput.ok === true
        && composerReady.matched
        && hostComposer?.value === portalCase.token
        && hostComposer?.focused === true,
      {
        setInput,
        agentChat: agentChatDigest(composerReady.state),
        elements: hostElementReceipt,
      },
    ));

    const openPortal = await driver.simulateGpuiEvent(
      { type: "keyDown", key: ".", modifiers: ["cmd"] },
      { target: { type: "kind", kind: "main" }, timeoutMs: 10_000 },
    );
    const portalReady = await pollState(
      driver,
      (state) =>
        state.promptType === "none"
        && state.surfaceContract?.surfaceKind === "ScriptList"
        && state.inputValue === portalCase.query
        && state.windowVisible === true,
    );
    const portalElements = await driver.getElements(
      { target: { type: "kind", kind: "main" } },
      { timeoutMs: 5_000 },
    );
    const portalElementReceipt = elementDigest(portalElements);
    const portalInput = (portalElementReceipt.inputs as Json[])
      .find((input) => input.semanticId === "input:filter");
    checks.push(check(
      portalCase.name,
      "script_list_portal_opened",
      openPortal.success === true
        && (openPortal.dispatchCompleted === true || openPortal.dispatchScheduled === true)
        && portalReady.matched
        && portalReady.state.filterInputDiagnostics?.canonicalFilterText === portalCase.query
        && portalReady.state.filterInputDiagnostics?.computedFilterText === portalCase.query
        && portalInput?.value === portalCase.query
        && portalInput?.focused === true,
      {
        dispatch: openPortal,
        state: topStateDigest(portalReady.state),
        elements: portalElementReceipt,
      },
    ));

    const escapeDispatch = portalCase.escapeRoute === "physical_bubble"
      ? await driver.simulateGpuiEvent(
          { type: "keyDown", key: "escape", modifiers: [] },
          { target: { type: "kind", kind: "main" }, timeoutMs: 10_000 },
        )
      : await driver.request(
          {
            type: "simulateKey",
            target: { type: "kind", kind: "main" },
            key: "escape",
            modifiers: [],
          },
          { expect: "externalCommandResult", timeoutMs: 10_000 },
        );

    const restored = await pollState(
      driver,
      (state) =>
        state.promptType === "agentChatChat"
        && state.surfaceContract?.surfaceKind === "AgentChat"
        && state.windowVisible === true,
    );
    const restoredComposer = await pollAgentChatInput(driver, portalCase.token, 2_000);
    const restoredElements = await driver.getElements(
      { target: { type: "kind", kind: "main" } },
      { timeoutMs: 5_000 },
    );
    const restoredElementReceipt = elementDigest(restoredElements);
    const restoredInput = (restoredElementReceipt.inputs as Json[])
      .find((input) => input.semanticId === "input:agent-chat-composer");

    const logEvidence = await pollPortalEvidence(driver, logsBefore, portalCase);
    const caseLogs = logEvidence.messages;
    const consumedLog = caseLogs.some((message) =>
      message.includes("event=script_list_attachment_portal_escape_consumed")
      && message.includes(`routing_path=${portalCase.escapeRoute}`)
    );
    const cancelledLog = caseLogs.some((message) =>
      message.includes("event=attachment_portal_cancelled")
      && message.includes(`kind=Some(${portalCase.kind})`)
    );
    const openedLog = caseLogs.some((message) =>
      message.includes("event=attachment_portal_opened")
      && message.includes(`kind=${portalCase.kind}`)
    );
    const sessionCancelledLog = caseLogs.some((message) =>
      message.includes("event=agent_chat_portal_session_cancelled")
    );

    const dispatchSucceeded = portalCase.escapeRoute === "physical_bubble"
      ? escapeDispatch.success === true
        && escapeDispatch.resolvedWindowId === "main"
        && (escapeDispatch.dispatchCompleted === true || escapeDispatch.dispatchScheduled === true)
      : escapeDispatch.ok === true;

    checks.push(check(
      portalCase.name,
      "escape_restores_agent_chat_host",
      dispatchSucceeded
        && restored.matched
        && restoredComposer.matched
        && restoredComposer.state.cursorIndex === portalCase.token.length
        && restoredComposer.state.messageCount === composerReady.state.messageCount
        && restoredComposer.state.contextChipCount === composerReady.state.contextChipCount
        && restoredInput?.value === portalCase.token
        && restoredInput?.focused === true,
      {
        dispatch: escapeDispatch,
        state: topStateDigest(restored.state),
        agentChat: agentChatDigest(restoredComposer.state),
        elements: restoredElementReceipt,
      },
    ));
    checks.push(check(
      portalCase.name,
      "escape_receipts_match_route_and_kind",
      openedLog && consumedLog && cancelledLog && sessionCancelledLog,
      {
        expectedKind: portalCase.kind,
        expectedRoute: portalCase.escapeRoute,
        openedLog,
        consumedLog,
        cancelledLog,
        sessionCancelledLog,
        evidenceSettled: logEvidence.matched,
        messages: caseLogs,
      },
    ));

    caseReceipts.push({
      name: portalCase.name,
      token: portalCase.token,
      query: portalCase.query,
      kind: portalCase.kind,
      escapeRoute: portalCase.escapeRoute,
      pass: checks.every((entry) => entry.pass === true),
      checks,
    });
  }

  receipt.classification = failures.length === 0 ? "ok" : "reproduced-failure";
} catch (error) {
  receipt.classification = "blocked";
  receipt.error = error instanceof Error ? error.message : String(error);
} finally {
  receipt.sessionDir = driver.sessionDir;
  receipt.appLog = driver.logPath;
  receipt.driverStats = driver.stats;
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.classification !== "ok") process.exit(1);
