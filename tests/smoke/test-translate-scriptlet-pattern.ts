import '../../scripts/kit-sdk';

// Test the pattern used in AI Text Tools scriptlets
// This simulates what "Translate to English" does when accessibility fails
const test = 'translate-scriptlet-pattern';

function log(status: string, extra: any = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

log('running');
const start = Date.now();

// Simulate the scriptlet pattern with try/catch
let text: string | undefined;
try {
  text = await getSelectedText();
} catch {
  // Fall through - text will be undefined (e.g., accessibility permission denied)
}

if (!text?.trim()) {
  // This is the expected path when accessibility fails
  await hud('No text selected');
  log('pass', {
    result: 'correctly showed hud and exits cleanly',
    duration_ms: Date.now() - start
  });
  process.exit(0);
}

// If we got text (unlikely in test), still pass
log('pass', {
  result: 'got text successfully',
  text_length: text.length,
  duration_ms: Date.now() - start
});
process.exit(0);
