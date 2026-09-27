#!/usr/bin/env bun
/**
 * Runtime proof for the Day Page editor-host contract:
 * - Day uses components.notes_editor but does not mount a local spine overlay.
 * - Day @context typing round-trips through the main menu and cancels back.
 */
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const binary =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/day-notes-host-contract/script-kit-gpui";
const runId = `day-notes-host-${Date.now().toString(36)}`;
const outPath = ".test-output/day-notes-editor-host-contract-probe.json";

type Check = { name: string; pass: boolean; detail?: Json };

const checks: Check[] = [];
const failures: string[] = [];

function check(name: string, pass: boolean, detail: Json = {}) {
  checks.push({ name, pass, detail });
  if (!pass) failures.push(name);
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function waitFor<T>(
  label: string,
  read: () => T | Promise<T>,
  accept: (value: T) => boolean,
  timeoutMs = 12_000,
  intervalMs = 100,
): Promise<T> {
  const deadline = Date.now() + timeoutMs;
  let last: T | undefined;
  while (Date.now() < deadline) {
    last = await read();
    if (accept(last)) return last;
    await sleep(intervalMs);
  }
  throw new Error(`timeout waiting for ${label}: ${JSON.stringify(last)}`);
}

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") out.push(json);
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

function ids(elements: Json): string[] {
  return walkElements(elements)
    .map((el) => String(el.semanticId ?? el.id ?? ""))
    .filter(Boolean);
}

function localOverlayIds(elements: Json): string[] {
  return ids(elements).filter((id) => {
    const lower = id.toLowerCase();
    return (
      lower.includes("day-page-spine") ||
      lower.includes("day-spine") ||
      lower.includes("ready-to-send") ||
      lower.includes("prompt-builder") ||
      lower === "notes-spine-list"
    );
  });
}

function gpuiMainKey(
  driver: Driver,
  key: string,
  modifiers: string[] = [],
  text?: string,
): Promise<Json> {
  const event: Json = { type: "keyDown", key, modifiers };
  if (text !== undefined) event.text = text;
  return driver.request(
    { type: "simulateGpuiEvent", target: { type: "kind", kind: "main" }, event },
    { expect: "simulateGpuiEventResult", timeoutMs: 5000 },
  );
}

async function typeMainText(driver: Driver, text: string) {
  for (const ch of text) {
    const result = await gpuiMainKey(driver, ch === " " ? "space" : ch, [], ch);
    if (result.success !== true) {
      return { ok: false, result };
    }
    await sleep(30);
  }
  return { ok: true };
}

let driver: Driver | null = null;
let driverClosed = false;

try {
  driver = await Driver.launch({
    binary,
    sandboxHome: true,
    sessionName: "day-notes-editor-host-contract",
    defaultTimeoutMs: 8000,
    env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
  });

  const dayState = await openDayPage(driver, runId);
  check("day_opened", dayState.promptType === "dayPage", {
    promptType: dayState.promptType,
  });

  const dayElements = (await driver.getElements(
    { target: { type: "main" }, limit: 220 },
    { timeoutMs: 5000 },
  )) as Json;
  const dayEditor = walkElements(dayElements).find(
    (el) => el.semanticId === "input:day-page-editor" || el.id === "day-page-editor",
  );
  const dayStyleText = JSON.stringify(dayEditor ?? {});
  check("day_editor_uses_shared_notes_editor", dayStyleText.includes("components.notes_editor"), {
    editor: dayEditor ?? null,
  });
  check("day_has_no_local_overlay_initially", localOverlayIds(dayElements).length === 0, {
    localOverlayIds: localOverlayIds(dayElements),
  });

  const prefix = `Day Notes host contract ${runId} `;
  const seed = (await driver.batch(
    [
      { type: "setInput", text: prefix },
      {
        type: "waitFor",
        condition: { type: "stateMatch", state: { promptType: "dayPage", inputValue: prefix } },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check("day_seeded_prefix", seed.success === true, { seed });

  const typed = await typeMainText(driver, "@con");
  check("day_typed_context_mention", typed.ok === true, { typed });
  const menuState = await waitFor(
    "Day @context handoff to main menu",
    () => driver!.getState({ timeoutMs: 5000 }) as Promise<Json>,
    (state) => state.promptType === "none" && state.inputValue === "@con",
    6000,
  );
  check("day_context_uses_main_menu_round_trip", true, {
    promptType: menuState.promptType,
    inputValue: menuState.inputValue,
  });

  const mainMenuElements = (await driver.getElements(
    { target: { type: "main" }, limit: 260 },
    { timeoutMs: 5000 },
  )) as Json;
  const mainIds = ids(mainMenuElements);
  const hasContextRow = mainIds.some(
    (id) => id.includes("spine:@") || id.includes("what-i-m-looking-at"),
  );
  check("main_menu_context_rows_visible", hasContextRow, {
    sampleIds: mainIds.slice(0, 50),
  });
  check("day_has_no_local_overlay_during_main_menu_round_trip", localOverlayIds(mainMenuElements).length === 0, {
    localOverlayIds: localOverlayIds(mainMenuElements),
  });

  await driver.simulateKey("escape");
  const cancelledDayState = await waitFor(
    "Day context round trip cancels back",
    () => driver!.getState({ timeoutMs: 5000 }) as Promise<Json>,
    (state) => state.promptType === "dayPage",
    6000,
  );
  check("day_context_cancel_returns_to_day", cancelledDayState.promptType === "dayPage", {
    promptType: cancelledDayState.promptType,
    inputValue: cancelledDayState.inputValue,
  });

  const appLog = await Bun.file(driver.logPath).text();
  check("day_context_round_trip_started_logged", appLog.includes("event=day_page_context_round_trip_started"), {
    logPath: driver.logPath,
  });
  check("day_context_round_trip_cancelled_logged", appLog.includes("event=day_page_context_round_trip_cancelled"), {
    logPath: driver.logPath,
  });

  await driver.close();
  driverClosed = true;
  check("driver_closed", true, {});
} catch (error) {
  check("probe_exception", false, {
    message: error instanceof Error ? error.message : String(error),
    stack: error instanceof Error ? error.stack : null,
  });
} finally {
  if (driver && !driverClosed) {
    await driver.close().catch(() => {});
    driverClosed = true;
  }
}

const receipt = {
  schemaVersion: 1,
  source: "runtime.dayNotesEditorHostContract",
  pass: failures.length === 0,
  failures,
  binary,
  sessionDir: driver?.sessionDir ?? null,
  appLog: driver?.logPath ?? null,
  checks,
};

await Bun.write(outPath, `${JSON.stringify(receipt, null, 2)}\n`);
console.log(JSON.stringify(receipt, null, 2));
process.exit(receipt.pass ? 0 : 1);
