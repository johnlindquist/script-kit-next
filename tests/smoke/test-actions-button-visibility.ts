import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: 'Test Actions Button Visibility',
  description: 'Capture main window to verify Actions button visibility',
};

await show();
await new Promise((resolve) => setTimeout(resolve, 500));

const shot = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `actions-button-${Date.now()}.png`);
writeFileSync(path, Buffer.from(shot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);
process.exit(0);
