#!/usr/bin/env bun
/**
 * Red/green receipts for the Flow Desk + Threadline sessions (2026-07-09).
 * Every flow is an agent identity; Enter means CONVERSE on Script Kit's own
 * ChatPrompt surface — no engine TUI is ever wrapped.
 *
 * Runs against deterministic fixtures, zero tokens:
 *  - fixtures/flow-ux-project: project flows discovered via `md roster`
 *    (real mdflow + fake engines on PATH).
 *  - fixtures/flow-desk-package: fake @johnlindquist/flows package with a
 *    codex-engine flow and a fasteng-engine flow.
 *  - fixtures/flow-desk-package/bin/fake-codex: deterministic
 *    `codex app-server` (SCRIPT_KIT_CODEX_BIN seam) that echoes each turn's
 *    prompt back as "FAKE-CODEX-REPLY: …".
 *
 * Receipt matrix:
 *  1. deskOpens — single "Flows" built-in opens the desk, corpus ready.
 *  2. packageProvenance — package flow row shows purpose + origin.
 *  3. enterConverses — Enter opens a Threadline session (chat surface, no
 *     auto-message from a name lookup, codexThread transport, honest idle).
 *  4. firstTurnRoundTrip — submitted message streams back; the reply echo
 *     proves the prompt carried the flow's MISSION + the user text.
 *  5. secondTurnRawMessage — second turn commits (thread holds context).
 *  6. escapeBackgrounds — Esc returns to the desk; session + turns survive.
 *  7. reentrySameSession — re-entering restores the SAME session id and
 *     transcript (no respawn).
 *  8. mdflowTransport — non-codex engine converses via --_task/--events
 *     turns on the same chat surface.
 *  9. cmdKActions — ⌘K in a session shows session verbs.
 * 10. runOnceBackground — ⇧↵ registry run, no session created.
 * 11. cleanupHidden — app left hidden.
 */
import { join } from "node:path";
import { Driver } from "../devtools/driver.ts";

const FIXTURE = join(import.meta.dir, "fixtures/flow-ux-project");
const PACKAGE_FIXTURE = join(import.meta.dir, "fixtures/flow-desk-package");
const SHOTS = join(import.meta.dir, "../../.test-screenshots");

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/flow-ux/script-kit-gpui";

const driver = await Driver.launch({
  binary,
  sessionName: `flow-desk-probe-${process.pid}`,
  sandboxHome: true,
  env: {
    SCRIPT_KIT_FLOW_UX_CWD: FIXTURE,
    SCRIPT_KIT_FLOWS_PACKAGE_DIR: PACKAGE_FIXTURE,
    SCRIPT_KIT_FLOWS_BIN_DIR: join(PACKAGE_FIXTURE, "bin"),
    SCRIPT_KIT_CODEX_BIN: join(PACKAGE_FIXTURE, "bin/fake-codex"),
    PATH: `${join(FIXTURE, "bin")}:${join(PACKAGE_FIXTURE, "bin")}:${process.env.PATH ?? ""}`,
  },
});

type Json = Record<string, any>;
const receipt: Json = { binary, fixture: FIXTURE, packageFixture: PACKAGE_FIXTURE, checks: {} };
const checks = receipt.checks as Json;
let shotIndex = 0;

const flowUx = (state: Json): Json | null => (state?.flowUx as Json) ?? null;
const lastSession = (state: Json): Json | undefined =>
  ((flowUx(state)?.sessions as Json[]) ?? []).at(-1);

const pollState = async (
  pred: (state: Json) => boolean,
  timeoutMs = 8_000,
): Promise<Json> => {
  const deadline = Date.now() + timeoutMs;
  let state: Json = {};
  while (Date.now() < deadline) {
    state = (await driver.getState()) as Json;
    if (pred(state)) return state;
    await Bun.sleep(100);
  }
  return state;
};

const shot = async (name: string) => {
  shotIndex += 1;
  await driver.captureScreenshot({
    savePath: join(SHOTS, `flow-desk-${String(shotIndex).padStart(2, "0")}-${name}.png`),
  });
};

// Real-dispatch key press through the GPUI window (the only path that hits
// element-level on_key_down handlers).
const pressMain = (key: string, modifiers: string[] = []) =>
  driver
    .simulateGpuiEvent(
      { type: "keyDown", key, modifiers },
      { target: { type: "main" }, timeoutMs: 5_000 },
    )
    .catch((error) => ({ error: String(error) }));

// Escape until the app rests on the ScriptList root with an empty filter.
// NEVER escapes when already at root — a root escape hides the window.
const returnToRoot = async () => {
  for (let i = 0; i < 10; i++) {
    const st = (await driver.getState()) as Json;
    if (st.windowVisible === false) {
      driver.send({ type: "show" });
      await Bun.sleep(300);
      continue;
    }
    if (st.promptType === "none" && !st.inputValue) return;
    await pressMain("escape");
    await Bun.sleep(250);
  }
};

const visibleTexts = async (): Promise<string> => {
  const result = (await driver.getElements({ limit: 200 })) as Json;
  return ((result.elements as Json[]) ?? [])
    .map((e) => `${e.text ?? ""}|${e.value ?? ""}|${e.label ?? ""}`)
    .join("\n");
};

// Send one chat message in the open flow session: seed the composer via
// protocol setInput (routed to ChatPrompt), then real-dispatch Enter.
const sendChatMessage = async (text: string) => {
  driver.send({
    type: "batch",
    requestId: `flow-chat-${Date.now()}`,
    commands: [{ type: "setInput", text }],
  });
  await Bun.sleep(250);
  await pressMain("enter");
};

const openDesk = async () => {
  await returnToRoot();
  await driver.setFilterAndWait("Flows");
  await pressMain("enter");
  await Bun.sleep(400);
};

const deskFilter = async (text: string) => {
  await driver.setFilterAndWait(text);
  await Bun.sleep(300);
};

try {
  driver.send({ type: "show" });
  await driver.waitForSettle();

  // -- 1. Desk entry + combined corpus ------------------------------------
  await driver.setFilterAndWait("Flows");
  await Bun.sleep(300);
  const flowsEntryVisible = (await visibleTexts())
    .split("\n")
    .some((line) => line.startsWith("Flows|"));
  await openDesk();
  let state = await pollState(
    (s) => flowUx(s)?.activeVariant === "flash" && flowUx(s)?.roster?.status === "ready",
  );
  let fx = flowUx(state);
  checks.deskOpens = {
    flowsEntryVisible,
    activeVariant: fx?.activeVariant,
    rosterStatus: fx?.roster?.status,
    rosterCount: fx?.roster?.count,
    ok:
      flowsEntryVisible &&
      fx?.activeVariant === "flash" &&
      fx?.roster?.status === "ready" &&
      (fx?.roster?.count ?? 0) >= 6,
  };
  await shot("desk-corpus");

  // -- 2. Package provenance ----------------------------------------------
  await deskFilter("hello-codex");
  state = (await driver.getState()) as Json;
  fx = flowUx(state);
  const deskTexts = await visibleTexts();
  checks.packageProvenance = {
    selectedFlowId: fx?.selectedFlowId,
    rowShowsFriendlyName: deskTexts.includes("Hello Codex"),
    rowShowsOrigin: deskTexts.includes("@johnlindquist/flows"),
    ok:
      fx?.selectedFlowId === "package:flow-hello-codex" &&
      deskTexts.includes("Hello Codex") &&
      deskTexts.includes("@johnlindquist/flows"),
  };
  await shot("package-provenance");

  // -- 3. Enter = converse: Threadline session, honest idle ---------------
  await pressMain("enter");
  state = await pollState(
    (s) => s.promptType === "flowSession" && Boolean(lastSession(s)),
  );
  fx = flowUx(state);
  const session = lastSession(state);
  const sessionElements = await visibleTexts();
  checks.enterConverses = {
    promptType: state.promptType,
    flowId: session?.flowId,
    transport: session?.transport,
    state: session?.state,
    turns: session?.turns,
    turnInFlight: session?.turnInFlight,
    hasChatComposer: sessionElements.includes("chat-input") || sessionElements.includes("Message"),
    ok:
      state.promptType === "flowSession" &&
      session?.flowId === "package:flow-hello-codex" &&
      session?.transport === "codexThread" &&
      session?.state === "needs you" &&
      session?.turns === 0 &&
      session?.turnInFlight === false,
  };
  await shot("threadline-open");

  // -- 4. First turn: mission + message round trip -------------------------
  await sendChatMessage("what is the answer");
  state = await pollState((s) => lastSession(s)?.turns === 1, 10_000);
  const afterFirst = lastSession(state);
  const transcript = await visibleTexts();
  checks.firstTurnRoundTrip = {
    turns: afterFirst?.turns,
    state: afterFirst?.state,
    replyEchoed: transcript.includes("FAKE-CODEX-REPLY"),
    missionInPrompt: transcript.includes("You are Hello Codex"),
    taskInPrompt: transcript.includes("what is the answer"),
    ok:
      afterFirst?.turns === 1 &&
      afterFirst?.state === "needs you" &&
      transcript.includes("FAKE-CODEX-REPLY") &&
      transcript.includes("You are Hello Codex") &&
      transcript.includes("what is the answer"),
  };
  await shot("first-turn-reply");

  // -- 5. Second turn: raw message (thread holds context) ------------------
  await sendChatMessage("and a follow up");
  state = await pollState((s) => lastSession(s)?.turns === 2, 10_000);
  const afterSecond = lastSession(state);
  checks.secondTurnRawMessage = {
    turns: afterSecond?.turns,
    state: afterSecond?.state,
    ok: afterSecond?.turns === 2 && afterSecond?.state === "needs you",
  };
  await shot("second-turn");

  // -- 6. Esc backgrounds: desk returns, session + transcript survive ------
  await pressMain("escape");
  state = await pollState((s) => flowUx(s)?.activeVariant === "flash", 5_000);
  fx = flowUx(state);
  const bgSession = lastSession(state);
  checks.escapeBackgrounds = {
    activeVariant: fx?.activeVariant,
    live: bgSession?.live,
    turns: bgSession?.turns,
    ok: fx?.activeVariant === "flash" && bgSession?.live === true && bgSession?.turns === 2,
  };
  await shot("backgrounded-desk");

  // -- 7. Active row re-entry: SAME session, transcript intact -------------
  const sessionId = bgSession?.sessionId;
  await deskFilter("");
  await pressMain("enter"); // sessions sort first: selection 0 is the live row
  state = await pollState((s) => s.promptType === "flowSession");
  const reSession = lastSession(state);
  checks.reentrySameSession = {
    sessionId,
    reenteredId: reSession?.sessionId,
    sessionCount: ((flowUx(state)?.sessions as Json[]) ?? []).length,
    turns: reSession?.turns,
    ok:
      state.promptType === "flowSession" &&
      typeof sessionId === "number" &&
      reSession?.sessionId === sessionId &&
      reSession?.turns === 2 &&
      ((flowUx(state)?.sessions as Json[]) ?? []).length === 1,
  };
  await shot("reentered-session");

  // -- 8. mdflow transport: non-codex engine converses too ------------------
  await pressMain("escape");
  await pollState((s) => flowUx(s)?.activeVariant === "flash", 5_000);
  await deskFilter("hello-agent");
  await pressMain("enter");
  state = await pollState(
    (s) => s.promptType === "flowSession" && lastSession(s)?.flowId === "package:flow-hello-agent",
  );
  await sendChatMessage("hello task words");
  state = await pollState(
    (s) => lastSession(s)?.flowId === "package:flow-hello-agent" && lastSession(s)?.turns === 1,
    20_000,
  );
  const mdSession = lastSession(state);
  const mdTranscript = await visibleTexts();
  checks.mdflowTransport = {
    transport: mdSession?.transport,
    turns: mdSession?.turns,
    state: mdSession?.state,
    engineReplyVisible: mdTranscript.includes("FASTENG_OK"),
    taskReached: mdTranscript.includes("hello task words"),
    ok:
      mdSession?.transport === "mdflowTurns" &&
      mdSession?.turns === 1 &&
      mdSession?.state === "needs you" &&
      mdTranscript.includes("FASTENG_OK") &&
      mdTranscript.includes("hello task words"),
  };
  await shot("mdflow-transport");

  // -- 9. ⌘K in a session: session verbs -----------------------------------
  await pressMain("k", ["cmd"]);
  state = await pollState((s) => Boolean(s.actionsDialog), 5_000);
  const actionIds = ((state.actionsDialog?.visibleActions as Json[]) ?? [])
    .map((a) => a.id ?? a.title ?? "")
    .join(", ");
  checks.cmdKActions = {
    actionsOpened: Boolean(state.actionsDialog),
    actionIds,
    ok:
      Boolean(state.actionsDialog) &&
      actionIds.includes("flow_desk_session_copy_transcript") &&
      actionIds.includes("flow_desk_session_stop"),
  };
  await shot("cmdk-actions");
  // Close by toggling ⌘K again — the detached actions window owns real
  // Escape presses; a main-window-targeted Escape would background the
  // session underneath instead of closing the dialog.
  if (state.actionsDialog) {
    await pressMain("k", ["cmd"]);
    await pollState((s) => !s.actionsDialog, 5_000);
  }

  // -- 10. Run once (⇧↵) on a project flow: registry, not a session --------
  // ⌘⇧D backgrounds via the app-level interceptor — robust to whatever
  // focus state the detached actions window left behind.
  await pressMain("d", ["cmd", "shift"]);
  state = await pollState((s) => flowUx(s)?.activeVariant === "flash", 5_000);
  if (flowUx(state)?.activeVariant !== "flash") {
    await pressMain("escape");
    state = await pollState((s) => flowUx(s)?.activeVariant === "flash", 5_000);
  }
  checks.cmdKActions.deskReturned = flowUx(state)?.activeVariant === "flash";
  await deskFilter("fast-success");
  const sessionsBefore = ((flowUx((await driver.getState()) as Json)?.sessions as Json[]) ?? []).length;
  await pressMain("enter", ["shift"]);
  state = await pollState((s) =>
    ((flowUx(s)?.runs as Json[]) ?? []).some(
      (r) => r.flowId === "project:fast-success.fasteng" && r.phase === "Succeeded",
    ),
  );
  fx = flowUx(state);
  const runOnce = ((fx?.runs as Json[]) ?? []).findLast(
    (r) => r.flowId === "project:fast-success.fasteng",
  );
  checks.runOnceBackground = {
    phase: runOnce?.phase,
    sessionsBefore,
    sessionsAfter: ((fx?.sessions as Json[]) ?? []).length,
    ok:
      runOnce?.phase === "Succeeded" &&
      ((fx?.sessions as Json[]) ?? []).length === sessionsBefore,
  };
  await shot("run-once-succeeded");

  // -- 11. Cleanup: leave the app hidden ------------------------------------
  await returnToRoot();
  await pressMain("escape"); // root escape hides
  await Bun.sleep(400);
  state = (await driver.getState()) as Json;
  checks.cleanupHidden = {
    windowVisible: state.windowVisible,
    ok: state.windowVisible === false,
  };
} finally {
  const names = Object.keys(checks);
  const passed = names.filter((n) => checks[n]?.ok === true);
  receipt.summary = { passed: passed.length, total: names.length };
  console.log(JSON.stringify(receipt, null, 2));
  await driver.close();
}
