#!/usr/bin/env bun
/**
 * Storybook Screenshot Test
 * 
 * Launches the storybook, waits for it to render, and captures a screenshot.
 * Uses the main app's captureScreenshot() capability via a helper script.
 */

import { spawn, execSync } from 'child_process';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

const SCREENSHOT_DIR = join(process.cwd(), 'test-screenshots');
const STORYBOOK_BINARY = join(process.cwd(), 'target/debug/storybook');

// Ensure screenshot directory exists
mkdirSync(SCREENSHOT_DIR, { recursive: true });

async function captureStorybookScreenshot(storyName?: string): Promise<string> {
    // Build storybook first
    console.log('Building storybook...');
    execSync('cargo build --bin storybook', { stdio: 'inherit' });
    
    // Launch storybook
    const args = storyName ? ['--story', storyName] : [];
    console.log(`Launching storybook${storyName ? ` with story: ${storyName}` : ''}...`);
    
    const storybook = spawn(STORYBOOK_BINARY, args, {
        stdio: ['pipe', 'pipe', 'pipe'],
        detached: false,
    });

    // Wait for window to render
    await new Promise(r => setTimeout(r, 2000));

    // Use screencapture to capture the storybook window
    // -l flag captures a specific window by ID, but we'll use -w for interactive or just capture frontmost
    const timestamp = Date.now();
    const screenshotPath = join(SCREENSHOT_DIR, `storybook-${storyName || 'main'}-${timestamp}.png`);
    
    try {
        // Capture the frontmost window (storybook should be frontmost)
        // Using -o to capture without shadow, -x for no sound
        execSync(`screencapture -o -x -l $(osascript -e 'tell app "System Events" to id of first window of (first process whose frontmost is true)') "${screenshotPath}"`, {
            stdio: 'pipe',
            timeout: 5000,
        });
    } catch (e) {
        // Fallback: capture entire screen and we'll crop later
        console.log('Window capture failed, capturing screen...');
        execSync(`screencapture -x "${screenshotPath}"`, { stdio: 'pipe' });
    }

    // Kill storybook
    storybook.kill('SIGTERM');
    
    console.log(`Screenshot saved: ${screenshotPath}`);
    return screenshotPath;
}

// Main
const storyName = process.argv[2] || 'header-variations';
captureStorybookScreenshot(storyName)
    .then(path => {
        console.log(`\nScreenshot captured: ${path}`);
        console.log('Open with: open', path);
        process.exit(0);
    })
    .catch(err => {
        console.error('Failed:', err);
        process.exit(1);
    });
