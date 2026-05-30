import { mkdtempSync, rmSync, writeFileSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { describe, expect, test } from "bun:test";

function runImageDiffWithReceipts(redReceipt: Record<string, unknown>, greenReceipt: Record<string, unknown>) {
  const root = mkdtempSync(join(tmpdir(), "image-diff-os-evidence-"));
  try {
    const redReceiptPath = join(root, "red.json");
    const greenReceiptPath = join(root, "green.json");
    const outPath = join(root, "diff.png");
    const receiptPath = join(root, "receipt.json");
    writeFileSync(redReceiptPath, JSON.stringify(redReceipt));
    writeFileSync(greenReceiptPath, JSON.stringify(greenReceipt));

    const proc = Bun.spawnSync([
      "bun",
      "scripts/devtools/image-diff.ts",
      "compare",
      "--red",
      join(root, "red.png"),
      "--green",
      join(root, "green.png"),
      "--out",
      outPath,
      "--red-receipt",
      redReceiptPath,
      "--green-receipt",
      greenReceiptPath,
      "--require-os-evidence",
    ], {
      stdout: "pipe",
      stderr: "pipe",
    });
    writeFileSync(receiptPath, new TextDecoder().decode(proc.stdout));
    return {
      exitCode: proc.exitCode,
      receipt: JSON.parse(readFileSync(receiptPath, "utf8")),
    };
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function capturedOsReceipt() {
  return {
    visualEvidence: {
      source: "os-window-capture",
      classification: "captured",
      captureKind: "screen-rect",
      countsAsOsScreenshotEvidence: true,
      countsAsCompositorEvidence: true,
      pixelAudit: { blank: false },
    },
  };
}

describe("image-diff OS evidence gate", () => {
  test("blocks when red input lacks OS compositor evidence", () => {
    const result = runImageDiffWithReceipts({}, capturedOsReceipt());

    expect(result.exitCode).toBe(2);
    expect(result.receipt.classification).toBe("blocked");
    expect(result.receipt.blockerCode).toBe("red-os-evidence-missing");
    expect(result.receipt.assertions.redCountsAsOsScreenshotEvidence).toBe(false);
    expect(result.receipt.assertions.greenCountsAsOsScreenshotEvidence).toBe(true);
  });

  test("blocks when green input lacks OS compositor evidence", () => {
    const result = runImageDiffWithReceipts(capturedOsReceipt(), {});

    expect(result.exitCode).toBe(2);
    expect(result.receipt.classification).toBe("blocked");
    expect(result.receipt.blockerCode).toBe("green-os-evidence-missing");
    expect(result.receipt.assertions.redCountsAsOsScreenshotEvidence).toBe(true);
    expect(result.receipt.assertions.greenCountsAsOsScreenshotEvidence).toBe(false);
  });
});
