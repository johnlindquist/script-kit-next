import { mkdir, writeFile, readFile } from "node:fs/promises";
import { join } from "node:path";

export const metadata = {
  name: "Create Calendar Event",
  description: "Create an .ics file from ;cal / cal: menu syntax",
  icon: "calendar-plus",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["cal"],
      accepts: ["tags", "date", "duration", "kv"],
      label: "Create calendar event",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");
const payload = JSON.parse(await readFile(payloadPath, "utf8"));
const skPath = process.env.SK_PATH || join(process.env.HOME || ".", ".scriptkit");

const dir = join(skPath, "menu-syntax", "calendar");
await mkdir(dir, { recursive: true });

const start = payload.dates?.find((d: any) => d.role === "start") ?? payload.dates?.[0];
if (!start) throw new Error("Calendar capture requires a date");

const startDate = new Date(start.iso);
const durationMinutes = Number(String(payload.duration || "30m").match(/\d+/)?.[0] || 30);
const endDate = new Date(startDate.getTime() + durationMinutes * 60_000);

const fmt = (d: Date) => d.toISOString().replace(/[-:]/g, "").replace(/\.\d{3}Z$/, "Z");
const safeName = payload.body.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "event";

const ics = [
  "BEGIN:VCALENDAR",
  "VERSION:2.0",
  "PRODID:-//Script Kit//Menu Syntax//EN",
  "BEGIN:VEVENT",
  `UID:${crypto.randomUUID()}@scriptkit`,
  `SUMMARY:${payload.body || "Untitled event"}`,
  `DTSTART:${fmt(startDate)}`,
  `DTEND:${fmt(endDate)}`,
  `DESCRIPTION:${payload.raw}`,
  "END:VEVENT",
  "END:VCALENDAR",
  "",
].join("\n");

await writeFile(join(dir, `${Date.now()}-${safeName}.ics`), ics);
