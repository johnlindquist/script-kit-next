/**
 * Warm imp: holds ONE persistent `codex app-server` process alive (via
 * AppServerClient) so each invocation skips process spawn + auth/config load +
 * WebSocket connect/prewarm. Measured: ~2s for a short answer vs ~5.4s cold,
 * with the first protocol frame back in ~1ms.
 *
 * Protocol: newline-delimited JSON over a Unix socket at
 *   /tmp/codex-imp-{name}.sock
 *
 * Client -> Imp (one line):
 *   { "prompt": "...", "quiet": false, "cwd": "/abs/path", "effort": "low" }
 *
 * Imp -> Client (many lines, then close):
 *   { "type": "notif", "method": "...", "params": {...} }   (streaming, non-quiet)
 *   { "type": "final", "text": "..." }                      (always, on completion)
 *   { "type": "error", "message": "..." }
 *   { "type": "done" }
 */

import { createServer, createConnection, type Socket } from "net";
import { existsSync, unlinkSync, readFileSync, readdirSync, writeFileSync, realpathSync } from "fs";
import { join, dirname } from "path";
import { createHash } from "crypto";
import { spawn } from "child_process";
import type { ImpConfig } from "./isolated.ts";
import { AppServerClient } from "./appserver.ts";
import { selfImproveFingerprintParts } from "./self-improve.ts";

export function socketPath(name: string): string {
  return `/tmp/codex-imp-${name}.sock`;
}

/** Sidecar file recording the running imp's pid + source fingerprint. */
export function metaPath(name: string): string {
  return `/tmp/codex-imp-${name}.meta.json`;
}

/**
 * Fingerprint the imp's own source: the executable that defines the profile
 * (its instructions, model, env) plus every lib/*.ts module it loads at startup.
 * A change to any of these means a long-lived warm imp is now running stale
 * code, so the next prompt must restart it. We hash file *contents* (not just
 * mtime) so edits are caught even when timestamps are preserved (git checkout,
 * tarball extraction, etc.).
 */
export function sourceFingerprint(config?: ImpConfig): string {
  const files: string[] = [];
  let exe: string;
  try {
    exe = realpathSync(process.argv[1]);
  } catch {
    exe = process.argv[1];
  }
  files.push(exe);
  // lib/ sits next to the imps/ dir: <repo>/imps/imp-X -> <repo>/lib/*.ts
  const libDir = join(dirname(exe), "..", "lib");
  try {
    for (const f of readdirSync(libDir)) {
      if (f.endsWith(".ts")) files.push(join(libDir, f));
    }
  } catch {}
  const hash = createHash("sha256");
  for (const part of config ? selfImproveFingerprintParts(config) : []) {
    hash.update(part);
    hash.update("\0");
    if (part.startsWith("path:")) files.push(part.slice("path:".length));
  }
  files.sort();
  for (const f of files) {
    try {
      hash.update(f);
      hash.update("\0");
      hash.update(readFileSync(f));
      hash.update("\0");
    } catch {}
  }
  return hash.digest("hex");
}

export interface ImpMeta {
  pid?: number;
  fp?: string;
  startedAt?: number;
  idleMinutes?: number;
}

export function readMeta(name: string): ImpMeta | null {
  try {
    return JSON.parse(readFileSync(metaPath(name), "utf8"));
  } catch {
    return null;
  }
}

export async function serveImp(config: ImpConfig): Promise<void> {
  const sock = socketPath(config.name);
  if (existsSync(sock)) {
    // Probe — if no one's listening, remove stale socket
    try {
      await new Promise<void>((resolve, reject) => {
        const probe = createConnection(sock);
        probe.once("connect", () => { probe.end(); reject(new Error("alive")); });
        probe.once("error", () => resolve());
      });
      unlinkSync(sock);
    } catch {
      console.error(`${config.name} warm imp already running at ${sock}`);
      process.exit(1);
    }
  }

  const client = new AppServerClient(config);
  await client.start();
  console.error(`${config.name} warm imp ready at ${sock} (pid ${process.pid}, app-server warm)`);

  // Idle shutdown: a warm imp nobody is talking to exits on its own, so the
  // fleet doesn't accumulate resident app-server processes. The next call
  // auto-respawns it (ensureWarmImp) — costs one warm-up, never a failure.
  // CODEX_IMP_IDLE_MINUTES=0 disables; default 30.
  const idleMinutes = Number(process.env.CODEX_IMP_IDLE_MINUTES ?? "30");
  const idleMs = Number.isFinite(idleMinutes) && idleMinutes > 0 ? idleMinutes * 60_000 : 0;
  let lastActivity = Date.now();
  let activeTurns = 0;
  if (idleMs > 0) {
    setInterval(() => {
      if (activeTurns === 0 && Date.now() - lastActivity > idleMs) {
        console.error(`${config.name} warm imp idle for ${idleMinutes}m — shutting down`);
        shutdown();
      }
    }, 30_000).unref?.();
  }

  // Requests are serialized: one warm app-server, one turn at a time.
  let chain: Promise<void> = Promise.resolve();

  const server = createServer((socket: Socket) => {
    let buf = "";
    socket.on("data", (chunk) => {
      buf += chunk.toString("utf8");
      const nl = buf.indexOf("\n");
      if (nl === -1) return;
      const line = buf.slice(0, nl);
      buf = buf.slice(nl + 1);

      let req: { prompt?: string; quiet?: boolean; cwd?: string; effort?: string; ctl?: string };
      try {
        req = JSON.parse(line);
      } catch (e: any) {
        socket.write(JSON.stringify({ type: "error", message: `bad json: ${e.message}` }) + "\n");
        socket.end();
        return;
      }

      const send = (obj: unknown) => socket.write(JSON.stringify(obj) + "\n");

      // Control message: graceful stop (used to restart a stale imp when we
      // don't have its pid). Ack, flush, then shut down.
      if (req.ctl === "stop") {
        send({ type: "done" });
        socket.end();
        setTimeout(shutdown, 50);
        return;
      }

      if (typeof req.prompt !== "string") {
        send({ type: "error", message: "missing prompt" });
        socket.end();
        return;
      }
      const prompt = req.prompt;

      lastActivity = Date.now();
      activeTurns++;
      chain = chain.then(async () => {
        try {
          const finalText = await client.runTurn(
            prompt,
            {
              onNotification: (method, params) => {
                if (!req.quiet) send({ type: "notif", method, params });
              },
            },
            { cwd: req.cwd, effort: req.effort },
          );
          send({ type: "final", text: finalText });
          send({ type: "done" });
        } catch (e: any) {
          send({ type: "error", message: e.message || String(e) });
        } finally {
          activeTurns--;
          lastActivity = Date.now();
          socket.end();
        }
      });
    });
    socket.on("error", () => {});
  });

  server.listen(sock, () => {
    // Record pid + the source fingerprint this imp was started with, so a
    // freshly-launched client can detect when the on-disk source has changed
    // and restart us. Written once the socket is accepting connections.
    try {
      writeFileSync(
        metaPath(config.name),
        JSON.stringify({ pid: process.pid, fp: sourceFingerprint(config), startedAt: Date.now(), idleMinutes }),
      );
    } catch {}
  });

  const shutdown = () => {
    server.close();
    try { unlinkSync(sock); } catch {}
    try { unlinkSync(metaPath(config.name)); } catch {}
    client.close();
    process.exit(0);
  };
  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);

  await new Promise(() => {}); // wait forever
}

export interface ClientEventHandlers {
  onNotification?: (method: string, params: any) => void;
  onFinal?: (text: string) => void;
  onError?: (message: string) => void;
}

export function clientAvailable(name: string): boolean {
  return existsSync(socketPath(name));
}

/** Try to open the imp socket; resolves true only if a live listener accepts. */
export function tryConnect(sock: string, timeoutMs: number): Promise<boolean> {
  return new Promise((resolve) => {
    const socket = createConnection(sock);
    const done = (ok: boolean) => { clearTimeout(timer); socket.destroy(); resolve(ok); };
    const timer = setTimeout(() => done(false), timeoutMs);
    socket.once("connect", () => done(true));
    socket.once("error", () => done(false));
  });
}

/** Ask a running imp to stop via its socket (no pid needed). Resolves once closed. */
function sendStop(sock: string): Promise<void> {
  return new Promise((resolve) => {
    const s = createConnection(sock);
    const done = () => { try { s.destroy(); } catch {} resolve(); };
    s.once("connect", () => { try { s.write(JSON.stringify({ ctl: "stop" }) + "\n"); } catch { done(); } });
    s.once("end", done);
    s.once("close", done);
    s.once("error", done);
    setTimeout(done, 2000);
  });
}

/**
 * Stop a running imp and wait until its socket stops accepting connections.
 * Prefers a pid-targeted SIGTERM (works regardless of imp busyness); falls
 * back to a socket control message when the pid is unknown (e.g. an imp
 * started by an older build with no meta file). Escalates to SIGKILL if needed,
 * then clears any stale socket/meta so a fresh imp can start clean.
 */
export async function stopWarmImp(name: string, pid?: number): Promise<void> {
  const sock = socketPath(name);
  if (pid) {
    try { process.kill(pid, "SIGTERM"); } catch {}
  } else {
    await sendStop(sock);
  }

  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    if (!existsSync(sock) || !(await tryConnect(sock, 300))) break;
    await new Promise((r) => setTimeout(r, 100));
  }

  // Still alive? Force it down.
  if (pid && existsSync(sock) && (await tryConnect(sock, 300))) {
    try { process.kill(pid, "SIGKILL"); } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }

  try { if (existsSync(sock)) unlinkSync(sock); } catch {}
  try { unlinkSync(metaPath(name)); } catch {}
}

/**
 * Ensure a warm imp is running and reachable for this profile, auto-starting
 * one in the background if needed. This makes warm mode the DEFAULT: the first
 * call pays the startup cost once, then every later call routes through the
 * persistent app-server for instant responses. Returns true if a live imp is
 * reachable, false if startup failed (caller should fall back to a cold run).
 *
 * Hot reload: this runs in a freshly-launched client process, so it always sees
 * the current on-disk source. If a warm imp is running stale code (its
 * recorded fingerprint no longer matches the source on disk), we stop it and
 * spawn a fresh one — so editing an imp's instructions/model, or any lib/*.ts,
 * takes effect on the very next prompt.
 */
export async function ensureWarmImp(config: ImpConfig, readyTimeoutMs = 30000): Promise<boolean> {
  const sock = socketPath(config.name);
  const current = sourceFingerprint(config);

  // Already warm and accepting connections?
  if (existsSync(sock) && (await tryConnect(sock, 500))) {
    const meta = readMeta(config.name);
    if (meta && meta.fp === current) return true; // up-to-date warm imp — reuse it
    // Source changed since this imp started (or it predates fingerprinting).
    // Restart so the next prompt runs the edited code.
    await stopWarmImp(config.name, meta?.pid);
    // fall through to spawn a fresh imp
  }

  // Spawn a detached background imp: re-run THIS executable with --serve.
  // It cleans up any stale socket on start, then listens once the app-server is warm.
  // cwd is pinned to HOME, NOT the caller's cwd: the server outlives the caller, and
  // a deleted cwd (e.g. a temp dir) makes codex fail to load configuration on every
  // later turn. Per-request cwd is passed explicitly with each prompt.
  try {
    const child = spawn(process.argv[0], [process.argv[1], "--serve"], {
      detached: true,
      stdio: "ignore",
      cwd: process.env.HOME || "/",
    });
    child.unref();
  } catch {
    return false;
  }

  // Poll until the imp accepts connections (= app-server warm) or we give up.
  const deadline = Date.now() + readyTimeoutMs;
  while (Date.now() < deadline) {
    if (await tryConnect(sock, 500)) return true;
    await new Promise((r) => setTimeout(r, 150));
  }
  return false;
}

export async function runViaWarmImp(
  name: string,
  req: { prompt: string; quiet: boolean; cwd: string; effort?: string },
  handlers: ClientEventHandlers,
  signal?: AbortSignal,
): Promise<void> {
  const sock = socketPath(name);
  await new Promise<void>((resolve, reject) => {
    const socket = createConnection(sock);
    let buf = "";

    const onAbort = () => { socket.destroy(); reject(new Error("aborted")); };
    if (signal) signal.addEventListener("abort", onAbort);

    socket.once("connect", () => {
      socket.write(JSON.stringify(req) + "\n");
    });
    socket.on("data", (chunk) => {
      buf += chunk.toString("utf8");
      let nl;
      while ((nl = buf.indexOf("\n")) !== -1) {
        const line = buf.slice(0, nl);
        buf = buf.slice(nl + 1);
        if (!line) continue;
        let msg: any;
        try { msg = JSON.parse(line); } catch { continue; }
        if (msg.type === "notif") handlers.onNotification?.(msg.method, msg.params);
        else if (msg.type === "final") handlers.onFinal?.(msg.text);
        else if (msg.type === "error") handlers.onError?.(msg.message);
      }
    });
    socket.once("end", () => {
      if (signal) signal.removeEventListener("abort", onAbort);
      resolve();
    });
    socket.once("error", (e) => {
      if (signal) signal.removeEventListener("abort", onAbort);
      reject(e);
    });
  });
}
