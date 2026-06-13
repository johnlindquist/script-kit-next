/**
 * T8 gesture grammar runtime proof:
 * - key-down → main window visible (ShowImmediate), opening tap stays on launcher
 * - tap-while-open (down+up + double window) → Day Page surface
 * - launcher query carry-over into day-page editor; cleared and not restored
 * - tap-while-open on Day Page → back to launcher
 * - double-tap → Agent Chat view
 * - stable main window id across transitions
 *
 * Timing intent: the opening press is a TAP — key-up is sent ~30ms after
 * key-down, well inside HOLD_MS (250ms). The 400ms settles afterwards let the
 * classifier's deferred Tap resolve (DOUBLE_MS = 300ms) before the next gesture.
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/t8-gesture/script-kit-gpui \
 *     bun scripts/agentic/main-hotkey-gesture-probe.ts
 */
import { Driver } from "../devtools/driver";
import { tapMainHotkey } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/today/script-kit-gpui";

type Json = Record<string, unknown>;
const receipts: Record<string, Json> = {};
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

async function simulateMainHotkeyGesture(
  driver: Driver,
  phase: "down" | "up",
  requestId: string,
): Promise<Json> {
  return driver.request(
    {
      type: "simulateMainHotkeyGesture",
      phase,
      requestId,
    },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  ) as Promise<Json>;
}

async function listMainWindow(): Promise<Json | null> {
  const driver = globalDriver!;
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 5000 },
  )) as Json;
  const windows = (result.windows ?? []) as Json[];
  return windows.find((w) => w.id === "main") ?? null;
}

async function getEditorText(driver: Driver): Promise<string | null> {
  const elements = (await driver.request(
    { type: "getElements", target: { type: "main" } },
    { timeoutMs: 5000 },
  )) as Json;
  const editor = walkElements(elements).find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  return (editor?.value as string | undefined) ?? null;
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

async function doubleTapHotkey(driver: Driver, label: string) {
  await simulateMainHotkeyGesture(driver, "down", `${label}-d1-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${label}-d1-up`);
  await Bun.sleep(80);
  await simulateMainHotkeyGesture(driver, "down", `${label}-d2-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${label}-d2-up`);
  await Bun.sleep(200);
}

let globalDriver: Driver | null = null;

try {
  const driver = await Driver.launch({
    binary: BINARY,
    sandboxHome: true,
    sessionName: "main-hotkey-gesture-probe",
    env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
  });
  globalDriver = driver;
  const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

  const before = await listMainWindow();
  check("starts_hidden", before?.visible !== true, { before });

  // Opening tap: key-down shows the window immediately; release quickly so the
  // press is a tap (not a hold). The deferred Tap for this opening press must
  // resolve to the launcher steady state, NOT toggle to Day Page.
  await simulateMainHotkeyGesture(driver, "down", "show-down");
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", "show-up");
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 8000 });

  const afterDown = await listMainWindow();
  const windowIdAfterDown = afterDown?.id;
  check("keydown_shows_main", afterDown?.visible === true, { afterDown });

  // Let the opening tap's double window expire, then confirm we're still on
  // the launcher (opening tap must not toggle).
  await Bun.sleep(400);
  const stateAfterShow = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("opening_tap_stays_on_launcher", stateAfterShow.promptType === "none", {
    promptType: stateAfterShow.promptType,
  });

  await driver.setFilterAndWait("carry me to the page");
  await tapMainHotkey(driver, runId, "toggle-to-day-page");

  const stateAfterTap = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("tap_opens_day_page_surface", stateAfterTap.promptType === "dayPage", {
    promptType: stateAfterTap.promptType,
  });

  const editorAfterCarry = await getEditorText(driver);
  check(
    "carry_over_in_editor",
    editorAfterCarry?.includes("carry me to the page") === true,
    { editorAfterCarry },
  );

  // Tap back to the launcher: query must NOT be restored.
  await tapMainHotkey(driver, runId, "toggle-back-to-launcher");
  const stateBack = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("tap_back_returns_to_launcher", stateBack.promptType === "none", {
    promptType: stateBack.promptType,
  });
  check("launcher_query_cleared", (stateBack.inputValue ?? "") === "", {
    inputValue: stateBack.inputValue,
  });

  await doubleTapHotkey(driver, "agent-chat");
  await Bun.sleep(600);
  const stateAgent = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "double_tap_agent_chat",
    String(stateAgent.promptType ?? "")
      .toLowerCase()
      .includes("agentchat"),
    {
      promptType: stateAgent.promptType,
    },
  );

  const afterAgent = await listMainWindow();
  check(
    "stable_main_window_id",
    windowIdAfterDown === "main" && afterAgent?.id === "main",
    { windowIdAfterDown, afterAgentId: afterAgent?.id },
  );

  console.log(
    JSON.stringify({ ok: failures.length === 0, failures, receipts }, null, 2),
  );
  await driver.close();
  process.exit(failures.length === 0 ? 0 : 1);
} catch (error) {
  console.error(error);
  if (globalDriver) await globalDriver.close().catch(() => {});
  process.exit(1);
}
