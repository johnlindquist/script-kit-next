import { mkdtempSync, rmSync, writeFileSync, readFileSync } from "node:fs";
import { createHash } from "node:crypto";
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
      "--receipt-out",
      receiptPath,
      "--red-receipt",
      redReceiptPath,
      "--green-receipt",
      greenReceiptPath,
      "--require-os-evidence",
    ], {
      stdout: "pipe",
      stderr: "pipe",
    });
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

function sha256(path: string) {
  return createHash("sha256").update(readFileSync(path)).digest("hex");
}

function runImageDiffHashGate(expectedRed: string | null, expectedGreen: string | null) {
  const root = mkdtempSync(join(tmpdir(), "image-diff-hash-evidence-"));
  try {
    const redPath = join(root, "red.png");
    const greenPath = join(root, "green.png");
    const png = Bun.spawnSync(["magick", "-size", "2x2", "xc:#123456", "png:-"], {
      stdout: "pipe",
      stderr: "pipe",
    });
    expect(png.exitCode).toBe(0);
    writeFileSync(redPath, png.stdout);
    writeFileSync(greenPath, png.stdout);
    const redReceiptPath = join(root, "red.json");
    const greenReceiptPath = join(root, "green.json");
    writeFileSync(redReceiptPath, JSON.stringify({ screenshotEvidence: { sha256: expectedRed === "actual" ? sha256(redPath) : expectedRed } }));
    writeFileSync(greenReceiptPath, JSON.stringify({ screenshotEvidence: { sha256: expectedGreen === "actual" ? sha256(greenPath) : expectedGreen } }));
    const receiptPath = join(root, "receipt.json");
    const proc = Bun.spawnSync([
      "bun", "scripts/devtools/image-diff.ts", "compare",
      "--red", redPath, "--green", greenPath, "--out", join(root, "diff.png"),
      "--receipt-out", receiptPath,
      "--red-receipt", redReceiptPath, "--green-receipt", greenReceiptPath,
      "--require-input-hashes",
    ], { stdout: "pipe", stderr: "pipe" });
    return {
      exitCode: proc.exitCode,
      actualHash: sha256(redPath),
      receipt: JSON.parse(readFileSync(receiptPath, "utf8")),
    };
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}

function runImageDiffBoundingBox() {
  const root = mkdtempSync(join(tmpdir(), "image-diff-bounds-"));
  try {
    const redPath = join(root, "red.png");
    const greenPath = join(root, "green.png");
    const receiptPath = join(root, "receipt.json");
    const red = Bun.spawnSync([
      "magick", "-size", "20x20", "xc:#010203", redPath,
    ], { stdout: "pipe", stderr: "pipe" });
    const green = Bun.spawnSync([
      "magick", "-size", "20x20", "xc:#010203",
      "-fill", "#fefdfc", "-draw", "rectangle 7,9 9,12", greenPath,
    ], { stdout: "pipe", stderr: "pipe" });
    expect(red.exitCode).toBe(0);
    expect(green.exitCode).toBe(0);

    const proc = Bun.spawnSync([
      "bun", "scripts/devtools/image-diff.ts", "compare",
      "--red", redPath, "--green", greenPath, "--out", join(root, "diff.png"),
      "--receipt-out", receiptPath, "--fuzz", "0%",
    ], { stdout: "pipe", stderr: "pipe" });
    return {
      exitCode: proc.exitCode,
      receipt: JSON.parse(readFileSync(receiptPath, "utf8")),
    };
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
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

describe("image-diff capture hash gate", () => {
  test("blocks when a receipt hash is stale", () => {
    const result = runImageDiffHashGate("0".repeat(64), "0".repeat(64));
    expect(result.exitCode).toBe(2);
    expect(result.receipt.blockerCode).toBe("red-receipt-hash-mismatch");
    expect(result.receipt.inputEvidence.inputHashes.red.sha256).toBe(result.actualHash);
  });

  test("accepts inputs that match both capture receipts", () => {
    const result = runImageDiffHashGate("actual", "actual");
    expect(result.exitCode).toBe(0);
    expect(result.receipt.classification).toBe("ok");
    expect(result.receipt.inputHashes.red.matchesReceipt).toBe(true);
    expect(result.receipt.inputHashes.green.matchesReceipt).toBe(true);
  });
});

describe("image-diff changed-pixel bounds", () => {
  test("preserves an off-origin changed region in source-image coordinates", () => {
    const result = runImageDiffBoundingBox();

    expect(result.exitCode).toBe(0);
    expect(result.receipt.changedPixels).toBe(12);
    expect(result.receipt.diffBoundingBox).toEqual({
      width: 3,
      height: 4,
      x: 7,
      y: 9,
    });
  });
});
