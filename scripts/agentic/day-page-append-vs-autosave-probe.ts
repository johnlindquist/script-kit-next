#!/usr/bin/env bun
/**
 * Runtime proof for the day-file write-integrity fix (Plan 01, Step 4).
 *
 * Reproduces the silent-data-loss race: while the Day Page editor has an
 * in-progress edit and its 300ms autosave is pending, a background writer
 * (e.g. ;todo / clipboard sediment) appends a line directly to the bound day
 * file. Before the fix the debounced autosave blindly overwrote the file and
 * dropped the appended line. After the fix save_content merges external appends,
 * so BOTH the typed line and the externally appended marker must survive on disk.
 *
 * Pass: the day file contains both the typed line and the marker line.
 */
import { join } from "node:path";
import { readFileSync, appendFileSync, existsSync, readdirSync } from "node:fs";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/brain-write-integrity/script-kit-gpui";

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function todayLocalDate(): string {
  const now = new Date();
  const y = now.getFullYear();
  const m = String(now.getMonth() + 1).padStart(2, "0");
  const d = String(now.getDate()).padStart(2, "0");
  return `${y}-${m}-${d}`;
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-append-vs-autosave",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

const sandboxHome = join(driver.sessionDir, "home");
const daysDir = join(sandboxHome, ".scriptkit", "brain", "days");
const todayFile = join(daysDir, `${todayLocalDate()}.md`);

const typedLine = `PROBE typed distinctive line ${runId}`;
const editedLine = `${typedLine} EDITED`;
const markerLine = `PROBE-MARKER external append ${runId}`;

async function setDayPageInput(text: string, label: string) {
  const batch = (await driver.batch(
    [
      { type: "setInput", text },
      {
        type: "waitFor",
        condition: {
          type: "stateMatch",
          state: { promptType: "dayPage", inputValue: text },
        },
      },
    ],
    { timeoutMs: 5000 },
  )) as Json;
  check(`batch_set_${label}`, batch.success === true, { batch });
}

try {
  // --- Enter the Day Page through the real hold gesture ---
  const state = await openDayPage(driver, runId);
  check("opened_day_page", state.promptType === "dayPage", {
    promptType: state.promptType,
  });

  // --- Type a distinctive line and let the first (immediate) autosave settle so
  //     the file has a known baseline on disk and the debounce timer is armed ---
  await setDayPageInput(typedLine, "typed_line");
  await Bun.sleep(700);

  // --- Now make a second edit so the editor is dirty with a *pending* debounced
  //     autosave, then WHILE it is pending append a marker line to the bound day
  //     file directly on disk (an external writer the app did not author). This
  //     is the exact race that silently dropped the append before the fix. ---
  await setDayPageInput(editedLine, "edited_line");
  appendFileSync(todayFile, `\n${markerLine}\n`);
  const diskRightAfterAppend = existsSync(todayFile)
    ? readFileSync(todayFile, "utf8")
    : null;
  check("external_append_landed_on_disk", Boolean(diskRightAfterAppend?.includes(markerLine)), {
    diskRightAfterAppend,
  });

  // --- Wait well past the autosave debounce, then read the merged file ---
  await Bun.sleep(1500);
  const diskContent = existsSync(todayFile) ? readFileSync(todayFile, "utf8") : null;
  const hasTyped = Boolean(diskContent?.includes(editedLine));
  const hasMarker = Boolean(diskContent?.includes(markerLine));
  check("typed_line_survives_autosave", hasTyped, { diskContent });
  check("external_append_survives_autosave", hasMarker, { diskContent });
  check("both_lines_present_after_race", hasTyped && hasMarker, {
    todayFile,
    diskContent,
    daysDirListing: existsSync(daysDir) ? readdirSync(daysDir) : null,
  });

  const ok = failures.length === 0;
  console.log(
    JSON.stringify(
      { ok, failures, sessionDir: driver.sessionDir, todayFile, receipts },
      null,
      2,
    ),
  );
  if (!ok) process.exitCode = 1;
} finally {
  await driver.close();
}
