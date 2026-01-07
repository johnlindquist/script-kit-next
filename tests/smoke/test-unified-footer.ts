// Test: Verify unified footer across prompts
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

async function saveScreenshot(name: string): Promise<string> {
  const screenshot = await captureScreenshot();
  const dir = join(process.cwd(), 'test-screenshots', 'unified-footer');
  if (!existsSync(dir)) {
    mkdirSync(dir, { recursive: true });
  }
  
  const path = join(dir, `${name}-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${name}: ${path} (${screenshot.width}x${screenshot.height})`);
  return path;
}

console.error('[TEST] Starting unified footer test...');

// Test editor with plain content (no snippet)
console.error('[TEST] Testing editor footer with plain content...');
editor("// Plain TypeScript content\nconst x = 1;", "typescript");

await new Promise(r => setTimeout(r, 2000));
const editorPath = await saveScreenshot('editor-plain');

// Get layout info
const layout = await getLayoutInfo();
console.error(`[LAYOUT] Window: ${layout.windowWidth}x${layout.windowHeight}, Type: ${layout.promptType}`);

console.error('[TEST] Test complete');
process.exit(0);
