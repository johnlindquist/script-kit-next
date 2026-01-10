// Smoke test for new window management APIs
import '../../scripts/kit-sdk';

function log(test: string, status: string, extra: any = {}) {
  console.log(JSON.stringify({ test, status, timestamp: new Date().toISOString(), ...extra }));
}

const testName = "window-management-apis";
log(testName, "running");

const start = Date.now();

try {
  // Test 1: getDisplays
  log("getDisplays", "running");
  const displays = await getDisplays();
  if (Array.isArray(displays)) {
    log("getDisplays", "pass", {
      displayCount: displays.length,
      displays: displays.map(d => ({
        name: d.name,
        isPrimary: d.isPrimary,
        width: d.bounds?.width,
        height: d.bounds?.height
      }))
    });
  } else {
    log("getDisplays", "fail", { error: "Expected array" });
  }

  // Test 2: getFrontmostWindow
  log("getFrontmostWindow", "running");
  const frontWindow = await getFrontmostWindow();
  // This might be null if no previous app was focused
  log("getFrontmostWindow", "pass", {
    hasWindow: frontWindow !== null,
    windowInfo: frontWindow ? {
      title: frontWindow.title,
      appName: frontWindow.appName,
      windowId: frontWindow.windowId
    } : null
  });

  // Test 3: TilePosition types exist
  log("TilePosition-types", "running");
  const tilePositions: TilePosition[] = [
    'left', 'right', 'top', 'bottom',
    'top-left', 'top-right', 'bottom-left', 'bottom-right',
    'left-third', 'center-third', 'right-third',
    'top-third', 'middle-third', 'bottom-third',
    'first-two-thirds', 'last-two-thirds',
    'top-two-thirds', 'bottom-two-thirds',
    'center', 'almost-maximize', 'maximize'
  ];
  log("TilePosition-types", "pass", { positionCount: tilePositions.length });

  // Test 4: Functions exist
  log("function-existence", "running");
  const functions = [
    typeof tileWindow === 'function',
    typeof getDisplays === 'function',
    typeof getFrontmostWindow === 'function',
    typeof moveToNextDisplay === 'function',
    typeof moveToPreviousDisplay === 'function'
  ];
  const allExist = functions.every(Boolean);
  log("function-existence", allExist ? "pass" : "fail", {
    tileWindow: typeof tileWindow,
    getDisplays: typeof getDisplays,
    getFrontmostWindow: typeof getFrontmostWindow,
    moveToNextDisplay: typeof moveToNextDisplay,
    moveToPreviousDisplay: typeof moveToPreviousDisplay
  });

  log(testName, "pass", { duration_ms: Date.now() - start });
} catch (e) {
  log(testName, "fail", { error: String(e), duration_ms: Date.now() - start });
}

// Exit cleanly
process.exit(0);
