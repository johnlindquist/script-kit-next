# Confirm Modal Inventory

Goal source: `.goals/fix-quit-add-shortcut-modal-parity.md`

Scope rule: this inventory tracks true confirm/deny interactions only. Actions
Menu, trigger popups, hover/dropdown menus, browse panels, and choice popups are
not modals for this goal. The actions popup is only the native background/window
reference for confirm popups.

## Shared Modal System

| Surface | Owner | Shared status | Guard/proof status |
| --- | --- | --- | --- |
| Add Shortcut / shortcut recorder | `src/components/shortcut_recorder/render.rs`, `src/components/confirm_modal_shell.rs` | Uses `confirm_modal_shell` and `confirm_modal_header` with `shortcut-modal-content` under `modal-shell:confirm`. | Source-guarded with `shortcut_recorder_and_confirm_popup_use_the_same_shell`; runtime receipt captured for `shortcut-recorder-popup` target/focus. DevTools currently exposes only `panel:prompt-popup` internals for this popup, so shell details are source-guarded. |
| Parent-attached confirm popup | `src/confirm/window.rs`, `src/confirm/parent_dialog.rs`, `src/platform/secondary_window_config.rs` | Uses `confirm_modal_shell` and `confirm_modal_header` with `confirm-modal-content` under `modal-shell:confirm`; window kind/config matches actions popup path. | Source-guarded for shared shell, footer-derived button/keycap rendering, no red/danger shell, no local mini-buttons, and native background delegation to `configure_actions_popup_window`. |
| SDK in-window confirm prompt | `src/prompt_handler/mod.rs`, `src/stdin_commands/mod.rs`, `scripts/kit-sdk.ts` | SDK `confirm()` routes to the shared in-window `ConfirmPrompt` surface through `open_confirm_prompt`. | Source-guarded for single-word `confirm()` API, protocol `type: 'confirm'`, and no `modal.confirm` namespace. Script-facing runtime proof remains blocked by the broader app `run`/stdout-reader path, not by modal styling. |

## Confirm Routes

| Interaction | Owner | Route | Status |
| --- | --- | --- | --- |
| Quit Script Kit | `src/app_execute/builtin_execution.rs`, `src/app_execute/builtin_confirmation.rs` | Built-in confirmation gate opens `confirm_with_parent_dialog` with `quit_script_kit_confirm_options`. | Runtime-proven via real launcher row: `confirm-popup`, `PromptPopup`, semantic surface `confirmDialog`, title `Quit Script Kit`, parent `main`, buttons `Quit`/`Cancel`, focused `button:0:confirm`. |
| Remove/delete script | `src/app_actions/handle_action/scripts.rs` | `open_parent_confirm_dialog_for_entity` before destructive remove. | Source-guarded by `destructive_confirm_routes_use_shared_parent_confirm_helpers`. |
| Move file to trash | `src/app_actions/handle_action/files.rs` | `confirm_with_parent_dialog` with `ParentConfirmOptions::destructive`. | Source-guarded by `destructive_confirm_routes_use_shared_parent_confirm_helpers`; runtime route proof intentionally not pursued in this goal because source guard covers shared modal ownership and the File Search proof path was unrelated tool churn. |
| Clipboard bulk delete / clear unpinned | `src/app_actions/handle_action/clipboard.rs` | `confirm_with_parent_dialog` with `ParentConfirmOptions::destructive`. | Source-guarded by `destructive_confirm_routes_use_shared_parent_confirm_helpers`. |
| Notes delete confirmation | `src/notes/window/notes.rs`, `src/notes/window/keyboard.rs` | `open_parent_confirm_dialog_for_automation_parent("notes", ...)`. | Runtime-proven through real Notes `Cmd+Shift+Backspace` route: `confirm-popup`, parent `notes`, semantic surface `confirmDialog`, title `Move note to Trash`, buttons `Delete`/`Cancel`, cancel closes popup and sandbox DB remains unchanged. Receipt: `/tmp/confirm-modal-notes-delete-route-proof.json`. |
| SDK `confirm` | `scripts/kit-sdk.ts`, `src/prompt_handler/mod.rs`, `src/stdin_commands/mod.rs` | Global `confirm()` sends protocol `type: 'confirm'`; host opens shared `ConfirmPrompt`. | Source-guarded; runtime proof pending on non-modal app-run plumbing. |
| `openConfirmPrompt` stdin fixture | `src/stdin_commands/mod.rs`, `src/app_impl/simulate_key_dispatch.rs` | Deterministic fixture opens shared in-window `ConfirmPrompt`. | Runtime receipts captured for elements, keyboard policy, Escape cancel, and no destructive mutation in dry-run fixture. |
| `showShortcutRecorder` stdin fixture | `src/stdin_commands/mod.rs`, `src/components/shortcut_recorder/**` | Deterministic fixture opens Add Shortcut-style shared modal. | Runtime receipts captured for target/focus; popup internals remain source-guarded because DevTools does not expose child shell/button elements for this popup. |

## Explicit Non-Modals

| Surface | Owner | Reason excluded |
| --- | --- | --- |
| Actions Menu | `src/actions/**` | Searchable contextual operations menu, not a confirm/deny modal. Used only as the background/window reference. |
| Trigger popup | `src/app_impl/menu_syntax_trigger_popup*.rs` | Input suggestion/dropdown surface, not a confirm/deny modal. |
| Hover/dropdown menus | Various popup/dropdown owners | Menu selection surfaces, not confirm/deny interactions. |
| Browse panels | `src/notes/window/render_overlays.rs` and related owners | Panel overlays, not confirm/deny interactions. |
| Editor choice popup | `src/editor/mod.rs` | Choice completion popup, not a confirm/deny modal. |

## Anti-Drift Contract

- All true confirm/deny popup routes must use the shared confirm modal shell or
  the shared in-window `ConfirmPrompt` wrapper.
- Confirm popup windows must stay on the same GPUI `WindowKind::PopUp` and
  native background/config path as the actions popup.
- Confirm popup actions must reuse footer button/keycap rendering through
  `render_footer_hint_button_like`, footer slot widths, footer height, footer
  gap, hover/active tokens, and `Esc` / `↵` keycap treatment.
- Destructive meaning is semantic only. It must not reintroduce a red shell,
  different native background, smaller bespoke buttons, local button constants,
  or `Button::new(...)` modal action rows.
- `tests/source_audits/confirm_modal_shared_shell.rs` is the focused guard that
  fails on shell, background, footer-button, SDK naming, and route-owner drift.
