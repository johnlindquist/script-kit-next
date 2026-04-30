/**
 * Demo: declare `metadata.menuSyntax` using the kit-init SDK helpers.
 *
 * The helpers (`menuSyntax`, `captureTarget`, `commandSchema`, `skillSpec`)
 * are pure identity functions that pin literal types — they don't change
 * the runtime shape, so the launcher reads the same JSON it would from a
 * hand-written object literal.
 *
 * The import below resolves through the templated kit-init at
 * `$SK_PATH/kit-init/sdk/menu-syntax` once the user has run setup; until
 * then this demo is shipped as a TYPE-LEVEL example and the inline shape
 * (rendered below in `metadataInline` for reference) is what runs.
 */

import {
  captureTarget,
  commandSchema,
  menuSyntax,
  skillSpec,
} from "../../../kit-init/sdk/menu-syntax";

export const metadata = {
  name: "SDK Helpers Demo",
  description:
    "Reference: declare capture / command / skill handlers with the kit-init helpers",
  icon: "code",
  menuSyntax: menuSyntax(
    captureTarget("note", {
      accepts: ["tags", "kv"],
      label: "Save quick note",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
    }),
    commandSchema("snippet", {
      label: "Insert a snippet by name",
      args: [{ name: "name", required: true }],
      flags: [
        {
          name: "--lang",
          description: "Filter by language (e.g. ts, py)",
        },
      ],
      usage: "!snippet -- <name> [--lang <id>]",
    }),
    skillSpec("summarize", {
      label: "Summarize the current selection",
      contextRequirements: ["selection.text"],
    }),
  ),
};

// Reference: the inline shape the helpers above expand to. Useful for
// authors who want to copy/paste without taking the helper dependency.
export const metadataInline = {
  name: metadata.name,
  description: metadata.description,
  icon: metadata.icon,
  menuSyntax: [
    {
      family: "capture.v1",
      targets: ["note"],
      accepts: ["tags", "kv"],
      label: "Save quick note",
      payloadSchema: "kit://schema/menu-syntax/payload-v1",
    },
    {
      family: "command.v1",
      head: "snippet",
      label: "Insert a snippet by name",
      args: [{ name: "name", required: true }],
      flags: [{ name: "--lang", description: "Filter by language (e.g. ts, py)" }],
      usage: "!snippet -- <name> [--lang <id>]",
    },
    {
      family: "skill.v1",
      slug: "summarize",
      label: "Summarize the current selection",
      contextRequirements: ["selection.text"],
    },
  ],
};
