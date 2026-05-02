// Name: Audit Template Prompt
// Description: Opens a mustache template prompt with enough fields to exercise scrolling.

import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'fs';
import { join } from 'path';

export const metadata = {
  name: 'Audit Template Prompt',
  description: 'Shows TemplatePrompt placeholder editing for chrome audits',
};

setTimeout(async () => {
  try {
    const screenshot = await captureScreenshot();
    const dir = join(process.cwd(), '.test-screenshots', 'grid-audit');
    mkdirSync(dir, { recursive: true });
    const file = join(dir, '08-template-prompt.png');
    writeFileSync(file, Buffer.from(screenshot.data, 'base64'));
    console.error(`[SCREENSHOT] ${file}`);
  } catch (error) {
    console.error('[SCREENSHOT] failed', error);
  } finally {
    process.exit(0);
  }
}, 5000);

process.stdout.write(
  `${JSON.stringify({
    type: 'template',
    id: 'audit-template-prompt',
    template: `Script {{name}}
Author {{author}}
Description {{description}}
Command {{command}}
Content {{body}}
Tag {{tag}}
Slug {{script_slug}}`,
  })}\n`
);
