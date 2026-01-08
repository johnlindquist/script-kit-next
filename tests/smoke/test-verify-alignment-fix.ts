// Visual verification test for file search input alignment fix
// Captures both main menu and file search to compare
// @ts-nocheck
import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const dir = join(process.cwd(), 'test-screenshots', 'alignment-fix');
mkdirSync(dir, { recursive: true });

// Helper to capture screenshot
async function capture(name: string): Promise<string> {
  await new Promise(r => setTimeout(r, 600)); 
  const shot = await captureScreenshot();
  const path = join(dir, `${name}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

// Capture main menu first
console.error('[TEST] Capturing main menu with placeholder...');
await capture('01-main-menu');

// Exit and let coordinator run separate test for file search
console.error('[TEST] Done - main menu captured');
process.exit(0);
