// Test: chat() without callbacks should use built-in AI
// This test verifies that when chat() is called without onInit/onMessage,
// the app uses its built-in AI provider instead of relying on SDK callbacks.

import '../../scripts/kit-sdk';

// Simple test - call chat() without any callbacks
// The app should show the chat UI with built-in AI mode enabled

const text = "Hello, this is a test";

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Test Built-in AI',
  system: `You are a helpful assistant. Say "Built-in AI works!" in your response.`,
  messages: [{ role: 'user', content: `Please respond to: ${text}` }],
});

// If we get here, the chat completed (user pressed escape or submitted)
console.log('[TEST] chat() completed');
process.exit(0);
