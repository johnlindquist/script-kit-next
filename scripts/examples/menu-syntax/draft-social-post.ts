import { mkdir, writeFile, readFile } from "node:fs/promises";
import { join } from "node:path";
import { spawnSync } from "node:child_process";

export const metadata = {
  name: "Draft Social Post",
  description: "Create a markdown social draft from ;social / social: menu syntax",
  icon: "send",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["social"],
      accepts: ["tags", "date", "url", "kv"],
      label: "Draft social post",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");
const payload = JSON.parse(await readFile(payloadPath, "utf8"));
const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");

const dir = join(skPath, "menu-syntax", "social-drafts");
await mkdir(dir, { recursive: true });

const hashtags = (payload.tags || []).map((tag: string) => `#${tag}`).join(" ");
const body = `${payload.body}${hashtags ? "\n\n" + hashtags : ""}${payload.url ? "\n\n" + payload.url : ""}\n`;

await writeFile(join(dir, `${Date.now()}.md`), body);

if (process.platform === "darwin") {
  spawnSync("pbcopy", { input: body });
}
