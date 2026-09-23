# Script Kit patches to gpui-kit 0.6.6

`vendor/gpui-kit/*` is the crates.io release of Longbridge gpui-kit 0.6.6
(`gpui-component`, `gpui-base`, `gpui-component-macros`, `gpui-kit-assets`),
applied through `[patch.crates-io]` in the root `Cargo.toml` so gpui-ai and the
app link one patched copy. It replaces the former Script Kit fork of
gpui-component 0.5 (`vendor/gpui-component`, deleted). Every Script Kit change
is additive and marked `Script Kit` in a comment; keep this ledger current
when upgrading.

## gpui-base

| Area | Files | What / why |
| --- | --- | --- |
| Input extension events | `input/base/script_kit.rs`, `input/mod.rs` | `ScriptKitInputEvent { SelectionChange, PressTab { secondary } }`. gpui-ai matches `InputEvent` exhaustively, so extensions use a second event stream. |
| Input revision / automation | `input/base/script_kit.rs`, `input/base/state.rs` | `revision()` epoch (text version, selection version, selection, IME marked range; selection version defeats A→B→A), `automation_scroll_metrics()`, scroll/value helpers, `soft_wrapped_lines_len`, `set_value_preserving_*`, `set_selection_focused` (fork `set_selection` contract: clamp, scroll, focus), `last_cursor_bounds`, `edit_*` wrappers. |
| Tab navigation | `input/base/script_kit.rs`, `input/editor/indent.rs` | `tab_navigation` / `set_tab_navigation`: Tab/Shift-Tab emit `PressTab` and propagate (snippet tabstops, menu-syntax forms). |
| Highlight ranges | `input/base/script_kit.rs`, `input/base/element.rs` | Host-owned colored/role ranges painted over syntax highlighting, hover tracking and bounds (menu-syntax chips, Notes links). |
| Single-line ghost text | `input/base/kind.rs`, `input/base/script_kit.rs`, `input/base/element.rs` | `InputMode::Extras = SingleLineExtras`: `set_inline_completion_text`, `has/clear/accept_inline_completion`, `inline_completion_visible_without_focus` (launcher ghost prediction). |
| Editor inline completion | `input/editor/lsp/completions.rs`, `input/base/script_kit.rs` | `has/clear_inline_completion` public; `EditorState::set_inline_completion_text`, `rows`. |
| Input line fidelity | `input/editor/display_map/text_wrapper.rs`, `input/base/element.rs`, `input/base/script_kit.rs` | `fidelity_scope(scope)`: during a devtools fidelity capture each soft-wrapped line paints inside a `TextLine` scope `<scope>.line.<row>.<wrap>` with hashed text/layout evidence (never raw text). |
| Scrollbar fidelity | `scrollbar.rs` | `Scrollbar::fidelity_scope`: capture freezes the bar fully shown (no hover/drag/fade) and records axis ⊃ layer, track, thumb scopes with real prepaint geometry. |
| Slider drag state | `slider.rs` | `SliderState::is_dragging` (theme chooser skips syncing into a held slider). |
| Text view | `text/state.rs`, `text/text_view.rs`, `text/format/markdown.rs`, `text/markdown_ext.rs`, `text/style.rs`, `text/node.rs` | WP-B3 hot counters (`set_hot_metered`), `markdown_immediate`, `source_text`, `TextView::fidelity_source`, compact long bare HTTP link labels, blockquote style refinement. |

## gpui-component

| Area | Files | What / why |
| --- | --- | --- |
| TextView compat | `text/compat.rs`, `text/style.rs`, `text/mod.rs` | `fidelity_scope` → `<scope>/document` `TextDocument` scope with exact-source hash + `selectable`/`scrollable` metadata (flow/agent-chat parity probes); link-label policy; code-block copy button; blockquote. |
| Scrollable | `scroll/scrollable.rs` | `vertical_scrollbar_with_fidelity_scope` (Agent Chat transcript scrollbar). |
| Root layers | `root.rs`, `lib.rs` | `RootDialogId`, `RootLayerSnapshot`, `layer_revision_with`, `layer_snapshot`, `is_current_dialog`, `close_dialog_if_current`, `dismiss_notification_if_current`, `paint_background`, `new_transparent`; `Tab`/`TabPrev` re-export. |
| Notifications | `notification.rs` | Layer snapshot/revision, `autohide_after`, `dismissible`, `content_only`, `dismiss_from_control`. |
| Actions module | `lib.rs` | `pub mod actions` (namespaced `sk_*` actions avoid global GPUI action-name collisions). |

## App-side adoption notes

- Theme: `src/theme/gpui_integration.rs` writes the Script Kit palette into
  `Theme.colors` **and rebuilds `Theme.tokens`** (Button variants and gpui-ai
  cards read tokens); `src/theme/gpui_kit_bridge.rs` mirrors into gpui-base
  (`Theme::sync_base`) and re-applies after registry resets.
- Multi-line editors (Notes, Day Page, editor prompt) keep the fork look:
  12×8 inset, `rems(1.25)` line box, no folding, no scroll-beyond-last-line
  (`KIT_MULTILINE_EDITOR_*` in `gpui_kit_bridge.rs`).
- Multi-line is a type in gpui-kit (`InputState` / `TextareaState` /
  `EditorState`); mixed collections use `AnyInputState`.
- Lucide icons Script Kit names that gpui-kit dropped live in
  `vendor/script-kit-lucide-assets` + `src/icons/lucide_catalog.rs`.
