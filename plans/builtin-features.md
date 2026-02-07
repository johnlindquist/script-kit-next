# Built-in Features Audit

Date: 2026-02-07  
Agent: `codex-builtin-features`  
Scope: `src/builtins.rs`, `src/app_execute.rs`, `src/app_impl.rs`

## Summary

The built-in feature architecture is generally coherent: registration (`get_builtin_entries`) feeds unified search, grouped rendering, and `execute_builtin` dispatch. Variant-level execution coverage is mostly complete.

The biggest gaps are UX/behavior mismatches rather than missing match arms:

1. Some enum variants are executable but not discoverable from built-in entries (`OpenNotes`, `OpenAi`, `ClearConversation`).
2. Several exposed AI built-ins are not implemented (`ClearConversation`) or intentionally “coming soon” but still surfaced as first-class commands.
3. Alias and command-id execution resolve built-ins using `BuiltInConfig::default()` instead of runtime config, which can ignore user toggles.
4. Built-in frecency identity uses display name (`builtin:{name}`) instead of stable id (`builtin:{id}`), causing fragility.
5. Built-ins are aggressively prioritized in tie-break sorting, which can displace user scripts/apps for ambiguous queries.

## End-to-End Flow (registration -> search/render -> execution)

1. Built-ins are loaded at startup from user config:
   - `src/app_impl.rs:28` calls `builtins::get_builtin_entries(&config.get_builtins())`.
2. Unified search includes built-ins on each filter cache miss:
   - `src/app_impl.rs:2103` -> `scripts::fuzzy_search_unified_all(...)` with `self.builtin_entries`.
3. Grouped rendering path also includes built-ins and dynamically injects menu bar entries in search mode:
   - `src/app_impl.rs:2189` -> `get_grouped_results(...)`
   - `src/scripts/grouping.rs:105-108` merges `menu_bar_items_to_entries(...)` into built-ins when filtering.
4. Search ordering prefers built-ins on equal score:
   - `src/scripts/search.rs:1551-1574` tie-breaks by type with `BuiltIn` first.
5. Built-in execution dispatch is centralized:
   - `src/app_execute.rs:20` `execute_builtin(...)`.
   - Confirmation gating: `src/app_execute.rs:31` via `self.config.requires_confirmation(&entry.id)`.
   - Confirmed actions path: `src/app_execute.rs:1964-1989` -> `execute_builtin_confirmed(...)`.

## Coverage Audit

## 1) `BuiltInFeature` variant coverage

Variant execution coverage is complete in `execute_builtin`:
- Covered match arms: `src/app_execute.rs:127-1072`
- Includes all variants from `src/builtins.rs:210-250`.

Notable nuances:
- `BuiltInFeature::AppLauncher` and `BuiltInFeature::App(String)` are executable (`src/app_execute.rs:162`, `src/app_execute.rs:191`) but no longer registered by `get_builtin_entries` (intentional legacy behavior documented at `src/builtins.rs:364-386`).
- `BuiltInFeature::MenuBarAction` is not in static entries; it is generated dynamically from active app menus (`src/builtins.rs:1177-1244`, `src/scripts/grouping.rs:105-108`).

## 2) Command subtype completeness

### Notes commands
- Enum variants: `OpenNotes`, `NewNote`, `SearchNotes`, `QuickCapture` (`src/builtins.rs:106-111`)
- Registered entries: `NewNote`, `SearchNotes`, `QuickCapture` only (`src/builtins.rs:758-783`)
- Executed variants: all four (`src/app_execute.rs:519-522`)

Gap: `OpenNotes` is executable but not discoverable from built-in entries.

### AI commands
- Enum variants include `OpenAi`, `NewConversation`, `ClearConversation`, screenshot/selection flows, presets (`src/builtins.rs:116-136`)
- Registered entries include most, but no `OpenAi` and no `ClearConversation` (`src/builtins.rs:789-910`)
- Execution behavior:
  - `OpenAi | NewConversation` opens AI window (`src/app_execute.rs:552-565`)
  - `ClearConversation` TODO; currently just opens AI window (`src/app_execute.rs:567-572`)
  - `SendScreenAreaToAi` “coming soon” toast (`src/app_execute.rs:733-743`)
  - Preset commands “coming soon” toast (`src/app_execute.rs:746-760`)

Gaps:
- `OpenAi` is executable but not discoverable.
- `ClearConversation` is executable in enum but not implemented and not discoverable.
- Several discoverable AI entries are intentionally non-functional placeholders.

### System actions
- System action entries are broadly complete and mapped in both direct and confirmed execution paths (`src/app_execute.rs:386-458`, `src/app_execute.rs:2027-2115`).

## UX Friction Points

## P0

### 1) Config mismatch in alias/command-id execution path

Issue:
- Alias and command-id builtin resolution use `BuiltInConfig::default()` instead of runtime `self.config.get_builtins()`.
- Evidence:
  - Alias path: `src/app_impl.rs:2367-2369`
  - Command ID path: `src/app_impl.rs:6199-6201`

Impact:
- User-disabled built-ins can still resolve/execute through aliases or direct command IDs.
- Runtime behavior diverges between search UI and direct command execution.

Recommendation:
- Replace default config with runtime config in both call sites:
  - `builtins::get_builtin_entries(&self.config.get_builtins())`
- Add regression tests for disabled built-ins not resolving by alias/command-id.

### 2) Surfaced commands with incomplete behavior

Issue:
- User-visible built-ins are presented as functional but execute to “coming soon” toasts or TODO behavior.
- Evidence:
  - TODO clear conversation: `src/app_execute.rs:568`
  - Screen area “coming soon”: `src/app_execute.rs:738`
  - Presets “coming soon”: `src/app_execute.rs:752`
  - These entries are registered in `get_builtin_entries`: `src/builtins.rs:860-910`.

Impact:
- Breaks trust in built-ins as actionable commands.

Recommendation:
- Either hide non-functional commands behind feature flags or `#[cfg(debug_assertions)]` until implemented.
- If intentionally discoverable, annotate names/descriptions as “Preview” and provide deterministic fallback action.

## P1

### 3) Missing discoverability for executable commands

Issue:
- `OpenNotes` and `OpenAi` exist in enum + execution, but have no built-in entries.
- Evidence:
  - Enums: `src/builtins.rs:106-117`
  - Execution: `src/app_execute.rs:519-522`, `src/app_execute.rs:552-565`
  - Registration lacks entries: `src/builtins.rs:758-910`

Impact:
- Users cannot explicitly choose “open window” semantics vs action-specific commands.
- Increases ambiguity between parent entries (`Notes`, `AI Chat`) and command entries.

Recommendation:
- Add explicit entries:
  - `builtin-open-notes` -> `NotesCommand(OpenNotes)`
  - `builtin-open-ai` -> `AiCommand(OpenAi)`
- Optionally keep `Notes`/`AI Chat` top-level entries as simple aliases to these commands.

### 4) Built-in identity for frecency is unstable

Issue:
- Frecency paths use display names (`builtin:{entry.name}`), not stable IDs.
- Evidence:
  - Search mode path: `src/scripts/grouping.rs:130`
  - Group mode path: `src/scripts/grouping.rs:280`

Impact:
- Renaming commands breaks usage history and exclusions.
- Name collisions across commands are possible.

Recommendation:
- Switch to `builtin:{entry.id}` everywhere and migrate old keys opportunistically.

### 5) Tie-break sorting strongly favors built-ins over user content

Issue:
- Equal-score ordering always puts built-ins before apps/scripts/scriptlets.
- Evidence:
  - `src/scripts/search.rs:1551-1574`

Impact:
- Common user script names can be displaced by built-ins, especially with short queries.

Recommendation:
- Restrict tie-break preference to a smaller “core built-in” set or apply after a minimum query length.
- Alternatively incorporate recency/frequency earlier for tie cases.

## P2

### 6) Hardcoded no-main-window list can drift

Issue:
- `NO_MAIN_WINDOW_BUILTINS` is a static ID list in command-id path.
- Evidence: `src/app_impl.rs:6169-6176`

Impact:
- New built-ins that open their own windows require manual list maintenance.

Recommendation:
- Encode this in builtin metadata (e.g., `opens_external_window: bool`) and derive behavior from entry data.

### 7) Stale “BLOCKED” commentary in file search section

Issue:
- `app_execute.rs` includes a large “BLOCKED” note claiming `FileSearchView` wiring is missing.
- Evidence: `src/app_execute.rs:1598-1638`
- But file search execution is active (`src/app_execute.rs:1067-1072`) and `FileSearchView` usage exists in app codebase.

Impact:
- Misleads future maintainers and agents.

Recommendation:
- Remove or update stale block comment.

## Missing Built-ins That Would Improve UX

1. `Open Notes` command entry (explicit and searchable action).
2. `Open AI` command entry (parallel to “New AI Conversation” and screenshot/send actions).
3. `Clear AI Conversation` as a real implemented action (not TODO), then expose as an entry.
4. Optional: `Reload Built-ins/Config` command to refresh builtin list and related runtime toggles without full restart (would reduce confusion when changing `~/.scriptkit/config.ts`).

## Recommended Implementation Plan

## Phase 1 (high confidence, low risk)

1. Fix config consistency:
   - Replace `BuiltInConfig::default()` with runtime config in:
     - `src/app_impl.rs:2367`
     - `src/app_impl.rs:6199`
2. Add discoverability entries:
   - `builtin-open-notes`
   - `builtin-open-ai`
3. Update tests in `src/builtins.rs` for new entries and command-id lookups.

## Phase 2 (UX correctness)

1. Hide or mark preview-only AI commands until implemented:
   - `SendScreenAreaToAi`
   - `CreateAiPreset`
   - `ImportAiPresets`
   - `SearchAiPresets`
2. Implement `ClearConversation` behavior in AI window module and expose it.

## Phase 3 (ranking and maintainability)

1. Migrate builtin frecency keying to `entry.id`.
2. Replace `NO_MAIN_WINDOW_BUILTINS` hardcoded list with metadata-driven behavior.
3. Revisit tie-break sort policy to reduce built-in crowding against user content.

## Suggested Tests

1. `test_get_builtin_entries_includes_open_notes_and_open_ai_commands`
2. `test_execute_by_command_id_respects_runtime_builtin_config`
3. `test_find_alias_match_respects_runtime_builtin_config`
4. `test_builtin_frecency_uses_stable_entry_id`
5. `test_preview_ai_commands_hidden_when_feature_flag_disabled`
6. `test_clear_conversation_builtin_clears_ai_window_state`

## Validation

This task was an audit/report pass. No runtime Rust behavior was changed.
