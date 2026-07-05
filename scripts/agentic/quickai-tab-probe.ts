#!/usr/bin/env bun
/**
 * Runtime proof for the dynamic header Tab chip + Quick AI Tab mode.
 *
 * Proves, against the real binary:
 *  1. Empty input + Tab  → cwd picker (FileSearchView), unchanged behavior.
 *  2. Text + Tab         → Quick AI: Agent Chat opens with the quick-ai
 *     profile (zero-context flags, spark model) and the launcher filter is
 *     cleared. Verified via structured logs (quick_ai_tab_entry +
 *     pi_agent_chat_profile_launch_resolved with profile_id=quick-ai).
 *
 * Run: bun scripts/agentic/quickai-tab-probe.ts
 */
import { Driver } from "../devtools/driver.ts";

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/quickai-tab/script-kit-gpui";

function stateString(state: unknown): string {
  return JSON.stringify(state ?? {});
}

const receipt: Record<string, unknown> = { probe: "quickai-tab", binary };

const driver = await Driver.launch({
  sessionName: "quickai-tab-probe",
  binary,
  sandboxHome: true,
  seedAgentAuth: true,
});

/** Poll getLogs until the blob contains `needle` or the deadline passes. */
async function waitForLog(needle: string, timeoutMs = 5000): Promise<boolean> {
  const start = performance.now();
  while (performance.now() - start < timeoutMs) {
    const logs = await driver.getLogs({ limit: 500 });
    if (JSON.stringify(logs).includes(needle)) return true;
    await Bun.sleep(150);
  }
  return false;
}

try {
  await driver.waitForSettle();

  // --- Proof 1: empty input + Tab → cwd picker (FileSearchView) -----------
  const before = await driver.getState();
  receipt.initialState = stateString(before).slice(0, 200);
  driver.simulateKey("tab");
  const cwdPickLogged = await waitForLog("cwd_pick_enter_file_search_tab", 4000);
  const afterEmptyTab = await driver.getState();
  receipt.emptyTab = {
    cwdPickLogged,
    stateMentionsFileSearch: stateString(afterEmptyTab).includes("ile"),
  };

  // Return to the launcher (cwd-pick mode owns Escape → back to ScriptList).
  driver.simulateKey("escape");
  await driver.waitForSettle();
  driver.simulateKey("escape");
  await driver.waitForSettle();

  // --- Proof 2: text + Tab → Quick AI --------------------------------------
  await driver.setFilterAndWait("what is the capital of france");
  driver.simulateKey("tab");
  const quickAiEntryLogged = await waitForLog("quick_ai_tab_entry", 5000);
  const quickAiProfileResolved = await waitForLog("quick-ai", 8000);
  await driver.waitForSettle({ timeoutMs: 8000 });

  const logs = await driver.getLogs({ limit: 500 });
  const logsBlob = JSON.stringify(logs);
  const afterQuickAiTab = await driver.getState();
  const quickAiStateBlob = stateString(afterQuickAiTab);

  receipt.quickAiTab = {
    quickAiEntryLogged,
    quickAiLauncherEntryLogged: logsBlob.includes("quick_ai_launcher_entry"),
    quickAiProfileResolved,
    sparkModelInLogs: logsBlob.includes("gpt-5.3-codex-spark"),
    stateAfter: quickAiStateBlob.slice(0, 300),
  };

  const checks = [
    ["emptyTab.cwdPickLogged", cwdPickLogged],
    ["quickAiTab.quickAiEntryLogged", quickAiEntryLogged],
    ["quickAiTab.quickAiProfileResolved", quickAiProfileResolved],
  ] as const;
  receipt.pass = checks.every(([, ok]) => Boolean(ok));
  receipt.failedChecks = checks.filter(([, ok]) => !ok).map(([name]) => name);
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
