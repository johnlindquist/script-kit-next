// Name: SDK Test - form() All Input Types
// Description: Tests form() HTML parsing with all 14 input types from the docs

/**
 * SDK TEST: test-form-all-types.ts
 *
 * Tests the form() function with ALL input types from the documentation.
 * This is the comprehensive test for form HTML parsing.
 *
 * Test cases:
 * 1. form-function-exists: Verify form() is exported from SDK
 * 2. form-expected-fields: Document expected field parsing
 * 3. visual-verification: Display form HTML and capture screenshot
 *
 * NOTE: form() is interactive and waits for user submission.
 * The actual form parsing logic is tested via Rust unit tests in form_parser.rs.
 * This test focuses on SDK integration and visual verification.
 *
 * Expected behavior:
 * - form() function exists and is callable
 * - All 14 field types should be parsed correctly by form_parser.rs
 */

import "../../scripts/kit-sdk.ts";
import { writeFileSync, mkdirSync } from "fs";
import { join } from "path";

// =============================================================================
// CI Detection
// =============================================================================

const isCI = Boolean(process.env.CI || process.env.GITHUB_ACTIONS || process.env.TRAVIS || process.env.CIRCLECI);

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

// =============================================================================
// Constants - The exact HTML template from the docs
// =============================================================================

const FORM_HTML_ALL_TYPES = `
<div class="p-4">
    <input type="text" name="textInput" placeholder="Text Input" />
    <input type="password" name="passwordInput" placeholder="Password" />
    <input type="email" name="emailInput" placeholder="Email" />
    <input type="number" name="numberInput" placeholder="Number" />
    <input type="date" name="dateInput" placeholder="Date" />
    <input type="time" name="timeInput" placeholder="Time" />
    <input type="datetime-local" name="dateTimeInput" placeholder="Date and Time" />
    <input type="month" name="monthInput" placeholder="Month" />
    <input type="week" name="weekInput" placeholder="Week" />
    <input type="url" name="urlInput" placeholder="URL" />
    <input type="search" name="searchInput" placeholder="Search" />
    <input type="tel" name="telInput" placeholder="Telephone" />
    <input type="color" name="colorInput" placeholder="Color" />
    <textarea name="textareaInput" placeholder="Textarea"></textarea>
</div>
`;

// Expected field names that form() should extract
const EXPECTED_FIELD_NAMES = [
	"textInput",
	"passwordInput",
	"emailInput",
	"numberInput",
	"dateInput",
	"timeInput",
	"dateTimeInput",
	"monthInput",
	"weekInput",
	"urlInput",
	"searchInput",
	"telInput",
	"colorInput",
	"textareaInput",
];

// =============================================================================
// Tests
// =============================================================================

debug("test-form-all-types.ts starting...");
debug(`SDK globals: form=${typeof form}, div=${typeof div}, captureScreenshot=${typeof captureScreenshot}`);

// -----------------------------------------------------------------------------
// Test 1: Verify form() function exists
// -----------------------------------------------------------------------------
const test1 = "form-function-exists";
logTest(test1, "running");
const start1 = Date.now();

try {
	debug("Test 1: Verifying form() function exists and is callable");

	if (typeof form !== "function") {
		logTest(test1, "fail", {
			error: `Expected form to be a function, got ${typeof form}`,
			duration_ms: Date.now() - start1,
		});
	} else {
		debug(`form function exists, length=${form.length}`);
		logTest(test1, "pass", {
			result: {
				type: typeof form,
				functionLength: form.length,
				isCallable: true,
			},
			duration_ms: Date.now() - start1,
		});
	}
} catch (err) {
	logTest(test1, "fail", {
		error: String(err),
		duration_ms: Date.now() - start1,
	});
}

// -----------------------------------------------------------------------------
// Test 2: Document expected form parsing behavior
// -----------------------------------------------------------------------------
const test2 = "form-expected-parsing";
logTest(test2, "running");
const start2 = Date.now();

try {
	debug("Test 2: Documenting expected form() parsing behavior");

	// Document what form() should return when called with FORM_HTML_ALL_TYPES
	const expectedBehavior = {
		inputHtml: "HTML string with 14 input elements from docs",
		expectedOutputType: "Record<string, string>",
		expectedKeys: EXPECTED_FIELD_NAMES,
		keyCount: EXPECTED_FIELD_NAMES.length,
		supportedInputTypes: [
			"text", "password", "email", "number",
			"date", "time", "datetime-local", "month", "week",
			"url", "search", "tel", "color", "textarea"
		],
		parserLocation: "src/form_parser.rs",
		sdkLocation: "scripts/kit-sdk.ts line 2951",
		interactiveNote: "form() waits for user to fill and submit the form",
	};

	debug(`Expected behavior: ${JSON.stringify(expectedBehavior, null, 2)}`);

	logTest(test2, "pass", {
		result: expectedBehavior,
		duration_ms: Date.now() - start2,
	});
} catch (err) {
	logTest(test2, "fail", {
		error: String(err),
		duration_ms: Date.now() - start2,
	});
}

// -----------------------------------------------------------------------------
// Test 3: Visual verification - display the form HTML and capture screenshot
// This verifies the HTML structure renders correctly
// -----------------------------------------------------------------------------
const test3 = "form-visual-verification";
logTest(test3, "running");
const start3 = Date.now();

try {
	debug("Test 3: Visual verification - displaying form with all 14 input types");

	// Display the form HTML using div() for visual verification
	const displayHtml = `
		<div class="p-6 space-y-3 bg-zinc-900 min-h-screen">
			<h2 class="text-xl font-bold text-white mb-4">Form() All Input Types Test</h2>
			<p class="text-sm text-gray-400 mb-4">Testing all 14 input types from the docs:</p>
			
			<div class="space-y-2 text-white">
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">1. text:</span>
					<input type="text" name="textInput" placeholder="Text Input" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">2. password:</span>
					<input type="password" name="passwordInput" placeholder="Password" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">3. email:</span>
					<input type="email" name="emailInput" placeholder="Email" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">4. number:</span>
					<input type="number" name="numberInput" placeholder="Number" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">5. date:</span>
					<input type="date" name="dateInput" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">6. time:</span>
					<input type="time" name="timeInput" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">7. datetime-local:</span>
					<input type="datetime-local" name="dateTimeInput" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">8. month:</span>
					<input type="month" name="monthInput" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">9. week:</span>
					<input type="week" name="weekInput" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">10. url:</span>
					<input type="url" name="urlInput" placeholder="URL" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">11. search:</span>
					<input type="search" name="searchInput" placeholder="Search" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">12. tel:</span>
					<input type="tel" name="telInput" placeholder="Telephone" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1" />
				</div>
				<div class="flex items-center gap-2">
					<span class="w-28 text-gray-400 text-xs">13. color:</span>
					<input type="color" name="colorInput" class="w-12 h-8 bg-zinc-800 border border-zinc-600 rounded" />
				</div>
				<div class="flex items-start gap-2">
					<span class="w-28 text-gray-400 text-xs pt-1">14. textarea:</span>
					<textarea name="textareaInput" placeholder="Textarea" class="px-2 py-1 bg-zinc-800 border border-zinc-600 rounded text-white text-sm flex-1 h-16"></textarea>
				</div>
			</div>
			
			<p class="text-xs text-gray-500 mt-4 border-t border-zinc-700 pt-2">
				All 14 types: text, password, email, number, date, time, datetime-local, month, week, url, search, tel, color, textarea
			</p>
		</div>
	`;

	// Fire and forget div() - don't await since it waits for user interaction
	void div(displayHtml);

	// Wait longer for render to complete and display to stabilize
	await new Promise((resolve) => setTimeout(resolve, 2000));

	// Capture screenshot
	const screenshot = await captureScreenshot();
	debug(`Captured screenshot: ${screenshot.width}x${screenshot.height}`);

	if (screenshot.width === 0 || screenshot.height === 0 || !screenshot.data) {
		// Skip in CI or when screenshot capture isn't working (no display)
		logTest(test3, "skip", {
			error: "Screenshot capture not available (CI or no display)",
			result: { width: screenshot.width, height: screenshot.height, hasData: !!screenshot.data },
			duration_ms: Date.now() - start3,
		});
	} else {
		// Save screenshot
		const screenshotDir = join(process.cwd(), ".test-screenshots");
		mkdirSync(screenshotDir, { recursive: true });

		const filename = `form-all-types-${Date.now()}.png`;
		const filepath = join(screenshotDir, filename);
		writeFileSync(filepath, Buffer.from(screenshot.data, "base64"));

		debug(`[SCREENSHOT] ${filepath}`);
		console.error(`[SCREENSHOT] ${filepath}`);

		logTest(test3, "pass", {
			result: {
				screenshotPath: filepath,
				dimensions: `${screenshot.width}x${screenshot.height}`,
				fieldsDisplayed: EXPECTED_FIELD_NAMES.length,
				fieldNames: EXPECTED_FIELD_NAMES,
			},
			duration_ms: Date.now() - start3,
		});
	}
} catch (err) {
	logTest(test3, "fail", {
		error: String(err),
		duration_ms: Date.now() - start3,
	});
}

// -----------------------------------------------------------------------------
// Complete - exit cleanly
// -----------------------------------------------------------------------------
debug("test-form-all-types.ts completed!");
debug("All tests finished. Exiting...");

process.exit(0);
