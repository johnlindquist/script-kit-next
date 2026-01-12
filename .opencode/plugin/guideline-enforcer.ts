import type { Plugin } from "@opencode-ai/plugin"
import { logTriggered, logSkipped, extractSessionId } from "../lib/logger"

const PLUGIN_NAME = "guideline-enforcer"

/**
 * Guideline Enforcer Plugin
 * 
 * Enforces guidelines through system prompt injection and tool hooks.
 * Does NOT use client.session.prompt() to avoid interfering with subagent sessions.
 * 
 * Guidelines enforced:
 * 1. Visual Testing: If user provided an image, agent must capture AND read a screenshot
 * 2. Verification Gate: If code was changed, agent must run cargo check/clippy/test
 * 3. Stdin Protocol: If tests were run, must use stdin JSON protocol
 * 
 * IMPORTANT: This plugin uses passive enforcement via system prompts and tool hooks.
 * It does NOT actively inject prompts on session.idle because that interferes with
 * subagent sessions spawned by the Task tool.
 */

// =============================================================================
// Session State Tracking
// =============================================================================

interface SessionState {
  // Visual testing tracking
  userProvidedImage: boolean
  screenshotCaptured: boolean
  screenshotFileRead: boolean
  
  // Verification gate tracking
  codeFilesModified: boolean
  cargoCheckRan: boolean
  cargoClippyRan: boolean
  cargoTestRan: boolean
  verificationPassed: boolean
  
  // Stdin protocol tracking  
  appTestAttempted: boolean
  stdinProtocolUsed: boolean
  
  // Git tracking
  commitAttempted: boolean
}

function createSessionState(): SessionState {
  return {
    userProvidedImage: false,
    screenshotCaptured: false,
    screenshotFileRead: false,
    
    codeFilesModified: false,
    cargoCheckRan: false,
    cargoClippyRan: false,
    cargoTestRan: false,
    verificationPassed: false,
    
    appTestAttempted: false,
    stdinProtocolUsed: false,
    
    commitAttempted: false,
  }
}

// Global state - reset on plugin load
let state = createSessionState()

// =============================================================================
// Detection Patterns
// =============================================================================

const CODE_FILE_PATTERNS = [
  /\.rs$/,
  /\.ts$/,
  /\.tsx$/,
  /\.js$/,
  /\.jsx$/,
  /Cargo\.toml$/,
]

const STDIN_PROTOCOL_PATTERN = /echo\s+['"]\{.*"type"\s*:\s*"run".*\}['"]\s*\|\s*.*script-kit-gpui/

const SCREENSHOT_DIR_PATTERN = /test-screenshots\//

// =============================================================================
// Compliance Checkers
// =============================================================================

interface ComplianceResult {
  compliant: boolean
  violations: string[]
  requiredActions: string[]
}

function checkCompliance(): ComplianceResult {
  const violations: string[] = []
  const requiredActions: string[] = []
  
  // Rule 1: Visual Testing - If image provided, must capture AND read screenshot
  if (state.userProvidedImage) {
    if (!state.screenshotCaptured) {
      violations.push("User provided an image but no screenshot was captured")
      requiredActions.push("Use captureScreenshot() SDK function to capture the current UI state")
    }
    if (state.screenshotCaptured && !state.screenshotFileRead) {
      violations.push("Screenshot was captured but the PNG file was never READ")
      requiredActions.push("Use the Read tool to read the screenshot file from ./test-screenshots/ and analyze it")
    }
  }
  
  // Rule 2: Verification Gate - If code modified, must run verification before commit
  if (state.codeFilesModified && state.commitAttempted) {
    if (!state.cargoCheckRan || !state.cargoClippyRan || !state.cargoTestRan) {
      const missing: string[] = []
      if (!state.cargoCheckRan) missing.push("cargo check")
      if (!state.cargoClippyRan) missing.push("cargo clippy")
      if (!state.cargoTestRan) missing.push("cargo test")
      
      violations.push(`Code was modified and commit attempted but verification incomplete: missing ${missing.join(", ")}`)
      requiredActions.push(`Run: cargo check && cargo clippy --all-targets -- -D warnings && cargo test`)
    }
  }
  
  // Rule 3: Stdin Protocol - If app test attempted, must use stdin protocol
  if (state.appTestAttempted && !state.stdinProtocolUsed) {
    violations.push("App test was attempted but stdin JSON protocol was not used")
    requiredActions.push('Run tests using: echo \'{"type":"run","path":"..."}\' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1')
  }
  
  return {
    compliant: violations.length === 0,
    violations,
    requiredActions
  }
}

function generateComplianceStatus(): string {
  const result = checkCompliance()
  
  if (result.compliant) {
    return "All guidelines followed."
  }
  
  return `
## ⚠️ Guideline Violations Detected

The following guidelines have NOT been followed:

${result.violations.map((v, i) => `${i + 1}. **${v}**`).join("\n")}

### Required Actions:

${result.requiredActions.map((a, i) => `${i + 1}. ${a}`).join("\n")}

**You should complete these actions before finishing your task.**
`.trim()
}

// Type for tool execution input
interface ToolInput {
  tool: string
  sessionID: string
  callID: string
  args?: Record<string, unknown>
  result?: Record<string, unknown>
}

// =============================================================================
// Plugin Export
// =============================================================================

const GuidelineEnforcer: Plugin = async () => {
  // Reset state on plugin load
  state = createSessionState()
  
  return {
    // Track session creation and detect images in messages
    // NOTE: We do NOT call client.session.prompt() on session.idle 
    // because that interferes with Task tool subagent responses
    event: async ({ event }) => {
      const eventWithSession = event as { session_id?: string; sessionID?: string }
      const sessionId = eventWithSession.session_id || eventWithSession.sessionID
      
      // Reset state on new session
      if (event.type === "session.created") {
        state = createSessionState()
        logTriggered(sessionId, PLUGIN_NAME, "event", "Session created - reset guideline state")
        return
      }
      
      // Detect images in user messages
      if (event.type === "message.updated") {
        const props = (event as { properties?: { message?: { role?: string; content?: unknown } } }).properties
        const message = props?.message
        if (message?.role === "user") {
          const content = JSON.stringify(message.content || "")
          if (content.includes("image/") || content.includes("data:image") || 
              /\.(png|jpg|jpeg|gif|webp)/i.test(content)) {
            state.userProvidedImage = true
            logTriggered(sessionId, PLUGIN_NAME, "event", "Image detected in user message - visual testing required")
          } else {
            logSkipped(sessionId, PLUGIN_NAME, "event", "User message without image")
          }
        }
        return
      }
      
      // Just log compliance status on session.idle - do NOT inject prompts
      if (event.type === "session.idle") {
        const result = checkCompliance()
        if (!result.compliant) {
          logTriggered(sessionId, PLUGIN_NAME, "event", "Session idle with violations", { violations: result.violations })
        } else {
          logSkipped(sessionId, PLUGIN_NAME, "event", "Session idle - all guidelines followed")
        }
        return
      }
      
    },
    
    // Track tool executions using the documented hook
    "tool.execute.after": async (input: ToolInput) => {
      const sessionId = input.sessionID
      const tool = input.tool
      const args = input.args || {}
      const result = input.result || {}
      
      const trackedActions: string[] = []
      
      // Track file modifications
      if (tool === "edit" || tool === "write") {
        const filePath = (args.filePath as string) || ""
        if (CODE_FILE_PATTERNS.some(pattern => pattern.test(filePath))) {
          state.codeFilesModified = true
          trackedActions.push(`code file modified: ${filePath}`)
        }
      }
      
      // Track bash commands
      if (tool === "bash") {
        const command = (args.command as string) || ""
        const output = (result.output as string) || ""
        
        // Track verification commands
        if (/cargo\s+check/.test(command)) {
          state.cargoCheckRan = true
          trackedActions.push("cargo check ran")
        }
        if (/cargo\s+clippy/.test(command)) {
          state.cargoClippyRan = true
          trackedActions.push("cargo clippy ran")
        }
        if (/cargo\s+test/.test(command)) {
          state.cargoTestRan = true
          trackedActions.push("cargo test ran")
        }
        
        // Track git commits
        if (/git\s+commit/.test(command)) {
          state.commitAttempted = true
          trackedActions.push("git commit attempted")
        }
        
        // Track stdin protocol usage
        if (STDIN_PROTOCOL_PATTERN.test(command)) {
          state.stdinProtocolUsed = true
          state.appTestAttempted = true
          trackedActions.push("stdin protocol used for app test")
        }
        
        // Track app launch attempts (without stdin = wrong)
        if (/\.\/target\/.*script-kit-gpui/.test(command) && !STDIN_PROTOCOL_PATTERN.test(command)) {
          state.appTestAttempted = true
          trackedActions.push("app launched without stdin protocol (violation)")
        }
        
        // Track screenshot capture in test output
        if (output.includes("captureScreenshot") || output.includes("[SCREENSHOT]")) {
          state.screenshotCaptured = true
          trackedActions.push("screenshot captured")
        }
      }
      
      // Track file reads for screenshot verification
      if (tool === "read") {
        const filePath = (args.filePath as string) || ""
        if (SCREENSHOT_DIR_PATTERN.test(filePath) && /\.png$/i.test(filePath)) {
          state.screenshotFileRead = true
          trackedActions.push(`screenshot file read: ${filePath}`)
        }
      }
      
      if (trackedActions.length > 0) {
        logTriggered(sessionId, PLUGIN_NAME, "tool.execute.after", `Tracked: ${trackedActions.join(", ")}`, { tool })
      } else {
        logSkipped(sessionId, PLUGIN_NAME, "tool.execute.after", `No guideline-related actions in ${tool} call`)
      }
    },
    
    // Add enforcement status to system prompt
    "experimental.chat.system.transform": async (input, output) => {
      const sessionId = extractSessionId(input)
      output.system.push(`
<guideline-enforcer>
This session is monitored for guideline compliance:

1. **Visual Testing**: If user provided an image, you MUST capture a screenshot AND read the PNG file
2. **Verification Gate**: If you modified code and commit, you MUST run cargo check/clippy/test first
3. **Stdin Protocol**: If you test the app, you MUST use: echo '{"type":"run",...}' | ./target/debug/script-kit-gpui

Complete all required actions before finishing your task.
</guideline-enforcer>
`.trim())
      logTriggered(sessionId, PLUGIN_NAME, "system.transform", "Injected guideline enforcement policy")
    },
    
    // Preserve state through compaction
    "experimental.session.compacting": async (input, output) => {
      const sessionId = extractSessionId(input)
      const complianceStatus = generateComplianceStatus()
      output.context.push(`<guideline-state>
${complianceStatus}

State: ${JSON.stringify({
        userProvidedImage: state.userProvidedImage,
        screenshotCaptured: state.screenshotCaptured,
        screenshotFileRead: state.screenshotFileRead,
        codeFilesModified: state.codeFilesModified,
        verificationComplete: state.cargoCheckRan && state.cargoClippyRan && state.cargoTestRan,
      })}
</guideline-state>`)
      logTriggered(sessionId, PLUGIN_NAME, "session.compacting", "Preserved guideline state in compaction context", {
        compliant: checkCompliance().compliant
      })
    }
  }
}

export default GuidelineEnforcer
