/**
 * Script Kit — Power Syntax SDK helpers.
 *
 * Identity-typed builders that infer narrow types for `metadata.menuSyntax`
 * entries. The runtime shape is unchanged — these are pure type-level helpers
 * that pin literal types so authors get autocomplete on `accepts`, `family`,
 * and target slugs without manually annotating every entry.
 *
 * Pairs with `kit-init/types/menu-syntax.d.ts` (which defines the shapes
 * mirrored from `MenuSyntaxHandlerSpec` in src/menu_syntax/payload.rs).
 */

import type {
  CaptureAccept,
  CaptureHandlerSpec,
  CaptureTarget,
  CommandArgSpec,
  CommandFlagSpec,
  CommandHandlerSpec,
  MenuSyntaxHandlerSpec,
  MenuSyntaxMetadata,
  SkillContextRequirement,
  SkillHandlerSpec,
} from "../types/menu-syntax";

/** Options accepted by `captureTarget`. Narrower than the underlying spec
 *  because `family` and `targets[0]` are derived from the slug positionally. */
export interface CaptureTargetOptions {
  accepts?: readonly CaptureAccept[];
  label?: string;
  payloadSchema?: string;
  defaultHandler?: boolean;
  /** Additional aliases this handler also opts into. */
  alsoTargets?: readonly CaptureTarget[];
}

/**
 * Builds a `CaptureHandlerSpec` from a slug + options. Slug becomes
 * `targets[0]`; alsoTargets append after.
 */
export function captureTarget(
  slug: CaptureTarget,
  opts: CaptureTargetOptions = {},
): CaptureHandlerSpec {
  const targets = opts.alsoTargets
    ? [slug, ...opts.alsoTargets]
    : [slug];
  return {
    family: "capture.v1",
    targets,
    ...(opts.accepts ? { accepts: [...opts.accepts] } : {}),
    ...(opts.label ? { label: opts.label } : {}),
    ...(opts.payloadSchema ? { payloadSchema: opts.payloadSchema } : {}),
    ...(opts.defaultHandler ? { defaultHandler: opts.defaultHandler } : {}),
  };
}

/** Options accepted by `commandSchema`. */
export interface CommandSchemaOptions {
  label?: string;
  description?: string;
  args?: readonly CommandArgSpec[];
  flags?: readonly CommandFlagSpec[];
  usage?: string;
  defaultHandler?: boolean;
}

/**
 * Builds a `CommandHandlerSpec` for a `>head` command. `head` is the bare
 * slug — no leading `>`.
 */
export function commandSchema(
  head: string,
  opts: CommandSchemaOptions = {},
): CommandHandlerSpec {
  return {
    family: "command.v1",
    head,
    ...(opts.label ? { label: opts.label } : {}),
    ...(opts.description ? { description: opts.description } : {}),
    ...(opts.args ? { args: [...opts.args] } : {}),
    ...(opts.flags ? { flags: [...opts.flags] } : {}),
    ...(opts.usage ? { usage: opts.usage } : {}),
    ...(opts.defaultHandler ? { defaultHandler: opts.defaultHandler } : {}),
  };
}

/** Options accepted by `skillSpec`. */
export interface SkillSpecOptions {
  label?: string;
  description?: string;
  contextRequirements?: readonly SkillContextRequirement[];
  acceptsCaptureTarget?: CaptureTarget;
}

/** Builds a `SkillHandlerSpec` for a `/slug` skill. */
export function skillSpec(
  slug: string,
  opts: SkillSpecOptions = {},
): SkillHandlerSpec {
  return {
    family: "skill.v1",
    slug,
    ...(opts.label ? { label: opts.label } : {}),
    ...(opts.description ? { description: opts.description } : {}),
    ...(opts.contextRequirements
      ? { contextRequirements: [...opts.contextRequirements] }
      : {}),
    ...(opts.acceptsCaptureTarget
      ? { acceptsCaptureTarget: opts.acceptsCaptureTarget }
      : {}),
  };
}

/**
 * Variadic builder that returns the array authors export as
 * `metadata.menuSyntax`. Accepts any mix of capture / command / skill specs
 * (including the output of `captureTarget`, `commandSchema`, `skillSpec`).
 */
export function menuSyntax(
  ...specs: MenuSyntaxHandlerSpec[]
): MenuSyntaxMetadata {
  return specs;
}
