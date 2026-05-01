# Surfaces

The launcher shell is a routed collection of surfaces rather than one generic prompt container.

`AppView` is the authoritative list of top-level launcher surfaces, and `render_impl.rs` is where those routes are rendered and focus-managed.

## Key Facts

These facts describe the stable routing and focus behavior across launcher surfaces.

- `AppView` covers script prompts, utility views, built-in browsers, ACP history, Notes browse, and the ACP chat surface.
- `AcpChatView` is the launcher's embedded ACP surface, while verification-bearing creation flows can still route to `QuickTerminalView`.
- Shared search-input synchronization spans many of the list-style surfaces, including script list, file search, clipboard history, emoji picker, settings, ACP history, and notes browse.
- Clipboard history and dictation history normalize wheel input at the list pane so mouse and trackpad scrolling keep moving even when GPUI's default scroll path misses those routed browser surfaces.
- Clipboard history, browser history, dictation history, and ACP history reanchor selection after handle movement so Enter and attachment shortcuts still act on a visible row even after scrollbar drag, track click, or native scroll-handle movement.
- The browser-tabs surface uses that same shared input contract, but keeps a compact Raycast-style single-column shell so the launcher can behave like a fast tab switcher instead of an expanded preview browser.
- Chat transcript surfaces are the opposite class: they are free-scroll panes with vendor scrollbars, so wheel input should update the scroll handle directly and then reconcile follow-tail state from the resulting offset.
- `CurrentAppCommandsView` is session-backed rather than list-only: it keeps the captured app PID and bundle identity beside the cached entries so filter changes can refresh the tray-opened surface and execution can fail closed instead of firing a stale menu action after an app switch or relaunch, while keyboard and click execution resolve against the original cached entry identity.
- File search keeps its empty left pane readable with a bounded info stack, so helper copy wraps inside the results column instead of stretching across the split view.
- The `~` main-menu trigger opens mini file search with directory rows seeded before first paint, then preserves them until the async directory stream replaces them.
- File search directory browsing tracks whether the current cache includes hidden entries; typing a dot-prefixed filter such as `~/.` or `dir/.` restreams that directory with dotfiles unlocked.
- File search row chrome follows the shared launcher row-state contract for selection and text hierarchy, while hover paint is gated through explicit `hovered_index` state that the header can clear during drag re-entry.
- File search rows avoid direct GPUI hover styling because native drag-out can leave row hitbox hover state stale; the shared filter header clears row hover and occludes mouse movement while hovering the input, and the regression audit pins this exception.
- File search rows deliberately avoid row hover tooltips because GPUI visible tooltips can survive later occlusion; the footer advertises Open/Actions instead.
- File search drag-out hands file rows to AppKit as native drags, defers GPUI active-drag cleanup until after the row `on_drag` callback finishes, clears any leftover GPUI `FileDragPayload` on drag-move, and restores focus from both the root and the occluding file-search header so clicking the input after drag-out reactivates Script Kit before the input handles the click.
- Focus and blur behavior are part of the surface contract. `render_impl.rs` handles dismiss-on-blur rules, pending focus application, popup coexistence, and shared footer synchronization.
- Some surfaces are prompt-hosted entities (`DivPrompt`, `FormPrompt`, `EditorPrompt`, `TermPrompt`), while others are view-state routes with their own cached data and selection state.

## Surface Contract Vocabulary

Launcher surfaces have shared words for family, input ownership, and preview role in the exhaustive surface registry.

The initial vocabulary lives in `src/main_sections/app_view_state.rs`:

- `LauncherSurfaceFamily` names why a route exists: `MainMenu`, `ScriptPrompt`, `FilterableLauncherList`, `UtilityWorkspace`, `AttachmentPortal`, `AssistantWorkspace`, or `FeedbackSurface`.
- `LauncherSurfaceInputOwnership` names who owns typed input: `LauncherFilter`, `PromptEntity`, `ChildView`, or `NoEditableInput`.
- `LauncherSurfacePreviewRole` names the expected detail shape: `NoPersistentPreview`, `OptionalInfoPanel`, `RequiredSplitPreview`, `ContentPane`, or `FeedbackPanel`.
- `LauncherSurfaceContractVocabulary` groups those dimensions so each `AppView` variant can declare its behavior in one place.

This vocabulary is deliberately behavior-oriented. Agents should describe a surface as "filterable launcher list with launcher-owned input" or "child-view content pane", not by guessing from render file names.

## Surface Contract Registry

`AppView::surface_contract()` is the single registry for top-level launcher behavior declarations.

The registry returns a `LauncherSurfaceContract` with the surface vocabulary, dismiss policy, and automation `semanticSurface` tag. It uses an exhaustive `match` with no wildcard arm, so new `AppView` variants must declare their behavior before the crate compiles.

`semantic_surface_for_main_view()` now delegates to that registry instead of carrying its own fallback map. This keeps stdin `triggerBuiltin` re-keying aligned with the same surface contract that owns dismiss behavior.

## About Surface

The About surface is a full-window launcher route for brand, version, update, and community links opened from the tray menu.

`AppView::About` is a `FeedbackSurface` with `NoEditableInput`, a `ContentPane` preview role, explicit dismissal, and automation semantic surface `about`. See [[about#About]] for layout and update-state details.

## Shared Actions Contract

Shared actions-dialog routing now resolves from one view-to-host map and closes through one host-aware path.

- `src/app_impl/actions_dialog.rs` owns the canonical surface map through `actions_host_for_view(...)` and keeps live popup routing on `live_actions_host_for_view(...)`.
- Generic `BuiltinList` surfaces remain in the static host map, but stay out of `live_actions_host_for_view(...)` until they provide selection-specific actions, so `Cmd+K` cannot open stale or global-only launcher actions for a built-in row. Theme Chooser is not generic: it owns a dedicated `ThemeChooser` actions host and catalog.
- `src/app_impl/startup.rs`, `src/render_builtins/theme_chooser.rs`, and `src/render_builtins/settings.rs` must route popup-owned keys through `route_key_to_actions_dialog(...)` before local shortcuts such as `Cmd+K`, `Escape`, `Cmd+W`, or theme tweaking chords.
- Actions-dialog shortcut dispatch resolves only against the current filtered action rows, so a hidden action cannot win a shortcut while the user is filtering toward another visible action.
- File search can combine selected-row file actions with current-directory actions; duplicate shortcut badges are removed from later actions so displayed shortcuts always match the action that will execute.
- Launcher `Cmd+K` opens now collapse onto `handle_cmd_k_actions_toggle(...)` after that popup-first routing check, so keyboard toggles and footer Actions clicks share the same dispatcher instead of drifting through `toggle_actions_for_host(...)`.
- In the launcher, plain `Up` and `Down` belong to the dedicated arrow interceptor once the actions popup is open. The later actions interceptor must yield those keys so one physical arrow press advances exactly one action row instead of double-stepping.
- `src/main_sections/render_impl.rs` closes the popup through `close_actions_popup_for_current_view(...)` so focus restoration, shared-filter resync, and pending explicit ACP target pickup stay aligned across backdrop-click and focus-regain closes.
- Launcher ACP close now restores the embedded composer through the `AcpChatView` focus handle. `ActionsDialogHost::AcpChat` maps to `FocusRequest::acp_chat()`, so the shared close path restores the ACP-owned surface instead of flattening back to the generic chat prompt target.
- `src/app_impl/tab_ai_mode/mod.rs` seeds ACP return origin from the live launcher surface before shared actions handoff, plain `Tab`, or global `Cmd+Enter` opens ACP, so ACP close can restore the originating routed view and shared filter focus instead of falling back to generic launcher state.
- `src/render_builtins/file_search.rs`, `src/render_builtins/clipboard.rs`, and `src/render_builtins/emoji_picker.rs` execute shared dialog selections through `execute_action_for_actions_host(...)` so host-specific execution stays consistent.
- `src/app_impl/actions_dialog.rs::was_actions_recently_closed` is the 300ms debounce that suppresses Cmd+K when the footer click raced the activation observer's deferred close. `tests/was_actions_recently_closed_debounce_contract.rs` pins the signature `(&self) -> bool`, the `Duration::from_millis(300)` window, the exact `t.elapsed() < ACTIONS_CLOSE_DEBOUNCE` comparator, the `self.actions_closed_at` field read, and the three rationale anchors (`300ms`, `footer ⌘K button`, `activation observer`) against a `RecentCloseDebouncer` extraction that would consolidate with the sibling ACP-history debounce and silently drop the 300ms literal or the anchor comment.
- Every actions-popup OPEN path must clear `self.actions_closed_at = None;` (debounce reset). The canonical helper `begin_actions_popup_window_open(cx, window)` at `src/app_impl/actions_toggle.rs:256` carries the `// Clear debounce on open` comment, and the three built-in `toggle_*_actions` OPEN branches in `src/render_builtins/actions.rs` (`toggle_dictation_history_actions` line 82, `toggle_file_search_actions` line 199, `toggle_clipboard_actions` line 359) each mirror that line. Without this clear, a stale `actions_closed_at = Some(T-old)` from an earlier close would persist across a built-in open and later cause `was_actions_recently_closed()` to suppress an otherwise-valid Cmd+K reopen.
- Every actions-popup CLOSE path that inlines its own teardown (instead of delegating to the canonical `close_actions_popup(host, window, cx)` in `src/app_impl/actions_dialog.rs`) must record `self.actions_closed_at = Some(std::time::Instant::now());` immediately after setting `self.show_actions_popup = false;`. The canonical helper sets it at `src/app_impl/actions_dialog.rs:653`; the two inline close branches in `src/render_builtins/actions.rs` — `toggle_file_search_actions` (line 153) and `toggle_clipboard_actions` (line 337) — mirror that line with the `// Record debounce on close` comment. `toggle_dictation_history_actions` already delegates to the canonical helper at line 77, so dictation's close debounce is set inside that helper. Without this record, a subsequent `was_actions_recently_closed()` call would return `false` even within 300ms of a real close, defeating the footer-⌘K-vs-activation-observer race guard for these two built-in hosts.

## Global Key Intent Routing
Main-window global shortcuts should be classified into named intents before dispatch so shortcut behavior has an explicit owner.

AURP-08 starts with [[src/app_impl/startup.rs#MainWindowGlobalKeyIntent]], which names the global Cmd+Enter behavior as `OpenAcpWithCurrentContext` before dispatch. The live interceptor calls `main_window_global_key_intent(event)` and then `handle_main_window_global_key_intent(...)`, keeping the modifier gate separate from the ACP routing effect.

[[tests/main_window_global_key_intent_contract.rs#cmd_enter_to_acp_is_classified_as_a_named_global_key_intent]] pins this first intent so future keyboard refactors keep the Cmd+Enter gate explicit and behavior-named.

AURP-10 extends the pattern to [[src/app_impl/startup.rs#MainWindowActionsKeyIntent]], naming actions-interceptor `Cmd+K` and embedded ACP `Cmd+W` behavior while preserving shared actions-dialog routing first.

[[tests/main_window_actions_key_intent_contract.rs#actions_interceptor_routes_shared_dialog_before_local_intents]] pins the popup-first order so local key intent dispatch cannot consume keys before the shared actions dialog declines them.

## Dismiss Policy Contract

Per-view dismiss behavior is declared by `AppView::surface_contract()` in [src/main_sections/app_view_state.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/app_view_state.rs), not by a negative match in the HUD/grid file.

`DismissPolicy` pairs a `DismissTrigger` (`WindowBlur`, `BackdropClick`, `Escape`, `CmdW`) with a `DismissEffect` (`Ignore`, `CloseMainWindow`, `LetViewHandle`) for each surface. `AppView::dismiss_policy()` delegates to the registry; the exhaustive registry match has no `_ =>` arm and `DismissPolicy` has no `Default` impl, so rustc refuses to compile when a new `AppView` variant is added without an explicit behavior decision.

`is_dismissable_view()` in `src/app_impl/shortcuts_hud_grid.rs` is now a three-line delegate to `self.current_view.dismiss_policy().closes_main_window_on(DismissTrigger::WindowBlur)`. The focus-lost block in `render_impl.rs` keeps every runtime coexistence guard (pinned, focus grace, actions popup, confirm popup, shortcut recorder popup, detached ACP, dictation, Tab AI) — those are not per-view policy and should stay on their own paths.

Source-level audits in [tests/app_view_policy_contract.rs](/Users/johnlindquist/dev/script-kit-gpui/tests/app_view_policy_contract.rs) pin the guarantees that rustc alone cannot: no wildcard arm in the registry, no semantic-surface fallback map, no `Default` escape hatch on `DismissPolicy`, and sticky blur behavior for ThemeChooser. Confirm / actions / dictation popup dismissal remains runtime overlay state and keeps its own close paths.

## Key Files

These files define the routed launcher surfaces and how they are entered.

- [src/main_sections/app_view_state.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/app_view_state.rs) - Full `AppView` enum, focus targets, actions-dialog host types, and related surface state enums.
- [src/main_sections/render_impl.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/render_impl.rs) - Surface rendering, focus application, blur dismissal, and shared input synchronization.
- [src/app_impl/actions_dialog.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/actions_dialog.rs) - Shared actions-dialog host resolution, popup routing, focus restore, and close semantics.
- [src/app_impl/startup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/startup.rs) - Main-window global key interception for popup routing and Cmd+K toggle handling.
- [src/app_execute/utility_views.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/utility_views.rs) - Openers for utility surfaces such as file search and quick terminal.
- [src/render_builtins/file_search.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/file_search.rs) - File search pane layout, empty-state messaging, and preview rendering.
- [src/render_builtins/theme_chooser.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/theme_chooser.rs) - Theme chooser shell and popup-first shortcut routing.
- [src/render_builtins/settings.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/settings.rs) - Settings list shell and popup-first shortcut routing.
- [src/render_builtins/browser_tabs.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/browser_tabs.rs) - Browser tab switcher layout, shared filter handling, and activation actions.
- [src/app_impl/tab_ai_mode/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/tab_ai_mode/mod.rs) - ACP and PTY-handoff routing.

## Source Documents

These source files back the surface model described on this page.

- [src/main_sections/app_view_state.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/app_view_state.rs)
- [src/main_sections/render_impl.rs](/Users/johnlindquist/dev/script-kit-gpui/src/main_sections/render_impl.rs)
- [src/app_impl/actions_dialog.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/actions_dialog.rs)
- [src/app_impl/startup.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/startup.rs)
- [src/app_execute/utility_views.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_execute/utility_views.rs)
- [src/render_builtins/file_search.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/file_search.rs)
- [src/render_builtins/theme_chooser.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/theme_chooser.rs)
- [src/render_builtins/settings.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/settings.rs)
- [src/render_builtins/browser_tabs.rs](/Users/johnlindquist/dev/script-kit-gpui/src/render_builtins/browser_tabs.rs)
- [src/app_impl/tab_ai_mode/mod.rs](/Users/johnlindquist/dev/script-kit-gpui/src/app_impl/tab_ai_mode/mod.rs)

## Related Pages

These pages cover the built-ins, Notes host, and ACP flows that share the launcher shell.

- [architecture](./architecture.md)
- [builtins](./builtins.md)
- [notes](./notes.md)
- [acp-chat](./acp-chat.md)

## Current Surface Families

- launcher list surfaces: `ScriptList`, clipboard history, app launcher, window switcher, browser tabs, favorites, settings, current-app commands
- prompt entities: arg, div, form, editor, term, select, path, env, drop, template, chat, webcam, naming
- utility views: file search, scratch pad, quick terminal, design gallery, theme chooser, emoji picker
- AI and context surfaces: ACP chat, ACP history, notes browse

That grouping is more useful than the old one-doc-per-prompt sprawl because it matches the current routed app model.

## Expanded Exceptions

Expanded split views are exceptions, not the launcher default.

New command surfaces should start from the main-menu shell and its focused-info toggle instead of jumping straight to a 50/50 list-and-preview browser. The expanded scaffold is reserved for cases where the preview is necessary to decide, such as file contents, clipboard payloads, saved dictations, notes, or ACP transcripts.

That means scaffold choice is part of the surface contract:

- `ScriptList` is the default launcher pattern, with the info panel revealed explicitly through `show_info_panel`
- `PromptChromeAudit::expanded(...)` should only appear on preview-dense surfaces
- `FileSearchPresentation` is an explicit exception because it supports both compact entry from `ScriptList` and a full browser mode
