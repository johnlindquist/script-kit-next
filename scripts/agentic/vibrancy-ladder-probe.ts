/**
 * Probe: hot-swap opacity.vibrancy_background via the sandbox theme.json
 * watcher and screencapture each step, to compare window-root tint levels
 * over the current backdrop.
 *
 * Env:
 *   VIB_LADDER  comma-separated alphas (default "0.85,0.75,0.65,0.5")
 *   VIB_PREFIX  capture filename prefix (default "vib-ladder")
 *
 * Each step receipt includes the on-screen Script Kit window list
 * (CGWindowList) so stacked-instance artifacts can be ruled out.
 */
import { join } from "node:path";
import { readFileSync, writeFileSync } from "node:fs";
import { Driver } from "../devtools/driver.ts";

const LADDER = (process.env.VIB_LADDER ?? "0.85,0.75,0.65,0.5")
  .split(",")
  .map((s) => Number(s.trim()))
  .filter((n) => Number.isFinite(n));
const PREFIX = process.env.VIB_PREFIX ?? "vib-ladder";
const TEMPLATE = JSON.parse(
  readFileSync("tests/theme/snapshots/theme_dark_default.json", "utf8"),
);

const driver = await Driver.launch({
  binary: "target-agent/artifacts/vibrancy-085/script-kit-gpui",
  sandboxHome: true,
  sessionName: `vib-ladder-${PREFIX}`,
});

const themePath = join(driver.sessionDir, "home", ".scriptkit", "theme.json");

async function capture(tag: string) {
  const proc = Bun.spawn([
    "screencapture",
    "-x",
    `/tmp/${PREFIX}-${tag}-d1.png`,
    `/tmp/${PREFIX}-${tag}-d2.png`,
    `/tmp/${PREFIX}-${tag}-d3.png`,
  ]);
  await proc.exited;
}

function onScreenScriptKitWindows(): unknown {
  // /tmp/list-sk-windows: tiny C tool over CGWindowListCopyWindowInfo that
  // prints on-screen windows owned by *script* processes. Fails loudly —
  // a missing/erroring tool must not read as "no other windows".
  const wins = Bun.spawnSync(["/tmp/list-sk-windows"]);
  if (wins.exitCode !== 0) {
    return { error: `list-sk-windows exit ${wins.exitCode}` };
  }
  return JSON.parse(wins.stdout.toString().trim());
}

try {
  (driver as any).send({ type: "show" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 10_000 });
  await new Promise((r) => setTimeout(r, 1500));

  for (const value of LADDER) {
    TEMPLATE.opacity.vibrancy_background = value;
    writeFileSync(themePath, JSON.stringify(TEMPLATE, null, 2));
    // theme watcher polls every 200ms; give reload + repaint time to settle
    await new Promise((r) => setTimeout(r, 1800));
    const tag = String(Math.round(value * 100));
    await capture(tag);
    console.log(
      JSON.stringify({
        step: value,
        captured: tag,
        appPid: driver.pid,
        onScreenScriptKitWindows: onScreenScriptKitWindows(),
      }),
    );
  }
  console.log(JSON.stringify({ ok: true, sessionDir: driver.sessionDir }));
} finally {
  await driver.close();
}
