// tests/autonomous/screenshot-diff.ts
// Pixel-by-pixel image comparison for visual regression testing
//
// This module provides functions to compare PNG screenshots and detect differences.
// Uses pure TypeScript/Bun with no external image processing dependencies.
//
// Run with: bun run tests/autonomous/screenshot-diff.ts

// Bun globals for Node.js compatibility
declare const process: { argv: string[]; exit: (code: number) => never };
declare const Buffer: { from(data: Uint8Array | ArrayBuffer): Uint8Array };
declare global {
  interface ImportMeta {
    main: boolean;
  }
}

// @ts-ignore - Bun supports node: protocol
const fs = await import('node:fs/promises');
// @ts-ignore - Bun supports node: protocol
const path = await import('node:path');
// @ts-ignore - Bun supports node: protocol
const zlib = await import('node:zlib');

/**
 * Result of comparing two images
 */
export interface DiffResult {
  /** Whether images match within tolerance */
  match: boolean;
  /** Percentage of pixels that differ (0-100) */
  diffPercent: number;
  /** Number of pixels that differ */
  diffPixelCount: number;
  /** Total number of pixels compared */
  totalPixels: number;
  /** Width of compared images */
  width: number;
  /** Height of compared images */
  height: number;
  /** Path to generated diff image (if requested) */
  diffImagePath?: string;
  /** Error message if comparison failed */
  error?: string;
  /** Whether dimensions matched */
  dimensionsMatch: boolean;
  /** Details about dimension mismatch */
  dimensionDetails?: {
    image1: { width: number; height: number };
    image2: { width: number; height: number };
  };
}

/**
 * Options for image comparison
 */
export interface DiffOptions {
  /** Tolerance for color difference per channel (0-255, default: 0) */
  tolerance?: number;
  /** Whether to generate a diff image (default: false) */
  generateDiffImage?: boolean;
  /** Path to save diff image (default: auto-generated) */
  diffImagePath?: string;
  /** Color for highlighting differences in diff image (RGBA, default: [255, 0, 0, 255] = red) */
  diffColor?: [number, number, number, number];
  /** Threshold percentage for match (default: 0 = exact match required) */
  thresholdPercent?: number;
}

// PNG constants
const PNG_MAGIC = new Uint8Array([0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);

/**
 * Parse PNG file and extract raw RGBA pixel data
 */
async function decodePNG(filePath: string): Promise<{
  width: number;
  height: number;
  pixels: Uint8Array; // RGBA format, 4 bytes per pixel
}> {
  const buffer = await fs.readFile(filePath);
  const bytes = new Uint8Array(buffer);
  
  // Validate magic bytes
  for (let i = 0; i < PNG_MAGIC.length; i++) {
    if (bytes[i] !== PNG_MAGIC[i]) {
      throw new Error(`Invalid PNG file: ${filePath}`);
    }
  }
  
  // Parse chunks
  let offset = 8;
  let width = 0;
  let height = 0;
  let bitDepth = 0;
  let colorType = 0;
  const idatChunks: Uint8Array[] = [];
  
  while (offset < bytes.length) {
    const length = (bytes[offset] << 24) | (bytes[offset + 1] << 16) | (bytes[offset + 2] << 8) | bytes[offset + 3];
    const type = String.fromCharCode(bytes[offset + 4], bytes[offset + 5], bytes[offset + 6], bytes[offset + 7]);
    const data = bytes.slice(offset + 8, offset + 8 + length);
    
    if (type === 'IHDR') {
      const view = new DataView(data.buffer, data.byteOffset, data.byteLength);
      width = view.getUint32(0, false);
      height = view.getUint32(4, false);
      bitDepth = data[8];
      colorType = data[9];
    } else if (type === 'IDAT') {
      idatChunks.push(data);
    } else if (type === 'IEND') {
      break;
    }
    
    offset += 12 + length; // 4 (length) + 4 (type) + length + 4 (crc)
  }
  
  if (width === 0 || height === 0) {
    throw new Error(`Could not parse PNG dimensions: ${filePath}`);
  }
  
  // Concatenate IDAT chunks
  const totalIdatLength = idatChunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const compressedData = new Uint8Array(totalIdatLength);
  let idatOffset = 0;
  for (const chunk of idatChunks) {
    compressedData.set(chunk, idatOffset);
    idatOffset += chunk.length;
  }
  
  // Decompress using zlib
  const decompressed = zlib.inflateSync(Buffer.from(compressedData));
  const rawData = new Uint8Array(decompressed);
  
  // Determine bytes per pixel based on color type
  let bytesPerPixel: number;
  switch (colorType) {
    case 0: bytesPerPixel = 1; break; // Grayscale
    case 2: bytesPerPixel = 3; break; // RGB
    case 3: bytesPerPixel = 1; break; // Indexed (palette)
    case 4: bytesPerPixel = 2; break; // Grayscale + Alpha
    case 6: bytesPerPixel = 4; break; // RGBA
    default: throw new Error(`Unsupported PNG color type: ${colorType}`);
  }
  
  // Unfilter the data (PNG uses filtering on each scanline)
  const bytesPerRow = width * bytesPerPixel + 1; // +1 for filter type byte
  const pixels = new Uint8Array(width * height * 4); // Always output RGBA
  
  for (let y = 0; y < height; y++) {
    const filterType = rawData[y * bytesPerRow];
    const rowStart = y * bytesPerRow + 1;
    const prevRowStart = (y - 1) * bytesPerRow + 1;
    
    for (let x = 0; x < width * bytesPerPixel; x++) {
      let value = rawData[rowStart + x];
      
      // Apply PNG filter reconstruction
      const a = x >= bytesPerPixel ? rawData[rowStart + x - bytesPerPixel] : 0; // Left
      const b = y > 0 ? rawData[prevRowStart + x] : 0; // Above
      const c = (x >= bytesPerPixel && y > 0) ? rawData[prevRowStart + x - bytesPerPixel] : 0; // Upper-left
      
      switch (filterType) {
        case 0: break; // None
        case 1: value = (value + a) & 0xFF; break; // Sub
        case 2: value = (value + b) & 0xFF; break; // Up
        case 3: value = (value + Math.floor((a + b) / 2)) & 0xFF; break; // Average
        case 4: value = (value + paethPredictor(a, b, c)) & 0xFF; break; // Paeth
      }
      
      rawData[rowStart + x] = value; // Store for use in next iteration
    }
    
    // Convert to RGBA
    for (let x = 0; x < width; x++) {
      const srcIdx = rowStart + x * bytesPerPixel;
      const dstIdx = (y * width + x) * 4;
      
      switch (colorType) {
        case 0: // Grayscale
          pixels[dstIdx] = rawData[srcIdx];
          pixels[dstIdx + 1] = rawData[srcIdx];
          pixels[dstIdx + 2] = rawData[srcIdx];
          pixels[dstIdx + 3] = 255;
          break;
        case 2: // RGB
          pixels[dstIdx] = rawData[srcIdx];
          pixels[dstIdx + 1] = rawData[srcIdx + 1];
          pixels[dstIdx + 2] = rawData[srcIdx + 2];
          pixels[dstIdx + 3] = 255;
          break;
        case 4: // Grayscale + Alpha
          pixels[dstIdx] = rawData[srcIdx];
          pixels[dstIdx + 1] = rawData[srcIdx];
          pixels[dstIdx + 2] = rawData[srcIdx];
          pixels[dstIdx + 3] = rawData[srcIdx + 1];
          break;
        case 6: // RGBA
          pixels[dstIdx] = rawData[srcIdx];
          pixels[dstIdx + 1] = rawData[srcIdx + 1];
          pixels[dstIdx + 2] = rawData[srcIdx + 2];
          pixels[dstIdx + 3] = rawData[srcIdx + 3];
          break;
      }
    }
  }
  
  return { width, height, pixels };
}

/**
 * Paeth predictor for PNG filtering
 */
function paethPredictor(a: number, b: number, c: number): number {
  const p = a + b - c;
  const pa = Math.abs(p - a);
  const pb = Math.abs(p - b);
  const pc = Math.abs(p - c);
  if (pa <= pb && pa <= pc) return a;
  if (pb <= pc) return b;
  return c;
}

/**
 * Encode RGBA pixels to PNG format
 */
function encodePNG(width: number, height: number, pixels: Uint8Array): Uint8Array {
  // Prepare filtered scanlines (using filter type 0 = None for simplicity)
  const bytesPerRow = width * 4 + 1;
  const rawData = new Uint8Array(height * bytesPerRow);
  
  for (let y = 0; y < height; y++) {
    rawData[y * bytesPerRow] = 0; // Filter type: None
    for (let x = 0; x < width * 4; x++) {
      rawData[y * bytesPerRow + 1 + x] = pixels[y * width * 4 + x];
    }
  }
  
  // Compress with zlib
  const compressed = zlib.deflateSync(Buffer.from(rawData), { level: 6 });
  
  // Build PNG file
  const chunks: Uint8Array[] = [];
  
  // Signature
  chunks.push(PNG_MAGIC);
  
  // IHDR chunk
  const ihdr = new Uint8Array(13);
  const ihdrView = new DataView(ihdr.buffer);
  ihdrView.setUint32(0, width, false);
  ihdrView.setUint32(4, height, false);
  ihdr[8] = 8; // bit depth
  ihdr[9] = 6; // color type (RGBA)
  ihdr[10] = 0; // compression
  ihdr[11] = 0; // filter
  ihdr[12] = 0; // interlace
  chunks.push(createChunk('IHDR', ihdr));
  
  // IDAT chunk(s)
  chunks.push(createChunk('IDAT', new Uint8Array(compressed)));
  
  // IEND chunk
  chunks.push(createChunk('IEND', new Uint8Array(0)));
  
  // Concatenate all chunks
  const totalLength = chunks.reduce((sum, chunk) => sum + chunk.length, 0);
  const result = new Uint8Array(totalLength);
  let offset = 0;
  for (const chunk of chunks) {
    result.set(chunk, offset);
    offset += chunk.length;
  }
  
  return result;
}

/**
 * Create a PNG chunk with CRC
 */
function createChunk(type: string, data: Uint8Array): Uint8Array {
  const chunk = new Uint8Array(12 + data.length);
  const view = new DataView(chunk.buffer);
  
  // Length
  view.setUint32(0, data.length, false);
  
  // Type
  for (let i = 0; i < 4; i++) {
    chunk[4 + i] = type.charCodeAt(i);
  }
  
  // Data
  chunk.set(data, 8);
  
  // CRC (of type + data)
  const crcData = new Uint8Array(4 + data.length);
  crcData.set(chunk.slice(4, 8), 0);
  crcData.set(data, 4);
  view.setUint32(8 + data.length, crc32(crcData), false);
  
  return chunk;
}

/**
 * CRC32 calculation for PNG chunks
 */
function crc32(data: Uint8Array): number {
  let crc = 0xFFFFFFFF;
  for (let i = 0; i < data.length; i++) {
    crc ^= data[i];
    for (let j = 0; j < 8; j++) {
      crc = (crc >>> 1) ^ (crc & 1 ? 0xEDB88320 : 0);
    }
  }
  return crc ^ 0xFFFFFFFF;
}

/**
 * Compare two PNG images pixel-by-pixel
 * 
 * @param path1 - Path to first PNG image (usually the baseline)
 * @param path2 - Path to second PNG image (usually the new screenshot)
 * @param options - Comparison options
 * @returns Detailed comparison result
 */
export async function compareImages(
  path1: string,
  path2: string,
  options: DiffOptions = {}
): Promise<DiffResult> {
  const {
    tolerance = 0,
    generateDiffImage = false,
    diffColor = [255, 0, 0, 255],
    thresholdPercent = 0,
  } = options;
  
  let { diffImagePath } = options;
  
  try {
    // Load both images
    const [img1, img2] = await Promise.all([
      decodePNG(path1),
      decodePNG(path2),
    ]);
    
    // Check dimensions
    const dimensionsMatch = img1.width === img2.width && img1.height === img2.height;
    
    if (!dimensionsMatch) {
      return {
        match: false,
        diffPercent: 100,
        diffPixelCount: Math.max(img1.width * img1.height, img2.width * img2.height),
        totalPixels: Math.max(img1.width * img1.height, img2.width * img2.height),
        width: Math.max(img1.width, img2.width),
        height: Math.max(img1.height, img2.height),
        dimensionsMatch: false,
        dimensionDetails: {
          image1: { width: img1.width, height: img1.height },
          image2: { width: img2.width, height: img2.height },
        },
        error: `Dimension mismatch: ${img1.width}x${img1.height} vs ${img2.width}x${img2.height}`,
      };
    }
    
    const { width, height } = img1;
    const totalPixels = width * height;
    let diffPixelCount = 0;
    
    // Create diff image buffer if requested
    const diffPixels = generateDiffImage ? new Uint8Array(totalPixels * 4) : null;
    
    // Compare pixels
    for (let i = 0; i < totalPixels; i++) {
      const idx = i * 4;
      
      const r1 = img1.pixels[idx];
      const g1 = img1.pixels[idx + 1];
      const b1 = img1.pixels[idx + 2];
      const a1 = img1.pixels[idx + 3];
      
      const r2 = img2.pixels[idx];
      const g2 = img2.pixels[idx + 1];
      const b2 = img2.pixels[idx + 2];
      const a2 = img2.pixels[idx + 3];
      
      // Check if pixels differ beyond tolerance
      const isDifferent = 
        Math.abs(r1 - r2) > tolerance ||
        Math.abs(g1 - g2) > tolerance ||
        Math.abs(b1 - b2) > tolerance ||
        Math.abs(a1 - a2) > tolerance;
      
      if (isDifferent) {
        diffPixelCount++;
      }
      
      // Build diff image
      if (diffPixels) {
        if (isDifferent) {
          // Highlight difference with diff color
          diffPixels[idx] = diffColor[0];
          diffPixels[idx + 1] = diffColor[1];
          diffPixels[idx + 2] = diffColor[2];
          diffPixels[idx + 3] = diffColor[3];
        } else {
          // Dim the matching pixels (grayscale with reduced opacity)
          const gray = Math.round((r1 + g1 + b1) / 3 * 0.3);
          diffPixels[idx] = gray;
          diffPixels[idx + 1] = gray;
          diffPixels[idx + 2] = gray;
          diffPixels[idx + 3] = 128;
        }
      }
    }
    
    const diffPercent = (diffPixelCount / totalPixels) * 100;
    const match = diffPercent <= thresholdPercent;
    
    // Save diff image if requested
    if (generateDiffImage && diffPixels) {
      if (!diffImagePath) {
        const dir = path.dirname(path2);
        const base = path.basename(path2, '.png');
        diffImagePath = path.join(dir, `${base}-diff.png`);
      }
      
      const pngData = encodePNG(width, height, diffPixels);
      await fs.writeFile(diffImagePath, pngData);
    }
    
    return {
      match,
      diffPercent: Math.round(diffPercent * 100) / 100,
      diffPixelCount,
      totalPixels,
      width,
      height,
      dimensionsMatch: true,
      diffImagePath: generateDiffImage ? diffImagePath : undefined,
    };
  } catch (error) {
    return {
      match: false,
      diffPercent: 100,
      diffPixelCount: 0,
      totalPixels: 0,
      width: 0,
      height: 0,
      dimensionsMatch: false,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

/**
 * Quick check if two images are identical (byte-by-byte)
 * Faster than pixel comparison when exact match is expected
 */
export async function imagesIdentical(path1: string, path2: string): Promise<boolean> {
  try {
    const [buffer1, buffer2] = await Promise.all([
      fs.readFile(path1),
      fs.readFile(path2),
    ]);
    
    if (buffer1.length !== buffer2.length) {
      return false;
    }
    
    const bytes1 = new Uint8Array(buffer1);
    const bytes2 = new Uint8Array(buffer2);
    
    for (let i = 0; i < bytes1.length; i++) {
      if (bytes1[i] !== bytes2[i]) {
        return false;
      }
    }
    
    return true;
  } catch {
    return false;
  }
}

/**
 * Format diff result as human-readable string
 */
export function formatDiffResult(result: DiffResult): string {
  const lines: string[] = [];
  
  if (result.error) {
    lines.push(`ERROR: ${result.error}`);
    return lines.join('\n');
  }
  
  const status = result.match ? 'MATCH' : 'DIFFERENT';
  lines.push(`Status: ${status}`);
  lines.push(`Dimensions: ${result.width}x${result.height}`);
  lines.push(`Diff: ${result.diffPercent}% (${result.diffPixelCount}/${result.totalPixels} pixels)`);
  
  if (result.diffImagePath) {
    lines.push(`Diff Image: ${result.diffImagePath}`);
  }
  
  if (result.dimensionDetails) {
    lines.push(`Image 1: ${result.dimensionDetails.image1.width}x${result.dimensionDetails.image1.height}`);
    lines.push(`Image 2: ${result.dimensionDetails.image2.width}x${result.dimensionDetails.image2.height}`);
  }
  
  return lines.join('\n');
}

/**
 * Format diff result as JSONL for machine parsing
 */
export function formatDiffResultJSON(result: DiffResult, testName?: string): string {
  return JSON.stringify({
    test: testName || 'visual-diff',
    status: result.match ? 'pass' : 'fail',
    timestamp: new Date().toISOString(),
    diff_percent: result.diffPercent,
    diff_pixels: result.diffPixelCount,
    total_pixels: result.totalPixels,
    dimensions: `${result.width}x${result.height}`,
    dimensions_match: result.dimensionsMatch,
    diff_image: result.diffImagePath,
    error: result.error,
  });
}

// Export for use as module
export { decodePNG, encodePNG };

// Self-test when run directly
if (import.meta.main) {
  console.log('Screenshot Diff Utility');
  console.log('=======================');
  console.log('');
  console.log('Usage: bun run tests/autonomous/screenshot-diff.ts <image1.png> <image2.png> [options]');
  console.log('');
  console.log('Options:');
  console.log('  --tolerance <n>     Color tolerance per channel (0-255, default: 0)');
  console.log('  --threshold <n>     Pass threshold percentage (default: 0)');
  console.log('  --diff              Generate diff image');
  console.log('  --json              Output as JSONL');
  console.log('');
  
  const args = process.argv.slice(2);
  
  if (args.length >= 2 && !args[0].startsWith('-')) {
    const path1 = args[0];
    const path2 = args[1];
    const tolerance = args.includes('--tolerance') 
      ? parseInt(args[args.indexOf('--tolerance') + 1]) 
      : 0;
    const threshold = args.includes('--threshold')
      ? parseFloat(args[args.indexOf('--threshold') + 1])
      : 0;
    const generateDiff = args.includes('--diff');
    const jsonOutput = args.includes('--json');
    
    const result = await compareImages(path1, path2, {
      tolerance,
      thresholdPercent: threshold,
      generateDiffImage: generateDiff,
    });
    
    if (jsonOutput) {
      console.log(formatDiffResultJSON(result));
    } else {
      console.log(formatDiffResult(result));
    }
    
    process.exit(result.match ? 0 : 1);
  }
}
