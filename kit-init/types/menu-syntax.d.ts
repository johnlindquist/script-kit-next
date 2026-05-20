/**
 * Script Kit — Power Syntax (`menuSyntax`) author types.
 *
 * Mirrors the Rust `MenuSyntaxHandlerSpec` shape (src/menu_syntax/payload.rs)
 * and the capture / command / skill SDK surfaces script authors declare in
 * `metadata.menuSyntax`. Every field that the Rust parser tolerates is typed
 * here so authors get IDE autocompletion AND a compile-time error if they
 * mistype a target slug or accept token.
 *
 * The runtime contract: any object an author exports under
 * `metadata.menuSyntax` is round-tripped through serde_json, so the TS shapes
 * here MUST stay in sync with payload.rs. Adding a new variant means updating
 * both sides + a unit test in `src/menu_syntax/metadata.rs`.
 */

/** Built-in capture targets recognized by the launcher's main hint surface. */
export type BuiltInCaptureTarget =
  | "todo"
  | "cal"
  | "note"
  | "social"
  | "link"
  | "snippet";

/** Wildcard target — handler accepts any ;slug it sees. */
export type WildcardTarget = "*";

/** Capture target slug. Authors may declare custom slugs as plain strings. */
export type CaptureTarget = BuiltInCaptureTarget | WildcardTarget | (string & {});

/**
 * Capture payload tokens this handler is willing to consume. The launcher's
 * required-field schemas (e.g. `;cal` requires a date) are independent — these
 * `accepts` entries describe what the handler *understands*, not what the
 * launcher *requires*.
 */
export type CaptureAccept =
  | "tags"
  | "date"
  | "dateRange"
  | "duration"
  | "recurrence"
  | "relativeDate"
  | "daily"
  | "multiWeekday"
  | "monthly"
  | "yearly"
  | "url"
  | "priority"
  | "kv";

/** Schema URL for the capture v1 payload contract. */
export type CapturePayloadSchema = "kit://schema/menu-syntax/payload-v1" | (string & {});

/**
 * Capture handler — `;todo Renew passport p1 due:friday` style entries.
 * `family` is fixed to `"capture.v1"`; future families will be added as
 * additional discriminated variants.
 */
export interface CaptureHandlerSpec {
  family: "capture.v1";
  targets: CaptureTarget[];
  accepts?: CaptureAccept[];
  required?: readonly string[];
  optional?: readonly string[];
  forbidden?: readonly string[];
  kvEnums?: Record<string, readonly string[]>;
  label?: string;
  payloadSchema?: CapturePayloadSchema;
  defaultHandler?: boolean;
}

/** Argv flag declaration for `>command --flag value` style commands. */
export interface CommandFlagSpec {
  /** `--name` form. Lowercase recommended. */
  name: string;
  /** Short alias such as `-n`. */
  alias?: string;
  /** Human description rendered in the main hint card. */
  description?: string;
  /** When true the flag must be present for the command to validate. */
  required?: boolean;
  /** Provide a sample value to render in the hint card. */
  example?: string;
  /** Restrict accepted values to this enum. */
  values?: readonly string[];
}

/** Positional argument declaration for `>command <arg>` style commands. */
export interface CommandArgSpec {
  name: string;
  description?: string;
  required?: boolean;
  example?: string;
  values?: readonly string[];
}

/**
 * Command handler — `>deploy -- prod --dry-run` style invocations. Authors
 * declare argv shape here so the main hint card can render expected fields
 * before the script runs.
 */
export interface CommandHandlerSpec {
  family: "command.v1";
  /** The `>head` token. Lowercase, no leading `>`. */
  head: string;
  label?: string;
  description?: string;
  args?: CommandArgSpec[];
  flags?: CommandFlagSpec[];
  /** Free-form usage string shown verbatim in the hint card. */
  usage?: string;
  defaultHandler?: boolean;
}

/**
 * Context the skill needs from the host environment to be useful. The
 * launcher uses these to filter `:type:skill` results and to decide whether to
 * show a skill in the AI proposal panel for the current selection.
 */
export type SkillContextRequirement =
  | "selection.text"
  | "selection.file"
  | "selection.url"
  | "frontmost.app"
  | "clipboard.text"
  | "clipboard.image"
  | "workspace.path"
  | (string & {});

/**
 * Skill handler — `/skill` invocations. Skills are AI-routed actions that
 * appear in `:type:skill` filters but do NOT auto-register a `>command` head.
 */
export interface SkillHandlerSpec {
  family: "skill.v1";
  /** Slug used in `/skill` (no leading `/`). */
  slug: string;
  label?: string;
  description?: string;
  contextRequirements?: SkillContextRequirement[];
  /** Optional capture target the skill can consume as input. */
  acceptsCaptureTarget?: CaptureTarget;
}

/** Discriminated union of every handler family the launcher recognizes. */
export type MenuSyntaxHandlerSpec =
  | CaptureHandlerSpec
  | CommandHandlerSpec
  | SkillHandlerSpec;

/** Top-level shape an author exports as `metadata.menuSyntax`. */
export type MenuSyntaxMetadata = MenuSyntaxHandlerSpec[];

export type MenuSyntaxDateRole = "due" | "at" | "start" | "end" | "inferred";
export type MenuSyntaxDateGranularity = "date" | "minute" | "second";

export interface MenuSyntaxResolvedDate {
  role: MenuSyntaxDateRole;
  source: string;
  sourceSpan: readonly [number, number];
  iso: string;
  endIso?: string;
  relative: string;
  timezone: string;
  allDay: boolean;
  granularity: MenuSyntaxDateGranularity;
  confidence: number;
}

export interface MenuSyntaxUnresolvedDate {
  role: MenuSyntaxDateRole;
  source: string;
  sourceSpan: readonly [number, number];
}

export interface MenuSyntaxResolvedDuration {
  source: string;
  sourceSpan: readonly [number, number];
  seconds: number;
  minutes: number;
  iso8601: string;
}

export type MenuSyntaxRecurrenceFrequency = "weekly" | "daily" | "monthly" | "yearly";
export type MenuSyntaxRecurrenceWeekday =
  | "mon"
  | "tue"
  | "wed"
  | "thu"
  | "fri"
  | "sat"
  | "sun";

export interface MenuSyntaxResolvedRecurrence {
  source: string;
  sourceSpan: readonly [number, number];
  frequency: MenuSyntaxRecurrenceFrequency;
  weekdays: MenuSyntaxRecurrenceWeekday[];
  rrule: string;
  label: string;
}

export type MenuSyntaxObjectKind = "todo" | "note" | "link" | "snippet";

export interface MenuSyntaxObjectRef {
  role: "primary" | "related" | (string & {});
  kind: MenuSyntaxObjectKind;
  id: string;
  label: string;
  source: "inline-token" | "picker" | (string & {});
  deeplink?: string;
  query?: string;
  range?: readonly [number, number];
  token?: string;
  resolved: boolean;
}

export interface MenuSyntaxCapturePayload {
  schemaId: string;
  schemaVersion: number;
  raw: string;
  target: string;
  body: string;
  tags?: string[];
  url?: string;
  priority?: number;
  /** Legacy raw duration string. */
  duration?: string;
  /** Structured duration parsed from `for 30mins` / `for 1h`. */
  durationResolved?: MenuSyntaxResolvedDuration;
  kv?: Record<string, string>;
  dates?: MenuSyntaxResolvedDate[];
  unresolvedDates?: MenuSyntaxUnresolvedDate[];
  recurrence?: MenuSyntaxResolvedRecurrence;
  objectRefs?: MenuSyntaxObjectRef[];
  primaryObjectRef?: MenuSyntaxObjectRef;
}

/** Module augmentation for the `script-kit` package main metadata interface. */
declare module "script-kit" {
  interface ScriptMetadata {
    menuSyntax?: MenuSyntaxMetadata;
  }
}
