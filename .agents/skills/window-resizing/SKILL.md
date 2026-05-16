---
name: window-resizing
description: >-
  Main-window presentation modes, Mini vs Full sizing, resize_to_view paths,
  update_window_size_deferred, content-aware mini sizing, and resize audits.
---

# Window Resizing

This skill owns main-window sizing behavior for Script Kit GPUI and keeps changes grounded in current source and the narrowest useful proof.

## Use When

Use this skill for tasks involving:

- Mini vs Full main-window presentation, `MainWindowMode`, `ViewType`, `resize_to_view_sync`, `defer_resize_to_view`, `update_window_size_deferred`, and `calculate_window_size_params`.
- Bugs where an entry path opens a surface at the wrong width or height.
- Built-in surface sizing audits, especially when keyboard shortcuts, tray actions, triggerBuiltin, or filter changes resize after a view opens.

Do not use this skill as the primary owner for AppKit activation/floating-level bugs, popup-only height changes, or row filtering semantics; load the adjacent owning skill instead.

## First Reads

Start with these sources before editing:

- `src/app_impl/ui_window.rs`
- `src/window_resize/mod.rs`
- `.agents/subagents/window-resizing-reader.md` for broad or high-risk investigation.

## Owned Paths and Concepts

Primary paths and concepts:

- `src/app_impl/ui_window.rs`, `src/window_resize/`, `src/app_execute/builtin_execution.rs`
- `MainWindowMode::{Mini, Full}` and `ViewType::{MiniMainWindow, ScriptList}`
- Deferred and synchronous main-window resize contracts.
- Tests that pin Mini/Full surface classification.

## Core Rules

- Pick width from the surface layout contract, not from command importance, dataset size, or entry source.
- Single-column filterable built-ins stay Mini unless their render layout has a preview/detail column.
- Preview/detail surfaces stay Full and use `ViewType::ScriptList`.
- Any deferred resize path must preserve the same Mini/Full classification as the open helper used to enter the surface.
- Prefer current source and generated contracts over legacy notes or memory.

## Workflow

1. Review `AGENTS.md`, the owning skill, and current source context before editing.
2. Identify every entry path that can open or resize the affected surface: main menu, triggerBuiltin, tray, shortcut, filter change, and protocol.
3. Trace both the initial open call and any follow-up resize call. Bugs often come from a correct open helper followed by a stale deferred resize.
4. Make the narrowest change at the shared sizing source of truth.
5. Add or update a source-audit test that pins the surface's Mini/Full classification and the high-risk entry path.
6. Verify with focused tests, then use `$agentic-testing` only when runtime bounds proof is required.

## Proof Ladder

Use the smallest proof that can falsify the change.

1. Source-audit proof: for Mini/Full classification, helper routing, and direct resize-call audits.
2. Compile/static proof: for Rust changes that affect resize helpers or enums.
3. Targeted test proof: for behavior encoded in contract tests.
4. State-first runtime proof: when the bug depends on a real entry path and automation exposes window bounds.
5. Visual proof: only when text/layout clipping or screenshot acceptance criteria are part of the change.
6. Native input / OS focus proof: only when the resize depends on real keyboard shortcuts, global hotkeys, AppKit focus, or display placement.

Always clean up any process, session, or window the proof started. Report the tier used, exact commands or receipts, and why higher tiers were unnecessary.

Default check for this skill:

```bash
cargo test --test source_audits deferred_sizing -- --nocapture
cargo test --test trigger_builtin_current_app_commands_contract -- --nocapture
```

## Adjacent Skills

Use adjacent skills when the work crosses boundaries:

- `$builtin-filterable-surfaces` for row data, visible counts, and built-in list rendering.
- `$launcher-surface-contracts` for `AppView`, `SurfaceKind`, and triggerBuiltin route ownership.
- `$keyboard-focus-routing` for global hotkeys and shortcut delivery.
- `$platform-windowing-macos` for AppKit panel placement, activation, or native window invariants.
- `$agentic-testing` for state-first runtime bounds receipts and cleanup.
- `$testing-quality-gates` for choosing narrow build/test gates.

## Migration Notes

Legacy `.claude/skills/*` material can be mined for durable facts, but this repo-local skill is the canonical Codex routing name for main-window resize work.
