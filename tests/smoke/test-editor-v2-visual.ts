// Name: Test Editor V2 Visual
// Description: Visual test for EditorPromptV2 - monospace font, minimal padding

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[TEST] test-editor-v2-visual.ts starting...');

// Wait for initial render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot BEFORE editor (to see empty state)
try {
  const preScreenshot = await captureScreenshot();
  console.error(`[TEST] Pre-editor screenshot: ${preScreenshot.width}x${preScreenshot.height}`);
  
  const dir = join(process.cwd(), '.test-screenshots');
  mkdirSync(dir, { recursive: true });
  const prePath = join(dir, `editor-v2-pre-${Date.now()}.png`);
  writeFileSync(prePath, Buffer.from(preScreenshot.data, 'base64'));
  console.error(`[SCREENSHOT] Pre: ${prePath}`);
} catch (e) {
  console.error(`[TEST] Pre-screenshot failed: ${e}`);
}

// Show editor with TypeScript content to test:
// 1. Monospace font (code should align properly)
// 2. Line numbers visible
// 3. Syntax highlighting 
// 4. Minimal left padding (line numbers provide structure)
console.error('[TEST] Calling editor()...');

// Use setTimeout to capture DURING the editor prompt, not after
setTimeout(async () => {
  try {
    console.error('[TEST] Attempting to capture editor screenshot...');
    const screenshot = await captureScreenshot();
    console.error(`[TEST] Editor screenshot: ${screenshot.width}x${screenshot.height}`);
    
    const dir = join(process.cwd(), '.test-screenshots');
    mkdirSync(dir, { recursive: true });
    const filepath = join(dir, `editor-v2-visual-${Date.now()}.png`);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] Editor: ${filepath}`);
  } catch (e) {
    console.error(`[TEST] Editor screenshot failed: ${e}`);
  }
}, 2000); // Capture 2 seconds after editor opens

const code = await editor(`// EditorPromptV2 Visual Test
// This should render with:
// 1. Monospace font (Menlo on macOS)
// 2. Line numbers on the left
// 3. TypeScript syntax highlighting

interface User {
  id: number;
  name: string;
  email?: string;
}

function greet(user: User): string {
  return \`Hello, \${user.name}!\`;
}

const user: User = {
  id: 1,
  name: "Script Kit",
};

console.log(greet(user));

// Press Cmd+Enter to submit
`, "typescript");

console.error(`[TEST] Editor returned: ${code?.slice(0, 50)}...`);
console.error('[TEST] Editor V2 visual test completed!');
process.exit(0);
