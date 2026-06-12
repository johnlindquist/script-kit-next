#!/usr/bin/env bun
/**
 * Runtime proof for the Today → Agent Chat handoff scope:
 * with multi-line day content and the cursor on the last line, the
 * "Send Line to Agent Chat" Cmd+K action opens Agent Chat carrying ONLY the
 * current line (not the whole day note).
 */
import { Driver, type Json } from "../devtools/driver";

const BINARY =
  process.env.PROBE_BINARY ?? "target-agent/artifacts/today/script-kit-gpui";

const receipts: Record<string, Json> = {};
const failures: string[] = [];
const runId = `${Date.now()}-${Math.random().toString(36).slice(2)}`;

function check(name: string, ok: boolean, detail: Json = {}) {
  receipts[name] = { ok, ...detail };
  if (!ok) failures.push(name);
}

async function gesture(driver: Driver, phase: "down" | "up", label: string) {
  return driver.request(
    { type: "simulateMainHotkeyGesture", phase, requestId: `${runId}-${label}` },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  );
}

async function tapHotkey(driver: Driver, label: string) {
  await gesture(driver, "down", `${label}-down`);
  await Bun.sleep(30);
  await gesture(driver, "up", `${label}-up`);
  await Bun.sleep(400);
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-handoff",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

// The handoff submits into a live Agent Chat thread; without Pi auth the
// sandbox opens Agent Chat in setup mode and refuses the submit. Seed ONLY
// the small auth files into the sandbox HOME post-launch, pre-handoff
// (never `cp -R ~/.codex` — that directory is tens of GB of session logs).
const sandboxHome = `${driver.sessionDir}/home`;
const realHome = process.env.HOME ?? "";
for (const rel of [
  ".codex/auth.json",
  ".pi/agent/auth.json",
  ".pi/agent/settings.json",
]) {
  const src = `${realHome}/${rel}`;
  const dest = `${sandboxHome}/${rel}`;
  try {
    await Bun.$`mkdir -p ${dest.slice(0, dest.lastIndexOf("/"))} && cp ${src} ${dest}`.quiet();
  } catch {
    // Missing auth file — the probe will report setup mode honestly below.
  }
}

try {
  await tapHotkey(driver, "show");
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 8000 });
  await Bun.sleep(400);
  let state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  if (state.promptType !== "dayPage") {
    await tapHotkey(driver, "toggle-day-page");
    state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  }
  check("opened_day_page", state.promptType === "dayPage", {
    promptType: state.promptType,
  });

  // Multi-line content; setInput leaves the cursor at the end (line 2).
  // The two lines have DIFFERENT lengths so the app-log submit receipt
  // (`intent_len`) can prove which text was handed off.
  const lineOne = "context line one stays in the note";
  const lineTwo = "hand exactly this line to agent chat today please";
  const batch = (await driver.batch(
    [{ type: "setInput", text: `${lineOne}\n${lineTwo}` }],
    { timeoutMs: 5000 },
  )) as Json;
  check("seeded_two_lines", batch.success === true, { batch });
  await Bun.sleep(300);

  // Cmd+K → filter to the handoff row → Enter executes it. The startup Pi
  // warm can race the auth seeding and leave a cached setup-mode failure on
  // the first open, so retry once from the Day Page.
  let afterState: Json = {};
  let haystack = "";
  const attempts: Json[] = [];
  for (let attempt = 0; attempt < 2; attempt++) {
    // Make sure we're on the Day Page with the seeded content before Cmd+K.
    let pre = (await driver.getState({ timeoutMs: 5000 })) as Json;
    for (let nav = 0; nav < 4 && pre.promptType !== "dayPage"; nav++) {
      if (pre.promptType === "agentChatChat") {
        await driver.simulateKey("escape");
      } else {
        await tapHotkey(driver, `nav-${attempt}-${nav}`);
      }
      await Bun.sleep(500);
      pre = (await driver.getState({ timeoutMs: 5000 })) as Json;
    }
    if (pre.promptType !== "dayPage") {
      attempts.push({ attempt, error: "could not navigate back to dayPage", pre });
      break;
    }
    await driver.batch(
      [{ type: "setInput", text: `${lineOne}\n${lineTwo}` }],
      { timeoutMs: 5000 },
    );
    await Bun.sleep(300);

    await driver.simulateKey("k", ["cmd"]);
    await Bun.sleep(900);
    for (const ch of "send line") {
      await driver.simulateKey(ch === " " ? "space" : ch);
      await Bun.sleep(40);
    }
    await Bun.sleep(400);
    await driver.simulateKey("enter");
    await Bun.sleep(2500);

    afterState = (await driver.getState({ timeoutMs: 5000 })) as Json;
    haystack = await Bun.file(`${driver.sessionDir}/app.log`).text();
    const submitted = haystack.includes(
      "event=agent_chat_reused_entry_intent_with_host_context_submitted source=day_page_line_handoff",
    );
    attempts.push({
      attempt,
      promptType: afterState.promptType,
      submitted,
    });
    if (afterState.promptType === "agentChatChat" && submitted) {
      break;
    }
    // Setup-mode landing: escape back and retry once.
    await driver.simulateKey("escape");
    await Bun.sleep(800);
  }
  receipts.attempts = attempts;
  check("handoff_opens_agent_chat", afterState.promptType === "agentChatChat", {
    promptType: afterState.promptType,
  });

  // Scope check via the runtime submit receipt: intent_len must equal the
  // current line's length — not line one's, not the whole note's.
  const submitLine = haystack
    .split("\n")
    .find((l) =>
      l.includes(
        "event=agent_chat_reused_entry_intent_with_host_context_submitted source=day_page_line_handoff",
      ),
    );
  const intentLen = Number(/intent_len=(\d+)/.exec(submitLine ?? "")?.[1] ?? -1);
  check(
    "handoff_carries_current_line_only",
    intentLen === lineTwo.length,
    {
      intentLen,
      lineTwoLen: lineTwo.length,
      lineOneLen: lineOne.length,
      fullNoteLen: `${lineOne}\n${lineTwo}`.length,
      submitLine: submitLine ?? null,
    },
  );

  const pass = failures.length === 0;
  console.log(
    JSON.stringify({ pass, failures, sessionDir: driver.sessionDir, receipts }, null, 2),
  );
  if (!pass) process.exitCode = 1;
} finally {
  await driver.close();
}
