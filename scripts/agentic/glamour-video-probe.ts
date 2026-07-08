// Glamour video capture: demo-reel candidates of the app in real use, for
// the scriptkit.com landing page hero.
//
// Scenarios (pick with CLI args, default all):
//   shaders-loop    — type "next effect", Return cycles shaders; arrows +
//                     mouse sweeps show the focus halo tracking live use.
//   today-capture   — dictation transcripts seeded into today's Day Page
//                     (pushDictationResult target "today"), then a typed
//                     space opens Today showing the timestamped captures.
//   agent-chat      — kitchen-sink Agent Chat fixture (rich provider-free
//                     transcript), scroll it, paste clipboard into composer.
//   theme-preview   — type "theme", Enter, arrows live-preview themes; the
//                     whole app AND the theme-anchored shader recolor.
//   montage         — greatest hits: search, shader cycling, theme flips.
//
// Mechanics (shared with shader-screenshot-probe.ts — keep in sync):
// - Capture rect from tahoe_window_geometry (global points) + generous
//   padding (some views grow the main window), fed to `screencapture -v -R`.
// - Effects/theme are startup-only: pre-seed the home dir config.ts.
// - protocol simulateKey handles named keys only; typing = setFilter prefixes.
// - Day Page: typing a bare space opens Today. Dictation pushes write to
//   disk only (no live view update) — seed BEFORE opening the Day Page.
// - Clipboard watcher is on by default (200-500ms poll, content-hash dedup):
//   distinct strings written via osascript are captured. The user's
//   clipboard text is saved and restored afterward.
// - Mouse sweeps via cliclick with easing; cursor position saved/restored.
// - Hide every other visible app first (snapshot names, restore after).
// - Requires Screen Recording + Automation permissions in an interactive,
//   unlocked session — not runnable from an agent sandbox.
//
// Run: bun scripts/agentic/glamour-video-probe.ts [scenario ...]
// Masters land in .test-screenshots/glamour/video/<name>.mov (VFR); encode
// with ffmpeg (fps=30, h264, faststart) for the web.

import { join } from "node:path";
import { mkdirSync, writeFileSync, rmSync } from "node:fs";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = new URL("../..", import.meta.url).pathname;
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/glamour/video");
const GEO = join(PROJECT_ROOT, "scripts/devtools/bin/tahoe_window_geometry");
mkdirSync(OUT_DIR, { recursive: true });

const PAD_X = 150;
const PAD_TOP = 120;
const PAD_BOTTOM = 340; // day page / agent chat views grow the window downward

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

type Geo = {
  win: { x: number; y: number; w: number; h: number };
  display: { x: number; y: number; w: number; h: number };
};

async function windowGeometry(): Promise<Geo> {
  let lastErr = "";
  for (let attempt = 0; attempt < 20; attempt++) {
    const r = Bun.spawnSync([GEO, "--owner", "script-kit-gpui"]);
    const geo = JSON.parse(r.stdout.toString());
    if (geo.ok && geo.winRect && geo.displayBounds) {
      return { win: geo.winRect, display: geo.displayBounds };
    }
    lastErr = JSON.stringify(geo);
    await Bun.sleep(500);
  }
  throw new Error(`window geometry failed after retries: ${lastErr}`);
}

function log(step: string) {
  console.error(`[reel ${new Date().toISOString().slice(11, 19)}] ${step}`);
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

async function keys(driver: Driver, sequence: string[], stepMs = 340) {
  for (const key of sequence) {
    driver.simulateKey(key);
    await Bun.sleep(stepMs);
  }
}

function mousePos(): { x: number; y: number } | null {
  const r = Bun.spawnSync(["cliclick", "p"]);
  const m = r.stdout.toString().match(/(\d+),(\d+)/);
  return m ? { x: Number(m[1]), y: Number(m[2]) } : null;
}

async function mouseSweep(points: Array<{ x: number; y: number }>, easeMs = 700) {
  for (const p of points) {
    Bun.spawnSync(["cliclick", "-e", String(easeMs), `m:${Math.round(p.x)},${Math.round(p.y)}`]);
    await Bun.sleep(120);
  }
}

function setClipboard(text: string) {
  const escaped = text.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  osascript(`set the clipboard to "${escaped}"`);
}

function getClipboardText(): string | null {
  try {
    return osascript("the clipboard as text");
  } catch {
    return null;
  }
}

type Waypoints = { left: any; right: any; lower: any; upper: any };

type Scenario = {
  name: string;
  theme: string | null;
  effect: string;
  durationSecs: number;
  /** Runs after launch+show, BEFORE recording starts (invisible staging). */
  stage?: (driver: Driver) => Promise<void>;
  run: (driver: Driver, w: Waypoints) => Promise<void>;
};

const DICTATION_SEEDS = [
  "Morning run done before it got hot",
  "Idea: end the glamour reel on the day page",
  "Call with Sarah moved to two thirty, bring the demo laptop",
];

const SCENARIOS: Scenario[] = [
  {
    name: "shaders-loop",
    theme: "synthwave-84",
    effect: "starfield",
    durationSecs: 26,
    run: async (driver, w) => {
      // Starfield reacts subtly to the cursor — no sweep on the opener.
      await Bun.sleep(1400);
      await typeText(driver, "next effect");
      await Bun.sleep(800);
      driver.simulateKey("enter");
      await Bun.sleep(1400);
      await keys(driver, ["down", "down", "down", "up", "up", "up"]);
      driver.simulateKey("enter");
      await Bun.sleep(1200);
      await mouseSweep([w.lower, w.upper, w.right], 650);
      driver.simulateKey("enter");
      await Bun.sleep(1200);
      await keys(driver, ["down", "down", "up", "up"]);
      driver.simulateKey("enter");
      await Bun.sleep(1200);
      await mouseSweep([w.right, w.left], 900);
      driver.simulateKey("enter");
      await Bun.sleep(1200);
      await backspaceAll(driver, "next effect", 45);
      await Bun.sleep(400);
      await keys(driver, ["down", "down", "down", "up", "up", "up"], 300);
      await mouseSweep([w.left, w.right], 800);
    },
  },
  {
    name: "today-capture",
    theme: "rose-pine",
    effect: "starfield",
    durationSecs: 22,
    stage: async (driver) => {
      for (const transcript of DICTATION_SEEDS) {
        driver.send({ type: "pushDictationResult", transcript, target: "today" });
        await Bun.sleep(500);
      }
    },
    run: async (driver, w) => {
      await Bun.sleep(900);
      await keys(driver, ["down", "down", "up", "up"], 340);
      await Bun.sleep(600);
      // A bare space is the Day Page trigger: Today opens with the seeded
      // timestamped captures.
      driver.setFilter(" ");
      await Bun.sleep(2200);
      await keys(driver, ["down", "down", "down", "down"], 420);
      await Bun.sleep(1400);
      await keys(driver, ["down", "down"], 420);
      await Bun.sleep(1200);
      await keys(driver, ["up", "up", "up", "up", "up", "up"], 300);
      await Bun.sleep(1600);
    },
  },
  {
    name: "agent-chat",
    theme: "dracula",
    effect: "lavalamp",
    durationSecs: 24,
    stage: async () => {
      setClipboard("Add a --dry-run flag so I can preview the rename first.");
      await Bun.sleep(300);
    },
    run: async (driver, w) => {
      await Bun.sleep(900);
      await typeText(driver, "agent chat");
      await Bun.sleep(1100);
      driver.send({ type: "openAgentChatKitchenSinkFixture" });
      await Bun.sleep(2400);
      await keys(driver, ["down", "down", "down", "down", "down"], 500);
      await Bun.sleep(800);
      await keys(driver, ["up", "up"], 450);
      await Bun.sleep(600);
      await mouseSweep([w.lower, w.upper], 800);
      driver.send({ type: "pasteClipboardIntoAgentChat" });
      await Bun.sleep(2200);
    },
  },
  {
    name: "theme-preview",
    theme: null, // start on Script Kit Dark so the recolors travel far
    effect: "plasma",
    durationSecs: 24,
    run: async (driver, w) => {
      await Bun.sleep(800);
      await mouseSweep([w.left, w.right], 800);
      await typeText(driver, "theme");
      await Bun.sleep(1000);
      driver.simulateKey("enter");
      await Bun.sleep(1600);
      // Each arrow live-previews: the whole app and the theme-anchored
      // shader recolor immediately.
      await keys(driver, ["down"], 1);
      await Bun.sleep(1300);
      await keys(driver, ["down"], 1);
      await Bun.sleep(1300);
      await keys(driver, ["down"], 1);
      await Bun.sleep(1300);
      await keys(driver, ["down"], 1);
      await Bun.sleep(1300);
      await keys(driver, ["down"], 1);
      await Bun.sleep(1300);
      await keys(driver, ["up", "up"], 900);
      await Bun.sleep(1000);
      await mouseSweep([w.lower, w.upper], 700);
      driver.simulateKey("escape");
      await Bun.sleep(1400);
    },
  },
  {
    name: "montage",
    theme: "synthwave-84",
    effect: "starfield",
    durationSecs: 26,
    stage: async (driver) => {
      for (const transcript of DICTATION_SEEDS) {
        driver.send({ type: "pushDictationResult", transcript, target: "today" });
        await Bun.sleep(400);
      }
    },
    run: async (driver) => {
      await Bun.sleep(900);
      await typeText(driver, "agent chat", 85);
      await Bun.sleep(1100);
      await backspaceAll(driver, "agent chat", 40);
      await typeText(driver, "next effect", 80);
      await Bun.sleep(700);
      driver.simulateKey("enter");
      await Bun.sleep(1700);
      driver.simulateKey("enter");
      await Bun.sleep(1700);
      await backspaceAll(driver, "next effect", 40);
      await typeText(driver, "theme", 80);
      await Bun.sleep(600);
      driver.simulateKey("enter");
      await Bun.sleep(1400);
      await keys(driver, ["down"], 1);
      await Bun.sleep(1200);
      await keys(driver, ["down"], 1);
      await Bun.sleep(1200);
      driver.simulateKey("escape");
      await Bun.sleep(900);
      // Space → Today with the seeded captures as the closing shot.
      driver.setFilter(" ");
      await Bun.sleep(2400);
      await keys(driver, ["down", "down"], 500);
      await Bun.sleep(1200);
    },
  },
];

const requested = process.argv.slice(2);
const toRun = requested.length
  ? SCENARIOS.filter((s) => requested.includes(s.name))
  : SCENARIOS;
if (requested.length && toRun.length !== requested.length) {
  console.error(`unknown scenario in [${requested.join(", ")}]; known: ${SCENARIOS.map((s) => s.name).join(", ")}`);
  process.exit(1);
}

const receipt: any = { videos: [], errors: [] };
const hidden = hideOtherApps();
receipt.hiddenApps = hidden;
const originalMouse = mousePos();
const originalClipboard = getClipboardText();

try {
  for (const scenario of toRun) {
    const home = `/tmp/sk-video-home-${scenario.name}`;
    rmSync(home, { recursive: true, force: true });
    const kitDir = join(home, ".scriptkit");
    mkdirSync(kitDir, { recursive: true });
    const themeLine = scenario.theme ? `  theme: { presetId: "${scenario.theme}" },\n` : "";
    writeFileSync(
      join(kitDir, "config.ts"),
      `export default {\n${themeLine}  effects: { background: "${scenario.effect}", intensity: 0.9 },\n};\n`,
    );

    const driver = await Driver.launch({
      binary: join(PROJECT_ROOT, "target-agent/artifacts/glamour/script-kit-gpui"),
      sessionName: `video-${scenario.name}`,
      env: { HOME: home, SK_PATH: kitDir },
    });
    try {
      driver.send({ type: "show" });
      await driver.waitForSettle();
      if (scenario.stage) {
        log(`${scenario.name}: staging`);
        await scenario.stage(driver);
        await driver.waitForSettle();
      }
      const { win: f, display } = await windowGeometry();
      // Clamp to the display: screencapture -v rejects/or misbehaves on
      // rects that extend past the screen.
      const x0 = Math.max(display.x, Math.round(f.x - PAD_X));
      const y0 = Math.max(display.y, Math.round(f.y - PAD_TOP));
      const x1 = Math.min(display.x + display.w, Math.round(f.x + f.w + PAD_X));
      const y1 = Math.min(display.y + display.h, Math.round(f.y + f.h + PAD_BOTTOM));
      const rect = { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
      const cx = f.x + f.w / 2;
      const listY = f.y + f.h * 0.55;
      const waypoints: Waypoints = {
        left: { x: f.x + f.w * 0.18, y: listY },
        right: { x: f.x + f.w * 0.82, y: listY },
        lower: { x: cx, y: f.y + f.h * 0.8 },
        upper: { x: cx, y: f.y + f.h * 0.3 },
      };

      const mov = join(OUT_DIR, `${scenario.name}.mov`);
      rmSync(mov, { force: true });
      log(`${scenario.name}: recording rect=${JSON.stringify(rect)}`);
      const rec = Bun.spawn([
        "screencapture", "-x", "-v",
        "-V", String(scenario.durationSecs),
        "-R", `${rect.x},${rect.y},${rect.w},${rect.h}`,
        mov,
      ]);
      await Bun.sleep(900);
      await scenario.run(driver, waypoints);
      log(`${scenario.name}: interaction done, waiting for recorder`);
      const recResult = await Promise.race([
        rec.exited,
        Bun.sleep((scenario.durationSecs + 20) * 1000).then(() => "timeout" as const),
      ]);
      if (recResult === "timeout") {
        rec.kill();
        throw new Error("screencapture did not exit; killed");
      }
      if (!(await Bun.file(mov).exists())) {
        throw new Error(`screencapture exited (code ${recResult}) but wrote no file`);
      }
      log(`${scenario.name}: recorded ok`);
      receipt.videos.push({ name: scenario.name, rect, mov });
    } catch (err: any) {
      receipt.errors.push({ name: scenario.name, error: String(err?.message ?? err) });
    } finally {
      const closed = await Promise.race([
        driver.close().then(() => true),
        Bun.sleep(15000).then(() => false),
      ]);
      if (!closed) {
        log(`${scenario.name}: driver.close timed out; force-killing app`);
        Bun.spawnSync(["pkill", "-f", "artifacts/glamour/script-kit-gpui"]);
      }
    }
  }
} finally {
  restoreApps(hidden);
  if (originalMouse) {
    Bun.spawnSync(["cliclick", `m:${originalMouse.x},${originalMouse.y}`]);
  }
  if (originalClipboard !== null) {
    try {
      setClipboard(originalClipboard);
    } catch {}
  }
}

console.log(JSON.stringify(receipt, null, 2));
if (receipt.errors.length > 0 || receipt.videos.length !== toRun.length) process.exit(1);
