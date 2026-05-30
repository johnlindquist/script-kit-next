impl ScriptListApp {
    pub fn build_layout_info(&self, _cx: &mut gpui::Context<Self>) -> protocol::LayoutInfo {
        use crate::list_item::LIST_ITEM_HEIGHT;
        use protocol::{LayoutComponentInfo, LayoutComponentType, LayoutInfo};

        // Keep automation layout receipts aligned with the same sizing contract
        // used by real window resize paths.
        let layout_view_type = match &self.current_view {
            AppView::ScriptList => match self.main_window_mode {
                MainWindowMode::Full => crate::window_resize::ViewType::ScriptList,
                MainWindowMode::Mini => crate::window_resize::ViewType::MainWindow,
            },
            AppView::FileSearchView { presentation, .. } => match presentation {
                FileSearchPresentation::Full => crate::window_resize::ViewType::MainWindow,
                FileSearchPresentation::Mini => crate::window_resize::ViewType::MainWindow,
            },
            AppView::About { .. } => crate::window_resize::ViewType::ScriptList,
            AppView::CreationFeedback { .. } => crate::window_resize::ViewType::DivPrompt,
            AppView::EnvPrompt { .. } => crate::window_resize::ViewType::DivPrompt,
            AppView::WebcamView { .. } => crate::window_resize::ViewType::DivPrompt,
            AppView::NamingPrompt { .. } | AppView::CreateAiPresetView { .. } => {
                crate::window_resize::ViewType::ArgPromptNoChoices
            }
            AppView::ScratchPadView { .. } => crate::window_resize::ViewType::EditorPrompt,
            AppView::QuickTerminalView { .. } => crate::window_resize::ViewType::TermPrompt,
            AppView::ClipboardHistoryView { .. }
            | AppView::ThemeChooserView { .. }
            | AppView::SdkReferenceView { .. }
            | AppView::ScriptTemplateCatalogView { .. }
            | AppView::AcpHistoryView { .. }
            | AppView::BrowserHistoryView { .. }
            | AppView::DictationHistoryView { .. }
            | AppView::NotesBrowseView { .. } => crate::window_resize::ViewType::MainWindow,
            AppView::AppLauncherView { .. }
            | AppView::WindowSwitcherView { .. }
            | AppView::BrowserTabsView { .. }
            | AppView::DesignGalleryView { .. }
            | AppView::FooterGalleryView { .. }
            | AppView::EmojiPickerView { .. }
            | AppView::BrowseKitsView { .. }
            | AppView::InstalledKitsView { .. }
            | AppView::ProcessManagerView { .. }
            | AppView::CurrentAppCommandsView { .. }
            | AppView::SearchAiPresetsView { .. }
            | AppView::SettingsView { .. }
            | AppView::FavoritesBrowseView { .. } => crate::window_resize::ViewType::MainWindow,
            _ => crate::window_resize::ViewType::ScriptList,
        };
        let window_width =
            crate::window_resize::width_for_view(layout_view_type).unwrap_or(750.0_f32);
        let window_height = f32::from(crate::window_resize::height_for_view(layout_view_type, 0));
        let uses_split_preview = matches!(
            layout_view_type,
            crate::window_resize::ViewType::MainWindow | crate::window_resize::ViewType::ScriptList
        );

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
            AppView::FooterGalleryView { .. } => "footerGallery",
            AppView::NonListStatesView { .. } => "nonListStates",
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
        use crate::ui::chrome as chrome_tokens;
        const HEADER_PADDING_Y: f32 = 8.0;
        const HEADER_PADDING_X: f32 = 16.0;
        const BUTTON_HEIGHT: f32 = 28.0;
        const DIVIDER_HEIGHT: f32 = 1.0;
        let header_height = HEADER_PADDING_Y * 2.0 + BUTTON_HEIGHT + DIVIDER_HEIGHT; // 45px
        let list_width = if uses_split_preview {
            window_width * 0.5
        } else {
            window_width
        };
        let content_top = header_height;
        let content_height = window_height - header_height;

        // Root container
        components.push(
            LayoutComponentInfo::new("Window", LayoutComponentType::Container)
                .with_bounds(0.0, 0.0, window_width, window_height)
                .with_visual_style(
                    chrome_tokens::CHROME_LAYER_WINDOW_BACKDROP,
                    chrome_tokens::MATERIAL_NATIVE_WINDOW_BACKDROP,
                    Some(chrome_tokens::LIQUID_GLASS_WINDOW_RADIUS_PX),
                )
                .with_visual_token("window.backdrop")
                .with_flex_column()
                .with_depth(0)
                .with_explanation(
                    "Root native window backdrop. Not a content layer or Liquid Glass content surface.",
                ),
        );

        if matches!(self.current_view, AppView::About { .. }) {
            const ABOUT_HEADER_HEIGHT: f32 = 52.0;
            const ABOUT_SCROLL_PADDING_X: f32 = 32.0;
            const ABOUT_SCROLL_PADDING_Y: f32 = 14.0;
            const ABOUT_STACK_WIDTH: f32 = 560.0;
            const ABOUT_LOGO_SIZE: f32 = 56.0;
            const ABOUT_LOGO_ICON_SIZE: f32 = 36.0;
            const ABOUT_TITLE_HEIGHT: f32 = 34.0;
            const ABOUT_BADGE_HEIGHT: f32 = 20.0;
            const ABOUT_TAGLINE_HEIGHT: f32 = 19.0;
            const ABOUT_CREATOR_HEIGHT: f32 = 32.0;
            const ABOUT_BUTTON_HEIGHT: f32 = 34.0;
            const ABOUT_BUTTON_WIDTH: f32 = 128.0;
            const ABOUT_BUTTON_GAP: f32 = 8.0;
            const ABOUT_CARD_HEIGHT: f32 = 60.0;
            const ABOUT_ACK_HEIGHT: f32 = 34.0;
            const ABOUT_ITEM_GAP: f32 = 10.0;

            let content_height = window_height - ABOUT_HEADER_HEIGHT;
            let stack_width = ABOUT_STACK_WIDTH.min(window_width - ABOUT_SCROLL_PADDING_X * 2.0);
            let stack_x = (window_width - stack_width) / 2.0;
            let mut cursor_y = ABOUT_HEADER_HEIGHT + ABOUT_SCROLL_PADDING_Y;

            components.push(
                LayoutComponentInfo::new("AboutHeader", LayoutComponentType::Header)
                    .with_bounds(0.0, 0.0, window_width, ABOUT_HEADER_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                    )
                    .with_visual_token("about.header")
                    .with_padding(0.0, 16.0, 0.0, 16.0)
                    .with_flex_row()
                    .with_depth(1)
                    .with_parent("Window")
                    .with_explanation("About header is 52px tall and owns only title and close control."),
            );
            components.push(
                LayoutComponentInfo::new("AboutCloseButton", LayoutComponentType::Button)
                    .with_bounds(window_width - 44.0, 12.0, 28.0, 28.0)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_CONTROL_RADIUS_PX),
                    )
                    .with_visual_token("about.closeButton")
                    .with_hit_bounds(window_width - 44.0, 12.0, 28.0, 28.0)
                    .with_depth(2)
                    .with_parent("AboutHeader")
                    .with_explanation("28x28 minimum macOS hit target; rounded as a circular icon control."),
            );
            components.push(
                LayoutComponentInfo::new("AboutScrollContainer", LayoutComponentType::Container)
                    .with_bounds(0.0, ABOUT_HEADER_HEIGHT, window_width, content_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("about.scrollContainer")
                    .with_padding(
                        ABOUT_SCROLL_PADDING_Y,
                        ABOUT_SCROLL_PADDING_X,
                        ABOUT_SCROLL_PADDING_Y,
                        ABOUT_SCROLL_PADDING_X,
                    )
                    .with_flex_column()
                    .with_depth(1)
                    .with_parent("Window")
                    .with_explanation("Scrollable content region below the About header."),
            );
            components.push(
                LayoutComponentInfo::new("AboutContentStack", LayoutComponentType::Container)
                    .with_bounds(stack_x, cursor_y, stack_width, content_height - ABOUT_SCROLL_PADDING_Y * 2.0)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("about.contentStack")
                    .with_gap(ABOUT_ITEM_GAP)
                    .with_flex_column()
                    .with_depth(2)
                    .with_parent("AboutScrollContainer")
                    .with_explanation("Centered 560px max-width content stack with 10px item rhythm."),
            );

            let centered_x = |width: f32| stack_x + (stack_width - width) / 2.0;
            components.push(
                LayoutComponentInfo::new("AboutLogoTile", LayoutComponentType::Container)
                    .with_bounds(centered_x(ABOUT_LOGO_SIZE), cursor_y, ABOUT_LOGO_SIZE, ABOUT_LOGO_SIZE)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                    )
                    .with_visual_token("about.logoTile")
                    .with_depth(3)
                    .with_parent("AboutContentStack")
                    .with_explanation(format!(
                        "56px logo tile with {}px icon and compact Liquid Glass radius.",
                        ABOUT_LOGO_ICON_SIZE
                    )),
            );
            cursor_y += ABOUT_LOGO_SIZE + ABOUT_ITEM_GAP;

            components.push(
                LayoutComponentInfo::new("AboutTitle", LayoutComponentType::Other)
                    .with_bounds(stack_x, cursor_y, stack_width, ABOUT_TITLE_HEIGHT + 6.0 + ABOUT_BADGE_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("about.titleVersion")
                    .with_depth(3)
                    .with_parent("AboutContentStack")
                    .with_explanation("Product title plus version badge block."),
            );
            cursor_y += ABOUT_TITLE_HEIGHT + 6.0 + ABOUT_BADGE_HEIGHT + ABOUT_ITEM_GAP;

            components.push(
                LayoutComponentInfo::new("AboutTagline", LayoutComponentType::Other)
                    .with_bounds(centered_x(440.0), cursor_y, 440.0_f32.min(stack_width), ABOUT_TAGLINE_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("about.tagline")
                    .with_depth(3)
                    .with_parent("AboutContentStack")
                    .with_explanation("Centered tagline text with bounded width."),
            );
            cursor_y += ABOUT_TAGLINE_HEIGHT + ABOUT_ITEM_GAP;

            components.push(
                LayoutComponentInfo::new("AboutCreatorRow", LayoutComponentType::Other)
                    .with_bounds(centered_x(260.0), cursor_y, 260.0_f32.min(stack_width), ABOUT_CREATOR_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("about.creatorRow")
                    .with_depth(3)
                    .with_parent("AboutContentStack")
                    .with_explanation("Creator avatar and label row."),
            );
            cursor_y += ABOUT_CREATOR_HEIGHT + ABOUT_ITEM_GAP;

            let quick_actions_width = ABOUT_BUTTON_WIDTH * 3.0 + ABOUT_BUTTON_GAP * 2.0;
            components.push(
                LayoutComponentInfo::new("AboutQuickActions", LayoutComponentType::Container)
                    .with_bounds(centered_x(quick_actions_width), cursor_y, quick_actions_width.min(stack_width), ABOUT_BUTTON_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("about.quickActions")
                    .with_gap(ABOUT_BUTTON_GAP)
                    .with_flex_row()
                    .with_depth(3)
                    .with_parent("AboutContentStack")
                    .with_explanation("Three compact action controls with 8px gap."),
            );
            for (index, name) in ["AboutOpenGithub", "AboutOpenDiscord", "AboutFollowX"]
                .into_iter()
                .enumerate()
            {
                let x = centered_x(quick_actions_width) + index as f32 * (ABOUT_BUTTON_WIDTH + ABOUT_BUTTON_GAP);
                components.push(
                    LayoutComponentInfo::new(name, LayoutComponentType::Button)
                        .with_bounds(x, cursor_y, ABOUT_BUTTON_WIDTH, ABOUT_BUTTON_HEIGHT)
                        .with_visual_style(
                            chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                            chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                            Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                        )
                        .with_visual_token("about.actionButton")
                        .with_hit_bounds(x, cursor_y, ABOUT_BUTTON_WIDTH, ABOUT_BUTTON_HEIGHT)
                        .with_depth(4)
                        .with_parent("AboutQuickActions")
                        .with_explanation("34px tall compact text button; hit target exceeds 28px."),
                );
            }
            cursor_y += ABOUT_BUTTON_HEIGHT + ABOUT_ITEM_GAP;

            components.push(
                LayoutComponentInfo::new("AboutUpdateCard", LayoutComponentType::Panel)
                    .with_bounds(stack_x, cursor_y, 500.0_f32.min(stack_width), ABOUT_CARD_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                    )
                    .with_visual_token("about.updateCard")
                    .with_padding(14.0, 16.0, 14.0, 16.0)
                    .with_flex_row()
                    .with_depth(3)
                    .with_parent("AboutContentStack")
                    .with_explanation("Update card uses compact 10px radius and content-layer material."),
            );
            components.push(
                LayoutComponentInfo::new("AboutUpdateButton", LayoutComponentType::Button)
                    .with_bounds(stack_x + 500.0_f32.min(stack_width) - 16.0 - 142.0, cursor_y + 13.0, 142.0, ABOUT_BUTTON_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                    )
                    .with_visual_token("about.actionButton")
                    .with_hit_bounds(stack_x + 500.0_f32.min(stack_width) - 16.0 - 142.0, cursor_y + 13.0, 142.0, ABOUT_BUTTON_HEIGHT)
                    .with_depth(4)
                    .with_parent("AboutUpdateCard")
                    .with_explanation("Update action is 34px high with 142px minimum width."),
            );
            cursor_y += ABOUT_CARD_HEIGHT + ABOUT_ITEM_GAP;

            components.push(
                LayoutComponentInfo::new("AboutAcknowledgementsCard", LayoutComponentType::Panel)
                    .with_bounds(stack_x, cursor_y, 500.0_f32.min(stack_width), ABOUT_ACK_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                    )
                    .with_visual_token("about.acknowledgementsCard")
                    .with_depth(3)
                    .with_parent("AboutContentStack")
                    .with_explanation("Collapsed acknowledgements panel with compact 10px radius."),
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

        if matches!(self.current_view, AppView::CreationFeedback { .. }) {
            const FEEDBACK_PADDING_X: f32 = 24.0;
            const FEEDBACK_PADDING_Y: f32 = 18.0;
            const FEEDBACK_STACK_GAP: f32 = 16.0;
            const FEEDBACK_INTRO_HEIGHT: f32 = 58.0;
            const FEEDBACK_SECTION_LABEL_HEIGHT: f32 = 17.0;
            const FEEDBACK_SECTION_GAP: f32 = 8.0;
            const FEEDBACK_PATH_HEIGHT: f32 = 42.0;
            const FEEDBACK_BUTTON_HEIGHT: f32 = 28.0;
            const FEEDBACK_BUTTON_GAP: f32 = 8.0;
            const FEEDBACK_REVEAL_WIDTH: f32 = 128.0;
            const FEEDBACK_COPY_WIDTH: f32 = 92.0;
            const FEEDBACK_OPEN_WIDTH: f32 = 58.0;

            let panel_x = FEEDBACK_PADDING_X;
            let panel_y = FEEDBACK_PADDING_Y;
            let panel_width = window_width - FEEDBACK_PADDING_X * 2.0;
            let panel_height = window_height - FEEDBACK_PADDING_Y * 2.0;
            let mut cursor_y = panel_y;

            components.push(
                LayoutComponentInfo::new("CreationFeedbackPanel", LayoutComponentType::Panel)
                    .with_bounds(panel_x, panel_y, panel_width, panel_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                    )
                    .with_visual_token("feedback.creationPanel")
                    .with_padding(
                        FEEDBACK_PADDING_Y,
                        FEEDBACK_PADDING_X,
                        FEEDBACK_PADDING_Y,
                        FEEDBACK_PADDING_X,
                    )
                    .with_gap(FEEDBACK_STACK_GAP)
                    .with_flex_column()
                    .with_depth(1)
                    .with_parent("Window")
                    .with_explanation("CreationFeedback fills the standard-height window with a padded Liquid Glass content panel."),
            );

            components.push(
                LayoutComponentInfo::new("CreationFeedbackIntro", LayoutComponentType::Header)
                    .with_bounds(panel_x, cursor_y, panel_width, FEEDBACK_INTRO_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("feedback.intro")
                    .with_depth(2)
                    .with_parent("CreationFeedbackPanel")
                    .with_explanation("Title and supporting copy for the created-file confirmation."),
            );
            cursor_y += FEEDBACK_INTRO_HEIGHT + FEEDBACK_STACK_GAP;

            let section_height =
                FEEDBACK_SECTION_LABEL_HEIGHT + FEEDBACK_SECTION_GAP + FEEDBACK_PATH_HEIGHT;
            components.push(
                LayoutComponentInfo::new("CreationFeedbackPathSection", LayoutComponentType::Container)
                    .with_bounds(panel_x, cursor_y, panel_width, section_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("feedback.pathSection")
                    .with_gap(FEEDBACK_SECTION_GAP)
                    .with_flex_column()
                    .with_depth(2)
                    .with_parent("CreationFeedbackPanel")
                    .with_explanation("Path section owns the read-only path surface and label spacing."),
            );
            components.push(
                LayoutComponentInfo::new("CreationFeedbackPathSurface", LayoutComponentType::Input)
                    .with_bounds(
                        panel_x,
                        cursor_y + FEEDBACK_SECTION_LABEL_HEIGHT + FEEDBACK_SECTION_GAP,
                        panel_width,
                        FEEDBACK_PATH_HEIGHT,
                    )
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_CONTROL_RADIUS_PX),
                    )
                    .with_visual_token("feedback.pathSurface")
                    .with_padding(10.0, 12.0, 10.0, 12.0)
                    .with_depth(3)
                    .with_parent("CreationFeedbackPathSection")
                    .with_explanation("Read-only path surface uses a 14px control radius and 42px height for long-path scrolling."),
            );
            cursor_y += section_height + FEEDBACK_STACK_GAP;

            let button_widths = [
                ("CreationFeedbackRevealButton", FEEDBACK_REVEAL_WIDTH),
                ("CreationFeedbackCopyButton", FEEDBACK_COPY_WIDTH),
                ("CreationFeedbackOpenButton", FEEDBACK_OPEN_WIDTH),
            ];
            let action_row_width = FEEDBACK_REVEAL_WIDTH
                + FEEDBACK_COPY_WIDTH
                + FEEDBACK_OPEN_WIDTH
                + FEEDBACK_BUTTON_GAP * 2.0;
            components.push(
                LayoutComponentInfo::new("CreationFeedbackActions", LayoutComponentType::Container)
                    .with_bounds(panel_x, cursor_y, action_row_width, FEEDBACK_BUTTON_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("feedback.actions")
                    .with_gap(FEEDBACK_BUTTON_GAP)
                    .with_flex_row()
                    .with_depth(2)
                    .with_parent("CreationFeedbackPanel")
                    .with_explanation("Action row keeps three 28px-tall controls on an 8px rhythm."),
            );

            let mut button_x = panel_x;
            for (name, width) in button_widths {
                components.push(
                    LayoutComponentInfo::new(name, LayoutComponentType::Button)
                        .with_bounds(button_x, cursor_y, width, FEEDBACK_BUTTON_HEIGHT)
                        .with_visual_style(
                            chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                            chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                            Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                        )
                        .with_visual_token("feedback.actionButton")
                        .with_hit_bounds(button_x, cursor_y, width, FEEDBACK_BUTTON_HEIGHT)
                        .with_depth(3)
                        .with_parent("CreationFeedbackActions")
                        .with_explanation("Compact ghost button uses the shared 10px Liquid Glass button radius and 28px minimum hit height."),
                );
                button_x += width + FEEDBACK_BUTTON_GAP;
            }

            return LayoutInfo {
                window_width,
                window_height,
                prompt_type: prompt_type.to_string(),
                components,
                handler_form: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
        }

        if matches!(self.current_view, AppView::ConfirmPrompt { .. }) {
            const CONFIRM_CONTENT_PADDING_X: f32 = 24.0;
            const CONFIRM_CONTENT_PADDING_Y: f32 = 24.0;
            const CONFIRM_FOOTER_HEIGHT: f32 = 38.0;
            const CONFIRM_STACK_WIDTH: f32 = 560.0;
            const CONFIRM_TITLE_HEIGHT: f32 = 28.0;
            const CONFIRM_BODY_HEIGHT: f32 = 40.0;
            const CONFIRM_STACK_GAP: f32 = 16.0;
            const CONFIRM_BUTTON_HEIGHT: f32 = 28.0;
            const CONFIRM_BUTTON_WIDTH: f32 = 78.0;
            const CONFIRM_BUTTON_GAP: f32 = 8.0;

            let content_height = window_height - CONFIRM_FOOTER_HEIGHT;
            let stack_width = CONFIRM_STACK_WIDTH.min(window_width - CONFIRM_CONTENT_PADDING_X * 2.0);
            let stack_height = CONFIRM_TITLE_HEIGHT + CONFIRM_STACK_GAP + CONFIRM_BODY_HEIGHT;
            let stack_x = (window_width - stack_width) / 2.0;
            let stack_y = (content_height - stack_height) / 2.0;
            let footer_y = window_height - CONFIRM_FOOTER_HEIGHT;
            let cancel_x = window_width - 16.0 - CONFIRM_BUTTON_WIDTH;
            let confirm_x = cancel_x - CONFIRM_BUTTON_GAP - CONFIRM_BUTTON_WIDTH;

            components.push(
                LayoutComponentInfo::new("ConfirmPromptContent", LayoutComponentType::Panel)
                    .with_bounds(0.0, 0.0, window_width, content_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                    )
                    .with_visual_token("confirm.content")
                    .with_padding(
                        CONFIRM_CONTENT_PADDING_Y,
                        CONFIRM_CONTENT_PADDING_X,
                        CONFIRM_CONTENT_PADDING_Y,
                        CONFIRM_CONTENT_PADDING_X,
                    )
                    .with_flex_column()
                    .with_depth(1)
                    .with_parent("Window")
                    .with_explanation("ConfirmPrompt content fills the standard-height window above the native footer."),
            );
            components.push(
                LayoutComponentInfo::new("ConfirmPromptStack", LayoutComponentType::Container)
                    .with_bounds(stack_x, stack_y, stack_width, stack_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("confirm.stack")
                    .with_gap(CONFIRM_STACK_GAP)
                    .with_flex_column()
                    .with_depth(2)
                    .with_parent("ConfirmPromptContent")
                    .with_explanation("Centered title/body stack with 560px maximum text width."),
            );
            components.push(
                LayoutComponentInfo::new("ConfirmPromptTitle", LayoutComponentType::Header)
                    .with_bounds(stack_x, stack_y, stack_width, CONFIRM_TITLE_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("confirm.title")
                    .with_depth(3)
                    .with_parent("ConfirmPromptStack")
                    .with_explanation("20px semibold title centered in the confirm prompt."),
            );
            components.push(
                LayoutComponentInfo::new("ConfirmPromptBody", LayoutComponentType::Other)
                    .with_bounds(
                        stack_x,
                        stack_y + CONFIRM_TITLE_HEIGHT + CONFIRM_STACK_GAP,
                        stack_width,
                        CONFIRM_BODY_HEIGHT,
                    )
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("confirm.body")
                    .with_depth(3)
                    .with_parent("ConfirmPromptStack")
                    .with_explanation("Centered body copy is bounded to 560px to avoid edge-to-edge reading lines."),
            );
            components.push(
                LayoutComponentInfo::new("ConfirmPromptFooter", LayoutComponentType::Panel)
                    .with_bounds(0.0, footer_y, window_width, CONFIRM_FOOTER_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                    )
                    .with_visual_token("confirm.footer")
                    .with_flex_row()
                    .with_depth(1)
                    .with_parent("Window")
                    .with_explanation("Native footer region owns confirm/cancel button affordances."),
            );
            for (name, x) in [
                ("ConfirmPromptConfirmButton", confirm_x),
                ("ConfirmPromptCancelButton", cancel_x),
            ] {
                components.push(
                    LayoutComponentInfo::new(name, LayoutComponentType::Button)
                        .with_bounds(x, footer_y + 5.0, CONFIRM_BUTTON_WIDTH, CONFIRM_BUTTON_HEIGHT)
                        .with_visual_style(
                            chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                            chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                            Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                        )
                        .with_visual_token("confirm.footerButton")
                        .with_hit_bounds(x, footer_y + 5.0, CONFIRM_BUTTON_WIDTH, CONFIRM_BUTTON_HEIGHT)
                        .with_depth(2)
                        .with_parent("ConfirmPromptFooter")
                        .with_explanation("Footer button is 28px tall with the shared 10px compact Liquid Glass radius."),
                );
            }

            return LayoutInfo {
                window_width,
                window_height,
                prompt_type: prompt_type.to_string(),
                components,
                handler_form: None,
                timestamp: chrono::Utc::now().to_rfc3339(),
            };
        }

        if let AppView::BrowseKitsView { results, .. } = &self.current_view {
            const KIT_STORE_HEADER_HEIGHT: f32 = 44.0;
            const KIT_STORE_HEADER_PADDING_X: f32 = 16.0;
            const KIT_STORE_HEADER_PADDING_Y: f32 = 8.0;
            const KIT_STORE_HEADER_GAP: f32 = 12.0;
            const KIT_STORE_TITLE_WIDTH: f32 = 132.0;
            const KIT_STORE_COUNT_WIDTH: f32 = 68.0;
            const KIT_STORE_INPUT_HEIGHT: f32 = 28.0;
            const KIT_STORE_DIVIDER_HEIGHT: f32 = 1.0;
            const KIT_STORE_LIST_PADDING_Y: f32 = 4.0;
            const KIT_STORE_ROW_HEIGHT: f32 = 72.0;
            const KIT_STORE_ROW_PADDING_X: f32 = 12.0;
            const KIT_STORE_ROW_PADDING_Y: f32 = 8.0;
            const KIT_STORE_ROW_GAP: f32 = 12.0;
            const KIT_STORE_INSTALL_WIDTH: f32 = 62.0;
            const KIT_STORE_INSTALL_HEIGHT: f32 = 28.0;
            const KIT_STORE_FOOTER_HEIGHT: f32 = 34.0;

            let divider_y = KIT_STORE_HEADER_HEIGHT;
            let list_top =
                divider_y + KIT_STORE_DIVIDER_HEIGHT + KIT_STORE_LIST_PADDING_Y;
            let footer_y = window_height - KIT_STORE_FOOTER_HEIGHT;
            let list_height =
                (footer_y - list_top - KIT_STORE_LIST_PADDING_Y).max(0.0);
            let input_x =
                KIT_STORE_HEADER_PADDING_X + KIT_STORE_TITLE_WIDTH + KIT_STORE_HEADER_GAP;
            let input_width = (window_width
                - input_x
                - KIT_STORE_HEADER_GAP
                - KIT_STORE_COUNT_WIDTH
                - KIT_STORE_HEADER_PADDING_X)
                .max(0.0);
            let input_y =
                KIT_STORE_HEADER_PADDING_Y + (KIT_STORE_INPUT_HEIGHT - 28.0) / 2.0;
            let count_x = input_x + input_width + KIT_STORE_HEADER_GAP;

            components.push(
                LayoutComponentInfo::new("KitStoreBrowseSurface", LayoutComponentType::Panel)
                    .with_bounds(0.0, 0.0, window_width, window_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                    )
                    .with_visual_token("kitStoreBrowse.surface")
                    .with_flex_column()
                    .with_depth(1)
                    .with_parent("Window")
                    .with_explanation("Browse Kit Store owns a custom full-window surface instead of the generic launcher split shell."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreBrowseHeader", LayoutComponentType::Header)
                    .with_bounds(0.0, 0.0, window_width, KIT_STORE_HEADER_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                    )
                    .with_visual_token("kitStoreBrowse.header")
                    .with_padding(
                        KIT_STORE_HEADER_PADDING_Y,
                        KIT_STORE_HEADER_PADDING_X,
                        KIT_STORE_HEADER_PADDING_Y,
                        KIT_STORE_HEADER_PADDING_X,
                    )
                    .with_gap(KIT_STORE_HEADER_GAP)
                    .with_flex_row()
                    .with_depth(2)
                    .with_parent("KitStoreBrowseSurface")
                    .with_explanation("Custom browse header: title, pseudo-search input, and result count on a 12px gap."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreBrowseTitle", LayoutComponentType::Other)
                    .with_bounds(
                        KIT_STORE_HEADER_PADDING_X,
                        KIT_STORE_HEADER_PADDING_Y,
                        KIT_STORE_TITLE_WIDTH,
                        28.0,
                    )
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("kitStoreBrowse.title")
                    .with_depth(3)
                    .with_parent("KitStoreBrowseHeader")
                    .with_explanation("Static Browse Kit Store title in the custom header."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreBrowseSearch", LayoutComponentType::Input)
                    .with_bounds(input_x, input_y, input_width, 22.0)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_CONTROL_RADIUS_PX),
                    )
                    .with_visual_token("kitStoreBrowse.search")
                    .with_hit_bounds(input_x, KIT_STORE_HEADER_PADDING_Y, input_width, 28.0)
                    .with_depth(3)
                    .with_parent("KitStoreBrowseHeader")
                    .with_explanation("Pseudo-search input keeps a 22px visual text lane with a 28px minimum hit target."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreBrowseCount", LayoutComponentType::Other)
                    .with_bounds(
                        count_x,
                        KIT_STORE_HEADER_PADDING_Y,
                        KIT_STORE_COUNT_WIDTH,
                        28.0,
                    )
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("kitStoreBrowse.count")
                    .with_depth(3)
                    .with_parent("KitStoreBrowseHeader")
                    .with_explanation("Result count text remains in the functional header chrome."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreBrowseDivider", LayoutComponentType::Other)
                    .with_bounds(
                        KIT_STORE_HEADER_PADDING_X,
                        divider_y,
                        window_width - KIT_STORE_HEADER_PADDING_X * 2.0,
                        KIT_STORE_DIVIDER_HEIGHT,
                    )
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("kitStoreBrowse.divider")
                    .with_depth(2)
                    .with_parent("KitStoreBrowseSurface")
                    .with_explanation("One-pixel divider inset to the same 16px horizontal header padding."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreBrowseList", LayoutComponentType::List)
                    .with_bounds(0.0, list_top, window_width, list_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("kitStoreBrowse.list")
                    .with_flex_column()
                    .with_depth(2)
                    .with_parent("KitStoreBrowseSurface")
                    .with_explanation("Custom kit-store list region uses full width and 72px browse rows."),
            );

            let visible_rows = ((list_height / KIT_STORE_ROW_HEIGHT) as usize)
                .min(results.len())
                .min(5);
            if visible_rows == 0 {
                components.push(
                    LayoutComponentInfo::new("KitStoreBrowseEmptyState", LayoutComponentType::Panel)
                        .with_bounds(0.0, list_top, window_width, list_height)
                        .with_visual_style(
                            chrome_tokens::CHROME_LAYER_CONTENT,
                            chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                            Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                        )
                        .with_visual_token("kitStoreBrowse.emptyState")
                        .with_depth(3)
                        .with_parent("KitStoreBrowseList")
                        .with_explanation("Centered empty state occupies the browse list when no remote rows are available."),
                );
            } else {
                for i in 0..visible_rows {
                    let row_y = list_top + i as f32 * KIT_STORE_ROW_HEIGHT;
                    let install_x =
                        window_width - KIT_STORE_ROW_PADDING_X - KIT_STORE_INSTALL_WIDTH;
                    let install_y = row_y + (KIT_STORE_ROW_HEIGHT - KIT_STORE_INSTALL_HEIGHT) / 2.0;
                    components.push(
                        LayoutComponentInfo::new(
                            format!("KitStoreBrowseRow[{}]", i),
                            LayoutComponentType::ListItem,
                        )
                        .with_bounds(0.0, row_y, window_width, KIT_STORE_ROW_HEIGHT)
                        .with_visual_style(
                            chrome_tokens::CHROME_LAYER_CONTENT,
                            chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                            Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                        )
                        .with_visual_token("kitStoreBrowse.row")
                        .with_padding(
                            KIT_STORE_ROW_PADDING_Y,
                            KIT_STORE_ROW_PADDING_X,
                            KIT_STORE_ROW_PADDING_Y,
                            KIT_STORE_ROW_PADDING_X,
                        )
                        .with_gap(KIT_STORE_ROW_GAP)
                        .with_flex_row()
                        .with_depth(3)
                        .with_parent("KitStoreBrowseList")
                        .with_explanation("Browse rows are 72px tall with 12px horizontal padding, 8px vertical padding, and a 12px text/action gap."),
                    );
                    components.push(
                        LayoutComponentInfo::new(
                            format!("KitStoreBrowseInstallButton[{}]", i),
                            LayoutComponentType::Button,
                        )
                        .with_bounds(
                            install_x,
                            install_y,
                            KIT_STORE_INSTALL_WIDTH,
                            KIT_STORE_INSTALL_HEIGHT,
                        )
                        .with_visual_style(
                            chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                            chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                            Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                        )
                        .with_visual_token("kitStoreBrowse.installButton")
                        .with_hit_bounds(
                            install_x,
                            install_y,
                            KIT_STORE_INSTALL_WIDTH,
                            KIT_STORE_INSTALL_HEIGHT,
                        )
                        .with_depth(4)
                        .with_parent(format!("KitStoreBrowseRow[{}]", i))
                        .with_explanation("Inline Install action keeps the renderer's badge-sized visual with a 28px minimum hit target."),
                    );
                }
            }

            components.push(
                LayoutComponentInfo::new("KitStoreBrowseFooter", LayoutComponentType::Container)
                    .with_bounds(0.0, footer_y, window_width, KIT_STORE_FOOTER_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                    )
                    .with_visual_token("kitStoreBrowse.footer")
                    .with_depth(2)
                    .with_parent("KitStoreBrowseSurface")
                    .with_explanation("Native footer slot for Install and Back/Clear Search hints; it must not overlap the list viewport."),
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

        if let AppView::InstalledKitsView { kits, .. } = &self.current_view {
            const KIT_STORE_HEADER_HEIGHT: f32 = 44.0;
            const KIT_STORE_HEADER_PADDING_X: f32 = 16.0;
            const KIT_STORE_HEADER_PADDING_Y: f32 = 8.0;
            const KIT_STORE_TITLE_WIDTH: f32 = 132.0;
            const KIT_STORE_COUNT_WIDTH: f32 = 96.0;
            const KIT_STORE_DIVIDER_HEIGHT: f32 = 1.0;
            const KIT_STORE_LIST_PADDING_Y: f32 = 4.0;
            const KIT_STORE_ROW_HEIGHT: f32 = 72.0;
            const KIT_STORE_ROW_PADDING_X: f32 = 12.0;
            const KIT_STORE_ROW_PADDING_Y: f32 = 8.0;
            const KIT_STORE_ROW_GAP: f32 = 12.0;
            const KIT_STORE_ACTION_GAP: f32 = 8.0;
            const KIT_STORE_UPDATE_WIDTH: f32 = 62.0;
            const KIT_STORE_REMOVE_WIDTH: f32 = 66.0;
            const KIT_STORE_ACTION_HEIGHT: f32 = 28.0;
            const KIT_STORE_FOOTER_HEIGHT: f32 = 34.0;

            let divider_y = KIT_STORE_HEADER_HEIGHT;
            let list_top =
                divider_y + KIT_STORE_DIVIDER_HEIGHT + KIT_STORE_LIST_PADDING_Y;
            let footer_y = window_height - KIT_STORE_FOOTER_HEIGHT;
            let list_height =
                (footer_y - list_top - KIT_STORE_LIST_PADDING_Y).max(0.0);
            let count_x =
                window_width - KIT_STORE_HEADER_PADDING_X - KIT_STORE_COUNT_WIDTH;

            components.push(
                LayoutComponentInfo::new("KitStoreInstalledSurface", LayoutComponentType::Panel)
                    .with_bounds(0.0, 0.0, window_width, window_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                    )
                    .with_visual_token("kitStoreInstalled.surface")
                    .with_flex_column()
                    .with_depth(1)
                    .with_parent("Window")
                    .with_explanation("Installed Kits owns a custom full-window surface instead of the generic launcher split shell."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreInstalledHeader", LayoutComponentType::Header)
                    .with_bounds(0.0, 0.0, window_width, KIT_STORE_HEADER_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                    )
                    .with_visual_token("kitStoreInstalled.header")
                    .with_padding(
                        KIT_STORE_HEADER_PADDING_Y,
                        KIT_STORE_HEADER_PADDING_X,
                        KIT_STORE_HEADER_PADDING_Y,
                        KIT_STORE_HEADER_PADDING_X,
                    )
                    .with_flex_row()
                    .with_depth(2)
                    .with_parent("KitStoreInstalledSurface")
                    .with_explanation("Custom installed-kits header owns the title and installed count."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreInstalledTitle", LayoutComponentType::Other)
                    .with_bounds(
                        KIT_STORE_HEADER_PADDING_X,
                        KIT_STORE_HEADER_PADDING_Y,
                        KIT_STORE_TITLE_WIDTH,
                        28.0,
                    )
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("kitStoreInstalled.title")
                    .with_depth(3)
                    .with_parent("KitStoreInstalledHeader")
                    .with_explanation("Static Installed Kits title in the custom header."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreInstalledCount", LayoutComponentType::Other)
                    .with_bounds(
                        count_x,
                        KIT_STORE_HEADER_PADDING_Y,
                        KIT_STORE_COUNT_WIDTH,
                        28.0,
                    )
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("kitStoreInstalled.count")
                    .with_depth(3)
                    .with_parent("KitStoreInstalledHeader")
                    .with_explanation("Installed count text remains in the functional header chrome."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreInstalledDivider", LayoutComponentType::Other)
                    .with_bounds(
                        KIT_STORE_HEADER_PADDING_X,
                        divider_y,
                        window_width - KIT_STORE_HEADER_PADDING_X * 2.0,
                        KIT_STORE_DIVIDER_HEIGHT,
                    )
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("kitStoreInstalled.divider")
                    .with_depth(2)
                    .with_parent("KitStoreInstalledSurface")
                    .with_explanation("One-pixel divider inset to the same 16px horizontal header padding."),
            );
            components.push(
                LayoutComponentInfo::new("KitStoreInstalledList", LayoutComponentType::List)
                    .with_bounds(0.0, list_top, window_width, list_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("kitStoreInstalled.list")
                    .with_flex_column()
                    .with_depth(2)
                    .with_parent("KitStoreInstalledSurface")
                    .with_explanation("Custom installed-kits list region uses full width and 72px rows."),
            );

            let visible_rows = ((list_height / KIT_STORE_ROW_HEIGHT) as usize)
                .min(kits.len())
                .min(5);
            if visible_rows == 0 {
                components.push(
                    LayoutComponentInfo::new(
                        "KitStoreInstalledEmptyState",
                        LayoutComponentType::Panel,
                    )
                    .with_bounds(0.0, list_top, window_width, list_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                    )
                    .with_visual_token("kitStoreInstalled.emptyState")
                    .with_depth(3)
                    .with_parent("KitStoreInstalledList")
                    .with_explanation("Centered empty state occupies the installed-kits list when no kits are installed."),
                );
            } else {
                for i in 0..visible_rows {
                    let row_y = list_top + i as f32 * KIT_STORE_ROW_HEIGHT;
                    let remove_x =
                        window_width - KIT_STORE_ROW_PADDING_X - KIT_STORE_REMOVE_WIDTH;
                    let update_x = remove_x - KIT_STORE_ACTION_GAP - KIT_STORE_UPDATE_WIDTH;
                    let action_y = row_y + (KIT_STORE_ROW_HEIGHT - KIT_STORE_ACTION_HEIGHT) / 2.0;
                    components.push(
                        LayoutComponentInfo::new(
                            format!("KitStoreInstalledRow[{}]", i),
                            LayoutComponentType::ListItem,
                        )
                        .with_bounds(0.0, row_y, window_width, KIT_STORE_ROW_HEIGHT)
                        .with_visual_style(
                            chrome_tokens::CHROME_LAYER_CONTENT,
                            chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                            Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                        )
                        .with_visual_token("kitStoreInstalled.row")
                        .with_padding(
                            KIT_STORE_ROW_PADDING_Y,
                            KIT_STORE_ROW_PADDING_X,
                            KIT_STORE_ROW_PADDING_Y,
                            KIT_STORE_ROW_PADDING_X,
                        )
                        .with_gap(KIT_STORE_ROW_GAP)
                        .with_flex_row()
                        .with_depth(3)
                        .with_parent("KitStoreInstalledList")
                        .with_explanation("Installed kit rows are 72px tall with 12px horizontal padding, 8px vertical padding, and a 12px text/action gap."),
                    );
                    for (name, x, width) in [
                        ("KitStoreInstalledUpdateButton", update_x, KIT_STORE_UPDATE_WIDTH),
                        ("KitStoreInstalledRemoveButton", remove_x, KIT_STORE_REMOVE_WIDTH),
                    ] {
                        components.push(
                            LayoutComponentInfo::new(
                                format!("{}[{}]", name, i),
                                LayoutComponentType::Button,
                            )
                            .with_bounds(x, action_y, width, KIT_STORE_ACTION_HEIGHT)
                            .with_visual_style(
                                chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                                chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                                Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                            )
                            .with_visual_token("kitStoreInstalled.actionButton")
                            .with_hit_bounds(x, action_y, width, KIT_STORE_ACTION_HEIGHT)
                            .with_depth(4)
                            .with_parent(format!("KitStoreInstalledRow[{}]", i))
                            .with_explanation("Inline installed-kit action keeps a badge-sized visual with a 28px minimum hit target."),
                        );
                    }
                }
            }

            components.push(
                LayoutComponentInfo::new("KitStoreInstalledFooter", LayoutComponentType::Container)
                    .with_bounds(0.0, footer_y, window_width, KIT_STORE_FOOTER_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                    )
                    .with_visual_token("kitStoreInstalled.footer")
                    .with_depth(2)
                    .with_parent("KitStoreInstalledSurface")
                    .with_explanation("Native footer slot for Update and Remove hints; it must not overlap the list viewport."),
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

        // Header
        components.push(
            LayoutComponentInfo::new("Header", LayoutComponentType::Header)
                .with_bounds(0.0, 0.0, window_width, header_height)
                .with_visual_style(
                    chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                    chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                    Some(chrome_tokens::LIQUID_GLASS_PANEL_RADIUS_PX),
                )
                .with_visual_token("chrome.header")
                .with_padding(HEADER_PADDING_Y, HEADER_PADDING_X, HEADER_PADDING_Y, HEADER_PADDING_X)
                .with_flex_row()
                .with_depth(1)
                .with_parent("Window")
                .with_explanation(format!(
                    "Height = padding({}) + content({}) + padding({}) + divider({}) = {}px. Uses flex-row with items-center.",
                    HEADER_PADDING_Y, BUTTON_HEIGHT, HEADER_PADDING_Y, DIVIDER_HEIGHT, header_height
                )),
        );

        // Header controls, right-to-left. Keep these before SearchInput so the
        // measured input width is derived from the real button group edge.
        let button_y = HEADER_PADDING_Y;
        let button_height = BUTTON_HEIGHT;
        let logo_x = window_width - HEADER_PADDING_X - 20.0;
        let actions_width = 85.0;
        let actions_x = logo_x - 24.0 - actions_width;
        let run_width = 55.0;
        let run_x = actions_x - 24.0 - run_width;
        let input_to_run_gap = 12.0;

        // Search input in header
        const INPUT_HEIGHT: f32 = 22.0;
        let input_y = HEADER_PADDING_Y + (BUTTON_HEIGHT - INPUT_HEIGHT) / 2.0;
        let input_width = (run_x - input_to_run_gap - HEADER_PADDING_X).max(0.0);
        let buttons_area_width = window_width - HEADER_PADDING_X - input_width;

        components.push(
            LayoutComponentInfo::new("SearchInput", LayoutComponentType::Input)
                .with_bounds(HEADER_PADDING_X, input_y, input_width, INPUT_HEIGHT)
                .with_visual_style(
                    chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                    chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                    Some(chrome_tokens::LIQUID_GLASS_CONTROL_RADIUS_PX),
                )
                .with_visual_token("chrome.searchInput")
                .with_hit_bounds(HEADER_PADDING_X, HEADER_PADDING_Y, input_width, BUTTON_HEIGHT)
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
            {
                let component = LayoutComponentInfo::new("ContentArea", LayoutComponentType::Container)
                .with_bounds(0.0, content_top, window_width, content_height)
                .with_visual_style(
                    chrome_tokens::CHROME_LAYER_CONTENT,
                    chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                    None,
                )
                .with_visual_token("content.background")
                .with_flex_grow(1.0)
                .with_depth(1)
                .with_parent("Window");
                if uses_split_preview {
                    component
                        .with_flex_row()
                        .with_explanation("flex-grow:1 fills remaining height after header. Uses flex-row to create side-by-side panels.".to_string())
                } else {
                    component
                        .with_flex_column()
                        .with_explanation("flex-grow:1 fills remaining height after header. Mini receipts use a single full-width column.".to_string())
                }
            },
        );

        if matches!(
            self.current_view,
            AppView::WebcamView { .. }
                | AppView::DivPrompt { .. }
                | AppView::EditorPrompt { .. }
                | AppView::TermPrompt { .. }
                | AppView::ScratchPadView { .. }
                | AppView::QuickTerminalView { .. }
        ) {
            let (component_name, explanation, token) = match &self.current_view {
                AppView::WebcamView { .. } => (
                    "WebcamContent",
                    "WebcamView fills the standard prompt content area with the media preview or deterministic camera startup/error state; native footer owns Capture Photo and Actions.",
                    "content.webcamPreview",
                ),
                AppView::DivPrompt { .. } => (
                    "DivContent",
                    "DivPrompt fills the content area with scrollable HTML content and footer ownership routed through the shared main-window footer slot.",
                    "content.promptBody",
                ),
                AppView::EditorPrompt { .. } => (
                    "EditorContent",
                    "EditorPrompt fills the content area; footer ownership is routed through the shared main-window footer slot.",
                    "content.promptBody",
                ),
                AppView::QuickTerminalView { .. } => (
                    "TerminalContent",
                    "QuickTerminalView fills the compact content area and reserves native-footer space through the shared main-window footer slot.",
                    "content.promptBody",
                ),
                AppView::ScratchPadView { .. } => (
                    "ScratchPadContent",
                    "ScratchPadView fills the editor-height utility content area with an auto-saving editor and prompt-owned footer handling.",
                    "content.promptBody",
                ),
                _ => (
                    "TerminalContent",
                    "TermPrompt fills the content area and owns the SDK terminal hint strip through the shared main-window footer slot.",
                    "content.promptBody",
                ),
            };

            components.push(
                LayoutComponentInfo::new(component_name, LayoutComponentType::Prompt)
                    .with_bounds(0.0, content_top, window_width, content_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token(token)
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
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token(match &self.current_view {
                        AppView::SelectPrompt { .. } => "content.promptChoices",
                        _ => "content.promptDrop",
                    })
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
            AppView::EnvPrompt { .. }
                | AppView::NamingPrompt { .. }
                | AppView::CreateAiPresetView { .. }
        ) {
            let (component_name, explanation, token) = match &self.current_view {
                AppView::EnvPrompt { .. } => (
                    "EnvPromptContent",
                    "EnvPrompt fills the standard prompt content area with setup text, a secret-aware input, and a prompt-owned submit footer.",
                    "content.explicitEnvPrompt",
                ),
                AppView::NamingPrompt { .. } => (
                    "NamingPromptContent",
                    "NamingPrompt fills the compact explicit prompt content area with a name input and prompt-owned submit handling.",
                    "content.explicitNamingPrompt",
                ),
                _ => (
                    "CreateAiPresetContent",
                    "CreateAiPresetView fills the compact explicit prompt content area with preset setup fields and prompt-owned submit handling.",
                    "content.explicitCreateAiPreset",
                ),
            };

            components.push(
                LayoutComponentInfo::new(component_name, LayoutComponentType::Prompt)
                    .with_bounds(0.0, content_top, window_width, content_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token(token)
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

        // Script list: full width for MainWindow, left panel for split-preview surfaces.
        components.push(
            LayoutComponentInfo::new("ScriptList", LayoutComponentType::List)
                .with_bounds(0.0, content_top, list_width, content_height)
                .with_visual_style(
                    chrome_tokens::CHROME_LAYER_CONTENT,
                    chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                    None,
                )
                .with_visual_token("content.list")
                .with_flex_column()
                .with_depth(2)
                .with_parent("ContentArea")
                .with_explanation(format!(
                    "Width = {}px. Uses uniform_list for virtualized scrolling with {}px item height.",
                    list_width, LIST_ITEM_HEIGHT
                )),
        );

        if uses_split_preview {
            // Preview panel (right panel) - remaining 50%
            let preview_width = window_width - list_width;
            components.push(
                LayoutComponentInfo::new("PreviewPanel", LayoutComponentType::Panel)
                    .with_bounds(list_width, content_top, preview_width, content_height)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        None,
                    )
                    .with_visual_token("content.previewPanel")
                    .with_padding(16.0, 16.0, 16.0, 16.0)
                    .with_flex_column()
                    .with_depth(2)
                    .with_parent("ContentArea")
                    .with_explanation(format!(
                        "Width = remaining 50% = {}px. Has 16px padding on all sides.",
                        preview_width
                    )),
            );
        }

        // List items (sample of first few visible)
        let visible_items = ((content_height / LIST_ITEM_HEIGHT) as usize).min(5);
        for i in 0..visible_items {
            let item_top = content_top + (i as f32 * LIST_ITEM_HEIGHT);
            components.push(
                LayoutComponentInfo::new(format!("ListItem[{}]", i), LayoutComponentType::ListItem)
                    .with_bounds(0.0, item_top, list_width, LIST_ITEM_HEIGHT)
                    .with_visual_style(
                        chrome_tokens::CHROME_LAYER_CONTENT,
                        chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                        Some(chrome_tokens::LIQUID_GLASS_COMPACT_RADIUS_PX),
                    )
                    .with_visual_token("content.listItem")
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

        // Logo button (rightmost)
        components.push(
            LayoutComponentInfo::new("LogoButton", LayoutComponentType::Button)
                .with_bounds(logo_x, button_y, 20.0, button_height)
                .with_visual_style(
                    chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                    chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                    Some(button_height / 2.0),
                )
                .with_visual_token("chrome.headerButton")
                .with_hit_bounds(
                    logo_x,
                    button_y,
                    chrome_tokens::LIQUID_GLASS_MIN_HIT_PX,
                    button_height,
                )
                .with_visual_exception("compactIconButton")
                .with_padding(4.0, 4.0, 4.0, 4.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation("Fixed 20px width. Positioned at right edge with 16px margin."),
        );

        // Actions button
        components.push(
            LayoutComponentInfo::new("ActionsButton", LayoutComponentType::Button)
                .with_bounds(actions_x, button_y, actions_width, button_height)
                .with_visual_style(
                    chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                    chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                    Some(button_height / 2.0),
                )
                .with_visual_token("chrome.headerButton")
                .with_padding(4.0, 8.0, 4.0, 8.0)
                .with_depth(2)
                .with_parent("Header")
                .with_explanation(format!(
                    "Width = {}px. Positioned left of logo with 24px spacing (includes divider).",
                    actions_width
                )),
        );

        // Run button
        components.push(
            LayoutComponentInfo::new("RunButton", LayoutComponentType::Button)
                .with_bounds(run_x, button_y, run_width, button_height)
                .with_visual_style(
                    chrome_tokens::CHROME_LAYER_FUNCTIONAL,
                    chrome_tokens::MATERIAL_SOLID_THEME_TOKEN,
                    Some(button_height / 2.0),
                )
                .with_visual_token("chrome.headerButton")
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
        let field_width = content_width;

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
                        "surface": "menuSyntaxTriggerPopup",
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
