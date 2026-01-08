// Visual design test for File Search
// This test captures the file search view state after it's triggered
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOT_DIR = join(process.cwd(), 'test-screenshots', 'file-search');

async function saveScreenshot(name: string): Promise<string> {
  mkdirSync(SCREENSHOT_DIR, { recursive: true });
  const shot = await captureScreenshot();
  const path = join(SCREENSHOT_DIR, `${name}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${name}: ${path}`);
  return path;
}

async function wait(ms: number): Promise<void> {
  return new Promise(r => setTimeout(r, ms));
}

async function main() {
  console.error('[TEST] Starting File Search design test');
  console.error('[TEST] Note: File search should be triggered via stdin before this script');
  
  // Wait for UI to stabilize
  await wait(500);
  
  // State 1: Capture current state (should be file search if triggered)
  console.error('[TEST] State 1: Initial state');
  await saveScreenshot('01-initial-state');
  
  // State 2: Type a directory path
  console.error('[TEST] State 2: Directory ~/');
  await setInput('~/');
  await wait(800);
  await saveScreenshot('02-home-directory');
  
  // State 3: Navigate to dev
  console.error('[TEST] State 3: Directory ~/dev/');
  await setInput('~/dev/');
  await wait(800);
  await saveScreenshot('03-dev-directory');
  
  console.error('[TEST] File Search design test complete!');
  console.error(`[TEST] Screenshots saved to: ${SCREENSHOT_DIR}`);
  
  process.exit(0);
}

main().catch(e => {
  console.error('[ERROR]', e);
  process.exit(1);
});
