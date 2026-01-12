import type { Plugin } from "@opencode-ai/plugin"

/**
 * Project Reminders Plugin
 * 
 * Injects AI logging legend, autonomous testing reminders, and visual testing
 * protocol into:
 * 1. The system prompt (via experimental.chat.system.transform)
 * 2. Compaction context (via experimental.session.compacting)
 */

const LOG_LEGEND = `
## AI Log Format (SCRIPT_KIT_AI_LOG=1)
Format: \`SS.mmm|L|C|message\` | Levels: i/w/e/d/t | Categories: P=POSITION A=APP U=UI S=STDIN H=HOTKEY V=VISIBILITY E=EXEC K=KEY F=FOCUS T=THEME C=CACHE R=PERF W=WINDOW_MGR X=ERROR M=MOUSE_HOVER L=SCROLL_STATE Q=SCROLL_PERF D=DESIGN G=SCRIPT N=CONFIG Z=RESIZE
`.trim()

const TESTING_REMINDER = `
## Testing Protocol
- **Run app**: \`echo '{"type":"run","path":"..."}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1\`
- **Before commit**: \`cargo check && cargo clippy && cargo test\`
- **Visual changes**: MUST write test script using \`captureScreenshot()\`, save PNG to \`./test-screenshots/\`, read file to verify
- **If user provides image**: YOU MUST use visual testing - create test, capture screenshot, compare, fix issues shown in image
`.trim()

const VISUAL_TESTING_PROTOCOL = `
## Visual Testing Protocol (MANDATORY)

### When User Provides an Image
If the user shares a screenshot or describes a visual issue:
1. **CREATE** a test script that reproduces the UI state
2. **CAPTURE** a screenshot using \`captureScreenshot()\` SDK function
3. **SAVE** the PNG to \`./test-screenshots/<descriptive-name>.png\`
4. **READ** the screenshot file using the Read tool - YOU MUST ACTUALLY READ IT
5. **COMPARE** your screenshot to what the user showed/described
6. **ITERATE** until your screenshot matches the expected state

### Screenshot Analysis Workflow
\`\`\`
# 1. Write test script that sets up UI and captures screenshot
# 2. Run via stdin JSON protocol:
echo '{"type":"run","path":"'$(pwd)'/tests/smoke/my-visual-test.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# 3. MANDATORY: Read the screenshot file to verify
# Use Read tool on ./test-screenshots/<filename>.png
\`\`\`

### Anti-Patterns (DO NOT DO)
- "I captured a screenshot" without using Read tool to analyze it
- "The screenshot shows..." without having actually read the file
- Claiming visual verification is complete without reading the PNG
- Skipping screenshot comparison when user provides an image
- Using only logs to verify visual changes (logs can't show layout)

### Visual Test Script Template
\`\`\`typescript
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Set up UI state to test
await div(\`<div class="...">Your test content</div>\`);
await new Promise(r => setTimeout(r, 500)); // Wait for render

// Capture and save
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, 'my-test-' + Date.now() + '.png');
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error('[SCREENSHOT] ' + path);

process.exit(0);
\`\`\`

### Verification Checklist
Before claiming visual fix is complete:
- [ ] Test script created and runs without error
- [ ] Screenshot saved to ./test-screenshots/
- [ ] Screenshot file READ using Read tool (not just captured)
- [ ] Visual output matches user's expected state
- [ ] If mismatch found, iterated until fixed
`.trim()

const COMBINED_REMINDER = `
<project-reminder>
${LOG_LEGEND}

${TESTING_REMINDER}

${VISUAL_TESTING_PROTOCOL}
</project-reminder>
`.trim()

// Compact context for compaction - preserves essential visual testing state
const COMPACTION_CONTEXT = `
<project-reminder>
${LOG_LEGEND}

${TESTING_REMINDER}

## Visual Testing (Compacted)
When user provides image: CREATE test → CAPTURE screenshot → SAVE to ./test-screenshots/ → READ file with Read tool → COMPARE → ITERATE until match.
Anti-pattern: Claiming "I captured a screenshot" without actually reading the PNG file.
Visual verification is NOT complete until you READ the screenshot file and analyze it.
</project-reminder>
`.trim()

const ProjectReminders: Plugin = async () => {
  return {
    // Inject into system prompt - runs on every conversation
    "experimental.chat.system.transform": async (_input, output) => {
      output.system.push(COMBINED_REMINDER)
    },

    // Preserve in compaction context - uses shorter version to save tokens
    "experimental.session.compacting": async (_input, output) => {
      output.context.push(COMPACTION_CONTEXT)
    }
  }
}

export default ProjectReminders
