// Name: AI Titlebar Visual Debug
// Description: Captures AI window titlebar screenshots for open/collapsed states

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), 'test-screenshots', 'ai-titlebar');
mkdirSync(screenshotDir, { recursive: true });

const state = process.env.AI_TITLEBAR_STATE ?? 'open';

console.error(`[TEST] CWD: ${process.cwd()}`);

if (process.stdin && typeof (process.stdin as any).ref === 'function') {
  (process.stdin as any).ref();
}

async function capture(): Promise<void> {
  console.error(`[TEST] Capturing AI titlebar (${state})...`);
  await new Promise((resolve) => setTimeout(resolve, 800));

  const screenshot = await captureScreenshot();
  const filePath = join(screenshotDir, `ai-titlebar-${state}-${Date.now()}.png`);
  writeFileSync(filePath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filePath}`);
}

capture()
  .then(() => process.exit(0))
  .catch((err) => {
    console.error('[TEST] Failed to capture screenshot:', err);
    process.exit(1);
  });
