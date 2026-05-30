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
  category: "cornerRadius" | "concentricity" | "controlSize" | "padding" | "spacing" | "hitTarget" | "typography";
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
    id: "layout.footer.itemGap",
    category: "spacing",
    confidence: "derived",
    normativeStrength: "soft",
    // Footer hint chips are NOT regular buttons, so Apple's 16pt hard regular-
    // control gap is a reference ceiling, not a hard floor. The soft floor is the
    // ~12pt bezel-padding heuristic: adjacent compact controls should have at
    // least that much breathing room. Below it reads as cramped.
    target: { kind: "minimum", minPt: 12 },
    tolerance: { absPt: 1, nearAbsPt: 3 },
    derivation: {
      fromMetricIds: ["layout.regularButton.minimumGap", "layout.bezelElement.padding"],
      formula: "footer item gap should be >= ~12pt (bezel-padding floor); Apple's hard regular-control gap is 16pt",
      notes: "Soft because footer hint chips are compact, non-bezeled controls Apple's HIG does not size explicitly.",
    },
    copyrightSafeSummary: "Derived (soft): adjacent compact footer controls should keep ~12pt+ of gap; 6pt reads as cramped vs Apple's 16pt regular-control gap.",
  },
  {
    id: "macos.window.toolbarRadius.nativeBaseline",
    category: "cornerRadius",
    confidence: "measuredNative",
    normativeStrength: "soft",
    // Measured: native Tahoe titled / full-size-content / glass windows all round
    // at 15pt on macOS 26.5 (borderless windows get no system mask = 0pt). Modeled
    // as a soft MINIMUM: a launcher panel should be AT LEAST as rounded as native;
    // being rounder is not a violation. Our 22pt token clears this (refutes the
    // "not round enough" concern).
    target: { kind: "minimum", minPt: 15 },
    tolerance: { absPt: 1, pct: 0.05, nearAbsPt: 2 },
    nativeMeasurement: {
      probeId: "macos26-nswindow-mask-corner-radius",
      probeSource: "scripts/devtools/tahoe_window_mask_probe.swift",
      receiptPath: "artifacts/liquid-glass/receipts/tahoe-window-mask-baseline.json",
      osVersion: "26.5",
      measuredAt: "2026-05-30",
      control: "NSWindow (titled / fullSizeContent / glass)",
      field: "cornerRadiusPt",
    },
    copyrightSafeSummary: "Native Tahoe titled/glass windows round at 15pt (measured, macOS 26.5); a launcher should be at least that rounded.",
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
  {
    id: "typography.native.body.fontSize",
    category: "typography",
    confidence: "measuredNative",
    // SOFT: macOS Body / regular control-content text is 13pt, but a launcher's
    // hero search field intentionally runs larger (Spotlight-style). So divergence
    // above 13pt is reported, not a hard failure. Result-row body text (a later
    // slice) is where 13pt should hold tightly.
    normativeStrength: "soft",
    target: { kind: "constant", valuePt: 13 },
    tolerance: { absPt: 1, nearAbsPt: 2 },
    nativeMeasurement: {
      probeId: "macos26-regular-control-content-font-size",
      probeSource: "scripts/devtools/tahoe_native_baseline.swift",
      receiptPath: "artifacts/liquid-glass/receipts/tahoe-native-baseline.json",
      osVersion: "26.5",
      measuredAt: "2026-05-30",
      control: "NSFont.controlContentFont(.regular) / NSSearchField regular",
      field: "fontMetrics.regularControlContentFont.pointSizePt",
    },
    copyrightSafeSummary: "Native regular macOS control/body text is 13pt (measured, macOS 26.5); a launcher hero search field may run larger by design.",
  },
  {
    id: "typography.native.body.lineHeight",
    category: "typography",
    confidence: "measuredNative",
    normativeStrength: "soft",
    target: { kind: "constant", valuePt: 16 },
    tolerance: { absPt: 1, nearAbsPt: 3 },
    nativeMeasurement: {
      probeId: "macos26-regular-control-content-font-lineheight",
      probeSource: "scripts/devtools/tahoe_native_baseline.swift",
      receiptPath: "artifacts/liquid-glass/receipts/tahoe-native-baseline.json",
      osVersion: "26.5",
      measuredAt: "2026-05-30",
      control: "NSLayoutManager.defaultLineHeight(for: regular control content font)",
      field: "fontMetrics.regularControlContentFont.defaultLineHeightPt",
    },
    copyrightSafeSummary: "Native default line height for 13pt regular control/body text is 16pt (measured, macOS 26.5).",
  },
  {
    id: "typography.native.input.fontWeight",
    category: "typography",
    confidence: "measuredNative",
    // HARD: native search/input text is Regular (weight trait 0 ~= 400). Apple
    // reserves Semibold for emphasis, not for ordinary editable field text. Our
    // launcher search input must NOT render bold/semibold. Compared in GPUI
    // numeric weight space (Regular=400, Medium=500, Semibold=600).
    normativeStrength: "hard",
    target: { kind: "constant", valuePt: 400 },
    tolerance: { absPt: 50, nearAbsPt: 150 },
    nativeMeasurement: {
      probeId: "macos26-regular-control-content-font-weight",
      probeSource: "scripts/devtools/tahoe_native_baseline.swift",
      receiptPath: "artifacts/liquid-glass/receipts/tahoe-native-baseline.json",
      osVersion: "26.5",
      measuredAt: "2026-05-30",
      control: "NSSearchField / NSTextField regular font",
      field: "fontMetrics.regularControlContentFont.weightTrait (0 == Regular == 400)",
    },
    copyrightSafeSummary: "Native regular search/input text is Regular weight (trait 0 == 400); Semibold is reserved for emphasis, so input text must not be bold.",
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
  boxModel?: unknown;
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
  // This is an EXACT native inset, classified with classifyDeviation (penalizing
  // both too-little AND too-much), NOT a minimum — a 24pt inset does not "match"
  // the 9pt native text-field inset. (Oracle review tahoe-guideline-review-remaining:
  // the separate unbezeled-lane edge-distance minimum ~24pt is later-polish.)
  // Soft band context retained (8–12pt) for `minPt`/`maxPt` reporting only.
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
    const classification = classifyDeviation(observed - nativeTargetPt, nativeTargetPt, m.tolerance, scale);
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

// --- Footer item spacing ----------------------------------------------------
// The user's "footer lacks padding" concern, measured: a footer rail that
// declares its inter-item gap (boxModel.gap) is classified against the soft
// ~12pt floor. SOFT — footer hint chips are compact, non-bezeled controls.
function footerGapFromNode(node: NodeLike): number | null {
  const box = node.boxModel && typeof node.boxModel === "object" ? (node.boxModel as Record<string, unknown>) : null;
  if (box && typeof box.gap === "number" && Number.isFinite(box.gap)) return box.gap;
  return null;
}

export function footerSpacingDeviations(nodes: NodeLike[], scale: number | null): GuidelineDeviation[] {
  const m = metric("layout.footer.itemGap");
  const out: GuidelineDeviation[] = [];
  for (const node of nodes) {
    const name = String(node.name ?? "");
    const type = String(node.type ?? "").toLowerCase();
    const isFooter = /footer/i.test(name) || type === "footer";
    if (!isFooter) continue;
    const gap = footerGapFromNode(node);
    if (gap == null) {
      out.push({
        metricId: m.id,
        source: sourceFor(m.confidence),
        confidence: m.confidence,
        normativeStrength: m.normativeStrength,
        nodeName: name,
        nodeType: String(node.type ?? ""),
        observedPt: null,
        targetPt: 12,
        minPt: 12,
        deltaPt: null,
        deltaPct: null,
        tolerance: m.tolerance,
        classification: "unmeasured",
        failureReason: "footer node lacks boxModel.gap; cannot measure inter-item spacing",
      });
      continue;
    }
    out.push({
      metricId: m.id,
      source: sourceFor(m.confidence),
      confidence: m.confidence,
      normativeStrength: m.normativeStrength,
      nodeName: name,
      nodeType: String(node.type ?? ""),
      observedPt: gap,
      targetPt: 12,
      minPt: 12,
      deltaPt: gap - 12,
      deltaPct: (gap - 12) / 12,
      tolerance: m.tolerance,
      classification: classifyMinimum(gap, 12, m.tolerance, scale),
      derivation: {
        formula: "footer item gap >= ~12pt soft floor (Apple hard regular-control gap = 16pt)",
        inputs: { observedGapPt: gap, softFloorPt: 12, appleRegularControlGapPt: 16 },
      },
    });
  }
  return out;
}

// --- Window corner radius vs measured native baseline -----------------------
// The user's "window corners aren't rounded enough" concern, measured against
// the native Tahoe window mask (15pt). Modeled as a soft MINIMUM: a launcher
// panel should be at least as rounded as native; rounder is fine. So our 22pt
// token PASSES (refutes the concern); a value below 15pt would be outOfBand.
export function windowRadiusDeviations(nodes: NodeLike[], scale: number | null): GuidelineDeviation[] {
  const m = metric("macos.window.toolbarRadius.nativeBaseline");
  const nativeFloorPt = m.target.kind === "minimum" ? m.target.minPt : 15;
  const out: GuidelineDeviation[] = [];
  for (const node of nodes) {
    if (String(node.type ?? "").toLowerCase() !== "window") continue;
    const radii = radiiFromStyle(node.visualStyle);
    if (!radii) continue;
    const observed = radii.topLeft;
    out.push({
      metricId: m.id,
      source: sourceFor(m.confidence),
      confidence: m.confidence,
      normativeStrength: m.normativeStrength,
      nodeName: String(node.name ?? ""),
      nodeType: String(node.type ?? ""),
      observedPt: observed,
      targetPt: nativeFloorPt,
      minPt: nativeFloorPt,
      deltaPt: observed - nativeFloorPt,
      deltaPct: nativeFloorPt !== 0 ? (observed - nativeFloorPt) / nativeFloorPt : null,
      tolerance: m.tolerance,
      classification: classifyMinimum(observed, nativeFloorPt, m.tolerance, scale),
      derivation: {
        formula: "window radius >= native Tahoe window mask (15pt measured); rounder is acceptable",
        inputs: { observedRadiusPt: observed, nativeFloorPt, direction: observed >= nativeFloorPt ? "atOrAboveNative" : "belowNative" },
      },
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

// --- Typography deviations --------------------------------------------------
// For nodes that emit `visualStyle.typography`, classify rendered text against
// the measured-native macOS baselines. The launcher's hero search input is
// expected to satisfy the HARD weight metric (input text must be Regular, not
// bold) while diverging on the SOFT size/line-height metrics (a Spotlight-style
// hero field legitimately runs larger than 13pt body). Honest by construction:
// the size divergence is surfaced as a soft finding, never silently passed.
export interface TypographyStyle {
  role?: string;
  fontFamily?: string;
  fontSizePt?: number;
  fontWeight?: string;
  fontWeightNumeric?: number;
  lineHeightPt?: number;
  textAlign?: string;
}

export function typographyFromStyle(style: unknown): TypographyStyle | null {
  if (!style || typeof style !== "object") return null;
  const raw = (style as Record<string, unknown>).typography;
  if (!raw || typeof raw !== "object") return null;
  return raw as TypographyStyle;
}

// Map a GPUI/CSS-ish weight name to its numeric weight when the explicit numeric
// field is absent. Regular=400, Medium=500, Semibold=600, Bold=700.
function weightNumeric(typo: TypographyStyle): number | null {
  if (typeof typo.fontWeightNumeric === "number") return typo.fontWeightNumeric;
  const byName: Record<string, number> = {
    thin: 100, extralight: 200, light: 300, normal: 400, regular: 400,
    medium: 500, semibold: 600, bold: 700, extrabold: 800, black: 900,
  };
  const key = String(typo.fontWeight ?? "").toLowerCase();
  return key in byName ? byName[key] : null;
}

export function typographyDeviations(nodes: NodeLike[], scale: number | null): GuidelineDeviation[] {
  const out: GuidelineDeviation[] = [];
  for (const node of nodes) {
    const typo = typographyFromStyle(node.visualStyle);
    if (!typo) continue;
    const nodeName = String(node.name ?? "");
    const nodeType = String(node.type ?? "");

    const push = (
      m: GuidelineMetric,
      observed: number | null,
      target: number,
      classification: GuidelineClassification,
      failureReason?: string,
    ) => {
      out.push({
        metricId: m.id,
        source: sourceFor(m.confidence),
        confidence: m.confidence,
        normativeStrength: m.normativeStrength,
        nodeName,
        nodeType,
        observedPt: observed,
        targetPt: target,
        deltaPt: observed == null ? null : observed - target,
        deltaPct: observed == null ? null : (observed - target) / target,
        tolerance: m.tolerance,
        classification,
        ...(failureReason ? { failureReason } : {}),
      });
    };

    // HARD: input/search text must be Regular weight (not bold/semibold).
    const wMetric = metric("typography.native.input.fontWeight");
    const wTarget = wMetric.target.kind === "constant" ? wMetric.target.valuePt : 400;
    const wObserved = weightNumeric(typo);
    if (wObserved == null) {
      push(wMetric, null, wTarget, "unmeasured", "node typography lacks fontWeight/fontWeightNumeric evidence");
    } else {
      push(wMetric, wObserved, wTarget, classifyDeviation(wObserved - wTarget, wTarget, wMetric.tolerance, scale));
    }

    // SOFT: body/control font size baseline 13pt; a hero search input may exceed it.
    const sMetric = metric("typography.native.body.fontSize");
    const sTarget = sMetric.target.kind === "constant" ? sMetric.target.valuePt : 13;
    if (typeof typo.fontSizePt === "number") {
      push(sMetric, typo.fontSizePt, sTarget, classifyDeviation(typo.fontSizePt - sTarget, sTarget, sMetric.tolerance, scale));
    } else {
      push(sMetric, null, sTarget, "unmeasured", "node typography lacks fontSizePt evidence");
    }

    // SOFT: body/control line height baseline 16pt.
    const lMetric = metric("typography.native.body.lineHeight");
    const lTarget = lMetric.target.kind === "constant" ? lMetric.target.valuePt : 16;
    if (typeof typo.lineHeightPt === "number") {
      push(lMetric, typo.lineHeightPt, lTarget, classifyDeviation(typo.lineHeightPt - lTarget, lTarget, lMetric.tolerance, scale));
    } else {
      push(lMetric, null, lTarget, "unmeasured", "node typography lacks lineHeightPt evidence");
    }
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
    ...footerSpacingDeviations(nodes, scale),
    ...windowRadiusDeviations(nodes, scale),
    ...capsuleRadiusDeviations(nodes, scale),
    ...typographyDeviations(nodes, scale),
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
