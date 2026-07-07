// Glamour video capture: short screen recordings of the app in use, for the
// scriptkit.com landing page loops.
//
// Scenarios:
// - search-loop: human-paced typing in unified search, showing instant
//   filtering across queries, ending on an empty filter for a clean loop.
// - shaders-loop: type "next effect" once, then press Enter repeatedly —
//   each press cycles the background shader live (aurora → plasma → …).
//
// Mechanics (shared with shader-screenshot-probe.ts — keep in sync):
// - Record on the display the window actually lands on (Script Kit positions
//   on the display with the mouse); the capture rect comes from
//   tahoe_window_geometry (global points), padded, fed to
//   `screencapture -v -R` which records that region at retina scale.
// - Effects are startup-only: pre-seed the home dir config.ts per scenario.
// - Hide every other visible app first (snapshot names, restore after) so
//   the panel floats over the bare wallpaper.
// - Requires Screen Recording + Automation permissions in an interactive,
//   unlocked session — not runnable from an agent sandbox.
//
// Run: bun scripts/agentic/glamour-video-probe.ts
// Masters land in .test-screenshots/glamour/video/*.mov (VFR); encode for
// the web with ffmpeg (fps=30, h264, faststart) into site/videos/.

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

// simulateKey only handles named keys (enter/backspace/…), not letters —
// per-letter key names are silently dropped. Progressive setFilter prefixes
// render identically to typing.
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

type Scenario = {
  name: string;
  effect: string | null;
  theme: string | null;
  durationSecs: number;
  run: (driver: Driver) => Promise<void>;
};

const SCENARIOS: Scenario[] = [
  {
    // Fireflies on the default amber theme: warm on-brand orbs drifting
    // behind the search interaction.
    name: "search-loop",
    effect: "fireflies",
    theme: null,
    durationSecs: 16,
    run: async (driver) => {
      await Bun.sleep(900);
      await typeText(driver, "clip");
      await Bun.sleep(1500);
      await backspaceAll(driver, "clip");
      await Bun.sleep(500);
      await typeText(driver, "notes");
      await Bun.sleep(1500);
      await backspaceAll(driver, "notes");
      await Bun.sleep(500);
      await typeText(driver, "window");
      await Bun.sleep(1500);
      await backspaceAll(driver, "window");
      await Bun.sleep(1200);
    },
  },
  {
    // Starfield on SynthWave '84 (pink stars on deep blue) is the strongest
    // opener; cycling from it walks LavaLamp -> Nebula -> Rain.
    name: "shaders-loop",
    effect: "starfield",
    theme: "synthwave-84",
    durationSecs: 18,
    run: async (driver) => {
      await Bun.sleep(1200);
      await typeText(driver, "next effect");
      await Bun.sleep(900);
      for (let i = 0; i < 3; i++) {
        driver.simulateKey("enter");
        await Bun.sleep(3300);
      }
      await backspaceAll(driver, "next effect", 45);
      await Bun.sleep(800);
    },
  },
];

const receipt: any = { videos: [], errors: [] };
const hidden = hideOtherApps();
receipt.hiddenApps = hidden;

try {
  for (const scenario of SCENARIOS) {
    const home = `/tmp/sk-video-home-${scenario.name}`;
    rmSync(home, { recursive: true, force: true });
    const kitDir = join(home, ".scriptkit");
    mkdirSync(kitDir, { recursive: true });
    const themeLine = scenario.theme ? `  theme: { presetId: "${scenario.theme}" },\n` : "";
    const effectLine = scenario.effect
      ? `  effects: { background: "${scenario.effect}", intensity: 0.9 },\n`
      : "";
    writeFileSync(join(kitDir, "config.ts"), `export default {\n${themeLine}${effectLine}};\n`);

    const driver = await Driver.launch({
      binary: join(PROJECT_ROOT, "target-agent/artifacts/glamour/script-kit-gpui"),
      sessionName: `video-${scenario.name}`,
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
      const mov = join(OUT_DIR, `${scenario.name}.mov`);
      rmSync(mov, { force: true });
      const rec = Bun.spawn([
        "screencapture", "-x", "-v",
        "-V", String(scenario.durationSecs),
        "-R", `${rect.x},${rect.y},${rect.w},${rect.h}`,
        mov,
      ]);
      // let the recorder actually start before driving the UI
      await Bun.sleep(900);
      await scenario.run(driver);
      await rec.exited;
      let effectCycles: number | null = null;
      if (scenario.name === "shaders-loop") {
        const logs = (await driver.getLogs({
          limit: 200,
          contains: "background-effect-next",
        })) as any;
        effectCycles = (logs?.entries ?? []).filter((e: any) =>
          String(e.message ?? "").includes("status=success"),
        ).length;
        if (!effectCycles) throw new Error("shader cycling never fired during recording");
      }
      receipt.videos.push({ name: scenario.name, rect, mov, effectCycles });
    } catch (err: any) {
      receipt.errors.push({ name: scenario.name, error: String(err?.message ?? err) });
    } finally {
      await driver.close();
    }
  }
} finally {
  restoreApps(hidden);
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.errors.length > 0 || receipt.videos.length !== SCENARIOS.length) process.exit(1);
