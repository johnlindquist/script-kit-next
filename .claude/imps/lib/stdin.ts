/**
 * Bounded stdin reading.
 *
 * `await Bun.stdin.text()` blocks until EOF. Callers that background an imp
 * (agent harnesses, task runners) can leave stdin open-but-silent forever,
 * parking the run before it spawns anything. These helpers read with a
 * first-byte window and an inter-chunk idle window instead, so an idle pipe
 * degrades to "no piped input" and a stalled pipe yields what arrived.
 */
import { impStdinFirstByteMs, impStdinIdleMs } from "./defaults.ts";

/**
 * Read a byte stream to a string with time bounds.
 *
 * - No first chunk within `firstByteMs` → "" (treated as no piped input).
 * - A stall longer than `idleMs` after data started → returns the partial
 *   text and warns on stderr (never hangs).
 * - Clean EOF → the full text, exactly like `Bun.stdin.text()`.
 */
export async function readBoundedText(
  stream: ReadableStream<Uint8Array>,
  firstByteMs: number,
  idleMs: number,
): Promise<string> {
  const reader = stream.getReader();
  const decoder = new TextDecoder();
  let out = "";
  let budgetMs = firstByteMs;
  try {
    for (;;) {
      let timer: ReturnType<typeof setTimeout> | undefined;
      const timeout = new Promise<"timeout">((resolve) => {
        timer = setTimeout(() => resolve("timeout"), budgetMs);
      });
      const result = await Promise.race([reader.read(), timeout]);
      clearTimeout(timer);
      if (result === "timeout") {
        if (out) {
          console.error(
            `imp: piped stdin stalled after ${out.length} chars — continuing with partial input`,
          );
        }
        break;
      }
      if (result.done) break;
      out += decoder.decode(result.value, { stream: true });
      budgetMs = idleMs;
    }
  } finally {
    try { reader.cancel(); } catch {}
    try { reader.releaseLock(); } catch {}
  }
  return out + decoder.decode();
}

/**
 * Drop-in replacement for `await Bun.stdin.text()` on the non-interactive
 * prompt path. TTY stdin is ignored; a silent pipe yields "" after the
 * first-byte window instead of hanging until EOF.
 */
export async function readPipedStdin(opts?: {
  firstByteMs?: number;
  idleMs?: number;
}): Promise<string> {
  if (process.stdin.isTTY) return "";
  return readBoundedText(
    Bun.stdin.stream(),
    opts?.firstByteMs ?? impStdinFirstByteMs(),
    opts?.idleMs ?? impStdinIdleMs(),
  );
}
