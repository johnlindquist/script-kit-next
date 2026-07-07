// Glamour video capture: the single hero loop of the app in use, for the
// scriptkit.com landing page.
//
// shaders-loop storyboard (~24s, SynthWave '84 theme, starts on Starfield):
// type "next effect" live → Enter cycles the shader (LavaLamp → Nebula →
// Rain → Waves → Fireflies), and between cycles the probe arrows the list
// selection up/down and sweeps the real mouse across the panel — every
// effect carries a focus halo that visibly tracks the caret, the selected
// row, and the cursor, so the video shows the shaders reacting to live use.
//
// Mechanics (shared with shader-screenshot-probe.ts — keep in sync):
// - Record on the display the window actually lands on (Script Kit positions
//   on the display with the mouse); the capture rect comes from
//   tahoe_window_geometry (global points), padded, fed to
//   `screencapture -v -R` which records that region at retina scale.
// - Effects/theme are startup-only: pre-seed the home dir config.ts.
// - protocol simulateKey only handles named keys (enter/backspace/up/down);
//   typing is progressive setFilter prefixes, which renders identically.
// - Mouse sweeps use cliclick (brew install cliclick) with easing; the
//   original cursor position is saved and restored.
// - Hide every other visible app first (snapshot names, restore after) so
//   the panel floats over the bare wallpaper.
// - Requires Screen Recording + Automation permissions in an interactive,
//   unlocked session — not runnable from an agent sandbox.
//
// Run: bun scripts/agentic/glamour-video-probe.ts
// Master lands in .test-screenshots/glamour/video/shaders-loop.mov (VFR);
// encode for the web with ffmpeg (fps=30, h264, faststart) into site/videos/.

import { join } from "node:path";
import { mkdirSync, writeFileSync, rmSync } from "node:fs";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = new URL("../..", import.meta.url).pathname;
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/glamour/video");
const GEO = join(PROJECT_ROOT, "scripts/devtools/bin/tahoe_window_geometry");
mkdirSync(OUT_DIR, { recursive: true });

const PAD_X = 130;
const PAD_Y = 110;

function osascript(script: string): string {
  const r = Bun.spawnSync(["osascript", "-e", script]);
  if (r.exitCode !== 0) throw new Error(`osascript failed: ${r.stderr.toString()}`);
  return r.stdout.toString().trim();
}

function hideOtherApps(): string[] {
  const out = osascript(`
    set hiddenApps to {}
    tell application "System Events"
      set visibleNames to name of every process whose visible is true and background only is false
    end tell
    repeat with pname in visibleNames
      set n to contents of pname
      if n is not "script-kit-gpui" then
        try
          tell application "System Events" to set visible of process n to false
          copy n to end of hiddenApps
        end try
      end if
    end repeat
    set AppleScript's text item delimiters to "\\n"
    return hiddenApps as text
  `);
  return out.length ? out.split("\n") : [];
}

function restoreApps(names: string[]) {
  for (const name of names) {
    try {
      osascript(`tell application "System Events" to set visible of process "${name}" to true`);
    } catch {}
  }
}

async function windowRectPoints(): Promise<{ x: number; y: number; w: number; h: number }> {
  let lastErr = "";
  for (let attempt = 0; attempt < 20; attempt++) {
    const r = Bun.spawnSync([GEO, "--owner", "script-kit-gpui"]);
    const geo = JSON.parse(r.stdout.toString());
    if (geo.ok && geo.winRect) {
      const rect = geo.winRect;
      return { x: rect.x, y: rect.y, w: rect.w, h: rect.h };
    }
    lastErr = JSON.stringify(geo);
    await Bun.sleep(500);
  }
  throw new Error(`window geometry failed after retries: ${lastErr}`);
}

async function typeText(driver: Driver, text: string, perKeyMs = 95) {
  for (let i = 1; i <= text.length; i++) {
    driver.setFilter(text.slice(0, i));
    await Bun.sleep(perKeyMs);
  }
}

async function backspaceAll(driver: Driver, text: string, perKeyMs = 55) {
  for (let i = text.length - 1; i >= 0; i--) {
    driver.setFilter(text.slice(0, i));
    await Bun.sleep(perKeyMs);
  }
}

async function arrowDance(driver: Driver, downs: number, ups: number, stepMs = 340) {
  for (let i = 0; i < downs; i++) {
    driver.simulateKey("down");
    await Bun.sleep(stepMs);
  }
  for (let i = 0; i < ups; i++) {
    driver.simulateKey("up");
    await Bun.sleep(stepMs);
  }
}

function mousePos(): { x: number; y: number } | null {
  const r = Bun.spawnSync(["cliclick", "p"]);
  const m = r.stdout.toString().match(/(\d+),(\d+)/);
  return m ? { x: Number(m[1]), y: Number(m[2]) } : null;
}

// Smoothly move the real cursor through the given points (global coords).
async function mouseSweep(points: Array<{ x: number; y: number }>, easeMs = 700) {
  for (const p of points) {
    Bun.spawnSync(["cliclick", "-e", String(easeMs), `m:${Math.round(p.x)},${Math.round(p.y)}`]);
    await Bun.sleep(120);
  }
}

const receipt: any = { videos: [], errors: [] };
const hidden = hideOtherApps();
receipt.hiddenApps = hidden;
const originalMouse = mousePos();

try {
  const home = `/tmp/sk-video-home-shaders-loop`;
  rmSync(home, { recursive: true, force: true });
  const kitDir = join(home, ".scriptkit");
  mkdirSync(kitDir, { recursive: true });
  writeFileSync(
    join(kitDir, "config.ts"),
    `export default {\n  theme: { presetId: "synthwave-84" },\n  effects: { background: "starfield", intensity: 0.9 },\n};\n`,
  );

  const driver = await Driver.launch({
    binary: join(PROJECT_ROOT, "target-agent/artifacts/glamour/script-kit-gpui"),
    sessionName: "video-shaders-loop",
    env: { HOME: home, SK_PATH: kitDir },
  });
  try {
    driver.send({ type: "show" });
    await driver.waitForSettle();
    const f = await windowRectPoints();
    const rect = {
      x: Math.max(0, Math.round(f.x - PAD_X)),
      y: Math.max(0, Math.round(f.y - PAD_Y)),
      w: Math.round(f.w + PAD_X * 2),
      h: Math.round(f.h + PAD_Y * 2),
    };
    // Sweep waypoints inside the window body (below the input row).
    const cx = f.x + f.w / 2;
    const listY = f.y + f.h * 0.55;
    const left = { x: f.x + f.w * 0.18, y: listY };
    const right = { x: f.x + f.w * 0.82, y: listY };
    const lower = { x: cx, y: f.y + f.h * 0.8 };
    const upper = { x: cx, y: f.y + f.h * 0.3 };

    const DURATION = 26;
    const mov = join(OUT_DIR, "shaders-loop.mov");
    rmSync(mov, { force: true });
    const rec = Bun.spawn([
      "screencapture", "-x", "-v",
      "-V", String(DURATION),
      "-R", `${rect.x},${rect.y},${rect.w},${rect.h}`,
      mov,
    ]);
    await Bun.sleep(900);

    // — Starfield: mouse sweep, then type the command live.
    await mouseSweep([left, right], 900);
    await typeText(driver, "next effect");
    await Bun.sleep(800);

    // — LavaLamp: arrows walk the results, halo tracks the selection.
    driver.simulateKey("enter");
    await Bun.sleep(1400);
    await arrowDance(driver, 3, 3);

    // — Nebula: diagonal mouse sweep.
    driver.simulateKey("enter");
    await Bun.sleep(1200);
    await mouseSweep([lower, upper, right], 650);

    // — Rain: a short arrow run.
    driver.simulateKey("enter");
    await Bun.sleep(1200);
    await arrowDance(driver, 2, 2);

    // — Waves: wide sweep.
    driver.simulateKey("enter");
    await Bun.sleep(1200);
    await mouseSweep([right, left], 900);

    // — Fireflies: clear the filter, dance the Suggested list, final sweep.
    driver.simulateKey("enter");
    await Bun.sleep(1200);
    await backspaceAll(driver, "next effect", 45);
    await Bun.sleep(400);
    await arrowDance(driver, 3, 3, 300);
    await mouseSweep([left, right], 800);

    await rec.exited;

    const logs = (await driver.getLogs({
      limit: 200,
      contains: "background-effect-next",
    })) as any;
    const effectCycles = (logs?.entries ?? []).filter((e: any) =>
      String(e.message ?? "").includes("status=success"),
    ).length;
    if (effectCycles < 5) throw new Error(`expected 5 shader cycles, saw ${effectCycles}`);
    receipt.videos.push({ name: "shaders-loop", rect, mov, effectCycles });
  } finally {
    await driver.close();
  }
} catch (err: any) {
  receipt.errors.push({ error: String(err?.message ?? err) });
} finally {
  restoreApps(hidden);
  if (originalMouse) {
    Bun.spawnSync(["cliclick", `m:${originalMouse.x},${originalMouse.y}`]);
  }
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.errors.length > 0 || receipt.videos.length !== 1) process.exit(1);
