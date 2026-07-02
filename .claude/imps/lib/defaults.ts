export const DEFAULT_IMP_MODEL = "gpt-5.5";
export const DEFAULT_IMP_REASONING_EFFORT = "medium";
export const DEFAULT_IMP_READY_TIMEOUT_MS = 120_000;
export const DEFAULT_IMP_RPC_TIMEOUT_MS = 180_000;
export const DEFAULT_IMP_TURN_TIMEOUT_MS = 300_000;

export function envNumber(name: string, fallback: number): number {
  const raw = process.env[name];
  if (raw === undefined || raw.trim() === "") return fallback;
  const value = Number(raw);
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

export function parsePositiveMs(raw: string | undefined): number | undefined {
  if (!raw) return undefined;
  const value = Number(raw);
  return Number.isFinite(value) && value > 0 ? value : undefined;
}

export function impReadyTimeoutMs(): number {
  return envNumber("CODEX_IMP_READY_TIMEOUT_MS", DEFAULT_IMP_READY_TIMEOUT_MS);
}

export function impRpcTimeoutMs(): number {
  return envNumber("CODEX_IMP_START_TIMEOUT_MS", DEFAULT_IMP_RPC_TIMEOUT_MS);
}

export function impTurnTimeoutMs(): number {
  return envNumber("CODEX_IMP_TURN_TIMEOUT_MS", DEFAULT_IMP_TURN_TIMEOUT_MS);
}

// ── Hang guards ──────────────────────────────────────────────────────
// First-byte window for piped stdin: a backgrounded caller can hand the imp
// an open-but-silent stdin that never EOFs; without a bound the run parks
// forever before spawning anything.
export const DEFAULT_IMP_STDIN_FIRST_BYTE_MS = 2_000;
// Idle window between stdin chunks once data has started flowing.
export const DEFAULT_IMP_STDIN_IDLE_MS = 10_000;
// Client-side grace on top of the turn timeout when talking to a warm imp:
// the server guarantees an error/final within its turn timeout, so silence
// beyond turnTimeout + grace means the server is wedged or gone.
export const DEFAULT_IMP_CLIENT_GRACE_MS = 60_000;

export function impStdinFirstByteMs(): number {
  return envNumber("CODEX_IMP_STDIN_FIRST_BYTE_MS", DEFAULT_IMP_STDIN_FIRST_BYTE_MS);
}

export function impStdinIdleMs(): number {
  return envNumber("CODEX_IMP_STDIN_IDLE_MS", DEFAULT_IMP_STDIN_IDLE_MS);
}

export function impClientGraceMs(): number {
  return envNumber("CODEX_IMP_CLIENT_GRACE_MS", DEFAULT_IMP_CLIENT_GRACE_MS);
}
