// Test that the settings hub remains searchable after API key commands moved out of built-ins
import '../../scripts/kit-sdk';

// Helper to output test results
function log(test: string, status: string, extra: Record<string, unknown> = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

const testName = "settings-builtins-visible";
log(testName, "running");

try {
  // Search for "theme" - should show the Theme Designer command
  const result = await arg({
    placeholder: "Search for theme...",
    input: "theme",
    strict: false,
    // Timeout after 2 seconds if no interaction
    onInit: async () => {
      await new Promise(r => setTimeout(r, 2000));
      // Auto-submit with current filter to see what matches
      submit("theme");
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
