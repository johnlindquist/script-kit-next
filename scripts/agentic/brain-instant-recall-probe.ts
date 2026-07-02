#!/usr/bin/env bun
/**
 * Runtime proof: a Day Page capture is recallable from the launcher's Brain
 * search IMMEDIATELY, without waiting for the background indexer cycle.
 *
 * The indexer's first cycle only runs 20s after startup (then every 120s, see
 * src/brain/indexer.rs). So if a just-typed, autosaved marker is recalled well
 * inside that 20s window, the hit can only come from the synchronous
 * capture-time index (index_capture_now, wired via index_capture_after_write in
 * the Day Page save path), never from a cycle.
 *
 * Flow: launch sandboxed -> open Day Page -> type a globally-unique marker ->
 * wait for autosave -> return to the launcher -> `brain:` search the marker ->
 * assert a Brain row appears, all before the first cycle could fire.
 *
 * Modeled on scripts/agentic/brain-prefix-search-cleanup.ts (passive/`brain:`
 * root Brain rows) and the day-page-open-helper gesture flow. Data is created
 * only through the running app, so sandboxHome:true is safe (no pre-seeding).
 */
import { Database } from "bun:sqlite";
import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage, tapMainHotkey } from "./day-page-open-helper";

const repoRoot = resolve(import.meta.dir, "../..");

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

const session = argValue("--session", `brain-instant-recall-${process.pid}`);
const timeoutMs = Number(argValue("--timeout", "12000"));
const pollMs = Number(argValue("--poll", "50"));
// First indexer cycle is 20s after startup; a result before this bound can only
// be from synchronous capture indexing. Kept conservative below 20s.
const preCycleBudgetMs = Number(argValue("--pre-cycle-budget", "18000"));
const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/brain-instant-recall/script-kit-gpui";
const outputDir = join(repoRoot, ".test-output", "brain-instant-recall", session);
const receiptPath = join(outputDir, "receipt.json");

const marker = `zebrafishreceipt${process.pid}`;
const dayBody = `instant recall capture marker ${marker}`;
const runId = `${Date.now().toString(36)}-${process.pid}`;

mkdirSync(outputDir, { recursive: true });

function rowsFrom(state: Json): Json[] {
  return Array.isArray(state?.mainWindowPreflight?.visibleResults)
    ? state.mainWindowPreflight.visibleResults
    : [];
}

function brainRows(rows: Json[]): Json[] {
  return rows.filter(
    (row) => row.role === "rootPassive" && row.sourceName === "From Your Brain",
  );
}

function rowsMentioningMarker(rows: Json[]): Json[] {
  return rows.filter((row) => JSON.stringify(row).includes(marker));
}

const receipt: Json = {
  schemaVersion: 1,
  tool: "brain-instant-recall",
  binary,
  marker,
  preCycleBudgetMs,
  checks: [] as Json[],
};
const failures: string[] = [];
function check(name: string, pass: boolean, detail: Json = {}) {
  receipt.checks.push({ name, pass, detail });
  if (!pass) failures.push(name);
}

const probeStart = performance.now();
const driver = await Driver.launch({
  sessionName: session,
  sandboxHome: true,
  binary,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
  readyTimeoutMs: 15000,
});
receipt.sessionDir = driver.sessionDir;
const brainDbPath = join(driver.sessionDir, "home", ".scriptkit", "db", "brain.sqlite");

try {
  // 1. Open the Day Page and type the marker so the editor autosave writes a
  //    real day file (the capture path we index synchronously).
  const dayState = await openDayPage(driver, runId);
  check("opened_day_page", dayState.promptType === "dayPage", {
    promptType: dayState.promptType,
  });

  const setDay = await driver.batch(
    [
      { type: "setInput", text: dayBody },
      {
        type: "waitFor",
        condition: { type: "stateMatch", state: { promptType: "dayPage", inputValue: dayBody } },
      },
    ],
    { timeoutMs },
  );
  check("typed_marker_into_day_page", (setDay as Json).success === true, { setDay });

  // Autosave debounce writes the day file and runs index_capture_now.
  await Bun.sleep(800);

  // 2. Return to the launcher (tap hides the Day Page, tap opens the launcher).
  await tapMainHotkey(driver, runId, "hide-day-page");
  await driver.waitForState({ windowVisible: false }, { timeoutMs });
  await tapMainHotkey(driver, runId, "show-launcher");
  await driver.waitForState({ windowVisible: true, promptType: "none" }, { timeoutMs });
  await Bun.sleep(400); // let the opening tap's transient double-window settle

  // 3. Search the marker via the Brain prefix IMMEDIATELY and poll for the hit.
  const query = `brain: ${marker}`;
  await driver.setFilterAndWait(query, { timeoutMs });
  let brain: Json[] = [];
  let markerRows: Json[] = [];
  let lastState: Json = {};
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    lastState = await driver.getState({ timeoutMs });
    const rows = rowsFrom(lastState);
    brain = brainRows(rows);
    markerRows = rowsMentioningMarker(rows);
    const refreshing = lastState?.mainWindowPreflight?.rootPassiveFrame?.brain?.refreshing;
    if ((brain.length > 0 || markerRows.length > 0) && !refreshing) break;
    await Bun.sleep(pollMs);
  }

  const msToRecall = Math.round(performance.now() - probeStart);
  receipt.msToRecall = msToRecall;
  receipt.brainRowCount = brain.length;
  receipt.markerRowCount = markerRows.length;
  receipt.brainRowsSample = brain.slice(0, 4);
  receipt.visibleResultCount =
    lastState?.mainWindowPreflight?.visibleResultCount ?? rowsFrom(lastState).length;

  // Supplementary: confirm the doc row exists in the sandbox brain DB (direct
  // proof the synchronous upsert landed). Non-fatal if the DB is momentarily
  // locked by the app.
  try {
    const db = new Database(brainDbPath, { readonly: true });
    const row = db
      .query("SELECT COUNT(*) AS n FROM brain_docs WHERE content LIKE ?")
      .get(`%${marker}%`) as { n: number };
    receipt.brainDocRowsWithMarker = row?.n ?? 0;
    db.close();
    check("brain_doc_row_written_synchronously", (row?.n ?? 0) >= 1, {
      brainDocRowsWithMarker: row?.n ?? 0,
    });
  } catch (error) {
    receipt.brainDbReadError = error instanceof Error ? error.message : String(error);
  }

  check("brain_recall_row_appeared", brain.length > 0 || markerRows.length > 0, {
    brainRowCount: brain.length,
    markerRowCount: markerRows.length,
  });
  check("recall_before_first_indexer_cycle", msToRecall < preCycleBudgetMs, {
    msToRecall,
    preCycleBudgetMs,
  });

  receipt.classification = failures.length === 0 ? "pass" : "fail";
} catch (error) {
  receipt.classification = "fail";
  receipt.error = error instanceof Error ? error.stack ?? error.message : String(error);
  failures.push("threw");
} finally {
  receipt.failures = failures;
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  await driver.close();
  console.log(
    JSON.stringify(
      {
        classification: receipt.classification,
        msToRecall: receipt.msToRecall ?? null,
        brainRowCount: receipt.brainRowCount ?? null,
        markerRowCount: receipt.markerRowCount ?? null,
        brainDocRowsWithMarker: receipt.brainDocRowsWithMarker ?? null,
        failures,
        receiptPath,
      },
      null,
      2,
    ),
  );
  process.exit(failures.length === 0 ? 0 : 1);
}
