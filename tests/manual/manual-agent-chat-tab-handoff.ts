// Manual helper for Agent Chat handoff from the main menu.
// Automated coverage should use scripts/agentic with getAcpState receipts.

import '../../scripts/kit-sdk';

// Helper to log test status
function log(test: string, status: string, extra: any = {}) {
  console.error(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

const testName = "manual-agent-chat-tab-handoff";

async function runTest() {
  log(testName, "running");
  const start = Date.now();

  try {
    // Note: This test primarily verifies that Agent Chat opens with the expected entry intent.
    // The active agent configuration is managed through config.ts.

    // Display a message about how to test
    await div(`
      <div class="p-4 flex flex-col gap-4">
        <h1 class="text-xl font-bold">Agent Chat Tab Handoff Test</h1>
        <p>To test the Agent Chat handoff:</p>
        <ol class="list-decimal ml-6 space-y-2">
          <li>Press Escape to close this prompt</li>
          <li>Type something in the main search bar (e.g., "hello")</li>
          <li>Press Tab</li>
          <li>Agent Chat should appear with your text as the initial query</li>
          <li>If an agent is configured, you can submit to get responses</li>
        </ol>
        <p class="text-sm text-gray-400 mt-4">
          Note: configure agents in config.ts before testing live responses.
        </p>
      </div>
    `);

    log(testName, "manual", {
      result: "Manual instructions shown",
      duration_ms: Date.now() - start
    });
  } catch (e) {
    log(testName, "fail", {
      error: String(e),
      duration_ms: Date.now() - start
    });
  }

  process.exit(0);
}

runTest();
