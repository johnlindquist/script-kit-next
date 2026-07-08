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
//   team-review-actions      — shared compact Actions search + empty guidance.
//   team-review-permissions  — shared permissions intro/progress surface.
//   team-review-agent-setup  — provider-free Agent Chat setup recovery card.
//   team-review-day-page     — compact clipboard shelf + past-day back affordance.
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
import { existsSync, mkdirSync, readFileSync, writeFileSync, rmSync } from "node:fs";
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

const ACTIONS_DIALOG_TARGET = { type: "kind", kind: "actionsDialog" };

async function setActionsDialogInput(driver: Driver, text: string) {
  const result = await driver.request(
    {
      type: "batch",
      target: ACTIONS_DIALOG_TARGET,
      commands: [{ type: "setInput", text }],
      options: { stopOnError: true, rollbackOnError: false, timeout: 5000 },
      trace: "on",
    },
    { expect: "batchResult", timeoutMs: 6000 },
  );
  if (result.success === false) {
    throw new Error(`ActionsDialog setInput failed for ${JSON.stringify(text)}: ${JSON.stringify(result)}`);
  }
  return result;
}

async function progressivelySetActionsDialogInput(
  driver: Driver,
  text: string,
  perKeyMs = 85,
) {
  for (let i = 1; i <= text.length; i++) {
    await setActionsDialogInput(driver, text.slice(0, i));
    await Bun.sleep(perKeyMs);
  }
}

async function progressivelyClearActionsDialogInput(
  driver: Driver,
  text: string,
  perKeyMs = 45,
) {
  for (let i = text.length - 1; i >= 0; i--) {
    await setActionsDialogInput(driver, text.slice(0, i));
    await Bun.sleep(perKeyMs);
  }
}

async function actionsDialogState(driver: Driver) {
  return driver.request(
    { type: "getState", target: ACTIONS_DIALOG_TARGET, summaryOnly: true },
    { expect: "stateResult", timeoutMs: 6000 },
  );
}

async function waitForActionsDialog(driver: Driver) {
  for (let attempt = 0; attempt < 50; attempt++) {
    try {
      const state = await actionsDialogState(driver);
      if (state.actionsDialog?.surface === "actionsDialog") return state;
    } catch {}
    await Bun.sleep(100);
  }
  throw new Error("ActionsDialog did not become automation-ready");
}

async function waitForAgentChatSetup(driver: Driver) {
  for (let attempt = 0; attempt < 80; attempt++) {
    try {
      const result = await driver.request(
        { type: "getAgentChatState" },
        { timeoutMs: 5000 },
      );
      const state = result.state ?? result;
      if (state.setup) return state;
    } catch {}
    await Bun.sleep(125);
  }
  throw new Error(
    "Agent Chat setup did not become deterministic; missing setup fixture/empty-sandbox setup state",
  );
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

async function mouseClick(point: { x: number; y: number }, easeMs = 500) {
  Bun.spawnSync([
    "cliclick",
    "-e",
    String(easeMs),
    `m:${Math.round(point.x)},${Math.round(point.y)}`,
    `c:${Math.round(point.x)},${Math.round(point.y)}`,
  ]);
  await Bun.sleep(180);
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

type ScenarioContext = {
  home: string;
  kitDir: string;
};

type Scenario = {
  name: string;
  theme: string | null;
  effect: string;
  durationSecs: number;
  env?: Record<string, string>;
  /** Runs after launch+show, BEFORE recording starts (invisible staging). */
  stage?: (driver: Driver, context: ScenarioContext) => Promise<Record<string, any> | void>;
  run: (driver: Driver, w: Waypoints) => Promise<Record<string, any> | void>;
};

const DICTATION_SEEDS = [
  "Morning run done before it got hot",
  "Idea: end the glamour reel on the day page",
  "Call with Sarah moved to two thirty, bring the demo laptop",
];

const ACTIONS_NO_MATCH_QUERY = "zz-no-action-848b3428a";
const TEAM_REVIEW_PAST_DATE = "2026-06-30";
const TEAM_REVIEW_PAST_DAY = [
  "# Team review archive",
  "",
  "Polish the recovery states before sharing the next build.",
  "",
  "- Verify compact search alignment",
  "- Review onboarding progress",
].join("\n");

function localDateStamp() {
  return new Intl.DateTimeFormat("en-CA", {
    timeZone: process.env.SCRIPT_KIT_BRAIN_TZ ?? "America/Denver",
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
  }).format(new Date());
}

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
  {
    name: "team-review-actions",
    theme: "rose-pine",
    effect: "starfield",
    durationSecs: 18,
    stage: async (driver) => {
      driver.setFilter("theme");
      await driver.waitForState({ inputValue: "theme" }, { timeoutMs: 5000 });
      const opened = await driver.batch([{ type: "openActions" }], { timeoutMs: 6000 });
      const initial = await waitForActionsDialog(driver);
      return {
        openActionsSuccess: opened.success !== false,
        initialSearchTextLength: initial.actionsDialog?.search?.textLength ?? null,
      };
    },
    run: async (driver) => {
      await Bun.sleep(1300);
      await progressivelySetActionsDialogInput(driver, ACTIONS_NO_MATCH_QUERY, 78);
      await Bun.sleep(2200);
      const noMatch = await actionsDialogState(driver);
      await progressivelyClearActionsDialogInput(driver, ACTIONS_NO_MATCH_QUERY, 42);
      await Bun.sleep(2200);
      const cleared = await actionsDialogState(driver);
      await keys(driver, ["down", "down", "up", "up"], 320);
      await Bun.sleep(900);
      return {
        target: "ActionsDialog",
        noMatchSearchTextLength: noMatch.actionsDialog?.search?.textLength ?? null,
        noMatchFilteredActionCount: noMatch.actionsDialog?.actions?.filteredCount ?? null,
        clearedSearchTextLength: cleared.actionsDialog?.search?.textLength ?? null,
        clearedFilteredActionCount: cleared.actionsDialog?.actions?.filteredCount ?? null,
      };
    },
  },
  {
    name: "team-review-permissions",
    theme: "synthwave-84",
    effect: "plasma",
    durationSecs: 18,
    run: async (driver) => {
      await Bun.sleep(900);
      await typeText(driver, "permissions", 85);
      await Bun.sleep(650);
      // Take the normal launcher path so the recording proves the same
      // interaction a user performs: select the second permissions command,
      // then open the wizard with Return.
      await keys(driver, ["down"], 250);
      driver.simulateKey("enter");
      await driver.waitForState({ promptType: "permissionsWizard" }, { timeoutMs: 6000 });
      await Bun.sleep(1800);
      await keys(driver, ["down", "down", "up"], 700);
      await Bun.sleep(1800);
      const state = await driver.getState({ timeoutMs: 5000 });
      return {
        builtin: "builtin/setup-permissions",
        opened: state.promptType === "permissionsWizard",
        promptType: state.promptType ?? null,
        selectedIndex: state.selectedIndex ?? null,
        selectedValue: state.selectedValue ?? null,
      };
    },
  },
  {
    name: "team-review-agent-setup",
    theme: "dracula",
    effect: "lavalamp",
    durationSecs: 17,
    // Keep provider CLIs out of discovery. The starter catalog remains, so
    // the empty sandbox deterministically shows install recovery instead of
    // launching a provider or borrowing the user's auth.
    env: {
      PATH: "/usr/bin:/bin",
      SCRIPT_KIT_DISABLE_CODEX_AGENT_CHAT: "1",
    },
    stage: async (driver) => {
      driver.send({ type: "triggerBuiltin", builtinId: "builtin/ai-chat" });
      const setupState = await waitForAgentChatSetup(driver);
      // The restricted PATH intentionally trips the unrelated Bun discovery
      // toast. Dismiss it before recording so the clip stays about recovery.
      const { win } = await windowGeometry();
      await mouseClick({ x: win.x + win.w - 20, y: win.y + 17 }, 150);
      await Bun.sleep(400);
      return {
        providerFree: true,
        setup: setupState.setup,
      };
    },
    run: async (driver, w) => {
      const before = await waitForAgentChatSetup(driver);
      await Bun.sleep(1800);
      await mouseSweep([w.upper, w.lower], 700);
      await Bun.sleep(900);
      driver.simulateKey("enter");
      await Bun.sleep(2200);
      const after = await waitForAgentChatSetup(driver);
      const logs = await driver.getLogs({ contains: "agent_chat_setup_retry_requested" });
      await Bun.sleep(1800);
      return {
        providerFree: true,
        beforeSetup: before.setup,
        afterSetup: after.setup,
        enterPrimaryRetryLogCount: Array.isArray(logs.logs) ? logs.logs.length : null,
      };
    },
  },
  {
    name: "team-review-day-page",
    theme: "rose-pine",
    effect: "starfield",
    durationSecs: 22,
    stage: async (driver, { kitDir }) => {
      const daysDir = join(kitDir, "brain", "days");
      mkdirSync(daysDir, { recursive: true });
      writeFileSync(join(daysDir, `${TEAM_REVIEW_PAST_DATE}.md`), TEAM_REVIEW_PAST_DAY);
      driver.send({
        type: "pushDictationResult",
        transcript: "Team review: unify guidance, recovery, and navigation surfaces",
        target: "today",
      });
      await Bun.sleep(650);
      setClipboard("https://scriptkit.com/review/compact-actions");
      await Bun.sleep(1100);
      setClipboard("https://scriptkit.com/review/recovery-states");
      await Bun.sleep(1300);
      const todayPath = join(daysDir, `${localDateStamp()}.md`);
      const todayText = existsSync(todayPath) ? readFileSync(todayPath, "utf8") : "";
      return {
        todayPath,
        clipboardShelfRefs: (todayText.match(/kit:\/\/clipboard-history\?id=/g) ?? []).length,
        pastDayPath: join(daysDir, `${TEAM_REVIEW_PAST_DATE}.md`),
        pastDaySeeded: existsSync(join(daysDir, `${TEAM_REVIEW_PAST_DATE}.md`)),
      };
    },
    run: async (driver) => {
      await Bun.sleep(800);
      driver.setFilter(" ");
      await driver.waitForState({ promptType: "dayPage" }, { timeoutMs: 7000 });
      await Bun.sleep(1900);

      // The clipboard shelf sits directly above the footer. Use the real
      // cursor so hover/click behavior is represented in the recording.
      const dayGeo = await windowGeometry();
      await mouseClick({
        x: dayGeo.win.x + Math.min(150, dayGeo.win.w * 0.24),
        y: dayGeo.win.y + dayGeo.win.h - 30,
      });
      await Bun.sleep(2400);

      driver.simulateKey("p", ["cmd"]);
      await waitForActionsDialog(driver);
      await setActionsDialogInput(driver, TEAM_REVIEW_PAST_DATE);
      await Bun.sleep(1300);
      driver.simulateKey("enter");
      await driver.waitForState(
        { promptType: "dayPage", inputValue: TEAM_REVIEW_PAST_DAY },
        { timeoutMs: 7000 },
      );
      await Bun.sleep(2600);
      const state = await driver.getState({ timeoutMs: 5000 });
      return {
        promptType: state.promptType ?? null,
        pastDay: TEAM_REVIEW_PAST_DATE,
        inputMatchesPastDay: state.inputValue === TEAM_REVIEW_PAST_DAY,
        visibleWindowRetained: state.windowVisible === true,
      };
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
      env: { HOME: home, SK_PATH: kitDir, ...scenario.env },
    });
    try {
      driver.send({ type: "show" });
      await driver.waitForSettle();
      let stageEvidence: Record<string, any> | void;
      if (scenario.stage) {
        log(`${scenario.name}: staging`);
        stageEvidence = await scenario.stage(driver, { home, kitDir });
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
      const runEvidence = await scenario.run(driver, waypoints);
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
      receipt.videos.push({ name: scenario.name, rect, mov, stageEvidence, runEvidence });
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
