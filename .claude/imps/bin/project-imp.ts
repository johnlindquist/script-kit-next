#!/usr/bin/env bun
import { spawn } from "child_process";
import { createHash } from "crypto";
import { appendFileSync, mkdirSync } from "fs";
import { basename, join } from "path";
import { allImps, impsRoot, repoRoot, routePrompt } from "../lib/project-config.ts";

const args = process.argv.slice(2);
const which = args.includes("--which");
const list = args.includes("--list") || args.includes("-l");

function takeOption(name: string): string | undefined {
  const index = args.indexOf(name);
  if (index === -1) return undefined;
  const value = args[index + 1];
  args.splice(index, value === undefined ? 1 : 2);
  return value;
}

function timeoutFrom(value: string | undefined, fallback: number): number {
  if (value === undefined || value.trim() === "") return fallback;
  const parsed = Number(value);
  return Number.isFinite(parsed) && parsed >= 0 ? parsed : fallback;
}

const cliProgressTimeout =
  takeOption("--progress-timeout-ms") ??
  takeOption("--idle-timeout-ms") ??
  takeOption("--timeout-ms") ??
  takeOption("--advisory-timeout-ms");
const progressTimeoutMs = timeoutFrom(
  cliProgressTimeout ??
    process.env.SCRIPT_KIT_IMP_PROGRESS_TIMEOUT_MS ??
    process.env.SCRIPT_KIT_IMP_ADVISORY_TIMEOUT_MS,
  600_000,
);
const maxRuntimeMs = timeoutFrom(
  takeOption("--max-runtime-ms") ?? process.env.SCRIPT_KIT_IMP_MAX_RUNTIME_MS,
  0,
);
const prompt = args.filter((arg) => arg !== "--which" && arg !== "--list" && arg !== "-l").join(" ");

if (list) {
  for (const imp of allImps()) {
    console.log(`${imp.name}\t${imp.phase}\t${imp.permission}\t${imp.summary}`);
  }
  process.exit(0);
}

if (!prompt) {
  console.error("Usage: project-imp [--which] <task prompt>");
  process.exit(1);
}

const routed = routePrompt(prompt);

if (which) {
  console.log(routed.map((imp) => imp.name).join("\n"));
  process.exit(0);
}

const [primary, ...secondary] = routed;
if (secondary.length) {
  console.error(`project-imp: primary ${primary.name}; also consider ${secondary.map((imp) => imp.name).join(", ")}`);
}

const command = join(impsRoot, "imps", primary.name);

function promptWithBudget(text: string): string {
  if (progressTimeoutMs <= 0) return text;
  const seconds = Math.max(1, Math.floor(progressTimeoutMs / 1000));
  return `${text}

Project imp progress budget: emit useful progress at least every ${seconds}s. Serious work may continue as long as progress is visible. Keep interim output concrete: files inspected, findings, next checks, or partial recommendations.`;
}

function receipt(
  status: "completed" | "idle-timeout" | "max-runtime-timeout" | "error",
  detail: Record<string, unknown> = {},
) {
  const receiptsDir = join(impsRoot, "receipts");
  mkdirSync(receiptsDir, { recursive: true });
  const entry = {
    at: new Date().toISOString(),
    imp: primary.name,
    status,
    progressTimeoutMs,
    maxRuntimeMs,
    promptSha256: createHash("sha256").update(prompt).digest("hex"),
    ...detail,
  };
  appendFileSync(join(receiptsDir, `${primary.name}.jsonl`), JSON.stringify(entry) + "\n", "utf8");
}

const startedAt = Date.now();
// The updated runtime defaults to an interactive Codex TUI; the router needs
// the warm non-interactive streaming path, so always pass --run.
const child = spawn(command, ["--run", promptWithBudget(prompt)], {
  cwd: repoRoot,
  stdio: ["inherit", "pipe", "pipe"],
  env: process.env,
  detached: process.platform !== "win32",
});

let timeoutStatus: "idle-timeout" | "max-runtime-timeout" | null = null;
let lastProgressAt = Date.now();
let progressEvents = 0;
let idleTimer: ReturnType<typeof setTimeout> | undefined;

function terminateFor(status: "idle-timeout" | "max-runtime-timeout") {
  if (timeoutStatus) return;
  timeoutStatus = status;
  try {
    if (process.platform === "win32") child.kill("SIGTERM");
    else process.kill(-child.pid!, "SIGTERM");
  } catch {
    try {
      child.kill("SIGTERM");
    } catch {}
  }
  setTimeout(() => {
    try {
      if (process.platform === "win32") child.kill("SIGKILL");
      else process.kill(-child.pid!, "SIGKILL");
    } catch {}
  }, 2_000).unref?.();
}

function armIdleTimer() {
  if (progressTimeoutMs <= 0) return;
  if (idleTimer) clearTimeout(idleTimer);
  idleTimer = setTimeout(() => terminateFor("idle-timeout"), progressTimeoutMs);
  idleTimer.unref?.();
}

function progress(chunk: Buffer, stream: NodeJS.WriteStream) {
  progressEvents++;
  lastProgressAt = Date.now();
  stream.write(chunk);
  armIdleTimer();
}

child.stdout?.on("data", (chunk) => progress(chunk, process.stdout));
child.stderr?.on("data", (chunk) => progress(chunk, process.stderr));
armIdleTimer();

const maxRuntimeTimer =
  maxRuntimeMs > 0
    ? setTimeout(() => {
        terminateFor("max-runtime-timeout");
        try {
          process.stderr.write(
            `${basename(command)}: max runtime ${maxRuntimeMs}ms reached\n`,
          );
        } catch {}
      }, maxRuntimeMs)
    : undefined;
maxRuntimeTimer?.unref?.();

child.on("error", (error) => {
  idleTimer && clearTimeout(idleTimer);
  maxRuntimeTimer && clearTimeout(maxRuntimeTimer);
  receipt("error", { elapsedMs: Date.now() - startedAt, error: error.message });
  console.error(`${basename(command)}: ${error.message}`);
  process.exit(1);
});

child.on("exit", (code, signal) => {
  idleTimer && clearTimeout(idleTimer);
  maxRuntimeTimer && clearTimeout(maxRuntimeTimer);
  const elapsedMs = Date.now() - startedAt;
  if (timeoutStatus) {
    receipt(timeoutStatus, {
      elapsedMs,
      signal,
      progressEvents,
      idleMs: Date.now() - lastProgressAt,
    });
    console.error(
      `${basename(command)}: ${timeoutStatus} after ${elapsedMs}ms`,
    );
    process.exit(124);
  }
  receipt("completed", { elapsedMs, code, signal, progressEvents });
  process.exit(code ?? (signal ? 1 : 0));
});
