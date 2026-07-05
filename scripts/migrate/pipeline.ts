/**
 * Per-script migration pipeline:
 *
 *   classify → port (agent; skipped for `ready` scripts) → validator ladder
 *            → repair loop (validator output fed back raw, max N attempts)
 *            → honesty refute pass (only when a rewrite claims zero changes)
 *
 * Copy, never move: the v1 source is read-only; verified ports are written to
 * a separate plugin directory with a provenance trailer.
 */

import { basename, join } from "node:path";
import { mkdirSync, mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { classify, formatFindings, loadCompatMap } from "./classify.ts";
import { callAgent, extractBlock, parseJsonBlock } from "./agent.ts";
import { runLadder } from "./validators.ts";
import type {
  Classification,
  Finding,
  MigrationNote,
  PipelineOptions,
  PortAttempt,
  PortResult,
  ValidatorVerdict,
} from "./types.ts";

const PORT_PROMPT = await Bun.file(
  join(import.meta.dir, "prompts", "port.md"),
).text();
const REPAIR_PROMPT = await Bun.file(
  join(import.meta.dir, "prompts", "repair.md"),
).text();
const HONESTY_PROMPT = await Bun.file(
  join(import.meta.dir, "prompts", "honesty.md"),
).text();

const DEFAULT_MAX_REPAIRS = 3;

function fill(template: string, vars: Record<string, string>): string {
  return template.replace(/\{\{(\w+)\}\}/g, (_, key) => vars[key] ?? "");
}

/** Compat-map guidance for exactly the APIs this script was flagged for. */
function compatGuidance(classification: Classification): string {
  const map = loadCompatMap();
  const apis = new Map<string, Finding>();
  for (const f of classification.findings) {
    if (f.status !== "supported" && !apis.has(f.api)) apis.set(f.api, f);
  }
  if (apis.size === 0) return "(none needed — no incompatible APIs found)";
  const sections: string[] = [];
  for (const api of apis.keys()) {
    const entry = map.apis[api];
    if (!entry) continue;
    let s = `### ${api} (${entry.status})`;
    if (entry.replacement) s += `\nReplacement: ${entry.replacement}`;
    if (entry.note) s += `\n${entry.note}`;
    if (entry.snippet) s += `\n\`\`\`ts\n${entry.snippet}\n\`\`\``;
    sections.push(s);
  }
  return sections.join("\n\n");
}

interface HonestyVerdict {
  verdict: "honest" | "dropped-behavior";
  dropped: string[];
  reasoning: string;
}

const PROVENANCE_MARKER = "// Ported-from:";

function withProvenance(content: string, originalPath: string): string {
  if (content.includes(PROVENANCE_MARKER)) return content;
  return `${content.trimEnd()}\n\n${PROVENANCE_MARKER} ${originalPath} (Script Kit v1, migrated by scripts/migrate)\n`;
}

export async function portScript(
  scriptPath: string,
  opts: PipelineOptions,
): Promise<PortResult> {
  const file = basename(scriptPath);
  const progress = (phase: string) => opts.onProgress?.(file, phase);
  const maxRepairs = opts.maxRepairs ?? DEFAULT_MAX_REPAIRS;

  let source: string;
  try {
    source = await Bun.file(scriptPath).text();
  } catch (e) {
    return {
      file,
      bucket: "ready",
      status: "error",
      attempts: [],
      failure: `unreadable: ${e}`,
      agentUsed: false,
    };
  }

  progress("classify");
  const classification = classify(source);
  const scratch = mkdtempSync(join(tmpdir(), "sk-port-"));
  const attempts: PortAttempt[] = [];

  const useAgent = opts.forceAgent || classification.bucket !== "ready";
  let candidate = source;
  let note: MigrationNote | undefined;

  const baseVars = {
    FILENAME: file,
    SCRIPT_SOURCE: source,
    FINDINGS: formatFindings(classification),
    COMPAT_GUIDANCE: compatGuidance(classification),
  };

  for (let attempt = 1; attempt <= 1 + maxRepairs; attempt++) {
    let agentCostUsd: number | undefined;

    if (useAgent) {
      progress(attempt === 1 ? "porting" : `repair ${attempt - 1}/${maxRepairs}`);
      const prompt =
        attempt === 1
          ? fill(PORT_PROMPT, baseVars)
          : fill(REPAIR_PROMPT, {
              ...baseVars,
              PREVIOUS_OUTPUT: candidate,
              VALIDATOR_ID: attempts.at(-1)?.verdicts.at(-1)?.id ?? "output-contract",
              VALIDATOR_FAILURE:
                attempts.at(-1)?.verdicts.at(-1)?.detail ??
                attempts.at(-1)?.verdicts.at(-1)?.summary ??
                "output contract violated: PORTED_SCRIPT block missing",
            });

      let text: string;
      try {
        const result = await callAgent(prompt);
        text = result.text;
        agentCostUsd = result.costUsd;
      } catch (e) {
        return {
          file,
          bucket: classification.bucket,
          status: "error",
          attempts,
          failure: String(e),
          agentUsed: true,
        };
      }

      const extracted = extractBlock(text, "PORTED_SCRIPT");
      note = parseJsonBlock<MigrationNote>(text, "MIGRATION_NOTE") ?? note;
      if (!extracted) {
        attempts.push({
          attempt,
          agentCostUsd,
          verdicts: [
            {
              id: "api-scan",
              outcome: "fail",
              summary: "output contract violated: PORTED_SCRIPT block missing",
              detail:
                "Your response did not contain the ===PORTED_SCRIPT=== block. Follow the output contract exactly.",
            } satisfies ValidatorVerdict,
          ],
        });
        continue;
      }
      candidate = extracted;
    }

    progress("validating");
    const candidatePath = join(scratch, `attempt-${attempt}-${file}`);
    await Bun.write(candidatePath, candidate);
    const ladder = await runLadder(candidatePath, candidate, source, {
      noExec: opts.noExec,
    });
    attempts.push({ attempt, verdicts: ladder.verdicts, note, agentCostUsd });

    if (ladder.failed) {
      if (!useAgent) {
        // A verbatim copy of a "ready" script failed the ladder — the classifier
        // missed something. Don't loop; hand it to a human with the receipts.
        return {
          file,
          bucket: classification.bucket,
          status: "needs-review",
          attempts,
          note,
          failure: `classified ready, but verbatim copy failed ${ladder.failed.id}: ${ladder.failed.summary}`,
          agentUsed: false,
        };
      }
      continue; // repair loop
    }

    // Ladder passed. Honesty refute pass for suspicious zero-change claims.
    if (
      useAgent &&
      opts.honesty !== false &&
      classification.bucket === "needs-rewrite" &&
      (note?.behavior_changes ?? []).length === 0
    ) {
      progress("honesty check");
      try {
        const { text } = await callAgent(
          fill(HONESTY_PROMPT, {
            FILENAME: file,
            SCRIPT_SOURCE: source,
            PORTED_SOURCE: candidate,
          }),
        );
        const verdict = parseJsonBlock<HonestyVerdict>(text, "HONESTY_VERDICT");
        const honestyVerdict: ValidatorVerdict =
          verdict?.verdict === "dropped-behavior"
            ? {
                id: "honesty",
                outcome: "fail",
                summary: `refuter found dropped behavior: ${verdict.dropped.join("; ")}`,
                detail: verdict.reasoning,
              }
            : {
                id: "honesty",
                outcome: "pass",
                summary: "zero-change claim survived the refute pass",
              };
        attempts.at(-1)?.verdicts.push(honestyVerdict);
        if (honestyVerdict.outcome === "fail") {
          return {
            file,
            bucket: classification.bucket,
            status: "needs-review",
            attempts,
            note,
            failure: `port passed all mechanical validators but the honesty check found silently dropped behavior: ${verdict?.dropped.join("; ")}`,
            agentUsed: true,
          };
        }
      } catch {
        attempts.at(-1)?.verdicts.push({
          id: "honesty",
          outcome: "warn",
          summary: "refute pass unavailable (agent error) — claim unaudited",
        });
      }
    }

    // Success: write the port (unless dry run).
    const hasWarnings = attempts
      .at(-1)!
      .verdicts.some((v) => v.outcome === "warn");
    let portedPath: string | undefined;
    if (!opts.dryRun) {
      mkdirSync(opts.outDir, { recursive: true });
      portedPath = join(opts.outDir, file);
      await Bun.write(portedPath, withProvenance(candidate, scriptPath));
    }
    progress("verified");
    return {
      file,
      bucket: classification.bucket,
      status: hasWarnings ? "verified-with-warnings" : "verified",
      portedPath,
      attempts,
      note,
      agentUsed: useAgent,
    };
  }

  // Repair budget exhausted.
  const last = attempts.at(-1);
  const lastFail = last?.verdicts.find((v) => v.outcome === "fail");
  return {
    file,
    bucket: classification.bucket,
    status: "needs-review",
    attempts,
    note,
    failure: `repair budget exhausted after ${attempts.length} attempt(s); last failure [${lastFail?.id}]: ${lastFail?.summary}\n${lastFail?.detail ?? ""}`,
    agentUsed: useAgent,
  };
}
