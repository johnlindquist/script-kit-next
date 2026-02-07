impl EditorPrompt {
    /// Create a new EditorPrompt with explicit height
    ///
    /// This is the compatible constructor that matches the original EditorPrompt API.
    /// The InputState is created lazily on first render when window is available.
    #[allow(clippy::too_many_arguments)]
    pub fn with_height(
        id: String,
        content: String,
        language: String,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<Theme>,
        config: Arc<Config>,
        content_height: Option<gpui::Pixels>,
    ) -> Self {
        logging::log(
            "EDITOR",
            &format!(
                "EditorPrompt::with_height id={}, lang={}, content_len={}, height={:?}",
                id,
                language,
                content.len(),
                content_height
            ),
        );

        Self {
            id,
            editor_state: None, // Created on first render
            pending_init: Some(PendingInit {
                content,
                language: language.clone(),
            }),
            snippet_state: None,
            language,
            focus_handle,
            on_submit,
            theme,
            config,
            content_height,
            subscriptions: Vec::new(),
            suppress_keys: false,
            choices_popup: None,
            needs_focus: true, // Auto-focus on first render
            needs_initial_tabstop_selection: false,
        }
    }

    /// Create a new EditorPrompt in template/snippet mode
    ///
    /// Parses the template for VSCode-style tabstops and enables Tab/Shift+Tab navigation.
    /// Template syntax:
    /// - `$1`, `$2`, `$3` - Simple tabstops (numbered positions)
    /// - `${1:default}` - Tabstops with placeholder text
    /// - `${1|a,b,c|}` - Choice tabstops (first choice is used as default)
    /// - `$0` - Final cursor position
    #[allow(clippy::too_many_arguments)]
    pub fn with_template(
        id: String,
        template: String,
        language: String,
        focus_handle: FocusHandle,
        on_submit: SubmitCallback,
        theme: Arc<Theme>,
        config: Arc<Config>,
        content_height: Option<gpui::Pixels>,
    ) -> Self {
        logging::log(
            "EDITOR",
            &format!(
                "EditorPrompt::with_template id={}, lang={}, template_len={}, height={:?}",
                id,
                language,
                template.len(),
                content_height
            ),
        );

        // Parse the template for tabstops
        let snippet = ParsedSnippet::parse(&template);

        logging::log(
            "EDITOR",
            &format!(
                "Template parsed: {} tabstops, expanded_len={}",
                snippet.tabstops.len(),
                snippet.text.len()
            ),
        );

        // If there are tabstops, set up snippet state
        let (content, snippet_state, needs_initial_selection) = if snippet.tabstops.is_empty() {
            // No tabstops - use the expanded text as plain content
            (snippet.text.clone(), None, false)
        } else {
            // Has tabstops - set up navigation state
            // Initialize current_values with the original placeholder text
            let current_values: Vec<String> = snippet
                .tabstops
                .iter()
                .map(|ts| {
                    ts.placeholder
                        .clone()
                        .or_else(|| ts.choices.as_ref().and_then(|c| c.first().cloned()))
                        .unwrap_or_default()
                })
                .collect();

            // Initialize last_selection_ranges from the original ranges
            let last_selection_ranges: Vec<Option<(usize, usize)>> = snippet
                .tabstops
                .iter()
                .map(|ts| ts.ranges.first().copied())
                .collect();

            let state = SnippetState {
                snippet: snippet.clone(),
                current_tabstop_idx: 0, // Start at first tabstop
                current_values,
                last_selection_ranges,
            };
            (snippet.text.clone(), Some(state), true)
        };

        Self {
            id,
            editor_state: None, // Created on first render
            pending_init: Some(PendingInit {
                content,
                language: language.clone(),
            }),
            snippet_state,
            language,
            focus_handle,
            on_submit,
            theme,
            config,
            content_height,
            subscriptions: Vec::new(),
            suppress_keys: false,
            choices_popup: None,
            needs_focus: true, // Auto-focus on first render
            needs_initial_tabstop_selection: needs_initial_selection,
        }
    }

    /// Initialize the editor state (called on first render)
    fn ensure_initialized(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.editor_state.is_some() {
            return; // Already initialized
        }

        let Some(pending) = self.pending_init.take() else {
            logging::log("EDITOR", "Warning: No pending init data");
            return;
        };

        logging::log(
            "EDITOR",
            &format!(
                "Initializing editor state: lang={}, content_len={}",
                pending.language,
                pending.content.len()
            ),
        );

        // Create the gpui-component InputState in code_editor mode
        // Enable tab_navigation mode if we're in snippet mode (Tab moves between tabstops)
        let in_snippet = self.snippet_state.is_some();
        let editor_state = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor(&pending.language) // Sets up syntax highlighting
                .searchable(true) // Enable Cmd+F find/replace
                .line_number(false) // No line numbers - cleaner UI
                .soft_wrap(false) // Code should not wrap by default
                .default_value(pending.content)
                .tab_navigation(in_snippet) // Propagate Tab when in snippet mode
        });

        // Subscribe to editor changes
        let editor_sub = cx.subscribe_in(&editor_state, window, {
            move |_this, _, ev: &InputEvent, _window, cx| match ev {
                InputEvent::Change => {
                    cx.notify();
                }
                InputEvent::PressEnter { secondary: _ } => {
                    // Multi-line editor handles Enter internally for newlines
                }
                InputEvent::Focus => {
                    logging::log("EDITOR", "Editor focused");
                }
                InputEvent::Blur => {
                    logging::log("EDITOR", "Editor blurred");
                }
            }
        });

        self.subscriptions = vec![editor_sub];
        self.editor_state = Some(editor_state);

        logging::log("EDITOR", "Editor initialized, focus pending");

        // CRITICAL: Notify to trigger re-render after initialization
        // Without this, the layout may not be computed correctly until a focus change
        cx.notify();
    }

    /// Get the current content as a String
    pub fn content(&self, cx: &Context<Self>) -> String {
        self.editor_state
            .as_ref()
            .map(|state| state.read(cx).value().to_string())
            .unwrap_or_else(|| {
                // Fall back to pending content if not yet initialized
                self.pending_init
                    .as_ref()
                    .map(|p| p.content.clone())
                    .unwrap_or_default()
            })
    }

    /// Get the language
    #[allow(dead_code)]
    pub fn language(&self) -> &str {
        &self.language
    }

}
