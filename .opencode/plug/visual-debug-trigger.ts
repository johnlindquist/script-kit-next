import type { Plugin, Hooks } from "@opencode-ai/plugin"
import type { Part } from "@opencode-ai/sdk"

/**
 * Visual Debug Trigger Plugin
 *
 * Detects UI/visual-related trigger words in user prompts and automatically
 * injects the visual debugging workflow instructions. This ensures agents
 * follow proper visual testing practices even when no image is provided.
 *
 * Trigger patterns:
 * - Layout: layout, spacing, padding, margin, alignment, position, overflow
 * - Sizing: size, width, height, resize, too big/small/wide/narrow/tall/short
 * - Visual: color, background, border, style, styling, theme, dark/light mode
 * - Visibility: visible, hidden, missing, not showing, doesn't appear, can't see
 * - Components + action: button/list/panel/etc. combined with broken/wrong/fix/etc.
 */

// Trigger patterns for visual/UI related terms
const TRIGGER_PATTERNS = {
  layout: /\b(layout|spacing|padding|margin|alignment|position|overflow)\b/i,
  sizing: /\b(size|width|height|resize|too (big|small|wide|narrow|tall|short))\b/i,
  visual: /\b(color|background|border|style|styling|theme|dark mode|light mode)\b/i,
  visibility: /\b(visible|hidden|missing|not showing|doesn't appear|can't see)\b/i,
  components: /\b(button|list|panel|dialog|input|header|scrollbar|window).*(broken|wrong|fix|change|update)\b/i,
}

const VISUAL_DEBUG_INSTRUCTIONS = `
<visual-debugging-triggered>
## MANDATORY: Visual Debugging Workflow

Your prompt contains UI/visual-related keywords. You MUST follow this visual debugging workflow:

### Step 1: Build First
\`\`\`bash
cargo build
\`\`\`

### Step 2: Create Test Script
Write a test script in \`tests/smoke/\` that:
- Sets up the UI state relevant to the visual issue
- Calls \`captureScreenshot()\` after the UI renders
- Saves the PNG to \`./.test-screenshots/\`

Example template:
\`\`\`typescript
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Set up UI state (arg, div, editor, etc.)
await div(\`<div class="...">Your content</div>\`);
await new Promise(resolve => setTimeout(resolve, 500));

// Capture screenshot
const screenshot = await captureScreenshot();
console.error(\`Screenshot: \${screenshot.width}x\${screenshot.height}\`);

// Save to ./test-screenshots/
const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });
const filepath = join(screenshotDir, \`visual-debug-\${Date.now()}.png\`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(\`[SCREENSHOT] Saved to: \${filepath}\`);

process.exit(0);
\`\`\`

### Step 3: Run via stdin JSON Protocol
\`\`\`bash
echo '{"type":"run","path":"'$(pwd)'/tests/smoke/your-test.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
\`\`\`

### Step 4: READ the Screenshot File
**CRITICAL**: You MUST read the generated screenshot file to verify the visual state!
Use the Read tool to examine the screenshot at \`./test-screenshots/visual-debug-<timestamp>.png\`

### Step 5: Verify and Iterate
- Examine what the screenshot shows
- Compare to expected behavior
- Fix any visual issues
- Repeat from Step 3 until the visual state is correct

### Anti-Patterns (DO NOT DO)
- ❌ Skip building before testing
- ❌ Use command line args instead of stdin JSON protocol
- ❌ Capture screenshot but never read the file
- ❌ Assume changes worked without visual verification
- ❌ Say "should be fixed" without proving it with a screenshot

### Required Evidence Before Completion
1. Show the screenshot file path you captured
2. Confirm you READ the screenshot file using the Read tool
3. Describe what you see in the screenshot
4. Explain how it matches the expected behavior
</visual-debugging-triggered>
`.trim()

const SYSTEM_VISUAL_TRIGGER_REMINDER = `
## Visual Debug Trigger Active
UI/visual-related keywords detected. When working on visual changes:
1. MUST build: cargo build
2. MUST create test script in tests/smoke/ using captureScreenshot()
3. MUST run via stdin: echo '{"type":"run","path":"..."}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
4. MUST read the screenshot file using Read tool
5. MUST iterate until visual issue is fixed
`.trim()

/**
 * Checks if any text part contains visual/UI trigger words
 */
function hasVisualTriggerWords(parts: Part[]): boolean {
  for (const part of parts) {
    if (part.type === "text" && "text" in part) {
      const text = (part as { text: string }).text

      // Check each trigger pattern
      for (const pattern of Object.values(TRIGGER_PATTERNS)) {
        if (pattern.test(text)) {
          return true
        }
      }
    }
  }
  return false
}

/**
 * Gets which trigger categories matched (for logging/debugging)
 */
function getMatchedTriggers(parts: Part[]): string[] {
  const matched: string[] = []

  for (const part of parts) {
    if (part.type === "text" && "text" in part) {
      const text = (part as { text: string }).text

      for (const [category, pattern] of Object.entries(TRIGGER_PATTERNS)) {
        if (pattern.test(text) && !matched.includes(category)) {
          matched.push(category)
        }
      }
    }
  }

  return matched
}

export const VisualDebugTrigger: Plugin = async () => {
  const hooks: Hooks = {
    // Detect visual trigger words in user message and inject debugging instructions
    "chat.message": async (_input, output) => {
      if (hasVisualTriggerWords(output.parts)) {
        const matchedCategories = getMatchedTriggers(output.parts)

        // Inject the visual debugging instructions as an additional text part
        output.parts.push({
          type: "text",
          text: `[Visual triggers detected: ${matchedCategories.join(", ")}]\n\n${VISUAL_DEBUG_INSTRUCTIONS}`,
        } as Part)
      }
    },

    // Add visual debugging reminder to system prompt when triggers detected
    "experimental.chat.system.transform": async (_input, output) => {
      output.system.push(SYSTEM_VISUAL_TRIGGER_REMINDER)
    },

    // Preserve visual debugging context during compaction
    "experimental.session.compacting": async (_input, output) => {
      output.context.push(SYSTEM_VISUAL_TRIGGER_REMINDER)
    }
  }

  return hooks
}
