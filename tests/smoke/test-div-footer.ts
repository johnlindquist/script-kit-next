// Test: Verify div prompt shows footer
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

setTimeout(async () => {
  const screenshot = await captureScreenshot();
  const dir = join(process.cwd(), 'test-screenshots', 'unified-footer');
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  
  const path = join(dir, `div-footer-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path} (${screenshot.width}x${screenshot.height})`);
  process.exit(0);
}, 1500);

div(`
  <div class="p-8 text-white">
    <h1 class="text-2xl mb-4">Div Footer Test</h1>
    <p>The unified footer should be visible below this content.</p>
  </div>
`);
