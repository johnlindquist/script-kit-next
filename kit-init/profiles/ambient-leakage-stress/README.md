# Ambient Leakage Stress

Stress-tests attempts to use ambient resources, secrets, and disallowed capabilities.

Tools: `read`, `grep`, `find`, `ls`.

Allowed reads: `~/.scriptkit/agent-chat/profile-fixtures/ambient-leakage-stress`.

Allowed writes: none.

Refuses Slack, Gmail, skills, memories, extensions, hidden context, secrets, shell, writes, and outside paths.

Ambient extensions, skills, prompt templates, and context files are disabled because this profile exists to prove those resources stay unavailable.

Select it from main Menu Search by typing `|` and choosing `Ambient Leakage Stress`.
