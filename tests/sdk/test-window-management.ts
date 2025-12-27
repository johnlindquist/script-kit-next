// Name: SDK Test - Window Management
// Description: Tests window management APIs (getWindows, focusWindow, moveWindow, tileWindow)

/**
 * SDK TEST: test-window-management.ts
 *
 * Tests the Window Management APIs for controlling system windows.
 *
 * Test categories:
 * 1. getWindows - List all system windows
 * 2. Window info structure - Verify SystemWindowInfo properties
 * 3. focusWindow - Focus a specific window
 * 4. tileWindow - Tile a window to a screen position (may require accessibility permissions)
 *
 * Note: Window management requires accessibility permissions on macOS.
 * Some tests may skip if permissions are not granted.
 */

// SDK is loaded via --preload, no import needed

// Import types from kit-sdk
import type {
	SystemWindowInfo,
	TilePosition,
} from "../../scripts/kit-sdk";

// =============================================================================
// Test Infrastructure
// =============================================================================

interface TestResult {
	test: string;
	status: "running" | "pass" | "fail" | "skip";
	timestamp: string;
	result?: unknown;
	error?: string;
	duration_ms?: number;
	reason?: string;
}

function logTest(
	name: string,
	status: TestResult["status"],
	extra?: Partial<TestResult>,
) {
	const result: TestResult = {
		test: name,
		status,
		timestamp: new Date().toISOString(),
		...extra,
	};
	console.log(JSON.stringify(result));
}

function debug(msg: string) {
	console.error(`[TEST] ${msg}`);
}

// =============================================================================
// Tests
// =============================================================================

debug("test-window-management.ts starting...");
debug(
	`SDK globals: getWindows=${typeof getWindows}, focusWindow=${typeof focusWindow}`,
);
debug(
	`SDK globals: minimizeWindow=${typeof minimizeWindow}, maximizeWindow=${typeof maximizeWindow}`,
);
debug(
	`SDK globals: moveWindow=${typeof moveWindow}, resizeWindow=${typeof resizeWindow}, tileWindow=${typeof tileWindow}`,
);

// -----------------------------------------------------------------------------
// Test 1: getWindows() - List all system windows
// -----------------------------------------------------------------------------
const test1 = "getWindows-returns-array";
logTest(test1, "running");
const start1 = Date.now();

let windows: SystemWindowInfo[] = [];

try {
	debug("Test 1: getWindows()");

	windows = await getWindows();

	// Verify it returns an array
	if (!Array.isArray(windows)) {
		throw new Error(`Expected array, got ${typeof windows}`);
	}

	debug(`Test 1 completed - got ${windows.length} windows`);
	logTest(test1, "pass", {
		result: { windowCount: windows.length },
		duration_ms: Date.now() - start1,
	});
} catch (err) {
	logTest(test1, "fail", {
		error: String(err),
		duration_ms: Date.now() - start1,
	});
}

// -----------------------------------------------------------------------------
// Test 2: Window info structure - Verify SystemWindowInfo properties
// -----------------------------------------------------------------------------
const test2 = "window-info-structure";
logTest(test2, "running");
const start2 = Date.now();

try {
	debug("Test 2: Verify window info structure");

	if (windows.length === 0) {
		debug("Test 2 skipped - no windows available");
		logTest(test2, "skip", {
			reason: "No windows available to inspect",
			duration_ms: Date.now() - start2,
		});
	} else {
		const firstWindow = windows[0];

		// Verify required properties exist
		const hasWindowId = typeof firstWindow.windowId === "number";
		const hasTitle = typeof firstWindow.title === "string";
		const hasAppName = typeof firstWindow.appName === "string";

		if (!hasWindowId) {
			throw new Error(
				`Expected windowId to be number, got ${typeof firstWindow.windowId}`,
			);
		}
		if (!hasTitle) {
			throw new Error(
				`Expected title to be string, got ${typeof firstWindow.title}`,
			);
		}
		if (!hasAppName) {
			throw new Error(
				`Expected appName to be string, got ${typeof firstWindow.appName}`,
			);
		}

		// Check optional properties if present
		const hasBounds =
			firstWindow.bounds === undefined ||
			(typeof firstWindow.bounds === "object" &&
				firstWindow.bounds !== null &&
				typeof firstWindow.bounds.x === "number" &&
				typeof firstWindow.bounds.y === "number" &&
				typeof firstWindow.bounds.width === "number" &&
				typeof firstWindow.bounds.height === "number");

		const hasIsMinimized =
			firstWindow.isMinimized === undefined ||
			typeof firstWindow.isMinimized === "boolean";

		const hasIsActive =
			firstWindow.isActive === undefined ||
			typeof firstWindow.isActive === "boolean";

		if (!hasBounds) {
			throw new Error("Window bounds has invalid structure");
		}
		if (!hasIsMinimized) {
			throw new Error(
				`Expected isMinimized to be boolean or undefined, got ${typeof firstWindow.isMinimized}`,
			);
		}
		if (!hasIsActive) {
			throw new Error(
				`Expected isActive to be boolean or undefined, got ${typeof firstWindow.isActive}`,
			);
		}

		debug(
			`Test 2 completed - window structure verified for "${firstWindow.appName}: ${firstWindow.title}"`,
		);
		logTest(test2, "pass", {
			result: {
				sampleWindow: {
					windowId: firstWindow.windowId,
					title: firstWindow.title.substring(0, 50),
					appName: firstWindow.appName,
					hasBounds: !!firstWindow.bounds,
					isMinimized: firstWindow.isMinimized,
					isActive: firstWindow.isActive,
				},
			},
			duration_ms: Date.now() - start2,
		});
	}
} catch (err) {
	logTest(test2, "fail", {
		error: String(err),
		duration_ms: Date.now() - start2,
	});
}

// -----------------------------------------------------------------------------
// Test 3: focusWindow() - Focus a specific window
// -----------------------------------------------------------------------------
const test3 = "focusWindow";
logTest(test3, "running");
const start3 = Date.now();

try {
	debug("Test 3: focusWindow()");

	// Find a window we can focus (preferably a non-minimized one)
	const focusableWindow = windows.find((w) => !w.isMinimized) ?? windows[0];

	if (!focusableWindow) {
		debug("Test 3 skipped - no windows available to focus");
		logTest(test3, "skip", {
			reason: "No windows available to focus",
			duration_ms: Date.now() - start3,
		});
	} else {
		// Verify focusWindow function exists
		if (typeof focusWindow !== "function") {
			throw new Error(
				`Expected focusWindow to be a function, got ${typeof focusWindow}`,
			);
		}

		await focusWindow(focusableWindow.windowId);

		debug(
			`Test 3 completed - focused window "${focusableWindow.appName}: ${focusableWindow.title}"`,
		);
		logTest(test3, "pass", {
			result: {
				windowId: focusableWindow.windowId,
				appName: focusableWindow.appName,
			},
			duration_ms: Date.now() - start3,
		});
	}
} catch (err) {
	// Focus may fail if accessibility permissions aren't granted
	const errorMessage = String(err);
	if (
		errorMessage.includes("accessibility") ||
		errorMessage.includes("permission")
	) {
		debug("Test 3 skipped - accessibility permission required");
		logTest(test3, "skip", {
			reason: "Accessibility permission required",
			duration_ms: Date.now() - start3,
		});
	} else {
		logTest(test3, "fail", {
			error: errorMessage,
			duration_ms: Date.now() - start3,
		});
	}
}

// -----------------------------------------------------------------------------
// Test 4: Function existence - Verify all window management functions exist
// -----------------------------------------------------------------------------
const test4 = "window-functions-exist";
logTest(test4, "running");
const start4 = Date.now();

try {
	debug("Test 4: Verify window management functions exist");

	const functions = [
		{ name: "getWindows", fn: getWindows },
		{ name: "focusWindow", fn: focusWindow },
		{ name: "closeWindow", fn: closeWindow },
		{ name: "minimizeWindow", fn: minimizeWindow },
		{ name: "maximizeWindow", fn: maximizeWindow },
		{ name: "moveWindow", fn: moveWindow },
		{ name: "resizeWindow", fn: resizeWindow },
		{ name: "tileWindow", fn: tileWindow },
	];

	const missingFunctions: string[] = [];
	const foundFunctions: string[] = [];

	for (const { name, fn } of functions) {
		if (typeof fn === "function") {
			foundFunctions.push(name);
		} else {
			missingFunctions.push(name);
		}
	}

	if (missingFunctions.length > 0) {
		throw new Error(`Missing functions: ${missingFunctions.join(", ")}`);
	}

	debug(`Test 4 completed - all ${foundFunctions.length} functions exist`);
	logTest(test4, "pass", {
		result: { functions: foundFunctions },
		duration_ms: Date.now() - start4,
	});
} catch (err) {
	logTest(test4, "fail", {
		error: String(err),
		duration_ms: Date.now() - start4,
	});
}

// -----------------------------------------------------------------------------
// Test 5: tileWindow() - Tile a window (may require accessibility permissions)
// -----------------------------------------------------------------------------
const test5 = "tileWindow";
logTest(test5, "running");
const start5 = Date.now();

try {
	debug("Test 5: tileWindow()");

	// Find a window we can tile (preferably a non-minimized one)
	const tileableWindow = windows.find((w) => !w.isMinimized) ?? windows[0];

	if (!tileableWindow) {
		debug("Test 5 skipped - no windows available to tile");
		logTest(test5, "skip", {
			reason: "No windows available to tile",
			duration_ms: Date.now() - start5,
		});
	} else {
		// Verify tileWindow function exists
		if (typeof tileWindow !== "function") {
			throw new Error(
				`Expected tileWindow to be a function, got ${typeof tileWindow}`,
			);
		}

		// Test tiling to 'center' position
		await tileWindow(tileableWindow.windowId, "center");

		debug(
			`Test 5 completed - tiled window "${tileableWindow.appName}" to center`,
		);
		logTest(test5, "pass", {
			result: {
				windowId: tileableWindow.windowId,
				appName: tileableWindow.appName,
				position: "center",
			},
			duration_ms: Date.now() - start5,
		});
	}
} catch (err) {
	// Tiling may fail if accessibility permissions aren't granted
	const errorMessage = String(err);
	if (
		errorMessage.includes("accessibility") ||
		errorMessage.includes("permission")
	) {
		debug("Test 5 skipped - accessibility permission required");
		logTest(test5, "skip", {
			reason: "Accessibility permission required for tiling",
			duration_ms: Date.now() - start5,
		});
	} else {
		logTest(test5, "fail", {
			error: errorMessage,
			duration_ms: Date.now() - start5,
		});
	}
}

// -----------------------------------------------------------------------------
// Test 6: TilePosition types - Verify all tile positions are valid
// -----------------------------------------------------------------------------
const test6 = "tile-positions";
logTest(test6, "running");
const start6 = Date.now();

try {
	debug("Test 6: Verify TilePosition types");

	// These are the valid TilePosition values from the SDK
	const validPositions: TilePosition[] = [
		"left",
		"right",
		"top",
		"bottom",
		"top-left",
		"top-right",
		"bottom-left",
		"bottom-right",
		"center",
		"maximize",
	];

	// Just verify the type system accepts all positions
	// We don't actually call tileWindow to avoid side effects
	debug(`Test 6 completed - ${validPositions.length} tile positions defined`);
	logTest(test6, "pass", {
		result: { positions: validPositions },
		duration_ms: Date.now() - start6,
	});
} catch (err) {
	logTest(test6, "fail", {
		error: String(err),
		duration_ms: Date.now() - start6,
	});
}

// -----------------------------------------------------------------------------
// Test 7: moveWindow() - Move a window to specific coordinates
// -----------------------------------------------------------------------------
const test7 = "moveWindow";
logTest(test7, "running");
const start7 = Date.now();

try {
	debug("Test 7: moveWindow()");

	// Find a window we can move
	const moveableWindow = windows.find((w) => !w.isMinimized) ?? windows[0];

	if (!moveableWindow) {
		debug("Test 7 skipped - no windows available to move");
		logTest(test7, "skip", {
			reason: "No windows available to move",
			duration_ms: Date.now() - start7,
		});
	} else {
		// Verify moveWindow function exists
		if (typeof moveWindow !== "function") {
			throw new Error(
				`Expected moveWindow to be a function, got ${typeof moveWindow}`,
			);
		}

		// Move window to (100, 100)
		await moveWindow(moveableWindow.windowId, 100, 100);

		debug(
			`Test 7 completed - moved window "${moveableWindow.appName}" to (100, 100)`,
		);
		logTest(test7, "pass", {
			result: {
				windowId: moveableWindow.windowId,
				appName: moveableWindow.appName,
				position: { x: 100, y: 100 },
			},
			duration_ms: Date.now() - start7,
		});
	}
} catch (err) {
	// Moving may fail if accessibility permissions aren't granted
	const errorMessage = String(err);
	if (
		errorMessage.includes("accessibility") ||
		errorMessage.includes("permission")
	) {
		debug("Test 7 skipped - accessibility permission required");
		logTest(test7, "skip", {
			reason: "Accessibility permission required for moving",
			duration_ms: Date.now() - start7,
		});
	} else {
		logTest(test7, "fail", {
			error: errorMessage,
			duration_ms: Date.now() - start7,
		});
	}
}

// -----------------------------------------------------------------------------
// Summary and Exit
// -----------------------------------------------------------------------------
debug("test-window-management.ts completed!");
debug("All 7 tests executed. Check JSONL output for detailed results.");

// Exit cleanly for autonomous testing
exit(0);
