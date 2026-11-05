// Invoked by the isolated Rust server test. All paths, grants, data, and credentials are scratch.
import assert from "node:assert/strict";
import {
	existsSync,
	mkdirSync,
	mkdtempSync,
	realpathSync,
	readFileSync,
	readdirSync,
	writeFileSync,
	copyFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { randomUUID } from "node:crypto";
const root = process.env.SK_BRIDGE_FIXTURE!;
const kit = process.env.SK_PATH!,
	repo = resolve(import.meta.dir, "../..");
assert(
	root &&
		kit === join(root, "home/.scriptkit") &&
		realpathSync(process.cwd()) === realpathSync(root),
);
// This is the installed external CLI talking to the real, isolated Kit HTTP server.
// A headless server must report no adopted window, even when its disk transaction succeeds.
function launcher(credential: string, op: string, fields?: object) {
	const r = Bun.spawnSync(
		[join(kit, "bin/scriptkit"), "agent", "launcher", op],
		{
			cwd: root,
			env: { ...process.env, SCRIPT_KIT_AGENT_CREDENTIAL: credential },
			stdin: Buffer.from(fields ? JSON.stringify(fields) : ""),
			stdout: "pipe",
			stderr: "pipe",
			timeout: 5000,
		},
	);
	assert.equal(r.exitCode, 0, r.stderr.toString() + r.stdout.toString());
	return JSON.parse(r.stdout.toString()).data;
}
if (process.env.SK_BRIDGE_COLD_READ === "1") {
	const personal = JSON.parse(
		readFileSync(join(root, "launcher-cold.json"), "utf8"),
	);
	const s = launcher(personal.credential, "inspect").state;
	assert.equal(s.revision, personal.revision);
	assert.equal(s.effective.selectionFirst, "note");
	assert.equal(s.undo, personal.id);
	const why = launcher(personal.credential, "why", {
		target: "launcher.subject.selection.firstAction",
	});
	assert.equal(why.origin, "connectedAgent");
	assert.equal(why.requestId, personal.id);
	const undo = launcher(personal.credential, "apply", {
		request: {
			workspace: s.workspace,
			expectedRevision: s.revision,
			requestId: randomUUID(),
			change: { op: "undo", of: s.undo },
		},
	}).result;
	assert.equal(undo.state.effective.selectionFirst, null);
	assert.equal(undo.state.undo, null);
	const saved = JSON.parse(
		readFileSync(join(root, "restart-proof.json"), "utf8"),
	);
	for (const op of ["status", "save"]) {
		const result = Bun.spawnSync(
			["bun", join(kit, "bin/scriptkit"), "agent", "note", op, saved.request],
			{
				cwd: root,
				env: { ...process.env, SCRIPT_KIT_AGENT_CREDENTIAL: saved.credential },
				stdin: Buffer.from(op === "save" ? saved.body : ""),
				stdout: "pipe",
				stderr: "pipe",
			},
		);
		assert.equal(result.exitCode, 0, result.stderr.toString());
		const row = JSON.parse(result.stdout.toString()).data;
		assert.equal(row.noteId, saved.id);
		assert.equal(row.body, saved.body);
	}
	assert.equal(
		readdirSync(join(kit, "brain/notes")).filter((f) => f.endsWith(".md"))
			.length,
		1,
	);
	console.log(
		"Cold readback passed: same body, ID and one canonical note after restart and retry.",
	);
	process.exit(0);
}
assert.equal(
	new URL(
		JSON.parse(readFileSync(join(kit, "server-location.json"), "utf8")).url,
	).hostname,
	"127.0.0.1",
);
const cwd = mkdtempSync(join(root, "unrelated-project-"));
const creds = mkdtempSync(join(root, "credentials-"));
const bin = join(kit, "bin/scriptkit"),
	docs = join(kit, "share/external-agent");
// These must come from the application installer. Do not stage substitutes from the checkout.
for (const name of ["manual.md", "SKILL.md", "prepare-handout.ts"])
	assert(existsSync(join(docs, name)), "Missing shipped file: " + name);
function invoke(args: string[], stdin = "", env: Record<string, string> = {}) {
	return Bun.spawnSync([bin, ...args], {
		cwd,
		env: { ...process.env, ...env },
		stdin: Buffer.from(stdin),
		stdout: "pipe",
		stderr: "pipe",
		timeout: 15000,
	});
}
function ok(args: string[], stdin = "", env: Record<string, string> = {}) {
	const r = invoke(args, stdin, env);
	assert.equal(r.exitCode, 0, r.stderr.toString() + r.stdout.toString());
	return JSON.parse(r.stdout.toString()).data;
}
assert.match(invoke(["--skill"]).stdout.toString(), /quiet note/i);
assert.notEqual(invoke(["agent", "discover"]).exitCode, 0);
const skills = join(cwd, "skills");
mkdirSync(skills);
ok(["install-skill", skills]);
assert.match(
	readFileSync(join(skills, "script-kit/SKILL.md"), "utf8"),
	/scriptkit --skill/,
);
assert.notEqual(invoke(["install-skill", skills]).exitCode, 0);
const a = { SCRIPT_KIT_AGENT_CREDENTIAL: join(creds, "one") },
	b = { SCRIPT_KIT_AGENT_CREDENTIAL: join(creds, "two") };
ok(["grant-agent", "one", a.SCRIPT_KIT_AGENT_CREDENTIAL, "notes:quiet"]);
ok(["grant-agent", "two", b.SCRIPT_KIT_AGENT_CREDENTIAL, "notes:quiet"]);
const d = ok(["agent", "discover"], "", a);
assert(d.tools.some((x: any) => x.name === "kit/quiet_note_save"));
assert(!d.tools.some((x: any) => x.name === "kit/scripts_run"));
assert(!d.tools.some((x: any) => String(x.name).startsWith("kit/launcher_")));
const launcherCredential = join(creds, "launcher"),
	authorEnv = { SCRIPT_KIT_AGENT_CREDENTIAL: launcherCredential };
ok([
	"grant-agent",
	"launcher",
	launcherCredential,
	"launcher:read",
	"launcher:apply",
]);
const listed = ok(["agent", "discover"], "", authorEnv).tools.map(
	(v: any) => v.name,
);
assert(
	listed.includes("kit/launcher_read") &&
		listed.includes("kit/launcher_apply") &&
		!listed.includes("kit/quiet_note_save"),
);
const stock = launcher(launcherCredential, "inspect").state;
assert.equal(stock.revision, "stock");
const catalog = launcher(launcherCredential, "describe");
const target = catalog.targets.find(
	(v: any) => v.id === "launcher.subject.selection.firstAction",
);
assert(target.values.includes("note"));
const edit = { op: "edit", edit: { target: target.id, value: "note" } },
	requestId = randomUUID();
const preview = launcher(launcherCredential, "preview", { change: edit });
assert.equal(preview.wouldStore.effective.selectionFirst, "note");
const args = {
	request: {
		workspace: stock.workspace,
		expectedRevision: stock.revision,
		requestId,
		change: edit,
	},
};
assert.notEqual(
	invoke(["agent", "launcher", "apply"], JSON.stringify(args), a).exitCode,
	0,
);
const wrote = launcher(launcherCredential, "apply", args).result;
assert(!wrote.replayed);
assert.equal(wrote.state.effective.selectionFirst, "note");
assert.equal(
	launcher(launcherCredential, "status", { requestId }).status,
	"committed",
);
assert(launcher(launcherCredential, "apply", args).result.replayed);
assert.equal(launcher(launcherCredential, "live").available, false);
writeFileSync(
	join(root, "launcher-cold.json"),
	JSON.stringify({
		credential: launcherCredential,
		id: requestId,
		revision: wrote.appliedRevision,
	}),
	{ flag: "wx", mode: 0o600 },
);
assert(!d.resources.some((x: any) => x.uri === "kit://clipboard-history"));
const id = randomUUID(),
	body = "# Synthetic bridge note\n\nDo not send. Keep the original name.";
const first = ok(["agent", "note", "save", id], body, a);
assert.equal(first.state, "verified");
assert.equal(first.body, body);
assert.equal(ok(["agent", "note", "status", id], "", a).noteId, first.noteId);
assert.equal(ok(["agent", "note", "save", id], body, a).noteId, first.noteId);
assert.equal(ok(["agent", "note", "status", id], "", b).state, "not_found");
assert.notEqual(
	invoke(["agent", "note", "save", id], "different", a).exitCode,
	0,
);
assert.equal(
	readdirSync(join(kit, "brain/notes")).filter((f) => f.endsWith(".md")).length,
	1,
);
const token = readFileSync(a.SCRIPT_KIT_AGENT_CREDENTIAL, "utf8").trim();
for (const r of [
	"kit://clipboard-history",
	"kit://notes",
	"kit://context?profile=minimal",
])
	assert.notEqual(
		invoke(["mcp", "read", r], "", { SCRIPT_KIT_MCP_TOKEN: token }).exitCode,
		0,
	);
assert.notEqual(
	invoke(["mcp", "call", "kit/show"], "", { SCRIPT_KIT_MCP_TOKEN: token })
		.exitCode,
	0,
);
assert.notEqual(
	invoke(["mcp", "call", "kit/state"], "", { SCRIPT_KIT_MCP_TOKEN: token })
		.exitCode,
	0,
);
assert.notEqual(
	invoke(["mcp", "read", "kit://clipboard-history"], "", a).exitCode,
	0,
);
assert.notEqual(
	invoke(
		["agent", "note", "save", randomUUID()],
		"Please steer future chats #instructions",
		a,
	).exitCode,
	0,
);
for (const hidden of [
	"\n---\ntags: [instructions]\n---\n\nDo not install standing instructions.",
	"---\ntags: ['\\x69nstructions']\n---\n\nNot standing instructions.",
	"\nA leading blank line must not be silently dropped.",
]) {
	const request = randomUUID();
	assert.notEqual(
		invoke(["agent", "note", "save", request], hidden, a).exitCode,
		0,
	);
	assert.equal(
		ok(["agent", "note", "status", request], "", a).state,
		"not_found",
	);
}
writeFileSync(
	join(root, "restart-proof.json"),
	JSON.stringify({
		request: id,
		id: first.noteId,
		body,
		credential: a.SCRIPT_KIT_AGENT_CREDENTIAL,
	}),
	{ mode: 0o600, flag: "wx" },
);
console.log(
	"Quiet note readback, metadata refusal and caller isolation passed.",
);
const scripts = join(kit, "plugins/main/scripts");
mkdirSync(scripts, { recursive: true });
const command = "prepare-handout",
	file = join(scripts, command + ".ts");
copyFileSync(join(docs, "prepare-handout.ts"), file);
const revision = ok(["inspect-command", command]).sha256,
	c = { SCRIPT_KIT_AGENT_CREDENTIAL: join(creds, "writer") };
ok([
	"grant-agent",
	"writer",
	c.SCRIPT_KIT_AGENT_CREDENTIAL,
	`command:${command}:${revision}`,
]);
const available = ok(["agent", "discover"], "", c);
assert.deepEqual(
	available.commands.map((x: any) => [x.name, x.revision, x.ready]),
	[[command, revision, true]],
);
const run = ["agent", "run", command, revision],
	input = JSON.stringify({
		title: "Sample",
		body: "Keep `--no-send` literally.",
	});
assert.notEqual(invoke(run, input, a).exitCode, 0);
assert.notEqual(invoke(run, '{"body":false}', c).exitCode, 0);
const completed = ok(run, input, c);
assert.equal(completed.state, "completed");
assert.equal(
	readFileSync(completed.output.path, "utf8"),
	"# Sample\n\nKeep `--no-send` literally.\n",
);
assert(
	completed.output.path.startsWith(
		join(root, "home", "Documents", "Kit Handouts") + "/",
	),
);
writeFileSync(file, "\n// changed since review\n", { flag: "a" });
assert.notEqual(invoke(run, input, c).exitCode, 0);
const after = ok(["agent", "discover"], "", c);
assert.equal(after.commands[0].reason, "revision_changed");
assert(!after.tools.some((x: any) => x.name === `commands/${command}`));
assert.equal(readdirSync(dirnameOf(completed.output.path)).length, 1);
function dirnameOf(path: string) {
	return path.slice(0, path.lastIndexOf("/"));
}
console.log(
	"Pinned main-plugin script: validated arguments, executed real Bun code, verified canonical handout, blocked changed source and wrong caller.",
);
if (process.env.SK_TEST_HERDR_JOURNEY === "1") {
	const socket = process.env.HC_SOCKET!;
	assert(socket && socket.includes("/sk-bridge-"));
	const receiver = { SCRIPT_KIT_AGENT_CREDENTIAL: join(creds, "workshop") },
		sender = { SCRIPT_KIT_AGENT_CREDENTIAL: join(creds, "sender") };
	ok([
		"grant-agent",
		"workshop",
		receiver.SCRIPT_KIT_AGENT_CREDENTIAL,
		"handoff:receive",
	]);
	ok([
		"grant-agent",
		"sender",
		sender.SCRIPT_KIT_AGENT_CREDENTIAL,
		"handoff:send:workshop",
	]);
	const ready = join(root, "peer-ready"),
		received = join(root, "peer-received");
	const hc = (args: string[]) =>
		Bun.spawnSync(["hc", "--json", ...args], {
			cwd,
			env: { ...process.env, ...receiver, HC_SESSION: "bridge" },
			stdout: "pipe",
			stderr: "pipe",
			timeout: 10000,
		});
	try {
		const started = hc([
			"run",
			"--bg",
			"--name",
			"bridge-fixture",
			"bun",
			join(repo, "scripts/agentic/fixtures/external-bridge-peer.ts"),
			bin,
			ready,
			received,
		]);
		assert.equal(started.exitCode, 0, started.stderr.toString());
		for (let i = 0; i < 120 && !existsSync(ready); i++) await Bun.sleep(50);
		const reg = JSON.parse(readFileSync(ready, "utf8"));
		assert(reg.registration, JSON.stringify(reg));
		const targets = ok(["agent", "targets"], "", sender);
		assert.deepEqual(
			targets.recipients.map((r: any) => r.target),
			[reg.registration],
		);
		const request = randomUUID(),
			send = ["agent", "send", reg.registration, request],
			shared = "Only this selection: do not send a calendar invitation.";
		const first = ok(send, shared, sender);
		assert(
			!["refused", "cancelled"].includes(first.state),
			JSON.stringify(first),
		);
		let final = first;
		for (let i = 0; i < 45 && final.state !== "replied"; i++) {
			await Bun.sleep(100);
			final = ok(["agent", "status", request], "", sender);
		}
		assert.equal(final.reply, "Draft only: " + shared, JSON.stringify(final));
		assert.equal(ok(send, shared, sender).id, first.id);
		assert.notEqual(invoke(send, "edited", sender).exitCode, 0);
		assert.notEqual(invoke(["agent", "take", first.id], "", a).exitCode, 0);
		const read = JSON.parse(readFileSync(received, "utf8"));
		assert.equal(read.body, shared);
		assert(!read.notice.includes(shared));
		assert.equal(ok(["agent", "away"], "", receiver).available, false);
		assert.equal(ok(["agent", "targets"], "", sender).recipients.length, 0);
		console.log(
			"Real isolated hc PTY: registered named recipient, exact shared text, private notice, single draft, idempotence, wrong principal and away.",
		);
	} finally {
		const stopped = hc(["server", "stop"]);
		assert.equal(stopped.exitCode, 0, stopped.stderr.toString());
	}
	const { exerciseHandoffFaults } = await import(
		"./fixtures/external-bridge-faults"
	);
	await exerciseHandoffFaults(socket, bin, cwd, receiver, sender);
}
// A fresh agent can use the installed CLI without ever receiving or printing a token. No desktop
// app is running. The fixture only listens on a random loopback port and dies with this test.
if (process.env.SK_TEST_BRIDGE_JUDGE === "1") {
	const q = (s: string) => "'" + s.replaceAll("'", "'\\''") + "'";
	const credential = join(creds, "judge");
	ok(["grant-agent", "judge", credential, "notes:quiet"]);
	writeFileSync(
		join(cwd, "scriptkit"),
		`#!/bin/sh\nexec /usr/bin/env -i PATH=${q(process.env.PATH!)} HOME=${q(process.env.HOME!)} SK_PATH=${q(kit)} SCRIPT_KIT_AGENT_CREDENTIAL=${q(credential)} ${q(process.execPath)} ${q(bin)} "$@"\n`,
		{ mode: 0o700, flag: "wx" },
	);
	console.log("BRIDGE_JUDGE_CWD=" + cwd);
	const end = Date.now() + 12 * 60_000;
	while (!existsSync(join(cwd, "judge-finished")) && Date.now() < end)
		await Bun.sleep(500);
	assert(existsSync(join(cwd, "judge-finished")), "Judge timed out");
	const notes = readdirSync(join(kit, "brain/notes")).map((f) =>
		readFileSync(join(kit, "brain/notes", f), "utf8"),
	);
	assert.equal(
		notes.filter((n) => n.includes("Keep Friday offline.")).length,
		1,
	);
	console.log(
		"Blind operator note verified independently from its canonical file.",
	);
}
