#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type SurfaceContract = {
  surfaceKind: string;
  appViewVariants: string[];
  appViewFooters: Array<{ variant: string; nativeFooterSurface: string | null }>;
  vocabulary?: {
    family?: string;
    inputOwnership?: string;
    previewRole?: string;
  };
  focusPolicy?: string;
  keyboardPolicy?: string;
  actionsPolicy?: string;
  proofPolicy?: string;
  visualPolicy?: string;
  dismissPolicy?: {
    policy: string;
    windowBlur: string;
    backdropClick: string;
    escape: string;
    cmdW: string;
  };
  automationSemanticSurface?: string;
};

const root = new URL("../..", import.meta.url);

function usage() {
  return [
    "Usage:",
    "  bun scripts/devtools/surface.ts inspect --surface <SurfaceKind> [target args]",
    "",
    "Target args are forwarded to scripts/devtools/targets.ts inspect, e.g. --session <name> --main --strict --start --show.",
  ].join("\n");
}

function parseArgs(argv: string[]) {
  const command = argv[0] === "inspect" ? "inspect" : "";
  const args = {
    command,
    surfaceKind: "",
    timeoutMs: 8000,
    forwarded: [] as string[],
  };

  if (!command) {
    console.error(usage());
    process.exit(2);
  }

  for (let index = 1; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === "--surface") {
      args.surfaceKind = argv[++index] ?? "";
      args.forwarded.push("--surface", args.surfaceKind);
    } else if (arg === "--help" || arg === "-h") {
      console.log(usage());
      process.exit(0);
    } else {
      args.forwarded.push(arg);
      if (
        [
          "--session",
          "--target-id",
          "--target-kind",
          "--target-index",
          "--target-title",
          "--timeout",
        ].includes(arg)
      ) {
        const value = argv[++index] ?? "";
        args.forwarded.push(value);
        if (arg === "--timeout") {
          args.timeoutMs = Number(value) || args.timeoutMs;
        }
      }
    }
  }

  if (!args.surfaceKind) {
    console.error(usage());
    process.exit(2);
  }

  return args;
}

function requestId(prefix: string) {
  return `devtools-surface-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function run(command: string[], label: string): Promise<JsonObject> {
  const proc = Bun.spawn(command, { stdout: "pipe", stderr: "pipe" });
  const [stdout, stderr, exitCode] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  if (exitCode !== 0) {
    return { status: "error", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
  }
  try {
    return JSON.parse(stdout);
  } catch {
    return { status: "error", label, exitCode, stdout: stdout.trim(), stderr: stderr.trim(), error: "invalid_json_output" };
  }
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run(
    [
      "bash",
      "scripts/agentic/session.sh",
      "rpc",
      session,
      JSON.stringify(payload),
      "--expect",
      expect,
      "--timeout",
      String(timeoutMs),
    ],
    String(payload.type ?? "rpc"),
  );
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

async function readContracts() {
  const payload = JSON.parse(await Bun.file(new URL("docs/ai/contracts/surface-contracts.json", root)).text()) as {
    generatedFrom: string;
    registry: string;
    entries: SurfaceContract[];
  };
  return payload;
}

function runtimeValue(snapshot: JsonObject, ...names: string[]) {
  for (const name of names) {
    const value = snapshot[name];
    if (value !== undefined) {
      return value;
    }
  }
  return null;
}

function runtimeSurface(targetReceipt: JsonObject) {
  const rawInspect = targetReceipt.rawInspect as JsonObject | undefined;
  const snapshot = (rawInspect?.snapshot as JsonObject | undefined) ?? rawInspect ?? {};
  return {
    activeFooterSurface: runtimeValue(snapshot, "activeFooterSurface", "nativeFooterSurface"),
    inputOwnerSemanticId: runtimeValue(snapshot, "inputOwnerSemanticId"),
    previewSemanticId: runtimeValue(snapshot, "previewSemanticId"),
    focusedSemanticId: runtimeValue(snapshot, "focusedSemanticId"),
    selectedSemanticId: runtimeValue(snapshot, "selectedSemanticId"),
    rowCountVisible: runtimeValue(snapshot, "rowCountVisible", "visibleRowCount"),
    rowCountTotal: runtimeValue(snapshot, "rowCountTotal", "totalRowCount"),
    filterText: runtimeValue(snapshot, "filterText", "query"),
    sourceFilter: runtimeValue(snapshot, "sourceFilter"),
    capabilities: [
      "targetIdentity",
      targetReceipt.classification === "ok" ? "strictTargetInspect" : "",
      snapshot.screenshotWidth || snapshot.screenshot_width ? "screenshotMetadata" : "",
    ].filter(Boolean),
    missingPrimitives: [
      runtimeValue(snapshot, "focusedSemanticId") == null ? "focusedSemanticId" : "",
      runtimeValue(snapshot, "selectedSemanticId") == null ? "selectedSemanticId" : "",
      runtimeValue(snapshot, "rowCountVisible", "visibleRowCount") == null ? "rowCountVisible" : "",
    ].filter(Boolean),
  };
}

function enrichedRuntimeSurface(targetReceipt: JsonObject, state: JsonObject) {
  const base = runtimeSurface(targetReceipt);
  const stateStatus = state.status === "error" ? "error" : "ok";
  const activeFooter = state.activeFooter as JsonObject | undefined;
  const surfaceContract = state.surfaceContract as JsonObject | undefined;
  const rowCountVisible = runtimeValue(state, "visibleChoiceCount") ?? base.rowCountVisible;
  const rowCountTotal = runtimeValue(state, "choiceCount") ?? base.rowCountTotal;
  const filterText = runtimeValue(state, "inputValue") ?? base.filterText;
  const activeFooterSurface = activeFooter?.activeSurface ?? activeFooter?.expectedSurface ?? base.activeFooterSurface;
  const missingPrimitives = [
    ...base.missingPrimitives.filter((missing) => missing !== "rowCountVisible"),
    rowCountVisible == null ? "rowCountVisible" : "",
    rowCountTotal == null ? "rowCountTotal" : "",
    stateStatus === "error" ? "stateResult" : "",
  ].filter(Boolean);

  return {
    ...base,
    activeFooterSurface,
    rowCountVisible,
    rowCountTotal,
    filterText,
    selectedIndex: runtimeValue(state, "selectedIndex"),
    selectedValue: runtimeValue(state, "selectedValue"),
    windowVisible: runtimeValue(state, "windowVisible"),
    isFocused: runtimeValue(state, "isFocused"),
    surfaceContract,
    activeFooter,
    capabilities: [
      ...base.capabilities,
      stateStatus === "ok" ? "stateResult" : "",
      surfaceContract ? "surfaceContract" : "",
      activeFooter ? "activeFooter" : "",
    ].filter(Boolean),
    missingPrimitives,
  };
}

function contractPayload(contracts: Awaited<ReturnType<typeof readContracts>>, surfaceKind: string) {
  const contract = contracts.entries.find((entry) => entry.surfaceKind === surfaceKind);
  if (!contract) {
    return null;
  }
  return {
    sourcePath: "docs/ai/contracts/surface-contracts.json",
    generatedFrom: contracts.generatedFrom,
    registry: contracts.registry,
    surfaceKind: contract.surfaceKind,
    appViewVariants: contract.appViewVariants,
    nativeFooterSurfaces: contract.appViewFooters
      .map((footer) => footer.nativeFooterSurface)
      .filter((footer): footer is string => Boolean(footer)),
    family: contract.vocabulary?.family ?? null,
    inputOwnership: contract.vocabulary?.inputOwnership ?? null,
    previewRole: contract.vocabulary?.previewRole ?? null,
    focusPolicy: contract.focusPolicy ?? null,
    keyboardPolicy: contract.keyboardPolicy ?? null,
    actionsPolicy: contract.actionsPolicy ?? null,
    proofPolicy: contract.proofPolicy ?? null,
    visualPolicy: contract.visualPolicy ?? null,
    dismissPolicy: contract.dismissPolicy ?? null,
    automationSemanticSurface: contract.automationSemanticSurface ?? null,
  };
}

function classify(targetReceipt: JsonObject, contract: ReturnType<typeof contractPayload>) {
  if (!contract) {
    return "blocked-by-unknown-surface";
  }
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (!contract.dismissPolicy) {
    return "blocked-by-missing-primitive";
  }
  return "ok";
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const contracts = await readContracts();
  const targetReceipt = await run(["bun", "scripts/devtools/targets.ts", "inspect", ...args.forwarded], "targets.inspect");
  const stateEnvelope = await rpc(
    String((targetReceipt as JsonObject).session ?? "default"),
    {
      type: "getState",
      requestId: requestId("state"),
      target: (targetReceipt.requestedTarget as JsonObject | undefined)?.selector ?? { type: "main" },
      summaryOnly: true,
    },
    "stateResult",
    args.timeoutMs,
  );
  const state = responseOf(stateEnvelope);
  const contract = contractPayload(contracts, args.surfaceKind);
  const classification = classify(targetReceipt, contract);
  const runtime = enrichedRuntimeSurface(targetReceipt, state);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.surface",
    command: "surface.inspect",
    classification,
    requestedSurfaceKind: args.surfaceKind,
    target: targetReceipt.resolvedTarget ?? null,
    requestedTarget: targetReceipt.requestedTarget ?? null,
    contract,
    runtime,
    missingPrimitives: [
      ...(contract ? [] : ["surfaceContract"]),
      ...runtime.missingPrimitives,
      targetReceipt.classification !== "ok" ? "strictTargetIdentity" : "",
    ].filter(Boolean),
    state,
    targetReceipt,
  }, null, 2));
}

await main();
