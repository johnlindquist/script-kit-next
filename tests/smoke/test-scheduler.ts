// Cron: * * * * *
// Name: Scheduler Test - Every Minute
// Description: Tests that cron scheduling works - runs every minute

import '../../scripts/kit-sdk';
import { appendFileSync, mkdirSync } from 'fs';
import { join } from 'path';

console.error('[SCHEDULER TEST] Cron-scheduled script executed at:', new Date().toISOString());

// Ensure log directory exists
const logDir = join(process.env.HOME || '/tmp', '.kenv', 'logs');
try {
  mkdirSync(logDir, { recursive: true });
} catch (e) {
  // Directory may already exist
}

// Write to log file to prove execution
const logPath = join(logDir, 'scheduler-test.log');
const logEntry = `[CRON] Executed: ${new Date().toISOString()}\n`;

try {
  appendFileSync(logPath, logEntry);
  console.error('[SCHEDULER TEST] Log written to:', logPath);
} catch (e) {
  console.error('[SCHEDULER TEST] Failed to write log:', e);
}

// Display notification that script ran
await div(md(`
# Cron Scheduler Test

This script was triggered by the cron scheduler.

**Cron Expression:** \`* * * * *\` (every minute)

**Executed at:** ${new Date().toISOString()}

**Log file:** \`${logPath}\`

---

This window will close in 3 seconds...
`));

// Auto-close after 3 seconds
setTimeout(() => {
  console.error('[SCHEDULER TEST] Auto-closing after display');
  process.exit(0);
}, 3000);
