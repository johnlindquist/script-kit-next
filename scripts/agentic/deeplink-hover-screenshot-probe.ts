#!/usr/bin/env bun
/**
 * Visual proof for the deeplink hover hint chip: hover a kit:// link in the
 * Day Page and the Notes window, screenshot each surface while hovered (and
 * once un-hovered for contrast). A human/agent reviews the PNGs to confirm
 * the chip paints; runtime state is asserted by deeplink-hover-hint-probe.ts.
 */
import { join, resolve } from "node:path";
import { mkdirSync } from "node:fs";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/deeplink-hover/script-kit-gpui");
const OUT_DIR = process.env.PROBE_SHOT_DIR ?? join(PROJECT_ROOT, ".test-output/deeplink-hover-shots");

type Obj = Record<string, any>;
const LINK_MARKDOWN = "[scripts](kit://scripts)";

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Obj) : {};
}

function walk(node: unknown, out: Obj[] = []): Obj[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walk(item, out);
    return out;
  }
  out.push(node as Obj);
  for (const value of Object.values(node as Obj)) walk(value, out);
  return out;
}

function linkCenter(elements: Json, editorSemanticId: string): { x: number; y: number } | null {
  const editor = walk(elements).find((el) => el.semanticId === editorSemanticId);
  const info = asObj(asObj(asObj(editor?.style).editorRuntime).markdownLinkHighlightRanges);
  const ranges = Array.isArray(info.ranges) ? info.ranges.map(asObj) : [];
  const target = ranges.find((r) => r.role === "markdownLinkUri") ?? ranges[0];
  const b = asObj(target?.bounds);
  if (!(b.width > 0)) return null;
  return { x: b.x + b.width / 2, y: b.y + b.height / 2 };
}

mkdirSync(OUT_DIR, { recursive: true });

/// Screenshot right when the hover receipt confirms the chip was built: a
/// blind sleep lets a real OS mouse event clear the synthetic hover first
/// and captures a chip-less frame.
async function pollHoverReceipt(read: () => Promise<Obj | null>, timeoutMs = 3000): Promise<Obj | null> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    const receipt = await read();
    if (receipt && receipt.verb) return receipt;
    await Bun.sleep(100);
  }
  return null;
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "deeplink-hover-shots",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

const results: Obj = { outDir: OUT_DIR };
try {
  await openDayPage(driver, `hover-shots-${Date.now().toString(36)}`);
  await driver.batch([{ type: "setInput", text: LINK_MARKDOWN }], { timeoutMs: 8000 });
  await Bun.sleep(400);

  let elements = await driver.getElements({ target: { type: "main" }, limit: 300 }, { timeoutMs: 8000 });
  const dayCenter = linkCenter(elements, "input:day-page-editor");
  results.dayCenter = dayCenter;
  if (dayCenter) {
    await driver.simulateGpuiEvent({ type: "mouseMove", x: dayCenter.x, y: dayCenter.y }, { timeoutMs: 8000 });
    results.dayHoverReceipt = await pollHoverReceipt(async () => {
      const state = asObj(await driver.getState({ timeoutMs: 8000 }));
      return asObj(state.dayPage).deeplinkHoverHint ?? null;
    });
    results.dayHover = asObj(
      await driver.captureScreenshot({ savePath: join(OUT_DIR, "day-page-hovered.png") }),
    ).error ?? "saved";
    await driver.simulateGpuiEvent({ type: "mouseMove", x: dayCenter.x, y: dayCenter.y + 300 }, { timeoutMs: 8000 });
    await Bun.sleep(400);
    results.dayIdle = asObj(
      await driver.captureScreenshot({ savePath: join(OUT_DIR, "day-page-idle.png") }),
    ).error ?? "saved";
  }

  driver.send({ type: "openNotes" });
  await Bun.sleep(1200);
  const notesTarget: Json = { type: "kind", kind: "notes" };
  await driver.request(
    {
      type: "batch",
      target: notesTarget,
      commands: [{ type: "setInput", text: LINK_MARKDOWN }],
      options: { stopOnError: true, timeout: 8000 },
    },
    { expect: "batchResult", timeoutMs: 9000 },
  );
  await Bun.sleep(400);

  elements = await driver.getElements({ target: notesTarget, limit: 300 }, { timeoutMs: 8000 });
  const notesCenter = linkCenter(elements, "input:notes-editor");
  results.notesCenter = notesCenter;
  if (notesCenter) {
    await driver.simulateGpuiEvent(
      { type: "mouseMove", x: notesCenter.x, y: notesCenter.y },
      { target: notesTarget, timeoutMs: 8000 },
    );
    results.notesHoverReceipt = await pollHoverReceipt(async () => {
      const res = asObj(
        await driver.request(
          { type: "getState", target: notesTarget },
          { expect: "stateResult", timeoutMs: 8000 },
        ),
      );
      return asObj(res.notes).deeplinkHoverHint ?? null;
    });
    results.notesHover = asObj(
      await driver.captureScreenshot({ target: notesTarget, savePath: join(OUT_DIR, "notes-hovered.png") }),
    ).error ?? "saved";
  }
} finally {
  await driver.close().catch(() => {});
}

console.log(JSON.stringify(results, null, 2));
