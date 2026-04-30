/**
 * Compile-only fixture for `kit-init/sdk/menu-syntax.ts` helpers.
 *
 * Run via `bun tsc --noEmit kit-init/sdk/menu-syntax.test.ts` — clean exit
 * is the receipt. Each block exercises a helper so a regression in inferred
 * types (e.g. dropping the literal `"capture.v1"` narrowing) breaks here.
 */

import {
  captureTarget,
  commandSchema,
  menuSyntax,
  skillSpec,
} from "./menu-syntax";
import type {
  CaptureHandlerSpec,
  CommandHandlerSpec,
  MenuSyntaxMetadata,
  SkillHandlerSpec,
} from "../types/menu-syntax";

// 1. captureTarget infers `family: "capture.v1"` and target slug literal.
const cal = captureTarget("cal", {
  accepts: ["tags", "date", "duration", "kv"],
  label: "Create calendar event",
  payloadSchema: "kit://schema/menu-syntax/payload-v1",
  defaultHandler: true,
});
const _calTyped: CaptureHandlerSpec = cal;

// 2. captureTarget with alsoTargets fans out to multiple slugs.
const noteAndJournal = captureTarget("note", {
  accepts: ["tags"],
  alsoTargets: ["journal"],
});
const _noteTyped: CaptureHandlerSpec = noteAndJournal;

// 3. commandSchema preserves the head literal and accepts readonly args.
const deploy = commandSchema("deploy", {
  label: "Deploy a service",
  args: [
    {
      name: "env",
      required: true,
      values: ["prod", "staging", "dev"] as const,
    },
  ],
  flags: [{ name: "--dry-run", alias: "-n" }],
  usage: ">deploy -- <env> [--dry-run]",
});
const _deployTyped: CommandHandlerSpec = deploy;

// 4. skillSpec carries contextRequirements through.
const review = skillSpec("review", {
  label: "Review current file",
  contextRequirements: ["selection.file", "frontmost.app"],
});
const _reviewTyped: SkillHandlerSpec = review;

// 5. menuSyntax(...) variadic returns the metadata array. Authors splat the
//    helper outputs in the order they want them registered.
export const metadata = {
  name: "Mixed Power Syntax Author",
  menuSyntax: menuSyntax(cal, noteAndJournal, deploy, review),
};
const _metadataTyped: MenuSyntaxMetadata = metadata.menuSyntax;

// 6. menuSyntax accepts inline literals too — no helper required.
const _direct: MenuSyntaxMetadata = menuSyntax(
  { family: "capture.v1", targets: ["todo"], accepts: ["tags"] },
  { family: "command.v1", head: "log" },
);
