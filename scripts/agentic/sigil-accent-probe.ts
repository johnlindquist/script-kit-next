#!/usr/bin/env bun
/**
 * scripts/agentic/sigil-accent-probe.ts
 *
 * Visual + state receipts for completed-sigil accent highlighting:
 *  - `@file:<query>` + Enter must rewrite the input to the compact
 *    `@file:basename` token AND keep the WHOLE token accent colored.
 *  - `@clipboard:<query>` + Enter behaves the same (seeded clipboard db).
 *  - `@selection ` (no sub-query) keeps its existing full-token accent.
 *
 * Usage: bun scripts/agentic/sigil-accent-probe.ts
 */

import { join, resolve } from "node:path";
import { mkdirSync, writeFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { Driver } from "../devtools/driver";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/sigil-accent/script-kit-gpui");
const OUT_DIR = join(PROJECT_ROOT, ".test-output/sigil-accent", String(process.pid));
const HOME_DIR = join(OUT_DIR, "home");
const KIT_DIR = join(HOME_DIR, ".scriptkit");
const CLIP_QUERY = `sigilaccent${Date.now()}`;

function seedClipboardDb() {
  const dbDir = join(KIT_DIR, "db");
  mkdirSync(dbDir, { recursive: true });
  const now = Date.now();
  const sql = `
CREATE TABLE IF NOT EXISTS history (
  id TEXT PRIMARY KEY,
  content TEXT NOT NULL,
  content_hash TEXT,
  content_type TEXT NOT NULL DEFAULT 'text',
  timestamp INTEGER NOT NULL,
  pinned INTEGER DEFAULT 0,
  ocr_text TEXT,
  text_preview TEXT,
  image_width INTEGER,
  image_height INTEGER,
  byte_size INTEGER
);
DELETE FROM history;
INSERT INTO history (id, content, content_hash, content_type, timestamp, pinned, ocr_text, text_preview, image_width, image_height, byte_size)
VALUES ('clip-sigil-accent', '${CLIP_QUERY} seeded clipboard text', 'fixture-hash', 'text', ${now}, 0, NULL, '${CLIP_QUERY} seeded clipboard text', NULL, NULL, 64);
`;
  const result = spawnSync("sqlite3", [join(dbDir, "clipboard-history.sqlite")], {
    input: sql,
    encoding: "utf8",
  });
  if (result.status !== 0) throw new Error(`sqlite3 seed failed: ${result.stderr}`);
}

async function main() {
  mkdirSync(OUT_DIR, { recursive: true });
  mkdirSync(KIT_DIR, { recursive: true });
  seedClipboardDb();

  const driver = await Driver.launch({
    binary: BINARY,
    sessionName: "sigil-accent",
    defaultTimeoutMs: 8000,
    env: {
      HOME: HOME_DIR,
      SK_PATH: KIT_DIR,
      // Deterministic file-search results: the sandbox home has no Spotlight
      // index, so inject a fixture row for the `@file:sigfile` sub-query.
      SCRIPT_KIT_ROOT_FILE_SEARCH_TEST_PROVIDER: JSON.stringify({
        query: "sigfile",
        delayMs: 0,
        results: [
          {
            path: "/tmp/sigfile-fixture.txt",
            name: "sigfile-fixture.txt",
            fileType: "document",
            size: 42,
            modified: Date.now(),
          },
        ],
      }),
    },
  });
  const receipts: Record<string, unknown> = {};
  try {
    const gpuiEnter = () =>
      driver.request(
        {
          type: "simulateGpuiEvent",
          target: { type: "kind", kind: "main" },
          event: { type: "keyDown", key: "enter", modifiers: [] },
        },
        { expect: "simulateGpuiEventResult" },
      );

    const chips = async () => {
      const state = await driver.getState();
      return {
        input: state.inputValue,
        chips: (state as Record<string, any>).filterInputDecorations?.chips ?? [],
      };
    };
    const capture = async (name: string) => {
      const path = join(OUT_DIR, `${name}.png`);
      const result = (await driver.captureScreenshot({
        target: { type: "kind", kind: "main" },
        savePath: path,
      })) as { error?: string };
      return result.error ? `ERROR: ${result.error}` : path;
    };

    const resetToScriptList = async () => {
      driver.simulateKey("escape");
      await Bun.sleep(400);
      driver.simulateKey("escape");
      await Bun.sleep(400);
      await driver.request({ type: "show" }, { timeoutMs: 1500 }).catch(() => {});
      await driver.setFilterAndWait("");
      await Bun.sleep(300);
    };

    await driver.request({ type: "show" }, { timeoutMs: 1500 }).catch(() => {});
    await Bun.sleep(500);

    // --- Case 1: @file: select-a-file flow ---
    await driver.setFilterAndWait("@file:sigfile");
    await Bun.sleep(1200); // debounce + fixture delivery
    const fileBefore = await chips();
    await gpuiEnter();
    await Bun.sleep(900);
    const fileAfter = await chips();
    receipts.file = {
      before: fileBefore,
      after: fileAfter,
      resolved:
        typeof fileAfter.input === "string" &&
        /^@file:\S+ $/.test(fileAfter.input),
      fullTokenChip: fileAfter.chips.some(
        (c: { text: string }) => c.text === String(fileAfter.input).trimEnd(),
      ),
      shot: await capture("file-token-after-select"),
    };

    // --- Case 2: @clipboard: select flow (seeded entry) ---
    await resetToScriptList();
    await driver.setFilterAndWait(`@clipboard:${CLIP_QUERY}`);
    await Bun.sleep(900);
    const clipBefore = await chips();
    await gpuiEnter();
    await Bun.sleep(900);
    const clipAfter = await chips();
    receipts.clipboard = {
      before: clipBefore,
      after: clipAfter,
      resolved:
        typeof clipAfter.input === "string" &&
        /^@clipboard:\S+ $/.test(clipAfter.input),
      fullTokenChip: clipAfter.chips.some(
        (c: { text: string }) => c.text === String(clipAfter.input).trimEnd(),
      ),
      shot: await capture("clipboard-token-after-select"),
    };

    // --- Case 3: @selection full-token accent (existing behavior guard) ---
    await resetToScriptList();
    await driver.setFilterAndWait("@selection ");
    await Bun.sleep(500);
    receipts.selection = {
      state: await chips(),
      shot: await capture("selection-token"),
    };

    const summary = { sessionDir: driver.sessionDir, outDir: OUT_DIR, receipts };
    writeFileSync(join(OUT_DIR, "receipt.json"), JSON.stringify(summary, null, 2));
    console.log(JSON.stringify(summary, null, 2));
  } finally {
    await driver.close();
  }
}

main();
