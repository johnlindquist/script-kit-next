#!/usr/bin/env bun
/**
 * scripts/agentic/style-dot-probe.ts
 *
 * Runtime proof for the A9 decision (2026-06-09): typing `.` then picking a
 * style is a single-keystroke "rewrite selected text" flow — accepting a
 * style row when the style segment is the whole input auto-submits the spine
 * prompt plan (style sugar expands it to `@selection` + `/rewrite` + style
 * profile) and lands in Agent Chat.
 *
 * Usage: bun scripts/agentic/style-dot-probe.ts
 */

import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/input-ambiguity/script-kit-gpui");

function collectStrings(node: Json, out: string[]) {
  if (!node || typeof node !== "object") return;
  if (Array.isArray(node)) {
    for (const child of node) collectStrings(child, out);
    return;
  }
  const record = node as Record<string, Json>;
  for (const key of ["semanticId", "text", "value"]) {
    if (typeof record[key] === "string") out.push(record[key] as string);
  }
  for (const value of Object.values(record)) collectStrings(value, out);
}

async function main() {
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: "style-dot",
    sandboxHome: true,
    defaultTimeoutMs: 8000,
  });
  try {
    // Step 1: "." lists the style catalog in the spine projection.
    await driver.setFilterAndWait(".");
    await Bun.sleep(400);
    const dotLabels: string[] = [];
    collectStrings(await driver.getElements(), dotLabels);
    const styleTitles = ["Professional", "Concise", "Friendly", "Direct"];
    const listedStyles = styleTitles.filter((t) =>
      dotLabels.some((l) => l.includes(t)),
    );

    // Step 2: ".con" narrows to Concise; Enter accepts the row, which must
    // auto-submit the rewrite-selection plan to Agent Chat.
    await driver.setFilterAndWait(".con");
    await Bun.sleep(400);
    driver.simulateKey("enter");
    await Bun.sleep(1200);
    const after = await driver.getState();

    const log = await Bun.file(driver.logPath).text();
    const autoSubmitLogged = log.includes("spine_style_only_auto_submit");
    const planSubmitted = log.includes("spine_prompt_plan_submit");

    console.log(
      JSON.stringify(
        {
          sessionDir: driver.sessionDir,
          step1: {
            listedStyles,
            pass: listedStyles.length === styleTitles.length,
          },
          step2: {
            inputValue: after.inputValue,
            promptType: after.promptType,
            autoSubmitLogged,
            planSubmitted,
            pass: autoSubmitLogged && planSubmitted,
          },
          pass:
            listedStyles.length === styleTitles.length &&
            autoSubmitLogged &&
            planSubmitted,
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
