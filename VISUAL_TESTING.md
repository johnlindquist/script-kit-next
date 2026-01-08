# Visual Testing Guide: Using AI to Iterate on UX and UI

> A comprehensive guide for AI agents and developers to effectively test, debug, and iterate on UI/UX changes in Script Kit GPUI.

This document captures patterns that have proven effective across many AI coding sessions for visual testing and UI iteration.

---

## Table of Contents

1. [Philosophy](#1-philosophy)
2. [Core Infrastructure](#2-core-infrastructure)
3. [The Build-Test-Iterate Loop](#3-the-build-test-iterate-loop)
4. [Screenshot Capture Patterns](#4-screenshot-capture-patterns)
5. [Layout Inspection Tools](#5-layout-inspection-tools)
6. [Visual Test Templates](#6-visual-test-templates)
7. [Multi-State Visual Regression](#7-multi-state-visual-regression)
8. [Debugging Visual Issues](#8-debugging-visual-issues)
9. [Anti-Patterns](#9-anti-patterns)
10. [Real-World Examples](#10-real-world-examples)

---

## 1. Philosophy

### Why Visual Testing Matters for AI-Driven Development

Traditional development relies on human eyes to verify UI changes. When an AI agent makes UI changes, it cannot "see" the result without explicit visual testing infrastructure. This creates a gap that must be bridged systematically.

**Core Principles:**

1. **Never guess at visual state** - Always capture and verify
2. **Screenshots are evidence** - If you didn't capture it, you didn't verify it
3. **Logs complement but don't replace visuals** - Layout issues are often invisible in logs
4. **Iterate autonomously** - Don't ask the user to verify; verify yourself

### The Verification Hierarchy

```
Most Reliable          Screenshots + Visual Inspection
        |              getLayoutInfo() + Programmatic Bounds
        |              Grid Overlay + Component Boundaries  
        |              Structured Logs (resize, bounds)
Least Reliable         Console Output Only
```

---

## 2. Core Infrastructure

### 2.1 SDK Screenshot API

The SDK provides `captureScreenshot()` which captures **only the app window** (not the entire screen):

```typescript
interface Screenshot {
  data: string;     // Base64-encoded PNG
  width: number;    // Pixel width
  height: number;   // Pixel height
}

const screenshot = await captureScreenshot();
```

**Why SDK over system tools:**
- Captures only the app window (privacy, cleaner)
- Works consistently across platforms
- No permission dialogs
- Returns structured data

**Blocked tools (do NOT use):**
- `screencapture` (macOS)
- `scrot`, `gnome-screenshot`, `flameshot`, `maim` (Linux)
- ImageMagick `import`

### 2.2 Window Bounds API

```typescript
interface WindowBounds {
  x: number;
  y: number;
  width: number;
  height: number;
}

const bounds = await getWindowBounds();
```

### 2.3 Layout Info API

```typescript
interface LayoutInfo {
  windowWidth: number;
  windowHeight: number;
  promptType: string;  // "arg"|"div"|"editor"|"mainMenu"|...
  components: LayoutComponentInfo[];
  timestamp: string;
}

interface LayoutComponentInfo {
  name: string;
  type: "prompt"|"input"|"button"|"list"|"listItem"|"header"|"container"|"panel"|"other";
  bounds: { x: number; y: number; width: number; height: number };
  boxModel?: { 
    padding?: { top: number; right: number; bottom: number; left: number };
    margin?: { top: number; right: number; bottom: number; left: number };
    gap?: number;
  };
  flex?: { 
    direction?: "row"|"column";
    grow?: number;
    shrink?: number;
    basis?: string;
    alignItems?: string;
    justifyContent?: string;
  };
  depth: number;
  parent?: string;
  children: string[];
}

const layout = await getLayoutInfo();
```

### 2.4 stdin JSON Protocol

**CRITICAL: Never pass scripts as CLI arguments.** The app uses stdin JSON messages:

```bash
# CORRECT - Use stdin JSON protocol
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-visual.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# WRONG - Does nothing!
./target/debug/script-kit-gpui tests/smoke/test-my-visual.ts
```

---

## 3. The Build-Test-Iterate Loop

This loop is **non-negotiable** for any UI change:

```bash
# Step 1: Build
cargo build

# Step 2: Run visual test via stdin JSON
echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-feature-visual.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Step 3: Check screenshot output path in logs
# [SCREENSHOT] /path/to/test-screenshots/my-feature-1234567890.png

# Step 4: READ the screenshot file to verify visual state
# Use Read tool on the PNG path

# Step 5: If broken, fix code and repeat from Step 1
```

### Key Environment Variables

| Variable | Purpose |
|----------|---------|
| `SCRIPT_KIT_AI_LOG=1` | Compact logs (~70% fewer tokens) |
| `SCRIPT_KIT_DEBUG_GRID=1` | Show layout grid overlay |

---

## 4. Screenshot Capture Patterns

### 4.1 Minimal Screenshot Test

```typescript
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

// Set up UI state
await div(`<div class="p-4 bg-blue-500 text-white rounded-lg">Test Content</div>`);

// Wait for render (critical!)
await new Promise(r => setTimeout(r, 500));

// Capture screenshot
const screenshot = await captureScreenshot();
console.error(`[SCREENSHOT] Size: ${screenshot.width}x${screenshot.height}`);

// Save to test-screenshots/
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });

const path = join(dir, `my-test-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
console.error(`[SCREENSHOT] Saved: ${path}`);

// Exit cleanly
process.exit(0);
```

### 4.2 Reusable Capture Helper

```typescript
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const SCREENSHOT_DIR = join(process.cwd(), 'test-screenshots');
mkdirSync(SCREENSHOT_DIR, { recursive: true });

async function capture(name: string, waitMs: number = 500): Promise<string> {
  await new Promise(r => setTimeout(r, waitMs));
  
  const screenshot = await captureScreenshot();
  const filename = `${name}-${Date.now()}.png`;
  const filepath = join(SCREENSHOT_DIR, filename);
  
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${name}: ${screenshot.width}x${screenshot.height} -> ${filepath}`);
  
  return filepath;
}

// Usage
await div(`<div>My UI</div>`);
await capture('my-feature-initial');
```

### 4.3 Combined Screenshot + Layout

```typescript
async function captureWithLayout(name: string, waitMs: number = 500) {
  await new Promise(r => setTimeout(r, waitMs));
  
  // Capture both in parallel
  const [screenshot, layout] = await Promise.all([
    captureScreenshot(),
    getLayoutInfo()
  ]);
  
  // Log key layout info
  console.error(`[LAYOUT] Window: ${layout.windowWidth}x${layout.windowHeight}`);
  console.error(`[LAYOUT] PromptType: ${layout.promptType}`);
  console.error(`[LAYOUT] Components: ${layout.components.length}`);
  
  // Log component bounds for debugging
  layout.components.forEach(c => {
    console.error(`[LAYOUT]   ${c.name}: ${c.bounds.width}x${c.bounds.height} at (${c.bounds.x},${c.bounds.y})`);
  });
  
  // Save screenshot
  const path = join(SCREENSHOT_DIR, `${name}-${Date.now()}.png`);
  writeFileSync(path, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${path}`);
  
  return { screenshot, layout, path };
}
```

---

## 5. Layout Inspection Tools

### 5.1 Grid Overlay

Enable visual layout debugging with the grid overlay:

```bash
# Show grid with bounds
echo '{"type":"showGrid","showBounds":true}' | ./target/debug/script-kit-gpui

# Full debug mode
echo '{"type":"showGrid","showBounds":true,"showBoxModel":true,"showAlignmentGuides":true,"showDimensions":true}' | \
  ./target/debug/script-kit-gpui

# Hide grid
echo '{"type":"hideGrid"}' | ./target/debug/script-kit-gpui
```

**Grid overlay options:**
- `gridSize`: Grid spacing (default: 8)
- `showBounds`: Show component boundaries
- `showBoxModel`: Show padding/margin
- `showAlignmentGuides`: Show alignment lines
- `showDimensions`: Show size labels
- `depth`: "prompts" | "all" | specific component names

**Color coding:**
- Red: Prompts
- Teal: Inputs
- Yellow: Buttons
- Mint: Lists
- Plum: Headers
- Sky: Containers

### 5.2 Combined Grid + Test Workflow

```bash
# Show grid overlay, then run test
(echo '{"type":"showGrid","showBounds":true,"showDimensions":true}'; \
 echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-layout.ts"}') | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
```

### 5.3 Programmatic Layout Verification

```typescript
import '../../scripts/kit-sdk';

const EXPECTED_WINDOW_HEIGHT = 700;
const TOLERANCE = 20;

// Show editor
const editorPromise = editor("test content", "typescript");

// Wait for render
await new Promise(r => setTimeout(r, 500));

// Get layout info
const layout = await getLayoutInfo();

// Verify window height
const heightDiff = Math.abs(layout.windowHeight - EXPECTED_WINDOW_HEIGHT);
if (heightDiff > TOLERANCE) {
  console.error(`[FAIL] Window height ${layout.windowHeight} != expected ${EXPECTED_WINDOW_HEIGHT}`);
} else {
  console.error(`[PASS] Window height ${layout.windowHeight} within tolerance`);
}

// Verify specific component exists and has expected size
const editor = layout.components.find(c => c.name.includes('editor'));
if (editor) {
  console.error(`[INFO] Editor bounds: ${JSON.stringify(editor.bounds)}`);
} else {
  console.error(`[FAIL] Editor component not found`);
}

process.exit(0);
```

---

## 6. Visual Test Templates

### 6.1 Basic Visual Test

```typescript
// Name: Test Feature Visual
// Description: Visual verification of feature X

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SMOKE] test-feature-visual.ts starting...');

// Setup
const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

// Show UI
await div(`
  <div class="flex flex-col gap-4 p-8">
    <h1 class="text-2xl font-bold text-white">Feature Test</h1>
    <p class="text-gray-300">Verify this renders correctly</p>
  </div>
`);

// Wait for render
await new Promise(r => setTimeout(r, 600));

// Capture
const screenshot = await captureScreenshot();
const path = join(screenshotDir, `feature-test-${Date.now()}.png`);
writeFileSync(path, Buffer.from(screenshot.data, 'base64'));

console.error(`[SCREENSHOT] ${path}`);
console.error(`[SCREENSHOT] Size: ${screenshot.width}x${screenshot.height}`);

process.exit(0);
```

### 6.2 Window Resize Verification

```typescript
// Name: Test Window Resize Visual
// Description: Verify window resizes correctly for different content

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const screenshotDir = join(process.cwd(), 'test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

// Constants from window_resize.rs
const EXPECTED_HEIGHTS = {
  compact: 120,    // MIN_HEIGHT
  standard: 500,   // STANDARD_HEIGHT  
  full: 700,       // MAX_HEIGHT
};
const TOLERANCE = 20;

interface TestResult {
  test: string;
  status: 'pass' | 'fail';
  expected: number;
  actual: number;
  diff: number;
}

const results: TestResult[] = [];

// Test 1: Editor (should be MAX_HEIGHT = 700)
console.error('[TEST] Editor height...');
editor("test content", "typescript");
await new Promise(r => setTimeout(r, 800));

const bounds1 = await getWindowBounds();
const shot1 = await captureScreenshot();
writeFileSync(
  join(screenshotDir, `resize-editor-${Date.now()}.png`),
  Buffer.from(shot1.data, 'base64')
);

const diff1 = Math.abs(bounds1.height - EXPECTED_HEIGHTS.full);
results.push({
  test: 'editor-height',
  status: diff1 <= TOLERANCE ? 'pass' : 'fail',
  expected: EXPECTED_HEIGHTS.full,
  actual: bounds1.height,
  diff: diff1,
});

// Summary
console.error('[RESULTS]');
results.forEach(r => {
  console.error(`  ${r.status.toUpperCase()}: ${r.test} - expected ${r.expected}, got ${r.actual} (diff: ${r.diff})`);
});

process.exit(results.every(r => r.status === 'pass') ? 0 : 1);
```

### 6.3 Visual Regression Test Suite

```typescript
// Name: Visual Regression Test Suite
// Description: Captures multiple UI states for regression comparison

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync, existsSync } from 'fs';
import { join } from 'path';

const SUITE_NAME = 'my-feature';
const SCREENSHOT_DIR = join(process.cwd(), 'test-screenshots', SUITE_NAME);
const DELAY_MS = 600;

if (!existsSync(SCREENSHOT_DIR)) {
  mkdirSync(SCREENSHOT_DIR, { recursive: true });
}

async function captureState(name: string, description: string): Promise<void> {
  console.error(`[SUITE] Capturing: ${name} - ${description}`);
  
  await new Promise(r => setTimeout(r, DELAY_MS));
  
  const screenshot = await captureScreenshot();
  const filename = `${name}-${Date.now()}.png`;
  const filepath = join(SCREENSHOT_DIR, filename);
  
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SUITE] Saved: ${filepath} (${screenshot.width}x${screenshot.height})`);
}

// =============================================================================
// State 1: Initial
// =============================================================================
console.error('[SUITE] 1/3: Initial state');
await captureState('01-initial', 'Default view on load');

// =============================================================================
// State 2: With choices
// =============================================================================
console.error('[SUITE] 2/3: Arg with choices');
arg('Select item', [
  { name: 'Apple', value: 'apple' },
  { name: 'Banana', value: 'banana' },
  { name: 'Cherry', value: 'cherry' },
]);
await captureState('02-arg-choices', 'Arg prompt with choices');

// =============================================================================
// State 3: Div content
// =============================================================================
console.error('[SUITE] 3/3: Div content');
div(`
  <div class="p-6 space-y-4">
    <h1 class="text-2xl font-bold text-blue-400">Visual Test</h1>
    <p class="text-gray-300">Content for visual verification</p>
  </div>
`);
await captureState('03-div-content', 'Div prompt with HTML');

// =============================================================================
// Summary
// =============================================================================
console.error('[SUITE] All screenshots captured!');
console.error(`[SUITE] Output directory: ${SCREENSHOT_DIR}`);

process.exit(0);
```

---

## 7. Multi-State Visual Regression

### 7.1 Before/After Comparison

When fixing a visual bug, capture before and after states:

```typescript
// Save to test-screenshots/before/ before making changes
// Save to test-screenshots/after/ after making changes
// Compare manually or with image diff tools

const screenshotDir = join(process.cwd(), 'test-screenshots', 'before'); // or 'after'
```

### 7.2 Verification Checklist Pattern

```typescript
// At the end of your test, output a verification checklist
console.error('[VERIFY] Visual verification checklist:');
console.error('[VERIFY]   [ ] Component visible at expected position');
console.error('[VERIFY]   [ ] Colors match theme');
console.error('[VERIFY]   [ ] Text is readable');
console.error('[VERIFY]   [ ] No layout overflow');
console.error('[VERIFY]   [ ] Footer visible (40px height)');
console.error('[VERIFY]   [ ] Expected window height');
```

---

## 8. Debugging Visual Issues

### 8.1 Log Filtering

```bash
# Filter for resize events
echo '{"type":"run","path":"..."}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1 | \
  grep -iE 'RESIZE|height_for_view|bounds'

# Filter for layout events
grep -iE 'layout|flex|size' ~/.scriptkit/logs/script-kit-gpui.jsonl | tail -50

# Filter for specific component
grep -i 'editor' ~/.scriptkit/logs/script-kit-gpui.jsonl | tail -20
```

### 8.2 Expected Log Patterns

**Good signs (editor height):**
```
height_for_view(EditorPrompt) = 700
Resize: 501 -> 700
```

**Good signs (arg with choices):**
```
height_for_view(ArgPrompt) = 500
```

**Good signs (compact input):**
```
height_for_view(ArgPrompt) = 120
```

### 8.3 Common Issues and Fixes

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| Screenshot blank | Captured before render | Increase wait time to 500-1000ms |
| Wrong window size | Height calculation bug | Check `height_for_view()` in logs |
| Content cut off | Missing `overflow_hidden` | Add `.overflow_hidden()` to container |
| Layout not filling | Missing `flex_1()` | Add `.flex_1()` or `.h_full()` |
| Colors wrong | Hardcoded colors | Use `theme.colors.*` |

---

## 9. Anti-Patterns

### What NOT to Do

| Anti-Pattern | Why It's Wrong | Correct Approach |
|--------------|----------------|------------------|
| `./target/debug/script-kit-gpui test.ts` | CLI args don't work | Use stdin JSON protocol |
| "I captured a screenshot" (but didn't read it) | No verification | Use Read tool on the PNG file |
| "The user should test this" | AI should verify autonomously | Run the test yourself |
| Using `screencapture` or system tools | Captures wrong content | Use SDK `captureScreenshot()` |
| Not waiting for render | Screenshot may be blank/incomplete | Wait 500-1000ms |
| Guessing at layout | Invisible bugs go undetected | Use grid overlay + getLayoutInfo() |
| Running without `SCRIPT_KIT_AI_LOG=1` | Wastes tokens on verbose logs | Always use AI log mode |
| Skipping verification gate | Broken code gets committed | Run check/clippy/test |

### The Verification Hierarchy

```
NOT ACCEPTABLE:  "I made the change, please verify"
NOT ACCEPTABLE:  "I captured a screenshot" (but didn't read it)
MINIMUM:         Screenshot captured AND read AND analyzed
BETTER:          Screenshot + getLayoutInfo() + log analysis
BEST:            Screenshot + layout + bounds verification + regression comparison
```

---

## 10. Real-World Examples

### 10.1 Editor Height Bug Fix

**Problem:** Editor wasn't filling the 700px window.

**Testing approach:**
1. Create test script that opens editor with content
2. Wait for render
3. Capture screenshot
4. Get window bounds
5. Compare expected (700px) vs actual
6. Read screenshot to verify visual fill

```typescript
// test-editor-visual-fill.ts
const EXPECTED = 700;
const bounds = await getWindowBounds();
const diff = Math.abs(bounds.height - EXPECTED);

if (diff > 20) {
  console.error(`[FAIL] Editor height ${bounds.height} != ${EXPECTED}`);
} else {
  console.error(`[PASS] Editor height correct`);
}
```

### 10.2 Footer Visual Regression

**Problem:** Footer wasn't appearing consistently across views.

**Testing approach:**
1. Capture screenshots of multiple views (main menu, arg, div, editor)
2. Output verification checklist
3. Manual comparison of before/after

```typescript
// test-footer-visual-regression.ts
await capture('01-main-menu-footer');

arg('Select', ['A', 'B', 'C']);
await capture('02-arg-footer');

div(`<div>Content</div>`);
await capture('03-div-footer');

editor('code', 'typescript');
await capture('04-editor-footer');

console.error('[VERIFY] Footer should be visible in all 4 screenshots');
```

### 10.3 Vibrancy Effect Testing

**Problem:** Verifying semi-transparent blur effect looks correct.

**Testing approach:**
1. Show UI with various opacity levels
2. Capture screenshot
3. Visual inspection for blur quality

```typescript
await div(`
  <div class="flex flex-col gap-4 p-8">
    <div class="text-white text-2xl font-bold">Vibrancy Test</div>
    <div class="p-4 bg-white/10 rounded-lg">10% white</div>
    <div class="p-4 bg-black/30 rounded-lg">30% black</div>
  </div>
`);
await capture('vibrancy-test');
```

---

## Summary Checklist

For every UI change, verify:

- [ ] Test script created in `tests/smoke/test-*-visual.ts`
- [ ] Uses stdin JSON protocol (not CLI args)
- [ ] Waits for render (500-1000ms)
- [ ] Captures screenshot with `captureScreenshot()`
- [ ] Saves to `test-screenshots/` directory
- [ ] Logs screenshot path with `[SCREENSHOT]` prefix
- [ ] Uses `SCRIPT_KIT_AI_LOG=1`
- [ ] Screenshot file actually READ and analyzed
- [ ] Layout verified (bounds, heights match expected)
- [ ] Exit cleanly with `process.exit(0)`

---

## Quick Reference

```bash
# Build and run visual test
cargo build && echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-visual.ts"}' | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Show grid overlay then run test
(echo '{"type":"showGrid","showBounds":true}'; \
 echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-my-visual.ts"}') | \
  SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1

# Filter logs for visual events
grep -iE 'RESIZE|bounds|height|SCREENSHOT' output.log
```

```typescript
// Minimal screenshot capture
import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

await div(`<div>Test</div>`);
await new Promise(r => setTimeout(r, 500));

const shot = await captureScreenshot();
const dir = join(process.cwd(), 'test-screenshots');
mkdirSync(dir, { recursive: true });
writeFileSync(join(dir, `test-${Date.now()}.png`), Buffer.from(shot.data, 'base64'));

process.exit(0);
```
