// Name: Test EditorPromptV2 with Actions
// Description: Verify the actions dialog appears in the new editor

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting EditorPromptV2 with actions test...');

// Create screenshot directory
const screenshotDir = join(process.cwd(), '.test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

// Capture screenshot function
async function captureAndSave(name: string) {
  const screenshot = await captureScreenshot();
  console.error(`[TEST] Screenshot "${name}": ${screenshot.width}x${screenshot.height}`);
  const filename = `${name}-${Date.now()}.png`;
  const filepath = join(screenshotDir, filename);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
  return filepath;
}

// Show editor WITH actions - using correct SDK signature:
// editor(content: string, language: string, actions?: Action[])
console.error('[TEST] Calling editor() with actions...');

const content = `// TypeScript code with actions
function greet(name: string): string {
  return \`Hello, \${name}!\`;
}

const result = greet("World");
console.log(result);
`;

const actions = [
  { name: "Save File", shortcut: "cmd+s", onAction: () => console.error("Save!") },
  { name: "Format Code", shortcut: "cmd+shift+f", onAction: () => console.error("Format!") },
  { name: "Run Script", shortcut: "cmd+enter", onAction: () => console.error("Run!") },
  { name: "Copy All", shortcut: "cmd+shift+c", onAction: () => console.error("Copy!") }
];

// Start editor (don't await since it blocks until submit)
const editorPromise = editor(content, "typescript", actions);

// Wait for editor to render
await new Promise(r => setTimeout(r, 1500));

// Capture screenshot showing the Actions button in the titlebar
console.error('[TEST] Capturing editor screenshot...');
await captureAndSave('editor-v2-actions');

// Wait a bit more then exit
await new Promise(r => setTimeout(r, 500));
console.error('[TEST] Test complete - look for "Actions" button in top-right corner');
process.exit(0);
