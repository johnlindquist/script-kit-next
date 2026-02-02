import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// This test verifies that the code preview background in light mode
// uses a light background color instead of the dark gray from design tokens

// Wait for UI to fully render
await new Promise(r => setTimeout(r, 1000));

// Capture screenshot to verify light mode code preview
const screenshot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
const path = join(dir, `light-mode-preview-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] ${path}`);

// Get layout info
const layoutInfo = await getLayoutInfo();
console.error(`[LAYOUT] Window: ${layoutInfo.windowWidth}x${layoutInfo.windowHeight}`);
console.error(`[LAYOUT] Prompt type: ${layoutInfo.promptType}`);

process.exit(0);
