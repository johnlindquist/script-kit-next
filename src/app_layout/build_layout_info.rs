impl ScriptListApp {
    pub fn build_layout_info(&self, _cx: &mut gpui::Context<Self>) -> protocol::LayoutInfo {
        use crate::list_item::LIST_ITEM_HEIGHT;
        use protocol::{LayoutComponentInfo, LayoutComponentType, LayoutInfo};

        // Keep automation layout receipts aligned with the same sizing contract
        // used by real window resize paths.
        let layout_view_type = match &self.current_view {
            AppView::ScriptList => match self.main_window_mode {
                MainWindowMode::Full => crate::window_resize::ViewType::ScriptList,
                MainWindowMode::Mini => crate::window_resize::ViewType::MiniMainWindow,
            },
            AppView::FileSearchView { presentation, .. } => match presentation {
                FileSearchPresentation::Full => crate::window_resize::ViewType::ExpandedMainWindow,
                FileSearchPresentation::Mini => crate::window_resize::ViewType::MiniMainWindow,
            },
            AppView::ClipboardHistoryView { .. }
            | AppView::ThemeChooserView { .. }
            | AppView::SdkReferenceView { .. }
            | AppView::ScriptTemplateCatalogView { .. }
            | AppView::AcpHistoryView { .. }
            | AppView::BrowserHistoryView { .. }
            | AppView::DictationHistoryView { .. }
            | AppView::NotesBrowseView { .. } => crate::window_resize::ViewType::ExpandedMainWindow,
            _ => crate::window_resize::ViewType::ScriptList,
        };
        let window_width =
            crate::window_resize::width_for_view(layout_view_type).unwrap_or(750.0_f32);
        let window_height = f32::from(crate::window_resize::height_for_view(layout_view_type, 0));

        // Determine current prompt type
        let prompt_type = match &self.current_view {
            AppView::ScriptList => "mainMenu",
            AppView::About { .. } => "about",
            AppView::ArgPrompt { .. } => "arg",
            AppView::DivPrompt { .. } => "div",
            AppView::FormPrompt { .. } => "form",
            AppView::TermPrompt { .. } => "term",
            AppView::EditorPrompt { .. } => "editor",
            AppView::SelectPrompt { .. } => "select",
            AppView::PathPrompt { .. } => "path",
            AppView::EnvPrompt { .. } => "env",
            AppView::DropPrompt { .. } => "drop",
            AppView::TemplatePrompt { .. } => "template",
            AppView::HotkeyPrompt { .. } => "hotkey",
            AppView::ChatPrompt { .. } => "chat",
            AppView::MiniPrompt { .. } => "mini",
            AppView::MicroPrompt { .. } => "micro",
            AppView::ClipboardHistoryView { .. } => "clipboardHistory",
            AppView::AppLauncherView { .. } => "appLauncher",
            AppView::WindowSwitcherView { .. } => "windowSwitcher",
            AppView::BrowserTabsView { .. } => "browserTabs",
            AppView::DesignGalleryView { .. } => "designGallery",
            #[cfg(feature = "storybook")]
            AppView::DesignExplorerView { .. } => "designExplorer",
            AppView::ScratchPadView { .. } => "scratchPad",
            AppView::QuickTerminalView { .. } => "quickTerminal",
            AppView::FileSearchView { .. } => "fileSearch",
            AppView::ThemeChooserView { .. } => "themeChooser",
            AppView::EmojiPickerView { .. } => "emojiPicker",
            AppView::ActionsDialog => "actionsDialog",
            AppView::WebcamView { .. } => "webcam",
            AppView::CreationFeedback { .. } => "creationFeedback",
            AppView::NamingPrompt { .. } => "namingPrompt",
            AppView::BrowseKitsView { .. } => "browseKits",
            AppView::InstalledKitsView { .. } => "installedKits",
            AppView::ProcessManagerView { .. } => "processManager",
            AppView::CurrentAppCommandsView { .. } => "currentAppCommands",
            AppView::SearchAiPresetsView { .. } => "searchAiPresets",
            AppView::CreateAiPresetView { .. } => "createAiPreset",
            AppView::SettingsView { .. } => "settings",
            AppView::FavoritesBrowseView { .. } => "favoritesBrowse",
            AppView::AcpHistoryView { .. } => "acpHistory",
            AppView::BrowserHistoryView { .. } => "browserHistory",
            AppView::DictationHistoryView { .. } => "dictationHistory",
            AppView::NotesBrowseView { .. } => "notesBrowse",
            AppView::AcpChatView { .. } => "acpChat",
            AppView::ScriptIssuesView { .. } => "scriptIssues",
            AppView::SdkReferenceView { .. } => "sdkReference",
            AppView::ScriptTemplateCatalogView { .. } => "scriptTemplateCatalog",
            AppView::ConfirmPrompt { .. } => "confirmPrompt",
        };

        let mut components = Vec::new();

        // Layout constants (same as build_component_bounds)
        const HEADER_PADDING_Y: f32 = 8.0;
        const HEADER_PADDING_X: f32 = 16.0;
        const BUTTON_HEIGHT: f32 = 28.0;
        const DIVIDER_HEIGHT: f32 = 1.0;
        let header_height = HEADER_PADDING_Y * 2.0 + BUTTON_HEIGHT + DIVIDER_HEIGHT; // 45px
        let list_width = window_width * 0.5;
        let content_top = header_height;
        let content_height = window_height - header_height;

        // Root container
        components.push(
            LayoutComponentInfo::new("Window", LayoutComponentType::Container)
                .with_bounds(0.0, 0.0, window_width, window_height)
                .with_flex_column()
                .with_depth(0)
                .with_explanation("Root window container. Uses flex-column layout."),
        );

        // Header
        components.push(
            LayoutComponentInfo::new("Header", LayoutComponentType::Header)
                .with_bounds(0.0, 0.0, window_width, header_height)
                .with_padding(HEADER_PADDING_Y, HEADER_PADDING_X, HEADER_PADDING_Y, HEADER_PADDING_X)
                .with_flex_row()
                .with_depth(1)
                .with_parent("Window")
                .with_explanation(format!(
                    "Height = padding({}) + content({}) + padding({}) + divider({}) = {}px. Uses flex-row with items-center.",
                    HEADER_PADDING_Y, BUTTON_HEIGHT, HEADER_PADDING_Y, DIVIDER_HEIGHT, header_height
                )),
        );

        // Search input in header
        const INPUT_HEIGHT: f32 = 22.0;
        let input_y = HEADER_PADDING_Y + (BUTTON_HEIGHT - INPUT_HEIGHT) / 2.0;
        let buttons_area_width = 200.0;
        let input_width = window_width - HEADER_PADDING_X - buttons_area_width;

        components.push(
            LayoutComponentInfo::new("SearchInput", LayoutComponentType::Input)
                .with_bounds(HEADER_PADDING_X, input_y, input_width, INPUT_HEIGHT)
                .with_flex_grow(1.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "flex-grow:1 fills remaining space. Width = window({}) - padding({}) - buttons_area({}) = {}px. Vertically centered in header.",
                    window_width, HEADER_PADDING_X, buttons_area_width, input_width
                )),
        );

        // Content area
        components.push(
            LayoutComponentInfo::new("ContentArea", LayoutComponentType::Container)
                .with_bounds(0.0, content_top, window_width, content_height)
                .with_flex_row()
                .with_flex_grow(1.0)
                .with_depth(1)
                .with_parent("Window")
                .with_explanation(
                    "flex-grow:1 fills remaining height after header. Uses flex-row to create side-by-side panels.".to_string()
                ),
        );

        if matches!(
            self.current_view,
            AppView::DivPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::TermPrompt { .. }
                | AppView::QuickTerminalView { .. }
        ) {
            let (component_name, explanation) = match &self.current_view {
                AppView::DivPrompt { .. } => (
                    "DivContent",
                    "DivPrompt fills the content area with scrollable HTML content and footer ownership routed through the shared main-window footer slot.",
                ),
                AppView::EditorPrompt { .. } => (
                    "EditorContent",
                    "EditorPrompt fills the content area; footer ownership is routed through the shared main-window footer slot.",
                ),
                AppView::QuickTerminalView { .. } => (
                    "TerminalContent",
                    "QuickTerminalView fills the compact content area and reserves native-footer space through the shared main-window footer slot.",
                ),
                _ => (
                    "TerminalContent",
                    "TermPrompt fills the content area and owns the SDK terminal hint strip through the shared main-window footer slot.",
                ),
            };

            components.push(
                LayoutComponentInfo::new(component_name, LayoutComponentType::Prompt)
                    .with_bounds(0.0, content_top, window_width, content_height)
                    .with_flex_column()
                    .with_flex_grow(1.0)
                    .with_depth(2)
                    .with_parent("ContentArea")
                    .with_explanation(explanation.to_string()),
            );

            return LayoutInfo {
                window_width,
                window_height,
                prompt_type: prompt_type.to_string(),
                components,
                handler_form: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
        }

        if matches!(
            self.current_view,
            AppView::SelectPrompt { .. } | AppView::DropPrompt { .. }
        ) {
            let (component_name, component_type, explanation) = match &self.current_view {
                AppView::SelectPrompt { .. } => (
                    "SelectChoices",
                    LayoutComponentType::List,
                    "SelectPrompt fills the content area with a keyboard-owned minimal list and footer-aware hint strip.",
                ),
                _ => (
                    "DropContent",
                    LayoutComponentType::Prompt,
                    "DropPrompt fills the content area with a focused drop target and prompt-owned keyboard handling.",
                ),
            };

            components.push(
                LayoutComponentInfo::new(component_name, component_type)
                    .with_bounds(0.0, content_top, window_width, content_height)
                    .with_flex_column()
                    .with_flex_grow(1.0)
                    .with_depth(2)
                    .with_parent("ContentArea")
                    .with_explanation(explanation.to_string()),
            );

            return LayoutInfo {
                window_width,
                window_height,
                prompt_type: prompt_type.to_string(),
                components,
                handler_form: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
        }

        // Script list (left panel) - 50% width
        components.push(
            LayoutComponentInfo::new("ScriptList", LayoutComponentType::List)
                .with_bounds(0.0, content_top, list_width, content_height)
                .with_flex_column()
                .with_depth(2)
                .with_parent("ContentArea")
                .with_explanation(format!(
                    "Width = 50% of window = {}px. Uses uniform_list for virtualized scrolling with {}px item height.",
                    list_width, LIST_ITEM_HEIGHT
                )),
        );

        // Preview panel (right panel) - remaining 50%
        let preview_width = window_width - list_width;
        components.push(
            LayoutComponentInfo::new("PreviewPanel", LayoutComponentType::Panel)
                .with_bounds(list_width, content_top, preview_width, content_height)
                .with_padding(16.0, 16.0, 16.0, 16.0)
                .with_flex_column()
                .with_depth(2)
                .with_parent("ContentArea")
                .with_explanation(format!(
                    "Width = remaining 50% = {}px. Has 16px padding on all sides.",
                    preview_width
                )),
        );

        // List items (sample of first few visible)
        let visible_items = ((content_height / LIST_ITEM_HEIGHT) as usize).min(5);
        for i in 0..visible_items {
            let item_top = content_top + (i as f32 * LIST_ITEM_HEIGHT);
            components.push(
                LayoutComponentInfo::new(format!("ListItem[{}]", i), LayoutComponentType::ListItem)
                    .with_bounds(0.0, item_top, list_width, LIST_ITEM_HEIGHT)
                    .with_padding(12.0, 16.0, 12.0, 16.0)
                    .with_gap(8.0)
                    .with_flex_row()
                    .with_depth(3)
                    .with_parent("ScriptList")
                    .with_explanation(format!(
                        "Fixed height = {}px. Uses flex-row with gap:8px for icon + text layout. Padding: 12px vertical, 16px horizontal.",
                        LIST_ITEM_HEIGHT
                    )),
            );
        }

        // Button group in header
        let button_y = HEADER_PADDING_Y;
        let button_height = BUTTON_HEIGHT;

        // Logo button (rightmost)
        let logo_x = window_width - HEADER_PADDING_X - 20.0;
        components.push(
            LayoutComponentInfo::new("LogoButton", LayoutComponentType::Button)
                .with_bounds(logo_x, button_y, 20.0, button_height)
                .with_padding(4.0, 4.0, 4.0, 4.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation("Fixed 20px width. Positioned at right edge with 16px margin."),
        );

        // Actions button
        let actions_width = 85.0;
        let actions_x = logo_x - 24.0 - actions_width;
        components.push(
            LayoutComponentInfo::new("ActionsButton", LayoutComponentType::Button)
                .with_bounds(actions_x, button_y, actions_width, button_height)
                .with_padding(4.0, 8.0, 4.0, 8.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "Width = {}px. Positioned left of logo with 24px spacing (includes divider).",
                    actions_width
                )),
        );

        // Run button
        let run_width = 55.0;
        let run_x = actions_x - 24.0 - run_width;
        components.push(
            LayoutComponentInfo::new("RunButton", LayoutComponentType::Button)
                .with_bounds(run_x, button_y, run_width, button_height)
                .with_padding(4.0, 8.0, 4.0, 8.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "Width = {}px. Positioned left of Actions with 24px spacing.",
                    run_width
                )),
        );

        LayoutInfo {
            window_width,
            window_height,
            prompt_type: prompt_type.to_string(),
            components,
            handler_form: self.build_handler_form_layout_info(
                content_top,
                content_height,
                window_width,
            ),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    fn build_handler_form_layout_info(
        &self,
        content_top: f32,
        content_height: f32,
        window_width: f32,
    ) -> Option<serde_json::Value> {
        if !matches!(self.current_view, AppView::ScriptList)
            || !self.menu_syntax_capture_form_owns_input()
        {
            return None;
        }
        let form = self
            .menu_syntax_main_hint_snapshot(&self.filter_text, false)?
            .form?;

        const OUTER_PADDING_X: f32 = 18.0;
        const OUTER_PADDING_TOP: f32 = 12.0;
        const TITLE_BLOCK_HEIGHT: f32 = 72.0;
        const FIELD_HEIGHT: f32 = 68.0;
        const FIELD_GAP: f32 = 8.0;
        const SIDEBAR_WIDTH: f32 = 180.0;
        const SIDEBAR_GAP: f32 = 8.0;

        let offset = self.menu_syntax_main_hint_scroll_handle.offset();
        let max_offset = self.menu_syntax_main_hint_scroll_handle.max_offset();
        let scroll_offset_y = (-offset.y.as_f32()).max(0.0);
        let viewport = serde_json::json!({
            "x": OUTER_PADDING_X,
            "y": content_top + OUTER_PADDING_TOP,
            "width": (window_width - (OUTER_PADDING_X * 2.0)).max(0.0),
            "height": (content_height - OUTER_PADDING_TOP).max(0.0),
        });
        let viewport_top = content_top + OUTER_PADDING_TOP;
        let viewport_bottom = content_top + content_height;
        let form_top = content_top + OUTER_PADDING_TOP + TITLE_BLOCK_HEIGHT;
        let content_width = (window_width - (OUTER_PADDING_X * 2.0)).max(0.0);
        let sidebar_field_id = self.menu_syntax_form_suggestion_field_id.as_deref();
        let field_width = if sidebar_field_id.is_some() {
            (content_width - SIDEBAR_GAP - SIDEBAR_WIDTH).max(0.0)
        } else {
            content_width
        };

        let mut focused_visibility = serde_json::Value::Null;
        let fields = form
            .fields
            .iter()
            .enumerate()
            .map(|(index, field)| {
                let y = form_top + (index as f32 * (FIELD_HEIGHT + FIELD_GAP)) - scroll_offset_y;
                let bottom = y + FIELD_HEIGHT;
                let visible_top = y.max(viewport_top);
                let visible_bottom = bottom.min(viewport_bottom);
                let visible_height = (visible_bottom - visible_top).max(0.0).min(FIELD_HEIGHT);
                let visible_ratio = if FIELD_HEIGHT > 0.0 {
                    visible_height / FIELD_HEIGHT
                } else {
                    0.0
                };
                let fully_visible = visible_ratio >= 0.999;
                let semantic_id = format!("handler-form:{}:{}", form.target, field.id);
                let bounds = serde_json::json!({
                    "x": OUTER_PADDING_X,
                    "y": y,
                    "width": field_width,
                    "height": FIELD_HEIGHT,
                });
                let field_info = serde_json::json!({
                    "semanticId": semantic_id,
                    "fieldId": field.id,
                    "index": index,
                    "focused": field.focused && self.menu_syntax_form_input_active,
                    "fullyVisible": fully_visible,
                    "visibleRatio": visible_ratio,
                    "handlerFormFieldBounds": bounds,
                });
                if field.focused && self.menu_syntax_form_input_active {
                    focused_visibility = serde_json::json!({
                        "semanticId": format!("handler-form:{}:{}", form.target, field.id),
                        "index": index,
                        "fullyVisible": fully_visible,
                        "visibleRatio": visible_ratio,
                        "bounds": bounds,
                    });
                }
                field_info
            })
            .collect::<Vec<_>>();

        let popup = sidebar_field_id.and_then(|field_id| {
            form.fields
                .iter()
                .any(|field| field.id == field_id)
                .then(|| {
                    let field_count = form.fields.len() as f32;
                    let sidebar_height = (field_count * FIELD_HEIGHT
                        + (field_count - 1.0).max(0.0) * FIELD_GAP)
                        .min((content_height - OUTER_PADDING_TOP - TITLE_BLOCK_HEIGHT).max(0.0));
                    serde_json::json!({
                        "ownerFieldId": field_id,
                        "role": "listbox",
                        "surface": "handlerFormAutocompleteSidebar",
                        "bounds": {
                            "x": OUTER_PADDING_X + field_width + SIDEBAR_GAP,
                            "y": form_top - scroll_offset_y,
                            "width": SIDEBAR_WIDTH,
                            "height": sidebar_height,
                        }
                    })
                })
        });

        Some(serde_json::json!({
            "target": form.target,
            "source": "menuSyntaxMainHint.form",
            "scrollContainerId": "menu-syntax-main-hint-scroll",
            "scrollOffsetY": scroll_offset_y,
            "maxScrollOffsetY": max_offset.y.as_f32().max(0.0),
            "viewport": viewport,
            "focusedSemanticId": if self.menu_syntax_form_input_active {
                form.fields
                    .iter()
                    .find(|field| field.focused)
                    .map(|field| format!("handler-form:{}:{}", form.target, field.id))
            } else {
                None
            },
            "focusedIndex": if self.menu_syntax_form_input_active {
                Some(form.focused_index)
            } else {
                None
            },
            "handlerFormFocusedVisibility": focused_visibility,
            "fields": fields,
            "popup": popup,
        }))
    }
}
