# Confirm Modal Inventory

Goal source: `.goals/modal-consistency-design-system.md`

Scope rule: this inventory tracks confirm/deny interactions only. Actions Menu,
trigger popups, hover/dropdown menus, browse panels, and choice popups are not
modals for this goal.

## Shared Shell

| Route | Owner | Shared shell status | Verification status |
| --- | --- | --- | --- |
| Add Shortcut / shortcut recorder | `src/components/shortcut_recorder/render.rs`, `src/app_actions/handle_action/shortcuts.rs` | Uses `src/components/confirm_modal_shell.rs` with `shortcut-modal-content` route content under `modal-shell:confirm`. | Source contract added; runtime target/focus proof captured for `shortcut-recorder-popup`; DevTools only exposes `panel:prompt-popup` inside prompt popups, so shared shell marker/button internals remain source-contract proven. |
| Parent-attached confirm popup | `src/confirm/window.rs`, `src/confirm/parent_dialog.rs` | Uses `src/components/confirm_modal_shell.rs` with `confirm-modal-content` route content under `modal-shell:confirm`. | Source contract added; runtime ConfirmPrompt proof captured for panel, confirm/cancel buttons, native footer bindings, and keyboard policy. DevTools does not expose shared shell element bounds/ids. |

## Confirm Routes

| Interaction | Owner | Current route | Status |
| --- | --- | --- | --- |
| Quit Script Kit | `src/app_actions/handle_action/scripts.rs` | `open_parent_confirm_dialog_for_entity` with `quit_script_kit_confirm_options`. | Shared popup shell via `src/confirm/window.rs`; runtime proof pending. |
| Remove/delete script | `src/app_actions/handle_action/scripts.rs` | Entity-owned parent confirm helper before destructive action. | Shared popup shell via `src/confirm/window.rs`; representative runtime proof pending. |
| Move file to trash | `src/app_actions/handle_action/files.rs` | `confirm_with_parent_dialog` with destructive options. | Shared popup shell via `src/confirm/window.rs`; representative runtime proof pending. |
| Clipboard bulk delete / clear unpinned | `src/app_actions/handle_action/clipboard.rs` | `confirm_with_parent_dialog` with destructive options. | Shared popup shell via `src/confirm/window.rs`; representative runtime proof pending. |
| Built-in confirmation gate | `src/config/defaults.rs`, `src/config/types.rs`, execution/handler confirmation path | Command config requires confirmation for destructive built-ins. | Route audit pending in later iteration. |
| SDK `confirm` | `scripts/kit-sdk.ts`, `src/execute_script/mod.rs`, `src/prompt_handler/mod.rs`, `src/stdin_commands/mod.rs` | SDK sends confirm prompt data through stdin/protocol confirm handling. | Existing SDK API retained; host-route proof pending. |
| `openConfirmPrompt` stdin fixture | `src/stdin_commands/mod.rs` | Deterministic fixture for DevTools/runtime confirm proof. | Runtime proof captured in `/tmp/confirm-modal-confirm-elements.json`, `/tmp/confirm-modal-confirm-keyboard.json`, and `/tmp/confirm-modal-confirm-escape.json`. |
| `showShortcutRecorder` stdin fixture | `src/stdin_commands/mod.rs`, `src/components/shortcut_recorder/**` | Deterministic fixture for Add Shortcut-style modal proof. | Runtime proof captured in `/tmp/confirm-modal-shortcut-popup-elements.json` and `/tmp/confirm-modal-shortcut-popup-focus.json`; popup internals require source contract because DevTools exposes only the popup panel node. |
| Notes delete confirmation | `src/notes/window/notes.rs`, `src/notes/window/keyboard.rs` | Parent-id-aware confirm helper for Notes window. | Shared popup shell via `src/confirm/window.rs`; runtime proof pending. |

## Explicit Non-Modals

| Surface | Owner | Reason excluded |
| --- | --- | --- |
| Actions Menu | `src/actions/**` | Searchable contextual operations menu, not a confirm/deny modal. |
| Trigger popup | `src/app_impl/menu_syntax_trigger_popup*.rs` | Input suggestion/dropdown surface, not a confirm/deny modal. |
| Hover/dropdown menus | Various popup/dropdown owners | Menu selection surfaces, not confirm/deny interactions. |
| Browse panels | `src/notes/window/render_overlays.rs` and related owners | Panel overlays, not confirm/deny interactions. |
| Editor choice popup | `src/editor/mod.rs` | Choice completion popup, not a confirm/deny modal. |

## Iteration 1 Proof Plan

- Source contracts:
  - `./scripts/agentic/agent-cargo.sh test --test source_audits confirm_modal_shared_shell`
  - `./scripts/agentic/agent-cargo.sh test --test source_audits no_popup_confirm_callers`
  - `./scripts/agentic/agent-cargo.sh test --test confirm_prompt_surface_contract`
- Runtime proof:
  - Add Shortcut / shortcut recorder modal through a real route.
  - Quit Script Kit confirmation through a real route.
  - For each route, capture DevTools target/elements/layout/focus/keyboard receipts proving the shared shell marker, button metadata, focus owner, and key routing.

## Iteration 1 Runtime Receipts

Captured in session `confirm-modal-proof` against
`target-agent/pools/agent-debug/debug/script-kit-gpui`.

| Receipt | Status | Notes |
| --- | --- | --- |
| `/tmp/confirm-modal-shortcut-popup-elements.json` | Partial proof | Target `shortcut-recorder-popup`, semantic surface `shortcutRecorder`, focused `panel:prompt-popup`. DevTools does not expose child shell/button elements for prompt popups. |
| `/tmp/confirm-modal-shortcut-popup-focus.json` | Partial proof | Target identity and focus inspection succeeded; detailed focus node unavailable for popup internals. |
| `/tmp/confirm-modal-confirm-elements.json` | Proof | Target `main`, surface `ConfirmPrompt`, selected `button:0:quit`, visible `button:1:cancel`, native footer apply/close buttons. |
| `/tmp/confirm-modal-confirm-keyboard.json` | Proof with limitation | Keyboard policy `NoEditableKeyboard`; bindings `Enter -> Quit`, `Esc -> Cancel`; missing `nativeFooterActivationReceipt` primitive. |
| `/tmp/confirm-modal-confirm-escape.json` | Primitive gap | DevTools key dispatch reports missing `nativeFooterActivationReceipt`, so post-dismiss transition is not machine-provable with the current tool. |
