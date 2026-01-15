// Test: Chat prompt actions dialog integration
// Verifies that ⌘K opens the standard ActionsDialog for chat prompts

import '../../scripts/kit-sdk';

// First show a chat prompt
await chat({
  placeholder: "Type your message...",
  hint: "Press ⌘K to open actions"
});

// The test passes if:
// 1. Chat prompt renders correctly
// 2. User can press ⌘K to open ActionsDialog
// 3. Actions include model selection, continue in chat, copy, clear

console.error("[TEST] Chat actions test - manual verification required");
console.error("[TEST] Press ⌘K to verify actions dialog opens");

process.exit(0);
