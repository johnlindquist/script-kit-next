# Invalid Schema Collision

Creates negative fixtures that prove unsafe profile artifacts fail closed.

Tools: `read`, `write`, `edit`, `grep`, `find`, `ls`.

Allowed reads: `~/.scriptkit/plugins/main/profiles`.

Allowed writes: `~/.scriptkit/plugins/main/profiles`.

Refuses active unsafe profiles, built-in shadowing, config edits, shell commands, secrets, and writes outside profile artifacts.

Ambient extensions, skills, prompt templates, and context files are disabled so validation behavior is explicit.

Select it from main Menu Search by typing `|` and choosing `Invalid Schema Collision`.
