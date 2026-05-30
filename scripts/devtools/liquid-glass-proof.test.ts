import { mkdtempSync, rmSync, mkdirSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { describe, expect, test } from "bun:test";

type RenderEvidence = Record<string, unknown> | null;

function runProof(renderEvidence: RenderEvidence) {
  const root = mkdtempSync(join(tmpdir(), "liquid-glass-proof-"));
  try {
    const artifactRoot = join(root, "artifacts", "liquid-glass");
    const receipts = join(artifactRoot, "receipts");
    mkdirSync(receipts, { recursive: true });
    mkdirSync(join(artifactRoot, "screenshots"), { recursive: true });
    mkdirSync(join(artifactRoot, "diffs"), { recursive: true });

    const inventory = join(root, "inventory.json");
    const out = join(root, "matrix.json");
    writeFileSync(inventory, JSON.stringify({
      auditSurfaceContracts: [{ surfaceKind: "ScriptList" }],
      recommendedOracleBatches: [],
    }));
    writeFileSync(join(receipts, "fixture-main-render.json"), JSON.stringify(
      renderEvidence ? { renderEvidence } : {},
    ));

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
    expect(matrix.summary.visualTierDebtSurfaceCount).toBe(1);
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
