import type { Plugin } from "@opencode-ai/plugin"

/**
 * Visual Testing Reminder Plugin
 *
 * Reminds the agent to use visual testing if UI changes were made
 * during the session but no screenshot verification was performed.
 */

// =============================================================================
// Session State
// =============================================================================

interface SessionState {
  uiFilesModified: boolean
  screenshotCaptured: boolean
  modifiedUiFiles: string[]
}

function createSessionState(): SessionState {
  return {
    uiFilesModified: false,
    screenshotCaptured: false,
    modifiedUiFiles: [],
  }
}

// Session-keyed state
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

function extractSessionId(input: unknown): string | null {
  const obj = input as { sessionID?: string; session_id?: string }
  return obj?.sessionID || obj?.session_id || null
}

// =============================================================================
// Detection Patterns
// =============================================================================

// Files that likely contain UI code
const UI_FILE_PATTERNS = [
  /\.rs$/, // Rust files (GPUI components)
  /theme\.rs$/,
  /prompts?\.rs$/,
  /list.*\.rs$/,
  /panel\.rs$/,
  /window\.rs$/,
  /render/i,
  /component/i,
  /view/i,
  /\.tsx$/,
  /\.jsx$/,
  /\.css$/,
  /\.scss$/,
  /styles/i,
]

// Keywords in file paths that suggest UI work
const UI_PATH_KEYWORDS = [
  "theme",
  "color",
  "style",
  "layout",
  "render",
  "view",
  "component",
  "prompt",
  "panel",
  "window",
  "list",
  "button",
  "input",
  "ui",
  "visual",
]

function isUiFile(filePath: string): boolean {
  const lowerPath = filePath.toLowerCase()
  
  // Check patterns
  if (UI_FILE_PATTERNS.some(pattern => pattern.test(filePath))) {
    // Further check if it contains UI keywords
    if (UI_PATH_KEYWORDS.some(keyword => lowerPath.includes(keyword))) {
      return true
    }
  }
  
  // Direct keyword match in path
  return UI_PATH_KEYWORDS.some(keyword => lowerPath.includes(keyword))
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

const VisualTestingReminder: Plugin = async ({ client }) => {
  return {
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

    "tool.execute.after": async (input: ToolInput) => {
      const tool = input.tool
      const args = input.args || {}
      const result = input.result || {}
      const sessionId = input.sessionID

      if (!sessionId) return

      const state = getState(sessionId)

      // Track UI file modifications
      if (tool === "edit" || tool === "write") {
        const filePath = (args.filePath as string) || ""
        if (isUiFile(filePath)) {
          state.uiFilesModified = true
          if (!state.modifiedUiFiles.includes(filePath)) {
            state.modifiedUiFiles.push(filePath)
          }
        }
      }

      // Track screenshot captures (via bash output or specific patterns)
      if (tool === "bash") {
        const command = (args.command as string) || ""
        const output = (result.output as string) || ""

        if (
          command.includes("captureScreenshot") ||
          output.includes("[SCREENSHOT]") ||
          output.includes("test-screenshots/") ||
          /screenshot.*\.png/i.test(output)
        ) {
          state.screenshotCaptured = true
        }
      }

      // Track if Read tool was used on a screenshot
      if (tool === "read") {
        const filePath = (args.filePath as string) || ""
        if (/test-screenshots\/.*\.png$/i.test(filePath) || /screenshot.*\.png$/i.test(filePath)) {
          state.screenshotCaptured = true
        }
      }
    },

    stop: async (input) => {
      const sessionId = extractSessionId(input)
      if (!sessionId) return

      const state = getState(sessionId)

      // Only remind if UI files were modified but no screenshot was taken
      if (state.uiFilesModified && !state.screenshotCaptured) {
        const fileList = state.modifiedUiFiles
          .slice(0, 5)
          .map(f => `  - ${f.split("/").pop()}`)
          .join("\n")

        const moreFiles = state.modifiedUiFiles.length > 5
          ? `\n  ... and ${state.modifiedUiFiles.length - 5} more`
          : ""

        const message = `## Visual Testing Reminder

If any UI changes were made, please use visual testing to verify they're complete if you haven't already.

**UI-related files modified:**
${fileList}${moreFiles}

**To verify visually:**
1. Create a test script that renders the changed UI
2. Use \`captureScreenshot()\` to capture the result
3. Save to \`./test-screenshots/\`
4. Read the PNG file to verify it looks correct

Example:
\`\`\`typescript
const screenshot = await captureScreenshot();
writeFileSync('test-screenshots/my-change.png', Buffer.from(screenshot.data, 'base64'));
\`\`\`

If no visual changes were made, or you've already verified, you can proceed.`

        await client.session.prompt({
          path: { id: sessionId },
          body: {
            parts: [{ type: "text", text: message }],
          },
        })
      }
    },
  }
}

export default VisualTestingReminder
