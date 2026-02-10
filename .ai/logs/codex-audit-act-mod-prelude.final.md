Implemented the audit fix and wrote the findings log.

**Changed files**
- `src/actions/prelude.rs:7`  
  Added missing consumer-facing exports so `actions::prelude` now includes:
  - command-bar API (`is_command_bar_open`, `CommandBar`, `CommandBarConfig`, `CommandBarHost`)
  - builder functions/types already public at `actions` root (`get_ai_command_bar_actions`, `get_new_chat_actions`, `get_notes_command_bar_actions`, `get_note_switcher_actions`, `to_deeplink_name`, `ClipboardEntryInfo`)
- `src/actions/prelude.rs:23`  
  Added regression test `test_actions_prelude_exports_full_consumer_surface_when_imported`.
- `.ai/logs/codex-audit-act-mod-prelude.final.md:1`  
  Wrote full audit findings and verification notes.

**How to test**
1. `cargo test --lib actions::prelude::tests::test_actions_prelude_exports_full_consumer_surface_when_imported`

**Verification run**
- Ran the command above successfully: 1 passed, 0 failed.

**Risks / known gaps**
- I only ran a scoped test (per parallel-agent constraints), not full workspace checks.
- `src/actions/mod.rs` currently shows an unrelated pre-existing unstaged delta at `src/actions/mod.rs:71` (`#[path = "tests/mod.rs"] mod tests;`) that was already present in the working tree.
- One unrelated warning was previously observed in another area (`ClipboardTypeFilter::from_str` dead code).

**Commits**
- No commits were made in this task.