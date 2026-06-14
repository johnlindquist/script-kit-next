/**
 * Warm app-server client.
 *
 * Launches ONE persistent `codex app-server` process (NDJSON JSON-RPC over stdio)
 * and holds it alive. The expensive setup — process spawn, auth/config load,
 * WebSocket connection + prewarm — is paid once at launch. Each user prompt is a
 * fresh `thread/start` + `turn/start` on the already-warm process, so time-to-first
 * activity is ~1ms and a short answer streams back in ~2s instead of ~5.4s cold.
 *
 * Wire protocol verified against codex-rs:
 *   - newline-delimited JSON, jsonrpc 2.0 (app-server-transport/src/transport/stdio.rs)
 *   - methods: initialize, thread/start, turn/start (app-server-protocol common.rs)
 *   - notifications: item/started, item/completed, item/agentMessage/delta,
 *     item/reasoning/textDelta, item/commandExecution/outputDelta, turn/completed
 */

import { spawn, type ChildProcess } from "child_process";
import { rmSync } from "fs";
import type { ImpConfig } from "./isolated.ts";
import { applyLessonOverlay, prepareIsolatedCodexHome } from "./codex-runtime.ts";
import { createSelfImproveObserver } from "./self-improve.ts";

export interface TurnHandlers {
  /** Raw app-server notification (method + params). */
  onNotification?: (method: string, params: any) => void;
}

export class AppServerClient {
  private child!: ChildProcess;
  private isolatedHome: string;
  private rbuf = "";
  private handlers = new Set<(msg: any) => void>();
  private nextId = 1;
  private stderrTail = "";
  private config: ImpConfig;
  private model: string;
  private ready = false;
  private hooksEnabled = false;

  constructor(config: ImpConfig) {
    // Fold accumulated self-improvement lessons into developerInstructions once,
    // at imp start. Hot-reload restarts this imp when the overlay changes.
    this.config = applyLessonOverlay(config);
    this.model = this.config.model || process.env.CODEX_IMP_MODEL || process.env.CODEX_PROFILE_MODEL || "gpt-5.3-codex-spark";
    this.isolatedHome = `/tmp/codex-appserver-${this.config.name}-${process.pid}`;
  }

  /** Spawn the app-server and complete the initialize handshake. */
  async start(): Promise<void> {
    const realHome = process.env.HOME!;
    // Symlinks auth, and (when self-improvement is enabled) writes the hook config.
    const runtime = prepareIsolatedCodexHome(this.config, this.isolatedHome, realHome);
    this.hooksEnabled = runtime.hooksEnabled;

    this.child = spawn("codex", ["app-server"], {
      env: {
        PATH: process.env.PATH || "/usr/local/bin:/usr/bin:/bin",
        HOME: realHome,
        CODEX_HOME: this.isolatedHome,
        ...this.config.extraEnv,
        ...runtime.extraEnv,
      },
      stdio: ["pipe", "pipe", "pipe"],
    });
    this.child.stderr!.on("data", (c) => {
      this.stderrTail = (this.stderrTail + c.toString()).slice(-4000);
    });
    this.child.stdout!.on("data", (chunk) => this.onData(chunk));
    this.child.on("exit", (code) => {
      this.ready = false;
      for (const h of [...this.handlers]) h({ __exit: code });
    });

    const initId = this.send("initialize", {
      clientInfo: { name: `codex-imp-${this.config.name}`, version: "0.3.0" },
      capabilities: { experimentalApi: true },
    });
    await this.awaitResponse(initId);
    this.notify("initialized");
    this.ready = true;
  }

  isReady(): boolean {
    return this.ready;
  }

  private onData(chunk: Buffer) {
    this.rbuf += chunk.toString("utf8");
    let nl: number;
    while ((nl = this.rbuf.indexOf("\n")) !== -1) {
      const line = this.rbuf.slice(0, nl);
      this.rbuf = this.rbuf.slice(nl + 1);
      if (!line.trim()) continue;
      let msg: any;
      try { msg = JSON.parse(line); } catch { continue; }
      for (const h of [...this.handlers]) h(msg);
    }
  }

  private send(method: string, params?: unknown): number {
    const id = this.nextId++;
    this.child.stdin!.write(JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n");
    return id;
  }

  private notify(method: string, params?: unknown) {
    this.child.stdin!.write(JSON.stringify({ jsonrpc: "2.0", method, params }) + "\n");
  }

  private awaitResponse(id: number, timeoutMs = 60000): Promise<any> {
    return new Promise((resolve, reject) => {
      const t = setTimeout(() => {
        this.handlers.delete(h);
        reject(new Error(`timeout waiting for id ${id}\nstderr:\n${this.stderrTail}`));
      }, timeoutMs);
      const h = (msg: any) => {
        if (msg.__exit !== undefined) {
          clearTimeout(t); this.handlers.delete(h);
          reject(new Error(`app-server exited (code ${msg.__exit})\nstderr:\n${this.stderrTail}`));
          return;
        }
        if (msg.id === id && (msg.result !== undefined || msg.error !== undefined)) {
          clearTimeout(t); this.handlers.delete(h);
          if (msg.error) reject(new Error(`rpc error id ${id}: ${JSON.stringify(msg.error)}`));
          else resolve(msg.result);
        }
      };
      this.handlers.add(h);
    });
  }

  /** Start a fresh thread carrying this profile's isolation config. Returns thread id. */
  private async startThread(): Promise<string> {
    const id = this.send("thread/start", {
      model: this.model,
      sandbox: this.config.sandboxMode || "danger-full-access",
      approvalPolicy: "never",
      baseInstructions: this.config.baseInstructions,
      developerInstructions: this.config.developerInstructions,
      ephemeral: true,
      config: {
        model_reasoning_effort: this.config.reasoningEffort || "low",
        show_raw_agent_reasoning: true,
        skills: { include_instructions: false },
        include_apps_instructions: false,
        include_environment_context: false,
        include_collaboration_mode_instructions: false,
        include_permissions_instructions: false,
        project_doc_max_bytes: 0,
        memories: { use_memories: false },
        mcp_servers: {},
        web_search: "disabled",
        // Non-interactive isolated imps own a throwaway CODEX_HOME, so user
        // hooks can't be approved via a TUI — bypass trust to let them run.
        // Passed here (not just config.toml) because thread/start config does
        // not inherit the on-disk bypass flag.
        bypass_hook_trust: this.hooksEnabled,
        features: {
          plugins: false, hooks: this.hooksEnabled, memories: false, apps: false,
          image_generation: false, tool_search: false, tool_suggest: false,
        },
      },
    });
    const res = await this.awaitResponse(id);
    return res.thread?.id || res.thread_id || res.threadId;
  }

  /**
   * Run one prompt on a FRESH thread (stateless, isolated per invocation).
   * Streams notifications via handlers.onNotification. Resolves with the final
   * agent message text on turn/completed.
   */
  async runTurn(prompt: string, handlers: TurnHandlers, opts?: { cwd?: string; effort?: string }): Promise<string> {
    if (!this.ready) throw new Error("app-server not ready");
    const threadId = await this.startThread();
    const observer = createSelfImproveObserver(this.config);

    return new Promise<string>((resolve, reject) => {
      let finalText = "";
      const t = setTimeout(() => {
        this.handlers.delete(h);
        observer.finish({ status: "timeout", transport: "app-server" });
        reject(new Error(`turn timeout\nstderr:\n${this.stderrTail}`));
      }, 120000);
      const h = (msg: any) => {
        if (msg.__exit !== undefined) {
          clearTimeout(t); this.handlers.delete(h);
          observer.finish({ status: "app-server-exit", transport: "app-server", code: msg.__exit });
          reject(new Error(`app-server exited mid-turn (code ${msg.__exit})\nstderr:\n${this.stderrTail}`));
          return;
        }
        if (!msg.method) return;
        observer.onAppServerNotification(msg.method, msg.params);
        handlers.onNotification?.(msg.method, msg.params);
        if (msg.method === "item/agentMessage/delta") {
          finalText += msg.params?.delta ?? "";
        } else if (msg.method === "item/completed" && msg.params?.item?.type === "agentMessage") {
          // Authoritative full text (covers non-streamed/low-effort paths)
          if (msg.params.item.text) finalText = msg.params.item.text;
        } else if (msg.method === "turn/completed") {
          clearTimeout(t); this.handlers.delete(h);
          observer.finish({ status: "completed", transport: "app-server" });
          resolve(finalText);
        }
      };
      this.handlers.add(h);
      this.send("turn/start", {
        threadId,
        input: [{ type: "text", text: prompt, text_elements: [] }],
        cwd: opts?.cwd || process.cwd(),
        effort: opts?.effort || this.config.reasoningEffort || "low",
      });
    });
  }

  close() {
    try { this.child?.kill("SIGTERM"); } catch {}
    try { rmSync(this.isolatedHome, { recursive: true, force: true }); } catch {}
  }
}
