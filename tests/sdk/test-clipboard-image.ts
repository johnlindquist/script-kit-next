// Name: SDK Test - Clipboard Image
// Description: Verifies image clipboard write/read roundtrip through the SDK.

import "../../scripts/kit-sdk";

type TestStatus = "running" | "pass" | "fail";

function logTest(
  name: string,
  status: TestStatus,
  extra?: Record<string, unknown>,
) {
  console.log(JSON.stringify({
    test: name,
    status,
    timestamp: new Date().toISOString(),
    ...extra,
  }));
}

function assertPng(buffer: Buffer, expectedWidth: number, expectedHeight: number) {
  const pngMagic = "89504e470d0a1a0a";
  if (buffer.subarray(0, 8).toString("hex") !== pngMagic) {
    throw new Error("readImage did not return PNG bytes");
  }

  const width = buffer.readUInt32BE(16);
  const height = buffer.readUInt32BE(20);
  if (width !== expectedWidth || height !== expectedHeight) {
    throw new Error(`PNG dimensions mismatch: ${width}x${height}`);
  }
}

const testName = "clipboard-image-roundtrip";
logTest(testName, "running");
const started = Date.now();

try {
  const oneByOnePng = Buffer.from(
    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==",
    "base64",
  );

  assertPng(oneByOnePng, 1, 1);
  await clipboard.writeImage(oneByOnePng);
  const readBack = await clipboard.readImage();
  assertPng(readBack, 1, 1);

  await clipboard.writeText("not an image");
  try {
    await clipboard.readImage();
    throw new Error("readImage unexpectedly resolved for text clipboard");
  } catch (err) {
    const code = (err as { code?: string }).code;
    if (code !== "ERR_CLIPBOARD_IMAGE_NOT_AVAILABLE") {
      throw err;
    }
  }

  try {
    await clipboard.writeImage(Buffer.from("not an encoded image"));
    throw new Error("writeImage unexpectedly resolved for invalid bytes");
  } catch (err) {
    const code = (err as { code?: string }).code;
    if (code !== "ERR_CLIPBOARD_IMAGE_DECODE_FAILED") {
      throw err;
    }
  }

  logTest(testName, "pass", {
    result: {
      format: "png",
      width: readBack.readUInt32BE(16),
      height: readBack.readUInt32BE(20),
      bytes: readBack.length,
      magic: readBack.subarray(0, 8).toString("hex"),
    },
    duration_ms: Date.now() - started,
  });
} catch (err) {
  logTest(testName, "fail", {
    error: String(err),
    duration_ms: Date.now() - started,
  });
  process.exitCode = 1;
}
