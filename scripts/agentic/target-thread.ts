#!/usr/bin/env bun
/**
 * Shared exact-target identity guard for advanced agentic scenarios.
 *
 * Kind/index targets are discovery-only. A proof run must promote them to a
 * stable exact `{ type: "id" }` target once, then reuse that target for every
 * RPC, native input focus receipt, and strict capture assertion.
 */

import { resolve } from "path";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");

export type AutomationTargetJson =
  | { type: "focused" }
  | { type: "main" }
  | { type: "id"; id: string }
  | { type: "kind"; kind: string; index?: number }
  | { type: "titleContains"; text: string };

export interface TargetThreadIdentity {
  targetJson: AutomationTargetJson;
  automationWindowId: string;
  osWindowId: number | null;
  surfaceId: string | null;
  windowKind: string | null;
  title?: string | null;
  semanticSurface?: string | null;
  parentAutomationWindowId?: string | null;
  popupFamily?: string | null;
  popupId?: string | null;
  acpViewId?: string | null;
  acpGeneration?: number | null;
}

export interface TargetThreadFailure {
  code:
    | "target_resolution_failed"
    | "target_identity_drift"
    | "missing_os_window_id"
    | "missing_surface_id"
    | "wrong_popup_family"
    | "wrong_popup_id"
    | "insufficient_target_count"
    | "untargeted_rpc_forbidden";
  expected?: Partial<TargetThreadIdentity>;
  actual?: Partial<TargetThreadIdentity>;
  stepName: string;
  message: string;
}

export interface StepReceipt {
  name: string;
  status: "pass" | "fail" | "error" | "skipped";
  output: unknown;
  durationMs: number;
}

export async function runTool(
  cmd: string[],
  label: string
): Promise<{ exitCode: number; stdout: string; stderr: string }> {
  const proc = Bun.spawn(cmd, {
    stdout: "pipe",
    stderr: "pipe",
    cwd: PROJECT_ROOT,
  });
  const stdout = await new Response(proc.stdout).text();
  const stderr = await new Response(proc.stderr).text();
  const exitCode = await proc.exited;
  console.error(
    JSON.stringify({
      event: "target_thread.tool_complete",
      label,
      exitCode,
      ts: new Date().toISOString(),
    })
  );
  return { exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
}

function parseJson(text: string): Record<string, unknown> {
  try {
    return JSON.parse(text);
  } catch {
    return { raw: text };
  }
}

function responsePayload(envelope: Record<string, unknown>): Record<string, unknown> {
  const response = envelope.response;
  return response && typeof response === "object"
    ? (response as Record<string, unknown>)
    : envelope;
}

function identityFromInspect(
  parsed: Record<string, unknown>,
  exactId?: string
): TargetThreadIdentity | null {
  if (parsed.status && parsed.status !== "ok") return null;

  const inspect = parsed.inspect && typeof parsed.inspect === "object"
    ? (parsed.inspect as Record<string, unknown>)
    : {};
  const automationWindowId =
    parsed.automationWindowId != null
      ? String(parsed.automationWindowId)
      : inspect.windowId != null
        ? String(inspect.windowId)
        : exactId ?? "";

  if (!automationWindowId) return null;

  const osWindowId =
    typeof parsed.osWindowId === "number"
      ? parsed.osWindowId
      : typeof inspect.osWindowId === "number"
        ? (inspect.osWindowId as number)
        : null;

  const windowKind =
    typeof parsed.windowKind === "string"
      ? parsed.windowKind
      : typeof inspect.windowKind === "string"
        ? (inspect.windowKind as string)
        : null;

  const title =
    typeof parsed.title === "string"
      ? parsed.title
      : typeof inspect.title === "string"
        ? (inspect.title as string)
        : null;

  const semanticSurface =
    typeof inspect.semanticSurface === "string"
      ? (inspect.semanticSurface as string)
      : null;

  return {
    targetJson: { type: "id", id: automationWindowId },
    automationWindowId,
    osWindowId,
    surfaceId: typeof parsed.surfaceId === "string" ? parsed.surfaceId : null,
    windowKind,
    title,
    semanticSurface,
    parentAutomationWindowId:
      typeof inspect.parentAutomationWindowId === "string"
        ? (inspect.parentAutomationWindowId as string)
        : null,
    popupFamily:
      typeof inspect.popupFamily === "string"
        ? (inspect.popupFamily as string)
        : null,
    popupId:
      typeof inspect.popupId === "string"
        ? (inspect.popupId as string)
        : automationWindowId.startsWith("acp-") ? automationWindowId : null,
    acpViewId:
      typeof inspect.acpViewId === "string" ? (inspect.acpViewId as string) : null,
    acpGeneration:
      typeof inspect.acpGeneration === "number"
        ? (inspect.acpGeneration as number)
        : null,
  };
}

export async function promoteExactTarget(opts: {
  session: string;
  kind: string;
  index: number;
  expected?: {
    automationWindowId?: string;
    popupFamily?: string;
    popupId?: string;
    windowKind?: string;
  };
}): Promise<TargetThreadIdentity> {
  const result = await runTool(
    [
      "bun",
      "scripts/agentic/automation-window.ts",
      "inspect",
      "--session",
      opts.session,
      "--kind",
      opts.kind,
      "--index",
      String(opts.index),
    ],
    "promote-exact-target"
  );
  const parsed = parseJson(result.stdout);
  const identity = identityFromInspect(parsed);

  if (result.exitCode !== 0 || !identity) {
    throw Object.assign(
      new Error(result.stdout || result.stderr || "target promotion failed"),
      {
        failure: {
          code: "target_resolution_failed",
          stepName: "promote-exact-target",
          message: "Could not promote kind/index target to exact automation id",
          actual: parsed,
        } satisfies TargetThreadFailure,
      }
    );
  }

  const expected = opts.expected ?? {};
  if (
    expected.automationWindowId &&
    identity.automationWindowId !== expected.automationWindowId
  ) {
    throwDrift("target_identity_drift", "promote-exact-target", expected, identity);
  }
  if (expected.windowKind && identity.windowKind !== expected.windowKind) {
    throwDrift("target_identity_drift", "promote-exact-target", expected, identity);
  }
  if (expected.popupFamily && identity.popupFamily !== expected.popupFamily) {
    throwDrift("wrong_popup_family", "promote-exact-target", expected, identity);
  }
  if (expected.popupId && identity.popupId !== expected.popupId) {
    throwDrift("wrong_popup_id", "promote-exact-target", expected, identity);
  }

  return identity;
}

function throwDrift(
  code: TargetThreadFailure["code"],
  stepName: string,
  expected: Partial<TargetThreadIdentity>,
  actual: Partial<TargetThreadIdentity>
): never {
  throw Object.assign(new Error(`${code} at ${stepName}`), {
    failure: {
      code,
      stepName,
      expected,
      actual,
      message: `${code}: target identity did not match expected values`,
    } satisfies TargetThreadFailure,
  });
}

export async function assertTargetStable(opts: {
  session: string;
  identity: TargetThreadIdentity;
  stepName: string;
}): Promise<
  | { ok: true; identity: TargetThreadIdentity }
  | { ok: false; failure: TargetThreadFailure }
> {
  const result = await runTool(
    [
      "bun",
      "scripts/agentic/automation-window.ts",
      "inspect",
      "--session",
      opts.session,
      "--id",
      opts.identity.automationWindowId,
    ],
    `assert-target-stable:${opts.stepName}`
  );
  const actual = identityFromInspect(parseJson(result.stdout), opts.identity.automationWindowId);
  if (result.exitCode !== 0 || !actual) {
    return {
      ok: false,
      failure: {
        code: "target_resolution_failed",
        stepName: opts.stepName,
        expected: opts.identity,
        message: "Exact target could not be inspected during stability check",
      },
    };
  }

  const expected = opts.identity;
  const driftFields: Array<keyof TargetThreadIdentity> = [
    "automationWindowId",
    "surfaceId",
    "windowKind",
    "osWindowId",
    "acpViewId",
    "acpGeneration",
  ];
  for (const field of driftFields) {
    if (expected[field] != null && actual[field] !== expected[field]) {
      return {
        ok: false,
        failure: {
          code: "target_identity_drift",
          stepName: opts.stepName,
          expected: { [field]: expected[field] } as Partial<TargetThreadIdentity>,
          actual: { [field]: actual[field] } as Partial<TargetThreadIdentity>,
          message: `Target identity drifted at ${String(field)}`,
        },
      };
    }
  }
  return { ok: true, identity: actual };
}

export async function targetedRpc(opts: {
  session: string;
  identity: TargetThreadIdentity;
  requestId: string;
  command: Record<string, unknown>;
  expect: string;
  timeout?: number;
  stepName?: string;
}): Promise<StepReceipt> {
  const start = Date.now();
  if (opts.identity.targetJson.type !== "id") {
    return {
      name: opts.stepName ?? opts.requestId,
      status: "error",
      durationMs: Date.now() - start,
      output: {
        failure: {
          code: "untargeted_rpc_forbidden",
          stepName: opts.stepName ?? opts.requestId,
          message: "targetedRpc requires an exact id target",
          actual: opts.identity,
        } satisfies TargetThreadFailure,
      },
    };
  }

  const payload = {
    ...opts.command,
    requestId: opts.requestId,
    target: opts.identity.targetJson,
  };
  const result = await runTool(
    [
      "bash",
      "scripts/agentic/session.sh",
      "rpc",
      opts.session,
      JSON.stringify(payload),
      "--expect",
      opts.expect,
      "--timeout",
      String(opts.timeout ?? 5000),
    ],
    `targeted-rpc:${opts.requestId}`
  );
  return {
    name: opts.stepName ?? opts.requestId,
    status: result.exitCode === 0 ? "pass" : result.exitCode === 2 ? "error" : "fail",
    output: responsePayload(parseJson(result.stdout || result.stderr)),
    durationMs: Date.now() - start,
  };
}

export async function listNativePeerWindows(opts: {
  family: "acpDetached" | "promptPopup";
}): Promise<Array<Record<string, unknown>>> {
  const result = await runTool(["bun", "scripts/agentic/window.ts", "list"], "window-list");
  if (result.exitCode !== 0) return [];
  const parsed = parseJson(result.stdout);
  const surfaces = (parsed.data as { surfaces?: Array<Record<string, unknown>> } | undefined)
    ?.surfaces ?? [];
  if (opts.family === "acpDetached") {
    return surfaces.filter((surface) => {
      const surfaceId = String(surface.surfaceId ?? "");
      const title = String(surface.title ?? "");
      return surfaceId.includes("acp") || title.toLowerCase().includes("agent");
    });
  }
  return surfaces.filter((surface) => {
    const title = String(surface.title ?? "");
    return title.toLowerCase().includes("popup") || title.toLowerCase().includes("prompt");
  });
}
