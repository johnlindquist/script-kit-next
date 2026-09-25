/// <reference types="bun-types" />

// Proves a goose binary speaks ACP well enough to back Script Kit: it must
// answer `initialize` with protocol version 1 and identify itself as goose.
// The probe runs goose under a throwaway GOOSE_PATH_ROOT so it never reads or
// writes the user's goose configuration, sessions, or credentials.

import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join } from "node:path";

export type GooseSidecarHealthFailure =
	| "not_executable"
	| "spawn_failed"
	| "timeout"
	| "invalid_json"
	| "wrong_id"
	| "process_exit"
	| "protocol_incompatible"
	| "invalid_response";

export type GooseSidecarHealthClassification =
	| "acp_ready"
	| GooseSidecarHealthFailure;

type HealthReceipt = {
	elapsedMs: number;
	classification: GooseSidecarHealthClassification;
};

export type GooseSidecarHealthResult =
	| (HealthReceipt & { ok: true; agentName: string; agentVersion: string })
	| (HealthReceipt & {
			ok: false;
			failure: GooseSidecarHealthFailure;
			detail: string;
	  });

const HEALTH_ID = 1;
const ACP_PROTOCOL_VERSION = 1;

/** Reads one property of parsed JSON without trusting its shape. */
function field(value: unknown, key: string): unknown {
	if (value && typeof value === "object" && key in value) {
		return (value as Record<string, unknown>)[key];
	}
	return undefined;
}

export async function checkGooseSidecarHealth(
	binary: string,
	timeoutMs = 5_000,
): Promise<GooseSidecarHealthResult> {
	const startedAt = performance.now();
	const receipt = (
		classification: GooseSidecarHealthClassification,
	): HealthReceipt => ({
		elapsedMs: Math.max(0, Math.round(performance.now() - startedAt)),
		classification,
	});
	const unhealthy = (
		failure: GooseSidecarHealthFailure,
		detail: string,
	): GooseSidecarHealthResult => ({
		ok: false,
		failure,
		detail,
		...receipt(failure),
	});

	if (!(await Bun.file(binary).exists())) {
		return unhealthy("not_executable", `${binary} does not exist`);
	}

	const root = await mkdtemp(join(tmpdir(), "goose-health-"));
	let child: Bun.Subprocess<"pipe", "pipe", "pipe">;
	try {
		child = Bun.spawn([binary, "acp"], {
			stdin: "pipe",
			stdout: "pipe",
			stderr: "pipe",
			cwd: root,
			env: {
				PATH: process.env.PATH ?? "/usr/bin:/bin",
				HOME: root,
				GOOSE_PATH_ROOT: root,
				GOOSE_DISABLE_KEYRING: "1",
				GOOSE_TELEMETRY_ENABLED: "false",
			},
		});
	} catch (error) {
		await rm(root, { recursive: true, force: true });
		return unhealthy("spawn_failed", String(error));
	}

	const request = JSON.stringify({
		jsonrpc: "2.0",
		id: HEALTH_ID,
		method: "initialize",
		params: {
			protocolVersion: ACP_PROTOCOL_VERSION,
			clientCapabilities: {
				fs: { readTextFile: false, writeTextFile: false },
				terminal: false,
			},
			clientInfo: { name: "script-kit-sidecar-health", version: "1" },
		},
	});
	child.stdin.write(`${request}\n`);
	child.stdin.flush();

	const deadline = Date.now() + timeoutMs;
	const reader = child.stdout.getReader();
	const decoder = new TextDecoder();
	let buffered = "";

	try {
		while (Date.now() < deadline) {
			const remaining = Math.max(1, deadline - Date.now());
			const result = await Promise.race([
				reader.read().then((read) => ({ kind: "read" as const, read })),
				Bun.sleep(remaining).then(() => ({ kind: "timeout" as const })),
			]);

			if (result.kind === "timeout") break;
			if (result.read.done) {
				const exitCode = await child.exited;
				return unhealthy(
					"process_exit",
					`goose closed stdout with exit code ${exitCode} before answering initialize`,
				);
			}

			buffered += decoder.decode(result.read.value, { stream: true });
			let newline = buffered.indexOf("\n");
			while (newline >= 0) {
				const line = buffered.slice(0, newline).trim();
				buffered = buffered.slice(newline + 1);
				newline = buffered.indexOf("\n");
				if (!line) continue;

				let message: unknown;
				try {
					message = JSON.parse(line);
				} catch {
					return unhealthy("invalid_json", `goose returned invalid JSON: ${line}`);
				}
				const id = field(message, "id");
				// Notifications may precede the response; only the answer counts.
				if (id === undefined && typeof field(message, "method") === "string") {
					continue;
				}
				if (id !== HEALTH_ID) {
					return unhealthy(
						"wrong_id",
						`goose answered id ${JSON.stringify(id)}; expected ${HEALTH_ID}`,
					);
				}
				const error = field(message, "error");
				if (error !== undefined && error !== null) {
					return unhealthy(
						"protocol_incompatible",
						`initialize failed: ${JSON.stringify(error)}`,
					);
				}
				const initialized = field(message, "result");
				const protocolVersion = field(initialized, "protocolVersion");
				if (protocolVersion !== ACP_PROTOCOL_VERSION) {
					return unhealthy(
						"protocol_incompatible",
						`goose negotiated protocol ${JSON.stringify(protocolVersion)}; expected ${ACP_PROTOCOL_VERSION}`,
					);
				}
				const agentInfo = field(initialized, "agentInfo");
				const agentName = field(agentInfo, "name");
				if (agentName !== "goose") {
					return unhealthy(
						"invalid_response",
						`initialize identified agent ${JSON.stringify(agentName)}; expected "goose"`,
					);
				}
				return {
					ok: true,
					agentName,
					agentVersion: String(field(agentInfo, "version") ?? ""),
					...receipt("acp_ready"),
				};
			}
		}
		return unhealthy("timeout", `initialize did not respond within ${timeoutMs}ms`);
	} finally {
		reader.releaseLock();
		child.kill();
		await child.exited;
		await rm(root, { recursive: true, force: true });
	}
}

if (import.meta.main) {
	const binary = process.argv[2];
	if (!binary) {
		console.error("usage: bun scripts/agentic/goose-sidecar-health.ts <goose-binary> [timeout-ms]");
		process.exit(2);
	}
	const result = await checkGooseSidecarHealth(binary, Number(process.argv[3] ?? 5_000));
	console.log(JSON.stringify(result));
	process.exit(result.ok ? 0 : 1);
}
