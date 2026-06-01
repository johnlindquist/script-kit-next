# Legacy Agent Import

Converts one legacy agent markdown file into reviewable profile artifacts.

Tools: `read`, `write`, `edit`, `grep`, `find`, `ls`.

Allowed reads: `~/.scriptkit/plugins/main/agents`, `~/.scriptkit/plugins/main/profiles`.

Allowed writes: `~/.scriptkit/plugins/main/profiles`.

Refuses bulk imports, unreviewed overwrites, shell commands, secrets, and writes outside profile artifacts.

Ambient extensions, skills, prompt templates, and context files are disabled so imported behavior must be explicit in the generated profile.

Select it from main Menu Search by typing `|` and choosing `Legacy Agent Import`.
