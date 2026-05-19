#!/usr/bin/env bun

type JsonObject = Record<string, unknown>;

function rpcErrorCode(envelope: JsonObject): string | null {
  if (envelope.status !== "error") {
    return null;
  }
  const error = envelope.error;
  if (typeof error === "object" && error != null && "code" in error) {
    const code = (error as { code?: unknown }).code;
    return typeof code === "string" ? code : null;
  }
  return null;
}

/** Map session RPC transport failures to DevTools receipt classifications. */
export function classifyTransportError(envelope: JsonObject): string {
  const code = rpcErrorCode(envelope);
  if (!code) {
    return envelope.status === "error" ? "blocked-by-response-timeout" : "ok";
  }
  switch (code) {
    case "queue_timeout":
      return "blocked-by-session-queue";
    case "response_timeout":
    case "timeout":
      return "blocked-by-response-timeout";
    case "parse_error":
      return "blocked-by-parse-error";
    default:
      return `blocked-by-${code}`;
  }
}
