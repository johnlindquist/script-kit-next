#!/usr/bin/env bun
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { createHash } from "node:crypto";
import { join } from "node:path";

type Json = Record<string, any>;

const outDir = valueAfter("--out") ?? ".test-output/native-window-capture";
const screenshotDir =
  valueAfter("--screenshots") ?? ".test-screenshots/native-window-capture";
const includeImage = process.argv.includes("--include-image");

mkdirSync(outDir, { recursive: true });
mkdirSync(screenshotDir, { recursive: true });

function valueAfter(flag: string): string | undefined {
  const index = process.argv.indexOf(flag);
  return index >= 0 ? process.argv[index + 1] : undefined;
}

function readServer() {
  const serverPath = `${process.env.HOME}/.scriptkit/server.json`;
  if (!existsSync(serverPath)) {
    throw new Error(`${serverPath} does not exist; start Script Kit before running this proof`);
  }
  const parsed = JSON.parse(readFileSync(serverPath, "utf8"));
  const token =
    parsed.token ??
    readFileSync(`${process.env.HOME}/.scriptkit/agent-token`, "utf8").trim();

  return {
    url: (parsed.url ?? `http://127.0.0.1:${parsed.port ?? 43210}`).replace(
      "http://localhost:",
      "http://127.0.0.1:",
    ),
    token,
  };
}

const server = readServer();

async function mcp(method: string, params: Json) {
  const response = await fetch(`${server.url}/rpc`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${server.token}`,
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: `native-window-capture-${Date.now()}-${Math.random()}`,
      method,
      params,
    }),
  });
  const json = await response.json();
  if (json.error) {
    throw new Error(JSON.stringify(json.error));
  }
  return json.result;
}

async function callTool(name: string, args: Json) {
  const result = await mcp("tools/call", { name, arguments: args });
  return parseToolResult(name, result);
}

function parseToolResult(name: string, result: Json) {
  const text = result.content?.find((item: Json) => item.type === "text")?.text;
  if (!text) {
    throw new Error(`Tool ${name} returned no text content`);
  }
  return JSON.parse(text);
}

function nativeWindows(list: Json) {
  const rows: Array<{ app: Json; window: Json }> = [];
  for (const appEntry of list.apps ?? []) {
    const app = appEntry.app ?? appEntry;
    for (const window of appEntry.windows ?? []) {
      rows.push({ app, window });
    }
  }
  return rows;
}

const list = await callTool("computer/list_native_windows", {
  includeHidden: true,
  includeBackground: true,
});
const selected = nativeWindows(list).find(
  ({ window }) => window.observation?.captureSelectionCandidate?.status === "candidate",
);
const nonCandidate = nativeWindows(list).find(
  ({ window }) => window.observation?.captureSelectionCandidate?.status !== "candidate",
);

if (!selected) {
  throw new Error("No capture-ready native window found");
}

writeFileSync(join(outDir, "selected-window.json"), JSON.stringify(selected, null, 2));

const captureReceipt = await callTool("computer/capture_native_window", {
  pid: selected.app.pid,
  nativeWindowId: selected.window.nativeWindowId,
  expectedBundleId: selected.app.bundleId,
  includeImage,
  hiDpi: false,
});
writeFileSync(
  join(outDir, "capture-receipt.json"),
  JSON.stringify(captureReceipt, null, 2),
);

if (captureReceipt.status !== "captured") {
  throw new Error(`Capture failed: ${JSON.stringify(captureReceipt, null, 2)}`);
}
if (captureReceipt.capture?.mimeType !== "image/png") {
  throw new Error("Capture receipt did not report image/png");
}
if (!captureReceipt.capture?.sha256 || captureReceipt.capture.sha256.length !== 64) {
  throw new Error("Capture receipt did not include a 64-character SHA-256");
}
if (captureReceipt.capture.pixelAudit?.blankLike !== false) {
  throw new Error("Capture receipt pixel audit did not prove a nonblank image");
}

if (includeImage) {
  if (!captureReceipt.capture?.pngBase64) {
    throw new Error("Capture receipt omitted pngBase64 even though includeImage was true");
  }
  const decoded = Buffer.from(captureReceipt.capture.pngBase64, "base64");
  const pngMagic = decoded.subarray(0, 8).toString("hex");
  if (pngMagic !== "89504e470d0a1a0a") {
    throw new Error(`Decoded image did not start with PNG magic bytes: ${pngMagic}`);
  }
  if (decoded.byteLength !== captureReceipt.capture.byteLength) {
    throw new Error(
      `Decoded image byteLength ${decoded.byteLength} did not match receipt byteLength ${captureReceipt.capture.byteLength}`,
    );
  }
  const decodedSha256 = createHash("sha256").update(decoded).digest("hex");
  if (decodedSha256 !== captureReceipt.capture.sha256) {
    throw new Error(
      `Decoded image sha256 ${decodedSha256} did not match receipt sha256 ${captureReceipt.capture.sha256}`,
    );
  }
  const safeCorrelation = captureReceipt.correlationId.replace(/[^a-zA-Z0-9_.-]/g, "_");
  writeFileSync(
    join(screenshotDir, `${safeCorrelation}.png`),
    decoded,
  );
}

const schemaRejection = await callTool("computer/capture_native_window", {
  pid: selected.app.pid,
  nativeWindowId: selected.window.nativeWindowId,
  expectedBundleId: selected.app.bundleId,
  unexpectedFieldForProof: true,
});
writeFileSync(
  join(outDir, "schema-rejection-receipt.json"),
  JSON.stringify(schemaRejection, null, 2),
);
if (schemaRejection.errorCode !== "invalid_arguments") {
  throw new Error(`Expected invalid_arguments, got ${JSON.stringify(schemaRejection)}`);
}

if (nonCandidate) {
  const nonCandidateReceipt = await callTool("computer/capture_native_window", {
    pid: nonCandidate.app.pid,
    nativeWindowId: nonCandidate.window.nativeWindowId,
    expectedBundleId: nonCandidate.app.bundleId,
    includeImage: true,
  });
  writeFileSync(
    join(outDir, "non-candidate-receipt.json"),
    JSON.stringify(nonCandidateReceipt, null, 2),
  );
  if (nonCandidateReceipt.status !== "notCaptureCandidate") {
    throw new Error(`Expected notCaptureCandidate, got ${nonCandidateReceipt.status}`);
  }
  if (nonCandidateReceipt.capture != null) {
    throw new Error("Non-candidate proof unexpectedly returned a capture");
  }
} else {
  writeFileSync(
    join(outDir, "non-candidate-receipt.json"),
    JSON.stringify(
      {
        skipped: true,
        reason: "No listed native window had captureSelectionCandidate.status other than candidate",
      },
      null,
      2,
    ),
  );
}

const wrongOwnerReceipt = await callTool("computer/capture_native_window", {
  pid: selected.app.pid,
  nativeWindowId: selected.window.nativeWindowId,
  expectedBundleId: "invalid.bundle.id.for.proof",
  includeImage: true,
});
writeFileSync(
  join(outDir, "wrong-owner-receipt.json"),
  JSON.stringify(wrongOwnerReceipt, null, 2),
);
if (wrongOwnerReceipt.status !== "ownershipMismatch") {
  throw new Error(`Expected ownershipMismatch, got ${wrongOwnerReceipt.status}`);
}
if (wrongOwnerReceipt.capture != null) {
  throw new Error("Wrong-owner proof unexpectedly returned a capture");
}

const missingWindowReceipt = await callTool("computer/capture_native_window", {
  pid: selected.app.pid,
  nativeWindowId: 4_294_967_295,
  expectedBundleId: selected.app.bundleId,
  includeImage: true,
});
writeFileSync(
  join(outDir, "missing-window-receipt.json"),
  JSON.stringify(missingWindowReceipt, null, 2),
);
if (missingWindowReceipt.status !== "windowNotFound") {
  throw new Error(`Expected windowNotFound, got ${missingWindowReceipt.status}`);
}
if (missingWindowReceipt.capture != null) {
  throw new Error("Missing-window proof unexpectedly returned a capture");
}

console.log(
  JSON.stringify(
    {
      ok: true,
      selected: {
        pid: selected.app.pid,
        bundleId: selected.app.bundleId,
        nativeWindowId: selected.window.nativeWindowId,
      },
      capture: {
        status: captureReceipt.status,
        correlationId: captureReceipt.correlationId,
        sha256: captureReceipt.capture.sha256,
        byteLength: captureReceipt.capture.byteLength,
        width: captureReceipt.capture.width,
        height: captureReceipt.capture.height,
        blankLike: captureReceipt.capture.pixelAudit.blankLike,
      },
    },
    null,
    2,
  ),
);
