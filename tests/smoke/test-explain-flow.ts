// Test script that mimics the "Explain This" scriptlet flow
// This simulates: getSelectedText() -> chat()
import '../../scripts/kit-sdk';

console.error('[TEST] Starting explain flow test');

// Step 1: Hide window (like getSelectedText does)
console.error('[TEST] Step 1: Hiding window');
await hide();

// Small delay to simulate getting selected text
await new Promise(r => setTimeout(r, 200));

// Step 2: Show chat with pre-populated message
console.error('[TEST] Step 2: Showing chat');
const fakeSelectedText = "Hello world - this is fake selected text for testing";

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Explain This Test',
  system: `You are a helpful assistant. Explain the following text in simple terms. Just say "Explanation: [brief explanation]" as your response.`,
  messages: [{ role: 'user', content: `Explain this:\n\n${fakeSelectedText}` }],
});

console.error('[TEST] chat() completed - this should not print if script was killed');
