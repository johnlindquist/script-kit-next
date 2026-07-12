#!/usr/bin/env bun
/**
 * Capture the deterministic, read-only GPUI state/layout receipt paired with
 * design/mockups/screens/agent-chat/reference/agent-chat@2x.png.
 *
 * Before declaring readiness, the probe normalizes the fixture's composer to
 * its canonical empty value. This protects the receipt from an unrelated
 * native keystroke landing while the driver-owned debug window becomes key.
 * Readiness is then defined as three consecutive identical getAgentChatState +
 * getLayoutInfo snapshots after recursively removing protocol-only volatile
 * fields. The final stable sample becomes the receipt, so no scroll, expansion,
 * selection, or other mutation occurs after the fixture is ready.
 *
 * Usage:
 *   SCRIPT_KIT_GPUI_BINARY=target-agent/artifacts/design-agent-chat/script-kit-gpui \
 *     bun scripts/agentic/agent-chat-design-reference-receipt.ts
 *
 * Optional:
 *   PROBE_RECEIPT=/tmp/agent-chat-runtime.json bun scripts/agentic/agent-chat-design-reference-receipt.ts
 */
import { createHash } from "node:crypto";
import {
  copyFileSync,
  createReadStream,
  mkdirSync,
  readFileSync,
  renameSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { dirname, isAbsolute, relative, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver.ts";
import {
  createOpaqueDesignCaptureHome,
  opaqueDesignCaptureEnv,
} from "./design-capture-environment.ts";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const DEFAULT_BINARY = resolve(
  PROJECT_ROOT,
  "target-agent/artifacts/design-agent-chat/script-kit-gpui",
);
const RECEIPT_PATH = resolve(
  PROJECT_ROOT,
  process.env.PROBE_RECEIPT ??
    "design/mockups/screens/agent-chat/reference/agent-chat-runtime-receipt.json",
);
const SCREENSHOT_PATH = resolve(
  PROJECT_ROOT,
  process.env.PROBE_SCREENSHOT ??
    "design/mockups/screens/agent-chat/reference/agent-chat@2x.png",
);
const FIDELITY_MANIFEST_PATH = resolve(
  PROJECT_ROOT,
  "design/mockups/screens/agent-chat/fidelity-manifest.json",
);
const REQUIRED_PAINT_SELECTOR_COUNT = 13;
const REQUIRED_SCREENSHOT_WIDTH = 1500;
const REQUIRED_SCREENSHOT_HEIGHT = 960;
const CAPTURE_BRACKET_ATTEMPTS = 4;
const TARGET = { type: "kind", kind: "main", index: 0 } as const;

const REQUIRED_COMPONENT_BOUNDS: Record<
  string,
  { x: number; y: number; width: number; height: number }
> = {
  Window: { x: 0, y: 0, width: 750, height: 480 },
  MainViewHeader: { x: 0, y: 0, width: 750, height: 58 },
  MainViewContextZone: { x: -6, y: 4, width: 762, height: 22 },
  MainViewInput: { x: 2, y: 28, width: 746, height: 26 },
  MainViewMain: { x: 0, y: 58, width: 750, height: 422 },
  AgentChatConversation: { x: 0, y: 58, width: 750, height: 416.4 },
  AgentChatTranscript: { x: 1, y: 59, width: 748, height: 382.5 },
  MainViewFooter: { x: 0, y: 448, width: 750, height: 32 },
};

// These fields identify one RPC sample rather than the UI state it observed.
// Removing them is also what makes readiness settling meaningful.
const VOLATILE_PROTOCOL_KEYS = new Set([
  "requestId",
  "timestamp",
  "generatedAt",
  "capturedAt",
  "sessionId",
  "sessionDir",
  "pid",
]);

function sanitizeString(value: string): string {
  return value
    .replaceAll(PROJECT_ROOT, "<repo>")
    .replace(/\/tmp\/sk-driver-sessions\/[^\s"']+/g, "<driver-session>")
    .replace(/drv-\d+-\d+/g, "<request-id>")
    .replace(/\b\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})\b/g, "<timestamp>");
}

function sanitizeProtocolValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sanitizeProtocolValue);
  if (typeof value === "string") return sanitizeString(value);
  if (value === null || typeof value !== "object") return value;

  const sanitized: Json = {};
  for (const [key, nested] of Object.entries(value as Json)) {
    if (VOLATILE_PROTOCOL_KEYS.has(key)) continue;
    sanitized[key] = sanitizeProtocolValue(nested);
  }
  return sanitized;
}

function asJson(value: unknown): Json {
  return value !== null && typeof value === "object" ? (value as Json) : {};
}

function protocolState(response: Json): Json {
  return asJson(sanitizeProtocolValue(response.state ?? response));
}

function protocolLayout(response: Json): Json {
  return asJson(sanitizeProtocolValue(response.info ?? response.layout ?? response));
}

async function readState(driver: Driver): Promise<Json> {
  const stateResponse = await driver.request(
    { type: "getAgentChatState", target: TARGET },
    { expect: "agent_chatStateResult", timeoutMs: 10_000 },
  );
  return protocolState(stateResponse);
}

async function readLayout(driver: Driver): Promise<Json> {
  const layoutResponse = await driver.getLayoutInfo(
    { target: TARGET },
    { timeoutMs: 10_000 },
  );
  return protocolLayout(layoutResponse);
}

function stableJson(value: unknown): string {
  return JSON.stringify(value);
}

function layoutStabilityValue(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(layoutStabilityValue);
  if (value === null || typeof value !== "object") return value;
  const stable: Json = {};
  for (const [key, nested] of Object.entries(value as Json)) {
    if (key === "measurementFrameGeneration") continue;
    stable[key] = layoutStabilityValue(nested);
  }
  return stable;
}

function stableLayoutJson(layout: Json): string {
  return stableJson(layoutStabilityValue(layout));
}

function layoutFrameGeneration(
  layout: Json,
  requiredSelectorIds: string[],
): { valid: boolean; generations: number[]; generation: number | null } {
  const components = Array.isArray(layout.components) ? layout.components : [];
  const generations: number[] = [];
  let valid = true;
  for (const selectorId of requiredSelectorIds) {
    const matches = components.filter(
      (component: Json) => component.name === selectorId,
    );
    if (matches.length !== 1) {
      valid = false;
      continue;
    }
    const generation = matches[0].measurementFrameGeneration;
    if (!Number.isSafeInteger(generation) || generation <= 0) {
      valid = false;
      continue;
    }
    generations.push(generation);
  }
  const unique = [...new Set(generations)];
  valid = valid && generations.length === requiredSelectorIds.length && unique.length === 1;
  return { valid, generations: unique, generation: valid ? unique[0] : null };
}

async function collectAtomicSnapshot(
  driver: Driver,
  requiredSelectorIds: string[],
): Promise<Json> {
  const layoutBefore = await readLayout(driver);
  const state = await readState(driver);
  const layoutAfter = await readLayout(driver);
  const beforeFrame = layoutFrameGeneration(layoutBefore, requiredSelectorIds);
  const afterFrame = layoutFrameGeneration(layoutAfter, requiredSelectorIds);
  const layoutStable = stableLayoutJson(layoutBefore) === stableLayoutJson(layoutAfter);
  const frameCoherent = beforeFrame.valid && afterFrame.valid &&
    Number(afterFrame.generation) >= Number(beforeFrame.generation);
  return {
    state,
    rawLayout: asJson(layoutStabilityValue(layoutAfter)),
    atomic: {
      valid: layoutStable && frameCoherent,
      layoutStable,
      frameCoherent,
    },
  };
}

async function sha256File(path: string): Promise<string> {
  return await new Promise((resolveHash, rejectHash) => {
    const hash = createHash("sha256");
    const stream = createReadStream(path);
    stream.on("data", (chunk) => hash.update(chunk));
    stream.on("error", rejectHash);
    stream.on("end", () => resolveHash(hash.digest("hex")));
  });
}

function pngDimensions(path: string): { width: number; height: number } {
  const bytes = readFileSync(path);
  const pngSignature = "89504e470d0a1a0a";
  if (bytes.length < 24 || bytes.subarray(0, 8).toString("hex") !== pngSignature) {
    throw new Error(`Screenshot is not a valid PNG: ${path}`);
  }
  return { width: bytes.readUInt32BE(16), height: bytes.readUInt32BE(20) };
}

function imageStandardDeviation(path: string): number {
  const result = Bun.spawnSync(
    ["magick", path, "-colorspace", "RGB", "-format", "%[fx:standard_deviation]", "info:"],
    { stdout: "pipe", stderr: "pipe" },
  );
  if (result.exitCode !== 0) {
    throw new Error(`ImageMagick pixel audit failed: ${result.stderr.toString().trim()}`);
  }
  const value = Number(result.stdout.toString().trim());
  if (!Number.isFinite(value)) {
    throw new Error(`ImageMagick returned an invalid standard deviation for ${path}`);
  }
  return value;
}

async function resolveCaptureIdentity(driver: Driver): Promise<Json> {
  const response = asJson(await driver.request(
    { type: "inspectAutomationWindow", target: TARGET, hiDpi: false, probes: [] },
    { expect: "automationInspectResult", timeoutMs: 10_000 },
  ));
  return {
    windowId: response.windowId ?? null,
    windowKind: response.windowKind ?? null,
    osWindowId: response.osWindowId ?? null,
    pid: response.pid ?? null,
    resolvedBounds: response.resolvedBounds ?? null,
    screenshotWidth: response.screenshotWidth ?? null,
    screenshotHeight: response.screenshotHeight ?? null,
    warnings: sanitizeProtocolValue(response.warnings ?? []),
  };
}

function captureNativeWindowWithMacOsTool(
  path: string,
  captureIdentity: Json,
): void {
  const nativeWindowId = numberAt(captureIdentity.osWindowId);
  if (!Number.isSafeInteger(nativeWindowId) || Number(nativeWindowId) <= 0) {
    throw new Error(
      "resolver-driven macOS capture fallback requires a positive inspectAutomationWindow osWindowId",
    );
  }
  const capture = Bun.spawnSync(
    [
      "/usr/sbin/screencapture",
      "-x",
      "-o",
      `-l${nativeWindowId}`,
      path,
    ],
    { stdout: "pipe", stderr: "pipe" },
  );
  if (capture.exitCode !== 0) {
    throw new Error(
      `macOS native-window capture failed: ${sanitizeString(capture.stderr.toString().trim())}`,
    );
  }
}

async function captureScreenshotTo(
  driver: Driver,
  path: string,
  captureIdentity: Json,
): Promise<Json> {
  const response = await driver.captureScreenshot({
    hiDpi: true,
    target: TARGET,
    savePath: path,
    timeoutMs: 10_000,
  });
  const driverError = typeof response.error === "string" && response.error.length > 0
    ? response.error
    : null;
  let source = "captureScreenshot resolver-driven OS window capture";
  if (driverError) {
    if (!driverError.includes("blank/black image")) {
      throw new Error(`captureScreenshot failed: ${driverError}`);
    }
    captureNativeWindowWithMacOsTool(path, captureIdentity);
    source = "inspectAutomationWindow osWindowId + macOS screencapture";
  }
  const dimensions = pngDimensions(path);
  return {
    width: driverError ? dimensions.width : response.width ?? null,
    height: driverError ? dimensions.height : response.height ?? null,
    pngWidth: dimensions.width,
    pngHeight: dimensions.height,
    standardDeviation: imageStandardDeviation(path),
    sha256: await sha256File(path),
    source,
    driverCaptureError: driverError,
    nativeWindowId: captureIdentity.osWindowId ?? null,
    pid: captureIdentity.pid ?? null,
  };
}

type CaptureBracket = {
  valid: boolean;
  attempt: number;
  state: Json;
  rawLayout: Json;
  frameEvidence: Json;
  screenshotEvidence: Json;
  reasons: string[];
};

function screenCaptureEnvironment(): Json {
  if (process.platform !== "darwin") {
    return { platform: process.platform, screenLocked: null, source: "unsupported" };
  }
  const inspection = Bun.spawnSync(
    ["/usr/sbin/ioreg", "-l", "-d", "2"],
    { stdout: "pipe", stderr: "pipe" },
  );
  if (inspection.exitCode !== 0) {
    return {
      platform: process.platform,
      screenLocked: null,
      source: "ioreg",
      error: sanitizeString(inspection.stderr.toString().trim()),
    };
  }
  const output = inspection.stdout.toString();
  const screenLocked = output.includes('"CGSSessionScreenIsLocked"=Yes')
    ? true
    : output.includes('"CGSSessionScreenIsLocked"=No')
    ? false
    : null;
  return { platform: process.platform, screenLocked, source: "ioreg.IOConsoleUsers" };
}

async function collectCaptureBracket(
  driver: Driver,
  requiredSelectorIds: string[],
  captureIdentity: Json,
  captureEnvironment: Json,
): Promise<CaptureBracket> {
  mkdirSync(dirname(SCREENSHOT_PATH), { recursive: true });
  const diagnosticDir = resolve(
    PROJECT_ROOT,
    ".test-output/design-fidelity/agent-chat",
  );
  const diagnosticA = resolve(diagnosticDir, "gpui-capture-a@2x.png");
  const diagnosticB = resolve(diagnosticDir, "gpui-capture-b@2x.png");
  rmSync(diagnosticA, { force: true });
  rmSync(diagnosticB, { force: true });
  let latestState: Json = {};
  let latestRawLayout: Json = {};
  let last: CaptureBracket = {
    valid: false,
    attempt: 0,
    state: {},
    rawLayout: {},
    frameEvidence: {},
    screenshotEvidence: {},
    reasons: ["capture bracket was not attempted"],
  };

  if (captureEnvironment.screenLocked === true) {
    const layoutBefore = await readLayout(driver);
    const state = await readState(driver);
    const layoutAfter = await readLayout(driver);
    const beforeFrame = layoutFrameGeneration(layoutBefore, requiredSelectorIds);
    const afterFrame = layoutFrameGeneration(layoutAfter, requiredSelectorIds);
    return {
      valid: false,
      attempt: 0,
      state,
      rawLayout: layoutAfter,
      frameEvidence: {
        generation: afterFrame.generation,
        layoutGenerations: [beforeFrame.generation, afterFrame.generation],
        stateStable: true,
        layoutStable:
          stableLayoutJson(layoutBefore) === stableLayoutJson(layoutAfter),
        frameCoherent:
          beforeFrame.valid && afterFrame.valid &&
          Number(afterFrame.generation) >= Number(beforeFrame.generation),
        bracket: "L0-S0-L1 (OS capture blocked before screenshot)",
      },
      screenshotEvidence: {
        target: TARGET,
        source: "blocked-before-os-capture",
        pixelAudit: { blank: true },
        captureIdentity: {
          windowId: captureIdentity.windowId ?? null,
          windowKind: captureIdentity.windowKind ?? null,
          osWindowId: captureIdentity.osWindowId ?? null,
          pid: captureIdentity.pid ?? null,
          resolvedBounds: captureIdentity.resolvedBounds ?? null,
        },
      },
      reasons: [
        "macOS console session is locked; OS compositor capture is unavailable",
      ],
    };
  }

  for (let attempt = 1; attempt <= CAPTURE_BRACKET_ATTEMPTS; attempt++) {
    const suffix = `${process.pid}-${Date.now()}-${attempt}`;
    const captureAPath = `${SCREENSHOT_PATH}.capture-a-${suffix}.png`;
    const captureBPath = `${SCREENSHOT_PATH}.capture-b-${suffix}.png`;
    try {
      const layout0 = await readLayout(driver);
      latestRawLayout = layout0;
      const state0 = await readState(driver);
      latestState = state0;
      const layout1 = await readLayout(driver);
      latestRawLayout = layout1;
      const captureA = await captureScreenshotTo(
        driver,
        captureAPath,
        captureIdentity,
      );
      const layout2 = await readLayout(driver);
      latestRawLayout = layout2;
      const captureB = await captureScreenshotTo(
        driver,
        captureBPath,
        captureIdentity,
      );
      const layout3 = await readLayout(driver);
      latestRawLayout = layout3;
      const state1 = await readState(driver);
      latestState = state1;
      const layout4 = await readLayout(driver);
      latestRawLayout = layout4;

      const layouts = [layout0, layout1, layout2, layout3, layout4];
      const frames = layouts.map((layout) =>
        layoutFrameGeneration(layout, requiredSelectorIds)
      );
      const generations = frames.map((frame) => frame.generation);
      const validGenerations = frames.every((frame) => frame.valid);
      const generationsMonotonic = validGenerations && generations.every(
        (generation, index) =>
          index === 0 || Number(generation) >= Number(generations[index - 1]),
      );
      const layoutStable = layouts.every(
        (layout) => stableLayoutJson(layout) === stableLayoutJson(layout0),
      );
      const stateStable = stableJson(state0) === stableJson(state1);
      const capturesIdentical = captureA.sha256 === captureB.sha256;
      const nonBlank = captureA.standardDeviation > 0.001 &&
        captureB.standardDeviation > 0.001;
      const dimensionsValid = [captureA, captureB].every(
        (capture) =>
          capture.width === REQUIRED_SCREENSHOT_WIDTH &&
          capture.height === REQUIRED_SCREENSHOT_HEIGHT &&
          capture.pngWidth === REQUIRED_SCREENSHOT_WIDTH &&
          capture.pngHeight === REQUIRED_SCREENSHOT_HEIGHT,
      );
      const reasons = [
        validGenerations && generationsMonotonic
          ? ""
          : "paint frame generations were invalid or moved backwards",
        layoutStable ? "" : "layout changed inside capture bracket",
        stateStable ? "" : "Agent Chat state changed inside capture bracket",
        capturesIdentical ? "" : "consecutive OS captures were not byte-identical",
        nonBlank ? "" : "OS capture was blank or effectively uniform",
        dimensionsValid ? "" : "OS capture dimensions were not exactly 1500x960",
      ].filter(Boolean);
      const valid = reasons.length === 0;
      last = {
        valid,
        attempt,
        state: state1,
        rawLayout: layout4,
        frameEvidence: {
          generation: validGenerations ? generations.at(-1) : null,
          layoutGenerations: generations,
          stateStable,
          layoutStable,
          frameCoherent: validGenerations && generationsMonotonic,
          bracket: "L0-S0-L1-shotA-L2-shotB-L3-S1-L4",
        },
        screenshotEvidence: {
          path: repoRelativePath(SCREENSHOT_PATH),
          sha256: captureB.sha256,
          duplicateCaptureSha256: captureA.sha256,
          capturesIdentical,
          pixelAudit: {
            blank: !nonBlank,
            standardDeviation: captureB.standardDeviation,
          },
          width: captureB.pngWidth,
          height: captureB.pngHeight,
          scaleFactor: 2,
          target: TARGET,
          source: captureB.source,
          driverCaptureError: captureB.driverCaptureError,
          captureIdentity: {
            windowId: captureIdentity.windowId ?? null,
            windowKind: captureIdentity.windowKind ?? null,
            osWindowId: captureIdentity.osWindowId ?? null,
            pid: captureIdentity.pid ?? null,
            resolvedBounds: captureIdentity.resolvedBounds ?? null,
          },
        },
        reasons,
      };
      if (valid) {
        renameSync(captureBPath, SCREENSHOT_PATH);
        rmSync(diagnosticA, { force: true });
        rmSync(diagnosticB, { force: true });
        rmSync(captureAPath, { force: true });
        rmSync(captureBPath, { force: true });
        return last;
      }
      mkdirSync(diagnosticDir, { recursive: true });
      copyFileSync(captureAPath, diagnosticA);
      copyFileSync(captureBPath, diagnosticB);
      last.screenshotEvidence.diagnosticPaths = [
        repoRelativePath(diagnosticA),
        repoRelativePath(diagnosticB),
      ];
      last.screenshotEvidence.diagnosticCaptureAttempt = attempt;
    } catch (error) {
      last = {
        ...last,
        attempt,
        valid: false,
        state: latestState,
        rawLayout: latestRawLayout,
        reasons: [error instanceof Error ? error.message : String(error)],
      };
    } finally {
      rmSync(captureAPath, { force: true });
      rmSync(captureBPath, { force: true });
    }
    await Bun.sleep(100);
  }
  return last;
}

function repoRelativePath(path: string): string {
  const candidate = relative(PROJECT_ROOT, path);
  return !candidate.startsWith("..") && !isAbsolute(candidate)
    ? candidate
    : sanitizeString(path);
}

function componentByName(layout: Json, name: string): Json | null {
  const components = Array.isArray(layout.components) ? layout.components : [];
  return components.find((component: Json) => component.name === name) ?? null;
}

function manifestGpuiSelectorIds(): string[] {
  const manifest = asJson(JSON.parse(readFileSync(FIDELITY_MANIFEST_PATH, "utf8")));
  const elements = Array.isArray(manifest.elements) ? manifest.elements : [];
  return elements.map((element: Json) =>
    typeof element.gpuiId === "string" ? element.gpuiId : ""
  );
}

function duplicateStrings(values: string[]): string[] {
  return [...new Set(values.filter((value, index) => values.indexOf(value) !== index))];
}

type PaintSelectorValidation = {
  requiredSelectorIds: string[];
  observedSelectorIds: string[];
  missingSelectorIds: string[];
  duplicateSelectorIds: string[];
  invalidProvenanceSelectorIds: string[];
  invalidCoordinateSpaceSelectorIds: string[];
  invalidVisibleBoundsSelectorIds: string[];
  invalidClipBoundsSelectorIds: string[];
  invalidFrameGenerationSelectorIds: string[];
  uniqueFrameGenerations: number[];
  frameGeneration: number | null;
  frameCoherent: boolean;
};

function numberAt(value: unknown): number | null {
  return typeof value === "number" && Number.isFinite(value) ? value : null;
}

const failures: Json[] = [];
const assertions: Json[] = [];

function checkEqual(name: string, actual: unknown, expected: unknown): void {
  const passed = Object.is(actual, expected);
  assertions.push({ name, passed, expected, actual: actual ?? null });
  if (!passed) failures.push({ name, expected, actual: actual ?? null });
}

function checkApprox(
  name: string,
  actualValue: unknown,
  expected: number,
  tolerance = 0.01,
): void {
  const actual = numberAt(actualValue);
  const delta = actual === null ? null : Math.abs(actual - expected);
  const passed = delta !== null && delta <= tolerance;
  assertions.push({ name, passed, expected, actual, delta, tolerance });
  if (!passed) failures.push({ name, expected, actual, delta, tolerance });
}

function checkEmptyStringList(name: string, actual: string[]): void {
  const passed = actual.length === 0;
  assertions.push({ name, passed, expected: [], actual });
  if (!passed) failures.push({ name, expected: [], actual });
}

function checkPaintSelectors(
  layout: Json,
  requiredSelectorIds: string[],
): PaintSelectorValidation {
  const components = Array.isArray(layout.components) ? layout.components : [];
  const missingSelectorIds: string[] = [];
  const duplicateSelectorIds: string[] = [];
  const invalidProvenanceSelectorIds: string[] = [];
  const invalidCoordinateSpaceSelectorIds: string[] = [];
  const invalidVisibleBoundsSelectorIds: string[] = [];
  const invalidClipBoundsSelectorIds: string[] = [];
  const invalidFrameGenerationSelectorIds: string[] = [];
  const frameGenerations: number[] = [];
  const observedSelectorIds: string[] = [];

  checkEqual(
    "manifest.paintSelectorCount",
    requiredSelectorIds.length,
    REQUIRED_PAINT_SELECTOR_COUNT,
  );
  checkEmptyStringList(
    "manifest.paintSelectorIds.valid",
    requiredSelectorIds.includes("") ? ["<missing-gpuiId>"] : [],
  );
  checkEmptyStringList(
    "manifest.paintSelectorIds.unique",
    duplicateStrings(requiredSelectorIds),
  );

  for (const selectorId of requiredSelectorIds) {
    if (!selectorId) continue;
    const matches = components.filter(
      (component: Json) => component.name === selectorId,
    );
    if (matches.length === 0) missingSelectorIds.push(selectorId);
    if (matches.length > 0) observedSelectorIds.push(selectorId);
    if (matches.length > 1) duplicateSelectorIds.push(selectorId);

    checkEqual(`layout.paintSelector.${selectorId}.matchCount`, matches.length, 1);
    if (matches.length !== 1) continue;

    const component = matches[0];
    if (component.measurementProvenance !== "paint-time") {
      invalidProvenanceSelectorIds.push(selectorId);
    }
    if (component.coordinateSpace !== "window") {
      invalidCoordinateSpaceSelectorIds.push(selectorId);
    }
    const visibleBounds = asJson(component.visibleBounds);
    const clipBounds = asJson(component.clipBounds);
    const visibleBoundsValid = ["x", "y", "width", "height"].every(
      (edge) => numberAt(visibleBounds[edge]) !== null,
    );
    const clipBoundsValid = ["x", "y", "width", "height"].every(
      (edge) => numberAt(clipBounds[edge]) !== null,
    );
    if (!visibleBoundsValid) invalidVisibleBoundsSelectorIds.push(selectorId);
    if (!clipBoundsValid) invalidClipBoundsSelectorIds.push(selectorId);
    const frameGeneration = component.measurementFrameGeneration;
    if (!Number.isSafeInteger(frameGeneration) || frameGeneration <= 0) {
      invalidFrameGenerationSelectorIds.push(selectorId);
    } else {
      frameGenerations.push(frameGeneration);
    }
    checkEqual(
      `layout.paintSelector.${selectorId}.measurementProvenance`,
      component.measurementProvenance,
      "paint-time",
    );
    checkEqual(
      `layout.paintSelector.${selectorId}.coordinateSpace`,
      component.coordinateSpace,
      "window",
    );
    checkEqual(
      `layout.paintSelector.${selectorId}.measurementFrameGeneration.valid`,
      Number.isSafeInteger(frameGeneration) && frameGeneration > 0,
      true,
    );
    checkEqual(
      `layout.paintSelector.${selectorId}.visibleBounds.valid`,
      visibleBoundsValid,
      true,
    );
    checkEqual(
      `layout.paintSelector.${selectorId}.clipBounds.valid`,
      clipBoundsValid,
      true,
    );
  }

  // Keep the exact missing/invalid selector sets in the receipt. A binary
  // without the paint instrumentation must fail closed with all required IDs named.
  checkEmptyStringList("layout.paintSelectors.missingIds", missingSelectorIds);
  checkEmptyStringList("layout.paintSelectors.duplicateIds", duplicateSelectorIds);
  checkEmptyStringList(
    "layout.paintSelectors.invalidProvenanceIds",
    invalidProvenanceSelectorIds,
  );
  checkEmptyStringList(
    "layout.paintSelectors.invalidCoordinateSpaceIds",
    invalidCoordinateSpaceSelectorIds,
  );
  checkEmptyStringList(
    "layout.paintSelectors.invalidFrameGenerationIds",
    invalidFrameGenerationSelectorIds,
  );
  checkEmptyStringList(
    "layout.paintSelectors.invalidVisibleBoundsIds",
    invalidVisibleBoundsSelectorIds,
  );
  checkEmptyStringList(
    "layout.paintSelectors.invalidClipBoundsIds",
    invalidClipBoundsSelectorIds,
  );
  const uniqueFrameGenerations = [...new Set(frameGenerations)];
  const frameCoherent =
    frameGenerations.length === requiredSelectorIds.length &&
    uniqueFrameGenerations.length === 1;
  checkEqual("layout.paintSelectors.frameCoherent", frameCoherent, true);

  return {
    requiredSelectorIds,
    observedSelectorIds,
    missingSelectorIds,
    duplicateSelectorIds,
    invalidProvenanceSelectorIds,
    invalidCoordinateSpaceSelectorIds,
    invalidVisibleBoundsSelectorIds,
    invalidClipBoundsSelectorIds,
    invalidFrameGenerationSelectorIds,
    uniqueFrameGenerations,
    frameGeneration: frameCoherent ? uniqueFrameGenerations[0] ?? null : null,
    frameCoherent,
  };
}

function checkPinnedFixture(
  state: Json,
  layout: Json,
  requiredPaintSelectorIds: string[],
): PaintSelectorValidation {
  checkEqual("target.windowId", state.resolvedTarget?.windowId, "main");
  checkEqual("target.windowKind", state.resolvedTarget?.windowKind, "main");
  checkEqual("state.schemaVersion", state.schemaVersion, 4);
  checkEqual("state.status", state.status, "idle");
  checkEqual("state.uiVariant", state.uiVariant, "standard");
  checkEqual("state.inputText", state.inputText, "");
  checkEqual("state.cursorIndex", state.cursorIndex, 0);
  checkEqual("state.hasSelection", state.hasSelection, false);
  checkEqual("state.messageCount", state.messageCount, 21);
  checkEqual("state.contextChipCount", state.contextChipCount, 0);
  checkEqual("state.contextReady", state.contextReady, true);
  checkEqual("state.hasPendingPermission", state.hasPendingPermission, false);
  checkEqual("state.transcriptScroll.rowCount", state.transcriptScroll?.rowCount, 22);
  checkEqual(
    "state.transcriptScroll.scrollTopItem",
    state.transcriptScroll?.scrollTopItem,
    state.transcriptScroll?.rowCount,
  );
  checkApprox("state.transcriptScroll.scrollTopOffsetPx", state.transcriptScroll?.scrollTopOffsetPx, 0);
  checkApprox(
    "state.transcriptScroll.atBottom",
    state.transcriptScroll?.scrollTopPx,
    Number(state.transcriptScroll?.maxScrollTopPx),
  );
  checkEqual(
    "state.transcriptScroll.measurementSource",
    state.transcriptScroll?.measurementSource,
    "listState",
  );
  checkApprox("state.composerScroll.viewportHeightPx", state.composerScroll?.viewportHeightPx, 26);
  checkEqual("state.composerScroll.canScrollY", state.composerScroll?.canScrollY, false);

  checkApprox("layout.windowWidth", layout.windowWidth, 750);
  checkApprox("layout.windowHeight", layout.windowHeight, 480);
  checkEqual("layout.promptType", layout.promptType, "agentChatChat");

  const componentNames = Array.isArray(layout.components)
    ? layout.components
        .filter(
          (component: Json) =>
            component.measurementProvenance !== "paint-time" &&
            !requiredPaintSelectorIds.includes(String(component.name ?? "")),
        )
        .map((component: Json) => component.name)
    : [];
  const expectedComponentNames = Object.keys(REQUIRED_COMPONENT_BOUNDS);
  checkEqual(
    "layout.componentNameSet",
    JSON.stringify([...componentNames].sort()),
    JSON.stringify([...expectedComponentNames].sort()),
  );
  for (const [componentName, expected] of Object.entries(REQUIRED_COMPONENT_BOUNDS)) {
    const component = componentByName(layout, componentName);
    for (const edge of ["x", "y", "width", "height"] as const) {
      checkApprox(
        `layout.${componentName}.${edge}`,
        component?.bounds?.[edge],
        expected[edge],
      );
    }
  }

  checkApprox(
    "crossCheck.transcriptViewportHeight",
    state.transcriptScroll?.viewportHeightPx,
    Number(componentByName(layout, "AgentChatTranscript")?.bounds?.height),
  );

  return checkPaintSelectors(layout, requiredPaintSelectorIds);
}

function designFidelityTargetKind(value: unknown): unknown {
  return value === "main" ? "Main" : value;
}

const binaryPath = resolve(
  PROJECT_ROOT,
  process.env.SCRIPT_KIT_GPUI_BINARY ?? DEFAULT_BINARY,
);
let driver: Driver | null = null;
let captureHome: string | null = null;
let receipt: Json = {
  schemaVersion: 1,
  tool: "script-kit-devtools.agent-chat-design-reference",
  command: "capture-exact-state",
  screenId: "agent-chat",
  classification: "blocked-by-runtime",
  pass: false,
  binary: {
    path: repoRelativePath(binaryPath),
    sha256: null,
  },
  fixture: {
    command: "openAgentChatKitchenSinkFixture",
    presentation: "standard-idle-collapsed-at-transcript-bottom",
    preReadyNormalizationCommands: ["setAgentChatInput(empty)"],
    mutationCommandsAfterReady: 0,
  },
  safety: {
    preparedFixtureHome: true,
    providerRequired: false,
    liveThreadRequired: false,
    fixtureOnly: true,
    environmentOverrides: {
      // Driver-owned debug windows do not inherit the production NSPanel
      // collection/animation/restorable flags. This matches the existing
      // Agent Chat layout probe and changes assertion behavior only.
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
      SCRIPT_KIT_DEBUG_NO_GLASS: "1",
    },
  },
  settling: {
    observation: "sanitized getLayoutInfo + getAgentChatState + getLayoutInfo atomic bracket",
    consecutiveIdenticalSamplesRequired: 3,
    intervalMs: 100,
    timeoutMs: 10_000,
    volatileKeysRemoved: [...VOLATILE_PROTOCOL_KEYS],
    preReadyObservationSettled: false,
    settled: false,
  },
  assertions,
  failures,
};

try {
  const requiredPaintSelectorIds = manifestGpuiSelectorIds();
  receipt.binary.sha256 = await sha256File(binaryPath);
  captureHome = createOpaqueDesignCaptureHome("design-capture-agent-chat-");
  driver = await Driver.launch({
    binary: binaryPath,
    sessionName: "agent-chat-design-reference-receipt",
    readyTimeoutMs: 30_000,
    defaultTimeoutMs: 10_000,
    env: {
      ...opaqueDesignCaptureEnv(captureHome),
      SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
    },
  });

  const openResponse = await driver.request(
    { type: "openAgentChatKitchenSinkFixture" },
    { expect: "externalCommandResult", timeoutMs: 10_000 },
  );
  const opened = asJson(sanitizeProtocolValue(openResponse));
  checkEqual("fixture.open.command", opened.command, "openAgentChatKitchenSinkFixture");
  checkEqual("fixture.open.ok", opened.ok, true);

  // Let the newly-keyed native window reach a stable state before the one
  // permitted pre-ready normalization. A transient native keystroke can land
  // during this interval when other local automation or the user is active.
  const preReadyObservation = await driver.waitForSettle({
    samples: 3,
    intervalMs: 100,
    timeoutMs: 10_000,
    probe: () => collectAtomicSnapshot(driver!, requiredPaintSelectorIds),
  });
  receipt.settling.preReadyObservationSettled = preReadyObservation.settled;
  checkEqual("fixture.preReadyObservationSettled", preReadyObservation.settled, true);

  const normalizationResponse = await driver.request(
    { type: "setAgentChatInput", text: "", submit: false },
    { expect: "externalCommandResult", timeoutMs: 10_000 },
  );
  const normalized = asJson(sanitizeProtocolValue(normalizationResponse));
  checkEqual("fixture.normalize.command", normalized.command, "setAgentChatInput");
  checkEqual("fixture.normalize.ok", normalized.ok, true);

  const settled = await driver.waitForSettle({
    samples: 3,
    intervalMs: 100,
    timeoutMs: 10_000,
    probe: () => collectAtomicSnapshot(driver!, requiredPaintSelectorIds),
  });
  receipt.settling.settled = settled.settled;
  if (!settled.settled) {
    failures.push({
      name: "fixture.ready",
      expected: true,
      actual: false,
      reason: "sanitized state/layout did not become stable before the deadline",
    });
  } else {
    assertions.push({ name: "fixture.ready", passed: true, expected: true, actual: true });
  }
  checkEqual("fixture.ready.atomic", settled.lastState.atomic?.valid, true);

  const captureIdentity = await resolveCaptureIdentity(driver);
  checkEqual("captureIdentity.windowId", captureIdentity.windowId, "main");
  checkEqual("captureIdentity.windowKind", captureIdentity.windowKind, "Main");
  checkEqual(
    "captureIdentity.osWindowId",
    Number.isSafeInteger(numberAt(captureIdentity.osWindowId)) &&
      Number(captureIdentity.osWindowId) > 0,
    true,
  );
  checkEqual(
    "captureIdentity.pid",
    Number.isSafeInteger(numberAt(captureIdentity.pid)) && Number(captureIdentity.pid) > 0,
    true,
  );
  checkApprox("captureIdentity.bounds.width", captureIdentity.resolvedBounds?.width, 750);
  checkApprox("captureIdentity.bounds.height", captureIdentity.resolvedBounds?.height, 480);
  const captureEnvironment = screenCaptureEnvironment();

  // Pair the final state/layout with two identical resolver-driven OS captures.
  // The bracket is entirely read-only and retries as a unit if any draw lands
  // between its observations.
  const captureBracket = await collectCaptureBracket(
    driver,
    requiredPaintSelectorIds,
    captureIdentity,
    captureEnvironment,
  );
  checkEqual("captureBracket.valid", captureBracket.valid, true);
  if (!captureBracket.valid) {
    failures.push({
      name: "captureBracket.reasons",
      expected: [],
      actual: captureBracket.reasons,
    });
  }
  const state = asJson(captureBracket.state);
  const rawLayout = asJson(captureBracket.rawLayout);
  const paintSelectorValidation = checkPinnedFixture(
    state,
    rawLayout,
    requiredPaintSelectorIds,
  );

  const width = numberAt(rawLayout.windowWidth) ?? 0;
  const height = numberAt(rawLayout.windowHeight) ?? 0;
  const runtimeTarget = asJson(state.resolvedTarget);
  // Normalize the protocol vocabulary to the identity keys consumed by the
  // design-fidelity comparator while retaining the exact runtime fields.
  const resolvedTarget = {
    ...runtimeTarget,
    automationId: runtimeTarget.windowId ?? null,
    targetKind: designFidelityTargetKind(
      runtimeTarget.windowKind ?? runtimeTarget.targetKind ?? null,
    ),
    surfaceKind: rawLayout.promptType ?? null,
  };
  const pass = failures.length === 0;
  receipt = {
    ...receipt,
    classification: pass ? "ok" : "reproduced",
    pass,
    opened,
    normalized,
    resolvedTarget,
    target: resolvedTarget,
    viewport: { width, height },
    windowRect: { x: 0, y: 0, width, height },
    measurementCoordinateSpace: "window",
    measurementEvidence: {
      source: "getLayoutInfo",
      transcriptViewportSource: state.transcriptScroll?.measurementSource ?? null,
      selectorManifest: repoRelativePath(FIDELITY_MANIFEST_PATH),
      requiredPaintSelectorCount: REQUIRED_PAINT_SELECTOR_COUNT,
      paintTimeElementBoundsAvailable:
        paintSelectorValidation.requiredSelectorIds.length ===
          REQUIRED_PAINT_SELECTOR_COUNT &&
        paintSelectorValidation.missingSelectorIds.length === 0 &&
        paintSelectorValidation.duplicateSelectorIds.length === 0 &&
        paintSelectorValidation.invalidProvenanceSelectorIds.length === 0 &&
        paintSelectorValidation.invalidCoordinateSpaceSelectorIds.length === 0 &&
        paintSelectorValidation.invalidVisibleBoundsSelectorIds.length === 0 &&
        paintSelectorValidation.invalidClipBoundsSelectorIds.length === 0 &&
        paintSelectorValidation.invalidFrameGenerationSelectorIds.length === 0 &&
        paintSelectorValidation.frameCoherent,
      ...paintSelectorValidation,
    },
    frameEvidence: captureBracket.frameEvidence,
    screenshotEvidence: captureBracket.screenshotEvidence,
    visualEvidence: {
      source: "os-window-capture",
      classification: captureBracket.valid ? "captured" : "blocked",
      captureKind: "resolver-targeted-window",
      countsAsOsScreenshotEvidence: captureBracket.valid,
      countsAsCompositorEvidence: captureBracket.valid,
      pixelAudit: captureBracket.screenshotEvidence.pixelAudit ?? { blank: true },
      captureIdentity: {
        target: TARGET,
        width: captureBracket.screenshotEvidence.width ?? null,
        height: captureBracket.screenshotEvidence.height ?? null,
        sha256: captureBracket.screenshotEvidence.sha256 ?? null,
      },
    },
    captureIdentity,
    screenCaptureEnvironment: captureEnvironment,
    state,
    rawLayout,
    assertions,
    failures,
  };
} catch (error) {
  failures.push({
    name: "probe.runtime",
    error: sanitizeString(error instanceof Error ? error.message : String(error)),
  });
  receipt = {
    ...receipt,
    classification: "blocked-by-runtime",
    pass: false,
    assertions,
    failures,
  };
} finally {
  if (driver) await driver.close();
  if (captureHome) rmSync(captureHome, { recursive: true, force: true });
  mkdirSync(dirname(RECEIPT_PATH), { recursive: true });
  writeFileSync(RECEIPT_PATH, `${JSON.stringify(receipt, null, 2)}\n`);
}

console.log(JSON.stringify({
  tool: receipt.tool,
  classification: receipt.classification,
  pass: receipt.pass,
  receipt: repoRelativePath(RECEIPT_PATH),
  binary: receipt.binary,
  viewport: receipt.viewport ?? null,
  failures: receipt.failures,
}, null, 2));

if (!receipt.pass) process.exitCode = 1;
