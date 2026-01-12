import type { Plugin } from "@opencode-ai/plugin"
import { spawn } from "child_process"

/**
 * Cargo Format Check Plugin
 *
 * Automatically runs `cargo fmt --check` after any Rust file edit.
 * - If formatting passes: silent (no interruption)
 * - If formatting fails: sends diff output to agent for immediate fix
 *
 * This catches formatting issues instantly rather than waiting for
 * the verification gate at commit time.
 */

// =============================================================================
// Configuration
// =============================================================================

const RUST_FILE_PATTERN = /\.rs$/

// Timeout for cargo fmt (should be fast, <2 seconds typically)
const TIMEOUT_MS = 5000

// Debounce rapid edits to the same file
const DEBOUNCE_MS = 500

// Backoff for parallel work detection
const PARALLEL_WORK_WAIT_MS = 60_000 // 1 minute

interface PendingCheck {
  timeout: NodeJS.Timeout
  sessionId: string
}
const pendingChecks = new Map<string, PendingCheck>()

// Track which sessions have modified which files to detect parallel work
// Maps filePath -> Set of sessionIds that have modified it
const fileModifiedBy = new Map<string, Set<string>>()

// Track last modification time per file for staleness cleanup
const fileLastModified = new Map<string, number>()

// Consider a file "actively being worked on" within this window
const ACTIVE_WORK_WINDOW_MS = 5 * 60_000 // 5 minutes

function trackModifiedFile(sessionId: string, filePath: string): void {
  let sessions = fileModifiedBy.get(filePath)
  if (!sessions) {
    sessions = new Set()
    fileModifiedBy.set(filePath, sessions)
  }
  sessions.add(sessionId)
  fileLastModified.set(filePath, Date.now())
}

function isParallelWorkOnFile(filePath: string): boolean {
  const sessions = fileModifiedBy.get(filePath)
  const lastModified = fileLastModified.get(filePath)
  
  // If multiple sessions have touched this file recently, it's parallel work
  if (sessions && sessions.size > 1) {
    if (lastModified && Date.now() - lastModified < ACTIVE_WORK_WINDOW_MS) {
      return true
    }
  }
  return false
}

function cleanupStaleTracking(): void {
  const now = Date.now()
  for (const [filePath, lastModified] of fileLastModified.entries()) {
    if (now - lastModified > ACTIVE_WORK_WINDOW_MS) {
      fileModifiedBy.delete(filePath)
      fileLastModified.delete(filePath)
    }
  }
}

function sleep(ms: number): Promise<void> {
  return new Promise(resolve => setTimeout(resolve, ms))
}

// =============================================================================
// Helpers
// =============================================================================

interface FmtResult {
  success: boolean
  output: string
  filePath: string
}

async function runCargoFmtCheck(filePath: string): Promise<FmtResult> {
  return new Promise((resolve) => {
    const startTime = Date.now()
    let stdout = ""
    let stderr = ""
    let resolved = false

    const proc = spawn("cargo", ["fmt", "--check", "--", filePath], {
      cwd: process.cwd(),
      timeout: TIMEOUT_MS,
    })

    proc.stdout?.on("data", (data) => {
      stdout += data.toString()
    })

    proc.stderr?.on("data", (data) => {
      stderr += data.toString()
    })

    proc.on("close", (code) => {
      if (resolved) return
      resolved = true
      
      const duration = Date.now() - startTime
      const output = (stdout + stderr).trim()
      
      if (code === 0) {
        console.log(`[CargoFmtCheck] ✓ ${filePath} (${duration}ms)`)
        resolve({ success: true, output: "", filePath })
      } else {
        console.log(`[CargoFmtCheck] ✗ ${filePath} needs formatting (${duration}ms)`)
        resolve({ success: false, output, filePath })
      }
    })

    proc.on("error", (err) => {
      if (resolved) return
      resolved = true
      console.log(`[CargoFmtCheck] Error running cargo fmt: ${err.message}`)
      // Don't fail on errors (cargo might not be available)
      resolve({ success: true, output: "", filePath })
    })

    // Timeout fallback
    setTimeout(() => {
      if (resolved) return
      resolved = true
      proc.kill()
      console.log(`[CargoFmtCheck] Timeout checking ${filePath}`)
      resolve({ success: true, output: "", filePath })
    }, TIMEOUT_MS)
  })
}

// =============================================================================
// Plugin Export
// =============================================================================

interface ToolInput {
  tool: string
  sessionID: string
  callID: string
  args?: Record<string, unknown>
  result?: Record<string, unknown>
}

const CargoFmtCheck: Plugin = async ({ client }) => {
  // Periodically clean up stale file tracking
  setInterval(cleanupStaleTracking, ACTIVE_WORK_WINDOW_MS)

  return {
    // Clean up session from file tracking on session end
    event: async ({ event }) => {
      const eventWithSession = event as { session_id?: string; sessionID?: string }
      const sessionId = eventWithSession.session_id || eventWithSession.sessionID
      
      if (event.type === "session.deleted" && sessionId) {
        // Remove this session from all file tracking
        for (const sessions of fileModifiedBy.values()) {
          sessions.delete(sessionId)
        }
      }
    },

    "tool.execute.after": async (input: ToolInput) => {
      const tool = input.tool
      const args = input.args || {}
      const sessionId = input.sessionID

      // Only check after edit/write operations
      if (tool !== "edit" && tool !== "write") return

      // Need session ID to send feedback
      if (!sessionId) return

      const filePath = (args.filePath as string) || ""

      // Only check Rust files
      if (!RUST_FILE_PATTERN.test(filePath)) return

      // Track that this session modified this file
      trackModifiedFile(sessionId, filePath)

      // Debounce: cancel any pending check for this file
      const existing = pendingChecks.get(filePath)
      if (existing) {
        clearTimeout(existing.timeout)
      }

      // Schedule the check with debounce (capture sessionId for this edit)
      const timeout = setTimeout(async () => {
        const pending = pendingChecks.get(filePath)
        const targetSessionId = pending?.sessionId || sessionId
        pendingChecks.delete(filePath)

        let result = await runCargoFmtCheck(filePath)

        if (!result.success && result.output) {
          // Check if multiple sessions are working on this file (parallel work)
          const hasParallelWork = isParallelWorkOnFile(filePath)
          
          if (hasParallelWork) {
            // Parallel work detected - wait and retry
            const sessions = fileModifiedBy.get(filePath)
            console.log(`[CargoFmtCheck] Format error on ${filePath} - ${sessions?.size || 0} sessions working on this file, waiting ${PARALLEL_WORK_WAIT_MS / 1000}s...`)
            await sleep(PARALLEL_WORK_WAIT_MS)
            
            // Retry after waiting
            result = await runCargoFmtCheck(filePath)
            
            if (result.success) {
              console.log(`[CargoFmtCheck] ✓ ${filePath} now passes (parallel work completed)`)
              return
            }
            
            // Still failing after wait
            console.log(`[CargoFmtCheck] ${filePath} still has format issues after wait`)
            
            // Notify but with softer messaging about parallel work
            const fileName = filePath.split("/").pop() || filePath
            const message = `## Note: Formatting Issue in \`${fileName}\`

\`cargo fmt --check\` found formatting issues. Multiple agents/users appear to be working on this file.

The system waited 1 minute but the issue persists. This may be from another agent's in-progress work.

**Options:**
1. If you need this file now, run \`cargo fmt -- ${filePath}\`
2. If unrelated to your task, you can safely ignore this
3. Wait for the other agent to complete their work

\`\`\`diff
${result.output.slice(0, 1000)}${result.output.length > 1000 ? "\n... (truncated)" : ""}
\`\`\``

            await client.session.prompt({
              path: { id: targetSessionId },
              body: {
                parts: [{ type: "text", text: message }],
              },
            })
            return
          }

          // Single session working on this file - prompt immediately
          const fileName = filePath.split("/").pop() || filePath
          const message = `## Formatting Issue Detected

\`cargo fmt --check\` found formatting issues in \`${fileName}\`:

\`\`\`diff
${result.output.slice(0, 2000)}${result.output.length > 2000 ? "\n... (truncated)" : ""}
\`\`\`

Please run \`cargo fmt\` on this file or apply the suggested formatting changes before continuing.

You can fix this by running:
\`\`\`bash
cargo fmt -- ${filePath}
\`\`\`

Or manually apply the diff shown above.`

          await client.session.prompt({
            path: { id: targetSessionId },
            body: {
              parts: [{ type: "text", text: message }],
            },
          })
        }
      }, DEBOUNCE_MS)

      pendingChecks.set(filePath, { timeout, sessionId })
    },
  }
}

export default CargoFmtCheck
