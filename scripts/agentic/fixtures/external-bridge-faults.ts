// A deliberately faulty local Herdr protocol peer, not a live terminal. The Rust fixture
// supplies this private socket. Drive the public CLI while the socket loop stays responsive.
import assert from "node:assert/strict";
import { createServer } from "node:net";
import { existsSync } from "node:fs";
import { randomUUID } from "node:crypto";
export async function exerciseHandoffFaults(
	socket: string,
	bin: string,
	cwd: string,
	receiver: Record<string, string>,
	sender: Record<string, string>,
) {
	assert(socket.includes("/sk-bridge-") && socket.endsWith("/peer.sock"));
	for (let i = 0; i < 100 && existsSync(socket); i++) await Bun.sleep(20);
	assert(!existsSync(socket), "previous owned Herdr socket still live");
	const core = "1".repeat(32),
		address = {
			name: "workshop",
			session: "bridge",
			core,
			terminal: "t1",
			createdMs: 1234,
			pid: 123,
		};
	let mode: "register" | "busy" | "changed" | "lost" | "malformed" = "register";
	const notices: string[] = [];
	const server = createServer((client) => {
		let buffer = "";
		client.setEncoding("utf8");
		client.on("error", () => {});
		client.on("data", (part) => {
			buffer += part;
			let end;
			while ((end = buffer.indexOf("\n")) >= 0) {
				const q = JSON.parse(buffer.slice(0, end));
				buffer = buffer.slice(end + 1);
				let result: unknown;
				switch (q.method) {
					case "hello":
						result = { generation: 1, core: { id: core } };
						break;
					case "terminal.get":
						result = {
							address: "t1",
							state: "live",
							pid: mode === "changed" ? 124 : 123,
							created_ms: 1234,
							reported: true,
							status: { state: mode === "busy" ? "working" : "idle" },
						};
						break;
					case "terminal.send":
						assert.equal(q.params.if, "idle");
						notices.push(q.params.text);
						if (mode === "lost") {
							client.destroy();
							return;
						}
						assert.equal(mode, "malformed");
						result = {};
						break;
					default:
						throw Error("Unexpected RPC: " + q.method);
				}
				client.write(
					JSON.stringify({ jsonrpc: "2.0", id: q.id, result }) + "\n",
				);
			}
		});
	});
	await new Promise<void>((ok, fail) => {
		server.once("error", fail);
		server.listen(socket, ok);
	});
	const cli = async (
		env: Record<string, string>,
		args: string[],
		body = "",
	) => {
		const p = Bun.spawn(["bun", bin, ...args], {
			cwd,
			env: { ...process.env, ...env },
			stdin: Buffer.from(body),
			stdout: "pipe",
			stderr: "pipe",
		});
		const [out, err, exit] = await Promise.all([
			new Response(p.stdout).text(),
			new Response(p.stderr).text(),
			p.exited,
		]);
		return { exit, ...JSON.parse(out || err) };
	};
	try {
		const registration = await cli(receiver, [
			"mcp",
			"call",
			"kit/handoff_available",
			JSON.stringify(address),
		]);
		assert.equal(registration.exit, 0, JSON.stringify(registration));
		const target = JSON.parse(
			registration.data.result.content[0].text,
		).registration;
		assert(target);
		const text = "Only these frozen words.";
		for (const fault of ["busy", "changed", "lost", "malformed"] as const) {
			mode = fault;
			const key = randomUUID(),
				send = ["agent", "send", target, key],
				before = notices.length;
			assert.notEqual(
				(await cli(sender, send, text)).exit,
				0,
				`Unexpected success after ${mode}`,
			);
			const { data: t } = await cli(sender, ["agent", "status", key]);
			if (mode === "busy" || mode === "changed") {
				assert.equal(t.state, "refused");
				assert.equal(
					t.reason,
					mode === "busy" ? "herdr_not_idle" : "herdr_terminal_changed",
				);
				assert.equal(notices.length, before);
			} else {
				assert.equal(t.state, "notification_unknown");
				assert.equal(notices.length, before + 1);
				assert(!notices.at(-1)!.includes(text));
				assert.notEqual((await cli(sender, send, text)).exit, 0);
				assert.equal(notices.length, before + 1);
				assert.equal(
					(await cli(sender, ["agent", "cancel", key])).data.state,
					"cancelled",
				);
				assert.notEqual((await cli(receiver, ["agent", "take", t.id])).exit, 0);
			}
		}
		mode = "busy";
		let limited = false;
		for (let i = 0; i < 14 && !limited; i++) {
			const r = await cli(
				sender,
				["agent", "send", target, randomUUID()],
				text,
			);
			limited = JSON.stringify(r).includes("sender_handoff_limit");
		}
		assert(
			limited,
			"one narrow sender must not exhaust the global receipt budget",
		);
		const second = {
			SCRIPT_KIT_AGENT_CREDENTIAL: sender.SCRIPT_KIT_AGENT_CREDENTIAL + "-two",
		};
		assert.equal(
			(
				await cli({}, [
					"grant-agent",
					"second-sender",
					second.SCRIPT_KIT_AGENT_CREDENTIAL,
					"handoff:send:workshop",
				])
			).exit,
			0,
		);
		const key = randomUUID();
		await cli(second, ["agent", "send", target, key], text);
		assert.equal(
			(await cli(second, ["agent", "status", key])).data.reason,
			"herdr_not_idle",
		);
		console.log(
			"Fault-injected local protocol: busy and changed terminals refused before send; lost/malformed acknowledgements stayed unknown, did not resend, and cancellation blocked collection.",
		);
	} finally {
		await new Promise<void>((done) => server.close(() => done()));
	}
}
