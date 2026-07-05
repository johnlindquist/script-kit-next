#!/usr/bin/env bun
/**
 * Script Kit v1 → v2 migration CLI.
 *
 *   bun scripts/migrate/cli.ts scan <dir|file>
 *       Classify scripts against the v2 compat map. Read-only, instant, free.
 *
 *   bun scripts/migrate/cli.ts port <dir|file> [flags]
 *       Run the full agentic pipeline: port → validator ladder → repair loop.
 *       Copies, never moves. Writes a migration-report.json next to the ports.
 *
 * Flags for `port`:
 *   --out <dir>        Output dir (default: ~/.scriptkit/plugins/v1-imports/scripts)
 *   --dry-run          Validate everything, write nothing
 *   --no-exec          Skip validators that execute the script (smoke, walkthrough)
 *   --force-agent      Route even clean `ready` scripts through the agent
 *   --max-repairs <n>  Repair-loop budget per script (default 3)
 *   --no-honesty       Skip the refute pass on zero-change rewrite claims
 *   --concurrency <n>  Scripts ported in parallel (default 4)
 *   --json             Machine-readable output on stdout
 *   --progress-jsonl   ALL stdout becomes JSONL events for a supervising UI:
 *                      {"event":"start","files":[...]}, then per-script
 *                      {"event":"phase","file","phase"} and
 *                      {"event":"result","result":<PortResult>}, finally
 *                      {"event":"done","report":<full report>}
 *
 * Agent backend: `claude -p --output-format json` by default; override with
 * SK_MIGRATE_AGENT_CMD (any command: prompt on stdin, response on stdout).
 */

import { existsSync, readdirSync, readFileSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { basename, join, resolve } from "node:path";
import { classify, formatFindings } from "./classify.ts";
import { portScript } from "./pipeline.ts";
import type { Bucket, PortResult } from "./types.ts";

function collectScripts(target: string): string[] {
  const abs = resolve(target);
  if (!existsSync(abs)) {
    console.error(`not found: ${abs}`);
    process.exit(1);
  }
  if (statSync(abs).isFile()) return [abs];
  return readdirSync(abs)
    .filter((f) => /\.(ts|js|mjs|tsx)$/.test(f) && !f.endsWith(".d.ts"))
    .map((f) => join(abs, f))
    .sort();
}

function parseFlags(argv: string[]) {
  const flags: Record<string, string | boolean> = {};
  const positional: string[] = [];
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a === "--out" || a === "--max-repairs" || a === "--concurrency") {
      flags[a.slice(2)] = argv[++i];
    } else if (a.startsWith("--")) {
      flags[a.slice(2)] = true;
    } else {
      positional.push(a);
    }
  }
  return { flags, positional };
}

const BUCKET_LABEL: Record<Bucket, string> = {
  ready: "READY — imports as-is",
  "needs-changes": "NEEDS CHANGES — mechanical, agent will handle",
  "needs-rewrite": "NEEDS REWRITE — uses APIs v2 doesn't have",
};

async function cmdScan(target: string, json: boolean) {
  const files = collectScripts(target);
  const rows = files.map((f) => {
    const source = readFileSync(f, "utf8");
    const c = classify(source);
    return { file: basename(f), path: f, ...c };
  });

  if (json) {
    console.log(JSON.stringify(rows, null, 2));
    return;
  }

  console.log(`Scanned ${rows.length} script(s) in ${resolve(target)}\n`);
  for (const bucket of ["ready", "needs-changes", "needs-rewrite"] as Bucket[]) {
    const group = rows.filter((r) => r.bucket === bucket);
    if (group.length === 0) continue;
    console.log(`${BUCKET_LABEL[bucket]} (${group.length})`);
    for (const r of group) {
      const apis = [
        ...new Set(
          r.findings.filter((f) => f.status !== "supported").map((f) => f.api),
        ),
      ];
      const extra = r.hasKitImport ? ["@johnlindquist/kit import"] : [];
      const detailStr = [...extra, ...apis].join(", ");
      console.log(`  ${bucket === "ready" ? "✓" : bucket === "needs-changes" ? "~" : "✗"} ${r.file}${detailStr ? `   ${detailStr}` : ""}`);
    }
    console.log();
  }
}

function statusIcon(r: PortResult): string {
  switch (r.status) {
    case "verified":
      return "✓";
    case "verified-with-warnings":
      return "⚠";
    case "needs-review":
      return "!";
    case "error":
      return "✗";
  }
}

async function cmdPort(target: string, flags: Record<string, string | boolean>) {
  const files = collectScripts(target);
  const outDir =
    typeof flags.out === "string"
      ? resolve(flags.out)
      : join(homedir(), ".scriptkit", "plugins", "v1-imports", "scripts");
  const concurrency = Math.max(1, parseInt(String(flags.concurrency ?? "4"), 10) || 4);
  const jsonl = flags["progress-jsonl"] === true;
  const json = flags.json === true || jsonl;
  const emit = (event: Record<string, unknown>) => {
    if (jsonl) console.log(JSON.stringify(event));
  };

  const opts = {
    outDir,
    dryRun: flags["dry-run"] === true,
    noExec: flags["no-exec"] === true,
    forceAgent: flags["force-agent"] === true,
    honesty: flags["no-honesty"] === true ? false : undefined,
    maxRepairs: flags["max-repairs"]
      ? parseInt(String(flags["max-repairs"]), 10)
      : undefined,
    onProgress: jsonl
      ? (file: string, phase: string) => emit({ event: "phase", file, phase })
      : json
        ? undefined
        : (file: string, phase: string) => console.log(`  ◐ ${file} — ${phase}`),
  };

  if (!json) {
    console.log(
      `Porting ${files.length} script(s) → ${opts.dryRun ? "(dry run)" : outDir}\n`,
    );
  }
  emit({ event: "start", files: files.map((f) => basename(f)), outDir, dryRun: opts.dryRun === true });

  // Simple worker pool.
  const results: PortResult[] = [];
  let next = 0;
  await Promise.all(
    Array.from({ length: Math.min(concurrency, files.length) }, async () => {
      while (next < files.length) {
        const idx = next++;
        results[idx] = await portScript(files[idx], opts);
        emit({ event: "result", result: results[idx] });
      }
    }),
  );

  const report = {
    source: resolve(target),
    outDir: opts.dryRun ? null : outDir,
    dryRun: opts.dryRun,
    results,
  };

  if (jsonl) {
    emit({ event: "done", report });
  } else if (json) {
    console.log(JSON.stringify(report, null, 2));
  } else {
    console.log("\n──────────────────────────────────────────────────");
    for (const r of results) {
      const cost = r.attempts.reduce((s, a) => s + (a.agentCostUsd ?? 0), 0);
      const costStr = cost > 0 ? ` ($${cost.toFixed(2)})` : "";
      console.log(`${statusIcon(r)} ${r.file} — ${r.status}${costStr}`);
      if (r.note?.summary) console.log(`    ${r.note.summary}`);
      for (const change of r.note?.behavior_changes ?? []) {
        console.log(`    Δ ${change}`);
      }
      const lastVerdicts = r.attempts.at(-1)?.verdicts ?? [];
      for (const v of lastVerdicts) {
        const mark =
          v.outcome === "pass" ? "·" : v.outcome === "warn" ? "⚠" : v.outcome === "skipped" ? "○" : "✗";
        console.log(`      ${mark} ${v.id}: ${v.summary}`);
      }
      if (r.failure) console.log(`    NEEDS YOU: ${r.failure.split("\n")[0]}`);
    }
    const verified = results.filter((r) => r.status.startsWith("verified")).length;
    const review = results.filter((r) => r.status === "needs-review").length;
    const errors = results.filter((r) => r.status === "error").length;
    console.log(
      `\n${verified} verified · ${review} need review · ${errors} errors`,
    );
  }

  if (!opts.dryRun) {
    const reportPath = join(outDir, "..", "migration-report.json");
    await Bun.write(reportPath, JSON.stringify(report, null, 2));
    if (!json) console.log(`report: ${reportPath}`);
  }

  if (results.some((r) => r.status === "error")) process.exit(2);
}

const { flags, positional } = parseFlags(process.argv.slice(2));
const [command, target] = positional;

if (command === "scan" && target) {
  await cmdScan(target, flags.json === true);
} else if (command === "port" && target) {
  await cmdPort(target, flags);
} else {
  console.log(
    "usage:\n  bun scripts/migrate/cli.ts scan <dir|file> [--json]\n  bun scripts/migrate/cli.ts port <dir|file> [--out <dir>] [--dry-run] [--no-exec] [--force-agent] [--max-repairs n] [--no-honesty] [--concurrency n] [--json]",
  );
  process.exit(command ? 1 : 0);
}
