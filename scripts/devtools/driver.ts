#!/usr/bin/env bun
/**
 * scripts/devtools/driver.ts — persistent, event-driven protocol driver.
 *
 * Owns the app process directly: commands are written to the app's stdin
 * pipe and responses are matched event-driven from the app's stdout pipe
 * (the app writes one flushed JSON line per protocol response — see
 * src/stdin_commands/mod.rs create_stdout_response_sender). This replaces
 * the per-command path of: bun process spawn → session.sh → FIFO forwarder
 * → 50ms polling of protocol-responses.ndjson, which costs ~0.5-2s per
 * command. A driver round trip is one pipe write + one pipe read.
 *
 * Usage (library):
 *   import { Driver } from "./driver";
 *   const d = await Driver.launch({ sandboxHome: true });
 *   await d.setFilterAndWait("notes");
 *   const state = await d.getState();
 *   await d.close();
 *
 * Usage (smoke check):
 *   bun scripts/devtools/driver.ts smoke
 */

import { mkdirSync, existsSync, rmSync, statSync, symlinkSync } from "node:fs";
import { homedir } from "node:os";
import { join, resolve } from "node:path";
import type { Subprocess } from "bun";

const PROJECT_ROOT = resolve(import.meta.dir, "../..");
/**
 * Both build paths produce a runnable binary: ./dev.sh owns target/debug and
 * agent-cargo.sh owns the shared agent pool. Historically the driver silently
 * defaulted to target/debug, so an agent that had just built via agent-cargo
 * verified a stale dev.sh binary. With no explicit override we now pick the
 * freshest candidate by mtime and say so on stderr.
 */
const BINARY_CANDIDATES = [
  join(PROJECT_ROOT, "target/debug/script-kit-gpui"),
  join(PROJECT_ROOT, "target-agent/pools/agent-debug/debug/script-kit-gpui"),
];

function resolveDefaultBinary(): string {
  const explicit = process.env.SCRIPT_KIT_GPUI_BINARY;
  if (explicit) return explicit;

  const found = BINARY_CANDIDATES.flatMap((path) => {
    try {
      return [{ path, mtimeMs: statSync(path).mtimeMs }];
    } catch {
      return [];
    }
  }).sort((a, b) => b.mtimeMs - a.mtimeMs);

  if (found.length === 0) return BINARY_CANDIDATES[0];
  const chosen = found[0];
  if (found.length > 1) {
    const stale = found[1];
    const ageGapSec = Math.round((chosen.mtimeMs - stale.mtimeMs) / 1000);
    console.error(
      `[driver] binary: ${chosen.path} (freshest of ${found.length} candidates; ` +
        `${stale.path} is ${ageGapSec}s older). Pass binary:/SCRIPT_KIT_GPUI_BINARY to override.`,
    );
  } else {
    console.error(`[driver] binary: ${chosen.path} (only candidate present)`);
  }
  return chosen.path;
}
const READY_MARKER_STARTUP = "STARTUP_READY ";
const READY_MARKER_APP =
  "APP_READY|main-window-ready show=false focus=false stdin-safe";
const DEFAULT_RUST_LOG =
  process.env.SCRIPT_KIT_AGENTIC_RUST_LOG ??
  "info,gpui::window=off,gpui=warn,hyper=warn,reqwest=warn";

export type Json = Record<string, any>;

let launchCounter = 0;

export interface DriverOptions {
  /**
   * Path to the app binary. Defaults to SCRIPT_KIT_GPUI_BINARY, else the
   * freshest (by mtime) of target/debug and the agent-cargo pool binary.
   */
  binary?: string;
  /**
   * Session label reported to the app (logs/protocol bus). Treated as a
   * label, not an address: the derived artifact directory is always
   * uniquified per launch so parallel loops reusing the same name never
   * clobber each other. Pass `sessionDir` to take full control.
   */
  sessionName?: string;
  /** Directory for driver artifacts (app.log, protocol bus). */
  sessionDir?: string;
  /**
   * When true, point HOME/SK_PATH at a fresh sandbox under sessionDir so the
   * driven app never touches real user data and starts from a known state.
   */
  sandboxHome?: boolean;
  /**
   * With sandboxHome, symlink the real ~/.scriptkit/models into the sandbox
   * so the app reuses the multi-GB dictation/brain model downloads instead
   * of re-downloading into every session dir. Pass false only when a probe
   * specifically tests model-download behavior. Default true.
   */
  sharedModels?: boolean;
  /**
   * With sandboxHome, seed the sandbox with the Pi/Codex auth state live
   * Agent Chat probes need (runs scripts/agentic/seed-sandbox-home.sh:
   * APFS-clones ~/.pi plus ~/.codex/{auth.json,config.toml}). Default false —
   * leave it off unless the probe drives a live agent.
   */
  seedAgentAuth?: boolean;
  /** Extra env vars for the app process (test providers, feature flags). */
  env?: Record<string, string>;
  /** Max ms to wait for the readiness log marker. Default 10000. */
  readyTimeoutMs?: number;
  /** Default per-request timeout. Default 5000. */
  defaultTimeoutMs?: number;
  /**
   * Also mirror responses to protocol-responses.ndjson like session.sh
   * sessions do (useful for debugging with existing tooling). Default true;
   * the driver itself never reads this file.
   */
  protocolBusFile?: boolean;
}

export interface DriverStats {
  requestsSent: number;
  responsesMatched: number;
  unmatchedResponses: number;
  readyWaitMs: number;
}

interface Pending {
  resolve: (value: Json) => void;
  reject: (error: Error) => void;
  expect?: string;
  timer: ReturnType<typeof setTimeout>;
}

export class Driver {
  readonly sessionName: string;
  readonly sessionDir: string;
  readonly logPath: string;
  readonly stats: DriverStats = {
    requestsSent: 0,
    responsesMatched: 0,
    unmatchedResponses: 0,
    readyWaitMs: 0,
  };

  private proc: Subprocess<"pipe", "pipe", "pipe">;
  private pending = new Map<string, Pending>();
  private requestCounter = 0;
  private defaultTimeoutMs: number;
  private logWriter: ReturnType<ReturnType<typeof Bun.file>["writer"]>;
  private readyResolve: (() => void) | null = null;
  private exited = false;
  private exitError: Error | null = null;

  private constructor(
    proc: Subprocess<"pipe", "pipe", "pipe">,
    opts: Required<Pick<DriverOptions, "sessionName" | "sessionDir" | "defaultTimeoutMs">>,
  ) {
    this.proc = proc;
    this.sessionName = opts.sessionName;
    this.sessionDir = opts.sessionDir;
    this.defaultTimeoutMs = opts.defaultTimeoutMs;
    this.logPath = join(opts.sessionDir, "app.log");
    this.logWriter = Bun.file(this.logPath).writer();
  }

  static async launch(options: DriverOptions = {}): Promise<Driver> {
    const binary = options.binary ?? resolveDefaultBinary();
    if (!existsSync(binary)) {
      throw new Error(
        `Binary not found at ${binary} (candidates checked: ${BINARY_CANDIDATES.join(", ")}). ` +
          `Build one with ./scripts/agentic/agent-cargo.sh build --bin script-kit-gpui`,
      );
    }

    // Unique per launch — multiple drivers in one process, and parallel
    // processes reusing the same sessionName, must never share artifacts.
    const launchId = `${process.pid}-${++launchCounter}-${Date.now().toString(36)}`;
    const sessionName = options.sessionName ?? `driver-${launchId}`;
    const sessionDir =
      options.sessionDir ??
      join(
        "/tmp/sk-driver-sessions",
        options.sessionName ? `${sessionName}-${launchId}` : sessionName,
      );
    rmSync(sessionDir, { recursive: true, force: true });
    mkdirSync(sessionDir, { recursive: true });

    const env: Record<string, string> = {
      ...(process.env as Record<string, string>),
      SCRIPT_KIT_AI_LOG: "1",
      SCRIPT_KIT_SHORTCUT_DEBUG: "1",
      RUST_LOG: DEFAULT_RUST_LOG,
      SCRIPT_KIT_AGENTIC_SESSION_NAME: sessionName,
      SCRIPT_KIT_AGENTIC_SESSION_GENERATION: `driver-${Date.now()}`,
      ...(options.env ?? {}),
    };
    if (options.protocolBusFile !== false) {
      env.SCRIPT_KIT_AGENTIC_PROTOCOL_RESPONSES_PATH = join(
        sessionDir,
        "protocol-responses.ndjson",
      );
    }
    if (options.sandboxHome) {
      const home = join(sessionDir, "home");
      const kitDir = join(home, ".scriptkit");
      mkdirSync(kitDir, { recursive: true });
      env.HOME = home;
      env.SK_PATH = kitDir;
      if (options.sharedModels !== false) {
        // Every model path resolves under $SK_PATH/models (dictation
        // Whisper/Parakeet, brain GGUF). Symlink the real cache so sandboxed
        // launches never re-download 1-2GB per session; downloads triggered
        // inside a sandbox land in the shared cache for future runs.
        const realModels = join(homedir(), ".scriptkit", "models");
        mkdirSync(realModels, { recursive: true });
        symlinkSync(realModels, join(kitDir, "models"));
      }
      if (options.seedAgentAuth) {
        const seed = Bun.spawnSync(
          ["bash", join(PROJECT_ROOT, "scripts/agentic/seed-sandbox-home.sh"), home],
          { stdout: "pipe", stderr: "pipe" },
        );
        if (seed.exitCode !== 0) {
          throw new Error(
            `seed-sandbox-home failed (exit ${seed.exitCode}): ${seed.stderr.toString().trim()}`,
          );
        }
        console.error(`[driver] ${seed.stdout.toString().trim()}`);
      }
    }

    const proc = Bun.spawn([binary], {
      cwd: PROJECT_ROOT,
      env,
      stdin: "pipe",
      stdout: "pipe",
      stderr: "pipe",
    });

    const driver = new Driver(proc, {
      sessionName,
      sessionDir,
      defaultTimeoutMs: options.defaultTimeoutMs ?? 5000,
    });

    const readyPromise = new Promise<void>((resolveReady) => {
      driver.readyResolve = resolveReady;
    });
    driver.consumeStream(proc.stdout, true);
    driver.consumeStream(proc.stderr, false);
    proc.exited.then((code) => {
      driver.exited = true;
      driver.exitError = new Error(
        `App process exited (code ${code}) — see ${driver.logPath}`,
      );
      driver.failAllPending(driver.exitError);
      driver.readyResolve?.();
    });

    const readyStart = performance.now();
    const readyTimeoutMs = options.readyTimeoutMs ?? 10_000;
    const timedOut = await Promise.race([
      readyPromise.then(() => false),
      Bun.sleep(readyTimeoutMs).then(() => true),
    ]);
    driver.stats.readyWaitMs = Math.round(performance.now() - readyStart);
    if (driver.exited) {
      throw driver.exitError ?? new Error("App process exited during startup");
    }
    if (timedOut) {
      // Marker not seen — fall back to a protocol probe before giving up.
      try {
        await driver.request({ type: "getState" }, { timeoutMs: 2000 });
      } catch {
        await driver.close();
        throw new Error(
          `App did not become ready within ${readyTimeoutMs}ms — see ${driver.logPath}`,
        );
      }
    }
    return driver;
  }

  // --- transport -------------------------------------------------------------

  /** Fire-and-forget: write one command line to the app's stdin. */
  send(command: Json): void {
    if (this.exited) {
      throw this.exitError ?? new Error("App process has exited");
    }
    this.proc.stdin.write(`${JSON.stringify(command)}\n`);
    this.proc.stdin.flush();
  }

  /**
   * Send a command and resolve when the response line carrying the same
   * requestId arrives on stdout. Event-driven — no polling, no subprocesses.
   */
  request(
    command: Json,
    opts: { expect?: string; timeoutMs?: number } = {},
  ): Promise<Json> {
    const requestId: string =
      typeof command.requestId === "string" && command.requestId.length > 0
        ? command.requestId
        : `drv-${++this.requestCounter}`;
    const payload = { ...command, requestId };
    const timeoutMs = opts.timeoutMs ?? this.defaultTimeoutMs;

    return new Promise<Json>((resolvePromise, rejectPromise) => {
      const timer = setTimeout(() => {
        this.pending.delete(requestId);
        rejectPromise(
          new Error(
            `Timeout (${timeoutMs}ms) waiting for response to requestId '${requestId}' (${payload.type})`,
          ),
        );
      }, timeoutMs);
      this.pending.set(requestId, {
        resolve: resolvePromise,
        reject: rejectPromise,
        expect: opts.expect,
        timer,
      });
      this.stats.requestsSent += 1;
      try {
        this.send(payload);
      } catch (error) {
        clearTimeout(timer);
        this.pending.delete(requestId);
        rejectPromise(error instanceof Error ? error : new Error(String(error)));
      }
    });
  }

  // --- typed helpers -----------------------------------------------------------

  getState(opts: { timeoutMs?: number } = {}): Promise<Json> {
    return this.request({ type: "getState" }, { expect: "stateResult", ...opts });
  }

  getElements(extra: Json = {}, opts: { timeoutMs?: number } = {}): Promise<Json> {
    return this.request({ type: "getElements", ...extra }, opts);
  }

  getLayoutInfo(extra: Json = {}, opts: { timeoutMs?: number } = {}): Promise<Json> {
    return this.request({ type: "getLayoutInfo", ...extra }, opts);
  }

  setFilter(text: string): void {
    this.send({ type: "setFilter", text });
  }

  simulateKey(key: string, modifiers: string[] = []): void {
    this.send({ type: "simulateKey", key, modifiers });
  }

  simulateGpuiEvent(
    event: Json,
    opts: { target?: Json; timeoutMs?: number } = {},
  ): Promise<Json> {
    const command: Json = { type: "simulateGpuiEvent", event };
    if (opts.target !== undefined) command.target = opts.target;
    return this.request(command, {
      expect: "simulateGpuiEventResult",
      timeoutMs: opts.timeoutMs ?? this.defaultTimeoutMs,
    });
  }

  async simulateGpuiClick(
    x: number,
    y: number,
    opts: { target?: Json; button?: string; timeoutMs?: number } = {},
  ): Promise<Json[]> {
    const eventTarget = opts.target;
    const timeoutMs = opts.timeoutMs;
    const button = opts.button ?? "left";
    const move = await this.simulateGpuiEvent(
      { type: "mouseMove", x, y },
      { target: eventTarget, timeoutMs },
    );
    const click = await this.simulateGpuiEvent(
      { type: "mouseClick", button, x, y },
      { target: eventTarget, timeoutMs },
    );
    return [move, click];
  }

  waitFor(
    condition: Json | string,
    opts: { timeoutMs?: number; pollIntervalMs?: number } = {},
  ): Promise<Json> {
    const timeout = opts.timeoutMs ?? this.defaultTimeoutMs;
    return this.request(
      {
        type: "waitFor",
        condition,
        timeout,
        pollInterval: opts.pollIntervalMs ?? 5,
      },
      { expect: "waitForResult", timeoutMs: timeout + 1000 },
    );
  }

  /** Wait until getState matches the given partial state. */
  waitForState(
    state: Json,
    opts: { timeoutMs?: number; pollIntervalMs?: number } = {},
  ): Promise<Json> {
    return this.waitFor({ type: "stateMatch", state }, opts);
  }

  /**
   * Wait until the observed state stops changing: resolves once `samples`
   * consecutive probes return an identical fingerprint. Use this instead of
   * hardcoded sleeps (the scattered `sleep(1500)` settle hacks) before the
   * first submit after opening a surface — it returns as soon as the surface
   * is actually stable rather than after a guessed delay.
   *
   * Returns { settled, elapsedMs, probes, lastState }. `settled: false`
   * means the timeout elapsed while state was still changing — treat that
   * as a receipt to report, not a silent pass.
   */
  async waitForSettle(
    opts: {
      /** Consecutive identical samples required. Default 3. */
      samples?: number;
      /** Delay between samples. Default 100ms. */
      intervalMs?: number;
      /** Overall deadline. Default 5000ms. */
      timeoutMs?: number;
      /** Custom probe; defaults to getState. Must return comparable JSON. */
      probe?: () => Promise<Json>;
    } = {},
  ): Promise<{ settled: boolean; elapsedMs: number; probes: number; lastState: Json }> {
    const required = Math.max(2, opts.samples ?? 3);
    const intervalMs = opts.intervalMs ?? 100;
    const timeoutMs = opts.timeoutMs ?? 5000;
    const probe = opts.probe ?? (() => this.getState());
    const start = performance.now();

    let lastFingerprint = "";
    let stableCount = 0;
    let probes = 0;
    let lastState: Json = {};
    while (performance.now() - start < timeoutMs) {
      lastState = await probe();
      probes += 1;
      // Every response carries its own requestId; exclude it (top-level)
      // from the fingerprint or no two probes would ever match.
      const { requestId: _requestId, ...comparable } = lastState;
      const fingerprint = JSON.stringify(comparable);
      stableCount = fingerprint === lastFingerprint ? stableCount + 1 : 1;
      lastFingerprint = fingerprint;
      if (stableCount >= required) {
        return {
          settled: true,
          elapsedMs: Math.round(performance.now() - start),
          probes,
          lastState,
        };
      }
      await Bun.sleep(intervalMs);
    }
    return {
      settled: false,
      elapsedMs: Math.round(performance.now() - start),
      probes,
      lastState,
    };
  }

  /** One round trip: setFilter + wait until the input value is applied. */
  async setFilterAndWait(
    text: string,
    opts: { timeoutMs?: number } = {},
  ): Promise<Json> {
    this.setFilter(text);
    // stdin is processed serially by the app, so by the time waitFor runs
    // the setFilter has already been applied — this usually hits the
    // already-satisfied fast path and returns immediately.
    return this.waitForState({ inputValue: text }, opts);
  }

  batch(
    commands: Json[],
    opts: { stopOnError?: boolean; timeoutMs?: number } = {},
  ): Promise<Json> {
    const timeout = opts.timeoutMs ?? this.defaultTimeoutMs;
    return this.request(
      {
        type: "batch",
        commands,
        options: { stopOnError: opts.stopOnError ?? true, timeout },
      },
      { expect: "batchResult", timeoutMs: timeout + 1000 },
    );
  }

  listAutomationWindows(opts: { timeoutMs?: number } = {}): Promise<Json> {
    return this.request({ type: "listAutomationWindows" }, opts);
  }

  /**
   * Fetch recent structured log entries from the app's in-process ring
   * buffer (last 500 events). Filters: limit, level (min severity),
   * target (substring), contains (message substring). Lets a probe assert
   * on log content without reading files off disk.
   */
  getLogs(
    filters: { limit?: number; level?: string; target?: string; contains?: string } = {},
    opts: { timeoutMs?: number } = {},
  ): Promise<Json> {
    return this.request(
      { type: "getLogs", ...filters },
      { expect: "logsResult", ...opts },
    );
  }

  /**
   * Capture a screenshot of the app (whole main window by default, or a
   * specific automation window via `target`). Returns the screenshotResult
   * message ({ data: base64 PNG, width, height } or { error }). Pass
   * `savePath` to also decode and write the PNG to disk.
   */
  async captureScreenshot(
    opts: {
      hiDpi?: boolean;
      target?: Json;
      savePath?: string;
      timeoutMs?: number;
    } = {},
  ): Promise<Json> {
    const command: Json = { type: "captureScreenshot" };
    if (opts.hiDpi !== undefined) command.hiDpi = opts.hiDpi;
    if (opts.target !== undefined) command.target = opts.target;
    const result = (await this.request(command, {
      expect: "screenshotResult",
      timeoutMs: opts.timeoutMs ?? 10_000,
    })) as { data?: string; error?: string };
    if (opts.savePath && result.data && !result.error) {
      const { writeFileSync } = await import("node:fs");
      writeFileSync(opts.savePath, Buffer.from(result.data, "base64"));
    }
    return result as Json;
  }

  // --- lifecycle ---------------------------------------------------------------

  get alive(): boolean {
    return !this.exited;
  }

  /** OS pid of the app process (for `sample`/profiling). */
  get pid(): number | undefined {
    return this.proc.pid;
  }

  async close(): Promise<void> {
    this.failAllPending(new Error("Driver closed"));
    if (!this.exited) {
      try {
        this.proc.kill();
      } catch {
        // already gone
      }
      await Promise.race([this.proc.exited, Bun.sleep(2000)]);
      if (!this.exited) {
        try {
          this.proc.kill(9);
        } catch {
          // already gone
        }
        await Promise.race([this.proc.exited, Bun.sleep(1000)]);
      }
    }
    try {
      await this.logWriter.flush();
      await this.logWriter.end();
    } catch {
      // log writer may already be closed
    }
  }

  // --- internals -----------------------------------------------------------------

  private failAllPending(error: Error): void {
    for (const [, pending] of this.pending) {
      clearTimeout(pending.timer);
      pending.reject(error);
    }
    this.pending.clear();
  }

  private async consumeStream(
    stream: ReadableStream<Uint8Array>,
    isStdout: boolean,
  ): Promise<void> {
    const decoder = new TextDecoder();
    let buffer = "";
    try {
      for await (const chunk of stream) {
        buffer += decoder.decode(chunk, { stream: true });
        let newlineIndex = buffer.indexOf("\n");
        while (newlineIndex >= 0) {
          const line = buffer.slice(0, newlineIndex);
          buffer = buffer.slice(newlineIndex + 1);
          this.handleLine(line, isStdout);
          newlineIndex = buffer.indexOf("\n");
        }
      }
    } catch {
      // stream closed with the process
    }
    if (buffer.length > 0) {
      this.handleLine(buffer, isStdout);
    }
  }

  private handleLine(line: string, isStdout: boolean): void {
    this.logWriter.write(`${line}\n`);

    if (
      this.readyResolve &&
      (line.includes(READY_MARKER_STARTUP) || line.includes(READY_MARKER_APP))
    ) {
      const resolveReady = this.readyResolve;
      this.readyResolve = null;
      resolveReady();
    }

    if (!isStdout) return;
    const trimmed = line.trimStart();
    if (!trimmed.startsWith("{")) return;

    let parsed: Json;
    try {
      parsed = JSON.parse(trimmed);
    } catch {
      return;
    }
    const requestId = parsed.requestId;
    if (typeof requestId !== "string") return;
    const pending = this.pending.get(requestId);
    if (!pending) {
      this.stats.unmatchedResponses += 1;
      return;
    }
    if (pending.expect && parsed.type !== pending.expect) {
      // A different message reusing this requestId (e.g. an error envelope)
      // still settles the request — callers inspect `type` themselves.
      if (parsed.type !== "error" && parsed.type !== undefined) {
        // Allow mismatched-but-real responses through rather than hanging.
      }
    }
    this.pending.delete(requestId);
    clearTimeout(pending.timer);
    this.stats.responsesMatched += 1;
    pending.resolve(parsed);
  }
}

// --- CLI smoke check -------------------------------------------------------------

if (import.meta.main) {
  const mode = process.argv[2] ?? "smoke";
  if (mode !== "smoke") {
    console.error("Usage: bun scripts/devtools/driver.ts smoke");
    process.exit(2);
  }
  const started = performance.now();
  const driver = await Driver.launch({ sandboxHome: true });
  const launchedMs = Math.round(performance.now() - started);

  const rpcStart = performance.now();
  const state = await driver.getState();
  const stateMs = Math.round(performance.now() - rpcStart);

  const filterStart = performance.now();
  await driver.setFilterAndWait("smoke");
  const filterMs = Math.round(performance.now() - filterStart);

  await driver.close();
  console.log(
    JSON.stringify(
      {
        schemaVersion: 1,
        status: "ok",
        launchMs: launchedMs,
        readyWaitMs: driver.stats.readyWaitMs,
        getStateMs: stateMs,
        setFilterAndWaitMs: filterMs,
        promptType: state.promptType ?? null,
        inputValueAfterFilter: "smoke",
        stats: driver.stats,
        log: driver.logPath,
      },
      null,
      2,
    ),
  );
}
