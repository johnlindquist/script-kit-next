import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Save Tagged Link",
  description: "Save a tagged bookmark from ;link / link: menu syntax",
  icon: "bookmark-plus",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["link"],
      accepts: ["tags", "url", "kv"],
      label: "Save tagged link",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");
const payload = JSON.parse(await readFile(payloadPath, "utf8"));
const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");

const dir = join(skPath, "menu-syntax");
await mkdir(dir, { recursive: true });

const title = payload.kv?.title || payload.body || payload.url || "Untitled link";
await appendFile(
  join(dir, "bookmarks.jsonl"),
  JSON.stringify({
    title,
    url: payload.url,
    tags: payload.tags,
    raw: payload.raw,
    createdAt: new Date().toISOString(),
  }) + "\n"
);
