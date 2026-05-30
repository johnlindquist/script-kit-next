#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Args = {
  inventory: string;
  artifactRoot: string;
  out: string;
};

type Evidence = {
  screenshots: string[];
  layoutReceipts: string[];
  inspectReceipts: string[];
  imageDiffReceipts: string[];
  diffMasks: string[];
  visualAudit: JsonObject | null;
  notes: string[];
};

const RECEIPT_ROOT = "artifacts/liquid-glass/receipts";
const SCREENSHOT_ROOT = "artifacts/liquid-glass/screenshots";
const DIFF_ROOT = "artifacts/liquid-glass/diffs";

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/liquid-glass-proof.ts --inventory <surface-inventory.json> --artifact-root artifacts/liquid-glass --out <proof-matrix.json>",
  ].join("\n");
}

function parseArgs(argv: string[]): Args {
  if (argv.includes("--help") || argv.includes("-h")) {
    console.log(usage());
    process.exit(0);
  }
  const args: Args = {
    inventory: `${RECEIPT_ROOT}/surface-inventory-2026-05-29-2050.json`,
    artifactRoot: "artifacts/liquid-glass",
    out: `${RECEIPT_ROOT}/liquid-glass-proof-matrix.json`,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--inventory") {
      args.inventory = argv[++index] ?? args.inventory;
    } else if (arg === "--artifact-root") {
      args.artifactRoot = argv[++index] ?? args.artifactRoot;
    } else if (arg === "--out") {
      args.out = argv[++index] ?? args.out;
    }
  }
  return args;
}

function asObject(value: unknown): JsonObject {
  return typeof value === "object" && value !== null ? value as JsonObject : {};
}

function asArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : [];
}

async function listFiles(dir: string) {
  const proc = Bun.spawnSync(["find", dir, "-maxdepth", "1", "-type", "f", "-print"], {
    stdout: "pipe",
    stderr: "pipe",
  });
  if (proc.exitCode !== 0) {
    return [];
  }
  return new TextDecoder().decode(proc.stdout).trim().split("\n").filter(Boolean).sort();
}

async function readJsonIfExists(path: string) {
  try {
    return JSON.parse(await Bun.file(path).text()) as JsonObject;
  } catch {
    return null;
  }
}

function includesAny(path: string, terms: string[]) {
  const lower = path.toLowerCase();
  return terms.some((term) => lower.includes(term.toLowerCase()));
}

function evidenceFor(terms: string[], files: { receipts: string[]; screenshots: string[]; diffs: string[] }, visualAuditPath?: string): Evidence {
  const receipts = files.receipts.filter((path) => includesAny(path, terms));
  return {
    screenshots: files.screenshots.filter((path) => includesAny(path, terms)),
    layoutReceipts: receipts.filter((path) => path.includes("layout")),
    inspectReceipts: receipts.filter((path) => path.includes("inspect") || path.includes("window")),
    imageDiffReceipts: receipts.filter((path) => path.includes("image-diff")),
    diffMasks: files.diffs.filter((path) => includesAny(path, terms)),
    visualAudit: null,
    notes: visualAuditPath ? [`visualAudit sourced from ${visualAuditPath}`] : [],
  };
}

function classify(evidence: Evidence) {
  const hasScreenshot = evidence.screenshots.length > 0;
  const hasLayout = evidence.layoutReceipts.length > 0;
  const hasImageDiff = evidence.imageDiffReceipts.length > 0;
  const audit = asObject(evidence.visualAudit);
  const styled = typeof audit.styledNodeCount === "number" ? audit.styledNodeCount : null;
  const nodeCount = typeof audit.nodeCount === "number" ? audit.nodeCount : null;
  const hitFailures = asArray(audit.controlsWithHitFailures).length;
  const contentGlass = asArray(audit.contentGlassNodes).length;
  const missingStyle = asArray(audit.missingStyleNodeNames).length;

  if (hasScreenshot && hasLayout && hasImageDiff && nodeCount != null && styled === nodeCount && hitFailures === 0 && contentGlass === 0 && missingStyle === 0) {
    return "strong-proof";
  }
  if (hasScreenshot && hasLayout) {
    return "numeric-proof-no-image-diff";
  }
  if (hasScreenshot) {
    return "baseline-only";
  }
  return "missing-proof";
}

async function attachVisualAudit(evidence: Evidence, preferred: string[]) {
  for (const path of preferred) {
    const json = await readJsonIfExists(path);
    const audit = Object.keys(asObject(json?.visualAudit)).length > 0
      ? asObject(json?.visualAudit)
      : asObject(asObject(asObject(json?.receipts).layout).visualAudit);
    if (Object.keys(audit).length > 0) {
      evidence.visualAudit = audit;
      evidence.notes.push(`visualAudit: ${path}`);
      return;
    }
  }
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const inventory = JSON.parse(await Bun.file(args.inventory).text()) as JsonObject;
  const files = {
    receipts: await listFiles(`${args.artifactRoot}/receipts`),
    screenshots: await listFiles(`${args.artifactRoot}/screenshots`),
    diffs: await listFiles(`${args.artifactRoot}/diffs`),
  };

  const surfaceTerms: Record<string, string[]> = {
    ScriptList: ["main", "launcher"],
    ActionsDialog: ["actions"],
    ConfirmPrompt: ["confirm"],
    About: ["about"],
    AcpChat: ["acp"],
    AcpHistory: ["acp-history"],
    ClipboardHistory: ["clipboard"],
    NotesWindow: ["notes"],
    Dictation: ["dictation"],
  };

  const surfaceContracts = asArray(inventory.auditSurfaceContracts ?? inventory.surfaceContracts).map((entry) => asObject(entry));
  const recommendedBatches = asArray(inventory.recommendedOracleBatches).map((entry) => asObject(entry));
  const surfaces = await Promise.all(surfaceContracts.map(async (contract) => {
    const surfaceKind = String(contract.surfaceKind ?? "");
    const terms = surfaceTerms[surfaceKind] ?? [surfaceKind.replace(/([a-z])([A-Z])/g, "$1-$2").toLowerCase(), surfaceKind.toLowerCase()];
    const evidence = evidenceFor(terms, files);
    if (surfaceKind === "ScriptList") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/tahoe-next-main-layout.json`,
        `${RECEIPT_ROOT}/after-main-layout-visual-style.json`,
      ]);
    } else if (surfaceKind === "ActionsDialog") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/tahoe-next-actions-layout.json`,
        `${RECEIPT_ROOT}/after-actions-layout-visual-style.json`,
      ]);
    } else if (surfaceKind === "About") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-about-layout-after.json`,
      ]);
    }
    const status = classify(evidence);
    return {
      surfaceKind,
      appViewVariants: contract.appViewVariants ?? [],
      automationSemanticSurface: contract.automationSemanticSurface ?? null,
      coverageAliases: contract.coverageAliases ?? [],
      proofStatus: status,
      requiredEvidence: {
        screenshot: evidence.screenshots.length > 0,
        numericLayout: evidence.layoutReceipts.length > 0,
        imageDiff: evidence.imageDiffReceipts.length > 0,
        visualAudit: evidence.visualAudit != null,
      },
      visualAudit: evidence.visualAudit,
      evidence,
    };
  }));

  const practicalTargets = await Promise.all([
    { id: "notes", terms: ["notes"] },
    { id: "dictation", terms: ["dictation"] },
    { id: "inline-agent", terms: ["inline-agent"] },
    { id: "notes-acp", terms: ["notes-acp"] },
  ].map(async (target) => {
    const evidence = evidenceFor(target.terms, files);
    if (target.id === "notes") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/notes-next-layout.json`,
      ]);
    } else if (target.id === "inline-agent") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/inline-agent-main-layout.json`,
      ]);
    } else if (target.id === "notes-acp") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/notes-acp-next-actions-open-inspect.json`,
        `${RECEIPT_ROOT}/notes-acp-actions-open-inspect.json`,
      ]);
    }
    const dictationMedia = target.id === "dictation"
      ? await readJsonIfExists(`${RECEIPT_ROOT}/dictation-next-post-delivery-media.json`)
      : null;
    const dictationDelivery = target.id === "dictation"
      ? await readJsonIfExists(`${RECEIPT_ROOT}/dictation-next-deliver-fixture.json`)
      : null;
    const baseStatus = classify(evidence);
    const proofStatus = target.id === "dictation" && dictationMedia
      ? "media-proof-missing-visual"
      : target.id === "inline-agent" && evidence.visualAudit && evidence.layoutReceipts.length > 0
        ? "numeric-proof-missing-visual-capture"
        : target.id === "notes-acp" && evidence.visualAudit && evidence.inspectReceipts.length > 0
          ? "actions-panel-proof-acp-startup-blocked"
        : baseStatus;
    return {
      id: target.id,
      proofStatus,
      requiredEvidence: {
        screenshot: evidence.screenshots.length > 0,
        numericLayout: evidence.layoutReceipts.length > 0,
        imageDiff: evidence.imageDiffReceipts.length > 0,
        visualAudit: evidence.visualAudit != null,
        mediaProof: dictationMedia != null,
        syntheticDelivery: dictationDelivery != null,
      },
      mediaProof: dictationMedia
        ? {
          status: dictationMedia.status ?? null,
          missingRuntimePrimitives: dictationMedia.missingRuntimePrimitives ?? [],
        }
        : null,
      syntheticDelivery: dictationDelivery
        ? {
          classification: dictationDelivery.classification ?? null,
          target: dictationDelivery.target ?? null,
          delivery: dictationDelivery.delivery ?? null,
        }
        : null,
      evidence,
    };
  }));

  const byKind = new Map(surfaces.map((surface) => [String(surface.surfaceKind), surface]));
  const batches = recommendedBatches.map((batch) => {
    const surfaceKinds = asArray(batch.surfaceKinds).map(String);
    const entries = surfaceKinds.map((surfaceKind) => byKind.get(surfaceKind)).filter(Boolean);
    const statuses = entries.map((entry) => String(entry?.proofStatus ?? "missing-proof"));
    const complete = statuses.length > 0 && statuses.every((status) => status === "strong-proof");
    return {
      id: batch.id,
      name: batch.name,
      owners: batch.owners ?? [],
      surfaceKinds,
      requiredDevToolsPrimitives: batch.requiredDevToolsPrimitives ?? [],
      proofStatus: complete ? "strong-proof" : statuses.some((status) => status !== "missing-proof") ? "partial-proof" : "missing-proof",
      provenSurfaceCount: statuses.filter((status) => status === "strong-proof").length,
      partialSurfaceCount: statuses.filter((status) => status !== "missing-proof" && status !== "strong-proof").length,
      missingSurfaceCount: statuses.filter((status) => status === "missing-proof").length,
      surfaceStatuses: entries.map((entry) => ({
        surfaceKind: entry?.surfaceKind,
        proofStatus: entry?.proofStatus,
      })),
    };
  });

  const summary = {
    surfaceCount: surfaces.length,
    strongProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus === "strong-proof").length,
    partialProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus !== "missing-proof" && surface.proofStatus !== "strong-proof").length,
    missingProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus === "missing-proof").length,
    batchCount: batches.length,
    strongProofBatchCount: batches.filter((batch) => batch.proofStatus === "strong-proof").length,
    partialProofBatchCount: batches.filter((batch) => batch.proofStatus === "partial-proof").length,
    missingProofBatchCount: batches.filter((batch) => batch.proofStatus === "missing-proof").length,
  };

  const receipt = {
    schemaVersion: 1,
    tool: "script-kit-devtools.liquid-glass-proof",
    command: "proof.matrix",
    classification: summary.missingProofSurfaceCount === 0 ? "ok" : "incomplete",
    inventoryPath: args.inventory,
    artifactRoot: args.artifactRoot,
    generatedAt: new Date().toISOString(),
    summary,
    batches,
    surfaces,
    practicalTargets,
    warnings: [
      summary.missingProofSurfaceCount === 0 ? "" : `${summary.missingProofSurfaceCount} contract surfaces still lack proof artifacts`,
      "strong-proof means current artifacts include screenshot, numeric layout visualAudit, and image diff evidence; it is not an Apple conformance claim by itself",
    ].filter(Boolean),
    errors: [],
  };

  await Bun.write(args.out, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
}

await main();
