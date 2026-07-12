#!/usr/bin/env bun
/**
 * Runtime proof that the REAL GPUI Tab interceptor (startup.rs) routes
 * text + Tab to Quick AI — not the flow router.
 *
 * Regression context (2026-07-10): the Conversation Desk pivot (220941a92)
 * replaced the Quick AI Tab entry with `route_text_to_flow`, so typing a
 * message and hitting Tab launched a flow. This probe dispatches Tab via
 * `simulateGpuiEvent` (real GPUI event pipeline — the same path a user's
 * keystroke takes), unlike quickai-tab-probe.ts which exercises the legacy
 * simulateKey protocol mirror. Run both for full lockstep proof.
 *
 * Receipts:
 *  - quick_ai_tab_entry + quick_ai_launcher_entry in structured logs
 *  - NO flow_router_tab_entry / flow_router_auto_start in logs
 *  - post-Tab state is Agent Chat, not a flow session/desk
 *
 * Run: bun scripts/agentic/quickai-tab-interceptor-probe.ts
 */
import { Driver } from "../devtools/driver.ts";

const binary =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  "target-agent/artifacts/quickai-tab/script-kit-gpui";

const receipt: Record<string, unknown> = {
  probe: "quickai-tab-interceptor",
  binary,
};

const driver = await Driver.launch({
  sessionName: "quickai-tab-interceptor-probe",
  binary,
  sandboxHome: true,
  seedAgentAuth: true,
});

async function waitForLog(needle: string, timeoutMs = 6000): Promise<boolean> {
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

  // Real GPUI dispatch goes through window.dispatch_keystroke, which needs
  // the main window visible — hidden windows only schedule the event.
  driver.send({ type: "show" });
  await driver.waitForState({ windowVisible: true });

  await driver.setFilterAndWait("what is the capital of france");

  const dispatch = await driver.simulateGpuiEvent(
    { type: "keyDown", key: "tab" },
    { target: { type: "kind", kind: "main" } },
  );
  receipt.dispatch = dispatch;

  const quickAiEntryLogged = await waitForLog("quick_ai_tab_entry");
  const quickAiLauncherEntryLogged = await waitForLog(
    "quick_ai_launcher_entry",
  );
  await driver.waitForSettle({ timeoutMs: 8000 });

  const logs = await driver.getLogs({ limit: 500 });
  const logsBlob = JSON.stringify(logs);
  const state = await driver.getState();
  const stateBlob = JSON.stringify(state ?? {});

  const flowRouterFired =
    logsBlob.includes("flow_router_tab_entry") ||
    logsBlob.includes("flow_router_auto_start");
  const flowSessionOpened =
    stateBlob.includes("flowSession") || stateBlob.includes("FlowSession");
  const agentChatOpen =
    stateBlob.toLowerCase().includes("agentchat") ||
    logsBlob.includes("quick_ai_launcher_entry");

  receipt.checks = {
    quickAiEntryLogged,
    quickAiLauncherEntryLogged,
    flowRouterFired,
    flowSessionOpened,
    agentChatOpen,
  };
  receipt.stateAfter = stateBlob.slice(0, 300);

  const pass =
    quickAiEntryLogged &&
    quickAiLauncherEntryLogged &&
    !flowRouterFired &&
    !flowSessionOpened &&
    agentChatOpen;
  receipt.pass = pass;
} finally {
  await driver.close();
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
