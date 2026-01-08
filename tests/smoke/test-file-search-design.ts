// Visual design test for File Search - captures all states
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
  
  // State 1: Initial empty state
  console.error('[TEST] State 1: Initial empty state');
  await wait(800);
  await saveScreenshot('01-initial-empty');
  
  // State 2: Type a directory path to trigger directory listing
  console.error('[TEST] State 2: Directory listing ~/');
  await setInput('~/');
  await wait(500);
  await saveScreenshot('02-directory-listing-home');
  
  // State 3: Navigate deeper into a directory
  console.error('[TEST] State 3: Directory listing ~/dev/');
  await setInput('~/dev/');
  await wait(500);
  await saveScreenshot('03-directory-listing-dev');
  
  // State 4: Filter within directory
  console.error('[TEST] State 4: Filtered directory ~/dev/script');
  await setInput('~/dev/script');
  await wait(500);
  await saveScreenshot('04-directory-filtered');
  
  // State 5: Search for a common file type
  console.error('[TEST] State 5: Search for .ts files');
  await setInput('.ts');
  await wait(800);
  await saveScreenshot('05-search-ts-files');
  
  // State 6: No matches
  console.error('[TEST] State 6: No matches state');
  await setInput('xyznonexistent12345');
  await wait(500);
  await saveScreenshot('06-no-matches');
  
  // State 7: Clear and show empty again
  console.error('[TEST] State 7: Cleared empty state');
  await setInput('');
  await wait(300);
  await saveScreenshot('07-cleared-empty');
  
  console.error('[TEST] File Search design test complete!');
  console.error(`[TEST] Screenshots saved to: ${SCREENSHOT_DIR}`);
  
  process.exit(0);
}

main().catch(e => {
  console.error('[ERROR]', e);
  process.exit(1);
});
