/**
 * Red/green proof: clicking the main window (dead space used for window-drag,
 * or a list row) must NOT blur the main filter input — typing afterwards must
 * still land in the filter.
 *
 * Root cause under test: GPUI auto-transfers keyboard focus to the
 * script_list root's tracked handle on any mouse down that no deeper
 * focusable claimed (paint_mouse_listeners in vendor/gpui/src/elements/div.rs),
 * which silently killed typing until the window was reopened.
 *
 * Uses simulateGpuiEvent (real GPUI dispatch pipeline, window-relative
 * coordinates) — NOT legacy simulateKey/setFilter, which bypass GPUI focus
 * and would mask the bug.
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/<name>/script-kit-gpui \
 *     bun scripts/agentic/main-click-focus-probe.ts
 */
import { Driver } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/main-click-focus/script-kit-gpui";

type Json = Record<string, any>;
const receipts: Json = {};
const failures: string[] = [];
function check(name: string, ok: boolean, detail: Json) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "main-click-focus-probe",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

async function gpuiEvent(event: Json): Promise<Json> {
  return (await driver.request(
    { type: "simulateGpuiEvent", event },
    { expect: "simulateGpuiEventResult", timeoutMs: 3000 },
  )) as Json;
}

async function typeText(text: string) {
  for (const ch of text) {
    await gpuiEvent({ type: "keyDown", key: ch, text: ch });
  }
}

async function clickAt(x: number, y: number) {
  await gpuiEvent({ type: "mouseDown", x, y });
  await gpuiEvent({ type: "mouseUp", x, y });
}

async function snapshot(): Promise<Json> {
  const state = (await driver.getState({ timeoutMs: 3000 })) as Json;
  return {
    inputValue: state.inputValue ?? null,
    isFocused: state.isFocused ?? null,
    windowVisible: state.windowVisible ?? null,
  };
}

async function mainBounds(): Promise<Json | null> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  const windows: Json[] = result.windows ?? [];
  return windows.find((w) => w.id === "main")?.bounds ?? null;
}

try {
  driver.send({ type: "show", requestId: "probe-show-main" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 5000 });
  await Bun.sleep(800);
  // NOTE: a freshly launched instance has GPUI focus on the app ROOT handle
  // (app_run_setup focuses the root after the input's pending focus), so
  // typing before any click does not insert. That startup quirk is the same
  // blur state this fix recovers from, which is why every round below clicks
  // first and types second.
  receipts.after_show = await snapshot();

  const bounds = await mainBounds();
  check("main_window_bounds_available", bounds !== null, { bounds });

  if (bounds) {
    const clickX = bounds.width / 2;
    const clickY = bounds.height - 90;

    // Round 1: with an empty filter the click lands on a list row.
    // Typing right after must land in the filter input. The typed text is a
    // no-results filter, which makes round 2's click hit pure dead space.
    await clickAt(clickX, clickY);
    await Bun.sleep(300);
    await typeText("zzqj");
    await Bun.sleep(300);
    const afterRowClick = await snapshot();
    check(
      "typing_works_after_list_row_click",
      afterRowClick.inputValue === "zzqj",
      afterRowClick,
    );

    // Round 2: same point is now dead space (no results below the input) —
    // the window-drag scenario from the bug report.
    await clickAt(clickX, clickY);
    await Bun.sleep(300);
    await typeText("x");
    await Bun.sleep(300);
    const afterDeadSpaceClick = await snapshot();
    check(
      "typing_works_after_dead_space_click",
      afterDeadSpaceClick.inputValue === "zzqjx",
      afterDeadSpaceClick,
    );
  }
} finally {
  await driver.close();
}

console.log(
  JSON.stringify({ ok: failures.length === 0, failures, receipts }, null, 2),
);
process.exit(failures.length === 0 ? 0 : 1);
