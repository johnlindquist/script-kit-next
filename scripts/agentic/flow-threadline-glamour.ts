// Threadline smoke evals with glamour-video proof.
//
// Three FULL user journeys through the flow launcher, each recorded as a
// video (screencapture -v on the app window rect) AND verified with protocol
// receipts. A scenario only counts as passed when its receipts pass — the
// video is the human-verifiable proof, the receipts are the machine proof.
//
//   converse-live — launcher → "flows" → desk → Enter on Hello Codex →
//                   typed message → REAL `codex app-server` turn (seeded
//                   auth) → honest working → needs-you chip → reply visible.
//   resume-thread — (fake-codex) first turn, ⌘⇧D backgrounds mid-desk,
//                   session row shows live state, re-enter → SAME session,
//                   second turn on the same thread. Esc never kills.
//   stop-honest   — (fake-codex SLOW) streaming turn, ⌘K → search "stop" →
//                   Stop Current Turn → interrupted, conversation survives,
//                   next turn completes.
//
// Mechanics follow glamour-video-probe.ts (keep in sync): window rect from
// tahoe_window_geometry + padding, per-scenario sandbox HOME with startup
// config.ts (theme/effects are startup-only), hide other apps, restore after.
// Requires Screen Recording permission + unlocked interactive session.
//
// Run: bun scripts/agentic/flow-threadline-glamour.ts [scenario ...]
// Masters: .test-screenshots/flow-threadline/<name>.mov

import { join } from "node:path";
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = new URL("../..", import.meta.url).pathname;
const OUT_DIR = join(PROJECT_ROOT, ".test-screenshots/flow-threadline");
const GEO = join(PROJECT_ROOT, "scripts/devtools/bin/tahoe_window_geometry");
const FIXTURE = join(PROJECT_ROOT, "scripts/agentic/fixtures/flow-ux-project");
const PACKAGE_FIXTURE = join(PROJECT_ROOT, "scripts/agentic/fixtures/flow-desk-package");
const BINARY = join(PROJECT_ROOT, "target-agent/artifacts/flow-ux/script-kit-gpui");
mkdirSync(OUT_DIR, { recursive: true });

const PAD_X = 150;
const PAD_TOP = 120;
const PAD_BOTTOM = 240;

type Json = Record<string, any>;

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

async function windowGeometry() {
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
  console.error(`[threadline ${new Date().toISOString().slice(11, 19)}] ${step}`);
}

const flowUx = (s: Json) => (s?.flowUx as Json) ?? null;
const lastSession = (s: Json) => ((flowUx(s)?.sessions as Json[]) ?? []).at(-1);

type Scene = {
  driver: Driver;
};

async function pollState(
  driver: Driver,
  pred: (s: Json) => boolean,
  timeoutMs = 10_000,
): Promise<Json> {
  const deadline = Date.now() + timeoutMs;
  let state: Json = {};
  while (Date.now() < deadline) {
    state = (await driver.getState()) as Json;
    if (pred(state)) return state;
    await Bun.sleep(150);
  }
  return state;
}

function pressMain(driver: Driver, key: string, modifiers: string[] = []) {
  return driver
    .simulateGpuiEvent(
      { type: "keyDown", key, modifiers },
      { target: { type: "main" }, timeoutMs: 5_000 },
    )
    .catch((e: unknown) => ({ error: String(e) }));
}

/** Progressive typing into the MAIN filter (desk search). */
async function typeFilter(driver: Driver, text: string, perKeyMs = 90) {
  for (let i = 1; i <= text.length; i++) {
    driver.setFilter(text.slice(0, i));
    await Bun.sleep(perKeyMs);
  }
}

/** Progressive typing into the Threadline composer (setInput routes to the chat). */
async function typeComposer(driver: Driver, text: string, perKeyMs = 65) {
  for (let i = 1; i <= text.length; i++) {
    driver.send({
      type: "batch",
      requestId: `type-${Date.now()}-${i}`,
      commands: [{ type: "setInput", text: text.slice(0, i) }],
    });
    await Bun.sleep(perKeyMs);
  }
}

async function openDesk(driver: Driver) {
  await typeFilter(driver, "flows");
  await Bun.sleep(500);
  await pressMain(driver, "enter");
  await pollState(driver, (s) => flowUx(s)?.activeVariant === "flash");
  await Bun.sleep(700);
}

async function openHelloCodexSession(driver: Driver) {
  await typeFilter(driver, "hello codex", 80);
  await Bun.sleep(500);
  await pressMain(driver, "enter");
  return pollState(driver, (s) => s.promptType === "flowSession");
}

async function sendComposerMessage(driver: Driver, text: string) {
  await typeComposer(driver, text);
  await Bun.sleep(450);
  await pressMain(driver, "enter");
}

async function visibleTexts(driver: Driver): Promise<string> {
  const r = (await driver.getElements({ limit: 200 })) as Json;
  return ((r.elements as Json[]) ?? [])
    .map((e) => `${e.text ?? ""}|${e.value ?? ""}`)
    .join("\n");
}

type Scenario = {
  name: string;
  theme: string;
  effect: string;
  durationSecs: number;
  /** true → no SCRIPT_KIT_CODEX_BIN override and real auth seeded into HOME */
  live: boolean;
  run: (driver: Driver) => Promise<Json>;
};

const SCENARIOS: Scenario[] = [
  {
    name: "converse-live",
    theme: "synthwave-84",
    effect: "starfield",
    durationSecs: 55,
    live: true,
    run: async (driver) => {
      await Bun.sleep(1200);
      await openDesk(driver);
      const opened = await openHelloCodexSession(driver);
      await Bun.sleep(800);
      await sendComposerMessage(
        driver,
        "In one short sentence: what makes a launcher feel alive?",
      );
      const working = await pollState(
        driver,
        (s) => lastSession(s)?.turnInFlight === true,
        10_000,
      );
      const done = await pollState(driver, (s) => lastSession(s)?.turns === 1, 90_000);
      await Bun.sleep(3500); // linger on the reply for the recording
      return {
        promptTypeAfterEnter: opened.promptType,
        transport: lastSession(opened)?.transport,
        workingState: lastSession(working)?.state,
        finalState: lastSession(done)?.state,
        turns: lastSession(done)?.turns,
        pass:
          opened.promptType === "flowSession" &&
          lastSession(opened)?.transport === "codexThread" &&
          lastSession(working)?.state === "working" &&
          lastSession(done)?.turns === 1 &&
          lastSession(done)?.state === "needs you",
      };
    },
  },
  {
    name: "resume-thread",
    theme: "rose-pine",
    effect: "starfield",
    durationSecs: 45,
    live: false,
    run: async (driver) => {
      await Bun.sleep(1000);
      await openDesk(driver);
      const opened = await openHelloCodexSession(driver);
      const transport = lastSession(opened)?.transport;
      await Bun.sleep(600);
      await sendComposerMessage(driver, "First turn: remember the number 41.");
      let state = await pollState(driver, (s) => lastSession(s)?.turns === 1, 15_000);
      const firstSessionId = lastSession(state)?.sessionId;
      await Bun.sleep(1800);

      // Background: Esc-family gesture — the conversation must survive.
      await pressMain(driver, "d", ["cmd", "shift"]);
      state = await pollState(driver, (s) => flowUx(s)?.activeVariant === "flash", 6_000);
      const deskSessions = (flowUx(state)?.sessions as Json[]) ?? [];
      await Bun.sleep(2500); // show the live session row on the desk

      // Re-enter the SAME session from the desk row.
      await pressMain(driver, "enter");
      state = await pollState(driver, (s) => s.promptType === "flowSession", 6_000);
      const resumedId = lastSession(state)?.sessionId;
      await Bun.sleep(1000);
      await sendComposerMessage(driver, "Second turn: what number did I ask you to remember?");
      state = await pollState(driver, (s) => lastSession(s)?.turns === 2, 15_000);
      await Bun.sleep(2500);
      return {
        transport,
        firstSessionId,
        deskSessionCount: deskSessions.length,
        resumedId,
        turns: lastSession(state)?.turns,
        finalState: lastSession(state)?.state,
        pass:
          transport === "codexThread" &&
          typeof firstSessionId === "number" &&
          firstSessionId === resumedId &&
          deskSessions.length === 1 &&
          lastSession(state)?.turns === 2 &&
          lastSession(state)?.state === "needs you",
      };
    },
  },
  {
    name: "stop-honest",
    theme: "dracula",
    effect: "plasma",
    durationSecs: 45,
    live: false,
    run: async (driver) => {
      await Bun.sleep(1000);
      await openDesk(driver);
      const opened = await openHelloCodexSession(driver);
      const transport = lastSession(opened)?.transport;
      await Bun.sleep(600);
      // SLOW triggers fake-codex word-by-word streaming (~12s window).
      await sendComposerMessage(driver, "Take it SLOW and explain everything.");
      let state = await pollState(driver, (s) => lastSession(s)?.turnInFlight === true, 8_000);
      const workingState = lastSession(state)?.state;
      await Bun.sleep(2500); // let words visibly stream

      // ⌘K → search "stop" (visible in the recording) → fire the action by
      // id (reliable activation), then toggle the popup closed.
      await pressMain(driver, "k", ["cmd"]);
      await Bun.sleep(900);
      for (let i = 1; i <= 4; i++) {
        await driver
          .request(
            {
              type: "batch",
              target: { type: "kind", kind: "actionsDialog" },
              commands: [{ type: "setInput", text: "stop".slice(0, i) }],
              options: { stopOnError: true, rollbackOnError: false, timeout: 5000 },
            },
            { expect: "batchResult", timeoutMs: 6000 },
          )
          .catch(() => null);
        await Bun.sleep(180);
      }
      await Bun.sleep(600);
      driver.send({ type: "triggerAction", actionId: "flow_desk_session_stop" });
      await Bun.sleep(400);
      await pressMain(driver, "k", ["cmd"]); // toggle the popup closed
      state = await pollState(
        driver,
        (s) => lastSession(s)?.turnInFlight === false,
        10_000,
      );
      const stoppedState = lastSession(state)?.state;
      const transcriptAfterStop = await visibleTexts(driver);
      await Bun.sleep(1800);

      // The conversation survives: a quick follow-up turn completes.
      await sendComposerMessage(driver, "Still with me? One quick reply.");
      state = await pollState(driver, (s) => lastSession(s)?.turns === 2, 15_000);
      await Bun.sleep(2200);
      return {
        transport,
        workingState,
        stoppedState,
        transcriptSurvived: transcriptAfterStop.includes("SLOW"),
        turns: lastSession(state)?.turns,
        finalState: lastSession(state)?.state,
        pass:
          transport === "codexThread" &&
          workingState === "working" &&
          stoppedState === "needs you" &&
          transcriptAfterStop.includes("SLOW") &&
          lastSession(state)?.turns === 2 &&
          lastSession(state)?.state === "needs you",
      };
    },
  },
];

const requested = process.argv.slice(2);
const toRun = requested.length
  ? SCENARIOS.filter((s) => requested.includes(s.name))
  : SCENARIOS;
if (requested.length && toRun.length !== requested.length) {
  console.error(
    `unknown scenario in [${requested.join(", ")}]; known: ${SCENARIOS.map((s) => s.name).join(", ")}`,
  );
  process.exit(1);
}

const receipt: Json = { evals: [], errors: [] };
const hidden = hideOtherApps();
receipt.hiddenApps = hidden.length;

try {
  for (const scenario of toRun) {
    const home = `/tmp/sk-threadline-home-${scenario.name}`;
    rmSync(home, { recursive: true, force: true });
    const kitDir = join(home, ".scriptkit");
    mkdirSync(kitDir, { recursive: true });
    writeFileSync(
      join(kitDir, "config.ts"),
      `export default {\n  theme: { presetId: "${scenario.theme}" },\n  effects: { background: "${scenario.effect}", intensity: 0.9 },\n};\n`,
    );
    if (scenario.live) {
      const seeded = Bun.spawnSync([
        "bash",
        join(PROJECT_ROOT, "scripts/agentic/seed-sandbox-home.sh"),
        home,
      ]);
      if (seeded.exitCode !== 0) {
        throw new Error(`seed-sandbox-home failed: ${seeded.stderr.toString()}`);
      }
      // Flow tests run on the cheap/fast tier: gpt-5.6-luna at low effort.
      // Strip any seeded model pins, then append ours (TOML forbids dup keys).
      const cfgPath = join(home, ".codex/config.toml");
      const seededCfg = (await Bun.file(cfgPath).text().catch(() => "")) ?? "";
      const filtered = seededCfg
        .split("\n")
        .filter((l) => !/^\s*model(_reasoning_effort)?\s*=/.test(l))
        .join("\n");
      writeFileSync(
        cfgPath,
        `${filtered.trimEnd()}\nmodel = "gpt-5.6-luna"\nmodel_reasoning_effort = "low"\n`,
      );
    }

    const env: Record<string, string> = {
      HOME: home,
      SK_PATH: kitDir,
      SCRIPT_KIT_FLOW_UX_CWD: FIXTURE,
      SCRIPT_KIT_FLOWS_PACKAGE_DIR: PACKAGE_FIXTURE,
      SCRIPT_KIT_FLOWS_BIN_DIR: join(PACKAGE_FIXTURE, "bin"),
    };
    if (!scenario.live) {
      env.SCRIPT_KIT_CODEX_BIN = join(PACKAGE_FIXTURE, "bin/fake-codex");
    }

    const driver = await Driver.launch({
      binary: BINARY,
      sessionName: `threadline-${scenario.name}`,
      env,
    });
    try {
      driver.send({ type: "show" });
      await driver.waitForSettle();
      const { win: f, display } = await windowGeometry();
      const x0 = Math.max(display.x, Math.round(f.x - PAD_X));
      const y0 = Math.max(display.y, Math.round(f.y - PAD_TOP));
      const x1 = Math.min(display.x + display.w, Math.round(f.x + f.w + PAD_X));
      const y1 = Math.min(display.y + display.h, Math.round(f.y + f.h + PAD_BOTTOM));
      const rect = { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };

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
      const evidence = await scenario.run(driver);
      log(`${scenario.name}: interaction done (pass=${evidence.pass}), waiting for recorder`);
      const recResult = await Promise.race([
        rec.exited,
        Bun.sleep((scenario.durationSecs + 20) * 1000).then(() => "timeout" as const),
      ]);
      if (recResult === "timeout") {
        rec.kill();
        throw new Error("screencapture did not exit; killed");
      }
      if (!(await Bun.file(mov).exists())) {
        throw new Error(`screencapture exited (${recResult}) but wrote no file`);
      }
      receipt.evals.push({ name: scenario.name, mov, rect, evidence });
    } catch (err: any) {
      receipt.errors.push({ name: scenario.name, error: String(err?.message ?? err) });
    } finally {
      const closed = await Promise.race([
        driver.close().then(() => true),
        Bun.sleep(15000).then(() => false),
      ]);
      if (!closed) {
        log(`${scenario.name}: driver.close timed out; force-killing app`);
        Bun.spawnSync(["pkill", "-f", "artifacts/flow-ux/script-kit-gpui"]);
      }
    }
  }
} finally {
  restoreApps(hidden);
}

receipt.summary = {
  passed: receipt.evals.filter((e: Json) => e.evidence?.pass === true).length,
  total: toRun.length,
  errors: receipt.errors.length,
};
console.log(JSON.stringify(receipt, null, 2));
writeFileSync(join(OUT_DIR, "receipt.json"), JSON.stringify(receipt, null, 2));
if (receipt.errors.length > 0 || receipt.summary.passed !== toRun.length) process.exit(1);
