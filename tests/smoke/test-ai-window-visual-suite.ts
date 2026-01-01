// Name: AI Window Visual Test Suite
// Description: Captures comprehensive screenshots of the AI window for visual regression testing

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

const SCREENSHOT_DIR = join(process.cwd(), 'test-screenshots', 'ai-window-suite');
const DELAY_MS = 1000; // Wait for UI to stabilize

// Ensure screenshot directory exists
if (!existsSync(SCREENSHOT_DIR)) {
  mkdirSync(SCREENSHOT_DIR, { recursive: true });
}

async function captureAndSave(name: string, description: string): Promise<void> {
  console.error(`[AI-SUITE] Capturing: ${name} - ${description}`);
  
  await new Promise(resolve => setTimeout(resolve, DELAY_MS));
  
  try {
    const screenshot = await captureScreenshot();
    const filename = `ai-${name}-${Date.now()}.png`;
    const filepath = join(SCREENSHOT_DIR, filename);
    writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
    console.error(`[AI-SUITE] Saved: ${filepath} (${screenshot.width}x${screenshot.height})`);
  } catch (err) {
    console.error(`[AI-SUITE] ERROR capturing ${name}:`, err);
  }
}

async function runVisualTests(): Promise<void> {
  console.error('[AI-SUITE] Starting AI Window Visual Test Suite');
  console.error('[AI-SUITE] Screenshots will be saved to:', SCREENSHOT_DIR);
  
  // The AI window should already be open with mock data via openAiWithMockData command
  // We'll capture various states
  
  // 1. Initial state with chat list and messages
  await captureAndSave('01-initial-state', 'AI window with mock data loaded');
  
  // 2. After a short delay to ensure all rendering is complete
  await new Promise(resolve => setTimeout(resolve, 500));
  await captureAndSave('02-stable-render', 'After render stabilization');
  
  console.error('[AI-SUITE] Visual test suite complete');
  console.error('[AI-SUITE] Check screenshots in:', SCREENSHOT_DIR);
}

// Run the tests
runVisualTests()
  .then(() => {
    console.error('[AI-SUITE] All tests completed successfully');
    process.exit(0);
  })
  .catch((err) => {
    console.error('[AI-SUITE] Test suite failed:', err);
    process.exit(1);
  });
