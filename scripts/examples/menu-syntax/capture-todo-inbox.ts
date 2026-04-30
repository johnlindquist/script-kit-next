import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Capture Todo Inbox",
  description: "Capture a todo from ;todo / todo: menu syntax into a local JSONL inbox",
  icon: "check-circle",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["todo"],
      accepts: [
        "tags",
        "date",
        "relativeDate",
        "recurrence",
        "daily",
        "multiWeekday",
        "priority",
        "url",
        "kv",
      ],
      label: "Add todo",
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
await appendFile(
  join(dir, "todos.jsonl"),
  JSON.stringify({
    body: payload.body,
    tags: payload.tags,
    priority: payload.priority,
    due: payload.dates?.[0]?.iso ?? null,
    raw: payload.raw,
    createdAt: new Date().toISOString(),
  }) + "\n"
);
