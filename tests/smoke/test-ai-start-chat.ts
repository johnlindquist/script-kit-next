import '../../scripts/kit-sdk';

/**
 * Test the aiStartChat() and aiFocus() SDK functions.
 * This mimics the pattern used by the translate scriptlet.
 *
 * Run with: echo '{"type":"run","path":"'"$(pwd)"'/tests/smoke/test-ai-start-chat.ts"}' | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1
 */

const test = 'ai-start-chat';

function log(status: string, extra: any = {}) {
  console.error(JSON.stringify({
    test,
    status,
    timestamp: new Date().toISOString(),
    ...extra
  }));
}

log('running');

const start = Date.now();

try {
  // Test 1: aiStartChat should not throw "Unhandled message type"
  log('calling_aiStartChat');
  const result = await aiStartChat('Hello AI! Please respond with "Test successful"', {
    systemPrompt: 'You are a helpful assistant. Respond briefly.',
    noResponse: false,  // We want AI to respond
  });

  log('aiStartChat_result', {
    chatId: result.chatId,
    title: result.title,
    modelId: result.modelId,
    provider: result.provider,
    streamingStarted: result.streamingStarted,
  });

  // Test 2: aiFocus should work
  log('calling_aiFocus');
  const focusResult = await aiFocus();

  log('aiFocus_result', {
    wasOpen: focusResult.wasOpen,
  });

  log('pass', {
    duration_ms: Date.now() - start,
    result: 'Both aiStartChat and aiFocus completed successfully'
  });

  // Give a moment for user to see the AI window
  await new Promise(r => setTimeout(r, 2000));

} catch (e: any) {
  log('fail', {
    error: e.message,
    stack: e.stack,
    duration_ms: Date.now() - start
  });
}

exit(0);
