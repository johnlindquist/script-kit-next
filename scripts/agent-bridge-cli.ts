// External agent surface of the scriptkit shim. Operator commands never contact the desktop.
import { createHash, randomBytes } from "node:crypto";
import { isDeepStrictEqual } from "node:util";
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
	readSync,
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
function agentDocPath(stub: boolean): string {
	const checkout = basename(import.meta.dir) === "scripts";
	return checkout
		? join(
				import.meta.dir,
				stub ? "../skills/script-kit/SKILL.md" : "../kit-init/agent-manual.md",
			)
		: join(
				import.meta.dir,
				"../share/external-agent",
				stub ? "SKILL.md" : "manual.md",
			);
}
export function agentDoc(stub = false): string {
	return readFileSync(agentDocPath(stub), "utf8");
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
				(!privateOnly || (st.nlink === 1 && !(st.mode & 0o077) && st.uid === process.getuid?.())),
			"Expected an owner-only regular credential file or a bounded regular source file",
		);
		return readFileSync(fd);
	} finally {
		closeSync(fd);
	}
}
function validBearer(value: unknown): value is string {
	return typeof value === "string" && value.length <= 4096 && /^[A-Za-z0-9._~+/\-]+=*$/.test(value);
}
export function credentialToken(path: string): string {
	requireOk(isAbsolute(path) && !path.split(process.platform === "win32" ? /[\\/]/ : "/").includes(".."),
		"Supply an absolute credential file path without '..'. No operator credential was used.");
	let token: string;
	try { token = privateFile(path).toString("utf8").replace(/\r?\n$/, ""); }
	catch { throw Error("The selected credential file is unavailable or not private. No operator credential was used."); }
	requireOk(validBearer(token), "The selected credential file is empty or invalid. No operator credential was used.");
	return token;
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
	// Bun's realpath can collapse a link and '..' to a different private directory.
	requireOk(!requested.split(process.platform === "win32" ? /[\\/]/ : "/").includes(".."), "Choose an absolute credential destination without '..'.");
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
function installAgentSkill(requested: string) {
	// The caller may deliberately use a directory link. Resolve that selection once,
	// then use the physical parent; do not follow the alias again to place the file.
	const parent = realpathSync.native(requested);
	const selected = statSync(requested, { bigint: true });
	const id = fileIdentity(selected);
	const parentStillSelected = () => {
		const named = statSync(requested, { bigint: true, throwIfNoEntry: false });
		const physical = lstatSync(parent, { bigint: true, throwIfNoEntry: false });
		return named?.isDirectory() && physical?.isDirectory() && fileIdentity(named) === id && fileIdentity(physical) === id;
	};
	requireOk(selected.isDirectory() && parentStillSelected(), "Choose an existing skills directory.");

	let stub: Buffer;
	try {
		stub = privateFile(agentDocPath(true), false, 262144);
		requireOk(stub.length > 0 && stub.length <= 262144, "Invalid skill size");
		new TextDecoder("utf-8", { fatal: true }).decode(stub);
	} catch {
		throw Error("Kit needs a readable, nonempty, direct UTF-8 skill file (up to 256 KiB). No skill was installed.");
	}
	requireOk(parentStillSelected(), "The selected skills directory changed. No skill was installed.");
	const dir = join(parent, "script-kit");
	try { mkdirSync(dir, { mode: 0o700 }); }
	catch (e) {
		if ((e as NodeJS.ErrnoException).code === "EEXIST") throw Error("A script-kit entry already exists in that skills directory. It was not replaced.");
		throw e;
	}

	const unconfirmed = "Could not confirm the skill installation. Inspect the selected script-kit directory before trying again.";
	try {
		const created = lstatSync(dir, { bigint: true });
		const folderStillOurs = () => {
			const named = lstatSync(dir, { bigint: true, throwIfNoEntry: false });
			return named?.isDirectory() && fileIdentity(named) === fileIdentity(created) && parentStillSelected();
		};
		requireOk(created.isDirectory() && folderStillOurs(), unconfirmed);
		const file = join(dir, "SKILL.md"), fd = openSync(file, "wx+", 0o600);
		try {
			const opened = fstatSync(fd, { bigint: true });
			const fileStillOurs = () => {
				const named = lstatSync(file, { bigint: true, throwIfNoEntry: false });
				return named?.isFile() && named.nlink === 1n && fileIdentity(named) === fileIdentity(opened) && folderStillOurs();
			};
			requireOk(fileStillOurs(), unconfirmed);
			writeFileSync(fd, stub);
			const saved = Buffer.alloc(stub.length);
			for (let pos = 0; pos < saved.length;) {
				const n = readSync(fd, saved, pos, saved.length - pos, pos);
				requireOk(n > 0, unconfirmed);
				pos += n;
			}
			requireOk(saved.equals(stub) && fstatSync(fd).size === stub.length && fileStillOurs(), unconfirmed);
		} finally { closeSync(fd); }
	} catch { throw Error(unconfirmed); } // Never remove an entry that another installer may now own.
	return { installed: dir };
}

export function operator(args: string[], root: string) {
	const [verb, name, ...rest] = args;
	if (verb === "install-skill") {
		requireOk(typeof name === "string" && isAbsolute(name) && !rest.length &&
			!name.split(process.platform === "win32" ? /[\\/]/ : "/").includes(".."),
			"Choose an existing absolute skills directory without '..'. No skill was installed.");
		return installAgentSkill(name);
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
		"Usage: scriptkit grant-agent <name> /private/path/token notes:quiet|handoff:receive|launcher:read|launcher:apply|command:<stem>:<sha256> ...",
	);
	const [credential, ...scopes] = rest;
	requireOk(
		isAbsolute(credential) &&
			scopes.every(
				(s) =>
					s === "notes:quiet" ||
					s === "launcher:read" ||
					s === "launcher:apply" ||
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
class ToolRefusal extends Error {}
const launcherFields = new Map<string, string | null>([
	["describe", null],
	["inspect", null],
	["live", null],
	["why", "target"],
	["status", "requestId"],
	["preview", "change"],
	["apply", "request"],
]);
const object = (v: unknown): v is Record<string, unknown> =>
	v !== null && typeof v === "object" && !Array.isArray(v);
const digest = (v: unknown) => typeof v === "string" && /^[a-f0-9]{64}$/.test(v);
const revision = (v: unknown) => v === "stock" || digest(v);
// Store::state and Store::preview expose declared rules even while disabled. The effective
// rules must be those same declarations when enabled, or LauncherRules::default() when not.
const stockRules = { pinnedText: "muted", pinnedWeight: "regular", selectionFirst: null, clipboardFirst: null, homeMode: "standard" };
function snapshot(v: unknown): v is Record<string, unknown> {
	if (!object(v) || !digest(v.workspace) || !revision(v.revision) || typeof v.enabled !== "boolean") return false;
	const { declared, effective, undo } = v;
	return object(declared) && object(effective) && Object.keys(stockRules).every(k => Object.hasOwn(declared, k))
		&& isDeepStrictEqual(effective, v.enabled ? declared : stockRules)
		&& (undo === null || (v.revision !== "stock" && requestIdentity(undo) !== undefined));
}

// Rust UUID receipts use the canonical spelling, including for accepted upper-case inputs.
function requestIdentity(value: unknown): string | undefined {
	if (typeof value !== "string" || value.length > 45) return;
	let s = value.toLowerCase();
	if (s.startsWith("urn:uuid:")) s = s.slice(9);
	else if (s.startsWith("{") && s.endsWith("}")) s = s.slice(1, -1);
	if (uuid.test(s) || /^[a-f0-9]{32}$/.test(s)) return s.replaceAll("-", "");
}
function sameRequest(a: unknown, b: unknown): boolean {
	const expected = requestIdentity(a);
	return expected !== undefined && expected === requestIdentity(b);
}

// Check the requested declaration, not just a well-shaped snapshot of some change.
// The store owns validation and history; the client cannot infer prior values for Undo.
const declaredFields = new Map([
	["launcher.home.mode", "homeMode"],
	["launcher.header.pinned-label.textRole", "pinnedText"],
	["launcher.header.pinned-label.weight", "pinnedWeight"],
	["launcher.subject.selection.firstAction", "selectionFirst"],
	["launcher.subject.clipboard.firstAction", "clipboardFirst"],
]);
function reflectsChange(change: unknown, state: Record<string, unknown>, savedId?: unknown): boolean {
	if (!object(change)) return false;
	// For the original stored revision, the new non-Undo operation is precisely what can be undone.
	if (savedId !== undefined && change.op !== "undo" && !sameRequest(savedId, state.undo)) return false;
	switch (change.op) {
		case "enable": return state.enabled === true;
		case "disable": return state.enabled === false;
		case "edit": {
			const edit = change.edit;
			if (!object(edit) || typeof edit.target !== "string" || !Object.hasOwn(edit, "value")) return false;
			const field = declaredFields.get(edit.target);
			return field !== undefined && object(state.declared) && state.declared[field] === edit.value
				&& (state.enabled === false || (object(state.effective) && state.effective[field] === edit.value));
		}
		case "putView":
		case "removeView": {
			const put = change.op === "putView";
			const id = put ? (object(change.view) ? change.view.id : undefined) : change.id;
			// The store omits an empty list; null or malformed members are not evidence of absence.
			const views = state.views === undefined ? [] : state.views;
			if (typeof id !== "string" || !Array.isArray(views) || !views.every(v => object(v) && typeof v.id === "string")) return false;
			const matches = views.filter(v => v.id === id);
			return put ? matches.length === 1 && isDeepStrictEqual(matches[0], change.view) : matches.length === 0;
		}
		case "undo":
			if (requestIdentity(change.of) === undefined) return false;
			// Preview keeps the current undo tip. A committed Undo consumes it, exposing the
			// preceding tip or null. The preceding values aren't independently known here.
			return savedId === undefined ? sameRequest(state.undo, change.of) :
				!sameRequest(savedId, change.of) && !sameRequest(state.undo, change.of) && !sameRequest(state.undo, savedId);
		default: return false;
	}
}

// A matching JSON-RPC id binds the network response, not the underlying edit or window.
function launcherReceipt(verb: string, args: Record<string, unknown>, data: unknown): boolean {
	if (!object(data)) return false;
	switch (verb) {
		case "describe":
			return data.schemaVersion === 1 && Array.isArray(data.targets) && Array.isArray(data.operations);
		case "inspect":
			return data.scope === "stored" && typeof data.safeMode === "boolean" && snapshot(data.state);
		case "preview":
			return data.scope === "preview" && snapshot(data.wouldStore) && reflectsChange(args.change, data.wouldStore);
		case "live": {
			if (data.scope !== "runningWindow") return false;
			if (data.available === false) return !Object.hasOwn(data, "adoption");
			const a = data.adoption;
			return data.available === true && object(a) && object(a.effective) && typeof a.pendingWrite === "boolean"
				&& ((a.workspace === null && a.appliedRevision === null) || (digest(a.workspace) && revision(a.appliedRevision)));
		}
		case "why":
			return typeof args.target === "string" && data.target === args.target && typeof data.origin === "string" && revision(data.revision);
		case "status":
			return sameRequest(args.requestId, data.requestId) && revision(data.storedRevision)
				// Stock has no active document or commit chain, even if an orphan journal exists.
				&& (data.status === "committed" ? digest(data.storedRevision) : data.status === "unknown" || data.status === "notInActiveHistory");
		case "apply": {
			const req = args.request, res = data.result;
			if (data.scope !== "stored" || !object(req) || !object(res) || !snapshot(res.state)) return false;
			// A replay identifies the ORIGINAL commit, but carries the CURRENT state. A first
			// application must report its new revision in both places; neither proves UI adoption.
			return sameRequest(req.requestId, res.requestId) && req.workspace === res.state.workspace
				&& revision(req.expectedRevision) && digest(res.appliedRevision) && res.appliedRevision !== req.expectedRevision
				&& digest(res.state.revision) && typeof res.replayed === "boolean"
				&& (res.state.revision === res.appliedRevision ? reflectsChange(req.change, res.state, req.requestId) : res.replayed);
		}
		default: return false;
	}
}
function toolBody(result: unknown, launcher: boolean): Record<string, unknown> {
	requireOk(object(result) && Array.isArray(result.content) && typeof result.content[0]?.text === "string"
		&& (!launcher || (result.content.length === 1 && result.content[0].type === "text")), "Invalid tool receipt");
	let body: unknown;
	try { body = JSON.parse(result.content[0].text); } catch { throw Error("Invalid tool receipt"); }
	requireOk(object(body), "Invalid tool receipt");
	return body;
}

// Only these store/permission checks are known to end this attempt before publishing the active file.
// In particular, stale_revision and wrong_workspace can also be raised during the *post*-rename read.
const beforeSave = new Set(["scope_denied", "safe_mode", "invalid_request", "writer_busy", "nothing_to_undo",
	"invalid_view", "view_limit", "view_not_found", "unsupported"]);
const reconcileReasons = new Set(["storage_unavailable", "stale_revision", "wrong_workspace", "unsafe_path",
	"invalid_document", "request_conflict", "history_limit"]);
// A read and a write must agree on what the peer actually said. Never pick one
// of two conflicting MCP representations or credit a success that also carries an error.
function launcherReply(response: unknown) {
	if (!object(response) || !object(response.result)) return;
	const result = response.result;
	let body: Record<string, unknown>;
	try { body = toolBody(result, true); } catch { return; }
	if (Object.hasOwn(result, "structuredContent") && !isDeepStrictEqual(result.structuredContent, body)) return;
	if (result.isError === true && body.ok === false && !Object.hasOwn(body, "data") && object(body.error) && typeof body.error.code === "string") {
		return { kind: "failure" as const, code: body.error.code };
	}
	if ((result.isError === undefined || result.isError === false) && body.ok === true && !Object.hasOwn(body, "error") && Object.hasOwn(body, "data")) {
		return { kind: "success" as const, data: body.data };
	}
}

class UnverifiedLauncherRead extends Error {
	readonly code = "launcher_read_unverified";
	constructor(op?: string) { super("Invalid launcher " + (op ?? "read") + " receipt. No launcher state was verified."); }
}

export async function checkedLauncherRead(args: unknown, send: () => Promise<unknown>) {
	const response = await send();
	const receipt = launcherReply(response);
	if (receipt?.kind === "failure") {
		throw new ToolRefusal(beforeSave.has(receipt.code) || reconcileReasons.has(receipt.code) ? receipt.code : "Launcher read refused");
	}
	const op = object(args) && typeof args.op === "string" && args.op !== "apply" && launcherFields.has(args.op) ? args.op : undefined;
	if (receipt?.kind === "success" && op && object(args) && launcherReceipt(op, args, receipt.data)) {
		return { response, data: receipt.data };
	}
	throw new UnverifiedLauncherRead(op);
}

class UnverifiedLauncherApply extends Error {
	readonly code = "launcher_apply_unverified";
	constructor(reason?: string) {
		super(`Launcher apply could not be verified${reason ? ` (${reason})` : ""}. Check launcher status with the original requestId in the same workspace. Do not resend automatically.`);
	}
}

/** Send exactly once. The named MCP CLI preserves its raw envelope on success; the typed CLI returns the stored result. */
export async function checkedLauncherApply(args: unknown, send: () => Promise<unknown>) {
	let reason: string | undefined;
	try {
		const response = await send();
		const receipt = launcherReply(response);
		if (receipt?.kind === "failure") {
			if (beforeSave.has(receipt.code)) throw new ToolRefusal(receipt.code);
			if (reconcileReasons.has(receipt.code)) reason = receipt.code;
		} else if (receipt?.kind === "success" && object(args) && args.op === "apply" && launcherReceipt("apply", args, receipt.data)) {
			return { response, data: receipt.data };
		}
	} catch (error) {
		// A malformed response, transport failure, or validation exception does not undo a save.
		if (error instanceof ToolRefusal || (error instanceof Error && "code" in error && error.code === "mcp_request_not_sent")) throw error;
	}
	throw new UnverifiedLauncherApply(reason);
}

export async function agent(
	args: string[],
	rpc: Rpc,
	input: AsyncIterable<Uint8Array | string> = process.stdin,
) {
	const [verb, a, b, ...extra] = args;
	const file = process.env.SCRIPT_KIT_AGENT_CREDENTIAL;
	const raw = file === undefined ? process.env.SCRIPT_KIT_MCP_TOKEN : credentialToken(file);
	requireOk(
		validBearer(raw),
		"Set SCRIPT_KIT_AGENT_CREDENTIAL to your scoped token file. No operator token was used.",
	);
	const call = async (name: string, parameters: unknown) => {
		const r = (await rpc("tools/call", { name, arguments: parameters }, raw))
			.result;
		const body = toolBody(r, false);
		if (r.isError || body.ok === false) {
			const code = object(body.error) ? body.error.code : null;
			throw new ToolRefusal(typeof code === "string" && /^[a-z][a-z0-9_]{0,63}$/.test(code) && !code.includes(raw) ? code : "Tool refused");
		}
		return body;
	};
	const stdin = async (limit = 65536) => {
		const parts: Buffer[] = [];
		let n = 0;
		for await (const chunk of input) {
			const p = Buffer.from(chunk);
			requireOk((n += p.length) <= limit, "Input exceeds byte limit");
			parts.push(p);
		}
		// Decode after the last chunk: a multibyte character can span pipe writes.
		const bytes = Buffer.concat(parts, n);
		try {
			// A leading U+FEFF in a note or handoff is part of its text.
			return new TextDecoder("utf-8", { fatal: true, ignoreBOM: true }).decode(bytes);
		} catch {
			throw Error("Input must be valid UTF-8. No request was sent.");
		}
	};
	requireOk(!extra.length, "Too many arguments");
	if (verb === "launcher") {
		const field = launcherFields.get(a);
		requireOk(
			field !== undefined && b === undefined,
			"Usage: scriptkit agent launcher describe|inspect|live|why|status|preview|apply",
		);
		const parameters: Record<string, unknown> = { op: a };
		if (field !== null) {
			const body = JSON.parse(await stdin());
			requireOk(
				body !== null &&
					typeof body === "object" &&
					!Array.isArray(body) &&
					Object.keys(body).length === 1 &&
					Object.hasOwn(body, field),
				"Expected one JSON field: " + field,
			);
			parameters[field] = body[field];
		}
		if (a === "apply") return (await checkedLauncherApply(parameters, () =>
			rpc("tools/call", { name:"kit/launcher_apply", arguments:parameters }, raw))).data;
		return (await checkedLauncherRead(parameters, () =>
			rpc("tools/call", { name:"kit/launcher_read", arguments:parameters }, raw))).data;
	}
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
