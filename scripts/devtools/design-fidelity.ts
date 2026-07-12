#!/usr/bin/env bun

export type Rect = { x: number; y: number; width: number; height: number; right?: number; bottom?: number };

export type FidelityManifest = {
  screen: {
    id: string;
    target: { automationId?: string; targetKind?: string; surfaceKind?: string };
    viewport: { width: number; height: number };
  };
  tolerance?: number;
  requirePaintMeasurement?: boolean;
  requireVisibleMeasurement?: boolean;
  imageDiff?: { required?: boolean; maxChangedPixelRatio?: number; requireSameSize?: boolean; requireInputHashes?: boolean; requireRedOsEvidence?: boolean };
  elements: Array<{ fidelityId: string; gpuiId: string; exactText?: boolean }>;
};

export type DomElementMeasurement = {
  fidelityId: string;
  rect: Rect;
  visibleRect: Rect;
  clipRect: Rect;
  text: string;
  style: Record<string, string>;
  overflow: { x: boolean; y: boolean; truncated: boolean };
};

export type DomReceipt = {
  screenId: string;
  viewport: { width: number; height: number };
  windowRect: Rect;
  devicePixelRatio: number;
  elements: DomElementMeasurement[];
};

type AnyObject = Record<string, any>;

export const BROWSER_COLLECTOR_SOURCE = String.raw`(() => {
  const win = document.querySelector('#window');
  if (!win) throw new Error('Missing mockup #window');
  const rect = (node) => { const r = node.getBoundingClientRect(); return {x:r.x,y:r.y,right:r.right,bottom:r.bottom,width:r.width,height:r.height}; };
  const windowRect = rect(win);
  const clippedOverflow = (value) => ['hidden','clip','scroll','auto'].includes(value);
  const visibleGeometry = (node) => {
    let clip = {...windowRect};
    let ancestor = node.parentElement;
    while (ancestor) {
      const ancestorRect = rect(ancestor);
      const ancestorStyle = getComputedStyle(ancestor);
      const clipX = ancestor === win || clippedOverflow(ancestorStyle.overflowX);
      const clipY = ancestor === win || clippedOverflow(ancestorStyle.overflowY);
      if (clipX) { clip.x=Math.max(clip.x,ancestorRect.x); clip.right=Math.min(clip.right,ancestorRect.right); clip.width=Math.max(0,clip.right-clip.x); }
      if (clipY) { clip.y=Math.max(clip.y,ancestorRect.y); clip.bottom=Math.min(clip.bottom,ancestorRect.bottom); clip.height=Math.max(0,clip.bottom-clip.y); }
      if (ancestor === win) break;
      ancestor = ancestor.parentElement;
    }
    const full = rect(node);
    const x=Math.max(full.x,clip.x), y=Math.max(full.y,clip.y);
    const right=Math.max(x,Math.min(full.right,clip.right)), bottom=Math.max(y,Math.min(full.bottom,clip.bottom));
    return { clipRect:clip, visibleRect:{x,y,right,bottom,width:right-x,height:bottom-y} };
  };
  const elements = [...document.querySelectorAll('[data-fidelity-id]')].map((node) => {
    const style = getComputedStyle(node);
    const visibility = visibleGeometry(node);
    return {
      fidelityId: node.getAttribute('data-fidelity-id'), rect: rect(node), ...visibility,
      text: node.innerText ?? node.textContent ?? '',
      style: { fontFamily:style.fontFamily, fontSize:style.fontSize, fontWeight:style.fontWeight,
        lineHeight:style.lineHeight, color:style.color, backgroundColor:style.backgroundColor,
        borderTopWidth:style.borderTopWidth, borderRightWidth:style.borderRightWidth,
        borderBottomWidth:style.borderBottomWidth, borderLeftWidth:style.borderLeftWidth,
        borderTopColor:style.borderTopColor, borderRightColor:style.borderRightColor,
        borderBottomColor:style.borderBottomColor, borderLeftColor:style.borderLeftColor,
        borderTopLeftRadius:style.borderTopLeftRadius, borderTopRightRadius:style.borderTopRightRadius,
        borderBottomRightRadius:style.borderBottomRightRadius, borderBottomLeftRadius:style.borderBottomLeftRadius,
        opacity:style.opacity, overflowX:style.overflowX, overflowY:style.overflowY,
        textOverflow:style.textOverflow, whiteSpace:style.whiteSpace },
      overflow: { x:node.scrollWidth > node.clientWidth, y:node.scrollHeight > node.clientHeight,
        truncated:node.scrollWidth > node.clientWidth || node.scrollHeight > node.clientHeight }
    };
  });
  return { screenId: document.documentElement.getAttribute('data-fidelity-screen') || document.body.getAttribute('data-fidelity-screen'),
    viewport:{width:windowRect.width,height:windowRect.height}, windowRect, devicePixelRatio:window.devicePixelRatio, elements };
})()`;

function object(value: unknown): AnyObject { return value && typeof value === "object" ? value as AnyObject : {}; }
function finite(value: unknown): value is number { return typeof value === "number" && Number.isFinite(value); }
function normalizedText(value: unknown) { return String(value ?? "").replace(/\s+/g, " ").trim(); }

function rect(value: unknown): Rect | null {
  const r = object(value);
  if (![r.x, r.y, r.width, r.height].every(finite)) return null;
  return { x: r.x, y: r.y, width: r.width, height: r.height, right: finite(r.right) ? r.right : r.x + r.width, bottom: finite(r.bottom) ? r.bottom : r.y + r.height };
}

function relative(r: Rect, root: Rect): Required<Rect> {
  return { x:r.x-root.x, y:r.y-root.y, width:r.width, height:r.height,
    right:(r.right ?? r.x+r.width)-root.x, bottom:(r.bottom ?? r.y+r.height)-root.y };
}

function firstRect(...values: unknown[]) { for (const value of values) { const found = rect(value); if (found) return found; } return null; }
function gpuiRoot(receipt: AnyObject) {
  const explicit = firstRect(
    receipt.windowRect,
    receipt.viewportRect,
    receipt.window?.rect,
    receipt.result?.windowRect,
    receipt.result?.viewportRect,
  );
  if (explicit) return explicit;
  const viewport = object(receipt.viewport ?? receipt.result?.viewport);
  const width = viewport.clientWidth ?? viewport.width;
  const height = viewport.clientHeight ?? viewport.height;
  return finite(width) && finite(height)
    ? rect({ x: 0, y: 0, width, height })
    : null;
}
function gpuiElements(receipt: AnyObject): AnyObject[] {
  for (const value of [
    receipt.elements,
    receipt.nodes,
    receipt.regions,
    receipt.components,
    receipt.rawLayout?.info?.components,
    receipt.rawLayout?.components,
    receipt.result?.elements,
    receipt.result?.nodes,
    receipt.result?.regions,
    receipt.result?.components,
  ]) {
    if (Array.isArray(value)) return value.map(object);
    if (value && typeof value === "object") return Object.entries(value).map(([id, v]) => ({ id, ...object(v) }));
  }
  return [];
}
function gpuiId(node: AnyObject) { return String(node.semanticId ?? node.id ?? node.name ?? node.componentId ?? ""); }
function gpuiRect(node: AnyObject) { return firstRect(node.rect, node.bounds, node.frame); }
function provenance(node: AnyObject, receipt: AnyObject) { return String(node.measurementProvenance ?? node.provenance ?? receipt.measurementProvenance ?? receipt.provenance ?? ""); }
function identity(receipt: AnyObject) { return object(receipt.resolvedTarget ?? receipt.target ?? receipt.result?.resolvedTarget ?? receipt.result?.target); }

function gpuiCoordinateSpace(node: AnyObject, receipt: AnyObject) {
  const explicit = String(
    node.coordinateSpace ??
    node.measurementCoordinateSpace ??
    receipt.elementCoordinateSpace ??
    receipt.measurementCoordinateSpace ??
    "",
  ).toLowerCase();
  if (explicit === "window" || explicit === "viewport" || explicit === "local" || explicit === "windowlogicalpx") return "window";
  if (explicit === "screen" || explicit === "absolute") return "screen";
  // `scripts/devtools/layout.ts` intentionally reports every node in the
  // GPUI window's logical coordinate space while `windowRect` is the OS
  // screen-space target rect. Raw getLayoutInfo components follow the same
  // contract. Treating those nodes as screen-absolute produces huge false
  // deltas whenever the window is not at (0, 0).
  if (receipt.tool === "script-kit-devtools.layout" || receipt.rawLayout) return "window";
  return "screen";
}

function gpuiRelativeRect(
  node: AnyObject,
  value: Rect,
  root: Rect,
  receipt: AnyObject,
): Required<Rect> {
  if (gpuiCoordinateSpace(node, receipt) === "window") {
    return {
      x: value.x,
      y: value.y,
      width: value.width,
      height: value.height,
      right: value.right ?? value.x + value.width,
      bottom: value.bottom ?? value.y + value.height,
    };
  }
  return relative(value, root);
}

function assertion(name: string, passed: boolean, details?: unknown) { return { name, passed, ...(details === undefined ? {} : { details }) }; }
function iou(a: Required<Rect>, b: Required<Rect>) {
  const area = Math.max(0, Math.min(a.right,b.right)-Math.max(a.x,b.x))*Math.max(0,Math.min(a.bottom,b.bottom)-Math.max(a.y,b.y));
  const union = a.width*a.height+b.width*b.height-area;
  return union > 0 ? area/union : (a.width === 0 && a.height === 0 && b.width === 0 && b.height === 0 ? 1 : 0);
}

export function compareDesignFidelity(manifest: FidelityManifest, gpuiInput: unknown, domInput: unknown, imageDiffInput?: unknown) {
  const gpui = object(gpuiInput), dom = object(domInput), imageDiff = imageDiffInput == null ? null : object(imageDiffInput);
  const tolerance = finite(manifest.tolerance) ? manifest.tolerance : 0.5;
  const assertions: AnyObject[] = [], warnings: string[] = [], errors: string[] = [], elements: AnyObject[] = [];
  const fail = (name: string, message: string, details?: unknown) => { assertions.push(assertion(name, false, details)); errors.push(message); };
  const pass = (name: string, details?: unknown) => assertions.push(assertion(name, true, details));
  const target = identity(gpui);
  for (const key of ["automationId", "targetKind", "surfaceKind"] as const) {
    const expected = manifest.screen.target[key]; if (!expected) continue;
    const actual = target[key] ?? gpui[key]; actual === expected ? pass(`target.${key}`, {expected,actual}) : fail(`target.${key}`, `GPUI target ${key} mismatch`, {expected,actual:actual ?? null});
  }
  dom.screenId === manifest.screen.id ? pass("screenIdentity") : fail("screenIdentity", "DOM screen identity mismatch", {expected:manifest.screen.id,actual:dom.screenId ?? null});
  const domRoot = rect(dom.windowRect), gpuiWindow = gpuiRoot(gpui);
  if (!domRoot) fail("domWindowRect", "DOM receipt is missing #window rect"); else pass("domWindowRect");
  if (!gpuiWindow) fail("gpuiWindowRect", "GPUI receipt is missing real window rect"); else pass("gpuiWindowRect");
  const viewportChecks = [
    ["manifestVsDom", manifest.screen.viewport, dom.viewport],
    ["manifestVsGpui", manifest.screen.viewport, gpuiWindow && {width:gpuiWindow.width,height:gpuiWindow.height}],
  ] as const;
  for (const [name, expected, actual] of viewportChecks) {
    const ok = !!actual && actual.width === expected.width && actual.height === expected.height;
    ok ? pass(`viewport.${name}`, {expected,actual}) : fail(`viewport.${name}`, `Viewport mismatch: ${name}`, {expected,actual:actual ?? null});
  }
  const domNodes = Array.isArray(dom.elements) ? dom.elements.map(object) : [];
  const gpuiNodes = gpuiElements(gpui);
  const paintFrameGenerations: number[] = [];
  const manifestIds = manifest.elements.map(e => e.fidelityId), manifestGpuiIds = manifest.elements.map(e => e.gpuiId);
  const duplicateManifest = [...new Set([...manifestIds.filter((id,i,a)=>a.indexOf(id)!==i), ...manifestGpuiIds.filter((id,i,a)=>a.indexOf(id)!==i)])];
  duplicateManifest.length ? fail("mappingUnique", "Manifest mappings must be one-to-one", duplicateManifest) : pass("mappingUnique");
  const markedIds = domNodes.map(n => String(n.fidelityId ?? ""));
  const duplicateDom = [...new Set(markedIds.filter((id,i,a)=>!id || a.indexOf(id)!==i))];
  duplicateDom.length ? fail("domIdsUnique", "Every data-fidelity-id must occur exactly once", duplicateDom) : pass("domIdsUnique");
  const unmappedDom = markedIds.filter(id => !manifestIds.includes(id));
  unmappedDom.length ? fail("allDomNodesMapped", "Every data-fidelity-id must be mapped", unmappedDom) : pass("allDomNodesMapped");
  for (const mapping of manifest.elements) {
    const domMatches = domNodes.filter(n => n.fidelityId === mapping.fidelityId);
    const gpuiMatches = gpuiNodes.filter(n => gpuiId(n) === mapping.gpuiId);
    if (domMatches.length !== 1) { fail(`element.${mapping.fidelityId}.domPresent`, `DOM mapping ${mapping.fidelityId} resolved ${domMatches.length} times`); continue; }
    if (gpuiMatches.length !== 1) { fail(`element.${mapping.fidelityId}.gpuiPresent`, `GPUI mapping ${mapping.gpuiId} resolved ${gpuiMatches.length} times`); continue; }
    const dr = rect(domMatches[0].rect), gr = gpuiRect(gpuiMatches[0]);
    if (!dr || !gr || !domRoot || !gpuiWindow) { fail(`element.${mapping.fidelityId}.geometryAvailable`, `Missing geometry for ${mapping.fidelityId}`); continue; }
    if (manifest.requirePaintMeasurement) {
      const source = provenance(gpuiMatches[0], gpui).toLowerCase();
      if (!source.includes("paint")) { fail(`element.${mapping.fidelityId}.paintProvenance`, `Paint-time GPUI measurement required for ${mapping.gpuiId}`, {provenance:source || null}); continue; }
      pass(`element.${mapping.fidelityId}.paintProvenance`, source);
      const frameGeneration = gpuiMatches[0].measurementFrameGeneration;
      if (!finite(frameGeneration)) fail(`element.${mapping.fidelityId}.paintFrame`, `Paint-frame generation required for ${mapping.gpuiId}`, {frameGeneration:frameGeneration ?? null});
      else { paintFrameGenerations.push(frameGeneration); pass(`element.${mapping.fidelityId}.paintFrame`, frameGeneration); }
    }
    const html = relative(dr, domRoot);
    const native = gpuiRelativeRect(gpuiMatches[0], gr, gpuiWindow, gpui);
    const deltas = Object.fromEntries(["x","y","right","bottom","width","height"].map(k => [k, Math.abs(html[k as keyof Rect] as number-native[k as keyof Rect] as number)]));
    const fullWithinTolerance = Object.values(deltas).every(delta => delta <= tolerance);
    let htmlVisible: Required<Rect> | null = null;
    let gpuiVisible: Required<Rect> | null = null;
    let visibleDeltas: Record<string, number> | null = null;
    let visibleWithinTolerance = !manifest.requireVisibleMeasurement;
    if (manifest.requireVisibleMeasurement) {
      const domVisible = rect(domMatches[0].visibleRect);
      const nativeVisible = firstRect(gpuiMatches[0].visibleBounds, gpuiMatches[0].visibleRect);
      if (!domVisible || !nativeVisible) {
        fail(`element.${mapping.fidelityId}.visibleGeometryAvailable`, `Visible paint geometry is required for ${mapping.fidelityId}`, {domVisible:!!domVisible,gpuiVisible:!!nativeVisible});
      } else {
        htmlVisible = relative(domVisible, domRoot);
        gpuiVisible = gpuiRelativeRect(gpuiMatches[0], nativeVisible, gpuiWindow, gpui);
        visibleDeltas = Object.fromEntries(["x","y","right","bottom","width","height"].map(k => [k, Math.abs(htmlVisible![k as keyof Rect] as number-gpuiVisible![k as keyof Rect] as number)]));
        visibleWithinTolerance = Object.values(visibleDeltas).every(delta => delta <= tolerance);
        visibleWithinTolerance
          ? pass(`element.${mapping.fidelityId}.visibleGeometry`, visibleDeltas)
          : fail(`element.${mapping.fidelityId}.visibleGeometry`, `Visible geometry exceeds tolerance for ${mapping.fidelityId}`, visibleDeltas);
      }
    }
    const withinTolerance = fullWithinTolerance && visibleWithinTolerance;
    const text = { html:normalizedText(domMatches[0].text), gpui:normalizedText(gpuiMatches[0].text ?? gpuiMatches[0].label) };
    const textExact = !mapping.exactText || text.html === text.gpui;
    fullWithinTolerance ? pass(`element.${mapping.fidelityId}.geometry`, deltas) : fail(`element.${mapping.fidelityId}.geometry`, `Geometry exceeds tolerance for ${mapping.fidelityId}`, deltas);
    textExact ? pass(`element.${mapping.fidelityId}.text`, text) : fail(`element.${mapping.fidelityId}.text`, `Normalized text mismatch for ${mapping.fidelityId}`, text);
    elements.push({ fidelityId:mapping.fidelityId, gpuiId:mapping.gpuiId, htmlRect:html, gpuiRect:native, deltas, iou:iou(html,native), htmlVisibleRect:htmlVisible, gpuiVisibleRect:gpuiVisible, visibleDeltas, visibleWithinTolerance, tolerance, withinTolerance, paintFrameGeneration:gpuiMatches[0].measurementFrameGeneration ?? null, ...(mapping.exactText ? {text,exactText:textExact}:{}), domStyle:domMatches[0].style ?? null, domOverflow:domMatches[0].overflow ?? null });
  }
  if (manifest.requirePaintMeasurement) {
    const uniqueFrames = [...new Set(paintFrameGenerations)];
    uniqueFrames.length === 1 && paintFrameGenerations.length === manifest.elements.length
      ? pass("paintFrameCoherent", {frameGeneration:uniqueFrames[0],elementCount:paintFrameGenerations.length})
      : fail("paintFrameCoherent", "All mapped GPUI elements must come from one completed paint frame", {uniqueFrames,measured:paintFrameGenerations.length,required:manifest.elements.length});
  }
  let imageDiffEvidence: AnyObject | null = null;
  if (imageDiff) {
    const valid = imageDiff.tool === "script-kit-devtools.image-diff" && imageDiff.classification === "ok";
    imageDiffEvidence = {
      valid,
      tool:imageDiff.tool ?? null,
      classification:imageDiff.classification ?? null,
      changedPixelCount:imageDiff.changedPixelCount ?? imageDiff.changedPixels ?? imageDiff.metrics?.changedPixelCount ?? null,
      changedPixelRatio:imageDiff.changedPixelRatio ?? imageDiff.metrics?.changedPixelRatio ?? null,
      inputHashes:imageDiff.inputHashes ?? null,
      inputEvidence:imageDiff.inputEvidence ?? null,
    };
    if (!valid) warnings.push("Ignored malformed image-diff receipt");
    else if (!manifest.imageDiff?.required) warnings.push("Image-diff is supplemental evidence only; geometry determines fidelity");
  }
  if (manifest.imageDiff?.required) {
    if (!imageDiffEvidence?.valid) fail("imageDiff.required", "A valid image-diff receipt is required");
    else {
      const changedRatio = imageDiffEvidence.changedPixelRatio;
      const maxRatio = finite(manifest.imageDiff.maxChangedPixelRatio) ? manifest.imageDiff.maxChangedPixelRatio : 0;
      finite(changedRatio) && changedRatio <= maxRatio
        ? pass("imageDiff.changedPixelRatio", {actual:changedRatio,maximum:maxRatio})
        : fail("imageDiff.changedPixelRatio", "Changed-pixel ratio exceeds the manifest limit", {actual:changedRatio ?? null,maximum:maxRatio});
      if (manifest.imageDiff.requireSameSize) {
        const sameSize = imageDiff.dimensions?.sameSize === true;
        sameSize ? pass("imageDiff.sameSize") : fail("imageDiff.sameSize", "Image dimensions must match exactly", {dimensions:imageDiff.dimensions ?? null});
      }
      if (manifest.imageDiff.requireInputHashes) {
        const redMatches = imageDiff.inputHashes?.red?.matchesReceipt === true;
        const greenMatches = imageDiff.inputHashes?.green?.matchesReceipt === true;
        redMatches && greenMatches
          ? pass("imageDiff.inputHashes", imageDiff.inputHashes)
          : fail("imageDiff.inputHashes", "Both image inputs must match their capture receipt hashes", {inputHashes:imageDiff.inputHashes ?? null});
      }
      if (manifest.imageDiff.requireRedOsEvidence) {
        const redEvidence = imageDiff.inputEvidence?.red;
        const redIsOsCapture =
          redEvidence?.source === "os-window-capture" &&
          redEvidence?.classification === "captured" &&
          redEvidence?.countsAsOsScreenshotEvidence === true &&
          redEvidence?.countsAsCompositorEvidence === true &&
          redEvidence?.pixelAuditBlank === false;
        redIsOsCapture
          ? pass("imageDiff.redOsEvidence", redEvidence)
          : fail("imageDiff.redOsEvidence", "The GPUI image must be a nonblank OS compositor capture", {redEvidence:redEvidence ?? null});
      }
    }
  }
  const classification = errors.length ? "reproduced" : "ok";
  return { schemaVersion:1, tool:"script-kit-devtools.design-fidelity", command:"design-fidelity.compare", classification,
    screenId:manifest.screen.id, tolerance, targetIdentity:target, viewport:{expected:manifest.screen.viewport,dom:dom.viewport ?? null,gpui:gpuiWindow && {width:gpuiWindow.width,height:gpuiWindow.height}},
    elements, imageDiffEvidence, assertions, warnings, errors };
}

async function main() {
  const args = process.argv.slice(2); if (args[0] !== "compare") return;
  const value = (flag:string) => { const i=args.indexOf(flag); return i < 0 ? "" : args[i+1] ?? ""; };
  const manifestPath=value("--manifest"), gpuiPath=value("--gpui"), domPath=value("--dom"), imagePath=value("--image-diff"), outputPath=value("--out");
  if (!manifestPath || !gpuiPath || !domPath) { console.error("Usage: bun scripts/devtools/design-fidelity.ts compare --manifest <json> --gpui <getLayoutInfo.json> --dom <browser.json> [--image-diff <json>] [--out <receipt.json>]"); process.exit(2); }
  try {
    const [manifest,gpui,dom,image] = await Promise.all([manifestPath,gpuiPath,domPath,imagePath].map(async p => p ? JSON.parse(await Bun.file(p).text()) : undefined));
    const receipt=compareDesignFidelity(manifest,gpui,dom,image); const serialized=`${JSON.stringify(receipt,null,2)}\n`; if (outputPath) await Bun.write(outputPath,serialized); console.log(serialized.trimEnd()); if (receipt.errors.length) process.exitCode=1;
  } catch (error) {
    console.log(JSON.stringify({schemaVersion:1,tool:"script-kit-devtools.design-fidelity",command:"design-fidelity.compare",classification:"blocked-by-parse-error",assertions:[],warnings:[],errors:[error instanceof Error?error.message:String(error)]},null,2)); process.exitCode=2;
  }
}

if (import.meta.main) await main();
