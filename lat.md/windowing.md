# Windowing

Windowing depends on a small set of shared rules for rem sizing, blur, and popup configuration.

The gpui-component root sets the rem size, Script Kit syncs theme fonts into that root theme, and popup vibrancy goes through shared AppKit configuration with a footer-specific host path.

## Key Facts

These facts describe the current cross-layer sizing and vibrancy contract.

- `vendor/gpui-component/crates/ui/src/root.rs` calls `window.set_rem_size(cx.theme().font_size)` during `Root::render`, so rem-based sizing follows the current gpui-component theme on every render.
- `src/theme/gpui_integration.rs` pushes Script Kit's UI and mono font sizes into the global gpui-component theme, which is what ultimately drives rem sizing.
- Main and popup-adjacent overlay windows still use `WindowBackgroundAppearance::Blurred` across launcher-adjacent surfaces such as actions, confirm, ACP popup, ACP chat window, dictation, and notes.
- Main ScriptList focus loss hides the launcher without resetting filter, selection, scroll, or input state; if detached MainList actions are open, they close first and are not restored.
- Shared popup vibrancy configuration uses recursive `NSVisualEffectView` setup with `BehindWindow` blending for detached popup-family windows.
- The shortcut recorder opens as a compact detached popup-family window, not an in-window dimming overlay, so shortcut capture uses native blur and parent-child window ordering.
- Shortcut recorder bounds stay modal-sized around the capture surface instead of matching the launcher width.
- The native footer host is a special case: it uses an in-window `NSVisualEffectView` with `WithinWindow` blending and a custom `hitTest:` path that forwards non-button interaction back to the GPUI surface.
- Blur tint still depends on Script Kit opacity helpers. `theme.opacity.vibrancy_background` overrides the fallback; otherwise the defaults come from `VIBRANCY_DARK_OPACITY` and `VIBRANCY_LIGHT_OPACITY`.
- Script Kit still swizzles GPUI's `BlurredView.updateLayer` so the native tint layer survives instead of being flattened away.

## Key Files

These files define the rem-sizing and vibrancy behavior across windows.

- [vendor/gpui-component/crates/ui/src/root.rs](/Users/johnlindquist/dev/script-kit-gpui/vendor/gpui-component/crates/ui/src/root.rs) - Root wrapper that applies `window.set_rem_size`.
- [src/theme/gpui_integration.rs](/Users/johnlindquist/dev/script-kit-gpui/src/theme/gpui_integration.rs) - Syncs Script Kit fonts and theme into gpui-component.
- [src/platform/vibrancy_config.rs](/Users/johnlindquist/dev/script-kit-gpui/src/platform/vibrancy_config.rs) - Recursive `NSVisualEffectView` configuration for blurred windows.
- [src/platform/secondary_window_config.rs](/Users/johnlindquist/dev/script-kit-gpui/src/platform/secondary_window_config.rs) - Shared popup-family window vibrancy and ACP inline dropdown configuration.
- [src/platform/vibrancy_swizzle_materials.rs](/Users/johnlindquist/dev/script-kit-gpui/src/platform/vibrancy_swizzle_materials.rs) - `BlurredView.updateLayer` swizzle.
- [src/footer_popup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/footer_popup.rs) - Native footer effect host, `WithinWindow` blending, and passthrough hit-testing.
- [src/ui_foundation/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/ui_foundation/mod.rs) - Vibrancy opacity fallback helpers.
- [src/platform/panel_invariants.rs](/Users/johnlindquist/dev/script-kit-gpui/src/platform/panel_invariants.rs) - Runtime invariant audit for the main NSPanel and the `collection_behavior_ok` predicate.
- [src/platform/app_window_management.rs](/Users/johnlindquist/dev/script-kit-gpui/src/platform/app_window_management.rs) - `ensure_main_panel_configured` centralizing helper; sole writer of `PANEL_CONFIGURED`.
- [src/app_impl/shortcut_recorder.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/shortcut_recorder.rs) - Opens the shortcut recorder as a blurred popup-family window and owns close/save handoff.
- [src/components/shortcut_recorder/render.rs](/Users/johnlindquist/dev/script-kit-gpui/src/components/shortcut_recorder/render.rs) - Renders compact recorder content differently for detached popup mode versus legacy inline-overlay mode.
- [tests/shortcut_recorder_popup_window_contract.rs](/Users/johnlindquist/dev/script-kit-gpui/tests/shortcut_recorder_popup_window_contract.rs) - Pins the popup window, vibrancy, and no-parent-backdrop contract.

## Source Documents

These source files justify the windowing rules summarized here.

- [vendor/gpui-component/crates/ui/src/root.rs](/Users/johnlindquist/dev/script-kit-gpui/vendor/gpui-component/crates/ui/src/root.rs)
- [src/theme/gpui_integration.rs](/Users/johnlindquist/dev/script-kit-gpui/src/theme/gpui_integration.rs)
- [src/platform/vibrancy_config.rs](/Users/johnlindquist/dev/script-kit-gpui/src/platform/vibrancy_config.rs)
- [src/platform/secondary_window_config.rs](/Users/johnlindquist/dev/script-kit-gpui/src/platform/secondary_window_config.rs)
- [src/platform/vibrancy_swizzle_materials.rs](/Users/johnlindquist/dev/script-kit-gpui/src/platform/vibrancy_swizzle_materials.rs)
- [src/footer_popup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/footer_popup.rs)
- [src/ui_foundation/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/ui_foundation/mod.rs)
- [src/app_impl/shortcut_recorder.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/shortcut_recorder.rs)
- [src/components/shortcut_recorder/render.rs](/Users/johnlindquist/dev/script-kit-gpui/src/components/shortcut_recorder/render.rs)

## Related Pages

This page connects directly to the design rules built on top of the windowing stack.

- [design](./design.md)
- [architecture](./architecture.md)

## Operational Rules

These rules describe the behavior constraints new windows and overlays should follow.

- Any new top-level window that should participate in rem sizing needs the gpui-component `Root` wrapper.
- Detached popup-family windows should stay on the shared vibrancy path instead of inventing their own AppKit blur stack.
- The footer host is not interchangeable with detached popups; its `WithinWindow` blending and `hitTest:` passthrough are part of the behavior contract.
- Native footer refresh paths must tear down the AppKit footer host when their resolved config is `None`; clearing only the active surface state can leave a stale footer visible.
- `window_resize::mini_layout::NATIVE_MAIN_WINDOW_FOOTER_HEIGHT` is the shared height contract for the AppKit footer host, GPUI spacer, launcher hover blocker, and footer-safe list reveal math.
- Main-window resize work belongs to `$window-resizing`: audit both the open helper and follow-up deferred resize path before changing Mini/Full classification.
- Spawned async resize work must re-read `current_view` before calling raw resize primitives. Term and editor prompt resize tasks capture the prompt id, derive sizing through `calculate_window_size_params()`, and skip if the view changed.
- Detached actions popups stay footerless; shortcuts belong in the rows, not duplicated in popup chrome.
- Shortcut recorder modals belong on the detached popup path, should not dim the launcher with the old full-window backdrop, and should stay narrower than the launcher.
- The shortcut recorder popup stays child-attached to the parent, resurfaces with `orderFrontRegardless`, and must not override GPUI's `WindowKind::PopUp` level.
- When a detached actions popup is open over the main window, the GPUI content behind it should be interaction-shielded so background hover/click/scroll state does not mutate; only the native actions toggle and click-anywhere dismissal path stay live.
- Passive desktop click-away from the main ScriptList must use the preserve-state focus-loss hide path; detached MainList actions close first, while Escape, Cmd+W, script completion, prompt cancellation, and explicit hide/reset commands remain reset paths.
- Main-window hide paths are main-panel-only dismissals. They must use `defer_hide_main_window` rather than `cx.hide()` / `ctx.hide()`, because app-level hide can conceal independent secondary hosts such as Notes if secondary-window detection is stale or racing.
- HUD messages are standalone feedback and must not reveal the launcher. `PromptMessage::ShowHud` clears script-requested hide restore intent before delegating to the HUD manager, pinned by `tests/hud_visibility_decoupled_contract.rs`.

## Mini Main Window Contract

Mini mode is an atomic main-window contract covering mode state, bounds, active prompt render mode, popup cleanup, and native footer sync.

`set_main_window_mode` is the window-backed mode toggle owner. It short-circuits unchanged modes, updates active ChatPrompt mini rendering, closes shared or detached actions popups, runs deferred sizing, syncs the native footer, and emits `main_window_mode_changed`.

Prompt sizing follows the same split. `MiniPrompt` resolves through `ViewType::MiniPrompt`; inline Mini AI and mini-hosted ACP resolve through `ViewType::MiniAiChat`; Full ChatPrompt and ACP continue to use `ViewType::DivPrompt`.

Hide/reset paths snapshot Mini state before `reset_to_script_list`, set `windowVisible:false` before reset or prompt cancellation can make `ScriptList` current, then reset hidden bounds when either the pre-reset or post-reset mode is Mini. This prevents reset normalization from leaking wide hidden bounds into automation receipts and prevents visible main-menu frames during state-first close paths such as Quick Terminal Cmd-W.

## Mini Popup Dismiss Parity

Mini and Full footer dismissals share one close-only behavior when an actions popup is already open.

For any non-Actions footer target, `dispatch_main_window_footer_action` closes both the shared dialog host and the detached hostless actions window when present, logs `main_window_footer_action_closed_actions_only` with `main_window_mode`, and returns without dispatching the clicked footer action.

## Selected Text Clipboard Restore

Selected-text replacement must preserve the user's whole pasteboard when it borrows the clipboard to paste.

`set_selected_text` still uses the clipboard plus Core Graphics Cmd+V fallback, but `src/selected_text.rs` now snapshots every `NSPasteboardItem` type/data representation before writing the replacement text. Snapshot failure aborts before mutation. After the paste attempt, the helper rebuilds the saved pasteboard items and returns an explicit restore error if AppKit cannot restore them.

The helper records the pasteboard change count after writing the temporary text. If another process changes the pasteboard during the paste window, restore is skipped and `setSelectedText` returns an explicit error instead of overwriting newer clipboard state.

The snapshot summary logged around this path is content-light: item count, type count, total byte count, and coarse booleans for text, rich text, image, file URL, and other types. Logs must not include selected text, replacement text, pasteboard type names, file names, or pasteboard bytes.

The source-audit proof lives in `tests/source_audits/selected_text_clipboard_restore.rs`; it pins snapshot-before-mutation, item/type/data preservation, explicit restore failure, and content-light logging without running a live native paste in CI.

## Main Panel Invariants Contract

The main NSPanel's floating-panel posture is a runtime invariant, not just a sequence of one-time AppKit calls.

Oracle-Session `window-activation-invariants-guard` PR1 centralizes the first-show configure sequence and adds a runtime audit that fails loud in debug and grep-logs in release. The audit pins concrete AppKit values (window level, collection-behavior bits, activation policy, animation behavior) so that a silent regression in any of them is caught at the next show rather than discovered by screenshot diffing.

### Pinned constants

These constants live in `src/platform/panel_invariants.rs` and are grep-targets; renaming them is a contract change.

- `EXPECTED_MAIN_PANEL_LEVEL = 101` — `NSPopUpMenuWindowLevel`, NOT `NSFloatingWindowLevel` (3). The launcher actually needs to float above other floating UI. Regressing to 3 would put the panel underneath menu bar extras and other panels.
- `NONACTIVATING_PANEL_STYLE_BIT = 1 << 7` — must be ORed into the panel's style mask so the window never steals activation from the frontmost app.
- `ACTIVATION_POLICY_ACCESSORY = 1` — `NSApplicationActivationPolicyAccessory` keeps the app out of the Dock and menu bar.
- `ANIMATION_BEHAVIOR_NONE = 2` — `NSWindowAnimationBehaviorNone` for the instant dismiss Raycast-style feel.

### Collection behavior shape

The collection-behavior bits are audited by a pure predicate `platform::collection_behavior_ok(bits)` rather than equality. Two shapes are valid:

- `MoveToActiveSpace (1<<1) | FullScreenAuxiliary (1<<8) | IgnoresCycle (1<<6)` — the main-panel default.
- `CanJoinAllSpaces (1<<0) | FullScreenAuxiliary (1<<8) | IgnoresCycle (1<<6)` — acceptable alternate shape for the popup family.

`MoveToActiveSpace` and `CanJoinAllSpaces` are mutually exclusive; the predicate rejects any bit pattern that sets both or neither. `FullScreenAuxiliary` and `IgnoresCycle` are both required.

### One-shot ownership

The `PANEL_CONFIGURED` one-shot guard and its sole legitimate writer together fix the bug where any failing call inside the configure sequence would still flip the atomic and lock the launcher into a broken posture.

- `script_kit_gpui::PANEL_CONFIGURED` (mirror in `main.rs` for the bin crate) is a one-shot `AtomicBool`. Only `platform::ensure_main_panel_configured(context)` is permitted to write it, and only after `assert_main_panel_invariants(context, AfterConfigure).ok()` is true.
- Any show path (`show_main_window_helper`, `stdin_run`, `stdin_show`, the `runtime_stdin_*` dispatchers, orchestrator reveal/focus commands, and dictation delivery reveals) must route through `ensure_main_panel_configured`. Source-audit tests in `tests/panel_invariants_contract.rs` pin this — a refactor that drops the helper on one show path fails the build.
- The audit is instrumented at four phases: `PreShow`, `PostMakeKey`, `BackgroundShow`, `AfterConfigure`. Each phase reports a `PanelInvariantReport` whose `checked`/`mismatched` vectors feed the `PANEL_INVARIANTS` log line for production grep.

### Soft invariants

`is_key_window` at `PostMakeKey` softens because AppKit's `makeKeyWindow` is async — observation can race promotion. It records into a separate `soft_mismatched` bucket.

Soft failures still emit a `PANEL_INVARIANTS SOFT` log line but do NOT flip `ok()` or panic in debug builds. All other `PostMakeKey` invariants (level, style mask, collection behavior, activation policy, animation behavior, restorable, autosave-name) read state configured synchronously and stay on fail-loud `record`.

The split is pinned at source level by `tests/panel_invariants_soft_is_key_window_contract.rs`. It asserts `record_soft` has exactly one call site (the `is_key_window` check) and the other 11 invariant names continue to use fail-loud `record`. A refactor that softens another invariant, or that reverts `is_key_window` to `record`, fails the contract test.
