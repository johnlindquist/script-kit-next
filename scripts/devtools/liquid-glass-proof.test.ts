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
  screenshotReceipt?: Record<string, unknown>;
  screenshotFile?: boolean;
  imageDiffReceipt?: Record<string, unknown>;
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
    expect(queueEntry.blockingClass).toBe("devtools-capture-limitation");
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
});
