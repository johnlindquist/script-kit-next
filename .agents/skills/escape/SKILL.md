---
name: escape
description: >-
  Escape key UX across Script Kit GPUI: close vs back, direct launch vs launcher return, prompt cancellation, actions/confirm popups, ACP streaming/popups, simulateKey parity, dismiss policy, and automation surface reset.
---

# Escape

This skill owns Escape-key behavior across Script Kit GPUI so dismissal, cancellation, back navigation, and window closing feel predictable from every entry path.

## Use When

Use this skill for tasks involving:

- Escape key behavior, close-vs-back decisions, prompt cancellation, popup dismissal, direct shortcut launches, launcher-return behavior, or stale main-menu return state.
- Physical Escape and stdin `simulateKey` parity.
- Escape interactions with actions dialog, confirm popup, shortcut recorder, ACP chat, built-in filterable surfaces, prompt surfaces, notes, terminal, or secondary windows.
- Automation state after Escape, especially `semanticSurface`, `getState`, `getElements`, and `listAutomationWindows`.

Do not use this as the primary owner for non-Escape keyboard work; use `$keyboard-focus-routing` unless Escape-specific UX is the risk.

## First Reads

Start with these sources before editing:

- `.agents/subagents/escape-reader.md` for broad or high-risk investigation.
- `.agents/skills/escape/references/escape-key-map.md` when tracing multiple Escape paths.

## Owned Paths and Concepts

Primary paths and concepts:

- `src/app_impl/lifecycle_reset.rs`, `src/main_sections/window_visibility.rs`, `src/main_sections/app_state.rs`, `src/main_sections/app_view_state.rs`
- `src/app_impl/startup.rs`, `src/app_impl/startup_new_actions.rs`, `src/main_entry/runtime_stdin_match_simulate_key.rs`, `src/main_entry/app_run_setup.rs`
- Escape launch-origin state: launcher-entered surfaces may return to ScriptList; direct shortcut/deeplink/stdin surfaces should close/reset unless a surface owns a stronger return target.
- Popup-first routing: actions, confirm, shortcut recorder, ACP picker/model/attach popups, and prompt child popups consume Escape before parent surfaces.
- Prompt cancellation and ACP streaming cancellation before window close.
- Automation reset/re-key after Escape close or hide.

## Core Rules

- Treat Escape as layered ownership: child popup, active prompt/surface, launch origin, then window lifecycle.
- Do not solve entry-origin behavior by changing `DismissPolicy`; it is per surface, while shortcut-vs-launcher behavior is per entry path.
- Keep physical Escape and `simulateKey` behavior aligned unless a protocol contract documents the difference.
- Direct entry paths must clear stale launcher-return state before executing a command. Launcher selection paths must mark return-to-main only when the user actually came from ScriptList.
- Closing/resetting the main window must leave automation in a true ScriptList state, including `semanticSurface = scriptList`.
- Preserve established local contracts: actions dialog Escape is filter-agnostic, route-stack Escape restores parent route state, ACP Escape cancels streaming before close, and main-menu Escape keeps its own filter/hide behavior.

## Workflow

1. Review `AGENTS.md`, the owning skill, and current source context before editing.
2. Identify which layer owns the Escape press: popup, prompt, surface, launch origin, window lifecycle, or OS/secondary window.
3. Trace both physical key routing and stdin `simulateKey` for the same surface.
4. Check source-audit contracts before changing lifecycle, surface registry, ACP, actions popup, or automation re-key paths.
6. Verify with the smallest proof that would fail if Escape regressed.

## Proof Ladder

Use the smallest proof that can falsify the Escape behavior.

2. Targeted source tests: Escape contracts such as app view policy, hide/reset, actions dialog Escape, ACP shortcut, or stdin simulateKey audits.
3. State-first runtime proof: physical or simulated Escape from the real entry path, followed by `getState`, `getElements`, or `listAutomationWindows`.
4. Native input proof: only when OS focus, global hotkeys, AppKit routing, or actual hardware delivery is the behavior under test.

Default check for this skill:

```bash
cargo test --test app_view_policy_contract
cargo test --test hide_rpc_surface_reset_contract
cargo test --test actions_dialog_escape_filter_agnostic_contract
```

## Adjacent Skills

- `$keyboard-focus-routing` for non-Escape key intent routing and focus restoration.
- `$actions-popups` for Cmd+K actions dialog, attached popup routing, and route-stack behavior.
- `$acp-chat-core` for ACP streaming, embedded/detached chat, and ACP return-origin decisions.
- `$prompt-runtime` for prompt-specific cancellation, dirty state, child popups, and SDK prompt response semantics.
- `$protocol-automation` for stdin `simulateKey`, receipts, target identity, and state inspection.
- `$agentic-testing` and `$testing-quality-gates` for proof selection and runtime verification.

## Reference Notes

Oracle session `shortcut-escape-window-close-2` recommended modeling the shortcut scenario as launch-origin bookkeeping plus Escape-path parity. Load `references/escape-key-map.md` when implementing or reviewing broad Escape work.
