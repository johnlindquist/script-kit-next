// Hidden, owned GPUI event paths. Synthetic subject/recipient only; no native input or live agents.
import assert from "node:assert/strict";
import { writeFileSync, rmSync } from "node:fs";
import { resolve, join } from "node:path";
import { OwnedEvaluationClient } from "../devtools/lib/owned-evaluation";
import { readArtifactReference } from "../devtools/design";
import {
	claimOutput,
	validateOutputTarget,
	beginManagedTask,
	finalizeManagedTask,
	emptyOwnedCleanup,
} from "./artifact-lifecycle";
import { DriverLifecycleError } from "../devtools/driver";
const [refPath, dest, mode = "reply"] = process.argv.slice(2);
assert(["cancel", "reply"].includes(mode));
assert(refPath && dest);
const out = resolve(dest),
	fixture = "main.subject-tabs",
	ref = readArtifactReference(resolve(refPath));
const claim = claimOutput(
	validateOutputTarget({
		repoRoot: process.cwd(),
		candidate: out,
		kind: "directory",
		probeId: "external-agent-handoff",
	}),
);
const task = beginManagedTask(claim, "evidence-run", [ref]);
const c = await OwnedEvaluationClient.launch(
	process.cwd(),
	ref,
	claim,
	[fixture],
	undefined,
	{ nativeGlass: "disabled" },
).catch((e) => {
	finalizeManagedTask(
		task,
		e instanceof DriverLifecycleError ? e.cleanup : emptyOwnedCleanup(),
	);
	throw e;
});
const results: any = {};
const session = c.driver.sessionDir;
try {
	const t = await c.mount(fixture);
	await c.frame(t);
	const state = () => c.inspect(t);
	const key = (k: string, mods: string[] = []) =>
		c.act(t, { type: "key", key: k, modifiers: mods });
	const control = async (operation: "handoffRecipient" | "handoffAnswer") =>
		c.design({
			operation: "fixtureControl",
			target: t,
			expected: (await state()).targetIdentity,
			control: { family: "oneWindow", operation },
		});
	const snapshot = async (name: string) => {
		const image = [
			"staged",
			"recipients",
			"hold",
			"arrived",
			"editable",
			"edited",
			"after-escape",
			"reopened",
		].includes(name);
		const f = await c.captureFrame(t, image);
		if (image)
			writeFileSync(
				join(out, name + ".png"),
				Buffer.from(f.snapshot.capture.pngBase64, "base64"),
			);
		results[name] = {
			line: f.state.oneWindow?.line?.title,
			enter: f.state.oneWindow?.enter?.label,
			view: f.state.promptType,
			input: f.state.promptObservation?.input ?? f.state.inputValue,
			copy: f.state.copySink,
		};
		return f.state;
	};
	// Action text is intentionally redacted in the transport. For the known synthetic
	// menus select the observed row by its semantic identity and check the next view.
	const choose = async (label?: string, index = 0, count?: number) => {
		await Bun.sleep(430);
		const rows = (await c.query(t, "elements")).elements.filter(
			(x: any) => x.role === "row" && x.selectable,
		);
		if (count !== undefined)
			assert.equal(rows.length, count, "Unexpected menu: " + label);
		const match =
			rows.find((x: any) => label && x.text === label) ?? rows[index];
		assert(
			match && (match.text === undefined || !label || match.text === label),
			"Expected " + label + ", got " + match?.text,
		);
		await c.act(t, {
			type: "select",
			semanticId: match.semanticId,
			submit: true,
		});
	};
	await control("handoffRecipient");
	await key("space");
	await snapshot("staged");
	await choose("Share selection with a named agent…", 3, 4);
	await snapshot("recipients");
	await choose("workshop · t1", 0, 1);
	assert.equal((await snapshot("hold")).oneWindow.line.kind, "holding");
	if (mode === "cancel") {
		await key("z", ["cmd"]);
		assert.notEqual((await snapshot("stopped")).oneWindow.line.kind, "holding");
		await Bun.sleep(8450);
		await c.act(t, { type: "setInput", text: "Agent Handoffs" });
		await choose("Agent Handoffs");
		assert.match(
			(await snapshot("empty-inbox")).oneWindow.line.title,
			/No pending handoffs or replies/,
		);
	} else {
		await Bun.sleep(8450);
		await c.frame(t);
		const waited = await snapshot("notified");
		assert.match(
			waited.oneWindow.line.title,
			/Notified workshop · waiting for a reply/,
		);
		await c.act(t, { type: "setInput", text: "keep typing here" });
		await control("handoffAnswer");
		for (
			let i = 0;
			i < 40 &&
			!(await state()).oneWindow.line.title.includes("workshop replied");
			i++
		)
			await Bun.sleep(40);
		const untouched = await snapshot("arrived");
		assert.match(untouched.oneWindow.line.title, /workshop replied/);
		assert.equal(untouched.inputValue, "keep typing here");
		assert.notEqual(untouched.promptType, "editor");
		await c.act(t, { type: "setInput", text: "Agent Handoffs" });
		await choose("Agent Handoffs");
		await snapshot("inbox");
		await choose(undefined, 0, 1); // one reply, with a fresh non-predictable ID in its row label
		const draft = await snapshot("editable");
		assert.equal(draft.promptType, "editor");
		assert.equal(
			draft.promptObservation.input,
			"Draft: TypeError: Cannot read properties of undefined",
		);
		assert.equal(draft.copySink, null);
		await c.act(t, { type: "key", key: "x", text: "x" });
		const edited = (await snapshot("edited")).promptObservation.input;
		assert.notEqual(
			edited,
			draft.promptObservation.input,
			"typing must change the reply",
		);
		assert.equal(edited.length, draft.promptObservation.input.length + 1);
		await key("escape");
		const back = await snapshot("after-escape");
		assert.notEqual(
			back.promptType,
			"editor",
			"Escape should leave local draft review",
		);
		assert.equal(back.oneWindow.line.title, "Agent Handoffs");
		await choose(undefined, 0, 1);
		assert.equal((await snapshot("reopened")).promptObservation.input, edited);
		await key("s", ["cmd"]);
		for (let i = 0; i < 30 && (await state()).promptType === "editor"; i++)
			await Bun.sleep(40);
		assert.equal(
			(await snapshot("saved-back")).oneWindow.line.title,
			"Agent Handoffs",
		);
	}
	for (const step of Object.values(results) as { copy: unknown }[])
		assert.equal(step.copy, null);
} finally {
	const close = finalizeManagedTask(task, await c.close()).cleanup;
	results.closed = close.closed;
	if (close.closed && session.startsWith(out + "/"))
		rmSync(session, { recursive: true, force: true });
	writeFileSync(join(out, "outcomes.json"), JSON.stringify(results, null, 2));
	assert(close.closed);
}
console.log(
	mode === "cancel"
		? "Verified cancellation: inbox stayed empty past the send deadline."
		: "Verified named handoff, noninterrupting arrival, real editor typing, Esc/Save return and reopening the exact edit.",
);
