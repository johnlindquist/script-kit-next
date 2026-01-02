// Test script for EditorPromptV2 footer with language indicator
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: "Editor Footer Test",
  description: "Tests that the editor footer displays the language",
};

console.error('[TEST] Starting editor footer test...');

// Test with explicit TypeScript language
const content = `// TypeScript code
function greet(name: string): string {
  return \`Hello, \${name}!\`;
}

const result = greet("World");
console.log(result);
`;

console.error('[TEST] Calling editor() with typescript language...');
// Call editor with explicit language
editor(content, "typescript");

// Wait for render
await new Promise(r => setTimeout(r, 1500));

console.error('[TEST] Capturing screenshot...');
const screenshot = await captureScreenshot();
console.error(`[TEST] Screenshot captured: ${screenshot.width}x${screenshot.height}`);

// Save screenshot
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });
const filepath = join(dir, `editor-footer-${Date.now()}.png`);
writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${filepath}`);

process.exit(0);
