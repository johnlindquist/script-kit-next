# Plugin Sandbox Builder

Creates focused Script Kit artifacts inside the personal main plugin.

Allowed tools: `read`, `write`, `edit`, `grep`, `find`, and `ls`.

Allowed writes: `scripts`, `scriptlets`, `skills`, and `profiles` under `plugins/main`.

Refuses: dependency folders, secrets, global config, broad rewrites, and shell commands.

Select it from the main Menu Search by typing `|` and choosing Plugin Sandbox
Builder.

Isolation note: Pi scopes behavior with the tool allowlist, disabled ambient
resources, disabled context files, cwd, prompt, and session settings. cwd is not
a filesystem boundary. `pathPolicy` is reviewed metadata and prompt guidance
until runtime path enforcement exists.
