/**
 * Shared types for the v1 → v2 migration engine.
 *
 * The unit of migration is one script. Every script flows through:
 *   classify → port (agent, skipped for clean scripts) → validator ladder → repair loop
 */

export type CompatStatus =
  | "supported" // works as-is in v2
  | "renamed" // mechanical rename, `replacement` is the new name
  | "caveat" // exists in v2 but semantics differ / partially a no-op
  | "stub" // exists in the v2 SDK but throws UnsupportedSdkFeatureError or is unimplemented in the app
  | "removed"; // gone from v2; `replacement` describes the recommended pattern

export interface CompatEntry {
  status: CompatStatus;
  /** New API name (renamed) or recommended replacement pattern (stub/removed). */
  replacement?: string;
  /** Human-readable caveat / migration note, quoted into the port prompt. */
  note?: string;
  /** Ready-to-adapt replacement code, included in the port prompt when the API is hit. */
  snippet?: string;
}

export interface CompatMap {
  apis: Record<string, CompatEntry>;
}

export interface Finding {
  /** Compat-map key, e.g. "db" or "keyboard.type". */
  api: string;
  /** 1-indexed line in the scanned source. */
  line: number;
  status: CompatStatus;
  replacement?: string;
  note?: string;
  snippet?: string;
}

export type Bucket = "ready" | "needs-changes" | "needs-rewrite";

export interface Classification {
  bucket: Bucket;
  findings: Finding[];
  /** True when the script imports "@johnlindquist/kit" (v2 preloads the SDK; the import must go). */
  hasKitImport: boolean;
}

/** Effective metadata after "typed wins, comments fill gaps" (mirrors src/scripts/metadata.rs). */
export type ScriptMetadata = Partial<
  Record<
    | "name"
    | "description"
    | "icon"
    | "alias"
    | "shortcut"
    | "cron"
    | "schedule"
    | "keyword"
    | "background",
    string
  >
>;

export type ValidatorId =
  | "typecheck"
  | "api-scan"
  | "metadata"
  | "smoke"
  | "walkthrough"
  | "honesty";

export interface ValidatorVerdict {
  id: ValidatorId;
  outcome: "pass" | "fail" | "warn" | "skipped";
  /** One-line human-readable verdict shown in receipts. */
  summary: string;
  /** Raw failure detail fed verbatim to the repair prompt. */
  detail?: string;
}

export interface MigrationNote {
  summary: string;
  behavior_changes: string[];
  confidence: "high" | "medium" | "low";
}

export interface PortAttempt {
  /** 1-indexed attempt number; attempt 1 is the initial port, later ones are repairs. */
  attempt: number;
  verdicts: ValidatorVerdict[];
  note?: MigrationNote;
  agentCostUsd?: number;
}

export type PortStatus =
  | "verified" // every validator passed
  | "verified-with-warnings" // passed, but with warn/skipped verdicts (caveats or incomplete validation)
  | "needs-review" // repair loop exhausted or honesty check flagged it
  | "error"; // engine-level failure (agent unreachable, unreadable file, ...)

export interface PortResult {
  file: string;
  bucket: Bucket;
  status: PortStatus;
  /** Where the verified port was written (absent for dry runs and failures). */
  portedPath?: string;
  attempts: PortAttempt[];
  note?: MigrationNote;
  /** Present when status is needs-review/error: everything a human (or Agent Chat) needs to continue. */
  failure?: string;
  agentUsed: boolean;
}

export interface PipelineOptions {
  outDir: string;
  dryRun?: boolean;
  /** Run the agent even for scripts classified `ready` (default: ready scripts are copied verbatim). */
  forceAgent?: boolean;
  /** Skip validators that execute the script (smoke, walkthrough). */
  noExec?: boolean;
  maxRepairs?: number;
  /** Run the refute pass when a rewrite claims zero behavior changes. */
  honesty?: boolean;
  onProgress?: (file: string, phase: string) => void;
}
