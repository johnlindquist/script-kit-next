/**
 * Session-based logger for OpenCode plugins
 * 
 * Writes logs to .opencode/logs/<session-id>.log
 * Each log line is JSON for easy parsing
 * 
 * Completely fail-safe - never throws, falls back to console
 */

let fs: typeof import("fs") | null = null
let path: typeof import("path") | null = null
let LOGS_DIR: string | null = null
let logsInitialized = false

// Lazy load fs/path to avoid module load errors
function getFs() {
  if (fs === null) {
    try {
      fs = require("fs")
      path = require("path")
      LOGS_DIR = path!.join(process.cwd(), ".opencode", "logs")
    } catch {
      // fs not available
    }
  }
  return fs
}

function ensureLogsDir(): boolean {
  if (logsInitialized) return true
  const fsModule = getFs()
  if (!fsModule || !LOGS_DIR) return false
  
  try {
    if (!fsModule.existsSync(LOGS_DIR)) {
      fsModule.mkdirSync(LOGS_DIR, { recursive: true })
    }
    logsInitialized = true
    return true
  } catch {
    return false
  }
}

export interface LogEntry {
  timestamp: string
  plugin: string
  hook: string
  sessionId: string
  triggered: boolean
  message: string
  details?: Record<string, unknown>
}

export function log(
  sessionId: string | null | undefined,
  plugin: string,
  hook: string,
  triggered: boolean,
  message: string,
  details?: Record<string, unknown>
): void {
  const safeSessionId = sessionId || "unknown-session"
  
  const entry: LogEntry = {
    timestamp: new Date().toISOString(),
    plugin,
    hook,
    sessionId: safeSessionId,
    triggered,
    message,
    ...(details && { details })
  }
  
  // Try to write to file, fall back to console silently
  try {
    if (ensureLogsDir() && fs && path && LOGS_DIR) {
      const logPath = path.join(LOGS_DIR, `${safeSessionId}.log`)
      fs.appendFileSync(logPath, JSON.stringify(entry) + "\n")
    }
  } catch {
    // Silent fallback - don't spam console
  }
}

/**
 * Helper to extract session ID from various input shapes
 */
export function extractSessionId(input: unknown): string | null {
  const obj = input as { 
    sessionID?: string
    session_id?: string
    properties?: { session_id?: string }
  }
  return obj?.sessionID || obj?.session_id || obj?.properties?.session_id || null
}

/**
 * Log a hook that was triggered (matched a scenario)
 */
export function logTriggered(
  sessionId: string | null | undefined,
  plugin: string,
  hook: string,
  message: string,
  details?: Record<string, unknown>
): void {
  log(sessionId, plugin, hook, true, message, details)
}

/**
 * Log a hook that ran but didn't match any scenario
 */
export function logSkipped(
  sessionId: string | null | undefined,
  plugin: string,
  hook: string,
  message: string,
  details?: Record<string, unknown>
): void {
  log(sessionId, plugin, hook, false, message, details)
}
