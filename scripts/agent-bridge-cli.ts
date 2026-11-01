// External agent surface of the scriptkit shim. Operator commands never contact the desktop.
import { createHash, randomBytes } from "node:crypto";
import {
	constants,
	closeSync,
	existsSync,
	fstatSync,
	lstatSync,
	mkdirSync,
	openSync,
	readFileSync,
	statSync,
	writeFileSync,
} from "node:fs";
import { basename, dirname, isAbsolute, join } from "node:path";
import { execFileSync } from "node:child_process";
const label = /^[a-z][a-z0-9-]{0,47}$/;
const commandGrant = /^command:[a-z][a-z0-9-]{0,47}:[a-f0-9]{64}$/;
const uuid = /^[a-f0-9]{8}-(?:[a-f0-9]{4}-){3}[a-f0-9]{12}$/i;
function requireOk(condition: unknown, message: string): asserts condition {
	if (!condition) throw Error(message);
}
export function agentDoc(stub = false): string {
	const checkout = basename(import.meta.dir) === "scripts";
	const file = checkout
		? join(
				import.meta.dir,
				stub ? "../skills/script-kit/SKILL.md" : "../kit-init/agent-manual.md",
			)
		: join(
				import.meta.dir,
				"../share/external-agent",
				stub ? "SKILL.md" : "manual.md",
			);
	return readFileSync(file, "utf8");
}
function privateFile(path: string, privateOnly = true, limit = 4096): Buffer {
	const fd = openSync(
		path,
		constants.O_RDONLY | constants.O_NOFOLLOW | constants.O_NONBLOCK,
	);
	try {
		const st = fstatSync(fd);
		requireOk(
			st.isFile() &&
				st.size <= limit &&
				(!privateOnly || (!(st.mode & 0o077) && st.uid === process.getuid?.())),
			"Expected an owner-only regular credential file or a bounded regular source file",
		);
		return readFileSync(fd);
	} finally {
		closeSync(fd);
	}
}
export function credentialToken(path: string): string {
	return privateFile(path).toString().trim();
}
export function operator(args: string[], root: string) {
	const [verb, name, ...rest] = args;
	if (verb === "install-skill") {
		requireOk(
			isAbsolute(name ?? "") && !rest.length,
			"Usage: scriptkit install-skill /absolute/skills/directory",
		);
		const dir = join(name, "script-kit");
		mkdirSync(dir, { mode: 0o700 });
		writeFileSync(join(dir, "SKILL.md"), agentDoc(true), {
			mode: 0o600,
			flag: "wx",
		});
		return { installed: dir };
	}
	if (verb === "inspect-command") {
		requireOk(
			label.test(name ?? "") && !rest.length,
			"Usage: scriptkit inspect-command <main-script-stem>",
		);
		const path = join(root, "plugins/main/scripts", name + ".ts");
		const src = privateFile(path, false, 262144);
		return {
			name,
			path,
			effect:
				src
					.toString()
					.split(/\r?\n/)
					.find((l) => l.startsWith("// Agent effect: "))
					?.slice(17) ?? null,
			source: src.toString(),
			sha256: createHash("sha256").update(src).digest("hex"),
			trust:
				"Review the entire source and its dependencies. This runs as your account, not in a sandbox.",
		};
	}
	requireOk(
		verb === "grant-agent" && label.test(name ?? "") && rest.length >= 2,
		"Usage: scriptkit grant-agent <name> /private/path/token notes:quiet|handoff:receive|command:<stem>:<sha256> ...",
	);
	const [credential, ...scopes] = rest;
	requireOk(
		isAbsolute(credential) &&
			scopes.every(
				(s) =>
					s === "notes:quiet" ||
					s === "handoff:receive" ||
					/^handoff:send:[a-z][a-z0-9-]{0,47}$/.test(s) ||
					commandGrant.test(s),
			),
		"Invalid credential path or grant",
	);
	const dir = lstatSync(dirname(credential));
	requireOk(
		dir.isDirectory() && !(dir.mode & 0o077) && dir.uid === process.getuid?.(),
		"Choose an owned private directory (0700) for the credential",
	);
	const reg = join(root, "agents");
	mkdirSync(reg, { recursive: true, mode: 0o700 });
	const regStat = lstatSync(reg);
	requireOk(
		regStat.isDirectory() &&
			!(regStat.mode & 0o022) &&
			regStat.uid === process.getuid?.(),
		"Agent registry must be an owned, non-writable-by-others directory",
	);
	const file = join(reg, name + ".json");
	requireOk(!existsSync(file), "Agent already registered; no grant changed");
	const token = randomBytes(32).toString("hex");
	writeFileSync(credential, token + "\n", { mode: 0o600, flag: "wx" });
	writeFileSync(
		file,
		JSON.stringify({
			tokenHash: createHash("sha256").update(token).digest("hex"),
			scopes: scopes.map((s) =>
				s === "handoff:receive" ? `handoff:receive:${name}` : s,
			),
		}) + "\n",
		{ mode: 0o600, flag: "wx" },
	);
	return {
		name,
		credentialFile: credential,
		registration: file,
		revocation: "Remove the named registration to revoke this bearer.",
	};
}
type Rpc = (method: string, params: unknown, token: string) => Promise<any>;
export async function agent(args: string[], rpc: Rpc) {
	const [verb, a, b, ...extra] = args;
	const raw = process.env.SCRIPT_KIT_AGENT_CREDENTIAL
		? credentialToken(process.env.SCRIPT_KIT_AGENT_CREDENTIAL)
		: process.env.SCRIPT_KIT_MCP_TOKEN;
	requireOk(
		raw,
		"Set SCRIPT_KIT_AGENT_CREDENTIAL to your scoped token file. No operator token was used.",
	);
	const call = async (name: string, parameters: unknown) => {
		const r = (await rpc("tools/call", { name, arguments: parameters }, raw))
			.result;
		let body;
		try {
			body = JSON.parse(r.content[0].text);
		} catch {
			throw Error("Invalid tool receipt");
		}
		requireOk(
			!r.isError && body.ok !== false,
			body.error?.code ?? "Tool refused",
		);
		return body;
	};
	const stdin = async (limit = 65536) => {
		const parts: Buffer[] = [];
		let n = 0;
		for await (const chunk of process.stdin) {
			const p = Buffer.from(chunk);
			requireOk((n += p.length) <= limit, "Input exceeds byte limit");
			parts.push(p);
		}
		return Buffer.concat(parts).toString("utf8");
	};
	requireOk(!extra.length, "Too many arguments");
	if (verb === "discover" && !a) {
		const tools = (await rpc("tools/list", {}, raw)).result.tools,
			resources = (await rpc("resources/list", {}, raw)).result.resources;
		const { commands } = await call("kit/agent_access", {});
		return {
			connected: true,
			tools,
			resources,
			commands,
			readiness:
				"Listed operations are authorized; runtime prerequisites are checked at use.",
		};
	}
	if (verb === "note" && uuid.test(b ?? "") && (a === "save" || a === "status"))
		return call(
			"kit/quiet_note_" + a,
			a === "save" ? { requestId: b, body: await stdin() } : { requestId: b },
		);
	if (verb === "run" && label.test(a ?? "") && /^[a-f0-9]{64}$/.test(b ?? ""))
		return call("commands/" + a, {
			revision: b,
			input: JSON.parse(await stdin()),
		});
	if (verb === "available" && label.test(a ?? "") && !b) {
		const terminal = process.env.HC_TERMINAL_ID,
			session = process.env.HC_SESSION || "default";
		requireOk(
			terminal && /^t[1-9]\d*$/.test(terminal) && /^[\w-]{1,64}$/.test(session),
			"Register from your own local Herdr terminal",
		);
		const info = (...args: string[]) => {
			try {
				return JSON.parse(
					execFileSync("hc", ["--json", ...args], {
						env: { ...process.env, HC_SESSION: session },
						timeout: 4000,
						maxBuffer: 1024 * 1024,
						stdio: ["ignore", "pipe", "pipe"],
					}).toString(),
				);
			} catch {
				throw Error(
					"Could not read the current local Herdr terminal identity. Check hc and the selected session.",
				);
			}
		};
		const status = info("server", "status");
		requireOk(status.core?.id, "Herdr is not running");
		const t = info("get", terminal);
		return call("kit/handoff_available", {
			name: a,
			session,
			core: status.core.id,
			terminal,
			createdMs: t.created_ms,
			pid: t.pid,
		});
	}
	if (verb === "targets" && !a) return call("kit/handoff_targets", {});
	if (verb === "send" && uuid.test(a ?? "") && uuid.test(b ?? ""))
		return call("kit/handoff_send", {
			target: a,
			requestId: b,
			body: await stdin(),
		});
	if ((verb === "status" || verb === "cancel") && uuid.test(a ?? "") && !b)
		return call("kit/handoff_" + verb, { requestId: a });
	if (verb === "away" && !a) return call("kit/handoff_away", {});
	if ((verb === "take" || verb === "reply") && uuid.test(a ?? "") && !b)
		return call(
			"kit/handoff_" + verb,
			verb === "reply" ? { id: a, body: await stdin() } : { id: a },
		);
	throw Error(
		"Unknown agent operation or invalid arguments. Read scriptkit --skill.",
	);
}
