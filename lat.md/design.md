# Design

Script Kit GPUI's design language stays keyboard-first, macOS-native, and deliberately quiet. The chrome should stay out of the way while still giving the user clear affordances.

## Launcher contract

The main launcher footer keeps at most three primary affordances: `Run`, `Actions`, and `AI`. Anything beyond that belongs in the `Actions` dialog or a more specific surface rather than in persistent chrome.

When Enter will paste into another app, the footer should swap the generic `Run` copy for a paste-specific label that includes the tracked frontmost app name when it is available.

When the ACP chat surface is active, the footer hides the `Run` button until a validated `SCRIPT_READY path=... validated=true` receipt exists in the assistant output. Once a receipt is present, `Run` dispatches `execute_script_by_path` for that specific script path rather than the generic `execute_selected`. This prevents running the wrong target during script generation.

## Select prompt contract

Select prompts use the minimal list shell while making row identity, keyboard ownership, and selection ownership explicit.

Typed filtering is owned by the prompt entity. Single-select submission is focus-owned, so stale selected-set state cannot paint as a second active row; multi-select submission is selected-set-owned and keeps checked rows independent from focus. Row semantic IDs resolve through [[src/prompts/select/types.rs#select_choice_semantic_id]], which keeps rendered row IDs and `getElements` IDs aligned as explicit `semanticId` > stable key > source-index fallback.

Keyboard routing is classified through [[src/prompts/select/render.rs#classify_select_key]] before dispatch. Plain navigation, Enter, Backspace, printable filter text, multi-select Cmd+A, and multi-select Cmd+Space stay prompt-owned; platform shortcuts such as Cmd+Enter and Cmd+K yield to the shared global/action routing advertised by the universal footer.

Select rows still render through [[src/components/unified_list_item/render.rs#UnifiedListItem]], but pass `.with_direct_hover(false)` so the prompt's modality-adjusted row state owns hover paint. This avoids a double-hover path where GPUI hover styling could contradict keyboard modality.

Footer ownership stays single-source. When the native main-window footer reports the `select_prompt` surface, select renders the shared native-footer spacer instead of the GPUI hint strip so the prompt cannot stack two footer rows.

Built-in renderers delegate native footer ownership through `main_window_footer_slot` rather than checking `active_main_window_footer_surface()` directly. PromptFooter exceptions such as Design Gallery and Kit Store stay off the native footer map until their domain-specific actions have native footer buttons.

SDK-facing expanded browser surfaces may use surface-specific footer labels, but they still keep the three-affordance budget and audit through `emit_surface_prompt_hint_audit` so intentional custom copy does not look like universal-footer drift.

Those SDK-facing browser surfaces also route secondary labels, counts, empty states, and preview metadata through `AppChromeColors` rather than raw muted or dimmed theme colors so custom themes keep the same semantic text ladder as launcher chrome.

Create-flow prompts share the same text ladder through `prompt_text_palette` and `prompt_field_style`: primary values use `text_primary`, empty field values use `placeholder_text_rgba`, and help/count/path-prefix text uses hint or muted opacity tokens instead of local packed-alpha math. Path and Env prompts also use the shared form intro/section/help helpers so secret entry, path filtering, and starter creation keep one compact GPUI surface contract.

Path prompts own native footer submission: `Run` is labeled `Select`, dispatches to `PathPrompt::handle_enter`, and never falls through to launcher selection. Their native footer keeps `⌘K Actions` but omits launcher AI unless a future path-specific route is designed.

Template prompts follow that create-flow contract while owning their tab-through behavior. Their placeholder field list scrolls within the prompt body, click targets move the current field, plain `{{name}}` accepts human-readable text, and both GPUI and native footer paths advertise `Submit`, `Next Field`, and `Actions` through a surface-specific hint audit.

Native `form()` fields resolve text, placeholder, label, cursor, and checkbox mark colors before render through shared chrome tokens rather than wrapping muted/secondary base colors in renderer code. Text fields and text areas both show a cursor plus placeholder while focused and empty, and the form prompt synchronizes parent focus state when fields are clicked so key routing follows the visible focus target.

Editor prompt chrome keeps the code editor body separate from wrapper-owned footer and layout metadata. Snippet choice popups use inline dropdown chrome tokens, and editor layout probes report an `EditorContent` prompt branch instead of main-menu list/preview components.

Terminal prompt chrome distinguishes SDK `term()` from Quick Terminal without changing their footer contracts. SDK terminals keep the GPUI hint strip, Quick Terminal reserves native-footer space, and layout probes report `TerminalContent` instead of launcher list/preview components.

Select and drop prompts keep ownership of their prompt surfaces in layout metadata rather than falling back to launcher list/preview probes. Select rows pair pointer chrome with mouse activation so keyboard and mouse selection ownership stay aligned.

Webcam prompts own capture semantics in both GPUI and native footer chrome. Their primary footer action is `Capture Photo`, the actions footer remains `⌘K Actions`, and footer Run routes to the webcam capture helper before any launcher execution fallback.

SDK `mic()` is still a media stub, not a rendered prompt surface. It must remain routed to coming-soon feedback until a real microphone view can own explicit recording footer semantics; `micro()` remains the ultra-compact text prompt with no footer.

`mini()` is an arg-like compact list prompt: Enter submits the current prompt state, the shared minimal-list shell owns header padding, and footer hints route through the native footer slot. `micro()` stays footerless and off native-footer routing so the main window does not reserve space the renderer never draws around.

## Launcher query memory

Exact non-empty launcher queries should reopen with the last submitted result promoted back to the first selectable row so `Up` recall and retyping the same query both support one-keystroke reruns.

That preference should only apply when the normalized query text matches exactly. Unrelated queries still use the normal fuzzy-plus-frecency ranking, and unsupported result types fall back to plain text history without inventing a fake remembered target.

## Footer-safe list reveal

The main launcher list treats the footer as an overlay, so selected-row reveal math must subtract footer height from the visible viewport and preserve enough trailing scroll range to lift the last row clear of the blur.

Wheel scrolling in that list also belongs to the launcher's selection-driven path, so the wrapper must stop propagation after handling a wheel event instead of letting GPUI pixel-scroll the list separately.

The launcher scrollbar should read from the real list handle instead of a second visual-only thumb model. If the handle moves selection-owned content directly, the launcher must immediately reanchor selection back into the visible window before execution shortcuts run.

That same scrollbar must size and clip itself against the footer-safe viewport rather than the full list pane. Otherwise the thumb grows too large, drifts against the true scroll range, and stays visible underneath the footer overlay.

When debugging bottom-of-list behavior, the launcher wheel handler should emit `SCROLL_STATE` fields for wheel delta, selection before/after, logical scroll top before/after, and propagation so scroll drift can be proven from logs.

The footer-safe reveal should also keep a small visual gap above the blur instead of aligning the last row flush to the footer edge.

## Chrome style

The visual system uses whisper-thin borders, low-opacity fills, and stable spacing instead of card-heavy composition. Theme work should route through the shared opacity and chrome tokens in `src/theme/opacity.rs` and `src/theme/chrome.rs`.

Warning banners are compact one-line launcher chrome. They resolve warning background and readable foreground through shared theme helpers, keep hover opacity tokenized, and dismiss clicks must not trigger the banner action.

All text elements use `text_primary` (white on dark) as the base color; brightness is controlled purely via semantic opacity tiers defined on `BackgroundOpacity`: `text_name` (1.0), `text_strong` (0.80), `text_muted_alpha` (0.65), `text_hint` (0.45), `text_placeholder` (0.40), `text_icon` (0.50). Dark row backgrounds use `hover` (0.06) and `selected` (0.23) opacity on `text_primary`; light row backgrounds use `hover` (0.04) and `selected` (0.08). No double-dimming from secondary/muted/dimmed base colors.

Theme validation should also treat `hover >= selected` as a warning because equal row opacities erase the focus hierarchy and make hovered rows compete visually with the focused item.

The current theme layer also has a unified resolver path in `src/theme/color_resolver.rs` for colors, typography, and spacing. New theme-aware UI should prefer those resolver types instead of reintroducing ad hoc default-vs-design branching.

Launcher microcopy hints should stay visible enough to teach the shortcut without reading as a button. The `Ask` label and tab badge resolve through the shared `placeholder_text_rgba` token on `AppChromeColors` — `text_primary` composited with `opacity.text_placeholder` — so they match the main input placeholder ("Script Kit") and the actions dialog search placeholder exactly. Section headers stay quieter with natural casing and a slightly bolder weight instead of all-caps shouting.

The vendor gpui-component theme bridge follows the same rule: `theme_color.muted_foreground` maps to `text_primary + opacity.text_placeholder` instead of a pre-dimmed `colors.text.muted` hex, which keeps any input component's empty-state placeholder on the same semantic ladder.

That natural casing rule starts at the label source, not just the renderer. Launcher grouped-result builders should emit `Suggested`, `Main`, `Commands`, and `Apps` directly, without count suffixes, so the shared section-header component does not have to undo pre-uppercased strings or strip extra metadata.

Actions dialogs should inherit that same semantic text ladder instead of using separate secondary or dimmed base colors. Titles, section labels, search placeholder text, and shortcut chrome should all resolve from `text_primary` plus the shared opacity tiers so the popup hierarchy matches the launcher hierarchy.

Default actions-dialog row selection and hover backgrounds resolve through `AppChromeColors`, and the live fallback keeps the container border off so detached popups stay material-first.

## Transient Feedback

Toast notifications are queued as simple message feedback, then bridged into gpui-component notifications during render.

`ToastManager` is a staging queue, not a custom visual runtime. It preserves message text, variant, duplicate-count suffixes, and persistent-vs-default-autohide behavior. The conversion bridge owns Script Kit chrome tokens and vibrancy shadow suppression so notifications follow popup material rules instead of the older custom toast renderer.

## Shortcut recorder modal

The shortcut recorder is a compact popup modal for capture, not an instructional overlay.

It uses the command name as the title, a short `Press keys` placeholder, visible action buttons, and no footer or long instruction copy. Its shell stays narrower than the launcher so the parent remains visually behind it.

It dismisses on explicit cancel shortcuts (Esc, Cmd+W) and on any focus loss, including backdrop clicks and clicks back into launcher surfaces such as the main filter input.

The detached popup consumes handled key events, treats Esc and Cmd+W as cancel rather than captured shortcuts, and resolves its surface and border through `AppChromeColors`. Its detached margin is an invisible click-to-cancel target while the modal body stops propagation so inside clicks stay local.

## Quit modal

Destructive system commands like Quit Script Kit, Empty Trash, Restart, Shut Down, and Log Out share a single confirm flow with two routes: an in-window state of the main window (default) and a separate native popup window (cross-window fallback).

Routing: every confirm caller funnels through [[src/confirm/parent_dialog.rs#confirm_with_parent_dialog]]. When the main window is visible the function asks a process-static `InWindowRouter` registered at startup to push [[src/main_sections/app_view_state.rs#AppView]] `ConfirmPrompt` onto the main `ScriptListApp` entity. The router unwraps `gpui_component::Root` to reach the inner ScriptListApp AnyView before calling [[src/app_impl/about_route.rs#ScriptListApp#open_confirm_prompt]]. If the main window is hidden, or if the active context is not the main window's root, the router declines and the legacy popup path runs via [[src/confirm/parent_dialog.rs#open_parent_confirm_dialog]] — preserving the popup for notes/chat/script-execution callers.

### In-window confirm state

The confirm UI is a state of the main window — title + body in the content area, native AppKit footer reused for the confirm/cancel buttons.

The `AppView::ConfirmPrompt { options, sender, focused_button, previous }` variant carries an `async_channel::Sender<bool>` and the previous launcher view it must restore. Surface contract: `FeedbackSurface + NoEditableInput + FeedbackPanel`, `explicit` dismiss policy, automation tag `confirmPrompt`. [[src/render_prompts/other.rs#ScriptListApp#render_confirm_prompt]] draws title + body; [[src/app_impl/ui_window.rs#ScriptListApp#confirm_prompt_footer_buttons]] maps `FooterAction::Apply` to confirm and `FooterAction::Close` to cancel — labels and the `selected` flag come from `ParentConfirmOptions` and `focused_button` so no native ObjC selector wiring changes.

Esc / Enter / Tab are owned by the view's `on_key_down`: Esc resolves false, Enter resolves based on `focused_button`, Tab toggles focus. All three call `cx.stop_propagation()` so keys cannot leak to the launcher filter, ACP, or the actions dialog. [[src/app_impl/ui_window.rs#ScriptListApp#resolve_confirm_prompt]] sends the bool down `sender` and restores `previous`.

### Confirm popup focus colors

Both Esc and ↵ keycaps in [[src/confirm/window.rs#ConfirmPopupWindow#render]] share a single visual key style: focused button uses `theme.colors.accent.selected` for keycap bg + glyph, unfocused stays on `theme.colors.ui.border @ 0.06` + muted text.

Danger semantics live on the *label*, not the keycap glyph — the `Quit` / `Empty Trash` / `Shut Down` verb is painted in `theme.colors.ui.error` when `is_danger`, so the destructive intent reads through the verb while the keycap stays consistent with the Cancel side.

## Rem sizing

Rem sizing follows the gpui-component `Root` wrapper and the current theme font size.

`Root` pushes `cx.theme().font_size` into `window.set_rem_size(...)` during render. UI that should scale with the theme should stay on rem-based helpers such as `text_sm()` and `rems(...)`.

## Vibrancy

Vibrant popups should stay translucent enough for the desktop blur to remain visible.

That effect depends on blurred GPUI windows, Script Kit's `BlurredView` swizzle, popup-specific `NSVisualEffectView` configuration, and low-opacity overlays instead of opaque fills.

## Overlay split

Different overlay families use different macOS blur recipes on purpose.

The footer is an in-window `NSVisualEffectView` host with `WithinWindow` blending, detached popups use `configure_secondary_window_vibrancy()`, and ACP inline dropdowns use `configure_inline_dropdown_popup_window()`.

## Window levels

Popup windows should stay inside GPUI's popup-level contract instead of inventing new levels.

`WindowKind::PopUp` already provides the needed level, `orderFrontRegardless` is the resurfacing tool, and child-window attachment keeps confirm overlays above their parent without breaking that contract.

## Context portalling

Inline `@` mentions are designed as stable pointers into other context surfaces. Passive preview is allowed, but entering or replacing a mention must be explicit and must preserve a clear return path back to the original editor or chat surface.

## Popup behavior

Parent-relative popup windows and consistent row heights keep the app feeling like one system instead of a stack of unrelated dialogs. That rule matters most for the main window, actions popup, and context-picker surfaces.

Window mechanics for dense inline popups (ACP `/` slash commands, ACP `@` mentions, ACP model selector, ACP history, and menu-syntax `:`, `;`, and `!` trigger popups) share one implementation in [[src/components/inline_popup_window.rs#inline_popup_window_options]]. The shared module owns popup bounds math, `no-focus-steal` window options, child-window attach/detach, and AppKit pointer plumbing. Surface-specific files like [[src/ai/acp/popup_window.rs]] remain as thin facades that re-export the shared symbols under their ACP-compatible names, so historical callers and source-text audits stay stable.

Row rendering for those popups is owned by [[src/components/inline_dropdown/mod.rs]] (`InlineDropdown`, `render_soft_compact_picker_row`, `inline_dropdown_visible_range_from_start`, `InlineDropdownColors`). The neutral row **shape** that cross-surface callers and the menu-syntax trigger popup carry is [[src/components/inline_picker.rs#InlinePickerRow]] — a behavior-free struct with `id`, `kind`, title/token/subtitle/detail/example text slots, optional leading visual, badges, accessory, precomputed highlight ranges, and an `enabled` flag. Owners map their domain row (ACP's `ContextPickerItem`, menu-syntax's `TriggerPickerRow`) into `InlinePickerRow` via a small adapter function kept in the owner's module, never in the shared file. The shared file also exposes enabled-aware selection helpers (`inline_picker_next_enabled_index`, `inline_picker_previous_enabled_index`, `inline_picker_normalize_selected_index`) that skip disabled rows — something the generic `inline_dropdown` selection helpers do not know about.

ACP popup files should source shared row renderers and row-height constants directly from `inline_dropdown`. The ACP-local `context_picker_row` module must not become the owner of popup row mechanics again.

Menu-syntax popup rows keep footer actions explicit but clickable. Keyboard default selection skips footer rows via [[src/app_impl/menu_syntax_trigger_popup.rs#trigger_popup_row_is_default_selectable]], while mouse clicks on enabled footer rows still route through the same accept outcome as keyboard activation.

Menu-syntax trigger popup footers are pinned below the paged normal row body. [[src/app_impl/menu_syntax_trigger_popup_window.rs#trigger_popup_normal_row_capacity]] subtracts footer rows from the shared visible-row budget so long `:` / `;` / `!` lists cannot hide the footer or create duplicate footer chrome. When footer rows are present, the menu-syntax popup suppresses the shared synopsis strip so the action footer remains the only bottom chrome.

Menu-syntax popup updates preserve the current visible page before handing the new snapshot back to `inline_dropdown_visible_range_from_start`. This keeps arrow navigation from shifting the window on every row movement once the selection has left the first page.

Menu-syntax popup window height reserves the shared synopsis strip when the selected row has detail or example text. The row window and synopsis chrome should size together instead of clipping the lower strip.

Menu-syntax trigger popups register as attached automation windows with the `menuSyntaxTriggerPopup` surface. Runtime screenshot and layout probes should target that child window rather than the suppressed main surface behind it. Popup sync discard paths clear the attached automation registration before dropping the singleton slot, keeping screenshot targets from resolving stale child bounds.

The detached actions dialog is intentionally footerless. Action rows already expose their shortcuts inline, so extra footer chrome would duplicate information and compete with the list.

Actions dialog configuration is normalized to that footerless contract at construction and config-update boundaries. Runtime audits should report resolved chrome, such as rendered icon visibility, rather than raw requested flags.

Detached actions popups let the wrapper own focus tracking. The shared spawn helper sets the dialog's skip-track-focus flag so generic, chat, webcam, and terminal popup hosts follow the same keyboard ownership contract.

Detached actions-window key handlers consume handled navigation, execution, filter, close, and matched-shortcut keys so popup-owned input cannot leak back into parent launcher surfaces.

Selection-owned popup lists should keep wheel behavior index-based even when their scrollbar becomes handle-driven. Free-scroll transcript surfaces can use pixel offsets, but selection-owned browsers still need a visible active row after scrolling.

Inline dropdown lists keep the current visible page fixed until keyboard selection leaves its top or bottom edge. This avoids premature centered scrolling while preserving the invariant that the selected row remains visible.

That reanchor rule now applies to the launcher plus the built-in history browsers that track plain `ScrollHandle` state. When browser history, dictation history, or ACP history scrollbars move independently of arrow-key navigation, render should clamp the selected row back into the visible window before Enter or attach actions read it.

ACP slash-command rows use the main launcher row rhythm: the slash command itself is the primary label, and source ownership is quiet theme-token metadata.

The slash picker should not duplicate `/command` as a dim right-side echo or render owner badges as outlined controls. Accent bars, fuzzy highlights, checkmarks, and owner metadata resolve through `InlineDropdownColors` and `AppChromeColors` so custom themes control the popup.

The slash-picker typography storybook isolates five row treatments so font weight, row height, and metadata chrome can be selected visually before adopting a final runtime style. The live ACP slash and mention pickers share the Soft Compact treatment: 36px rows, normal-weight labels, softer selected fill, and theme-token metadata badges.

The Ask+Tab glyph storybook evaluates top-right launcher header affordances in a fresh mock menu instead of reusing the existing main-menu stories, which are not a reliable design baseline.

The story now exposes 30 variants, including nine additional options that all use the keyboard tab glyph `⇥` while varying only emphasis, containment, and spacing.

The live launcher Ask+Tab affordance adopts option 22, `tab-glyph-soft-right`: a bare `Ask ⇥` treatment using muted theme text, 15px normal-weight labels, 5px spacing, and no keycap or pill chrome.

That mock menu should preserve the launcher density while it compares glyph variants: 40px rows, compact SVG icons, 14/12px row text, a visible footer, and a scaled full-menu thumbnail in compare mode.

The Storybook measurement grid is opt-in via `SCRIPT_KIT_STORYBOOK_GRID`; design comparison previews should not render ruler lines by default because they distort color and spacing judgment.

The live dictation overlay uses the compact capsule direction: a narrower standalone capsule with main-menu density, quiet waveform chrome, readable primary timer/target text, and a neutral native rim.

The target badge should name the actual delivery surface. Internal targets use explicit labels such as `Script Kit` or `ACP`, while external-app dictation uses the tracked frontmost app name and ellipsizes inside the capsule.

The dictation UI variation storybook keeps the overlay concepts standalone while comparing related default-state treatments against that compact capsule baseline.

The same handle-owned contract now covers the launcher-family uniform-list builtins. App launcher, window switcher, browser tabs, and current app commands all attach the vendor scrollbar to the real `UniformListScrollHandle`, keep wheel scrolling row-stepped, and reanchor selection after handle-driven movement.

## Kit store row hover

Kit store rows follow the shared row-state ladder: selected rows keep the stronger selected fill, and unselected rows rely on direct GPUI hover styling instead of local hover bookkeeping.

This keeps the built-in browser aligned with the same hover contract used by other popup lists after the GPUI vendor cleanup work.

The row backgrounds, action chips, and long text constraints resolve through `AppChromeColors` and fixed-height ellipsis rules, so theme changes and repository metadata cannot break the 72px list rhythm.

Kit Store consumes the keyboard, wheel, row-click, and action-chip events it owns. Browse and Installed views keep wheel movement selection-owned through the shared builtin scroll helpers, reanchor scroll after query/registry refresh, and stop propagation after install/update/remove chip clicks so parent row handlers cannot double-handle the action.

## Current sources

This page is justified by the live chrome, popup, and portal code plus the root repo contract:

- [CLAUDE.md](../CLAUDE.md)
- [AGENTS.md](../AGENTS.md)
- [src/render_builtins/kit_store.rs](../src/render_builtins/kit_store.rs)
- [src/footer_popup.rs](../src/footer_popup.rs)
- [src/input_history/mod.rs](../src/input_history/mod.rs)
- [src/scripts/grouping.rs](../src/scripts/grouping.rs)
- [src/scripts/grouping/search_mode.rs](../src/scripts/grouping/search_mode.rs)
- [src/app_impl/selection_fallback.rs](../src/app_impl/selection_fallback.rs)
- [src/app_impl/startup.rs](../src/app_impl/startup.rs)
- [src/app_navigation/impl_scroll.rs](../src/app_navigation/impl_scroll.rs)
- [src/app_impl/ui_window.rs](../src/app_impl/ui_window.rs)
- [src/main_window_preflight/build.rs](../src/main_window_preflight/build.rs)
- [src/app_impl/attachment_portal.rs](../src/app_impl/attachment_portal.rs)
- [src/actions/window.rs](../src/actions/window.rs)
- [src/confirm/window.rs](../src/confirm/window.rs)
- [src/components/inline_popup_window.rs](../src/components/inline_popup_window.rs)
- [src/ai/acp/popup_window.rs](../src/ai/acp/popup_window.rs)
- [src/ai/acp/picker_popup.rs](../src/ai/acp/picker_popup.rs)
- [src/components/inline_dropdown/row.rs](../src/components/inline_dropdown/row.rs)
- [src/components/shortcut_recorder/render.rs](../src/components/shortcut_recorder/render.rs)
- [src/app_impl/shortcut_recorder.rs](../src/app_impl/shortcut_recorder.rs)
- [src/components/launcher_ask_ai_hint.rs](../src/components/launcher_ask_ai_hint.rs)
- [src/prompts/select/prompt.rs](../src/prompts/select/prompt.rs)
- [src/prompts/select/render.rs](../src/prompts/select/render.rs)
- [src/prompts/select/types.rs](../src/prompts/select/types.rs)
- [src/components/unified_list_item/render.rs](../src/components/unified_list_item/render.rs)
- [src/render_builtins/sdk_reference.rs](../src/render_builtins/sdk_reference.rs)
- [src/render_builtins/script_templates.rs](../src/render_builtins/script_templates.rs)
- [src/components/prompt_layout_shell.rs](../src/components/prompt_layout_shell.rs)
- [src/prompts/path/render.rs](../src/prompts/path/render.rs)
- [src/prompts/env/render.rs](../src/prompts/env/render.rs)
- [src/form_prompt.rs](../src/form_prompt.rs)
- [src/components/form_fields/colors.rs](../src/components/form_fields/colors.rs)
- [src/components/form_fields/text_field/render.rs](../src/components/form_fields/text_field/render.rs)
- [src/components/form_fields/text_area/render.rs](../src/components/form_fields/text_area/render.rs)
- [src/components/form_fields/checkbox.rs](../src/components/form_fields/checkbox.rs)
- [src/editor/mod.rs](../src/editor/mod.rs)
- [src/render_prompts/editor.rs](../src/render_prompts/editor.rs)
- [src/app_layout/build_layout_info.rs](../src/app_layout/build_layout_info.rs)
- [src/stories/ask_tab_glyph_options.rs](../src/stories/ask_tab_glyph_options.rs)
- [src/storybook/dictation_ui_variations.rs](../src/storybook/dictation_ui_variations.rs)
- [src/storybook/browser.rs](../src/storybook/browser.rs)
- [src/storybook/context_picker_popup_playground/mod.rs](../src/storybook/context_picker_popup_playground/mod.rs)
- [src/platform/secondary_window_config.rs](../src/platform/secondary_window_config.rs)
- [src/platform/vibrancy_swizzle_materials.rs](../src/platform/vibrancy_swizzle_materials.rs)
- [src/theme/chrome.rs](../src/theme/chrome.rs)
- [src/theme/color_resolver.rs](../src/theme/color_resolver.rs)
- [src/theme/opacity.rs](../src/theme/opacity.rs)
- [src/ui_foundation/mod.rs](../src/ui_foundation/mod.rs)
- [vendor/gpui-component/crates/ui/src/root.rs](../vendor/gpui-component/crates/ui/src/root.rs)
- [vendor/gpui/src/window.rs](../vendor/gpui/src/window.rs)

## Related Pages

This page connects most directly to the windowing rules that implement the visual contract.

- [windowing](./windowing.md)
