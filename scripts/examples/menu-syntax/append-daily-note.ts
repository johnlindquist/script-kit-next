import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Append Daily Note",
  description: "Append captured note text from ;note / note: menu syntax to today's markdown note",
  icon: "notebook-pen",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["note"],
      accepts: ["tags", "date", "kv"],
      label: "Append daily note",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");
const payload = JSON.parse(await readFile(payloadPath, "utf8"));
const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");

const dir = join(skPath, "notes");
await mkdir(dir, { recursive: true });

const now = new Date();
const day = now.toISOString().slice(0, 10);
const time = now.toTimeString().slice(0, 5);
const tags = (payload.tags || []).map((tag: string) => `#${tag}`).join(" ");

await appendFile(
  join(dir, `${day}.md`),
  `\n- ${time} ${payload.body}${tags ? " " + tags : ""}\n`
);
