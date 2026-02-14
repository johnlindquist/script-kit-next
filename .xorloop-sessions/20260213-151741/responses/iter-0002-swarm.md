All done. Here's the summary:

## Swarm Results

| Worker | Status | Details |
|--------|--------|---------|
| `fix-truncate-utf8-l2` | **done** | Fixed 5 files: `prompt_handler/mod.rs`, `render_script_list/mod.rs`, `fallbacks/builtins.rs`, `executor/errors.rs`, `designs/compact.rs` — committed as `1fb115d0` |
| `fix-truncate-utf8-l3` | **done** (reporting artifact) | Fixed 2 files: `designs/core/parts.rs`, `prompts/chat/render_turns.rs` — committed as `90364620` |

**Both commits pushed to `main`.** Each file now has a local `truncate_str_chars()` helper that uses `char_indices().nth()` to find safe byte boundaries instead of raw `&str[..N]` slicing. This prevents panics on multi-byte UTF-8 input (e.g., emoji, CJK characters, accented text).

**Verification:** `cargo check` and `cargo clippy` pass clean.
