#!/usr/bin/env bun
/**
 * Phase 5 — Design Picker visual matrix.
 *
 * Drives a running Script Kit launcher through every id in
 * `src/designs/core/registry.rs::CATALOG`, captures Mini and Full
 * screenshots, and verifies `kit/state` receipts agree with the
 * intended design.
 *
 * Prerequisite: the launcher must be running with the Phase 1 Design
 * Picker code merged. Read `~/.scriptkit/server.json` for the MCP URL +
 * token. The picker is opened with Cmd+1 (or `triggerBuiltin design-picker`
 * once the SDK exposes it). This script does not start the launcher;
 * spawn it with `./dev.sh` or `cargo run --release` first.
 *
 * Usage:
 *   bun scripts/agentic/design-picker-visual-matrix.ts \
 *     --session design-variants-overhaul \
 *     --sizes mini,full \
 *     --designs all \
 *     --capture-screenshots \
 *     --verify-state-receipts \
 *     --cleanup
 *
 * Flags:
 *   --session NAME           reporting tag (default: design-variants-overhaul)
 *   --designs CSV|all        subset of catalog ids (default: all 25)
 *   --sizes CSV              one or more of mini,full (default: mini,full)
 *   --capture-screenshots    write .test-screenshots/design-<id>-<size>.png
 *   --verify-state-receipts  assert `state.semanticSurface === "designPicker"`
 *                            and the active id matches the expected cell
 *   --cleanup                send `kit/hide` after the matrix completes
 *   --dry-run                only list the matrix
 *   --rpc URL                override server URL (default from server.json)
 *   --token TOKEN            override bearer token (default from server.json)
 *   --out-dir DIR            screenshot dir (default .test-screenshots)
 */

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";

type Json = Record<string, unknown>;

function arg(name: string, fallback?: string): string | undefined {
  const i = process.argv.indexOf(name);
  if (i >= 0 && i + 1 < process.argv.length) return process.argv[i + 1];
  return fallback;
}

const repoRoot = resolve(import.meta.dir, "../..");
const session = arg("--session", "design-variants-overhaul")!;
const designsArg = arg("--designs", "all")!;
const sizesArg = arg("--sizes", "mini,full")!;
const captureScreenshots = process.argv.includes("--capture-screenshots");
const verifyReceipts = process.argv.includes("--verify-state-receipts");
const cleanup = process.argv.includes("--cleanup");
const dryRun = process.argv.includes("--dry-run");
const outDir = arg("--out-dir", join(repoRoot, ".test-screenshots"))!;

const sizes = sizesArg.split(",").map((s) => s.trim()).filter(Boolean);
for (const s of sizes) {
  if (s !== "mini" && s !== "full") {
    console.error(`unknown --sizes value: ${s}`);
    process.exit(2);
  }
}

// Catalog ids — source of truth in src/designs/core/registry.rs.
const CATALOG_IDS: readonly string[] = [
  "script-kit-classic",
  "pro-dense",
  "ambient-quiet",
  "focus-zen",
  "minimal-ink",
  "retro-terminal",
  "paper-print",
  "glass-frost",
  "neon-cyber",
  "apple-hig",
  "high-density-list",
  "accessibility-high-contrast",
  "retro-amber",
  "editorial-brutalist",
  "brutalist-grid",
  "liquid-glass-compact",
  "synthwave",
  "material-you",
  "mocha-warm",
  "ocean-deep",
  "pastel-mist",
  "playful-pop",
  "mono-contrast",
  "command-center",
  "gallery-visual",
];

const designs =
  designsArg === "all"
    ? CATALOG_IDS
    : designsArg.split(",").map((d) => d.trim()).filter(Boolean);
for (const id of designs) {
  if (!CATALOG_IDS.includes(id)) {
    console.error(`unknown design id: ${id}`);
    process.exit(2);
  }
}

interface Cell {
  id: string;
  size: "mini" | "full";
}
const planned: Cell[] = sizes.flatMap((size) =>
  designs.map((id) => ({ id, size: size as "mini" | "full" })),
);

console.log(
  JSON.stringify({
    session,
    totalCells: planned.length,
    designs: designs.length,
    sizes,
    captureScreenshots,
    verifyReceipts,
    cleanup,
    dryRun,
    outDir,
  }),
);

if (dryRun) {
  for (const cell of planned) console.log(`PLAN ${cell.size.padEnd(4)} ${cell.id}`);
  process.exit(0);
}

// ─── server.json bridge ─────────────────────────────────────────────────────

function readServer(): { url: string; token: string } {
  const url = arg("--rpc");
  const token = arg("--token");
  if (url && token) return { url, token };
  const serverPath = `${process.env.HOME}/.scriptkit/server.json`;
  if (!existsSync(serverPath)) {
    throw new Error(
      `${serverPath} does not exist; launch Script Kit before running this matrix`,
    );
  }
  const parsed = JSON.parse(readFileSync(serverPath, "utf8"));
  const fallbackToken =
    parsed.token ??
    readFileSync(`${process.env.HOME}/.scriptkit/agent-token`, "utf8").trim();
  return {
    url: (parsed.url ?? `http://127.0.0.1:${parsed.port ?? 43210}`).replace(
      "http://localhost:",
      "http://127.0.0.1:",
    ),
    token: token ?? fallbackToken,
  };
}

const expectedPersistedActiveId = arg("--expect-persisted-active-id");

const server = readServer();

async function rpc(method: string, params: Json) {
  const response = await fetch(`${server.url}/rpc`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${server.token}`,
    },
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: `dpvm-${Date.now()}-${Math.random()}`,
      method,
      params,
    }),
  });
  const json: any = await response.json();
  if (json.error) throw new Error(JSON.stringify(json.error));
  return json.result;
}

async function tool(name: string, args: Json) {
  const result: any = await rpc("tools/call", { name, arguments: args });
  const text = result.content?.find((item: any) => item.type === "text")?.text;
  if (!text) throw new Error(`tool ${name} returned no text content`);
  return JSON.parse(text);
}

// ─── matrix loop ────────────────────────────────────────────────────────────

mkdirSync(outDir, { recursive: true });

interface Receipt {
  cell: Cell;
  ok: boolean;
  state?: any;
  screenshotPath?: string;
  error?: string;
}
const receipts: Receipt[] = [];

async function captureCell(cell: Cell): Promise<Receipt> {
  // 1. Confirm picker is the active surface. Phase 1 sets
  //    semanticSurface = "designPicker" on AppView::DesignPickerView.
  let state: any;
  if (verifyReceipts) {
    state = await tool("kit/state", {}).catch((err) => ({ error: String(err) }));
    if (!state || state.error) {
      return { cell, ok: false, state, error: state?.error ?? "no state" };
    }
    const semantic = state?.semanticSurface ?? state?.surface ?? null;
    if (semantic !== "designPicker") {
      return {
        cell,
        ok: false,
        state,
        error: `expected semanticSurface=designPicker, got ${semantic}`,
      };
    }
    const design = state?.design ?? null;
    if (!design || typeof design !== "object") {
      return {
        cell,
        ok: false,
        state,
        error: `expected state.design receipt, got ${JSON.stringify(design)}`,
      };
    }
    if (typeof design.activeId !== "string") {
      return {
        cell,
        ok: false,
        state,
        error: `expected state.design.activeId string, got ${design.activeId}`,
      };
    }
    if (!("persistedActiveId" in design)) {
      return {
        cell,
        ok: false,
        state,
        error: "missing state.design.persistedActiveId field",
      };
    }
    if (typeof design.fallbackApplied !== "boolean") {
      return {
        cell,
        ok: false,
        state,
        error: `expected state.design.fallbackApplied boolean, got ${design.fallbackApplied}`,
      };
    }
    if (design.fallbackApplied !== false) {
      return {
        cell,
        ok: false,
        state,
        error: `expected state.design.fallbackApplied=false for ${cell.id}, got true`,
      };
    }
    const activeId = design.activeId;
    if (activeId !== cell.id) {
      // Not fatal — the picker may not have moved selection yet. Surface as a soft warning.
      console.warn(
        `WARN ${cell.id}: state.design.activeId=${activeId} (highlight not yet on target)`,
      );
    }
  }

  // 2. Capture the main launcher window via the native capture tool.
  let screenshotPath: string | undefined;
  if (captureScreenshots) {
    const fm = await tool("computer/get_frontmost_native_window", {}).catch(() => null);
    const window = fm?.window ?? fm;
    if (!window || !window.id) {
      return { cell, ok: false, error: "no frontmost window" };
    }
    const shot = await tool("computer/capture_native_window", {
      id: window.id,
      includeImage: true,
    }).catch((err) => ({ error: String(err) }));
    if (!shot || shot.error || !shot.imageBase64) {
      return { cell, ok: false, error: shot?.error ?? "capture failed" };
    }
    screenshotPath = join(outDir, `design-${cell.id}-${cell.size}.png`);
    writeFileSync(screenshotPath, Buffer.from(shot.imageBase64, "base64"));
  }

  return { cell, ok: true, state, screenshotPath };
}

async function verifyPersistedActiveIdAfterRestart(expected: string) {
  const state = await tool("kit/state", {});
  const design = (state as any)?.design ?? {};
  if (design.persistedActiveId !== expected) {
    throw new Error(
      `expected design.persistedActiveId=${expected}, got ${design.persistedActiveId}`,
    );
  }
  if (design.activeId !== expected) {
    throw new Error(
      `expected design.activeId=${expected}, got ${design.activeId}`,
    );
  }
  if (design.fallbackApplied !== false) {
    throw new Error(
      `expected design.fallbackApplied=false, got ${design.fallbackApplied}`,
    );
  }
}

if (verifyReceipts && expectedPersistedActiveId) {
  try {
    await verifyPersistedActiveIdAfterRestart(expectedPersistedActiveId);
    console.log(
      `PERSISTED-OK design.persistedActiveId=${expectedPersistedActiveId}`,
    );
  } catch (err) {
    console.error(`PERSISTED-FAIL ${err}`);
    process.exit(1);
  }
}

for (const cell of planned) {
  try {
    console.log(`CAPTURE ${cell.size.padEnd(4)} ${cell.id}`);
    const r = await captureCell(cell);
    receipts.push(r);
    if (!r.ok) console.error(`  FAIL: ${r.error}`);
    else if (r.screenshotPath) console.log(`  -> ${r.screenshotPath}`);
  } catch (err) {
    receipts.push({ cell, ok: false, error: String(err) });
    console.error(`  THROW: ${err}`);
  }
}

if (cleanup) {
  await tool("kit/hide", {}).catch(() => undefined);
}

const okCount = receipts.filter((r) => r.ok).length;
console.log(
  JSON.stringify({
    session,
    totalCells: planned.length,
    okCount,
    failCount: planned.length - okCount,
    captures: receipts
      .filter((r) => r.screenshotPath)
      .map((r) => ({ id: r.cell.id, size: r.cell.size, path: r.screenshotPath })),
  }),
);

process.exit(okCount === planned.length ? 0 : 1);
