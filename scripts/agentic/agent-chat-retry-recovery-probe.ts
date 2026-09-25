#!/usr/bin/env bun
import { mkdirSync, writeFileSync } from "node:fs";
import { mkdir, mkdtemp, writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { tmpdir } from "node:os";
import { Driver, type Json } from "../devtools/driver.ts";

const binary =
	process.env.SCRIPT_KIT_GPUI_BINARY ??
	"target-agent/artifacts/agent-chat-recovery/script-kit-gpui";
const outputPath = resolve(
	process.env.PROBE_RECEIPT ?? ".test-output/agent-chat-retry-recovery.json",
);

function asArray(value: unknown): Json[] {
	return Array.isArray(value) ? (value as Json[]) : [];
}

function sleep(ms: number): Promise<void> {
	return new Promise((resolveSleep) => setTimeout(resolveSleep, ms));
}

const dir = await mkdtemp(join(tmpdir(), "sk-agent-chat-recovery-"));
const readyMarker = join(dir, "ready");
// The mock fails every prompt until the ready marker exists, then answers.
const mockGoose = resolve(import.meta.dir, "mock-goose-acp.ts");
await mkdir(resolve(".test-output"), { recursive: true });
await mkdir(resolve(".test-screenshots/agent-chat-retry-recovery"), {
	recursive: true,
});

const receipt: Json = {
	schemaVersion: 1,
	verifier: "agent-chat-retry-recovery-probe",
	binary,
	mockGoose,
	status: "fail",
};

const driver = await Driver.launch({
	binary,
	sessionName: "agent-chat-retry-recovery",
	sandboxHome: true,
	readyTimeoutMs: 20_000,
	defaultTimeoutMs: 12_000,
	env: {
		SCRIPT_KIT_GOOSE_BINARY: mockGoose,
		MOCK_GOOSE_READY_MARKER: readyMarker,
		SCRIPT_KIT_AGENT_CHAT_HOT_PREWARM: "1",
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


try {
	await driver.request({ type: "show" });
	await driver.setFilterAndWait("recovery proof", { timeoutMs: 8_000 });
	await driver.waitForSettle();
	receipt.before = await driver.getState({ timeoutMs: 8_000 });

	// Let the mock prewarm report its model before opening Agent Chat.
	await sleep(500);
	await driver.simulateGpuiEvent(
		{ type: "keyDown", key: "enter", modifiers: ["cmd"] },
		{ target: { type: "main" } },
	);

	const recoveryDeadline = Date.now() + 15_000;
	const waitForRecovery = async (): Promise<Json> => {
		const state = await driver.request(
			{ type: "getAgentChatState", target: { type: "id", id: "main" } },
			{ expect: "agent_chatStateResult", timeoutMs: 8_000 },
		);
		if ((state.reliability as Json | undefined)?.phase === "awaitingRecovery") {
			return state;
		}
		if (Date.now() >= recoveryDeadline) {
			throw new Error(
				`typed Agent Chat recovery did not appear: ${JSON.stringify(state)}`,
			);
		}
		await sleep(100);
		return waitForRecovery();
	};
	const failureState = await waitForRecovery();
	receipt.failureState = failureState;
	receipt.failureScreenshot = await driver.captureScreenshot({
		target: { type: "id", id: "main" },
		savePath: resolve(
			".test-screenshots/agent-chat-retry-recovery/failure-modal.png",
		),
		timeoutMs: 10_000,
	});
	const recovery = failureState.reliability as Json | undefined;
	if (
		!asArray(recovery?.recoveryActions).includes("ai-recovery-retry") ||
		(recovery?.diagnostic as Json | undefined)?.rawPrimaryVisible !== false
	) {
		throw new Error(`typed recovery contract missing: ${JSON.stringify(recovery)}`);
	}
	const beforeRows = asArray(
		(failureState.transcriptScroll as Json | undefined)?.rowSemanticIds,
	).map(String);
	const beforeUserRows = beforeRows.filter((id) => id.includes("-user-")).length;

	// Replace the sidecar behavior in place. Retry stays in the same thread and
	// must not duplicate the preserved user turn.
	await writeFile(readyMarker, "ready\n");
	await driver.simulateGpuiEvent(
		{ type: "keyDown", key: "r", modifiers: ["cmd", "shift"] },
		{ target: { type: "main" } },
	);

	const deadline = Date.now() + 12_000;
	const waitForSuccess = async (): Promise<Json> => {
		const state = await driver.request(
			{ type: "getAgentChatState", target: { type: "id", id: "main" } },
			{ expect: "agent_chatStateResult", timeoutMs: 8_000 },
		);
		if ((state.reliability as Json | undefined)?.phase === "succeeded") {
			return state;
		}
		if (Date.now() >= deadline) return state;
		await sleep(150);
		return waitForSuccess();
	};
	const state = await waitForSuccess();
	receipt.afterRetry = state;
	receipt.logs = await driver.getLogs({
		limit: 300,
		target: "script_kit::agent_chat",
	});
	const afterRows = asArray(
		(state.transcriptScroll as Json | undefined)?.rowSemanticIds,
	).map(String);
	const afterUserRows = afterRows.filter((id) => id.includes("-user-")).length;
	if ((state.reliability as Json | undefined)?.phase !== "succeeded")
		throw new Error(
			`Agent Chat did not succeed after healthy retry: ${JSON.stringify(state)}`,
		);
	if (beforeUserRows !== 1 || afterUserRows !== beforeUserRows) {
		throw new Error(
			`retry duplicated the preserved user turn: before=${beforeRows} after=${afterRows}`,
		);
	}

	receipt.status = "pass";
} catch (error) {
	receipt.error = error instanceof Error ? error.message : String(error);
} finally {
	await driver.close();
	await writeFile(outputPath, `${JSON.stringify(receipt, null, 2)}\n`);
}

console.log(JSON.stringify(receipt, null, 2));
process.exit(receipt.status === "pass" ? 0 : 1);
