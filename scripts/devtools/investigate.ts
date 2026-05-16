#!/usr/bin/env bun

type InvestigationStatus =
  | "needs-red-proof"
  | "blocked-by-missing-primitive"
  | "blocked-by-unknown-surface"
  | "ready-for-green-proof";

type SurfaceInventory = {
  tool: string;
  surfaceContracts: Array<{
    surfaceKind: string;
    appViewVariants: string[];
    vocabulary?: { family?: string; inputOwnership?: string; previewRole?: string };
    focusPolicy?: string;
    keyboardPolicy?: string;
    actionsPolicy?: string;
    proofPolicy?: string;
    visualPolicy?: string;
    automationSemanticSurface?: string;
  }>;
  existingDevToolsCoverage: { surfaceIds: string[] };
  coverageSurfaceAliases: Array<{
    alias: string;
    resolvesTo: { surfaceKind: string; appViewVariant?: string; hostKind?: string };
    countsAsCoverage: false;
  }>;
  recommendedOracleBatches: Array<{
    id: string;
    name: string;
    surfaceKinds: string[];
    featureIds: string[];
    requiredDevToolsPrimitives: string[];
  }>;
};

const root = new URL("../..", import.meta.url);

function usage() {
  return [
    "Usage: bun scripts/devtools/investigate.ts --surface <id-or-SurfaceKind> --bug <text> [--screenshot <path>] [--fixed]",
    "",
    "Creates a fail-closed Script Kit DevTools investigation plan from the surface inventory.",
  ].join("\n");
}

function parseArgs(argv: string[]) {
  const args = {
    surface: "",
    bug: "",
    screenshots: [] as string[],
    fixed: false,
    markdown: false,
    inventory: ".test-output/devtools-surfaces.json",
  };

  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--surface") {
      args.surface = argv[++index] ?? "";
    } else if (arg === "--bug") {
      args.bug = argv[++index] ?? "";
    } else if (arg === "--screenshot") {
      args.screenshots.push(argv[++index] ?? "");
    } else if (arg === "--fixed") {
      args.fixed = true;
    } else if (arg === "--markdown") {
      args.markdown = true;
    } else if (arg === "--inventory") {
      args.inventory = argv[++index] ?? args.inventory;
    } else if (arg === "--help" || arg === "-h") {
      console.log(usage());
      process.exit(0);
    }
  }

  if (!args.surface || !args.bug) {
    console.error(usage());
    process.exit(2);
  }

  return args;
}

async function readInventory(path: string): Promise<SurfaceInventory> {
  const file = Bun.file(new URL(path, root));
  if (!(await file.exists())) {
    const { stdout } = Bun.spawnSync(["bun", "scripts/devtools/surfaces.ts"], { cwd: root.pathname });
    return JSON.parse(stdout.toString()) as SurfaceInventory;
  }
  return JSON.parse(await file.text()) as SurfaceInventory;
}

function normalizeSurface(value: string) {
  return value
    .replace(/([a-z])([A-Z])/g, "$1-$2")
    .replace(/[_\s]+/g, "-")
    .toLowerCase();
}

function findSurface(inventory: SurfaceInventory, requested: string) {
  const normalized = normalizeSurface(requested);
  const coverageIds = new Set(inventory.existingDevToolsCoverage.surfaceIds);

  const contract = inventory.surfaceContracts.find((entry) => {
    const aliases = inventory.coverageSurfaceAliases
      .filter((alias) => alias.resolvesTo.surfaceKind === entry.surfaceKind)
      .map((alias) => alias.alias);
    const candidates = [entry.surfaceKind, ...entry.appViewVariants, entry.automationSemanticSurface ?? "", ...aliases].map(normalizeSurface);
    return candidates.includes(normalized);
  });

  const explicitCoverageId = [...coverageIds].find((id) => normalizeSurface(id) === normalized);
  const hasExplicitCoverage = Boolean(explicitCoverageId);

  return {
    requested,
    normalized,
    contract,
    explicitCoverageId: explicitCoverageId ?? null,
    hasExplicitCoverage,
  };
}

function scenarioHints(surface: string, bug: string) {
  const text = `${surface} ${bug}`.toLowerCase();
  const hints = new Set<string>();
  if (/(action|popup|menu|cmd\+k|palette)/.test(text)) hints.add("actions popup route stack, anchor rect, disabled reason, and clipping proof");
  if (/(resize|width|height|too tall|scroll|overflow|container|div|md)/.test(text)) hints.add("layout box model, scroll extent, overflow, resize pressure, and before/after bounds proof");
  if (/(note|editor|markdown|preview|selection|cursor)/.test(text)) hints.add("Notes target, editor selection, preview scroll, dirty state, and shortcut focus proof");
  if (/(dictation|mic|transcript|audio|recording|hotkey)/.test(text)) hints.add("passive media readiness, recording generation, target delivery, transcript fingerprint, and cleanup proof");
  if (/(focus|keyboard|tab|escape|enter|shortcut)/.test(text)) hints.add("focus owner transition, shortcut registry, tab order, and wrong-target refusal proof");
  if (/(screenshot|visual|overlap|clip|contrast|text)/.test(text)) hints.add("semantic-to-screenshot agreement, text fit, contrast, overlap, and occlusion proof");
  if (/(portal|file|attachment|context|resource)/.test(text)) hints.add("portal origin, return target, staged context parts, resource redaction, and privacy proof");
  if (hints.size === 0) hints.add("target identity, semantic elements, layout, screenshot, focus, and event timeline proof");
  return [...hints];
}

function buildInvestigation(inventory: SurfaceInventory, args: ReturnType<typeof parseArgs>) {
  const target = findSurface(inventory, args.surface);
  const owningBatches = inventory.recommendedOracleBatches.filter((batch) =>
    target.contract
      ? batch.surfaceKinds.includes(target.contract.surfaceKind)
      : batch.id.includes(target.normalized) || normalizeSurface(batch.name).includes(target.normalized)
  );
  const missingPrimitives = [...new Set(owningBatches.flatMap((batch) => batch.requiredDevToolsPrimitives))];
  const status: InvestigationStatus = target.hasExplicitCoverage
    ? args.fixed
      ? "ready-for-green-proof"
      : "needs-red-proof"
    : target.contract
      ? "blocked-by-missing-primitive"
      : "blocked-by-unknown-surface";

  return {
    schemaVersion: 1,
    tool: "script-kit-devtools.investigate",
    generatedAt: new Date().toISOString(),
    classification: status,
    intake: {
      bugReport: args.bug,
      requestedSurface: args.surface,
      screenshots: args.screenshots,
      phase: args.fixed ? "green-after-fix" : "red-before-fix",
    },
    target: {
      requested: target.requested,
      normalized: target.normalized,
      surfaceKind: target.contract?.surfaceKind ?? null,
      appViewVariants: target.contract?.appViewVariants ?? [],
      explicitCoverageId: target.explicitCoverageId,
      hasExplicitCoverage: target.hasExplicitCoverage,
      proofPolicy: target.contract?.proofPolicy ?? null,
      visualPolicy: target.contract?.visualPolicy ?? null,
      focusPolicy: target.contract?.focusPolicy ?? null,
      keyboardPolicy: target.contract?.keyboardPolicy ?? null,
      actionsPolicy: target.contract?.actionsPolicy ?? null,
    },
    scenarioHints: scenarioHints(args.surface, args.bug),
    requiredReceipts: [
      "target identity: listAutomationWindows + inspectAutomationWindow with exact target id",
      "semantic state: getState/getElements for the same target id",
      "layout state: getLayoutInfo or explicit missing target-scoped layout primitive",
      "interaction transcript: action intent, primitive used, target id, visible result, and receipt id",
      "visual proof: strict target screenshot only after semantic and layout identity agree",
      "red/green comparison: same target, same action path, same metric names before and after fix",
    ],
    missingRuntimePrimitives: target.hasExplicitCoverage ? [] : missingPrimitives,
    failClosedRules: [
      "Do not call a recipe pass a green investigation.",
      "Do not treat screenshots as sufficient without target, semantic, and layout identity.",
      "Do not drive native input unless the bug depends on OS focus, pointer delivery, permissions, or screen capture.",
      "If a required primitive is missing, report blocked-by-missing-primitive with the exact field needed.",
    ],
    recommendedCommands: {
      inspect: `bun scripts/devtools/inspect.ts --session dev --show --main`,
      inventory: `bun scripts/devtools/surfaces.ts`,
      coverage: target.explicitCoverageId ? `bun scripts/devtools/coverage.ts --surface ${target.explicitCoverageId}` : null,
      measure: target.explicitCoverageId
        ? `bun scripts/devtools/measure.ts --inspect <inspect.json> --coverage <coverage.json> --surface ${target.explicitCoverageId}`
        : null,
    },
    owningBatches: owningBatches.map((batch) => ({
      id: batch.id,
      name: batch.name,
      featureIds: batch.featureIds,
      requiredDevToolsPrimitives: batch.requiredDevToolsPrimitives,
    })),
  };
}

function markdown(report: ReturnType<typeof buildInvestigation>) {
  return [
    "# Script Kit DevTools Investigation",
    "",
    `Classification: ${report.classification}`,
    "",
    "## Intake",
    "",
    `Surface: ${report.intake.requestedSurface}`,
    "",
    `Bug: ${report.intake.bugReport}`,
    "",
    "## Target",
    "",
    `SurfaceKind: ${report.target.surfaceKind ?? "unknown"}`,
    "",
    `Explicit coverage: ${report.target.explicitCoverageId ?? "none"}`,
    "",
    "## Scenario Hints",
    "",
    ...report.scenarioHints.map((hint) => `- ${hint}`),
    "",
    "## Required Receipts",
    "",
    ...report.requiredReceipts.map((receipt) => `- ${receipt}`),
    "",
    "## Missing Runtime Primitives",
    "",
    ...(report.missingRuntimePrimitives.length ? report.missingRuntimePrimitives.map((primitive) => `- ${primitive}`) : ["none"]),
  ].join("\n");
}

const args = parseArgs(Bun.argv.slice(2));
const report = buildInvestigation(await readInventory(args.inventory), args);

if (args.markdown) {
  console.log(markdown(report));
} else {
  console.log(JSON.stringify(report, null, 2));
}
