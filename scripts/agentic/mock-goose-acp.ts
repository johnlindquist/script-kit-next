#!/usr/bin/env bun
/**
 * Deterministic local goose ACP shim for Script Kit GPUI runtime proof.
 *
 * Point SCRIPT_KIT_GOOSE_BINARY at this file. It answers the ACP calls the
 * goose runtime makes (initialize, session/new, session/set_config_option,
 * session/prompt, session/cancel) and streams a fixed reply. It never echoes
 * prompt text; stderr carries length-only diagnostics.
 *
 * MOCK_GOOSE_READY_MARKER=<path>: until that file exists, every prompt fails
 * with a JSON-RPC error ("connection lost") so recovery flows can be driven.
 *
 * MOCK_GOOSE_RECORD_PATH=<path>: append one NDJSON record per session/new
 * (the decoded recipe: instructions length, extensions, settings) and per
 * prompt (content block types), so probes can prove what a session was
 * given without reading app internals.
 *
 * MOCK_GOOSE_REPLY_DELAY_MS=<ms>: wait before streaming the reply, like a model
 * that takes time to its first token.
 *
 * MOCK_GOOSE_TOOL_CALL=<extension>__<tool>: before replying, stream one
 * completed tool call named the way goose names it (e.g.
 * `scriptkit__web_search`).
 *
 * MOCK_GOOSE_SIGN_IN=1 enables browser sign-in, like real goose: when the
 * session's provider signs in
 * through a browser (`chatgpt_codex`, `gemini_oauth`, `xai_oauth`) and
 * `$GOOSE_PATH_ROOT/config/<provider>/tokens.json` is missing, a prompt first
 * records `oauth_started` and waits for that file. Nothing opens a real
 * browser. The simulated user:
 *   MOCK_GOOSE_SIGN_IN_APPROVE_AFTER_MS=<ms>  writes the token file after the
 *     delay ("finishes signing in"); unset = never approves.
 *   MOCK_GOOSE_SIGN_IN_APPROVE_FROM_ATTEMPT=<n>  approves only from the n-th
 *     sign-in attempt on (default 1), so a failed first try can be driven.
 *   MOCK_GOOSE_SIGN_IN_TIMEOUT_MS=<ms>  how long to wait before failing with
 *     ACP `auth_required` (default 300000, like goose).
 */

import { appendFileSync, existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const recordPath = process.env.MOCK_GOOSE_RECORD_PATH;

function record(entry: Record<string, unknown>): void {
	if (recordPath) appendFileSync(recordPath, `${JSON.stringify(entry)}\n`);
}

function decodeRecipe(deeplink: unknown): Record<string, unknown> | null {
	if (typeof deeplink !== "string" || !deeplink) return null;
	try {
		const json = Buffer.from(deeplink, "base64url").toString("utf8");
		return JSON.parse(json) as Record<string, unknown>;
	} catch {
		return null;
	}
}

const REPLY = "Bonjour le monde.";
const BROWSER_SIGN_IN_PROVIDERS = new Set(["chatgpt_codex", "gemini_oauth", "xai_oauth"]);
const gooseRoot = process.env.GOOSE_PATH_ROOT;

function tokensPath(): string | null {
	return gooseRoot ? join(gooseRoot, "config", provider, "tokens.json") : null;
}

/** Simulate goose's browser login; resolves whether the user signed in. */
async function simulateBrowserSignIn(): Promise<boolean> {
	const path = tokensPath();
	if (!path || existsSync(path)) return true;
	const attemptsPath = join(dirname(path), "mock-sign-in-attempts");
	mkdirSync(dirname(path), { recursive: true });
	const attempt = (existsSync(attemptsPath) ? Number(readFileSync(attemptsPath, "utf8")) || 0 : 0) + 1;
	writeFileSync(attemptsPath, String(attempt));
	record({ event: "oauth_started", provider, attempt });
	log("oauth_started", { attempt });
	const approveAfter = process.env.MOCK_GOOSE_SIGN_IN_APPROVE_AFTER_MS;
	const approveFrom = Number(process.env.MOCK_GOOSE_SIGN_IN_APPROVE_FROM_ATTEMPT ?? "1");
	if (approveAfter !== undefined && attempt >= approveFrom) {
		setTimeout(() => writeFileSync(path, JSON.stringify({ mock: true })), Number(approveAfter));
	}
	const deadline = Date.now() + Number(process.env.MOCK_GOOSE_SIGN_IN_TIMEOUT_MS ?? "300000");
	while (Date.now() < deadline) {
		if (existsSync(path)) {
			record({ event: "oauth_completed", provider, attempt });
			return true;
		}
		await Bun.sleep(100);
	}
	record({ event: "oauth_timed_out", provider, attempt });
	return false;
}
const readyMarker = process.env.MOCK_GOOSE_READY_MARKER;
const toolCall = process.env.MOCK_GOOSE_TOOL_CALL;
let provider = "chatgpt_codex";
let model = "gpt-5.4";
let sessionCounter = 0;

function write(message: unknown): void {
	process.stdout.write(`${JSON.stringify(message)}\n`);
}

function log(event: string, fields: Record<string, string | number> = {}): void {
	const payload = Object.entries(fields)
		.map(([key, value]) => `${key}=${value}`)
		.join(" ");
	process.stderr.write(`[mock-goose-acp] event=${event}${payload ? ` ${payload}` : ""}\n`);
}

function configOptions() {
	return [
		{
			id: "provider",
			name: "Provider",
			type: "select",
			currentValue: provider,
			options: [{ value: provider, name: provider }],
		},
		{
			id: "model",
			name: "Model",
			category: "model",
			type: "select",
			currentValue: model,
			options: [{ value: model, name: `Mock ${model}` }],
		},
	];
}

function update(sessionId: string, value: Record<string, unknown>): void {
	write({ jsonrpc: "2.0", method: "session/update", params: { sessionId, update: value } });
}

async function handle(message: Record<string, unknown>): Promise<void> {
	const id = message.id;
	const method = typeof message.method === "string" ? message.method : "";
	const params = (message.params ?? {}) as Record<string, unknown>;
	if (id === undefined) {
		log("notification", { methodChars: method.length });
		return;
	}
	switch (method) {
		case "initialize":
			write({
				jsonrpc: "2.0",
				id,
				result: {
					protocolVersion: 1,
					agentCapabilities: { promptCapabilities: { image: true } },
					agentInfo: { name: "goose", version: "mock" },
				},
			});
			return;
		case "session/new": {
			sessionCounter += 1;
			const meta = (params._meta ?? {}) as Record<string, unknown>;
			const recipe = decodeRecipe(meta.recipeDeeplink);
			const settings = (recipe?.settings ?? {}) as Record<string, unknown>;
			if (typeof settings.goose_provider === "string") provider = settings.goose_provider;
			if (typeof settings.goose_model === "string") model = settings.goose_model;
			record({
				event: "session_new",
				sessionId: `mock-session-${sessionCounter}`,
				hidden: meta.hidden ?? null,
				title: recipe?.title ?? null,
				instructionsChars: typeof recipe?.instructions === "string" ? recipe.instructions.length : 0,
				settings,
				extensions: Array.isArray(recipe?.extensions)
					? recipe.extensions.map((extension: Record<string, unknown>) => ({
							type: extension.type,
							name: extension.name,
							availableTools: extension.available_tools ?? null,
						}))
					: null,
			});
			write({
				jsonrpc: "2.0",
				id,
				result: { sessionId: `mock-session-${sessionCounter}`, configOptions: configOptions() },
			});
			return;
		}
		case "session/set_config_option": {
			const configId = String(params.configId ?? "");
			const value = String(params.value ?? "");
			if (configId === "provider") provider = value;
			if (configId === "model") model = value;
			log("set_config_option", { configIdChars: configId.length, valueChars: value.length });
			write({ jsonrpc: "2.0", id, result: { configOptions: configOptions() } });
			return;
		}
		case "session/prompt": {
			const sessionId = String(params.sessionId ?? "");
			const blocks = Array.isArray(params.prompt) ? params.prompt.length : 0;
			log("prompt", { blocks });
			record({
				event: "prompt",
				sessionId,
				blockTypes: Array.isArray(params.prompt)
					? params.prompt.map((block: Record<string, unknown>) => block.type)
					: [],
			});
			if (readyMarker && !existsSync(readyMarker)) {
				write({
					jsonrpc: "2.0",
					id,
					error: { code: -32603, message: "Internal error", data: "connection lost" },
				});
				return;
			}
			if (
				process.env.MOCK_GOOSE_SIGN_IN === "1" &&
				BROWSER_SIGN_IN_PROVIDERS.has(provider) &&
				!(await simulateBrowserSignIn())
			) {
				write({
					jsonrpc: "2.0",
					id,
					error: { code: -32000, message: "Authentication required" },
				});
				return;
			}
			const replyDelay = Number(process.env.MOCK_GOOSE_REPLY_DELAY_MS ?? "0");
			if (replyDelay > 0) await Bun.sleep(replyDelay);
			update(sessionId, {
				sessionUpdate: "agent_thought_chunk",
				content: { type: "text", text: "reading" },
			});
			if (toolCall) {
				// goose names tools `<extension>__<tool>` in its ACP metadata.
				const [extensionName] = toolCall.split("__");
				const meta = { goose: { toolCall: { toolName: toolCall, extensionName } } };
				update(sessionId, {
					sessionUpdate: "tool_call",
					toolCallId: "mock-tool-1",
					title: toolCall,
					status: "pending",
					rawInput: { query: "mock query" },
					_meta: meta,
				});
				update(sessionId, {
					sessionUpdate: "tool_call_update",
					toolCallId: "mock-tool-1",
					status: "completed",
					content: [{ type: "content", content: { type: "text", text: "Mock result https://example.com" } }],
					_meta: meta,
				});
			}
			update(sessionId, {
				sessionUpdate: "agent_message_chunk",
				content: { type: "text", text: REPLY },
			});
			write({ jsonrpc: "2.0", id, result: { stopReason: "end_turn" } });
			return;
		}
		default:
			write({
				jsonrpc: "2.0",
				id,
				error: { code: -32601, message: `Mock goose does not implement ${method}` },
			});
	}
}

const decoder = new TextDecoder();
let buffer = "";
for await (const chunk of Bun.stdin.stream()) {
	buffer += decoder.decode(chunk, { stream: true });
	let newline = buffer.indexOf("\n");
	while (newline >= 0) {
		const line = buffer.slice(0, newline).trim();
		buffer = buffer.slice(newline + 1);
		newline = buffer.indexOf("\n");
		if (!line) continue;
		try {
			void handle(JSON.parse(line));
		} catch {
			log("invalid_json", { chars: line.length });
		}
	}
}
