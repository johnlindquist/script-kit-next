// A synthetic full-screen-agent substitute in an isolated Herdr PTY, not a model or a host window.
import { createInterface } from "node:readline";
import { writeFileSync } from "node:fs";
const [bin, ready, received] = process.argv.slice(2);
function agent(args: string[], stdin = "") {
	const r = Bun.spawnSync(["bun", bin, "agent", ...args], {
		stdin: Buffer.from(stdin),
		stdout: "pipe",
		stderr: "pipe",
		timeout: 10000,
	});
	const v = JSON.parse(r.stdout.toString());
	if (!v.success) throw Error(v.error);
	return v.data;
}
process.stdout.write("\x1b]7501;state=idle\x1b\\");
try {
	const reg = agent(["available", "workshop"]);
	writeFileSync(ready, JSON.stringify(reg));
} catch (e) {
	writeFileSync(ready, JSON.stringify({ error: String(e) }));
	process.exit(3);
}
for await (const line of createInterface({ input: process.stdin })) {
	const id = line.match(/Script Kit handoff ([a-f0-9-]{36})/)?.[1];
	if (!id) continue;
	process.stdout.write("\x1b]7501;state=working\x1b\\");
	const { body } = agent(["take", id]);
	writeFileSync(received, JSON.stringify({ id, notice: line, body }));
	agent(["reply", id], "Draft only: " + body);
	process.stdout.write("\x1b]7501;state=idle\x1b\\");
}
