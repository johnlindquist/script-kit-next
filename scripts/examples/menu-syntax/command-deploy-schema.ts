/**
 * Demo: declare a `command.v1` schema for `>deploy` so the launcher's
 * main hint card surfaces expected args and flags before the user types
 * anything past the head.
 *
 * The runtime contract: typing `>deploy` in the launcher should produce
 * a `getState.menuSyntaxMainHint` snapshot whose `rows` includes labels
 * `env` (with a "required" chip and "prod | staging | dev" value) and
 * `--dry-run` (with the alias and description text). That's the receipt
 * for the Pass-10 sdk-command-schema story; the in-process equivalent is
 * `cargo test --lib menu_syntax::main_hint::tests::command_composer_renders_schema_rows_for_registered_head`.
 */

import {
  commandSchema,
  menuSyntax,
} from "../../../kit-init/sdk/menu-syntax";

export const metadata = {
  name: "Deploy Service",
  description: "Reference: >deploy schema surfaces env arg + --dry-run flag",
  icon: "rocket-launch",
  menuSyntax: menuSyntax(
    commandSchema("deploy", {
      label: "Deploy a service",
      description: "Run a guarded production deploy",
      args: [
        {
          name: "env",
          required: true,
          values: ["prod", "staging", "dev"],
          description: "Target environment",
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
    }),
  ),
};

// The actual deploy is a stub — this demo exists to exercise the schema
// surface, not to ship a real deploy tool. Real deploys belong elsewhere.
const env = process.env.KIT_MENU_SYNTAX_COMMAND_ARGV ?? "<no argv>";
console.log(`Would deploy with argv: ${env}`);
