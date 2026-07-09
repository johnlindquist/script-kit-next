#!/usr/bin/env bun
/**
 * Red/green receipts for the Flow UX variations + Flow Manager
 * (docs/ai/flow-ux-protocol.md §5/§6).
 *
 * Runs against the deterministic fixture project — fake engines on PATH,
 * SCRIPT_KIT_FLOW_UX_CWD pins discovery — so every receipt is reproducible
 * with zero tokens. sandboxHome keeps ~/.mdflow empty (project flows only).
 *
 * Receipt matrix:
 *  1. Hidden built-ins: absent from the empty-query menu, revealed by search.
 *  2. Flash: Enter launches inline, launch ack fast, run reaches Succeeded.
 *  3. Flash Esc: backgrounds the engaged run, never cancels it.
 *  4. Dispatch: Enter backgrounds immediately, view keeps focus.
 *  5. Streaming: outputTail grows across polls.
 *  6. Failing flow: phase Failed, exit code 42 surfaced.
 *  7. Cancel: slow run SIGTERMed via manager path, process group dies.
 *  8. Workflow DAG: three steps recorded in order, all exit 0.
 *  9. Password redaction: state JSON never contains a password value.
 * 10. Multi-run coexistence: cancel one run, siblings unaffected.
 * 11. Cleanup: escape + hide leaves windowVisible:false.
 */
import { join } from "node:path";
import { Driver } from "../devtools/driver.ts";

const FIXTURE = join(import.meta.dir, "fixtures/flow-ux-project");
const SHOTS = join(import.meta.dir, "../../.test-screenshots");

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/flow-ux/script-kit-gpui";

const driver = await Driver.launch({
  binary,
  sessionName: `flow-ux-probe-${process.pid}`,
  sandboxHome: true,
  env: {
    SCRIPT_KIT_FLOW_UX_CWD: FIXTURE,
    PATH: `${join(FIXTURE, "bin")}:${process.env.PATH ?? ""}`,
  },
});

type Json = Record<string, any>;
const receipt: Json = { binary, fixture: FIXTURE, checks: {} };
const checks = receipt.checks as Json;
let shotIndex = 0;

const flowUx = (state: Json): Json | null => (state?.flowUx as Json) ?? null;

const pollFlowUx = async (
  pred: (fx: Json) => boolean,
  timeoutMs = 8_000,
): Promise<Json | null> => {
  const deadline = Date.now() + timeoutMs;
  let fx: Json | null = null;
  while (Date.now() < deadline) {
    fx = flowUx((await driver.getState()) as Json);
    if (fx && pred(fx)) return fx;
    await Bun.sleep(100);
  }
  return fx;
};

const runByFlow = (fx: Json | null, flowId: string): Json | undefined =>
  (fx?.runs as Json[] | undefined)?.findLast?.((r) => r.flowId === flowId) ??
  (fx?.runs as Json[] | undefined)?.filter((r) => r.flowId === flowId).at(-1);

// OS-level truth: registry phase is not enough — cancellation receipts
// verify the process GROUP is actually dead (pgrep -g matches any member).
const groupAlive = (pgid: number): boolean =>
  Bun.spawnSync(["pgrep", "-g", String(pgid)]).exitCode === 0;

const waitForGroupDead = async (pgid: number, timeoutMs: number): Promise<boolean> => {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (!groupAlive(pgid)) return true;
    await Bun.sleep(100);
  }
  return !groupAlive(pgid);
};

const shot = async (name: string) => {
  shotIndex += 1;
  await driver.captureScreenshot({
    savePath: join(SHOTS, `flow-ux-${String(shotIndex).padStart(2, "0")}-${name}.png`),
  });
};

// Real-dispatch key press through the GPUI window (the only path that hits
// element-level on_key_down handlers like the Flow UX view's).
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
  for (let i = 0; i < 5; i++) {
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

const openVariant = async (entryName: string) => {
  // Use real typed search + Enter — these built-ins are not in the
  // triggerBuiltin registry by design.
  await returnToRoot();
  await driver.setFilterAndWait(entryName);
  await pressMain("enter");
  await Bun.sleep(300);
};

const launchSelected = async (
  filter: string,
  key: string,
  modifiers: string[] = [],
) => {
  await driver.setFilterAndWait(filter);
  await Bun.sleep(250); // roster filter applies synchronously; settle paint
  await pressMain(key, modifiers);
};

try {
  driver.send({ type: "show" });
  await driver.waitForSettle();

  // -- 1. Hidden built-ins ------------------------------------------------
  // getElements returns the visible rows with display text — the state-first
  // way to prove menu membership (getState carries counts, not names).
  const visibleTexts = async (): Promise<string> => {
    const result = (await driver.getElements({ limit: 200 })) as Json;
    return ((result.elements as Json[]) ?? [])
      .map((e) => `${e.text ?? ""}|${e.value ?? ""}`)
      .join("\n");
  };
  await driver.setFilterAndWait("");
  await Bun.sleep(300);
  const emptyTexts = await visibleTexts();
  const hiddenWhenEmpty =
    !emptyTexts.includes("Flow UX — Flash") && !emptyTexts.includes("Flow Manager");
  await driver.setFilterAndWait("Flow UX");
  await Bun.sleep(300);
  const searchTexts = await visibleTexts();
  const revealedBySearch =
    searchTexts.includes("Flow UX — Flash") &&
    searchTexts.includes("Flow UX — Dispatch") &&
    searchTexts.includes("Flow UX — Lens") &&
    searchTexts.includes("Flow UX — Mission Control");
  // The single VISIBLE entry point (council: one "Flows" entry, not four
  // hidden built-ins) must rank for its own name.
  await driver.setFilterAndWait("Flows");
  await Bun.sleep(300);
  const flowsEntryVisible = (await visibleTexts())
    .split("\n")
    .some((line) => line.startsWith("Flows|"));
  checks.hiddenBuiltins = {
    hiddenWhenEmpty,
    revealedBySearch,
    flowsEntryVisible,
    ok: hiddenWhenEmpty && revealedBySearch && flowsEntryVisible,
  };
  await shot("search-reveals-variants");

  // -- 2. Flash (via the visible "Flows" entry): inline launch + speed -----
  await openVariant("Flows");
  let fx = await pollFlowUx((f) => f.activeVariant === "flash" && f.roster?.status === "ready");
  checks.flashOpen = {
    activeVariant: fx?.activeVariant,
    rosterStatus: fx?.roster?.status,
    rosterCount: fx?.roster?.count,
    ok: fx?.activeVariant === "flash" && fx?.roster?.status === "ready" && (fx?.roster?.count ?? 0) >= 6,
  };
  await shot("flash-roster");

  await launchSelected("fast-success", "enter");
  fx = await pollFlowUx((f) => runByFlow(f, "project:fast-success.fasteng")?.phase === "Succeeded");
  const fastRun = runByFlow(fx, "project:fast-success.fasteng");
  checks.flashInlineRun = {
    phase: fastRun?.phase,
    engagement: fastRun?.engagement,
    launchAckMs: fastRun?.launchAckMs,
    firstOutputMs: fastRun?.firstOutputMs,
    ok:
      fastRun?.phase === "Succeeded" &&
      fastRun?.engagement === "Inline" &&
      typeof fastRun?.launchAckMs === "number" &&
      fastRun.launchAckMs <= 100,
  };
  await shot("flash-inline-succeeded");

  // -- 3. Flash Esc backgrounds a live run, never cancels ------------------
  await launchSelected("slow-cancellable", "enter");
  fx = await pollFlowUx((f) => runByFlow(f, "project:slow-cancellable.sloweng")?.phase === "Running");
  await pressMain("escape");
  await Bun.sleep(400);
  fx = flowUx((await driver.getState()) as Json);
  const slowRun = runByFlow(fx, "project:slow-cancellable.sloweng");
  checks.flashEscBackgrounds = {
    phase: slowRun?.phase,
    engagement: slowRun?.engagement,
    ok: slowRun?.phase === "Running" && slowRun?.engagement === "Background",
  };
  await shot("flash-esc-backgrounded");

  // -- 4+5. Dispatch: background launch + streaming tail growth ------------
  await openVariant("Flow UX — Dispatch");
  await pollFlowUx((f) => f.activeVariant === "dispatch");
  await launchSelected("streaming-output", "enter");
  fx = await pollFlowUx((f) => {
    const r = runByFlow(f, "project:streaming-output.streameng");
    return r?.phase === "Running" || r?.phase === "Succeeded";
  });
  const streamEarly = runByFlow(fx, "project:streaming-output.streameng");
  const earlyLines = streamEarly?.outputLineCount ?? 0;
  const dispatchStillFlowView = fx?.activeVariant === "dispatch";
  fx = await pollFlowUx(
    (f) => runByFlow(f, "project:streaming-output.streameng")?.phase === "Succeeded",
    10_000,
  );
  const streamDone = runByFlow(fx, "project:streaming-output.streameng");
  checks.dispatchStreaming = {
    engagement: streamDone?.engagement,
    earlyLines,
    finalLines: streamDone?.outputLineCount,
    viewStayed: dispatchStillFlowView,
    ok:
      streamDone?.phase === "Succeeded" &&
      streamDone?.engagement === "Background" &&
      dispatchStillFlowView &&
      (streamDone?.outputLineCount ?? 0) > earlyLines,
  };
  await shot("dispatch-streamed");

  // -- 5b. Lens: free explain preview, no run launched ----------------------
  await openVariant("Flow UX — Lens");
  await pollFlowUx((f) => f.activeVariant === "lens");
  const runsBeforeLens = ((flowUx((await driver.getState()) as Json)?.runs as Json[]) ?? []).length;
  await driver.setFilterAndWait("input-matrix");
  fx = await pollFlowUx((f) => f.preview?.flowId === "project:input-matrix" && f.preview?.valid === true);
  const runsAfterLens = ((fx?.runs as Json[]) ?? []).length;
  checks.lensPreview = {
    previewFlowId: fx?.preview?.flowId,
    previewValid: fx?.preview?.valid,
    fingerprint: typeof fx?.preview?.fingerprint === "string",
    runsLaunched: runsAfterLens - runsBeforeLens,
    ok:
      fx?.preview?.flowId === "project:input-matrix" &&
      fx?.preview?.valid === true &&
      runsAfterLens === runsBeforeLens,
  };
  await shot("lens-preview");

  // -- 5c. Mission Control built-in opens the Flow Manager window -----------
  await openVariant("Flow UX — Mission Control");
  await Bun.sleep(600);
  const mcWindows = JSON.stringify(await driver.listAutomationWindows());
  checks.missionControlOpensManager = {
    managerListed: mcWindows.includes("flowManager"),
    ok: mcWindows.includes("flowManager"),
  };
  await shot("mission-control-manager");
  // Close the manager (Esc in manager window) and return to Dispatch for the
  // remaining receipts.
  await driver.simulateGpuiEvent(
    { type: "keyDown", key: "escape" },
    { target: { type: "id", id: "flowManager" }, timeoutMs: 4_000 },
  ).catch(() => null);
  await Bun.sleep(400);
  await openVariant("Flow UX — Dispatch");
  await pollFlowUx((f) => f.activeVariant === "dispatch");

  // -- 6. Failing flow surfaces exit 42 ------------------------------------
  await launchSelected("failing-flow", "enter");
  fx = await pollFlowUx((f) => runByFlow(f, "project:failing-flow.faileng")?.phase === "Failed");
  const failRun = runByFlow(fx, "project:failing-flow.faileng");
  checks.failingFlow = {
    phase: failRun?.phase,
    exitCode: failRun?.exitCode,
    errorMessage: failRun?.errorMessage,
    ok: failRun?.phase === "Failed" && failRun?.exitCode === 42,
  };

  // -- 8. Workflow DAG steps ------------------------------------------------
  await launchSelected("workflow-dag", "enter");
  fx = await pollFlowUx((f) => runByFlow(f, "project:workflow-dag")?.phase === "Succeeded");
  const dagRun = runByFlow(fx, "project:workflow-dag");
  const stepIds = (dagRun?.steps as Json[] | undefined)?.map((s) => s.stepId) ?? [];
  checks.workflowDag = {
    steps: dagRun?.steps,
    ok:
      dagRun?.phase === "Succeeded" &&
      JSON.stringify(stepIds) === JSON.stringify(["gather", "draft", "polish"]) &&
      (dagRun?.steps as Json[]).every((s) => s.completed && s.exitCode === 0),
  };
  await shot("dispatch-dag-done");

  // -- 9a. Input honesty: required input missing → Failed, surfaced --------
  await launchSelected("input-matrix", "enter");
  fx = await pollFlowUx((f) => {
    const r = runByFlow(f, "project:input-matrix");
    return r != null && r.phase !== "Starting" && r.phase !== "Running";
  });
  const inputMatrixRun = runByFlow(fx, "project:input-matrix");
  checks.requiredInputRejection = {
    phase: inputMatrixRun?.phase,
    errorMessage: inputMatrixRun?.errorMessage,
    // _token is a required password with no default; --events never
    // prompts, so the ONLY honest outcome is a surfaced failure.
    ok:
      inputMatrixRun?.phase === "Failed" &&
      typeof inputMatrixRun?.errorMessage === "string" &&
      inputMatrixRun.errorMessage.length > 0,
  };

  // -- 9b. Password redaction: default value must never surface ------------
  await launchSelected("input-defaults", "enter");
  fx = await pollFlowUx((f) => runByFlow(f, "project:input-defaults")?.phase === "Succeeded");
  const defaultsRun = runByFlow(fx, "project:input-defaults");
  const wholeState = JSON.stringify(await driver.getState());
  const elementsText = JSON.stringify(await driver.getElements({ limit: 200 }));
  const leaked =
    wholeState.includes("FIXTURE-SECRET-TOKEN-9F2") ||
    elementsText.includes("FIXTURE-SECRET-TOKEN-9F2");
  checks.passwordRedaction = {
    runPhase: defaultsRun?.phase,
    leaked,
    ok: defaultsRun?.phase === "Succeeded" && !leaked,
  };

  // -- 9c. Giant single-line output is bounded, never dropped --------------
  await launchSelected("giant-line", "enter");
  fx = await pollFlowUx((f) => runByFlow(f, "project:giant-line.gianteng")?.phase === "Succeeded");
  const giantRun = runByFlow(fx, "project:giant-line.gianteng");
  checks.giantLine = {
    phase: giantRun?.phase,
    outputLineCount: giantRun?.outputLineCount,
    tailBytes: (giantRun?.outputTail ?? "").length,
    ok:
      giantRun?.phase === "Succeeded" &&
      (giantRun?.outputLineCount ?? 0) > 0 &&
      (giantRun?.outputTail ?? "").length > 0 &&
      (giantRun?.outputTail ?? "").length <= 70_000,
  };

  // -- 7+10. Cancel one run; siblings unaffected ------------------------------
  // slow-cancellable from check 3 is still Running in Background. Launch a
  // second slow run, cancel only the FIRST, prove the second survives.
  await launchSelected("slow-cancellable", "enter", ["shift"]);
  await Bun.sleep(600);
  fx = flowUx((await driver.getState()) as Json);
  const slowRuns = ((fx?.runs as Json[]) ?? []).filter(
    (r) => r.flowId === "project:slow-cancellable.sloweng",
  );
  const [firstSlow, secondSlow] = slowRuns;
  // Open the manager WITHOUT launching anything: Cmd+Enter would launch the
  // filtered flow as a side effect (the council predicted exactly this trap)
  // — use the Flow Manager entry instead, then return to Dispatch for the
  // remaining launches.
  await openVariant("Flow Manager");
  await Bun.sleep(600);
  await openVariant("Flow UX — Dispatch");
  await pollFlowUx((f) => f.activeVariant === "dispatch");
  const windowsAfterManager = (await driver.listAutomationWindows()) as Json;
  const managerListed = JSON.stringify(windowsAfterManager).includes("flowManager");
  await shot("manager-open");

  // In the manager: Runs zone is default, newest-first selection = latest
  // run. Navigate to the FIRST slow run (older) and Cmd+Backspace it.
  // Selection order is registry order; use arrows then cancel.
  const managerTarget = { type: "id", id: "flowManager" };
  const pressManager = (key: string, modifiers: string[] = []) =>
    driver
      .simulateGpuiEvent(
        { type: "keyDown", key, modifiers },
        { target: managerTarget, timeoutMs: 4_000 },
      )
      .catch(() => null);
  const selectRunInManager = async (localId: number | undefined): Promise<boolean> => {
    if (localId == null) return false;
    const selectedNow = async () =>
      (((flowUx((await driver.getState()) as Json)?.runs as Json[]) ?? []).find(
        (r) => r.selected,
      ))?.localId;
    for (const direction of ["up", "down"] as const) {
      for (let i = 0; i < 8; i++) {
        if ((await selectedNow()) === localId) return true;
        await pressManager(direction);
        await Bun.sleep(120);
      }
      if ((await selectedNow()) === localId) return true;
    }
    return (await selectedNow()) === localId;
  };

  await selectRunInManager(firstSlow?.localId);
  await pressManager("backspace", ["cmd"]);

  fx = await pollFlowUx((f) => {
    const runs = (f.runs as Json[]) ?? [];
    return runs.some(
      (r) => r.localId === firstSlow?.localId && r.phase === "Cancelled",
    );
  }, 6_000);
  const runsNow = (fx?.runs as Json[]) ?? [];
  const firstNow = runsNow.find((r) => r.localId === firstSlow?.localId);
  const secondNow = runsNow.find((r) => r.localId === secondSlow?.localId);
  // OS truth: the cancelled run's process GROUP (md + engine + child
  // sleeper) must actually die; the sibling's group must stay alive.
  const firstGroupDead =
    typeof firstNow?.pid === "number" ? await waitForGroupDead(firstNow.pid, 4_000) : false;
  const secondGroupAlive =
    typeof secondNow?.pid === "number" ? groupAlive(secondNow.pid) : false;
  checks.selectiveCancel = {
    managerListed,
    firstPhase: firstNow?.phase,
    secondPhase: secondNow?.phase,
    firstGroupDead,
    secondGroupAlive,
    ok:
      managerListed &&
      firstNow?.phase === "Cancelled" &&
      firstGroupDead &&
      secondNow?.phase === "Running" &&
      secondGroupAlive,
  };
  await shot("manager-selective-cancel");

  // Cancel the surviving run too so nothing outlives the probe.
  if (secondNow?.phase === "Running") {
    await selectRunInManager(secondNow.localId);
    await pressManager("backspace", ["cmd"]);
    await pollFlowUx((f) =>
      ((f.runs as Json[]) ?? []).every((r) => r.phase !== "Running" && r.phase !== "Starting"),
    );
  }

  // -- 10b. Stubborn engine: SIGTERM ignored → SIGKILL escalation wins ------
  await launchSelected("stubborn-cancel", "enter", ["shift"]);
  fx = await pollFlowUx(
    (f) => runByFlow(f, "project:stubborn-cancel.stubborneng")?.phase === "Running",
  );
  const stubbornRun = runByFlow(fx, "project:stubborn-cancel.stubborneng");
  await selectRunInManager(stubbornRun?.localId);
  await pressManager("backspace", ["cmd"]);
  fx = await pollFlowUx(
    (f) => runByFlow(f, "project:stubborn-cancel.stubborneng")?.phase === "Cancelled",
    6_000,
  );
  const stubbornNow = runByFlow(fx, "project:stubborn-cancel.stubborneng");
  // SIGTERM is trapped, so death requires the 2s SIGKILL escalation.
  const stubbornGroupDead =
    typeof stubbornNow?.pid === "number"
      ? await waitForGroupDead(stubbornNow.pid, 5_000)
      : false;
  checks.stubbornKillEscalation = {
    phase: stubbornNow?.phase,
    stubbornGroupDead,
    ok: stubbornNow?.phase === "Cancelled" && stubbornGroupDead,
  };

  // -- 11. Cleanup: leave the app hidden -------------------------------------
  await driver.simulateGpuiEvent(
    { type: "keyDown", key: "escape" },
    { target: managerTarget, timeoutMs: 4_000 },
  ).catch(() => null);
  await Bun.sleep(300);
  await pressMain("escape");
  await Bun.sleep(200);
  await pressMain("escape");
  await Bun.sleep(200);
  driver.send({ type: "hide" });
  await Bun.sleep(400);
  const finalState = (await driver.getState()) as Json;
  const finalRuns = (flowUx(finalState)?.runs as Json[]) ?? [];
  const liveRuns = finalRuns.filter(
    (r) => r.phase === "Running" || r.phase === "Starting",
  );
  checks.cleanup = {
    windowVisible: finalState.windowVisible,
    liveRuns: liveRuns.length,
    liveFlowIds: liveRuns.map((r) => r.flowId),
    ok: finalState.windowVisible === false && liveRuns.length === 0,
  };

  receipt.ok = Object.values(checks).every((c: any) => c.ok === true);
} catch (error) {
  receipt.error = String(error);
  receipt.ok = false;
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
process.exit(receipt.ok ? 0 : 1);
