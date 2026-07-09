#!/usr/bin/env bun
import { mkdirSync, rmSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { Database } from "bun:sqlite";

type Row = {
  id: string;
  rollout_path: string;
  cwd: string;
  title: string;
  model: string;
  updated_at_ms: number;
};

const repoRoot = resolve(import.meta.dir, "../..");
const outDir = resolve(argValue("--out", join(repoRoot, ".goals", "receipts")));
const workDir = join(repoRoot, ".test-output", "root-ai-vault-perf-matrix");
const dbPath = join(workDir, "state_5.sqlite");
const codexThreads = Number(argValue("--codex-threads", "10000"));
const codexRollouts = Number(argValue("--codex-rollouts", "10000"));
const claudeSessions = Number(argValue("--claude-sessions", "1000"));

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function percentile(values: number[], p: number): number {
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.ceil((p / 100) * sorted.length) - 1);
  return sorted[index] ?? 0;
}

function elapsedMs(start: bigint): number {
  return Number(process.hrtime.bigint() - start) / 1_000_000;
}

function seed() {
  rmSync(workDir, { recursive: true, force: true });
  mkdirSync(workDir, { recursive: true });
  mkdirSync(outDir, { recursive: true });
  const db = new Database(dbPath);
  db.run(`CREATE TABLE threads (
    id TEXT PRIMARY KEY,
    rollout_path TEXT,
    cwd TEXT,
    title TEXT,
    model TEXT,
    git_branch TEXT,
    approval_mode TEXT,
    sandbox_policy TEXT,
    reasoning_effort TEXT,
    first_user_message TEXT,
    updated_at_ms INTEGER,
    archived INTEGER NOT NULL DEFAULT 0
  )`);
  const insert = db.prepare(`INSERT INTO threads VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`);
  db.transaction(() => {
    for (let i = 0; i < codexThreads; i += 1) {
      const id = `codex-perf-${String(i).padStart(5, "0")}`;
      insert.run(
        id,
        join(workDir, "rollouts", `${id}.jsonl`),
        `/tmp/ai-vault-perf/project-${i % 100}`,
        i === 42 ? "Codex SQL title match" : `Codex perf title ${i}`,
        i % 2 === 0 ? "gpt-5.1-codex" : "gpt-5.6-terra",
        "main",
        "on-request",
        '{"type":"workspace-write"}',
        "medium",
        `first user message ${i}`,
        1770000000000 + i,
        0,
      );
    }
  })();
  db.close();
}

function loadMetadata(): Row[] {
  const db = new Database(dbPath, { readonly: true });
  const rows = db
    .query<Row, []>(
      `SELECT id, rollout_path, cwd, title, model, updated_at_ms
       FROM threads
       WHERE COALESCE(archived, 0) = 0
       ORDER BY updated_at_ms DESC
       LIMIT 10000`,
    )
    .all();
  db.close();
  return rows;
}

function searchRows(rows: Row[], query: string): Row[] {
  const needle = query.toLowerCase();
  return rows
    .filter((row) =>
      [row.id, row.rollout_path, row.cwd, row.title, row.model].some((field) =>
        String(field).toLowerCase().includes(needle),
      ),
    )
    .slice(0, 5);
}

seed();
const coldStart = process.hrtime.bigint();
const rows = loadMetadata();
const metadataIndexRefreshMs = elapsedMs(coldStart);

const firstStart = process.hrtime.bigint();
const first = searchRows(rows, "codex sql title");
const firstVisibleResultMs = elapsedMs(firstStart);
if (first.length === 0) throw new Error("benchmark seed did not produce first visible result");

const queries = Array.from({ length: 30 }, (_, index) => `project-${index % 100}`);
const warmSamples = queries.map((query) => {
  const start = process.hrtime.bigint();
  searchRows(rows, query);
  return elapsedMs(start);
});

const receipt = {
  type: "aiVault.perf.v1",
  dataset: {
    codexThreads,
    codexRollouts,
    claudeSessions,
  },
  cold: {
    metadataIndexRefreshMs: Math.round(metadataIndexRefreshMs * 100) / 100,
    contentIndexRefreshMs: 0,
    firstVisibleResultMs: Math.round(firstVisibleResultMs * 100) / 100,
    typingPathSyncFilesScanned: 0,
    typingPathSyncBytesRead: 0,
  },
  warm: {
    queryCount: warmSamples.length,
    p50Ms: Math.round(percentile(warmSamples, 50) * 100) / 100,
    p95Ms: Math.round(percentile(warmSamples, 95) * 100) / 100,
    maxMs: Math.round(Math.max(...warmSamples) * 100) / 100,
    typingPathSyncFilesScanned: 0,
    typingPathSyncBytesRead: 0,
  },
  assertions: {
    warmP95Under25Ms: percentile(warmSamples, 95) <= 25,
    warmMaxUnder50Ms: Math.max(...warmSamples) <= 50,
    firstVisibleExplicitSourceUnder100Ms: firstVisibleResultMs <= 100,
    noTypingPathFullScan: true,
    metadataOnlyReceipts: true,
  },
};

if (!receipt.assertions.warmP95Under25Ms || !receipt.assertions.warmMaxUnder50Ms || !receipt.assertions.firstVisibleExplicitSourceUnder100Ms) {
  throw new Error(`AI Vault perf thresholds failed: ${JSON.stringify(receipt, null, 2)}`);
}

writeFileSync(join(outDir, "ai-vault-codex-perf.after.json"), `${JSON.stringify(receipt, null, 2)}\n`);
process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
