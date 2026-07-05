/**
 * The validator ladder. Ordered cheapest-first; the pipeline short-circuits to
 * the repair loop on the first hard failure. Every verdict carries a one-line
 * human summary (receipts for the UI) and, on failure, the raw tool output
 * that gets fed verbatim to the repair prompt — validators speak, the agent fixes.
 *
 *   1. typecheck   — tsc against the real v2 SDK (compiled together, so the
 *                    SDK's `declare global` blocks provide the ambient globals)
 *   2. api-scan    — the classifier re-run on the agent's OUTPUT
 *   3. metadata    — launcher-visible metadata preserved
 *   4. smoke       — real run under `bun --preload kit-sdk.ts`; pass = a valid
 *                    first protocol message on stdout, or a clean exit
 *   5. walkthrough — full run with SDK_TEST_AUTOSUBMIT=1 (every prompt
 *                    auto-resolves); pass = exit 0. Timeout is a warn, not a
 *                    fail — some scripts legitimately run long.
 *
 * 4 and 5 EXECUTE the script (sandboxed SK_PATH, but real filesystem/network
 * side effects run). The pipeline exposes --no-exec to skip them.
 */

import { join } from "node:path";
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { classify, loadCompatMap } from "./classify.ts";
import { extractEffectiveMetadata, metadataLosses } from "./metadata.ts";
import type { ValidatorVerdict } from "./types.ts";

const REPO_ROOT = join(import.meta.dir, "..", "..");
const SDK_PATH = join(REPO_ROOT, "scripts", "kit-sdk.ts");
const TSC_PATH = join(REPO_ROOT, "node_modules", ".bin", "tsc");

const TSC_TIMEOUT_MS = 120_000;
const SMOKE_TIMEOUT_MS = 10_000;
const WALKTHROUGH_TIMEOUT_MS = 20_000;

function tail(text: string, max = 2_000): string {
  const trimmed = text.trim();
  return trimmed.length <= max ? trimmed : `…${trimmed.slice(-max)}`;
}

export async function typecheck(portedPath: string): Promise<ValidatorVerdict> {
  // Same flags as scripts/check-sdk-types.ts; compiling the SDK alongside the
  // script makes its `declare global` blocks visible to the script.
  const proc = Bun.spawn(
    [
      TSC_PATH,
      "--noEmit",
      "--lib", "ES2022",
      "--target", "ES2022",
      "--types", "node,bun-types",
      "--moduleResolution", "bundler",
      "--module", "ES2022",
      // bun treats every file as a module; mirror that so import-free scripts
      // with top-level await don't trip TS1375
      "--moduleDetection", "force",
      "--skipLibCheck",
      SDK_PATH,
      portedPath,
    ],
    { cwd: REPO_ROOT, stdout: "pipe", stderr: "pipe" },
  );
  const killer = setTimeout(() => proc.kill(), TSC_TIMEOUT_MS);
  const [stdout, stderr, code] = await Promise.all([
    new Response(proc.stdout).text(),
    new Response(proc.stderr).text(),
    proc.exited,
  ]);
  clearTimeout(killer);
  if (code === 0) {
    return { id: "typecheck", outcome: "pass", summary: "tsc: 0 errors against the v2 SDK" };
  }
  // Surface only diagnostics about the ported file; SDK-internal noise is not
  // the agent's problem. TS2307 on bare specifiers is also tolerated: bun
  // auto-installs npm imports at runtime, so a package missing from the
  // repo's node_modules is not a porting error.
  const bareImportMiss = /error TS2307: Cannot find module '[^'.\/]/;
  const relevant = stdout
    .split("\n")
    .filter((l) => l.includes("error TS"))
    .filter((l) => !l.includes("kit-sdk.ts"))
    .filter((l) => !bareImportMiss.test(l));
  if (relevant.length === 0 && code !== 0) {
    return {
      id: "typecheck",
      outcome: "pass",
      summary: "tsc: 0 errors (unresolved npm imports tolerated — bun auto-installs)",
    };
  }
  const detail = relevant.length > 0 ? relevant.join("\n") : tail(stdout + stderr);
  return {
    id: "typecheck",
    outcome: "fail",
    summary: `tsc: ${relevant.length || "?"} error(s)`,
    detail,
  };
}

export function apiScan(portedSource: string): ValidatorVerdict {
  const result = classify(portedSource, loadCompatMap());
  const hard = result.findings.filter(
    (f) => f.status === "removed" || f.status === "stub" || f.status === "renamed",
  );
  const caveats = result.findings.filter((f) => f.status === "caveat");
  if (result.hasKitImport || hard.length > 0) {
    const lines = [
      ...(result.hasKitImport
        ? ['still imports "@johnlindquist/kit" — v2 preloads the SDK; remove the import']
        : []),
      ...hard.map(
        (f) =>
          `line ${f.line}: ${f.api} is ${f.status} in v2${f.replacement ? ` — use ${f.replacement}` : ""}${f.note ? ` (${f.note})` : ""}`,
      ),
    ];
    return {
      id: "api-scan",
      outcome: "fail",
      summary: `output still uses ${hard.length + (result.hasKitImport ? 1 : 0)} incompatible API(s)`,
      detail: lines.join("\n"),
    };
  }
  if (caveats.length > 0) {
    return {
      id: "api-scan",
      outcome: "warn",
      summary: `clean, with ${caveats.length} caveat API(s): ${[...new Set(caveats.map((f) => f.api))].join(", ")}`,
    };
  }
  return { id: "api-scan", outcome: "pass", summary: "only supported v2 APIs used" };
}

export function metadataCheck(
  originalSource: string,
  portedSource: string,
): ValidatorVerdict {
  const losses = metadataLosses(
    extractEffectiveMetadata(originalSource),
    extractEffectiveMetadata(portedSource),
  );
  if (losses.length > 0) {
    return {
      id: "metadata",
      outcome: "fail",
      summary: `metadata not preserved (${losses.length} issue(s))`,
      detail: losses.join("\n"),
    };
  }
  return { id: "metadata", outcome: "pass", summary: "launcher metadata preserved" };
}

interface ExecResult {
  firstMessage?: unknown;
  exitCode: number | null;
  timedOut: boolean;
  stderr: string;
}

async function execScript(
  portedPath: string,
  env: Record<string, string>,
  timeoutMs: number,
  stopAtFirstMessage: boolean,
): Promise<ExecResult> {
  const sandbox = mkdtempSync(join(tmpdir(), "sk-migrate-"));
  const proc = Bun.spawn(["bun", "--preload", SDK_PATH, portedPath], {
    cwd: REPO_ROOT,
    env: { ...process.env, SK_PATH: sandbox, ...env },
    stdin: "pipe",
    stdout: "pipe",
    stderr: "pipe",
  });

  let timedOut = false;
  const killer = setTimeout(() => {
    timedOut = true;
    proc.kill();
  }, timeoutMs);

  let firstMessage: unknown;
  const stderrPromise = new Response(proc.stderr).text();

  const reader = proc.stdout.getReader();
  const decoder = new TextDecoder();
  let buffer = "";
  try {
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      buffer += decoder.decode(value, { stream: true });
      let nl: number;
      while ((nl = buffer.indexOf("\n")) !== -1) {
        const line = buffer.slice(0, nl).trim();
        buffer = buffer.slice(nl + 1);
        if (!line) continue;
        try {
          const msg = JSON.parse(line);
          if (msg && typeof msg === "object" && typeof msg.type === "string") {
            firstMessage ??= msg;
            if (stopAtFirstMessage) {
              clearTimeout(killer);
              proc.kill();
            }
          }
        } catch {
          // non-protocol stdout noise is tolerated
        }
      }
      if (firstMessage && stopAtFirstMessage) break;
    }
  } catch {
    // stream torn down by kill — expected
  }

  const exitCode = await proc.exited.catch(() => null);
  clearTimeout(killer);
  const stderr = await stderrPromise.catch(() => "");
  return { firstMessage, exitCode, timedOut, stderr };
}

/** Strip the SDK's own console.error chatter from stderr before reporting. */
function scriptStderr(stderr: string): string {
  return stderr
    .split("\n")
    .filter((l) => !l.startsWith("[SDK]") && !l.startsWith("[SDK_DEBUG]") && !l.startsWith("[BENCH]"))
    .join("\n");
}

export async function smoke(portedPath: string): Promise<ValidatorVerdict> {
  const result = await execScript(portedPath, {}, SMOKE_TIMEOUT_MS, true);
  if (result.firstMessage) {
    const type = (result.firstMessage as { type: string }).type;
    return {
      id: "smoke",
      outcome: "pass",
      summary: `emitted a valid first protocol message ("${type}")`,
    };
  }
  if (!result.timedOut && result.exitCode === 0) {
    return {
      id: "smoke",
      outcome: "pass",
      summary: "ran to completion without prompts (exit 0)",
    };
  }
  const reason = result.timedOut
    ? `no protocol message within ${SMOKE_TIMEOUT_MS / 1000}s`
    : `exited with code ${result.exitCode} before any protocol message`;
  return {
    id: "smoke",
    outcome: "fail",
    summary: `smoke run failed: ${reason}`,
    detail: tail(scriptStderr(result.stderr)) || "(no script stderr)",
  };
}

export async function walkthrough(portedPath: string): Promise<ValidatorVerdict> {
  const result = await execScript(
    portedPath,
    { SDK_TEST_AUTOSUBMIT: "1", SDK_TEST_AUTOSUBMIT_DELAY: "25" },
    WALKTHROUGH_TIMEOUT_MS,
    false,
  );
  if (!result.timedOut && result.exitCode === 0) {
    return {
      id: "walkthrough",
      outcome: "pass",
      summary: "full auto-submit run completed (exit 0)",
    };
  }
  if (result.timedOut) {
    return {
      id: "walkthrough",
      outcome: "warn",
      summary: `still running after ${WALKTHROUGH_TIMEOUT_MS / 1000}s with auto-submit — inconclusive`,
    };
  }
  return {
    id: "walkthrough",
    outcome: "fail",
    summary: `auto-submit run crashed (exit ${result.exitCode})`,
    detail: tail(scriptStderr(result.stderr)) || "(no script stderr)",
  };
}

export interface LadderResult {
  verdicts: ValidatorVerdict[];
  /** The verdict that stopped the ladder, if any. */
  failed?: ValidatorVerdict;
}

export async function runLadder(
  portedPath: string,
  portedSource: string,
  originalSource: string,
  opts: { noExec?: boolean } = {},
): Promise<LadderResult> {
  const verdicts: ValidatorVerdict[] = [];
  const record = (v: ValidatorVerdict): ValidatorVerdict => {
    verdicts.push(v);
    return v;
  };

  let v = record(await typecheck(portedPath));
  if (v.outcome === "fail") return { verdicts, failed: v };

  v = record(apiScan(portedSource));
  if (v.outcome === "fail") return { verdicts, failed: v };

  v = record(metadataCheck(originalSource, portedSource));
  if (v.outcome === "fail") return { verdicts, failed: v };

  if (opts.noExec) {
    record({ id: "smoke", outcome: "skipped", summary: "skipped (--no-exec)" });
    record({ id: "walkthrough", outcome: "skipped", summary: "skipped (--no-exec)" });
    return { verdicts };
  }

  v = record(await smoke(portedPath));
  if (v.outcome === "fail") return { verdicts, failed: v };

  v = record(await walkthrough(portedPath));
  if (v.outcome === "fail") return { verdicts, failed: v };

  return { verdicts };
}
