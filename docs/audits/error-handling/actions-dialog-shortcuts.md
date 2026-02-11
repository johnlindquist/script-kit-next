# ActionsDialog Keyboard Routing + Shortcut Audit

## Scope
- Core dialog + selection state: `src/actions/dialog.rs`
- Main modal key routing: `src/app_impl/actions_dialog.rs`
- Path prompt key routing + path action execution: `src/render_prompts/path.rs`
- Separate actions window fallback routing: `src/actions/window.rs`
- Shortcut conversion/formatting: `src/shortcuts/hotkey_compat.rs`, `src/actions/builders/shared.rs`, `src/actions/builders/file_path.rs`

## Flow Summary
1. Path actions are opened via `ActionsDialog::with_path(...)` (`src/actions/dialog.rs:422`) from `handle_show_path_actions` (`src/render_prompts/path.rs:138`).
2. The dialog starts with `selected_index = initial_selection_index(...)` (`src/actions/dialog.rs:395`), where empty rows fall back to index `0` (`src/actions/dialog.rs:132`).
3. Selection resolves through `grouped_items -> filtered_actions -> actions` (`src/actions/dialog.rs:1087`, `src/actions/dialog.rs:1105`, `src/actions/dialog.rs:865`).
4. Execution routes call `get_selected_action_id()` (`src/actions/dialog.rs:1264`) and `selected_action_should_close()` (`src/actions/dialog.rs:1282`):
   - Generic route: `route_key_to_actions_dialog` (`src/app_impl/actions_dialog.rs:4`)
   - Path route: outer handler in `render_path_prompt` (`src/render_prompts/path.rs:273`)
   - Actions window route: `ActionsWindow` key handler (`src/actions/window.rs:345`)

## Current User-Visible Behavior (Failure Inventory)
| ID | Location | Trigger | User-visible behavior now | Why this is risky |
|---|---|---|---|---|
| `ACT-DLG-EMPTY-001` | `src/actions/builders/file_path.rs:167`, `src/actions/dialog.rs:1550`, `src/app_impl/actions_dialog.rs:83`, `src/render_prompts/path.rs:291` | Path context is invalid, so `get_path_context_actions` returns empty actions | Popup opens with `No actions available`; pressing Enter does nothing | Enter no-op has no inline explanation or toast; looks like keyboard is broken |
| `ACT-DLG-SEL-001` | `src/actions/dialog.rs:132`, `src/actions/dialog.rs:1105`, `src/actions/dialog.rs:1264`, `src/app_impl/actions_dialog.rs:83`, `src/render_prompts/path.rs:291`, `src/actions/window.rs:422` | Selection index points to header/out-of-bounds (or no selectable rows) | `get_selected_action_id()` returns `None`; Enter does nothing in all routes | Invalid selection and empty selection are indistinguishable from "nothing happened" |
| `ACT-DLG-SHORTCUT-001` | `src/actions/builders/shared.rs:49`, `src/app_impl/actions_dialog.rs:169`, `src/shortcuts/hotkey_compat.rs:164` | Shortcut display glyphs are not fully normalized back to keystroke format | Some visible shortcut hints do not trigger actions | Shortcut appears valid in UI but matching fails silently |
| `ACT-DLG-SHORTCUT-002` | `src/app_impl/actions_dialog.rs:110`, `src/app_impl/actions_dialog.rs:130`, `src/render_prompts/path.rs:320`, `src/render_prompts/path.rs:342`, `src/actions/window.rs:133`, `src/actions/window.rs:435`, `src/actions/builders/file_path.rs:227` | Backspace/delete is handled as search edit before shortcut matching | Shortcut-bound actions on backspace/delete keys are hard or impossible to trigger from keyboard routing path | User sees key edit search, not action dispatch; retrying same shortcut keeps failing |
| `ACT-DLG-SHORTCUT-003` | `src/app_impl/actions_dialog.rs:130`, `src/app_impl/actions_dialog.rs:136`, `src/render_prompts/path.rs:344`, `src/render_prompts/path.rs:349` | Matching iterates full `actions` list, not visible filtered rows | Hidden actions can still be triggered by shortcut while filtered out | Behavior is surprising; may execute unexpected action in filtered context |
| `ACT-DLG-SWALLOW-001` | `src/app_impl/actions_dialog.rs:164`, `src/app_impl/startup_new_actions.rs:227` | Popup is open and key does not match movement/enter/escape/search/shortcut | Key is swallowed by modal route with no feedback | Shortcut normalization failures look identical to random ignored keys |
| `ACT-DLG-STUCK-001` | `src/app_impl/actions_dialog.rs:19`, `src/app_impl/actions_dialog.rs:20`, `src/render_prompts/path.rs:274`, `src/render_prompts/path.rs:281` | `show_actions_popup == true` while `actions_dialog == None` | Generic route swallows keys with no log; path route logs warning and returns | Inconsistent state can look like frozen popup; recovery path is not explicit in UI |
| `ACT-DLG-CLOSE-ORDER-001` | `src/app_impl/actions_dialog.rs:96`, `src/app_impl/actions_dialog.rs:100`, `src/render_prompts/path.rs:307`, `src/render_prompts/path.rs:313`, `src/render_prompts/path.rs:371`, `src/render_prompts/path.rs:375` | Dialog closes before action execution/validation completes | Popup disappears even if action cannot run or later fails | Failure can look like successful completion and forces user to reopen/rebuild context |

## Shortcut Normalization Mismatch Details
- Formatter emits display glyphs for keys (for example, space and arrow keys) in `format_shortcut_hint` (`src/actions/builders/shared.rs:49`).
- Runtime matching uses `keystroke_to_shortcut(key, modifiers)` (`src/shortcuts/hotkey_compat.rs:164`) and compares against `normalize_display_shortcut(...)` (`src/app_impl/actions_dialog.rs:169`).
- `normalize_display_shortcut` only explicitly remaps a subset of glyph keys; others are lowered as raw glyph characters, which do not match `keystroke_to_shortcut` token format.
- Result: some shortcuts can be displayed correctly but never match at runtime.

## Failure Modes That Look Like Success
| ID | Why it looks successful | Actual behavior |
|---|---|---|
| `ACT-DLG-CLOSE-ORDER-001` | Dialog closes immediately on Enter/shortcut | Action may not execute (missing path info or downstream failure) after close |
| `ACT-DLG-SHORTCUT-002` | Key press has visible effect (search text changes) | Intended shortcut action never fires |
| `ACT-DLG-SHORTCUT-003` | Shortcut "works" and closes popup | Executed action may not be the one currently visible in filtered list |

## Failure Modes With Poor Retryability
| ID | Why immediate retry is hard |
|---|---|
| `ACT-DLG-CLOSE-ORDER-001` | User must reopen actions popup and rebuild search/selection context after close-before-failure |
| `ACT-DLG-STUCK-001` | No inline recovery hint; generic path swallows keys without explaining state mismatch |
| `ACT-DLG-SHORTCUT-001` | Repeating same visible shortcut does not change outcome because normalization mismatch is deterministic |
| `ACT-DLG-SHORTCUT-002` | Repeating backspace/delete shortcut repeats search edit path, not action dispatch path |

## Testing Coverage Notes
- `src/actions/dialog.rs` includes tests for empty-state copy and unrelated rendering helpers, but there is no direct test for invalid selection no-op behavior in keyboard routing.
- `src/app_impl/actions_dialog.rs` currently has a close-order regression test (`src/app_impl/actions_dialog.rs:282`) but no targeted tests for shortcut normalization edge cases or modal swallow behavior.
