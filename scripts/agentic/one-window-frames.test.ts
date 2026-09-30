import { describe, expect, test } from "bun:test";
import {
  actionsForStep, compositeOver, contactSheet, cornerRadiusGuess, decodePngRgba, encodePngRgba, frameStem, leftEdgeBar,
  MAIN_FRAMES_PER_RUNTIME, actionsForStep, measureFrame, modeColor, parseKeySpec, parseSteps, planRuntimes, stepWaitMs, summarize, type RgbaImage,
} from "./one-window-frames.ts";

function solid(width: number, height: number, rgba: [number, number, number, number]): RgbaImage {
  const data = new Uint8Array(width * height * 4);
  for (let i = 0; i < data.length; i += 4) data.set(rgba, i);
  return { width, height, rgba: data };
}
function fill(image: RgbaImage, x0: number, y0: number, w: number, h: number, rgba: [number, number, number, number]) {
  for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) image.rgba.set(rgba, (y * image.width + x) * 4);
}
const paint = (name: string, x: number, y: number, width: number, height: number) => ({
  name, measurementProvenance: "paint-time", bounds: { x, y, width, height }, visibleBounds: { x, y, width, height },
});

describe("steps", () => {
  const spec = [{ name: "empty" }, { name: "typed-sl", type: "sl" }, { name: "sentence", type: "a b", waitMs: 600 },
    { name: "cmdk", key: "cmd-k" }, { name: "esc", key: "escape" }, { name: "down", key: "down" }];

  test("parses the documented step list", () => {
    expect(parseSteps(spec).map(step => step.name)).toEqual(["empty", "typed-sl", "sentence", "cmdk", "esc", "down"]);
  });
  test("rejects duplicates, unknown fields, type+key, bad waits and bad keys", () => {
    expect(() => parseSteps([{ name: "a" }, { name: "a" }])).toThrow();
    expect(() => parseSteps([{ name: "a", text: "x" }])).toThrow();
    expect(() => parseSteps([{ name: "a", type: "x", key: "down" }])).toThrow();
    expect(parseSteps([{ name: "to-note", control: { family: "oneWindow", operation: "talkTo", destination: "note" } }])[0]?.control)
      .toEqual({ family: "oneWindow", operation: "talkTo", destination: "note" });
    expect(parseSteps([{ name: "back", control: { family: "oneWindow", operation: "summon" } }])[0]?.control)
      .toEqual({ family: "oneWindow", operation: "summon" });
    expect(() => parseSteps([{ name: "a", control: { family: "oneWindow", operation: "summon", destination: "note" } }])).toThrow();
    expect(() => parseSteps([{ name: "a", control: { family: "notes", operation: "summon" } }])).toThrow();
    expect(parseSteps([{ name: "front", control: { family: "oneWindow", operation: "focusWindow", app: "Ghostty", title: "zsh" } }])[0]?.control)
      .toEqual({ family: "oneWindow", operation: "focusWindow", app: "Ghostty", title: "zsh" });
    expect(() => parseSteps([{ name: "a", control: { family: "oneWindow", operation: "focusWindow", app: "Ghostty" } }])).toThrow();
    expect(() => parseSteps([{ name: "a", control: { family: "oneWindow", operation: "focusWindow", app: " ", title: "zsh" } }])).toThrow();
    expect(() => parseSteps([{ name: "a", control: { family: "oneWindow", operation: "talkTo", destination: "slack" } }])).toThrow();
    expect(() => parseSteps([{ name: "a", key: "enter", control: { family: "oneWindow", operation: "talkTo", destination: "chat" } }])).toThrow();
    expect(() => parseSteps([{ name: "a", waitMs: -1 }])).toThrow();
    expect(() => parseSteps([{ name: "a", key: "hyper-k" }])).toThrow();
    expect(() => parseSteps([{ name: "Bad Name" }])).toThrow();
    expect(() => parseSteps([])).toThrow();
  });
  test("key specs map to GPUI key events with modifiers", () => {
    expect(parseKeySpec("cmd-k")).toEqual({ key: "k", modifiers: ["cmd"] });
    expect(parseKeySpec("escape")).toEqual({ key: "escape", modifiers: [] });
    expect(() => parseKeySpec("cmd-")).toThrow();
  });
  test("type steps select all, then send one key per character with spaces named", () => {
    expect(actionsForStep({ name: "t", type: "a b" })).toEqual([
      { type: "key", key: "a", modifiers: ["cmd"] },
      { type: "key", key: "a", text: "a" }, { type: "key", key: "space", text: " " }, { type: "key", key: "b", text: "b" },
    ]);
    expect(actionsForStep({ name: "k", key: "down" })).toEqual([{ type: "key", key: "down", modifiers: [] }]);
    expect(actionsForStep({ name: "e" })).toEqual([]);
  });
  test("append steps keep typing after what's there: no select-all, and never combined with type or key", () => {
    expect(actionsForStep({ name: "a", append: "s " })).toEqual([
      { type: "key", key: "s", text: "s" }, { type: "key", key: "space", text: " " },
    ]);
    expect(parseSteps([{ name: "a", append: "x" }])[0]).toEqual({ name: "a", append: "x" });
    expect(() => parseSteps([{ name: "a", append: "x", type: "y" }])).toThrow();
    expect(() => parseSteps([{ name: "a", append: "x", key: "down" }])).toThrow();
    expect(() => parseSteps([{ name: "a", append: "" }])).toThrow();
    expect(parseSteps([{ name: "a", click: "footer-action:actions" }])[0]).toEqual({ name: "a", click: "footer-action:actions" });
    expect(actionsForStep({ name: "a", click: "footer-action:actions" })).toEqual([]);
    expect(() => parseSteps([{ name: "a", click: "x", key: "down" }])).toThrow();
    expect(() => parseSteps([{ name: "a", click: "" }])).toThrow();
    expect(stepWaitMs({ name: "a", append: "x" })).toBe(300);
  });
  test("wait defaults: type steps settle, key steps do not, explicit zero wins", () => {
    expect(stepWaitMs({ name: "t", type: "x" })).toBe(300);
    expect(stepWaitMs({ name: "t", type: "x", waitMs: 0 })).toBe(0);
    expect(stepWaitMs({ name: "k", key: "down" })).toBe(0);
  });
  test("runtime plans stay within the retained-image budget and replay the prefix", () => {
    expect(MAIN_FRAMES_PER_RUNTIME).toBe(6);
    expect(planRuntimes(7)).toEqual([{ replay: 0, capture: [0, 1, 2, 3, 4, 5] }, { replay: 6, capture: [6] }]);
    expect(planRuntimes(3, 6)).toEqual([{ replay: 0, capture: [0, 1, 2] }]);
    expect(frameStem(3, "cmdk")).toBe("03-cmdk");
  });
});

describe("png and compositing", () => {
  test("encode/decode round-trips straight RGBA", () => {
    const image = solid(3, 2, [10, 20, 30, 40]);
    image.rgba.set([255, 0, 0, 255], 4);
    const decoded = decodePngRgba(encodePngRgba(image));
    expect(decoded.width).toBe(3); expect(decoded.height).toBe(2);
    expect([...decoded.rgba]).toEqual([...image.rgba]);
  });
  test("transparent pixels composite over #1c1d22, opaque pixels are unchanged", () => {
    const image = solid(2, 1, [0, 0, 0, 0]); image.rgba.set([200, 100, 50, 255], 4);
    const out = compositeOver(image);
    expect([...out.rgba]).toEqual([0x1c, 0x1d, 0x22, 255, 200, 100, 50, 255]);
  });
});

describe("pixel analyzers", () => {
  test("modeColor ignores sparse glyph pixels", () => {
    const image = solid(10, 10, [50, 60, 70, 255]); fill(image, 4, 4, 1, 2, [255, 255, 255, 255]);
    expect(modeColor(image, 0, 0, 9, 9)).toEqual([50, 60, 70, 255]);
  });
  test("leftEdgeBar detects a 3pt accent bar and not a plain row", () => {
    const scale = 2; const row = { x: 20, y: 10, w: 120, h: 40 };
    const plain = solid(200, 60, [30, 30, 35, 255]); fill(plain, row.x, row.y, row.w, row.h, [60, 60, 70, 255]);
    expect(leftEdgeBar(plain, row, scale).present).toBe(false);
    const barred = solid(200, 60, [30, 30, 35, 255]); fill(barred, row.x, row.y, row.w, row.h, [60, 60, 70, 255]);
    fill(barred, row.x, row.y, 6, row.h, [250, 200, 0, 255]);
    const bar = leftEdgeBar(barred, row, scale);
    expect(bar.present).toBe(true); expect(bar.widthPt).toBe(3);
  });
  test("cornerRadiusGuess is ~0 for a square tile and positive for a rounded one", () => {
    const square = solid(60, 60, [40, 40, 40, 255]); fill(square, 20, 20, 20, 20, [200, 50, 50, 255]);
    expect(cornerRadiusGuess(square, { x: 20, y: 20, w: 20, h: 20 }, 1)).toBe(0);
    const rounded = solid(60, 60, [40, 40, 40, 255]);
    const r = 8;
    for (let y = 0; y < 20; y++) for (let x = 0; x < 20; x++) {
      const cx = Math.min(Math.max(x, r), 19 - r); const cy = Math.min(Math.max(y, r), 19 - r);
      if ((x - cx) ** 2 + (y - cy) ** 2 <= r * r) rounded.rgba.set([200, 50, 50, 255], ((20 + y) * 60 + 20 + x) * 4);
    }
    const guess = cornerRadiusGuess(rounded, { x: 20, y: 20, w: 20, h: 20 }, 1)!;
    expect(guess).toBeGreaterThan(5); expect(guess).toBeLessThan(12);
  });
});

describe("measureFrame", () => {
  const image = solid(200, 160, [0, 0, 0, 0]);
  fill(image, 10, 70, 180, 20, [80, 80, 90, 255]);
  const layout = { windowWidth: 100, windowHeight: 80, components: [
    paint("main-view-context-zone", 8, 4, 84, 10), paint("main-view-input-shell", 8, 16, 84, 14),
    paint("oneWindowSectionHeader:0", 0, 32, 100, 0), paint("oneWindowSectionHeader:3", 0, 50, 100, 8),
    paint("list-row:row-a", 5, 34, 90, 10), paint("list-row:row-b", 5, 35 + 10, 90, 10),
    paint("row-b:selection-icon", 8, 36 + 10, 6, 6), paint("oneWindowRowKeycaps:row-b:2", 80, 36 + 10, 10, 6),
    { name: "MainViewInput", measurementProvenance: "model", bounds: { x: 0, y: 0, width: 1, height: 1 } },
  ] };
  const elements = { elements: [
    { semanticId: "row-a", role: "row", text: "Alpha" }, { semanticId: "row-b", role: "row", text: "Beta", selected: true },
    { semanticId: "main-view-context-cwd-button", text: "~/work" },
  ] };
  const headerElements = { elements: [{ semanticId: "section:0:suggested", role: "sectionHeader", text: "Suggested" },
    { semanticId: "section:3:scripts", role: "sectionHeader", text: "Scripts" }] };

  test("joins paint-time bounds with collector text and skips zero-height headers", () => {
    const m = measureFrame({ layout, elements, headerElements, state: { mainListScroll: { selectedSemanticId: "row-b" } }, image });
    expect(m.scale).toBe(2);
    expect(m.input).toEqual({ x: 8, y: 16, w: 84, h: 14, source: "paint-time:main-view-input-shell" });
    expect(m.aboveInput).toEqual({ y: 4, h: 10, text: "~/work", source: "paint-time:main-view-context-zone" });
    expect(m.firstHeader).toEqual({ y: 50, h: 8, groupedIndex: 3, text: "Scripts" });
    expect(m.firstRow?.title).toBe("Alpha");
    expect(m.selectedRow).toEqual({ y: 45, h: 10, title: "Beta", semanticId: "row-b" });
    expect(m.selectedIcon?.w).toBe(6);
    expect(m.rowKeycaps).toBe(2);
    expect(m.capsules).toBe("native");
    expect(m.windowHeight).toBe(80);
    expect(m.issues).toEqual([]);
  });
  test("reports missing paint bounds as issues instead of guessing", () => {
    const m = measureFrame({ layout: { windowWidth: 100, windowHeight: 80, components: [] }, elements: { elements: [] }, state: {}, image });
    expect(m.input).toBeNull(); expect(m.firstRow).toBeNull(); expect(m.issues).toContain("input_paint_bounds_missing");
  });
  test("summary deltas are relative to step 0", () => {
    const base = measureFrame({ layout, elements, headerElements, state: {}, image });
    const moved = { ...base, input: { ...base.input!, y: base.input!.y + 12 }, firstHeader: null };
    const summary = summarize([{ index: 0, name: "empty", measurement: base }, { index: 1, name: "typed", measurement: moved }, { index: 2, name: "lost", measurement: null, error: "timeout" }]);
    expect(summary[0]).toMatchObject({ inputDy: 0, firstRowDy: 0, firstHeaderPresent: true });
    expect(summary[1]).toMatchObject({ inputDy: 12, firstHeaderPresent: false });
    expect(summary[2]).toMatchObject({ error: "timeout", inputDy: null, firstHeaderPresent: null });
  });
});

test("contact sheet lays frames out in a labelled grid", () => {
  const frames = [0, 1, 2, 3].map(i => ({ label: `0${i} step`, image: solid(40, 20, [i * 60, 0, 0, 255]) }));
  const sheet = contactSheet(frames, { columns: 3, factor: 2 });
  expect(sheet.width).toBe(3 * 20 + 4 * 16);
  expect(sheet.height).toBe(2 * (10 + 24) + 3 * 16);
  expect(decodePngRgba(encodePngRgba(sheet)).width).toBe(sheet.width);
});
