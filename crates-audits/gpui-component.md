# gpui-component Crate Audit

## Current Usage

### Dependency Baseline
- `Cargo.toml` uses a forked git dependency:
  - `gpui-component = { git = "https://github.com/johnlindquist/gpui-component", package = "gpui-component" }` (`Cargo.toml:113`)
  - Fork rationale in repo comments: adds `set_selection()` for programmatic text selection (`Cargo.toml:110`)
- `Cargo.lock` currently resolves to:
  - `git+https://github.com/johnlindquist/gpui-component#b9996cafc2eab35740d4b8dc2fe3268ecd76414e` (`Cargo.lock:3260`)

### Integration Patterns (Correct)
- Initialization is done once at app startup via `gpui_component::init(cx)` before windows are opened (`src/main.rs:2244`).
- All gpui-component-backed windows are wrapped in `Root::new(...)`:
  - `src/main.rs:2316`
  - `src/actions/window.rs:563`
  - `src/confirm/window.rs:270`
  - `src/ai/window.rs:7933`
  - `src/notes/window.rs:4764`
- Theme sync is wired to Script Kit theme updates:
  - `src/main.rs:2255`
  - `src/theme/service.rs:124`
  - `src/theme/gpui_integration.rs`

### Modules/Components Used
- `button` (`Button`) in AI/Notes/BrowsePanel UIs (`src/ai/window.rs:30`, `src/notes/window.rs:17`, `src/notes/browse_panel.rs:18`)
- `input` (`Input`, `InputState`, `InputEvent`) across main app, AI, notes, editor (`src/main.rs:13`, `src/ai/window.rs:30`, `src/notes/window.rs:17`, `src/editor.rs:16`)
- `tooltip` (`Tooltip`) for icon/button hover help (`src/ai/window.rs:30`, `src/notes/window.rs:17`)
- `kbd` (`Kbd`) for showing shortcut hints in tooltips (`src/ai/window.rs:30`, `src/notes/window.rs:19`)
- `scroll` (`ScrollableElement`) in multiple prompt/window surfaces (`src/prompts/chat.rs:18`, `src/components/form_fields.rs:22`)
- `notification` (`Notification`, `NotificationType`) in app notifications (`src/main.rs:14`)
- `theme` (`ActiveTheme`, `Theme`, `ThemeMode`) and theme mapping (`src/theme/gpui_integration.rs:10`)
- `highlighter` (`LanguageRegistry`, `HighlightTheme`) for markdown/editor highlighting (`src/theme/gpui_integration.rs:7`, `src/notes/markdown_highlighting.rs:1`)
- Re-exported primitives/utilities: `Root`, `WindowExt`, `Icon`, `IconName`, `IconNamed`, `Sizable`, `Size`

### Overall Assessment
- The integration is structurally correct: one-time `init`, `Root` wrappers, and centralized theme sync are all in place.
- Current usage is concentrated on core primitives (input/button/tooltip/theme/highlighter). Advanced gpui-component compositional UI elements are mostly not leveraged.

## What gpui-component Provides That We Are Not Using

Fork export surface (`~/.cargo/git/checkouts/gpui-component-*/b9996ca/crates/ui/src/lib.rs`) has 51 `pub mod` entries. Current repo uses 8 module families directly (`button`, `input`, `kbd`, `scroll`, `theme`, `tooltip`, `notification`, `highlighter`).

Likely-unused modules in this codebase:
- `accordion`, `alert`, `animation`, `avatar`, `badge`, `breadcrumb`
- `chart`, `checkbox`, `clipboard`, `collapsible`, `color_picker`
- `description_list`, `dialog`, `divider`, `dock`, `form`, `group_box`
- `history`, `label`, `link`, `list`, `menu`
- `pagination`, `plot`, `popover`, `progress`, `radio`, `rating`, `resizable`
- `select`, `setting`, `sheet`, `sidebar`, `skeleton`, `slider`, `spinner`, `stepper`, `switch`, `tab`, `table`, `tag`, `text`, `tree`

## Are We Using Components Correctly?

Short answer: mostly yes.

What is correct:
- `init` ordering and `Root` usage are correct for gpui-component context/global state.
- Theme synchronization is intentionally centralized and avoids re-calling `init` in secondary windows (`src/notes/window.rs:4592`).
- `InputState` usage patterns (subscribe to `InputEvent`, then `cx.notify()`) are consistent.
- Tooltips use gpui-component API correctly, including key-binding rendering (`Tooltip::new(...).key_binding(...).build(...)`).

Where we are still hand-rolling instead of using built-ins:
- AI dropdown/modal overlays are custom absolute panels + backdrop handlers:
  - shortcuts overlay: `src/ai/window.rs:7066`
  - presets dropdown: `src/ai/window.rs:7176`
  - new chat dropdown: `src/ai/window.rs:7311`
- Notes overlays are also custom:
  - shortcuts help overlay: `src/notes/window.rs:2158`
  - actions/browse overlays: `src/notes/window.rs:3793`, `src/notes/window.rs:3837`
  - browse panel component: `src/notes/browse_panel.rs:446`

These custom implementations are functional, but they duplicate behavior already present in `dialog`, `popover`, `menu`, `select`, and `sheet`.

## `set_selection()` Audit (Fork-Specific)

Definition in fork:
- `~/.cargo/git/checkouts/gpui-component-*/b9996ca/crates/ui/src/input/state.rs:884`

Current behavior:
- Clamps `start`/`end` to `text.len()`
- Sets `selected_range = start..end`
- Sets `selection_reversed = false`
- Scrolls to `end`, focuses input, emits `cx.notify()`

Verdict:
- For current call sites in this repo, it works correctly.
- Most usages are caret placement (`len,len` or `0,0`) in:
  - `src/app_impl.rs`
  - `src/ai/window.rs`
  - `src/notes/window.rs`
  - `src/editor.rs`
- Editor snippet flow explicitly converts char offsets to byte offsets before calling `set_selection` (`src/editor.rs:759`, `src/editor.rs:760`) and has Unicode coverage tests for conversion logic (`src/editor.rs:1387`+).

Important caveats:
- `set_selection` does not normalize reversed ranges (`start > end`).
- It assumes valid UTF-8 byte boundaries; it does not enforce boundary checks.
- The fork appears to have no direct upstream-style regression test for `set_selection` itself (method exists only in `state.rs`, no dedicated component test found).

Practical conclusion:
- Safe for current patterns, but new callers should continue to normalize ranges and convert char offsets to bytes before calling.

## Components We Could Leverage Instead of Hand-Rolled UI

### 1) Dropdowns / Menus
- Use `menu::DropdownMenu` + `PopupMenu` (or `popover`) for simple action menus (presets trigger, attachment picker options) instead of full-screen manual overlays.
- Benefits:
  - Built-in focus/dismiss behavior
  - Reduced custom state and key-routing code

### 2) Searchable Selection Flows
- Use `select` for model/preset picking where there is search + keyboard navigation + sections.
- The current new-chat dropdown manually re-implements filtering, sections, and selection indices (`src/ai/window.rs:2276`, `src/ai/window.rs:2312`, `src/ai/window.rs:2354`, `src/ai/window.rs:7311`).

### 3) Modal/Sheet Overlays
- Use `dialog::Dialog` or `sheet` for keyboard-shortcuts help and notes browse/help panels.
- Current overlays manually manage backdrop clicks, z-order, and close behavior (`src/ai/window.rs:7066`, `src/notes/window.rs:2158`, `src/notes/browse_panel.rs:446`).

### 4) Tooltip Enhancements
- Existing tooltip usage is good; incremental improvement is using `Tooltip::action(...)` for keybinding lookup where actions exist, reducing hardcoded keystroke strings.

## Recommended Priorities

1. Pilot migrating one AI dropdown (`render_presets_dropdown`) to `DropdownMenu`/`PopupMenu`.
2. Pilot migrating one modal overlay (Notes shortcuts help) to `Dialog`.
3. Evaluate replacing new-chat custom selector with `select` if sectioned searchable UX is acceptable.
4. Keep `set_selection` usage pattern: byte offsets only, normalize ranges before call.
