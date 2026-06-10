#!/usr/bin/env bun
/**
 * scripts/agentic/spine-at-enter-probe.ts
 *
 * Repro probe for the A3 bug: typing "@fi" then Enter should complete the
 * input to "@file:" (token + file search beneath) and must NEVER clear the
 * prompt-builder input.
 *
 * Usage: bun scripts/agentic/spine-at-enter-probe.ts
 */

import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/input-ambiguity/script-kit-gpui");

function gpuiKey(driver: Driver, key: string, modifiers: string[] = [], text?: string) {
  const event: Json = { type: "keyDown", key, modifiers };
  if (text !== undefined) event.text = text;
  return driver.request(
    { type: "simulateGpuiEvent", target: { type: "kind", kind: "main" }, event },
    { expect: "simulateGpuiEventResult" },
  );
}

async function main() {
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: "spine-at-enter",
    sandboxHome: true,
    defaultTimeoutMs: 8000,
  });
  try {
    // Deliberately NOT shown: hidden-window protocol proof avoids stealing
    // focus from the user (simulateGpuiEvent targets the main window by kind).
    await driver.setFilterAndWait("@fi");
    await Bun.sleep(300);
    const before = await driver.getState();
    const elementsBefore = await driver.getElements();
    const labels: string[] = [];
    const walk = (node: Json) => {
      if (!node || typeof node !== "object") return;
      if (Array.isArray(node)) return node.forEach(walk);
      if (typeof node.label === "string") labels.push(node.label);
      for (const v of Object.values(node)) walk(v as Json);
    };
    walk(elementsBefore);

    // stdin simulateKey reaches try_handle_spine_enter even with the window
    // hidden (the GPUI PressEnter path is gated on visibility).
    driver.simulateKey("enter");
    await Bun.sleep(700);
    const after = await driver.getState();

    console.log(
      JSON.stringify(
        {
          sessionDir: driver.sessionDir,
          before: {
            inputValue: before.inputValue,
            promptType: before.promptType,
            selectedIndex: before.selectedIndex,
            visibleChoiceCount: before.visibleChoiceCount,
            firstLabels: labels.slice(0, 12),
          },
          after: {
            inputValue: after.inputValue,
            promptType: after.promptType,
            selectedIndex: after.selectedIndex,
            visibleChoiceCount: after.visibleChoiceCount,
          },
          pass: after.inputValue === "@file:",
        },
        null,
        2,
      ),
    );
  } finally {
    await driver.close();
  }
}

main();
