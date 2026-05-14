#!/usr/bin/env bun
/**
 * Deterministic PNG fixture matrix for the verify-shot pixel audit.
 */

import { mkdirSync, readFileSync, writeFileSync } from "fs";
import { resolve } from "path";
import { deflateSync, inflateSync } from "node:zlib";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const OUT_DIR = resolve(PROJECT_ROOT, ".test-output/verify-shot-blank-rejection-matrix");
const PNG_SIGNATURE = Buffer.from([0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a]);

interface ScreenshotContentAudit {
  sampledPixels: number;
  nonBlackPixels: number;
  nonTransparentPixels: number;
  uniqueBucketCount: number;
  meanLuma: number;
  maxLuma: number;
  nonBlackRatio: number;
  blank: boolean;
}

interface FixtureCase {
  name: string;
  width: number;
  height: number;
  pixelAt: (x: number, y: number) => [number, number, number, number];
  expectedBlank: boolean;
}

const CRC_TABLE = new Uint32Array(256).map((_, n) => {
  let c = n;
  for (let k = 0; k < 8; k += 1) {
    c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  }
  return c >>> 0;
});

function crc32(buf: Buffer): number {
  let c = 0xffffffff;
  for (const byte of buf) {
    c = CRC_TABLE[(c ^ byte) & 0xff] ^ (c >>> 8);
  }
  return (c ^ 0xffffffff) >>> 0;
}

function chunk(type: string, data: Buffer): Buffer {
  const typeBuf = Buffer.from(type, "ascii");
  const length = Buffer.alloc(4);
  length.writeUInt32BE(data.length, 0);
  const crc = Buffer.alloc(4);
  crc.writeUInt32BE(crc32(Buffer.concat([typeBuf, data])), 0);
  return Buffer.concat([length, typeBuf, data, crc]);
}

function writeRgbaPng(path: string, fixture: FixtureCase): void {
  const ihdr = Buffer.alloc(13);
  ihdr.writeUInt32BE(fixture.width, 0);
  ihdr.writeUInt32BE(fixture.height, 4);
  ihdr[8] = 8;
  ihdr[9] = 6;
  ihdr[10] = 0;
  ihdr[11] = 0;
  ihdr[12] = 0;

  const rowBytes = fixture.width * 4;
  const raw = Buffer.alloc((rowBytes + 1) * fixture.height);
  let offset = 0;
  for (let y = 0; y < fixture.height; y += 1) {
    raw[offset++] = 0;
    for (let x = 0; x < fixture.width; x += 1) {
      const [r, g, b, a] = fixture.pixelAt(x, y);
      raw[offset++] = r;
      raw[offset++] = g;
      raw[offset++] = b;
      raw[offset++] = a;
    }
  }

  writeFileSync(
    path,
    Buffer.concat([
      PNG_SIGNATURE,
      chunk("IHDR", ihdr),
      chunk("IDAT", deflateSync(raw)),
      chunk("IEND", Buffer.alloc(0)),
    ]),
  );
}

function readPngContent(filePath: string): {
  width: number;
  height: number;
  colorType: number;
  data: Buffer;
} {
  const bytes = readFileSync(filePath);
  if (!bytes.subarray(0, 8).equals(PNG_SIGNATURE)) {
    throw new Error(`${filePath} is not a PNG`);
  }
  let offset = 8;
  let width = 0;
  let height = 0;
  let colorType = 0;
  const idat: Buffer[] = [];
  while (offset < bytes.length) {
    const length = bytes.readUInt32BE(offset);
    const type = bytes.subarray(offset + 4, offset + 8).toString("ascii");
    const data = bytes.subarray(offset + 8, offset + 8 + length);
    offset += 12 + length;
    if (type === "IHDR") {
      width = data.readUInt32BE(0);
      height = data.readUInt32BE(4);
      colorType = data[9];
      if (data[8] !== 8 || colorType !== 6) {
        throw new Error(`${filePath} must be 8-bit RGBA for this matrix`);
      }
    } else if (type === "IDAT") {
      idat.push(Buffer.from(data));
    } else if (type === "IEND") {
      break;
    }
  }
  return { width, height, colorType, data: inflateSync(Buffer.concat(idat)) };
}

function auditPngContent(filePath: string): ScreenshotContentAudit {
  const { width, height, data } = readPngContent(filePath);
  const bytesPerPixel = 4;
  const rowBytes = width * bytesPerPixel;
  let readOffset = 0;
  let sampledPixels = 0;
  let nonBlackPixels = 0;
  let nonTransparentPixels = 0;
  let lumaSum = 0;
  let maxLuma = 0;
  const buckets = new Set<string>();

  for (let y = 0; y < height; y += 1) {
    const filter = data[readOffset++];
    if (filter !== 0) {
      throw new Error(`${filePath} used unsupported fixture filter ${filter}`);
    }
    for (let x = 0; x < rowBytes; x += bytesPerPixel) {
      const r = data[readOffset++];
      const g = data[readOffset++];
      const b = data[readOffset++];
      const a = data[readOffset++];
      sampledPixels += 1;
      if (a > 0) nonTransparentPixels += 1;
      if (a > 0 && (r > 8 || g > 8 || b > 8)) nonBlackPixels += 1;
      const luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
      lumaSum += luma;
      maxLuma = Math.max(maxLuma, luma);
      buckets.add(`${r >> 5}:${g >> 5}:${b >> 5}:${a === 0 ? 0 : 1}`);
    }
  }

  const meanLuma = sampledPixels > 0 ? lumaSum / sampledPixels : 0;
  const uniqueBucketCount = buckets.size;
  const nonBlackRatio =
    sampledPixels > 0 ? nonBlackPixels / sampledPixels : 0;
  const solidLike = uniqueBucketCount <= 1;
  const darkEmptyLike =
    uniqueBucketCount <= 2 &&
    meanLuma < 5.0 &&
    nonBlackRatio < 0.001 &&
    maxLuma < 16.0;
  const blank =
    sampledPixels === 0 ||
    nonTransparentPixels === 0 ||
    solidLike ||
    darkEmptyLike;

  return {
    sampledPixels,
    nonBlackPixels,
    nonTransparentPixels,
    uniqueBucketCount,
    meanLuma,
    maxLuma,
    nonBlackRatio,
    blank,
  };
}

const solid = (rgba: [number, number, number, number]) => () => rgba;

const fixtures: FixtureCase[] = [
  {
    name: "transparent",
    width: 16,
    height: 16,
    pixelAt: solid([0, 0, 0, 0]),
    expectedBlank: true,
  },
  {
    name: "solid-black",
    width: 16,
    height: 16,
    pixelAt: solid([0, 0, 0, 255]),
    expectedBlank: true,
  },
  {
    name: "solid-white",
    width: 16,
    height: 16,
    pixelAt: solid([255, 255, 255, 255]),
    expectedBlank: true,
  },
  {
    name: "solid-gray",
    width: 16,
    height: 16,
    pixelAt: solid([96, 96, 96, 255]),
    expectedBlank: true,
  },
  {
    name: "valid-dark-ui",
    width: 16,
    height: 16,
    pixelAt: (x, y) => (y === 8 && x >= 4 && x < 12 ? [40, 40, 40, 255] : [0, 0, 0, 255]),
    expectedBlank: false,
  },
];

mkdirSync(OUT_DIR, { recursive: true });

const results = fixtures.map((fixture) => {
  const path = resolve(OUT_DIR, `${fixture.name}.png`);
  writeRgbaPng(path, fixture);
  const audit = auditPngContent(path);
  const status = audit.blank === fixture.expectedBlank ? "pass" : "fail";
  return { name: fixture.name, path, expectedBlank: fixture.expectedBlank, status, audit };
});

const failed = results.filter((result) => result.status !== "pass");
const receipt = {
  schemaVersion: 1,
  status: failed.length === 0 ? "pass" : "fail",
  outputDir: OUT_DIR,
  results,
};

process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
process.exit(failed.length === 0 ? 0 : 1);
