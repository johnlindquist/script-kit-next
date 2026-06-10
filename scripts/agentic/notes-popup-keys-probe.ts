/**
 * Runtime proof for Notes command-bar popups (Cmd+P note switcher, Cmd+K actions).
 *
 * Drives the REAL key path (osascript System Events keystrokes), because the
 * reported bug lives in global keystroke interceptors that protocol
 * simulateKey does not exercise: when the detached actions popup became the
 * key window, the main app's actions interceptor swallowed every keystroke.
 *
 * Proves: typing filters the list, the popup window shrinks to fit, arrows
 * move the selection, Enter executes (switches note), Escape closes, and the
 * Cmd+K actions panel filters + shrinks the same way.
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
  sessionName: "notes-popup-keys-probe",
  // Pre-existing main-window collection-behavior mismatch in this branch
  // aborts debug builds; the popup proof doesn't depend on it.
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

/** Redacted ActionsDialog automation state (search fingerprint, selection). */
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

/** Bounds of the detached actions popup window, or null when closed. */
async function popupBounds(): Promise<Json | null> {
  const result = (await driver.request(
    { type: "listAutomationWindows" },
    { timeoutMs: 3000 },
  )) as Json;
  const windows: Json[] = result.windows ?? [];
  return windows.find((w) => w.id === "actions-dialog")?.bounds ?? null;
}

try {
  // 1. Open the Notes window and make the app frontmost for native keys.
  driver.send({ type: "openNotes", requestId: "probe-open-notes" });
  await Bun.sleep(2000);
  await osa(
    `tell application "System Events" to set frontmost of (first process whose unix id is ${driver.pid}) to true`,
  );
  await Bun.sleep(600);

  // 2. Seed three notes through the real editor (keystrokes + Cmd+N).
  await keystroke("alpha first note");
  await Bun.sleep(300);
  await keystroke("n", ["command"]);
  await Bun.sleep(500);
  await keystroke("beta second note");
  await Bun.sleep(300);
  await keystroke("n", ["command"]);
  await Bun.sleep(500);
  await keystroke("gamma third note");
  await Bun.sleep(600);

  // 3. Cmd+P opens the note switcher.
  await keystroke("p", ["command"]);
  await Bun.sleep(900);
  const openBounds = await popupBounds();
  const openState = await dialogState();
  check("switcher_opened", openBounds !== null && openState !== null, {
    openBounds,
    searchTextLength: openState?.search?.textLength,
  });

  // 4. Typing filters the list and the popup window shrinks.
  await keystroke("gamma");
  await Bun.sleep(900);
  const filteredState = await dialogState();
  const filteredBounds = await popupBounds();
  check(
    "switcher_typing_filters",
    filteredState?.search?.textLength === 5,
    { searchTextLength: filteredState?.search?.textLength },
  );
  check(
    "switcher_window_shrinks",
    typeof openBounds?.height === "number" &&
      typeof filteredBounds?.height === "number" &&
      filteredBounds.height < openBounds.height,
    { heightOpen: openBounds?.height, heightFiltered: filteredBounds?.height },
  );

  // 5. Backspace restores the full list (window grows back).
  for (let i = 0; i < 5; i += 1) await keyCode(51);
  await Bun.sleep(900);
  const restoredState = await dialogState();
  const restoredBounds = await popupBounds();
  check(
    "switcher_backspace_restores",
    restoredState?.search?.textLength === 0 &&
      typeof restoredBounds?.height === "number" &&
      restoredBounds.height > (filteredBounds?.height ?? Infinity),
    {
      searchTextLength: restoredState?.search?.textLength,
      heightRestored: restoredBounds?.height,
    },
  );

  // 6. Arrow down moves the selection.
  const beforeArrow = restoredState?.selection?.groupedIndex;
  await keyCode(125); // down
  await Bun.sleep(500);
  const afterArrowState = await dialogState();
  check(
    "switcher_arrow_moves_selection",
    typeof beforeArrow === "number" &&
      afterArrowState?.selection?.groupedIndex !== beforeArrow,
    { beforeArrow, afterArrow: afterArrowState?.selection?.groupedIndex },
  );

  // 7. Enter executes the selected note switch: popup closes.
  await keyCode(36); // return
  await Bun.sleep(900);
  check("switcher_enter_executes_and_closes", (await popupBounds()) === null, {});

  // 8. Cmd+K actions panel: open, type to filter + shrink, escape closes.
  await keystroke("k", ["command"]);
  await Bun.sleep(900);
  const actionsOpenBounds = await popupBounds();
  const actionsOpenState = await dialogState();
  check(
    "actions_opened",
    actionsOpenBounds !== null && actionsOpenState !== null,
    { bounds: actionsOpenBounds },
  );

  await keystroke("find");
  await Bun.sleep(900);
  const actionsFilteredState = await dialogState();
  const actionsFilteredBounds = await popupBounds();
  check(
    "actions_typing_filters_and_shrinks",
    actionsFilteredState?.search?.textLength === 4 &&
      typeof actionsOpenBounds?.height === "number" &&
      typeof actionsFilteredBounds?.height === "number" &&
      actionsFilteredBounds.height < actionsOpenBounds.height,
    {
      searchTextLength: actionsFilteredState?.search?.textLength,
      heightOpen: actionsOpenBounds?.height,
      heightFiltered: actionsFilteredBounds?.height,
    },
  );

  await keyCode(53); // escape
  await Bun.sleep(700);
  check("actions_escape_closes", (await popupBounds()) === null, {});
} finally {
  try {
    await keyCode(53);
  } catch {}
  await driver.close();
}

console.log(JSON.stringify({ ok: failures.length === 0, failures, receipts }, null, 2));
process.exit(failures.length === 0 ? 0 : 1);
