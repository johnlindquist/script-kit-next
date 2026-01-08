// Visual test: Trigger and capture file search view to verify alignment fix
// @ts-nocheck
import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

// Helper to capture screenshot
async function capture(name: string): Promise<string> {
  await new Promise(r => setTimeout(r, 800)); // Wait for render
  const shot = await captureScreenshot();
  const path = join(dir, `alignment-${name}-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

console.error('[TEST] File search view alignment test');

// Use path() to trigger the file search/path prompt view
// This should show the file browser UI similar to typing ~/dev/ in main menu
const result = await path({
  startPath: "~/dev",
  hint: "Select a file or folder",
  onInit: async () => {
    await new Promise(r => setTimeout(r, 500));
    await capture('file-search-view');
    process.exit(0);
  }
});

console.error('[TEST] Done');
process.exit(0);
