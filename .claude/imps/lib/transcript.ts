/**
 * Per-invocation audit transcript + run stats.
 *
 * Every non-interactive imp run appends one JSON line to
 *   $IMP_HOME/<imp>/transcripts/YYYY-MM.jsonl   (default IMP_HOME = ~/.imp)
 * capturing what the imp ran: prompt, model, transport, duration, status,
 * every command it executed (with exit codes), token usage when the backend
 * surfaces it, and how much text it answered with. It's a greppable,
 * SIEM-shippable audit trail. Best-effort: a write failure never breaks a run.
 *
 * A single dim stderr stats line (`⚡ imp-git · warm · 3.2s · 5.8k tokens`) is
 * printed after each run so the speed is felt on every invocation. stdout stays
 * pipe-clean — the stats line only ever touches stderr.
 *
 * Token usage plumbing (verified against @openai/codex-sdk 0.x + codex 0.142):
 *   - Cold SDK stream: `turn.completed` event carries `usage`
 *     ({ input_tokens, cached_input_tokens, output_tokens, reasoning_output_tokens }).
 *   - Cold SDK quiet: `Turn.usage` (same shape, may be null).
 *   - Warm app-server: `thread/tokenUsage/updated` notification carries
 *     `params.tokenUsage.total.totalTokens` (a TokenUsageBreakdown; `.last` is the
 *     latest turn). Imps run one turn per fresh thread, so total == the turn.
 */

import { appendFileSync, mkdirSync } from "fs";
import { dirname, join } from "path";
import { impHome } from "./evolution.ts";

const PROMPT_MAX_CHARS = 2000;

export interface TranscriptCommand {
  command: string;
  exitCode: number | null;
}

export interface TranscriptEntry {
  ts: string;
  imp: string;
  cwd: string;
  prompt: string;
  transport: string;
  model?: string;
  durationMs: number;
  status: string;
  commands: TranscriptCommand[];
  tokens?: number;
  answerChars: number;
}

/** `$IMP_HOME/<imp>/transcripts/YYYY-MM.jsonl`, month derived from the entry's ISO ts. */
export function transcriptFilePath(imp: string, ts: string): string {
  const month = ts.slice(0, 7); // "2026-07" from "2026-07-01T..."
  return join(impHome(), imp, "transcripts", `${month}.jsonl`);
}

/**
 * Append one audit line. Best-effort: honors IMP_NO_TRANSCRIPT=1, truncates the
 * prompt to 2000 chars, and swallows every error (unwritable dir, bad path, …)
 * so an audit-write failure can never break the run it is auditing.
 */
export function writeTranscriptEntry(entry: TranscriptEntry): void {
  if (process.env.IMP_NO_TRANSCRIPT === "1") return;
  try {
    const record: TranscriptEntry = {
      ...entry,
      prompt: entry.prompt.length > PROMPT_MAX_CHARS ? entry.prompt.slice(0, PROMPT_MAX_CHARS) : entry.prompt,
    };
    const file = transcriptFilePath(entry.imp, entry.ts);
    mkdirSync(dirname(file), { recursive: true });
    appendFileSync(file, JSON.stringify(record) + "\n", "utf8");
  } catch {}
}

/** `1234` -> `1.2k tokens`; `12` -> `12 tokens`. */
export function formatTokens(n: number): string {
  if (n >= 1000) return `${(n / 1000).toFixed(1)}k tokens`;
  return `${n} token${n === 1 ? "" : "s"}`;
}

/** `3200` -> `3.2s`; `83000` -> `1m23s`. */
export function formatDuration(ms: number): string {
  if (ms < 60_000) return `${(ms / 1000).toFixed(1)}s`;
  const totalSec = Math.round(ms / 1000);
  return `${Math.floor(totalSec / 60)}m${totalSec % 60}s`;
}

/**
 * Pure stats-line text (no ANSI): `⚡ imp-git · warm · 3.2s · 5.8k tokens`.
 * `transport` is the audit transport (warm/sdk-stream/sdk-quiet); it collapses
 * to `warm`/`cold` for the human-facing line. Tokens are appended only when known.
 */
export function formatStatsLine(input: { imp: string; transport: string; durationMs: number; tokens?: number }): string {
  const mode = input.transport === "warm" ? "warm" : "cold";
  const parts = [`⚡ ${input.imp}`, mode, formatDuration(input.durationMs)];
  if (typeof input.tokens === "number" && input.tokens > 0) parts.push(formatTokens(input.tokens));
  return parts.join(" · ");
}

/** Emit the stats line to stderr (dim), unless IMP_NO_STATS=1. stdout stays clean. */
export function emitStatsLine(input: { imp: string; transport: string; durationMs: number; tokens?: number }): void {
  if (process.env.IMP_NO_STATS === "1") return;
  try {
    process.stderr.write(`\x1b[2m${formatStatsLine(input)}\x1b[0m\n`);
  } catch {}
}

export interface TranscriptRecorderOptions {
  cwd: string;
  prompt: string;
  model?: string;
}

export interface TranscriptFinishExtra {
  status: string;
  transport: string;
  /** Overrides the collected token count when the backend reports it out-of-band. */
  tokens?: number;
  /** Overrides collected answer text (e.g. the buffered final from a quiet run). */
  finalText?: string;
}

export interface TranscriptRecorder {
  onAppServerNotification(method: string, params: any): void;
  onSdkEvent(event: any): void;
  finish(extra: TranscriptFinishExtra): TranscriptEntry;
}

/**
 * Collects audit data from a live run's event stream and writes the transcript
 * line on finish, mirroring the evolution observer's dual-funnel shape:
 * `onAppServerNotification` for the warm path, `onSdkEvent` for the cold path.
 * Starts the wall clock at construction.
 */
export function createTranscriptRecorder(imp: string, opts: TranscriptRecorderOptions): TranscriptRecorder {
  const startedAt = Date.now();
  const commands: TranscriptCommand[] = [];
  const commandById = new Map<string, TranscriptCommand>();
  let tokens: number | undefined;
  let answerText = "";
  let finishedEntry: TranscriptEntry | undefined;

  const upsertCommand = (id: string | undefined, command: string | undefined, exitCode: number | null) => {
    const existing = id ? commandById.get(id) : undefined;
    if (existing) {
      if (command) existing.command = command;
      if (exitCode !== null) existing.exitCode = exitCode;
      return;
    }
    const record: TranscriptCommand = { command: command ?? "", exitCode };
    commands.push(record);
    if (id) commandById.set(id, record);
  };

  return {
    onAppServerNotification(method, params) {
      try {
        if (method === "item/started" && params?.item?.type === "commandExecution") {
          upsertCommand(params.item.id, params.item.command, null);
        } else if (method === "item/completed" && params?.item?.type === "commandExecution") {
          upsertCommand(params.item.id, params.item.command, typeof params.item.exitCode === "number" ? params.item.exitCode : null);
        } else if (method === "thread/tokenUsage/updated") {
          const total = params?.tokenUsage?.total?.totalTokens;
          if (typeof total === "number") tokens = total;
        } else if (method === "item/agentMessage/delta") {
          answerText += params?.delta ?? "";
        } else if (method === "item/completed" && params?.item?.type === "agentMessage" && params.item.text) {
          answerText = params.item.text;
        }
      } catch {}
    },
    onSdkEvent(event) {
      try {
        if (event?.type === "item.completed" && event?.item?.type === "command_execution") {
          upsertCommand(event.item.id, event.item.command, typeof event.item.exit_code === "number" ? event.item.exit_code : null);
        } else if (event?.type === "turn.completed" && event.usage) {
          const input = event.usage.input_tokens ?? 0;
          const output = event.usage.output_tokens ?? 0;
          tokens = input + output;
        } else if (event?.type === "item.completed" && event?.item?.type === "agent_message" && event.item.text) {
          answerText = event.item.text;
        }
      } catch {}
    },
    finish(extra) {
      // Idempotent: warm→cold fallback and signal-handler races may both fire.
      if (finishedEntry) return finishedEntry;
      const finalTokens = extra.tokens ?? tokens;
      const answer = extra.finalText != null ? extra.finalText : answerText;
      const entry: TranscriptEntry = {
        ts: new Date().toISOString(),
        imp,
        cwd: opts.cwd,
        prompt: opts.prompt,
        transport: extra.transport,
        model: opts.model,
        durationMs: Date.now() - startedAt,
        status: extra.status,
        commands,
        ...(typeof finalTokens === "number" ? { tokens: finalTokens } : {}),
        answerChars: answer.length,
      };
      writeTranscriptEntry(entry);
      finishedEntry = entry;
      return entry;
    },
  };
}
