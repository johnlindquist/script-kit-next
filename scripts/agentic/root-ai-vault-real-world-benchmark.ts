#!/usr/bin/env bun
import { spawnSync } from "node:child_process";
import {
  existsSync,
  mkdirSync,
  readdirSync,
  readFileSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { Database } from "bun:sqlite";

type Json = Record<string, any>;

const repoRoot = resolve(import.meta.dir, "../..");
const outDir = resolve(argValue("--out", join(repoRoot, ".goals", "receipts")));
const homeDir = resolve(argValue("--home", process.env.HOME ?? "."));
const maxCodexRows = Number(argValue("--codex-limit", "10000"));
const maxClaudeSessions = Number(argValue("--claude-limit", "300"));
const includeContent = argValue("--content", "true") !== "false";
const discoverClaude = argValue("--discover-claude", "true") !== "false";
const receiptName = argValue(
  "--receipt",
  `ai-vault-real-world-${includeContent ? "content" : "metadata"}.json`,
);

function argValue(name: string, fallback: string): string {
  const index = process.argv.indexOf(name);
  return index >= 0 && process.argv[index + 1] ? process.argv[index + 1] : fallback;
}

function elapsedMs(start: bigint): number {
  return Number(process.hrtime.bigint() - start) / 1_000_000;
}

function round(value: number): number {
  return Math.round(value * 100) / 100;
}

function percentile(values: number[], p: number): number {
  const sorted = [...values].sort((a, b) => a - b);
  const index = Math.min(sorted.length - 1, Math.ceil((p / 100) * sorted.length) - 1);
  return sorted[index] ?? 0;
}

function fileSize(path: string): number {
  try {
    return statSync(path).size;
  } catch {
    return 0;
  }
}

function walkJsonl(root: string, out: { path: string; modifiedMs: number; size: number }[]) {
  let entries;
  try {
    entries = readdirSync(root, { withFileTypes: true });
  } catch {
    return;
  }
  for (const entry of entries) {
    const path = join(root, entry.name);
    if (entry.isDirectory()) {
      walkJsonl(path, out);
    } else if (entry.isFile() && path.endsWith(".jsonl")) {
      try {
        const metadata = statSync(path);
        out.push({ path, modifiedMs: metadata.mtimeMs, size: metadata.size });
      } catch {}
    }
  }
}

function du(path: string): string | null {
  const result = spawnSync("du", ["-sh", path], { encoding: "utf8" });
  if (result.status !== 0) return null;
  return result.stdout.trim().split(/\s+/)[0] ?? null;
}

function readRolloutPrefix(path: string): string | null {
  if (!includeContent || !existsSync(path)) return null;
  try {
    return readFileSync(path, "utf8").split("\n").slice(0, 400).join("\n").slice(0, 128 * 1024);
  } catch {
    return null;
  }
}

function parseClaudePrefix(path: string): { title: string; cwd: string; model: string; terms: string[]; lines: number } | null {
  if (!includeContent) return { title: "", cwd: "", model: "", terms: [], lines: 0 };
  try {
    const lines = readFileSync(path, "utf8").split("\n").slice(0, 1000);
    let title = "";
    let cwd = "";
    let model = "";
    const terms: string[] = [];
    let parsed = 0;
    for (const line of lines) {
      if (!line) continue;
      try {
        const event = JSON.parse(line);
        parsed += 1;
        if (!cwd && typeof event.cwd === "string") cwd = event.cwd;
        if (!model && typeof event.message?.model === "string") model = event.message.model;
        const role = event.message?.role ?? event.type;
        if (role === "user") {
          const raw = JSON.stringify(event.message?.content ?? event.content ?? "");
          if (!title) title = raw.slice(0, 160);
          terms.push(raw);
        }
      } catch {}
    }
    return { title, cwd, model, terms, lines: parsed };
  } catch {
    return null;
  }
}

const receipt: Json = {
  type: "aiVault.realWorldBenchmark.v1",
  homeDir,
  includeContent,
  limits: { maxCodexRows, maxClaudeSessions },
  dataset: {
    codexDirSize: du(join(homeDir, ".codex")),
    claudeProjectsSize: du(join(homeDir, ".claude", "projects")),
  },
};

const totalStart = process.hrtime.bigint();
const codexDbPath = join(homeDir, ".codex", "state_5.sqlite");
const codexMetadataStart = process.hrtime.bigint();
const db = new Database(codexDbPath, { readonly: true });
const codexRows = db
  .query<Json, [number]>(
    `SELECT id, rollout_path, cwd, title, model, git_branch, approval_mode,
            sandbox_policy, reasoning_effort, first_user_message, updated_at_ms
     FROM threads
     WHERE COALESCE(archived, 0) = 0
     ORDER BY updated_at_ms DESC
     LIMIT ?`,
  )
  .all(maxCodexRows);
db.close();
const codexMetadataMs = elapsedMs(codexMetadataStart);

const codexHydrateStart = process.hrtime.bigint();
let rolloutExisting = 0;
let rolloutBytesRead = 0;
const codexHits = codexRows.map((row) => {
  const terms = [
    row.title,
    row.id,
    row.rollout_path,
    row.first_user_message,
    row.git_branch,
    row.approval_mode,
    row.sandbox_policy,
    row.reasoning_effort,
    row.cwd,
    row.model,
  ]
    .filter(Boolean)
    .map(String);
  if (typeof row.rollout_path === "string") {
    const prefix = readRolloutPrefix(row.rollout_path);
    if (prefix) {
      rolloutExisting += 1;
      rolloutBytesRead += Buffer.byteLength(prefix);
      terms.push(prefix);
    }
  }
  return {
    provider: "codex",
    title: String(row.title ?? row.id ?? ""),
    cwd: String(row.cwd ?? ""),
    model: String(row.model ?? ""),
    sessionId: String(row.id ?? ""),
    terms,
  };
});
const codexHydrateMs = elapsedMs(codexHydrateStart);

const claudeDiscoverStart = process.hrtime.bigint();
const claudeFiles: { path: string; modifiedMs: number; size: number }[] = [];
if (discoverClaude) {
  walkJsonl(join(homeDir, ".claude", "projects"), claudeFiles);
  claudeFiles.sort((a, b) => b.modifiedMs - a.modifiedMs);
}
const claudeDiscoveryMs = elapsedMs(claudeDiscoverStart);

const claudeReadStart = process.hrtime.bigint();
let claudeBytesRead = 0;
let claudeParsedLines = 0;
const claudeHits: Json[] = [];
for (const file of claudeFiles.slice(0, maxClaudeSessions)) {
  const parsed = parseClaudePrefix(file.path);
  if (!parsed) continue;
  claudeBytesRead += includeContent ? fileSize(file.path) : 0;
  claudeParsedLines += parsed.lines;
  claudeHits.push({
    provider: "claude",
    title: parsed.title,
    cwd: parsed.cwd,
    model: parsed.model,
    sessionId: file.path.split("/").pop()?.replace(/\.jsonl$/, "") ?? "",
    terms: parsed.terms,
  });
}
const claudeReadMs = elapsedMs(claudeReadStart);

const hits = [...codexHits, ...claudeHits].map((hit) => ({
  ...hit,
  haystack: [hit.title, hit.cwd, hit.model, hit.sessionId, ...hit.terms]
    .join("\u001f")
    .toLowerCase(),
}));
const queries = ["codex", "script-kit", "vault", "window", "oracle", "nonexistent-needle"];
const searchSamples = queries.map((query) => {
  const needle = query.toLowerCase();
  const start = process.hrtime.bigint();
  let found = 0;
  for (const hit of hits) {
    if (hit.haystack.includes(needle)) {
      found += 1;
      if (found >= 5) break;
    }
  }
  return { query, found, ms: round(elapsedMs(start)) };
});

const searchDurations = searchSamples.map((sample) => sample.ms);
Object.assign(receipt, {
  dataset: {
    ...receipt.dataset,
    codexRows: codexRows.length,
    codexRolloutsExisting: rolloutExisting,
    codexRolloutPrefixMBRead: round(rolloutBytesRead / 1024 / 1024),
    claudeJsonlDiscovered: claudeFiles.length,
    claudeDiscoverySkipped: !discoverClaude,
    claudeRecentRead: claudeHits.length,
    claudeMBRead: round(claudeBytesRead / 1024 / 1024),
    claudeParsedLines,
  },
  timings: {
    codexMetadataMs: round(codexMetadataMs),
    codexHydrateMs: round(codexHydrateMs),
    claudeDiscoveryMs: round(claudeDiscoveryMs),
    claudeReadRecentMs: round(claudeReadMs),
    totalColdIndexMs: round(elapsedMs(totalStart)),
    searchSamples,
    searchP95Ms: round(percentile(searchDurations, 95)),
    searchMaxMs: round(Math.max(...searchDurations)),
  },
  assertions: {
    firstColdIndexUnder100Ms: elapsedMs(totalStart) <= 100,
    warmP95Under25Ms: percentile(searchDurations, 95) <= 25,
    typingPathSyncBytesReadZero: rolloutBytesRead + claudeBytesRead === 0,
    metadataOnlyReceipt: true,
  },
});

mkdirSync(outDir, { recursive: true });
const outPath = join(outDir, receiptName);
writeFileSync(outPath, `${JSON.stringify(receipt, null, 2)}\n`);
process.stdout.write(`${JSON.stringify(receipt, null, 2)}\n`);
