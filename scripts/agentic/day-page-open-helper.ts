import { Driver, type Json } from "../devtools/driver";

const HOLD_MS = 360;

async function simulateMainHotkeyGesture(
  driver: Driver,
  phase: "down" | "up",
  requestId: string,
) {
  return driver.request(
    { type: "simulateMainHotkeyGesture", phase, requestId },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  );
}

export async function tapMainHotkey(driver: Driver, runId: string, label: string) {
  await simulateMainHotkeyGesture(driver, "down", `${runId}-${label}-down`);
  await Bun.sleep(30);
  await simulateMainHotkeyGesture(driver, "up", `${runId}-${label}-up`);
  await Bun.sleep(420);
}

export async function openDayPage(driver: Driver, runId: string): Promise<Json> {
  let state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  if (state.promptType === "dayPage" && state.windowVisible === true) {
    return state;
  }

  if (state.windowVisible === true && state.promptType === "none") {
    await driver.batch([{ type: "setInput", text: "" }], { timeoutMs: 5000 });
    await Bun.sleep(120);
    await tapMainHotkey(driver, runId, "tap-launcher-to-day-page");
    await driver.waitForState(
      { windowVisible: true, promptType: "dayPage" },
      { timeoutMs: 8000 },
    );
    await Bun.sleep(250);
    return (await driver.getState({ timeoutMs: 5000 })) as Json;
  }

  if (state.windowVisible === true) {
    await driver.simulateKey("escape");
    await Bun.sleep(420);
    state = (await driver.getState({ timeoutMs: 5000 })) as Json;
    if (state.windowVisible === true && state.promptType === "none") {
      await tapMainHotkey(driver, runId, "tap-launcher-to-day-page-after-escape");
      await driver.waitForState(
        { windowVisible: true, promptType: "dayPage" },
        { timeoutMs: 8000 },
      );
      await Bun.sleep(250);
      return (await driver.getState({ timeoutMs: 5000 })) as Json;
    }
  }

  await simulateMainHotkeyGesture(driver, "down", `${runId}-open-day-page-hold-down`);
  await Bun.sleep(HOLD_MS);
  await simulateMainHotkeyGesture(driver, "up", `${runId}-open-day-page-hold-up`);
  await driver.waitForState(
    { windowVisible: true, promptType: "dayPage" },
    { timeoutMs: 8000 },
  );
  await Bun.sleep(250);

  state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  return state;
}
