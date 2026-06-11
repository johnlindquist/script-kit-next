/**
 * Probe: grid over vibrancy material x window-root tint alpha, hot-swapped
 * via the sandbox theme.json watcher, screencapturing each cell to compare
 * how much backdrop color/saturation survives each combination.
 */
import { join } from "node:path";
import { readFileSync, writeFileSync } from "node:fs";
import { Driver } from "../devtools/driver.ts";

const MATERIALS = (process.env.VIB_MATERIALS ?? "hud,popover,menu,sidebar,content")
  .split(",")
  .map((s) => s.trim())
  .filter(Boolean);
const ALPHAS = (process.env.VIB_ALPHAS ?? "0.85,0.75")
  .split(",")
  .map((s) => Number(s.trim()))
  .filter((n) => Number.isFinite(n));
const PREFIX = process.env.VIB_PREFIX ?? "vibgrid";
const TEMPLATE = JSON.parse(
  readFileSync("tests/theme/snapshots/theme_dark_default.json", "utf8"),
);

const driver = await Driver.launch({
  binary: "target-agent/artifacts/vibrancy-085/script-kit-gpui",
  sandboxHome: true,
  sessionName: "vibrancy-material-grid",
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

function onScreenScriptKitWindows(): Array<{
  pid: number;
  x: number;
  y: number;
  w: number;
  h: number;
}> {
  // /tmp/list-sk-windows: C tool over CGWindowListCopyWindowInfo printing
  // on-screen windows owned by *script* processes (global CG coords, points).
  const wins = Bun.spawnSync(["/tmp/list-sk-windows"]);
  if (wins.exitCode !== 0) {
    throw new Error(`list-sk-windows exit ${wins.exitCode}`);
  }
  return JSON.parse(wins.stdout.toString().trim());
}

try {
  (driver as any).send({ type: "show" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 10_000 });
  await new Promise((r) => setTimeout(r, 1500));

  for (const material of MATERIALS) {
    for (const alpha of ALPHAS) {
      TEMPLATE.vibrancy = { enabled: true, material };
      TEMPLATE.opacity.vibrancy_background = alpha;
      writeFileSync(themePath, JSON.stringify(TEMPLATE, null, 2));
      await new Promise((r) => setTimeout(r, 1800));
      const tag = `${material}-${Math.round(alpha * 100)}`;
      await capture(tag);
      console.log(
        JSON.stringify({
          material,
          alpha,
          tag,
          windows: onScreenScriptKitWindows(),
        }),
      );
    }
  }
  console.log(JSON.stringify({ ok: true, sessionDir: driver.sessionDir }));
} finally {
  await driver.close();
}
