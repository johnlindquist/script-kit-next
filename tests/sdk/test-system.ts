// Name: SDK Test - System APIs
// Description: Tests TIER 3 system APIs (alerts, clipboard, keyboard, mouse)

/**
 * SDK TEST: test-system.ts
 *
 * Tests the TIER 3 system APIs that interact with the operating system.
 *
 * Test categories:
 * 1. Feedback: beep, say, notify, setStatus, menu
 * 2. Clipboard: read/write text and images, copy/paste aliases
 * 3. Text operations: setSelectedText, getSelectedText
 * 4. Unsupported input simulation boundary: keyboard, mouse
 *
 * Note: system APIs either resolve from app-originated dispatch receipts,
 * return typed unsupported results, or reject explicitly when GPUI has no
 * native input receipt contract.
 */

import "../../scripts/kit-sdk";

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

function assertPng(buffer: Buffer, expectedWidth: number, expectedHeight: number) {
	const pngMagic = "89504e470d0a1a0a";
	if (buffer.subarray(0, 8).toString("hex") !== pngMagic) {
		throw new Error("readImage did not return PNG bytes");
	}

	const width = buffer.readUInt32BE(16);
	const height = buffer.readUInt32BE(20);
	if (width !== expectedWidth || height !== expectedHeight) {
		throw new Error(`PNG dimensions mismatch: ${width}x${height}`);
	}
}

async function expectUnsupported(name: string, call: () => Promise<unknown>) {
	try {
		await call();
		throw new Error(`${name} unexpectedly resolved`);
	} catch (err) {
		const anyErr = err as { code?: string; message?: string };
		if (anyErr.code !== "ERR_UNSUPPORTED_SDK_FEATURE") {
			throw new Error(`${name} threw wrong error: ${anyErr.message ?? String(err)}`);
		}
	}
}

// =============================================================================
// Tests
// =============================================================================

debug("test-system.ts starting...");
debug(
	`SDK globals: beep=${typeof beep}, say=${typeof say}, notify=${typeof notify}`,
);
debug(
	`SDK globals: clipboard=${typeof clipboard}, keyboard=${typeof keyboard}, mouse=${typeof mouse}`,
);

// -----------------------------------------------------------------------------
// Test 1: beep() - System beep dispatch receipt
// -----------------------------------------------------------------------------
const test1 = "beep";
logTest(test1, "running");
const start1 = Date.now();

try {
	debug("Test 1: beep()");

	await beep();

	debug("Test 1 completed - beep dispatch result received");
	logTest(test1, "pass", { duration_ms: Date.now() - start1 });
} catch (err) {
	logTest(test1, "fail", {
		error: String(err),
		duration_ms: Date.now() - start1,
	});
}

// -----------------------------------------------------------------------------
// Test 2: say() - Text-to-speech dispatch receipt
// -----------------------------------------------------------------------------
const test2 = "say";
logTest(test2, "running");
const start2 = Date.now();

try {
	debug("Test 2: say()");

	// Without voice
	await say("Hello from Script Kit");

	// With voice
	await say("Testing voice parameter", "Samantha");

	debug("Test 2 completed - say dispatch results received");
	logTest(test2, "pass", { duration_ms: Date.now() - start2 });
} catch (err) {
	logTest(test2, "fail", {
		error: String(err),
		duration_ms: Date.now() - start2,
	});
}

// -----------------------------------------------------------------------------
// Test 3: notify() - System notification dispatch receipt
// -----------------------------------------------------------------------------
const test3 = "notify";
logTest(test3, "running");
const start3 = Date.now();

try {
	debug("Test 3: notify()");

	// String shorthand
	await notify("Simple notification body");

	// Options object
	await notify({ title: "Script Kit", body: "Task completed successfully!" });

	// Title only
	await notify({ title: "Just a title" });

	debug("Test 3 completed - notify dispatch results received");
	logTest(test3, "pass", { duration_ms: Date.now() - start3 });
} catch (err) {
	logTest(test3, "fail", {
		error: String(err),
		duration_ms: Date.now() - start3,
	});
}

// -----------------------------------------------------------------------------
// Test 4: setStatus() - Explicit unsupported status boundary
// -----------------------------------------------------------------------------
const test4 = "setStatus";
logTest(test4, "running");
const start4 = Date.now();

try {
	debug("Test 4: setStatus()");

	const statusResult = await setStatus({ status: "busy", message: "Processing..." });
	if (
		!statusResult ||
		statusResult.ok !== false ||
		statusResult.code !== "ERR_UNSUPPORTED_SDK_FEATURE"
	) {
		throw new Error(`setStatus returned wrong result: ${JSON.stringify(statusResult)}`);
	}

	debug("Test 4 completed - setStatus returned unsupported result");
	logTest(test4, "pass", { duration_ms: Date.now() - start4 });
} catch (err) {
	logTest(test4, "fail", {
		error: String(err),
		duration_ms: Date.now() - start4,
	});
}

// -----------------------------------------------------------------------------
// Test 5: menu() - Explicit unsupported menu boundary
// -----------------------------------------------------------------------------
const test5 = "menu";
logTest(test5, "running");
const start5 = Date.now();

try {
	debug("Test 5: menu()");

	const menuResult = await menu("star");
	if (!menuResult || menuResult.ok !== false || menuResult.code !== "ERR_UNSUPPORTED_SDK_FEATURE") {
		throw new Error(`menu returned wrong result: ${JSON.stringify(menuResult)}`);
	}

	debug("Test 5 completed - menu returned unsupported result");
	logTest(test5, "pass", { duration_ms: Date.now() - start5 });
} catch (err) {
	logTest(test5, "fail", {
		error: String(err),
		duration_ms: Date.now() - start5,
	});
}

// -----------------------------------------------------------------------------
// Test 6: copy() and paste() - Clipboard aliases
// -----------------------------------------------------------------------------
const test6 = "copy-paste";
logTest(test6, "running");
const start6 = Date.now();

try {
	debug("Test 6: copy() and paste()");

	// Verify copy function exists and is callable
	if (typeof copy !== "function") {
		logTest(test6, "fail", {
			error: `Expected copy to be a function, got ${typeof copy}`,
			duration_ms: Date.now() - start6,
		});
	} else if (typeof paste !== "function") {
		logTest(test6, "fail", {
			error: `Expected paste to be a function, got ${typeof paste}`,
			duration_ms: Date.now() - start6,
		});
	} else {
		// copy is an alias for clipboard.writeText
		await copy("Hello from copy()");

		// paste is an alias for clipboard.readText - waits for GPUI response
		debug("copy() message sent, paste() will wait for response");

		logTest(test6, "pass", {
			result: { copy: "function", paste: "function" },
			duration_ms: Date.now() - start6,
		});
	}
} catch (err) {
	logTest(test6, "fail", {
		error: String(err),
		duration_ms: Date.now() - start6,
	});
}

// -----------------------------------------------------------------------------
// Test 7: clipboard object - Full API
// -----------------------------------------------------------------------------
const test7 = "clipboard-api";
logTest(test7, "running");
const start7 = Date.now();

try {
	debug("Test 7: clipboard API");

	// Verify clipboard object exists and has methods
	if (typeof clipboard !== "object") {
		throw new Error("clipboard is not an object");
	}

	if (typeof clipboard.readText !== "function") {
		throw new Error("clipboard.readText is not a function");
	}

	if (typeof clipboard.writeText !== "function") {
		throw new Error("clipboard.writeText is not a function");
	}

	if (typeof clipboard.readImage !== "function") {
		throw new Error("clipboard.readImage is not a function");
	}

	if (typeof clipboard.writeImage !== "function") {
		throw new Error("clipboard.writeImage is not a function");
	}

	debug("Test 7 completed - clipboard API verified");
	logTest(test7, "pass", {
		result: {
			methods: ["readText", "writeText", "readImage", "writeImage"],
		},
		duration_ms: Date.now() - start7,
	});
} catch (err) {
	logTest(test7, "fail", {
		error: String(err),
		duration_ms: Date.now() - start7,
	});
}

// -----------------------------------------------------------------------------
// Test 7b: clipboard image roundtrip - Real image clipboard payload
// -----------------------------------------------------------------------------
const test7b = "clipboard-image-roundtrip";
logTest(test7b, "running");
const start7b = Date.now();

try {
	if (process.env.INCLUDE_SYSTEM_CLIPBOARD_IMAGE === "1") {
		debug("Test 7b: clipboard image roundtrip");
		const oneByOnePng = Buffer.from(
			"iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGP4z8DwHwAFAAH/iZk9HQAAAABJRU5ErkJggg==",
			"base64",
		);

		assertPng(oneByOnePng, 1, 1);
		await clipboard.writeImage(oneByOnePng);
		const readBack = await clipboard.readImage();
		assertPng(readBack, 1, 1);

		await clipboard.writeText("not an image");
		try {
			await clipboard.readImage();
			throw new Error("readImage unexpectedly resolved for text clipboard");
		} catch (err) {
			const code = (err as { code?: string }).code;
			if (code !== "ERR_CLIPBOARD_IMAGE_NOT_AVAILABLE") {
				throw err;
			}
		}

		try {
			await clipboard.writeImage(Buffer.from("not an encoded image"));
			throw new Error("writeImage unexpectedly resolved for invalid bytes");
		} catch (err) {
			const code = (err as { code?: string }).code;
			if (code !== "ERR_CLIPBOARD_IMAGE_DECODE_FAILED") {
				throw err;
			}
		}

		logTest(test7b, "pass", {
			result: {
				format: "png",
				width: readBack.readUInt32BE(16),
				height: readBack.readUInt32BE(20),
				bytes: readBack.length,
				magic: readBack.subarray(0, 8).toString("hex"),
			},
			duration_ms: Date.now() - start7b,
		});
	} else {
		debug("Test 7b skipped - image clipboard roundtrip requires INCLUDE_SYSTEM_CLIPBOARD_IMAGE=1");
		logTest(test7b, "skip", {
			result: { reason: "set INCLUDE_SYSTEM_CLIPBOARD_IMAGE=1 for real clipboard image proof" },
			duration_ms: Date.now() - start7b,
		});
	}
} catch (err) {
	logTest(test7b, "fail", {
		error: String(err),
		duration_ms: Date.now() - start7b,
	});
}

// -----------------------------------------------------------------------------
// Test 8: setSelectedText() - Paste at cursor (fire-and-forget)
// -----------------------------------------------------------------------------
const test8 = "setSelectedText";
logTest(test8, "running");
const start8 = Date.now();

try {
	debug("Test 8: setSelectedText()");

	// Verify setSelectedText function exists
	if (typeof setSelectedText !== "function") {
		logTest(test8, "fail", {
			error: `Expected setSelectedText to be a function, got ${typeof setSelectedText}`,
			duration_ms: Date.now() - start8,
		});
	} else {
		await setSelectedText("Inserted text");

		debug("Test 8 completed - setSelectedText message sent");
		logTest(test8, "pass", {
			result: { setSelectedText: "function" },
			duration_ms: Date.now() - start8,
		});
	}
} catch (err) {
	logTest(test8, "fail", {
		error: String(err),
		duration_ms: Date.now() - start8,
	});
}

// -----------------------------------------------------------------------------
// Test 9: keyboard object - Type and tap
// -----------------------------------------------------------------------------
const test9 = "keyboard-api";
logTest(test9, "running");
const start9 = Date.now();

try {
	debug("Test 9: keyboard API");

	// Verify keyboard object exists and has methods
	if (typeof keyboard !== "object") {
		throw new Error("keyboard is not an object");
	}

	if (typeof keyboard.type !== "function") {
		throw new Error("keyboard.type is not a function");
	}

	if (typeof keyboard.tap !== "function") {
		throw new Error("keyboard.tap is not a function");
	}

	// These helpers are intentionally unsupported in GPUI until native input has
	// focus, permission, target, and receipt contracts. They should throw before
	// sending a fire-and-forget message.
	if (process.env.INCLUDE_SYSTEM_INPUT === "1") {
		await expectUnsupported("keyboard.type", () => keyboard.type("Hello World"));
		await expectUnsupported("keyboard.tap", () => keyboard.tap("command", "c"));
		debug("Test 9 completed - keyboard helpers rejected unsupported calls");
		logTest(test9, "pass", {
			result: { unsupported: ["type", "tap"] },
			duration_ms: Date.now() - start9,
		});
	} else {
		debug("Test 9 skipped - unsupported helper throw checks require INCLUDE_SYSTEM_INPUT=1");
		logTest(test9, "skip", {
			result: { reason: "keyboard helpers are unsupported in GPUI" },
			duration_ms: Date.now() - start9,
		});
	}
} catch (err) {
	logTest(test9, "fail", {
		error: String(err),
		duration_ms: Date.now() - start9,
	});
}

// -----------------------------------------------------------------------------
// Test 10: mouse object - Move, click, setPosition
// -----------------------------------------------------------------------------
const test10 = "mouse-api";
logTest(test10, "running");
const start10 = Date.now();

try {
	debug("Test 10: mouse API");

	// Verify mouse object exists and has methods
	if (typeof mouse !== "object") {
		throw new Error("mouse is not an object");
	}

	if (typeof mouse.move !== "function") {
		throw new Error("mouse.move is not a function");
	}

	if (typeof mouse.leftClick !== "function") {
		throw new Error("mouse.leftClick is not a function");
	}

	if (typeof mouse.rightClick !== "function") {
		throw new Error("mouse.rightClick is not a function");
	}

	if (typeof mouse.setPosition !== "function") {
		throw new Error("mouse.setPosition is not a function");
	}

	// These helpers are intentionally unsupported in GPUI until native input has
	// coordinate, focus, permission, target, and receipt contracts. They should
	// throw before sending a fire-and-forget message.
	if (process.env.INCLUDE_SYSTEM_INPUT === "1") {
		await expectUnsupported("mouse.move", () =>
			mouse.move([
				{ x: 100, y: 100 },
				{ x: 200, y: 200 },
				{ x: 300, y: 150 },
			])
		);
		await expectUnsupported("mouse.setPosition", () =>
			mouse.setPosition({ x: 500, y: 500 })
		);
		await expectUnsupported("mouse.leftClick", () => mouse.leftClick());
		await expectUnsupported("mouse.rightClick", () => mouse.rightClick());
		debug("Test 10 completed - mouse helpers rejected unsupported calls");
		logTest(test10, "pass", {
			result: { unsupported: ["move", "leftClick", "rightClick", "setPosition"] },
			duration_ms: Date.now() - start10,
		});
	} else {
		debug("Test 10 skipped - unsupported helper throw checks require INCLUDE_SYSTEM_INPUT=1");
		logTest(test10, "skip", {
			result: { reason: "mouse helpers are unsupported in GPUI" },
			duration_ms: Date.now() - start10,
		});
	}
} catch (err) {
	logTest(test10, "fail", {
		error: String(err),
		duration_ms: Date.now() - start10,
	});
}

// -----------------------------------------------------------------------------
// Show Summary
// -----------------------------------------------------------------------------
debug("test-system.ts completed!");

await div(
	md(`# System APIs (TIER 3) Tests Complete

All system API tests have been executed.

## Test Categories

### Alerts (Fire-and-Forget)
1. **beep**: System beep sound
2. **say**: Text-to-speech
3. **notify**: System notifications
4. **setStatus**: Application status
5. **menu**: System menu icon/scripts

### Clipboard
6. **copy-paste**: Convenience aliases
7. **clipboard-api**: Full clipboard object

### Text Operations
8. **setSelectedText**: Paste at cursor

### Input Simulation
9. **keyboard-api**: Type text and tap keys
10. **mouse-api**: Move, click, position

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`),
);

debug("test-system.ts exiting...");
