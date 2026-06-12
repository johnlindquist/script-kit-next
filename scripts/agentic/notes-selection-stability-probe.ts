#!/usr/bin/env bun
/**
 * Repro probe for main-list selection resets while async root providers refresh.
 *
 * It types "notes", moves down twice, then samples the selected stable key for a
 * short window. The probe fails if the originally selected row is still visible
 * but selection jumps back to the first visible row.
 */

import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1]
    ? process.argv[index + 1]
    : fallback;
}

const binary = resolve(
  PROJECT_ROOT,
  argValue("--binary", process.env.SCRIPT_KIT_GPUI_BINARY ?? "target/debug/script-kit-gpui"),
);
const sampleMs = Number(argValue("--sample-ms", process.env.NOTES_SELECTION_SAMPLE_MS ?? "2500"));
const pollMs = Number(argValue("--poll-ms", "100"));

const sandboxRoot = `/tmp/sk-notes-selection-${process.pid}-${Date.now().toString(36)}`;
const home = join(sandboxRoot, "home");
const scriptsDir = join(home, ".scriptkit", "plugins", "main", "scripts");
rmSync(sandboxRoot, { recursive: true, force: true });
mkdirSync(scriptsDir, { recursive: true });

for (let i = 0; i < 12; i += 1) {
  const id = String(i).padStart(2, "0");
  writeFileSync(
    join(scriptsDir, `notes-selection-${id}.ts`),
    `// Name: Notes Selection ${id}\n// Description: notes selection stability fixture ${id}\n\nexport {};\n`,
  );
}

function preflight(state: Json): Json {
  const frame = state.mainWindowPreflight;
  if (!frame || !Array.isArray(frame.visibleResults)) {
    throw new Error("missing mainWindowPreflight.visibleResults in getState receipt");
  }
  return frame;
}

function frameSummary(state: Json): Json {
  const frame = preflight(state);
  const rows = frame.visibleResults as Json[];
  const selectedKey = frame.selectedResultKey ?? null;
  const selectedIndex = rows.findIndex((row) => row.stableKey === selectedKey);
  return {
    selectedKey,
    selectedIndex,
    firstKey: rows[0]?.stableKey ?? null,
    rowCount: rows.length,
    selectedValue: state.selectedValue ?? null,
    visibleRowFingerprint: frame.visibleRowFingerprint ?? null,
    rows: rows.slice(0, 8).map((row) => ({
      stableKey: row.stableKey ?? null,
      label: row.label ?? row.name ?? row.title ?? null,
      role: row.role ?? null,
    })),
  };
}

const driver = await Driver.launch({
  sessionName: "notes-selection-stability",
  binary,
  env: { HOME: home, SK_PATH: join(home, ".scriptkit") },
  defaultTimeoutMs: 60_000,
});

const receipt: Json = {
  schemaVersion: 1,
  probe: "notes-selection-stability",
  binary,
  sampleMs,
  pollMs,
  sandboxRoot,
};

try {
  const deadline = performance.now() + 60_000;
  let state = await driver.getState({ timeoutMs: 60_000 });
  while (Number(state.visibleChoiceCount ?? 0) < 12 && performance.now() < deadline) {
    await Bun.sleep(250);
    state = await driver.getState();
  }

  await driver.setFilterAndWait("notes", { timeoutMs: 10_000 });
  await Bun.sleep(250);

  driver.simulateKey("down");
  await driver.getState();
  driver.simulateKey("down");
  const afterDown = await driver.getState();
  const baseline = frameSummary(afterDown);
  receipt.baseline = baseline;

  if (!baseline.selectedKey) {
    throw new Error(`missing selected key after two down keys: ${JSON.stringify(baseline)}`);
  }
  if (baseline.selectedKey === baseline.firstKey) {
    throw new Error(`two down keys did not leave the first visible row: ${JSON.stringify(baseline)}`);
  }

  const samples: Json[] = [];
  const end = performance.now() + sampleMs;
  while (performance.now() < end) {
    await Bun.sleep(pollMs);
    const sampleState = await driver.getState();
    const sample = frameSummary(sampleState);
    samples.push(sample);
    const originalStillVisible = sample.rows.some(
      (row: Json) => row.stableKey === baseline.selectedKey,
    );
    if (
      originalStillVisible &&
      sample.selectedKey !== baseline.selectedKey &&
      sample.selectedKey === sample.firstKey
    ) {
      throw new Error(
        `selection reset to first visible row while baseline row remained visible: ${JSON.stringify({
          baseline,
          sample,
        })}`,
      );
    }
  }

  receipt.samples = samples;
  receipt.pass = true;
} catch (error) {
  receipt.pass = false;
  receipt.error = error instanceof Error ? error.message : String(error);
  throw error;
} finally {
  console.log(JSON.stringify(receipt, null, 2));
  await driver.close();
}
