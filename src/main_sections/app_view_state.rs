/// Application state - what view are we currently showing
#[derive(Debug, Clone)]
enum AppView {
    /// Showing the script list
    ScriptList,
    /// Showing the actions dialog (mini searchable popup)
    #[allow(dead_code)]
    ActionsDialog,
    /// Showing an arg prompt from a script
    ArgPrompt {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
        actions: Option<Vec<ProtocolAction>>,
    },
    /// Showing a div prompt from a script
    DivPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<DivPrompt>,
    },
    /// Showing a form prompt from a script (HTML form with submit button)
    FormPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<FormPromptState>,
    },
    /// Showing a terminal prompt from a script
    TermPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<term_prompt::TermPrompt>,
    },
    /// Showing an editor prompt from a script (gpui-component based with Find/Replace)
    EditorPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<EditorPrompt>,
        /// Separate focus handle for the editor (not shared with parent)
        /// Note: This is kept for API compatibility but focus is managed via entity.focus()
        #[allow(dead_code)]
        focus_handle: FocusHandle,
    },
    /// Showing a select prompt from a script (multi-select)
    SelectPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<SelectPrompt>,
    },
    /// Showing a path prompt from a script (file/folder picker)
    PathPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<PathPrompt>,
        focus_handle: FocusHandle,
    },
    /// Showing env prompt for environment variable input with keyring storage
    EnvPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<EnvPrompt>,
    },
    /// Showing drop prompt for drag and drop file handling
    DropPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<DropPrompt>,
    },
    /// Showing template prompt for string template editing
    TemplatePrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<TemplatePrompt>,
    },
    /// Showing chat prompt for conversational interfaces
    ChatPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<prompts::ChatPrompt>,
    },
    /// Compact single-line arg prompt (mini variant)
    MiniPrompt {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
    },
    /// Ultra-compact inline arg prompt (micro variant)
    MicroPrompt {
        id: String,
        placeholder: String,
        choices: Vec<Choice>,
    },
    /// Showing clipboard history
    /// P0 FIX: View state only - data comes from clipboard_history module cache
    ClipboardHistoryView {
        filter: String,
        selected_index: usize,
    },
    /// Showing app launcher
    /// P0 FIX: View state only - data comes from ScriptListApp.apps or app_launcher module
    AppLauncherView {
        filter: String,
        selected_index: usize,
    },
    /// Showing window switcher
    /// P0 FIX: View state only - windows stored in ScriptListApp.cached_windows
    WindowSwitcherView {
        filter: String,
        selected_index: usize,
    },
    /// Showing design gallery (separator and icon variations)
    DesignGalleryView {
        filter: String,
        selected_index: usize,
    },
    /// Showing the in-app storybook compare view for design exploration
    #[cfg(feature = "storybook")]
    DesignExplorerView {
        entity: Entity<script_kit_gpui::storybook::StoryBrowser>,
    },
    /// Showing webcam prompt
    WebcamView {
        entity: Entity<prompts::WebcamPrompt>,
    },
    /// Showing scratch pad editor (auto-saves to disk)
    ScratchPadView {
        entity: Entity<EditorPrompt>,
        #[allow(dead_code)]
        focus_handle: FocusHandle,
    },
    /// Showing quick terminal
    QuickTerminalView {
        entity: Entity<term_prompt::TermPrompt>,
    },
    /// Showing file search results
    FileSearchView {
        query: String,
        selected_index: usize,
    },
    /// Showing theme chooser with live preview and search
    ThemeChooserView {
        filter: String,
        selected_index: usize,
    },
    /// Showing emoji picker grid with category sections
    EmojiPickerView {
        filter: String,
        selected_index: usize,
        selected_category: Option<crate::emoji::EmojiCategory>,
    },
    /// Showing naming dialog for script/extension creation.
    /// Non-dismissable — requires explicit submit or cancel.
    NamingPrompt {
        #[allow(dead_code)]
        id: String,
        entity: Entity<prompts::NamingPrompt>,
    },
    /// Showing creation feedback with file path and quick actions after script/extension creation.
    /// Requires explicit dismiss (Enter/Escape/button) — non-dismissable by click-outside.
    CreationFeedback { path: std::path::PathBuf },
    /// Browsing the Kit Store (GitHub search for installable kits)
    BrowseKitsView {
        query: String,
        selected_index: usize,
        results: Vec<KitStoreSearchResult>,
    },
    /// Managing locally installed kits (update/remove)
    InstalledKitsView {
        selected_index: usize,
        kits: Vec<script_kit_gpui::kit_store::InstalledKit>,
    },
    /// Showing process manager (running background scripts)
    /// Data comes from cached_processes field populated on open
    ProcessManagerView {
        filter: String,
        selected_index: usize,
    },
    /// Showing searchable list of saved AI presets
    /// Selecting a preset opens AI chat with its system prompt and model
    SearchAiPresetsView {
        filter: String,
        selected_index: usize,
    },
    /// Showing create AI preset form
    /// Name + system prompt + model fields
    CreateAiPresetView {
        name: String,
        system_prompt: String,
        model: String,
        active_field: usize,
    },
    /// Showing settings hub with configuration panels
    /// Lists categories: API Keys, Theme, Window Positions, Feature Toggles, Hotkeys
    SettingsView {
        selected_index: usize,
    },
    /// Browsing favorites with search/filter
    /// Supports Enter to run, D to remove, U/J to reorder, Esc to go back
    FavoritesBrowseView {
        filter: String,
        selected_index: usize,
    },
    /// Showing menu bar commands from the frontmost application
    /// Data comes from cached_current_app_entries populated on open
    CurrentAppCommandsView {
        filter: String,
        selected_index: usize,
    },
    /// Full-view Tab AI chat surface. Replaces the current view (like ChatPrompt)
    /// instead of painting over it as an overlay.
    TabAiChat {
        entity: Entity<TabAiChat>,
    },
}

/// Wrapper to hold a script session that can be shared across async boundaries
/// Uses parking_lot::Mutex which doesn't poison on panic, avoiding .unwrap() calls
type SharedSession = Arc<ParkingMutex<Option<executor::ScriptSession>>>;

/// Tracks which input field currently has focus for cursor display
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusedInput {
    /// Main script list filter input
    MainFilter,
    /// Actions dialog search input
    ActionsSearch,
    /// Arg prompt input (when running a script)
    ArgPrompt,
    /// No input focused (e.g., terminal prompt)
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum MainWindowMode {
    Full,
    #[default]
    Mini,
}

/// Pending focus target - identifies which element should receive focus
/// when window access becomes available. This prevents the "perpetual focus
/// enforcement in render()" anti-pattern that causes focus thrash.
///
/// Focus is applied once when pending_focus is set, then cleared.
/// This mechanism allows non-render code paths (like handle_prompt_message)
/// to request focus changes that are applied on the next render.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FocusTarget {
    /// Focus the main filter input (gpui_input_state)
    MainFilter,
    /// Focus the app root (self.focus_handle)
    AppRoot,
    /// Focus the actions dialog (if open)
    ActionsDialog,
    /// Focus the path prompt's focus handle
    PathPrompt,
    /// Focus the form prompt (delegates to active field)
    FormPrompt,
    /// Focus the editor prompt
    EditorPrompt,
    /// Focus the select prompt
    SelectPrompt,
    /// Focus the env prompt
    EnvPrompt,
    /// Focus the drop prompt
    DropPrompt,
    /// Focus the template prompt
    TemplatePrompt,
    /// Focus the term prompt
    TermPrompt,
    /// Focus the chat prompt
    ChatPrompt,
    /// Focus the Tab AI chat prompt
    TabAiChat,
    /// Focus the naming prompt
    NamingPrompt,
}

/// Identifies which prompt type is hosting the actions dialog.
///
/// This determines focus restoration behavior when the dialog closes,
/// since different prompt types have different focus targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // MainList variant reserved for render_script_list.rs refactoring
enum ActionsDialogHost {
    /// Actions in arg prompt (restore focus to ArgPrompt input)
    ArgPrompt,
    /// Actions in div prompt (restore focus to None - div has no input)
    DivPrompt,
    /// Actions in editor prompt (restore focus to None - editor handles its own focus)
    EditorPrompt,
    /// Actions in term prompt (restore focus to None - terminal handles its own focus)
    TermPrompt,
    /// Actions in form prompt (restore focus to None - form handles field focus)
    FormPrompt,
    /// Actions in chat prompt (restore focus to ChatPrompt input)
    ChatPrompt,
    /// Actions in main script list (restore focus to MainFilter)
    MainList,
    /// Actions in file search (restore focus to file search input)
    FileSearch,
    /// Actions in clipboard history (restore focus to clipboard search input)
    ClipboardHistory,
    /// Actions in emoji picker (restore focus to emoji search input)
    EmojiPicker,
    /// Actions in app launcher (restore focus to app launcher input)
    AppLauncher,
    /// Actions in webcam prompt (restore focus to None - webcam has no input)
    WebcamPrompt,
}

/// Input mode for list navigation - tracks whether user is using keyboard or mouse.
/// When in Keyboard mode, hover effects are disabled to prevent dual-highlight.
/// Mouse movement switches back to Mouse mode, re-enabling hover.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputMode {
    #[default]
    Mouse,
    Keyboard,
}

/// Result of routing a key event to the actions dialog.
///
/// Returned by `route_key_to_actions_dialog` to indicate how the caller
/// should proceed after routing.
#[derive(Debug, Clone)]
enum ActionsRoute {
    /// Actions popup is not open - key was not handled, caller should process normally
    NotHandled,
    /// Key was handled by the actions dialog - caller should return/stop propagation
    Handled,
    /// User selected an action - caller should execute it via trigger_action_by_name
    Execute { action_id: String },
}

/// File-search preview thumbnail lifecycle state.
///
/// Tracks async thumbnail loading for the right-side FileSearch preview panel.
/// The `path` field on non-idle variants guards against stale async updates.
#[derive(Clone)]
enum FileSearchThumbnailPreviewState {
    /// No thumbnail should be rendered (no selection or non-image selection).
    Idle,
    /// Thumbnail load is in-flight for this path.
    Loading {
        path: String,
    },
    /// Thumbnail loaded successfully with decoded image and dimensions.
    Ready {
        path: String,
        image: Arc<gpui::RenderImage>,
        width: u32,
        height: u32,
    },
    /// Thumbnail not available for this path (size/format/decode constraints).
    Unavailable {
        path: String,
        message: String,
    },
}

/// State for the inline shortcut recorder overlay.
///
/// When this is Some, the ShortcutRecorder modal is displayed.
/// Used for configuring keyboard shortcuts without opening an external editor.
#[derive(Debug, Clone)]
struct ShortcutRecorderState {
    /// The unique command identifier (e.g., "scriptlet/my-script", "builtin/clipboard-history")
    command_id: String,
    /// Human-readable name of the command being configured
    command_name: String,
}

/// State for the inline alias input overlay.
///
/// When this is Some, the alias input modal is displayed.
/// Used for configuring command aliases.
#[derive(Debug, Clone)]
struct AliasInputState {
    /// The unique command identifier (e.g., "builtin/clipboard-history", "app/com.apple.Safari")
    command_id: String,
    /// Human-readable name of the command being configured
    command_name: String,
    /// Current alias text being edited
    alias_text: String,
}

/// State for the Tab AI save-offer overlay, shown after a successful ephemeral
/// script execution when the user is prompted to persist the script.
#[derive(Debug, Clone)]
struct TabAiSaveOfferState {
    /// The execution record for the completed Tab AI run.
    record: crate::ai::TabAiExecutionRecord,
    /// Derived filename stem (e.g. "force-quit-this-app").
    filename_stem: String,
    /// Error message if save attempt failed, shown inline in the overlay.
    error: Option<SharedString>,
}

/// The kind of context a Tab AI card represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabAiContextCardKind {
    ExperienceStrip,
    SelectedItem,
    FilterText,
    VisibleItems,
    Desktop,
    Clipboard,
    PriorAutomations,
}

/// A single key→value row inside a context card.
#[derive(Debug, Clone)]
struct TabAiContextRow {
    label: SharedString,
    value: SharedString,
}

impl TabAiContextRow {
    fn new(label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

/// A suggested intent pill shown in the Tab AI empty state.
#[derive(Debug, Clone)]
struct TabAiSuggestedIntent {
    label: SharedString,
    intent: SharedString,
}

impl From<crate::ai::TabAiSuggestedIntentSpec> for TabAiSuggestedIntent {
    fn from(value: crate::ai::TabAiSuggestedIntentSpec) -> Self {
        Self {
            label: value.label.into(),
            intent: value.intent.into(),
        }
    }
}

/// A context card shown in the Tab AI chat empty state.
#[derive(Debug, Clone)]
struct TabAiContextCard {
    kind: TabAiContextCardKind,
    label: SharedString,
    title: SharedString,
    body: Option<SharedString>,
    rows: Vec<TabAiContextRow>,
    suggestions: Vec<TabAiSuggestedIntent>,
}

/// Kind of a turn in the Tab AI chat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TabAiTurnKind {
    User,
    AssistantText,
    AssistantCode,
}

/// A single turn (message) in the Tab AI chat.
#[derive(Debug, Clone)]
struct TabAiTurn {
    kind: TabAiTurnKind,
    body: SharedString,
    /// Whether this turn is currently being streamed (assistant response in progress).
    streaming: bool,
}

/// Full-view Tab AI chat entity. Replaces the overlay approach with a proper
/// entity-backed view that stores return state, input via `TextInputState`,
/// scrollable turns via `ListState`, and focus via `FocusHandle`.
struct TabAiChat {
    /// The view to restore when closing the chat.
    return_view: AppView,
    /// The focus target to restore when closing the chat.
    return_focus_target: FocusTarget,
    /// Focus handle for this entity (tracked in render).
    focus_handle: FocusHandle,
    /// Single-line text input with selection, clipboard, and undo support.
    input: TextInputState,
    /// Variable-height list state for chat turns (bottom-anchored).
    turns_list_state: ListState,
    /// UI snapshot captured when Tab was pressed.
    ui_snapshot: crate::ai::TabAiUiSnapshot,
    /// Machine-readable invocation receipt.
    invocation_receipt: crate::ai::TabAiInvocationReceipt,
    /// Frontmost app bundle ID captured at open time.
    frontmost_bundle_id: Option<String>,
    /// Best prior automation hint for the current intent.
    memory_hint: Option<crate::ai::TabAiMemorySuggestion>,
    /// Desktop snapshot captured at open time. Submitted as-is so the model
    /// always sees the same context the user saw in the preview cards.
    preview_desktop_snapshot: crate::context_snapshot::AiContextSnapshot,
    /// Context cards shown as the empty state before the user types.
    context_cards: Vec<TabAiContextCard>,
    /// Index of the currently highlighted suggestion pill (wraps around).
    selected_suggestion_index: usize,
    /// Chat turns (user messages and AI responses).
    turns: Vec<TabAiTurn>,
    /// Cursor blink visibility (toggled by timer).
    cursor_visible: bool,
    /// Whether an AI call is in-flight.
    running: bool,
    /// Error message from the last failed attempt.
    error: Option<SharedString>,
}

impl TabAiChat {
    fn new(
        return_view: AppView,
        return_focus_target: FocusTarget,
        ui_snapshot: crate::ai::TabAiUiSnapshot,
        invocation_receipt: crate::ai::TabAiInvocationReceipt,
        frontmost_bundle_id: Option<String>,
        preview_desktop_snapshot: crate::context_snapshot::AiContextSnapshot,
        context_cards: Vec<TabAiContextCard>,
        focus_handle: FocusHandle,
    ) -> Self {
        Self {
            return_view,
            return_focus_target,
            focus_handle,
            input: TextInputState::new(),
            turns_list_state: ListState::new(0, ListAlignment::Bottom, px(1024.0)),
            ui_snapshot,
            invocation_receipt,
            frontmost_bundle_id,
            memory_hint: None,
            preview_desktop_snapshot,
            context_cards,
            selected_suggestion_index: 0,
            turns: Vec::new(),
            cursor_visible: true,
            running: false,
            error: None,
        }
    }

    fn restore_target(&self) -> (AppView, FocusTarget) {
        (self.return_view.clone(), self.return_focus_target)
    }

    fn current_intent(&self) -> String {
        self.input.text().to_string()
    }

    fn can_submit(&self) -> bool {
        !self.running && !self.input.text().trim().is_empty()
    }

    /// Collect up to 3 suggestion pills across all context cards, prioritising
    /// the Experience card so its intents appear first in the keyboard path.
    fn context_suggestions(&self) -> Vec<TabAiSuggestedIntent> {
        const PRIORITY_ORDER: &[TabAiContextCardKind] = &[
            TabAiContextCardKind::ExperienceStrip,
            TabAiContextCardKind::SelectedItem,
            TabAiContextCardKind::Desktop,
            TabAiContextCardKind::PriorAutomations,
            TabAiContextCardKind::Clipboard,
            TabAiContextCardKind::FilterText,
            TabAiContextCardKind::VisibleItems,
        ];
        let mut ordered = Vec::new();
        for kind in PRIORITY_ORDER {
            ordered.extend(
                self.context_cards
                    .iter()
                    .filter(|card| card.kind == *kind)
                    .flat_map(|card| card.suggestions.iter().cloned()),
            );
            if ordered.len() >= 3 {
                break;
            }
        }
        ordered.truncate(3);
        ordered
    }

    /// Cycle the selected suggestion index by `delta` (wrapping).
    fn move_selected_suggestion(&mut self, delta: isize) {
        let count = self.context_suggestions().len();
        if count == 0 {
            self.selected_suggestion_index = 0;
            return;
        }
        self.selected_suggestion_index =
            (self.selected_suggestion_index as isize + delta).rem_euclid(count as isize) as usize;
    }

    /// Return the currently highlighted suggestion, if any.
    fn selected_suggestion(&self) -> Option<TabAiSuggestedIntent> {
        self.context_suggestions()
            .get(self.selected_suggestion_index)
            .cloned()
    }

    /// Dynamic placeholder based on the selected suggestion.
    fn input_placeholder(&self) -> SharedString {
        self.selected_suggestion()
            .map(|suggestion| {
                SharedString::from(format!("{} or type your own\u{2026}", suggestion.intent))
            })
            .unwrap_or_else(|| "What do you want to do?".into())
    }

    fn refresh_memory_hint(&mut self) {
        let intent = self.input.text().trim().to_string();
        self.memory_hint = match crate::ai::resolve_tab_ai_memory_suggestions_with_outcome(
            &intent,
            self.frontmost_bundle_id.as_deref(),
            1,
        ) {
            Ok(resolution) => resolution.suggestions.into_iter().next(),
            Err(error) => {
                tracing::warn!(event = "tab_ai_memory_hint_failed", error = %error);
                None
            }
        };
    }

    fn sync_turns_list_state(&mut self) {
        let old_count = self.turns_list_state.item_count();
        let new_count = self.turns.len();
        if old_count != new_count {
            self.turns_list_state.splice(0..old_count, new_count);
        } else if self
            .turns
            .last()
            .map(|turn| turn.streaming)
            .unwrap_or(false)
            && new_count > 0
        {
            // Re-measure the last item when streaming content into it
            let last = new_count - 1;
            self.turns_list_state.splice(last..new_count, 1);
        }
        if new_count > 0 {
            self.turns_list_state.set_follow_tail(true);
        }
    }

    /// Start a new assistant turn that will be streamed into progressively.
    /// Returns the turn index for use with `append_turn_chunk` and `complete_turn_stream`.
    fn start_assistant_turn(&mut self, kind: TabAiTurnKind) -> usize {
        self.turns.push(TabAiTurn {
            kind,
            body: SharedString::from(String::new()),
            streaming: true,
        });
        self.sync_turns_list_state();
        self.turns.len() - 1
    }

    /// Change the kind of an existing turn (e.g. from AssistantText to AssistantCode).
    fn set_turn_kind(&mut self, turn_index: usize, kind: TabAiTurnKind) {
        if let Some(turn) = self.turns.get_mut(turn_index) {
            turn.kind = kind;
        }
    }

    /// Append a text chunk to a streaming assistant turn.
    fn append_turn_chunk(&mut self, turn_index: usize, chunk: &str) {
        if let Some(turn) = self.turns.get_mut(turn_index) {
            let mut next = turn.body.to_string();
            next.push_str(chunk);
            turn.body = next.into();
            self.sync_turns_list_state();
        }
    }

    /// Mark a streaming assistant turn as complete.
    fn complete_turn_stream(&mut self, turn_index: usize) {
        if let Some(turn) = self.turns.get_mut(turn_index) {
            turn.streaming = false;
            self.sync_turns_list_state();
        }
    }

    fn clear_input(&mut self) {
        self.input.clear();
    }

    fn append_turn(&mut self, kind: TabAiTurnKind, body: impl Into<SharedString>) {
        self.turns.push(TabAiTurn {
            kind,
            body: body.into(),
            streaming: false,
        });
        self.sync_turns_list_state();
    }

    fn append_user_turn(&mut self, body: impl Into<SharedString>) {
        self.append_turn(TabAiTurnKind::User, body);
    }

    fn append_assistant_text_turn(&mut self, body: impl Into<SharedString>) {
        self.append_turn(TabAiTurnKind::AssistantText, body);
    }

    fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    fn set_error(&mut self, error: Option<SharedString>) {
        self.error = error;
    }

    /// Start the cursor blink timer. Spawns a detached async task that toggles
    /// `cursor_visible` every 530ms. Skips toggling while running or streaming.
    /// Exits cleanly when the entity is dropped.
    fn start_cursor_blink(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(std::time::Duration::from_millis(530))
                    .await;
                if !crate::is_main_window_visible() {
                    continue;
                }
                let result = cx.update(|cx| {
                    this.update(cx, |chat, cx| {
                        if chat.running
                            || chat
                                .turns
                                .last()
                                .map(|turn| turn.streaming)
                                .unwrap_or(false)
                        {
                            return;
                        }
                        chat.cursor_visible = !chat.cursor_visible;
                        cx.notify();
                    })
                });
                if result.is_err() {
                    break;
                }
            }
        })
        .detach();
    }

    fn collect_elements(&self, limit: usize) -> ElementCollectionOutcome {
        let total_count = self.turns.len() + 2;
        let mut elements = Vec::new();

        if elements.len() < limit {
            elements.push(crate::protocol::ElementInfo::input(
                "tab-ai-input",
                Some(self.input.text()),
                true,
            ));
        }
        if elements.len() < limit {
            elements.push(crate::protocol::ElementInfo::list(
                "tab-ai-turns",
                self.turns.len(),
            ));
        }
        for (index, turn) in self.turns.iter().enumerate() {
            if elements.len() >= limit {
                break;
            }
            let text = turn.body.to_string();
            elements.push(crate::protocol::ElementInfo {
                semantic_id: crate::protocol::generate_semantic_id("choice", index, &text),
                element_type: crate::protocol::ElementType::Choice,
                text: Some(text.clone()),
                value: Some(text),
                selected: Some(false),
                focused: None,
                index: Some(index),
            });
        }

        ElementCollectionOutcome::new(elements, total_count)
    }
}

impl Focusable for TabAiChat {
    fn focus_handle(&self, _cx: &gpui::App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
