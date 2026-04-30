import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Snooze Task",
  description: "Capture a snoozed task from ;snooze menu syntax into a local JSONL queue",
  icon: "clock",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["snooze"],
      accepts: ["tags", "date", "relativeDate", "duration"],
      required: ["body", "date"],
      label: "Snooze task",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");
const payload = JSON.parse(await readFile(payloadPath, "utf8"));
const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");

const wakeDate = payload.dates?.[0] ?? null;
let wakeAt = wakeDate?.iso;
if (!wakeAt && payload.durationResolved?.seconds) {
  wakeAt = new Date(
    Date.now() + payload.durationResolved.seconds * 1000,
  ).toISOString();
}
if (!wakeAt) {
  throw new Error(
    "Snooze capture requires a wake date; try `;snooze in 30 minutes Review PR`",
  );
}

const dir = join(skPath, "menu-syntax");
await mkdir(dir, { recursive: true });
await appendFile(
  join(dir, "snoozed.jsonl"),
  JSON.stringify({
    body: payload.body,
    tags: payload.tags,
    wakeAt,
    date: wakeDate,
    durationResolved: payload.durationResolved,
    unresolvedDates: payload.unresolvedDates,
    raw: payload.raw,
    createdAt: new Date().toISOString(),
  }) + "\n"
);
