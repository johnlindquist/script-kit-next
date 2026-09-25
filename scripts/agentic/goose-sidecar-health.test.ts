/// <reference types="bun-types" />

import { afterEach, describe, expect, test } from "bun:test";
import { chmod, mkdtemp, rm, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { checkGooseSidecarHealth } from "./goose-sidecar-health.ts";

const dirs: string[] = [];

async function fakeGoose(body: string): Promise<string> {
	const dir = await mkdtemp(join(tmpdir(), "goose-health-test-"));
	dirs.push(dir);
	const path = join(dir, "goose");
	await writeFile(path, `#!/usr/bin/env bash\nread request\n${body}\n`);
	await chmod(path, 0o755);
	return path;
}

afterEach(async () => {
	await Promise.all(
		dirs.splice(0).map((dir) => rm(dir, { recursive: true, force: true })),
	);
});

const initialized = (result: object) =>
	`echo '${JSON.stringify({ jsonrpc: "2.0", id: 1, result })}'`;

describe("goose sidecar ACP health", () => {
	test("accepts a goose initialize response on ACP v1", async () => {
		const binary = await fakeGoose(
			initialized({
				protocolVersion: 1,
				agentInfo: { name: "goose", version: "1.52.0" },
			}),
		);
		expect(await checkGooseSidecarHealth(binary, 2_000)).toMatchObject({
			ok: true,
			classification: "acp_ready",
			agentName: "goose",
			agentVersion: "1.52.0",
		});
	});

	test("skips notifications that precede the initialize answer", async () => {
		const binary = await fakeGoose(
			`echo '{"jsonrpc":"2.0","method":"notice","params":{}}'\n${initialized({
				protocolVersion: 1,
				agentInfo: { name: "goose", version: "1" },
			})}`,
		);
		expect((await checkGooseSidecarHealth(binary, 2_000)).ok).toBeTrue();
	});

	test("rejects another ACP protocol version", async () => {
		const binary = await fakeGoose(
			initialized({ protocolVersion: 2, agentInfo: { name: "goose" } }),
		);
		expect(await checkGooseSidecarHealth(binary, 2_000)).toMatchObject({
			ok: false,
			failure: "protocol_incompatible",
		});
	});

	test("rejects an agent that is not goose", async () => {
		const binary = await fakeGoose(
			initialized({ protocolVersion: 1, agentInfo: { name: "pi" } }),
		);
		expect(await checkGooseSidecarHealth(binary, 2_000)).toMatchObject({
			ok: false,
			failure: "invalid_response",
		});
	});

	test("reports a process that exits without answering", async () => {
		const binary = await fakeGoose("exit 3");
		expect(await checkGooseSidecarHealth(binary, 2_000)).toMatchObject({
			ok: false,
			failure: "process_exit",
		});
	});

	test("times out a silent process", async () => {
		const binary = await fakeGoose("sleep 5");
		expect(await checkGooseSidecarHealth(binary, 200)).toMatchObject({
			ok: false,
			failure: "timeout",
		});
	});

	test("reports a missing binary", async () => {
		expect(
			await checkGooseSidecarHealth(join(tmpdir(), "missing-goose-binary"), 200),
		).toMatchObject({ ok: false, failure: "not_executable" });
	});
});
