// apple-guideline-constants.ts
//
// Apple-documented design-guideline constants + a mathematical conformance
// engine for the Tahoe / Liquid Glass UI proof. This replaces the previous
// project-local "minimum radius/padding/gap" guesses (and the literal
// `exactAppleRadiusConstants: null`) with a provenance-tagged table of what
// Apple ACTUALLY documents, plus per-node deviation math.
//
// Design contract (from Oracle session `tahoe-apple-guideline-metrics`):
//   - Apple does NOT publish "a 750pt panel = radius N". It publishes FORMULAS
//     (capsule radius = height/2; concentric childRadius = parentRadius − inset),
//     a few HARD spacing constants (regular-button centers ≥ 60pt, gap ≥ 16pt),
//     and SOFT heuristics (~12pt padding around bezeled elements). Window radii
//     are style-dependent / system-owned, so their exact value comes from a
//     native baseline probe (roadmap), never a guessed constant.
//   - Every metric carries `confidence` (documented|derived|measuredNative|
//     historical|estimated) and `normativeStrength` (hard|soft|historical|
//     projectException). We never copy Apple prose — only short paraphrased
//     summaries + a reference id/section.
//   - A good weighted average must never hide a HARD out-of-band failure.

export type GuidelineConfidence =
  | "documented"
  | "derived"
  | "measuredNative"
  | "historical"
  | "estimated";

export type NormativeStrength = "hard" | "soft" | "historical" | "projectException";

export type GuidelineClassification =
  | "withinBand"
  | "nearBand"
  | "outOfBand"
  | "unmeasured"
  | "notApplicable";

export interface AppleReference {
  referenceId: string;
  title: string;
  section: string;
  retrievedAt: string;
  platform: "macOS";
  osVersion: "26";
}

export interface GuidelineTolerance {
  absPt?: number;
  pct?: number;
  nearAbsPt?: number;
  nearPct?: number;
}

export type GuidelineTarget =
  | { kind: "constant"; valuePt: number }
  | { kind: "minimum"; minPt: number }
  | { kind: "range"; minPt?: number; maxPt?: number }
  | { kind: "formula"; expression: string }
  | { kind: "nativeProbe"; probeId: string };

// Provenance for a `measuredNative` metric: the exact AppKit control geometry
// probe that produced the value, captured on a real Tahoe machine. The receipt
// path lets a reviewer re-run `tahoe_native_baseline.swift` and diff the number.
export interface NativeMeasurement {
  probeId: string;
  probeSource: string;
  receiptPath: string;
  osVersion: string;
  measuredAt: string;
  control: string;
  field: string;
}

export interface GuidelineMetric {
  id: string;
  category: "cornerRadius" | "concentricity" | "controlSize" | "padding" | "spacing" | "hitTarget";
  confidence: GuidelineConfidence;
  normativeStrength: NormativeStrength;
  target: GuidelineTarget;
  tolerance: GuidelineTolerance;
  appleReference?: AppleReference;
  derivation?: { fromMetricIds: string[]; formula: string; notes: string };
  nativeMeasurement?: NativeMeasurement;
  copyrightSafeSummary: string;
}

// --- Apple guideline provenance table -------------------------------------
// Short paraphrased summaries only. `appleReference.section` points at the doc
// anchor; we deliberately do not embed Apple's copyrighted wording.
export const APPLE_GUIDELINE_METRICS: GuidelineMetric[] = [
  {
    id: "shape.capsule.radius",
    category: "cornerRadius",
    confidence: "documented",
    normativeStrength: "hard",
    target: { kind: "formula", expression: "radiusPt = heightPt / 2" },
    tolerance: { absPt: 0.5, nearAbsPt: 1 },
    appleReference: {
      referenceId: "apple-wwdc25-new-design-system-shapes",
      title: "Get to know the new design system",
      section: "Shape types and concentric layouts",
      retrievedAt: "2026-05-30",
      platform: "macOS",
      osVersion: "26",
    },
    copyrightSafeSummary: "Apple defines a capsule's curvature as half the control height.",
  },
  {
    id: "shape.concentric.childRadius",
    category: "concentricity",
    confidence: "documented",
    normativeStrength: "hard",
    target: { kind: "formula", expression: "childRadiusPt = max(0, parentRadiusPt - separationPt)" },
    tolerance: { absPt: 1, pct: 0.05, nearAbsPt: 2, nearPct: 0.1 },
    appleReference: {
      referenceId: "apple-wwdc25-new-design-system-shapes",
      title: "Get to know the new design system",
      section: "Concentricity",
      retrievedAt: "2026-05-30",
      platform: "macOS",
      osVersion: "26",
    },
    copyrightSafeSummary: "Apple says nested rounded shapes should preserve concentric radii (inner = outer − inset).",
  },
  {
    id: "layout.regularButton.centerDistance",
    category: "spacing",
    confidence: "documented",
    normativeStrength: "hard",
    target: { kind: "minimum", minPt: 60 },
    tolerance: { absPt: 1, nearAbsPt: 4 },
    appleReference: {
      referenceId: "apple-hig-spatial-layout",
      title: "Layout",
      section: "Spatial layout — control spacing",
      retrievedAt: "2026-05-30",
      platform: "macOS",
      osVersion: "26",
    },
    copyrightSafeSummary: "Apple spatial-layout guidance gives a 60pt minimum center distance for regular buttons.",
  },
  {
    id: "layout.regularButton.minimumGap",
    category: "spacing",
    confidence: "documented",
    normativeStrength: "hard",
    target: { kind: "minimum", minPt: 16 },
    tolerance: { absPt: 1, nearAbsPt: 4 },
    appleReference: {
      referenceId: "apple-hig-spatial-layout",
      title: "Layout",
      section: "Spatial layout — control spacing",
      retrievedAt: "2026-05-30",
      platform: "macOS",
      osVersion: "26",
    },
    copyrightSafeSummary: "Apple recommends 16pt or more of visible gap between regular buttons.",
  },
  {
    id: "layout.bezelElement.padding",
    category: "padding",
    confidence: "documented",
    normativeStrength: "soft",
    target: { kind: "minimum", minPt: 12 },
    tolerance: { absPt: 2, nearAbsPt: 4 },
    appleReference: {
      referenceId: "apple-hig-accessibility-layout",
      title: "Accessibility",
      section: "Layout and spacing",
      retrievedAt: "2026-05-30",
      platform: "macOS",
      osVersion: "26",
    },
    copyrightSafeSummary: "Apple accessibility guidance recommends ~12pt padding around bezeled elements (~24pt without a bezel).",
  },
  {
    id: "control.searchField.textInset.horizontal",
    category: "padding",
    confidence: "measuredNative",
    normativeStrength: "hard",
    target: { kind: "constant", valuePt: 9 },
    tolerance: { absPt: 1, nearAbsPt: 2 },
    nativeMeasurement: {
      probeId: "macos26-nstextfield-regular-content-inset",
      probeSource: "scripts/devtools/tahoe_native_baseline.swift",
      receiptPath: "artifacts/liquid-glass/receipts/tahoe-native-baseline.json",
      osVersion: "26.5",
      measuredAt: "2026-05-30",
      control: "NSTextField (regular, rounded bezel)",
      field: "contentHorizontalInsetPt",
    },
    derivation: {
      fromMetricIds: ["layout.bezelElement.padding"],
      formula: "textInsetX = NSTextField regular drawingRect.minX − bounds.minX = 9pt (measured)",
      notes: "A plain input lane should match the native regular NSTextField horizontal inset (9pt); within the ~12pt bezel-padding soft band.",
    },
    copyrightSafeSummary: "Native regular NSTextField insets its text 9pt horizontally on macOS 26.5 (measured).",
  },
  {
    id: "control.searchField.textInset.vertical",
    category: "padding",
    confidence: "measuredNative",
    normativeStrength: "hard",
    target: { kind: "constant", valuePt: 3 },
    tolerance: { absPt: 1, nearAbsPt: 2 },
    nativeMeasurement: {
      probeId: "macos26-nstextfield-regular-content-inset-vertical",
      probeSource: "scripts/devtools/tahoe_native_baseline.swift",
      receiptPath: "artifacts/liquid-glass/receipts/tahoe-native-baseline.json",
      osVersion: "26.5",
      measuredAt: "2026-05-30",
      control: "NSTextField (regular, rounded bezel)",
      field: "contentVerticalInsetPt",
    },
    copyrightSafeSummary: "Native regular NSTextField insets its text 3pt vertically on macOS 26.5 (measured).",
  },
  {
    id: "control.regular.height",
    category: "controlSize",
    confidence: "measuredNative",
    normativeStrength: "hard",
    target: { kind: "constant", valuePt: 22 },
    tolerance: { absPt: 1, nearAbsPt: 3 },
    nativeMeasurement: {
      probeId: "macos26-control-regular-height",
      probeSource: "scripts/devtools/tahoe_native_baseline.swift",
      receiptPath: "artifacts/liquid-glass/receipts/tahoe-native-baseline.json",
      osVersion: "26.5",
      measuredAt: "2026-05-30",
      control: "NSTextField / NSSearchField (regular)",
      field: "intrinsicHeightPt",
    },
    copyrightSafeSummary: "Native regular text/search fields are 22pt tall on macOS 26.5 (mini 17, small 19, large 30).",
  },
  {
    id: "macos.glassEffectView.defaultRadius",
    category: "cornerRadius",
    confidence: "measuredNative",
    normativeStrength: "soft",
    target: { kind: "constant", valuePt: 8 },
    tolerance: { absPt: 1, pct: 0.05, nearAbsPt: 2 },
    nativeMeasurement: {
      probeId: "macos26-nsglasseffectview-default-radius",
      probeSource: "scripts/devtools/tahoe_native_baseline.swift",
      receiptPath: "artifacts/liquid-glass/receipts/tahoe-native-baseline.json",
      osVersion: "26.5",
      measuredAt: "2026-05-30",
      control: "NSGlassEffectView",
      field: "defaultCornerRadiusPt",
    },
    derivation: {
      fromMetricIds: ["shape.concentric.childRadius"],
      formula: "NSGlassEffectView().cornerRadius default = 8pt (measured)",
      notes: "A free-standing glass element's default radius; NOT the launcher window/panel mask radius, which remains a separate native probe.",
    },
    copyrightSafeSummary: "A default NSGlassEffectView rounds to 8pt on macOS 26.5 (measured); a free element default, not the window mask.",
  },
  {
    id: "macos.window.toolbarRadius.nativeBaseline",
    category: "cornerRadius",
    confidence: "measuredNative",
    normativeStrength: "hard",
    target: { kind: "nativeProbe", probeId: "macos26-nswindow-toolbar-panel-radius" },
    tolerance: { absPt: 1, pct: 0.05, nearAbsPt: 2 },
    derivation: {
      fromMetricIds: ["shape.concentric.childRadius"],
      formula: "Measure native Tahoe NSWindow/NSPanel mask radius for the launcher's style mask + material + scale.",
      notes: "Apple documents window radius as style-dependent/system-owned, so the exact value is a native measurement, not a constant.",
    },
    copyrightSafeSummary: "Exact Tahoe window radius is system/style-owned; resolve it via native measurement, not a guess.",
  },
  {
    id: "historical.searchField.height.regular",
    category: "controlSize",
    confidence: "historical",
    normativeStrength: "historical",
    target: { kind: "constant", valuePt: 22 },
    tolerance: { absPt: 1, nearAbsPt: 3 },
    appleReference: {
      referenceId: "apple-aqua-hig-controls",
      title: "Aqua HIG (retired) control metrics",
      section: "Text & search field heights",
      retrievedAt: "2026-05-30",
      platform: "macOS",
      osVersion: "26",
    },
    copyrightSafeSummary: "Historical Aqua-era regular search/text field height was 22pt (small 19, mini 15) — not a Tahoe hard target.",
  },
];

export const METRIC_WEIGHTS: Record<GuidelineConfidence, number> = {
  documented: 1.0,
  measuredNative: 1.0,
  derived: 0.8,
  historical: 0.3,
  estimated: 0.1,
};

export const CLASSIFICATION_SCORE: Record<Exclude<GuidelineClassification, "notApplicable">, number> = {
  withinBand: 1.0,
  nearBand: 0.5,
  outOfBand: 0.0,
  unmeasured: 0.0,
};

// --- Geometry helpers ------------------------------------------------------
export interface BoundsPt { x: number; y: number; width: number; height: number }
export interface CornerRadiiPt { topLeft: number; topRight: number; bottomRight: number; bottomLeft: number }
export type Corner = "topLeft" | "topRight" | "bottomRight" | "bottomLeft";

export interface NodeLike {
  name: unknown;
  type: unknown;
  parent?: unknown;
  bounds: BoundsPt;
  visualStyle?: unknown;
}

export interface GuidelineDeviation {
  metricId: string;
  source: "apple-documented" | "apple-derived" | "apple-measured-native" | "historical-apple" | "project-local";
  confidence: GuidelineConfidence;
  normativeStrength: NormativeStrength;
  nodeName: string;
  nodeType: string;
  parentName?: string | null;
  corner?: Corner;
  observedPt: number | null;
  targetPt: number | null;
  minPt?: number;
  maxPt?: number;
  deltaPt: number | null;
  deltaPct: number | null;
  tolerance: GuidelineTolerance;
  classification: GuidelineClassification;
  derivation?: { formula: string; inputs: Record<string, number | string | null> };
  failureReason?: string;
}

function num(value: unknown, fallback = 0): number {
  return typeof value === "number" && Number.isFinite(value) ? value : fallback;
}

export function radiiFromStyle(style: unknown): CornerRadiiPt | null {
  if (!style || typeof style !== "object") return null;
  const s = style as Record<string, unknown>;
  const raw = s.cornerRadius ?? s.radius;
  if (typeof raw === "number" && Number.isFinite(raw)) {
    return { topLeft: raw, topRight: raw, bottomRight: raw, bottomLeft: raw };
  }
  if (raw && typeof raw === "object") {
    const r = raw as Record<string, unknown>;
    return {
      topLeft: num(r.topLeft),
      topRight: num(r.topRight),
      bottomRight: num(r.bottomRight),
      bottomLeft: num(r.bottomLeft),
    };
  }
  return null;
}

const right = (b: BoundsPt) => b.x + b.width;
const bottom = (b: BoundsPt) => b.y + b.height;
const inside = (child: BoundsPt, parent: BoundsPt) =>
  child.x >= parent.x - 0.5 &&
  child.y >= parent.y - 0.5 &&
  right(child) <= right(parent) + 0.5 &&
  bottom(child) <= bottom(parent) + 0.5;

// Per-corner inset of a child within its parent (both in the same coordinate
// space). Returns the inset along each axis touching that corner.
export function cornerSeparation(child: BoundsPt, parent: BoundsPt, corner: Corner): { dx: number; dy: number } {
  const left = child.x - parent.x;
  const top = child.y - parent.y;
  const rightGap = right(parent) - right(child);
  const bottomGap = bottom(parent) - bottom(child);
  switch (corner) {
    case "topLeft": return { dx: left, dy: top };
    case "topRight": return { dx: rightGap, dy: top };
    case "bottomRight": return { dx: rightGap, dy: bottomGap };
    case "bottomLeft": return { dx: left, dy: bottomGap };
  }
}

// Container-ish node types eligible for concentric-radius analysis. Excludes
// controls (input/button/listitem) so a control floating in a bar does not
// generate bogus concentric "failures" against the bar's window-corner radius.
const CONCENTRIC_CONTAINER_TYPES = new Set(["window", "panel", "container", "card", "area", "list"]);

export function pixelQuantizationPt(scale: number | null): number {
  return scale && scale > 0 ? 1 / scale : 1;
}

export function allowedAbsPt(targetPt: number, tol: GuidelineTolerance, scale: number | null): number {
  const quant = 2 * pixelQuantizationPt(scale);
  const abs = tol.absPt ?? 0;
  const pct = tol.pct != null ? Math.abs(targetPt) * tol.pct : 0;
  return Math.max(abs, pct, quant);
}

export function classifyDeviation(
  deltaPt: number,
  targetPt: number,
  tol: GuidelineTolerance,
  scale: number | null,
): GuidelineClassification {
  const passBand = allowedAbsPt(targetPt, tol, scale);
  const nearBand = Math.max(
    tol.nearAbsPt ?? passBand * 2,
    tol.nearPct != null ? Math.abs(targetPt) * tol.nearPct : passBand * 2,
  );
  const absDelta = Math.abs(deltaPt);
  if (absDelta <= passBand) return "withinBand";
  if (absDelta <= nearBand) return "nearBand";
  return "outOfBand";
}

// Minimum-only metrics (padding/spacing floors): observed must be >= min.
export function classifyMinimum(
  observedPt: number,
  minPt: number,
  tol: GuidelineTolerance,
  scale: number | null,
): GuidelineClassification {
  const passBand = allowedAbsPt(minPt, tol, scale);
  const nearBand = Math.max(tol.nearAbsPt ?? passBand * 2, passBand * 2);
  if (observedPt >= minPt - passBand) return "withinBand";
  if (observedPt >= minPt - nearBand) return "nearBand";
  return "outOfBand";
}

function metric(id: string): GuidelineMetric {
  const m = APPLE_GUIDELINE_METRICS.find((entry) => entry.id === id);
  if (!m) throw new Error(`unknown guideline metric ${id}`);
  return m;
}

function sourceFor(confidence: GuidelineConfidence): GuidelineDeviation["source"] {
  switch (confidence) {
    case "documented": return "apple-documented";
    case "derived": return "apple-derived";
    case "measuredNative": return "apple-measured-native";
    case "historical": return "historical-apple";
    case "estimated": return "project-local";
  }
}

// --- Concentric-radius deviations ------------------------------------------
// For every container child meaningfully inset within a rounded container
// parent, verify Apple's concentric rule childRadius ≈ parentRadius − inset
// AND report the implied parent radius (parentRadius should be childRadius +
// inset). Skips flush / non-rounded / non-corner-proximate cases.
export function concentricRadiusDeviations(nodes: NodeLike[], scale: number | null): GuidelineDeviation[] {
  const m = metric("shape.concentric.childRadius");
  const byName = new Map<string, NodeLike>();
  for (const n of nodes) byName.set(String(n.name ?? ""), n);
  const out: GuidelineDeviation[] = [];
  const corners: Corner[] = ["topLeft", "topRight", "bottomRight", "bottomLeft"];
  const MIN_INSET = 2; // pt — below this the child is "flush", not concentric.

  for (const child of nodes) {
    const childType = String(child.type ?? "").toLowerCase();
    if (!CONCENTRIC_CONTAINER_TYPES.has(childType)) continue;
    const parentName = String(child.parent ?? "");
    const parent = byName.get(parentName);
    if (!parent) continue;
    const parentRadii = radiiFromStyle(parent.visualStyle);
    const childRadii = radiiFromStyle(child.visualStyle);
    if (!parentRadii || !childRadii) continue;
    if (!inside(child.bounds, parent.bounds)) continue;

    for (const corner of corners) {
      const sep = cornerSeparation(child.bounds, parent.bounds, corner);
      // Corner-proximate concentric inset: BOTH axes inset by >= MIN_INSET and
      // roughly equal (a true rounded inset wraps symmetrically at the corner).
      if (sep.dx < MIN_INSET || sep.dy < MIN_INSET) continue;
      const separationPt = Math.min(sep.dx, sep.dy);
      if (Math.abs(sep.dx - sep.dy) > separationPt) continue; // not corner-symmetric
      const parentR = parentRadii[corner];
      const childR = childRadii[corner];
      if (parentR <= 0 || childR <= 0) continue;

      const expectedChildR = Math.max(0, parentR - separationPt);
      const deltaPt = childR - expectedChildR;
      out.push({
        metricId: m.id,
        source: sourceFor(m.confidence),
        confidence: m.confidence,
        normativeStrength: m.normativeStrength,
        nodeName: String(child.name ?? ""),
        nodeType: childType,
        parentName,
        corner,
        observedPt: childR,
        targetPt: expectedChildR,
        deltaPt,
        deltaPct: expectedChildR !== 0 ? deltaPt / expectedChildR : null,
        tolerance: m.tolerance,
        classification: classifyDeviation(deltaPt, expectedChildR, m.tolerance, scale),
        derivation: {
          formula: "expectedChildRadiusPt = max(0, parentRadiusPt - separationPt); impliedParentRadiusPt = childRadiusPt + separationPt",
          inputs: { parentName, parentRadiusPt: parentR, separationPt, impliedParentRadiusPt: childR + separationPt },
        },
      });
    }
  }
  return out;
}

// --- Search-control internal padding ---------------------------------------
// Apple-derived soft band 8–12pt horizontal text inset. We can only PROVE this
// when the layout emits text/content bounds; otherwise it is an explicit
// `unmeasured` gap (the user's "input lacks padding" concern, honestly stated).
export function searchPaddingDeviations(nodes: NodeLike[], scale: number | null): GuidelineDeviation[] {
  const m = metric("control.searchField.textInset.horizontal");
  // Measured-native anchor: native regular NSTextField insets text 9pt (macOS 26.5).
  // Soft band context retained (8–12pt) for `minPt`/`maxPt` reporting.
  const nativeTargetPt = m.target.kind === "constant" ? m.target.valuePt : 9;
  const out: GuidelineDeviation[] = [];
  for (const node of nodes) {
    if (String(node.type ?? "").toLowerCase() !== "input") continue;
    const style = (node.visualStyle && typeof node.visualStyle === "object" ? node.visualStyle : {}) as Record<string, unknown>;
    const visual = (style.visualBounds as BoundsPt | undefined) ?? node.bounds;
    const text = style.textBounds as BoundsPt | undefined;
    const insets = style.contentInsets as { left?: number; right?: number } | undefined;
    let observed: number | null = null;
    if (text && typeof text === "object") {
      observed = Math.min(num(text.x) - visual.x, right(visual) - right(text));
    } else if (insets && typeof insets === "object") {
      observed = Math.min(num(insets.left), num(insets.right));
    }
    if (observed == null) {
      out.push({
        metricId: m.id,
        source: sourceFor(m.confidence),
        confidence: m.confidence,
        normativeStrength: m.normativeStrength,
        nodeName: String(node.name ?? ""),
        nodeType: "input",
        observedPt: null,
        targetPt: null,
        minPt: 8,
        maxPt: 12,
        deltaPt: null,
        deltaPct: null,
        tolerance: m.tolerance,
        classification: "unmeasured",
        failureReason: "input lacks contentInsets/textBounds evidence; cannot prove Apple-aligned internal padding",
      });
      continue;
    }
    const classification = classifyMinimum(observed, nativeTargetPt, m.tolerance, scale);
    out.push({
      metricId: m.id,
      source: sourceFor(m.confidence),
      confidence: m.confidence,
      normativeStrength: m.normativeStrength,
      nodeName: String(node.name ?? ""),
      nodeType: "input",
      observedPt: observed,
      targetPt: nativeTargetPt,
      minPt: 8,
      maxPt: 12,
      deltaPt: observed - nativeTargetPt,
      deltaPct: (observed - nativeTargetPt) / nativeTargetPt,
      tolerance: m.tolerance,
      classification,
    });
  }
  return out;
}

// --- Capsule radius (large/x-large controls only) --------------------------
export function capsuleRadiusDeviations(nodes: NodeLike[], scale: number | null): GuidelineDeviation[] {
  const m = metric("shape.capsule.radius");
  const out: GuidelineDeviation[] = [];
  for (const node of nodes) {
    const style = (node.visualStyle && typeof node.visualStyle === "object" ? node.visualStyle : {}) as Record<string, unknown>;
    const controlSize = String(style.controlSize ?? "").toLowerCase();
    const isCapsule = style.shape === "capsule" || controlSize === "large" || controlSize === "extraLarge".toLowerCase();
    if (!isCapsule) continue;
    const radii = radiiFromStyle(style);
    if (!radii) continue;
    const observed = radii.topLeft;
    const target = node.bounds.height / 2;
    const deltaPt = observed - target;
    out.push({
      metricId: m.id,
      source: sourceFor(m.confidence),
      confidence: m.confidence,
      normativeStrength: m.normativeStrength,
      nodeName: String(node.name ?? ""),
      nodeType: String(node.type ?? ""),
      observedPt: observed,
      targetPt: target,
      deltaPt,
      deltaPct: target !== 0 ? deltaPt / target : null,
      tolerance: m.tolerance,
      classification: classifyDeviation(deltaPt, target, m.tolerance, scale),
      derivation: { formula: "radiusPt = heightPt / 2", inputs: { heightPt: node.bounds.height } },
    });
  }
  return out;
}

export interface AppleGuidelineConformanceBlock {
  schemaVersion: 1;
  unit: "pt";
  backingScaleFactor: number | null;
  constants: GuidelineMetric[];
  deviations: GuidelineDeviation[];
  failures: GuidelineDeviation[];
  nearMisses: GuidelineDeviation[];
  unmeasured: GuidelineDeviation[];
  score: {
    weightedScore: number;
    hardPass: boolean;
    hardFailureCount: number;
    softFailureCount: number;
    measuredNodeCount: number;
    unmeasuredCount: number;
  };
}

export function appleGuidelineConformance(nodes: NodeLike[], scale: number | null): AppleGuidelineConformanceBlock {
  const deviations: GuidelineDeviation[] = [
    ...concentricRadiusDeviations(nodes, scale),
    ...searchPaddingDeviations(nodes, scale),
    ...capsuleRadiusDeviations(nodes, scale),
  ];
  const failures = deviations.filter((d) => d.classification === "outOfBand");
  const nearMisses = deviations.filter((d) => d.classification === "nearBand");
  const unmeasured = deviations.filter((d) => d.classification === "unmeasured");

  let weightNum = 0;
  let weightDen = 0;
  for (const d of deviations) {
    if (d.classification === "notApplicable") continue;
    const w = METRIC_WEIGHTS[d.confidence];
    weightNum += w * CLASSIFICATION_SCORE[d.classification];
    weightDen += w;
  }
  const hardFailures = failures.filter((d) => d.normativeStrength === "hard");
  const hardUnmeasured = unmeasured.filter((d) => d.normativeStrength === "hard");
  const softFailures = failures.filter((d) => d.normativeStrength !== "hard");

  return {
    schemaVersion: 1,
    unit: "pt",
    backingScaleFactor: scale,
    constants: APPLE_GUIDELINE_METRICS,
    deviations,
    failures,
    nearMisses,
    unmeasured,
    score: {
      weightedScore: weightDen > 0 ? weightNum / weightDen : 1,
      hardPass: hardFailures.length === 0 && hardUnmeasured.length === 0,
      hardFailureCount: hardFailures.length,
      softFailureCount: softFailures.length,
      measuredNodeCount: deviations.filter((d) => d.observedPt != null).length,
      unmeasuredCount: unmeasured.length,
    },
  };
}
