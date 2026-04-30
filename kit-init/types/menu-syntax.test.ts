/**
 * Compile-only fixture for `kit-init/types/menu-syntax.d.ts`.
 *
 * Run via `bun tsc --noEmit kit-init/types/menu-syntax.test.ts` (or `pnpm tsc
 * --noEmit ...`) — there is no test harness; the receipt is a clean tsc exit.
 *
 * Each fixture exercises a representative author surface so a typo in the
 * `.d.ts` (e.g. dropping `priority` from `CaptureAccept`) breaks this file.
 */

import type {
  MenuSyntaxMetadata,
  CaptureHandlerSpec,
  CommandHandlerSpec,
  SkillHandlerSpec,
  MenuSyntaxCapturePayload,
} from "./menu-syntax";

// 1. Built-in capture target with full accepts list — mirrors
//    scripts/examples/menu-syntax/create-calendar-event.ts.
const calCapture: CaptureHandlerSpec = {
  family: "capture.v1",
  targets: ["cal"],
  accepts: ["tags", "date", "duration", "kv"],
  label: "Create calendar event",
  payloadSchema: "kit://schema/menu-syntax/payload-v1",
  defaultHandler: true,
};

const mcalCapture: CaptureHandlerSpec = {
  family: "capture.v1",
  targets: ["mcal"],
  accepts: [
    "tags",
    "date",
    "dateRange",
    "duration",
    "recurrence",
    "relativeDate",
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
    calendar: ["Home", "Work", "Personal", "Family"] as const,
    alarm: ["0", "5", "15", "30", "60"] as const,
  },
};

// 2. Wildcard capture — handler willing to accept any ;slug.
const fallbackCapture: CaptureHandlerSpec = {
  family: "capture.v1",
  targets: ["*"],
  accepts: ["tags"],
};

// 3. Custom (non-built-in) target slug — author declares "expense".
const expenseCapture: CaptureHandlerSpec = {
  family: "capture.v1",
  targets: ["expense"],
  accepts: ["tags", "date", "kv"],
};

// 4. Command schema with positional arg + flags.
const deployCommand: CommandHandlerSpec = {
  family: "command.v1",
  head: "deploy",
  label: "Deploy a service",
  args: [
    {
      name: "env",
      description: "Target environment",
      required: true,
      values: ["prod", "staging", "dev"] as const,
    },
  ],
  flags: [
    {
      name: "--dry-run",
      alias: "-n",
      description: "Print the plan without applying",
    },
  ],
  usage: ">deploy -- <env> [--dry-run]",
};

// 5. Skill spec with context requirements.
const reviewSkill: SkillHandlerSpec = {
  family: "skill.v1",
  slug: "review",
  label: "Review current file",
  contextRequirements: ["selection.file", "frontmost.app"],
};

// 6. The full metadata array as an author would export it.
export const menuSyntax: MenuSyntaxMetadata = [
  calCapture,
  mcalCapture,
  fallbackCapture,
  expenseCapture,
  deployCommand,
  reviewSkill,
];

// 7. Runtime payload typing — mirrors what a handler sees after parsing
//    KIT_MENU_SYNTAX_PAYLOAD_PATH.
function _useCapturePayload(payload: MenuSyntaxCapturePayload): string {
  const title = payload.kv?.title ?? payload.body;
  const due = payload.dates?.find((date) => date.role === "due")?.iso;
  return due ? `${title} (due ${due})` : title;
}

function _useCalendarPayload(payload: MenuSyntaxCapturePayload): string {
  const start = payload.dates?.[0];
  const end =
    start?.endIso ??
    (payload.durationResolved
      ? `${payload.durationResolved.minutes}m`
      : payload.duration);
  const recurrence = payload.recurrence?.rrule;
  return [payload.body, start?.iso, end, recurrence].filter(Boolean).join(" ");
}

function _useRecurrenceFrequency(payload: MenuSyntaxCapturePayload): string | undefined {
  const frequency = payload.recurrence?.frequency;
  if (frequency === "daily" || frequency === "monthly" || frequency === "yearly") {
    return frequency;
  }
  return frequency === "weekly" ? frequency : undefined;
}

// 8. Negative-shape probes — uncomment any block to confirm tsc rejects it.
//
// const badAccept: CaptureHandlerSpec = {
//   family: "capture.v1",
//   targets: ["todo"],
//   accepts: ["nonsense"], // → not assignable to CaptureAccept
// };
//
// const badFamily: CaptureHandlerSpec = {
//   family: "capture.v2", // → wrong literal
//   targets: ["todo"],
// };
