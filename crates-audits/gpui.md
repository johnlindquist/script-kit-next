# GPUI Crate Audit

## Current Usage

### Dependency Baseline
- `Cargo.toml` uses a git dependency without `rev`: `gpui = { git = "https://github.com/zed-industries/zed" }`.
- `Cargo.lock` currently resolves GPUI to `94faaebfec5e46afa6d3aee5372cbbdb33b97c33` (commit date: 2026-01-02).
- Upstream `zed` `main` is currently `79f38fea649783e76b417bd3647fed45e0ac8d2a` (commit date: 2026-02-07), so the lockfile is behind current upstream.

### Patterns Used Well
- Virtualized lists are broadly adopted with `uniform_list(...)` + `UniformListScrollHandle`:
  - `src/render_builtins.rs:589`
  - `src/prompts/select.rs:883`
  - `src/prompts/path.rs:562`
  - `src/render_prompts/arg.rs:512`
  - `src/notes/actions_panel.rs:696`
- Variable-height virtualization with `list(...)` is used where appropriate:
  - `src/render_script_list.rs:253`
  - `src/prompts/chat.rs:3099`
  - `src/ai/window.rs:5810`
- Zero-copy GPU surface rendering is already in use for webcam/video:
  - `src/prompts/webcam.rs:95` (`gpui::surface(buf.clone())`)
  - `src/app_execute.rs:1922` (commented render path around `surface()` usage)
- Deferred window mutation is used correctly with `Window::defer(...)` to avoid borrow-cycle conflicts:
  - `src/window_ops.rs:165`
  - `src/app_execute.rs:1288`
  - `src/actions/window.rs:309`
- Keyboard focus + key contexts are widespread:
  - `src/render_script_list.rs:876`
  - `src/prompts/chat.rs:3205`
  - `src/prompts/select.rs:980`
  - `src/actions/dialog.rs:2058`
- External file drop handling is implemented in chat surfaces:
  - `src/prompts/chat.rs:3084`
  - `src/ai/window.rs:5981`
  - `src/ai/window.rs:6229`

### Overall Assessment
- The codebase is using core GPUI primitives effectively for retained-mode rendering, focus, and list performance.
- The strongest areas are list virtualization and deferred window ops.
- The weakest areas are action-driven key dispatch adoption, animation APIs, and deferred layered rendering primitives.

## Missed Opportunities

### 1) Deferred Floating Layers (`deferred`, `anchored`, `on_mouse_down_out`)
- Current pattern builds manual absolute backdrops for popups/dialogs:
  - `src/render_prompts/arg.rs:698`
  - `src/render_prompts/div.rs:205`
  - `src/render_prompts/editor.rs:316`
  - `src/render_prompts/form.rs:392`
  - `src/render_prompts/term.rs:315`
- Upstream GPUI pattern uses deferred floating layers:
  - `crates/gpui/examples/popover.rs` (uses `deferred(...)` + `anchored()` + `on_mouse_down_out(...)`)
- Opportunity:
  - Reduce overlay z-order complexity.
  - Simplify click-outside dismissal.
  - Better isolate floating UI painting order (`priority(...)`).

### 2) GPUI Animation APIs (`with_animation`, `Animation::new`)
- No direct usage found in `src/**/*.rs` of:
  - `with_animation(...)`
  - `Animation::new(...)`
- Existing code uses manual transition math/timers instead:
  - `src/transitions.rs`
  - `src/components/toast.rs:277`
  - `src/components/alias_input.rs:58`
  - `src/components/shortcut_recorder.rs:57`
- Upstream example:
  - `crates/gpui/examples/animation.rs`
- Opportunity:
  - Move repetitive bespoke easing/timer code to built-in GPUI animation pipelines.
  - Improve consistency of UI motion across prompts/dialogs.

### 3) Action-Based Key Dispatch Is Underused
- `on_key_down` usage is very high (42 callsites), while `on_action` usage is minimal (2 callsites: `src/editor.rs:1227`, `src/editor.rs:1228`).
- GPUI guidance is action-first key dispatch via `key_context` + actions (`docs/key_dispatch.md`).
- Manual string matching appears in many prompt components:
  - `src/prompts/select.rs:760`
  - `src/prompts/path.rs`
  - `src/prompts/div.rs`
  - `src/prompts/chat.rs`
- Opportunity:
  - Consolidate key handling and reduce platform key-name divergence bugs.
  - Make bindings configurable/testable through action schemas and context maps.

### 4) Internal Drag-and-Drop (`on_drag`) Not Used
- External drop (`on_drop`) exists, but no `on_drag(...)` callsites in app code.
- Upstream example:
  - `crates/gpui/examples/drag_drop.rs`
- Opportunity:
  - Reordering for lists/actions/presets/history entries without ad-hoc drag state.
  - Better UX for sidebar/chat list organization.

### 5) Canvas-Based Custom Rendering (`canvas`) Not Used
- No `canvas(...)` usage in app code.
- Upstream examples:
  - `crates/gpui/examples/painting.rs`
  - `crates/gpui/examples/data_table.rs` (custom scrollbar interactions)
- Opportunity:
  - Use for very dense custom visuals (diagnostic overlays, richer custom scrollbars, visual inspectors) where element tree overhead dominates.

### 6) GPUI Native Test Harness Not Used
- No `#[gpui::test]` usage found.
- Opportunity:
  - Add focused GPUI-level tests for keyboard dispatch/focus layering/overlay dismissal, reducing reliance on only app-level smoke workflows.

## Deprecated Patterns

### Confirmed
- No direct deprecated GPUI API usage was found in the current `gpui` crate source at HEAD (`crates/gpui`) via `#[deprecated]` scan.
- No obvious deprecated-callsite hotspots were detected from project usage scans.

### At-Risk / Legacy-Style Patterns (Not formally deprecated)
- Heavy raw `on_key_down` string matching rather than action-first dispatch can become brittle as key routing evolves.
- Manual overlay/backdrop stacks instead of `deferred` + `anchored` + `on_mouse_down_out` increase complexity and layering bug risk.
- Floating git dependency in `Cargo.toml` with lockfile-bound old commit can silently drift when lockfiles regenerate.

## Recommendations

### Priority 0 (stability and upgrade safety)
1. Decide GPUI version strategy explicitly:
   - Either pin `rev` in `Cargo.toml` for deterministic builds, or
   - Keep floating git dependency but add scheduled lockfile refresh + CI gate for GPUI updates.
2. Run a controlled GPUI bump from `94faaeb` to current upstream and validate:
   - `cargo check`
   - `cargo clippy --all-targets -- -D warnings`
   - `cargo test`

### Priority 1 (reduce UI complexity/bugs)
1. Migrate popup/dropdown overlays to deferred floating layers:
   - Start with actions dialogs in prompt renderers (`src/render_prompts/*.rs`).
2. Introduce action-first key dispatch in highest-churn prompts:
   - `src/prompts/chat.rs`
   - `src/prompts/select.rs`
   - `src/prompts/path.rs`
3. Standardize dismissal behavior with `on_mouse_down_out` where suitable.

### Priority 2 (UX and performance polish)
1. Adopt GPUI animations for repeated transition patterns:
   - Toast appear/dismiss
   - Recorder/input overlays
   - Loading/typing indicators
2. Add internal drag-and-drop where it improves organization flows (actions, presets, notes/chat lists).
3. Evaluate `canvas(...)` only for proven hotspots after profiling (avoid premature migration).

### Suggested Incremental Plan
1. Pilot `deferred` + `anchored` conversion on one popup (`arg` prompt actions).
2. Pilot action-based key routing on one prompt (`select`).
3. Add GPUI-level tests for those two refactors.
4. Expand pattern to other prompts once validated.

