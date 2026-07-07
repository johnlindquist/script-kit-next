#!/usr/bin/env bun
/**
 * Runtime proof for the Shift+Tab Quick AI profile picker + markdown profiles.
 *
 * Proves, against the real binary:
 *  1. ScriptList + Shift+Tab → Profile Search opens (profile_switcher_open_shift_tab).
 *  2. A markdown profile dropped in <kit>/profiles/*.md shows up as a
 *     Profile Search row (mdflow profile loading).
 *  3. Tab on a Profile Search row assigns it to Quick AI
 *     (profile_search_quick_ai_profile_persisted) and returns to the launcher.
 *  4. Text + Tab then launches Quick AI with the picked profile
 *     (pi launch resolves the picked profile id, not quick-ai).
 *
 * Run: bun scripts/agentic/quickai-profile-pick-probe.ts
 */
import { Driver } from "../devtools/driver.ts";

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/quickai-tab/script-kit-gpui";

const receipt: Record<string, unknown> = { probe: "quickai-profile-pick", binary };

const driver = await Driver.launch({
  sessionName: "quickai-profile-pick-probe",
  binary,
  sandboxHome: true,
  seedAgentAuth: true,
});

async function waitForLog(needle: string, timeoutMs = 5000): Promise<boolean> {
  const start = performance.now();
  while (performance.now() - start < timeoutMs) {
    const logs = await driver.getLogs({ limit: 500 });
    if (JSON.stringify(logs).includes(needle)) return true;
    await Bun.sleep(150);
  }
  return false;
}

async function elementsBlob(): Promise<string> {
  const [elements, layout] = await Promise.all([
    driver.getElements().catch(() => ({})),
    driver.getLayoutInfo().catch(() => ({})),
  ]);
  return JSON.stringify({ elements, layout });
}

try {
  await driver.waitForSettle();

  // Seed a markdown profile inside the sandbox home before opening the picker.
  const profilesDir = `${driver.sessionDir}/home/.scriptkit/profiles`;
  await Bun.write(
    `${profilesDir}/probe-flash.md`,
    [
      "---",
      "name: Probe Flash",
      "model: openai-codex/gpt-5.4",
      "tools: web_search",
      "no-session: true",
      "---",
      "",
      "You are the probe profile. Answer in one word.",
      "",
    ].join("\n"),
  );
  receipt.seededProfile = `${profilesDir}/probe-flash.md`;

  // --- Proof 1: Shift+Tab opens Profile Search ----------------------------
  driver.simulateKey("tab", ["shift"]);
  const openedLogged = await waitForLog("profile_switcher_open_shift_tab", 4000);
  await driver.waitForSettle();
  const pickerBlob = await elementsBlob();
  receipt.shiftTab = {
    openedLogged,
    quickAiRowVisible: pickerBlob.includes("profile-search-row:quick-ai"),
    mdflowRowVisible: pickerBlob.includes("profile-search-row:probe-flash"),
    createRowVisible: pickerBlob.includes("profile-search-row:create-new-profile"),
  };

  // --- Proof 2: Tab on the markdown profile row assigns Quick AI ----------
  await driver.setFilterAndWait("probe flash");
  await driver.waitForSettle();
  driver.simulateKey("tab");
  const persistedLogged = await waitForLog(
    "profile_search_quick_ai_profile_persisted",
    4000,
  );
  await driver.waitForSettle();
  receipt.tabAssign = { persistedLogged };

  // --- Proof 3: text + Tab launches Quick AI with the picked profile ------
  await driver.setFilterAndWait("ping");
  driver.simulateKey("tab");
  const quickAiEntryLogged = await waitForLog("quick_ai_tab_entry", 5000);
  // "probe-flash" already appears in logs from the persist event above, so
  // require the launch-resolution line specifically: a single log entry that
  // mentions both the launch resolve and the picked profile id.
  let pickedProfileResolved = false;
  {
    const start = performance.now();
    while (performance.now() - start < 8000 && !pickedProfileResolved) {
      const logs = await driver.getLogs({ limit: 500 });
      const lines: string[] = Array.isArray(logs)
        ? (logs as unknown[]).map((entry) => JSON.stringify(entry))
        : JSON.stringify(logs).split("\\n");
      pickedProfileResolved = lines.some(
        (line) => line.includes("launch_resolved") && line.includes("probe-flash"),
      );
      if (!pickedProfileResolved) await Bun.sleep(150);
    }
  }
  await driver.waitForSettle({ timeoutMs: 8000 });
  receipt.quickAiLaunch = { quickAiEntryLogged, pickedProfileResolved };

  const checks = [
    ["shiftTab.openedLogged", openedLogged],
    ["shiftTab.quickAiRowVisible", receipt.shiftTab && (receipt.shiftTab as any).quickAiRowVisible],
    ["shiftTab.mdflowRowVisible", receipt.shiftTab && (receipt.shiftTab as any).mdflowRowVisible],
    ["tabAssign.persistedLogged", persistedLogged],
    ["quickAiLaunch.quickAiEntryLogged", quickAiEntryLogged],
    ["quickAiLaunch.pickedProfileResolved", pickedProfileResolved],
  ] as const;
  receipt.pass = checks.every(([, ok]) => Boolean(ok));
  receipt.failedChecks = checks.filter(([, ok]) => !ok).map(([name]) => name);
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
