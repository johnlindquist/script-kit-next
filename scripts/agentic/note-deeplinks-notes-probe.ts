#!/usr/bin/env bun
/**
 * Runtime proof for Notes deeplink activation:
 * - Cmd+. with no link preserves the focus-mode fallback
 * - Cmd+. on a run deeplink opens a confirm popup instead of silently executing
 */
import { join, resolve } from "node:path";
import { mkdir } from "node:fs/promises";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/note-deeplinks-iter1/script-kit-gpui");

type Obj = Record<string, any>;

const receipt: Obj = {
  tool: "note-deeplinks-notes-probe",
  binary: BINARY,
  pass: false,
  failures: [] as string[],
};

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value)
    ? (value as Obj)
    : {};
}

function check(name: string, ok: boolean, detail: Obj = {}) {
  receipt[name] = { ok, ...detail };
  if (!ok) receipt.failures.push(name);
}

async function pollUntil(
  label: string,
  fn: () => Promise<boolean>,
  timeoutMs = 6000,
): Promise<boolean> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (await fn()) return true;
    await Bun.sleep(100);
  }
  receipt[`timeout_${label}`] = true;
  return false;
}

async function notesState(driver: Driver): Promise<Obj> {
  const res = await driver.request(
    { type: "getState", target: { type: "kind", kind: "notes" } },
    { expect: "stateResult", timeoutMs: 8000 },
  );
  return asObj(asObj(res).notes);
}

async function notesView(driver: Driver): Promise<Obj> {
  return asObj((await notesState(driver)).view);
}

async function openNotes(driver: Driver) {
  driver.send({ type: "openNotes" });
  return pollUntil("notes-open", async () => {
    const res = asObj(await driver.listAutomationWindows({ timeoutMs: 8000 }));
    const windows = (res.windows as Json[] | undefined) ?? [];
    return windows.map(asObj).some((window) => String(window.kind) === "notes");
  });
}

async function setNotesInput(driver: Driver, text: string) {
  return driver.request(
    {
      type: "batch",
      target: { type: "kind", kind: "notes" },
      commands: [{ type: "setInput", text }],
      options: { stopOnError: true, timeout: 8000 },
    },
    { expect: "batchResult", timeoutMs: 9000 },
  );
}

function legacyTargetedKey(driver: Driver, key: string, modifiers: string[] = []) {
  driver.send({
    type: "simulateKey",
    target: { type: "kind", kind: "notes" },
    key,
    modifiers,
  });
}

async function cmdDot(driver: Driver) {
  legacyTargetedKey(driver, ".", ["cmd"]);
  await Bun.sleep(100);
}

async function escape(driver: Driver) {
  legacyTargetedKey(driver, "escape");
  await Bun.sleep(100);
}

async function confirmWindows(driver: Driver): Promise<Obj[]> {
  const res = asObj(await driver.listAutomationWindows({ timeoutMs: 8000 }));
  const windows = (res.windows as Json[] | undefined) ?? [];
  return windows
    .map(asObj)
    .filter((window) => String(window.semanticSurface) === "confirmDialog");
}

async function closeConfirmWithEscape(driver: Driver, label: string) {
  await escape(driver).catch(() => legacyTargetedKey(driver, "escape"));
  const closed = await pollUntil(`${label}-confirm-closed`, async () => {
    const windows = await confirmWindows(driver);
    return windows.length === 0;
  });
  return {
    closed,
    confirmWindows: await confirmWindows(driver),
  };
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "note-deeplinks-notes",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  const opened = await openNotes(driver);
  check("notes_opened", opened);

  const plain = await setNotesInput(driver, "plain text with no deeplink");
  check("plain_text_seeded", plain.success === true, { batch: plain });
  await cmdDot(driver);
  const focusModeEnabled = await pollUntil("focus-mode-enabled", async () => {
    const view = await notesView(driver);
    return view.focusMode === true;
  });
  check("cmd_dot_without_link_toggles_focus_mode", focusModeEnabled, {
    view: await notesView(driver),
  });

  // Leave focus mode so the run-link leg starts from the normal editor state.
  await cmdDot(driver);
  await pollUntil("focus-mode-disabled", async () => {
    const view = await notesView(driver);
    return view.focusMode === false;
  });

  const runLink = "scriptkit://run/nonexistent-proof-script";
  const runSeed = await setNotesInput(driver, runLink);
  check("run_link_seeded", runSeed.success === true, { batch: runSeed });
  await cmdDot(driver);

  const confirmOpened = await pollUntil("run-confirm-open", async () => {
    const windows = await confirmWindows(driver);
    return windows.length > 0;
  });
  const confirmSnapshot = await confirmWindows(driver);
  check("cmd_dot_on_run_link_opens_confirm", confirmOpened, {
    confirmWindows: confirmSnapshot,
  });
  check("run_link_did_not_toggle_focus_mode", (await notesView(driver)).focusMode !== true, {
    view: await notesView(driver),
  });

  const runClose = await closeConfirmWithEscape(driver, "run");
  check("escape_closes_run_confirm", runClose.closed, {
    confirmWindows: runClose.confirmWindows,
  });

  const missingParent = join(driver.sessionDir, "existing-parent");
  await mkdir(missingParent, { recursive: true });
  const missingFile = join(missingParent, "missing-deeplink-target.txt");
  const missingSeed = await setNotesInput(driver, missingFile);
  check("missing_file_link_seeded", missingSeed.success === true, { batch: missingSeed });
  await cmdDot(driver);
  const missingModalOpened = await pollUntil("missing-file-modal-open", async () => {
    const windows = await confirmWindows(driver);
    return windows.some((window) => String(window.title) === "Can't open this link");
  });
  const missingModalSnapshot = await confirmWindows(driver);
  check("missing_file_opens_helpful_modal", missingModalOpened, {
    confirmWindows: missingModalSnapshot,
  });
  const missingClose = await closeConfirmWithEscape(driver, "missing-file");
  check("escape_closes_missing_file_modal", missingClose.closed, {
    confirmWindows: missingClose.confirmWindows,
    view: await notesView(driver),
  });

  receipt.sessionDir = driver.sessionDir;
  receipt.pass = receipt.failures.length === 0;
  console.log(JSON.stringify(receipt, null, 2));
  if (!receipt.pass) process.exitCode = 1;
} finally {
  await driver.close();
}
