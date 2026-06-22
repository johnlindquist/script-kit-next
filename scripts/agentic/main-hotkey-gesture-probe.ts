/**
 * T8 gesture grammar runtime proof:
 * - key-down → main window visible (ShowImmediate), opening tap stays on launcher
 * - tap-while-open (down+up + double window) → main window hidden
 * - hold-from-closed → Day Page surface
 * - tap-while-open on Day Page → main window hidden
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
import { openDayPage, tapMainHotkey } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/main-hotkey-open-close/script-kit-gpui";

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

  await driver.batch([{ type: "setInput", text: "" }], { timeoutMs: 5000 });
  await Bun.sleep(120);
  await tapMainHotkey(driver, runId, "close-main-from-launcher");

  const stateAfterClose = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("tap_while_open_hides_main", stateAfterClose.windowVisible === false, {
    promptType: stateAfterClose.promptType,
    windowVisible: stateAfterClose.windowVisible,
  });

  await openDayPage(driver, runId);
  const stateAfterHold = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("hold_from_closed_opens_day_page", stateAfterHold.promptType === "dayPage", {
    promptType: stateAfterHold.promptType,
    windowVisible: stateAfterHold.windowVisible,
  });

  await tapMainHotkey(driver, runId, "close-main-from-day-page");
  const stateAfterDayClose = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("tap_from_day_page_hides_main", stateAfterDayClose.windowVisible === false, {
    promptType: stateAfterDayClose.promptType,
    windowVisible: stateAfterDayClose.windowVisible,
  });

  const afterClose = await listMainWindow();
  check(
    "stable_main_window_id",
    windowIdAfterDown === "main" && afterClose?.id === "main",
    { windowIdAfterDown, afterCloseId: afterClose?.id },
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
