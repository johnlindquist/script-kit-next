import { Driver, type Json } from "../devtools/driver";

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

  // This helper opens Today; it must never exercise the microphone shortcut.
  for (const stroke of [1, 2]) {
    await simulateMainHotkeyGesture(driver, "down", `${runId}-today-${stroke}-down`);
    await simulateMainHotkeyGesture(driver, "up", `${runId}-today-${stroke}-up`);
  }
  await driver.waitForState(
    { windowVisible: true, promptType: "dayPage" },
    { timeoutMs: 8000 },
  );
  await Bun.sleep(250);

  state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  return state;
}
