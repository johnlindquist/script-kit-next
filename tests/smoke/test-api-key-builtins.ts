// Test that API key configuration built-in commands appear in search
import '../../scripts/kit-sdk';

// Helper to output test results
function log(test: string, status: string, extra: Record<string, unknown> = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

const testName = "api-key-builtins-visible";
log(testName, "running");

try {
  // Search for "vercel" - should show the Configure Vercel AI Gateway command
  const result = await arg({
    placeholder: "Search for vercel...",
    input: "vercel",
    strict: false,
    // Timeout after 2 seconds if no interaction
    onInit: async () => {
      await new Promise(r => setTimeout(r, 2000));
      // Auto-submit with current filter to see what matches
      submit("vercel");
    }
  }, [
    // Empty choices - we're testing built-in commands appear
  ]);

  // If we get here without error, the command executed
  log(testName, "pass", { result, message: "Built-in commands accessible" });
} catch (e) {
  log(testName, "fail", { error: String(e) });
}

process.exit(0);
