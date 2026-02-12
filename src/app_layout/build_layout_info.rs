impl ScriptListApp {
    pub fn build_layout_info(&self, _cx: &mut gpui::Context<Self>) -> protocol::LayoutInfo {
        use protocol::{LayoutComponentInfo, LayoutComponentType, LayoutInfo};

        // TODO: Get actual window size once we have access to window in this context
        // For now, use default values
        let window_width = 750.0_f32;
        let window_height = f32::from(crate::window_resize::initial_window_height());

        // Determine current prompt type
        let prompt_type = match &self.current_view {
            AppView::ScriptList => "mainMenu",
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
            AppView::ChatPrompt { .. } => "chat",
            AppView::ClipboardHistoryView { .. } => "clipboardHistory",
            AppView::AppLauncherView { .. } => "appLauncher",
            AppView::WindowSwitcherView { .. } => "windowSwitcher",
            AppView::DesignGalleryView { .. } => "designGallery",
            AppView::ScratchPadView { .. } => "scratchPad",
            AppView::QuickTerminalView { .. } => "quickTerminal",
            AppView::FileSearchView { .. } => "fileSearch",
            AppView::ThemeChooserView { .. } => "themeChooser",
            AppView::EmojiPickerView { .. } => "emojiPicker",
            AppView::ActionsDialog => "actionsDialog",
            AppView::WebcamView { .. } => "webcam",
            AppView::CreationFeedback { .. } => "creationFeedback",
            AppView::NamingPrompt { .. } => "namingPrompt",
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

        // Script list (left panel) - 50% width
        components.push(
            LayoutComponentInfo::new("ScriptList", LayoutComponentType::List)
                .with_bounds(0.0, content_top, list_width, content_height)
                .with_flex_column()
                .with_depth(2)
                .with_parent("ContentArea")
                .with_explanation(format!(
                    "Width = 50% of window = {}px. Uses uniform_list for virtualized scrolling with 48px item height.",
                    list_width
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
        const LIST_ITEM_HEIGHT: f32 = 48.0;
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
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}
