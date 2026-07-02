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
