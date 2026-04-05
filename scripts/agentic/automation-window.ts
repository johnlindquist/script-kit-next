#!/usr/bin/env bun
/**
 * scripts/agentic/automation-window.ts
 *
 * Resolve an automation target to the exact follow-on surface identity used by
 * native-input and verification helpers.
 */

import { resolve } from "path";

const SCHEMA_VERSION = 1;
const PROJECT_ROOT = resolve(import.meta.dir, "../..");

type AutomationTargetJson =
  | { type: "focused" }
  | { type: "main" }
  | { type: "id"; id: string }
  | { type: "kind"; kind: string; index?: number }
  | { type: "titleContains"; text: string };

interface ResolveResult {
  schemaVersion: number;
  status: "ok" | "error";
  targetJson?: AutomationTargetJson;
  windowKind?: string | null;
  automationWindowId?: string | null;
  title?: string | null;
  surfaceId?: string | null;
  error?: { code: string; message: string };
}

function stderrLog(event: string, fields: Record<string, unknown> = {}): void {
  console.error(JSON.stringify({ event, ts: new Date().toISOString(), ...fields }));
}

async function runTool(
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
  stderrLog("tool_complete", { label, exitCode });
  return { exitCode, stdout: stdout.trim(), stderr: stderr.trim() };
}

async function rpc(
  session: string,
  payload: Record<string, unknown>,
  expect: string,
  timeoutMs: number = 3000
): Promise<Record<string, unknown>> {
  const result = await runTool(
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
    "rpc"
  );
  if (result.exitCode !== 0) {
    throw new Error(result.stdout || result.stderr || `RPC failed with exit code ${result.exitCode}`);
  }
  return JSON.parse(result.stdout);
}

async function listSurfaces(): Promise<Record<string, unknown>> {
  const result = await runTool(
    ["bun", "scripts/agentic/window.ts", "list"],
    "list-surfaces"
  );
  if (result.exitCode !== 0) {
    throw new Error(result.stdout || result.stderr || "window.ts list failed");
  }
  return JSON.parse(result.stdout);
}

async function resolveTarget(
  session: string,
  targetJson: AutomationTargetJson
): Promise<ResolveResult> {
  stderrLog("resolve_start", { session, targetJson });

  let state: Record<string, unknown>;
  try {
    state = await rpc(
      session,
      {
        type: "getAcpState",
        requestId: `resolve-${Date.now()}`,
        target: targetJson,
      },
      "acpStateResult"
    );
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    stderrLog("resolve_rpc_failed", { error: message });
    return {
      schemaVersion: SCHEMA_VERSION,
      status: "error",
      targetJson,
      error: { code: "rpc_failed", message },
    };
  }

  // session.sh rpc returns { response: { type: "acpStateResult", resolvedTarget: {...}, ... } }
  const response = (state as { response?: Record<string, unknown> }).response;
  const resolvedTarget = (response?.resolvedTarget as Record<string, unknown>) ?? null;
  const windowKind = (resolvedTarget?.windowKind as string) ?? null;
  const automationWindowId = resolvedTarget?.windowId != null
    ? String(resolvedTarget.windowId)
    : null;
  const title = (resolvedTarget?.windowTitle as string) ?? (resolvedTarget?.title as string) ?? null;

  stderrLog("resolve_acp_state", { windowKind, automationWindowId, title });

  let surfaceId: string | null = null;
  try {
    const surfaceEnvelope = await listSurfaces();
    const surfaces =
      (surfaceEnvelope as { data?: { surfaces?: Array<Record<string, unknown>> } }).data?.surfaces ??
      [];

    if (title) {
      const titleMatch = surfaces.find(
        (surface) => typeof surface.title === "string" && surface.title === title
      );
      if (titleMatch) {
        surfaceId = (titleMatch.surfaceId as string) ?? null;
      }
    }

    if (!surfaceId && automationWindowId) {
      const windowIdMatch = surfaces.find(
        (surface) => String(surface.windowId) === String(automationWindowId)
      );
      if (windowIdMatch) {
        surfaceId = (windowIdMatch.surfaceId as string) ?? null;
      }
    }

    stderrLog("resolve_surface_match", { surfaceId, surfaceCount: surfaces.length });
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    stderrLog("resolve_surface_list_failed", { error: message });
  }

  return {
    schemaVersion: SCHEMA_VERSION,
    status: "ok",
    targetJson,
    windowKind,
    automationWindowId,
    title,
    surfaceId,
  };
}

function parseArgs() {
  const args = process.argv.slice(2);
  const subcmd = args[0] ?? "help";

  const sessionIdx = args.indexOf("--session");
  const session = sessionIdx >= 0 && args[sessionIdx + 1] ? args[sessionIdx + 1] : "default";

  const kindIdx = args.indexOf("--kind");
  const kind = kindIdx >= 0 && args[kindIdx + 1] ? args[kindIdx + 1] : "acpDetached";

  const indexIdx = args.indexOf("--index");
  const index = indexIdx >= 0 && args[indexIdx + 1] ? Number(args[indexIdx + 1]) : 0;

  const idIdx = args.indexOf("--id");
  const id = idIdx >= 0 && args[idIdx + 1] ? args[idIdx + 1] : undefined;

  const titleIdx = args.indexOf("--title");
  const titleText = titleIdx >= 0 && args[titleIdx + 1] ? args[titleIdx + 1] : undefined;

  return { subcmd, session, kind, index, id, titleText };
}

const { subcmd, session, kind, index, id, titleText } = parseArgs();

switch (subcmd) {
  case "resolve": {
    let targetJson: AutomationTargetJson;
    if (id) {
      targetJson = { type: "id", id };
    } else if (titleText) {
      targetJson = { type: "titleContains", text: titleText };
    } else if (kind === "main") {
      targetJson = { type: "main" };
    } else if (kind === "focused") {
      targetJson = { type: "focused" };
    } else {
      targetJson = { type: "kind", kind, index };
    }

    const result = await resolveTarget(session, targetJson);
    console.log(JSON.stringify(result, null, 2));
    process.exit(result.status === "ok" ? 0 : 1);
    break;
  }

  case "help":
  case "--help": {
    console.log(`Usage: bun scripts/agentic/automation-window.ts <command> [options]

Commands:
  resolve    Resolve an automation target to an exact surface identity
  help       Show this help

Options:
  --session NAME    Session name (default: "default")
  --kind KIND       Target kind: acpDetached | main | focused
  --index N         Kind index (default: 0)
  --id WINDOW_ID    Resolve by exact automation window ID
  --title TEXT      Resolve by titleContains target
`);
    process.exit(0);
  }

  default:
    console.error(`Unknown command: ${subcmd}`);
    process.exit(2);
}
