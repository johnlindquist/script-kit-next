// Name: SDK Test - chat()
// Description: Tests chat() conversational UI prompt

/**
 * SDK TEST: test-chat.ts
 *
 * Tests the conversational chat UI where messages can be added programmatically.
 *
 * Test cases:
 * 1. chat-basic: Basic chat with messages - returns ChatResult object
 * 2. chat-addmessage: Chat with addMessage controller method
 * 3. chat-simple: Simple chat without options - returns ChatResult object
 *
 * Requires GPUI support for:
 * - 'chat' message type to open chat UI
 * - 'chatAction' message type for addMessage actions
 * - Submit response returns ChatResult with messages array
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

// =============================================================================
// Tests
// =============================================================================

debug("test-chat.ts starting...");
debug(`SDK globals: chat=${typeof chat}`);

// -----------------------------------------------------------------------------
// Test 1: Basic chat with messages - returns ChatResult object
// -----------------------------------------------------------------------------
const test1 = "chat-basic";
logTest(test1, "running");
const start1 = Date.now();

try {
	debug("Test 1: chat() with initial messages");

	const result = await chat({
		messages: [
			{ role: "assistant", content: "Welcome! I'm your assistant." },
			{ role: "assistant", content: "How can I help you today?" },
		],
	});

	debug(`Test 1 result type: ${typeof result}`);
	debug(`Test 1 result keys: ${Object.keys(result || {}).join(", ")}`);

	// Assertion: result should be a ChatResult object with expected properties
	if (typeof result !== "object" || result === null) {
		logTest(test1, "fail", {
			error: `Expected ChatResult object, got ${typeof result}`,
			result,
			duration_ms: Date.now() - start1,
		});
	} else if (!("messages" in result) || !("action" in result)) {
		logTest(test1, "fail", {
			error: `ChatResult missing required fields (messages, action)`,
			result: Object.keys(result),
			duration_ms: Date.now() - start1,
		});
	} else {
		logTest(test1, "pass", {
			result: { action: result.action, messageCount: result.messages?.length },
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
// Test 2: Chat with addMessage controller method
// -----------------------------------------------------------------------------
const test2 = "chat-addmessage";
logTest(test2, "running");
const start2 = Date.now();

try {
	debug("Test 2: chat.addMessage() controller method exists");

	// Test that addMessage is a function on the chat object
	if (typeof chat.addMessage !== "function") {
		logTest(test2, "fail", {
			error: `Expected chat.addMessage to be a function, got ${typeof chat.addMessage}`,
			duration_ms: Date.now() - start2,
		});
	} else if (typeof chat.getMessages !== "function") {
		logTest(test2, "fail", {
			error: `Expected chat.getMessages to be a function, got ${typeof chat.getMessages}`,
			duration_ms: Date.now() - start2,
		});
	} else if (typeof chat.clear !== "function") {
		logTest(test2, "fail", {
			error: `Expected chat.clear to be a function, got ${typeof chat.clear}`,
			duration_ms: Date.now() - start2,
		});
	} else {
		logTest(test2, "pass", {
			result: "chat controller methods exist (addMessage, getMessages, clear)",
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
// Test 3: Simple chat without options - returns ChatResult object
// -----------------------------------------------------------------------------
const test3 = "chat-simple";
logTest(test3, "running");
const start3 = Date.now();

try {
	debug("Test 3: chat() without options");

	// Can also call without options
	const result = await chat();

	debug(`Test 3 result type: ${typeof result}`);

	// Assertion: result should be a ChatResult object
	if (typeof result !== "object" || result === null) {
		logTest(test3, "fail", {
			error: `Expected ChatResult object, got ${typeof result}`,
			result,
			duration_ms: Date.now() - start3,
		});
	} else if (!("messages" in result) || !("action" in result)) {
		logTest(test3, "fail", {
			error: `ChatResult missing required fields (messages, action)`,
			result: Object.keys(result),
			duration_ms: Date.now() - start3,
		});
	} else {
		logTest(test3, "pass", {
			result: { action: result.action, messageCount: result.messages?.length },
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
// Show Summary
// -----------------------------------------------------------------------------
debug("test-chat.ts completed!");

await div(
	md(`# chat() Tests Complete

All chat prompt tests have been executed.

## Test Cases Run
1. **chat-basic**: Basic chat with messages (expects ChatResult object)
2. **chat-addmessage**: Chat controller methods exist (addMessage, getMessages, clear)
3. **chat-simple**: Simple chat without options (expects ChatResult object)

---

*Check the JSONL output for detailed results*

Press Escape or click to exit.`),
);

debug("test-chat.ts exiting...");
