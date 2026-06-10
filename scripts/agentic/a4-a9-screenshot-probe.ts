#!/usr/bin/env bun
/**
 * scripts/agentic/a4-a9-screenshot-probe.ts
 *
 * Visual receipts for the A3/A4/A9 input flows: captures main-window
 * screenshots of the `@file:` subsearch, the `;` capture target picker, the
 * `todo;` capture form, and the `.` style list.
 *
 * Usage: bun scripts/agentic/a4-a9-screenshot-probe.ts
 */

import { join, resolve } from "node:path";
import { mkdirSync } from "node:fs";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/input-ambiguity/script-kit-gpui");
const OUT_DIR = join(PROJECT_ROOT, ".test-output/input-ambiguity-visuals", String(process.pid));

async function main() {
  mkdirSync(OUT_DIR, { recursive: true });
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: "a4-a9-shots",
    sandboxHome: true,
    defaultTimeoutMs: 8000,
  });
  const shots: Record<string, string> = {};
  try {
    const capture = async (name: string) => {
      const path = join(OUT_DIR, `${name}.png`);
      const result = (await driver.captureScreenshot({
        target: { type: "kind", kind: "main" },
        savePath: path,
      })) as { error?: string };
      shots[name] = result.error ? `ERROR: ${result.error}` : path;
    };

    // Deterministic OS-window capture requires the window on screen.
    await driver.request({ type: "show" }, { timeoutMs: 1500 }).catch(() => {});
    await Bun.sleep(500);

    await driver.setFilterAndWait("@file:");
    await Bun.sleep(800);
    await capture("a3-file-subsearch-recents");

    await driver.setFilterAndWait(";");
    await Bun.sleep(500);
    await capture("a4-semicolon-target-picker");

    await driver.setFilterAndWait("todo; Renew passport #errands");
    await Bun.sleep(600);
    await capture("a4-todo-postfix-capture-form");

    await driver.setFilterAndWait(".");
    await Bun.sleep(500);
    await capture("a9-dot-style-list");

    console.log(JSON.stringify({ sessionDir: driver.sessionDir, shots }, null, 2));
  } finally {
    await driver.close();
  }
}

main();
