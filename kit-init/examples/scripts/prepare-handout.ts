import "@scriptkit/sdk";
import { mkdir, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";

// Agent effect: Write one new private Markdown file under ~/Documents/Kit Handouts. Never overwrite an existing file or open a window when called with input.
export const metadata = {
	name: "Prepare Handout",
	description: "Save a Markdown handout and return its path",
};
const io = defineSchema({
	input: {
		title: { type: "string", required: true, min: 1, max: 120 },
		body: { type: "string", required: true, min: 1, max: 48000 },
	},
	output: { path: { type: "string", required: true } },
});
const supplied = await io.input();
const title = supplied.title ?? (await arg("Handout title"));
const body = supplied.body ?? (await editor("", "markdown"));
const dir = join(homedir(), "Documents", "Kit Handouts");
await mkdir(dir, { recursive: true, mode: 0o700 });
const path = join(dir, `${randomUUID()}.md`);
await writeFile(path, `# ${title}\n\n${body}\n`, { flag: "wx", mode: 0o600 });
io.output({ path });
if (!supplied.title)
	await div("Saved the handout to Documents → Kit Handouts.");
