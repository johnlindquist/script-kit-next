# Audit Report: `src/actions/builders/notes.rs`

## Scope
- `get_notes_command_bar_actions()` in `src/actions/builders/notes.rs:31`
- `get_new_chat_actions()` in `src/actions/builders/notes.rs:167`
- `get_note_switcher_actions()` in `src/actions/builders/notes.rs:229`
- Runtime handlers verified in:
  - `src/notes/window/panels.rs:179`
  - `src/ai/window/command_bar.rs:182`
  - `src/notes/window/notes_actions.rs:89`
  - `src/notes/window/notes.rs:234`

## Summary
- Action IDs are unique within each builder and consistently namespaced by prefix.
- `ActionCategory` usage is consistent (`ScriptContext`) across all three builders.
- `SectionStyle` usage is consistent with host context:
  - Notes command bars use `CommandBarConfig::notes_style()` (`SectionStyle::Separators`) in `src/notes/window/init.rs:147`.
  - AI new chat uses `CommandBarConfig::ai_style()` (`SectionStyle::Headers`) in `src/ai/window/init.rs:227`.
- Primary issues are UX/behavior mismatches and silent failure paths.

## Findings

### High
1. `Copy Note As...` does not match its own label/description.
- Builder advertises chooser behavior (`"Copy note in a chosen format"`) in `src/actions/builders/notes.rs:101`.
- Runtime handler maps `copy_note_as` directly to Markdown-only copy (`copy_note_as_markdown`) in `src/notes/window/panels.rs:209` and `src/notes/window/notes_actions.rs:89`.
- Impact: user expectation mismatch; no format selection occurs despite `...` label.

2. `Export...` behaves as fixed HTML-to-clipboard export with no choice/UI.
- Builder label/shortcut imply a multi-option export flow in `src/actions/builders/notes.rs:137`.
- Runtime handler always calls `export_note(ExportFormat::Html)` in `src/notes/window/panels.rs:212`.
- `export_note` copies to clipboard via `pbcopy` in `src/notes/window/notes.rs:254` (not file export, no picker).
- Impact: behavior likely surprises users expecting destination/format selection.

### Medium
3. New chat preset actions lack descriptions.
- Preset actions are created with `description: None` in `src/actions/builders/notes.rs:192`.
- This weakens scanability vs last-used/model rows that include provider descriptions.
- Existing tests explicitly encode this as current behavior in `src/actions/builders_tests/part_04.rs:61`.

4. Index-based IDs for `last_used_*` and `model_*` are not stable/descriptive.
- IDs use list positions (`last_used_{idx}`, `model_{idx}`) in `src/actions/builders/notes.rs:177` and `src/actions/builders/notes.rs:203`.
- Runtime re-reads by index in `src/ai/window/command_bar.rs:190` and `src/ai/window/command_bar.rs:209`.
- If backing lists change while dialog is open, selected IDs can resolve to a different item or no-op.

5. Silent no-op/error paths when selected note is missing or stale.
- `duplicate_selected_note` returns early with no feedback when selection or note is missing in `src/notes/window/notes_actions.rs:121` and `src/notes/window/notes_actions.rs:124`.
- `copy_note_deeplink`, `create_note_quicklink`, and `export_note` similarly no-op when lookup fails in `src/notes/window/notes_actions.rs:94`, `src/notes/window/notes_actions.rs:101`, `src/notes/window/notes.rs:235`.
- `execute_note_switcher_action` closes panel and only logs warning for unresolved `note_*` in `src/notes/window/panels.rs:294`.
- Impact: user gets no actionable feedback for invalid selection state.

6. Shortcut hint conflict around deeplink copy.
- Command bar shows `Copy Deeplink` with `⇧⌘D` in `src/actions/builders/notes.rs:118`.
- Global notes key handling maps `cmd+shift+d` to insert date/time in `src/notes/window/keyboard.rs:306`.
- While command bar selection runs via Enter, the displayed hint can still be confusing/inconsistent with direct shortcut behavior.

### Low
7. Missing expected trash actions in command bar.
- Trash-mode command bar intentionally suppresses edit/copy/export actions (`src/actions/builders/notes.rs:46`, `src/actions/builders_tests/part_02.rs:349`).
- In trash view, restore/permanent-delete are available in titlebar buttons (`src/notes/window/render_editor_titlebar.rs:138`, `src/notes/window/render_editor_titlebar.rs:149`) but absent from command bar.
- Likely user expectation: Cmd+K in trash should expose restore/delete equivalents.

## Criteria Check
1. Clear label + description:
- Mostly good for notes command bar and note switcher.
- Gap: new chat presets have no description.
- Gap: `Copy Note As...`/`Export...` labels suggest chooser flows not implemented.

2. Error handling for empty/missing notes:
- Empty notes handled in switcher via `no_notes` placeholder in `src/actions/builders/notes.rs:283` and execution path in `src/notes/window/panels.rs:287`.
- Missing/stale note handling exists but mostly silent and non-user-visible.

3. Proper user feedback:
- Limited feedback exists (e.g., duplicate uses footer toast in `src/notes/window/notes_actions.rs:136`).
- Most copy/export stale-selection cases provide no UI feedback.

4. Consistent `ActionCategory` + `SectionStyle`:
- Consistent and correct for current architecture.

5. Action IDs unique + descriptive:
- Unique in each list.
- `last_used_*` and `model_*` IDs are weakly descriptive and not stable.

6. Missing actions users would expect:
- Trash workflow actions missing from command bar surface.

## Existing Coverage Notes
- Builder tests cover presence/absence and ordering for notes command bar: `src/actions/builders_tests/part_02.rs:289`.
- Builder tests cover new chat sections/order/icons: `src/actions/builders_tests/part_04.rs:5`.
- Builder tests cover note switcher empty/current/pinned/description: `src/actions/builders_tests/part_03.rs:109`.
- No focused tests found for:
  - mismatch between `Copy Note As...` label and Markdown-only behavior,
  - export action semantics (`Export...` -> fixed HTML clipboard),
  - index-ID drift risk for new chat actions.
