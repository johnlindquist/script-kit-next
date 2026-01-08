import type { Plugin, Hooks } from "@opencode-ai/plugin"

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

// Type for tool execution input
interface ToolInput {
  tool: string
  sessionID: string
  callID: string
  args?: Record<string, unknown>
  result?: Record<string, unknown>
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

export const ScreenshotEnforcer: Plugin = async () => {
  const hooks: Hooks = {
    // Intercept bash commands before execution
    "tool.execute.before": async (input: ToolInput) => {
      if (input.tool !== "bash") return
      
      const args = input.args || {}
      const command = (args.command as string) || ""
      
      const { blocked, toolName } = detectBlockedScreenshotTool(command)
      
      if (blocked) {
        throw new Error(
          `BLOCKED: System screenshot tool "${toolName}" is not allowed.\n\n` +
          HELPFUL_ERROR
        )
      }
    },

    // Add screenshot policy to system prompt
    "experimental.chat.system.transform": async (_input, output) => {
      output.system.push(SYSTEM_PROMPT_REMINDER)
    },

    // Preserve policy during session compaction
    "experimental.session.compacting": async (_input, output) => {
      output.context.push(
        "<screenshot-policy>Use captureScreenshot() SDK, not system tools (screencapture, scrot, etc.)</screenshot-policy>"
      )
    }
  }
  
  return hooks
}
