// Test: Verify arg prompt shows footer
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

setTimeout(async () => {
  const screenshot = await captureScreenshot();
  const dir = join(process.cwd(), 'test-screenshots', 'unified-footer');
  if (!existsSync(dir)) mkdirSync(dir, { recursive: true });
  
  const path = join(dir, `arg-footer-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path} (${screenshot.width}x${screenshot.height})`);
  process.exit(0);
}, 1500);

arg({
  placeholder: "Arg Footer Test",
  choices: ["Option A", "Option B", "Option C"]
});
