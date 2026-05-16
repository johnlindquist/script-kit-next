#!/usr/bin/env bun

const classifications = [
  "ok",
  "reproduced",
  "fixed",
  "not-reproduced",
  "blocked-by-missing-primitive",
  "blocked-by-target-ambiguity",
  "blocked-by-stale-generation",
  "blocked-by-unsafe-operation",
  "blocked-by-permission",
  "blocked-by-real-data-risk",
  "blocked-by-native-escalation-required",
  "blocked-by-fixture-only",
  "blocked-by-timeout",
];

const receiptEnvelopeFields = [
  "schemaVersion",
  "tool",
  "command",
  "invocationId",
  "sessionId",
  "repo",
  "startedAt",
  "endedAt",
  "durationMs",
  "target",
  "preconditions",
  "result",
  "assertions",
  "classification",
  "warnings",
  "errors",
  "redaction",
];

const targetIdentityFields = [
  "requestedTarget.selector",
  "requestedTarget.strict",
  "resolvedTarget.automationId",
  "resolvedTarget.stableTargetId",
  "resolvedTarget.nativeWindowId",
  "resolvedTarget.axWindowId",
  "resolvedTarget.pid",
  "resolvedTarget.targetKind",
  "resolvedTarget.hostKind",
  "resolvedTarget.parentAutomationId",
  "resolvedTarget.openerAutomationId",
  "resolvedTarget.surfaceKind",
  "resolvedTarget.appViewVariant",
  "resolvedTarget.nativeFooterSurface",
  "resolvedTarget.surfaceFamily",
  "resolvedTarget.routeId",
  "resolvedTarget.routeStack",
  "resolvedTarget.targetGeneration",
  "resolvedTarget.surfaceGeneration",
  "resolvedTarget.dataGeneration",
  "resolvedTarget.bounds",
  "resolvedTarget.screenId",
  "resolvedTarget.zOrder",
  "resolvedTarget.visible",
  "resolvedTarget.frontmost",
  "resolvedTarget.focused",
  "resolvedTarget.screenshotIdentity",
  "resolvedTarget.strictTargetMatch",
  "resolvedTarget.ambiguity",
];

const primitiveSchemas = [
  {
    primitive: "devtools.targets.inspect",
    requiredResultFields: targetIdentityFields,
    failClosedWhen: ["strictTargetMatch is false", "target is hidden for visual proof", "targetGeneration changes during measurement"],
  },
  {
    primitive: "devtools.surface.inspect",
    requiredResultFields: [
      "contract.surfaceKind",
      "contract.appViewVariant",
      "contract.nativeFooterSurface",
      "contract.focusPolicy",
      "contract.keyboardPolicy",
      "contract.actionsPolicy",
      "contract.proofPolicy",
      "contract.visualPolicy",
      "contract.dismissPolicy",
      "runtime.activeFooterSurface",
      "runtime.focusedSemanticId",
      "runtime.selectedSemanticId",
      "runtime.capabilities",
      "runtime.missingPrimitives",
    ],
    failClosedWhen: ["surfaceKind mismatches requested surface", "dismissPolicy is missing", "runtime generation is stale"],
  },
  {
    primitive: "devtools.elements.snapshot",
    requiredResultFields: [
      "semanticSurface",
      "semanticSurfaceVersion",
      "nodes[].semanticId",
      "nodes[].role",
      "nodes[].label",
      "nodes[].actions",
      "nodes[].owner",
      "nodes[].bounds",
      "selectedSemanticId",
      "focusedSemanticId",
      "warnings",
    ],
    failClosedWhen: ["semantic ids are missing or duplicated", "focused/selected ids are absent when the surface requires them"],
  },
  {
    primitive: "devtools.layout.measure",
    requiredResultFields: [
      "viewportRect",
      "windowRect",
      "regions",
      "nodes[].bounds",
      "nodes[].clipped",
      "nodes[].overlaps",
      "resizePressure.windowCanGrow",
      "resizePressure.overflowY",
      "resizePressure.clippedNodeCount",
      "resizePressure.overlapCount",
      "resizePressure.pressureScore",
    ],
    failClosedWhen: ["region bounds are missing", "target identity is not strict", "layout generation changes mid-measurement"],
  },
  {
    primitive: "devtools.act",
    requiredResultFields: [
      "actionId",
      "actionKind",
      "targetBefore",
      "input",
      "safety",
      "expected",
      "targetAfter",
      "visibleResult",
      "result",
    ],
    failClosedWhen: ["targetBefore/targetAfter are missing", "destructive action lacks confirmation", "native escalation is used without safety reason"],
  },
  {
    primitive: "devtools.compare.redgreen",
    requiredResultFields: [
      "redReceiptIds",
      "greenReceiptIds",
      "samePrimitiveStack",
      "sameUserPath",
      "sameTargetSelector",
      "targetIdentityComparable",
      "assertions",
      "classification",
    ],
    failClosedWhen: ["samePrimitiveStack is false", "target identity is not comparable", "metric names differ between red and green"],
  },
  {
    primitive: "devtools.investigate",
    requiredResultFields: [
      "artifactKind",
      "bugId",
      "intake",
      "environment",
      "hypothesisLog",
      "primitiveStack",
      "red",
      "green",
      "comparison",
      "missingPrimitives",
      "likelyOwners",
      "cleanup",
    ],
    failClosedWhen: ["red classification is missing", "missing primitive lacks required fields", "cleanup state is unknown"],
  },
];

const acceptanceBar = [
  "target identity resolves strictly",
  "surface.inspect returns contract and runtime state",
  "elements.snapshot has stable semantic ids",
  "layout.measure includes regions, overlaps, and resize pressure",
  "scroll/text/focus receipts exist where applicable",
  "act.* can perform at least one user-like operation with pre/post receipts",
  "visual capture has strict target identity and nonblank proof",
  "events record correlates action to state change",
  "compare.redgreen can compare the same primitive stack",
  "investigate can export the artifact",
];

function parseArgs(argv: string[]) {
  return {
    markdown: argv.includes("--markdown"),
  };
}

function report() {
  return {
    schemaVersion: 1,
    tool: "script-kit-devtools.schema",
    generatedAt: new Date().toISOString(),
    source: ".agents/skills/script-kit-devtools/references/devtools-oracle-buildout-plan.md",
    philosophy: "Every DevTools primitive returns a shared fail-closed receipt envelope; screenshots and recipes never replace target-scoped proof.",
    receiptEnvelopeFields,
    classifications,
    targetIdentityFields,
    primitiveSchemas,
    acceptanceBar,
  };
}

function markdown(data: ReturnType<typeof report>) {
  return [
    "# Script Kit DevTools Receipt Schema",
    "",
    data.philosophy,
    "",
    "## Envelope Fields",
    "",
    data.receiptEnvelopeFields.join(", "),
    "",
    "## Classifications",
    "",
    data.classifications.join(", "),
    "",
    "## Primitive Schemas",
    "",
    "| Primitive | Required fields | Fail closed when |",
    "| --- | --- | --- |",
    ...data.primitiveSchemas.map(
      (schema) => `| ${schema.primitive} | ${schema.requiredResultFields.join(", ")} | ${schema.failClosedWhen.join(", ")} |`
    ),
    "",
    "## Acceptance Bar",
    "",
    ...data.acceptanceBar.map((item) => `- ${item}`),
  ].join("\n");
}

const data = report();
if (parseArgs(Bun.argv.slice(2)).markdown) {
  console.log(markdown(data));
} else {
  console.log(JSON.stringify(data, null, 2));
}
