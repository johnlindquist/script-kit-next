// Deterministic math proof for the Apple-guideline conformance engine.
// Pins the exact concentric-radius failure the user suspects (Oracle session
// `tahoe-apple-guideline-metrics`): a child rounded panel inset 16pt inside a
// parent with radius 22 implies a parent radius of 16 + 16 = 32, so the
// observed 22 is out of band by 10pt. Run: bun test scripts/devtools/apple-guideline-constants.test.ts

import { expect, test } from "bun:test";
import {
  appleGuidelineConformance,
  classifyDeviation,
  classifyMinimum,
  concentricRadiusDeviations,
  searchPaddingDeviations,
  type NodeLike,
} from "./apple-guideline-constants";

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

test("conformance rollup surfaces a hard failure and never reports hardPass when one exists", () => {
  const block = appleGuidelineConformance(CONCENTRIC_FIXTURE, 2);
  expect(block.score.hardFailureCount).toBeGreaterThan(0);
  expect(block.score.hardPass).toBe(false);
  expect(block.unit).toBe("pt");
  expect(block.backingScaleFactor).toBe(2);
});
