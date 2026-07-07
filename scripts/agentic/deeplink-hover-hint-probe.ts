#!/usr/bin/env bun
/**
 * Runtime proof for deeplink hover discoverability (Day Page + Notes window):
 * - Link highlight runtime info exposes pixel bounds for each link range.
 * - Moving the mouse over a kit:// link sets the hovered deeplink state
 *   (vendored input hover tracking → hoveredDeeplink href in runtime info).
 * - The hover hint chip ("Click · Preview" + href) is built on the render
 *   path while hovered: the surfaces write a render receipt
 *   (`deeplinkHoverHint: {verb, href}`) only when the chip element is
 *   actually constructed during render, so this also proves the
 *   hover→cx.observe→re-render wiring, not just the hover state.
 * - Moving the mouse off the link clears the hover state and the chip.
 */
import { join, resolve } from "node:path";
import { Driver, type Json } from "../devtools/driver";
import { openDayPage } from "./day-page-open-helper";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const BINARY =
  process.env.PROBE_BINARY ??
  process.env.SCRIPT_KIT_GPUI_BINARY ??
  join(PROJECT_ROOT, "target-agent/artifacts/deeplink-hover/script-kit-gpui");

type Obj = Record<string, any>;

const runId = `deeplink-hover-${Date.now().toString(36)}`;
const LINK_MARKDOWN = "[scripts](kit://scripts)";
const LINK_URI = "kit://scripts";

const receipt: Obj = {
  tool: "deeplink-hover-hint-probe",
  binary: BINARY,
  pass: false,
  failures: [] as string[],
};

function asObj(value: unknown): Obj {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Obj) : {};
}

function check(name: string, ok: boolean, detail: Obj = {}) {
  receipt[name] = { ok, ...detail };
  if (!ok) receipt.failures.push(name);
}

async function pollUntil(
  label: string,
  fn: () => Promise<boolean>,
  timeoutMs = 7000,
): Promise<boolean> {
  const started = Date.now();
  while (Date.now() - started < timeoutMs) {
    if (await fn()) return true;
    await Bun.sleep(100);
  }
  receipt[`timeout_${label}`] = true;
  return false;
}

function walkElements(node: unknown, out: Json[] = []): Json[] {
  if (!node || typeof node !== "object") return out;
  if (Array.isArray(node)) {
    for (const item of node) walkElements(item, out);
    return out;
  }
  const json = node as Obj;
  if (typeof json.semanticId === "string" || typeof json.id === "string") out.push(json);
  for (const value of Object.values(json)) walkElements(value, out);
  return out;
}

function findSemantic(elements: Json, semanticId: string): Obj | null {
  return (
    (walkElements(elements).find((el) => asObj(el).semanticId === semanticId) as Obj) ?? null
  );
}

function linkHighlights(elements: Json, editorSemanticId: string): Obj {
  const editor = findSemantic(elements, editorSemanticId);
  return asObj(asObj(asObj(editor?.style).editorRuntime).markdownLinkHighlightRanges);
}

function uriRangeCenter(highlights: Obj): { x: number; y: number } | null {
  const ranges = Array.isArray(highlights.ranges) ? highlights.ranges.map(asObj) : [];
  const target =
    ranges.find(
      (r) => r.role === "markdownLinkUri" && String(r.text ?? "").includes("kit://"),
    ) ?? ranges.find((r) => asObj(r.bounds).width > 0);
  const bounds = asObj(target?.bounds);
  if (!(bounds.width > 0) || !(bounds.height > 0)) return null;
  return { x: bounds.x + bounds.width / 2, y: bounds.y + bounds.height / 2 };
}

async function mouseMove(driver: Driver, x: number, y: number, target?: Json) {
  return driver.simulateGpuiEvent({ type: "mouseMove", x, y }, { target, timeoutMs: 8000 });
}

async function probeSurface(opts: {
  driver: Driver;
  surface: "dayPage" | "notes";
  target: Json | undefined;
  editorSemanticId: string;
  getElements: () => Promise<Json>;
  getHoverHintReceipt: () => Promise<Obj | null>;
}) {
  const { driver, surface, target, editorSemanticId, getElements, getHoverHintReceipt } = opts;

  const seededBounds = await pollUntil(`${surface}-link-bounds`, async () => {
    const highlights = linkHighlights(await getElements(), editorSemanticId);
    return Number(highlights.count ?? 0) > 0 && uriRangeCenter(highlights) !== null;
  });
  const highlights = linkHighlights(await getElements(), editorSemanticId);
  check(`${surface}_link_bounds_exposed`, seededBounds, { highlights });
  const center = uriRangeCenter(highlights);
  if (!center) return;

  await mouseMove(driver, center.x, center.y, target);
  const hovered = await pollUntil(`${surface}-hovered`, async () => {
    const info = linkHighlights(await getElements(), editorSemanticId);
    return asObj(info.hovered).href === LINK_URI;
  });
  check(`${surface}_hover_sets_href`, hovered, {
    hovered: asObj(linkHighlights(await getElements(), editorSemanticId).hovered),
  });

  // The receipt is written only when the render path builds the chip, so
  // this proves hover→observe→re-render→chip, not just the hover state.
  const chipRendered = await pollUntil(`${surface}-chip-rendered`, async () => {
    const receipt = asObj(await getHoverHintReceipt());
    return receipt.verb === "Preview" && receipt.href === LINK_URI;
  });
  check(`${surface}_hover_hint_chip_rendered`, chipRendered, {
    receipt: await getHoverHintReceipt(),
  });

  // Move well away from the link (same column, far below the first line).
  await mouseMove(driver, center.x, center.y + 300, target);
  const cleared = await pollUntil(`${surface}-hover-cleared`, async () => {
    const info = linkHighlights(await getElements(), editorSemanticId);
    return info.hovered === null || info.hovered === undefined;
  });
  check(`${surface}_hover_clears_off_link`, cleared, {
    hovered: asObj(linkHighlights(await getElements(), editorSemanticId).hovered),
  });

  const chipGone = await pollUntil(`${surface}-chip-cleared`, async () => {
    const receipt = await getHoverHintReceipt();
    return receipt === null || Object.keys(asObj(receipt)).length === 0;
  });
  check(`${surface}_hover_hint_chip_cleared_after_leave`, chipGone, {
    receipt: await getHoverHintReceipt(),
  });
}

const driver = await Driver.launch({
  binary: BINARY,
  sessionName: "deeplink-hover-hint",
  sandboxHome: true,
  defaultTimeoutMs: 8000,
  env: { SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1" },
});

try {
  // --- Day Page ---
  const opened = asObj(await openDayPage(driver, runId));
  check("day_page_opened", opened.promptType === "dayPage", { promptType: opened.promptType });

  const seedDay = asObj(
    await driver.batch([{ type: "setInput", text: LINK_MARKDOWN }], { timeoutMs: 8000 }),
  );
  check("day_link_seeded", seedDay.success === true, { batch: seedDay });

  await probeSurface({
    driver,
    surface: "dayPage",
    target: undefined,
    editorSemanticId: "input:day-page-editor",
    getElements: () =>
      driver.getElements({ target: { type: "main" }, limit: 300 }, { timeoutMs: 8000 }),
    getHoverHintReceipt: async () => {
      const state = asObj(await driver.getState({ timeoutMs: 8000 }));
      const hint = asObj(state.dayPage).deeplinkHoverHint;
      return hint && typeof hint === "object" ? (hint as Obj) : null;
    },
  });

  // --- Notes window ---
  driver.send({ type: "openNotes" });
  const notesOpen = await pollUntil("notes-open", async () => {
    const res = asObj(await driver.listAutomationWindows({ timeoutMs: 8000 }));
    const windows = (res.windows as Json[] | undefined) ?? [];
    return windows.map(asObj).some((window) => String(window.kind) === "notes");
  });
  check("notes_window_opened", notesOpen);

  const notesTarget: Json = { type: "kind", kind: "notes" };
  const seedNotes = asObj(
    await driver.request(
      {
        type: "batch",
        target: notesTarget,
        commands: [{ type: "setInput", text: LINK_MARKDOWN }],
        options: { stopOnError: true, timeout: 8000 },
      },
      { expect: "batchResult", timeoutMs: 9000 },
    ),
  );
  check("notes_link_seeded", seedNotes.success === true, { batch: seedNotes });

  await probeSurface({
    driver,
    surface: "notes",
    target: notesTarget,
    editorSemanticId: "input:notes-editor",
    getElements: () => driver.getElements({ target: notesTarget, limit: 300 }, { timeoutMs: 8000 }),
    getHoverHintReceipt: async () => {
      const res = asObj(
        await driver.request(
          { type: "getState", target: notesTarget },
          { expect: "stateResult", timeoutMs: 8000 },
        ),
      );
      const hint = asObj(res.notes).deeplinkHoverHint;
      return hint && typeof hint === "object" ? (hint as Obj) : null;
    },
  });

  receipt.pass = receipt.failures.length === 0;
} finally {
  await driver.close().catch(() => {});
}

console.log(JSON.stringify(receipt, null, 2));
if (!receipt.pass) process.exit(1);
