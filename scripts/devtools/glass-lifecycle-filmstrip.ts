#!/usr/bin/env bun

import { createHash } from "node:crypto";
import {
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { Driver, type Json } from "./driver.ts";
import {
  identityFromEnvironment,
  newRunId,
} from "./glass-evidence-contract.ts";
import { announceTestStatus } from "./test-status.ts";
import {
  computeLifecycleDisposition,
  expandCaptureBounds,
  filmstripVerdict,
  type LifecycleScenarioName,
  parseAnalysisMode,
  resolveScenarioNames,
  MAIN_EXIT_CAPTURE_PREROLL_MS,
  MAIN_EXIT_POST_HIDE_SETTLE_MS,
  SCENARIO_CAPTURE_DURATIONS_MS,
  type ScenarioProfile,
  type ScenarioTimingInterval,
  validateFilmstripCapture,
  validateScenarioTimingIntervals,
} from "./glass-lifecycle-filmstrip-contract.ts";
import {
  ACTIONS_GLASS_ENTRY_EXPECTATION,
  MAIN_GLASS_ENTRY_EXPECTATION,
  analyzeEntryFadeLog,
  analyzeEntryMotionEnvelope,
} from "./glass-entry-motion-contract.ts";
import {
  finishInterferenceMonitor,
  startInterferenceMonitor,
  waitForInterferenceReady,
} from "./glass-interference.ts";
import { classifyNativeInventory } from "./glass-topology-contract.ts";
import { requireValidatedHelper } from "./glass-native-helper-cache.ts";
import {
  classifyGlassObservation,
  type GlassObservationInput,
  validateOwnedRenderedFrames,
} from "./glass-observers.ts";
import os from "node:os";
import {
  type BoundarySnapshot,
  captureBoundarySnapshot,
  captureEdgeSnapshot,
  checkRuntimeContract,
  interferenceStatistics,
  parseMorphEnterLogs,
  probeGpuTelemetry,
  type ScenarioInterval,
  startSampler,
  summarizeTelemetry,
} from "../agentic/glass-system-telemetry.ts";
import { assertNoninteractiveVisualProbe } from "./lib/operator-safety.ts";

assertNoninteractiveVisualProbe("glass-lifecycle-filmstrip");

function arg(name: string, fallback?: string) {
  const index = process.argv.indexOf(name);
  return index >= 0 ? process.argv[index + 1] : fallback;
}

async function run(command: string[]) {
  const child = Bun.spawn(command, { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(child.stdout).text(),
    new Response(child.stderr).text(),
    child.exited,
  ]);
  return { stdout, stderr, exitCode };
}

async function waitForFile(path: string, timeoutMs = 3_000) {
  const started = performance.now();
  while (performance.now() - started < timeoutMs) {
    if (existsSync(path)) return JSON.parse(readFileSync(path, "utf8"));
    await Bun.sleep(10);
  }
  throw new Error(`timed out waiting for ${path}`);
}

async function waitForWindowCount(
  driver: Driver,
  kind: string,
  expected: number,
  timeoutMs = 3_000,
) {
  const started = performance.now();
  let last: Json = null;
  while (performance.now() - started < timeoutMs) {
    last = await driver.listAutomationWindows({ timeoutMs: 5_000 });
    const count = ((last?.windows ?? []) as Json[]).filter(
      (window) => window?.kind === kind && window?.visible !== false,
    ).length;
    if (count === expected) return { pass: true, count, snapshot: last };
    await Bun.sleep(15);
  }
  const count = ((last?.windows ?? []) as Json[]).filter(
    (window) => window?.kind === kind && window?.visible !== false,
  ).length;
  return { pass: false, count, snapshot: last };
}

async function nativeWindowIds(pid: number, title?: string) {
  // C08: when the orchestrator supplies a hash-validated window-query helper
  // the complete inventory comes from that pinned binary; standalone runs
  // keep the legacy source interpretation.
  const query = await run(
    windowQueryHelper
      ? [windowQueryHelper, "--pid", String(pid)]
      : [
        "swift",
        resolve(import.meta.dir, "../agentic/macos-window-query.swift"),
        "--pid",
        String(pid),
      ],
  );
  if (query.exitCode !== 0) {
    return {
      ids: [] as number[],
      windows: [] as Json[],
      completeWindows: [] as Json[],
      completeWindowIds: [] as number[],
      includesHiddenAndAlphaZero: true,
      error: query.stderr.trim(),
    };
  }
  const parsed = JSON.parse(query.stdout);
  const allWindows = (parsed.windows ?? [])
    .filter((window: Json) =>
      Number(window?.windowId ?? 0) > 0
      && Number(window?.bounds?.width ?? 0) > 1
      && Number(window?.bounds?.height ?? 0) > 1
    );
  const matching = allWindows
      .filter((window: Json) =>
        window?.onscreen === true
        && Number(window?.alpha ?? 0) > 0
        && (title == null || window?.title === title)
      )
      .sort((left: Json, right: Json) =>
        Number(right?.bounds?.width ?? 0) * Number(right?.bounds?.height ?? 0)
        - Number(left?.bounds?.width ?? 0) * Number(left?.bounds?.height ?? 0)
      );
  return {
    ids: matching.map((window: Json) => Number(window.windowId)),
    windows: matching,
    completeWindows: allWindows,
    completeWindowIds: allWindows.map((window: Json) => Number(window.windowId)),
    includesHiddenAndAlphaZero: true,
    error: null,
  };
}

function analyzeFilmstrip(directory: string, expectedWindowID: number) {
  const receiptPath = join(directory, "receipt.json");
  const receipt = existsSync(receiptPath)
    ? JSON.parse(readFileSync(receiptPath, "utf8"))
    : null;
  const frames = (receipt?.frames ?? []) as Json[];
  const hashes = new Set(
    frames.map((frame) => String(frame?.sha256 ?? "")).filter(Boolean),
  );
  const errors = [
    ...(receipt?.errors ?? []),
    ...validateFilmstripCapture(receipt, {
      runId: String(receiptRoot.runId),
      gitCommit: String(receiptRoot.gitCommit),
      binarySha256: String(receiptRoot.binarySha256),
      pid: Number(driver.pid),
      windowId: expectedWindowID,
    }),
    ...(receipt?.windowID === expectedWindowID
      ? []
      : [
        `pinned CGWindowID changed: expected ${expectedWindowID}, got ${
          receipt?.windowID ?? "missing"
        }`,
      ]),
    ...(frames.length >= 4 ? [] : [`only ${frames.length}/4 complete frames`]),
    ...(hashes.size >= 2 ? [] : ["filmstrip contains fewer than two visual states"]),
  ];
  return {
    receiptPath,
    receipt,
    frameCount: frames.length,
    distinctFrameHashes: hashes.size,
    pinnedWindowID: expectedWindowID,
    errors,
    pass: errors.length === 0,
  };
}

const binary = resolve(arg("--binary", process.env.SCRIPT_KIT_GPUI_BINARY ?? "")!);
const themeFixture = arg("--theme-fixture");
const outDir = resolve(
  arg("--out", ".artifacts/glass-motion-contrast/lifecycle-filmstrips")!,
);
const scenarioProfile = (arg("--profile", "full") ?? "full") as ScenarioProfile;
const resolvedScenarioNames = resolveScenarioNames(scenarioProfile);
const analysisMode = parseAnalysisMode(arg("--analysis-mode", "inline"));
// WP6: when the caller declares the build's morph contract, the observed
// instrumented log line must match it exactly — a mismatch is INVALID_SETUP
// (mislabeled artifact), never a product verdict.
const declaredStartAlphaArg = arg("--declared-start-alpha");
const declaredDurationNs = Number(arg("--declared-duration-ns", String(MAIN_GLASS_ENTRY_EXPECTATION.durationMs * 1_000_000)));
const contractWindowName = arg("--contract-window-name", "Main window")!;
const telemetryIntervalMs = Number(arg("--telemetry-interval-ms", "250"));
// WP9: when the orchestrator owns a backdrop fixture, its receipt identity
// is embedded so imported-evidence validation (validateArtifactReference)
// can match background-fixture mode/config/display on reuse.
const backgroundFixtureReceiptArg = arg("--background-fixture-receipt");
if (!binary || !existsSync(binary)) {
  throw new Error(`binary missing: ${binary || "<unset>"}`);
}
mkdirSync(outDir, { recursive: true });
const scriptStartedMs = performance.now();
const timingMs: Record<string, number> = {
  helperPreparation: 0,
  driverLaunchToMainVisible: 0,
  normalization: 0,
  pointerPreparation: 0,
  captureTotal: 0,
  inlineAnalysisTotal: 0,
  cleanup: 0,
  total: 0,
};
const helperPreparationStartedMs = performance.now();
// WP4: the v2 orchestrator supplies pre-compiled hash-validated helpers;
// standalone invocations keep the legacy per-run compile behavior.
const suppliedFilmstripHelper = arg("--filmstrip-helper");
const suppliedInterferenceHelper = arg("--interference-helper");
const suppliedWindowQueryHelper = arg("--window-query-helper");
const windowQueryHelper = suppliedWindowQueryHelper
  ? requireValidatedHelper(suppliedWindowQueryHelper, "window-query").binaryPath
  : null;
let helper: string;
if (suppliedFilmstripHelper) {
  helper = requireValidatedHelper(suppliedFilmstripHelper, "filmstrip")
    .binaryPath;
} else {
  helper = join(outDir, "macos-native-window-filmstrip");
  const compile = await run([
    "xcrun",
    "swiftc",
    "-parse-as-library",
    "-O",
    resolve(import.meta.dir, "../agentic/macos-native-window-filmstrip.swift"),
    "-o",
    helper,
  ]);
  if (compile.exitCode !== 0) {
    throw new Error(`filmstrip helper compile failed: ${compile.stderr}`);
  }
}
let interferenceHelper: string;
if (suppliedInterferenceHelper) {
  interferenceHelper = requireValidatedHelper(
    suppliedInterferenceHelper,
    "interference",
  ).binaryPath;
} else {
  interferenceHelper = join(outDir, "macos-glass-interference-monitor");
  const interferenceCompile = await run([
    "xcrun",
    "swiftc",
    "-O",
    resolve(import.meta.dir, "../agentic/macos-glass-interference-monitor.swift"),
    "-o",
    interferenceHelper,
  ]);
  if (interferenceCompile.exitCode !== 0) {
    throw new Error(
      `interference helper compile failed: ${interferenceCompile.stderr}`,
    );
  }
}
timingMs.helperPreparation = performance.now() - helperPreparationStartedMs;
let interferenceMonitor: ReturnType<typeof startInterferenceMonitor> | null = null;

// WP6 system telemetry: pre-run edge snapshot + continuous low-overhead
// sampler across the whole capture, boundary snapshots per scenario, and a
// post-run edge snapshot in the finally block. Boundary eligibility (load
// <= 6.0 pre/post) is computed from the same values the legacy harness used.
const telemetryPre = await captureEdgeSnapshot("pre");
const telemetryPreLoad1 = os.loadavg()[0];
const telemetrySampler = startSampler(telemetryIntervalMs);
const telemetryBoundaries: BoundarySnapshot[] = [];
const scenarioUnixIntervals: ScenarioInterval[] = [];

const receipt: Json = {
  schemaVersion: 2,
  startedAt: new Date().toISOString(),
  ...identityFromEnvironment({
    runId: newRunId(),
    gitCommit: (await run(["git", "rev-parse", "HEAD"])).stdout.trim(),
    binary,
    binarySha256: createHash("sha256").update(readFileSync(binary)).digest("hex"),
  }),
  scenario: process.env.SCRIPT_KIT_GLASS_SCENARIO ?? "lifecycle",
  scenarioProfile,
  analysisMode,
  requestedScenarioNames: resolvedScenarioNames,
  themeFixture: themeFixture
    ? {
      path: resolve(themeFixture),
      sha256: createHash("sha256")
        .update(readFileSync(resolve(themeFixture)))
        .digest("hex"),
    }
    : null,
  backgroundFixture: (() => {
    if (!backgroundFixtureReceiptArg) return null;
    const path = resolve(backgroundFixtureReceiptArg);
    if (!existsSync(path)) {
      throw new Error(`background fixture receipt missing: ${path}`);
    }
    const fixture = JSON.parse(readFileSync(path, "utf8"));
    if (fixture.status !== "ready") {
      throw new Error(
        `background fixture receipt status ${fixture.status} — refusing to bind a non-ready fixture`,
      );
    }
    return {
      receiptPath: path,
      mode: fixture.mode ?? null,
      configurationSha256: fixture.configurationSha256 ?? null,
      displayID: fixture.displayID ?? null,
      visualSha256: fixture.visualDiagnostics?.visualSha256 ?? null,
    };
  })(),
  helperSha256: createHash("sha256").update(readFileSync(helper)).digest("hex"),
  scenarios: [],
  pass: false,
};
const receiptRoot = receipt;

const driverLaunchStartedMs = performance.now();
const driver = await Driver.launch({
  binary,
  sessionDir: join(outDir, "session"),
  sharedModels: false,
  sandboxHome: true,
  themeFixturePath: themeFixture,
  sessionName: `glass-lifecycle-filmstrip-${process.pid}`,
  defaultTimeoutMs: 8_000,
  env: {
    SCRIPT_KIT_FIDELITY_CAPTURE: "agent-chat",
  },
});

function startFilmstrip(
  name: LifecycleScenarioName,
  selector: {
    windowID?: number;
    pid?: number;
    title?: string;
    displayStream?: boolean;
    bounds?: { x: number; y: number; width: number; height: number };
    regionArmed?: boolean;
    ownerClass?: "Actions";
    excludedWindowIds?: number[];
  },
) {
  const durationMs = SCENARIO_CAPTURE_DURATIONS_MS[name];
  const directory = join(outDir, name);
  rmSync(directory, { recursive: true, force: true });
  mkdirSync(directory, { recursive: true });
  const readyPath = join(directory, "ready.json");
  const command = [
    helper,
    ...(selector.windowID != null
      ? ["--window-id", String(selector.windowID), "--pid", String(driver.pid)]
      : [
        "--pid",
        String(selector.pid),
        ...(selector.title ? ["--title", selector.title] : []),
      ]),
    ...(selector.displayStream !== false ? ["--display-stream"] : []),
    ...(selector.regionArmed ? ["--region-armed"] : []),
    ...(selector.ownerClass ? ["--owner-class", selector.ownerClass] : []),
    ...(selector.excludedWindowIds ?? []).flatMap((windowId) => [
      "--exclude-window-id",
      String(windowId),
    ]),
    ...(selector.bounds
      ? [
        "--bounds",
        String(selector.bounds.x),
        String(selector.bounds.y),
        String(selector.bounds.width),
        String(selector.bounds.height),
      ]
      : []),
    "--out",
    directory,
    "--ready",
    readyPath,
    "--duration-ms",
    String(durationMs),
    "--fps",
    "120",
    "--run-id",
    String(receipt.runId),
    "--git-commit",
    String(receipt.gitCommit),
    "--binary-sha256",
    String(receipt.binarySha256),
  ];
  const process = Bun.spawn(command, { stdout: "pipe", stderr: "pipe" });
  return {
    name,
    directory,
    readyPath,
    process,
    command,
    processStartedAt: new Date().toISOString(),
    requestedCaptureDurationMs: durationMs,
    observerStartedMs: performance.now(),
    readyAtMs: null as number | null,
  };
}

/** waitForFile on the observer's ready receipt, stamping observer-ready time. */
async function awaitObserverReady(started: ReturnType<typeof startFilmstrip>) {
  const ready = await waitForFile(started.readyPath);
  started.readyAtMs = performance.now();
  return ready;
}

async function finishFilmstrip(
  started: ReturnType<typeof startFilmstrip>,
  expectedWindowID: number,
  metricsContext?: {
    bodyBounds?: { x: number; y: number; width: number; height: number };
    hiddenHostTimeNs?: number;
    visibleHostTimeNs?: number;
    expectedExitFrame?: [number, number, number, number];
    captureBounds?: { x: number; y: number; width: number; height: number };
    referenceImagePath?: string;
  },
) {
  const finishCalledMs = performance.now();
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(started.process.stdout).text(),
    new Response(started.process.stderr).text(),
    started.process.exited,
  ]);
  const observerExitedMs = performance.now();
  const metricsPath = join(started.directory, "metrics.json");
  const metricsCommand = [
    "python3",
    resolve(import.meta.dir, "../agentic/glass-lifecycle-metrics.py"),
    "--receipt",
    join(started.directory, "receipt.json"),
    "--scenario",
    started.name,
    "--out",
    metricsPath,
    ...(metricsContext?.bodyBounds
      ? [
        "--body-bounds",
        String(metricsContext.bodyBounds.x),
        String(metricsContext.bodyBounds.y),
        String(metricsContext.bodyBounds.width),
        String(metricsContext.bodyBounds.height),
      ]
      : []),
    ...(metricsContext?.visibleHostTimeNs != null
      ? ["--visible-host-time-ns", String(metricsContext.visibleHostTimeNs)]
      : []),
    ...(metricsContext?.hiddenHostTimeNs != null
      ? ["--hidden-host-time-ns", String(metricsContext.hiddenHostTimeNs)]
      : []),
    ...(metricsContext?.expectedExitFrame != null
      ? [
        "--expected-exit-frame",
        ...metricsContext.expectedExitFrame.map(String),
      ]
      : []),
    ...(metricsContext?.captureBounds != null
      ? [
        "--capture-bounds",
        String(metricsContext.captureBounds.x),
        String(metricsContext.captureBounds.y),
        String(metricsContext.captureBounds.width),
        String(metricsContext.captureBounds.height),
      ]
      : []),
    ...(metricsContext?.referenceImagePath != null
      ? ["--reference-image", metricsContext.referenceImagePath]
      : []),
  ];
  // Deferred mode never invokes Python while a display scenario could still
  // be active: the exact grader command is preserved so the offline finalizer
  // reruns it verbatim against the same hash-bound artifacts.
  let metricsExitCode: number | null = null;
  let metrics: Json | null = null;
  let analysisMs = 0;
  if (analysisMode === "inline") {
    const analysisStartedMs = performance.now();
    const metricsResult = await run(metricsCommand);
    analysisMs = performance.now() - analysisStartedMs;
    timingMs.inlineAnalysisTotal += analysisMs;
    metricsExitCode = metricsResult.exitCode;
    metrics = existsSync(metricsPath)
      ? JSON.parse(readFileSync(metricsPath, "utf8"))
      : null;
  }
  const capture = analyzeFilmstrip(started.directory, expectedWindowID);
  const verdict = filmstripVerdict({
    captureErrorCount: capture.errors.length,
    analysisMode,
    metricsExitCode,
    metricsPass: metrics?.pass === true,
  });
  const finishedMs = performance.now();
  return {
    command: started.command,
    exitCode,
    stderr: stderr.trim().slice(-1_000),
    stdout: stdout.trim().slice(-1_000),
    metricsPath,
    metricsCommand,
    metricsExitCode,
    metrics,
    analysisState: verdict.analysisState,
    ...capture,
    capturePass: verdict.capturePass,
    timing: {
      observerStartToReadyMs: started.readyAtMs == null
        ? null
        : started.readyAtMs - started.observerStartedMs,
      driveMs: started.readyAtMs == null
        ? null
        : finishCalledMs - started.readyAtMs,
      requestedCaptureDurationMs: started.requestedCaptureDurationMs,
      observerFinishMs: observerExitedMs - finishCalledMs,
      analysisMs,
      totalMs: finishedMs - started.observerStartedMs,
    },
    pass: verdict.pass,
  };
}

async function analyzePresentationGeometry(
  directory: string,
  expectedOwner: number,
  anchorBounds: { x: number; y: number; width: number; height: number },
) {
  const path = join(directory, "presentation-geometry.json");
  const result = await run([
    "python3",
    resolve(import.meta.dir, "../agentic/rendered-capsule-geometry.py"),
    "--receipt",
    join(directory, "receipt.json"),
    "--expected-owner",
    String(expectedOwner),
    "--anchor-bounds",
    String(anchorBounds.x),
    String(anchorBounds.y),
    String(anchorBounds.width),
    String(anchorBounds.height),
    "--out",
    path,
  ]);
  return {
    path,
    exitCode: result.exitCode,
    stderr: result.stderr.trim().slice(-1_000),
    receipt: existsSync(path) ? JSON.parse(readFileSync(path, "utf8")) : null,
  };
}

try {
  receipt.pid = driver.pid ?? null;
  receipt.sessionDir = driver.sessionDir;
  driver.send({ type: "show" });
  await waitForWindowCount(driver, "main", 1, 5_000);
  await driver.waitForSettle({ timeoutMs: 5_000 });
  timingMs.driverLaunchToMainVisible = performance.now() - driverLaunchStartedMs;
  const normalizationStartedMs = performance.now();
  await announceTestStatus(
    "Lifecycle setup · Normalize main owner",
    "Warm one hide/show lifecycle before pinning the exact native window and bounds",
  );
  driver.send({ type: "hide", requestId: "glass-life-warm-hide" });
  await driver.waitForState({ windowVisible: false }, { timeoutMs: 3_000 });
  await Bun.sleep(MAIN_EXIT_POST_HIDE_SETTLE_MS);
  driver.send({ type: "show", requestId: "glass-life-warm-show" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 3_000 });
  await driver.waitForSettle({ timeoutMs: 3_000 });
  await Bun.sleep(350);
  const pid = Number(driver.pid);
  const mainNative = await nativeWindowIds(pid);
  const mainWindowID = mainNative.ids[0];
  if (!mainWindowID) throw new Error(`main native window missing: ${mainNative.error}`);
  const mainBounds = mainNative.windows.find(
    (window: Json) => Number(window.windowId) === mainWindowID,
  )?.bounds;
  if (
    !mainBounds
    || !["x", "y", "width", "height"].every(
      (key) => Number.isFinite(Number(mainBounds[key])),
    )
  ) {
    throw new Error(`main native window ${mainWindowID} bounds are missing`);
  }
  receipt.initialCompleteNativeInventory = mainNative;
  const mainCaptureBounds = expandCaptureBounds({
    x: Number(mainBounds.x),
    y: Number(mainBounds.y),
    width: Number(mainBounds.width),
    height: Number(mainBounds.height),
  });
  const initialTopology = classifyNativeInventory(
    mainNative.completeWindows as any[],
    pid,
    mainWindowID,
  );
  receipt.initialNativeTopology = initialTopology;
  if (!initialTopology.pass) {
    throw new Error(
      `initial complete same-PID topology invalid: ${JSON.stringify(initialTopology.errors)}`,
    );
  }
  timingMs.normalization = performance.now() - normalizationStartedMs;
  const pointerPreparationStartedMs = performance.now();
  const pointerPreparation = await run(["cliclick", "m:2,2"]);
  if (pointerPreparation.exitCode !== 0) {
    throw new Error(
      `failed to park pointer away from entry capsules: ${pointerPreparation.stderr}`,
    );
  }
  timingMs.pointerPreparation = performance.now() - pointerPreparationStartedMs;
  interferenceMonitor = startInterferenceMonitor(interferenceHelper, outDir);
  receipt.interferenceReady = await waitForInterferenceReady(interferenceMonitor);

  async function runMainExitScenario() {
  await announceTestStatus(
    "Lifecycle filmstrip · Main exit",
    "Exact CGWindowID capture while the main surface and detached capsules fade together",
  );
  const mainExit = startFilmstrip(
    "main-exit",
    { windowID: mainWindowID, bounds: mainCaptureBounds },
  );
  await awaitObserverReady(mainExit);
  await Bun.sleep(MAIN_EXIT_CAPTURE_PREROLL_MS);
  driver.send({ type: "hide", requestId: "glass-life-main-hide" });
  await driver.waitForState({ windowVisible: false }, { timeoutMs: 3_000 });
  await Bun.sleep(MAIN_EXIT_POST_HIDE_SETTLE_MS);
  const mainExitReferencePath = join(mainExit.directory, "hidden-reference.png");
  const mainExitReferenceCapture = await run([
    "screencapture",
    "-x",
    `-R${mainCaptureBounds.x},${mainCaptureBounds.y},${mainCaptureBounds.width},${mainCaptureBounds.height}`,
    mainExitReferencePath,
  ]);
  const mainAfterExit = await nativeWindowIds(pid);
  const exitedOwner = mainAfterExit.completeWindows.find(
    (window: Json) => Number(window?.windowId) === mainWindowID,
  );
  const mainExitReference = {
    path: mainExitReferencePath,
    captureSource: "explicit-post-exit-display-screenshot",
    capturedAt: new Date().toISOString(),
    captureExitCode: mainExitReferenceCapture.exitCode,
    captureStderr: mainExitReferenceCapture.stderr.trim(),
    sha256: mainExitReferenceCapture.exitCode === 0
        && existsSync(mainExitReferencePath)
      ? createHash("sha256")
        .update(readFileSync(mainExitReferencePath))
        .digest("hex")
      : null,
    expectedWindowID: mainWindowID,
    ownerOnscreenAfterExit: exitedOwner?.onscreen ?? false,
    ownerAlphaAfterExit: exitedOwner?.alpha ?? 0,
  };
  const mainExitReferencePass = mainExitReference.captureExitCode === 0
    && /^[a-f0-9]{64}$/.test(String(mainExitReference.sha256 ?? ""))
    && mainExitReference.ownerOnscreenAfterExit === false
    && Number(mainExitReference.ownerAlphaAfterExit) <= 0;
  const mainExitFilmstrip = await finishFilmstrip(
    mainExit,
    mainWindowID,
    {
      captureBounds: mainCaptureBounds,
      referenceImagePath: mainExitReferencePath,
    },
  );
  (receipt.scenarios as Json[]).push({
    name: "main-exit",
    exactWindowID: mainWindowID,
    hiddenReference: mainExitReference,
    hiddenReferencePass: mainExitReferencePass,
    filmstrip: mainExitFilmstrip,
    structuralPass: mainExitFilmstrip.capturePass && mainExitReferencePass,
    pass: mainExitFilmstrip.pass && mainExitReferencePass,
  });
  }

  async function runMainEntryScenario() {
  await announceTestStatus(
    "Lifecycle filmstrip · Main entry",
    "Observer starts while hidden; the same CGWindowID and detached gutter emerge together",
  );
  const mainEntry = startFilmstrip(
    "main-entry",
    { windowID: mainWindowID, displayStream: true, bounds: mainCaptureBounds },
  );
  const mainEntryReady = await awaitObserverReady(mainEntry);
  const mainEntryCaptureBoundsMatch = ["x", "y", "width", "height"].every(
    (key) =>
      Math.abs(
        Number(mainEntryReady?.captureBounds?.[key])
        - Number(mainCaptureBounds[key as keyof typeof mainCaptureBounds]),
      ) <= 0.01,
  );
  await Bun.sleep(40);
  const showRequestedAt = new Date().toISOString();
  driver.send({ type: "show", requestId: "glass-life-main-show" });
  await driver.waitForState({ windowVisible: true }, { timeoutMs: 3_000 });
  await driver.waitForSettle({ timeoutMs: 3_000 });
  const mainEntrySettledLayout = await driver.getLayoutInfo(
    { target: { type: "id", id: "main" } },
    { timeoutMs: 5_000 },
  );
  const settledCaptures: Json[] = [];
  for (let index = 0; index < 3; index += 1) {
    const path = join(
      mainEntry.directory,
      `settled-${String(index).padStart(4, "0")}.png`,
    );
    const capture = await run([
      "screencapture",
      "-x",
      `-R${mainCaptureBounds.x},${mainCaptureBounds.y},${mainCaptureBounds.width},${mainCaptureBounds.height}`,
      path,
    ]);
    const native = await nativeWindowIds(pid);
    const owner = native.completeWindows.find(
      (window: Json) => Number(window?.windowId) === mainWindowID,
    );
    settledCaptures.push({
      sequence: `settled-${index}`,
      captureSource: "explicit-post-settle-display-screenshot",
      path,
      sha256: capture.exitCode === 0 && existsSync(path)
        ? createHash("sha256").update(readFileSync(path)).digest("hex")
        : null,
      capturedAt: new Date().toISOString(),
      captureExitCode: capture.exitCode,
      captureStderr: capture.stderr.trim(),
      windowBounds: owner
        ? [
          [Number(owner.bounds.x), Number(owner.bounds.y)],
          [Number(owner.bounds.width), Number(owner.bounds.height)],
        ]
        : null,
      windowAlpha: owner?.alpha ?? null,
      windowOnscreen: owner?.onscreen ?? null,
      actualWindowID: owner?.windowId ?? null,
      expectedWindowID: mainWindowID,
    });
    await Bun.sleep(17);
  }
  const settledCapturesPass = settledCaptures.length === 3
    && settledCaptures.every((capture) =>
      capture.captureExitCode === 0
      && /^[a-f0-9]{64}$/.test(String(capture.sha256 ?? ""))
      && capture.actualWindowID === mainWindowID
      && Number(capture.windowAlpha) >= 0.999
      && capture.windowOnscreen === true
    );
  const mainEntryFilmstrip = await finishFilmstrip(
    mainEntry,
    mainWindowID,
    { captureBounds: mainCaptureBounds },
  );
  const mainPresentationGeometry = await analyzePresentationGeometry(
    mainEntry.directory,
    mainWindowID,
    {
      x: Number(mainBounds.x),
      y: Number(mainBounds.y),
      width: Number(mainBounds.width),
      height: Number(mainBounds.height),
    },
  );
  const mainPresentationFrames = mainPresentationGeometry.receipt?.frames ?? [];
  const mainEntryMotionEnvelope = analyzeEntryMotionEnvelope(
    mainPresentationFrames,
    mainPresentationFrames.at(-1)?.windowBounds ?? null,
    MAIN_GLASS_ENTRY_EXPECTATION,
  );
  // C08 strict observation classification. The LOCKED evaluator is unchanged;
  // only the classification around it changes: missing settled bounds, wrong
  // pinned owner, capture-health failure, and under-resolved evidence are
  // INVALID_OBSERVER — never EVALUABLE_FAIL. Source-derived geometry
  // (entryEvidence, runtimeContract) stays diagnostic-only.
  // Interference and cleanup dominate at run level in the finally block.
  const mainEntryObservation = classifyGlassObservation({
    captureHealthPass: mainEntryFilmstrip.receipt?.captureHealthPass === true
      && mainEntryFilmstrip.capturePass === true
      && mainPresentationGeometry.exitCode === 0
      && mainPresentationGeometry.receipt?.pass === true,
    helperErrors: [],
    fixtureErrors: [],
    identityErrors: [
      ...(mainEntryFilmstrip.errors as string[]),
      ...(mainEntryCaptureBoundsMatch
        ? []
        : ["capture bounds do not enclose the pinned owner"]),
    ],
    ownerErrors: [
      ...validateOwnedRenderedFrames(
        mainEntryFilmstrip.receipt?.frames ?? [],
        mainWindowID,
      ),
      ...(settledCapturesPass
        ? []
        : ["settled native captures did not bind to the pinned owner"]),
    ],
    requiredPhaseErrors: [],
    cleanupErrors: [],
    interference: { validated: true, disposition: null, errors: [] },
    rendered: {
      present: Number(mainEntryMotionEnvelope.measuredFrameCount ?? 0) > 0,
      underResolved: mainEntryMotionEnvelope.underResolved === true,
      pass: mainEntryMotionEnvelope.pass === true,
      errors: mainEntryMotionEnvelope.errors ?? [],
    },
  } satisfies GlassObservationInput);
  (receipt.scenarios as Json[]).push({
    name: "main-entry",
    exactWindowID: mainWindowID,
    observerStartedAt: mainEntry.processStartedAt,
    showRequestedAt,
    streamReady: mainEntryReady,
    captureBounds: mainCaptureBounds,
    captureBoundsMatch: mainEntryCaptureBoundsMatch,
    settledCaptures,
    settledCapturesPass,
    motionEnvelope: mainEntryMotionEnvelope,
    presentationGeometry: mainPresentationGeometry,
    observation: mainEntryObservation,
    pointerPreparation: {
      command: ["cliclick", "m:2,2"],
      exitCode: pointerPreparation.exitCode,
      target: { x: 2, y: 2 },
      purpose: "prevent an inherited hover state from obscuring base material",
    },
    settledLayout: mainEntrySettledLayout,
    filmstrip: mainEntryFilmstrip,
    structuralPass: mainEntryFilmstrip.capturePass
      && mainEntryCaptureBoundsMatch
      && settledCapturesPass
      && mainEntryMotionEnvelope.pass,
    pass: mainEntryFilmstrip.pass
      && mainEntryCaptureBoundsMatch
      && settledCapturesPass
      && mainEntryMotionEnvelope.pass,
  });
  }

  // Dispatch the resolved profile in declared order. Timing intervals are
  // recorded per scenario and validated as monotone/non-overlapping so a
  // refactor can never silently interleave capture work.
  const scenarioRunners: Record<LifecycleScenarioName, () => Promise<void>> = {
    "main-exit": runMainExitScenario,
    "main-entry": runMainEntryScenario,
  };
  const captureStartedMs = performance.now();
  const scenarioIntervals: ScenarioTimingInterval[] = [];
  telemetryBoundaries.push(
    await captureBoundarySnapshot("before-first-scenario", Number(driver.pid)),
  );
  for (const name of resolvedScenarioNames) {
    const startedAtMs = performance.now();
    const startUnixMs = Date.now();
    await scenarioRunners[name]();
    scenarioIntervals.push({
      name,
      startedAtMs,
      finishedAtMs: performance.now(),
    });
    // Unix-clock twin of the interval so timestamped interference events
    // (atUnixMs) can be attributed to the scenario that was capturing.
    scenarioUnixIntervals.push({ name, startUnixMs, endUnixMs: Date.now() });
    telemetryBoundaries.push(
      await captureBoundarySnapshot(`after-${name}`, Number(driver.pid)),
    );
  }
  timingMs.captureTotal = performance.now() - captureStartedMs;
  receipt.scenarioIntervalsMs = scenarioIntervals;
  const intervalErrors = validateScenarioTimingIntervals(scenarioIntervals);
  if (intervalErrors.length > 0) {
    throw new Error(`scenario timing intervals invalid: ${intervalErrors}`);
  }

  receipt.requiredScenarioNames = resolvedScenarioNames;
  const scenarios = receipt.scenarios as Json[];
  receipt.pass = scenarios.length === resolvedScenarioNames.length
    && receipt.requiredScenarioNames.every((name: string) =>
      scenarios.some((scenario) => scenario?.name === name)
    )
    && scenarios.every((scenario) => scenario?.pass === true);
} catch (error) {
  receipt.error = String(error);
  receipt.pass = false;
} finally {
  const cleanupStartedMs = performance.now();
  if (interferenceMonitor) {
    receipt.interference = await finishInterferenceMonitor(interferenceMonitor);
    receipt.pass = receipt.pass === true && receipt.interference.pass === true;
  }
  await driver.close();
  receipt.cleanedUp = !driver.alive;
  receipt.finishedAt = new Date().toISOString();
  receipt.pass = receipt.pass === true && receipt.cleanedUp === true;
  if (analysisMode === "deferred") {
    // A deferred capture is NEVER a normal passing lifecycle receipt: the
    // Python graders have not run. The offline finalizer consumes
    // capture-receipt.json and writes the standard receipt.json later.
    receipt.analysisState = "pending";
    receipt.capturePass = (receipt.scenarios as Json[]).length > 0
      && (receipt.scenarios as Json[]).every(
        (scenario) => scenario?.filmstrip?.capturePass === true,
      )
      && receipt.error == null
      && receipt.cleanedUp === true
      && receipt.interference?.pass === true;
    receipt.pass = false;
  } else {
    receipt.analysisState = "inline";
  }
  receipt.disposition = computeLifecycleDisposition({
    interferenceDisposition: receipt.interference?.disposition ?? null,
    analysisState: receipt.analysisState,
    pass: receipt.pass === true,
    hasObserverError: receipt.error != null,
  });
  // C08 strict refinement: when a scenario's own observation classified its
  // failure as INVALID_OBSERVER (missing owner, under-resolved rendered
  // evidence, missing phase), the run-level EVALUABLE_FAIL is a lie — the
  // apparatus, not the product, failed. Interference still dominates.
  if (
    receipt.disposition === "EVALUABLE_FAIL"
    && (receipt.scenarios as Json[]).some(
      (scenario) => scenario?.observation?.disposition === "INVALID_OBSERVER",
    )
  ) {
    receipt.disposition = "INVALID_OBSERVER";
  }
  // WP6 telemetry finalization: post edge snapshot, sampler summary, GPU
  // capability probe (never a gate), interference statistics with scenario
  // attribution, and — when a contract was declared — the runtime
  // contract cross-check against the app's instrumented morph log.
  const telemetryPost = await captureEdgeSnapshot("post");
  const telemetryPostLoad1 = os.loadavg()[0];
  const telemetrySamplerSummary = telemetrySampler.stop();
  receipt.systemTelemetry = {
    ...summarizeTelemetry({
      pre: telemetryPre,
      post: telemetryPost,
      sampler: telemetrySamplerSummary,
      boundaries: telemetryBoundaries,
      preLoad1: telemetryPreLoad1,
      postLoad1: telemetryPostLoad1,
    }),
    samplerIntervalMs: telemetryIntervalMs,
    boundaries: telemetryBoundaries,
    pre: telemetryPre,
    post: telemetryPost,
    gpu: await probeGpuTelemetry(),
  } as Json;
  if (receipt.interference?.receipt) {
    receipt.interferenceStatistics = interferenceStatistics(
      receipt.interference.receipt,
      scenarioUnixIntervals,
    ) as Json;
  }
  // The main and Actions receipts carry the same logged fade evidence so the
  // two entries can be compared field by field.
  {
    const appLogLines = (existsSync(driver.logPath)
      ? readFileSync(driver.logPath, "utf8")
      : "").split("\n");
    const entryLines = appLogLines.filter((line) =>
      line.includes("event=glass_morph")
      && line.includes(contractWindowName)
      && line.includes("phase=enter")
    ).slice(-3);
    const timelineLines = appLogLines
      .filter((line) => line.includes("host_time_ns="))
      .filter((line) => /event=(main|actions)_/.test(line))
      .slice(-24);
    receipt.entryEvidence = {
      fade: analyzeEntryFadeLog(
        entryLines,
        contractWindowName === "Main window"
          ? MAIN_GLASS_ENTRY_EXPECTATION
          : ACTIONS_GLASS_ENTRY_EXPECTATION,
      ),
      timeline: timelineLines,
    } as Json;
  }
  if (declaredStartAlphaArg !== undefined) {
    const appLogText = existsSync(driver.logPath)
      ? readFileSync(driver.logPath, "utf8")
      : "";
    receipt.runtimeContract = checkRuntimeContract(
      {
        declaredMorphStartAlpha: Number(declaredStartAlphaArg),
        expectedDurationNs: declaredDurationNs,
      },
      parseMorphEnterLogs(appLogText),
      contractWindowName,
    ) as Json;
    if (receipt.runtimeContract.pass !== true) {
      // A mislabeled artifact invalidates the run's setup. Interference
      // still dominates the disposition; a contract mismatch never becomes
      // a product failure.
      receipt.pass = false;
      if (receipt.disposition !== "INVALID_INTERFERENCE") {
        receipt.disposition = "INVALID_SETUP";
      }
    }
  } else {
    receipt.runtimeContract = null;
  }
  timingMs.cleanup = performance.now() - cleanupStartedMs;
  timingMs.total = performance.now() - scriptStartedMs;
  receipt.timingMs = timingMs;
  const receiptPath = join(
    outDir,
    analysisMode === "deferred" ? "capture-receipt.json" : "receipt.json",
  );
  writeFileSync(receiptPath, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(
    JSON.stringify(
      {
        receiptPath,
        pass: receipt.pass,
        disposition: receipt.disposition,
        analysisMode,
      },
      null,
      2,
    ),
  );
}

// C08 exit contract: 0 only for EVALUABLE_PASS; 2 for evaluable failures and
// deferred (analysis-pending) captures; 4 for INVALID_* observations.
process.exit(
  receipt.disposition === "EVALUABLE_PASS"
    ? 0
    : receipt.disposition === "EVALUABLE_FAIL"
        || receipt.disposition === "ANALYSIS_PENDING"
    ? 2
    : 4,
);
