// Name: Test Editor Find/Replace Visual
// Description: Visual test for editor() find/replace overlay

import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

console.error('[TEST] Starting editor find/replace visual test');

const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

const capture = async () => {
  try {
    console.error('[TEST] Capturing screenshot...');
    const screenshot = await captureScreenshot();
    console.error(`[TEST] Screenshot: ${screenshot.width}x${screenshot.height}`);

    const filepath = join(screenshotDir, 'editor-find-replace.png');
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${filepath}`);

    process.exit(0);
  } catch (err) {
    console.error('[TEST] Screenshot error:', err);
    process.exit(1);
  }
};

setTimeout(() => {
  void capture();
}, 1200);

await editor(
  `function greet(name: string) {
  console.log('Hello', name);
  console.log('Goodbye', name);
  console.warn('Another console output');
}
`,
  'typescript',
  undefined,
  {
    findOverlay: {
      query: 'console',
      replacement: 'logger',
      showReplace: true,
      focus: 'query',
    },
  }
);
