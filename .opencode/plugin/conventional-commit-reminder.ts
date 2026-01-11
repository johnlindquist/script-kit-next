import type { Plugin } from "@opencode-ai/plugin"

/**
 * Conventional Commit Reminder Plugin
 * 
 * Triggers on session stop to remind the agent to commit with a conventional
 * commit message if:
 * 1. Code files were modified during the session
 * 2. No commit was made yet
 * 
 * Conventional commit format: <type>(<scope>): <description>
 * Types: feat, fix, docs, style, refactor, perf, test, chore, build, ci
 */

// =============================================================================
// Session State Tracking
// =============================================================================

interface SessionState {
  codeFilesModified: boolean
  commitMade: boolean
  modifiedFiles: string[]
}

function createSessionState(): SessionState {
  return {
    codeFilesModified: false,
    commitMade: false,
    modifiedFiles: [],
  }
}

// Session-keyed state to support parallel sessions
const sessions = new Map<string, SessionState>()

function getState(sessionId: string): SessionState {
  let state = sessions.get(sessionId)
  if (!state) {
    state = createSessionState()
    sessions.set(sessionId, state)
  }
  return state
}

function clearState(sessionId: string): void {
  sessions.delete(sessionId)
}

// Helper to extract session ID from various input shapes
function extractSessionId(input: unknown): string | null {
  const obj = input as { sessionID?: string; session_id?: string }
  return obj?.sessionID || obj?.session_id || null
}

// =============================================================================
// Detection Patterns
// =============================================================================

const CODE_FILE_PATTERNS = [
  /\.rs$/,
  /\.ts$/,
  /\.tsx$/,
  /\.js$/,
  /\.jsx$/,
  /\.json$/,
  /\.toml$/,
  /\.md$/,
  /\.css$/,
  /\.scss$/,
  /\.html$/,
]

const COMMIT_PATTERNS = [
  /git\s+commit/i,
]

// =============================================================================
// Conventional Commit Types
// =============================================================================

const CONVENTIONAL_COMMIT_TYPES = `
**Conventional Commit Types:**
- \`feat\`: A new feature
- \`fix\`: A bug fix
- \`docs\`: Documentation only changes
- \`style\`: Changes that don't affect code meaning (whitespace, formatting)
- \`refactor\`: Code change that neither fixes a bug nor adds a feature
- \`perf\`: Performance improvement
- \`test\`: Adding or correcting tests
- \`chore\`: Changes to build process or auxiliary tools
- \`build\`: Changes affecting build system or dependencies
- \`ci\`: Changes to CI configuration files and scripts
`.trim()

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

export const ConventionalCommitReminder: Plugin = async ({ client }) => {
  return {
    // Track session events
    event: async ({ event }) => {
      const eventWithSession = event as { session_id?: string; sessionID?: string }
      const sessionId = eventWithSession.session_id || eventWithSession.sessionID

      if (event.type === "session.created" && sessionId) {
        sessions.set(sessionId, createSessionState())
      }
      
      if (event.type === "session.deleted" && sessionId) {
        clearState(sessionId)
      }
    },
    
    // Track tool executions
    "tool.execute.after": async (input: ToolInput) => {
      const tool = input.tool
      const args = input.args || {}
      const sessionId = input.sessionID
      
      if (!sessionId) return
      
      const state = getState(sessionId)
      
      // Track file modifications
      if (tool === "edit" || tool === "write") {
        const filePath = (args.filePath as string) || ""
        if (CODE_FILE_PATTERNS.some(pattern => pattern.test(filePath))) {
          state.codeFilesModified = true
          if (!state.modifiedFiles.includes(filePath)) {
            state.modifiedFiles.push(filePath)
          }
        }
      }
      
      // Track bash commands for commits
      if (tool === "bash") {
        const command = (args.command as string) || ""
        
        // Track git commits
        if (COMMIT_PATTERNS.some(pattern => pattern.test(command))) {
          state.commitMade = true
        }
      }
    },
    
    // Hook into session stop - prompt for conventional commit if needed
    stop: async (input) => {
      const sessionId = extractSessionId(input)
      if (!sessionId) return
      
      const state = getState(sessionId)
      
      // Only prompt if files were modified but no commit was made
      if (state.codeFilesModified && !state.commitMade && state.modifiedFiles.length > 0) {
        const fileList = state.modifiedFiles
          .slice(0, 10) // Limit to first 10 files
          .map(f => `  - ${f}`)
          .join("\n")
        
        const moreFiles = state.modifiedFiles.length > 10 
          ? `\n  ... and ${state.modifiedFiles.length - 10} more files`
          : ""
        
        const message = `## Uncommitted Changes Detected

You modified files during this session but haven't committed them yet.

**Modified files:**
${fileList}${moreFiles}

Please create a **conventional commit** for these changes before ending the session.

${CONVENTIONAL_COMMIT_TYPES}

**Format:** \`<type>(<scope>): <short description>\`

**Examples:**
- \`feat(theme): add vibrancy support for list items\`
- \`fix(executor): handle script timeout gracefully\`
- \`refactor(prompts): extract common input handling logic\`
- \`docs(agents): update testing protocol section\`

**Steps:**
1. Run \`git status\` to review changes
2. Run \`git diff\` to see what changed (if needed)
3. Stage the changes: \`git add -A\` (or selectively add files)
4. Commit with conventional format: \`git commit -m "<type>(<scope>): <description>"\`

Please commit now with an appropriate conventional commit message.`

        // Target the specific session
        await client.session.prompt({
          path: { id: sessionId },
          body: {
            parts: [{ type: "text", text: message }],
          },
        })
      }
    },
    
    // Add reminder to system prompt about conventional commits
    "experimental.chat.system.transform": async (input, output) => {
      const sessionId = extractSessionId(input)
      if (!sessionId) return
      
      const state = getState(sessionId)
      const hasUncommittedChanges = state.codeFilesModified && !state.commitMade
      
      if (hasUncommittedChanges) {
        output.system.push(`
<conventional-commit-reminder>
You have uncommitted changes. When you're done with your task, commit using conventional commit format:
\`<type>(<scope>): <description>\`

Types: feat, fix, docs, style, refactor, perf, test, chore, build, ci
</conventional-commit-reminder>
`.trim())
      }
    },
    
    // Preserve state through compaction
    "experimental.session.compacting": async (input, output) => {
      const sessionId = extractSessionId(input)
      if (!sessionId) return
      
      const state = getState(sessionId)
      
      if (state.codeFilesModified && !state.commitMade) {
        output.context.push(`<commit-state>
Uncommitted changes detected. Modified ${state.modifiedFiles.length} file(s).
Remember to commit with conventional commit format before ending session.
</commit-state>`)
      }
    }
  }
}
