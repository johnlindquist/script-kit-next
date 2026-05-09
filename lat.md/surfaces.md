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
- `LauncherSurfaceFocusPolicy` names the expected focus owner: `LauncherFilterFocus`, `PromptEntityFocus`, `ChildViewFocus`, or `NoEditableFocus`.
- `LauncherSurfaceKeyboardPolicy` names the expected keyboard owner: `LauncherListKeyboard`, `PromptEntityKeyboard`, `ChildViewKeyboard`, `ActionsDialogKeyboard`, or `NoEditableKeyboard`.
- `LauncherSurfaceActionsPolicy` names the expected actions owner: `MainMenuActions`, `HostRowActions`, `PromptEntityActions`, `ChildViewActions`, `ActionsDialogActions`, or `NoSurfaceActions`.
- `LauncherSurfaceProofPolicy` names the expected first proof: `StateReceiptProof`, `StateAndElementsProof`, `ChildViewStateProof`, or `PopupStateProof`.
- `LauncherSurfaceVisualPolicy` names the expected visual shape: `CompactLauncherVisual`, `SplitPreviewVisual`, `ContentPaneVisual`, `PopupVisual`, or `FeedbackVisual`.
- `LauncherSurfaceContractVocabulary` groups those dimensions so each `AppView` variant can declare its behavior in one place.

This vocabulary is deliberately behavior-oriented. Agents should describe a surface as "filterable launcher list with launcher-owned input" or "child-view content pane", not by guessing from render file names.

## Surface Contract Registry

`SurfaceKind` is the payload-free identity layer for top-level launcher behavior declarations.

`AppView` still owns runtime payloads, focus handles, and child entities. `AppView::surface_kind()` maps each payload-bearing route to a stable `SurfaceKind`; file search intentionally splits mini and full presentation into separate kinds because they carry different preview contracts.

`SurfaceKind::surface_contract()` returns a `LauncherSurfaceContract` with the surface vocabulary, focus policy, keyboard policy, actions policy, proof policy, visual policy, dismiss policy, and automation `semanticSurface` tag. Both `AppView::surface_kind()` and `SurfaceKind::surface_contract()` use exhaustive matches with no wildcard arm, so new `AppView` or `SurfaceKind` variants must declare their behavior before the crate compiles.

`AppView::surface_contract()` is now a thin delegate through `self.surface_kind().surface_contract()`. Agents and proof matrices should key behavior expectations on `SurfaceKind` instead of reverse-engineering enum payloads or render files.

Main-window `getState` receipts expose the same contract through `stateResult.surfaceContract`. This lets agents compare live state receipts to `docs/ai/contracts/surface-contracts.json` without inferring behavior from legacy `promptType` strings.

`semantic_surface_for_main_view()` now delegates to that registry instead of carrying its own fallback map. This keeps stdin `triggerBuiltin` re-keying aligned with the same surface contract that owns dismiss behavior.

`ScriptListApp::rekey_main_automation_surface_from_current_view()` is the owner for main-window automation re-keying when a route has already changed `current_view`. Routed surfaces should call that helper instead of pairing `semantic_surface_for_main_view(&self.current_view)` with a raw registry write.

## Current-View Transition Owner

Some route transitions pair the `current_view` assignment with a main-window automation re-key.

`ScriptListApp::transition_current_view_and_rekey_main_automation_surface()` assigns the next `AppView`, then delegates to `rekey_main_automation_surface_from_current_view()`. About and parent-confirm routes use this owner so agents cannot split the AppView mutation from the semantic-surface update during refactors.

Embedded ACP entry has its own route owner because it must synchronize more than the main semantic surface. `ScriptListApp::enter_embedded_acp_chat_surface()` assigns `AppView::AcpChatView`, upserts the embedded `kind:"ai"` automation window, re-keys main from the active view, emits `EmbeddedOpened`, clears transient actions state, and targets chat prompt focus in one ordered block.

Return-view restoration has a separate owner. `ScriptListApp::restore_current_view_with_focus()` assigns the captured `AppView`, translates the captured `FocusTarget` into legacy `FocusedInput`, and leaves route-specific side effects such as automation re-keying, ACP teardown, window sizing, and `cx.notify()` at the caller.

ScriptList main-filter entries use `ScriptListApp::show_script_list_with_main_filter_focus()`. The helper delegates to `restore_current_view_with_focus(AppView::ScriptList, FocusTarget::MainFilter)`, then re-keys the main automation semantic surface from the restored `ScriptList` view while callers keep their own filter text, list cache, sizing, and notification work visible.

`AppView::native_footer_surface()` owns the exact native footer surface id for each view. It intentionally stays AppView-specific rather than SurfaceKind-specific because some grouped surface kinds still need distinct footer ids for prompt slots and specialized built-ins; the generated surface matrix exposes these per-variant footer ids.

## Agent-Readable Surface Contract Matrix

The generated surface contract matrix gives agents a checked JSON view of the typed Rust registry.

`scripts/generate-surface-contracts.ts` parses `AppView::surface_kind()`, `AppView::native_footer_surface()`, and `SurfaceKind::surface_contract()` from [src/main_sections/app_view_state.rs](src/main_sections/app_view_state.rs), then writes [surface-contracts.json](docs/ai/contracts/surface-contracts.json). Each entry lists the `SurfaceKind`, mapped `AppView` variants, per-variant native footer ids, vocabulary tuple, focus policy, keyboard policy, actions policy, proof policy, visual policy, dismiss effects, and automation semantic surface.

The JSON artifact is intentionally generated, not hand-authored. `tests/surface_contract_matrix_artifact_contract.rs` runs the generator in `--check` mode, requires every `SurfaceKind` to appear exactly once, and verifies the behavior fields agents need are present.

## Agent-Readable Current-View Transition Inventory

The generated transition inventory exposes `current_view` mutation sites and named transition-helper calls while they are migrated behind route APIs.

`scripts/generate-current-view-transitions.ts` scans `src/app_actions`, `src/app_execute`, `src/app_impl`, `src/main_entry`, `src/main_sections`, and `src/prompt_handler`, then writes [current-view-transitions.json](docs/ai/contracts/current-view-transitions.json). Each entry lists the file, line, owner function, receiver, operation, expression, inferred target, and whether the transition is dynamic enough to need manual review; named transition-helper calls also expose the helper name and contract flags such as main re-keying, focus target, embedded-AI upsert, and actions cleanup.

Named helper entries also carry checked `transitionContract` metadata for main re-keying, focus target/focused input, embedded AI upsert, actions cleanup, resize ownership, and state snapshot proof. This metadata is a source-derived transition contract, not a parallel router or public runtime receipt; runtime proof still flows through `getState.surfaceContract` and `activePopupContract`.

The inventory is deliberately source-derived. It helps agents find remaining transition owners without trusting grep output or stale memory, and `tests/current_view_transition_inventory_contract.rs` keeps the artifact from drifting.

## About Surface

The About surface is a full-window launcher route for brand, version, update, and community links opened from the tray menu.

`AppView::About` is a `FeedbackSurface` with `NoEditableInput`, a `ContentPane` preview role, explicit dismissal, and automation semantic surface `about`. See [[about#About]] for layout and update-state details.

## Shared Actions Contract

Shared actions-dialog routing now resolves from one view-to-host map and closes through one host-aware path.

- `src/app_impl/actions_dialog.rs` owns the canonical surface map through `actions_host_for_view(...)` and keeps live popup routing on `live_actions_host_for_view(...)`.
- Generic `BuiltinList` surfaces remain in the static host map, but stay out of `live_actions_host_for_view(...)` until they provide selection-specific actions, so `Cmd+K` cannot open stale or global-only launcher actions for a built-in row. Theme Chooser is not generic: it owns a dedicated `ThemeChooser` actions host and catalog.
- `src/app_impl/startup.rs`, `src/render_builtins/theme_chooser.rs`, `src/render_builtins/settings.rs`, and `src/render_builtins/current_app_commands.rs` must route popup-owned keys through `route_key_to_actions_dialog(...)` before local shortcuts such as `Cmd+K`, `Escape`, `Cmd+W`, or theme tweaking chords.
- Actions-dialog shortcut dispatch resolves only against the current filtered action rows, so a hidden action cannot win a shortcut while the user is filtering toward another visible action.
- File search can combine selected-row file actions with current-directory actions; duplicate shortcut badges are removed from later actions so displayed shortcuts always match the action that will execute.
- Launcher `Cmd+K` opens now collapse onto `handle_cmd_k_actions_toggle(...)` after that popup-first routing check, so keyboard toggles and footer Actions clicks share the same dispatcher instead of drifting through `toggle_actions_for_host(...)`.
- In the launcher, plain `Up` and `Down` belong to the dedicated arrow interceptor once the actions popup is open. The later actions interceptor must yield those keys so one physical arrow press advances exactly one action row instead of double-stepping.
- `src/main_sections/render_impl.rs` closes the popup through `close_actions_popup_for_current_view(...)` so focus restoration, shared-filter resync, and pending explicit ACP target pickup stay aligned across backdrop-click and focus-regain closes.
- Launcher ACP close now restores the embedded composer through the `AcpChatView` focus handle. `ActionsDialogHost::AcpChat` maps to `FocusRequest::acp_chat()`, so the shared close path restores the ACP-owned surface instead of flattening back to the generic chat prompt target.
- `src/app_impl/tab_ai_mode/mod.rs` seeds ACP return origin from the live launcher surface before shared actions handoff, plain `Tab`, or global `Cmd+Enter` opens ACP, so ACP close can restore the originating routed view and shared filter focus instead of falling back to generic launcher state.
- `src/render_builtins/file_search.rs`, `src/render_builtins/clipboard.rs`, and `src/render_builtins/emoji_picker.rs` execute shared dialog selections through `execute_action_for_actions_host(...)` so host-specific execution stays consistent.
- Filterable launcher-list rows select on a first unselected click and activate only on a selected-row click, double-click, or Enter. Handled row clicks consume propagation so the focused surface owns activation.
- `src/app_impl/actions_dialog.rs::was_actions_recently_closed` is the 300ms debounce that suppresses Cmd+K when the footer click raced the activation observer's deferred close. `tests/was_actions_recently_closed_debounce_contract.rs` pins the signature `(&self) -> bool`, the `Duration::from_millis(300)` window, the exact `t.elapsed() < ACTIONS_CLOSE_DEBOUNCE` comparator, the `self.actions_closed_at` field read, and the three rationale anchors (`300ms`, `footer ⌘K button`, `activation observer`) against a `RecentCloseDebouncer` extraction that would consolidate with the sibling ACP-history debounce and silently drop the 300ms literal or the anchor comment.
- Every actions-popup OPEN path must call `mark_actions_popup_opening()`, whose body sets `self.show_actions_popup = true;` and clears `self.actions_closed_at = None;` with the `// Clear debounce on open` comment. Without this clear, a stale `actions_closed_at = Some(T-old)` from an earlier close would persist across a built-in open and later cause `was_actions_recently_closed()` to suppress an otherwise-valid Cmd+K reopen.
- `clear_actions_popup_state()` owns non-debounced popup cleanup for route changes, resets, and stale-overlay removal. It clears both `show_actions_popup` and `actions_dialog` without touching `actions_closed_at`, so cleanup paths cannot accidentally create a recent-close debounce.
- Every actions-popup CLOSE path that inlines its own teardown (instead of delegating to the canonical `close_actions_popup(host, window, cx)` in `src/app_impl/actions_dialog.rs`) must call `mark_actions_popup_closed()`, whose body delegates to `clear_actions_popup_state()` and records `self.actions_closed_at = Some(std::time::Instant::now());` with the `// Record debounce on close` comment. Without this record, a subsequent `was_actions_recently_closed()` call would return `false` even within 300ms of a real close, defeating the footer-⌘K-vs-activation-observer race guard.
- Detached MainList actions external close may hide the parent only for passive ScriptList focus loss. The callback marks actions closed and pops the focus overlay before preserving ScriptList state, then skips normal focus restoration so actions do not reopen on restore.
- Detached actions windows must also close from their render fallback when both the parent main window and actions window are inactive, because AppKit activation observation can miss desktop click-away for never-key popups.
- Main-window `getState` receipts expose the host `surfaceContract` and, when the shared actions popup is attached, an `activePopupContract` for `SurfaceKind::ActionsDialog`. This keeps overlay keyboard/actions/proof policy machine-readable without changing the host `AppView`.
- Agentic screenshot-library coverage treats attached popups as both popup surfaces and host-dependent surfaces. Hosted Actions Dialog cases enter the host through its surface contract before shared Cmd+K, while the ACP slash Prompt Popup enters Agent Chat and opens through protocol `setAcpInput "/"`; both require `parent_capture_with_crop` proof afterward, and Prompt Popup promotion must resolve the expected `acp-mention-popup` automation id.

## Shared Timestamp Formatting

User-facing launcher timestamps should be readable and consistent across routed surfaces.

`src/formatting.rs` is the shared display layer for relative and absolute timestamp text. Relative labels route through `chrono-humanize` so clipboard history, file search, notes, browser history, ACP chat, env prompts, and process rows avoid local abbreviations such as `5m ago` or raw sortable formats. Storage, cache invalidation, and protocol identity timestamps still keep raw epoch or RFC3339 forms where machine precision matters.

## Global Key Intent Routing
Main-window global shortcuts should be classified into named intents before dispatch so shortcut behavior has an explicit owner.

AURP-08 starts with [[src/app_impl/startup.rs#MainWindowGlobalKeyIntent]], which names the global Cmd+Enter behavior as `OpenAcpWithCurrentContext` before dispatch. The live interceptor calls `main_window_global_key_intent(event)` and then `handle_main_window_global_key_intent(...)`, keeping the modifier gate separate from the ACP routing effect.

[[tests/main_window_global_key_intent_contract.rs#cmd_enter_to_acp_is_classified_as_a_named_global_key_intent]] pins this first intent so future keyboard refactors keep the Cmd+Enter gate explicit and behavior-named.

AURP-10 extends the pattern to [[src/app_impl/startup.rs#MainWindowActionsKeyIntent]], naming actions-interceptor `Cmd+K` and embedded ACP `Cmd+W` behavior while preserving shared actions-dialog routing first.

[[tests/main_window_actions_key_intent_contract.rs#actions_interceptor_routes_shared_dialog_before_local_intents]] pins the popup-first order so local key intent dispatch cannot consume keys before the shared actions dialog declines them.

## Dismiss Policy Contract

Per-view dismiss behavior is declared by `SurfaceKind::surface_contract()` in [src/main_sections/app_view_state.rs](src/main_sections/app_view_state.rs), not by a negative match in the HUD/grid file.

`DismissPolicy` pairs a `DismissTrigger` (`WindowBlur`, `BackdropClick`, `Escape`, `CmdW`) with a `DismissEffect` (`Ignore`, `CloseMainWindow`, `LetViewHandle`) for each surface. `AppView::dismiss_policy()` delegates through `AppView::surface_contract()` to the `SurfaceKind` registry; the exhaustive identity and registry matches have no `_ =>` arms and `DismissPolicy` has no `Default` impl, so rustc refuses to compile when a new route or kind is added without an explicit behavior decision.

`is_dismissable_view()` in `src/app_impl/shortcuts_hud_grid.rs` is now a three-line delegate to `self.current_view.dismiss_policy().closes_main_window_on(DismissTrigger::WindowBlur)`. The focus-lost block in `render_impl.rs` keeps every runtime coexistence guard (pinned, focus grace, actions popup, confirm popup, shortcut recorder popup, detached ACP, dictation, Tab AI) — those are not per-view policy and should stay on their own paths.

Source-level audits in [tests/app_view_policy_contract.rs](tests/app_view_policy_contract.rs) pin the guarantees that rustc alone cannot: no wildcard arm in the `AppView` identity map, no wildcard arm in the `SurfaceKind` contract registry, no semantic-surface fallback map, no `Default` escape hatch on `DismissPolicy`, and sticky blur behavior for ThemeChooser. Confirm / actions / dictation popup dismissal remains runtime overlay state and keeps its own close paths.

## Key Files

These files define the routed launcher surfaces and how they are entered.

- [src/main_sections/app_view_state.rs](src/main_sections/app_view_state.rs) - Full `AppView` enum, focus targets, actions-dialog host types, and related surface state enums.
- [src/main_sections/render_impl.rs](src/main_sections/render_impl.rs) - Surface rendering, focus application, blur dismissal, and shared input synchronization.
- [src/app_impl/actions_dialog.rs](src/app_impl/actions_dialog.rs) - Shared actions-dialog host resolution, popup routing, focus restore, and close semantics.
- [src/app_impl/startup.rs](src/app_impl/startup.rs) - Main-window global key interception for popup routing and Cmd+K toggle handling.
- [src/app_execute/utility_views.rs](src/app_execute/utility_views.rs) - Openers for utility surfaces such as file search and quick terminal.
- [src/render_builtins/file_search.rs](src/render_builtins/file_search.rs) - File search pane layout, empty-state messaging, and preview rendering.
- [src/render_builtins/theme_chooser.rs](src/render_builtins/theme_chooser.rs) - Theme chooser shell and popup-first shortcut routing.
- [src/render_builtins/settings.rs](src/render_builtins/settings.rs) - Settings list shell and popup-first shortcut routing.
- [src/render_builtins/browser_tabs.rs](src/render_builtins/browser_tabs.rs) - Browser tab switcher layout, shared filter handling, and activation actions.
- [src/app_impl/tab_ai_mode/mod.rs](src/app_impl/tab_ai_mode/mod.rs) - ACP and PTY-handoff routing.

## Source Documents

These source files back the surface model described on this page.

- [src/main_sections/app_view_state.rs](src/main_sections/app_view_state.rs)
- [src/main_sections/render_impl.rs](src/main_sections/render_impl.rs)
- [src/app_impl/actions_dialog.rs](src/app_impl/actions_dialog.rs)
- [src/app_impl/startup.rs](src/app_impl/startup.rs)
- [src/app_execute/utility_views.rs](src/app_execute/utility_views.rs)
- [src/render_builtins/file_search.rs](src/render_builtins/file_search.rs)
- [src/render_builtins/theme_chooser.rs](src/render_builtins/theme_chooser.rs)
- [src/render_builtins/settings.rs](src/render_builtins/settings.rs)
- [src/render_builtins/browser_tabs.rs](src/render_builtins/browser_tabs.rs)
- [src/app_impl/tab_ai_mode/mod.rs](src/app_impl/tab_ai_mode/mod.rs)

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

## Selection-Owned Expanded Browsers

Preview browsers keep one selected row synchronized with the visible list, preview pane, and portal attach target.

Notes Browse is preview-dense and uses the expanded scaffold because reading note content is required before attaching it to ACP. In portal mode, Enter attaches the selected note and Escape cancels the portal before clearing any filter. Wheel movement, scrollbar movement, and row clicks must update `selected_index` so the list, preview, and attached note cannot drift apart.

Uniform-list built-in wheel handlers must not immediately call [[src/render_builtins/common.rs#ScriptListApp#builtin_reanchor_selection_from_scroll]] after scheduling `scroll_to_item(new_selected, ScrollStrategy::Nearest)`. Wheel events already own `selected_index`; render-time reanchor is reserved for settled scrollbar/native-scroll state on views that intentionally opt into it. Current App Commands follows the main-menu list contract: keyboard, wheel, and row clicks own `selected_index`, and render must not reanchor that selection from scrollbar metrics. Selection-owned wheel handlers also record the exact wheel-selected index through [[src/render_builtins/common.rs#ScriptListApp#note_builtin_selection_owned_wheel_scroll]], and render-time reanchor ignores that index so stale or underestimated scrollbar metrics cannot snap the selected command back to row 0 after the user stops scrolling. Pinned by [[tests/builtin_wheel_reanchor_contract.rs#uniform_list_wheel_handlers_do_not_immediately_reanchor_deferred_scroll]] and [[tests/builtin_wheel_reanchor_contract.rs#current_app_commands_render_does_not_reanchor_selection_from_scroll]] across Current App Commands, Browser Tabs, Window Switcher, Process Manager, Clipboard History, App Launcher, and Kit Store lists.

Opening Current App Commands resets the shared list scroll handle to row 0 along with `selected_index: 0`, so stale deferred or live offsets from a previous session cannot leave the selected first row offscreen. Pinned by [[tests/current_app_commands.rs#current_app_commands_presentation_resets_scroll_to_top]].

Shared uniform-list scrollbar metrics must prefer a pending deferred `scroll_to_item` target over the live scroll offset even after the list has measured. GPUI applies deferred scroll during the next prepaint, so the scrollbar thumb should display the pending target instead of the previous live viewport; that pending offset is not settled selection truth for Current App Commands. Pinned by [[tests/builtin_wheel_reanchor_contract.rs#scrollbar_metrics_prefer_pending_deferred_scroll_for_thumb_position]] and the inline `preferred_scroll_offset` tests in [[src/components/scrollbar.rs]].
