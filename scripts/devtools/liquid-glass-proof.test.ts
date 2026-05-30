import { mkdtempSync, rmSync, mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { describe, expect, test } from "bun:test";

type RenderEvidence = Record<string, unknown> | null;

type ProofFixture = {
  surfaceKind?: string;
  term?: string;
  renderEvidence?: RenderEvidence;
  visualAudit?: Record<string, unknown>;
  nodes?: Array<Record<string, unknown>>;
  screenshotReceipt?: Record<string, unknown>;
  screenshotFile?: boolean;
  imageDiffReceipt?: Record<string, unknown>;
  extraReceiptName?: string;
  extraReceipt?: Record<string, unknown>;
};

function passingVisualAudit(overrides: Record<string, unknown> = {}) {
  return {
    nodeCount: 1,
    styledNodeCount: 1,
    controlsWithHitFailures: [],
    contentGlassNodes: [],
    contentNativeMaterialNodes: [],
    glassLayerViolations: [],
    missingStyleNodeNames: [],
    guidelineAssertions: { layering: { failures: [] } },
    ...overrides,
  };
}

function runProof(renderEvidence: RenderEvidence, fixture: Omit<ProofFixture, "renderEvidence"> = {}) {
  return runProofFixture({ ...fixture, renderEvidence });
}

function runProofFixture(fixture: ProofFixture) {
  const root = mkdtempSync(join(tmpdir(), "liquid-glass-proof-"));
  try {
    const artifactRoot = join(root, "artifacts", "liquid-glass");
    const receipts = join(artifactRoot, "receipts");
    const screenshots = join(artifactRoot, "screenshots");
    mkdirSync(receipts, { recursive: true });
    mkdirSync(screenshots, { recursive: true });
    mkdirSync(join(artifactRoot, "diffs"), { recursive: true });

    const inventory = join(root, "inventory.json");
    const out = join(root, "matrix.json");
    const surfaceKind = fixture.surfaceKind ?? "ScriptList";
    const term = fixture.term ?? (surfaceKind === "ScriptList" ? "main" : surfaceKind.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase());
    writeFileSync(inventory, JSON.stringify({
      auditSurfaceContracts: [{ surfaceKind }],
      recommendedOracleBatches: [],
    }));
    writeFileSync(join(receipts, `fixture-${term}-render.json`), JSON.stringify(
      fixture.renderEvidence ? { renderEvidence: fixture.renderEvidence } : {},
    ));
    if (fixture.visualAudit) {
      writeFileSync(join(receipts, `fixture-${term}-layout.json`), JSON.stringify({
        visualAudit: fixture.visualAudit,
        nodes: fixture.nodes ?? [],
      }));
    }
    if (fixture.screenshotReceipt) {
      writeFileSync(join(receipts, `fixture-${term}-screenshot.json`), JSON.stringify(fixture.screenshotReceipt));
    }
    if (fixture.screenshotFile) {
      writeFileSync(join(screenshots, `fixture-${term}.png`), "");
    }
    if (fixture.imageDiffReceipt) {
      writeFileSync(join(receipts, `image-diff-${term}.json`), JSON.stringify(fixture.imageDiffReceipt));
    }
    if (fixture.extraReceiptName && fixture.extraReceipt) {
      writeFileSync(join(receipts, fixture.extraReceiptName), JSON.stringify(fixture.extraReceipt));
    }

    const proc = Bun.spawnSync([
      "bun",
      "scripts/devtools/liquid-glass-proof.ts",
      "--inventory",
      inventory,
      "--artifact-root",
      artifactRoot,
      "--out",
      out,
    ], {
      stdout: "pipe",
      stderr: "pipe",
    });
    expect(proc.exitCode, new TextDecoder().decode(proc.stderr)).toBe(0);
    return JSON.parse(readFileSync(out, "utf8"));
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

describe("liquid-glass-proof app render tier", () => {
  test.each([
    ["gpui_readback_unavailable", { status: "unsupported", errorCode: "gpui_readback_unavailable" }],
    ["unknown_tool", { status: "unsupported", errorCode: "unknown_tool" }],
    ["runtime_unavailable", { status: "unsupported", errorCode: "runtime_unavailable" }],
    ["unsupported_platform", { status: "unsupported", errorCode: "unsupported_platform" }],
    ["unsupported_platform_reason", { status: "failed", reason: "unsupported_platform" }],
  ])("classifies unsupported readback as blocked: %s", (_name, attempt) => {
    const matrix = runProof({
      source: "gpui-render-readback",
      available: false,
      countsAsOsScreenshotEvidence: false,
      countsAsAppRenderEvidence: false,
      classification: "gpui-readback-unavailable",
      attempts: [attempt],
      path: null,
    });

    expect(matrix.surfaces[0].proofTiers.appRenderProof).toBe("blocked");
    expect(matrix.summary.appRenderBlockedSurfaceCount).toBe(1);
    expect(matrix.summary.appRenderFailedSurfaceCount).toBe(0);
    expect(matrix.summary.visualTierDebtSurfaceCount).toBe(0);
  });

  test("keeps captured readback as pass", () => {
    const matrix = runProof({
      source: "gpui-render-readback",
      available: true,
      countsAsOsScreenshotEvidence: false,
      countsAsAppRenderEvidence: true,
      classification: "captured",
      attempts: [{ status: "captured" }],
      pixelAudit: { blank: false },
    });

    expect(matrix.surfaces[0].proofTiers.appRenderProof).toBe("pass");
    expect(matrix.summary.appRenderProofSurfaceCount).toBe(1);
    expect(matrix.summary.appRenderBlockedSurfaceCount).toBe(0);
    expect(matrix.summary.appRenderFailedSurfaceCount).toBe(0);
  });

  test("keeps real non-unavailable readback errors as fail", () => {
    const matrix = runProof({
      source: "gpui-render-readback",
      available: false,
      countsAsOsScreenshotEvidence: false,
      countsAsAppRenderEvidence: false,
      classification: "blank-image-rejected",
      attempts: [{ status: "failed", errorCode: "capture_failed" }],
    });

    expect(matrix.surfaces[0].proofTiers.appRenderProof).toBe("fail");
    expect(matrix.summary.appRenderFailedSurfaceCount).toBe(1);
    expect(matrix.summary.appRenderBlockedSurfaceCount).toBe(0);
  });

  test("keeps absent readback evidence as missing", () => {
    const matrix = runProof(null);

    expect(matrix.surfaces[0].proofTiers.appRenderProof).toBe("missing");
    expect(matrix.summary.appRenderMissingSurfaceCount).toBe(1);
    expect(matrix.summary.appRenderBlockedSurfaceCount).toBe(0);
    expect(matrix.summary.appRenderFailedSurfaceCount).toBe(0);
  });
});

describe("liquid-glass-proof guidance domain split", () => {
  const blockedReadback = {
    source: "gpui-render-readback",
    available: false,
    countsAsOsScreenshotEvidence: false,
    countsAsAppRenderEvidence: false,
    classification: "gpui-readback-unavailable",
    attempts: [{ status: "unsupported", errorCode: "gpui_readback_unavailable" }],
  };
  const okImageDiff = {
    classification: "ok",
    assertions: { diffMaskWritten: true, changedPixelsMeasured: true },
    errors: [],
    sameSizeRequired: true,
    dimensions: { sameSize: true },
  };

  test("classifies OS screenshot receipt errors as capture limitations", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      visualAudit: passingVisualAudit(),
      screenshotReceipt: {
        status: "error",
        screenshotReceipt: { captured: false, error: "could not create image from window" },
      },
    });
    const surface = matrix.surfaces[0];
    const queueEntry = matrix.proofDebtWorkQueue[0];

    expect(surface.proofTiers.osScreenshotProof).toBe("blocked");
    expect(surface.guidanceProofStatus).toBe("guidance-proof-capture-blocked");
    expect(surface.sourceUiGaps).toEqual([]);
    expect(surface.devtoolsCaptureLimitations).toContain("osScreenshotProof:blocked");
    expect(surface.osScreenshotBlockers[0].classification).toBe("screenshot-receipt-error");
    expect(surface.osCapture.blockerCode).toBe("window-id-api-blocked");
    expect(matrix.summary.osScreenshotBlockerCounts["screenshot-receipt-error"]).toBe(1);
    expect(queueEntry.blockingClass).toBe("window-id-api-blocked");
  });

  test("preserves exact WindowServer capture blocker taxonomy", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      visualAudit: passingVisualAudit(),
      screenshotReceipt: {
        visualEvidence: {
          source: "os-window-capture",
          available: false,
          countsAsOsScreenshotEvidence: false,
          classification: "macos-windowserver-capture-blocked",
          limitation: "screencapture could not create an image from the target window or rect",
          attempts: [
            { method: "computer/capture_native_window", status: "failed", errorCode: "capture_failed", message: "native MCP unavailable" },
            { method: "screencapture-window-id", status: "failed", errorCode: "screencapture_window_failed", stderr: "could not create image from window" },
            { method: "screencapture-screen-rect", status: "failed", errorCode: "screencapture_rect_failed", stderr: "could not create image from rect" },
          ],
        },
      },
    });
    const surface = matrix.surfaces[0];
    const queueEntry = matrix.proofDebtWorkQueue[0];

    expect(surface.proofTiers.osScreenshotProof).toBe("blocked");
    expect(surface.proofTiers.imageDiffProof).toBe("blocked");
    expect(surface.sourceUiGaps).toEqual([]);
    expect(surface.osScreenshotBlockers[0].classification).toBe("macos-windowserver-capture-blocked");
    expect(surface.osScreenshotBlockers[0].attempts.map((attempt: Record<string, unknown>) => attempt.errorCode)).toContain("screencapture_window_failed");
    expect(surface.osCapture.blockerCode).toBe("screen-rect-capture-blocked");
    expect(queueEntry.osScreenshotBlockers[0].classification).toBe("macos-windowserver-capture-blocked");
    expect(queueEntry.blockingClass).toBe("screen-rect-capture-blocked");
    expect(queueEntry.recommendedNextAction).toContain("screen-rect-capture-blocked");
    expect(matrix.summary.osScreenshotBlockerCounts["macos-windowserver-capture-blocked"]).toBe(1);
  });

  test("normalizes legacy strict capture error logs into WindowServer blockers", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      visualAudit: passingVisualAudit(),
      screenshotReceipt: {
        status: "error",
        screenshotReceipt: {
          captured: false,
          error: [
            "Strict window capture failed: {\"event\":\"window_capture_attempt\",\"windowId\":84776}",
            "{\"event\":\"window_capture_screencapture_l_failed\",\"windowId\":84776,\"stderr\":\"could not create image from window\"}",
            "native=computer/capture_native_window did not return image data: status=notCaptureCandidate error=not_capture_candidate; screenRect=could not create image from rect",
          ].join("\n"),
        },
      },
    });
    const blocker = matrix.surfaces[0].osScreenshotBlockers[0];

    expect(blocker.classification).toBe("macos-windowserver-capture-blocked");
    expect(blocker.attempts.map((attempt: Record<string, unknown>) => attempt.errorCode)).toEqual([
      "screencapture_window_failed",
      "screencapture_rect_failed",
      "not_capture_candidate",
    ]);
    expect(matrix.surfaces[0].osCapture.blockerCode).toBe("screen-rect-capture-blocked");
    expect(matrix.proofDebtWorkQueue[0].blockingClass).toBe("screen-rect-capture-blocked");
    expect(matrix.summary.osScreenshotBlockerCounts["macos-windowserver-capture-blocked"]).toBe(1);
  });

  test("does not let diagnostic app-render blockers poison strong guidance proof", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      renderEvidence: blockedReadback,
      visualAudit: passingVisualAudit(),
      screenshotFile: true,
      screenshotReceipt: { status: "pass", screenshotReceipt: { captured: true } },
      imageDiffReceipt: okImageDiff,
    });
    const surface = matrix.surfaces[0];

    expect(surface.proofStatus).toBe("strong-proof");
    expect(surface.guidanceProofStatus).toBe("strong-guidance-proof");
    expect(surface.diagnosticLimitations).toContain("appRenderProof:blocked");
    expect(matrix.proofDebtWorkQueue).toHaveLength(0);
  });

  test("classifies numeric visual audit failures as source UI gaps", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      visualAudit: passingVisualAudit({ contentNativeMaterialNodes: ["ContentArea"] }),
    });
    const surface = matrix.surfaces[0];
    const queueEntry = matrix.proofDebtWorkQueue[0];

    expect(surface.guidanceProofStatus).toBe("source-ui-gap");
    expect(surface.sourceUiGaps).toContain("contentNativeMaterialNodes");
    expect(queueEntry.blockingClass).toBe("source-ui-gap");
  });

  test("classifies legacy zero-radius layout receipts as stale evidence, not source gaps", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      visualAudit: passingVisualAudit(),
      nodes: [
        {
          name: "ContentArea",
          visualStyle: {
            chromeLayer: "content",
            cornerRadius: {
              topLeft: 0,
              topRight: 0,
              bottomRight: 0,
              bottomLeft: 0,
            },
          },
        },
      ],
    });
    const surface = matrix.surfaces[0];
    const queueEntry = matrix.proofDebtWorkQueue[0];

    expect(surface.proofTiers.guidelineProof).toBe("pass");
    expect(surface.guidanceProofStatus).toBe("stale-layout-evidence");
    expect(surface.sourceUiGaps).toEqual([]);
    expect(queueEntry.guidelineFailures).toEqual([]);
    expect(surface.layoutReceiptFreshnessLimitations[0]).toContain("ContentArea");
    expect(queueEntry.blockingClass).toBe("stale-layout-evidence");
  });

  test("does not require radii on non-panel styled nodes", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      visualAudit: passingVisualAudit(),
      nodes: [
        {
          name: "AboutTitle",
          type: "other",
          visualStyle: {
            chromeLayer: "content",
            tokenSource: "about.titleVersion",
          },
        },
        {
          name: "ContentArea",
          visualStyle: {
            chromeLayer: "content",
            cornerRadius: {
              topLeft: 16,
              topRight: 16,
              bottomRight: 16,
              bottomLeft: 16,
            },
          },
        },
      ],
    });
    const surface = matrix.surfaces[0];

    expect(surface.proofTiers.guidelineProof).toBe("pass");
    expect(surface.sourceUiGaps).toEqual([]);
    expect(matrix.proofDebtWorkQueue[0]?.blockingClass).not.toBe("source-ui-gap");
  });

  test("does not treat empty timeout layout receipts as numeric proof", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      visualAudit: passingVisualAudit({
        nodeCount: 0,
        styledNodeCount: 0,
        guidelineAssertions: { layering: { failures: [] } },
      }),
      nodes: [],
    });
    const surface = matrix.surfaces[0];

    expect(surface.proofTiers.numericProof).toBe("fail");
    expect(surface.proofStatus).not.toBe("numeric-proof-missing-visual-capture");
    expect(surface.proofStatus).not.toBe("strong-proof");
  });

  test("keeps absent screenshot attempts as missing guidance visual evidence", () => {
    const matrix = runProofFixture({
      surfaceKind: "FixtureSurface",
      visualAudit: passingVisualAudit(),
    });
    const surface = matrix.surfaces[0];
    const queueEntry = matrix.proofDebtWorkQueue[0];

    expect(surface.proofTiers.osScreenshotProof).toBe("missing");
    expect(surface.guidanceProofStatus).toBe("numeric-guidance-proof-missing-os-visual");
    expect(queueEntry.blockingClass).toBe("missing-guidance-visual-evidence");
  });

  test("counts explicit OS visualEvidence blockers even when receipt filename is not capture-named", () => {
    const matrix = runProofFixture({
      surfaceKind: "PromptEntity",
      visualAudit: passingVisualAudit(),
      extraReceiptName: "fixture-prompt-entity-proof.json",
      extraReceipt: {
        visualEvidence: {
          source: "os-window-capture",
          available: false,
          countsAsOsScreenshotEvidence: false,
          countsAsCompositorEvidence: false,
          classification: "macos-windowserver-capture-blocked",
          blockerCode: "screen-rect-capture-blocked",
          attempts: [
            { status: "failed", errorCode: "screencapture_rect_failed" },
          ],
        },
      },
    });
    const surface = matrix.surfaces[0];

    expect(surface.proofTiers.osScreenshotProof).toBe("blocked");
    expect(surface.osCapture.blockerCode).toBe("screen-rect-capture-blocked");
    expect(surface.osCapture.assertions.screenRectCaptureAttempted).toBe(true);
  });
});

describe("liquid-glass-proof PromptEntity render tier", () => {
  test("classifies unavailable PromptEntity app render as diagnostic readback limitation", () => {
    const matrix = runProofFixture({
      surfaceKind: "PromptEntity",
      visualAudit: passingVisualAudit(),
      renderEvidence: {
        source: "gpui-render-readback",
        available: false,
        countsAsOsScreenshotEvidence: false,
        countsAsAppRenderEvidence: false,
        countsAsOffscreenRenderEvidence: false,
        classification: "gpui-readback-unavailable",
        attempts: [{ status: "unsupported", errorCode: "gpui_readback_unavailable" }],
      },
    });
    const surface = matrix.surfaces[0];

    expect(surface.surfaceKind).toBe("PromptEntity");
    expect(surface.proofTiers.numericProof).toBe("pass");
    expect(surface.proofTiers.appRenderProof).toBe("blocked");
    expect(surface.proofTiers.osScreenshotProof).toBe("missing");
    expect(surface.proofStatus).toBe("numeric-proof-app-render-blocked");
    expect(surface.diagnosticLimitations).toContain("appRenderProof:blocked");
    expect(surface.guidanceProofStatus).toBe("numeric-guidance-proof-missing-os-visual");
  });

  test("keeps captured PromptEntity app render separate from OS screenshot proof", () => {
    const matrix = runProofFixture({
      surfaceKind: "PromptEntity",
      visualAudit: passingVisualAudit(),
      renderEvidence: {
        source: "gpui-render-readback",
        available: true,
        countsAsOsScreenshotEvidence: false,
        countsAsAppRenderEvidence: true,
        countsAsOffscreenRenderEvidence: false,
        classification: "captured",
        attempts: [{ status: "captured" }],
        pixelAudit: { blank: false },
        path: "artifacts/liquid-glass/screenshots/window-priority-prompt-div-fixed-render.png",
      },
    });
    const surface = matrix.surfaces[0];

    expect(surface.proofTiers.appRenderProof).toBe("pass");
    expect(surface.proofTiers.osScreenshotProof).not.toBe("pass");
    expect(surface.proofStatus).toBe("numeric-plus-app-render-proof-missing-os-screenshot");
    expect(surface.guidanceProofStatus).toBe("numeric-guidance-proof-missing-os-visual");
  });
});
