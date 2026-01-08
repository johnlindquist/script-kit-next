// @ts-nocheck
import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const dir = join(process.cwd(), 'test-screenshots', 'comparison');
mkdirSync(dir, { recursive: true });

async function capture(name: string) {
  await new Promise(r => setTimeout(r, 500));
  const shot = await captureScreenshot();
  const path = join(dir, `${name}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
}

// Test file search view using path()
const result = await path({
  startPath: "~/dev",
  hint: "Test file search alignment",
  onInit: async () => {
    await capture('file-search-after-fix');
    process.exit(0);
  }
});
