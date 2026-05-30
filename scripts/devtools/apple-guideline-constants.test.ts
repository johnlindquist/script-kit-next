// Deterministic math proof for the Apple-guideline conformance engine.
// Pins the exact concentric-radius failure the user suspects (Oracle session
// `tahoe-apple-guideline-metrics`): a child rounded panel inset 16pt inside a
// parent with radius 22 implies a parent radius of 16 + 16 = 32, so the
// observed 22 is out of band by 10pt. Run: bun test scripts/devtools/apple-guideline-constants.test.ts

import { expect, test } from "bun:test";
import {
  APPLE_GUIDELINE_METRICS,
  appleGuidelineConformance,
  classifyDeviation,
  classifyMinimum,
  concentricRadiusDeviations,
  footerSpacingDeviations,
  searchPaddingDeviations,
  windowRadiusDeviations,
  type NodeLike,
} from "./apple-guideline-constants";
import nativeBaseline from "../../artifacts/liquid-glass/receipts/tahoe-native-baseline.json";
import windowMaskBaseline from "../../artifacts/liquid-glass/receipts/tahoe-window-mask-baseline.json";

const CONCENTRIC_FIXTURE: NodeLike[] = [
  {
    name: "Window",
    type: "window",
    parent: null,
    bounds: { x: 0, y: 0, width: 750, height: 480 },
    visualStyle: { cornerRadius: { topLeft: 22, topRight: 22, bottomRight: 22, bottomLeft: 22 } },
  },
  {
    name: "InnerPanel",
    type: "panel",
    parent: "Window",
    bounds: { x: 16, y: 16, width: 718, height: 448 },
    visualStyle: { cornerRadius: { topLeft: 16, topRight: 16, bottomRight: 16, bottomLeft: 16 } },
  },
];

test("concentric math flags a 16pt-inset r16 child under a r22 parent as outOfBand (implied parent r32, delta -10)", () => {
  const deviations = concentricRadiusDeviations(CONCENTRIC_FIXTURE, 2);
  expect(deviations.length).toBe(4); // one per corner, all symmetric
  for (const d of deviations) {
    // expectedChildRadius = parentRadius(22) - separation(16) = 6; observed 16 => delta +10
    expect(d.targetPt).toBe(6);
    expect(d.observedPt).toBe(16);
    expect(d.deltaPt).toBe(10);
    expect(d.classification).toBe("outOfBand");
    expect(d.normativeStrength).toBe("hard");
    // implied parent radius the user actually cares about: child(16) + inset(16) = 32
    expect(d.derivation?.inputs.impliedParentRadiusPt).toBe(32);
    expect(d.derivation?.inputs.separationPt).toBe(16);
  }
});

test("a flush (zero-inset) child generates NO concentric failure", () => {
  const flush: NodeLike[] = [
    { name: "Window", type: "window", parent: null, bounds: { x: 0, y: 0, width: 750, height: 480 }, visualStyle: { cornerRadius: 22 } },
    { name: "Header", type: "container", parent: "Window", bounds: { x: 0, y: 0, width: 750, height: 45 }, visualStyle: { cornerRadius: 16 } },
  ];
  // Header is flush to the top/left/right edges -> not a concentric inset.
  expect(concentricRadiusDeviations(flush, 2).length).toBe(0);
});

test("search input with no text/content metrics is reported as an explicit unmeasured gap", () => {
  const nodes: NodeLike[] = [
    { name: "SearchInput", type: "input", parent: "Header", bounds: { x: 16, y: 11, width: 498, height: 22 }, visualStyle: { cornerRadius: 14 } },
  ];
  const deviations = searchPaddingDeviations(nodes, 2);
  expect(deviations.length).toBe(1);
  expect(deviations[0].classification).toBe("unmeasured");
  expect(deviations[0].failureReason).toContain("contentInsets/textBounds");
});

test("search input WITH measured text inset below 8pt is outOfBand", () => {
  const nodes: NodeLike[] = [
    {
      name: "SearchInput",
      type: "input",
      parent: "Header",
      bounds: { x: 16, y: 11, width: 498, height: 22 },
      visualStyle: { cornerRadius: 14, visualBounds: { x: 16, y: 11, width: 498, height: 22 }, contentInsets: { left: 4, right: 4, top: 3, bottom: 3 } },
    },
  ];
  const d = searchPaddingDeviations(nodes, 2)[0];
  expect(d.observedPt).toBe(4);
  expect(d.classification).toBe("outOfBand");
});

test("the real main-launcher SearchInput (0pt flush text inset) is outOfBand vs the 9pt native target", () => {
  // Mirrors build_layout_info.rs SearchInput: 22pt-tall input whose text is a
  // flush flex_1 child (0pt horizontal inset, CURSOR_MARGIN_Y=2pt vertical).
  const searchInput: NodeLike[] = [
    {
      name: "SearchInput",
      type: "input",
      parent: "Header",
      bounds: { x: 16, y: 11, width: 498, height: 22 },
      visualStyle: { cornerRadius: 14, contentInsets: { top: 2, right: 0, bottom: 2, left: 0 } },
    },
  ];
  const d = searchPaddingDeviations(searchInput, 2)[0];
  expect(d.observedPt).toBe(0);
  expect(d.targetPt).toBe(9);
  expect(d.deltaPt).toBe(-9);
  expect(d.classification).toBe("outOfBand");
  expect(d.normativeStrength).toBe("hard");
});

test("the real main-launcher MainFooter (6pt item gap) is outOfBand vs the soft 12pt floor (SOFT, not hard)", () => {
  // Mirrors build_layout_info.rs MainFooter: a footer panel whose inter-item gap
  // is FOOTER_ACTION_ITEM_GAP_PX = 6pt, carried as boxModel.gap.
  const footer: NodeLike[] = [
    {
      name: "MainFooter",
      type: "panel",
      parent: "Window",
      bounds: { x: 0, y: 458, width: 750, height: 22 },
      visualStyle: { cornerRadius: 16, contentInsets: { top: 2, right: 14, bottom: 2, left: 14 } },
      boxModel: { gap: 6 },
    },
  ];
  const d = footerSpacingDeviations(footer, 2)[0];
  expect(d.observedPt).toBe(6);
  expect(d.targetPt).toBe(12);
  expect(d.deltaPt).toBe(-6);
  expect(d.classification).toBe("outOfBand");
  // Honest: footer hint chips are NOT regular buttons, so this is SOFT.
  expect(d.normativeStrength).toBe("soft");
});

test("a footer node with no boxModel.gap is an explicit unmeasured gap", () => {
  const footer: NodeLike[] = [
    { name: "MainFooter", type: "panel", parent: "Window", bounds: { x: 0, y: 458, width: 750, height: 22 } },
  ];
  const d = footerSpacingDeviations(footer, 2)[0];
  expect(d.classification).toBe("unmeasured");
  expect(d.failureReason).toContain("boxModel.gap");
});

test("a footer with a roomy 14pt gap is withinBand (refutes 'cramped' once spacing is adequate)", () => {
  const footer: NodeLike[] = [
    { name: "MainFooter", type: "panel", parent: "Window", bounds: { x: 0, y: 458, width: 750, height: 22 }, boxModel: { gap: 14 } },
  ];
  expect(footerSpacingDeviations(footer, 2)[0].classification).toBe("withinBand");
});

test("our 22pt window radius is withinBand vs the measured 15pt native floor (REFUTES 'not round enough')", () => {
  const window: NodeLike[] = [
    { name: "Window", type: "window", parent: null, bounds: { x: 0, y: 0, width: 750, height: 480 }, visualStyle: { cornerRadius: 22 } },
  ];
  const d = windowRadiusDeviations(window, 2)[0];
  expect(d.observedPt).toBe(22);
  expect(d.targetPt).toBe(15);
  expect(d.deltaPt).toBe(7); // 7pt rounder than native
  expect(d.classification).toBe("withinBand");
  expect(d.derivation?.inputs.direction).toBe("atOrAboveNative");
});

test("a hypothetical 8pt window radius WOULD be outOfBand (the check can still fail below native)", () => {
  const window: NodeLike[] = [
    { name: "Window", type: "window", parent: null, bounds: { x: 0, y: 0, width: 750, height: 480 }, visualStyle: { cornerRadius: 8 } },
  ];
  expect(windowRadiusDeviations(window, 2)[0].classification).toBe("outOfBand");
});

test("the window-mask native baseline receipt pins native Tahoe windows at 15pt", () => {
  const titled = (windowMaskBaseline.styles as Array<Record<string, unknown>>).find(
    (s) => s.style === "titledStandardWindow",
  );
  expect(titled?.cornerRadiusPt).toBe(15);
  expect(windowMaskBaseline.ourWindowRadiusTokenPt).toBe(22);
});

test("classifyDeviation respects retina pixel-quantization tolerance", () => {
  // target 6, tolerance absPt 1 on a 2x display -> passBand max(1, 2*0.5)=1
  expect(classifyDeviation(0.5, 6, { absPt: 1, nearAbsPt: 2 }, 2)).toBe("withinBand");
  expect(classifyDeviation(1.5, 6, { absPt: 1, nearAbsPt: 2 }, 2)).toBe("nearBand");
  expect(classifyDeviation(10, 6, { absPt: 1, nearAbsPt: 2 }, 2)).toBe("outOfBand");
});

test("classifyMinimum treats a value below the floor as outOfBand", () => {
  expect(classifyMinimum(12, 12, { absPt: 1 }, 2)).toBe("withinBand");
  expect(classifyMinimum(6, 12, { absPt: 1, nearAbsPt: 3 }, 2)).toBe("outOfBand");
});

test("measured-native metrics are pinned to the on-disk native baseline receipt (re-measure if these drift)", () => {
  const regularText = (nativeBaseline.textFields as Array<Record<string, number | string>>).find(
    (f) => f.controlSize === "regular",
  );
  expect(regularText).toBeTruthy();
  expect(regularText!.contentHorizontalInsetPt).toBe(9);
  expect(regularText!.contentVerticalInsetPt).toBe(3);
  expect(regularText!.intrinsicHeightPt).toBe(22);
  expect((nativeBaseline.glassEffectView as Record<string, unknown>).defaultCornerRadiusPt).toBe(8);

  const byId = (id: string) => {
    const m = APPLE_GUIDELINE_METRICS.find((entry) => entry.id === id);
    expect(m, `metric ${id} must exist`).toBeTruthy();
    return m!;
  };
  // Each measured-native constant must equal the number in the receipt.
  const hInset = byId("control.searchField.textInset.horizontal");
  expect(hInset.confidence).toBe("measuredNative");
  expect(hInset.target).toEqual({ kind: "constant", valuePt: 9 });
  expect(hInset.nativeMeasurement?.receiptPath).toContain("tahoe-native-baseline.json");

  const vInset = byId("control.searchField.textInset.vertical");
  expect(vInset.target).toEqual({ kind: "constant", valuePt: 3 });

  const height = byId("control.regular.height");
  expect(height.target).toEqual({ kind: "constant", valuePt: 22 });

  const glass = byId("macos.glassEffectView.defaultRadius");
  expect(glass.confidence).toBe("measuredNative");
  expect(glass.target).toEqual({ kind: "constant", valuePt: 8 });

  // Every measuredNative metric must cite a re-runnable swift probe + receipt.
  for (const m of APPLE_GUIDELINE_METRICS.filter((e) => e.confidence === "measuredNative" && e.target.kind === "constant")) {
    expect(m.nativeMeasurement, `${m.id} must carry native-measurement provenance`).toBeTruthy();
    expect(m.nativeMeasurement!.probeSource).toContain("tahoe_native_baseline.swift");
    expect(m.nativeMeasurement!.osVersion).toBe("26.5");
  }
});

test("a regular input measured at the native 9pt inset is withinBand; below it is outOfBand", () => {
  const aligned: NodeLike[] = [
    { name: "SearchInput", type: "input", parent: "Header", bounds: { x: 16, y: 11, width: 498, height: 22 }, visualStyle: { cornerRadius: 14, contentInsets: { left: 9, right: 9, top: 3, bottom: 3 } } },
  ];
  const a = searchPaddingDeviations(aligned, 2)[0];
  expect(a.observedPt).toBe(9);
  expect(a.targetPt).toBe(9);
  expect(a.classification).toBe("withinBand");
});

test("conformance rollup surfaces a hard failure and never reports hardPass when one exists", () => {
  const block = appleGuidelineConformance(CONCENTRIC_FIXTURE, 2);
  expect(block.score.hardFailureCount).toBeGreaterThan(0);
  expect(block.score.hardPass).toBe(false);
  expect(block.unit).toBe("pt");
  expect(block.backingScaleFactor).toBe(2);
});
