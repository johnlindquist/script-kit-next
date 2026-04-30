import "@scriptkit/sdk";
import { readFile } from "node:fs/promises";
import { homedir } from "node:os";
import { join } from "node:path";

type CalendarDate = {
  iso: string;
  endIso?: string;
};

type CalendarPayload = {
  body?: string;
  raw?: string;
  tags?: string[];
  dates?: CalendarDate[];
  duration?: string;
  durationResolved?: { seconds?: number };
  recurrence?: { rrule?: string };
  unresolvedDates?: { role?: string; source?: string }[];
  kv?: Record<string, string | undefined>;
};

type SavedToken = {
  token?: { access_token?: string };
};

type GoogleCalendarEvent = {
  id: string;
  htmlLink?: string;
  summary?: string;
};

export const metadata = {
  name: "Add to Google Calendar",
  description:
    "Create a Google Calendar event from ;gcal capture using a saved Google device-flow token. Example: `;gcal Design review tomorrow 2pm for 45m location=\"Zoom\"`",
  icon: "calendar-plus",
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["gcal"],
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
      label: "Add event to Google Calendar",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
      defaultHandler: true,
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
  const payload = JSON.parse(await readFile(payloadPath, "utf8")) as CalendarPayload;

  if (payload.unresolvedDates?.length) {
    throw new Error(
      `Unresolved date fragment: ${payload.unresolvedDates
        .map((date) => `${date.role || "date"}:${date.source || ""}`)
        .join(", ")}`,
    );
  }

  const start = payload.dates?.[0];
  if (!start) {
    throw new Error(
      "Google Calendar capture requires a date; try `tomorrow at 3pm` or `2026-05-01`",
    );
  }

  const startDate = parseDate(start.iso, "start date");
  const endDate = resolveEndDate(payload, startDate, start.endIso);
  const calendarId = payload.kv?.calendarId || payload.kv?.calendar || "primary";
  const token = await loadToken();
  const event = await insertEvent(token, calendarId, buildEvent(payload, startDate, endDate));
  const eventUrl = event.htmlLink || `https://calendar.google.com/calendar/u/0/r`;

  const action = await arg(`Created ${event.summary || payload.body || "event"}`, [
    { name: "Open Event", description: eventUrl, value: "event" },
    { name: "Open Calendar", description: calendarId, value: "calendar" },
    { name: "Done", description: `Google Calendar event id: ${event.id}`, value: "done" },
  ]);

  if (action === "event") {
    await open(eventUrl);
  } else if (action === "calendar") {
    await open("https://calendar.google.com/calendar/u/0/r");
  }
}

async function loadToken(): Promise<string> {
  const root = process.env.SK_PATH || join(homedir(), ".scriptkit");
  const path = join(root, "secrets", "device-auth", "google-calendar.json");
  let saved: SavedToken;
  try {
    saved = JSON.parse(await readFile(path, "utf8")) as SavedToken;
  } catch (error) {
    throw new Error(
      `No saved Google Calendar token at ${path}. Run the Google Calendar Device Login example first.`,
      { cause: error },
    );
  }

  const token = saved.token?.access_token;
  if (!token) throw new Error(`Saved Google Calendar token at ${path} is missing access_token`);
  return token;
}

async function insertEvent(
  token: string,
  calendarId: string,
  event: Record<string, unknown>,
): Promise<GoogleCalendarEvent> {
  const response = await fetch(
    `https://www.googleapis.com/calendar/v3/calendars/${encodeURIComponent(calendarId)}/events`,
    {
      method: "POST",
      headers: {
        Authorization: `Bearer ${token}`,
        "Content-Type": "application/json",
      },
      body: JSON.stringify(event),
    },
  );
  const body = await response.json();
  if (!response.ok) {
    throw new Error(`Google Calendar insert failed (${response.status}): ${JSON.stringify(body)}`);
  }
  return body as GoogleCalendarEvent;
}

function buildEvent(
  payload: CalendarPayload,
  startDate: Date,
  endDate: Date,
): Record<string, unknown> {
  const timezone = payload.kv?.timezone || payload.kv?.tz;
  const event: Record<string, unknown> = {
    summary: (payload.body || "Untitled event").trim(),
    description: buildDetails(payload),
    start: buildEventDate(startDate, timezone),
    end: buildEventDate(endDate, timezone),
  };

  const location = payload.kv?.location || payload.kv?.where;
  if (location) event.location = location;

  const guests = payload.kv?.guests || payload.kv?.add;
  if (guests) {
    event.attendees = guests
      .split(",")
      .map((email) => email.trim())
      .filter(Boolean)
      .map((email) => ({ email }));
  }

  if (payload.recurrence?.rrule) event.recurrence = [`RRULE:${payload.recurrence.rrule}`];
  return event;
}

function buildEventDate(date: Date, timeZone?: string): Record<string, string> {
  const eventDate: Record<string, string> = { dateTime: date.toISOString() };
  if (timeZone) eventDate.timeZone = timeZone;
  return eventDate;
}

function buildDetails(payload: CalendarPayload): string {
  const raw = payload.raw || "";
  const tags = (payload.tags || []).map((tag) => `#${tag}`).join(" ");
  const explicitDetails = payload.kv?.details || payload.kv?.description || "";
  return [explicitDetails, raw, tags].filter(Boolean).join("\n");
}

function resolveEndDate(
  payload: CalendarPayload,
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
