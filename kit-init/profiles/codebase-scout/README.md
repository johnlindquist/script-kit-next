# Codebase Scout

Read-only profile for local codebase orientation.

Allowed tools: `read`, `grep`, `find`, and `ls`.

Allowed writes: none.

Refuses: edits, shell commands, installs, secrets, and ambient skills/extensions.

Select it from the main Menu Search by typing `|` and choosing Codebase Scout.

Isolation note: Pi scopes behavior with the tool allowlist, disabled ambient
resources, disabled context files, cwd, prompt, and session settings. cwd is not
a filesystem boundary. `pathPolicy` is reviewed metadata and prompt guidance
until runtime path enforcement exists.
