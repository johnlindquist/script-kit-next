#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type MediaArgs = {
  coveragePath?: string;
  receiptPath?: string;
  surface: string;
  markdown: boolean;
};

const passiveFields = [
  "passive microphone permission status",
  "microphone device snapshot",
  "model readiness generation",
  "recording state generation",
  "audio level metrics",
  "target delivery generation",
  "transcript fingerprint",
  "cursor insertion range",
  "wrong-target refusal receipt",
  "hotkey binding snapshot",
  "media cleanup receipt",
];

function parseArgs(argv: string[]): MediaArgs {
  const args: MediaArgs = {
    surface: "dictation",
    markdown: false,
  };
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--coverage") {
      args.coveragePath = argv[++index];
    } else if (arg === "--receipt") {
      args.receiptPath = argv[++index];
    } else if (arg === "--surface") {
      args.surface = argv[++index] ?? args.surface;
    } else if (arg === "--markdown") {
      args.markdown = true;
    }
  }
  return args;
}

async function readJson(path: string | undefined) {
  if (!path) {
    return null;
  }
  try {
    return JSON.parse(await Bun.file(path).text()) as JsonObject;
  } catch (error) {
    return {
      status: "error",
      path,
      error: error instanceof Error ? error.message : String(error),
    };
  }
}

function asArray(value: unknown): unknown[] {
  return Array.isArray(value) ? value : [];
}

function asObject(value: unknown): JsonObject {
  return value && typeof value === "object" && !Array.isArray(value) ? value as JsonObject : {};
}

function coverageSurface(coverage: JsonObject | null, id: string) {
  return asArray(coverage?.surfaces).map(asObject).find((surface) => surface.id === id) ?? null;
}

function receiptHas(receipt: JsonObject | null, field: string) {
  if (!receipt) {
    return false;
  }
  const runtimeState = asObject(receipt.runtimeState);
  const dictation = asObject(
    asObject(receipt.response).dictation
      ?? receipt.dictation
      ?? runtimeState.dictation
      ?? (runtimeState.source === "runtime.dictation.automationState" ? runtimeState : null),
  );
  const setup = asObject(dictation.setup);
  const microphone = asObject(setup.microphone);
  const model = asObject(setup.model);
  const hotkey = asObject(setup.hotkey);
  const cleanup = asObject(dictation.cleanup);
  const lastDelivery = asObject(dictation.lastDelivery);
  const audioLevels = asObject(dictation.audioLevels);

  if (field === "passive microphone permission status") {
    return typeof microphone.permissionStatus === "string";
  }
  if (field === "microphone device snapshot") {
    return typeof microphone.deviceSnapshot === "object" && microphone.deviceSnapshot !== null;
  }
  if (field === "model readiness generation") {
    return typeof model.status === "string" && typeof model.generation === "number";
  }
  if (field === "recording state generation") {
    return typeof dictation.recordingStateGeneration === "number";
  }
  if (field === "audio level metrics") {
    return typeof audioLevels.available === "boolean" && Array.isArray(audioLevels.bars);
  }
  if (field === "target delivery generation") {
    return typeof lastDelivery.generation === "number";
  }
  if (field === "transcript fingerprint") {
    return typeof lastDelivery.transcriptFingerprint === "string";
  }
  if (field === "hotkey binding snapshot") {
    return typeof hotkey.enabled === "boolean" && typeof hotkey.generation === "number";
  }
  if (field === "media cleanup receipt") {
    return typeof cleanup.captureActive === "boolean" && typeof cleanup.generation === "number";
  }
  if (field === "wrong-target refusal receipt") {
    return typeof dictation.wrongTargetRefusal === "object" && dictation.wrongTargetRefusal !== null;
  }

  const normalized = field.replace(/[^a-z0-9]+/gi, "").toLowerCase();
  const serialized = JSON.stringify(receipt).replace(/[^a-z0-9]+/gi, "").toLowerCase();
  return serialized.includes(normalized);
}

function report(args: MediaArgs, coverage: JsonObject | null, receipt: JsonObject | null) {
  const surface = coverageSurface(coverage, args.surface);
  const missingFromCoverage = asArray(surface?.missingRuntimePrimitives).map(String);
  const required = passiveFields.map((field) => ({
    field,
    presentInReceipt: receiptHas(receipt, field),
    listedAsMissing: missingFromCoverage.includes(field) || (field === "passive microphone permission status" && missingFromCoverage.includes("devtools.media.inspect")),
  }));
  const missing = required.filter((field) => !field.presentInReceipt).map((field) => field.field);

  return {
    schemaVersion: 1,
    tool: "script-kit-devtools.media.inspect",
    status: missing.length === 0 ? "ok" : "blocked-by-missing-primitive",
    inputs: {
      coveragePath: args.coveragePath ?? null,
      receiptPath: args.receiptPath ?? null,
      surface: args.surface,
    },
    surface: surface
      ? {
        id: surface.id ?? null,
        name: surface.name ?? null,
        status: surface.status ?? null,
      }
      : null,
    passiveSafety: {
      noMicrophoneCaptureRequired: true,
      noSystemSettingsRequired: true,
      noTccMutationRequired: true,
    },
    redaction: {
      transcriptContentReturned: false,
      deviceLabelsReturned: false,
      targetLabelsReturned: false,
      rawDeviceIdsReturned: false,
    },
    requiredFields: required,
    missingRuntimePrimitives: missing,
    recommendedNext: [
      "Expose passive microphone permission and device snapshots without opening System Settings.",
      "Expose model readiness and recording generations before transcript delivery.",
      "Expose target delivery, cursor insertion, wrong-target refusal, and cleanup receipts.",
      "Only then promote live Dictation recipes from fail-closed backlog to regression proof.",
    ],
  };
}

function markdown(result: ReturnType<typeof report>) {
  return [
    "# Script Kit DevTools Media Inspect",
    "",
    `Status: ${result.status}`,
    `Surface: ${result.surface?.id ?? result.inputs.surface}`,
    "",
    "## Missing Runtime Primitives",
    "",
    ...result.missingRuntimePrimitives.map((item) => `- ${item}`),
  ].join("\n");
}

const args = parseArgs(Bun.argv.slice(2));
const [coverage, receipt] = await Promise.all([
  readJson(args.coveragePath),
  readJson(args.receiptPath),
]);
const result = report(args, coverage, receipt);
console.log(args.markdown ? markdown(result) : JSON.stringify(result, null, 2));
