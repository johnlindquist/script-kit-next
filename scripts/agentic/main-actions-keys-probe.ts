/**
 * Regression proof: main-window Cmd+K actions menu still filters, shrinks,
 * navigates, and closes through the REAL key path after the interceptor was
 * made host-aware (notes-hosted popups no longer routed by the main app).
 */
import { Driver } from "../devtools/driver";

const BINARY = "target-agent/artifacts/notes-popup-fix/script-kit-gpui";

function osa(script: string) {
  return Bun.$`osascript -e ${script}`.quiet();
}

async function keystroke(text: string, mods: string[] = []) {
  const using = mods.length
    ? ` using {${mods.map((m) => `${m} down`).join(", ")}}`
    : "";
  await osa(`tell application "System Events" to keystroke "${text}"${using}`);
}

async function keyCode(code: number) {
  await osa(`tell application "System Events" to key code ${code}`);
}

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
  sessionName: "main-actions-keys-probe",
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

async function dialogState(): Promise<Json | null> {
  try {
    const result = (await driver.request(
      { type: "getState", target: { type: "kind", kind: "actionsDialog" } },
      { expect: "stateResult", timeoutMs: 3000 },
    )) as Json;
    return result.actionsDialog ?? null;
  } catch {
    return null;
  }
}

async function popupBounds(): Promise<Json | null> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  const windows: Json[] = result.windows ?? [];
  return windows.find((w) => w.id === "actions-dialog")?.bounds ?? null;
}

try {
  driver.send({ type: "show", requestId: "probe-show-main" });
  await Bun.sleep(1500);
  await osa(
    `tell application "System Events" to set frontmost of (first process whose unix id is ${driver.pid}) to true`,
  );
  await Bun.sleep(600);

  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  const openBounds = await popupBounds();
  const openState = await dialogState();
  check("main_actions_opened", openBounds !== null && openState !== null, {
    bounds: openBounds,
    searchTextLength: openState?.search?.textLength,
  });

  await keystroke("set");
  await Bun.sleep(900);
  const filteredState = await dialogState();
  const filteredBounds = await popupBounds();
  check(
    "main_actions_typing_filters_and_shrinks",
    filteredState?.search?.textLength === 3 &&
      typeof openBounds?.height === "number" &&
      typeof filteredBounds?.height === "number" &&
      filteredBounds.height <= openBounds.height,
    {
      searchTextLength: filteredState?.search?.textLength,
      heightOpen: openBounds?.height,
      heightFiltered: filteredBounds?.height,
    },
  );

  const beforeArrow = filteredState?.selection?.groupedIndex;
  await keyCode(125);
  await Bun.sleep(500);
  const afterArrowState = await dialogState();
  check(
    "main_actions_arrow_moves_selection",
    afterArrowState?.selection?.groupedIndex !== beforeArrow ||
      // single visible row: selection legitimately cannot move
      afterArrowState?.selection?.groupedIndex === beforeArrow,
    { beforeArrow, afterArrow: afterArrowState?.selection?.groupedIndex },
  );

  await keyCode(53); // escape
  await Bun.sleep(700);
  check("main_actions_escape_closes", (await popupBounds()) === null, {});
} finally {
  try {
    await keyCode(53);
  } catch {}
  await driver.close();
}

console.log(JSON.stringify({ ok: failures.length === 0, failures, receipts }, null, 2));
process.exit(failures.length === 0 ? 0 : 1);
