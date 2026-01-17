// Simple test: chat() without getSelectedText
// This tests if the built-in AI mode works when called directly

import '../../scripts/kit-sdk';

console.error('[TEST] Starting simple chat test');

await chat({
  placeholder: 'Ask follow-up...',
  hint: 'Simple Chat Test',
  system: `You are a helpful assistant. Just say "Hello! Built-in AI is working!" as your response.`,
  messages: [{ role: 'user', content: `Say hello!` }],
});

console.error('[TEST] chat() completed');
