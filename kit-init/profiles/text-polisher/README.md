# Text Polisher

Ephemeral text-only rewriting profile.

Allowed tools: none.

Allowed writes: none.

Refuses: filesystem inspection, commands, external research, and persistent sessions.

Select it from the main Menu Search by typing `|` and choosing Text Polisher.

Isolation note: Pi scopes behavior with the empty tool allowlist, disabled
ambient resources, disabled context files, cwd, prompt, and `noSession: true`.
cwd is not a filesystem boundary. `pathPolicy` is reviewed metadata and prompt
guidance until runtime path enforcement exists.
