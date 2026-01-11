import '../../scripts/kit-sdk';

/**
 * Quick test to verify accessibility permission status.
 * Run with: echo '{"type":"run","path":"tests/smoke/test-accessibility-check.ts"}' | ./target/debug/script-kit-gpui
 */

const test = 'accessibility-check';

function log(status: string, extra: any = {}) {
  console.error(JSON.stringify({
    test,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  }));
}

log('running');

// Check permission status
const hasPermission = await hasAccessibilityPermission();
log('permission_check', { hasPermission });

if (!hasPermission) {
  // Try to request permission (shows system dialog)
  log('requesting_permission');
  const granted = await requestAccessibilityPermission();
  log('permission_result', { granted });

  if (!granted) {
    await hud('Accessibility permission required');
    log('fail', { reason: 'Permission not granted' });
    exit(1);
  }
}

// Now try to get selected text
log('getting_selected_text');
try {
  const text = await getSelectedText();
  if (text.trim()) {
    log('pass', { result: 'Got selected text', text_length: text.length, preview: text.substring(0, 50) });
    await hud(`Got: "${text.substring(0, 30)}${text.length > 30 ? '...' : ''}"`);
  } else {
    log('pass', { result: 'No text selected (permission works, just nothing selected)' });
    await hud('No text selected');
  }
} catch (e: any) {
  log('fail', { error: e.message });
  await hud(`Error: ${e.message}`);
}

exit(0);
