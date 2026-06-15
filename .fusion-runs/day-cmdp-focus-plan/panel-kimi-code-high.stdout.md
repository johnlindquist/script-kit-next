• I'll investigate the relevant source files to verify the contract and surface edge cases, then produce the plan under the requested headings.## Role Findings

  As the edge-case-tester panel, the boundary cases that stand out are **key-event routing drift**, **stale popup-state reconciliation**, **focus-restore target mismatch**, and **synthetic-key regression**. The Day Page Cmd+P popup is open but not usable because `src/main_sections/day_page_switcher.rs` hand-rolls a key router that is missing several pieces the Notes window already uses. The related Actions-menu Escape bug is a separate focus-restoration mismatch in `src/app_impl/actions_dialog.rs`.

  Key findings:

  1. **Day Page Cmd+P key routing is a narrow, incomplete re-implementation of the Notes router.**  
     `handle_day_switcher_key` maps keys manually instead of using the same `command_bar_key_intent` mapping that `CommandBarHost::handle_command_bar_key` and the Notes window use.

  2. **The router swallows keys it does not actually handle.**  
     It returns `true` for every keystroke while the switcher is open, including modifiers and navigation keys it never processes.

  3. **No stale-state reconciliation.**  
     Unlike Notes (`self.note_switcher.reconcile_open_state()`), Day Page never checks whether the detached popup was dismissed externally (click outside, focus loss). This can leave `is_open()` true and block subsequent Cmd+P.

  4. **Printable-character input uses `key` instead of `key_char`.**  
     This misses composed characters, dead-key input, and non-ASCII typing paths that `ui_foundation::printable_char(key_char)` covers.

  5. **Missing navigation/shortcut coverage.**  
     Home/End/PageUp/PageDown and filtered row shortcuts (e.g., note switcher row shortcuts) are not implemented, even though the underlying `CommandBar` supports them.

  6. **Focus restoration bypasses the app-wide coordinator.**  
     `wire_note_switcher_activation.on_close` calls `focus_editor` directly from a deferred callback, which races with `ScriptListApp` focus management and can leave focus on the app root or main filter.

  7. **Day Page Actions Escape restores focus to the wrong target.**  
     `request_focus_restore_for_actions_host` maps `ActionsDialogHost::MainList` to `FocusRequest::main_filter()`, so closing the shared Actions dialog from Day Page focuses the launcher filter instead of the Day Page editor.

  8. **A Day Page focus target already exists implicitly.**  
     `apply_pending_focus` already special-cases `AppView::DayPage` inside the `FocusTarget::EditorPrompt` arm, but there is no explicit `FocusTarget::DayPage` variant.

  ## Evidence And Assumptions

  Evidence from current source:

  - `src/main_sections/day_page_switcher.rs:306-370` — `handle_day_switcher_key` manually maps Escape, Cmd+P, up/down, Enter, Backspace, Option+Backspace, Cmd+V, and single-character typing, then returns `true` unconditionally.
  - `src/main_sections/day_page_switcher.rs:160-205` — `wire_note_switcher_activation` sets `on_close` to call `focus_editor` directly.
  - `src/notes/window/keyboard.rs:307-308` and `467-530` — Notes calls `reconcile_open_state()` and routes through `printable_char(key_char)`, `is_key_up`/`is_key_down`, and row-shortcut matching.
  - `src/app_impl/actions_dialog.rs:1190-1217` — `request_focus_restore_for_actions_host` maps `MainList` to `FocusRequest::main_filter()` with no Day Page exception.
  - `src/app_impl/theme_focus.rs:307-325` — `FocusTarget::EditorPrompt` already dispatches to `DayPageView::focus_editor`.
  - `src/main_sections/render_impl.rs:280` — `apply_pending_focus(window, cx)` runs for every render, including Day Page.

  Assumptions:

  - The desired behavior is **Notes-parity**: same search/filter UI, same up/down/Escape/typing behavior, and same focus restoration to the host editor.
  - Day Page selections (notes and past days) still open **locally** in the Day Page editor, as stated in `GLOSSARY.md`.
  - The fix must be narrowly scoped because the tree is dirty.
  - Cargo verification must use `./scripts/agentic/agent-cargo.sh`.
  - Runtime proof will use `script-kit-devtools`.

  ## Failure Modes

  | Failure | Likely Trigger | Current Code Path |
  |---|---|---|
  | Typing in Cmd+P does nothing or produces wrong characters | `key` string does not carry the composed character; `key.chars().count() == 1` rejects some inputs | `day_page_switcher.rs:354-365` |
  | Up/Down arrows do nothing | Variant mismatch (`"arrowdown"` vs `"down"` vs GPUI's internal naming) or keys swallowed by unconditional `true` return | `day_page_switcher.rs:328-335` |
  | Escape does not close switcher | Key event not routed because popup is key window and Day Page root handler is not reached; or `is_open()` stale after external dismissal | `day_page_switcher.rs:316-323` |
  | Focus does not return to Day view after Cmd+P | `focus_editor` deferred callback races with main-window focus coordinator | `day_page_switcher.rs:176-179` |
  | Focus does not return to Day view after Actions Escape | `MainList` host maps to `main_filter` instead of Day Page editor | `app_impl/actions_dialog.rs:1203-1214` |
  | Click outside / focus loss leaves switcher state stuck | No `reconcile_open_state()` call | missing in Day Page |
  | SimulateKey / automation regressions | `handle_key_parts` signature change breaks `simulate_key_dispatch.rs` | `app_impl/simulate_key_dispatch.rs:2072` |
  | Future key-routing drift | Any fix that duplicates `command_bar_key_intent` instead of sharing it | design risk |

  ## Recommendation

  ### 1. Source-owner functions to patch

  | File | Function(s) | Why |
  |---|---|---|
  | `src/main_sections/day_page_switcher.rs` | `handle_day_switcher_key`, `wire_note_switcher_activation`, `open_note_switcher`, `close_day_switcher` | Key routing and focus restoration owners |
  | `src/main_sections/day_page_view.rs` | `handle_key_down`, `handle_key_parts` | Pass `key_char` into the router; keep Cmd+P open path |
  | `src/app_impl/actions_dialog.rs` | `request_focus_restore_for_actions_host` | Fix Day Page Actions-menu focus restore |
  | `src/app_impl/theme_focus.rs` | `apply_pending_focus`, `sync_coordinator_to_legacy` | Add explicit Day Page target handling |
  | `src/focus_coordinator/mod.rs` | `FocusTarget`, `FocusRequest` | Add `DayPage` variant (recommended) |
  | `src/main_sections/app_view_state.rs` | `FocusTarget` | Add `DayPage` variant (recommended) |
  | `src/app_impl/simulate_key_dispatch.rs` | Day Page `handle_key_parts` call site | Update if signature adds `key_char` |

  ### 2. Reuse `CommandBar` routing more directly, but keep a thin local wrapper

  **Recommendation:** Do **not** implement the `CommandBarHost` trait for `DayPageView` — that trait is designed for a view whose primary command bar is `CommandBar`, and Day Page's switcher is a secondary popup. Instead:

  1. **Make `command_bar_key_intent` shared.** Promote `command_bar_key_intent` in `src/actions/command_bar.rs` from module-private to `pub(crate)` (or add a thin `pub(crate) fn note_switcher_key_intent` that wraps it). Day Page and Notes can then share the exact key-to-intent mapping, eliminating future drift.
  2. **Rewrite `handle_day_switcher_key`** to mirror the Notes `note_switcher.is_open()` branch:
     - Call `self.note_switcher.reconcile_open_state()` at entry; on stale state, focus the Day Page root and notify.
     - Dispatch through the shared intent function.
     - Use `ui_foundation::printable_char(key_char)` for typing.
     - Add Home/End/PageUp/PageDown.
     - Add filtered row-shortcut matching via `matching_filtered_action_id_for_keystroke`.
     - Return `false` for genuinely unhandled keys.

  **Why:** Notes already proves this pattern works. Reusing the intent function gives us correct behavior for printable chars, arrows, Escape, Enter, Backspace, Option+Backspace, page jumps, and row shortcuts without duplicating logic.

  ### 3. Focus-restoration path

  For both popups, route focus through the app's pending-focus mechanism rather than calling `focus_editor` directly from a deferred callback.

  **Day Cmd+P close:**
  - In `wire_note_switcher_activation.on_close`, after marking the switcher closed externally, set the app's pending focus target:
    ```rust
    if let Some(app) = view.app.upgrade() {
        app.update(cx, |app, _cx| {
            app.pending_focus = Some(FocusTarget::DayPage); // or EditorPrompt if reusing
        });
    }
    view.focus_editor(window, cx); // kept as a defensive synchronous call
    ```
  - `render_impl.rs` will apply `pending_focus` on the next frame through `apply_pending_focus`, which already knows how to focus the Day Page editor.

  **Day Actions Escape:**
  - In `request_focus_restore_for_actions_host`, detect `AppView::DayPage`:
    ```rust
    if matches!(self.current_view, AppView::DayPage { .. }) {
        self.focus_coordinator.request(FocusRequest::day_page());
        self.sync_coordinator_to_legacy();
        return;
    }
    ```
  - Add `FocusTarget::DayPage` / `FocusRequest::day_page()` to the coordinator and legacy enum, and handle it in `apply_pending_focus` and `sync_coordinator_to_legacy`.
  - If minimizing file count, reuse `FocusTarget::EditorPrompt` as a narrowly scoped fallback because `apply_pending_focus` already special-cases Day Page there.

  ### 4. Small behavior / source tests or runtime proof steps

  1. **Unit test key routing** in `src/main_sections/day_page_switcher.rs` (or a new test module):
     - Printable char filters the switcher.
     - Up/down change selection.
     - Escape closes the switcher.
     - Enter executes the selected action.
     - Option+Backspace deletes a word.
     - Unhandled keys return `false`.

  2. **Source-audit test** (only if no higher rung works) asserting:
     - `handle_day_switcher_key` calls `reconcile_open_state`.
     - It routes typing through `printable_char` or the shared intent function.
     - It does not unconditionally return `true`.

  3. **Build + unit tests:**
     ```bash
     ./scripts/agentic/agent-cargo.sh test --lib main_sections::day_page_switcher
     ./scripts/agentic/agent-cargo.sh test --lib actions::command_bar
     ./scripts/agentic/agent-cargo.sh check --lib
     ```

  4. **Runtime proof via script-kit-devtools:**
     - Open main window → Day Page.
     - Press Cmd+P; verify switcher opens with note/day rows.
     - Type a query; verify list filters.
     - Press Down; verify selection moves.
     - Press Escape; verify switcher closes and the caret is in the Day Page editor.
     - Press Cmd+K; verify Day Page Actions opens.
     - Press Escape; verify Actions closes and the caret is back in the Day Page editor (not the main filter).

  5. **Regression check:** run SimulateKey tests/automation for Day Page to ensure the `handle_key_parts` signature change does not break automation.

  ### 5. Risks in the current design

  1. **Focus race if both direct and pending paths run.**  
     Decide on one authoritative path. Recommendation: keep the direct `focus_editor` call in `on_close` for immediate feedback, but set `pending_focus` so the next render corrects any race.

  2. **`command_bar_key_intent` visibility.**  
     Making it `pub(crate)` slightly widens the API surface. The alternative is duplicating the intent logic in Day Page, which guarantees future drift. Sharing is the lower-risk choice.

  3. **External dismissal without `on_close`.**  
     If the popup closes via system gesture and `on_close` is not invoked, `reconcile_open_state()` is the only recovery. Ensure it is called on every key event while the switcher is supposed to be open.

  4. **Signature change for `handle_key_parts`.**  
     Adding `key_char` changes the call site in `simulate_key_dispatch.rs`. This is a one-line update but must not be forgotten.

  5. **Adding `FocusTarget::DayPage` touches coordinator + legacy + apply paths.**  
     If the dirty tree has uncommitted focus work, this could conflict. As a fallback, reuse `FocusTarget::EditorPrompt` for the narrowest scoped fix.

  6. **Row shortcuts in the note switcher.**  
     The shared switcher may expose note-specific shortcuts (e.g., delete). Day Page must handle or safely ignore them; row-shortcut matching in the router will execute whatever the action id resolves to via `execute_note_switcher_action`.

  ## Self Score

  **8 / 10**

  The analysis is grounded in the actual source, identifies the root cause of every reported symptom, and proposes a concrete, narrowly scoped implementation path. I did not run `agent-cargo.sh` or `script-kit-devtools` myself, so the plan still carries the normal "unverified until built and tested" risk; the requested verification steps are included so that risk is bounded.

