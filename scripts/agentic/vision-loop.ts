#!/usr/bin/env bun
/**
 * scripts/agentic/vision-loop.ts
 *
 * Reads an ACP proof receipt and materializes one crop file per
 * `proofBundle.visionCrops` entry, plus a compact manifest JSON.
 *
 * Usage:
 *   bun scripts/agentic/vision-loop.ts --receipt <path> --out-dir <dir>
 *
 * Exits nonzero when:
 *   - The receipt has no visionCrops
 *   - Crop materialization fails
 *   - The output directory cannot be prepared
 *
 * Output (stdout): The manifest JSON.
 * Logs (stderr): Structured JSON events.
 */

import { resolve, relative, join, basename, extname } from "path";
import { existsSync, mkdirSync, copyFileSync } from "fs";

const SCHEMA_VERSION = 1;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

interface VisionCrop {
  name: string;
  path: string;
  question: string;
  crop: { x: number; y: number; width: number; height: number } | null;
  expectedAnswer?: string | null;
  mustReview: boolean;
  failureMessage: string;
}

interface ManifestCrop {
  name: string;
  path: string;
  question: string;
  expectedAnswer: string | null;
  mustReview: boolean;
}

interface Manifest {
  schemaVersion: number;
  status: "ok" | "error";
  crops: ManifestCrop[];
  error?: string;
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function parseArgs(): { receiptPath: string; outDir: string } {
  const args = process.argv.slice(2);

  if (args.includes("--help") || args.includes("-h")) {
    console.log(`Usage: bun scripts/agentic/vision-loop.ts --receipt <path> --out-dir <dir>

Reads an ACP proof receipt and writes a manifest JSON plus one crop file
per visionCrops entry.

Options:
  --receipt <path>   Path to the proof receipt JSON file
  --out-dir <dir>    Directory to write crop files and manifest into
  --help             Show this help`);
    process.exit(0);
  }

  const receiptIdx = args.indexOf("--receipt");
  const outDirIdx = args.indexOf("--out-dir");

  if (receiptIdx < 0 || !args[receiptIdx + 1]) {
    console.error(
      JSON.stringify({ event: "error", message: "Missing required --receipt <path>" })
    );
    process.exit(1);
  }

  if (outDirIdx < 0 || !args[outDirIdx + 1]) {
    console.error(
      JSON.stringify({ event: "error", message: "Missing required --out-dir <dir>" })
    );
    process.exit(1);
  }

  return {
    receiptPath: resolve(args[receiptIdx + 1]),
    outDir: resolve(args[outDirIdx + 1]),
  };
}

function loadReceipt(path: string): Record<string, unknown> {
  if (!existsSync(path)) {
    console.error(
      JSON.stringify({ event: "error", message: `Receipt file not found: ${path}` })
    );
    process.exit(1);
  }

  try {
    const text = require("fs").readFileSync(path, "utf-8");
    return JSON.parse(text);
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error(
      JSON.stringify({ event: "error", message: `Failed to parse receipt: ${msg}` })
    );
    process.exit(1);
  }
}

/**
 * Extract visionCrops from a receipt. Supports both top-level visionCrops
 * and nested proofBundle.visionCrops.
 */
function extractVisionCrops(receipt: Record<string, unknown>): VisionCrop[] {
  // Direct top-level visionCrops (verify-shot.ts output)
  if (Array.isArray(receipt.visionCrops) && receipt.visionCrops.length > 0) {
    return receipt.visionCrops as VisionCrop[];
  }

  // Nested in proofBundle (orchestrator output)
  const proofBundle = receipt.proofBundle as Record<string, unknown> | undefined;
  if (proofBundle && Array.isArray(proofBundle.visionCrops) && proofBundle.visionCrops.length > 0) {
    return proofBundle.visionCrops as VisionCrop[];
  }

  return [];
}

function prepareOutDir(outDir: string): void {
  try {
    mkdirSync(outDir, { recursive: true });
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    console.error(
      JSON.stringify({ event: "error", message: `Failed to create output directory: ${msg}` })
    );
    process.exit(1);
  }
}

/**
 * Sanitize a crop name for use as a filename.
 */
function sanitizeName(name: string): string {
  return name.replace(/[^a-zA-Z0-9_-]/g, "-").replace(/-+/g, "-").replace(/^-|-$/g, "");
}

/**
 * Materialize a single crop file into outDir.
 * Returns the relative path from outDir.
 */
function materializeCrop(crop: VisionCrop, outDir: string): string {
  const sourcePath = resolve(crop.path);
  if (!existsSync(sourcePath)) {
    throw new Error(`Source crop file not found: ${crop.path}`);
  }

  const ext = extname(sourcePath) || ".png";
  const safeName = sanitizeName(crop.name) || `crop-${Date.now()}`;
  const destName = `${safeName}${ext}`;
  const destPath = join(outDir, destName);

  copyFileSync(sourcePath, destPath);

  return destName;
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

const { receiptPath, outDir } = parseArgs();

console.error(
  JSON.stringify({
    event: "vision_loop_start",
    receipt: receiptPath,
    outDir,
  })
);

const receipt = loadReceipt(receiptPath);
const visionCrops = extractVisionCrops(receipt);

if (visionCrops.length === 0) {
  const manifest: Manifest = {
    schemaVersion: SCHEMA_VERSION,
    status: "error",
    crops: [],
    error: "Receipt contains no visionCrops entries",
  };
  console.log(JSON.stringify(manifest, null, 2));
  console.error(
    JSON.stringify({ event: "vision_loop_no_crops", receipt: receiptPath })
  );
  process.exit(1);
}

prepareOutDir(outDir);

const manifestCrops: ManifestCrop[] = [];
const errors: string[] = [];

for (const crop of visionCrops) {
  try {
    const relativePath = materializeCrop(crop, outDir);
    manifestCrops.push({
      name: crop.name,
      path: relativePath,
      question: crop.question,
      expectedAnswer: crop.expectedAnswer ?? null,
      mustReview: crop.mustReview,
    });
    console.error(
      JSON.stringify({
        event: "vision_crop_materialized",
        name: crop.name,
        dest: relativePath,
      })
    );
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    errors.push(`${crop.name}: ${msg}`);
    console.error(
      JSON.stringify({
        event: "vision_crop_failed",
        name: crop.name,
        error: msg,
      })
    );
  }
}

if (errors.length > 0 && manifestCrops.length === 0) {
  // All crops failed
  const manifest: Manifest = {
    schemaVersion: SCHEMA_VERSION,
    status: "error",
    crops: [],
    error: `All crop materializations failed: ${errors.join("; ")}`,
  };
  console.log(JSON.stringify(manifest, null, 2));
  process.exit(1);
}

const manifest: Manifest = {
  schemaVersion: SCHEMA_VERSION,
  status: errors.length > 0 ? "error" : "ok",
  crops: manifestCrops,
  ...(errors.length > 0 ? { error: `Partial failures: ${errors.join("; ")}` } : {}),
};

// Write manifest file to outDir
const manifestPath = join(outDir, "manifest.json");
require("fs").writeFileSync(manifestPath, JSON.stringify(manifest, null, 2));

console.error(
  JSON.stringify({
    event: "vision_loop_complete",
    totalCrops: visionCrops.length,
    materialized: manifestCrops.length,
    failed: errors.length,
    manifestPath,
  })
);

// Stdout: the manifest for piping
console.log(JSON.stringify(manifest, null, 2));

process.exit(errors.length > 0 ? 1 : 0);
