import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const dir = join(process.cwd(), '.test-screenshots', 'grid-audit');
mkdirSync(dir, { recursive: true });

console.error('[AUDIT] Testing EDITOR prompt with grid overlay');

// Use void to not await - editor takes (content, language)
void editor(`// editor chrome audit
// Verify:
// - code starts after configured top/left padding
// - editor fills the body above the shared footer slot
// - no card/border chrome around the editor body
function hello() {
  console.log("Hello, World!");
}
const x = 42;`, 'typescript');

await new Promise(r => setTimeout(r, 1500));

console.error('[AUDIT] Capturing editor screenshot...');
const ss = await captureScreenshot();
const filepath = join(dir, '02-editor.png');
writeFileSync(filepath, Buffer.from(ss.data, 'base64'));
console.error(`[AUDIT] Screenshot saved: ${filepath}`);

process.exit(0);
