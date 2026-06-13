#!/usr/bin/env bun
import { mkdtempSync, rmSync } from "fs";
import { join } from "path";
import { tmpdir } from "os";

type JsonObject = Record<string, unknown>;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export function assert(condition: unknown, message: string): asserts condition {
  if (!condition) {
    throw new Error(message);
  }
}

export class NotesAgentChatHarness {
  readonly home: string;
  readonly dbPath: string;
  readonly logLines: string[] = [];

  private proc: ReturnType<typeof Bun.spawn>;
  private preserveHome: boolean;
  private writer: {
    write(chunk: string | Uint8Array): number | Promise<number>;
    end(): void | Promise<void>;
  };
  private buffer = "";
  private responseWaiters = new Map<
    string,
    { resolve: (value: JsonObject) => void; reject: (error: Error) => void }
  >();
  private logWaiters: Array<{
    predicate: (line: string) => boolean;
    resolve: (line: string) => void;
  }> = [];

  constructor(
    readonly scenario: string,
    options: { home?: string; preserveHome?: boolean } = {}
  ) {
    this.home = options.home ?? mkdtempSync(join(tmpdir(), `sk-${scenario}-`));
    this.preserveHome = options.preserveHome ?? false;
    this.dbPath = join(this.home, ".scriptkit", "db", "notes.sqlite");
    const binary = process.env.PROBE_BINARY ?? "target/debug/script-kit-gpui";
    this.proc = Bun.spawn([binary], {
      cwd: join(import.meta.dir, "../.."),
      stdin: "pipe",
      stdout: "pipe",
      stderr: "pipe",
      env: {
        ...process.env,
        HOME: this.home,
        SCRIPT_KIT_AI_LOG: "1",
      },
    });
    this.writer = this.proc.stdin;
    void this.readLines(this.proc.stdout);
    void this.readLines(this.proc.stderr);
  }

  async ready(): Promise<void> {
    await this.waitForLog(
      (line) => line.includes("STARTUP_READY") || line.includes("APP_READY"),
      12_000,
      "app readiness marker"
    );
  }

  async openNotes(): Promise<void> {
    await this.send({ type: "openNotes", requestId: `${this.scenario}-open-notes` });
    await this.waitForLog(
      (line) => line.includes("automation.runtime_handle_upserted window_id=notes"),
      8_000,
      "Notes automation window registration"
    );
  }

  async send(payload: JsonObject): Promise<void> {
    await this.writer.write(`${JSON.stringify(payload)}\n`);
  }

  async request(payload: JsonObject, timeoutMs = 8_000): Promise<JsonObject> {
    const requestId = String(payload.requestId ?? `${this.scenario}-${Date.now()}`);
    payload.requestId = requestId;
    const responsePromise = new Promise<JsonObject>((resolve, reject) => {
      this.responseWaiters.set(requestId, { resolve, reject });
    });
    await this.send(payload);
    return await this.withTimeout(responsePromise, timeoutMs, `response ${requestId}`);
  }

  async gpuiKey(
    requestId: string,
    key: string,
    modifiers: string[] = [],
    timeoutMs = 8_000
  ): Promise<JsonObject> {
    return await this.request(
      {
        type: "simulateGpuiEvent",
        requestId,
        target: { type: "kind", kind: "notes", index: 0 },
        event: { type: "keyDown", key, modifiers },
      },
      timeoutMs
    );
  }

  async notesBatch(
    requestId: string,
    commands: JsonObject[],
    timeoutMs = 8_000
  ): Promise<JsonObject> {
    return await this.request(
      {
        type: "batch",
        requestId,
        target: { type: "kind", kind: "notes", index: 0 },
        commands,
      },
      timeoutMs
    );
  }

  async getNotesAgentChatState(requestId: string): Promise<JsonObject> {
    return await this.request({
      type: "getAgentChatState",
      requestId,
      target: { type: "kind", kind: "notes", index: 0 },
    });
  }

  async waitForLog(
    predicate: (line: string) => boolean,
    timeoutMs: number,
    label: string
  ): Promise<string> {
    for (const line of this.logLines) {
      if (predicate(line)) return line;
    }
    const pending = new Promise<string>((resolve) => {
      this.logWaiters.push({ predicate, resolve });
    });
    return await this.withTimeout(pending, timeoutMs, label);
  }

  countLogs(pattern: string): number {
    return this.logLines.filter((line) => line.includes(pattern)).length;
  }

  async cleanup(): Promise<void> {
    try {
      await this.writer.end();
    } catch {
      // process may already be gone
    }
    this.proc.kill();
    try {
      await this.proc.exited;
    } catch {
      // process may already be gone
    }
    if (!this.preserveHome) {
      rmSync(this.home, { recursive: true, force: true });
    }
  }

  private async readLines(stream: ReadableStream<Uint8Array> | null): Promise<void> {
    if (!stream) return;
    const reader = stream.getReader();
    while (true) {
      const { done, value } = await reader.read();
      if (done) break;
      this.buffer += decoder.decode(value, { stream: true });
      let newline = this.buffer.indexOf("\n");
      while (newline >= 0) {
        const line = this.buffer.slice(0, newline);
        this.buffer = this.buffer.slice(newline + 1);
        this.handleLine(line);
        newline = this.buffer.indexOf("\n");
      }
    }
  }

  private handleLine(line: string): void {
    if (!line) return;
    this.logLines.push(line);
    for (const waiter of [...this.logWaiters]) {
      if (waiter.predicate(line)) {
        this.logWaiters = this.logWaiters.filter((entry) => entry !== waiter);
        waiter.resolve(line);
      }
    }
    if (!line.startsWith("{")) return;
    try {
      const parsed = JSON.parse(line) as JsonObject;
      const requestId = parsed.requestId;
      if (typeof requestId !== "string") return;
      const waiter = this.responseWaiters.get(requestId);
      if (!waiter) return;
      this.responseWaiters.delete(requestId);
      waiter.resolve(parsed);
    } catch {
      // non-JSON log line
    }
  }

  private async withTimeout<T>(
    promise: Promise<T>,
    timeoutMs: number,
    label: string
  ): Promise<T> {
    let timeout: Timer | undefined;
    const timeoutPromise = new Promise<T>((_, reject) => {
      timeout = setTimeout(() => {
        reject(new Error(`Timed out waiting for ${label}`));
      }, timeoutMs);
    });
    try {
      return await Promise.race([promise, timeoutPromise]);
    } finally {
      if (timeout) clearTimeout(timeout);
    }
  }
}
