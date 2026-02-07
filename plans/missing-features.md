# Missing Features Analysis (Modern Launcher / Script Runner Parity)

Scope: `src/**/*.rs` only  
Date: 2026-02-07

## Method

Reviewed built-in registry, execution paths, prompt/view surface area, stdin protocol commands, and major capability modules:

- `src/builtins.rs`
- `src/app_execute.rs`
- `src/main.rs`
- `src/stdin_commands.rs`
- `src/render_builtins.rs`
- `src/file_search.rs`
- `src/process_manager.rs`
- `src/executor/runner.rs`
- `src/hud_manager.rs`
- `src/protocol/message.rs`

## Capability Matrix

| Capability | Status | Evidence | Gap vs modern launcher |
|---|---|---|---|
| Clipboard history | Implemented | `src/builtins.rs:372`, `src/main.rs:856`, `src/render_builtins.rs:855` | Core behavior is present. |
| Snippet management | Partial | Scriptlet model/cache in `src/scriptlets.rs`, search integration in `src/scripts/search.rs:589`, clipboard save snippet action in `src/app_actions.rs:2078` | No dedicated snippet manager UI (library view, tagging, bulk edit, conflict handling, import/export UX). |
| System commands | Implemented | Extensive system built-ins in `src/builtins.rs:479` through `src/builtins.rs:747` | Broad coverage exists. |
| Window snapping / tiling | Partial | Window switcher supports tile actions (`src/render_builtins.rs:2022`), tiling engine in `src/window_control.rs:1009`, extension shipping noted in `src/setup.rs:1125` | Capability exists, but direct first-class “window snap” command set was removed from built-ins (`src/builtins.rs:99`, `src/builtins.rs:231`), reducing discoverability. |
| Process management | Partial (backend-only) | Process tracking/cleanup in `src/process_manager.rs`, kill lifecycle in `src/executor/runner.rs:278` | No user-facing process manager view/built-ins to inspect, stop, or restart running scripts/jobs. |
| Notification actions | Partial | Action-capable HUD exists (`src/hud_manager.rs:148`, `src/hud_manager.rs:583`) | Script protocol exposes `notify` (title/body) and `hud` (text/duration) without action payloads (`src/protocol/message.rs:441`, `src/protocol/message.rs:470`), so actions are not a first-class script API. |
| File preview | Partial | Quick Look/Open With support (`src/file_search.rs:877`, `src/file_search.rs:900`) | Built-in preview panel currently shows metadata sections (name/path/details) rather than rich inline content/media preview (`src/render_builtins.rs:4404`). |
| Quick math | Partial | Calculate fallback and meval evaluation (`src/fallbacks/builtins.rs:265`, `src/app_impl.rs:2668`) | Only basic expression eval; lacks unit conversion, currency/timezone conversion, history, variable memory, and richer result formatting. |
| Color picker utility | Missing (as standalone utility) | Theme chooser exists (`src/builtins.rs:1081`) | No dedicated color picker command/workflow for sampling/copying HEX/RGB/HSL from screen or palette. |
| Emoji picker utility | Missing | Emoji appears as icon rendering only (e.g. `src/list_item.rs:18`), no picker builtin in `src/builtins.rs` | No launcher-style emoji search/insert picker command. |

## Additional High-Impact Gaps

1. AI “coming soon” flows are still placeholders:
   - Screen-area capture to AI is not implemented (`src/app_execute.rs:733`).
   - AI preset create/import/search are placeholders (`src/app_execute.rs:746`).
2. Prompt module docs still label several prompts as skeleton (`src/prompts/mod.rs:10` to `src/prompts/mod.rs:14`), suggesting incomplete parity/maturity risk.

## Priority Recommendations

## P0 (big parity wins)

1. Add a built-in **Process Manager** view:
   - List active script processes from `PROCESS_MANAGER`.
   - Stop/restart/kill actions.
   - Show script path, start time, runtime, and status.
2. Upgrade **File Search Preview** to rich preview:
   - Text preview with syntax highlighting.
   - Image/audio/video thumbnail/metadata blocks.
   - Keep Quick Look as secondary action.
3. Implement AI placeholder features:
   - Screen area capture flow.
   - Preset CRUD/search/import UI.

## P1 (utility parity)

1. Ship **Color Picker** built-in:
   - Screen sample + copy formats (`#HEX`, `rgb()`, `hsl()`).
2. Ship **Emoji Picker** built-in:
   - Name/keyword search, recent/frequent, copy/insert.
3. Expand **Quick Math**:
   - Unit conversion, currency conversion, date/time arithmetic, calc history.

## P2 (workflow polish)

1. Dedicated **Snippet Manager** view:
   - Browse/tag/edit snippets, import/export, quick test.
2. Reintroduce direct **Window Snap Commands** in built-ins (keep extension-backed execution):
   - “Tile Left”, “Tile Right”, “Maximize”, etc., for discoverability.
3. Expose **Notification Actions** in script protocol:
   - Action label + callback command/message to close parity gap with modern actionable notifications.

## Known Limits of This Report

1. Scope is Rust sources only (`src/**/*.rs`), not full extension markdown or external SDK scripts.
2. A feature marked “missing” can still exist in external extension content; this report reflects first-party app surface and protocol behavior in Rust code.
