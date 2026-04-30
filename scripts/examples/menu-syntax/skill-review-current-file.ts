// Demo for sdk-skill-spec-metadata.
//
// Declares a `/review` skill that surfaces in `:type:skill review` filter
// rows but does NOT auto-register a `>review` command. To get a `>review`
// command, the author would also include a `command.v1` entry.
//
// Reading this file: the launcher picks up `metadata.menuSyntax`, parses
// each entry's `family`, and threads `skill.v1` rows through
// `src/menu_syntax/skill.rs::skill_specs_from_value` for the
// `:type:skill` filter and the inline `Suggested skills` UI.

export const metadata = {
  name: "Review the file in front of me",
  description:
    "Run a focused code-review pass on the file currently visible in the editor",
  icon: "eye",
  alias: "review-current-file",
  tags: ["menu-syntax", "demo", "skill"],
  category: "menu-syntax-demo",
  menuSyntax: [
    {
      family: "skill.v1",
      slug: "review",
      label: "Review the current file",
      description:
        "Look at the active editor buffer, list issues by severity, suggest fixes",
      contextRequirements: ["currentFile"],
    },
  ],
};

console.log("skill-review-current-file invoked");
console.log("Skill is registered for `:type:skill review` discovery only.");
console.log("To run as a command (`>review`), add a separate `command.v1` entry.");
