# Keyboard Focus Architecture Expert Bundle

## Original Goal

> everything related to keyboard focus. I need to ask an expert if we should make some sort of focus library or something that would be more robust for our needs

## Executive Summary

Script Kit GPUI has complex, ad-hoc focus management. Key patterns:

1. **pending_focus mechanism**: Avoids "perpetual focus in render()" anti-pattern
2. **Multiple enums**: `FocusedInput`, `FocusTarget`, `ActionsDialogHost` track state  
3. **Per-component boilerplate**: Every prompt has `focus_handle` + `impl Focusable` + `track_focus()`
4. **Unused shell abstraction**: `app_shell/focus.rs` defines `FocusPolicy`/`ShellFocus` but main app ignores it

### Questions for Expert:
1. Should we create a centralized FocusManager that owns all handles?
2. Would declarative focus policies reduce transition bugs?
3. How does Zed handle focus between panes? Pattern to adopt?
4. Should form parent own focus or delegate to children?

---
## Core Focus System Code

This file is a merged representation of the filtered codebase, combined into a single document by packx.

<file_summary>
This section contains a summary of this file.

<purpose>
This file contains a packed representation of filtered repository contents.
It is designed to be easily consumable by AI systems for analysis, code review,
or other automated processes.
</purpose>

<usage_guidelines>
- Treat this file as a snapshot of the repository's state
- Be aware that this file may contain sensitive information
</usage_guidelines>

<notes>
- Files were filtered by packx based on content and extension matching
- Total files included: 4
- Context lines: 25 lines around each match
</notes>
</file_summary>

<directory_structure>
src/app_impl.rs
src/form_prompt.rs
src/main.rs
src/app_shell/focus.rs
</directory_structure>

<files>
This section contains the contents of the repository's files.

<file path="src/app_impl.rs" matches="90" windows="12">
   128|                         }
   129|                         Err(std::sync::mpsc::TryRecvError::Empty) => continue,
   130|                         Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
   131|                     }
   132|                 }
   133|             })
   134|             .detach();
   135|         }
   136|         logging::log("UI", "Script Kit logo SVG loaded for header rendering");
   137| 
   138|         // Start cursor blink timer - updates all inputs that track cursor visibility
   139|         cx.spawn(async move |this, cx| {
   140|             loop {
   141|                 Timer::after(std::time::Duration::from_millis(530)).await;
   142|                 let _ = cx.update(|cx| {
   143|                     this.update(cx, |app, cx| {
   144|                         // Skip cursor blink when:
   145|                         // 1. Window is hidden (no visual feedback needed)
   146|                         // 2. No window is focused (main window OR actions popup)
   147|                         // 3. No input is focused (no cursor to blink)
   148|                         let actions_popup_open = is_actions_window_open();
   149|                         let any_window_focused =
   150|                             platform::is_main_window_focused() || actions_popup_open;
   151|                         if !script_kit_gpui::is_main_window_visible()
   152|                             || !any_window_focused
   153|                             || app.focused_input == FocusedInput::None
   154|                         {
   155|                             return;
   156|                         }
   157| 
   158|                         app.cursor_visible = !app.cursor_visible;
   159|                         // Also update ActionsDialog cursor if it exists
   160|                         if let Some(ref dialog) = app.actions_dialog {
   161|                             dialog.update(cx, |d, _cx| {
   162|                                 d.set_cursor_visible(app.cursor_visible);
   163|                             });
   164|                             // Notify the actions window to repaint with new cursor state
   165|                             notify_actions_window(cx);
   166|                         }
   167|                         // Also update AliasInput cursor if it exists
   168|                         if let Some(ref alias_input) = app.alias_input_entity {
   169|                             alias_input.update(cx, |input, _cx| {
   170|                                 input.set_cursor_visible(app.cursor_visible);
   171|                             });
   172|                         }
   173|                         cx.notify();
   174|                     })
   175|                 });
   176|             }
   177|         })
   178|         .detach();
   179| 
   180|         let gpui_input_state =
   181|             cx.new(|cx| InputState::new(window, cx).placeholder(DEFAULT_PLACEHOLDER));
   182|         let gpui_input_subscription = cx.subscribe_in(&gpui_input_state, window, {
   183|             move |this, _, event: &InputEvent, window, cx| match event {
   184|                 InputEvent::Focus => {
   185|                     this.gpui_input_focused = true;
   186|                     this.focused_input = FocusedInput::MainFilter;
   187| 
   188|                     // Close actions popup when main input receives focus
   189|                     // This ensures consistent behavior: clicking the input closes actions
   190|                     // just like pressing Cmd+K would
   191|                     if this.show_actions_popup || is_actions_window_open() {
   192|                         logging::log(
   193|                             "FOCUS",
   194|                             "Main input focused while actions open - closing actions (same as Cmd+K)",
   195|                         );
   196|                         this.show_actions_popup = false;
   197|                         this.actions_dialog = None;
   198|                         // Close the actions window
   199|                         cx.spawn(async move |_this, cx| {
   200|                             cx.update(|cx| {
   201|                                 close_actions_window(cx);
   202|                             })
   203|                             .ok();
   204|                         })
   205|                         .detach();
   206|                     }
   207| 
   208|                     cx.notify();
   209|                 }
   210|                 InputEvent::Blur => {
   211|                     this.gpui_input_focused = false;
   212|                     if this.focused_input == FocusedInput::MainFilter {
   213|                         this.focused_input = FocusedInput::None;
   214|                     }
   215|                     cx.notify();
   216|                 }
   217|                 InputEvent::Change => {
   218|                     let input_received_at = std::time::Instant::now();
   219|                     // Read the current input value to see what we're processing
   220|                     let current_value = this.gpui_input_state.read(cx).value().to_string();
   221|                     logging::log(
   222|                         "FILTER_PERF",
   223|                         &format!(
   224|                             "[1/5] INPUT_CHANGE value='{}' len={} at {:?}",
   225|                             current_value,
   226|                             current_value.len(),
   227|                             input_received_at
   228|                         ),
   229|                     );
   230|                     this.filter_perf_start = Some(input_received_at);
   231|                     this.handle_filter_input_change(window, cx);
   232|                 }
   233|                 InputEvent::PressEnter { .. } => {
   234|                     if matches!(this.current_view, AppView::ScriptList) && !this.show_actions_popup
   235|                     {
   236|                         // Check if we're in fallback mode first
   237|                         if this.fallback_mode && !this.cached_fallbacks.is_empty() {
   238|                             this.execute_selected_fallback(cx);

  ...
   285|             last_scroll_time: None,
   286|             current_view: AppView::ScriptList,
   287|             script_session: Arc::new(ParkingMutex::new(None)),
   288|             arg_input: TextInputState::new(),
   289|             arg_selected_index: 0,
   290|             prompt_receiver: None,
   291|             response_sender: None,
   292|             // Variable-height list state for main menu (section headers at 24px, items at 48px)
   293|             // Start with 0 items, will be reset when grouped_items changes
   294|             // .measure_all() ensures all items are measured upfront for correct scroll height
   295|             main_list_state: ListState::new(0, ListAlignment::Top, px(100.)).measure_all(),
   296|             list_scroll_handle: UniformListScrollHandle::new(),
   297|             arg_list_scroll_handle: UniformListScrollHandle::new(),
   298|             clipboard_list_scroll_handle: UniformListScrollHandle::new(),
   299|             window_list_scroll_handle: UniformListScrollHandle::new(),
   300|             design_gallery_scroll_handle: UniformListScrollHandle::new(),
   301|             file_search_scroll_handle: UniformListScrollHandle::new(),
   302|             file_search_loading: false,
   303|             file_search_debounce_task: None,
   304|             file_search_current_dir: None,
   305|             file_search_frozen_filter: None,
   306|             file_search_actions_path: None,
   307|             show_actions_popup: false,
   308|             actions_dialog: None,
   309|             cursor_visible: true,
   310|             focused_input: FocusedInput::MainFilter,
   311|             current_script_pid: None,
   312|             // P1: Initialize filter cache
   313|             cached_filtered_results: Vec::new(),
   314|             filter_cache_key: String::from("\0_UNINITIALIZED_\0"), // Sentinel value to force initial compute
   315|             // P1: Initialize grouped results cache (Arc for cheap clone)
   316|             cached_grouped_items: Arc::from([]),
   317|             cached_grouped_flat_results: Arc::from([]),
   318|             grouped_cache_key: String::from("\0_UNINITIALIZED_\0"), // Sentinel value to force initial compute
   319|             // P3: Two-stage filter coalescing
   320|             computed_filter_text: String::new(),
   321|             filter_coalescer: FilterCoalescer::new(),
   322|             // Scroll stabilization: start with no last scrolled index
   323|             last_scrolled_index: None,
   324|             // Preview cache: start empty, will populate on first render
   325|             preview_cache_path: None,
   326|             preview_cache_lines: Vec::new(),
   327|             // Scriptlet preview cache: avoid re-highlighting on every render
   328|             scriptlet_preview_cache_key: None,
   329|             scriptlet_preview_cache_lines: Vec::new(),
   330|             // Design system: start with default design
   331|             current_design: DesignVariant::default(),
   332|             // Toast manager: initialize for error notifications
   333|             toast_manager: ToastManager::new(),
   334|             // Clipboard image cache: decoded RenderImages for thumbnails/preview
   335|             clipboard_image_cache: std::collections::HashMap::new(),

  ...
   363|             // Alias/shortcut registries - populated below
   364|             alias_registry: std::collections::HashMap::new(),
   365|             shortcut_registry: std::collections::HashMap::new(),
   366|             // SDK actions - starts empty, populated by setActions() from scripts
   367|             sdk_actions: None,
   368|             action_shortcuts: std::collections::HashMap::new(),
   369|             // Debug grid overlay - check env var at startup
   370|             grid_config: if std::env::var("SCRIPT_KIT_DEBUG_GRID").is_ok() {
   371|                 logging::log(
   372|                     "DEBUG_GRID",
   373|                     "SCRIPT_KIT_DEBUG_GRID env var set - enabling grid overlay",
   374|                 );
   375|                 Some(debug_grid::GridConfig::default())
   376|             } else {
   377|                 None
   378|             },
   379|             // Navigation coalescing for rapid arrow key events
   380|             nav_coalescer: NavCoalescer::new(),
   381|             // Wheel scroll accumulator starts at 0
   382|             wheel_accum: 0.0,
   383|             // Window focus tracking - for detecting focus lost and auto-dismissing prompts
   384|             was_window_focused: false,
   385|             // Pin state - when true, window stays open on blur
   386|             is_pinned: false,
   387|             // Pending focus: start with MainFilter since that's what we want focused initially
   388|             pending_focus: Some(FocusTarget::MainFilter),
   389|             // Scroll stabilization: track last scrolled index for each handle
   390|             last_scrolled_main: None,
   391|             last_scrolled_arg: None,
   392|             last_scrolled_clipboard: None,
   393|             last_scrolled_window: None,
   394|             last_scrolled_design_gallery: None,
   395|             // Show warning banner when bun is not available
   396|             show_bun_warning: !bun_available,
   397|             // Builtin confirmation channel
   398|             builtin_confirm_sender: builtin_confirm_tx,
   399|             builtin_confirm_receiver: builtin_confirm_rx,
   400|             // Menu bar integration: Now handled by frontmost_app_tracker module
   401|             // which pre-fetches menu items in background when apps activate
   402|             // Shortcut recorder state - starts as None (no recorder showing)
   403|             shortcut_recorder_state: None,
   404|             // Shortcut recorder entity - persisted to maintain focus
   405|             shortcut_recorder_entity: None,
   406|             // Alias input state - starts as None (no alias input showing)
   407|             alias_input_state: None,
   408|             // Alias input entity - persisted to maintain focus
   409|             alias_input_entity: None,
   410|             // Input history for shell-like up/down navigation
   411|             input_history: {
   412|                 let mut history = input_history::InputHistory::new();
   413|                 if let Err(e) = history.load() {

  ...
  1212|             logging::log("APP", "Theme propagated to ActionsDialog");
  1213|         }
  1214| 
  1215|         cx.notify();
  1216|     }
  1217| 
  1218|     fn update_config(&mut self, cx: &mut Context<Self>) {
  1219|         self.config = config::load_config();
  1220|         clipboard_history::set_max_text_content_len(
  1221|             self.config.get_clipboard_history_max_text_length(),
  1222|         );
  1223|         // Hot-reload hotkeys from updated config
  1224|         hotkeys::update_hotkeys(&self.config);
  1225|         logging::log(
  1226|             "APP",
  1227|             &format!("Config reloaded: padding={:?}", self.config.get_padding()),
  1228|         );
  1229|         cx.notify();
  1230|     }
  1231| 
  1232|     /// Request focus for a specific target. Focus will be applied once on the
  1233|     /// next render when window access is available, then cleared.
  1234|     ///
  1235|     /// This avoids the "perpetually enforce focus in render()" anti-pattern.
  1236|     /// Use this instead of directly calling window.focus() from non-render code.
  1237|     #[allow(dead_code)] // Public API for external callers without direct pending_focus access
  1238|     pub fn request_focus(&mut self, target: FocusTarget, cx: &mut Context<Self>) {
  1239|         self.pending_focus = Some(target);
  1240|         cx.notify();
  1241|     }
  1242| 
  1243|     /// Apply pending focus if set. Called at the start of render() when window
  1244|     /// is focused. This applies focus exactly once, then clears pending_focus.
  1245|     ///
  1246|     /// Returns true if focus was applied (for logging/debugging).
  1247|     fn apply_pending_focus(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
  1248|         let Some(target) = self.pending_focus.take() else {
  1249|             return false;
  1250|         };
  1251| 
  1252|         logging::log("FOCUS", &format!("Applying pending focus: {:?}", target));
  1253| 
  1254|         match target {
  1255|             FocusTarget::MainFilter => {
  1256|                 let input_state = self.gpui_input_state.clone();
  1257|                 input_state.update(cx, |state, cx| {
  1258|                     state.focus(window, cx);
  1259|                 });
  1260|                 self.focused_input = FocusedInput::MainFilter;
  1261|             }
  1262|             FocusTarget::ActionsDialog => {
  1263|                 if let Some(ref dialog) = self.actions_dialog {
  1264|                     let fh = dialog.read(cx).focus_handle.clone();
  1265|                     window.focus(&fh, cx);
  1266|                     self.focused_input = FocusedInput::ActionsSearch;
  1267|                 }
  1268|             }
  1269|             FocusTarget::EditorPrompt => {
  1270|                 let entity = match &self.current_view {
  1271|                     AppView::EditorPrompt { entity, .. } => Some(entity),
  1272|                     AppView::ScratchPadView { entity, .. } => Some(entity),
  1273|                     _ => None,
  1274|                 };
  1275|                 if let Some(entity) = entity {
  1276|                     entity.update(cx, |editor, cx| {
  1277|                         editor.focus(window, cx);
  1278|                     });
  1279|                     // EditorPrompt has its own cursor management
  1280|                     self.focused_input = FocusedInput::None;
  1281|                 }
  1282|             }
  1283|             FocusTarget::PathPrompt => {
  1284|                 if let AppView::PathPrompt { focus_handle, .. } = &self.current_view {
  1285|                     let fh = focus_handle.clone();
  1286|                     window.focus(&fh, cx);
  1287|                     // PathPrompt has its own cursor management
  1288|                     self.focused_input = FocusedInput::None;
  1289|                 }
  1290|             }
  1291|             FocusTarget::FormPrompt => {
  1292|                 if let AppView::FormPrompt { entity, .. } = &self.current_view {
  1293|                     let fh = entity.read(cx).focus_handle(cx);
  1294|                     window.focus(&fh, cx);
  1295|                     // FormPrompt has its own focus handling
  1296|                     self.focused_input = FocusedInput::None;
  1297|                 }
  1298|             }
  1299|             FocusTarget::SelectPrompt => {
  1300|                 if let AppView::SelectPrompt { entity, .. } = &self.current_view {
  1301|                     let fh = entity.read(cx).focus_handle(cx);
  1302|                     window.focus(&fh, cx);
  1303|                     self.focused_input = FocusedInput::None;
  1304|                 }
  1305|             }
  1306|             FocusTarget::EnvPrompt => {
  1307|                 if let AppView::EnvPrompt { entity, .. } = &self.current_view {
  1308|                     let fh = entity.read(cx).focus_handle(cx);
  1309|                     window.focus(&fh, cx);
  1310|                     self.focused_input = FocusedInput::None;
  1311|                 }
  1312|             }
  1313|             FocusTarget::DropPrompt => {
  1314|                 if let AppView::DropPrompt { entity, .. } = &self.current_view {
  1315|                     let fh = entity.read(cx).focus_handle(cx);
  1316|                     window.focus(&fh, cx);
  1317|                     self.focused_input = FocusedInput::None;
  1318|                 }
  1319|             }
  1320|             FocusTarget::TemplatePrompt => {
  1321|                 if let AppView::TemplatePrompt { entity, .. } = &self.current_view {
  1322|                     let fh = entity.read(cx).focus_handle(cx);
  1323|                     window.focus(&fh, cx);
  1324|                     self.focused_input = FocusedInput::None;
  1325|                 }
  1326|             }
  1327|             FocusTarget::TermPrompt => {
  1328|                 let entity = match &self.current_view {
  1329|                     AppView::TermPrompt { entity, .. } => Some(entity),
  1330|                     AppView::QuickTerminalView { entity, .. } => Some(entity),
  1331|                     _ => None,
  1332|                 };
  1333|                 if let Some(entity) = entity {
  1334|                     let fh = entity.read(cx).focus_handle.clone();
  1335|                     window.focus(&fh, cx);
  1336|                     // Terminal handles its own cursor
  1337|                     self.focused_input = FocusedInput::None;
  1338|                 }
  1339|             }
  1340|             FocusTarget::ChatPrompt => {
  1341|                 if let AppView::ChatPrompt { entity, .. } = &self.current_view {
  1342|                     let fh = entity.read(cx).focus_handle(cx);
  1343|                     window.focus(&fh, cx);
  1344|                     self.focused_input = FocusedInput::None;
  1345|                 }
  1346|             }
  1347|             FocusTarget::AppRoot => {
  1348|                 window.focus(&self.focus_handle, cx);
  1349|                 // Don't reset focused_input here - the caller already set it appropriately.
  1350|                 // For example, ArgPrompt sets focused_input = ArgPrompt before setting
  1351|                 // pending_focus = AppRoot, and we want to preserve that so the cursor blinks.
  1352|             }
  1353|         }
  1354| 
  1355|         true
  1356|     }
  1357| 
  1358|     fn refresh_scripts(&mut self, cx: &mut Context<Self>) {
  1359|         self.scripts = scripts::read_scripts();
  1360|         // Use load_scriptlets() to load from ALL kits (kit/*/extensions/*.md)
  1361|         self.scriptlets = scripts::load_scriptlets();
  1362|         self.invalidate_filter_cache();
  1363|         self.invalidate_grouped_cache();
  1364| 
  1365|         // Sync list component state and validate selection
  1366|         // This moves state mutation OUT of render() (anti-pattern fix)
  1367|         self.sync_list_state();
  1368|         self.selected_index = 0;
  1369|         self.validate_selection_bounds(cx);
  1370|         self.main_list_state
  1371|             .scroll_to_reveal_item(self.selected_index);
  1372|         self.last_scrolled_index = Some(self.selected_index);
  1373| 
  1374|         // Rebuild alias/shortcut registries and show HUD for any conflicts
  1375|         let conflicts = self.rebuild_registries();
  1376|         for conflict in conflicts {

  ...
  2926|                 self.file_search_scroll_handle
  2927|                     .scroll_to_item(0, ScrollStrategy::Top);
  2928|                 // Mark that we need to sync the input text on next render
  2929|                 self.filter_text = text;
  2930|                 self.pending_filter_sync = true;
  2931|                 cx.notify();
  2932|             }
  2933|             _ => {}
  2934|         }
  2935|     }
  2936| 
  2937|     /// Helper to get filtered arg choices without cloning
  2938|     fn get_filtered_arg_choices<'a>(&self, choices: &'a [Choice]) -> Vec<&'a Choice> {
  2939|         if self.arg_input.is_empty() {
  2940|             choices.iter().collect()
  2941|         } else {
  2942|             let filter = self.arg_input.text().to_lowercase();
  2943|             choices
  2944|                 .iter()
  2945|                 .filter(|c| c.name.to_lowercase().contains(&filter))
  2946|                 .collect()
  2947|         }
  2948|     }
  2949| 
  2950|     fn focus_main_filter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
  2951|         self.focused_input = FocusedInput::MainFilter;
  2952|         let input_state = self.gpui_input_state.clone();
  2953|         input_state.update(cx, |state, cx| {
  2954|             state.focus(window, cx);
  2955|         });
  2956|     }
  2957| 
  2958|     fn toggle_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
  2959|         let popup_state = self.show_actions_popup;
  2960|         let window_open = is_actions_window_open();
  2961|         logging::log(
  2962|             "KEY",
  2963|             &format!(
  2964|                 "Toggling actions popup (show_actions_popup={}, is_actions_window_open={})",
  2965|                 popup_state, window_open
  2966|             ),
  2967|         );
  2968|         if self.show_actions_popup || is_actions_window_open() {
  2969|             // Close - return focus to main filter
  2970|             self.show_actions_popup = false;
  2971|             self.actions_dialog = None;
  2972|             self.focused_input = FocusedInput::MainFilter;
  2973|             self.pending_focus = Some(FocusTarget::MainFilter);
  2974| 
  2975|             // Close the separate actions window via spawn
  2976|             cx.spawn(async move |_this, cx| {
  2977|                 cx.update(|cx| {
  2978|                     close_actions_window(cx);
  2979|                 })
  2980|                 .ok();
  2981|             })
  2982|             .detach();
  2983| 
  2984|             // Refocus main filter
  2985|             self.focus_main_filter(window, cx);
  2986|             logging::log("FOCUS", "Actions closed, focus returned to MainFilter");
  2987|         } else {
  2988|             // Open actions as a separate window with vibrancy blur
  2989|             self.show_actions_popup = true;
  2990| 
  2991|             // CRITICAL: Transfer focus from Input to main focus_handle
  2992|             // This prevents the Input from receiving text (which would go to main filter)
  2993|             // while keeping keyboard focus in main window for routing to actions dialog
  2994|             self.focus_handle.focus(window, cx);
  2995|             self.gpui_input_focused = false;
  2996|             self.focused_input = FocusedInput::ActionsSearch;
  2997| 
  2998|             let script_info = self.get_focused_script_info();
  2999| 
  3000|             // Get the full scriptlet with actions if focused item is a scriptlet
  3001|             let focused_scriptlet = self.get_focused_scriptlet_with_actions();
  3002| 
  3003|             // Create the dialog entity HERE in main app (for keyboard routing)
  3004|             let theme_arc = std::sync::Arc::clone(&self.theme);
  3005|             // Create the dialog entity (search input shown at bottom, Raycast-style)
  3006|             let dialog = cx.new(|cx| {
  3007|                 let focus_handle = cx.focus_handle();
  3008|                 let mut dialog = ActionsDialog::with_script(
  3009|                     focus_handle,
  3010|                     std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
  3011|                     script_info.clone(),
  3012|                     theme_arc,
  3013|                 );
  3014| 
  3015|                 // If we have a scriptlet with actions, pass it to the dialog
  3016|                 if let Some(ref scriptlet) = focused_scriptlet {
  3017|                     dialog.set_focused_scriptlet(script_info.clone(), Some(scriptlet.clone()));
  3018|                 }
  3019| 
  3020|                 dialog
  3021|             });
  3022| 
  3023|             // Store the dialog entity for keyboard routing
  3024|             self.actions_dialog = Some(dialog.clone());
  3025| 
  3026|             // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
  3027|             // This ensures the same cleanup happens whether closing via Cmd+K toggle or Escape
  3028|             let app_entity = cx.entity().clone();
  3029|             dialog.update(cx, |d, _cx| {
  3030|                 d.set_on_close(std::sync::Arc::new(move |cx| {
  3031|                     app_entity.update(cx, |app, cx| {
  3032|                         app.show_actions_popup = false;
  3033|                         app.actions_dialog = None;
  3034|                         // Match what close_actions_popup does for MainList host:
  3035|                         // Set focused_input first, then pending_focus to AppRoot
  3036|                         // (AppRoot checks focused_input to know what to focus)
  3037|                         app.focused_input = FocusedInput::MainFilter;
  3038|                         app.pending_focus = Some(FocusTarget::AppRoot);
  3039|                         logging::log(
  3040|                             "FOCUS",
  3041|                             "Actions closed via escape, pending_focus=AppRoot, focused_input=MainFilter",
  3042|                         );
  3043|                         cx.notify();
  3044|                     });
  3045|                 }));
  3046|             });
  3047| 
  3048|             // Get main window bounds and display_id for positioning the actions popup
  3049|             //
  3050|             // CRITICAL: We use GPUI's window.bounds() which returns SCREEN-RELATIVE coordinates
  3051|             // (top-left origin, relative to the window's current screen). We also capture the
  3052|             // display_id so the actions window is created on the SAME screen as the main window.
  3053|             //
  3054|             // This fixes multi-monitor issues where the actions popup would appear on the wrong
  3055|             // screen or at wrong coordinates when the main window was on a secondary display.
  3056|             let main_bounds = window.bounds();
  3057|             let display_id = window.display(cx).map(|d| d.id());
  3058| 
  3059|             logging::log(
  3060|                 "ACTIONS",
  3061|                 &format!(
  3062|                     "Main window bounds (GPUI screen-relative): origin=({:?}, {:?}), size={:?}x{:?}, display_id={:?}",
  3063|                     main_bounds.origin.x, main_bounds.origin.y,
  3064|                     main_bounds.size.width, main_bounds.size.height,
  3065|                     display_id
  3066|                 ),

  ...
  3089|                 })
  3090|                 .ok();
  3091|             })
  3092|             .detach();
  3093| 
  3094|             logging::log("FOCUS", "Actions opened, keyboard routing active");
  3095|         }
  3096|         cx.notify();
  3097|     }
  3098| 
  3099|     /// Toggle actions dialog for arg prompts with SDK-defined actions
  3100|     fn toggle_arg_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
  3101|         logging::log(
  3102|             "KEY",
  3103|             &format!(
  3104|                 "toggle_arg_actions called: show_actions_popup={}, actions_dialog.is_some={}, sdk_actions.is_some={}",
  3105|                 self.show_actions_popup,
  3106|                 self.actions_dialog.is_some(),
  3107|                 self.sdk_actions.is_some()
  3108|             ),
  3109|         );
  3110|         if self.show_actions_popup {
  3111|             // Close - return focus to arg prompt
  3112|             self.show_actions_popup = false;
  3113|             self.actions_dialog = None;
  3114|             self.focused_input = FocusedInput::ArgPrompt;
  3115|             self.pending_focus = Some(FocusTarget::AppRoot); // ArgPrompt uses parent focus
  3116|             window.focus(&self.focus_handle, cx);
  3117|             logging::log("FOCUS", "Arg actions closed, focus returned to ArgPrompt");
  3118|         } else {
  3119|             // Check if we have SDK actions
  3120|             if let Some(ref sdk_actions) = self.sdk_actions {
  3121|                 logging::log("KEY", &format!("SDK actions count: {}", sdk_actions.len()));
  3122|                 if !sdk_actions.is_empty() {
  3123|                     // Open - create dialog entity with SDK actions
  3124|                     self.show_actions_popup = true;
  3125|                     self.focused_input = FocusedInput::ActionsSearch;
  3126| 
  3127|                     let theme_arc = std::sync::Arc::clone(&self.theme);
  3128|                     let sdk_actions_clone = sdk_actions.clone();
  3129|                     let dialog = cx.new(|cx| {
  3130|                         let focus_handle = cx.focus_handle();
  3131|                         let mut dialog = ActionsDialog::with_script(
  3132|                             focus_handle,
  3133|                             std::sync::Arc::new(|_action_id| {}), // Callback handled separately
  3134|                             None,                                 // No script info for arg prompts
  3135|                             theme_arc,
  3136|                         );
  3137|                         // Set SDK actions to replace built-in actions
  3138|                         dialog.set_sdk_actions(sdk_actions_clone);
  3139|                         dialog
  3140|                     });
  3141| 
  3142|                     // Show search input at bottom (Raycast-style)
  3143| 
  3144|                     // Focus the dialog's internal focus handle
  3145|                     self.actions_dialog = Some(dialog.clone());
  3146|                     self.pending_focus = Some(FocusTarget::ActionsDialog);
  3147|                     let dialog_focus_handle = dialog.read(cx).focus_handle.clone();
  3148|                     window.focus(&dialog_focus_handle, cx);
  3149|                     logging::log(
  3150|                         "FOCUS",
  3151|                         &format!(
  3152|                             "Arg actions OPENED: show_actions_popup={}, actions_dialog.is_some={}",
  3153|                             self.show_actions_popup,
  3154|                             self.actions_dialog.is_some()
  3155|                         ),
  3156|                     );
  3157|                 } else {
  3158|                     logging::log("KEY", "No SDK actions available to show (empty list)");
  3159|                 }
  3160|             } else {
  3161|                 logging::log("KEY", "No SDK actions defined for this arg prompt (None)");
  3162|             }
  3163|         }
  3164|         cx.notify();
  3165|     }
  3166| 
  3167|     /// Toggle actions dialog for chat prompts
  3168|     /// Opens ActionsDialog with model selection and chat-specific actions
  3169|     pub fn toggle_chat_actions(&mut self, cx: &mut Context<Self>, window: &mut Window) {
  3170|         use crate::actions::{ChatModelInfo, ChatPromptInfo};
  3171| 
  3172|         logging::log(
  3173|             "KEY",
  3174|             &format!(
  3175|                 "toggle_chat_actions called: show_actions_popup={}, actions_dialog.is_some={}",
  3176|                 self.show_actions_popup,
  3177|                 self.actions_dialog.is_some()
  3178|             ),
  3179|         );
  3180| 
  3181|         if self.show_actions_popup || is_actions_window_open() {
  3182|             // Close - return focus to chat prompt
  3183|             self.show_actions_popup = false;
  3184|             self.actions_dialog = None;
  3185|             self.focused_input = FocusedInput::None;
  3186|             self.pending_focus = Some(FocusTarget::AppRoot);
  3187| 
  3188|             // Close the separate actions window via spawn
  3189|             cx.spawn(async move |_this, cx| {
  3190|                 cx.update(|cx| {
  3191|                     close_actions_window(cx);
  3192|                 })
  3193|                 .ok();
  3194|             })
  3195|             .detach();
  3196| 
  3197|             window.focus(&self.focus_handle, cx);
  3198|             logging::log("FOCUS", "Chat actions closed, focus returned to ChatPrompt");
  3199|         } else {
  3200|             // Get chat info from current ChatPrompt entity
  3201|             let chat_info = if let AppView::ChatPrompt { entity, .. } = &self.current_view {
  3202|                 let chat = entity.read(cx);
  3203|                 ChatPromptInfo {
  3204|                     current_model: chat.model.clone(),
  3205|                     available_models: chat
  3206|                         .models
  3207|                         .iter()
  3208|                         .map(|m| ChatModelInfo {
  3209|                             id: m.id.clone(),
  3210|                             display_name: m.name.clone(),
  3211|                             provider: m.provider.clone(),
  3212|                         })
  3213|                         .collect(),
  3214|                     has_messages: !chat.messages.is_empty(),
  3215|                     has_response: chat
  3216|                         .messages
  3217|                         .iter()
  3218|                         .any(|m| m.position == crate::protocol::ChatMessagePosition::Left),
  3219|                 }
  3220|             } else {
  3221|                 logging::log(
  3222|                     "KEY",
  3223|                     "toggle_chat_actions called but current view is not ChatPrompt",
  3224|                 );
  3225|                 return;
  3226|             };
  3227| 
  3228|             // Open actions as a separate window with vibrancy blur
  3229|             self.show_actions_popup = true;
  3230|             self.focused_input = FocusedInput::ActionsSearch;
  3231| 
  3232|             let theme_arc = std::sync::Arc::clone(&self.theme);
  3233|             let dialog = cx.new(|cx| {
  3234|                 let focus_handle = cx.focus_handle();
  3235|                 ActionsDialog::with_chat(
  3236|                     focus_handle,
  3237|                     std::sync::Arc::new(|_action_id| {}), // Callback handled via main app
  3238|                     &chat_info,
  3239|                     theme_arc,
  3240|                 )
  3241|             });
  3242| 
  3243|             // Store the dialog entity for keyboard routing
  3244|             self.actions_dialog = Some(dialog.clone());
  3245| 
  3246|             // Set up the on_close callback to restore focus when escape is pressed in ActionsWindow
  3247|             // Match what close_actions_popup does for ChatPrompt host
  3248|             let app_entity = cx.entity().clone();
  3249|             dialog.update(cx, |d, _cx| {
  3250|                 d.set_on_close(std::sync::Arc::new(move |cx| {
  3251|                     app_entity.update(cx, |app, cx| {
  3252|                         app.show_actions_popup = false;
  3253|                         app.actions_dialog = None;
  3254|                         // ChatPrompt handles its own focus - restore to app root
  3255|                         app.focused_input = FocusedInput::None;
  3256|                         app.pending_focus = Some(FocusTarget::ChatPrompt);
  3257|                         logging::log(
  3258|                             "FOCUS",
  3259|                             "Chat actions closed via escape, pending_focus=ChatPrompt",
  3260|                         );
  3261|                         cx.notify();
  3262|                     });
  3263|                 }));
  3264|             });
  3265| 
  3266|             // Get main window bounds and display_id for positioning
  3267|             let main_bounds = window.bounds();
  3268|             let display_id = window.display(cx).map(|d| d.id());
  3269| 
  3270|             logging::log(
  3271|                 "ACTIONS",
  3272|                 &format!(
  3273|                     "Chat actions: Main window bounds origin=({:?}, {:?}), size={:?}x{:?}, display_id={:?}",
  3274|                     main_bounds.origin.x, main_bounds.origin.y,
  3275|                     main_bounds.size.width, main_bounds.size.height,
  3276|                     display_id
  3277|                 ),
  3278|             );
  3279| 
  3280|             // Open the actions window via spawn
  3281|             cx.spawn(async move |_this, cx| {
  3282|                 cx.update(|cx| {
  3283|                     match open_actions_window(
  3284|                         cx,

  ...
  3543|     fn close_actions_popup(
  3544|         &mut self,
  3545|         host: ActionsDialogHost,
  3546|         window: &mut Window,
  3547|         cx: &mut Context<Self>,
  3548|     ) {
  3549|         self.show_actions_popup = false;
  3550|         self.actions_dialog = None;
  3551| 
  3552|         // Close the separate actions window if open
  3553|         // This ensures consistent behavior whether closing via Cmd+K, Escape, backdrop click,
  3554|         // or any other close mechanism
  3555|         if is_actions_window_open() {
  3556|             cx.spawn(async move |_this, cx| {
  3557|                 cx.update(|cx| {
  3558|                     close_actions_window(cx);
  3559|                 })
  3560|                 .ok();
  3561|             })
  3562|             .detach();
  3563|         }
  3564| 
  3565|         // Restore focus based on host type
  3566|         match host {
  3567|             ActionsDialogHost::ArgPrompt => {
  3568|                 self.focused_input = FocusedInput::ArgPrompt;
  3569|                 self.pending_focus = Some(FocusTarget::AppRoot);
  3570|             }
  3571|             ActionsDialogHost::DivPrompt
  3572|             | ActionsDialogHost::EditorPrompt
  3573|             | ActionsDialogHost::TermPrompt
  3574|             | ActionsDialogHost::FormPrompt => {
  3575|                 self.focused_input = FocusedInput::None;
  3576|             }
  3577|             ActionsDialogHost::ChatPrompt => {
  3578|                 // ChatPrompt handles its own focus - restore to app root
  3579|                 self.focused_input = FocusedInput::None;
  3580|                 self.pending_focus = Some(FocusTarget::AppRoot);
  3581|             }
  3582|             ActionsDialogHost::MainList => {
  3583|                 self.focused_input = FocusedInput::MainFilter;
  3584|                 self.pending_focus = Some(FocusTarget::AppRoot);
  3585|             }
  3586|             ActionsDialogHost::FileSearch => {
  3587|                 // File search uses MainFilter input - restore focus to it
  3588|                 self.focused_input = FocusedInput::MainFilter;
  3589|                 self.pending_focus = Some(FocusTarget::AppRoot);
  3590|             }
  3591|         }
  3592| 
  3593|         window.focus(&self.focus_handle, cx);
  3594|         logging::log(
  3595|             "FOCUS",
  3596|             &format!("Actions popup closed, focus restored for {:?}", host),
  3597|         );
  3598|         cx.notify();
  3599|     }
  3600| 
  3601|     /// Edit a script in configured editor (config.editor > $EDITOR > "code")
  3602|     #[allow(dead_code)]
  3603|     fn edit_script(&mut self, path: &std::path::Path) {
  3604|         let editor = self.config.get_editor();
  3605|         logging::log(
  3606|             "UI",
  3607|             &format!("Opening script in editor '{}': {}", editor, path.display()),
  3608|         );
  3609|         let path_str = path.to_string_lossy().to_string();
  3610| 
  3611|         std::thread::spawn(move || {
  3612|             use std::process::Command;
  3613|             match Command::new(&editor).arg(&path_str).spawn() {
  3614|                 Ok(_) => logging::log("UI", &format!("Successfully spawned editor: {}", editor)),

  ...
  3787|             command_id,
  3788|             command_name,
  3789|         });
  3790| 
  3791|         // Clear any existing entity so a new one is created with correct focus
  3792|         self.shortcut_recorder_entity = None;
  3793| 
  3794|         // Close actions popup if open
  3795|         self.show_actions_popup = false;
  3796|         self.actions_dialog = None;
  3797| 
  3798|         cx.notify();
  3799|     }
  3800| 
  3801|     /// Close the shortcut recorder and clear state.
  3802|     /// Returns focus to the main filter input.
  3803|     pub fn close_shortcut_recorder(&mut self, cx: &mut Context<Self>) {
  3804|         if self.shortcut_recorder_state.is_some() || self.shortcut_recorder_entity.is_some() {
  3805|             logging::log(
  3806|                 "SHORTCUT",
  3807|                 "Closing shortcut recorder, returning focus to main filter",
  3808|             );
  3809|             self.shortcut_recorder_state = None;
  3810|             self.shortcut_recorder_entity = None;
  3811|             // Return focus to the main filter input
  3812|             self.pending_focus = Some(FocusTarget::MainFilter);
  3813|             cx.notify();
  3814|         }
  3815|     }
  3816| 
  3817|     /// Render the shortcut recorder overlay if state is set.
  3818|     ///
  3819|     /// Returns None if no recorder is active.
  3820|     ///
  3821|     /// The recorder is created once and persisted to maintain keyboard focus.
  3822|     /// Callbacks use cx.entity() to communicate back to the parent app.
  3823|     fn render_shortcut_recorder_overlay(
  3824|         &mut self,
  3825|         window: &mut Window,
  3826|         cx: &mut Context<Self>,
  3827|     ) -> Option<gpui::AnyElement> {
  3828|         use crate::components::shortcut_recorder::ShortcutRecorder;
  3829| 
  3830|         // Check if we have state but no entity yet - need to create the recorder
  3831|         let state = self.shortcut_recorder_state.as_ref()?;
  3832| 
  3833|         // Create entity if needed (only once per show)
  3834|         if self.shortcut_recorder_entity.is_none() {
  3835|             let command_id = state.command_id.clone();
  3836|             let command_name = state.command_name.clone();
  3837|             let theme = std::sync::Arc::clone(&self.theme);

  ...
  4063|         // Store state
  4064|         self.alias_input_state = Some(AliasInputState {
  4065|             command_id,
  4066|             command_name,
  4067|             alias_text: existing_alias,
  4068|         });
  4069| 
  4070|         // Close actions popup if open
  4071|         self.show_actions_popup = false;
  4072|         self.actions_dialog = None;
  4073| 
  4074|         cx.notify();
  4075|     }
  4076| 
  4077|     /// Close the alias input and clear state.
  4078|     /// Returns focus to the main filter input.
  4079|     pub fn close_alias_input(&mut self, cx: &mut Context<Self>) {
  4080|         if self.alias_input_state.is_some() || self.alias_input_entity.is_some() {
  4081|             logging::log(
  4082|                 "ALIAS",
  4083|                 "Closing alias input, returning focus to main filter",
  4084|             );
  4085|             self.alias_input_state = None;
  4086|             self.alias_input_entity = None; // Clear entity to reset for next open
  4087|                                             // Return focus to the main filter input (like close_shortcut_recorder does)
  4088|             self.pending_focus = Some(FocusTarget::MainFilter);
  4089|             cx.notify();
  4090|         }
  4091|     }
  4092| 
  4093|     /// Update the alias text in the input state.
  4094|     /// Currently unused - will be connected when real text input is added.
  4095|     #[allow(dead_code)]
  4096|     fn update_alias_text(&mut self, text: String, cx: &mut Context<Self>) {
  4097|         if let Some(ref mut state) = self.alias_input_state {
  4098|             state.alias_text = text;
  4099|             cx.notify();
  4100|         }
  4101|     }
  4102| 
  4103|     /// Save the current alias and close the input.
  4104|     /// If alias_from_entity is provided, use that; otherwise fall back to state.alias_text.
  4105|     fn save_alias_with_text(&mut self, alias_from_entity: Option<String>, cx: &mut Context<Self>) {
  4106|         let Some(ref state) = self.alias_input_state else {
  4107|             logging::log("ALIAS", "No alias input state when trying to save");
  4108|             return;
  4109|         };
  4110| 
  4111|         let command_id = state.command_id.clone();
  4112|         let command_name = state.command_name.clone();
  4113|         // Prefer alias from entity if provided, else use state

  ...
  5131|     ///
  5132|     /// If the current built-in view was opened from the main menu, this returns to the
  5133|     /// main menu (ScriptList). If it was opened directly via hotkey or protocol command,
  5134|     /// this closes the window entirely.
  5135|     ///
  5136|     /// This provides consistent UX: pressing ESC always "goes back" one step.
  5137|     fn go_back_or_close(&mut self, window: &mut Window, cx: &mut Context<Self>) {
  5138|         if self.opened_from_main_menu {
  5139|             logging::log(
  5140|                 "KEY",
  5141|                 "ESC - returning to main menu (opened from main menu)",
  5142|             );
  5143|             // Return to main menu
  5144|             self.current_view = AppView::ScriptList;
  5145|             self.filter_text.clear();
  5146|             self.selected_index = 0;
  5147|             // Reset the flag since we're now in main menu
  5148|             self.opened_from_main_menu = false;
  5149|             // Sync input and reset placeholder to default
  5150|             self.gpui_input_state.update(cx, |state, cx| {
  5151|                 state.set_value("", window, cx);
  5152|                 state.set_selection(0, 0, window, cx);
  5153|                 state.set_placeholder(DEFAULT_PLACEHOLDER.to_string(), window, cx);
  5154|             });
  5155|             self.update_window_size_deferred(window, cx);
  5156|             self.pending_focus = Some(FocusTarget::MainFilter);
  5157|             self.focused_input = FocusedInput::MainFilter;
  5158|             cx.notify();
  5159|         } else {
  5160|             logging::log(
  5161|                 "KEY",
  5162|                 "ESC - closing window (opened directly via hotkey/protocol)",
  5163|             );
  5164|             self.close_and_reset_window(cx);
  5165|         }
  5166|     }
  5167| 
  5168|     /// Handle global keyboard shortcuts with configurable dismissability
  5169|     ///
  5170|     /// Returns `true` if the shortcut was handled (caller should return early)
  5171|     ///
  5172|     /// # Arguments
  5173|     /// * `event` - The key down event to check
  5174|     /// * `is_dismissable` - If true, ESC key will also close the window (for prompts like arg, div, form, etc.)
  5175|     ///   If false, only Cmd+W closes the window (for prompts like term, editor)
  5176|     /// * `cx` - The context
  5177|     ///
  5178|     /// # Handled shortcuts
  5179|     /// - Cmd+W: Always closes window and resets to default state
  5180|     /// - Escape: Only closes window if `is_dismissable` is true AND actions popup is not showing
  5181|     /// - Cmd+Shift+M: Cycle vibrancy material (for debugging)
  5182|     fn handle_global_shortcut_with_options(

  ...
  5491|                 old_view, old_focused_input
  5492|             ),
  5493|         );
  5494| 
  5495|         // Belt-and-suspenders: Force-kill the process group using stored PID
  5496|         // This runs BEFORE clearing channels to ensure cleanup even if Drop doesn't fire
  5497|         if let Some(pid) = self.current_script_pid.take() {
  5498|             logging::log(
  5499|                 "CLEANUP",
  5500|                 &format!("Force-killing script process group {} during reset", pid),
  5501|             );
  5502|             #[cfg(unix)]
  5503|             {
  5504|                 let _ = std::process::Command::new("kill")
  5505|                     .args(["-9", &format!("-{}", pid)])
  5506|                     .output();
  5507|             }
  5508|         }
  5509| 
  5510|         // Reset view
  5511|         self.current_view = AppView::ScriptList;
  5512| 
  5513|         // CRITICAL: Reset focused_input to MainFilter so the cursor appears
  5514|         // This was a bug where focused_input could remain as ArgPrompt/None after
  5515|         // script exit, causing the cursor to not show in the main filter.
  5516|         self.focused_input = FocusedInput::MainFilter;
  5517|         self.gpui_input_focused = false;
  5518|         self.pending_focus = Some(FocusTarget::MainFilter);
  5519|         // Reset placeholder back to default for main menu
  5520|         self.pending_placeholder = Some(DEFAULT_PLACEHOLDER.to_string());
  5521|         logging::log(
  5522|             "FOCUS",
  5523|             "Reset focused_input to MainFilter for cursor display",
  5524|         );
  5525| 
  5526|         // Clear arg prompt state
  5527|         self.arg_input.clear();
  5528|         self.arg_selected_index = 0;
  5529|         // P0: Reset arg scroll handle
  5530|         self.arg_list_scroll_handle
  5531|             .scroll_to_item(0, ScrollStrategy::Top);
  5532| 
  5533|         // Clear filter and selection state for fresh menu
  5534|         self.filter_text.clear();
  5535|         self.computed_filter_text.clear();
  5536|         self.filter_coalescer.reset();
  5537|         self.pending_filter_sync = true;
  5538| 
  5539|         // Sync list component state and validate selection
  5540|         // This moves state mutation OUT of render() (anti-pattern fix)
  5541|         self.invalidate_grouped_cache(); // Ensure cache is fresh
  5542|         self.sync_list_state();
  5543|         self.selected_index = 0;

  ...
  5810|             let noop_callback: ChatSubmitCallback = std::sync::Arc::new(|_id, _text| {
  5811|                 crate::logging::log("CHAT", "No providers - submission ignored (setup mode)");
  5812|             });
  5813| 
  5814|             let chat_prompt = ChatPrompt::new(
  5815|                 "inline-ai-setup".to_string(),
  5816|                 Some("Configure API key to continue...".to_string()),
  5817|                 vec![],
  5818|                 None, // No hint needed - setup card is the UI
  5819|                 None,
  5820|                 self.focus_handle.clone(),
  5821|                 noop_callback,
  5822|                 std::sync::Arc::clone(&self.theme),
  5823|             )
  5824|             .with_title("Ask AI")
  5825|             .with_save_history(false) // Don't save setup state to history
  5826|             .with_escape_callback(escape_callback.clone())
  5827|             .with_needs_setup(true)
  5828|             .with_configure_callback(configure_callback);
  5829| 
  5830|             let entity = cx.new(|_| chat_prompt);
  5831|             self.current_view = AppView::ChatPrompt {
  5832|                 id: "inline-ai-setup".to_string(),
  5833|                 entity,
  5834|             };
  5835|             self.focused_input = FocusedInput::None;
  5836|             self.pending_focus = Some(FocusTarget::ChatPrompt);
  5837|             resize_to_view_sync(ViewType::DivPrompt, 0);
  5838|             cx.notify();
  5839|             return;
  5840|         }
  5841| 
  5842|         crate::logging::log(
  5843|             "CHAT",
  5844|             &format!(
  5845|                 "Showing inline AI chat with {} providers",
  5846|                 registry.provider_ids().len()
  5847|             ),
  5848|         );
  5849| 
  5850|         // Create a no-op callback since built-in AI handles submissions internally
  5851|         let noop_callback: ChatSubmitCallback = std::sync::Arc::new(|_id, _text| {
  5852|             // Built-in AI mode handles this internally
  5853|         });
  5854| 
  5855|         let placeholder = Some("Ask anything...".to_string());
  5856| 
  5857|         let mut chat_prompt = ChatPrompt::new(
  5858|             "inline-ai".to_string(),
  5859|             placeholder,
  5860|             vec![],
  5861|             None,
  5862|             None,
  5863|             self.focus_handle.clone(),
  5864|             noop_callback,
  5865|             std::sync::Arc::clone(&self.theme),
  5866|         )
  5867|         .with_title("Ask AI")
  5868|         .with_save_history(true)
  5869|         .with_escape_callback(escape_callback)
  5870|         .with_builtin_ai(registry, true); // true = prefer Vercel AI Gateway
  5871| 
  5872|         // If there's an initial query, set it in the input and auto-submit
  5873|         if let Some(query) = initial_query {
  5874|             chat_prompt.input.set_text(&query);
  5875|             chat_prompt = chat_prompt.with_pending_submit(true);
  5876|         }
  5877| 
  5878|         let entity = cx.new(|_| chat_prompt);
  5879|         self.current_view = AppView::ChatPrompt {
  5880|             id: "inline-ai".to_string(),
  5881|             entity,
  5882|         };
  5883|         self.focused_input = FocusedInput::None;
  5884|         self.pending_focus = Some(FocusTarget::ChatPrompt);
  5885|         resize_to_view_sync(ViewType::DivPrompt, 0);
  5886|         cx.notify();
  5887|     }
  5888| }
  5889| 
  5890| // Note: convert_menu_bar_items/convert_menu_bar_item functions were removed
  5891| // because frontmost_app_tracker is now compiled as part of the binary crate
  5892| // (via `mod frontmost_app_tracker` in main.rs) so it returns binary types directly.
  5893| 
</file>

<file path="src/form_prompt.rs" matches="7" windows="3">
     1| use gpui::{
     2|     div, prelude::*, px, rgb, App, Context, Entity, FocusHandle, Focusable, KeyDownEvent, Render,
     3|     Window,
     4| };
     5| 
     6| use crate::components::{FormCheckbox, FormFieldColors, FormTextArea, FormTextField};
     7| use crate::{form_parser, logging, protocol};
     8| 
     9| /// Enum to hold different types of form field entities.
    10| #[derive(Clone)]
    11| pub enum FormFieldEntity {
    12|     TextField(Entity<FormTextField>),
    13|     TextArea(Entity<FormTextArea>),
    14|     Checkbox(Entity<FormCheckbox>),
    15| }
    16| 
    17| /// Form prompt state - holds the parsed form fields and their entities.
    18| pub struct FormPromptState {
    19|     /// Prompt ID for response.
    20|     pub id: String,
    21|     /// Original HTML for reference.
    22|     #[allow(dead_code)]
    23|     pub html: String,
    24|     /// Parsed field definitions and their corresponding entities.
    25|     pub fields: Vec<(protocol::Field, FormFieldEntity)>,
    26|     /// Colors for form fields.
    27|     pub colors: FormFieldColors,
    28|     /// Currently focused field index (for Tab navigation).
    29|     pub focused_index: usize,
    30|     /// Focus handle for this form.
    31|     pub focus_handle: FocusHandle,
    32|     /// Whether we've done initial focus.
    33|     pub did_initial_focus: bool,
    34| }
    35| 
    36| impl FormPromptState {
    37|     fn build_values_json(values: impl IntoIterator<Item = (String, String)>) -> String {
    38|         let mut map = serde_json::Map::new();
    39|         for (key, value) in values {
    40|             map.insert(key, serde_json::Value::String(value));
    41|         }
    42|         serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string())
    43|     }
    44| 
    45|     /// Create a new form prompt state from HTML.
    46|     pub fn new(id: String, html: String, colors: FormFieldColors, cx: &mut App) -> Self {
    47|         let parsed_fields = form_parser::parse_form_html(&html);
    48| 
    49|         logging::log(
    50|             "FORM",
    51|             &format!("Parsed {} form fields from HTML", parsed_fields.len()),
    52|         );
    53| 
    54|         let fields: Vec<(protocol::Field, FormFieldEntity)> = parsed_fields
    55|             .into_iter()
    56|             .map(|field| {

  ...
   115|     }
   116| 
   117|     /// Focus the next field (for Tab navigation).
   118|     pub fn focus_next(&mut self, cx: &mut Context<Self>) {
   119|         if self.fields.is_empty() {
   120|             return;
   121|         }
   122|         self.focused_index = (self.focused_index + 1) % self.fields.len();
   123|         cx.notify();
   124|     }
   125| 
   126|     /// Focus the previous field (for Shift+Tab navigation).
   127|     pub fn focus_previous(&mut self, cx: &mut Context<Self>) {
   128|         if self.fields.is_empty() {
   129|             return;
   130|         }
   131|         if self.focused_index == 0 {
   132|             self.focused_index = self.fields.len() - 1;
   133|         } else {
   134|             self.focused_index -= 1;
   135|         }
   136|         cx.notify();
   137|     }
   138| 
   139|     /// Get the focus handle for the currently focused field.
   140|     pub fn current_focus_handle(&self, cx: &App) -> Option<FocusHandle> {
   141|         self.fields
   142|             .get(self.focused_index)
   143|             .map(|(_, entity)| match entity {
   144|                 FormFieldEntity::TextField(e) => e.read(cx).focus_handle(cx),
   145|                 FormFieldEntity::TextArea(e) => e.read(cx).focus_handle(cx),
   146|                 FormFieldEntity::Checkbox(e) => e.read(cx).focus_handle(cx),
   147|             })
   148|     }
   149| 
   150|     /// Handle keyboard input by forwarding to the currently focused field.
   151|     ///
   152|     /// This forwards key events to the field's unified `handle_key_event` method
   153|     /// which properly handles:
   154|     /// - Char-based cursor positioning (not byte-based)
   155|     /// - Modifier keys (Cmd/Ctrl+C/V/X/A work correctly)
   156|     /// - Selection with Shift+Arrow
   157|     /// - Clipboard operations
   158|     pub fn handle_key_input(&mut self, event: &KeyDownEvent, cx: &mut Context<Self>) {
   159|         if let Some((_, entity)) = self.fields.get(self.focused_index) {
   160|             match entity {
   161|                 FormFieldEntity::TextField(e) => {
   162|                     e.update(cx, |field, cx| {
   163|                         field.handle_key_event(event, cx);
   164|                     });
   165|                 }

  ...
   228|         // Build the form fields container
   229|         let mut container = div().flex().flex_col().gap(px(16.)).w_full();
   230| 
   231|         for (_field_def, entity) in &self.fields {
   232|             container = match entity {
   233|                 FormFieldEntity::TextField(e) => container.child(e.clone()),
   234|                 FormFieldEntity::TextArea(e) => container.child(e.clone()),
   235|                 FormFieldEntity::Checkbox(e) => container.child(e.clone()),
   236|             };
   237|         }
   238| 
   239|         // If no fields, show an error message
   240|         if self.fields.is_empty() {
   241|             container = container.child(
   242|                 div()
   243|                     .p(px(16.))
   244|                     .text_color(rgb(colors.label))
   245|                     .child("No form fields found in HTML"),
   246|             );
   247|         }
   248| 
   249|         container
   250|     }
   251| }
   252| 
   253| /// Delegated Focusable implementation for FormPromptState.
   254| ///
   255| /// This implements the "delegated focus" pattern from Zed's BufferSearchBar:
   256| /// Instead of returning our own focus_handle, we return the focused field's handle.
   257| /// This prevents the parent container from "stealing" focus from child fields during re-renders.
   258| ///
   259| /// When GPUI asks "what should be focused?", we answer with the currently focused
   260| /// text field's handle, so focus stays on the actual input field, not the form container.
   261| impl Focusable for FormPromptState {
   262|     fn focus_handle(&self, cx: &App) -> FocusHandle {
   263|         // Return the focused field's handle, not our own
   264|         // This delegates focus management to the child field, preventing focus stealing
   265|         if let Some((_, entity)) = self.fields.get(self.focused_index) {
   266|             match entity {
   267|                 FormFieldEntity::TextField(e) => e.read(cx).get_focus_handle(),
   268|                 FormFieldEntity::TextArea(e) => e.read(cx).get_focus_handle(),
   269|                 FormFieldEntity::Checkbox(e) => e.read(cx).focus_handle(cx),
   270|             }
   271|         } else {
   272|             // Fallback to our own handle if no fields exist
   273|             self.focus_handle.clone()
   274|         }
   275|     }
   276| }
   277| 
   278| #[cfg(test)]
   279| mod tests {
   280|     use super::*;
   281|     use serde_json::json;
   282| 
   283|     #[test]
   284|     fn build_values_json_serializes_string_values() {
   285|         let values = vec![
   286|             ("username".to_string(), "Bob".to_string()),
   287|             ("bio".to_string(), "Hello".to_string()),
</file>

<file path="src/main.rs" matches="22" windows="10">
     1| #![allow(unexpected_cfgs)]
     2| 
     3| use gpui::{
     4|     div, hsla, list, point, prelude::*, px, rgb, rgba, size, svg, uniform_list, AnyElement, App,
     5|     Application, BoxShadow, Context, ElementId, Entity, FocusHandle, Focusable, ListAlignment,
     6|     ListOffset, ListSizingBehavior, ListState, Render, ScrollStrategy, SharedString, Subscription,
     7|     Timer, UniformListScrollHandle, Window, WindowBackgroundAppearance, WindowBounds, WindowHandle,
     8|     WindowKind, WindowOptions,
     9| };
    10| 
    11| // gpui-component Root wrapper for theme and context provision
    12| use gpui_component::input::{Input, InputEvent, InputState};
    13| use gpui_component::notification::{Notification, NotificationType};
    14| use gpui_component::Root;
    15| use gpui_component::{Sizable, Size};
    16| use std::sync::atomic::{AtomicBool, Ordering};
    17| 
    18| mod process_manager;
    19| use cocoa::base::id;
    20| use cocoa::foundation::NSRect;
    21| use process_manager::PROCESS_MANAGER;
    22| 
    23| // Platform utilities - mouse position, display info, window movement, screenshots
    24| use platform::{
    25|     calculate_eye_line_bounds_on_mouse_display, capture_app_screenshot, capture_window_by_title,
    26| };
    27| #[macro_use]
    28| extern crate objc;
    29| 
    30| mod actions;

  ...
   431|         let focus_handle = view.focus_handle(ctx);
   432|         let _ = window.update(ctx, |_root, win, _cx| {
   433|             win.focus(&focus_handle, _cx);
   434|         });
   435| 
   436|         // Reset resize debounce to ensure proper window sizing
   437|         reset_resize_debounce();
   438| 
   439|         // Handle NEEDS_RESET: if set (e.g., script completed while hidden),
   440|         // reset to script list.
   441|         if NEEDS_RESET
   442|             .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst)
   443|             .is_ok()
   444|         {
   445|             logging::log(
   446|                 "VISIBILITY",
   447|                 "NEEDS_RESET was true - resetting to script list",
   448|             );
   449|             view.reset_to_script_list(ctx);
   450|         } else {
   451|             // FIX: Always ensure selection is at the first item when showing.
   452|             // This fixes the bug where the main menu sometimes opened with a
   453|             // random item selected (e.g., "Reset Window Positions" instead of "AI Chat").
   454|             view.ensure_selection_at_first_item(ctx);
   455| 
   456|             // FIX: Set pending_focus to MainFilter so the input gets focused
   457|             // when the window is shown. Without this, the cursor won't blink
   458|             // and typing won't work until the user clicks the input.
   459|             view.focused_input = FocusedInput::MainFilter;
   460|             view.pending_focus = Some(FocusTarget::MainFilter);
   461|         }
   462| 
   463|         // Always ensure window size matches current view using deferred resize.
   464|         // This uses Window::defer to avoid RefCell borrow conflicts.
   465|         let _ = window.update(ctx, |_root, win, win_cx| {
   466|             defer_resize_to_view(ViewType::ScriptList, 0, win, win_cx);
   467|         });
   468|     });
   469| 
   470|     logging::log("VISIBILITY", "Main window shown and focused");
   471| }
   472| 
   473| /// Hide the main window with proper state management.
   474| ///
   475| /// This is the canonical way to hide the main window. It:
   476| /// 1. Saves window position for the current display (per-display persistence)
   477| /// 2. Sets MAIN_WINDOW_VISIBLE state to false
   478| /// 3. Cancels any active prompt (if in prompt mode)
   479| /// 4. Resets to script list
   480| /// 5. Uses hide_main_window() if Notes/AI windows are open (to avoid hiding them)
   481| /// 6. Uses cx.hide() if no secondary windows are open
   482| ///
   483| /// # Arguments
   484| /// * `app_entity` - The ScriptListApp entity
   485| /// * `cx` - The application context

  ...
   768|     DivPrompt {
   769|         #[allow(dead_code)]
   770|         id: String,
   771|         entity: Entity<DivPrompt>,
   772|     },
   773|     /// Showing a form prompt from a script (HTML form with submit button)
   774|     FormPrompt {
   775|         #[allow(dead_code)]
   776|         id: String,
   777|         entity: Entity<FormPromptState>,
   778|     },
   779|     /// Showing a terminal prompt from a script
   780|     TermPrompt {
   781|         #[allow(dead_code)]
   782|         id: String,
   783|         entity: Entity<term_prompt::TermPrompt>,
   784|     },
   785|     /// Showing an editor prompt from a script (gpui-component based with Find/Replace)
   786|     EditorPrompt {
   787|         #[allow(dead_code)]
   788|         id: String,
   789|         entity: Entity<EditorPrompt>,
   790|         /// Separate focus handle for the editor (not shared with parent)
   791|         /// Note: This is kept for API compatibility but focus is managed via entity.focus()
   792|         #[allow(dead_code)]
   793|         focus_handle: FocusHandle,
   794|     },
   795|     /// Showing a select prompt from a script (multi-select)
   796|     SelectPrompt {
   797|         #[allow(dead_code)]
   798|         id: String,
   799|         entity: Entity<SelectPrompt>,
   800|     },
   801|     /// Showing a path prompt from a script (file/folder picker)
   802|     PathPrompt {
   803|         #[allow(dead_code)]
   804|         id: String,
   805|         entity: Entity<PathPrompt>,
   806|         focus_handle: FocusHandle,
   807|     },
   808|     /// Showing env prompt for environment variable input with keyring storage
   809|     EnvPrompt {
   810|         #[allow(dead_code)]
   811|         id: String,
   812|         entity: Entity<EnvPrompt>,
   813|     },
   814|     /// Showing drop prompt for drag and drop file handling
   815|     DropPrompt {
   816|         #[allow(dead_code)]
   817|         id: String,
   818|         entity: Entity<DropPrompt>,
   819|     },
   820|     /// Showing template prompt for string template editing
   821|     TemplatePrompt {
   822|         #[allow(dead_code)]
   823|         id: String,
   824|         entity: Entity<TemplatePrompt>,
   825|     },
   826|     /// Showing chat prompt for conversational interfaces
   827|     ChatPrompt {
   828|         #[allow(dead_code)]
   829|         id: String,
   830|         entity: Entity<prompts::ChatPrompt>,
   831|     },

  ...
   834|     ClipboardHistoryView {
   835|         filter: String,
   836|         selected_index: usize,
   837|     },
   838|     /// Showing app launcher
   839|     /// P0 FIX: View state only - data comes from ScriptListApp.apps or app_launcher module
   840|     AppLauncherView {
   841|         filter: String,
   842|         selected_index: usize,
   843|     },
   844|     /// Showing window switcher
   845|     /// P0 FIX: View state only - windows stored in ScriptListApp.cached_windows
   846|     WindowSwitcherView {
   847|         filter: String,
   848|         selected_index: usize,
   849|     },
   850|     /// Showing design gallery (separator and icon variations)
   851|     DesignGalleryView {
   852|         filter: String,
   853|         selected_index: usize,
   854|     },
   855|     /// Showing scratch pad editor (auto-saves to disk)
   856|     ScratchPadView {
   857|         entity: Entity<EditorPrompt>,
   858|         #[allow(dead_code)]
   859|         focus_handle: FocusHandle,
   860|     },
   861|     /// Showing quick terminal
   862|     QuickTerminalView {
   863|         entity: Entity<term_prompt::TermPrompt>,
   864|     },
   865|     /// Showing file search results
   866|     FileSearchView {
   867|         query: String,
   868|         selected_index: usize,
   869|     },
   870| }
   871| 
   872| /// Wrapper to hold a script session that can be shared across async boundaries
   873| /// Uses parking_lot::Mutex which doesn't poison on panic, avoiding .unwrap() calls
   874| type SharedSession = Arc<ParkingMutex<Option<executor::ScriptSession>>>;
   875| 
   876| /// Tracks which input field currently has focus for cursor display
   877| #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   878| enum FocusedInput {
   879|     /// Main script list filter input
   880|     MainFilter,
   881|     /// Actions dialog search input
   882|     ActionsSearch,
   883|     /// Arg prompt input (when running a script)
   884|     ArgPrompt,
   885|     /// No input focused (e.g., terminal prompt)
   886|     None,
   887| }
   888| 
   889| /// Pending focus target - identifies which element should receive focus
   890| /// when window access becomes available. This prevents the "perpetual focus
   891| /// enforcement in render()" anti-pattern that causes focus thrash.
   892| ///
   893| /// Focus is applied once when pending_focus is set, then cleared.
   894| /// This mechanism allows non-render code paths (like handle_prompt_message)
   895| /// to request focus changes that are applied on the next render.
   896| #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   897| enum FocusTarget {
   898|     /// Focus the main filter input (gpui_input_state)
   899|     MainFilter,
   900|     /// Focus the app root (self.focus_handle)
   901|     AppRoot,
   902|     /// Focus the actions dialog (if open)
   903|     ActionsDialog,
   904|     /// Focus the path prompt's focus handle
   905|     PathPrompt,
   906|     /// Focus the form prompt (delegates to active field)
   907|     FormPrompt,
   908|     /// Focus the editor prompt
   909|     EditorPrompt,
   910|     /// Focus the select prompt
   911|     SelectPrompt,
   912|     /// Focus the env prompt
   913|     EnvPrompt,
   914|     /// Focus the drop prompt
   915|     DropPrompt,
   916|     /// Focus the template prompt
   917|     TemplatePrompt,
   918|     /// Focus the term prompt
   919|     TermPrompt,
   920|     /// Focus the chat prompt
   921|     ChatPrompt,
   922| }

  ...
  1205|     apps: Vec<app_launcher::AppInfo>,
  1206|     /// P0 FIX: Cached clipboard entries for ClipboardHistoryView (avoids cloning per frame)
  1207|     cached_clipboard_entries: Vec<clipboard_history::ClipboardEntryMeta>,
  1208|     /// P0 FIX: Cached windows for WindowSwitcherView (avoids cloning per frame)
  1209|     cached_windows: Vec<window_control::WindowInfo>,
  1210|     /// Cached file results for FileSearchView (avoids cloning per frame)
  1211|     cached_file_results: Vec<file_search::FileResult>,
  1212|     selected_index: usize,
  1213|     /// Main menu filter text (mirrors gpui-component input state)
  1214|     filter_text: String,
  1215|     /// gpui-component input state for the main filter
  1216|     gpui_input_state: Entity<InputState>,
  1217|     gpui_input_focused: bool,
  1218|     #[allow(dead_code)]
  1219|     gpui_input_subscriptions: Vec<Subscription>,
  1220|     /// Subscription for window bounds changes (saves position on drag)
  1221|     #[allow(dead_code)]
  1222|     bounds_subscription: Option<Subscription>,
  1223|     /// Suppress handling of programmatic InputEvent::Change updates.
  1224|     suppress_filter_events: bool,
  1225|     /// Sync gpui input text on next render when window access is available.
  1226|     pending_filter_sync: bool,
  1227|     /// Pending placeholder text to set on next render (needs Window access).
  1228|     pending_placeholder: Option<String>,
  1229|     last_output: Option<SharedString>,
  1230|     focus_handle: FocusHandle,
  1231|     show_logs: bool,
  1232|     /// Theme wrapped in Arc for cheap cloning when passing to prompts/dialogs
  1233|     theme: std::sync::Arc<theme::Theme>,
  1234|     #[allow(dead_code)]
  1235|     config: config::Config,
  1236|     // Scroll activity tracking for scrollbar fade
  1237|     /// Whether scroll activity is happening (scrollbar should be visible)
  1238|     is_scrolling: bool,
  1239|     /// Timestamp of last scroll activity (for fade-out timer)
  1240|     last_scroll_time: Option<std::time::Instant>,
  1241|     // Interactive script state
  1242|     current_view: AppView,
  1243|     script_session: SharedSession,
  1244|     // Prompt-specific state (used when view is ArgPrompt or DivPrompt)
  1245|     // Uses TextInputState for selection and clipboard support
  1246|     arg_input: TextInputState,
  1247|     arg_selected_index: usize,
  1248|     // Channel for receiving prompt messages from script thread (async_channel for event-driven)
  1249|     prompt_receiver: Option<async_channel::Receiver<PromptMessage>>,
  1250|     // Channel for sending responses back to script
  1251|     // FIX: Use SyncSender (bounded channel) to prevent OOM from slow scripts
  1252|     response_sender: Option<mpsc::SyncSender<Message>>,
  1253|     // List state for variable-height list (supports section headers at 24px + items at 48px)
  1254|     main_list_state: ListState,
  1255|     // Scroll handle for uniform_list (still used for backward compat in some views)

  ...
  1262|     window_list_scroll_handle: UniformListScrollHandle,
  1263|     // Scroll handle for design gallery list
  1264|     design_gallery_scroll_handle: UniformListScrollHandle,
  1265|     // Scroll handle for file search list
  1266|     file_search_scroll_handle: UniformListScrollHandle,
  1267|     // File search loading state (true while mdfind is running)
  1268|     file_search_loading: bool,
  1269|     // Debounce task for file search (cancelled when new input arrives)
  1270|     file_search_debounce_task: Option<gpui::Task<()>>,
  1271|     // Current directory being listed (for instant filter mode)
  1272|     file_search_current_dir: Option<String>,
  1273|     // Frozen filter during directory transitions (prevents wrong results flash)
  1274|     // When Some, use this filter instead of deriving from query
  1275|     // Outer Option: None = use query filter, Some = use frozen filter
  1276|     // Inner Option: None = no filter, Some(s) = filter by s
  1277|     file_search_frozen_filter: Option<Option<String>>,
  1278|     // Path of the file selected for actions (for file search actions handling)
  1279|     file_search_actions_path: Option<String>,
  1280|     // Actions popup overlay
  1281|     show_actions_popup: bool,
  1282|     // ActionsDialog entity for focus management
  1283|     actions_dialog: Option<Entity<ActionsDialog>>,
  1284|     // Cursor blink state and focus tracking
  1285|     cursor_visible: bool,
  1286|     /// Which input currently has focus (for cursor display)
  1287|     focused_input: FocusedInput,
  1288|     // Current script process PID for explicit cleanup (belt-and-suspenders)
  1289|     current_script_pid: Option<u32>,
  1290|     // P1: Cache for filtered_results() - invalidate on filter_text change only
  1291|     cached_filtered_results: Vec<scripts::SearchResult>,
  1292|     filter_cache_key: String,
  1293|     // P1: Cache for get_grouped_results() - invalidate on filter_text change only
  1294|     // This avoids recomputing grouped results 9+ times per keystroke
  1295|     // P1-Arc: Use Arc<[T]> for cheap clone in render closures
  1296|     cached_grouped_items: Arc<[GroupedListItem]>,
  1297|     cached_grouped_flat_results: Arc<[scripts::SearchResult]>,
  1298|     grouped_cache_key: String,
  1299|     // P3: Two-stage filter - display vs search separation with coalescing
  1300|     /// What the search cache is built from (may lag behind filter_text during rapid typing)
  1301|     computed_filter_text: String,
  1302|     /// Coalesces filter updates and keeps only the latest value per tick
  1303|     filter_coalescer: FilterCoalescer,
  1304|     // Scroll stabilization: track last scrolled-to index to avoid redundant scroll_to_item calls
  1305|     last_scrolled_index: Option<usize>,
  1306|     // Preview cache: avoid re-reading file and re-highlighting on every render
  1307|     preview_cache_path: Option<String>,
  1308|     preview_cache_lines: Vec<syntax::HighlightedLine>,
  1309|     // Scriptlet preview cache: avoid re-highlighting scriptlet code on every render
  1310|     // Key is scriptlet name (unique within session), value is highlighted lines
  1311|     scriptlet_preview_cache_key: Option<String>,
  1312|     scriptlet_preview_cache_lines: Vec<syntax::HighlightedLine>,

  ...
  1359|     alias_registry: std::collections::HashMap<String, String>,
  1360|     /// Shortcut registry: shortcut -> script_path (for O(1) lookup)
  1361|     /// Conflict rule: first-registered wins
  1362|     shortcut_registry: std::collections::HashMap<String, String>,
  1363|     /// SDK actions set via setActions() - stored for trigger_action_by_name lookup
  1364|     sdk_actions: Option<Vec<protocol::ProtocolAction>>,
  1365|     /// SDK action shortcuts: normalized_shortcut -> action_name (for O(1) lookup)
  1366|     action_shortcuts: std::collections::HashMap<String, String>,
  1367|     /// Debug grid overlay configuration (None = hidden)
  1368|     grid_config: Option<debug_grid::GridConfig>,
  1369|     // Navigation coalescing for rapid arrow key events (20ms window)
  1370|     // NOTE: Currently unused - arrow keys handled in interceptor without coalescing
  1371|     #[allow(dead_code)]
  1372|     nav_coalescer: NavCoalescer,
  1373|     // Wheel scroll accumulator for smooth trackpad scrolling
  1374|     // Accumulates fractional deltas until they cross 1.0, then converts to item steps
  1375|     wheel_accum: f32,
  1376|     // Window focus tracking - for detecting focus lost and auto-dismissing prompts
  1377|     // When window loses focus while in a dismissable prompt, close and reset
  1378|     was_window_focused: bool,
  1379|     /// Pin state - when true, window stays open on blur (only closes via ESC/Cmd+W)
  1380|     /// Toggle with Cmd+Shift+P
  1381|     is_pinned: bool,
  1382|     /// Pending focus target - when set, focus will be applied once on next render
  1383|     /// then cleared. This avoids the "perpetually enforce focus in render()" anti-pattern.
  1384|     pending_focus: Option<FocusTarget>,
  1385|     // Show warning banner when bun is not available
  1386|     show_bun_warning: bool,
  1387|     // Builtin confirmation channel - for modal callback to signal completion
  1388|     // When a dangerous builtin requires confirmation, we open a modal and the callback
  1389|     // sends (entry_id, confirmed) through this channel
  1390|     builtin_confirm_sender: async_channel::Sender<(String, bool)>,
  1391|     builtin_confirm_receiver: async_channel::Receiver<(String, bool)>,
  1392|     // Scroll stabilization: track last scrolled-to index for each scroll handle
  1393|     #[allow(dead_code)]
  1394|     last_scrolled_main: Option<usize>,
  1395|     #[allow(dead_code)]
  1396|     last_scrolled_arg: Option<usize>,
  1397|     #[allow(dead_code)]
  1398|     last_scrolled_clipboard: Option<usize>,
  1399|     #[allow(dead_code)]
  1400|     last_scrolled_window: Option<usize>,
  1401|     #[allow(dead_code)]
  1402|     last_scrolled_design_gallery: Option<usize>,
  1403|     // Menu bar integration: Now handled by frontmost_app_tracker module
  1404|     // which pre-fetches menu items in background when apps activate
  1405|     /// Shortcut recorder state - when Some, shows the inline recorder overlay
  1406|     shortcut_recorder_state: Option<ShortcutRecorderState>,
  1407|     /// The shortcut recorder entity (persisted to maintain focus)
  1408|     shortcut_recorder_entity:
  1409|         Option<Entity<crate::components::shortcut_recorder::ShortcutRecorder>>,

  ...
  1447|     BuiltIn(Arc<builtins::BuiltInEntry>),
  1448|     App(Arc<app_launcher::AppInfo>),
  1449| }
  1450| 
  1451| // Core ScriptListApp implementation extracted to app_impl.rs
  1452| include!("app_impl.rs");
  1453| 
  1454| // Script execution logic (execute_interactive) extracted
  1455| include!("execute_script.rs");
  1456| 
  1457| // Prompt message handling (handle_prompt_message) extracted
  1458| include!("prompt_handler.rs");
  1459| 
  1460| // App navigation methods (selection movement, scrolling)
  1461| include!("app_navigation.rs");
  1462| 
  1463| // App execution methods (execute_builtin, execute_app, execute_window_focus)
  1464| include!("app_execute.rs");
  1465| 
  1466| // App actions handling (handle_action, trigger_action_by_name)
  1467| include!("app_actions.rs");
  1468| 
  1469| // Layout calculation methods (build_component_bounds, build_layout_info)
  1470| include!("app_layout.rs");
  1471| 
  1472| impl Focusable for ScriptListApp {
  1473|     fn focus_handle(&self, _cx: &App) -> FocusHandle {
  1474|         self.focus_handle.clone()
  1475|     }
  1476| }
  1477| 
  1478| impl Render for ScriptListApp {
  1479|     fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
  1480|         // Track render timing for filter perf analysis
  1481|         let render_start = std::time::Instant::now();
  1482|         let filter_snapshot = self.filter_text.clone();
  1483| 
  1484|         // Always log render start for "gr" prefix filters to debug the issue
  1485|         if filter_snapshot.starts_with("gr") {
  1486|             crate::logging::log(
  1487|                 "FILTER_PERF",
  1488|                 &format!(
  1489|                     "[FRAME_START] filter='{}' selected_idx={} view={:?}",
  1490|                     filter_snapshot,
  1491|                     self.selected_index,
  1492|                     match &self.current_view {
  1493|                         AppView::ScriptList => "ScriptList",
  1494|                         _ => "Other",
  1495|                     }
  1496|                 ),
  1497|             );
  1498|         }

  ...
  1566|                 logging::log(
  1567|                     "FOCUS",
  1568|                     "Main window lost focus but actions popup is open - staying open",
  1569|                 );
  1570|             } else if confirm::is_confirm_window_open() {
  1571|                 logging::log(
  1572|                     "FOCUS",
  1573|                     "Main window lost focus but confirm dialog is open - staying open",
  1574|                 );
  1575|             } else if script_kit_gpui::is_within_focus_grace_period() {
  1576|                 logging::log(
  1577|                     "FOCUS",
  1578|                     "Main window lost focus but within grace period - ignoring",
  1579|                 );
  1580|             } else if self.is_pinned {
  1581|                 logging::log(
  1582|                     "FOCUS",
  1583|                     "Main window lost focus but is pinned - staying open",
  1584|                 );
  1585|             }
  1586|         }
  1587|         self.was_window_focused = is_window_focused;
  1588| 
  1589|         // Apply pending focus request (if any). This is the new "apply once" mechanism
  1590|         // that replaces the old "perpetually enforce focus in render()" pattern.
  1591|         // Focus is applied exactly once when pending_focus is set, then cleared.
  1592|         self.apply_pending_focus(window, cx);
  1593| 
  1594|         // Sync filter input if needed (views that use shared input)
  1595|         if matches!(
  1596|             self.current_view,
  1597|             AppView::ScriptList
  1598|                 | AppView::ClipboardHistoryView { .. }
  1599|                 | AppView::AppLauncherView { .. }
  1600|                 | AppView::WindowSwitcherView { .. }
  1601|                 | AppView::FileSearchView { .. }
  1602|         ) {
  1603|             self.sync_filter_input_if_needed(window, cx);
  1604|         }
  1605| 
  1606|         // NOTE: Prompt messages are now handled via event-driven async_channel listener
  1607|         // spawned in execute_interactive() - no polling needed in render()
  1608| 
  1609|         // P0-4: Clone current_view only for dispatch (needed to call &mut self methods)
  1610|         // The clone is unavoidable due to borrow checker: we need &mut self for render methods
  1611|         // but also need to match on self.current_view. Future optimization: refactor render
  1612|         // methods to take &str/&[T] references instead of owned values.
  1613|         //
  1614|         // HUD is now handled by hud_manager as a separate floating window
  1615|         // No need to render it as part of this view
  1616|         let current_view = self.current_view.clone();
  1617|         let main_content: AnyElement = match current_view {

  ...
  3146|                                         // Check for Cmd+K to toggle actions popup
  3147|                                         if has_cmd && key_lower == "k" {
  3148|                                             logging::log("STDIN", "SimulateKey: Cmd+K - toggle arg actions");
  3149|                                             view.toggle_arg_actions(ctx, window);
  3150|                                         } else if view.show_actions_popup {
  3151|                                             // If actions popup is open, route to it
  3152|                                             if let Some(ref dialog) = view.actions_dialog {
  3153|                                                 match key_lower.as_str() {
  3154|                                                     "up" | "arrowup" => {
  3155|                                                         logging::log("STDIN", "SimulateKey: Up in actions dialog");
  3156|                                                         dialog.update(ctx, |d, cx| d.move_up(cx));
  3157|                                                     }
  3158|                                                     "down" | "arrowdown" => {
  3159|                                                         logging::log("STDIN", "SimulateKey: Down in actions dialog");
  3160|                                                         dialog.update(ctx, |d, cx| d.move_down(cx));
  3161|                                                     }
  3162|                                                     "enter" => {
  3163|                                                         logging::log("STDIN", "SimulateKey: Enter in actions dialog");
  3164|                                                         let action_id = dialog.read(ctx).get_selected_action_id();
  3165|                                                         let should_close = dialog.read(ctx).selected_action_should_close();
  3166|                                                         if let Some(action_id) = action_id {
  3167|                                                             logging::log("ACTIONS", &format!("SimulateKey: Executing action: {} (close={})", action_id, should_close));
  3168|                                                             if should_close {
  3169|                                                                 view.show_actions_popup = false;
  3170|                                                                 view.actions_dialog = None;
  3171|                                                                 view.focused_input = FocusedInput::ArgPrompt;
  3172|                                                                 window.focus(&view.focus_handle, ctx);
  3173|                                                             }
  3174|                                                             view.trigger_action_by_name(&action_id, ctx);
  3175|                                                         }
  3176|                                                     }
  3177|                                                     "escape" => {
  3178|                                                         logging::log("STDIN", "SimulateKey: Escape - close actions dialog");
  3179|                                                         view.show_actions_popup = false;
  3180|                                                         view.actions_dialog = None;
  3181|                                                         view.focused_input = FocusedInput::ArgPrompt;
  3182|                                                         window.focus(&view.focus_handle, ctx);
  3183|                                                     }
  3184|                                                     _ => {
  3185|                                                         logging::log("STDIN", &format!("SimulateKey: Unhandled key '{}' in ArgPrompt actions dialog", key_lower));
  3186|                                                     }
  3187|                                                 }
  3188|                                             }
  3189|                                         } else {
  3190|                                             // Normal arg prompt key handling
  3191|                                             let prompt_id = id.clone();
  3192|                                             match key_lower.as_str() {
  3193|                                                 "up" | "arrowup" => {
  3194|                                                     if view.arg_selected_index > 0 {
  3195|                                                         view.arg_selected_index -= 1;
  3196|                                                         view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
  3197|                                                         logging::log("STDIN", &format!("SimulateKey: Arg up, index={}", view.arg_selected_index));
  3198|                                                     }
  3199|                                                 }
  3200|                                                 "down" | "arrowdown" => {
  3201|                                                     let filtered = view.filtered_arg_choices();
  3202|                                                     if view.arg_selected_index < filtered.len().saturating_sub(1) {
  3203|                                                         view.arg_selected_index += 1;
  3204|                                                         view.arg_list_scroll_handle.scroll_to_item(view.arg_selected_index, ScrollStrategy::Nearest);
  3205|                                                         logging::log("STDIN", &format!("SimulateKey: Arg down, index={}", view.arg_selected_index));
  3206|                                                     }
</file>

<file path="src/app_shell/focus.rs" matches="5" windows="2">
     1| //! Focus management for the shell
     2| //!
     3| //! Centralized focus handling - focus handles are created once and owned by
     4| //! the window root. The shell receives them by reference and applies focus
     5| //! transitions once per view change (not every render).
     6| 
     7| use gpui::{App, Context, FocusHandle, Window};
     8| 
     9| /// Focus policy for a view
    10| ///
    11| /// Determines where focus should land when a view becomes active.
    12| /// Applied once per transition, not every render.
    13| #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    14| pub enum FocusPolicy {
    15|     /// Don't change focus when view becomes active
    16|     /// Used for: HUD notifications, overlays that shouldn't steal focus
    17|     Preserve,
    18| 
    19|     /// Focus the header input (search box)
    20|     /// Used for: ScriptList, ArgPrompt, most prompts with search
    21|     #[default]
    22|     HeaderInput,
    23| 
    24|     /// Focus the main content area
    25|     /// Used for: EditorPrompt (focus the editor), TermPrompt
    26|     Content,
    27| }
    28| 
    29| /// Stable focus handles for the shell
    30| ///
    31| /// Created once per window and stored in the root state.
    32| /// The shell receives these by reference - it never creates focus handles.
    33| pub struct ShellFocus {
    34|     /// Root focus handle for track_focus
    35|     pub shell: FocusHandle,
    36|     /// Focus handle for the header input (search box)
    37|     pub header_input: FocusHandle,
    38|     /// Focus handle for the main content area
    39|     pub content: FocusHandle,
    40| }
    41| 
    42| impl ShellFocus {
    43|     /// Create a new ShellFocus with fresh handles from the context
    44|     pub fn new<V: 'static>(cx: &mut Context<V>) -> Self {
    45|         Self {
    46|             shell: cx.focus_handle(),
    47|             header_input: cx.focus_handle(),
    48|             content: cx.focus_handle(),
    49|         }
    50|     }
    51| 
    52|     /// Apply a focus policy
    53|     ///
    54|     /// This should be called once per view transition, not every render.
    55|     /// The caller tracks whether the view has changed and calls this accordingly.
    56|     pub fn apply_policy(&self, policy: FocusPolicy, window: &mut Window, cx: &mut App) {
    57|         match policy {
    58|             FocusPolicy::Preserve => {
    59|                 // Do nothing - let existing focus remain
    60|             }
    61|             FocusPolicy::HeaderInput => {
    62|                 self.header_input.focus(window, cx);
    63|             }
    64|             FocusPolicy::Content => {

  ...
    73|     }
    74| 
    75|     /// Check if the content area is focused
    76|     pub fn is_content_focused(&self, window: &Window) -> bool {
    77|         self.content.is_focused(window)
    78|     }
    79| 
    80|     /// Check if any shell focus handle is focused
    81|     pub fn is_any_focused(&self, window: &Window) -> bool {
    82|         self.shell.is_focused(window)
    83|             || self.header_input.is_focused(window)
    84|             || self.content.is_focused(window)
    85|     }
    86| 
    87|     /// Focus the header input
    88|     pub fn focus_header(&self, window: &mut Window, cx: &mut App) {
    89|         self.header_input.focus(window, cx);
    90|     }
    91| 
    92|     /// Focus the content area
    93|     pub fn focus_content(&self, window: &mut Window, cx: &mut App) {
    94|         self.content.focus(window, cx);
    95|     }
    96| 
    97|     /// Get the shell root focus handle for track_focus
    98|     pub fn root_handle(&self) -> &FocusHandle {
    99|         &self.shell
   100|     }
   101| }
   102| 
</file>

</files>
---
## Boilerplate Pattern (repeated ~15 times per prompt)

```rust
pub struct MyPrompt {
    pub focus_handle: FocusHandle,
}

impl Focusable for MyPrompt {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for MyPrompt {
    fn render(...) {
        div().track_focus(&self.focus_handle)
    }
}
```

## Recommendation: Centralized FocusManager

```rust
pub struct FocusManager {
    handles: HashMap<FocusId, FocusHandle>,
    current: Option<FocusId>,
    history: Vec<FocusId>,
}

impl FocusManager {
    pub fn request(&mut self, id: FocusId) { ... }
    pub fn pop(&mut self) { ... }  // ESC navigation
    pub fn apply(&self, window: &mut Window, cx: &mut App) { ... }
}
```

Would replace: FocusedInput, FocusTarget, pending_focus, per-component handles.
