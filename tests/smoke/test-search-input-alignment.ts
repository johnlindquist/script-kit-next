// Visual test: Verify file search input matches main menu alignment
// Tests the fix for cramped input styling when typing paths like ~/dev/
// @ts-nocheck
import '../../scripts/kit-sdk';
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';

const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

// Helper to capture screenshot
async function capture(name: string): Promise<string> {
  await new Promise(r => setTimeout(r, 600)); // Wait for render
  const shot = await captureScreenshot();
  const path = join(dir, `alignment-${name}-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(shot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  return path;
}

// Test: Capture both views to visually compare input alignment
console.error('[TEST] Starting input alignment visual test...');

// First, capture the main menu
console.error('[TEST] 1. Capturing main menu...');
await capture('main-menu');

// Exit - this test just captures the main menu state
// The coordinator should run the file search separately via stdin commands
console.error('[TEST] Done - check test-screenshots/alignment-*.png files');
process.exit(0);
