import { mkdir, appendFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Defer Task",
  description: "Capture a deferred task from ;defer menu syntax into a local JSONL queue",
  icon: "calendar-clock",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["defer"],
      accepts: ["tags", "date", "relativeDate", "priority"],
      required: ["body", "date"],
      label: "Defer task",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");
const payload = JSON.parse(await readFile(payloadPath, "utf8"));
const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");

const deferDate = payload.dates?.[0] ?? null;
if (!deferDate?.iso) {
  throw new Error(
    "Defer capture requires a date; try `;defer until next week Refactor settings panel`",
  );
}

const dir = join(skPath, "menu-syntax");
await mkdir(dir, { recursive: true });
await appendFile(
  join(dir, "deferred.jsonl"),
  JSON.stringify({
    body: payload.body,
    tags: payload.tags,
    priority: payload.priority,
    deferUntil: deferDate.iso,
    date: deferDate,
    unresolvedDates: payload.unresolvedDates,
    raw: payload.raw,
    createdAt: new Date().toISOString(),
  }) + "\n"
);
