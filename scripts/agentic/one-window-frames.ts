#!/usr/bin/env bun
// One-window frames harness: mounts the production main window (ScriptList) in the
// owned evaluator (hidden Metal windows, GPU readback, owned GPUI key events only),
// applies a step list, and writes per-step PNG/RGBA frames plus paint-bound measurements.
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync, lstatSync } from "node:fs";
import { join, resolve } from "node:path";
import { deflateSync } from "node:zlib";
import { claimOutput, validateOutputTarget, beginManagedTask, finalizeManagedTask, type OwnedCleanup } from "./artifact-lifecycle.ts";
import { type ArtifactReference } from "./build-artifact.ts";
import { DriverCommandRefused, DriverLifecycleError, type Json } from "../devtools/driver.ts";
import { OwnedEvaluationClient, EvaluationContractError, type OwnedAction, type OwnedFrameCapture } from "../devtools/lib/owned-evaluation.ts";
import { OWNED_EVALUATION_LIMITS } from "../devtools/lib/operator-safety.ts";
import { type AutomationInstance, type AutomationTargetSnapshot } from "../devtools/lib/target-identity.ts";
import { aggregateCleanup } from "../devtools/lib/story-contract.ts";
import { visitPngRgbaRows } from "../devtools/lib/png-rgba.ts";

const REPOSITORY_ROOT = resolve(import.meta.dir, "../..");
export const MAIN_FIXTURE_ID = "main.script-list";
export const BACKGROUND_RGB = [0x1c, 0x1d, 0x22] as const;
export const DEFAULT_TYPE_WAIT_MS = 300;
/** Main-window frames per runtime; the remaining retained-image slots go to popups opened by steps. */
export const MAIN_FRAMES_PER_RUNTIME = OWNED_EVALUATION_LIMITS.maxRetainedImages - 2;
const NATURAL_FRAME_TIMEOUT_MS = 5000;
const MAX_WAIT_MS = 10_000;

// ---------------------------------------------------------------- steps

/** A voice session to a destination, started the way "Start Dictation to Agent Chat/Notes" starts one. */
export interface TalkToControl { family: "oneWindow"; operation: "talkTo"; destination: "chat" | "note" }
/** The window coming back the way the main hotkey brings it, after a launch or a voice session let it go. */
export interface SummonControl { family: "oneWindow"; operation: "summon" }
export type OneWindowControl = TalkToControl | SummonControl;
/** `click` names a painted node (its debug selector); the step clicks its center with an owned GPUI mouse event. */
export interface Step { name: string; type?: string; append?: string; key?: string; click?: string; control?: OneWindowControl; waitMs?: number }
const MODIFIERS = new Set(["cmd", "ctrl", "alt", "shift"]);
const NAMED_KEYS = new Set(["escape", "enter", "down", "up", "left", "right", "tab", "backspace", "space", "home", "end", "pageup", "pagedown"]);

export function parseKeySpec(spec: string): { key: string; modifiers: string[] } {
  const parts = spec.split("-");
  const key = parts.pop()!;
  const modifiers = parts;
  if (!key || modifiers.some(modifier => !MODIFIERS.has(modifier)) || new Set(modifiers).size !== modifiers.length ||
      !(NAMED_KEYS.has(key) || Array.from(key).length === 1)) throw new EvaluationContractError("invalid_key_spec", [spec]);
  return { key, modifiers };
}

export function parseSteps(value: unknown): Step[] {
  if (!Array.isArray(value) || value.length === 0 || value.length > 64) throw new EvaluationContractError("steps_must_be_nonempty_array");
  const names = new Set<string>();
  return value.map((raw, index) => {
    if (!raw || typeof raw !== "object" || Array.isArray(raw)) throw new EvaluationContractError("invalid_step", [String(index)]);
    const step = raw as Record<string, unknown>;
    const unknown = Object.keys(step).filter(key => !["name", "type", "append", "key", "click", "control", "waitMs"].includes(key));
    if (unknown.length) throw new EvaluationContractError("unknown_step_field", unknown);
    if (typeof step.name !== "string" || !/^[a-z0-9][a-z0-9-]{0,47}$/.test(step.name) || names.has(step.name))
      throw new EvaluationContractError("invalid_or_duplicate_step_name", [String(step.name)]);
    names.add(step.name);
    if ([step.type, step.append, step.key, step.click, step.control].filter(value => value !== undefined).length > 1)
      throw new EvaluationContractError("step_has_type_and_key", [step.name]);
    if (step.control !== undefined) {
      const control = step.control as Record<string, unknown> | null;
      const keys = control && typeof control === "object" ? Object.keys(control).sort().join(",") : "";
      const talkTo = keys === "destination,family,operation" && control!.operation === "talkTo" &&
        ["chat", "note"].includes(control!.destination as string);
      const summon = keys === "family,operation" && control!.operation === "summon";
      if (control?.family !== "oneWindow" || !(talkTo || summon))
        throw new EvaluationContractError("invalid_step_control", [step.name]);
    }
    for (const text of [step.type, step.append])
      if (text !== undefined && (typeof text !== "string" || !text.length || text.length > 256 || /[\n\r\t]/.test(text)))
        throw new EvaluationContractError("invalid_step_type_text", [step.name]);
    if (step.key !== undefined) { if (typeof step.key !== "string") throw new EvaluationContractError("invalid_key_spec", [step.name]); parseKeySpec(step.key); }
    if (step.click !== undefined && (typeof step.click !== "string" || !step.click.length || step.click.length > 128))
      throw new EvaluationContractError("invalid_click_target", [step.name]);
    if (step.waitMs !== undefined && (!Number.isSafeInteger(step.waitMs) || (step.waitMs as number) < 0 || (step.waitMs as number) > MAX_WAIT_MS))
      throw new EvaluationContractError("invalid_wait_ms", [step.name]);
    return { name: step.name, ...(step.type !== undefined ? { type: step.type as string } : {}),
      ...(step.append !== undefined ? { append: step.append as string } : {}),
      ...(step.key !== undefined ? { key: step.key as string } : {}), ...(step.click !== undefined ? { click: step.click as string } : {}),
      ...(step.control !== undefined ? { control: step.control as OneWindowControl } : {}),
      ...(step.waitMs !== undefined ? { waitMs: step.waitMs as number } : {}) };
  });
}

export function stepWaitMs(step: Step): number {
  return step.waitMs ?? (step.type !== undefined || step.append !== undefined ? DEFAULT_TYPE_WAIT_MS : 0);
}

const characterKeys = (text: string): OwnedAction[] =>
  Array.from(text).map(character => ({ type: "key" as const, key: character === " " ? "space" : character, text: character }));

/** A type step replaces the filter: select all, then one GPUI key event per character. An append step types
 * after what's already there, the way someone keeps typing a sentence. */
export function actionsForStep(step: Step): OwnedAction[] {
  if (step.type !== undefined) return [{ type: "key", key: "a", modifiers: ["cmd"] }, ...characterKeys(step.type)];
  if (step.append !== undefined) return characterKeys(step.append);
  if (step.key !== undefined) { const { key, modifiers } = parseKeySpec(step.key); return [{ type: "key", key, modifiers }]; }
  return [];
}

export interface RuntimePlan { replay: number; capture: number[] }
export function planRuntimes(stepCount: number, framesPerRuntime = MAIN_FRAMES_PER_RUNTIME): RuntimePlan[] {
  if (!Number.isSafeInteger(framesPerRuntime) || framesPerRuntime < 1) throw new EvaluationContractError("invalid_frames_per_runtime");
  const plans: RuntimePlan[] = [];
  for (let start = 0; start < stepCount; start += framesPerRuntime)
    plans.push({ replay: start, capture: Array.from({ length: Math.min(framesPerRuntime, stepCount - start) }, (_, i) => start + i) });
  return plans;
}
export function frameStem(index: number, name: string): string { return `${String(index).padStart(2, "0")}-${name}`; }

// ---------------------------------------------------------------- PNG

const CRC_TABLE = Uint32Array.from({ length: 256 }, (_, n) => {
  let c = n;
  for (let k = 0; k < 8; k++) c = c & 1 ? 0xedb88320 ^ (c >>> 1) : c >>> 1;
  return c >>> 0;
});
function crc32(bytes: Uint8Array): number {
  let crc = 0xffffffff;
  for (const byte of bytes) crc = CRC_TABLE[(crc ^ byte) & 0xff]! ^ (crc >>> 8);
  return (crc ^ 0xffffffff) >>> 0;
}
export interface RgbaImage { width: number; height: number; rgba: Uint8Array }

export function encodePngRgba(image: RgbaImage): Uint8Array {
  const { width, height, rgba } = image;
  if (rgba.length !== width * height * 4) throw new EvaluationContractError("rgba_length_mismatch");
  const raw = new Uint8Array(height * (width * 4 + 1));
  for (let y = 0; y < height; y++) raw.set(rgba.subarray(y * width * 4, (y + 1) * width * 4), y * (width * 4 + 1) + 1);
  const chunk = (type: string, data: Uint8Array) => {
    const out = new Uint8Array(12 + data.length);
    const view = new DataView(out.buffer);
    view.setUint32(0, data.length);
    out.set(Buffer.from(type, "latin1"), 4);
    out.set(data, 8);
    view.setUint32(8 + data.length, crc32(out.subarray(4, 8 + data.length)));
    return out;
  };
  const header = new Uint8Array(13);
  const headerView = new DataView(header.buffer);
  headerView.setUint32(0, width); headerView.setUint32(4, height);
  header.set([8, 6, 0, 0, 0], 8);
  return Buffer.concat([Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]), chunk("IHDR", header), chunk("IDAT", deflateSync(raw)), chunk("IEND", new Uint8Array())]);
}

export function decodePngRgba(bytes: Uint8Array): RgbaImage {
  if (bytes.length < 24) throw new EvaluationContractError("png_truncated");
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const declared = { width: view.getUint32(16), height: view.getUint32(20) };
  if (declared.width * declared.height > OWNED_EVALUATION_LIMITS.maxImagePixels) throw new EvaluationContractError("png_too_large");
  const rgba = new Uint8Array(declared.width * declared.height * 4);
  visitPngRgbaRows(bytes, (row, y) => rgba.set(row, y * declared.width * 4), declared);
  return { ...declared, rgba };
}

/** Straight-alpha source over an opaque background. */
export function compositeOver(image: RgbaImage, background: readonly [number, number, number] = BACKGROUND_RGB): RgbaImage {
  const out = new Uint8Array(image.rgba.length);
  for (let i = 0; i < image.rgba.length; i += 4) {
    const alpha = image.rgba[i + 3]! / 255;
    for (let c = 0; c < 3; c++) out[i + c] = Math.round(image.rgba[i + c]! * alpha + background[c]! * (1 - alpha));
    out[i + 3] = 255;
  }
  return { width: image.width, height: image.height, rgba: out };
}

// ---------------------------------------------------------------- measurements

export interface Rect { x: number; y: number; w: number; h: number }
type Rgba = [number, number, number, number];
interface PaintNode { name: string; bounds: Rect; visible: Rect | null }

function rect(value: Json | undefined): Rect | null {
  if (!value || ![value.x, value.y, value.width, value.height].every(n => typeof n === "number" && Number.isFinite(n))) return null;
  return { x: value.x, y: value.y, w: value.width, h: value.height };
}
function round(rect: Rect): Rect { return { x: +rect.x.toFixed(2), y: +rect.y.toFixed(2), w: +rect.w.toFixed(2), h: +rect.h.toFixed(2) }; }
export function paintNodes(layout: Json): PaintNode[] {
  return (Array.isArray(layout?.components) ? layout.components : [])
    .filter((node: Json) => node?.measurementProvenance === "paint-time" && typeof node.name === "string" && rect(node.bounds))
    .map((node: Json) => ({ name: node.name, bounds: rect(node.bounds)!, visible: rect(node.visibleBounds) }));
}
function isVisible(node: PaintNode): boolean { return (node.visible?.h ?? node.bounds.h) > 0.5 && (node.visible?.w ?? node.bounds.w) > 0.5; }
function elementText(element: Json | undefined): string | null {
  const text = element?.text ?? element?.label ?? element?.name;
  return typeof text === "string" && text.length ? text : null;
}
/** Collector rows whose text is withheld (private captures) are labelled by kind, marked as such. */
function rowTitle(element: Json | undefined): string | null {
  return elementText(element) ?? (typeof element?.kind === "string" ? `[${element.kind}; text withheld by collector]` : null);
}

export function pixelAt(image: RgbaImage, x: number, y: number): Rgba {
  const px = Math.min(image.width - 1, Math.max(0, Math.round(x))); const py = Math.min(image.height - 1, Math.max(0, Math.round(y)));
  const i = (py * image.width + px) * 4;
  return [image.rgba[i]!, image.rgba[i + 1]!, image.rgba[i + 2]!, image.rgba[i + 3]!];
}
function luma(rgb: readonly number[]): number { return (0.2126 * rgb[0]! + 0.7152 * rgb[1]! + 0.0722 * rgb[2]!) / 255; }
function composite(p: Rgba): [number, number, number] {
  const a = p[3] / 255; return [0, 1, 2].map(c => Math.round(p[c]! * a + BACKGROUND_RGB[c]! * (1 - a))) as [number, number, number];
}
function distance(a: readonly number[], b: readonly number[]): number { return Math.max(...[0, 1, 2, 3].map(c => Math.abs((a[c] ?? 255) - (b[c] ?? 255)))); }

/** Most frequent colour (4-level quantised buckets, exact representative) in a device-pixel strip. */
export function modeColor(image: RgbaImage, x0: number, y0: number, x1: number, y1: number): Rgba | null {
  const buckets = new Map<string, { count: number; sample: Rgba }>();
  for (let y = Math.max(0, Math.round(y0)); y <= Math.min(image.height - 1, Math.round(y1)); y++)
    for (let x = Math.max(0, Math.round(x0)); x <= Math.min(image.width - 1, Math.round(x1)); x++) {
      const p = pixelAt(image, x, y); const key = p.map(v => v >> 2).join(",");
      const entry = buckets.get(key); if (entry) entry.count++; else buckets.set(key, { count: 1, sample: p });
    }
  let best: { count: number; sample: Rgba } | undefined;
  for (const entry of buckets.values()) if (!best || entry.count > best.count) best = entry;
  return best?.sample ?? null;
}

/** Diagonal walk from the top-left corner until the icon leaves the surrounding row colour; r = d / (1 - 1/sqrt 2). */
export function cornerRadiusGuess(image: RgbaImage, device: Rect, scale: number): number | null {
  const outside = modeColor(image, device.x - 4 * scale, device.y + device.h * 0.3, device.x - 2 * scale, device.y + device.h * 0.7);
  if (!outside) return null;
  const limit = Math.floor(Math.min(device.w, device.h) / 2);
  for (let d = 0; d < limit; d++) {
    if (distance(pixelAt(image, device.x + d, device.y + d), outside) > 24) return d === 0 ? 0 : +((d / (1 - Math.SQRT1_2)) / scale).toFixed(1);
  }
  return null;
}

export interface EdgeBar { present: boolean; widthPx: number; widthPt: number; xPt: number | null }
/** A left-edge bar: a run of device columns near the row's left edge, uniform down the row and contrasting with both sides. */
export function leftEdgeBar(image: RgbaImage, device: Rect, scale: number): EdgeBar {
  const top = device.y + device.h * 0.2; const bottom = device.y + device.h * 0.8;
  const samples = (x: number) => { const values: number[] = []; for (let y = Math.round(top); y <= Math.round(bottom); y++) values.push(luma(composite(pixelAt(image, x, y)))); return values; };
  const median = (values: number[]) => [...values].sort((a, b) => a - b)[Math.floor(values.length / 2)]!;
  const outside = median(samples(Math.max(0, device.x - 14 * scale)));
  const inside = median(samples(device.x + 18 * scale));
  let run = 0; let best = 0; let bestStart = -1; let start = -1;
  for (let x = Math.max(0, Math.round(device.x - 8 * scale)); x <= Math.min(image.width - 1, Math.round(device.x + 6 * scale)); x++) {
    const values = samples(x); const m = median(values);
    const uniform = values.filter(v => Math.abs(v - m) <= 0.05).length >= values.length * 0.8;
    const bar = uniform && Math.abs(m - outside) >= 0.1 && Math.abs(m - inside) >= 0.1;
    if (bar) { if (!run) start = x; run++; if (run > best) { best = run; bestStart = start; } } else run = 0;
  }
  return { present: best >= 2 * scale, widthPx: best, widthPt: +(best / scale).toFixed(2), xPt: bestStart < 0 ? null : +(bestStart / scale).toFixed(2) };
}

export interface FrameInputs {
  layout: Json; elements: Json; headerElements?: Json; state: Json; image: RgbaImage;
}
export interface StepMeasurement {
  windowHeight: number | null; windowWidth: number | null; scale: number | null;
  input: (Rect & { source: string }) | null;
  aboveInput: { y: number; h: number; text: string | null; source: string } | null;
  aboveInputAll: { name: string; y: number; h: number; text: string | null }[];
  firstHeader: { y: number; h: number; text: string | null; groupedIndex: number } | null;
  firstRow: { y: number; h: number; title: string | null; semanticId: string } | null;
  selectedRow: { y: number; h: number; title: string | null; semanticId: string } | null;
  selectedIcon: (Rect & { centerRgba: Rgba; cornerRadiusGuess: number | null }) | null;
  selectedRowFill: { rgba: Rgba; composited: [number, number, number]; atPt: { x: number; y0: number; y1: number } } | null;
  leftEdgeBar: boolean | null; leftEdgeBarDetail: EdgeBar | null;
  rowKeycaps: number; rowKeycapRows: number;
  capsules: { x: number; y: number; w: number; h: number; label: string }[] | "native";
  footerSemantics: string[];
  visibleRows: number;
  issues: string[];
}

export function measureFrame(inputs: FrameInputs): StepMeasurement {
  const { layout, state, image } = inputs;
  const issues: string[] = [];
  const nodes = paintNodes(layout);
  const elements: Json[] = Array.isArray(inputs.elements?.elements) ? inputs.elements.elements : [];
  const byId = new Map(elements.map(element => [element.semanticId ?? element.id, element]));
  const windowWidth = typeof layout?.windowWidth === "number" ? layout.windowWidth : null;
  const windowHeight = typeof layout?.windowHeight === "number" ? layout.windowHeight : null;
  const scale = windowWidth ? image.width / windowWidth : null;
  if (!scale) issues.push("window_width_missing");
  const named = (name: string) => nodes.find(node => node.name === name && isVisible(node));
  const toDevice = (r: Rect): Rect => ({ x: r.x * scale!, y: r.y * scale!, w: r.w * scale!, h: r.h * scale! });

  const inputNode = named("main-view-input-shell");
  const modelInput = rect((layout?.components ?? []).find((node: Json) => node.name === "MainViewInput")?.bounds);
  const input = inputNode ? { ...round(inputNode.bounds), source: "paint-time:main-view-input-shell" } :
    modelInput ? { ...round(modelInput), source: "model:MainViewInput" } : null;
  if (!inputNode) issues.push("input_paint_bounds_missing");

  const containers = /^(main-view-shell|main-view-header|main-content-stage|main-view-main|main-view-overlay-clip|main-window-.*-boundary)$/;
  const aboveInputAll = input ? nodes.filter(node => isVisible(node) && !containers.test(node.name) && node.bounds.y + node.bounds.h <= input.y + 0.5)
    .sort((a, b) => a.bounds.y - b.bounds.y)
    .map(node => ({ name: node.name, y: +node.bounds.y.toFixed(2), h: +node.bounds.h.toFixed(2), text: elementText(byId.get(node.name)) })) : [];
  let aboveInput: StepMeasurement["aboveInput"] = null;
  const context = named("main-view-context-zone");
  if (context && input && context.bounds.y + context.bounds.h <= input.y + 0.5) {
    const texts = elements.filter(element => /context|cwd|model|quick-ai|selection-button/i.test(String(element.semanticId ?? element.id)))
      .map(elementText).filter((text): text is string => !!text);
    aboveInput = { y: +context.bounds.y.toFixed(2), h: +context.bounds.h.toFixed(2), text: texts.length ? texts.join(" | ") : null, source: "paint-time:main-view-context-zone" };
  } else if (input) {
    const above = nodes.filter(node => isVisible(node) && node.bounds.y + node.bounds.h <= input.y + 0.5 && !containers.test(node.name));
    if (above.length) {
      const y = Math.min(...above.map(node => node.bounds.y)); const bottom = Math.max(...above.map(node => node.bounds.y + node.bounds.h));
      aboveInput = { y: +y.toFixed(2), h: +(bottom - y).toFixed(2), text: null, source: `paint-time:${above.map(node => node.name).join(",")}` };
    }
  }

  const headerElements: Json[] = Array.isArray(inputs.headerElements?.elements) ? inputs.headerElements.elements : [];
  const headers = nodes.filter(node => /^oneWindowSectionHeader:\d+$/.test(node.name))
    .map(node => ({ node, index: Number(node.name.split(":")[1]) })).sort((a, b) => a.index - b.index);
  const firstPaintedHeader = headers.find(header => isVisible(header.node) && header.node.bounds.h > 0.5);
  const firstHeader = firstPaintedHeader ? {
    y: +firstPaintedHeader.node.bounds.y.toFixed(2), h: +firstPaintedHeader.node.bounds.h.toFixed(2), groupedIndex: firstPaintedHeader.index,
    text: elementText(headerElements.find(element => element.role === "sectionHeader" &&
      new RegExp(`^section[:_-]${firstPaintedHeader.index}([:_-]|$)`).test(String(element.semanticId ?? "")))) ??
      elementText(headerElements.filter(element => element.role === "sectionHeader")[headers.filter(h => h.node.bounds.h > 0.5).indexOf(firstPaintedHeader)]),
  } : null;

  const rows = nodes.filter(node => node.name.startsWith("list-row:") && isVisible(node)).sort((a, b) => a.bounds.y - b.bounds.y);
  const rowInfo = (node: PaintNode) => { const id = node.name.slice("list-row:".length);
    return { y: +node.bounds.y.toFixed(2), h: +node.bounds.h.toFixed(2), title: rowTitle(byId.get(id)), semanticId: id }; };
  const firstRow = rows[0] ? rowInfo(rows[0]) : null;
  const selectedId: string | undefined = state?.mainListScroll?.selectedSemanticId ??
    elements.find(element => element.selected === true && (element.role === "row" || element.elementType === "choice"))?.semanticId;
  const selectedNode = selectedId ? rows.find(node => node.name === `list-row:${selectedId}`) : undefined;
  const selectedRow = selectedNode ? rowInfo(selectedNode) : null;
  if (selectedId && !selectedNode) issues.push("selected_row_not_painted");

  let selectedIcon: StepMeasurement["selectedIcon"] = null;
  let selectedRowFill: StepMeasurement["selectedRowFill"] = null;
  let bar: EdgeBar | null = null;
  if (selectedNode && scale) {
    const iconNode = named(`${selectedId}:selection-icon`);
    if (iconNode) {
      const device = toDevice(iconNode.bounds);
      selectedIcon = { ...round(iconNode.bounds), centerRgba: pixelAt(image, device.x + device.w / 2, device.y + device.h / 2), cornerRadiusGuess: cornerRadiusGuess(image, device, scale) };
    } else issues.push("selected_icon_paint_bounds_missing");
    const row = selectedNode.bounds; const x = row.x + row.w * 0.6;
    const fill = modeColor(image, (x - 2) * scale, (row.y + row.h * 0.15) * scale, (x + 2) * scale, (row.y + row.h * 0.85) * scale);
    if (fill) selectedRowFill = { rgba: fill, composited: composite(fill), atPt: { x: +x.toFixed(2), y0: +(row.y + row.h * 0.15).toFixed(2), y1: +(row.y + row.h * 0.85).toFixed(2) } };
    bar = leftEdgeBar(image, toDevice(row), scale);
  }

  const keycaps = nodes.filter(node => node.name.startsWith("oneWindowRowKeycaps:") && isVisible(node));
  const rowKeycaps = keycaps.reduce((sum, node) => sum + (Number(node.name.split(":").at(-1)) || 0), 0);

  const footerNodes = nodes.filter(node => isVisible(node) && /^(hint-|footer-|gpui-footer|agent-chat\.footer-overlay)/.test(node.name));
  const capsules = footerNodes.length ? footerNodes.map(node => ({ ...round(node.bounds), label: node.name })) : "native" as const;
  const footerSemantics = elements.filter(element => /^footer[-_:]/.test(String(element.semanticId ?? "")))
    .map(element => `${element.semanticId}${elementText(element) ? `=${elementText(element)}` : ""}`);

  return { windowHeight, windowWidth, scale, input, aboveInput, aboveInputAll, firstHeader, firstRow, selectedRow, selectedIcon, selectedRowFill,
    leftEdgeBar: bar ? bar.present : null, leftEdgeBarDetail: bar, rowKeycaps, rowKeycapRows: keycaps.length, capsules, footerSemantics,
    visibleRows: rows.length, issues };
}

export interface SummaryStep { index: number; name: string; measurement: StepMeasurement | null; error?: string }
export function summarize(steps: readonly SummaryStep[]) {
  const base = steps[0]?.measurement ?? null;
  const delta = (a: number | undefined | null, b: number | undefined | null) => typeof a === "number" && typeof b === "number" ? +(a - b).toFixed(2) : null;
  return steps.map(step => ({
    index: step.index, name: step.name, ...(step.error ? { error: step.error } : {}),
    ...(step.measurement ?? {}),
    inputDy: delta(step.measurement?.input?.y, base?.input?.y),
    firstRowDy: delta(step.measurement?.firstRow?.y, base?.firstRow?.y),
    firstHeaderPresent: step.measurement ? step.measurement.firstHeader !== null : null,
  }));
}

// ---------------------------------------------------------------- contact sheet

// 5x7 glyphs, one hex digit pair per row (low 5 bits).
const GLYPHS: Record<string, string> = {
  A: "0e11111f111111", B: "1e11111e11111e", C: "0e11101010110e", D: "1e11111111111e", E: "1f10101e10101f", F: "1f10101e101010",
  G: "0e11101711110f", H: "1111111f111111", I: "0e04040404040e", J: "0702020202120c", K: "11121418141211",
  L: "1010101010101f", M: "111b1515111111", N: "11111915131111", O: "0e11111111110e", P: "1e11111e101010", Q: "0e11111115120d",
  R: "1e11111e141211", S: "0f10100e01011e", T: "1f040404040404", U: "1111111111110e", V: "11111111110a04", W: "1111111515150a",
  X: "11110a040a1111", Y: "1111110a040404", Z: "1f01020408101f", "0": "0e11131519110e", "1": "040c040404040e",
  "2": "0e11010204081f", "3": "1f02040201110e", "4": "02060a121f0202", "5": "1f101e0101110e", "6": "0608101e11110e",
  "7": "1f010204080808", "8": "0e11110e11110e", "9": "0e11110f01020c", "-": "0000001f000000", _: "0000000000001f",
  ".": "00000000000c0c", ":": "000c0c000c0c00", " ": "00000000000000", "(": "02040808080402", ")": "08040202020408",
  "/": "01010204081010", "?": "0e110102040004",
};
export function drawText(image: RgbaImage, text: string, x0: number, y0: number, scale: number, rgb: readonly [number, number, number]): void {
  let x = x0;
  for (const raw of text.toUpperCase()) {
    const glyph = GLYPHS[raw] ?? GLYPHS["?"]!;
    for (let row = 0; row < 7; row++) {
      const bits = parseInt(glyph.slice(row * 2, row * 2 + 2), 16);
      for (let col = 0; col < 5; col++) if (bits & (1 << (4 - col)))
        for (let dy = 0; dy < scale; dy++) for (let dx = 0; dx < scale; dx++) {
          const px = x + col * scale + dx; const py = y0 + row * scale + dy;
          if (px < 0 || py < 0 || px >= image.width || py >= image.height) continue;
          const i = (py * image.width + px) * 4; image.rgba.set([rgb[0], rgb[1], rgb[2], 255], i);
        }
    }
    x += 6 * scale;
  }
}
function downsample(image: RgbaImage, factor: number): RgbaImage {
  const width = Math.floor(image.width / factor); const height = Math.floor(image.height / factor);
  const rgba = new Uint8Array(width * height * 4);
  for (let y = 0; y < height; y++) for (let x = 0; x < width; x++) {
    const sum = [0, 0, 0, 0];
    for (let dy = 0; dy < factor; dy++) for (let dx = 0; dx < factor; dx++) {
      const i = ((y * factor + dy) * image.width + x * factor + dx) * 4;
      for (let c = 0; c < 4; c++) sum[c]! += image.rgba[i + c]!;
    }
    for (let c = 0; c < 4; c++) rgba[(y * width + x) * 4 + c] = Math.round(sum[c]! / (factor * factor));
  }
  return { width, height, rgba };
}
export function contactSheet(frames: readonly { label: string; image: RgbaImage }[], options: { columns?: number; factor?: number } = {}): RgbaImage {
  if (!frames.length) throw new EvaluationContractError("contact_sheet_requires_frames");
  const factor = options.factor ?? 2; const columns = Math.min(options.columns ?? 3, frames.length);
  const thumbs = frames.map(frame => ({ label: frame.label, image: factor > 1 ? downsample(frame.image, factor) : frame.image }));
  const cellW = Math.max(...thumbs.map(t => t.image.width)); const cellH = Math.max(...thumbs.map(t => t.image.height));
  const pad = 16; const labelH = 24; const rows = Math.ceil(thumbs.length / columns);
  const width = columns * cellW + (columns + 1) * pad; const height = rows * (cellH + labelH) + (rows + 1) * pad;
  const sheet: RgbaImage = { width, height, rgba: new Uint8Array(width * height * 4) };
  for (let i = 0; i < sheet.rgba.length; i += 4) sheet.rgba.set([0x10, 0x10, 0x14, 255], i);
  thumbs.forEach((thumb, index) => {
    const x0 = pad + (index % columns) * (cellW + pad); const y0 = pad + Math.floor(index / columns) * (cellH + labelH + pad);
    drawText(sheet, thumb.label, x0, y0 + 3, 2, [0xe8, 0xe8, 0xee]);
    for (let y = 0; y < thumb.image.height; y++)
      sheet.rgba.set(thumb.image.rgba.subarray(y * thumb.image.width * 4, (y + 1) * thumb.image.width * 4), ((y0 + labelH + y) * width + x0) * 4);
  });
  return sheet;
}

// ---------------------------------------------------------------- runtime

interface StepRecord {
  index: number; name: string; step: Step; runtime: number; captured: boolean;
  effects: Json[]; dispatchTarget: string; naturalFrame: Json; popups: Json[];
  frame?: Json; measurement?: StepMeasurement; inputValue?: string | null; files: string[]; error?: string;
}
const sleep = (ms: number) => new Promise(done => setTimeout(done, ms));
const MAX_STALE_RETRIES = 25;

/**
 * Real-time providers can publish between the pre-action inspect and the dispatch; the runtime then
 * refuses the key as stale before dispatch (dispatchCompleted false). Retry only when the observable
 * filter proves the key was not applied (text keys), and for other keys rely on that pre-dispatch refusal.
 */
async function actWithRetry(client: OwnedEvaluationClient, target: AutomationInstance, action: OwnedAction): Promise<{ response: Json; retries: number }> {
  const textKey = action.type === "key" && typeof action.text === "string" && !(action.modifiers ?? []).length;
  for (let retries = 0; ; retries++) {
    const before = textKey ? (await client.inspect(target)).inputValue : undefined;
    try { return { response: await client.act(target, action), retries }; }
    catch (error) {
      const refusedBeforeDispatch = (error instanceof EvaluationContractError && error.code === "completed_action_receipt_required") ||
        (error instanceof DriverCommandRefused && error.code === "stale_target_identity");
      if (!refusedBeforeDispatch || retries >= MAX_STALE_RETRIES) throw error;
      if (textKey && (await client.inspect(target)).inputValue !== before) throw new EvaluationContractError("refused_key_changed_input", [String(action.type === "key" ? action.key : "")]);
      await sleep(20);
    }
  }
}
/** An owned mouse click at the center of the painted node `name` in the frame the main window shows now. */
async function clickAction(client: OwnedEvaluationClient, main: AutomationInstance, name: string): Promise<OwnedAction> {
  const probe = await client.captureFrame(main, false);
  const node = paintNodes(probe.layout).find(candidate => candidate.name === name && isVisible(candidate));
  if (!node) throw new EvaluationContractError("click_target_not_painted", [name]);
  return { type: "gpuiEvent", frame: probe.frame, event: { type: "mouseClick", button: "left", x: node.bounds.x + node.bounds.w / 2, y: node.bounds.y + node.bounds.h / 2 } };
}
const instanceKey = (target: AutomationInstance | AutomationTargetSnapshot) =>
  "windowId" in target ? `${target.windowId}:${target.windowGeneration}` : `${target.id}:${target.generation}`;

function writeCaptureFiles(out: string, stem: string, capture: OwnedFrameCapture): { image: RgbaImage; files: string[] } {
  const png = capture.snapshot.capture?.pngBase64;
  if (typeof png !== "string" || !png.length) throw new EvaluationContractError("frame_image_missing", [stem]);
  const rawBytes = Buffer.from(png, "base64");
  const image = decodePngRgba(rawBytes);
  const files = [`${stem}.png`, `${stem}.rgba.png`];
  writeFileSync(join(out, `${stem}.rgba.png`), rawBytes);
  writeFileSync(join(out, `${stem}.png`), encodePngRgba(compositeOver(image)));
  return { image, files };
}

async function runRuntime(options: {
  runtime: number; plan: RuntimePlan; steps: Step[]; reference: ArtifactReference; claim: ReturnType<typeof claimOutput>;
  out: string; records: StepRecord[]; frames: { label: string; image: RgbaImage }[]; fixture: string; nativeGlass: NativeGlass;
}): Promise<OwnedCleanup> {
  const { runtime, plan, steps, reference, claim, out, records, frames, fixture, nativeGlass } = options;
  let client: OwnedEvaluationClient | undefined;
  let lifecycleCleanup: OwnedCleanup | undefined;
  let images = 0;
  try {
    client = await OwnedEvaluationClient.launch(REPOSITORY_ROOT, reference, claim, [fixture], undefined, { nativeGlass });
    const catalog = await client.discover();
    if (!catalog.fixtures.some(entry => entry.id === fixture)) throw new EvaluationContractError("main_fixture_unavailable", [fixture]);
    const main = await client.mount(fixture);
    await client.frame(main);
    const openPopups: AutomationInstance[] = [];
    const last = plan.capture.at(-1)!;
    for (let index = 0; index <= last; index++) {
      const step = steps[index]!; const capturing = plan.capture.includes(index);
      const record: StepRecord = { index, name: step.name, step, runtime, captured: false, effects: [], dispatchTarget: "none", naturalFrame: null, popups: [], files: [] };
      const beforeTargets = new Set((await client.diagnose()).targets.map(instanceKey));
      const dispatchTo = openPopups.at(-1) ?? main;
      const actions = step.click !== undefined ? [await clickAction(client, main, step.click)] : actionsForStep(step);
      if (step.control) {
        const expected = (await client.inspect(main)).targetIdentity;
        const result = await client.design({ operation: "fixtureControl", target: main, expected, control: step.control });
        if (result.operation !== "fixtureControl" || !result.ok) throw new EvaluationContractError("step_control_refused", [step.name]);
        record.effects.push({ control: step.control, ok: true });
      }
      let firstBefore: number | undefined; let lastEffect: string | undefined;
      for (const action of actions) {
        const { response, retries } = await actWithRetry(client, dispatchTo, action);
        const receipt = response.actionReceipt ?? response.results?.[0]?.actionReceipt;
        firstBefore ??= receipt?.before?.frameGeneration;
        lastEffect = receipt?.effect?.kind;
        record.effects.push({ action, effect: receipt?.effect, staleRetries: retries, frameBefore: receipt?.before?.frameGeneration, frameAfter: receipt?.after?.frameGeneration ?? null });
        if (receipt?.effect?.kind === "popupOpened") openPopups.push(receipt.effect.target);
        if (["popupClosed", "rootClosed"].includes(receipt?.effect?.kind) && dispatchTo !== main) openPopups.splice(openPopups.indexOf(dispatchTo), 1);
      }
      record.dispatchTarget = actions.length ? (dispatchTo === main ? "main" : `popup:${instanceKey(dispatchTo)}`) : "none";
      const mainAlive = (await client.diagnose()).targets.some(target => instanceKey(target) === instanceKey(main));
      if (!mainAlive) throw new EvaluationContractError("main_window_closed_by_step", [step.name]);
      if (actions.length && lastEffect !== "noOp" && firstBefore !== undefined && dispatchTo === main) {
        const waited = await client.wait(main, { type: "completedFrame", afterFrameGeneration: firstBefore }, NATURAL_FRAME_TIMEOUT_MS);
        record.naturalFrame = { observed: true, afterFrameGeneration: firstBefore, frameGeneration: waited.frameIdentity?.target?.frameGeneration };
      } else record.naturalFrame = { observed: false, reason: !actions.length ? "no-action" : dispatchTo !== main ? "dispatched-to-popup" : "noOp-effect" };
      const waitMs = stepWaitMs(step);
      if (waitMs) await sleep(waitMs);
      const afterTargets = (await client.diagnose()).targets;
      for (const target of afterTargets) {
        const instance: AutomationInstance = { type: "instance", id: target.windowId, generation: target.windowGeneration };
        if (!beforeTargets.has(instanceKey(target)) && !openPopups.some(popup => instanceKey(popup) === instanceKey(instance))) openPopups.push(instance);
      }
      for (let i = openPopups.length - 1; i >= 0; i--) if (!afterTargets.some(target => instanceKey(target) === instanceKey(openPopups[i]!))) openPopups.splice(i, 1);
      if (capturing) {
        const stem = frameStem(index, step.name);
        const capture = await client.captureFrame(main, true); images++;
        const headerElements = await client.query(main, "elements", { includeHeaders: true });
        const { image, files } = writeCaptureFiles(out, stem, capture);
        record.frame = { ...capture.frame, phaseDurationsMs: capture.phaseDurationsMs, capture: { width: capture.snapshot.capture?.width, height: capture.snapshot.capture?.height,
          sha256: capture.snapshot.capture?.sha256, scaleFactor: capture.snapshot.scaleFactor, limitation: capture.snapshot.limitation },
          headerElementsFrameGeneration: headerElements.targetIdentity?.frameGeneration ?? null };
        record.measurement = measureFrame({ layout: capture.layout, elements: capture.elements, headerElements, state: capture.state, image });
        record.inputValue = typeof capture.state.inputValue === "string" ? capture.state.inputValue : null;
        if (step.type !== undefined && record.inputValue !== step.type) record.measurement.issues.push("input_value_differs_from_typed_text");
        record.files.push(...files);
        writeFileSync(join(out, `${stem}.capture.json`), JSON.stringify({ frame: capture.frame, state: capture.state, elements: capture.elements, headerElements, layout: capture.layout }, null, 1));
        record.files.push(`${stem}.capture.json`);
        frames.push({ label: `${String(index).padStart(2, "0")} ${step.name}`, image: compositeOver(image) });
        for (const popup of openPopups) {
          const popupRecord: Json = { target: instanceKey(popup) };
          if (images >= OWNED_EVALUATION_LIMITS.maxRetainedImages) { popupRecord.captured = false; popupRecord.reason = "retained_image_budget"; }
          else {
            const popupCapture = await client.captureFrame(popup, true); images++;
            const popupStem = `${stem}.popup-${popup.id.replace(/[^a-zA-Z0-9-]/g, "_")}`;
            const written = writeCaptureFiles(out, popupStem, popupCapture);
            writeFileSync(join(out, `${popupStem}.capture.json`), JSON.stringify({ frame: popupCapture.frame, elements: popupCapture.elements, layout: popupCapture.layout }, null, 1));
            Object.assign(popupRecord, { captured: true, appViewVariant: popupCapture.frame.target.appViewVariant, files: [...written.files, `${popupStem}.capture.json`],
              width: popupCapture.snapshot.capture?.width, height: popupCapture.snapshot.capture?.height });
            record.files.push(...written.files, `${popupStem}.capture.json`);
          }
          record.popups.push(popupRecord);
        }
        record.captured = true;
        writeFileSync(join(out, `${stem}.json`), JSON.stringify({ step, index, runtime, frame: record.frame, naturalFrame: record.naturalFrame, dispatchTarget: record.dispatchTarget,
          inputValue: record.inputValue, staleRetries: record.effects.reduce((sum, effect) => sum + (effect.staleRetries ?? 0), 0),
          effects: record.effects.map(effect => ({ action: effect.action, effect: effect.effect })), popups: record.popups, ...record.measurement }, null, 1));
        record.files.unshift(`${stem}.json`);
        records.push(record);
      }
    }
  } catch (error) {
    const code = error instanceof EvaluationContractError || error instanceof DriverCommandRefused ? error.code :
      error instanceof DriverLifecycleError ? `driver_lifecycle:${client ? "after-launch" : "launch"}:${error.message}` : error instanceof Error ? error.message : String(error);
    const failed = plan.capture.find(index => !records.some(record => record.index === index));
    records.push({ index: failed ?? -1, name: failed === undefined ? "runtime" : steps[failed]!.name, step: failed === undefined ? { name: "runtime" } : steps[failed]!, runtime,
      captured: false, effects: [], dispatchTarget: "none", naturalFrame: null, popups: [], files: [], error: code });
    if (error instanceof DriverLifecycleError) lifecycleCleanup = error.cleanup;
  }
  if (client) { try { return await client.close(); } catch { return client.cleanup; } }
  return lifecycleCleanup ?? { resourcesAcquired: false, processExited: true, processGroupExited: true, streamsDrained: true,
    logWriterClosed: true, ownedWindowsClosed: null, referencesFinalized: false, closed: false, survivors: [], failureCodes: ["runtime_not_launched"] };
}

type NativeGlass = "platform-default" | "disabled";
export async function runFrames(argv: string[]): Promise<number> {
  const arg = (name: string) => { const index = argv.indexOf(name); return index < 0 ? undefined : argv[index + 1]; };
  const fixture = arg("--fixture") ?? MAIN_FIXTURE_ID;
  const nativeGlass = (arg("--native-glass") ?? "platform-default") as NativeGlass;
  if (!["platform-default", "disabled"].includes(nativeGlass)) throw new EvaluationContractError("invalid_native_glass", [nativeGlass]);
  if (argv[0] !== "run" || argv.includes("--help") || argv.includes("-h") || !arg("--artifact") || !arg("--out") || !arg("--steps")) {
    console.log("one-window-frames run --artifact <app.reference.json> --out <new-or-empty-dir> --steps <steps.json> [--fixture <id>] [--native-glass disabled]\n" +
      "steps: [{name, type?: text, append?: text, key?: 'cmd-k'|'escape'|'down'..., click?: <painted node>, control?: {family:'oneWindow', operation:'talkTo', destination:'chat'|'note'} | {family:'oneWindow', operation:'summon'}, waitMs?}].\n" +
      "type replaces the filter (cmd-a + one GPUI key per char); control starts a voice session to that destination or\n" +
      "brings the window back the way the main hotkey does,\n" +
      `default waitMs ${DEFAULT_TYPE_WAIT_MS} for type steps, 0 otherwise. Owned evaluator only: hidden windows, GPU readback, no OS input.`);
    return argv[0] === "run" ? 2 : 0;
  }
  const startedAt = new Date().toISOString(); const started = performance.now();
  const steps = parseSteps(JSON.parse(readFileSync(resolve(arg("--steps")!), "utf8")));
  const out = resolve(arg("--out")!);
  if (existsSync(out) && (!lstatSync(out).isDirectory() || readdirSync(out).length)) throw new EvaluationContractError("out_must_be_absent_or_empty");
  mkdirSync(out, { recursive: true });
  const { readArtifactReference } = await import("../devtools/design.ts");
  const referencePath = resolve(arg("--artifact")!);
  const reference = readArtifactReference(referencePath);
  const claimRoot = join(REPOSITORY_ROOT, ".test-output", "one-window-frames", `${startedAt.replace(/[:.]/g, "-")}-${process.pid}`);
  const claim = claimOutput(validateOutputTarget({ repoRoot: REPOSITORY_ROOT, candidate: claimRoot, kind: "directory", probeId: "one-window-frames" }));
  const task = beginManagedTask(claim, "evidence-run", [reference]);
  const records: StepRecord[] = []; const frames: { label: string; image: RgbaImage }[] = []; const cleanups: OwnedCleanup[] = [];
  const plans = planRuntimes(steps.length);
  for (const [runtime, plan] of plans.entries()) {
    cleanups.push(await runRuntime({ runtime, plan, steps, reference, claim, out, records, frames, fixture, nativeGlass }));
    if (records.some(record => record.error)) break;
  }
  let cleanup = aggregateCleanup(cleanups);
  try { cleanup = finalizeManagedTask(task, cleanup).cleanup; }
  catch { cleanup = { ...cleanup, closed: false, referencesFinalized: false, failureCodes: [...cleanup.failureCodes, "evidence_finalization_failed"] }; }
  const captured = records.filter(record => record.captured).sort((a, b) => a.index - b.index);
  const errors = records.filter(record => record.error).map(record => ({ step: record.name, index: record.index, error: record.error }));
  const missing = steps.map((step, index) => ({ step: step.name, index })).filter(({ index }) => !captured.some(record => record.index === index));
  const summary = summarize(steps.map((step, index) => {
    const record = captured.find(r => r.index === index);
    return { index, name: step.name, measurement: record?.measurement ?? null, ...(record ? {} : { error: errors.find(e => e.index === index)?.error ?? "not_captured" }) };
  }));
  writeFileSync(join(out, "summary.json"), JSON.stringify({ schemaVersion: 1, fixture, nativeGlass, background: "#1c1d22",
    units: "window points; PNGs are device pixels (scale in each step)", capsulesNote: "native AppKit footer capsules are outside GPUI readback",
    steps: summary }, null, 1));
  if (frames.length) writeFileSync(join(out, "contact.png"), encodePngRgba(contactSheet(frames)));
  const pass = !errors.length && !missing.length && cleanup.closed;
  const receipt = {
    schemaVersion: 1, tool: "one-window-frames", command: "run", classification: pass ? "ok" : !cleanup.closed ? "invalid-cleanup" : "failed",
    startedAt, durationMs: Math.round(performance.now() - started), artifactReference: referencePath, fixture, nativeGlass,
    proofLevel: "owned-production-runtime", evidence: "GPUI render readback of hidden owned windows; owned GPUI key events",
    nativeExclusions: ["WindowServer composition", "AppKit glass material", "native footer capsules", "native focus", "OS IME", "global input"],
    out, evaluatorClaim: claim.root, runtimes: plans.map((plan, runtime) => ({ runtime, replayedSteps: plan.replay, capturedSteps: plan.capture.map(i => steps[i]!.name), cleanupClosed: cleanups[runtime]?.closed ?? null })),
    frames: captured.map(record => ({ index: record.index, name: record.name, files: record.files, frameGeneration: record.frame?.target?.frameGeneration,
      naturalFrame: record.naturalFrame, effects: record.effects.map(effect => effect.effect?.kind), popups: record.popups, issues: record.measurement?.issues })),
    missing, errors, cleanup,
  };
  writeFileSync(join(out, "receipt.json"), JSON.stringify(receipt, null, 1));
  console.log(JSON.stringify(receipt));
  return pass ? 0 : 1;
}

if (import.meta.main) process.exitCode = await runFrames(Bun.argv.slice(2));
