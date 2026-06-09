#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

type Args = {
  session: string;
  target?: JsonObject;
  timeoutMs: number;
  forwarded: string[];
};

function usage() {
  return "Usage:\n  bun scripts/devtools/keyboard.ts inspect [target args]";
}

function parseArgs(argv: string[]): Args {
  if (argv[0] !== "inspect") {
    console.error(usage());
    process.exit(2);
  }
  const args: Args = { session: "default", timeoutMs: 8000, forwarded: [] };
  for (let index = 1; index < argv.length; index += 1) {
    const arg = argv[index];
    args.forwarded.push(arg);
    if (arg === "--session") {
      args.session = argv[++index] ?? args.session;
      args.forwarded.push(args.session);
    } else if (arg === "--target-id") {
      args.target = { type: "id", id: argv[++index] ?? "" };
      args.forwarded.push(String(args.target.id ?? ""));
    } else if (arg === "--target-kind") {
      const kind = argv[++index] ?? "main";
      args.target = { type: "kind", kind };
      args.forwarded.push(kind);
    } else if (arg === "--target-index") {
      const value = Number(argv[++index] ?? 0);
      if (!args.target || args.target.type !== "kind") {
        throw new Error("--target-index requires --target-kind first");
      }
      args.target.index = value;
      args.forwarded.push(String(value));
    } else if (arg === "--target-title") {
      args.target = { type: "titleContains", text: argv[++index] ?? "" };
      args.forwarded.push(String(args.target.text ?? ""));
    } else if (arg === "--focused") {
      args.target = { type: "focused" };
    } else if (arg === "--main") {
      args.target = { type: "main" };
    } else if (arg === "--surface") {
      args.forwarded.push(argv[++index] ?? "");
    } else if (arg === "--timeout") {
      args.timeoutMs = Number(argv[++index] ?? args.timeoutMs);
      args.forwarded.push(String(args.timeoutMs));
    } else if (arg === "--strict" || arg === "--start" || arg === "--show") {
      // Forwarded only.
    } else if (arg === "--help" || arg === "-h") {
      console.log(usage());
      process.exit(0);
    }
  }
  return args;
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

function requestId(prefix: string) {
  return `devtools-keyboard-${prefix}-${Date.now()}-${Math.random().toString(16).slice(2, 8)}`;
}

async function rpc(session: string, payload: JsonObject, expect: string, timeoutMs: number) {
  return run(["bash", "scripts/agentic/session.sh", "rpc", session, JSON.stringify(payload), "--expect", expect, "--timeout", String(timeoutMs)], String(payload.type ?? "rpc"));
}

function responseOf(envelope: JsonObject): JsonObject {
  return (envelope.response as JsonObject | undefined) ?? envelope;
}

function asArray(value: unknown): JsonObject[] {
  return Array.isArray(value) ? value.filter((entry): entry is JsonObject => typeof entry === "object" && entry !== null) : [];
}

function normalizeKey(value: unknown) {
  return typeof value === "string" ? value.trim().toLowerCase() : "";
}

function duplicateKeys(bindings: JsonObject[]) {
  const seen = new Set<string>();
  const duplicates = new Set<string>();
  for (const binding of bindings) {
    const key = normalizeKey(binding.key);
    if (!key) continue;
    if (seen.has(key)) {
      duplicates.add(key);
    }
    seen.add(key);
  }
  return [...duplicates];
}

function nativeFooterSnapshot(activeFooter: JsonObject, bindings: JsonObject[]) {
  return {
    hostInstalled: activeFooter.nativeFooterHostInstalled ?? null,
    owner: activeFooter.owner ?? null,
    activeSurface: activeFooter.activeSurface ?? null,
    expectedSurface: activeFooter.expectedSurface ?? null,
    bindingsAvailable: bindings.length > 0,
    activationPrimitiveAvailable: false,
    missingPrimitive: "nativeFooterActivationReceipt",
    nextSafeCommand: "bun scripts/devtools/focus.ts inspect --target-id <id> --strict",
  };
}

function classify(targetReceipt: JsonObject, stateEnvelope: JsonObject, bindings: JsonObject[]) {
  if (targetReceipt.classification !== "ok") {
    return targetReceipt.classification ?? "blocked-by-target-ambiguity";
  }
  if (stateEnvelope.status === "error") {
    return "blocked-by-timeout";
  }
  if (bindings.length === 0) {
    return "blocked-by-missing-primitive";
  }
  return "ok";
}

async function main() {
  const args = parseArgs(Bun.argv.slice(2));
  const targetReceipt = await run(["bun", "scripts/devtools/targets.ts", "inspect", ...args.forwarded], "targets.inspect");
  const selector = (targetReceipt.requestedTarget as JsonObject | undefined)?.selector ?? args.target ?? { type: "focused" };
  const stateEnvelope = await rpc(args.session, {
    type: "getState",
    requestId: requestId("state"),
    target: selector,
    summaryOnly: true,
  }, "stateResult", args.timeoutMs);
  const state = responseOf(stateEnvelope);
  const surfaceContract = (state.surfaceContract as JsonObject | undefined) ?? {};
  const activeFooter = (state.activeFooter as JsonObject | undefined) ?? {};
  const footerBindings = asArray(activeFooter.buttons).map((button) => ({
    source: "activeFooter",
    action: button.action ?? null,
    key: button.key ?? null,
    label: button.label ?? null,
    enabled: button.enabled ?? null,
    selected: button.selected ?? null,
    disabledReason: button.actionDisabled ?? null,
  }));
  const activePopup = (state.activePopupContract as JsonObject | undefined) ?? null;
  const actionsDialog = (state.actionsDialog as JsonObject | undefined) ?? null;
  const actionsDialogShortcutParity = (actionsDialog?.shortcutParity as JsonObject | undefined)
    ?? ((actionsDialog?.actions as JsonObject | undefined)?.shortcutParity as JsonObject | undefined)
    ?? null;
  const popupActions = asArray(actionsDialog?.visibleActions).map((action) => ({
    source: "actionsDialog",
    action: action.id ?? null,
    key: action.shortcut ?? null,
    canonicalKey: action.canonicalShortcut ?? null,
    label: action.label ?? null,
    section: action.section ?? null,
    enabled: action.enabled ?? null,
    disabledReason: action.actionDisabled ?? null,
  }));
  const bindings = [...footerBindings, ...popupActions];
  const nativeFooter = nativeFooterSnapshot(activeFooter, bindings);
  const duplicates = duplicateKeys(bindings);
  const classification = classify(targetReceipt, stateEnvelope, bindings);

  console.log(JSON.stringify({
    schemaVersion: 1,
    tool: "script-kit-devtools.keyboard",
    command: "keyboard.inspect",
    classification,
    session: args.session,
    requestedTarget: targetReceipt.requestedTarget ?? { selector },
    target: targetReceipt.resolvedTarget ?? null,
    keyboardPolicy: surfaceContract.keyboardPolicy ?? null,
    inputOwnership: surfaceContract.inputOwnership ?? null,
    focusPolicy: surfaceContract.focusPolicy ?? null,
    activeFooter: {
      owner: activeFooter.owner ?? null,
      activeSurface: activeFooter.activeSurface ?? null,
      expectedSurface: activeFooter.expectedSurface ?? null,
      nativeFooterHostInstalled: activeFooter.nativeFooterHostInstalled ?? null,
      buttonCount: activeFooter.buttonCount ?? footerBindings.length,
      actionSlotCount: activeFooter.actionSlotCount ?? null,
      contextChipCount: activeFooter.contextChipCount ?? null,
      duplicateShortcutKeys: activeFooter.duplicateShortcutKeys ?? [],
      slotContractViolation: activeFooter.slotContractViolation ?? null,
    },
    nativeFooter,
    activePopup,
    actionsDialogShortcutParity,
    bindings,
    duplicateKeys: duplicates,
    missingPrimitives: [
      bindings.length === 0 ? "keyboardBindings" : "",
      stateEnvelope.status === "error" ? "stateResult" : "",
      targetReceipt.classification !== "ok" ? "strictTargetIdentity" : "",
      nativeFooter.hostInstalled === true ? "nativeFooterActivationReceipt" : "",
    ].filter(Boolean),
    warnings: [
      duplicates.length > 0 ? `duplicate shortcut keys: ${duplicates.join(", ")}` : "",
      activePopup ? "popup contract is active; popup-first keyboard routing should be verified with devtools.act" : "",
    ].filter(Boolean),
    errors: [targetReceipt, stateEnvelope].filter((value) => value.status === "error"),
    state,
  }, null, 2));
}

await main();
