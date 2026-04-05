import '../../scripts/kit-sdk';
import { expectMiniMainWindow } from './helpers/mini_main_window';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const dir = join(process.cwd(), '.test-screenshots', 'grid-audit');
mkdirSync(dir, { recursive: true });

console.error('[AUDIT] Testing MINI MAIN WINDOW with grid overlay');

await expectMiniMainWindow('audit-main-menu', 1000);

console.error('[AUDIT] Capturing mini main window screenshot...');
const ss = await captureScreenshot();
const filepath = join(dir, '03-mini-main-window.png');
writeFileSync(filepath, Buffer.from(ss.data, 'base64'));
console.error(`[AUDIT] Screenshot saved: ${filepath}`);

process.exit(0);
