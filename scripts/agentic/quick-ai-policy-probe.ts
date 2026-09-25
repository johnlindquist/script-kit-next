#!/usr/bin/env bun
/**
 * Quick AI tool-lockdown runtime proof.
 *
 * Quick AI (launcher text + Tab) runs a zero-context goose session that may
 * only search the web, once per answer. The AUTHORITATIVE admission logic is
 * locked by unit tests (`ai::agent_chat::goose::recipe`,
 * `mcp_agent_tools`, `ui::capabilities`). This probe confirms against the REAL
 * binary, with a deterministic goose ACP shim, that:
 *
 *   1. Quick AI launches (`quick_ai_tab_entry`) without violating the
 *      zero-context launch invariant.
 *   2. The goose session it starts is hidden and carries exactly one
 *      extension: Script Kit's tools with only `web_search`. No developer
 *      (shell/write/edit) extension is loaded.
 *   3. The prompt reaches goose as a single text block, the goose-named
 *      `scriptkit__web_search` call is admitted by the web-search-only policy,
 *      and the turn completes with an assistant row in the Quick AI transcript.
 *
 * The per-answer search budget itself is enforced by Script Kit's MCP tool
 * endpoint and covered by `mcp_agent_tools::tests`.
 *
 * Run: bun scripts/agentic/quick-ai-policy-probe.ts [--receipt <path>]
 */
import { Driver } from "../devtools/driver.ts";
import { mkdirSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";

// Unset = the driver picks the freshest built binary and reports it.
const binary = process.env.SCRIPT_KIT_GPUI_BINARY;
const receiptArgIdx = process.argv.indexOf("--receipt");
const receiptPath =
	receiptArgIdx >= 0 ? process.argv[receiptArgIdx + 1] : undefined;

const fixtureDir = mkdtempSync(join(tmpdir(), "quick-ai-policy-"));
const recordPath = join(fixtureDir, "goose-record.ndjson");
const mockGoose = resolve(import.meta.dir, "mock-goose-acp.ts");
mkdirSync(resolve(".test-screenshots/ai-rock-solid-ux"), { recursive: true });

const receipt: Record<string, unknown> = {
	probe: "quick-ai-policy",
	binary,
	mockGoose,
	unautomatable: [
		"drag-and-drop file attach (no driver primitive)",
		"clipboard image paste (no driver primitive)",
	],
};

type GooseRecord = {
	event: string;
	sessionId?: string;
	hidden?: boolean | null;
	settings?: Record<string, unknown>;
	extensions?: Array<{ type?: string; name?: string; availableTools?: string[] | null }> | null;
	blockTypes?: string[];
};

function gooseRecords(): GooseRecord[] {
	try {
		return readFileSync(recordPath, "utf8")
			.trim()
			.split("\n")
			.filter(Boolean)
			.map((line) => JSON.parse(line) as GooseRecord);
	} catch {
		return [];
	}
}

const driver = await Driver.launch({
	sessionName: "quick-ai-policy-probe",
	binary,
	sandboxHome: true,
	env: {
		SCRIPT_KIT_GOOSE_BINARY: mockGoose,
		MOCK_GOOSE_RECORD_PATH: recordPath,
		MOCK_GOOSE_TOOL_CALL: "scriptkit__web_search",
		SCRIPT_KIT_PANEL_INVARIANTS_ALLOW_MISMATCH: "1",
	},
});

// The mock stands in for a signed-in ChatGPT account, so seed its stored
// sign-in; the sign-in flow itself is covered by agent-chat-sign-in-probe.ts.
{
	const tokens = join(driver.sessionDir, "home", ".scriptkit", "goose", "config", "chatgpt_codex");
	mkdirSync(tokens, { recursive: true });
	writeFileSync(join(tokens, "tokens.json"), JSON.stringify({ mock: true }));
}


async function logSeen(needle: string, timeoutMs = 500): Promise<boolean> {
	const start = performance.now();
	do {
		const logs = await driver.getLogs({ limit: 800 });
		if (JSON.stringify(logs).includes(needle)) return true;
		await Bun.sleep(150);
	} while (performance.now() - start < timeoutMs);
	return false;
}

try {
	await driver.request({ type: "show" });
	await driver.waitForSettle();
	await driver.setFilterAndWait("what is the capital of france");
	driver.simulateKey("tab");
	const quickAiEntry = await logSeen("quick_ai_tab_entry", 6000);

	// Wait for the Quick AI turn to reach goose and finish streaming.
	let state: any = null;
	const deadline = performance.now() + 20_000;
	while (performance.now() < deadline) {
		const prompted = gooseRecords().some((record) => record.event === "prompt");
		try {
			state = await driver.request(
				{ type: "getAgentChatState", target: { type: "id", id: "main" } },
				{ expect: "agent_chatStateResult", timeoutMs: 2500 },
			);
		} catch {
			state = null;
		}
		if (prompted && state?.reliability?.phase === "succeeded") break;
		await Bun.sleep(150);
	}
	receipt.state = state;
	receipt.screenshot = await driver.captureScreenshot({
		target: { type: "id", id: "main" },
		savePath: resolve(".test-screenshots/ai-rock-solid-ux/quick-ai-policy-goose.png"),
		timeoutMs: 10_000,
	});

	const records = gooseRecords();
	receipt.gooseRecords = records;
	const session = records.find((record) => record.event === "session_new");
	const prompt = records.find((record) => record.event === "prompt");
	const extensions = session?.extensions ?? [];
	const launchInvariantViolated = await logSeen(
		"quick_ai_zero_context_launch_invariant_violated",
		300,
	);

	const checks: Array<[string, boolean]> = [
		["launch.quickAiEntry", quickAiEntry],
		["launch.invariantNotViolated", !launchInvariantViolated],
		["session.started", Boolean(session)],
		["session.hidden", session?.hidden === true],
		["tools.singleExtension", extensions.length === 1],
		["tools.scriptKitOnly", extensions[0]?.name === "scriptkit"],
		[
			"tools.webSearchOnly",
			JSON.stringify(extensions[0]?.availableTools ?? []) === JSON.stringify(["web_search"]),
		],
		["tools.noDeveloper", !extensions.some((extension) => extension.name === "developer")],
		["prompt.singleTextBlock", JSON.stringify(prompt?.blockTypes ?? []) === JSON.stringify(["text"])],
		["reply.succeeded", state?.reliability?.phase === "succeeded"],
		// A goose-named web_search call that the policy failed to recognise
		// ends the turn as a denied tool instead.
		["reply.webSearchAdmitted", state?.reliability?.failureCode !== "UserDeniedTool"],
		["reply.quickAiSurface", state?.uiVariant === "quick-ai"],
		[
			"reply.assistantRow",
			(state?.transcriptScroll?.rowSemanticIds ?? []).some((id: string) =>
				id.includes("-assistant-"),
			),
		],
	];
	receipt.pass = checks.every(([, ok]) => ok);
	receipt.failedChecks = checks.filter(([, ok]) => !ok).map(([name]) => name);
} finally {
	await driver.close();
}

const serialized = JSON.stringify(receipt, null, 2);
if (receiptPath) writeFileSync(receiptPath, serialized);
console.log(serialized);
process.exit(receipt.pass ? 0 : 1);
