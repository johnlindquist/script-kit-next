# Scenario-Based Automated Testing

When debugging complex flows (hotkey → hide → chat), don't ask the user to test. Instead:

1. Write a test script that mimics the real scenario
2. Create a shell script that builds, runs, and checks logs
3. Iterate autonomously until the fix works

## Example: Testing `getSelectedText()` → `chat()` flow

**Step 1: Create the test script** (`tests/smoke/test-explain-flow.ts`):
```ts
import '../../scripts/kit-sdk';

console.error('[TEST] Starting explain flow test');

await hide();
await new Promise(r => setTimeout(r, 200));

await chat({
  placeholder: 'Ask follow-up...',
  system: `You are helpful. Say "Working!" as your response.`,
  messages: [{ role: 'user', content: `Test message` }],
});

console.error('[TEST] chat() completed');
```

**Step 2: Create the test runner** (`scripts/test-explain-flow.sh`):
```bash
#!/bin/bash
set -e
cd /path/to/script-kit-gpui

echo "=== Building ==="
cargo build 2>&1 | tail -3

echo "=== Running test (15s timeout) ==="
TEST_PATH="$(pwd)/tests/smoke/test-explain-flow.ts"
timeout 15 bash -c "echo '{\"type\":\"run\",\"path\":\"$TEST_PATH\"}' | \
  RUST_LOG=info ./target/debug/script-kit-gpui 2>&1" > /tmp/test-output.txt || true

echo "=== Checking for key events ==="
grep -E "(HideWindow|ShowChat|initial_response)" /tmp/test-output.txt | head -20

echo "=== VERDICT ==="
if grep -q "Built-in AI initial response complete" /tmp/test-output.txt; then
    echo "SUCCESS: AI responded"
elif grep -q "Force-killing script" /tmp/test-output.txt; then
    echo "FAILURE: Script was killed"
else
    echo "UNKNOWN: Check logs"
fi
```

## Why This Works Better Than Manual Testing

| Manual Testing | Automated Scenario Testing |
|----------------|---------------------------|
| User must switch contexts | Agent runs test autonomously |
| "It didn't work" - no details | Logs show exact failure point |
| Requires user availability | Can iterate at any time |
| Hard to reproduce edge cases | Exact same scenario every time |

## Key Patterns

1. **Mimic the real flow exactly** - if bug involves `hide()` then `chat()`, test must call both
2. **Log key checkpoints** - `console.error('[TEST] Step X completed')`
3. **Check for success AND failure indicators** - grep for both
4. **Use timeout** - tests shouldn't hang forever
5. **Capture all output** - redirect to file for analysis
6. **Auto-detect verdict** - SUCCESS/FAILURE based on log patterns

## Existing Test Scripts

- `scripts/test-explain-flow.sh` - Tests `hide()` → `chat()` flow
- `tests/smoke/test-chat-simple.ts` - Basic chat auto-response
- `tests/smoke/test-explain-flow.ts` - Mimics "Explain This" scriptlet
