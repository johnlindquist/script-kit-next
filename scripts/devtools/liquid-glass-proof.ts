#!/usr/bin/env bun

import { readFileSync } from "node:fs";

type JsonObject = Record<string, unknown>;

type Args = {
  inventory: string;
  artifactRoot: string;
  out: string;
};

type Evidence = {
  receipts: string[];
  screenshots: string[];
  layoutReceipts: string[];
  inspectReceipts: string[];
  imageDiffReceipts: string[];
  diffMasks: string[];
  visualAudit: JsonObject | null;
  notes: string[];
};

type ProofTiers = {
  osScreenshotProof: "pass" | "blocked" | "missing";
  appRenderProof: "pass" | "fail" | "missing";
  offscreenRenderProof: "pass" | "fail" | "missing";
  numericProof: "pass" | "fail" | "missing";
  imageDiffProof: "pass" | "blocked" | "missing";
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

function readJsonSync(path: string): JsonObject | null {
  try {
    return JSON.parse(readFileSync(path, "utf8")) as JsonObject;
  } catch {
    return null;
  }
}

type ScreenshotUsability = {
  usable: boolean;
  note?: string;
};

type ImageDiffUsability = {
  usable: boolean;
  note?: string;
};

function auditFromReceipt(json: JsonObject | null): JsonObject {
  const screenshot = asObject(json?.screenshot);
  const screenshotReceipt = asObject(json?.screenshotReceipt);
  const screenshotReceiptNested = asObject(screenshot.receipt);
  return asObject(
    json?.contentAudit
      ?? screenshot.contentAudit
      ?? screenshotReceipt.contentAudit
      ?? screenshotReceiptNested.contentAudit,
  );
}

function screenshotUsability(path: string, receipts: string[]): ScreenshotUsability {
  const fileName = path.split("/").pop() ?? path;
  const baseName = fileName.replace(/\.[^.]+$/, "");
  const matchingReceipts = receipts.filter((receipt) => {
    const receiptName = receipt.split("/").pop() ?? receipt;
    return receiptName.startsWith(baseName) && receiptName.includes("screenshot");
  });
  if (matchingReceipts.length === 0 && baseName.startsWith("tahoe-native-")) {
    return {
      usable: false,
      note: `ignored screenshot ${path}: no matching screenshot receipt with capture status`,
    };
  }

  for (const receipt of matchingReceipts) {
    const json = readJsonSync(receipt);
    if (json?.status === "error" || json?.classification === "error") {
      return {
        usable: false,
        note: `ignored screenshot ${path}: receipt ${receipt} is classified as an error`,
      };
    }
    const contentAudit = auditFromReceipt(json);
    const nonBlackRatio = contentAudit.nonBlackRatio;
    if (typeof nonBlackRatio === "number" && nonBlackRatio < 0.01) {
      return {
        usable: false,
        note: `ignored screenshot ${path}: receipt ${receipt} nonBlackRatio ${nonBlackRatio} is below 0.01 usable-capture threshold`,
      };
    }
  }

  return { usable: true };
}

function imageDiffUsability(path: string): ImageDiffUsability {
  const json = readJsonSync(path);
  if (!json) {
    return {
      usable: false,
      note: `ignored image diff ${path}: receipt could not be parsed`,
    };
  }
  const assertions = asObject(json.assertions);
  const errors = asArray(json.errors);
  const dimensions = asObject(json.dimensions);
  const sameSizeRequired = json.sameSizeRequired === true;
  const sameSize = dimensions.sameSize;
  if (json.classification !== "ok") {
    return {
      usable: false,
      note: `ignored image diff ${path}: classification is ${String(json.classification ?? "missing")}`,
    };
  }
  if (assertions.diffMaskWritten !== true) {
    return {
      usable: false,
      note: `ignored image diff ${path}: diffMaskWritten assertion is not true`,
    };
  }
  if (assertions.changedPixelsMeasured !== true) {
    return {
      usable: false,
      note: `ignored image diff ${path}: changedPixelsMeasured assertion is not true`,
    };
  }
  if (errors.length > 0) {
    return {
      usable: false,
      note: `ignored image diff ${path}: receipt has errors`,
    };
  }
  if (sameSizeRequired && sameSize !== true) {
    return {
      usable: false,
      note: `ignored image diff ${path}: sameSizeRequired is true but dimensions.sameSize is not true`,
    };
  }

  return { usable: true };
}

function evidenceFor(terms: string[], files: { receipts: string[]; screenshots: string[]; diffs: string[] }, visualAuditPath?: string): Evidence {
  const receipts = files.receipts.filter((path) => includesAny(path, terms));
  const screenshots: string[] = [];
  const screenshotNotes: string[] = [];
  for (const path of files.screenshots.filter((screenshotPath) => includesAny(screenshotPath, terms))) {
    const usability = screenshotUsability(path, files.receipts);
    if (usability.usable) {
      screenshots.push(path);
    } else if (usability.note) {
      screenshotNotes.push(usability.note);
    }
  }
  const imageDiffReceipts: string[] = [];
  const imageDiffNotes: string[] = [];
  for (const path of receipts.filter((receiptPath) => receiptPath.includes("image-diff"))) {
    const usability = imageDiffUsability(path);
    if (usability.usable) {
      imageDiffReceipts.push(path);
    } else if (usability.note) {
      imageDiffNotes.push(usability.note);
    }
  }
  return {
    receipts,
    screenshots,
    layoutReceipts: receipts.filter((path) => path.includes("layout")),
    inspectReceipts: receipts.filter((path) => path.includes("inspect") || path.includes("window")),
    imageDiffReceipts,
    diffMasks: files.diffs.filter((path) => includesAny(path, terms)),
    visualAudit: null,
    notes: [
      ...screenshotNotes,
      ...imageDiffNotes,
      ...(visualAuditPath ? [`visualAudit sourced from ${visualAuditPath}`] : []),
    ],
  };
}

function proofTiers(evidence: Evidence): ProofTiers {
  let osBlocked = false;
  let appRenderPass = false;
  let appRenderFail = false;
  let offscreenPass = false;
  let offscreenFail = false;
  for (const receipt of evidence.receipts) {
    const json = readJsonSync(receipt);
    const visualEvidence = asObject(json?.visualEvidence);
    if (visualEvidence.classification === "macos-windowserver-capture-blocked") {
      osBlocked = true;
    }
    if (visualEvidence.countsAsOsScreenshotEvidence === true) {
      osBlocked = false;
    }
    const renderEvidence = asObject(json?.renderEvidence);
    if (renderEvidence.countsAsAppRenderEvidence === true) {
      appRenderPass = true;
    } else if (Object.keys(renderEvidence).length > 0) {
      appRenderFail = true;
    }
    if (String(renderEvidence.source ?? "").includes("offscreen")) {
      if (renderEvidence.available === true) {
        offscreenPass = true;
      } else {
        offscreenFail = true;
      }
    }
  }
  const audit = asObject(evidence.visualAudit);
  const styled = typeof audit.styledNodeCount === "number" ? audit.styledNodeCount : null;
  const nodeCount = typeof audit.nodeCount === "number" ? audit.nodeCount : null;
  const numericPass =
    evidence.layoutReceipts.length > 0 &&
    nodeCount != null &&
    styled === nodeCount &&
    asArray(audit.controlsWithHitFailures).length === 0 &&
    asArray(audit.contentGlassNodes).length === 0 &&
    asArray(audit.contentNativeMaterialNodes).length === 0 &&
    asArray(audit.glassLayerViolations).length === 0 &&
    asArray(audit.missingStyleNodeNames).length === 0;
  return {
    osScreenshotProof: evidence.screenshots.length > 0 ? "pass" : osBlocked ? "blocked" : "missing",
    appRenderProof: appRenderPass ? "pass" : appRenderFail ? "fail" : "missing",
    offscreenRenderProof: offscreenPass ? "pass" : offscreenFail ? "fail" : "missing",
    numericProof: numericPass ? "pass" : evidence.layoutReceipts.length > 0 ? "fail" : "missing",
    imageDiffProof: evidence.imageDiffReceipts.length > 0 ? "pass" : osBlocked ? "blocked" : "missing",
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
  const contentNativeMaterial = asArray(audit.contentNativeMaterialNodes).length;
  const glassLayerViolations = asArray(audit.glassLayerViolations).length;
  const missingStyle = asArray(audit.missingStyleNodeNames).length;

  if (hasScreenshot && hasLayout && hasImageDiff && nodeCount != null && styled === nodeCount && hitFailures === 0 && contentGlass === 0 && contentNativeMaterial === 0 && glassLayerViolations === 0 && missingStyle === 0) {
    return "strong-proof";
  }
  if (hasLayout && nodeCount != null && styled === nodeCount && hitFailures === 0 && contentGlass === 0 && contentNativeMaterial === 0 && glassLayerViolations === 0 && missingStyle === 0) {
    return "numeric-proof-missing-visual-capture";
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
    ScriptList: ["main"],
    ActionsDialog: ["actions"],
    ConfirmPrompt: ["confirm"],
    PromptEntity: ["prompt-div", "promptentity"],
    PromptChildContent: ["prompt-child", "promptchildcontent"],
    ExplicitPromptEntity: ["prompt-explicit", "explicitpromptentity"],
    UtilityChildContent: ["utility-quick", "utilitychildcontent"],
    Webcam: ["webcam"],
    About: ["about"],
    Feedback: ["feedback", "creation-feedback", "creationFeedback"],
    AcpChat: ["acp"],
    AcpHistory: ["acp-history"],
    ClipboardHistory: ["clipboard"],
    AppLauncher: ["app-launcher", "applauncher"],
    WindowSwitcher: ["window-switcher", "windowswitcher"],
    BrowserTabs: ["browser-tabs", "browsertabs"],
    GenericFilterableList: [
      "generic-filterable",
      "generic-filterable-list",
      "favorites",
      "favoritesBrowse",
      "search-ai-presets",
      "searchAiPresets",
    ],
    ProcessManager: ["process-manager", "processmanager"],
    CurrentAppCommands: ["current-app", "current-app-commands", "currentappcommands"],
    Settings: ["settings"],
    KitStoreBrowse: ["kit-store-browse", "kitstorebrowse"],
    KitStoreInstalled: ["kit-store-installed", "kitstoreinstalled"],
    EmojiPicker: ["emoji", "emoji-picker", "emojipicker"],
    ThemeChooser: ["theme", "choose-theme", "theme-chooser", "themechooser"],
    FileSearchMini: ["file-search-mini"],
    FileSearchFull: ["file-search-full"],
    AttachmentPortalBrowser: ["attachment-portal", "dictation-history"],
    SdkReference: ["sdk-reference"],
    ScriptTemplateCatalog: ["script-template", "script-template-catalog"],
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
        `${RECEIPT_ROOT}/window-priority-main-backdrop-current-layout.json`,
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
    } else if (surfaceKind === "Feedback") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-feedback-layout-after.json`,
      ]);
    } else if (surfaceKind === "Dictation") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-dictation-layout-after.json`,
      ]);
    } else if (surfaceKind === "AcpChat") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-acp-detached-layout-after.json`,
      ]);
    } else if (surfaceKind === "ClipboardHistory") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-clipboard-current-layout.json`,
      ]);
    } else if (surfaceKind === "AppLauncher") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-app-launcher-current-layout.json`,
      ]);
    } else if (surfaceKind === "WindowSwitcher") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-window-switcher-current-layout.json`,
      ]);
    } else if (surfaceKind === "BrowserTabs") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-browser-tabs-current-layout.json`,
      ]);
    } else if (surfaceKind === "GenericFilterableList") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-generic-filterable-favorites-current-layout.json`,
        `${RECEIPT_ROOT}/window-priority-generic-filterable-search-ai-presets-current-layout.json`,
      ]);
    } else if (surfaceKind === "ProcessManager") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-process-manager-current-layout.json`,
      ]);
    } else if (surfaceKind === "CurrentAppCommands") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-current-app-current-layout.json`,
      ]);
    } else if (surfaceKind === "Settings") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-settings-current-layout.json`,
      ]);
    } else if (surfaceKind === "KitStoreBrowse") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-kit-store-browse-current-layout.json`,
      ]);
    } else if (surfaceKind === "KitStoreInstalled") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-kit-store-installed-current-layout.json`,
      ]);
    } else if (surfaceKind === "AcpHistory") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-acp-history-current-layout.json`,
      ]);
    } else if (surfaceKind === "FileSearchMini") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-file-search-mini-current-layout.json`,
      ]);
    } else if (surfaceKind === "FileSearchFull") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-file-search-full-current-layout.json`,
      ]);
    } else if (surfaceKind === "AttachmentPortalBrowser") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-attachment-portal-current-layout.json`,
      ]);
    } else if (surfaceKind === "SdkReference") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-sdk-reference-current-layout.json`,
      ]);
    } else if (surfaceKind === "ScriptTemplateCatalog") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-script-template-current-layout.json`,
      ]);
    } else if (surfaceKind === "EmojiPicker") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-emoji-current-layout.json`,
      ]);
    } else if (surfaceKind === "ThemeChooser") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-theme-current-layout.json`,
      ]);
    } else if (surfaceKind === "PromptEntity") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-prompt-div-fixed-layout-sdk.json`,
        `${RECEIPT_ROOT}/window-priority-prompt-div-current-layout-sdk.json`,
      ]);
    } else if (surfaceKind === "PromptChildContent") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-prompt-child-editor-fixed-layout-sdk.json`,
      ]);
    } else if (surfaceKind === "ExplicitPromptEntity") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-prompt-explicit-env-fixed-layout-sdk.json`,
        `${RECEIPT_ROOT}/window-priority-prompt-explicit-env-current-layout-sdk.json`,
      ]);
    } else if (surfaceKind === "UtilityChildContent") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-utility-quick-fixed-layout.json`,
        `${RECEIPT_ROOT}/window-priority-utility-quick-current-layout.json`,
      ]);
    } else if (surfaceKind === "Webcam") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-webcam-fixed-layout.json`,
        `${RECEIPT_ROOT}/window-priority-webcam-current-layout.json`,
      ]);
    }
    const status = classify(evidence);
    const tiers = proofTiers(evidence);
    return {
      surfaceKind,
      appViewVariants: contract.appViewVariants ?? [],
      automationSemanticSurface: contract.automationSemanticSurface ?? null,
      coverageAliases: contract.coverageAliases ?? [],
      proofStatus: status,
      requiredEvidence: {
        screenshot: evidence.screenshots.length > 0,
        osScreenshotProof: tiers.osScreenshotProof,
        appRenderProof: tiers.appRenderProof,
        offscreenRenderProof: tiers.offscreenRenderProof,
        numericLayout: evidence.layoutReceipts.length > 0,
        numericProof: tiers.numericProof,
        imageDiff: evidence.imageDiffReceipts.length > 0,
        imageDiffProof: tiers.imageDiffProof,
        visualAudit: evidence.visualAudit != null,
      },
      proofTiers: tiers,
      visualAudit: evidence.visualAudit,
      evidence,
    };
  }));

  const practicalTargets = await Promise.all([
    { id: "notes", terms: ["notes"] },
    { id: "dictation", terms: ["dictation"] },
    { id: "acp-detached", terms: ["acp-detached"] },
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
    } else if (target.id === "dictation") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-dictation-layout-after.json`,
      ]);
    } else if (target.id === "acp-detached") {
      await attachVisualAudit(evidence, [
        `${RECEIPT_ROOT}/window-priority-acp-detached-layout-after.json`,
      ]);
    }
    const dictationMedia = target.id === "dictation"
      ? await readJsonIfExists(`${RECEIPT_ROOT}/dictation-next-post-delivery-media.json`)
      : null;
    const dictationDelivery = target.id === "dictation"
      ? await readJsonIfExists(`${RECEIPT_ROOT}/dictation-next-deliver-fixture.json`)
      : null;
    const baseStatus = classify(evidence);
    const tiers = proofTiers(evidence);
    const proofStatus = target.id === "dictation" && evidence.visualAudit && evidence.layoutReceipts.length > 0
      ? "numeric-window-proof-screenshot-blocked"
      : target.id === "acp-detached" && evidence.visualAudit && evidence.layoutReceipts.length > 0
        ? "numeric-window-proof-screenshot-blocked"
      : target.id === "dictation" && dictationMedia
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
        osScreenshotProof: tiers.osScreenshotProof,
        appRenderProof: tiers.appRenderProof,
        offscreenRenderProof: tiers.offscreenRenderProof,
        numericLayout: evidence.layoutReceipts.length > 0,
        numericProof: tiers.numericProof,
        imageDiff: evidence.imageDiffReceipts.length > 0,
        imageDiffProof: tiers.imageDiffProof,
        visualAudit: evidence.visualAudit != null,
        mediaProof: dictationMedia != null,
        syntheticDelivery: dictationDelivery != null,
      },
      proofTiers: tiers,
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

  const visualTierDebtSurfaces = surfaces
    .filter((surface) =>
      surface.proofTiers.osScreenshotProof === "blocked" ||
      surface.proofTiers.appRenderProof === "fail" ||
      surface.proofTiers.offscreenRenderProof === "fail" ||
      surface.proofTiers.numericProof === "fail" ||
      surface.proofTiers.imageDiffProof === "blocked"
    )
    .map((surface) => ({
      surfaceKind: surface.surfaceKind,
      proofStatus: surface.proofStatus,
      proofTiers: surface.proofTiers,
      failedTiers: [
        surface.proofTiers.osScreenshotProof === "blocked" ? "osScreenshotProof" : "",
        surface.proofTiers.appRenderProof === "fail" ? "appRenderProof" : "",
        surface.proofTiers.offscreenRenderProof === "fail" ? "offscreenRenderProof" : "",
        surface.proofTiers.numericProof === "fail" ? "numericProof" : "",
        surface.proofTiers.imageDiffProof === "blocked" ? "imageDiffProof" : "",
      ].filter(Boolean),
      receipts: surface.evidence.receipts,
      screenshots: surface.evidence.screenshots,
      notes: surface.evidence.notes,
    }));
  const surfaceProofDebtSurfaces = surfaces
    .filter((surface) => surface.proofStatus !== "strong-proof")
    .map((surface) => ({
      surfaceKind: surface.surfaceKind,
      proofStatus: surface.proofStatus,
      proofTiers: surface.proofTiers,
      requiredEvidence: surface.requiredEvidence,
      receipts: surface.evidence.receipts,
      screenshots: surface.evidence.screenshots,
      notes: surface.evidence.notes,
    }));

  const summary = {
    surfaceCount: surfaces.length,
    strongProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus === "strong-proof").length,
    partialProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus !== "missing-proof" && surface.proofStatus !== "strong-proof").length,
    missingProofSurfaceCount: surfaces.filter((surface) => surface.proofStatus === "missing-proof").length,
    osScreenshotBlockedSurfaceCount: surfaces.filter((surface) => surface.proofTiers.osScreenshotProof === "blocked").length,
    appRenderProofSurfaceCount: surfaces.filter((surface) => surface.proofTiers.appRenderProof === "pass").length,
    appRenderFailedSurfaceCount: surfaces.filter((surface) => surface.proofTiers.appRenderProof === "fail").length,
    appRenderMissingSurfaceCount: surfaces.filter((surface) => surface.proofTiers.appRenderProof === "missing").length,
    offscreenRenderFailedSurfaceCount: surfaces.filter((surface) => surface.proofTiers.offscreenRenderProof === "fail").length,
    offscreenRenderMissingSurfaceCount: surfaces.filter((surface) => surface.proofTiers.offscreenRenderProof === "missing").length,
    visualTierDebtSurfaceCount: visualTierDebtSurfaces.length,
    surfaceProofDebtCount: surfaceProofDebtSurfaces.length,
    batchCount: batches.length,
    strongProofBatchCount: batches.filter((batch) => batch.proofStatus === "strong-proof").length,
    partialProofBatchCount: batches.filter((batch) => batch.proofStatus === "partial-proof").length,
    missingProofBatchCount: batches.filter((batch) => batch.proofStatus === "missing-proof").length,
  };

  const receipt = {
    schemaVersion: 1,
    tool: "script-kit-devtools.liquid-glass-proof",
    command: "proof.matrix",
    classification: summary.missingProofSurfaceCount === 0 && summary.visualTierDebtSurfaceCount === 0 && summary.surfaceProofDebtCount === 0 ? "ok" : "incomplete",
    inventoryPath: args.inventory,
    artifactRoot: args.artifactRoot,
    generatedAt: new Date().toISOString(),
    summary,
    batches,
    visualTierDebtSurfaces,
    surfaceProofDebtSurfaces,
    surfaces,
    practicalTargets,
    warnings: [
      summary.missingProofSurfaceCount === 0 ? "" : `${summary.missingProofSurfaceCount} contract surfaces still lack proof artifacts`,
      summary.surfaceProofDebtCount === 0 ? "" : `${summary.surfaceProofDebtCount} contract surfaces are not yet strong-proof`,
      summary.visualTierDebtSurfaceCount === 0 ? "" : `${summary.visualTierDebtSurfaceCount} contract surfaces have explicit visual-tier debt; inspect proofTiers before claiming exhaustive Liquid Glass proof`,
      summary.appRenderFailedSurfaceCount === 0 ? "" : `${summary.appRenderFailedSurfaceCount} contract surfaces attempted app-render proof and failed or returned unsupported`,
      "strong-proof means current artifacts include screenshot, numeric layout visualAudit, and image diff evidence; it is not an Apple conformance claim by itself",
      "proofTiers separate OS screenshots from GPUI app-render proof so WindowServer-blocked captures cannot become false visual evidence",
    ].filter(Boolean),
    errors: [],
  };

  await Bun.write(args.out, `${JSON.stringify(receipt, null, 2)}\n`);
  console.log(JSON.stringify(receipt, null, 2));
}

await main();
