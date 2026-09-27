#!/usr/bin/env bun
/**
 * Runtime proof for ScriptList Escape behavior:
 * - visible input text clears on the first Escape
 * - empty visible input closes on the next Escape
 * - filterInputDiagnostics exposes canonical/computed/raw visual state
 *
 * Usage:
 *   PROBE_BINARY=target-agent/artifacts/main-escape-visible-input/script-kit-gpui \
 *     bun scripts/agentic/main-escape-visible-input-probe.ts
 */
import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ??
  "target-agent/artifacts/main-escape-visible-input/script-kit-gpui";
const OUT_PATH = process.env.PROBE_OUT ?? "";
const READY_TIMEOUT_MS = Number(process.env.PROBE_READY_TIMEOUT_MS ?? 30_000);

const receipts: Record<string, Json> = {};
const failures: string[] = [];

function check(name: string, ok: boolean, detail: Json) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

function diagnostics(state: Json): Json {
  return (state.filterInputDiagnostics ?? {}) as Json;
}

let driver: Driver | null = null;

try {
  driver = await Driver.launch({
    binary: BINARY,
    sandboxHome: true,
    sessionName: "main-escape-visible-input-probe",
    readyTimeoutMs: READY_TIMEOUT_MS,
    env: {
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_STARTUP_PROFILE: "dev-fast",
      SCRIPT_KIT_DISABLE_AGENT_CHAT_HOT_PREWARM: "1",
      SCRIPT_KIT_DISABLE_AUTOMATIC_UPDATE_CHECK: "1",
    },
  });

  driver.send({ type: "show", requestId: "main-escape-show" });
  await driver.waitForState(
    { windowVisible: true, promptType: "none" },
    { timeoutMs: 8000 },
  );

  const input = `escape-probe-${Date.now()}`;
  await driver.setFilterAndWait(input, { timeoutMs: 8000 });
  const before = await driver.getState({ timeoutMs: 5000 });
  const beforeDiag = diagnostics(before);
  check("diagnostics_match_visible_input", before.inputValue === input
    && beforeDiag.canonicalFilterText === input
    && beforeDiag.computedFilterText === input
    && beforeDiag.rawVisualInputValue === input
    && beforeDiag.pendingFilterSync === false, {
    inputValue: before.inputValue,
    diagnostics: beforeDiag,
  });

  driver.simulateKey("escape");
  await driver.waitForState(
    { windowVisible: true, promptType: "none", inputValue: "" },
    { timeoutMs: 8000, pollIntervalMs: 10 },
  );
  const afterFirst = await driver.getState({ timeoutMs: 5000 });
  const afterFirstDiag = diagnostics(afterFirst);
  check("first_escape_clears_visible_input", afterFirst.inputValue === ""
    && afterFirst.windowVisible === true
    && afterFirstDiag.canonicalFilterText === ""
    && afterFirstDiag.computedFilterText === ""
    && afterFirstDiag.rawVisualInputValue === "", {
    inputValue: afterFirst.inputValue,
    windowVisible: afterFirst.windowVisible,
    diagnostics: afterFirstDiag,
  });

  driver.simulateKey("escape");
  await driver.waitForState(
    { windowVisible: false },
    { timeoutMs: 8000, pollIntervalMs: 10 },
  );
  const afterSecond = await driver.getState({ timeoutMs: 5000 });
  check("second_escape_closes_empty_main", afterSecond.windowVisible === false, {
    windowVisible: afterSecond.windowVisible,
    inputValue: afterSecond.inputValue,
    diagnostics: diagnostics(afterSecond),
  });

  const output = {
    ok: failures.length === 0,
    failures,
    binary: BINARY,
    sessionDir: driver.sessionDir,
    appLog: driver.logPath,
    receipts,
  };
  if (OUT_PATH) {
    const outDir = dirname(OUT_PATH);
    if (!existsSync(outDir)) mkdirSync(outDir, { recursive: true });
    writeFileSync(OUT_PATH, `${JSON.stringify(output, null, 2)}\n`);
  }
  console.log(JSON.stringify(output, null, 2));
  await driver.close();
  process.exit(failures.length === 0 ? 0 : 1);
} catch (error) {
  console.error(error);
  if (driver) await driver.close().catch(() => {});
  process.exit(1);
}
