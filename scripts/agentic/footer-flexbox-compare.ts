#!/usr/bin/env bun
/**
 * Red/green visual comparison probe: native AppKit footer glyphs
 * (SCRIPT_KIT_GPUI_FOOTER_OVERLAY=0) vs the default GPUI flexbox footer
 * overlay.
 *
 * Launches the app twice (sequentially — screenshots need the frontmost
 * window), shows the main window, and captures it to PNG for each mode.
 *
 *   bun scripts/agentic/footer-flexbox-compare.ts
 */

import { mkdirSync } from "node:fs";
import { join, resolve } from "node:path";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY = join(
  PROJECT_ROOT,
  "target-agent/pools/agent-debug/debug/script-kit-gpui",
);
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/footer-flexbox-compare");
const WINDOW_TITLE = "Script Kit";

async function captureFooter(mode: "native" | "flexbox"): Promise<string> {
  const env: Record<string, string> = {};
  if (mode === "native") {
    env.SCRIPT_KIT_GPUI_FOOTER_OVERLAY = "0";
  }
  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: `footer-${mode}`,
    sandboxHome: true,
    env,
  });
  const shotPath = join(OUT_DIR, `footer-${mode}.png`);
  try {
    await driver.request({ type: "show" }, { timeoutMs: 5000 }).catch(() => {});
    await driver.waitForState({ windowFocused: true }, { timeoutMs: 5000 }).catch(() => {});
    // Let the footer host + overlay settle one frame cycle.
    await Bun.sleep(500);
    // captureWindow does not echo a requestId response — fire and poll for files.
    const overlayShotPath = join(OUT_DIR, `footer-${mode}-overlay.png`);
    const captures: Array<[string, string]> = [[WINDOW_TITLE, shotPath]];
    if (mode === "flexbox") {
      captures.push(["Script Kit Footer Overlay", overlayShotPath]);
    }
    for (const [title, path] of captures) {
      driver.send({ type: "captureWindow", title, path });
      const deadline = Date.now() + 10_000;
      while (!(await Bun.file(path).exists())) {
        if (Date.now() > deadline) throw new Error(`Screenshot never appeared: ${path} (window '${title}')`);
        await Bun.sleep(100);
      }
    }
    const state = await driver.getState();
    console.log(
      JSON.stringify(
        {
          mode,
          shotPath,
          activeFooter: state.activeFooter ?? state.footer ?? null,
          sessionDir: driver.sessionDir,
        },
        null,
        2,
      ),
    );
  } finally {
    await driver.close();
  }
  return shotPath;
}

mkdirSync(OUT_DIR, { recursive: true });
const nativeShot = await captureFooter("native");
const flexShot = await captureFooter("flexbox");
console.log(JSON.stringify({ done: true, nativeShot, flexShot }));
