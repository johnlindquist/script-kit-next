#!/usr/bin/env bun
/**
 * Live positive proof that a legitimate dark UI capture is not rejected by the
 * verify-shot content audit.
 */

import { resolve } from "path";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
const session = process.argv.includes("--session")
  ? process.argv[process.argv.indexOf("--session") + 1]
  : "verify-shot-live-dark-surface";
const outDir = ".test-output/verify-shot-live-dark-surface";

const proc = Bun.spawn(
  [
    "bun",
    "scripts/agentic/surface-navigator.ts",
    "--session",
    session,
    "--group",
    "filterable-main",
    "--case",
    "process-manager-visible-rows",
    "--capture",
    "--out-dir",
    outDir,
    "--manifest",
    `${outDir}/manifest.json`,
    "--json",
  ],
  { cwd: PROJECT_ROOT, stdout: "pipe", stderr: "pipe" },
);

const stdout = await new Response(proc.stdout).text();
const stderr = await new Response(proc.stderr).text();
const exitCode = await proc.exited;
if (exitCode !== 0) {
  throw new Error(`surface-navigator failed: ${stdout || stderr || `exit ${exitCode}`}`);
}

const receipt = JSON.parse(stdout);
const entry = receipt.manifest?.entries?.[0];
const audit = entry?.contentAudit;
if (!audit || audit.blank !== false) {
  throw new Error(`expected nonblank contentAudit for live dark surface, got ${JSON.stringify(audit)}`);
}

process.stdout.write(
  `${JSON.stringify(
    {
      schemaVersion: 1,
      status: "pass",
      session,
      surface: entry.viewName,
      imagePath: entry.imagePath,
      contentAudit: audit,
    },
    null,
    2,
  )}\n`,
);
