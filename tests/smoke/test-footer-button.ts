import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

void div(`<div class="p-6 text-lg">Footer button test</div>`);

await new Promise((r) => setTimeout(r, 600));

const shot = await captureScreenshot();
const dir = join(process.cwd(), '.test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `footer-button-${Date.now()}.png`);
writeFileSync(path, Buffer.from(shot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);
process.exit(0);
