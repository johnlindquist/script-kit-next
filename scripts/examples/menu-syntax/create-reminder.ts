import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Create Reminder",
  description: "Capture a reminder from ;reminder menu syntax into a local JSONL inbox",
  icon: "bell",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["reminder"],
      accepts: [
        "tags",
        "date",
        "relativeDate",
        "duration",
        "recurrence",
        "daily",
        "multiWeekday",
      ],
      required: ["body"],
      label: "Create reminder",
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
  join(dir, "reminders.jsonl"),
  JSON.stringify({
    body: payload.body,
    tags: payload.tags,
    dates: payload.dates,
    durationResolved: payload.durationResolved,
    recurrence: payload.recurrence,
    raw: payload.raw,
    createdAt: new Date().toISOString(),
  }) + "\n"
);
