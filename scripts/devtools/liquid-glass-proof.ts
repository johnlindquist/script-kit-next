#!/usr/bin/env bun

import { readFileSync } from "node:fs";

type JsonObject = Record<string, unknown>;

type Args = {
  inventory: string;
  artifactRoot: string;
  out: string;
};

type Evidence = {
  receipts: string[];
  screenshots: string[];
  layoutReceipts: string[];
  inspectReceipts: string[];
  imageDiffReceipts: string[];
  diffMasks: string[];
  visualAudit: JsonObject | null;
  osScreenshotBlockers: JsonObject[];
  layoutReceiptFreshnessLimitations: string[];
  notes: string[];
};

type ProofTiers = {
  osScreenshotProof: "pass" | "blocked" | "missing";
  appRenderProof: "pass" | "blocked" | "fail" | "missing";
  offscreenRenderProof: "pass" | "fail" | "missing";
  numericProof: "pass" | "fail" | "missing";
  guidelineProof: "pass" | "fail" | "missing";
  imageDiffProof: "pass" | "blocked" | "missing";
};

type GuidanceProofStatus =
  | "strong-guidance-proof"
  | "numeric-guidance-proof-missing-os-visual"
  | "guidance-proof-capture-blocked"
  | "stale-layout-evidence"
  | "source-ui-gap"
  | "missing-guidance-proof";

const MACOS_WINDOWSERVER_CAPTURE_BLOCKED = "macos-windowserver-capture-blocked";
const SCREEN_CAPTURE_WINDOW_FAILED = "screencapture_window_failed";
const SCREEN_CAPTURE_RECT_FAILED = "screencapture_rect_failed";
const OS_CAPTURE_BLOCKER_CODES = [
  "target-identity-blocked",
  "target-not-capture-candidate",
  "window-id-api-blocked",
  "screen-rect-capture-blocked",
  "screen-recording-permission-blocked",
  "blank-compositor-capture",
  "occlusion-or-space-blocked",
  "pixel-audit-failed",
  "unknown-capture-failure",
];

let RECEIPT_ROOT = "artifacts/liquid-glass/receipts";
let SCREENSHOT_ROOT = "artifacts/liquid-glass/screenshots";
let DIFF_ROOT = "artifacts/liquid-glass/diffs";

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/liquid-glass-proof.ts --inventory <surface-inventory.json> --artifact-root artifacts/liquid-glass --out <proof-matrix.json>",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }
  const args: Args = {
    inventory: `${RECEIPT_ROOT}/surface-inventory-2026-05-29-2050.json`,
    artifactRoot: "artifacts/liquid-glass",
    out: `${RECEIPT_ROOT}/liquid-glass-proof-matrix.json`,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--inventory") {
      args.inventory = argv[++index] ?? args.inventory;
    } else if (arg === "--artifact-root") {
      args.artifactRoot = argv[++index] ?? args.artifactRoot;
    } else if (arg === "--out") {
      args.out = argv[++index] ?? args.out;
    }
  }
  return args;
}

function asObject(value: unknown): JsonObject {
  return typeof value === "object" && value !== null ? value as JsonObject : {};
}

function asArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : [];
}

function hasPositiveRadius(value: unknown) {
  if (typeof value === "number") {
    return Number.isFinite(value) && value > 0;
  }
  if (!value || typeof value !== "object") {
    return false;
  }
  const radii = Object.values(value as JsonObject).filter(
    (entry): entry is number => typeof entry === "number" && Number.isFinite(entry),
  );
  return radii.length > 0 && radii.every((entry) => entry > 0);
}

function isRadiusBearingNode(node: JsonObject) {
  const type = String(node.type ?? "").toLowerCase();
  if (type === "other" || type === "text") return false;
  const name = String(node.name ?? node.type ?? "");
  return /Area|Content|Panel|List|Window|Header|Footer|Input|Button|Item|Row|Card|Prompt|Choices|Search|Action|Close|Tile/i
    .test(name);
}

function nodesWithMissingPositiveRadius(receipt: JsonObject) {
  return asArray(receipt.nodes)
    .map(asObject)
    .filter((node) => {
      if (!isRadiusBearingNode(node)) return false;
      const style = asObject(node.visualStyle);
      if (Object.keys(style).length === 0) return false;
      return !hasPositiveRadius(style.cornerRadius) && !hasPositiveRadius(style.radius);
    })
    .map((node) => String(node.name ?? node.type ?? "unknown"));
}

async function listFiles(dir: string) {
  const proc = Bun.spawnSync(["find", dir, "-maxdepth", "1", "-type", "f", "-print"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  if (proc.exitCode !== 0) {
    return [];
  }
  return new TextDecoder().decode(proc.stdout).trim().split("\n").filter(Boolean).sort();
}

async function readJsonIfExists(path: string) {
  try {
    return JSON.parse(await Bun.file(path).text()) as JsonObject;
  } catch {
    return null;
  }
}

function includesAny(path: string, terms: string[]) {
  const lower = path.toLowerCase();
  return terms.some((term) => lower.includes(term.toLowerCase()));
}

function readJsonSync(path: string): JsonObject | null {
  try {
    return JSON.parse(readFileSync(path, "utf8")) as JsonObject;
  } catch {
    return null;
  }
}

type ScreenshotUsability = {
  usable: boolean;
  note?: string;
};

type ImageDiffUsability = {
  usable: boolean;
  note?: string;
};

function auditFromReceipt(json: JsonObject | null): JsonObject {
  const screenshot = asObject(json?.screenshot);
  const screenshotReceipt = asObject(json?.screenshotReceipt);
  const screenshotReceiptNested = asObject(screenshot.receipt);
  return asObject(
    json?.contentAudit
      ?? screenshot.contentAudit
      ?? screenshotReceipt.contentAudit
      ?? screenshotReceiptNested.contentAudit,
  );
}

function screenshotUsability(path: string, receipts: string[]): ScreenshotUsability {
  const fileName = path.split("/").pop() ?? path;
  const baseName = fileName.replace(/\.[^.]+$/, "");
  if (baseName.includes("render")) {
    return {
      usable: false,
      note: `ignored screenshot ${path}: app-render/readback images do not count as OS screenshot evidence`,
    };
  }
  const matchingReceipts = receipts.filter((receipt) => {
    const receiptName = receipt.split("/").pop() ?? receipt;
    return receiptName.startsWith(baseName) && receiptName.includes("screenshot");
  });
  if (matchingReceipts.length === 0 && baseName.startsWith("tahoe-native-")) {
    return {
      usable: false,
      note: `ignored screenshot ${path}: no matching screenshot receipt with capture status`,
    };
  }

  for (const receipt of matchingReceipts) {
    const json = readJsonSync(receipt);
    if (json?.status === "error" || json?.classification === "error") {
      return {
        usable: false,
        note: `ignored screenshot ${path}: receipt ${receipt} is classified as an error`,
      };
    }
    const contentAudit = auditFromReceipt(json);
    const nonBlackRatio = contentAudit.nonBlackRatio;
    if (typeof nonBlackRatio === "number" && nonBlackRatio < 0.01) {
      return {
        usable: false,
        note: `ignored screenshot ${path}: receipt ${receipt} nonBlackRatio ${nonBlackRatio} is below 0.01 usable-capture threshold`,
      };
    }
  }

  return { usable: true };
}

function captureAttemptSummary(attempts: unknown) {
  return asArray(attempts).map(asObject).map((attempt) => ({
    method: attempt.method ?? null,
    status: attempt.status ?? null,
    errorCode: attempt.errorCode ?? null,
    reason: attempt.reason ?? null,
    message: attempt.message ?? null,
    stderr: attempt.stderr ?? null,
  }));
}

function legacyStrictCaptureAttempts(error: unknown) {
  const text = typeof error === "string" ? error : JSON.stringify(error ?? "");
  const attempts: JsonObject[] = [];
  for (const line of text.split("\n")) {
    const jsonStart = line.indexOf("{");
    if (jsonStart < 0) continue;
    try {
      const event = asObject(JSON.parse(line.slice(jsonStart)));
      if (event.event === "window_capture_screencapture_l_failed") {
        attempts.push({
          method: "screencapture-window-id",
          status: "failed",
          errorCode: SCREEN_CAPTURE_WINDOW_FAILED,
          stderr: event.stderr ?? null,
          windowId: event.windowId ?? null,
        });
      }
    } catch {
      // Legacy receipts embed JSON event lines inside free-form error text.
    }
  }
  if (text.includes("screenRect=could not create image from rect") || text.includes("could not create image from rect")) {
    attempts.push({
      method: "screencapture-screen-rect",
      status: "failed",
      errorCode: SCREEN_CAPTURE_RECT_FAILED,
      stderr: "could not create image from rect",
    });
  }
  if (text.includes("native=computer/capture_native_window")) {
    attempts.push({
      method: "computer/capture_native_window",
      status: "failed",
      errorCode: text.includes("not_capture_candidate") ? "not_capture_candidate" : "capture_failed",
      stderr: null,
    });
  }
  return attempts;
}

function captureBlockerClassification(defaultClassification: string, attempts: JsonObject[]) {
  return attempts.some((attempt) =>
    attempt.errorCode === SCREEN_CAPTURE_WINDOW_FAILED ||
    attempt.errorCode === SCREEN_CAPTURE_RECT_FAILED
  )
    ? MACOS_WINDOWSERVER_CAPTURE_BLOCKED
    : defaultClassification;
}

function osScreenshotBlockerFromReceipt(receiptPath: string): JsonObject | null {
  const name = receiptPath.split("/").pop() ?? receiptPath;
  const json = readJsonSync(receiptPath);
  if (!json) {
    return null;
  }
  const visualEvidence = asObject(json.visualEvidence);
  if (
    visualEvidence.source === "os-window-capture" &&
    visualEvidence.available === false &&
    visualEvidence.countsAsOsScreenshotEvidence !== true
  ) {
    return {
      receipt: receiptPath,
      source: "os-window-capture",
      classification: String(visualEvidence.classification ?? MACOS_WINDOWSERVER_CAPTURE_BLOCKED),
      blockerCode: visualEvidence.blockerCode ?? null,
      limitation: visualEvidence.limitation ?? null,
      attempts: captureAttemptSummary(visualEvidence.attempts),
    };
  }
  if (!name.includes("screenshot") && !name.includes("capture")) {
    return null;
  }
  const screenshotReceipt = asObject(json.screenshotReceipt);
  const nestedScreenshotReceipt = asObject(asObject(json.screenshot).receipt);
  const receipt = Object.keys(screenshotReceipt).length > 0 ? screenshotReceipt : nestedScreenshotReceipt;
  if (json.status === "error" || json.classification === "error") {
    const attempts = [
      ...captureAttemptSummary(json.attempts),
      ...legacyStrictCaptureAttempts(receipt.error ?? json.error),
    ];
    return {
      receipt: receiptPath,
      source: "screenshot-receipt",
      classification: captureBlockerClassification("screenshot-receipt-error", attempts),
      error: receipt.error ?? json.error ?? null,
      attempts,
    };
  }
  if (receipt.captured === false || typeof receipt.error === "string") {
    const attempts = [
      ...captureAttemptSummary(json.attempts),
      ...legacyStrictCaptureAttempts(receipt.error),
    ];
    return {
      receipt: receiptPath,
      source: "screenshot-receipt",
      classification: captureBlockerClassification("screenshot-capture-failed", attempts),
      error: receipt.error ?? null,
      attempts,
    };
  }
  const audit = auditFromReceipt(json);
  if (audit.blank === true) {
    return {
      receipt: receiptPath,
      source: "content-audit",
      classification: "blank-image-rejected",
      attempts: captureAttemptSummary(json.attempts),
    };
  }
  if (typeof audit.nonBlackRatio === "number" && audit.nonBlackRatio < 0.01) {
    return {
      receipt: receiptPath,
      source: "content-audit",
      classification: "low-content-capture-rejected",
      nonBlackRatio: audit.nonBlackRatio,
      attempts: captureAttemptSummary(json.attempts),
    };
  }
  return null;
}

function osScreenshotAttemptBlocked(receiptPath: string) {
  const blocker = osScreenshotBlockerFromReceipt(receiptPath);
  return blocker ? String(blocker.classification ?? "os-window-capture-blocked") : null;
}

function usableAppRenderEvidence(renderEvidence: JsonObject) {
  const pixelAudit = asObject(renderEvidence.pixelAudit);
  return (
    renderEvidence.source === "gpui-render-readback" &&
    renderEvidence.available === true &&
    renderEvidence.countsAsAppRenderEvidence === true &&
    renderEvidence.countsAsOsScreenshotEvidence === false &&
    renderEvidence.classification === "captured" &&
    pixelAudit.blank === false
  );
}

const APP_RENDER_BLOCKED_ERROR_CODES = new Set([
  "runtime_unavailable",
  "unknown_tool",
  "gpui_readback_unavailable",
  "unsupported_platform",
]);

const APP_RENDER_BLOCKED_REASONS = new Set([
  "runtime_unavailable",
  "unsupported",
  "unsupported_platform",
]);

function appRenderReadbackBlocked(renderEvidence: JsonObject) {
  if (renderEvidence.source !== "gpui-render-readback") {
    return false;
  }
  if (renderEvidence.available === true || renderEvidence.countsAsAppRenderEvidence === true) {
    return false;
  }
  if (renderEvidence.classification !== "gpui-readback-unavailable") {
    return false;
  }
  return asArray(renderEvidence.attempts).map(asObject).some((attempt) =>
    attempt.status === "unsupported" ||
    APP_RENDER_BLOCKED_ERROR_CODES.has(String(attempt.errorCode ?? "")) ||
    APP_RENDER_BLOCKED_REASONS.has(String(attempt.reason ?? ""))
  );
}

function imageDiffUsability(path: string): ImageDiffUsability {
  const json = readJsonSync(path);
  if (!json) {
    return {
      usable: false,
      note: `ignored image diff ${path}: receipt could not be parsed`,
    };
  }
  const assertions = asObject(json.assertions);
  const errors = asArray(json.errors);
  const dimensions = asObject(json.dimensions);
  const sameSizeRequired = json.sameSizeRequired === true;
  const sameSize = dimensions.sameSize;
  if (json.classification !== "ok") {
    return {
      usable: false,
      note: `ignored image diff ${path}: classification is ${String(json.classification ?? "missing")}`,
    };
  }
  if (assertions.diffMaskWritten !== true) {
    return {
      usable: false,
      note: `ignored image diff ${path}: diffMaskWritten assertion is not true`,
    };
  }
  if (assertions.redCountsAsOsScreenshotEvidence === false) {
    return {
      usable: false,
      note: `ignored image diff ${path}: red-os-evidence-missing`,
    };
  }
  if (assertions.greenCountsAsOsScreenshotEvidence === false) {
    return {
      usable: false,
      note: `ignored image diff ${path}: green-os-evidence-missing`,
    };
  }
  if (assertions.changedPixelsMeasured !== true) {
    return {
      usable: false,
      note: `ignored image diff ${path}: changedPixelsMeasured assertion is not true`,
    };
  }
  if (errors.length > 0) {
    return {
      usable: false,
      note: `ignored image diff ${path}: receipt has errors`,
    };
  }
  if (sameSizeRequired && sameSize !== true) {
    return {
      usable: false,
      note: `ignored image diff ${path}: sameSizeRequired is true but dimensions.sameSize is not true`,
    };
  }

  return { usable: true };
}

function guidelineAssertionFailureCount(value: unknown): number {
  if (Array.isArray(value)) {
    return value.length;
  }
  if (typeof value !== "object" || value === null) {
    return 0;
  }
  const object = value as JsonObject;
  let count = 0;
  for (const [key, child] of Object.entries(object)) {
    if (
      key === "failures" ||
      key === "contentGlassNodes" ||
      key === "contentNativeMaterialNodes" ||
      key === "glassLayerViolations" ||
      key === "hardcodedColorNodes"
    ) {
      count += asArray(child).length;
      continue;
    }
    if (key === "clippedNodeCount" && typeof child === "number") {
      count += child;
      continue;
    }
    if (key === "overflowY" && child === true) {
      count += 1;
      continue;
    }
    count += guidelineAssertionFailureCount(child);
  }
  return count;
}

function guidelineFailureDetails(value: unknown, path: string[] = []): string[] {
  if (Array.isArray(value)) {
    if (value.length === 0) {
      return [];
    }
    const label = path.join(".");
    return value.map((entry) => `${label}: ${String(entry)}`);
  }
  if (typeof value !== "object" || value === null) {
    return [];
  }
  const object = value as JsonObject;
  const details: string[] = [];
  for (const [key, child] of Object.entries(object)) {
    if (
      key === "failures" ||
      key === "contentGlassNodes" ||
      key === "contentNativeMaterialNodes" ||
      key === "glassLayerViolations" ||
      key === "hardcodedColorNodes"
    ) {
      details.push(...guidelineFailureDetails(child, path));
      continue;
    }
    if (key === "clippedNodeCount" && typeof child === "number" && child > 0) {
      details.push(`${path.join(".")}.clippedNodeCount: ${child}`);
      continue;
    }
    if (key === "overflowY" && child === true) {
      details.push(`${path.join(".")}.overflowY: true`);
      continue;
    }
    details.push(...guidelineFailureDetails(child, [...path, key]));
  }
  return details;
}

function guidelineProof(audit: JsonObject): ProofTiers["guidelineProof"] {
  const assertions = asObject(audit.guidelineAssertions);
  if (Object.keys(assertions).length === 0) {
    return "missing";
  }
  return guidelineAssertionFailureCount(assertions) === 0 ? "pass" : "fail";
}

function sourceUiGapsFromAudit(audit: JsonObject) {
  const gaps: string[] = [];
  if (asArray(audit.controlsWithHitFailures).length > 0) gaps.push("controlsWithHitFailures");
  if (asArray(audit.contentGlassNodes).length > 0) gaps.push("contentGlassNodes");
  if (asArray(audit.contentNativeMaterialNodes).length > 0) gaps.push("contentNativeMaterialNodes");
  if (asArray(audit.glassLayerViolations).length > 0) gaps.push("glassLayerViolations");
  if (asArray(audit.missingStyleNodeNames).length > 0) gaps.push("missingStyleNodeNames");
  if (guidelineProof(audit) === "fail") gaps.push("guidelineAssertions");
  return gaps;
}

function sourceUiGapsFromEvidence(evidence: Evidence) {
  return sourceUiGapsFromAudit(asObject(evidence.visualAudit));
}

function devtoolsCaptureLimitations(tiers: ProofTiers) {
  return [
    tiers.osScreenshotProof === "blocked" ? "osScreenshotProof:blocked" : "",
    tiers.imageDiffProof === "blocked" ? "imageDiffProof:blocked" : "",
    tiers.appRenderProof === "blocked" ? "appRenderProof:blocked" : "",
    tiers.offscreenRenderProof === "fail" ? "offscreenRenderProof:fail" : "",
  ].filter(Boolean);
}

function osScreenshotBlockerCounts(surfaces: Array<{ osScreenshotBlockers?: unknown }>) {
  const counts: Record<string, number> = {};
  for (const surface of surfaces) {
    const classifications = new Set(
      asArray(surface.osScreenshotBlockers)
        .map(asObject)
        .map((blocker) => String(blocker.classification ?? "unknown"))
        .filter(Boolean),
    );
    for (const classification of classifications) {
      counts[classification] = (counts[classification] ?? 0) + 1;
    }
  }
  return counts;
}

function flattenOsCaptureAttempts(blockers: unknown[]) {
  return blockers.flatMap((blocker) => asArray(asObject(blocker).attempts).map(asObject));
}

function osCaptureBlockerCode(blockers: JsonObject[]) {
  const classifications = new Set(blockers.map((blocker) => String(blocker.classification ?? "")));
  const blockerCodes = new Set(blockers.map((blocker) => String(blocker.blockerCode ?? "")).filter(Boolean));
  const attempts = flattenOsCaptureAttempts(blockers);
  const errorCodes = new Set(attempts.map((attempt) => String(attempt.errorCode ?? "")));
  const errorText = blockers.map((blocker) => JSON.stringify(blocker.error ?? "")).join("\n");
  for (const code of OS_CAPTURE_BLOCKER_CODES) {
    if (blockerCodes.has(code)) return code;
  }
  if (classifications.has("blank-image-rejected") || classifications.has("low-content-capture-rejected")) {
    return "blank-compositor-capture";
  }
  if (errorCodes.has(SCREEN_CAPTURE_RECT_FAILED)) {
    return "screen-rect-capture-blocked";
  }
  if (errorCodes.has("not_capture_candidate")) {
    return "target-not-capture-candidate";
  }
  if (errorCodes.has(SCREEN_CAPTURE_WINDOW_FAILED) || classifications.has(MACOS_WINDOWSERVER_CAPTURE_BLOCKED)) {
    return "window-id-api-blocked";
  }
  if (errorText.includes("could not create image from window")) {
    return "window-id-api-blocked";
  }
  if (
    errorText.includes("No focused automation window") ||
    errorText.includes("requires a native osWindowId") ||
    errorText.includes("No response matching requestId")
  ) {
    return "target-identity-blocked";
  }
  return blockers.length > 0 ? "unknown-capture-failure" : null;
}

function osCaptureFromEvidence(evidence: Evidence, tiers: ProofTiers) {
  const capturedVisualEvidence = evidence.receipts
    .map(readJsonSync)
    .map((json) => asObject(json?.visualEvidence))
    .find((visualEvidence) =>
      visualEvidence.source === "os-window-capture" &&
      visualEvidence.classification === "captured" &&
      visualEvidence.countsAsOsScreenshotEvidence === true
    );
  const captureKind = capturedVisualEvidence
    ? String(capturedVisualEvidence.captureKind ?? "window-id")
    : null;
  const blockerCode = capturedVisualEvidence
    ? "none"
    : osCaptureBlockerCode(evidence.osScreenshotBlockers);
  const attempts = [
    ...flattenOsCaptureAttempts(evidence.osScreenshotBlockers),
    ...captureAttemptSummary(capturedVisualEvidence?.attempts),
  ];
  const windowIdFailed = attempts.some((attempt) => attempt.errorCode === SCREEN_CAPTURE_WINDOW_FAILED);
  const rectFailed = attempts.some((attempt) => attempt.errorCode === SCREEN_CAPTURE_RECT_FAILED);
  return {
    proof: tiers.osScreenshotProof,
    captureKind,
    windowIdCaptureProof: captureKind === "window-id"
      ? "pass"
      : windowIdFailed
        ? "blocked"
        : "missing",
    compositorCaptureProof: tiers.osScreenshotProof === "pass"
      ? "pass"
      : tiers.osScreenshotProof === "blocked"
        ? "blocked"
        : "missing",
    countsAsCompositorEvidence: capturedVisualEvidence
      ? capturedVisualEvidence.countsAsCompositorEvidence === true ||
        capturedVisualEvidence.countsAsOsScreenshotEvidence === true
      : false,
    blockerCode,
    blockers: evidence.osScreenshotBlockers,
    attempts,
    assertions: {
      countsAsOsScreenshotEvidence: capturedVisualEvidence?.countsAsOsScreenshotEvidence === true,
      countsAsCompositorEvidence: capturedVisualEvidence
        ? capturedVisualEvidence.countsAsCompositorEvidence === true ||
          capturedVisualEvidence.countsAsOsScreenshotEvidence === true
        : false,
      windowIdCaptureAttempted: windowIdFailed || captureKind === "window-id",
      screenRectCaptureAttempted: rectFailed || captureKind === "screen-rect",
    },
  };
}

function diagnosticLimitations(tiers: ProofTiers) {
  return [
    tiers.appRenderProof === "blocked" ? "appRenderProof:blocked" : "",
    tiers.appRenderProof === "fail" ? "appRenderProof:fail" : "",
    tiers.offscreenRenderProof === "fail" ? "offscreenRenderProof:fail" : "",
  ].filter(Boolean);
}

function guidanceProofStatus(evidence: Evidence, tiers: ProofTiers): GuidanceProofStatus {
  const sourceGaps = sourceUiGapsFromEvidence(evidence);
  if (tiers.numericProof === "fail" || sourceGaps.length > 0) {
    return "source-ui-gap";
  }
  if (evidence.layoutReceiptFreshnessLimitations.length > 0) {
    return "stale-layout-evidence";
  }
  if (tiers.numericProof !== "pass") {
    return "missing-guidance-proof";
  }
  if (tiers.osScreenshotProof === "pass" && tiers.imageDiffProof === "pass") {
    return "strong-guidance-proof";
  }
  if (tiers.osScreenshotProof === "blocked" || tiers.imageDiffProof === "blocked") {
    return "guidance-proof-capture-blocked";
  }
  return "numeric-guidance-proof-missing-os-visual";
}

function guidanceEvidenceNeeded(requiredEvidence: JsonObject) {
  return [
    requiredEvidence.numericProof !== "pass" ? "numericProof" : "",
    requiredEvidence.osScreenshotProof !== "pass" ? "osScreenshotProof" : "",
    requiredEvidence.imageDiffProof !== "pass" ? "imageDiffProof" : "",
  ].filter(Boolean);
}

function diagnosticEvidenceNeeded(requiredEvidence: JsonObject) {
  return [
    requiredEvidence.appRenderProof !== "pass" ? "appRenderProof" : "",
    requiredEvidence.offscreenRenderProof !== "pass" ? "offscreenRenderProof" : "",
  ].filter(Boolean);
}

function evidenceFor(terms: string[], files: { receipts: string[]; screenshots: string[]; diffs: string[] }, visualAuditPath?: string): Evidence {
  const receipts = files.receipts.filter((path) => includesAny(path, terms));
  const screenshots: string[] = [];
  const screenshotNotes: string[] = [];
  for (const path of files.screenshots.filter((screenshotPath) => includesAny(screenshotPath, terms))) {
    const usability = screenshotUsability(path, files.receipts);
    if (usability.usable) {
      screenshots.push(path);
    } else if (usability.note) {
      screenshotNotes.push(usability.note);
    }
  }
  const imageDiffReceipts: string[] = [];
  const imageDiffNotes: string[] = [];
  for (const path of receipts.filter((receiptPath) => receiptPath.includes("image-diff"))) {
    const usability = imageDiffUsability(path);
    if (usability.usable) {
      imageDiffReceipts.push(path);
    } else if (usability.note) {
      imageDiffNotes.push(usability.note);
    }
  }
  return {
    receipts,
    screenshots,
    layoutReceipts: receipts.filter((path) => path.includes("layout")),
    inspectReceipts: receipts.filter((path) => path.includes("inspect") || path.includes("window")),
    imageDiffReceipts,
    diffMasks: files.diffs.filter((path) => includesAny(path, terms)),
    visualAudit: null,
    osScreenshotBlockers: receipts
      .map(osScreenshotBlockerFromReceipt)
      .filter((blocker): blocker is JsonObject => blocker != null),
    layoutReceiptFreshnessLimitations: [],
    notes: [
      ...screenshotNotes,
      ...imageDiffNotes,
      ...(visualAuditPath ? [`visualAudit sourced from ${visualAuditPath}`] : []),
    ],
  };
}

function proofTiers(evidence: Evidence): ProofTiers {
  let osBlocked = evidence.osScreenshotBlockers.length > 0;
  let appRenderPass = false;
  let appRenderBlocked = false;
  let appRenderFail = false;
  let offscreenPass = false;
  let offscreenFail = false;
  for (const receipt of evidence.receipts) {
    const json = readJsonSync(receipt);
    const visualEvidence = asObject(json?.visualEvidence);
    if (
      visualEvidence.source === "os-window-capture" &&
      visualEvidence.available === false &&
      visualEvidence.countsAsOsScreenshotEvidence !== true &&
      typeof visualEvidence.classification === "string"
    ) {
      osBlocked = true;
    }
    if (visualEvidence.countsAsOsScreenshotEvidence === true) {
      osBlocked = false;
    }
    const renderEvidence = asObject(json?.renderEvidence);
    if (usableAppRenderEvidence(renderEvidence)) {
      appRenderPass = true;
    } else if (Object.keys(renderEvidence).length > 0) {
      if (appRenderReadbackBlocked(renderEvidence)) {
        appRenderBlocked = true;
      } else {
        appRenderFail = true;
      }
    }
    const offscreenEvidence = asObject(json?.offscreenEvidence);
    if (Object.keys(offscreenEvidence).length > 0) {
      if (
        offscreenEvidence.source === "gpui-offscreen-render" &&
        offscreenEvidence.available === true &&
        offscreenEvidence.countsAsOffscreenRenderEvidence === true
      ) {
        offscreenPass = true;
      } else {
        offscreenFail = true;
      }
    }
  }
  const audit = asObject(evidence.visualAudit);
  const styled = typeof audit.styledNodeCount === "number" ? audit.styledNodeCount : null;
  const nodeCount = typeof audit.nodeCount === "number" ? audit.nodeCount : null;
  const numericPass =
    evidence.layoutReceipts.length > 0 &&
    nodeCount != null &&
    nodeCount > 0 &&
    styled === nodeCount &&
    asArray(audit.controlsWithHitFailures).length === 0 &&
    asArray(audit.contentGlassNodes).length === 0 &&
    asArray(audit.contentNativeMaterialNodes).length === 0 &&
    asArray(audit.glassLayerViolations).length === 0 &&
    asArray(audit.missingStyleNodeNames).length === 0;
  const guidelines = guidelineProof(audit);
  return {
    osScreenshotProof: evidence.screenshots.length > 0 ? "pass" : osBlocked ? "blocked" : "missing",
    appRenderProof: appRenderPass ? "pass" : appRenderFail ? "fail" : appRenderBlocked ? "blocked" : "missing",
    offscreenRenderProof: offscreenPass ? "pass" : offscreenFail ? "fail" : "missing",
    numericProof: numericPass ? "pass" : evidence.layoutReceipts.length > 0 ? "fail" : "missing",
    guidelineProof: guidelines,
    imageDiffProof: evidence.imageDiffReceipts.length > 0 ? "pass" : osBlocked ? "blocked" : "missing",
  };
}

function classify(evidence: Evidence) {
  const hasScreenshot = evidence.screenshots.length > 0;
  const hasLayout = evidence.layoutReceipts.length > 0;
  const hasImageDiff = evidence.imageDiffReceipts.length > 0;
  const tiers = proofTiers(evidence);
  const audit = asObject(evidence.visualAudit);
  const styled = typeof audit.styledNodeCount === "number" ? audit.styledNodeCount : null;
  const nodeCount = typeof audit.nodeCount === "number" ? audit.nodeCount : null;
  const hitFailures = asArray(audit.controlsWithHitFailures).length;
  const contentGlass = asArray(audit.contentGlassNodes).length;
  const contentNativeMaterial = asArray(audit.contentNativeMaterialNodes).length;
  const glassLayerViolations = asArray(audit.glassLayerViolations).length;
  const missingStyle = asArray(audit.missingStyleNodeNames).length;

  if (hasScreenshot && hasLayout && hasImageDiff && tiers.guidelineProof === "pass" && nodeCount != null && nodeCount > 0 && styled === nodeCount && hitFailures === 0 && contentGlass === 0 && contentNativeMaterial === 0 && glassLayerViolations === 0 && missingStyle === 0) {
    return "strong-proof";
  }
  if (hasLayout && nodeCount != null && nodeCount > 0 && styled === nodeCount && hitFailures === 0 && contentGlass === 0 && contentNativeMaterial === 0 && glassLayerViolations === 0 && missingStyle === 0) {
    if (tiers.guidelineProof === "fail") {
      return "numeric-proof-guideline-failed";
    }
    if (tiers.guidelineProof === "missing") {
      return "numeric-proof-missing-guideline-assertions";
    }
    if (tiers.appRenderProof === "pass" && tiers.osScreenshotProof !== "pass") {
      return tiers.osScreenshotProof === "blocked"
        ? "numeric-plus-app-render-proof-os-screenshot-blocked"
        : "numeric-plus-app-render-proof-missing-os-screenshot";
    }
    if (tiers.appRenderProof === "fail") {
      return "numeric-proof-app-render-attempted-failed";
    }
    if (tiers.appRenderProof === "blocked") {
      return "numeric-proof-app-render-blocked";
    }
    return "numeric-proof-missing-visual-capture";
  }
  if (hasScreenshot && hasLayout) {
    return "numeric-proof-no-image-diff";
  }
  if (hasScreenshot) {
    return "baseline-only";
  }
  return "missing-proof";
}

const OUTSIDE_IN_SURFACE_PRIORITY: Record<string, number> = {
  PromptEntity: 10,
  PromptChildContent: 11,
  ExplicitPromptEntity: 12,
  UtilityChildContent: 13,
  AcpChat: 14,
  FileSearchMini: 15,
  FileSearchFull: 16,
  AttachmentPortalBrowser: 17,
  ConfirmPrompt: 18,
  Webcam: 19,
  ClipboardHistory: 30,
  AppLauncher: 31,
  WindowSwitcher: 32,
  BrowserTabs: 33,
  GenericFilterableList: 34,
  ProcessManager: 35,
  CurrentAppCommands: 36,
  Settings: 37,
  KitStoreBrowse: 38,
  KitStoreInstalled: 39,
  AcpHistory: 40,
  ThemeChooser: 41,
  EmojiPicker: 42,
  SdkReference: 43,
  ScriptTemplateCatalog: 44,
};

function outsideInPriority(surfaceKind: unknown) {
  return OUTSIDE_IN_SURFACE_PRIORITY[String(surfaceKind)] ?? 90;
}

async function attachVisualAudit(evidence: Evidence, preferred: string[]) {
  for (const path of [...preferred, ...evidence.layoutReceipts]) {
    const json = await readJsonIfExists(path);
    const audit = Object.keys(asObject(json?.visualAudit)).length > 0
      ? asObject(json?.visualAudit)
      : asObject(asObject(asObject(json?.receipts).layout).visualAudit);
    if (Object.keys(audit).length > 0) {
      const syntheticRadiusFailures = nodesWithMissingPositiveRadius(asObject(json));
      evidence.visualAudit = audit;
      if (syntheticRadiusFailures.length > 0 && !hasExplicitCornerRadiusFailures(audit)) {
        evidence.layoutReceiptFreshnessLimitations.push(
          `legacy layout receipt lacks explicit cornerRadiusTokens for ${syntheticRadiusFailures.join(", ")}: ${path}`,
        );
      }
      evidence.notes.push(`visualAudit: ${path}`);
      return;
    }
  }
}

function hasExplicitCornerRadiusFailures(audit: JsonObject) {
  const failures = asArray(
    asObject(asObject(asObject(audit.guidelineAssertions).projectLocal).cornerRadiusTokens).failures,
  );
  return failures.length > 0;
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  RECEIPT_ROOT = `${args.artifactRoot}/receipts`;
  SCREENSHOT_ROOT = `${args.artifactRoot}/screenshots`;
  DIFF_ROOT = `${args.artifactRoot}/diffs`;
  const inventory = JSON.parse(await Bun.file(args.inventory).text()) as JsonObject;
  const files = {
    receipts: await listFiles(`${args.artifactRoot}/receipts`),
    screenshots: await listFiles(`${args.artifactRoot}/screenshots`),
    diffs: await listFiles(`${args.artifactRoot}/diffs`),
  };

  const surfaceTerms: Record<string, string[]> = {
    ScriptList: ["main"],
    ActionsDialog: ["actions"],
    ConfirmPrompt: ["confirm"],
    PromptEntity: ["prompt-div", "promptentity", "prompt-entity"],
    PromptChildContent: ["prompt-child", "promptchildcontent"],
    ExplicitPromptEntity: ["prompt-explicit", "explicitpromptentity"],
    UtilityChildContent: ["utility-quick", "utilitychildcontent"],
    Webcam: ["webcam"],
    About: ["about"],
    Feedback: ["feedback", "creation-feedback", "creationFeedback"],
    AcpChat: ["acp"],
    AcpHistory: ["acp-history"],
    ClipboardHistory: ["clipboard"],
    AppLauncher: ["app-launcher", "applauncher"],
    WindowSwitcher: ["window-switcher", "windowswitcher"],
    BrowserTabs: ["browser-tabs", "browsertabs"],
    GenericFilterableList: [
      "generic-filterable",
      "generic-filterable-list",
      "favorites",
      "favoritesBrowse",
      "search-ai-presets",
      "searchAiPresets",
    ],
    ProcessManager: ["process-manager", "processmanager"],
    CurrentAppCommands: ["current-app", "current-app-commands", "currentappcommands"],
    Settings: ["settings"],
    KitStoreBrowse: ["kit-store-browse", "kitstorebrowse"],
    KitStoreInstalled: ["kit-store-installed", "kitstoreinstalled"],
    EmojiPicker: ["emoji", "emoji-picker", "emojipicker"],
    ThemeChooser: ["theme", "choose-theme", "theme-chooser", "themechooser"],
    FileSearchMini: ["file-search-mini", "file-search-owned"],
    FileSearchFull: ["file-search-full"],
    AttachmentPortalBrowser: ["attachment-portal", "dictation-history"],
    SdkReference: ["sdk-reference"],
    ScriptTemplateCatalog: ["script-template", "script-template-catalog"],
    NotesWindow: ["notes"],
    Dictation: ["dictation"],
  };

  const surfaceContracts = asArray(inventory.auditSurfaceContracts ?? inventory.surfaceContracts).map((entry) => asObject(entry));
  const recommendedBatches = asArray(inventory.recommendedOracleBatches).map((entry) => asObject(entry));
  const surfaces = await Promise.all(surfaceContracts.map(async (contract) => {
    const surfaceKind = String(contract.surfaceKind ?? "");
    const terms = surfaceTerms[surfaceKind] ?? [surfaceKind.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase(), surfaceKind.toLowerCase()];
    const evidence = evidenceFor(terms, files);
    if (surfaceKind === "ScriptList") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-main-backdrop-current-layout.json`,
        `${RECEIPT_ROOT}/tahoe-next-main-layout.json`,
        `${RECEIPT_ROOT}/after-main-layout-visual-style.json`,
      ]);
    } else if (surfaceKind === "ActionsDialog") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-actions-guideline-layout.json`,
        `${RECEIPT_ROOT}/tahoe-next-actions-layout.json`,
        `${RECEIPT_ROOT}/after-actions-layout-visual-style.json`,
      ]);
    } else if (surfaceKind === "ConfirmPrompt") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-confirm-guideline-layout.json`,
        `${RECEIPT_ROOT}/window-priority-confirm-layout-after.json`,
      ]);
      const confirmScreenshotReceipt = await readJsonIfExists(
        `${RECEIPT_ROOT}/window-priority-confirm-screenshot-after.json`,
      );
      const visualEvidence = asObject(confirmScreenshotReceipt?.visualEvidence);
      const screenshotReceipt = asObject(confirmScreenshotReceipt?.screenshotReceipt);
      const afterScreenshotPass =
        confirmScreenshotReceipt?.status === "pass" &&
        (visualEvidence.countsAsOsScreenshotEvidence === true ||
          screenshotReceipt.captured === true);
      if (!afterScreenshotPass) {
        evidence.screenshots = [];
        evidence.notes.push(
          "ignored ConfirmPrompt screenshot evidence: window-priority-confirm-screenshot-after.json is not a passing after-capture receipt",
        );
      }
    } else if (surfaceKind === "About") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-about-current-layout.json`,
        `${RECEIPT_ROOT}/window-priority-about-layout-after.json`,
      ]);
    } else if (surfaceKind === "Feedback") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-feedback-guideline-layout.json`,
        `${RECEIPT_ROOT}/window-priority-feedback-layout-after.json`,
      ]);
    } else if (surfaceKind === "Dictation") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-dictation-layout-after.json`,
      ]);
    } else if (surfaceKind === "AcpChat") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-acp-detached-current-layout.json`,
        `${RECEIPT_ROOT}/window-priority-acp-detached-layout-after.json`,
      ]);
    } else if (surfaceKind === "ClipboardHistory") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-clipboard-current-layout.json`,
      ]);
    } else if (surfaceKind === "AppLauncher") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-app-launcher-current-layout.json`,
      ]);
    } else if (surfaceKind === "WindowSwitcher") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-window-switcher-current-layout.json`,
      ]);
    } else if (surfaceKind === "BrowserTabs") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-browser-tabs-current-layout.json`,
      ]);
    } else if (surfaceKind === "GenericFilterableList") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-generic-filterable-favorites-current-layout.json`,
        `${RECEIPT_ROOT}/window-priority-generic-filterable-search-ai-presets-current-layout.json`,
      ]);
    } else if (surfaceKind === "ProcessManager") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-process-manager-current-layout.json`,
      ]);
    } else if (surfaceKind === "CurrentAppCommands") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-current-app-current-layout.json`,
      ]);
    } else if (surfaceKind === "Settings") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-settings-current-layout.json`,
      ]);
    } else if (surfaceKind === "KitStoreBrowse") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-kit-store-browse-current-layout.json`,
      ]);
    } else if (surfaceKind === "KitStoreInstalled") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-kit-store-installed-current-layout.json`,
      ]);
    } else if (surfaceKind === "AcpHistory") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-acp-history-current-layout.json`,
      ]);
    } else if (surfaceKind === "FileSearchMini") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-file-search-mini-current-layout.json`,
      ]);
    } else if (surfaceKind === "FileSearchFull") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-file-search-full-current-layout.json`,
      ]);
    } else if (surfaceKind === "AttachmentPortalBrowser") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-attachment-portal-current-layout.json`,
      ]);
    } else if (surfaceKind === "SdkReference") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-sdk-reference-current-layout.json`,
      ]);
    } else if (surfaceKind === "ScriptTemplateCatalog") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-script-template-current-layout.json`,
      ]);
    } else if (surfaceKind === "EmojiPicker") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-emoji-current-layout.json`,
      ]);
    } else if (surfaceKind === "ThemeChooser") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-theme-current-layout.json`,
      ]);
    } else if (surfaceKind === "PromptEntity") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-prompt-div-fixed-layout-devtools.json`,
        `${RECEIPT_ROOT}/window-priority-prompt-div-guideline-layout.json`,
        `${RECEIPT_ROOT}/window-priority-prompt-div-fixed-layout-sdk.json`,
        `${RECEIPT_ROOT}/window-priority-prompt-div-current-layout-sdk.json`,
      ]);
    } else if (surfaceKind === "PromptChildContent") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-prompt-child-editor-guideline-layout.json`,
        `${RECEIPT_ROOT}/window-priority-prompt-child-editor-fixed-layout-sdk.json`,
      ]);
    } else if (surfaceKind === "ExplicitPromptEntity") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-prompt-explicit-env-guideline-layout-sdk.json`,
        `${RECEIPT_ROOT}/window-priority-prompt-explicit-env-fixed-layout-sdk.json`,
        `${RECEIPT_ROOT}/window-priority-prompt-explicit-env-current-layout-sdk.json`,
      ]);
    } else if (surfaceKind === "UtilityChildContent") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-utility-quick-guideline-layout.json`,
        `${RECEIPT_ROOT}/window-priority-utility-quick-fixed-layout.json`,
        `${RECEIPT_ROOT}/window-priority-utility-quick-current-layout.json`,
      ]);
    } else if (surfaceKind === "Webcam") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-webcam-fixed-layout.json`,
        `${RECEIPT_ROOT}/window-priority-webcam-current-layout.json`,
      ]);
    }
    if (evidence.visualAudit == null) {
      await attachVisualAudit(evidence, []);
    }
    const status = classify(evidence);
    const tiers = proofTiers(evidence);
    const osCapture = osCaptureFromEvidence(evidence, tiers);
    const guidanceStatus = guidanceProofStatus(evidence, tiers);
    const sourceGaps = sourceUiGapsFromEvidence(evidence);
    const captureLimitations = devtoolsCaptureLimitations(tiers);
    const diagnostics = diagnosticLimitations(tiers);
    const requiredEvidence = {
      screenshot: evidence.screenshots.length > 0,
      osScreenshotProof: tiers.osScreenshotProof,
      appRenderProof: tiers.appRenderProof,
      offscreenRenderProof: tiers.offscreenRenderProof,
      numericLayout: evidence.layoutReceipts.length > 0,
      numericProof: tiers.numericProof,
      guidelineProof: tiers.guidelineProof,
      imageDiff: evidence.imageDiffReceipts.length > 0,
      imageDiffProof: tiers.imageDiffProof,
      visualAudit: evidence.visualAudit != null,
    };
    return {
      surfaceKind,
      appViewVariants: contract.appViewVariants ?? [],
      automationSemanticSurface: contract.automationSemanticSurface ?? null,
      coverageAliases: contract.coverageAliases ?? [],
      proofStatus: status,
      guidanceProofStatus: guidanceStatus,
      sourceUiGaps: sourceGaps,
      devtoolsCaptureLimitations: captureLimitations,
      osScreenshotBlockers: evidence.osScreenshotBlockers,
      layoutReceiptFreshnessLimitations: evidence.layoutReceiptFreshnessLimitations,
      diagnosticLimitations: diagnostics,
      guidanceEvidenceNeeded: guidanceEvidenceNeeded(requiredEvidence),
      diagnosticEvidenceNeeded: diagnosticEvidenceNeeded(requiredEvidence),
      requiredEvidence,
      proofTiers: tiers,
      osCapture,
      visualAudit: evidence.visualAudit,
      evidence,
    };
  }));

  const practicalTargets = await Promise.all([
    { id: "notes", terms: ["notes"] },
    { id: "dictation", terms: ["dictation"] },
    { id: "acp-detached", terms: ["acp-detached"] },
    { id: "inline-agent", terms: ["inline-agent"] },
    { id: "notes-acp", terms: ["notes-acp"] },
  ].map(async (target) => {
    const evidence = evidenceFor(target.terms, files);
    if (target.id === "notes") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/notes-next-layout.json`,
      ]);
    } else if (target.id === "inline-agent") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/inline-agent-main-layout.json`,
      ]);
    } else if (target.id === "notes-acp") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/notes-acp-next-actions-open-inspect.json`,
        `${RECEIPT_ROOT}/notes-acp-actions-open-inspect.json`,
      ]);
    } else if (target.id === "dictation") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-dictation-layout-after.json`,
      ]);
    } else if (target.id === "acp-detached") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-acp-detached-layout-after.json`,
      ]);
    }
    const dictationMedia = target.id === "dictation"
      ? await readJsonIfExists(`${RECEIPT_ROOT}/dictation-next-post-delivery-media.json`)
      : null;
    const dictationDelivery = target.id === "dictation"
      ? await readJsonIfExists(`${RECEIPT_ROOT}/dictation-next-deliver-fixture.json`)
      : null;
    const baseStatus = classify(evidence);
    const tiers = proofTiers(evidence);
    const proofStatus = target.id === "dictation" && evidence.visualAudit && evidence.layoutReceipts.length > 0
      ? "numeric-window-proof-screenshot-blocked"
      : target.id === "acp-detached" && evidence.visualAudit && evidence.layoutReceipts.length > 0
        ? "numeric-window-proof-screenshot-blocked"
      : target.id === "dictation" && dictationMedia
        ? "media-proof-missing-visual"
      : target.id === "inline-agent" && evidence.visualAudit && evidence.layoutReceipts.length > 0
        ? "numeric-proof-missing-visual-capture"
        : target.id === "notes-acp" && evidence.visualAudit && evidence.inspectReceipts.length > 0
          ? "actions-panel-proof-acp-startup-blocked"
        : baseStatus;
    return {
      id: target.id,
      proofStatus,
      requiredEvidence: {
        screenshot: evidence.screenshots.length > 0,
        osScreenshotProof: tiers.osScreenshotProof,
        appRenderProof: tiers.appRenderProof,
        offscreenRenderProof: tiers.offscreenRenderProof,
        numericLayout: evidence.layoutReceipts.length > 0,
        numericProof: tiers.numericProof,
        guidelineProof: tiers.guidelineProof,
        imageDiff: evidence.imageDiffReceipts.length > 0,
        imageDiffProof: tiers.imageDiffProof,
        visualAudit: evidence.visualAudit != null,
        mediaProof: dictationMedia != null,
        syntheticDelivery: dictationDelivery != null,
      },
      proofTiers: tiers,
      osCapture: osCaptureFromEvidence(evidence, tiers),
      mediaProof: dictationMedia
        ? {
          status: dictationMedia.status ?? null,
          missingRuntimePrimitives: dictationMedia.missingRuntimePrimitives ?? [],
        }
        : null,
      syntheticDelivery: dictationDelivery
        ? {
          classification: dictationDelivery.classification ?? null,
          target: dictationDelivery.target ?? null,
          delivery: dictationDelivery.delivery ?? null,
        }
        : null,
      evidence,
    };
  }));

  const byKind = new Map(surfaces.map((surface) => [String(surface.surfaceKind), surface]));
  const batches = recommendedBatches.map((batch) => {
    const surfaceKinds = asArray(batch.surfaceKinds).map(String);
    const entries = surfaceKinds.map((surfaceKind) => byKind.get(surfaceKind)).filter(Boolean);
    const statuses = entries.map((entry) => String(entry?.proofStatus ?? "missing-proof"));
    const complete = statuses.length > 0 && statuses.every((status) => status === "strong-proof");
    return {
      id: batch.id,
      name: batch.name,
      owners: batch.owners ?? [],
      surfaceKinds,
      requiredDevToolsPrimitives: batch.requiredDevToolsPrimitives ?? [],
      proofStatus: complete ? "strong-proof" : statuses.some((status) => status !== "missing-proof") ? "partial-proof" : "missing-proof",
      provenSurfaceCount: statuses.filter((status) => status === "strong-proof").length,
      partialSurfaceCount: statuses.filter((status) => status !== "missing-proof" && status !== "strong-proof").length,
      missingSurfaceCount: statuses.filter((status) => status === "missing-proof").length,
      surfaceStatuses: entries.map((entry) => ({
        surfaceKind: entry?.surfaceKind,
        proofStatus: entry?.proofStatus,
      })),
    };
  });

  const visualTierDebtSurfaces = surfaces
    .filter((surface) =>
      surface.proofTiers.osScreenshotProof === "blocked" ||
      surface.proofTiers.numericProof === "fail" ||
      surface.proofTiers.guidelineProof === "fail" ||
      surface.proofTiers.imageDiffProof === "blocked" ||
      surface.guidanceProofStatus === "source-ui-gap" ||
      surface.guidanceProofStatus === "guidance-proof-capture-blocked"
    )
    .map((surface) => ({
      surfaceKind: surface.surfaceKind,
      proofStatus: surface.proofStatus,
      guidanceProofStatus: surface.guidanceProofStatus,
      proofTiers: surface.proofTiers,
      failedTiers: [
        surface.proofTiers.osScreenshotProof === "blocked" ? "osScreenshotProof" : "",
        surface.proofTiers.numericProof === "fail" ? "numericProof" : "",
        surface.proofTiers.guidelineProof === "fail" ? "guidelineProof" : "",
        surface.proofTiers.imageDiffProof === "blocked" ? "imageDiffProof" : "",
      ].filter(Boolean),
      sourceUiGaps: surface.sourceUiGaps,
      devtoolsCaptureLimitations: surface.devtoolsCaptureLimitations,
      osScreenshotBlockers: surface.osScreenshotBlockers,
      osCapture: surface.osCapture,
      layoutReceiptFreshnessLimitations: surface.layoutReceiptFreshnessLimitations,
      guidanceEvidenceNeeded: surface.guidanceEvidenceNeeded,
      diagnosticLimitations: surface.diagnosticLimitations,
      guidelineFailures: guidelineFailureDetails(asObject(surface.visualAudit).guidelineAssertions),
      receipts: surface.evidence.receipts,
      screenshots: surface.evidence.screenshots,
      notes: surface.evidence.notes,
    }));
  const surfaceProofDebtSurfaces = surfaces
    .filter((surface) => surface.proofStatus !== "strong-proof")
    .map((surface) => ({
      surfaceKind: surface.surfaceKind,
      proofStatus: surface.proofStatus,
      guidanceProofStatus: surface.guidanceProofStatus,
      proofTiers: surface.proofTiers,
      requiredEvidence: surface.requiredEvidence,
      sourceUiGaps: surface.sourceUiGaps,
      devtoolsCaptureLimitations: surface.devtoolsCaptureLimitations,
      osScreenshotBlockers: surface.osScreenshotBlockers,
      osCapture: surface.osCapture,
      layoutReceiptFreshnessLimitations: surface.layoutReceiptFreshnessLimitations,
      diagnosticLimitations: surface.diagnosticLimitations,
      guidanceEvidenceNeeded: surface.guidanceEvidenceNeeded,
      diagnosticEvidenceNeeded: surface.diagnosticEvidenceNeeded,
      guidelineFailures: guidelineFailureDetails(asObject(surface.visualAudit).guidelineAssertions),
      receipts: surface.evidence.receipts,
      screenshots: surface.evidence.screenshots,
      notes: surface.evidence.notes,
    }));
  const proofDebtWorkQueue = surfaceProofDebtSurfaces.map((surface) => {
    const guidanceNeeded = guidanceEvidenceNeeded(surface.requiredEvidence);
    const diagnosticNeeded = diagnosticEvidenceNeeded(surface.requiredEvidence);
    const sourceUiGap = surface.guidanceProofStatus === "source-ui-gap";
    const staleLayoutEvidence = surface.guidanceProofStatus === "stale-layout-evidence";
    const compositorCaptureBlocked = surface.proofTiers.osScreenshotProof === "blocked" ||
      surface.proofTiers.imageDiffProof === "blocked";
    const diagnosticOnlyBlocked = surface.proofTiers.appRenderProof === "blocked" ||
      surface.proofTiers.appRenderProof === "fail" ||
      surface.proofTiers.offscreenRenderProof === "fail";
    const blockingClass = staleLayoutEvidence
      ? "stale-layout-evidence"
      : sourceUiGap
      ? "source-ui-gap"
      : compositorCaptureBlocked
        ? String(surface.osCapture?.blockerCode ?? "unknown-capture-failure")
        : guidanceNeeded.length > 0
          ? "missing-guidance-visual-evidence"
          : diagnosticOnlyBlocked
            ? "diagnostic-readback-limitation"
            : "none";
    return {
      rank: 0,
      surfaceKind: surface.surfaceKind,
      outsideInPriority: outsideInPriority(surface.surfaceKind),
      priorityGroup: outsideInPriority(surface.surfaceKind) < 30 ? "window-container" : "surface-content",
      proofStatus: surface.proofStatus,
      guidanceProofStatus: surface.guidanceProofStatus,
      blockingClass,
      priority: blockingClass === "source-ui-gap" || blockingClass === "devtools-capture-limitation"
        ? "capture-blocker"
        : blockingClass === "stale-layout-evidence"
          ? "freshness-refresh"
        : compositorCaptureBlocked
          ? "capture-blocker"
        : "missing-proof-tier",
      nextEvidenceNeeded: guidanceNeeded,
      guidanceEvidenceNeeded: guidanceNeeded,
      diagnosticEvidenceNeeded: diagnosticNeeded,
      sourceUiGaps: surface.sourceUiGaps,
      devtoolsCaptureLimitations: surface.devtoolsCaptureLimitations,
      osScreenshotBlockers: surface.osScreenshotBlockers,
      osCapture: surface.osCapture,
      layoutReceiptFreshnessLimitations: surface.layoutReceiptFreshnessLimitations,
      diagnosticLimitations: surface.diagnosticLimitations,
      guidelineFailures: surface.guidelineFailures,
      recommendedNextAction: blockingClass === "source-ui-gap"
        ? "fix UI/source layout/style gap, then rerun layout + OS visual proof"
        : blockingClass === "stale-layout-evidence"
          ? "refresh layout receipt from current target-agent binary; do not edit source until a current receipt still fails panel radii"
        : compositorCaptureBlocked
          ? `resolve OS compositor capture blocker ${blockingClass}; current source/layout proof is not contradicted`
          : blockingClass === "missing-guidance-visual-evidence"
            ? "run strict OS screenshot capture and image diff for this surface"
            : blockingClass === "diagnostic-readback-limitation"
              ? "record GPUI readback limitation as diagnostic only; do not block Apple compositor proof"
              : "no action",
      receipts: surface.receipts,
      screenshots: surface.screenshots,
      notes: surface.notes,
    };
  })
    .sort((left, right) =>
      left.outsideInPriority - right.outsideInPriority ||
      String(left.surfaceKind).localeCompare(String(right.surfaceKind))
    )
    .map((entry, index) => ({ ...entry, rank: index + 1 }));

  const summary = {
    surfaceCount: surfaces.length,
    strongProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus === "strong-proof").length,
    partialProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus !== "missing-proof" && surface.proofStatus !== "strong-proof").length,
    missingProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus === "missing-proof").length,
    osScreenshotBlockedSurfaceCount: surfaces.filter((surface) => surface.proofTiers.osScreenshotProof === "blocked").length,
    osScreenshotBlockerCounts: osScreenshotBlockerCounts(surfaces),
    appRenderProofSurfaceCount: surfaces.filter((surface) => surface.proofTiers.appRenderProof === "pass").length,
    appRenderBlockedSurfaceCount: surfaces.filter((surface) => surface.proofTiers.appRenderProof === "blocked").length,
    appRenderFailedSurfaceCount: surfaces.filter((surface) => surface.proofTiers.appRenderProof === "fail").length,
    appRenderMissingSurfaceCount: surfaces.filter((surface) => surface.proofTiers.appRenderProof === "missing").length,
    offscreenRenderFailedSurfaceCount: surfaces.filter((surface) => surface.proofTiers.offscreenRenderProof === "fail").length,
    offscreenRenderMissingSurfaceCount: surfaces.filter((surface) => surface.proofTiers.offscreenRenderProof === "missing").length,
    guidelineProofSurfaceCount: surfaces.filter((surface) => surface.proofTiers.guidelineProof === "pass").length,
    guidelineFailedSurfaceCount: surfaces.filter((surface) => surface.proofTiers.guidelineProof === "fail").length,
    guidelineMissingSurfaceCount: surfaces.filter((surface) => surface.proofTiers.guidelineProof === "missing").length,
    strongGuidanceProofSurfaceCount: surfaces.filter((surface) => surface.guidanceProofStatus === "strong-guidance-proof").length,
    staleLayoutEvidenceSurfaceCount: surfaces.filter((surface) => surface.guidanceProofStatus === "stale-layout-evidence").length,
    sourceUiGapSurfaceCount: surfaces.filter((surface) => surface.guidanceProofStatus === "source-ui-gap").length,
    guidanceCaptureBlockedSurfaceCount: surfaces.filter((surface) => surface.guidanceProofStatus === "guidance-proof-capture-blocked").length,
    numericGuidanceMissingOsVisualSurfaceCount: surfaces.filter((surface) => surface.guidanceProofStatus === "numeric-guidance-proof-missing-os-visual").length,
    missingGuidanceProofSurfaceCount: surfaces.filter((surface) => surface.guidanceProofStatus === "missing-guidance-proof").length,
    layoutReceiptFreshnessLimitationSurfaceCount: surfaces.filter((surface) => asArray(surface.layoutReceiptFreshnessLimitations).length > 0).length,
    diagnosticReadbackLimitationSurfaceCount: surfaces.filter((surface) => asArray(surface.diagnosticLimitations).length > 0).length,
    visualTierDebtSurfaceCount: visualTierDebtSurfaces.length,
    surfaceProofDebtCount: surfaceProofDebtSurfaces.length,
    proofDebtWorkQueueCount: proofDebtWorkQueue.length,
    batchCount: batches.length,
    strongProofBatchCount: batches.filter((batch) => batch.proofStatus === "strong-proof").length,
    partialProofBatchCount: batches.filter((batch) => batch.proofStatus === "partial-proof").length,
    missingProofBatchCount: batches.filter((batch) => batch.proofStatus === "missing-proof").length,
  };

  const receipt = {
    schemaVersion: 1,
    tool: "script-kit-devtools.liquid-glass-proof",
    command: "proof.matrix",
    classification: summary.missingProofSurfaceCount === 0 && summary.visualTierDebtSurfaceCount === 0 && summary.surfaceProofDebtCount === 0 ? "ok" : "incomplete",
    inventoryPath: args.inventory,
    artifactRoot: args.artifactRoot,
    generatedAt: new Date().toISOString(),
    summary,
    batches,
    visualTierDebtSurfaces,
    surfaceProofDebtSurfaces,
    proofDebtWorkQueue,
    surfaces,
    practicalTargets,
    warnings: [
      summary.missingProofSurfaceCount === 0 ? "" : `${summary.missingProofSurfaceCount} contract surfaces still lack proof artifacts`,
      summary.surfaceProofDebtCount === 0 ? "" : `${summary.surfaceProofDebtCount} contract surfaces are not yet strong-proof`,
      summary.visualTierDebtSurfaceCount === 0 ? "" : `${summary.visualTierDebtSurfaceCount} contract surfaces have explicit visual-tier debt; inspect proofTiers before claiming exhaustive Liquid Glass proof`,
      summary.guidelineFailedSurfaceCount === 0 ? "" : `${summary.guidelineFailedSurfaceCount} contract surfaces have failing Tahoe guideline assertions`,
      summary.guidelineMissingSurfaceCount === 0 ? "" : `${summary.guidelineMissingSurfaceCount} contract surfaces are missing Tahoe guideline assertions`,
      summary.appRenderFailedSurfaceCount === 0 ? "" : `${summary.appRenderFailedSurfaceCount} contract surfaces attempted app-render proof and failed`,
      summary.appRenderBlockedSurfaceCount === 0 ? "" : `${summary.appRenderBlockedSurfaceCount} contract surfaces attempted app-render proof but GPUI render readback was unavailable or unsupported`,
      "strong-guidance-proof means current artifacts include OS screenshot, numeric layout visualAudit, guideline assertions, and image diff evidence; GPUI readback is diagnostic only",
      "App-rendered GPUI pixels only; does not prove macOS WindowServer compositor output or native Liquid Glass blur",
      "proofTiers separate OS screenshots from GPUI app-render proof so WindowServer-blocked captures cannot become false visual evidence",
    ].filter(Boolean),
    errors: [],
  };

  await Bun.write(args.out, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
}

await main();
