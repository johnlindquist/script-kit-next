import type { Plugin, Hooks } from "@opencode-ai/plugin"
import { logTriggered, logSkipped, extractSessionId } from "../lib/logger"

const PLUGIN_NAME = "screenshot-enforcer"

/**
 * Screenshot Enforcer Plugin
 * 
 * Prevents agents from using system screenshot tools and enforces
 * the built-in captureScreenshot() SDK function.
 * 
 * Blocked tools:
 * - macOS: screencapture
 * - Linux: scrot, gnome-screenshot, flameshot, maim
 * - Cross-platform: ImageMagick's import command
 * 
 * The captureScreenshot() SDK function captures only the app window,
 * providing consistent screenshots for visual testing workflows.
 */

const BLOCKED_SCREENSHOT_PATTERNS = [
  // macOS
  { pattern: /\bscreencapture\b/, name: "screencapture (macOS)" },
  // Linux  
  { pattern: /\bscrot\b/, name: "scrot (Linux)" },
  { pattern: /\bgnome-screenshot\b/, name: "gnome-screenshot (Linux)" },
  { pattern: /\bflameshot\b/, name: "flameshot (Linux)" },
  { pattern: /\bmaim\b/, name: "maim (Linux)" },
  // ImageMagick (cross-platform)
  { pattern: /\bimport\s+(-\w+\s+)*-window\b/, name: "import -window (ImageMagick)" },
  { pattern: /\bimport\s+(-\w+\s+)*root:/, name: "import root: (ImageMagick)" },
]

const HELPFUL_ERROR = `
USE THIS INSTEAD:

  const screenshot = await captureScreenshot();
  // Returns { width, height, data } where data is base64 PNG
  
  // To save to file:
  import { writeFileSync, mkdirSync } from 'fs';
  import { join } from 'path';
  
  const screenshotDir = join(process.cwd(), 'test-screenshots');
  mkdirSync(screenshotDir, { recursive: true });
  const filepath = join(screenshotDir, 'screenshot-' + Date.now() + '.png');
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));

The captureScreenshot() SDK function:
- Captures only the Script Kit app window (not desktop)
- Returns consistent dimensions for visual testing
- Works cross-platform without external tools
`.trim()

const SYSTEM_PROMPT_REMINDER = `
<screenshot-tool-policy>
## Screenshot Policy
DO NOT use system screenshot tools (screencapture, scrot, gnome-screenshot, etc.).
USE the built-in captureScreenshot() SDK function instead:

  const screenshot = await captureScreenshot();
  writeFileSync('screenshot.png', Buffer.from(screenshot.data, 'base64'));

This captures only the app window and provides consistent results.
</screenshot-tool-policy>
`.trim()

// Type definitions for hook inputs/outputs matching @opencode-ai/plugin types
interface ToolExecuteBeforeInput {
  tool: string
  sessionID: string
  callID: string
}

interface ToolExecuteBeforeOutput {
  args: Record<string, unknown>
}

/**
 * Checks if a bash command attempts to use a system screenshot tool
 */
function detectBlockedScreenshotTool(command: string): { blocked: boolean; toolName?: string } {
  for (const { pattern, name } of BLOCKED_SCREENSHOT_PATTERNS) {
    if (pattern.test(command)) {
      return { blocked: true, toolName: name }
    }
  }
  return { blocked: false }
}

const ScreenshotEnforcer: Plugin = async () => {
  const hooks: Hooks = {
    // Intercept bash commands before execution
    "tool.execute.before": async (input: ToolExecuteBeforeInput, output: ToolExecuteBeforeOutput) => {
      const sessionId = input.sessionID
      
      if (input.tool !== "bash") {
        logSkipped(sessionId, PLUGIN_NAME, "tool.execute.before", `Skipped non-bash tool: ${input.tool}`)
        return
      }
      
      // For tool.execute.before, args are in the output object
      const args = output.args || {}
      const command = (args.command as string) || ""
      
      const { blocked, toolName } = detectBlockedScreenshotTool(command)
      
      if (blocked) {
        logTriggered(sessionId, PLUGIN_NAME, "tool.execute.before", `BLOCKED screenshot tool: ${toolName}`, { command: command.slice(0, 100) })
        throw new Error(
          `BLOCKED: System screenshot tool "${toolName}" is not allowed.\n\n` +
          HELPFUL_ERROR
        )
      }
      
      logSkipped(sessionId, PLUGIN_NAME, "tool.execute.before", "Bash command allowed (no blocked screenshot tools)", { command: command.slice(0, 100) })
    },

    // Add screenshot policy to system prompt
    "experimental.chat.system.transform": async (input, output) => {
      const sessionId = extractSessionId(input)
      output.system.push(SYSTEM_PROMPT_REMINDER)
      logTriggered(sessionId, PLUGIN_NAME, "system.transform", "Injected screenshot policy into system prompt")
    },

    // Preserve policy during session compaction
    "experimental.session.compacting": async (input, output) => {
      const sessionId = extractSessionId(input)
      output.context.push(
        "<screenshot-policy>Use captureScreenshot() SDK, not system tools (screencapture, scrot, etc.)</screenshot-policy>"
      )
      logTriggered(sessionId, PLUGIN_NAME, "session.compacting", "Preserved screenshot policy in compaction context")
    }
  }
  
  return hooks
}

export default ScreenshotEnforcer
