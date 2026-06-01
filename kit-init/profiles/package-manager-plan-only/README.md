# Package Manager Plan Only

Inspects package manifests and lockfiles, then proposes dependency plans without executing or editing.

Tools: `read`, `grep`, `find`, `ls`.

Allowed reads: `~/dev`.

Allowed writes: none.

Refuses installs, updates, shell commands, file edits, secrets, and package-manager credentials.

Ambient extensions, skills, prompt templates, and context files are disabled to keep plans based on explicit file inspection.

Select it from main Menu Search by typing `|` and choosing `Package Manager Plan Only`.
