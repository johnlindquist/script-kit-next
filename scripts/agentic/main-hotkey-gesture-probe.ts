/**
 * T8 gesture grammar runtime proof:
 * - key-down → main window visible (ShowImmediate)
 * - tap (down+up + double window) → Day Page surface
 * - launcher query carry-over into day-page editor
 * - double-tap → Agent Chat view
 * - stable main window id across transitions
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/t8-gesture/script-kit-gpui \
 *     bun scripts/agentic/main-hotkey-gesture-probe.ts
 */
import { Driver } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/t8-gesture/script-kit-gpui";

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
    { type: "getElements", target: { id: "main" } },
    { timeoutMs: 5000 },
  )) as Json;
  const list = (elements.elements ?? []) as Json[];
  const editor = list.find((el) => el.id === "day-page-editor");
  return (editor?.value as string | undefined) ?? null;
}

async function tapHotkey(driver: Driver, label: string) {
  await simulateMainHotkeyGesture(driver, "down", `${label}-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${label}-up`);
  await Bun.sleep(350);
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

  const before = await listMainWindow();
  check("starts_hidden", before?.visible !== true, { before });

  await simulateMainHotkeyGesture(driver, "down", "show-down");
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 8000 });
  await Bun.sleep(400);

  const afterDown = await listMainWindow();
  const windowIdAfterDown = afterDown?.id;
  check("keydown_shows_main", afterDown?.visible === true, { afterDown });

  await driver.setFilterAndWait("carry me to the page");
  await tapHotkey(driver, "toggle-to-day-page");

  const stateAfterTap = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("tap_opens_day_page_surface", stateAfterTap.semanticSurface === "dayPage", {
    semanticSurface: stateAfterTap.semanticSurface,
  });

  const editorAfterCarry = await getEditorText(driver);
  check("carry_over_in_editor", editorAfterCarry?.includes("carry me to the page") === true, {
    editorAfterCarry,
    inputValue: stateAfterTap.inputValue,
  });
  check("launcher_query_cleared", (stateAfterTap.inputValue ?? "") === "", {
    inputValue: stateAfterTap.inputValue,
  });

  await doubleTapHotkey(driver, "agent-chat");
  await Bun.sleep(600);
  const stateAgent = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check(
    "double_tap_agent_chat",
    String(stateAgent.semanticSurface ?? "")
      .toLowerCase()
      .includes("agentchat"),
    {
      semanticSurface: stateAgent.semanticSurface,
    },
  );

  const afterAgent = await listMainWindow();
  check(
    "stable_main_window_id",
    windowIdAfterDown === "main" && afterAgent?.id === "main",
    { windowIdAfterDown, afterAgentId: afterAgent?.id },
  );

  console.log(JSON.stringify({ ok: failures.length === 0, failures, receipts }, null, 2));
  await driver.close();
  process.exit(failures.length === 0 ? 0 : 1);
} catch (error) {
  console.error(error);
  if (globalDriver) await globalDriver.close().catch(() => {});
  process.exit(1);
}
