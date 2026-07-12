#!/usr/bin/env bun
/**
 * Runtime proof for the launcher's top-fade and phased boundary affordance.
 *
 * Build the named artifact first:
 *   SCRIPT_KIT_AGENT_ARTIFACT_NAME=launcher-scroll-affordance \
 *     ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui
 *
 * Override that pinned artifact with PROBE_BINARY or --binary when needed.
 */
import {
  existsSync,
  mkdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, relative, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver.ts";
import { inspectMainListScrollAffordance } from "../devtools/scroll.ts";

const repoRoot = resolve(import.meta.dir, "../..");
const outputDir = join(repoRoot, ".test-output", "launcher-scroll-affordance");
const receiptPath = join(outputDir, "receipt.json");
const screenshotPath = join(outputDir, "middle-top-fade.png");
const sessionDir = join(
  "/tmp",
  `sk-launcher-scroll-affordance-${process.pid}-${Date.now().toString(36)}`,
);
const query = "launcher-scroll-affordance";
// A trailing sentinel lets the debounced root-search frame observe the full
// query even though launcher input handoff clears the visible field afterward.
const filterInput = `${query} `;
const target = { type: "main" };
const pollMs = 20;
const defaultBinary = "target-agent/artifacts/launcher-scroll-affordance/script-kit-gpui";

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1]
    ? process.argv[index + 1]
    : fallback;
}

const binary = resolve(
  repoRoot,
  argValue(
    "--binary",
    process.env.PROBE_BINARY
      ?? process.env.SCRIPT_KIT_GPUI_BINARY
      ?? defaultBinary,
  ),
);
const timeoutMs = Number(argValue("--timeout", "10000"));

type Check = {
  name: string;
  pass: boolean;
  observed?: Json | string | number | boolean | null;
};

const checks: Check[] = [];
const failures: string[] = [];
const scenarios: Json = {};
let driver: Driver | null = null;
let screenshotResult: Json = { classification: "not-attempted" };

function check(
  name: string,
  pass: boolean,
  observed?: Json | string | number | boolean | null,
): boolean {
  checks.push(observed === undefined ? { name, pass } : { name, pass, observed });
  if (!pass) failures.push(name);
  return pass;
}

function finite(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

function affordanceOf(state: Json): Json {
  const scroll = state.mainListScroll;
  if (!scroll || typeof scroll !== "object") {
    throw new Error("getState omitted mainListScroll on ScriptList");
  }
  const inspection = inspectMainListScrollAffordance(scroll, true);
  if (!inspection.complete || inspection.affordance == null) {
    throw new Error(
      `mainListScroll.affordance incomplete: ${inspection.missingFields.join(", ")}`,
    );
  }
  return inspection.affordance as Json;
}

function compactState(state: Json): Json {
  const scroll = state.mainListScroll ?? {};
  const affordance = affordanceOf(state);
  return {
    inputValue: state.inputValue ?? null,
    surfaceKind: state.surfaceContract?.surfaceKind ?? null,
    visibleChoiceCount: state.visibleChoiceCount ?? null,
    rootFileQuery: state.rootFileSearch?.query ?? null,
    rootFileLoading: state.rootFileSearch?.loading ?? null,
    selectedIndex: scroll.selectedIndex ?? state.selectedIndex ?? null,
    itemCount: scroll.itemCount ?? state.visibleChoiceCount ?? null,
    scrollTop: scroll.scrollTop ?? null,
    scrollTopItem: scroll.scrollTopItem ?? null,
    scrollTopOffset: scroll.scrollTopOffset ?? null,
    maxScrollTop: scroll.maxScrollTop ?? null,
    atTop: affordance.atTop,
    atBottom: affordance.atBottom,
    topFadeActive: affordance.topFadeActive,
    topFadeProgress: affordance.topFadeProgress,
    topFadeAlpha: affordance.topFadeAlpha,
    overscrollOffsetPx: affordance.overscrollOffsetPx,
    overscrollMaxOffsetPx: affordance.overscrollMaxOffsetPx,
    overscrollEdge: affordance.overscrollEdge,
    overscrollPhase: affordance.overscrollPhase,
    generation: affordance.generation,
    lastTouchPhase: affordance.lastTouchPhase,
    lastSettleReason: affordance.lastSettleReason,
    reducedMotion: affordance.reducedMotion,
  };
}

function safeCompactState(state: Json): Json {
  try {
    return compactState(state);
  } catch {
    return {
      promptType: state.promptType ?? null,
      surfaceKind: state.surfaceContract?.surfaceKind ?? null,
      inputValue: state.inputValue ?? null,
      hasMainListScroll: state.mainListScroll != null,
      hasAffordance: state.mainListScroll?.affordance != null,
    };
  }
}

function sameLogicalTuple(before: Json, after: Json): boolean {
  const a = before.mainListScroll ?? {};
  const b = after.mainListScroll ?? {};
  return a.selectedIndex === b.selectedIndex
    && a.scrollTopItem === b.scrollTopItem
    && a.scrollTopOffset === b.scrollTopOffset
    && a.scrollTop === b.scrollTop;
}

async function pollState(
  label: string,
  predicate: (state: Json) => boolean,
  deadlineMs = timeoutMs,
): Promise<Json> {
  const deadline = performance.now() + deadlineMs;
  let last: Json = {};
  while (performance.now() < deadline) {
    last = await driver!.getState({ timeoutMs: Math.min(deadlineMs, 5_000) });
    if (predicate(last)) return last;
    await Bun.sleep(pollMs);
  }
  throw new Error(`${label}: timed out; last=${JSON.stringify(safeCompactState(last))}`);
}

async function waitForQuietPopulation(label: string): Promise<Json> {
  const startedAt = performance.now();
  const deadline = startedAt + Math.max(timeoutMs, 8_000);
  let lastSignature = "";
  let unchangedSince = startedAt;
  let last: Json = {};
  while (performance.now() < deadline) {
    last = await driver!.getState({ timeoutMs: 5_000 });
    const scroll = last.mainListScroll ?? {};
    const signature = JSON.stringify({
      itemCount: scroll.itemCount,
      contentHeight: scroll.contentHeight,
      maxScrollTop: scroll.maxScrollTop,
      visibleChoiceCount: last.visibleChoiceCount,
      rootFileQuery: last.rootFileSearch?.query,
      rootFileLoading: last.rootFileSearch?.loading,
    });
    if (signature !== lastSignature) {
      lastSignature = signature;
      unchangedSince = performance.now();
    }
    if (performance.now() - startedAt >= 3_000
      && performance.now() - unchangedSince >= 800
      && last.rootFileSearch?.query === query
      && last.rootFileSearch?.loading === false
      && Number(scroll.maxScrollTop) > 0) {
      return last;
    }
    await Bun.sleep(50);
  }
  throw new Error(`${label}: population did not settle; last=${JSON.stringify(safeCompactState(last))}`);
}

async function gpuiKey(key: string): Promise<Json> {
  const response = await driver!.simulateGpuiEvent(
    { type: "keyDown", key, modifiers: [] },
    { target, timeoutMs: 5_000 },
  );
  if (response.success !== true) {
    throw new Error(`${key} dispatch failed: ${JSON.stringify(response)}`);
  }
  return response;
}

async function wheel(
  point: { x: number; y: number },
  phase: "started" | "moved" | "ended",
  deltaY: number,
): Promise<Json> {
  const response = await driver!.simulateGpuiScrollWheel(
    { x: point.x, y: point.y, deltaX: 0, deltaY, phase },
    { target, timeoutMs: 5_000 },
  );
  if (response.success !== true) {
    throw new Error(
      `scrollWheel ${phase} dispatch failed: ${JSON.stringify(response)}`,
    );
  }
  return response;
}

async function selectableEndpoint(): Promise<Json> {
  const result = await driver!.getElements(
    { target, limit: 2_000, includeHeaders: true },
    { timeoutMs: 5_000 },
  );
  const rows = (Array.isArray(result.elements) ? result.elements : [])
    .filter((element: Json) => element.role === "row" && element.selectable === true)
    .sort((a: Json, b: Json) => Number(a.index) - Number(b.index));
  const selected = rows.find((row: Json) => row.selected === true) ?? null;
  return {
    count: rows.length,
    firstIndex: rows[0]?.index ?? null,
    lastIndex: rows.at(-1)?.index ?? null,
    selectedIndex: selected?.index ?? null,
    selectedSemanticId: selected?.semanticId ?? null,
  };
}

async function establishTop(): Promise<{ state: Json; endpoint: Json }> {
  await gpuiKey("home");
  const atTop = (candidate: Json) => {
    try {
      const affordance = affordanceOf(candidate);
      const scroll = candidate.mainListScroll ?? {};
      return Number(scroll.itemCount) >= 12
        && Number(scroll.maxScrollTop) > 0
        && affordance.atTop === true
        && affordance.overscrollPhase === "idle"
        && Number(affordance.overscrollOffsetPx) === 0;
    } catch {
      return false;
    }
  };
  let state: Json;
  try {
    state = await pollState("top endpoint", atTop, 1_200);
  } catch {
    // Home selects the first row but, after a deep list, selection reveal may
    // leave a leading section header just above the viewport. Replacing the
    // deterministic filter is the public reset seam that restores true zero.
    await driver!.setFilterAndWait("", { timeoutMs });
    await driver!.setFilterAndWait(filterInput, { timeoutMs });
    state = await pollState("top endpoint after filter reset", atTop);
  }
  await gpuiKey("up");
  const confirmed = await pollState("top endpoint confirmation", (candidate) => {
    try {
      return affordanceOf(candidate).atTop === true;
    } catch {
      return false;
    }
  });
  const endpoint = await selectableEndpoint();
  check("top-selection-is-first-selectable", endpoint.selectedIndex === endpoint.firstIndex, endpoint);
  check("top-endpoint-confirmation-does-not-move", sameLogicalTuple(state, confirmed), {
    before: compactState(state),
    after: compactState(confirmed),
  });
  return { state: confirmed, endpoint };
}

async function establishBottom(): Promise<{ state: Json; endpoint: Json }> {
  let state: Json = {};
  let confirmed: Json = {};
  let endpoint: Json = {};
  let quietEndpoint = false;
  for (let attempt = 0; attempt < 32; attempt += 1) {
    await gpuiKey("end");
    state = await pollState("bottom endpoint", (candidate) => {
      try {
        const affordance = affordanceOf(candidate);
        return affordance.atBottom === true
          && affordance.overscrollPhase === "idle";
      } catch {
        return false;
      }
    });
    await gpuiKey("down");
    confirmed = await pollState("bottom endpoint confirmation", (candidate) => {
      try {
        return affordanceOf(candidate).atBottom === true;
      } catch {
        return false;
      }
    });

    // End can deliberately trigger lazy source hydration/pagination. Require
    // a quiet endpoint before testing rebound so a legitimate list replacement
    // cannot masquerade as a boundary-state failure.
    quietEndpoint = true;
    let quietState = confirmed;
    for (let sample = 0; sample < 8; sample += 1) {
      await Bun.sleep(100);
      const candidate = await driver!.getState();
      const a = quietState.mainListScroll ?? {};
      const b = candidate.mainListScroll ?? {};
      let atBottom = false;
      try {
        atBottom = affordanceOf(candidate).atBottom === true
          && affordanceOf(candidate).overscrollPhase === "idle";
      } catch {
        atBottom = false;
      }
      if (!atBottom
        || a.itemCount !== b.itemCount
        || a.maxScrollTop !== b.maxScrollTop
        || !sameLogicalTuple(quietState, candidate)) {
        quietEndpoint = false;
        break;
      }
      quietState = candidate;
    }
    confirmed = quietState;
    endpoint = await selectableEndpoint();
    if (quietEndpoint
      && endpoint.selectedIndex === endpoint.lastIndex
      && Number(confirmed.mainListScroll?.selectedIndex)
        === Number(confirmed.mainListScroll?.itemCount) - 1) {
      break;
    }
  }
  check("bottom-selection-is-last-selectable", quietEndpoint
    && endpoint.selectedIndex === endpoint.lastIndex
    && Number(confirmed.mainListScroll?.selectedIndex)
      === Number(confirmed.mainListScroll?.itemCount) - 1, {
    endpoint,
    state: safeCompactState(confirmed),
  });
  check("bottom-endpoint-confirmation-does-not-move", sameLogicalTuple(state, confirmed), {
    before: compactState(state),
    after: compactState(confirmed),
  });
  return { state: confirmed, endpoint };
}

async function settleAfterEnded(point: { x: number; y: number }): Promise<Json> {
  await wheel(point, "ended", 0);
  return pollState("ended settle", (state) => {
    try {
      const affordance = affordanceOf(state);
      const expectedReason = affordance.reducedMotion === true ? "reducedMotion" : "ended";
      return affordance.overscrollPhase === "idle"
        && affordance.overscrollEdge == null
        && Number(affordance.overscrollOffsetPx) === 0
        && affordance.lastSettleReason === expectedReason;
    } catch {
      return false;
    }
  }, 3_000);
}

function screenshotBlocker(error: string): boolean {
  return /screen recording|screen & system audio|permission (?:is )?not granted|blank\/black image/i.test(error);
}

async function captureScreenshot(): Promise<Json> {
  rmSync(screenshotPath, { force: true });
  let response: Json = {};
  try {
    response = await driver!.captureScreenshot({
      target,
      hiDpi: true,
      savePath: screenshotPath,
      timeoutMs: 10_000,
    });
  } catch (error) {
    response = { error: error instanceof Error ? error.message : String(error) };
  }
  const error = typeof response.error === "string" ? response.error : "";
  const bytes = existsSync(screenshotPath) ? statSync(screenshotPath).size : 0;
  const captured = typeof response.data === "string"
    && response.data.length > 0
    && error.length === 0
    && bytes > 0;
  if (captured) {
    return {
      classification: "captured-unvalidated",
      path: relative(repoRoot, screenshotPath),
      bytes,
      width: response.width ?? null,
      height: response.height ?? null,
    };
  }
  const classification = screenshotBlocker(error)
    ? "blocked-by-screen-recording-permission"
    : "error";
  if (classification === "error") {
    check("screenshot-attempt-completed-or-honestly-permission-blocked", false, {
      error: error || "capture returned no PNG data",
    });
  }
  return {
    classification,
    path: null,
    bytes,
    error: error || "capture returned no PNG data",
  };
}

async function runProbe(): Promise<Json> {
  if (!existsSync(binary)) {
    throw new Error(
      `Pinned probe binary is missing: ${binary}. Build SCRIPT_KIT_AGENT_ARTIFACT_NAME=launcher-scroll-affordance first or pass PROBE_BINARY/--binary.`,
    );
  }

  const provider = {
    query,
    delayMs: 0,
    results: Array.from({ length: 80 }, (_, index) => {
      const id = String(index + 1).padStart(3, "0");
      return {
        path: `/tmp/${query}-${id}.txt`,
        name: `Affordance fixture ${id}.txt`,
        fileType: "document",
        size: 2_048 + index,
        modified: 1_700_000_000 - index,
      };
    }),
  };

  driver = await Driver.launch({
    binary,
    sessionName: "launcher-scroll-affordance",
    sessionDir,
    sandboxHome: true,
    sharedModels: false,
    readyTimeoutMs: 30_000,
    defaultTimeoutMs: 8_000,
    env: {
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER: JSON.stringify(provider),
    },
  });

  driver.send({ type: "show" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs });
  await driver.setFilterAndWait(filterInput, { timeoutMs });
  const ready = await pollState("deterministic long ScriptList", (state) => {
    try {
      const scroll = state.mainListScroll ?? {};
      const rootSearch = state.rootFileSearch ?? {};
      const inspection = inspectMainListScrollAffordance(scroll, true);
      return state.surfaceContract?.surfaceKind === "ScriptList"
        && rootSearch.query === query
        && rootSearch.loading === false
        && Number(state.visibleChoiceCount) >= 12
        && Number(scroll.maxScrollTop) > 0
        && inspection.complete;
    } catch {
      return false;
    }
  });
  const settledReady = await waitForQuietPopulation("deterministic long ScriptList");
  check("deterministic-long-script-list-ready", true, {
    initialVisibleChoiceCount: ready.visibleChoiceCount,
    settledVisibleChoiceCount: settledReady.visibleChoiceCount,
    itemCount: settledReady.mainListScroll?.itemCount,
    maxScrollTop: settledReady.mainListScroll?.maxScrollTop,
    rootFileQuery: settledReady.rootFileSearch?.query,
  });

  const layout = await driver.getLayoutInfo({ target }, { timeoutMs: 8_000 });
  const scriptList = (Array.isArray(layout.components) ? layout.components : [])
    .find((component: Json) => component.name === "ScriptList");
  const bounds = scriptList?.bounds;
  if (!bounds
    || finite(bounds.x) == null
    || finite(bounds.y) == null
    || finite(bounds.width) == null
    || finite(bounds.height) == null
    || Number(bounds.width) <= 0
    || Number(bounds.height) <= 0) {
    throw new Error(`getLayoutInfo omitted measured ScriptList bounds: ${JSON.stringify(scriptList)}`);
  }
  const point = {
    x: Number(bounds.x) + Number(bounds.width) / 2,
    y: Number(bounds.y) + Number(bounds.height) / 2,
  };
  check("wheel-point-derived-from-script-list-layout", true, { bounds, point });

  // Missing Ended runs first, before unrelated launcher sources can hydrate,
  // so the watchdog is measured against a stable logical list generation.
  const timeoutTop = await establishTop();
  await wheel(point, "started", 0);
  await wheel(point, "moved", 36);
  const timeoutTracking = await driver.getState();
  const timeoutGeneration = Number(affordanceOf(timeoutTracking).generation);
  const timeoutSettled = await pollState("missing-Ended idle watchdog", (state) => {
    try {
      const affordance = affordanceOf(state);
      const expectedReason = affordance.reducedMotion === true
        ? "reducedMotion"
        : "idleTimeout";
      return affordance.overscrollPhase === "idle"
        && affordance.overscrollEdge == null
        && Number(affordance.overscrollOffsetPx) === 0
        && Number(affordance.generation) > timeoutGeneration
        && affordance.lastSettleReason === expectedReason;
    } catch {
      return false;
    }
  }, 3_000);
  check("missing-ended-watchdog-settles-to-exact-idle", true, {
    tracking: compactState(timeoutTracking),
    settled: compactState(timeoutSettled),
  });
  check("missing-ended-pull-preserves-logical-scroll-and-selection",
    sameLogicalTuple(timeoutTop.state, timeoutTracking), {
      baseline: compactState(timeoutTop.state),
      tracking: compactState(timeoutTracking),
    });
  check("missing-ended-rebound-preserves-logical-scroll-and-selection",
    sameLogicalTuple(timeoutTop.state, timeoutSettled), {
      baseline: compactState(timeoutTop.state),
      settled: compactState(timeoutSettled),
    });
  scenarios.missingEnded = {
    tracking: compactState(timeoutTracking),
    settled: compactState(timeoutSettled),
  };

  // Top: endpoint and logical tuple are fixed while the rubber band tracks.
  const top = await establishTop();
  const topAffordance = affordanceOf(top.state);
  check("top-fade-is-exactly-inactive", topAffordance.topFadeActive === false
    && Number(topAffordance.topFadeProgress) === 0
    && Number(topAffordance.topFadeAlpha) === 0, compactState(top.state));
  await wheel(point, "started", 0);
  await wheel(point, "moved", 36);
  const topTracking = await driver.getState();
  const topTrackingAffordance = affordanceOf(topTracking);
  check("top-outward-pull-tracks-positive-bounded-offset",
    topTrackingAffordance.overscrollPhase === "tracking"
      && topTrackingAffordance.overscrollEdge === "top"
      && Number(topTrackingAffordance.overscrollOffsetPx) > 0
      && Number(topTrackingAffordance.overscrollOffsetPx)
        < Number(topTrackingAffordance.overscrollMaxOffsetPx),
    compactState(topTracking));
  check("top-pull-preserves-logical-scroll-and-selection", sameLogicalTuple(top.state, topTracking), {
    before: compactState(top.state),
    tracking: compactState(topTracking),
  });
  const topSettled = await settleAfterEnded(point);
  check("top-ended-gesture-settles-to-exact-zero",
    Number(affordanceOf(topSettled).overscrollOffsetPx) === 0,
    compactState(topSettled));
  check("top-rebound-preserves-logical-scroll-and-selection",
    sameLogicalTuple(top.state, topSettled), {
      before: compactState(top.state),
      settled: compactState(topSettled),
    });
  scenarios.top = {
    endpoint: top.endpoint,
    baseline: compactState(top.state),
    tracking: compactState(topTracking),
    settled: compactState(topSettled),
  };

  // Reversal: inward travel first unwinds the rubber band without moving the list.
  const reversalTop = await establishTop();
  await wheel(point, "started", 0);
  await wheel(point, "moved", 36);
  const reversalOutward = await driver.getState();
  await wheel(point, "moved", -18);
  const reversalInward = await driver.getState();
  const outwardAffordance = affordanceOf(reversalOutward);
  const inwardAffordance = affordanceOf(reversalInward);
  check("reversal-reduces-overscroll-before-list-motion",
    Number(inwardAffordance.overscrollOffsetPx) > 0
      && Number(inwardAffordance.overscrollOffsetPx)
        < Number(outwardAffordance.overscrollOffsetPx)
      && sameLogicalTuple(reversalTop.state, reversalInward), {
      outward: compactState(reversalOutward),
      inward: compactState(reversalInward),
    });
  const reversalSettled = await settleAfterEnded(point);

  // Excess inward travel in the same gesture becomes normal selection-owned
  // movement only after the visual pull has been fully unwound.
  const excessTop = await establishTop();
  await wheel(point, "started", 0);
  await wheel(point, "moved", 36);
  const excessOutward = await driver.getState();
  await wheel(point, "moved", -144);
  const excessResidual = await pollState("excess reversal residual", (state) => {
    try {
      const affordance = affordanceOf(state);
      return affordance.overscrollPhase === "idle"
        && Number(affordance.overscrollOffsetPx) === 0
        && Number(state.mainListScroll?.selectedIndex)
          > Number(excessTop.state.mainListScroll?.selectedIndex);
    } catch {
      return false;
    }
  });
  check("excess-reversal-resumes-selection-owned-movement-after-unwind",
    !sameLogicalTuple(excessTop.state, excessResidual), {
      baseline: compactState(excessTop.state),
      outward: compactState(excessOutward),
      residual: compactState(excessResidual),
    });
  await wheel(point, "ended", 0);
  const excessEnded = await pollState("excess reversal ended", (state) => {
    try {
      const affordance = affordanceOf(state);
      return affordance.overscrollPhase === "idle"
        && Number(affordance.overscrollOffsetPx) === 0
        && sameLogicalTuple(excessResidual, state);
    } catch {
      return false;
    }
  });
  scenarios.reversal = {
    partial: {
      outward: compactState(reversalOutward),
      inward: compactState(reversalInward),
      settled: compactState(reversalSettled),
    },
    excess: {
      baseline: compactState(excessTop.state),
      outward: compactState(excessOutward),
      residual: compactState(excessResidual),
      ended: compactState(excessEnded),
    },
  };

  // Middle: top occlusion is driven by logical scroll, not translated offset.
  await establishTop();
  let middle = await driver.getState();
  for (let attempt = 0; attempt < 8; attempt += 1) {
    await gpuiKey("pagedown");
    middle = await pollState("middle logical scroll", (state) => {
      try {
        return affordanceOf(state).overscrollPhase === "idle";
      } catch {
        return false;
      }
    });
    const affordance = affordanceOf(middle);
    if (affordance.atTop === false && affordance.atBottom === false) break;
  }
  const middleAffordance = affordanceOf(middle);
  check("middle-is-between-logical-boundaries",
    middleAffordance.atTop === false && middleAffordance.atBottom === false,
    compactState(middle));
  check("middle-top-fade-is-active",
    middleAffordance.topFadeActive === true
      && Number(middleAffordance.topFadeProgress) > 0
      && Number(middleAffordance.topFadeAlpha) > 0,
    compactState(middle));
  scenarios.middle = compactState(middle);
  screenshotResult = await captureScreenshot();

  // Bottom: endpoint and logical tuple stay fixed under an outward pull.
  const bottom = await establishBottom();
  const bottomAffordance = affordanceOf(bottom.state);
  check("bottom-top-fade-remains-active", bottomAffordance.topFadeActive === true,
    compactState(bottom.state));
  await wheel(point, "started", 0);
  await wheel(point, "moved", -36);
  const bottomTracking = await driver.getState();
  const bottomTrackingAffordance = affordanceOf(bottomTracking);
  check("bottom-outward-pull-tracks-negative-bounded-offset",
    bottomTrackingAffordance.overscrollPhase === "tracking"
      && bottomTrackingAffordance.overscrollEdge === "bottom"
      && Number(bottomTrackingAffordance.overscrollOffsetPx) < 0
      && Math.abs(Number(bottomTrackingAffordance.overscrollOffsetPx))
        < Number(bottomTrackingAffordance.overscrollMaxOffsetPx),
    compactState(bottomTracking));
  check("bottom-pull-preserves-logical-scroll-and-selection",
    sameLogicalTuple(bottom.state, bottomTracking), {
      before: compactState(bottom.state),
      tracking: compactState(bottomTracking),
    });
  const bottomSettled = await settleAfterEnded(point);
  check("bottom-ended-gesture-settles-to-exact-zero",
    Number(affordanceOf(bottomSettled).overscrollOffsetPx) === 0,
    compactState(bottomSettled));
  check("bottom-rebound-preserves-logical-scroll-and-selection",
    sameLogicalTuple(bottom.state, bottomSettled), {
      before: compactState(bottom.state),
      settled: compactState(bottomSettled),
    });
  scenarios.bottom = {
    endpoint: bottom.endpoint,
    baseline: compactState(bottom.state),
    tracking: compactState(bottomTracking),
    settled: compactState(bottomSettled),
  };

  return {
    eventPoint: { bounds, point },
    screenshot: screenshotResult,
  };
}

rmSync(outputDir, { recursive: true, force: true });
mkdirSync(outputDir, { recursive: true });

let runResult: Json = {};
let runtimeError: string | null = null;
let driverClosed = false;
let sandboxRemoved = false;
try {
  runResult = await runProbe();
} catch (error) {
  runtimeError = error instanceof Error ? error.stack ?? error.message : String(error);
  failures.push("runtime-error");
} finally {
  if (driver) {
    try {
      await driver.close();
      driverClosed = true;
    } catch (error) {
      failures.push("driver-cleanup");
      runtimeError ??= error instanceof Error ? error.message : String(error);
    }
  }
  try {
    rmSync(sessionDir, { recursive: true, force: true });
    sandboxRemoved = !existsSync(sessionDir);
  } catch (error) {
    failures.push("sandbox-cleanup");
    runtimeError ??= error instanceof Error ? error.message : String(error);
  }
}

const binaryStat = existsSync(binary) ? statSync(binary) : null;
const receipt = {
  schemaVersion: 1,
  probe: "launcher-scroll-affordance",
  status: failures.length === 0 ? "pass" : "fail",
  binary: {
    path: relative(repoRoot, binary),
    override: process.env.PROBE_BINARY != null || process.argv.includes("--binary"),
    bytes: binaryStat?.size ?? null,
    modifiedAt: binaryStat ? new Date(binaryStat.mtimeMs).toISOString() : null,
  },
  fixture: { source: "root-file-search-test-provider", query, resultCount: 80 },
  checks,
  scenarios,
  eventPoint: runResult.eventPoint ?? null,
  screenshot: runResult.screenshot ?? screenshotResult,
  cleanup: { driverClosed, sandboxRemoved },
  failures,
  error: runtimeError,
};
writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
if (failures.length > 0) process.exitCode = 1;
