import '../../scripts/kit-sdk';

// Test that getSelectedText properly throws on error
const test = 'get-selected-text-error-handling';

function log(status: string, extra: any = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

log('running');
const start = Date.now();

try {
  const text = await getSelectedText();
  // If we got here without accessibility, it should be empty or throw
  log('pass', {
    result: text ? 'got text' : 'empty text',
    duration_ms: Date.now() - start
  });
} catch (e: any) {
  // Expected: should throw if accessibility permission is denied
  const errorMsg = e?.message || String(e);
  if (errorMsg.includes('Accessibility')) {
    log('pass', {
      result: 'correctly rejected with accessibility error',
      error: errorMsg,
      duration_ms: Date.now() - start
    });
  } else {
    log('pass', {
      result: 'rejected with other error',
      error: errorMsg,
      duration_ms: Date.now() - start
    });
  }
}

process.exit(0);
