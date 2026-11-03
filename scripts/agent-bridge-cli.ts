// External agent surface of the scriptkit shim. Operator commands never contact the desktop.
import { createHash, randomBytes } from "node:crypto";
import {
	constants,
	closeSync,
	fstatSync,
	fsyncSync,
	lstatSync,
	mkdirSync,
	opendirSync,
	openSync,
	readFileSync,
	realpathSync,
	statSync,
	unlinkSync,
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
// The server trusts agent-token as operator authority, independently of scoped registrations.
// Kit must never mint a scoped bearer there or among files read as personal content or code.
const reservedGrantSlots = new Set([
	"agent-token", "server.json", "server-location.json", "theme.json", "config.ts", "secrets.age",
	"agents", "plugins", "themes", "brain", "sdk", "bin", "share",
]);
const grantLocationError = "Choose a private credential destination outside Kit’s managed files and content";
const catalogError = "Cannot verify Kit’s linked scripts; check that the plugin script folders and links are readable. No credential was issued.";
const fileIdentity = (stat: { dev: bigint; ino: bigint }) => stat.dev + ":" + stat.ino;
const scriptFolder = (root: string, plugin: string) => join(root, "plugins", plugin, "scripts");

function* folderEntries(path: string) {
	let dir;
	try { dir = opendirSync(path); }
	catch { throw Error(catalogError); }
	try {
		for (let entry; (entry = dir.readSync()) !== null;) yield entry;
	} catch { throw Error(catalogError); }
	finally { dir.closeSync(); }
}
function catalogStat(path: string) {
	try { return statSync(path, { bigint: true, throwIfNoEntry: false }); }
	catch { throw Error(catalogError); }
}
// The launcher reads direct .ts/.js entries in each plugin's scripts folder, including
// links. inspect-command uses the main plugin. Check the catalog's view of a destination,
// not just the destination's view of the workspace. No program or manifest is read.
function linkedScriptEntries(root: string, parents: Set<string>): string[] {
	const links: string[] = [], visited = new Set<string>();
	let entries = 0;
	const count = () => requireOk(++entries <= 65_536, "Too many Kit plugin or script entries to verify the credential destination; no grant was issued");
	const container = join(root, "plugins"), catalog = catalogStat(container);
	if (!catalog) return links; // A first-run workspace may not have any plugins yet.
	requireOk(catalog.isDirectory(), catalogError);
	for (const plugin of folderEntries(container)) {
		count();
		if (!plugin.isDirectory() && !(plugin.isSymbolicLink() && catalogStat(join(container, plugin.name))?.isDirectory())) continue;
		const folder = scriptFolder(root, plugin.name), st = catalogStat(folder);
		if (!st?.isDirectory()) continue;
		const id = fileIdentity(st);
		requireOk(!parents.has(id), grantLocationError);
		if (visited.has(id)) continue;
		visited.add(id);
		for (const entry of folderEntries(folder)) {
			count();
			if (entry.isSymbolicLink() && /\.(ts|js)$/.test(entry.name)) links.push(join(folder, entry.name));
		}
	}
	return links;
}
function grantDestination(root: string, requested: string) {
	const dir = dirname(requested), leaf = basename(requested), owner = lstatSync(dir);
	requireOk(owner.isDirectory() && !(owner.mode & 0o077) && owner.uid === process.getuid?.(),
		"Choose an owned private directory (0700) for the credential");
	requireOk(leaf && leaf !== "." && leaf !== ".." && !requested.endsWith("/"), "Expected a credential filename");
	const physical = realpathSync.native(dir);
	const kit = statSync(root, { bigint: true, throwIfNoEntry: false });
	let scriptLinks: string[] = [];
	if (kit) {
		const parents = new Set<string>();
		// A private-looking alias can enter Kit's scripts; a directory under plugins can also
		// link out. Compare the workspace identity along both routes, not just name prefixes.
		for (const parent of new Set([dir, physical])) {
			let ancestor = parent, slot = leaf;
			for (;;) {
				const id = statSync(ancestor, { bigint: true });
				parents.add(fileIdentity(id));
				if (id.dev === kit.dev && id.ino === kit.ino) {
					requireOk(!reservedGrantSlots.has(slot.toLowerCase()), grantLocationError);
					break;
				}
				const up = dirname(ancestor);
				if (up === ancestor) break;
				slot = basename(ancestor); ancestor = up;
			}
		}
		const master = lstatSync(join(root, "agent-token"), { throwIfNoEntry: false });
		requireOk(!master || (master.isFile() && master.nlink === 1),
			"Kit’s operator token is not a direct regular file; no scoped grant was created");
		scriptLinks = linkedScriptEntries(root, parents);
	}
	return { destination: join(physical, leaf), scriptLinks };
}

// Reserve both names without overwriting. An empty registration authenticates nobody. On
// error, remove only our own reservation, not a file that has taken its place.
function withGrantFile(path: string, collision: string, use: (fd: number, stillOurs: () => boolean) => void) {
	let fd: number;
	try { fd = openSync(path, "wx", 0o600); }
	catch (e) { if ((e as NodeJS.ErrnoException).code === "EEXIST") throw Error(collision); throw e; }
	try {
		const opened = fstatSync(fd, { bigint: true });
		const stillOurs = () => {
			const named = lstatSync(path, { bigint: true, throwIfNoEntry: false });
			return named?.dev === opened.dev && named.ino === opened.ino;
		};
		try { use(fd, stillOurs); }
		catch (e) { if (stillOurs()) unlinkSync(path); throw e; }
	} finally { closeSync(fd); }
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
		const path = join(scriptFolder(root, "main"), name + ".ts");
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
	const { destination, scriptLinks } = grantDestination(root, credential);
	const reg = join(root, "agents");
	mkdirSync(reg, { recursive: true, mode: 0o700 });
	const regStat = lstatSync(reg);
	requireOk(
		regStat.isDirectory() &&
			!(regStat.mode & 0o022) &&
			regStat.uid === process.getuid?.(),
		"Agent registry must be an owned, non-writable-by-others directory",
	);
	const file = join(realpathSync.native(reg), name + ".json");
	withGrantFile(file, "Agent already registered; no grant changed", (record, ownsRecord) => {
		withGrantFile(destination, "Credential file already exists; no grant changed", (secret, ownsSecret) => {
			// The exclusive output is still empty. The OS can now resolve even a previously
			// dangling script link through a chain of aliases, before any bearer exists.
			const empty = fstatSync(secret, { bigint: true });
			requireOk(empty.nlink === 1n, grantLocationError);
			for (const link of scriptLinks) {
				const target = catalogStat(link);
				requireOk(!target || empty.dev !== target.dev || empty.ino !== target.ino, grantLocationError);
			}
			const token = randomBytes(32).toString("hex");
			writeFileSync(secret, token + "\n");
			fsyncSync(secret);
			requireOk(ownsRecord() && ownsSecret(), "A grant destination changed; no grant was activated");
			writeFileSync(record, JSON.stringify({
				tokenHash: createHash("sha256").update(token).digest("hex"),
				scopes: scopes.map((s) => s === "handoff:receive" ? "handoff:receive:" + name : s),
			}) + "\n");
			fsyncSync(record);
		});
	});
	return {
		name,
		credentialFile: destination,
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
