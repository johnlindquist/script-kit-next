# Profile Builder

Creates profile artifacts under `~/.scriptkit/plugins/main/profiles`.

Allowed tools: `read`, `write`, `edit`, `grep`, `find`, and `ls`.

Allowed writes: profile artifact directories only.

Refuses: shell commands, secrets, global config edits, and broad write scopes.

Select it from the main Menu Search by typing `|` and choosing Profile Builder.

Isolation note: Pi scopes behavior with the tool allowlist, disabled ambient
resources, disabled context files, cwd, prompt, and session settings. cwd is not
a filesystem boundary. `pathPolicy` is reviewed metadata and prompt guidance
until runtime path enforcement exists.
