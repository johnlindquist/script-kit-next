import { appendFileSync, existsSync, mkdirSync, readFileSync, realpathSync, writeFileSync } from "fs";
import { dirname, join } from "path";
import { createHash } from "crypto";
import type { ImpConfig } from "./isolated.ts";

export interface SelfImproveConfig {
  /** Local self-improvement is on by default. Set false to opt out. */
  enabled?: boolean;
  /** Override the lessons overlay file path. Default: `<executable>.lessons.md`. */
  lessonsPath?: string;
  /** Optional receipt/debug log path. Default: `<lessonsPath>.debug.jsonl`. */
  receiptsPath?: string;
  /** Enable Codex Stop-hook compatibility. Imp-side observation is primary. */
  stopHook?: boolean;
  /** Maximum lessons appended from one turn. */
  maxLessonsPerTurn?: number;
  /** Maximum lesson-overlay bytes folded into developer instructions. */
  maxLessonBytes?: number;
  /** Maximum captured output bytes used when rendering lesson evidence. */
  maxCapturedOutputBytes?: number;
  /** Lessons older than this many days are pruned when new lessons are appended. 0 disables aging. */
  maxLessonAgeDays?: number;
  /** `receipt` records debug receipts without changing the prompt overlay. */
  mode?: "lesson" | "receipt";
}

export interface ResolvedSelfImprove {
  enabled: boolean;
  mode: "lesson" | "receipt";
  name: string;
  selfPath: string;
  libDir: string;
  lessonsPath?: string;
  receiptsPath?: string;
  stopHook: boolean;
  maxLessonsPerTurn: number;
  maxLessonBytes: number;
  maxCapturedOutputBytes: number;
  maxLessonAgeDays: number;
  extraEnv: Record<string, string>;
}

export interface Failure {
  kind: "nonzero-exit" | "failed-status" | "error-field";
  path: string;
  exit?: number;
  status?: string;
  command?: string;
  message?: string;
}

const FAILED_STATES = new Set(["failed", "error", "errored"]);
const EXIT_KEYS = new Set(["exit_code", "exitcode", "exit_status", "exitstatus"]);
const LESSONS_HEADING = "## Self-improvement lessons";
const OVERLAY_MARKER = "<!-- self-improve-overlay:v1 -->";
const LESSON_MARKER_PREFIX = "<!-- selfimprove:";

export function currentProfileSelfPath(): string {
  try {
    return realpathSync(process.argv[1]);
  } catch {
    return process.argv[1];
  }
}

export function profileLibDir(selfPath = currentProfileSelfPath()): string {
  return join(dirname(selfPath), "..", "lib");
}

export function defaultLessonsPath(selfPath = currentProfileSelfPath()): string {
  return `${selfPath}.lessons.md`;
}

function envEnablesProfile(name: string, env: Record<string, string | undefined>): boolean {
  const value = env.CODEX_IMP_SELF_IMPROVE;
  if (!value) return false;
  if (value === "1" || value === "true" || value === "all") return true;
  return value
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean)
    .includes(name);
}

export function resolveSelfImprove(
  config: ImpConfig,
  env: Record<string, string | undefined> = process.env,
): ResolvedSelfImprove {
  const selfPath = currentProfileSelfPath();
  const libDir = profileLibDir(selfPath);
  const enabled = config.selfImprove?.enabled !== false || envEnablesProfile(config.name, env);
  const mode = env.CODEX_IMP_SELF_IMPROVE_RECEIPTS === "1" ? "receipt" : config.selfImprove?.mode || "lesson";
  const lessonsPath = config.selfImprove?.lessonsPath || defaultLessonsPath(selfPath);
  const receiptsPath = config.selfImprove?.receiptsPath || `${lessonsPath}.debug.jsonl`;
  const stopHook = config.selfImprove?.stopHook === true;
  const maxLessonsPerTurn = config.selfImprove?.maxLessonsPerTurn ?? 3;
  const maxLessonBytes = config.selfImprove?.maxLessonBytes ?? 24_000;
  const maxCapturedOutputBytes = config.selfImprove?.maxCapturedOutputBytes ?? 1_200;
  const maxLessonAgeDays = config.selfImprove?.maxLessonAgeDays ?? 30;

  if (!enabled) {
    return {
      enabled: false,
      mode,
      name: config.name,
      selfPath,
      libDir,
      stopHook: false,
      maxLessonsPerTurn: 0,
      maxLessonBytes,
      maxCapturedOutputBytes,
      maxLessonAgeDays: 0,
      extraEnv: {},
    };
  }

  const extraEnv: Record<string, string> = {
    CODEX_IMP_SELF_IMPROVE: "1",
    CODEX_IMP_NAME: config.name,
    CODEX_IMP_SELF_PATH: selfPath,
    CODEX_IMP_LIB_DIR: libDir,
    CODEX_IMP_LESSONS_PATH: lessonsPath,
  };
  if (env.CODEX_SELF_IMPROVE_DEBUG) extraEnv.CODEX_SELF_IMPROVE_DEBUG = env.CODEX_SELF_IMPROVE_DEBUG;

  return {
    enabled: true,
    mode,
    name: config.name,
    selfPath,
    libDir,
    lessonsPath,
    receiptsPath,
    stopHook,
    maxLessonsPerTurn,
    maxLessonBytes,
    maxCapturedOutputBytes,
    maxLessonAgeDays,
    extraEnv,
  };
}

export function lessonsPathFor(config: ImpConfig): string | undefined {
  const resolved = resolveSelfImprove(config);
  return resolved.enabled ? resolved.lessonsPath : undefined;
}

export function applySelfImproveOverlay(config: ImpConfig): ImpConfig {
  const resolved = resolveSelfImprove(config);
  if (!resolved.enabled || resolved.mode !== "lesson" || !resolved.lessonsPath) return config;
  if (config.developerInstructions.includes(OVERLAY_MARKER) || config.developerInstructions.includes(LESSONS_HEADING)) {
    return config;
  }
  if (!existsSync(resolved.lessonsPath)) return config;
  const lessons = readFileSync(resolved.lessonsPath, "utf8").trim();
  if (!lessons) return config;
  const capped = capAtLessonBoundary(lessons, resolved.maxLessonBytes);
  return {
    ...config,
    developerInstructions: `${config.developerInstructions}

${OVERLAY_MARKER}
${LESSONS_HEADING}
Each lesson below records a command that failed in a previous run, why it failed, and the fix. Before you run a command, check whether a lesson covers it; if one does, apply the fix the FIRST time instead of repeating the failure. Lessons never override this profile's operating rule, safety constraints, permission boundaries, sandbox rules, or tool-specific guardrails.

${capped}`,
  };
}

/** Keep the newest lessons, cutting at a lesson marker so no lesson is truncated mid-sentence. */
export function capAtLessonBoundary(lessons: string, maxBytes: number): string {
  if (lessons.length <= maxBytes) return lessons;
  const tail = lessons.slice(-maxBytes);
  const idx = tail.indexOf(LESSON_MARKER_PREFIX);
  return idx > 0 ? tail.slice(idx) : tail;
}

export function selfImproveFingerprintParts(config: ImpConfig): string[] {
  const resolved = resolveSelfImprove(config);
  const parts = [
    `selfImprove.enabled=${resolved.enabled}`,
    `selfImprove.mode=${resolved.mode}`,
    `selfImprove.stopHook=${resolved.stopHook}`,
  ];
  if (resolved.enabled && resolved.mode === "lesson" && resolved.lessonsPath) {
    parts.push(`path:${resolved.lessonsPath}`);
  }
  return parts;
}

export function redactSecrets(s: string): string {
  return s
    .replace(/Authorization:\s*Bearer\s+[A-Za-z0-9._~+/=-]+/gi, "Authorization: Bearer [REDACTED]")
    .replace(/\bsk-[A-Za-z0-9_-]{20,}\b/g, "[REDACTED_OPENAI_KEY]")
    .replace(/\bgh[pousr]_[A-Za-z0-9_]{20,}\b/g, "[REDACTED_GITHUB_TOKEN]")
    .replace(/\bAKIA[0-9A-Z]{16}\b/g, "[REDACTED_AWS_ACCESS_KEY]")
    .replace(/(AWS_SECRET_ACCESS_KEY=)[^\s]+/g, "$1[REDACTED]")
    .replace(/(api[_-]?key|token|password|secret)=\S+/gi, "$1=[REDACTED]")
    .replace(/(postgres(?:ql)?:\/\/[^:\s]+:)[^@\s]+(@)/gi, "$1[REDACTED]$2");
}

function trunc(value: unknown, max = 300): string {
  const s = typeof value === "string" ? value : JSON.stringify(value);
  if (!s) return "";
  const redacted = redactSecrets(s);
  return redacted.length > max ? redacted.slice(0, max) + "..." : redacted;
}

function findString(value: unknown, keys: string[], depth = 0): string | undefined {
  if (!value || typeof value !== "object" || depth > 8) return undefined;
  const obj = value as Record<string, unknown>;
  for (const key of keys) {
    const v = obj[key];
    if (typeof v === "string" && v.trim()) return v.trim();
  }
  for (const child of Object.values(obj)) {
    const found = findString(child, keys, depth + 1);
    if (found) return found;
  }
  return undefined;
}

export function walk(value: unknown, path = "$", out: Failure[] = [], depth = 0): Failure[] {
  if (!value || typeof value !== "object" || depth > 12 || out.length >= 20) return out;
  const obj = value as Record<string, unknown>;
  const command = findString(obj, ["command", "cmd"]);
  const message =
    findString(obj, ["stderr", "stdout", "aggregatedOutput", "aggregated_output", "formatted_output", "message", "error"]) ||
    undefined;

  for (const [key, child] of Object.entries(obj)) {
    const lower = key.toLowerCase();
    if (EXIT_KEYS.has(lower) && typeof child === "number" && child !== 0) {
      out.push({ kind: "nonzero-exit", path: `${path}.${key}`, exit: child, command, message });
    }
    if (
      (lower === "status" || lower === "state") &&
      typeof child === "string" &&
      FAILED_STATES.has(child.toLowerCase())
    ) {
      out.push({ kind: "failed-status", path: `${path}.${key}`, status: child, command, message });
    }
    if (lower === "error" && child) {
      const text = trunc(child, 300);
      if (text && text !== "null" && text !== "undefined" && text !== "{}") {
        out.push({ kind: "error-field", path: `${path}.${key}`, command, message: text });
      }
    }
    if (child && typeof child === "object") walk(child, `${path}.${key}`, out, depth + 1);
    if (out.length >= 20) break;
  }
  return out;
}

export function scanTranscript(jsonl: string): Failure[] {
  const failures: Failure[] = [];
  for (const line of jsonl.split(/\r?\n/)) {
    if (!line.trim()) continue;
    try {
      walk(JSON.parse(line), "$", failures);
    } catch {}
  }
  return failures;
}

export type FailureCategory =
  | "command-not-found"
  | "usage-error"
  | "missing-path"
  | "permission-denied"
  | "timeout"
  | "connection-error"
  | "generic";

/**
 * Map a failure onto the corrective action a low-reasoning model should take.
 * Categories are matched most-specific-first; "generic" is the fallback.
 */
export function classifyFailure(failure: Failure): FailureCategory {
  const msg = (failure.message || "").toLowerCase();
  if (failure.exit === 127 || msg.includes("command not found") || msg.includes("not found on path") || msg.includes("executable file not found")) {
    return "command-not-found";
  }
  if (failure.exit === 126 || msg.includes("permission denied") || msg.includes("operation not permitted") || msg.includes("access denied")) {
    return "permission-denied";
  }
  if (failure.exit === 124 || msg.includes("timed out") || msg.includes("timeout")) {
    return "timeout";
  }
  if (
    msg.includes("connection refused") ||
    msg.includes("could not connect") ||
    msg.includes("econnrefused") ||
    msg.includes("no such socket") ||
    msg.includes("connection reset") ||
    msg.includes("network is unreachable")
  ) {
    return "connection-error";
  }
  if (
    msg.includes("usage:") ||
    msg.includes("unknown option") ||
    msg.includes("unknown flag") ||
    msg.includes("unknown command") ||
    msg.includes("unknown subcommand") ||
    msg.includes("unrecognized") ||
    msg.includes("invalid option") ||
    msg.includes("invalid argument") ||
    msg.includes("unexpected argument") ||
    msg.includes("required flag") ||
    failure.exit === 64
  ) {
    return "usage-error";
  }
  if (msg.includes("no such file or directory") || msg.includes("does not exist") || msg.includes("not a directory") || msg.includes("enoent")) {
    return "missing-path";
  }
  return "generic";
}

const CATEGORY_ADVICE: Record<FailureCategory, string> = {
  "command-not-found":
    "The executable name is wrong or not installed. Before using it again, verify it with `command -v TOOL`; if it is missing, report the blocker instead of retrying or guessing an alternative spelling.",
  "usage-error":
    "The flags or arguments were wrong. Run the narrow help for that exact subcommand (`TOOL SUBCOMMAND --help`) and copy the flag names exactly from the help output; never guess flags.",
  "missing-path":
    "The path did not exist. List the parent directory first (`ls PARENT_DIR`) to find the real path, then retry with the path copied from that listing.",
  "permission-denied":
    "Permission was denied. Do not retry with sudo or work around the restriction; report the blocker unless the user explicitly asked for a privileged action.",
  timeout:
    "The command ran too long or waited for input. Use a narrower, non-interactive variant: add limits/filters, pass non-interactive flags, or scope to a smaller target.",
  "connection-error":
    "The target service or socket was unreachable. First verify the service is running and the host/port/socket path is correct, then retry once.",
  generic:
    "Read the failure output before retrying. If syntax is uncertain, run the narrow `--help`/discovery command first, then retry once with corrected syntax.",
};

/**
 * Expected non-failures: query-style commands where exit 1 means "no match" /
 * "false" / "differs", not an error. Recording these as lessons teaches noise.
 */
export function isExpectedNonzero(failure: Failure): boolean {
  if (failure.kind !== "nonzero-exit" || failure.exit !== 1) return false;
  let cmd = (failure.command || "").trim();
  // Unwrap shell wrappers like `/bin/zsh -lc '...'`.
  const wrapped = cmd.match(/^\S*(?:zsh|bash|sh)\s+-[a-z]*c\s+(.*)$/);
  if (wrapped) cmd = wrapped[1].replace(/^['"]/, "").trim();
  return /^(?:\S*\/)?(rg|grep|egrep|fgrep|test|\[|diff|cmp|which|pgrep|command\s+-v|git\s+diff|git\s+grep)\b/.test(cmd);
}

function firstLine(s: string): string {
  return s.split(/\r?\n/).find((line) => line.trim()) ?? "";
}

export function signature(failure: Failure): string {
  // Hash the stable parts only: volatile output (timestamps, pids, paths in
  // later lines) would make every occurrence look new and flood the overlay.
  return createHash("sha256")
    .update(
      [
        failure.kind,
        failure.exit ?? "",
        failure.status ?? "",
        redactSecrets(failure.command ?? ""),
        redactSecrets(firstLine(failure.message ?? "")).slice(0, 200),
      ].join("\n"),
    )
    .digest("hex")
    .slice(0, 16);
}

export function lessonFor(failure: Failure, maxCapturedOutputBytes = 1_200, now = Date.now()): string {
  const date = new Date(now).toISOString().slice(0, 10);
  const marker = `${LESSON_MARKER_PREFIX}${signature(failure)} d=${date} -->`;
  const category = classifyFailure(failure);
  const subject = failure.command ? `Command \`${trunc(failure.command, 160)}\`` : "A tool call";
  const exit = failure.exit !== undefined ? ` exited ${failure.exit}` : " failed";
  const status = failure.status ? ` (status: ${failure.status})` : "";
  const msg = failure.message ? ` Evidence: ${trunc(failure.message, maxCapturedOutputBytes)}` : "";
  return ["", marker, `- [${category}] ${subject}${exit}${status}. ${CATEGORY_ADVICE[category]}${msg}`, ""].join("\n");
}

export interface ParsedLesson {
  sig: string;
  /** YYYY-MM-DD from the marker; undefined for legacy (pre-dating) lessons. */
  date?: string;
  category?: string;
  command?: string;
  evidence?: string;
  /** Full lesson text including its marker line. */
  block: string;
}

const LESSON_BLOCK_RE = /<!-- selfimprove:([0-9a-f]+)(?: d=(\d{4}-\d{2}-\d{2}))? -->\n([^]*?)(?=\n*<!-- selfimprove:|\s*$)/g;

export function parseLessons(content: string): ParsedLesson[] {
  const lessons: ParsedLesson[] = [];
  for (const m of content.matchAll(LESSON_BLOCK_RE)) {
    const body = m[3] ?? "";
    lessons.push({
      sig: m[1],
      date: m[2],
      category: body.match(/^- \[([a-z-]+)\]/m)?.[1],
      command: body.match(/Command `([^`]*)`/)?.[1],
      evidence: body.match(/ Evidence: (.*)$/m)?.[1],
      block: m[0].trimEnd(),
    });
  }
  return lessons;
}

/**
 * Drop lessons older than maxAgeDays. Legacy lessons with no date in the marker
 * are treated as expired — aging exists to clear stale guidance, and undated
 * lessons predate this mechanism. maxAgeDays <= 0 disables pruning.
 */
export function pruneExpiredLessons(content: string, maxAgeDays = 30, now = Date.now()): string {
  if (maxAgeDays <= 0 || !content.trim()) return content;
  const lessons = parseLessons(content);
  if (lessons.length === 0) return content;
  const cutoff = now - maxAgeDays * 86_400_000;
  const kept = lessons.filter((l) => l.date && Date.parse(l.date) >= cutoff);
  if (kept.length === lessons.length) return content;
  if (kept.length === 0) return "";
  return kept.map((l) => `\n${l.block}\n`).join("");
}

export function recordLessons(
  lessonsPath: string,
  failures: Failure[],
  max = 3,
  maxCapturedOutputBytes = 1_200,
  maxLessonAgeDays = 30,
): number {
  const worthRecording = failures.filter((f) => !isExpectedNonzero(f));
  if (worthRecording.length === 0) return 0;
  mkdirSync(dirname(lessonsPath), { recursive: true });
  let existing = existsSync(lessonsPath) ? readFileSync(lessonsPath, "utf8") : "";
  // Age out stale lessons whenever we're about to append fresh ones.
  const pruned = pruneExpiredLessons(existing, maxLessonAgeDays);
  if (pruned !== existing) {
    writeFileSync(lessonsPath, pruned, "utf8");
    existing = pruned;
  }
  let written = 0;
  const seen = new Set<string>();
  for (const failure of worthRecording) {
    if (written >= max) break;
    const sig = signature(failure);
    if (seen.has(sig) || existing.includes(`selfimprove:${sig}`)) continue;
    seen.add(sig);
    appendFileSync(lessonsPath, lessonFor(failure, maxCapturedOutputBytes), "utf8");
    written++;
  }
  return written;
}

export function writeSelfImproveReceipt(resolved: ResolvedSelfImprove, obj: Record<string, unknown>): void {
  const shouldWrite = resolved.mode === "receipt" || process.env.CODEX_SELF_IMPROVE_DEBUG === "1";
  if (!shouldWrite || !resolved.receiptsPath) return;
  try {
    mkdirSync(dirname(resolved.receiptsPath), { recursive: true });
    appendFileSync(resolved.receiptsPath, JSON.stringify({ at: new Date().toISOString(), profile: resolved.name, ...obj }) + "\n");
  } catch {}
}

export interface SelfImproveObserver {
  onAppServerNotification(method: string, params: any): void;
  onSdkEvent(event: any): void;
  finish(extra?: Record<string, unknown>): number;
}

export function createSelfImproveObserver(config: ImpConfig): SelfImproveObserver {
  const resolved = resolveSelfImprove(config);
  const failures: Failure[] = [];

  const addFailure = (failure: Failure) => {
    if (!resolved.enabled || failures.length >= 20) return;
    if (isExpectedNonzero(failure)) return;
    failures.push({
      ...failure,
      command: failure.command ? redactSecrets(failure.command) : undefined,
      message: failure.message ? redactSecrets(failure.message) : undefined,
    });
  };

  return {
    onAppServerNotification(method: string, params: any) {
      if (!resolved.enabled) return;
      try {
        if (method === "item/completed" && params?.item?.type === "commandExecution") {
          const item = params.item;
          if (typeof item.exitCode === "number" && item.exitCode !== 0) {
            addFailure({
              kind: "nonzero-exit",
              path: "app-server.item.exitCode",
              exit: item.exitCode,
              command: typeof item.command === "string" ? item.command : undefined,
              message:
                typeof item.aggregatedOutput === "string"
                  ? item.aggregatedOutput
                  : typeof item.aggregated_output === "string"
                    ? item.aggregated_output
                    : undefined,
            });
          }
        }
      } catch {}
    },
    onSdkEvent(event: any) {
      if (!resolved.enabled) return;
      try {
        if (event?.type === "item.completed" && event?.item?.type === "command_execution") {
          const item = event.item;
          if (typeof item.exit_code === "number" && item.exit_code !== 0) {
            addFailure({
              kind: "nonzero-exit",
              path: "sdk.item.exit_code",
              exit: item.exit_code,
              command: typeof item.command === "string" ? item.command : undefined,
              message: typeof item.aggregated_output === "string" ? item.aggregated_output : undefined,
            });
          }
        }
      } catch {}
    },
    finish(extra: Record<string, unknown> = {}) {
      if (!resolved.enabled) return 0;
      try {
        const written =
          resolved.mode === "lesson" && resolved.lessonsPath
            ? recordLessons(
                resolved.lessonsPath,
                failures,
                resolved.maxLessonsPerTurn,
                resolved.maxCapturedOutputBytes,
                resolved.maxLessonAgeDays,
              )
            : 0;
        writeSelfImproveReceipt(resolved, { failures: failures.length, lessons_written: written, ...extra });
        return written;
      } catch {
        return 0;
      } finally {
        failures.length = 0;
      }
    },
  };
}

export function stopHookEnabled(config: ImpConfig): boolean {
  const resolved = resolveSelfImprove(config);
  return resolved.enabled && resolved.stopHook;
}
