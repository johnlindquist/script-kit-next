#!/usr/bin/env bun
/**
 * Red/green receipts for the Flow Desk (Conversation Desk, fusion-ultra
 * 2026-07-09). Every flow is an agent identity; Enter means CONVERSE.
 *
 * Runs against deterministic fixtures — fake engines + a fake installed
 * flows package — so every receipt is reproducible with zero tokens:
 *  - fixtures/flow-ux-project: project flows discovered via `md roster`.
 *  - fixtures/flow-desk-package: fake @johnlindquist/flows package with a
 *    bun-linked-style wrapper (`flow-hello-agent`) that idles like a TUI.
 *
 * Receipt matrix:
 *  1. Desk entry: single visible "Flows" built-in opens the desk;
 *     roster + package corpus both present (selectedFlowId proves rows).
 *  2. Provenance: package flow row shows purpose + origin (screenshot +
 *     descriptor assertions via filtered selection).
 *  3. Enter = converse: creates a live PTY session running the wrapper
 *     (sessions[] gains a live entry; view flips to flow session).
 *  4. ⌘⇧D backgrounds: view returns to desk, session stays LIVE.
 *  5. Active row re-entry: selecting the session row reopens the SAME
 *     session id (same PTY entity — no respawn, sessions.length stable).
 *  6. ⌘K: desk actions dialog opens (actions window visible).
 *  7. Run once (⇧↵ on a project flow): registry run reaches Succeeded
 *     without creating a conversation session.
 *  8. Cleanup: escape + hide leaves windowVisible:false.
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
    PATH: `${join(FIXTURE, "bin")}:${join(PACKAGE_FIXTURE, "bin")}:${process.env.PATH ?? ""}`,
  },
});

type Json = Record<string, any>;
const receipt: Json = { binary, fixture: FIXTURE, packageFixture: PACKAGE_FIXTURE, checks: {} };
const checks = receipt.checks as Json;
let shotIndex = 0;

const flowUx = (state: Json): Json | null => (state?.flowUx as Json) ?? null;

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
// element-level on_key_down handlers like the desk's).
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
  for (let i = 0; i < 8; i++) {
    const st = (await driver.getState()) as Json;
    if (st.windowVisible === false) {
      driver.send({ type: "show" });
      await Bun.sleep(300);
      continue;
    }
    if (st.promptType === "none" && !st.inputValue) return;
    await pressMain("escape");
    await Bun.sleep(200);
  }
};

const visibleTexts = async (): Promise<string> => {
  const result = (await driver.getElements({ limit: 200 })) as Json;
  return ((result.elements as Json[]) ?? [])
    .map((e) => `${e.text ?? ""}|${e.value ?? ""}`)
    .join("\n");
};

const openDesk = async () => {
  await returnToRoot();
  await driver.setFilterAndWait("Flows");
  await pressMain("enter");
  await Bun.sleep(400);
};

const deskFilter = async (text: string) => {
  await driver.setFilterAndWait(text);
  await Bun.sleep(300); // corpus filter applies synchronously; settle paint
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

  // -- 2. Package provenance: filtering finds the package agent -----------
  await deskFilter("hello");
  state = (await driver.getState()) as Json;
  fx = flowUx(state);
  const packageRowSelected = fx?.selectedFlowId === "package:flow-hello-agent";
  const deskTexts = await visibleTexts();
  checks.packageProvenance = {
    selectedFlowId: fx?.selectedFlowId,
    rowShowsFriendlyName: deskTexts.includes("Hello Agent"),
    rowShowsOrigin: deskTexts.includes("@johnlindquist/flows"),
    ok:
      packageRowSelected &&
      deskTexts.includes("Hello Agent") &&
      deskTexts.includes("@johnlindquist/flows"),
  };
  await shot("package-provenance");

  // -- 3. Enter = converse: live PTY session running the wrapper ----------
  await pressMain("enter");
  state = await pollState((s) => (flowUx(s)?.sessions as Json[])?.length >= 1);
  fx = flowUx(state);
  const session = (fx?.sessions as Json[])?.at(-1);
  checks.enterConverses = {
    sessionCount: (fx?.sessions as Json[])?.length ?? 0,
    flowId: session?.flowId,
    live: session?.live,
    state: session?.state,
    promptType: state.promptType,
    ok:
      session?.flowId === "package:flow-hello-agent" &&
      session?.live === true &&
      state.promptType === "flowSession",
  };
  await shot("conversation-open");

  // -- 4. ⌘⇧D backgrounds: desk returns, session stays LIVE ---------------
  await pressMain("d", ["cmd", "shift"]);
  state = await pollState((s) => flowUx(s)?.activeVariant === "flash");
  fx = flowUx(state);
  const bgSession = (fx?.sessions as Json[])?.at(-1);
  checks.backgroundKeepsAlive = {
    activeVariant: fx?.activeVariant,
    live: bgSession?.live,
    ok: fx?.activeVariant === "flash" && bgSession?.live === true,
  };
  await shot("backgrounded-desk");

  // -- 5. Active row re-entry: SAME session id, no respawn ----------------
  const sessionId = bgSession?.sessionId;
  await deskFilter("");
  await pressMain("enter"); // sessions sort first: selection 0 is the live row
  state = await pollState((s) => s.promptType === "flowSession");
  fx = flowUx(state);
  const reSession = (fx?.sessions as Json[])?.at(-1);
  checks.reentrySameSession = {
    sessionId,
    reenteredId: reSession?.sessionId,
    sessionCount: (fx?.sessions as Json[])?.length ?? 0,
    ok:
      state.promptType === "flowSession" &&
      typeof sessionId === "number" &&
      reSession?.sessionId === sessionId &&
      ((fx?.sessions as Json[])?.length ?? 0) === 1,
  };
  await shot("reentered-session");

  // -- 6. ⌘K opens the Flow Desk actions dialog ---------------------------
  await pressMain("k", ["cmd"]);
  state = await pollState((s) => Boolean(s.actionsDialog), 5_000);
  const actionsOpened = Boolean(state.actionsDialog);
  const actionTitles = ((state.actionsDialog?.visibleActions as Json[]) ?? [])
    .map((a) => a.title ?? a.id ?? "")
    .join(", ");
  checks.cmdKActions = { actionsOpened, actionTitles, ok: actionsOpened };
  await shot("cmdk-actions");
  if (actionsOpened) {
    await pressMain("escape");
    await Bun.sleep(300);
  }

  // -- 7. Run once (⇧↵) on a project flow: registry, not a session --------
  await pressMain("d", ["cmd", "shift"]); // ensure we're back on the desk
  await Bun.sleep(300);
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

  // -- 8. Cleanup: leave the app hidden ------------------------------------
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
