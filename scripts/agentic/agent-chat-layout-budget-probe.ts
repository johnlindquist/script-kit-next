#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const receiptPath = resolve(
  process.env.PROBE_RECEIPT ?? ".test-output/agent-chat-layout-budget-probe.json",
);
const binary = process.env.SCRIPT_KIT_GPUI_BINARY;

function transcriptBounds(layout: Json): Json | null {
  return (
    (layout.components ?? []).find((component: Json) => component.name === "AgentChatTranscript")
      ?.bounds ?? null
  );
}

async function sample(driver: Driver, itemIx: number): Promise<Json> {
  const scroll = await driver.request(
    { type: "setAgentChatTranscriptScroll", itemIx, offsetPx: 0 },
    { expect: "externalCommandResult", timeoutMs: 5_000 },
  );
  await Bun.sleep(125);
  const stateResult = await driver.request(
    { type: "getAgentChatState" },
    { expect: "agent_chatStateResult", timeoutMs: 10_000 },
  );
  const state = stateResult.state ?? stateResult;
  const layout = await driver.getLayoutInfo({}, { timeoutMs: 10_000 });
  const bounds = transcriptBounds(layout);
  const listStateHeight = Number(state.transcriptScroll?.viewportHeightPx ?? 0);
  const layoutHeight = Number(bounds?.height ?? 0);
  return {
    itemIx,
    scrollOk: scroll.ok !== false && scroll.success !== false,
    listStateHeight,
    layoutHeight,
    heightDelta: Math.abs(listStateHeight - layoutHeight),
    bounds,
  };
}

mkdirSync(dirname(receiptPath), { recursive: true });
const receipt: Json = {
  schemaVersion: 1,
  tool: "agent-chat-layout-budget-probe",
  binary: binary ?? "freshest local agent/dev binary",
  pass: false,
  failures: [],
};

const driver = await Driver.launch({
  ...(binary ? { binary } : {}),
  sandboxHome: true,
  sessionName: "agent-chat-layout-budget",
  readyTimeoutMs: 30_000,
  defaultTimeoutMs: 10_000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  const opened = await driver.request(
    { type: "openAgentChatKitchenSinkFixture" },
    { expect: "externalCommandResult", timeoutMs: 10_000 },
  );
  receipt.opened = opened;
  await Bun.sleep(750);

  const collapsedSamples = [];
  for (const itemIx of [0, 10, 19]) {
    collapsedSamples.push(await sample(driver, itemIx));
  }
  receipt.collapsedSamples = collapsedSamples;

  for (const sampleReceipt of collapsedSamples) {
    if (!sampleReceipt.scrollOk) {
      receipt.failures.push({ name: "scroll_failed", itemIx: sampleReceipt.itemIx });
    }
    if (sampleReceipt.listStateHeight < 360) {
      receipt.failures.push({
        name: "transcript_viewport_starved",
        itemIx: sampleReceipt.itemIx,
        viewportHeightPx: sampleReceipt.listStateHeight,
      });
    }
    if (sampleReceipt.heightDelta > 2) {
      receipt.failures.push({
        name: "layout_list_state_viewport_mismatch",
        itemIx: sampleReceipt.itemIx,
        heightDelta: sampleReceipt.heightDelta,
      });
    }
  }

  // Use the real GPUI dispatch path so the focused Agent Chat entity owns the
  // shortcut instead of the protocol simulateKey fallback.
  receipt.expandKey = await driver.simulateGpuiEvent(
    { type: "keyDown", key: "e", modifiers: ["cmd", "shift"] },
    { timeoutMs: 10_000 },
  );
  await Bun.sleep(300);
  const expandedSample = await sample(driver, 19);
  receipt.expandedSample = expandedSample;
  const collapsedHeight = Number(collapsedSamples.at(-1)?.listStateHeight ?? 0);
  if (receipt.expandKey?.success !== true) {
    receipt.failures.push({
      name: "expanded_composer_key_dispatch_failed",
      action: receipt.expandKey,
    });
  }
  if (expandedSample.heightDelta > 2) {
    receipt.failures.push({
      name: "expanded_layout_list_state_viewport_mismatch",
      heightDelta: expandedSample.heightDelta,
    });
  }
  if (!(expandedSample.listStateHeight < collapsedHeight - 100)) {
    receipt.failures.push({
      name: "expanded_composer_did_not_consume_requested_height",
      collapsedHeight,
      expandedHeight: expandedSample.listStateHeight,
    });
  }

  receipt.pass = receipt.failures.length === 0;
} catch (error) {
  receipt.failures.push({ name: "probe_error", error: String(error) });
} finally {
  await driver.close();
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
}

if (!receipt.pass) {
  console.error(JSON.stringify(receipt, null, 2));
  process.exit(1);
}

console.log(JSON.stringify(receipt, null, 2));
