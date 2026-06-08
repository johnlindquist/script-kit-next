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
| Quit Script Kit | `src/app_execute/builtin_execution.rs` | Built-in confirmation gate opens `confirm_with_parent_dialog` with `quit_script_kit_confirm_options`. | Runtime-proven through real launcher row: `confirm-popup` parent dialog, shared confirm semantics, Quit/Cancel buttons, and parent target. Dismiss primitive gap remains; disposable session was stopped for cleanup. |
| Remove/delete script | `src/app_actions/handle_action/scripts.rs` | Entity-owned parent confirm helper before destructive action. | Source route audit proven via `destructive_confirm_routes_use_shared_parent_confirm_helpers`; representative runtime proof pending. |
| Move file to trash | `src/app_actions/handle_action/files.rs` | `confirm_with_parent_dialog` with destructive options. | Source route audit proven via `destructive_confirm_routes_use_shared_parent_confirm_helpers`; representative runtime proof pending. |
| Clipboard bulk delete / clear unpinned | `src/app_actions/handle_action/clipboard.rs` | `confirm_with_parent_dialog` with destructive options. | Source route audit proven via `destructive_confirm_routes_use_shared_parent_confirm_helpers`; representative runtime proof pending. |
| Destructive dry-run safety fixture | `scripts/agentic/scenario.ts`, `src/stdin_commands/mod.rs`, `src/app_impl/simulate_key_dispatch.rs` | `destructive-confirm-modal-safety-stress --dry-run-only` opens a deterministic `openConfirmPrompt` fixture and refuses non-dry-run execution. | Runtime-proven: prompt identity, footer buttons, Escape cancel, and no destructive flags. Receipt: `/tmp/confirm-modal-destructive-safety.json`. |
| Built-in confirmation gate | `src/config/defaults.rs`, `src/config/types.rs`, `src/app_execute/builtin_execution.rs`, `src/app_execute/builtin_confirmation.rs` | Command config requires confirmation for destructive built-ins, then routes through `confirm_with_parent_dialog` before execution. | Source route audit proven by `tests/source_audits/builtin_confirmation.rs`; representative runtime proof still pending. |
| SDK `confirm` | `scripts/kit-sdk.ts`, `src/execute_script/mod.rs`, `src/prompt_handler/mod.rs`, `src/stdin_commands/mod.rs` | SDK exposes single-word `confirm()` and sends protocol messages with `type: 'confirm'`; host route opens the shared in-window `ConfirmPrompt` through `open_confirm_prompt`. | Source guard added to prevent `modal.confirm` drift and to prevent reverting to `confirm_with_parent_dialog`; script-facing runtime proof still pending. |
| `openConfirmPrompt` stdin fixture | `src/stdin_commands/mod.rs` | Deterministic fixture for DevTools/runtime confirm proof. | Runtime proof captured in `/tmp/confirm-modal-confirm-elements.json`, `/tmp/confirm-modal-confirm-keyboard.json`, and `/tmp/confirm-modal-confirm-escape.json`. |
| `showShortcutRecorder` stdin fixture | `src/stdin_commands/mod.rs`, `src/components/shortcut_recorder/**` | Deterministic fixture for Add Shortcut-style modal proof. | Runtime proof captured in `/tmp/confirm-modal-shortcut-popup-elements.json` and `/tmp/confirm-modal-shortcut-popup-focus.json`; popup internals require source contract because DevTools exposes only the popup panel node. |
| Notes delete confirmation | `src/notes/window/notes.rs`, `src/notes/window/keyboard.rs` | Parent-id-aware confirm helper for Notes window. | Source route audit proven via `destructive_confirm_routes_use_shared_parent_confirm_helpers`; runtime proof pending. |

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

## Iteration 2 Dev Style Tool Proof

- Source contracts:
  - `./scripts/agentic/agent-cargo.sh test --test dev_style_tool_runtime_style_contract confirm_modal`
  - `./scripts/agentic/agent-cargo.sh test --test dev_style_tool_window_contract dev_style_tool`
  - `./scripts/agentic/agent-cargo.sh test --test dev_style_tool_runtime_style_contract export_current_settings_includes_agent_readable_overrides_and_effective_values`
- Runtime proof:
  - Session `confirm-modal-style-proof` launched with `SCRIPT_KIT_STYLE_DEVTOOLS=1`.
  - `/tmp/confirm-modal-style-tool-elements.json` exposed `tab:dev-style-tool:confirm-modal-styling`.
  - `/tmp/confirm-modal-style-tool-set-radius.json` applied `confirmModal.shell.radius=13`.
  - `/tmp/confirm-modal-style-tool-elements-full.json` contains the Confirm Modal shell and header group tabs and slider/input/reset controls.

## SDK Naming Proof

- Source contract:
  - `./scripts/agentic/agent-cargo.sh test --test source_audits confirm_modal_shared_shell`
- Guarded behavior:
  - `scripts/kit-sdk.ts` retains `ConfirmConfig`, global `confirm()`, and protocol `type: 'confirm'`.
  - `scripts/kit-sdk.ts` and `.goals/modal-consistency-design-system.md` must not introduce `modal.confirm`.

## Iteration 3 Destructive Dry-Run Proof

- Source contract:
  - `./scripts/agentic/agent-cargo.sh test --test source_audits confirm_modal_shared_shell`
- Runtime attempt:
  - `SCRIPT_KIT_GPUI_BINARY=/Users/johnlindquist/dev/script-kit-gpui/target-agent/pools/agent-debug/debug/script-kit-gpui bun scripts/agentic/index.ts destructive-confirm-modal-safety-stress --session confirm-modal-destructive-proof --dry-run-only --json > /tmp/confirm-modal-destructive-safety.json`
- Current result:
  - The scenario starts the app, waits for readiness, sends `show` to register the main window, opens `openConfirmPrompt`, inspects `ConfirmPrompt` state/elements, and fail-closes with explicit `destructiveCommandExecuted=false`, `systemCommandRequested=false`, and `trashMutationRequested=false`.
  - Runtime proof now passes: `stateBefore.promptType=confirmPrompt`, native footer exposes `apply` and `close`, `simulateKey Escape` restores `stateAfterCancel.promptType=none`, `noMutationAfterCancel=true`, `systemCommandRequested=false`, and `trashMutationRequested=false`.
  - Known primitive nuance: the Escape send envelope still reports `parseOutcome=timeout`, so the scenario uses post-state as the authoritative dismissal proof.

## Iteration 4 SDK Host Route And Built-In Gate Audit

- Source contracts:
  - `./scripts/agentic/agent-cargo.sh test --test source_audits confirm_modal_shared_shell`
  - `./scripts/agentic/agent-cargo.sh test --test source_audits builtin_confirmation`
- Guarded behavior:
  - SDK `confirm()` keeps the single-word API name and protocol `type: 'confirm'`.
  - The SDK host route in `src/prompt_handler/mod.rs` now opens the shared
    in-window `ConfirmPrompt` with `open_confirm_prompt` instead of the
    parent-attached `confirm_with_parent_dialog` path.
  - Built-in destructive confirmations keep the config gate
    `requires_confirmation(&entry.id)`, spawn the async confirmation flow, call
    `confirm_with_parent_dialog`, and handle accept, cancel, and error arms.
- Runtime attempt:
  - SDK runtime proof was attempted through agentic sessions after the source
    route change. Baseline state RPCs worked, but `session.sh send run` did not
    surface the expected script log, `ShowConfirm`, or `sdk-confirm-run*`
    evidence before follow-up RPCs timed out.
- Current result:
  - SDK and built-in confirm routes are source-contract proven.
  - SDK script-facing runtime proof remains pending because the external
    agentic `run` route did not produce confirm traffic during the proof run.

## Iteration 5 Quit Script Kit Runtime Proof

- User path:
  - Started disposable session `confirm-modal-quit-proof`.
  - Set launcher input to `quit script kit`.
  - Verified the selected row was the real `builtin/quit-script-kit` entry.
  - Sent low-level `simulateKey Enter` only after confirming the source
    `requires_confirmation(&entry.id)` gate for destructive built-ins.
- Runtime receipts:
  - `/tmp/confirm-modal-quit-set-input.json`
  - `/tmp/confirm-modal-quit-filtered-state.json`
  - `/tmp/confirm-modal-quit-sim-enter.json`
  - `/tmp/confirm-modal-quit-targets-after-enter.json`
  - `/tmp/confirm-modal-quit-elements.json`
  - `/tmp/confirm-modal-quit-focus.json`
  - `/tmp/confirm-modal-quit-keyboard.json`
  - `/tmp/confirm-modal-quit-layout.json`
  - `/tmp/confirm-modal-quit-targets-after-escape.json`
- Proven behavior:
  - `targets.list` exposed `confirm-popup` as a `PromptPopup`, title
    `Quit Script Kit`, semantic surface `confirmDialog`, parent
    `main`, bounds `360x132`.
  - Popup semantic inspection exposed `panel:confirm-dialog`,
    `button:0:confirm` text `Quit`, and `button:1:cancel` text `Cancel`.
  - Focus inspection reported `focusedSemanticId=button:0:confirm`.
  - Layout measurement resolved the popup window region and produced no
    overlap errors.
- Known primitive gaps:
  - `triggerBuiltin` cannot open `builtin/quit-script-kit`; the registry
    omits this destructive command, so the proof used the real launcher row
    instead.
  - `keyboard.inspect` for `PromptPopup` reports
    `blocked-by-missing-primitive` for `keyboardBindings`.
  - Target-scoped `simulateKey Escape` returned ok but did not dismiss the
    parent confirm popup. Cleanup used `session.sh stop` instead of attempting
    an Enter-based cancel path.

## Iteration 6 Remaining Confirm Route Source Audit

- Source contract:
  - `./scripts/agentic/agent-cargo.sh test --test source_audits confirm_modal_shared_shell`
- Guarded behavior:
  - Script removal and legacy quit action confirmations use
    `open_parent_confirm_dialog_for_entity`.
  - File move-to-trash and clipboard bulk delete/clear-unpinned confirmations
    use `ParentConfirmOptions::destructive` and `confirm_with_parent_dialog`.
  - Notes delete confirmation uses
    `open_parent_confirm_dialog_for_automation_parent("notes", ...)` and
    restores primary focus on cancel.
- Current result:
  - Remaining route owners are source-contract proven against the shared parent
    confirm helper path.
  - Runtime representatives remain pending for remove/delete script, file
    trash, clipboard bulk delete/clear-unpinned, and Notes delete.
