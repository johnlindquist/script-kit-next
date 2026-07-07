// Glamour capture: main launcher with background shader effects enabled,
// floating over the clean desktop wallpaper.
// Produces .test-screenshots/glamour/{22-shader-background,23-shader-background-alt}.png.
//
// Mechanics (each learned the hard way — keep them):
// - Effects are read ONCE at startup (src/effects.rs startup_prefs OnceLock),
//   so each shot pre-seeds a home dir with `effects.background` in config.ts
//   and launches a fresh app instance. Hot-reloading config does NOT apply.
// - Driver/protocol window frames are NOT global screen coordinates. The OS
//   capture rect comes from scripts/devtools/bin/tahoe_window_geometry
//   (CGWindowList, global points + display pixels).
// - Polished background: every other visible app is hidden (System Events)
//   before the capture so the translucent panel floats over the wallpaper,
//   then restored afterward. Requires Screen Recording + Automation
//   permissions in the invoking terminal — run from an interactive session,
//   not an agent sandbox (mdflow sandboxes fail with
//   "could not create image from display").
//
// Run: bun scripts/agentic/shader-screenshot-probe.ts

import { join } from "node:path";
import { mkdirSync, writeFileSync, rmSync } from "node:fs";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = new URL("../..", import.meta.url).pathname;
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/glamour");
const GEO = join(PROJECT_ROOT, "scripts/devtools/bin/tahoe_window_geometry");
mkdirSync(OUT_DIR, { recursive: true });

const SHOTS: Array<{ name: string; effect: string }> = [
  { name: "22-shader-background", effect: "aurora" },
  { name: "23-shader-background-alt", effect: "starfield" },
];

// Desktop margin (points) shown around the panel, matching the shot set.
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

const receipt: any = { shots: [], errors: [] };
const hidden = hideOtherApps();
receipt.hiddenApps = hidden;

try {
  for (const shot of SHOTS) {
    const home = `/tmp/sk-shader-glamour-home-${shot.effect}`;
    rmSync(home, { recursive: true, force: true });
    const kitDir = join(home, ".scriptkit");
    mkdirSync(kitDir, { recursive: true });
    writeFileSync(
      join(kitDir, "config.ts"),
      `export default {\n  effects: { background: "${shot.effect}", intensity: 0.9 },\n};\n`,
    );

    const driver = await Driver.launch({
      binary: join(PROJECT_ROOT, "target-agent/artifacts/glamour/script-kit-gpui"),
      sessionName: `shader-${shot.effect}`,
      env: { HOME: home, SK_PATH: kitDir },
    });
    try {
      driver.send({ type: "show" });
      await driver.waitForSettle();
      // let the shader animate a few seconds to a visually interesting frame
      await Bun.sleep(3000);

      const f = await windowRectPoints();
      const rect = {
        x: Math.max(0, Math.round(f.x - PAD_X)),
        y: Math.max(0, Math.round(f.y - PAD_Y)),
        w: Math.round(f.w + PAD_X * 2),
        h: Math.round(f.h + PAD_Y * 2),
      };
      const png = join(OUT_DIR, `${shot.name}.png`);
      const cap = Bun.spawnSync([
        "screencapture", "-x",
        "-R", `${rect.x},${rect.y},${rect.w},${rect.h}`,
        png,
      ]);
      if (cap.exitCode !== 0) {
        receipt.errors.push({ shot: shot.name, stderr: cap.stderr.toString() });
      } else {
        receipt.shots.push({ shot: shot.name, effect: shot.effect, windowRect: f, captureRect: rect, png });
      }
    } finally {
      await driver.close();
    }
  }
} finally {
  restoreApps(hidden);
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.errors.length > 0 || receipt.shots.length !== SHOTS.length) process.exit(1);
