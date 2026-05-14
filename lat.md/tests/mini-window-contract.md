---
lat:
  require-code-mention: true
---
# Mini Window Contract Tests

These specs bind Mini versus Full sizing, actions, telemetry, and hide/reset behavior to focused source audits and runtime probes.

## Mini AI sizing

Inline Mini AI must size through `compact_ai_view_type_for_mode` so Mini mode uses `MiniAiChat` width and Full mode uses `DivPrompt` width.

The source audit lives in [[tests/mini_window_sizing_contract.rs]]. Runtime proof lives in [[scripts/agentic/mini-ai-sizing.ts]] and asserts Mini and Full window bounds from automation receipts.

## MiniPrompt sizing

MiniPrompt must use `ViewType::MiniPrompt` instead of ArgPrompt view types so compact prompt chrome cannot inherit Full prompt width.

The source audit lives in [[tests/mini_window_sizing_contract.rs]] and checks both `calculate_window_size_params` and the `window_resize` enum.

## Chat and ACP mode sizing

ChatPrompt and AcpChatView sizing must branch on `main_window_mode` so Mini and Full modes cannot share one raw DivPrompt resize path.

The source audit lives in [[tests/mini_window_sizing_contract.rs]] and checks both AppView sizing arms.

## Mini AI actions

Mini AI actions must cross a typed parent-window request channel and dispatch through `dispatch_actions_toggle_for_current_view`.

The source audit lives in [[tests/mini_ai_actions_contract.rs]]. Runtime proof lives in [[scripts/agentic/mini-ai-actions-dismiss.ts]].

## Mini AI actions receiver

Mini AI action requests must be received by the main render loop while a real GPUI window handle is available.

The source audit lives in [[tests/mini_ai_actions_contract.rs]] and checks the receiver calls the shared dispatcher.

## Mini AI snapshot and close telemetry

Mini AI close must emit a typed snapshot and `getState.miniAi` must expose close, draft, handoff, and return-origin fields.

The source audit lives in [[tests/mini_ai_snapshot_contract.rs]] and checks the snapshot struct fields.

## Mini AI close telemetry

Mini AI close must log a close request and a live close snapshot before returning to the launcher.

The source audit lives in [[tests/mini_ai_snapshot_contract.rs]]. Runtime proof lives in [[scripts/agentic/mini-ai-close-telemetry.ts]].

## Mini AI getState snapshot

`getState.miniAi` must expose visibility, prompt id, mode, draft state, handoff source, return origin, and last close source.

The source audit lives in [[tests/mini_ai_snapshot_contract.rs]]. Runtime proof lives in [[scripts/agentic/mini-ai-handoff-return-origin.ts]].

## Mini mode toggles

Mini and Full mode changes must route through mode helpers that update ChatPrompt render mode, size, popup state, and native footer ownership together.

The source audit lives in [[tests/mini_mode_toggle_contract.rs]]. Runtime proof lives in [[scripts/agentic/mini-full-rapid-toggle.ts]].

## Mini mode caller routing

Mode-changing callers must use the shared mode helpers instead of directly assigning `main_window_mode`.

The source audit lives in [[tests/mini_mode_toggle_contract.rs]] and covers launcher, reset, and inline handoff callers.

## Mini resize width clamp

Width restoration must clamp to `MiniMainWindow` while the app is in Mini mode.

The source audit lives in [[tests/mini_mode_toggle_contract.rs]] and checks `resize_current_view_to_width`.

## Mini close reset

Hide/reset must snapshot Mini mode before `reset_to_script_list` and reset hidden bounds when either pre-reset or post-reset state is Mini.

The source audit lives in [[tests/mini_close_reset_contract.rs]]. Runtime proof lives in [[scripts/agentic/mini-close-reset.ts]].

## Mini popup dismiss parity

A non-Actions footer click must close shared and detached actions popups without dispatching the clicked footer action in both Mini and Full modes.

The source audit lives in [[tests/mini_popup_dismiss_parity_contract.rs]]. Runtime proof lives in [[scripts/agentic/mini-popup-dismiss-parity.ts]].
