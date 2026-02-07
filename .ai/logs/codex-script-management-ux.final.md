Implemented script-management UX updates in the requested scope with focused changes to list hierarchy, fuzzy filtering, and remove confirmation.

**Changed Files**
- `src/prompts/select.rs:62`
  - Added `ChoiceDisplayMetadata` parsing for description/type/shortcut/last-run.
  - Replaced simple substring filtering with weighted fuzzy scoring (`score_choice_for_filter`) across name/description/value/type/last-run/shortcut.
  - Updated rendering to `UnifiedListItem` with clearer hierarchy:
    - title (with match highlighting),
    - subtitle metadata,
    - trailing shortcut badge,
    - explicit selection indicator.
  - Added tests for metadata parsing, scoring priority, and UTF-8 highlight index conversion.
- `src/app_actions.rs:61`
  - Added script removal target resolution (`ScriptRemovalTarget`) for Script/Scriptlet/Agent results.
  - Added confirmation-driven `"remove_script" | "delete_script"` flow with user feedback HUD.
  - Added trash behavior helper (`move_path_to_trash`) and scriptlet source-path anchor stripping.
  - Added unit tests for scriptlet anchor parsing and removal-target mapping.
- `src/app_execute.rs:6`
  - Added `system_action_feedback_message` helper and consistent HUD feedback for successful system actions.
  - Prevents auto-proceeding dangerous actions if confirmation modal fails to open (logs and skips).

**How To Test**
1. Compile:
   - `cargo check -q`
2. Verify search/filter runtime behavior through stdin protocol:
   - `timeout 12 bash -c '(printf "%s\n" "{\"type\":\"show\"}" "{\"type\":\"setFilter\",\"text\":\"scr\"}" "{\"type\":\"setFilter\",\"text\":\"scriptlet\"}" "{\"type\":\"hide\"}") | SCRIPT_KIT_AI_LOG=1 ./target/debug/script-kit-gpui 2>&1'`
   - Confirm `SetFilter` and `[SEARCH_TOTAL]` lines in output and in `~/.scriptkit/logs/script-kit-gpui.jsonl`.
3. Verify select prompt unit tests:
   - `cargo test -q prompts::select::tests --lib`

**Verification Results**
- `cargo check -q`: passed.
- `cargo test -q prompts::select::tests --lib`: passed (4 tests).
- Stdin runtime verification: passed; filter performance logs observed with `correlation_id`.

**Risks / Known Gaps**
- Global gates are currently red due unrelated parallel changes:
  - `cargo clippy --all-targets -- -D warnings` fails (many unrelated warnings/errors, plus duplicate `tests` module error from `src/render_prompts/arg.rs:620` vs `src/main.rs:3846`).
  - `cargo test` currently fails 3 unrelated dialog validation tests.
- Scriptlet removal currently targets the backing snippet file path (anchor stripped). If a file contains multiple scriptlets, this removes the whole file, not a single section.