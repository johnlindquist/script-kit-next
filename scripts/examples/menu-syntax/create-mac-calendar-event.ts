import { execFile } from "node:child_process";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";
import { promisify } from "node:util";
import type { MenuSyntaxCapturePayload } from "../../../kit-init/types/menu-syntax";

const run = promisify(execFile);

export const metadata = {
  name: "Create macOS Calendar Event",
  description:
    "Add an event to macOS Calendar via ;mcal capture. Examples: " +
    "`;mcal Lunch with Ryan tomorrow at 12pm til 1pm`; " +
    "`;mcal Lunch with Ryan tom 12pm for 30mins`; " +
    "`;mcal Lunch w/ Ryan every mon from 1 til 2`",
  icon: "calendar-check",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["mcal"],
      accepts: [
        "tags",
        "date",
        "dateRange",
        "duration",
        "recurrence",
        "daily",
        "multiWeekday",
        "monthly",
        "yearly",
        "kv",
      ],
      required: ["body", "date"],
      label: "Add event to macOS Calendar",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
      kvEnums: {
        calendar: ["Home", "Work", "Personal", "Family"],
        alarm: ["0", "5", "15", "30", "60"],
      },
    },
  ],
};

const payloadPath = process.env.KIT_MENU_SYNTAX_PAYLOAD_PATH;
if (!payloadPath) throw new Error("KIT_MENU_SYNTAX_PAYLOAD_PATH is required");

void main(payloadPath).catch((error) => {
  console.error(error);
  process.exitCode = 1;
});

async function main(payloadPath: string): Promise<void> {
  const payload = JSON.parse(
    await readFile(payloadPath, "utf8"),
  ) as MenuSyntaxCapturePayload;

  if (payload.unresolvedDates?.length) {
    throw new Error(
      `Unresolved date fragment: ${payload.unresolvedDates
        .map((d) => `${d.role}:${d.source}`)
        .join(", ")}`,
    );
  }

  const start = payload.dates?.[0];
  if (!start) {
    throw new Error(
      "Calendar capture requires a date; try `tomorrow at 3pm` or `2026-05-01`",
    );
  }

  const startDate = parseDate(start.iso, "start date");
  const endDate = resolveEndDate(payload, startDate, start.endIso);
  const summary = (payload.body || "Untitled event").trim();
  const calendarName = payload.kv?.calendar;
  const uid = stableUid(payload.raw, start.iso);
  const ics = buildIcs({
    uid,
    summary,
    description: buildDescription(payload, calendarName),
    calendarName,
    dtstamp: formatIcsUtc(new Date()),
    dtstart: formatIcsUtc(startDate),
    dtend: formatIcsUtc(endDate),
    rrule: payload.recurrence?.rrule,
    alarmMinutes: parseAlarmMinutes(payload.kv?.alarm),
  });

  validateIcs(ics);

  const scriptKitRoot = process.env.SK_PATH || join(homedir(), ".scriptkit");
  const calendarDir = join(scriptKitRoot, "cache", "calendar");
  const icsPath = join(calendarDir, `${uid}.ics`);
  await mkdir(calendarDir, { recursive: true });
  await writeFile(icsPath, ics, "utf8");

  try {
    await run("open", ["-a", "Calendar.app", icsPath]);
  } catch (error) {
    const stderr =
      error && typeof error === "object" && "stderr" in error
        ? String(error.stderr)
        : "";
    throw Object.assign(new Error(`Failed to open Calendar.app for ${icsPath}`), {
      uid,
      path: icsPath,
      stderr,
    });
  }

  console.log(`Wrote and opened ${icsPath}`);
}

function resolveEndDate(
  payload: MenuSyntaxCapturePayload,
  startDate: Date,
  endIso?: string,
): Date {
  if (endIso) return parseDate(endIso, "end date");

  if (payload.durationResolved?.seconds) {
    return new Date(startDate.getTime() + payload.durationResolved.seconds * 1000);
  }

  const legacyMinutes = Number(String(payload.duration || "").match(/\d+/)?.[0]);
  if (Number.isFinite(legacyMinutes) && legacyMinutes > 0) {
    return new Date(startDate.getTime() + legacyMinutes * 60_000);
  }

  return new Date(startDate.getTime() + 30 * 60_000);
}

function parseDate(iso: string, label: string): Date {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) throw new Error(`Invalid ${label}: ${iso}`);
  return date;
}

function buildDescription(
  payload: MenuSyntaxCapturePayload,
  calendarName?: string,
): string {
  const tagLine = (payload.tags || []).map((tag) => `#${tag}`).join(" ");
  const calendarLine = calendarName ? `calendar: ${calendarName}` : "";
  return [payload.raw, tagLine, calendarLine].filter(Boolean).join("\n");
}

function buildIcs(event: {
  uid: string;
  summary: string;
  description: string;
  calendarName?: string;
  dtstamp: string;
  dtstart: string;
  dtend: string;
  rrule?: string;
  alarmMinutes?: number;
}): string {
  const lines = [
    "BEGIN:VCALENDAR",
    "VERSION:2.0",
    "PRODID:-//Script Kit//Menu Syntax//EN",
    "BEGIN:VEVENT",
    `UID:${event.uid}`,
    `DTSTAMP:${event.dtstamp}`,
    `DTSTART:${event.dtstart}`,
    `DTEND:${event.dtend}`,
    `SUMMARY:${escapeIcalText(event.summary)}`,
  ];

  if (event.rrule) lines.push(`RRULE:${event.rrule}`);
  if (event.calendarName) {
    lines.push(`X-APPLE-CALENDAR-NAME:${escapeIcalText(event.calendarName)}`);
  }
  lines.push(`DESCRIPTION:${escapeIcalText(event.description)}`);

  if (event.alarmMinutes !== undefined) {
    lines.push(
      "BEGIN:VALARM",
      "ACTION:DISPLAY",
      "DESCRIPTION:Reminder",
      `TRIGGER:-PT${event.alarmMinutes}M`,
      "END:VALARM",
    );
  }

  lines.push("END:VEVENT", "END:VCALENDAR");
  return `${lines.map(foldIcalLine).join("\r\n")}\r\n`;
}

function validateIcs(input: string): void {
  if (input.replace(/\r\n/g, "").includes("\n")) {
    throw new Error("Generated ICS contains non-CRLF line endings");
  }
  for (const line of input.split("\r\n")) {
    if (Buffer.byteLength(line, "utf8") > 75) {
      throw new Error(`Generated ICS line exceeds 75 octets: ${line}`);
    }
  }
  for (const required of [
    "BEGIN:VCALENDAR",
    "VERSION:2.0",
    "BEGIN:VEVENT",
    "UID:",
    "DTSTAMP:",
    "DTSTART:",
    "DTEND:",
    "SUMMARY:",
    "END:VEVENT",
    "END:VCALENDAR",
  ]) {
    if (!input.includes(required)) {
      throw new Error(`Generated ICS missing ${required}`);
    }
  }
}

function stableUid(raw: string, startIso: string): string {
  const hash = createHash("sha256")
    .update(raw)
    .update("|")
    .update(startIso)
    .digest("hex")
    .slice(0, 32);
  return `menu-syntax-${hash}@scriptkit`;
}

function parseAlarmMinutes(value?: string): number | undefined {
  if (value === undefined) return undefined;
  const minutes = Number(value);
  return Number.isFinite(minutes) && minutes >= 0 ? minutes : undefined;
}

function formatIcsUtc(date: Date): string {
  return date
    .toISOString()
    .replace(/[-:]/g, "")
    .replace(/\.\d{3}Z$/, "Z");
}

function escapeIcalText(value: string): string {
  return value
    .replace(/\\/g, "\\\\")
    .replace(/\n/g, "\\n")
    .replace(/\r/g, "")
    .replace(/,/g, "\\,")
    .replace(/;/g, "\\;");
}

function foldIcalLine(line: string): string {
  const chunks: string[] = [];
  let current = "";
  let limit = 75;

  for (const char of line) {
    const nextLength = Buffer.byteLength(current + char, "utf8");
    if (nextLength > limit) {
      chunks.push(current);
      current = ` ${char}`;
      limit = 75;
    } else {
      current += char;
    }
  }

  chunks.push(current);
  return chunks.join("\r\n");
}
