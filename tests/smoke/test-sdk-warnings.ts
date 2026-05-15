// Test explicit unsupported SDK behavior and dispatch-receipt feedback calls.
import '../../scripts/kit-sdk';

console.log('[TEST] Starting SDK warnings test');

// Feedback dispatch calls should not warn or throw.
console.log('[TEST] Testing beep()...');
await beep();

console.log('[TEST] Testing say()...');
await say('test');

console.log('[TEST] Testing notify()...');
await notify('test notification');

console.log('[TEST] Testing setStatus() - should return unsupported result...');
const statusResult = await setStatus({ status: 'busy', message: 'test status' });
console.log('[TEST] setStatus() result:', JSON.stringify(statusResult));

console.log('[TEST] Testing menu() - should return unsupported result...');
const menuResult = await menu('star');
console.log('[TEST] menu() result:', JSON.stringify(menuResult));

console.log('[TEST] Testing keyboard.type() - should throw...');
try {
  await keyboard.type('test');
  console.error('[TEST] ERROR: keyboard.type() should have thrown!');
} catch (err) {
  console.log('[TEST] keyboard.type() correctly threw:', (err as Error).message);
}

console.log('[TEST] Testing mouse.leftClick() - should throw...');
try {
  await mouse.leftClick();
  console.error('[TEST] ERROR: mouse.leftClick() should have thrown!');
} catch (err) {
  console.log('[TEST] mouse.leftClick() correctly threw:', (err as Error).message);
}

console.log('[TEST] Testing setPanel()...');
setPanel('<div>test</div>');

console.log('[TEST] Testing mini()...');
// This will auto-submit in test mode
const miniResult = await mini('Pick', ['A', 'B']);
console.log('[TEST] mini() returned:', miniResult);

// Test removed functions (should throw errors)
console.log('[TEST] Testing webcam() - should throw...');
try {
  await webcam();
  console.error('[TEST] ERROR: webcam() should have thrown!');
} catch (err) {
  console.log('[TEST] webcam() correctly threw:', (err as Error).message);
}

console.log('[TEST] Testing mic() - should throw...');
try {
  await mic();
  console.error('[TEST] ERROR: mic() should have thrown!');
} catch (err) {
  console.log('[TEST] mic() correctly threw:', (err as Error).message);
}

console.log('[TEST] Testing eyeDropper() - should throw...');
try {
  await eyeDropper();
  console.error('[TEST] ERROR: eyeDropper() should have thrown!');
} catch (err) {
  console.log('[TEST] eyeDropper() correctly threw:', (err as Error).message);
}

console.log('[TEST] SDK warnings test complete');
process.exit(0);
