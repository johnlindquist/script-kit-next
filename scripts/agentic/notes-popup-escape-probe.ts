#!/usr/bin/env bun
/**
 * scripts/agentic/notes-popup-escape-probe.ts
 *
 * Runtime proof for "escape isn't dismissing the popups" in the Notes window.
 *
 * Drives the real app through the protocol:
 *  1. openNotes
 *  2. batch(target notes) openActions  -> Cmd+K command bar (detached window)
 *  3. simulateKey escape (target notes) -> must close the popup
 *  4. re-open, then escape targeted at the actionsDialog window itself
 *
 * Pass criteria: notesOpened, openedViaBatch, and closedViaNotesEscape true.
 * Known gap: leg 4 (closedViaDialogEscape) stays false because protocol
 * simulateKey targeted at the detached actions-dialog window only routes
 * through the MAIN view's popup handling (simulate_key_dispatch), not the
 * notes-parented ActionsWindow's own on_key_down. Live keyboard input to that
 * window is handled by ActionsWindow::handle_key (escape -> Close). Missing
 * primitive: target-scoped simulateKey routing for detached actions windows
 * with non-main parents.
 *
 * Usage: bun scripts/agentic/notes-popup-escape-probe.ts
 */

import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/notes-popups/script-kit-gpui");

type Obj = Record<string, Json>;

function asObj(v: Json): Obj {
  return v && typeof v === "object" && !Array.isArray(v) ? (v as Obj) : {};
}

async function notesState(driver: Driver): Promise<Obj> {
  const res = await driver.request(
    { type: "getState", target: { type: "kind", kind: "notes" } },
    { expect: "stateResult" },
  );
  return asObj(asObj(res).notes);
}

function popupSnapshot(state: Obj): Obj {
  const commandBars = asObj(state.commandBars);
  return {
    showActionsPanel: asObj(state.view).showActionsPanel ?? null,
    actionsOpen: asObj(commandBars.actions).open ?? null,
    switcherOpen: asObj(commandBars.noteSwitcher).open ?? null,
  };
}

async function actionsWindowIds(driver: Driver): Promise<string[]> {
  const res = asObj(await driver.listAutomationWindows());
  const windows = (res.windows as Json[]) ?? [];
  return windows
    .map(asObj)
    .filter((w) => String(w.kind ?? "").toLowerCase().includes("action"))
    .map((w) => String(w.id));
}

async function pollUntil(
  label: string,
  fn: () => Promise<boolean>,
  timeoutMs = 5000,
): Promise<boolean> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    if (await fn()) return true;
    await Bun.sleep(100);
  }
  console.error(`pollUntil timeout: ${label}`);
  return false;
}

async function main() {
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: "notes-popup-escape",
    sandboxHome: true,
    defaultTimeoutMs: 8000,
  });
  const receipt: Obj = { binary: BINARY };
  try {
    // 1. Open the Notes window.
    driver.send({ type: "openNotes" });
    const notesOpened = await pollUntil("notes window", async () => {
      const res = asObj(await driver.listAutomationWindows());
      const windows = (res.windows as Json[]) ?? [];
      return windows.map(asObj).some((w) => String(w.kind) === "notes");
    });
    receipt.notesOpened = notesOpened;
    await Bun.sleep(400);

    // 2. Open the Cmd+K command bar via target-scoped batch openActions.
    const openBatch = await driver.request(
      {
        type: "batch",
        target: { type: "kind", kind: "notes" },
        commands: [{ type: "openActions" }],
        options: { stopOnError: true, timeout: 8000 },
      },
      { expect: "batchResult" },
    );
    receipt.openBatch = openBatch;
    const opened = await pollUntil("actions popup open", async () => {
      const snap = popupSnapshot(await notesState(driver));
      return snap.actionsOpen === true || snap.showActionsPanel === true;
    });
    receipt.afterOpen = popupSnapshot(await notesState(driver));
    receipt.actionsWindowsAfterOpen = await actionsWindowIds(driver);
    receipt.openedViaBatch = opened;

    // 3. Escape targeted at the Notes window (mirrors the live escape ladder).
    driver.send({
      type: "simulateKey",
      target: { type: "kind", kind: "notes" },
      key: "escape",
      modifiers: [],
    });
    const closedViaNotesEscape = await pollUntil(
      "popup closed after notes escape",
      async () => {
        const snap = popupSnapshot(await notesState(driver));
        const ids = await actionsWindowIds(driver);
        return (
          snap.actionsOpen !== true &&
          snap.showActionsPanel !== true &&
          ids.length === 0
        );
      },
    );
    receipt.afterNotesEscape = popupSnapshot(await notesState(driver));
    receipt.actionsWindowsAfterNotesEscape = await actionsWindowIds(driver);
    receipt.closedViaNotesEscape = closedViaNotesEscape;

    // 4. Re-open, then escape targeted at the actionsDialog window itself
    //    (the route taken when the detached popup is the key window).
    await driver.request(
      {
        type: "batch",
        target: { type: "kind", kind: "notes" },
        commands: [{ type: "openActions" }],
        options: { stopOnError: true, timeout: 8000 },
      },
      { expect: "batchResult" },
    );
    const reopened = await pollUntil("actions popup re-open", async () => {
      const snap = popupSnapshot(await notesState(driver));
      return snap.actionsOpen === true || snap.showActionsPanel === true;
    });
    receipt.reopened = reopened;
    const dialogIds = await actionsWindowIds(driver);
    receipt.dialogIds = dialogIds;
    if (dialogIds.length > 0) {
      driver.send({
        type: "simulateKey",
        target: { type: "id", id: dialogIds[0] },
        key: "escape",
        modifiers: [],
      });
      // Known protocol gap (see file header): expected false today.
      receipt.closedViaDialogEscape = await pollUntil(
        "popup closed after dialog escape",
        async () => {
          const snap = popupSnapshot(await notesState(driver));
          const ids = await actionsWindowIds(driver);
          return (
            snap.actionsOpen !== true &&
            snap.showActionsPanel !== true &&
            ids.length === 0
          );
        },
      );
      receipt.afterDialogEscape = popupSnapshot(await notesState(driver));
    } else {
      receipt.closedViaDialogEscape = "no-dialog-window-found";
    }

    console.log(JSON.stringify(receipt, null, 2));
  } finally {
    await driver.close();
  }
}

main().catch((error) => {
  console.error(error);
  process.exit(1);
});
