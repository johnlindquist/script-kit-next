#!/usr/bin/env bun
import { existsSync, renameSync, unlinkSync } from "fs";
import {
  evaluateTelemetry,
  readSessionTelemetry,
  recordEvaluation,
  type EvolutionJob,
} from "./evolution.ts";

function claim(path: string): string | undefined {
  if (!existsSync(path)) return undefined;
  const claimed = `${path}.running-${process.pid}`;
  try {
    renameSync(path, claimed);
    return claimed;
  } catch {
    return undefined;
  }
}

async function main(): Promise<void> {
  const jobPath = process.argv[2];
  if (!jobPath) {
    console.error("usage: bun lib/evolution-evaluator.ts <job.json>");
    process.exit(64);
  }

  const claimed = claim(jobPath);
  if (!claimed) return;

  try {
    const job = JSON.parse(await Bun.file(claimed).text()) as EvolutionJob;
    if (job.schema !== 1 || !job.imp || !job.event_log_path) return;
    const telemetry = readSessionTelemetry(job.event_log_path);
    if (!telemetry) return;
    const result = evaluateTelemetry(telemetry, job.event_log_path);
    recordEvaluation(result);
    unlinkSync(claimed);
  } catch (error) {
    try {
      renameSync(claimed, `${claimed}.failed`);
    } catch {}
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

if (import.meta.main) {
  await main();
}
