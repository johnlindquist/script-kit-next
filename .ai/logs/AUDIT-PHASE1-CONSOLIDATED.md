# Audit Phase 1 - Consolidated Summary

**Generated**: 2026-02-07
**Source**: 38 `codex-audit-act-*.final.md` reports
**Scope**: `src/actions/` subsystem -- builders, dialog, command bar, window, types, constants, execution, keyboard, search/filter, theming, lifecycle, tests

---

## Critical Issues

### C1. Unicode slicing panic risk in shortcut keycap parsing
- **Severity**: Critical
- **Source report**: `codex-audit-act-dialog-part02-01`
- **Files**: `src/actions/dialog/part_02/part_01.rs:154-155`
- **Description**: `count_section_headers` or `parse_shortcut_keycaps` performs byte-level string slicing on user-provided shortcut strings. If the input contains multi-byte Unicode characters, indexing by byte offset can panic at runtime with "byte index is not a char boundary."
- **Recommended fix**: Replace byte-index slicing with `chars()` iteration or `char_indices()` for safe Unicode handling.

### C2. `command_bar_key_intent` misclassifies named keys as typed characters
- **Severity**: Critical
- **Source report**: `codex-audit-act-cmdbar-part01`
- **Files**: `src/actions/command_bar/part_01.rs:88`
- **Description**: Named keys like `"space"` fall through to the `TypeChar` branch, extracting the first character (`'s'`). This corrupts search input and breaks shortcut behavior. Any named key longer than one character will be misinterpreted as a typed character.
- **Recommended fix**: Add an explicit match arm for known named keys (`"space"`, `"tab"`, `"backspace"`, etc.) before the `TypeChar` fallback.

### C3. `close_actions_popup` bypasses per-dialog `on_close` cleanup callbacks
- **Severity**: Critical
- **Source report**: `codex-audit-act-dialog-part05`
- **Files**: `src/app_impl/actions_dialog.rs:96,106,223`; `src/render_builtins/actions.rs:79`; `src/actions/window/part_01.rs:302,316`
- **Description**: The Escape/Enter close path always calls `close_actions_popup`, which clears shared popup state but never invokes the dialog's `on_close` callback. Host-specific cleanup (e.g., clearing `file_search_actions_path`) is only encoded in `on_close` callbacks, leaving stale state behind on main-window close paths.
- **Recommended fix**: Ensure `close_actions_popup` invokes the registered `on_close` callback before clearing state, or unify the close path with the window variant that already does this.

---

## Major Issues

### M1. `search_position = Hidden` still reserves search-row height in layout
- **Severity**: High
- **Source report**: `codex-audit-act-dialog-part04-body03`
- **Files**: `src/actions/dialog/part_04/body_part_03.rs:24,27,132,134`
- **Description**: Popup height is computed including `search_box_height` before `show_search` is derived. When `SearchPosition::Hidden`, the search box is not rendered but its height is still allocated, making the dialog taller than necessary and reducing visible rows.
- **Recommended fix**: Compute `search_box_height` conditionally based on `SearchPosition` before using it in the height calculation.

### M2. Footer can push total popup height past `POPUP_MAX_HEIGHT`
- **Severity**: High
- **Source report**: `codex-audit-act-dialog-part04-body03`
- **Files**: `src/actions/dialog/part_04/body_part_03.rs:24,26,74,128,129`
- **Description**: `items_height` is clamped against `POPUP_MAX_HEIGHT - search_box_height - header_height`, but footer height is added after this clamp. The final container height can exceed the configured max when `show_footer` is true.
- **Recommended fix**: Include footer height in the clamp calculation.

### M3. Scrollbar visible-range math ignores header/footer height
- **Severity**: High
- **Source report**: `codex-audit-act-dialog-part04-body01`
- **Files**: `src/actions/dialog/part_04/body_part_01.rs` (scrollbar calculation)
- **Description**: Scrollbar position/thumb calculation does not account for header and footer heights, causing inaccurate scroll feedback on long lists.
- **Recommended fix**: Subtract header/footer heights from the available viewport when computing scrollbar geometry.

### M4. `open_with` action behaves like `show_info` instead of opening with an application chooser
- **Severity**: High
- **Source report**: `codex-audit-act-builders-filepath`
- **Files**: `src/actions/builders/file_path.rs` (builder); `src/app_actions/handle_action.rs` (handler)
- **Description**: The `open_with` action's label and description suggest an "Open With..." chooser flow, but the runtime handler behaves identically to `show_info`, showing file info instead of presenting an application picker.
- **Recommended fix**: Implement the "Open With" chooser behavior or update the label/description to match current behavior.

### M5. `Copy Note As...` and `Export...` labels suggest chooser flows not implemented
- **Severity**: High
- **Source report**: `codex-audit-act-builders-notes`
- **Files**: `src/actions/builders/notes.rs:101,137`; `src/notes/window/panels.rs:209,212`; `src/notes/window/notes_actions.rs:89`; `src/notes/window/notes.rs:254`
- **Description**: "Copy Note As..." always copies as Markdown (no format chooser). "Export..." always copies HTML to clipboard via `pbcopy` (no file export, no format picker). Both labels use `...` convention implying a sub-dialog.
- **Recommended fix**: Either implement chooser UIs or rename to "Copy Note as Markdown" and "Copy as HTML to Clipboard".

### M6. Non-deterministic auto-close on focus loss for actions window
- **Severity**: High
- **Source report**: `codex-audit-act-window-part01`
- **Files**: `src/actions/window/part_01.rs`
- **Description**: Focus-loss handling for the actions window is non-deterministic. The window may or may not close when the user switches focus depending on timing and activation subscription state.
- **Recommended fix**: Implement deterministic focus-loss close via `ensure_activation_subscription()` as outlined in the `window-lifecycle-v2` audit.

### M7. Stale `ACTIONS_WINDOW` singleton handle on direct popup self-close
- **Severity**: High
- **Source report**: `codex-audit-act-window-part01`, `codex-audit-act-window-part02-v2`
- **Files**: `src/actions/window/part_01.rs:310,326`; `src/actions/window/part_02.rs`
- **Description**: When the actions window is closed by user-driven paths (Escape/Enter), the global `ACTIONS_WINDOW` handle may not be cleared, leaving a stale reference. Subsequent open attempts may fail or interact with a dead window handle.
- **Recommended fix**: Route all close paths through `defer_close()` which clears the singleton before `remove_window()`.

### M8. `to_deeplink_name` does not URL-encode non-ASCII characters
- **Severity**: Medium
- **Source report**: `codex-audit-act-builders-shared`
- **Files**: `src/actions/builders/shared.rs:7,10`; `src/actions/builders/script_context.rs:256`; `src/actions/builders/scriptlet.rs:191`; `src/main_sections/deeplink.rs:21-31`
- **Description**: `to_deeplink_name` keeps Unicode alphanumeric characters in URLs without percent-encoding. External clients may percent-encode these, causing mismatches when the deeplink parser does not percent-decode.
- **Recommended fix**: Either percent-encode the slug output or add percent-decoding to the deeplink parser.

### M9. Empty/symbol-only names produce empty deeplink segments
- **Severity**: Medium
- **Source report**: `codex-audit-act-builders-shared`
- **Files**: `src/actions/builders/shared.rs:12-15`; `src/actions/builders/script_context.rs:256`; `src/actions/builders/scriptlet.rs:191`
- **Description**: All-special-character or empty input to `to_deeplink_name` returns `""`, yielding `scriptkit://run/` which maps to a meaningless `script/` command target.
- **Recommended fix**: Return a sentinel slug (e.g., `"_unnamed"`) or prevent deeplink generation for empty slugs.

### M10. Index-based IDs for chat model/preset actions are not stable
- **Severity**: Medium
- **Source report**: `codex-audit-act-builders-notes`
- **Files**: `src/actions/builders/notes.rs:177,192,203`; `src/ai/window/command_bar.rs:190,209`
- **Description**: `last_used_{idx}` and `model_{idx}` action IDs use list positions. If backing model lists change while the dialog is open, selected IDs can resolve to a different model or no-op.
- **Recommended fix**: Use model identifiers (e.g., provider+model name) instead of positional indices.

### M11. Search-height calculation can diverge from actual search visibility
- **Severity**: Medium
- **Source report**: `codex-audit-act-dialog-part04-body02`
- **Files**: `src/actions/dialog/part_04/body_part_02.rs:24,470`; `src/actions/dialog/part_04/body_part_03.rs:134`
- **Description**: The search height used in layout calculations can differ from what is actually rendered, causing layout drift.

### M12. Multiple AI command failure paths log errors without user feedback
- **Severity**: Medium
- **Source report**: `codex-audit-act-execution-builtin`
- **Files**: `src/app_execute/builtin_execution.rs:87,634,677,709,752,796`
- **Description**: Several AI command failures produce only log messages ("Failed to open AI") with no user-facing toast, HUD, or error message.
- **Recommended fix**: Add user-visible error feedback (toast/HUD) for AI command execution failures.

### M13. `builtin-open-ai` / `builtin-open-notes` missing from `NO_MAIN_WINDOW_BUILTINS`
- **Severity**: Medium
- **Source report**: `codex-audit-act-execution-builtin`
- **Files**: `src/app_impl/execution_scripts.rs:317`
- **Description**: Hotkey/deeplink execution of these built-ins can inadvertently re-show the main window because they are not in the exclusion list.
- **Recommended fix**: Add both variants to `NO_MAIN_WINDOW_BUILTINS`.

### M14. Silent no-op paths when selected note is missing or stale
- **Severity**: Medium
- **Source report**: `codex-audit-act-builders-notes`
- **Files**: `src/notes/window/notes_actions.rs:94,101,121,124`; `src/notes/window/notes.rs:235`; `src/notes/window/panels.rs:294`
- **Description**: `duplicate_selected_note`, `copy_note_deeplink`, `create_note_quicklink`, and `export_note` return early silently when the selected note is missing or stale. User gets no feedback.
- **Recommended fix**: Show a toast/HUD indicating the selected note could not be found.

### M15. `edit_script` / `edit_scriptlet` launch failures are log-only
- **Severity**: Medium
- **Source report**: `codex-audit-act-execution-scripts`
- **Files**: `src/app_actions/handle_action.rs` (edit handler paths)
- **Description**: If launching the editor fails, only a log message is produced. The user sees a success HUD before the async operation completes.
- **Recommended fix**: Add error feedback after failed editor launch.

### M16. Duplicate `mod tests` definition blocks compilation
- **Severity**: Medium
- **Source report**: `codex-audit-act-cmdbar-part03`
- **Files**: `src/actions/command_bar/part_02.rs:387`; `src/actions/command_bar/part_03.rs:109`
- **Description**: Two `mod tests` definitions in the same module scope cause `E0428` and block all test execution for the command bar area.
- **Recommended fix**: Merge or rename one of the duplicate `mod tests` blocks.

### M17. Shortcut hint conflict: `Copy Deeplink` vs `Insert Date/Time`
- **Severity**: Medium
- **Source report**: `codex-audit-act-builders-notes`
- **Files**: `src/actions/builders/notes.rs:118`; `src/notes/window/keyboard.rs:306`
- **Description**: Command bar shows `Copy Deeplink` with `Shift+Cmd+D`, but the global notes keyboard handler maps that same shortcut to insert date/time.
- **Recommended fix**: Resolve the shortcut conflict by changing one of the bindings.

---

## Minor Issues

### m1. Stale size comments reference old row/header dimensions
- **Severity**: Low
- **Source report**: `codex-audit-act-constants`
- **Files**: `src/actions/dialog/part_01.rs:82`; `src/actions/dialog/part_04/body_part_02.rs:4,77,110,467`; `src/actions/dialog_part_04_rewire.rs:573`
- **Description**: Comments reference `ACTION_ITEM_HEIGHT (44px)` and `SECTION_HEADER_HEIGHT (24px)`, but actual constants are `36` and `22` respectively in `src/actions/constants.rs`.
- **Recommended fix**: Update comments to match current constant values.

### m2. Duplicate width constant `ACTIONS_WINDOW_WIDTH` vs `POPUP_WIDTH`
- **Severity**: Low
- **Source report**: `codex-audit-act-constants`
- **Files**: `src/actions/window/part_01.rs:146`; `src/actions/constants.rs` (`POPUP_WIDTH`)
- **Description**: `ACTIONS_WINDOW_WIDTH: f32 = 320.0` is defined separately from `POPUP_WIDTH = 320`, risking drift if only one is updated.
- **Recommended fix**: Use `POPUP_WIDTH` from `constants.rs` in the window module.

### m3. Hardcoded transparent color literal
- **Severity**: Low
- **Source report**: `codex-audit-act-dialog-part04-body02`
- **Files**: `src/actions/dialog/part_04/body_part_02.rs:305`
- **Description**: Uses `rgba(0x00000000)` instead of a theme-derived transparent value.
- **Recommended fix**: Replace with themed transparent background. (Noted as fixed in `codex-audit-act-theme-styling`.)

### m4. Uncentralized repeated layout literals (16.0 padding, 8.0 top padding)
- **Severity**: Low
- **Source report**: `codex-audit-act-constants`
- **Files**: `src/actions/dialog/part_04/body_part_02.rs:96`; `src/actions/dialog/part_04/body_part_03.rs:49,50`; `src/actions/dialog_part_04_rewire.rs:634`
- **Description**: Repeated `16.0` horizontal padding and `8.0` top padding are not centralized in constants.
- **Recommended fix**: Add `ACTION_PADDING_X` and `ACTION_PADDING_TOP` constants.

### m5. `scriptlet_action:*` IDs not guaranteed unique when H3 action commands collide
- **Severity**: Low
- **Source report**: `codex-audit-act-builders-scriptlet`
- **Files**: `src/actions/builders/scriptlet.rs` (H3 action parsing)
- **Description**: Two scriptlet H3 actions that resolve to the same command string produce duplicate IDs. Malformed H3 actions are silently dropped with no user-facing parse feedback.
- **Recommended fix**: Deduplicate or suffix IDs; optionally surface parse warnings.

### m6. New chat preset actions lack descriptions
- **Severity**: Low
- **Source report**: `codex-audit-act-builders-notes`
- **Files**: `src/actions/builders/notes.rs:192`
- **Description**: Preset actions are created with `description: None`, weakening scanability vs. other rows that include provider descriptions.
- **Recommended fix**: Add descriptive text for presets.

### m7. `dismiss_on_click_outside()` has no current call-site references
- **Severity**: Low
- **Source report**: `codex-audit-act-dialog-part02-03`
- **Files**: `src/actions/dialog/part_02/part_03.rs`
- **Description**: The method exists but no source code appears to call it, suggesting dead code or incomplete wiring.
- **Recommended fix**: Wire it to backdrop click handling or remove if unused.

### m8. No Tab/Shift+Tab navigation or focus-trap in action dialogs
- **Severity**: Low
- **Source report**: `codex-audit-act-keyboard-nav`
- **Files**: `src/actions/window/` (keyboard handling paths)
- **Description**: Action windows/dialogs have no Tab/Shift+Tab navigation and no focus-trap implementation. Search input focus is implicit via routed key events.
- **Recommended fix**: Add focus-trap and Tab key navigation for accessibility.

### m9. True-empty and search-empty states share same UI copy
- **Severity**: Low
- **Source report**: `codex-audit-act-empty-states`
- **Files**: Dialog rendering path
- **Description**: Both "no actions available" and "no actions match your search" show the same message (`"No actions match your search"`), which is misleading when the list is genuinely empty.
- **Recommended fix**: Differentiate messages based on whether a search filter is active.

### m10. Open-path empty-state height differs from resize-path behavior
- **Severity**: Low
- **Source report**: `codex-audit-act-window-part01`, `codex-audit-act-window-part02-v2`
- **Files**: `src/actions/window/part_02.rs:106`; `src/actions/window/part_03.rs:38`
- **Description**: Initial open can under-size the empty-result state compared to the resize logic, producing inconsistent window dimensions.
- **Recommended fix**: Unify the height calculation between open and resize paths.

### m11. Several window-operation failures silently ignored
- **Severity**: Low
- **Source report**: `codex-audit-act-window-part02-v2`
- **Files**: `src/actions/window/part_02.rs:204,248,279`
- **Description**: Window resize, position, and focus operations fail silently without logging or user feedback.
- **Recommended fix**: Add at minimum debug logging for failed window operations.

### m12. `Action` type missing `PartialEq` and `Display` implementations
- **Severity**: Low
- **Source report**: `codex-audit-act-types-model-v2`
- **Files**: `src/actions/types/action_model.rs`
- **Description**: `Action` has `Debug` derive but no `Display` or `PartialEq`, limiting test ergonomics and formatted output.
- **Recommended fix**: Derive or implement `PartialEq` and `Display`.

### m13. `ActionCategory` has future-facing variants not used at runtime
- **Severity**: Low
- **Source report**: `codex-audit-act-types-model-v2`
- **Files**: `src/actions/types/action_model.rs`
- **Description**: Some `ActionCategory` variants exist for forward compatibility but are never constructed, increasing dead code surface.
- **Recommended fix**: Consider gating behind a feature flag or removing until needed.

### m14. Global action ID uniqueness not enforced across builder functions
- **Severity**: Low
- **Source report**: `codex-audit-act-action-id-uniqueness`
- **Files**: Various `src/actions/builders/*.rs`
- **Description**: Action IDs are unique within each builder function but can overlap across different builders. No enforcement mechanism exists.
- **Recommended fix**: Namespace IDs by builder context (e.g., `file:open_directory` vs `script:open_directory`) or add a compile-time/test-time uniqueness check.

### m15. Reveal actions show success HUD before async reveal actually succeeds
- **Severity**: Low
- **Source report**: `codex-audit-act-builders-filepath`, `codex-audit-act-execution-scripts`
- **Files**: `src/app_actions/handle_action.rs` (reveal handlers)
- **Description**: The success HUD is shown synchronously before the async file reveal completes, potentially showing success even if the reveal fails.
- **Recommended fix**: Move feedback to the async completion callback.

### m16. `count_section_headers()` can overcount vs rendered section headers
- **Severity**: Low
- **Source report**: `codex-audit-act-window-part01`
- **Files**: `src/actions/window/part_01.rs` (count logic)
- **Description**: The section header count function may count more headers than are actually rendered, causing layout miscalculations.

### m17. Trash-mode command bar lacks restore/delete actions
- **Severity**: Low
- **Source report**: `codex-audit-act-builders-notes`
- **Files**: `src/actions/builders/notes.rs:46`; `src/notes/window/render_editor_titlebar.rs:138,149`
- **Description**: In trash view, restore/permanent-delete are available via titlebar buttons but absent from the command bar (Cmd+K), violating user expectations.
- **Recommended fix**: Add restore and permanent-delete actions to the trash-mode command bar.

---

## Consistency Patterns

Patterns that appeared across multiple reports:

### P1. Silent failure paths (6 reports)
Reports: `builders-filepath`, `builders-notes`, `execution-builtin`, `execution-scripts`, `window-part02-v2`, `empty-states`

Multiple handlers across the codebase follow a pattern of early-return with only a log message when operations fail. Users receive no visible feedback (no toast, HUD, or error state). This affects file operations, note actions, editor launches, AI commands, and window operations.

### P2. Label/description mismatch with runtime behavior (3 reports)
Reports: `builders-filepath`, `builders-notes`, `description-quality`

Several actions use `...` suffix labels or descriptions suggesting interactive chooser flows (e.g., "Copy Note As...", "Export...", "Open With...") but execute fixed single-path behavior at runtime.

### P3. Layout/height calculation inconsistencies (3 reports)
Reports: `dialog-part04-body01`, `dialog-part04-body03`, `window-part02-v2`

Multiple layout paths compute heights independently without sharing logic, leading to drift between dialog sizing, scrollbar math, and window resize behavior. Search-hidden and footer-enabled states are particularly problematic.

### P4. Stale or duplicate constants/comments (3 reports)
Reports: `constants`, `theme-styling`, `window-part01`

Dimension constants are duplicated between files (e.g., `ACTIONS_WINDOW_WIDTH` vs `POPUP_WIDTH`), and code comments reference old constant values that no longer match reality.

### P5. Blocked test verification (25+ reports)
Reports: Nearly all audit reports

The vast majority of audit reports could not complete scoped test verification due to pre-existing compile errors in unrelated modules. Common blockers include: `E0753` doc-comment placement errors, `E0428` duplicate module definitions, missing struct fields in clipboard history, and missing `BuiltInFeature` variants.

### P6. Missing test coverage for runtime dialog paths (3 reports)
Reports: `tests-coverage`, `dialog-behavior`, `keyboard-nav`

Key runtime behaviors lack test coverage: `submit_selected`, `submit_cancel`, `dismiss_on_click_outside`, `move_up`/`move_down`, `handle_char`/`handle_backspace`, and `should_render_section_separator`.

---

## Recommended Actions

Grouped by file/area, prioritized by severity.

### Actions Dialog (`src/actions/dialog/`)

| Priority | Action | Files |
|----------|--------|-------|
| Critical | Fix Unicode slicing panic in shortcut keycap parsing | `part_02/part_01.rs:154-155` |
| Critical | Ensure `close_actions_popup` invokes `on_close` callbacks | `src/app_impl/actions_dialog.rs:96,106,223` |
| High | Fix hidden-search height allocation in layout | `part_04/body_part_03.rs:24,27,132,134` |
| High | Include footer in `POPUP_MAX_HEIGHT` clamp | `part_04/body_part_03.rs:24,26,74,128,129` |
| High | Fix scrollbar visible-range math for header/footer | `part_04/body_part_01.rs` |
| Medium | Fix search-height divergence from visibility | `part_04/body_part_02.rs:24,470` |
| Low | Update stale dimension comments | `part_01.rs:82`, `part_04/body_part_02.rs:4,77,110,467` |
| Low | Centralize repeated padding literals | `part_04/body_part_02.rs:96`, `part_04/body_part_03.rs:49,50` |

### Command Bar (`src/actions/command_bar/`)

| Priority | Action | Files |
|----------|--------|-------|
| Critical | Fix named-key misclassification as TypeChar | `part_01.rs:88` |
| Medium | Fix duplicate `mod tests` definition | `part_02.rs:387`, `part_03.rs:109` |
| Medium | Add test coverage for TypeChar fallback path | `part_01.rs` |

### Actions Window (`src/actions/window/`)

| Priority | Action | Files |
|----------|--------|-------|
| High | Fix non-deterministic auto-close on focus loss | `part_01.rs` |
| High | Fix stale `ACTIONS_WINDOW` singleton on close | `part_01.rs:310,326` |
| Low | Unify open-path vs resize-path height for empty state | `part_02.rs:106`, `part_03.rs:38` |
| Low | Add logging for silently-ignored window operations | `part_02.rs:204,248,279` |
| Low | Use `POPUP_WIDTH` instead of duplicated `ACTIONS_WINDOW_WIDTH` | `part_01.rs:146` |

### Builders (`src/actions/builders/`)

| Priority | Action | Files |
|----------|--------|-------|
| High | Fix `open_with` behavior vs label mismatch | `file_path.rs` |
| High | Fix "Copy Note As..." / "Export..." label mismatch | `notes.rs:101,137` |
| Medium | Fix `to_deeplink_name` URL encoding gap | `shared.rs:7,10` |
| Medium | Handle empty deeplink slugs | `shared.rs:12-15` |
| Medium | Use stable IDs for chat model/preset actions | `notes.rs:177,192,203` |
| Medium | Resolve `Copy Deeplink` shortcut conflict | `notes.rs:118` |
| Low | Deduplicate scriptlet H3 action IDs | `scriptlet.rs` |
| Low | Add descriptions to new chat presets | `notes.rs:192` |
| Low | Add trash-mode command bar actions | `notes.rs:46` |

### Execution (`src/app_execute/`, `src/app_actions/`, `src/app_impl/`)

| Priority | Action | Files |
|----------|--------|-------|
| Medium | Add user-visible error feedback for AI command failures | `builtin_execution.rs:87,634,677,709,752,796` |
| Medium | Add `builtin-open-ai`/`builtin-open-notes` to `NO_MAIN_WINDOW_BUILTINS` | `execution_scripts.rs:317` |
| Medium | Add user feedback for silent note action failures | `notes_actions.rs:94,101,121,124` |
| Medium | Add error feedback for editor launch failures | `handle_action.rs` |
| Low | Move reveal success HUD to async completion | `handle_action.rs` |

### Types & Constants (`src/actions/types/`, `src/actions/constants.rs`)

| Priority | Action | Files |
|----------|--------|-------|
| Low | Add `PartialEq` and `Display` to `Action` | `action_model.rs` |
| Low | Remove or feature-gate unused `ActionCategory` variants | `action_model.rs` |
| Low | Add global action ID uniqueness enforcement | `builders/*.rs` |

### Test Infrastructure

| Priority | Action | Files |
|----------|--------|-------|
| Medium | Fix pre-existing compile errors blocking test execution | Various (`clipboard_history/*`, `app_impl/*`, test modules) |
| Medium | Add tests for runtime dialog paths (`submit_selected`, `submit_cancel`, `move_up`/`move_down`) | `src/actions/dialog/` |
| Low | Add tests for `handle_char`/`handle_backspace` in search | `src/actions/command_bar/` |
| Low | Add tests for `should_render_section_separator` | `src/actions/dialog/` |
| Low | Differentiate true-empty vs search-empty UI messages | Dialog rendering path |
| Low | Add accessibility: Tab/Shift+Tab navigation and focus-trap | `src/actions/window/` |

---

## Summary Statistics

| Severity | Count |
|----------|-------|
| Critical | 3 |
| Major (High/Medium) | 17 |
| Minor (Low) | 17 |
| **Total** | **37** |

| Area | Issues |
|------|--------|
| Actions Dialog | 8 |
| Command Bar | 3 |
| Actions Window | 5 |
| Builders | 9 |
| Execution/Handlers | 5 |
| Types & Constants | 3 |
| Test Infrastructure | 6 |
