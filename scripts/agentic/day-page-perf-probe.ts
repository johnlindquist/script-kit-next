#!/usr/bin/env bun
/**
 * Machine-readable receipts for the Today feature performance budgets
 * (.notes/today-requirements.md):
 * - main hotkey open/close: < 100ms (wall-clock including protocol overhead)
 * - hold-from-closed Today entry: < 100ms after the hold threshold fires
 * - typing latency: < 16ms average per applied edit (in-app setInput elapsed)
 * - actions readiness: < 250ms from Cmd+K to day_page rows queryable
 *
 * Wall-clock numbers include protocol/process overhead and are reported for
 * context; budget assertions use the tightest measurable signal per row.
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

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Json;
  if (typeof json.semanticId === "string" || typeof json.id === "string") {
    out.push(json);
  }
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

async function gesture(driver: Driver, phase: "down" | "up", label: string) {
  return driver.request(
    { type: "simulateMainHotkeyGesture", phase, requestId: `${runId}-${label}` },
    { expect: "externalCommandResult", timeoutMs: 5000 },
  );
}

/**
 * Tap the main hotkey, then poll until visibility changes. Returns wall-clock
 * ms including protocol overhead (an upper bound on user-perceived latency).
 */
async function tapAndMeasure(driver: Driver, label: string) {
  const before = (await driver.getState({ timeoutMs: 5000 })) as Json;
  const key = (s: Json) => `${s.promptType}|${s.windowVisible}`;
  const beforeKey = key(before);
  const wallStart = performance.now();
  await gesture(driver, "down", `${label}-down`);
  await gesture(driver, "up", `${label}-up`);
  let after = before;
  let wallMs = -1;
  for (let attempt = 0; attempt < 200; attempt++) {
    after = (await driver.getState({ timeoutMs: 2000 })) as Json;
    if (key(after) !== beforeKey) {
      wallMs = Math.round(performance.now() - wallStart);
      break;
    }
  }
  return {
    ok: wallMs >= 0,
    wallMs,
    from: beforeKey,
    to: key(after),
  };
}

async function holdDayPageAndMeasure(driver: Driver, label: string) {
  const wallStart = performance.now();
  await gesture(driver, "down", `${label}-down`);
  await Bun.sleep(260);
  let after = (await driver.getState({ timeoutMs: 5000 })) as Json;
  let wallMs = -1;
  for (let attempt = 0; attempt < 100; attempt++) {
    after = (await driver.getState({ timeoutMs: 2000 })) as Json;
    if (after.windowVisible === true && after.promptType === "dayPage") {
      wallMs = Math.round(performance.now() - wallStart - 250);
      break;
    }
    await Bun.sleep(5);
  }
  await gesture(driver, "up", `${label}-up`);
  await Bun.sleep(250);
  return {
    ok: wallMs >= 0,
    wallMs,
    to: `${after.promptType}|${after.windowVisible}`,
  };
}

const driver = await Driver.launch({
  binary: BINARY,
  sandboxHome: true,
  sessionName: "day-page-perf",
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  // --- Show window (first tap shows the launcher) ---
  await gesture(driver, "down", "show-down");
  await Bun.sleep(30);
  await gesture(driver, "up", "show-up");
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 8000 });
  await Bun.sleep(500);
  let state = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("window_shown", state.windowVisible === true, {
    promptType: state.promptType,
  });

  // --- Toggle budget: tap through the normal hotkey cycle (shown → hidden →
  // shown → …) and assert transitions land under 100ms wall-clock.
  const toggleSamples: Json[] = [];
  for (let i = 0; i < 8; i++) {
    const sample = await tapAndMeasure(driver, `toggle-${i}`);
    toggleSamples.push(sample);
    await Bun.sleep(250);
  }
  const toggleMax = Math.max(...toggleSamples.map((s) => s.wallMs as number));
  check(
    "main_hotkey_open_close_budget_under_100ms",
    toggleSamples.every((s) => s.ok) && toggleMax >= 0 && toggleMax < 100,
    { budgetMs: 100, maxWallMs: toggleMax, samples: toggleSamples },
  );

  // End hidden, then hold from closed to enter Day Page for the remaining
  // measurements.
  let settled = (await driver.getState({ timeoutMs: 5000 })) as Json;
  if (settled.windowVisible === true) {
    await tapAndMeasure(driver, "settle-hidden");
    await Bun.sleep(250);
    settled = (await driver.getState({ timeoutMs: 5000 })) as Json;
  }
  check("settled_hidden_before_day_page_hold", settled.windowVisible === false, {
    promptType: settled.promptType,
    windowVisible: settled.windowVisible,
  });

  const holdSample = await holdDayPageAndMeasure(driver, "hold-to-day-page");
  settled = (await driver.getState({ timeoutMs: 5000 })) as Json;
  check("settled_on_day_page", settled.promptType === "dayPage", {
    promptType: settled.promptType,
  });
  check(
    "hold_day_page_budget_under_100ms_after_threshold",
    holdSample.ok === true &&
      (holdSample.wallMs as number) >= 0 &&
      (holdSample.wallMs as number) < 100,
    { budgetMs: 100, sample: holdSample },
  );

  // --- Typing budget: avg in-app apply latency < 16ms across 20 edits ---
  const base = "perf typing latency sample text";
  const typingElapsed: number[] = [];
  for (let i = 1; i <= 20; i++) {
    const text = base.slice(0, i);
    const batch = (await driver.batch(
      [{ type: "setInput", text }],
      { timeoutMs: 5000 },
    )) as Json;
    const results = (batch.results ?? []) as Json[];
    const elapsed = (results[0]?.elapsed as number | undefined) ?? -1;
    if (batch.success !== true || elapsed < 0) {
      check(`typing_edit_${i}_applied`, false, { batch });
      break;
    }
    typingElapsed.push(elapsed);
  }
  const typingAvg =
    typingElapsed.length > 0
      ? typingElapsed.reduce((a, b) => a + b, 0) / typingElapsed.length
      : -1;
  const typingMax = typingElapsed.length > 0 ? Math.max(...typingElapsed) : -1;
  check(
    "typing_budget_avg_under_16ms",
    typingElapsed.length === 20 && typingAvg >= 0 && typingAvg < 16,
    {
      budgetMs: 16,
      avgMs: Math.round(typingAvg * 100) / 100,
      maxMs: typingMax,
      samples: typingElapsed,
    },
  );

  // --- Actions readiness: Cmd+K to day_page rows queryable < 250ms ---
  const actionsStart = performance.now();
  await driver.simulateKey("k", ["cmd"]);
  let actionsReadyMs = -1;
  for (let attempt = 0; attempt < 40; attempt++) {
    const elements = (await driver.getElements(
      { target: { type: "kind", kind: "actionsDialog" }, limit: 200 },
      { timeoutMs: 2000 },
    )) as Json;
    const flat = walkElements(elements);
    const hasTodayRow = flat.some((el) =>
      [el.semanticId, el.id, el.text, el.value].some(
        (v) =>
          typeof v === "string" &&
          (v.includes("day_page") || v.includes("Open Note")),
      ),
    );
    if (hasTodayRow) {
      actionsReadyMs = Math.round(performance.now() - actionsStart);
      break;
    }
    await Bun.sleep(10);
  }
  check(
    "actions_budget_under_250ms",
    actionsReadyMs >= 0 && actionsReadyMs < 250,
    { budgetMs: 250, wallMsIncludingProtocol: actionsReadyMs },
  );
  await driver.simulateKey("escape");
  await Bun.sleep(200);

  const pass = failures.length === 0;
  console.log(
    JSON.stringify({ pass, failures, sessionDir: driver.sessionDir, receipts }, null, 2),
  );
  if (!pass) process.exitCode = 1;
} finally {
  await driver.close();
}
