import type { Plugin, Hooks } from "@opencode-ai/plugin"
import type { Part } from "@opencode-ai/sdk"

/**
 * Visual Testing Enforcer Plugin
 * 
 * Detects when user provides an image attachment and enforces the visual testing workflow:
 * 1. Hooks into `chat.message` to detect image attachments in parts
 * 2. Injects mandatory context requiring the agent to:
 *    - Create a test script that sets up the relevant UI state
 *    - Call captureScreenshot() and save to ./test-screenshots/
 *    - READ the screenshot file to compare with user's provided image
 *    - Iterate until the visual issue shown in user's image is fixed
 * 3. Hooks into `experimental.chat.system.transform` to add visual testing protocol
 */

const VISUAL_TESTING_PROTOCOL = `
<visual-testing-mandatory>
## MANDATORY: Visual Testing Workflow

The user has provided an IMAGE. You MUST follow this exact workflow:

### Step 1: Create Test Script
Write a test script in \`tests/smoke/\` that:
- Sets up the UI state shown in the user's image
- Calls \`captureScreenshot()\` after the UI renders
- Saves the PNG to \`./test-screenshots/\`

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
const filepath = join(screenshotDir, \`test-name-\${Date.now()}.png\`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(\`[SCREENSHOT] Saved to: \${filepath}\`);

process.exit(0);
\`\`\`

### Step 2: Run Test via stdin JSON Protocol
\`\`\`bash
cargo build && echo '{"type":"run","path":"'$(pwd)'/tests/smoke/test-name.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
\`\`\`

### Step 3: READ the Screenshot File
**CRITICAL**: You MUST read the generated screenshot file to compare with the user's image!
Use the Read tool to examine the screenshot at \`./test-screenshots/test-name-<timestamp>.png\`

### Step 4: Compare and Fix
- Compare the screenshot you captured with the user's provided image
- Identify discrepancies (layout, colors, spacing, visibility, etc.)
- Fix the code
- Repeat from Step 2 until the screenshot matches the expected state

### Anti-Patterns (DO NOT DO)
- ❌ Capture screenshot but never read the file
- ❌ Assume the fix worked without visual verification
- ❌ Skip the screenshot comparison step
- ❌ Say "the issue should be fixed" without proving it with a screenshot

### Required Evidence
Before claiming the fix is complete, you MUST:
1. Show the screenshot file path you captured
2. Confirm you READ the screenshot file
3. Describe what you see in the screenshot
4. Explain how it matches (or differs from) the user's image
</visual-testing-mandatory>
`.trim()

const SYSTEM_VISUAL_REMINDER = `
## Visual Testing Protocol (when images provided)
If the user provides an image, you MUST:
1. Create test script using captureScreenshot() 
2. Run via stdin JSON: \`echo '{"type":"run",...}' | ./target/debug/script-kit-gpui\`
3. READ the resulting screenshot file (don't just capture it!)
4. Compare to user's image and iterate until fixed
`.trim()

/**
 * Detects if parts array contains an image file attachment
 * Images come through as FilePart with mime type starting with "image/"
 */
function hasImageInParts(parts: Part[]): boolean {
  return parts.some(part => {
    // FilePart has type: "file" and a mime field
    if (part.type === "file" && "mime" in part) {
      const mime = (part as { mime?: string }).mime
      return mime?.startsWith("image/")
    }
    return false
  })
}

export const VisualTestingEnforcer: Plugin = async () => {
  const hooks: Hooks = {
    // Detect image in user message parts and inject mandatory visual testing context
    "chat.message": async (_input, output) => {
      // Check if any of the user's parts contain an image
      if (hasImageInParts(output.parts)) {
        // Inject the visual testing protocol as an additional text part
        // The runtime will handle assigning proper IDs
        output.parts.push({
          type: "text",
          text: VISUAL_TESTING_PROTOCOL,
        } as Part)
      }
    },

    // Add visual testing reminder to system prompt
    "experimental.chat.system.transform": async (_input, output) => {
      output.system.push(SYSTEM_VISUAL_REMINDER)
    },

    // Preserve visual testing context during compaction
    "experimental.session.compacting": async (_input, output) => {
      output.context.push(SYSTEM_VISUAL_REMINDER)
    }
  }
  
  return hooks
}
