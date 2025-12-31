// Name: Arg Prompt Resize Scenarios
// Description: Verifies arg prompt resizing across multiple choice counts and inputs

import '../../scripts/kit-sdk';
import { writeFileSync, mkdirSync } from 'fs';
import { join } from 'path';

const filterChoices = [
  { name: 'cat-1', value: 'cat-1' },
  { name: 'cat-2', value: 'cat-2' },
  { name: 'cat-3', value: 'cat-3' },
  { name: 'car-1', value: 'car-1' },
  { name: 'car-2', value: 'car-2' },
  { name: 'dog-1', value: 'dog-1' },
  { name: 'dog-2', value: 'dog-2' },
  { name: 'eel-1', value: 'eel-1' },
  { name: 'fox-1', value: 'fox-1' },
  { name: 'gnu-1', value: 'gnu-1' },
];

const twoChoices = [
  { name: 'First', value: 'First' },
  { name: 'Second', value: 'Second' },
];

const largeChoices = [
  ...Array.from({ length: 10 }, (_, i) => ({
    name: `alpha-${String(i + 1).padStart(2, '0')}`,
    value: `alpha-${String(i + 1).padStart(2, '0')}`,
  })),
  ...Array.from({ length: 10 }, (_, i) => ({
    name: `beta-${String(i + 1).padStart(2, '0')}`,
    value: `beta-${String(i + 1).padStart(2, '0')}`,
  })),
  ...Array.from({ length: 10 }, (_, i) => ({
    name: `gamma-${String(i + 1).padStart(2, '0')}`,
    value: `gamma-${String(i + 1).padStart(2, '0')}`,
  })),
];

const longLabelChoices = [
  {
    name: 'Choice with a very long label that should remain a single row in the list',
    value: 'long-1',
  },
  {
    name: 'Another extremely long label to verify fixed row height with overflow text',
    value: 'long-2',
  },
  { name: 'Short', value: 'short' },
];

const runId = Date.now();
const screenshotDir = join(process.cwd(), '.test-screenshots');
mkdirSync(screenshotDir, { recursive: true });

const wait = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

async function capture(label: string) {
  const screenshot = await captureScreenshot();
  const filename = `arg-dynamic-filter-${runId}-${label}.png`;
  const filepath = join(screenshotDir, filename);
  writeFileSync(filepath, Buffer.from(screenshot.data, 'base64'));
  console.error(`[SCREENSHOT] ${filepath}`);
}

console.error('[SMOKE] Starting arg prompt resize scenarios...');

// Scenario 1: Filter 10 -> 5 -> 3 -> 1 -> 0 -> 10
const filterPromise = arg('Type anything', filterChoices);

await wait(400);
await capture('scenario-01-filter-initial-10');

setInput('ca');
await wait(400);
await capture('scenario-01-filter-5');

setInput('cat');
await wait(400);
await capture('scenario-01-filter-3');

setInput('CAT');
await wait(400);
await capture('scenario-01-filter-3-upper');

setInput('cat-2');
await wait(400);
await capture('scenario-01-filter-1');

setInput('cat ');
await wait(400);
await capture('scenario-01-filter-0-trailing-space');

setInput('');
await wait(400);
await capture('scenario-01-filter-restore-10');

submit('cat-1');
await filterPromise;

// Scenario 2: No choices (input-only height)
const emptyPromise = arg('Type anything', []);

await wait(400);
await capture('scenario-02-empty-initial-0');

setInput('hello');
await wait(400);
await capture('scenario-02-empty-input-text');

setInput('   ');
await wait(400);
await capture('scenario-02-empty-whitespace');

setInput('');
await wait(400);
await capture('scenario-02-empty-clear');

submit('hello');
await emptyPromise;

// Scenario 3: Two choices (tiny list)
const twoPromise = arg('Pick one', twoChoices);

await wait(400);
await capture('scenario-03-two-initial-2');

setInput('Second');
await wait(400);
await capture('scenario-03-two-filter-1');

setInput('');
await wait(400);
await capture('scenario-03-two-restore-2');

submit('First');
await twoPromise;

// Scenario 4: Large list (cap height) then shrink
const largePromise = arg('Pick from many', largeChoices);

await wait(400);
await capture('scenario-04-large-initial-30');

setInput('beta');
await wait(400);
await capture('scenario-04-large-filter-10');

setInput('alpha-0');
await wait(400);
await capture('scenario-04-large-filter-9');

setInput('beta-1');
await wait(400);
await capture('scenario-04-large-filter-2');

setInput('beta-10');
await wait(400);
await capture('scenario-04-large-filter-1');

setInput('');
await wait(400);
await capture('scenario-04-large-restore-30');

submit('beta-01');
await largePromise;

// Scenario 5: Long labels should not affect row height
const longPromise = arg('Long labels', longLabelChoices);

await wait(400);
await capture('scenario-05-long-initial-3');

setInput('long');
await wait(400);
await capture('scenario-05-long-filter-2');

setInput('Short');
await wait(400);
await capture('scenario-05-long-filter-1');

setInput('');
await wait(400);
await capture('scenario-05-long-restore-3');

submit('short');
await longPromise;

process.exit(0);
