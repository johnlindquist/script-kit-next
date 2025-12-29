import type { Plugin } from "@opencode-ai/plugin"

/**
 * Verification Gate Plugin
 * 
 * Enforces quality gates before commits by:
 * 1. Detecting git commit attempts in tool calls
 * 2. Injecting reminder to run verification commands first
 * 3. Tracking whether verification was run in the session
 */

const VERIFICATION_COMMANDS = `cargo check && cargo clippy --all-targets -- -D warnings && cargo test`

const PRE_COMMIT_REMINDER = `
<verification-gate>
## MANDATORY: Pre-Commit Verification

Before ANY git commit, you MUST run:
\`\`\`bash
${VERIFICATION_COMMANDS}
\`\`\`

**Commit is BLOCKED until all three pass:**
1. \`cargo check\` - Type errors, borrow checker
2. \`cargo clippy\` - Lints, anti-patterns (treat warnings as errors)
3. \`cargo test\` - Unit + integration tests

**If any fail:** Fix the issues first, then retry verification.

**Evidence required:** Show the passing output in your response before committing.
</verification-gate>
`.trim()

const COMMIT_PATTERNS = [
  /git\s+commit/i,
  /git\s+add\s+.*&&\s*git\s+commit/i,
  /"command"\s*:\s*"[^"]*git\s+commit/i,
]

const VERIFICATION_PATTERNS = [
  /cargo\s+check/,
  /cargo\s+clippy/,
  /cargo\s+test/,
]

// Session state to track if verification was run
let verificationRanInSession = false
let lastVerificationTime = 0
const VERIFICATION_EXPIRY_MS = 5 * 60 * 1000 // 5 minutes

function isVerificationStale(): boolean {
  return Date.now() - lastVerificationTime > VERIFICATION_EXPIRY_MS
}

function containsCommitAttempt(text: string): boolean {
  return COMMIT_PATTERNS.some(pattern => pattern.test(text))
}

function containsVerificationCommand(text: string): boolean {
  // Must have all three verification commands
  return VERIFICATION_PATTERNS.every(pattern => pattern.test(text))
}

export const VerificationGate: Plugin = async () => {
  return {
    // Track when verification commands are run
    "tool.bash.post": async (input, _output) => {
      const command = input.params?.command || ""
      
      // Check if this was a verification command
      if (containsVerificationCommand(command)) {
        // Only mark as verified if command succeeded (no error in result)
        const result = input.result?.output || ""
        const hasError = /error\[E\d+\]|^error:/im.test(result) ||
                        /warning:.*\n.*= help:/m.test(result) && /--\s*-D\s*warnings/.test(command)
        
        if (!hasError) {
          verificationRanInSession = true
          lastVerificationTime = Date.now()
        }
      }
    },

    // Intercept commit attempts and inject reminder if verification not done
    "tool.bash.pre": async (input, output) => {
      const command = input.params?.command || ""
      
      if (containsCommitAttempt(command)) {
        // Check if verification was run recently
        if (!verificationRanInSession || isVerificationStale()) {
          // Inject a strong reminder
          output.messages = output.messages || []
          output.messages.push({
            role: "user",
            content: PRE_COMMIT_REMINDER + "\n\n**WARNING**: No recent verification detected. Run the verification commands before committing!"
          })
        }
      }
    },

    // Add verification reminder to system prompt
    "experimental.chat.system.transform": async (_input, output) => {
      const reminder = `
<verification-gate-policy>
Before any git commit in this Rust project, ALWAYS run:
\`${VERIFICATION_COMMANDS}\`
All three must pass. Show evidence of passing before committing.
</verification-gate-policy>
`.trim()
      
      output.system.push(reminder)
    },

    // Preserve verification state through compaction
    "experimental.session.compacting": async (_input, output) => {
      const state = verificationRanInSession && !isVerificationStale()
        ? "Verification gate: PASSED (recent)"
        : "Verification gate: PENDING - run cargo check && cargo clippy && cargo test before commit"
      
      output.context.push(`<verification-state>${state}</verification-state>`)
    }
  }
}
