#!/usr/bin/env bun
/**
 * Codex Stop hook compatibility wrapper for the shared self-improvement module.
 *
 * Imp-side event observation is the primary path today. This wrapper remains
 * available for Codex builds that execute Stop hooks for isolated app-server or
 * exec turns. It always fails open.
 */
import { existsSync, readFileSync } from "fs";
import {
  recordLessons,
  scanTranscript,
  writeSelfImproveReceipt,
  type ResolvedSelfImprove,
} from "./self-improve.ts";

function ok(): void {
  process.stdout.write(JSON.stringify({ continue: true, suppressOutput: true }) + "\n");
}

async function main(): Promise<void> {
  try {
    const raw = await Bun.stdin.text();
    const input = raw.trim() ? JSON.parse(raw) : {};

    if (input.hook_event_name !== "Stop") return ok();
    if (input.stop_hook_active || process.env.CODEX_SELF_IMPROVE_SKIP === "1") return ok();

    const lessonsPath = process.env.CODEX_IMP_LESSONS_PATH;
    const transcriptPath = input.transcript_path;
    if (!lessonsPath || typeof lessonsPath !== "string") return ok();

    const resolved: ResolvedSelfImprove = {
      enabled: true,
      mode: process.env.CODEX_IMP_SELF_IMPROVE_RECEIPTS === "1" ? "receipt" : "lesson",
      name: process.env.CODEX_IMP_NAME || "unknown-profile",
      selfPath: process.env.CODEX_IMP_SELF_PATH || "",
      libDir: process.env.CODEX_IMP_LIB_DIR || "",
      lessonsPath,
      receiptsPath: `${lessonsPath}.debug.jsonl`,
      stopHook: true,
      maxLessonsPerTurn: 3,
      maxLessonBytes: 24_000,
      maxCapturedOutputBytes: 1_200,
      maxLessonAgeDays: 30,
      extraEnv: {},
    };

    if (!transcriptPath || typeof transcriptPath !== "string" || !existsSync(transcriptPath)) {
      writeSelfImproveReceipt(resolved, {
        event: input.hook_event_name,
        transcript_path: transcriptPath ?? null,
        transcript_exists: false,
      });
      return ok();
    }

    const failures = scanTranscript(readFileSync(transcriptPath, "utf8"));
    const written = resolved.mode === "lesson" ? recordLessons(lessonsPath, failures, resolved.maxLessonsPerTurn, resolved.maxCapturedOutputBytes, resolved.maxLessonAgeDays) : 0;
    writeSelfImproveReceipt(resolved, {
      event: input.hook_event_name,
      transcript_path: transcriptPath,
      transcript_exists: true,
      failures: failures.length,
      lessons_written: written,
    });
    return ok();
  } catch {
    return ok();
  }
}

if (import.meta.main) {
  await main();
}
