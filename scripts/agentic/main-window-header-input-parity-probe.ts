#!/usr/bin/env bun
/**
 * Real-app proof for the exhaustive main-window header/input policy.
 *
 * Coverage classes:
 * - launcher: ScriptList
 * - split/preview built-in: ThemeChooser
 * - list built-in: ClipboardHistory
 * - picker: EmojiPicker
 * - attachment preview: DictationHistory
 * - intentional compact exception: FocusedTextMini input and result/footer phases
 * - embedded multiline surface: AgentChat (empty one-line composer)
 * - view-owned context-only surface: DayPage
 * - root-owned context-only surfaces: PermissionsWizard and ConfirmPrompt
 *
 * Source contracts classify the remaining AppViews. The protocol currently
 * has no generic deterministic script-prompt fixture, so script-owned body
 * inputs are covered by the root-context policy/compiler inventory rather
 * than a synthetic runtime route.
 */
import {
  existsSync,
  mkdirSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { tapMainHotkey } from "./day-page-open-helper";

const binary = resolve(
  process.env.SCRIPT_KIT_GPUI_BINARY ??
    resolve(
      import.meta.dir,
      "../../target-agent/artifacts/main-header-parity/script-kit-gpui",
    ),
);
const outputDir = resolve(
  process.env.PROBE_OUTPUT_DIR ??
    ".test-output/main-window-header-input-parity",
);
mkdirSync(outputDir, { recursive: true });

type Bounds = { x: number; y: number; width: number; height: number };
type Size = { width: number; height: number };
type PaintMeasurement = {
  bounds: Bounds;
  measurementProvenance: string | null;
  coordinateSpace: string | null;
  measurementFrameGeneration: number | null;
};
type SnapshotExpectation = {
  surfaceKind: string;
  statePromptType: string;
  layoutPromptType: string;
};
type Check = { name: string; pass: boolean; detail: Json };
const checks: Check[] = [];
let previousSnapshotPaintFrameGeneration: number | null = null;

function check(name: string, pass: boolean, detail: Json = {}) {
  checks.push({ name, pass, detail });
}

function componentBounds(
  layout: Json,
  label: string,
  name: string,
): Bounds | null {
  if (!Array.isArray(layout.components)) {
    throw new Error(`${label}: getLayoutInfo returned no components array`);
  }
  const matches = (layout.components as Json[]).filter(
    (entry) => entry.name === name,
  );
  check(`${label}-${name}-unique`, matches.length <= 1, {
    component: name,
    count: matches.length,
  });
  if (matches.length > 1) {
    throw new Error(
      `${label}: expected at most one ${name}, got ${matches.length}`,
    );
  }
  const bounds = matches[0]?.bounds as Bounds | undefined;
  return bounds ?? null;
}

function paintMeasurement(
  layout: Json,
  label: string,
  name: string,
): PaintMeasurement | null {
  if (!Array.isArray(layout.components)) {
    throw new Error(`${label}: getLayoutInfo returned no components array`);
  }
  const matches = (layout.components as Json[]).filter(
    (entry) => entry.name === name,
  );
  check(`${label}-paint-${name}-unique`, matches.length <= 1, {
    component: name,
    count: matches.length,
  });
  if (matches.length > 1) {
    throw new Error(
      `${label}: expected at most one paint measurement ${name}, got ${matches.length}`,
    );
  }
  const entry = matches[0];
  if (entry === undefined) return null;
  const measurement = {
    bounds: entry.bounds as Bounds,
    measurementProvenance: String(entry.measurementProvenance ?? "") || null,
    coordinateSpace: String(entry.coordinateSpace ?? "") || null,
    measurementFrameGeneration:
      typeof entry.measurementFrameGeneration === "number"
        ? entry.measurementFrameGeneration
        : null,
  };
  check(
    `${label}-paint-${name}-has-frame-coherent-provenance`,
    measurement.measurementProvenance === "paint-time" &&
      measurement.coordinateSpace === "window" &&
      Number.isSafeInteger(measurement.measurementFrameGeneration) &&
      Number(measurement.measurementFrameGeneration) > 0,
    measurement,
  );
  return measurement;
}

function chrome(layout: Json, label: string): Json {
  return {
    shell: {
      width: layout.windowWidth ?? null,
      height: layout.windowHeight ?? null,
    },
    header: componentBounds(layout, label, "MainViewHeader"),
    context: componentBounds(layout, label, "MainViewContextZone"),
    input: componentBounds(layout, label, "MainViewInput"),
    main: componentBounds(layout, label, "MainViewMain"),
    footer: componentBounds(layout, label, "MainViewFooter"),
  };
}

function paintChrome(layout: Json, label: string): Json {
  const paint: Record<string, PaintMeasurement | null> = {
    shell: paintMeasurement(layout, label, "main-view-shell"),
    header: paintMeasurement(layout, label, "main-view-header"),
    context: paintMeasurement(layout, label, "main-view-context-zone"),
    inputShell: paintMeasurement(layout, label, "main-view-input-shell"),
    inputBody: paintMeasurement(layout, label, "main-view-input-body"),
    main: paintMeasurement(layout, label, "main-view-main"),
    cwdChip: paintMeasurement(layout, label, "main-view-context-cwd-button"),
    modelChip: paintMeasurement(
      layout,
      label,
      "main-view-context-model-button",
    ),
    sendButton: paintMeasurement(layout, label, "agent-chat-send-button"),
  };
  const frames = Object.values(paint)
    .map((entry) => entry?.measurementFrameGeneration ?? null)
    .filter((frame): frame is number => frame !== null);
  check(
    `${label}-paint-measurements-share-one-frame`,
    frames.length > 0 && new Set(frames).size === 1,
    { frames },
  );
  return paint;
}

function compactPaint(layout: Json, label: string): Json {
  const paint: Record<string, PaintMeasurement | null> = {
    root: paintMeasurement(layout, label, "focused-text-mini-root"),
    input: paintMeasurement(layout, label, "focused-text-mini-input-row"),
    content: paintMeasurement(layout, label, "focused-text-mini-content"),
    footerSpacer: paintMeasurement(
      layout,
      label,
      "native-main-window-footer-spacer",
    ),
    sharedShell: paintMeasurement(layout, label, "main-view-shell"),
    sharedHeader: paintMeasurement(layout, label, "main-view-header"),
    sharedInput: paintMeasurement(layout, label, "main-view-input-shell"),
  };
  const frames = Object.values(paint)
    .map((entry) => entry?.measurementFrameGeneration ?? null)
    .filter((frame): frame is number => frame !== null);
  check(
    `${label}-compact-paint-measurements-share-one-frame`,
    frames.length > 0 && new Set(frames).size === 1,
    { frames },
  );
  return paint;
}

function paintFrameGeneration(paint: Json): number | null {
  const measurements = Object.values(paint).filter(
    (entry): entry is PaintMeasurement => entry !== null,
  );
  if (measurements.length === 0) return null;
  const frames = measurements.map(
    (measurement) => measurement.measurementFrameGeneration,
  );
  if (
    frames.some((frame) => !Number.isSafeInteger(frame) || Number(frame) <= 0)
  ) {
    return null;
  }
  const unique = [...new Set(frames)];
  return unique.length === 1 ? Number(unique[0]) : null;
}

function near(a: number, b: number, tolerance = 1): boolean {
  return Math.abs(a - b) <= tolerance;
}

function boundsNear(
  a: Bounds | null,
  b: Bounds | null,
  tolerance = 1,
): boolean {
  if (!a || !b) return false;
  if (
    ![a.x, a.y, a.width, a.height, b.x, b.y, b.width, b.height].every(
      (value) => typeof value === "number" && Number.isFinite(value),
    )
  ) {
    return false;
  }
  return (
    near(a.x, b.x, tolerance) &&
    near(a.y, b.y, tolerance) &&
    near(a.width, b.width, tolerance) &&
    near(a.height, b.height, tolerance)
  );
}

function leadingBoundsNear(
  a: Bounds | null,
  b: Bounds | null,
  tolerance = 1,
): boolean {
  if (!a || !b) return false;
  return (
    near(a.x, b.x, tolerance) &&
    near(a.y, b.y, tolerance) &&
    near(a.height, b.height, tolerance)
  );
}

function measuredBounds(value: unknown): Bounds | null {
  return (value as PaintMeasurement | null)?.bounds ?? null;
}

function boundsAreFinite(bounds: Bounds | null): bounds is Bounds {
  return (
    bounds !== null &&
    [bounds.x, bounds.y, bounds.width, bounds.height].every(
      (value) => typeof value === "number" && Number.isFinite(value),
    )
  );
}

function boundsHavePositiveArea(bounds: Bounds | null): bounds is Bounds {
  return boundsAreFinite(bounds) && bounds.width > 0 && bounds.height > 0;
}

function boundsContainedBy(
  inner: Bounds | null,
  outer: Bounds | null,
  tolerance = 1,
): boolean {
  if (!boundsAreFinite(inner) || !boundsAreFinite(outer)) return false;
  return (
    inner.x >= outer.x - tolerance &&
    inner.y >= outer.y - tolerance &&
    inner.x + inner.width <= outer.x + outer.width + tolerance &&
    inner.y + inner.height <= outer.y + outer.height + tolerance
  );
}

function optionalBoundsNear(
  modeled: Bounds | null,
  painted: Bounds | null,
  tolerance = 1,
): boolean {
  if (modeled === null || painted === null) return modeled === painted;
  return boundsNear(modeled, painted, tolerance);
}

function sizeNear(a: Size | null, b: Size | null, tolerance = 1): boolean {
  if (!a || !b) return false;
  if (
    ![a.width, a.height, b.width, b.height].every(
      (value) => typeof value === "number" && Number.isFinite(value),
    )
  ) {
    return false;
  }
  return (
    near(a.width, b.width, tolerance) && near(a.height, b.height, tolerance)
  );
}

function surfaceKind(state: Json): string {
  return String(state.surfaceContract?.surfaceKind ?? "unknown");
}

function assertStateBinding(
  label: string,
  phase: "before-layout" | "after-layout",
  state: Json,
  expected: SnapshotExpectation,
) {
  const actualSurfaceKind = surfaceKind(state);
  const actualPromptType = String(state.promptType ?? "");
  check(
    `${label}-${phase}-state-bound-to-expected-surface-and-prompt-type`,
    actualSurfaceKind === expected.surfaceKind &&
      actualPromptType === expected.statePromptType,
    {
      expectedSurfaceKind: expected.surfaceKind,
      actualSurfaceKind,
      expectedPromptType: expected.statePromptType,
      actualPromptType,
    },
  );
}

function assertModelPaintGeometry(label: string, modeled: Json, painted: Json) {
  const modeledShell = modeled.shell as Size | null;
  const paintedShell = measuredBounds(painted.shell);
  const shellBounds =
    modeledShell === null
      ? null
      : {
          x: 0,
          y: 0,
          width: modeledShell.width,
          height: modeledShell.height,
        };
  check(
    `${label}-model-paint-shell-geometry-matches`,
    boundsNear(shellBounds, paintedShell),
    { modeled: shellBounds, painted: paintedShell, tolerancePx: 1 },
  );

  for (const [modelPart, paintPart] of [
    ["header", "header"],
    ["context", "context"],
    ["input", "inputShell"],
    ["main", "main"],
  ] as const) {
    const modeledBounds = modeled[modelPart] as Bounds | null;
    const paintedBounds = measuredBounds(painted[paintPart]);
    check(
      `${label}-model-paint-${modelPart}-geometry-matches`,
      optionalBoundsNear(modeledBounds, paintedBounds),
      {
        modeled: modeledBounds,
        painted: paintedBounds,
        tolerancePx: 1,
      },
    );
  }
}

async function waitForSurface(driver: Driver, expected: string): Promise<Json> {
  const deadline = Date.now() + 8_000;
  let state = await driver.getState({ timeoutMs: 8_000 });
  while (surfaceKind(state) !== expected && Date.now() < deadline) {
    await Bun.sleep(60);
    state = await driver.getState({ timeoutMs: 8_000 });
  }
  if (surfaceKind(state) !== expected) {
    throw new Error(`expected ${expected}, got ${surfaceKind(state)}`);
  }
  const settled = await driver.waitForSettle({ timeoutMs: 8_000 });
  check(`${expected}-settled`, settled.settled, {
    elapsedMs: settled.elapsedMs,
    probes: settled.probes,
    lastSurfaceKind: surfaceKind(settled.lastState),
  });
  if (!settled.settled) {
    throw new Error(`${expected} did not settle within 8000ms`);
  }
  const stableSurface = surfaceKind(settled.lastState);
  check(`${expected}-surface-remained-stable`, stableSurface === expected, {
    expected,
    actual: stableSurface,
  });
  if (stableSurface !== expected) {
    throw new Error(`${expected} changed to ${stableSurface} while settling`);
  }
  return settled.lastState;
}

async function resetToVisibleScriptList(driver: Driver): Promise<Json> {
  // A sandbox can still restore the last in-window surface (for example Day
  // Page). The production hide path is the canonical owner that tears down
  // child surfaces and resets the hidden main window to ScriptList; reopen it
  // through the real main-hotkey gesture before taking the baseline.
  driver.send({
    type: "hide",
    requestId: `main-header-parity-reset-${Date.now()}`,
  });
  await driver.waitForState({ windowVisible: false }, { timeoutMs: 8_000 });
  await tapMainHotkey(driver, "main-window-header-input-parity", "show-main");
  return waitForSurface(driver, "ScriptList");
}

function fileSize(path: string): number | null {
  try {
    const stats = statSync(path);
    return stats.isFile() ? stats.size : null;
  } catch {
    return null;
  }
}

async function snapshot(
  driver: Driver,
  label: string,
  expected: SnapshotExpectation,
  screenshot = false,
): Promise<Json> {
  const stateBefore = await driver.getState({ timeoutMs: 10_000 });
  assertStateBinding(label, "before-layout", stateBefore, expected);
  const layout = await driver.getLayoutInfo({}, { timeoutMs: 10_000 });
  check(
    `${label}-layout-bound-to-expected-prompt-type`,
    layout.promptType === expected.layoutPromptType,
    {
      expectedPromptType: expected.layoutPromptType,
      actualPromptType: layout.promptType ?? null,
    },
  );
  const stateAfter = await driver.getState({ timeoutMs: 10_000 });
  assertStateBinding(label, "after-layout", stateAfter, expected);
  check(
    `${label}-surface-and-prompt-type-stable-across-layout-capture`,
    surfaceKind(stateBefore) === surfaceKind(stateAfter) &&
      stateBefore.promptType === stateAfter.promptType,
    {
      before: {
        surfaceKind: surfaceKind(stateBefore),
        promptType: stateBefore.promptType ?? null,
      },
      after: {
        surfaceKind: surfaceKind(stateAfter),
        promptType: stateAfter.promptType ?? null,
      },
    },
  );
  const elements = await driver.getElements(
    { limit: 160, includeHeaders: true },
    { timeoutMs: 10_000 },
  );
  const activeFooter = (stateAfter.activeFooter ?? null) as Json | null;
  const footerContextChipCount = activeFooter?.contextChipCount ?? null;
  check(
    `${label}-footer-context-chips-do-not-duplicate-header`,
    footerContextChipCount === 0,
    {
      actual: footerContextChipCount,
      expected: 0,
      assumption:
        "CWD/model context belongs in MainViewContextZone; the sampled surface must not duplicate it in footer context chips.",
    },
  );
  const measuredChrome = chrome(layout, label);
  const measuredPaintChrome = paintChrome(layout, label);
  assertModelPaintGeometry(label, measuredChrome, measuredPaintChrome);
  const currentPaintFrameGeneration = paintFrameGeneration(measuredPaintChrome);
  check(
    `${label}-paint-frame-is-fresh-for-transitioned-surface`,
    currentPaintFrameGeneration !== null &&
      (previousSnapshotPaintFrameGeneration === null ||
        currentPaintFrameGeneration > previousSnapshotPaintFrameGeneration),
    {
      previousFrameGeneration: previousSnapshotPaintFrameGeneration,
      currentFrameGeneration: currentPaintFrameGeneration,
      requirement:
        previousSnapshotPaintFrameGeneration === null
          ? "baseline establishes a valid positive rendered-frame generation"
          : "transitioned surface must be measured from a strictly newer rendered frame",
    },
  );
  if (currentPaintFrameGeneration !== null) {
    previousSnapshotPaintFrameGeneration = currentPaintFrameGeneration;
  }
  const contextBounds = measuredBounds(measuredPaintChrome.context);
  const cwdChipBounds = measuredBounds(measuredPaintChrome.cwdChip);
  const modelChipBounds = measuredBounds(measuredPaintChrome.modelChip);
  check(
    `${label}-context-and-chip-bounds-have-positive-area`,
    boundsHavePositiveArea(contextBounds) &&
      boundsHavePositiveArea(cwdChipBounds) &&
      boundsHavePositiveArea(modelChipBounds),
    {
      context: contextBounds,
      cwdChip: cwdChipBounds,
      modelChip: modelChipBounds,
    },
  );
  check(
    `${label}-context-chips-stay-in-lanes-without-overlap`,
    boundsHavePositiveArea(contextBounds) &&
      boundsHavePositiveArea(cwdChipBounds) &&
      boundsHavePositiveArea(modelChipBounds) &&
      boundsContainedBy(cwdChipBounds, contextBounds) &&
      boundsContainedBy(modelChipBounds, contextBounds) &&
      cwdChipBounds.x + cwdChipBounds.width <= modelChipBounds.x + 1,
    {
      context: contextBounds,
      cwdChip: cwdChipBounds,
      modelChip: modelChipBounds,
      tolerancePx: 1,
    },
  );
  let screenshotReceipt: Json | null = null;
  if (screenshot) {
    const screenshotPath = join(outputDir, `${label}.png`);
    rmSync(screenshotPath, { force: true });
    let result: Json;
    try {
      result = await driver.captureScreenshot({
        hiDpi: true,
        savePath: screenshotPath,
        timeoutMs: 10_000,
      });
    } catch (error) {
      result = { error: String(error) };
    }
    const bytes = fileSize(screenshotPath);
    const responseOk =
      typeof result.data === "string" &&
      result.data.length > 0 &&
      result.error == null &&
      Number(result.width) > 0 &&
      Number(result.height) > 0;
    check(`${label}-screenshot-response-ok`, responseOk, {
      width: result.width ?? null,
      height: result.height ?? null,
      error: result.error ?? null,
      hasData: typeof result.data === "string" && result.data.length > 0,
    });
    check(`${label}-screenshot-file-present`, bytes !== null && bytes > 0, {
      path: screenshotPath,
      bytes,
    });
    screenshotReceipt = {
      width: result.width ?? null,
      height: result.height ?? null,
      error: result.error ?? null,
      path: screenshotPath,
      bytes,
    };
  }
  return {
    surfaceKind: surfaceKind(stateAfter),
    promptType: stateAfter.promptType ?? null,
    layoutPromptType: layout.promptType ?? null,
    windowVisible: stateAfter.windowVisible ?? null,
    activeFooter,
    activeFooterContextChipCount: footerContextChipCount,
    chrome: measuredChrome,
    paintChrome: measuredPaintChrome,
    paintFrameGeneration: currentPaintFrameGeneration,
    focusedSemanticId: elements.focusedSemanticId ?? null,
    selectedSemanticId: elements.selectedSemanticId ?? null,
    screenshot: screenshotReceipt,
  };
}

async function compactSnapshot(
  driver: Driver,
  label: string,
  expected: SnapshotExpectation,
  expectsResultFooter: boolean,
): Promise<Json> {
  const stateBefore = await driver.getState({ timeoutMs: 10_000 });
  assertStateBinding(label, "before-layout", stateBefore, expected);
  const layout = await driver.getLayoutInfo({}, { timeoutMs: 10_000 });
  check(
    `${label}-layout-bound-to-expected-prompt-type`,
    layout.promptType === expected.layoutPromptType,
    {
      expectedPromptType: expected.layoutPromptType,
      actualPromptType: layout.promptType ?? null,
    },
  );
  const stateAfter = await driver.getState({ timeoutMs: 10_000 });
  assertStateBinding(label, "after-layout", stateAfter, expected);

  const root = componentBounds(layout, label, "FocusedTextMiniRoot");
  const input = componentBounds(layout, label, "FocusedTextMiniInputRow");
  const scope = componentBounds(layout, label, "FocusedTextMiniScopeRow");
  const result = componentBounds(layout, label, "FocusedTextMiniResult");
  const footer = componentBounds(layout, label, "MainViewFooter");
  const sharedHeader = componentBounds(layout, label, "MainViewHeader");
  const sharedContext = componentBounds(layout, label, "MainViewContextZone");
  const sharedInput = componentBounds(layout, label, "MainViewInput");
  const sharedMain = componentBounds(layout, label, "MainViewMain");
  const paint = compactPaint(layout, label);
  const paintRoot = measuredBounds(paint.root);
  const paintInput = measuredBounds(paint.input);
  const paintContent = measuredBounds(paint.content);
  const paintFooterSpacer = measuredBounds(paint.footerSpacer);

  check(
    `${label}-intentional-compact-policy-emits-no-fake-shared-header-input-nodes`,
    sharedHeader === null &&
      sharedContext === null &&
      sharedInput === null &&
      sharedMain === null &&
      paint.sharedShell === null &&
      paint.sharedHeader === null &&
      paint.sharedInput === null,
    {
      sharedHeader,
      sharedContext,
      sharedInput,
      sharedMain,
      sharedPaint: {
        shell: paint.sharedShell,
        header: paint.sharedHeader,
        input: paint.sharedInput,
      },
    },
  );
  check(
    `${label}-compact-root-and-input-model-match-painted-geometry`,
    boundsHavePositiveArea(root) &&
      boundsHavePositiveArea(input) &&
      boundsHavePositiveArea(paintRoot) &&
      boundsHavePositiveArea(paintInput) &&
      boundsNear(root, paintRoot, 2) &&
      boundsNear(input, paintInput, 2),
    { root, input, paintRoot, paintInput, tolerancePx: 2 },
  );
  check(`${label}-compact-scope-row-is-absent-for-fixture`, scope === null, {
    scope,
  });
  check(
    `${label}-compact-footer-safe-area-matches-phase`,
    expectsResultFooter
      ? boundsHavePositiveArea(result) &&
          boundsHavePositiveArea(footer) &&
          result.y + result.height <= footer.y + 1 &&
          boundsHavePositiveArea(paintContent) &&
          boundsHavePositiveArea(paintFooterSpacer) &&
          paintContent.y + paintContent.height <= paintFooterSpacer.y + 1
      : result === null && footer === null && paintFooterSpacer === null,
    {
      expectsResultFooter,
      result,
      footer,
      paintContent,
      paintFooterSpacer,
      tolerancePx: 1,
    },
  );

  const currentPaintFrameGeneration = paintFrameGeneration(paint);
  check(
    `${label}-paint-frame-is-fresh-for-transitioned-surface`,
    currentPaintFrameGeneration !== null &&
      (previousSnapshotPaintFrameGeneration === null ||
        currentPaintFrameGeneration > previousSnapshotPaintFrameGeneration),
    {
      previousFrameGeneration: previousSnapshotPaintFrameGeneration,
      currentFrameGeneration: currentPaintFrameGeneration,
    },
  );
  if (currentPaintFrameGeneration !== null) {
    previousSnapshotPaintFrameGeneration = currentPaintFrameGeneration;
  }

  return {
    surfaceKind: surfaceKind(stateAfter),
    promptType: stateAfter.promptType ?? null,
    layoutPromptType: layout.promptType ?? null,
    windowVisible: stateAfter.windowVisible ?? null,
    compact: { root, input, scope, result, footer },
    paintCompact: paint,
    paintFrameGeneration: currentPaintFrameGeneration,
  };
}

function assertCanonical(
  label: string,
  candidate: Json,
  baseline: Json,
  allowTrailingInputAction = false,
) {
  const actual = candidate.chrome as Json;
  const expected = baseline.chrome as Json;
  const actualShell = actual.shell as Size | null;
  const expectedShell = expected.shell as Size | null;
  check(
    `${label}-shell-matches-main-menu`,
    sizeNear(actualShell, expectedShell),
    {
      actual: actualShell,
      expected: expectedShell,
      tolerancePx: 1,
    },
  );
  for (const part of [
    "header",
    "context",
    "input",
    "main",
    "footer",
  ] as const) {
    const actualBounds = actual[part] as Bounds | null;
    const expectedBounds = expected[part] as Bounds | null;
    check(
      `${label}-${part}-matches-main-menu`,
      boundsNear(actualBounds, expectedBounds),
      { actual: actualBounds, expected: expectedBounds, tolerancePx: 1 },
    );
  }

  const actualPaint = candidate.paintChrome as Json;
  const expectedPaint = baseline.paintChrome as Json;
  for (const part of [
    "shell",
    "header",
    "context",
    "inputShell",
    "main",
  ] as const) {
    const actualBounds = measuredBounds(actualPaint[part]);
    const expectedBounds = measuredBounds(expectedPaint[part]);
    check(
      `${label}-paint-${part}-matches-main-menu`,
      boundsNear(actualBounds, expectedBounds),
      { actual: actualBounds, expected: expectedBounds, tolerancePx: 1 },
    );
  }
  const actualInputBody = measuredBounds(actualPaint.inputBody);
  const expectedInputBody = measuredBounds(expectedPaint.inputBody);
  check(
    `${label}-paint-input-body-leading-geometry-matches-main-menu`,
    allowTrailingInputAction
      ? leadingBoundsNear(actualInputBody, expectedInputBody)
      : boundsNear(actualInputBody, expectedInputBody),
    {
      actual: actualInputBody,
      expected: expectedInputBody,
      tolerancePx: 1,
      trailingWidthMayDiffer: allowTrailingInputAction,
    },
  );
  if (allowTrailingInputAction) {
    const actualInputShell = measuredBounds(actualPaint.inputShell);
    const actualSendButton = measuredBounds(actualPaint.sendButton);
    check(
      `${label}-paint-input-body-has-positive-width-and-stays-inside-input-shell`,
      boundsHavePositiveArea(actualInputBody) &&
        boundsContainedBy(actualInputBody, actualInputShell),
      {
        inputBody: actualInputBody,
        inputShell: actualInputShell,
        tolerancePx: 1,
      },
    );
    check(
      `${label}-paint-send-button-is-measured-inside-input-shell-without-overlap`,
      boundsHavePositiveArea(actualSendButton) &&
        boundsContainedBy(actualSendButton, actualInputShell) &&
        boundsHavePositiveArea(actualInputBody) &&
        actualInputBody.x + actualInputBody.width <= actualSendButton.x + 1,
      {
        inputBody: actualInputBody,
        sendButton: actualSendButton,
        inputShell: actualInputShell,
        tolerancePx: 1,
      },
    );
  }
}

function assertContextOnly(label: string, candidate: Json, baseline: Json) {
  const actual = candidate.chrome as Json;
  const expected = baseline.chrome as Json;
  check(
    `${label}-shell-matches-main-menu`,
    sizeNear(actual.shell as Size | null, expected.shell as Size | null),
    { actual: actual.shell, expected: expected.shell, tolerancePx: 1 },
  );
  check(
    `${label}-footer-matches-main-menu`,
    boundsNear(
      actual.footer as Bounds | null,
      expected.footer as Bounds | null,
    ),
    { actual: actual.footer, expected: expected.footer, tolerancePx: 1 },
  );
  const header = actual.header as Bounds | null;
  const context = actual.context as Bounds | null;
  const main = actual.main as Bounds | null;
  const baselineHeader = expected.header as Bounds | null;
  const baselineContext = expected.context as Bounds | null;
  const baselineInput = expected.input as Bounds | null;
  const topInset =
    baselineHeader !== null && baselineContext !== null
      ? baselineContext.y - baselineHeader.y
      : null;
  const bottomInset =
    baselineHeader !== null && baselineInput !== null
      ? baselineHeader.y +
        baselineHeader.height -
        (baselineInput.y + baselineInput.height)
      : null;
  const expectedHeaderHeight =
    topInset !== null && bottomInset !== null && baselineContext !== null
      ? topInset + baselineContext.height + bottomInset
      : null;
  check(`${label}-has-no-main-input`, actual.input == null, {
    actual: actual.input,
  });
  check(
    `${label}-context-matches-main-menu`,
    boundsNear(context, expected.context as Bounds | null),
    { actual: context, expected: expected.context, tolerancePx: 1 },
  );
  check(
    `${label}-context-only-height-derived-from-main-menu`,
    header !== null &&
      expectedHeaderHeight !== null &&
      near(header.height, expectedHeaderHeight),
    {
      actual: header,
      expectedHeight: expectedHeaderHeight,
      measuredTopInset: topInset,
      measuredBottomInset: bottomInset,
      measuredContext: baselineContext,
      tolerancePx: 1,
    },
  );
  check(
    `${label}-main-starts-after-context-only-header`,
    header !== null && main !== null && near(main.y, header.y + header.height),
    { header, main, tolerancePx: 1 },
  );

  const actualPaint = candidate.paintChrome as Json;
  const expectedPaint = baseline.paintChrome as Json;
  const paintShell = measuredBounds(actualPaint.shell);
  const paintHeader = measuredBounds(actualPaint.header);
  const paintContext = measuredBounds(actualPaint.context);
  const paintMain = measuredBounds(actualPaint.main);
  check(
    `${label}-paint-shell-matches-main-menu`,
    boundsNear(paintShell, measuredBounds(expectedPaint.shell)),
    {
      actual: paintShell,
      expected: measuredBounds(expectedPaint.shell),
      tolerancePx: 1,
    },
  );
  check(
    `${label}-paint-context-matches-main-menu`,
    boundsNear(paintContext, measuredBounds(expectedPaint.context)),
    {
      actual: paintContext,
      expected: measuredBounds(expectedPaint.context),
      tolerancePx: 1,
    },
  );
  check(
    `${label}-paint-has-no-main-input`,
    actualPaint.inputShell == null && actualPaint.inputBody == null,
    {
      inputShell: actualPaint.inputShell,
      inputBody: actualPaint.inputBody,
    },
  );
  check(
    `${label}-paint-header-is-context-only-height`,
    paintHeader !== null &&
      expectedHeaderHeight !== null &&
      near(paintHeader.height, expectedHeaderHeight),
    {
      actual: paintHeader,
      expectedHeight: expectedHeaderHeight,
      tolerancePx: 1,
    },
  );
  check(
    `${label}-paint-main-starts-after-context-only-header`,
    paintHeader !== null &&
      paintMain !== null &&
      near(paintMain.y, paintHeader.y + paintHeader.height),
    { header: paintHeader, main: paintMain, tolerancePx: 1 },
  );
}

const snapshotExpectations = {
  mainMenu: {
    surfaceKind: "ScriptList",
    statePromptType: "none",
    layoutPromptType: "mainMenu",
  },
  themeChooser: {
    surfaceKind: "ThemeChooser",
    statePromptType: "unknown",
    layoutPromptType: "themeChooser",
  },
  clipboardHistory: {
    surfaceKind: "ClipboardHistory",
    statePromptType: "unknown",
    layoutPromptType: "clipboardHistory",
  },
  emojiPicker: {
    surfaceKind: "EmojiPicker",
    statePromptType: "unknown",
    layoutPromptType: "emojiPicker",
  },
  dictationHistory: {
    surfaceKind: "AttachmentPortalBrowser",
    statePromptType: "unknown",
    layoutPromptType: "dictationHistory",
  },
  agentChat: {
    surfaceKind: "AgentChat",
    statePromptType: "unknown",
    layoutPromptType: "agentChatChat",
  },
  focusedTextMini: {
    surfaceKind: "AgentChat",
    statePromptType: "unknown",
    layoutPromptType: "focusedTextMini",
  },
  dayPage: {
    surfaceKind: "DayPage",
    statePromptType: "dayPage",
    layoutPromptType: "dayPage",
  },
  permissionsWizard: {
    surfaceKind: "PermissionsWizard",
    statePromptType: "unknown",
    layoutPromptType: "permissionsWizard",
  },
  confirmPrompt: {
    surfaceKind: "ConfirmPrompt",
    statePromptType: "unknown",
    layoutPromptType: "confirmPrompt",
  },
} satisfies Record<string, SnapshotExpectation>;

const receipt: Json = {
  schemaVersion: 1,
  probe: "main-window-header-input-parity",
  binary,
  coverage: [
    "launcher",
    "split-preview-built-in",
    "list-built-in",
    "picker",
    "attachment-preview",
    "embedded-multiline",
    "view-owned-intentional-compact",
    "view-context-only",
    "root-context-only-built-in",
    "root-context-only-prompt",
  ],
  runtimeCoverageLimitations: [
    "No generic deterministic protocol fixture exists for arbitrary script-owned prompt body inputs; exhaustive AppView policy and source contracts cover that class.",
    "Paint receipts expose geometry but not resolved font family/weight; Agent Chat typography parity is enforced by the shared renderer/style contract and Rust tests.",
  ],
  proofAssumptions: {
    paintTimeGeometry:
      "Shared shell/header/context/input/main comparisons use GPUI debug-selector bounds from one completed rendered frame; modeled MainView nodes are checked separately.",
    canonicalShellAndFooter:
      "Canonical sampled surfaces preserve the ScriptList shell and MainViewFooter bounds within 1px.",
    footerContextOwnership:
      "CWD/model context is owned by MainViewContextZone, so activeFooter.contextChipCount must equal 0 on every sampled surface.",
    contextOnlyGeometry:
      "Context-only header height is derived from the measured ScriptList top inset, context height, and post-input bottom inset.",
    typography:
      "Runtime proof covers placement and sizing; resolved font-family/weight parity is covered by code-level contracts because GPUI paint receipts do not expose glyph typography.",
  },
  surfaces: {},
};

let driver: Driver | null = null;
try {
  const binaryBytes = fileSize(binary);
  check("binary-file-present", binaryBytes !== null && binaryBytes > 0, {
    path: binary,
    exists: existsSync(binary),
    bytes: binaryBytes,
  });
  if (binaryBytes === null || binaryBytes <= 0) {
    throw new Error(
      `Required Script Kit binary is missing or empty at ${binary}. ` +
        "Build it with SCRIPT_KIT_AGENT_ARTIFACT_NAME=main-header-parity ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui, or set SCRIPT_KIT_GPUI_BINARY.",
    );
  }

  driver = await Driver.launch({
    binary,
    sandboxHome: true,
    sessionName: "main-window-header-input-parity",
    readyTimeoutMs: 30_000,
    defaultTimeoutMs: 10_000,
    env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
  });

  await resetToVisibleScriptList(driver);
  const baseline = await snapshot(
    driver,
    "main-menu",
    snapshotExpectations.mainMenu,
    true,
  );
  receipt.surfaces.mainMenu = baseline;
  const baselineShell = (baseline.chrome as Json).shell as Size | null;
  check(
    "main-menu-shell-keeps-480px-height",
    baselineShell !== null && near(baselineShell.height, 480),
    { actual: baselineShell, expectedHeight: 480, tolerancePx: 1 },
  );

  const canonicalBuiltins = [
    ["themeChooser", "builtin/choose-theme", "ThemeChooser", true],
    [
      "clipboardHistory",
      "builtin/clipboard-history",
      "ClipboardHistory",
      false,
    ],
    ["emojiPicker", "builtin/emoji-picker", "EmojiPicker", false],
    [
      "dictationHistory",
      "builtin/dictation-history",
      "AttachmentPortalBrowser",
      false,
    ],
  ] as const;

  for (const [
    label,
    builtinId,
    expectedSurface,
    screenshot,
  ] of canonicalBuiltins) {
    driver.send({ type: "triggerBuiltin", builtinId });
    await waitForSurface(driver, expectedSurface);
    const current = await snapshot(
      driver,
      label,
      snapshotExpectations[label],
      screenshot,
    );
    receipt.surfaces[label] = current;
    assertCanonical(label, current, baseline);
  }

  await resetToVisibleScriptList(driver);
  const compactInputOpen = await driver.request(
    {
      type: "openFocusedTextAgentChatWithMockData",
      text: "Focused text fixture",
      instruction: "",
    },
    { expect: "focusedTextAgentChatFixtureOpenResult", timeoutMs: 10_000 },
  );
  check(
    "focused-text-mini-input-fixture-opened-without-submission",
    compactInputOpen.ok === true && compactInputOpen.submitted === false,
    compactInputOpen,
  );
  await waitForSurface(driver, "AgentChat");
  receipt.surfaces.focusedTextMiniInput = await compactSnapshot(
    driver,
    "focused-text-mini-input",
    snapshotExpectations.focusedTextMini,
    false,
  );

  const compactResultOpen = await driver.request(
    {
      type: "openFocusedTextAgentChatWithMockData",
      text: "Focused text fixture",
      instruction: "Rewrite clearly",
    },
    { expect: "focusedTextAgentChatFixtureOpenResult", timeoutMs: 10_000 },
  );
  check(
    "focused-text-mini-result-fixture-opened-with-submission",
    compactResultOpen.ok === true && compactResultOpen.submitted === true,
    compactResultOpen,
  );
  await waitForSurface(driver, "AgentChat");
  receipt.surfaces.focusedTextMiniResult = await compactSnapshot(
    driver,
    "focused-text-mini-result",
    snapshotExpectations.focusedTextMini,
    true,
  );

  await resetToVisibleScriptList(driver);
  await driver.request(
    { type: "openAgentChatKitchenSinkFixture" },
    { timeoutMs: 10_000 },
  );
  await waitForSurface(driver, "AgentChat");
  const agentChat = await snapshot(
    driver,
    "agent-chat",
    snapshotExpectations.agentChat,
    true,
  );
  let agentChatState: Json | null = null;
  let agentChatStateError: string | null = null;
  try {
    agentChatState = await driver.request(
      { type: "getAgentChatState" },
      { expect: "agent_chatStateResult", timeoutMs: 10_000 },
    );
  } catch (error) {
    agentChatStateError = String(error);
  }
  const agentChatInputLayout = agentChatState?.inputLayout ?? null;
  const agentChatComposerScroll = agentChatState?.composerScroll ?? null;
  check(
    "agent-chat-state-available",
    agentChatStateError === null &&
      agentChatState?.type === "agent_chatStateResult",
    {
      error: agentChatStateError,
      responseType: agentChatState?.type ?? null,
    },
  );
  check(
    "agent-chat-input-layout-present",
    agentChatInputLayout !== null && typeof agentChatInputLayout === "object",
    { actual: agentChatInputLayout },
  );
  check(
    "agent-chat-composer-scroll-present",
    agentChatComposerScroll !== null &&
      typeof agentChatComposerScroll === "object",
    { actual: agentChatComposerScroll },
  );
  const baselineInputHeight =
    ((baseline.chrome as Json).input as Bounds | null)?.height ?? null;
  const composerViewportHeight =
    typeof agentChatComposerScroll?.viewportHeightPx === "number"
      ? agentChatComposerScroll.viewportHeightPx
      : null;
  check(
    "agent-chat-one-line-composer-height-matches-main-menu-input",
    baselineInputHeight !== null &&
      composerViewportHeight !== null &&
      near(composerViewportHeight, baselineInputHeight),
    {
      actual: composerViewportHeight,
      expected: baselineInputHeight,
      tolerancePx: 1,
    },
  );
  agentChat.agentChatStateError = agentChatStateError;
  agentChat.agentChatInputLayout = agentChatInputLayout;
  agentChat.agentChatComposerScroll = agentChatComposerScroll;
  receipt.surfaces.agentChat = agentChat;
  assertCanonical("agent-chat", agentChat, baseline, true);

  await resetToVisibleScriptList(driver);
  // A bare-space launcher filter is the production Today/Day Page route. It
  // exercises the real filter-change transition without relying on the
  // timing-sensitive hold gesture, which is covered by dedicated Day probes.
  driver.setFilter(" ");
  await waitForSurface(driver, "DayPage");
  const dayPage = await snapshot(
    driver,
    "day-page",
    snapshotExpectations.dayPage,
    false,
  );
  receipt.surfaces.dayPage = dayPage;
  assertContextOnly("day-page", dayPage, baseline);

  await resetToVisibleScriptList(driver);
  driver.send({
    type: "triggerBuiltin",
    builtinId: "builtin/setup-permissions",
  });
  await waitForSurface(driver, "PermissionsWizard");
  const permissions = await snapshot(
    driver,
    "permissions-wizard",
    snapshotExpectations.permissionsWizard,
    false,
  );
  receipt.surfaces.permissionsWizard = permissions;
  assertContextOnly("permissions-wizard", permissions, baseline);

  driver.send({ type: "openConfirmPrompt" });
  await waitForSurface(driver, "ConfirmPrompt");
  const confirm = await snapshot(
    driver,
    "confirm-prompt",
    snapshotExpectations.confirmPrompt,
    false,
  );
  receipt.surfaces.confirmPrompt = confirm;
  assertContextOnly("confirm-prompt", confirm, baseline);
  check("probe-completed", true);
} catch (error) {
  receipt.error = String(error);
  check("probe-completed", false, { error: String(error) });
} finally {
  if (driver !== null) {
    try {
      await driver.close();
    } catch (error) {
      check("driver-closed", false, { error: String(error) });
    }
  }
}

receipt.checks = checks;
receipt.status =
  checks.length > 0 && checks.every((entry) => entry.pass) ? "green" : "red";
receipt.sessionDir = driver?.sessionDir ?? null;
writeFileSync(
  join(outputDir, "receipt.json"),
  `${JSON.stringify(receipt, null, 2)}\n`,
);

console.log(JSON.stringify(receipt, null, 2));
process.exit(receipt.status === "green" ? 0 : 1);
