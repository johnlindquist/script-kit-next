// Schedule: every minute
// Name: Natural Language Schedule Test
// Description: Tests natural language scheduling - runs every minute

import '../../scripts/kit-sdk';
import { appendFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SCHEDULE TEST] Natural language scheduled script executed at:', new Date().toISOString());

// Ensure log directory exists
const logDir = join(process.env.HOME || '/tmp', '.kenv', 'logs');
try {
  mkdirSync(logDir, { recursive: true });
} catch (e) {
  // Directory may already exist
}

// Write to log file to prove execution
const logPath = join(logDir, 'schedule-natural-test.log');
const logEntry = `[NATURAL] Executed: ${new Date().toISOString()}\n`;

try {
  appendFileSync(logPath, logEntry);
  console.error('[SCHEDULE TEST] Log written to:', logPath);
} catch (e) {
  console.error('[SCHEDULE TEST] Failed to write log:', e);
}

// Display notification that script ran
await div(md(`
# Natural Language Schedule Test

This script was triggered using natural language scheduling.

**Schedule Expression:** \`every minute\`

**Executed at:** ${new Date().toISOString()}

**Log file:** \`${logPath}\`

---

This window will close in 3 seconds...
`));

// Auto-close after 3 seconds
setTimeout(() => {
  console.error('[SCHEDULE TEST] Auto-closing after display');
  process.exit(0);
}, 3000);
