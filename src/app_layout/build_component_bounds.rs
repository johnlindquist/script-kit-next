impl ScriptListApp {
    fn build_component_bounds(
        &self,
        window_size: gpui::Size<gpui::Pixels>,
    ) -> Vec<debug_grid::ComponentBounds> {
        use crate::list_item::LIST_ITEM_HEIGHT;
        use debug_grid::{BoxModel, ComponentBounds, ComponentType};

        let mut bounds = Vec::new();
        let width = window_size.width;
        let height = window_size.height;
        let menu_def = self.current_main_menu_theme.def();

        // Header bounds follow the same active main-menu theme tokens as the
        // shared rendered chrome.

        // Content padding matches HEADER_PADDING_X
        let content_padding = menu_def.shell.header_padding_x;

        // Determine the current view type and build appropriate bounds
        let view_name = match &self.current_view {
            AppView::ScriptList => "ScriptList",
            AppView::About { .. } => "About",
            AppView::ArgPrompt { .. } => "ArgPrompt",
            AppView::DivPrompt { .. } => "DivPrompt",
            AppView::EditorPrompt { .. } => "EditorPrompt",
            AppView::TermPrompt { .. } => "TermPrompt",
            AppView::FormPrompt { .. } => "FormPrompt",
            AppView::SelectPrompt { .. } => "SelectPrompt",
            AppView::PathPrompt { .. } => "PathPrompt",
            AppView::EnvPrompt { .. } => "EnvPrompt",
            AppView::DropPrompt { .. } => "DropPrompt",
            AppView::TemplatePrompt { .. } => "TemplatePrompt",
            AppView::HotkeyPrompt { .. } => "HotkeyPrompt",
            AppView::ChatPrompt { .. } => "ChatPrompt",
            AppView::MiniPrompt { .. } => "MiniPrompt",
            AppView::MicroPrompt { .. } => "MicroPrompt",
            AppView::ClipboardHistoryView { .. } => "ClipboardHistory",
            AppView::AppLauncherView { .. } => "AppLauncher",
            AppView::WindowSwitcherView { .. } => "WindowSwitcher",
            AppView::BrowserTabsView { .. } => "BrowserTabs",
            AppView::DesignGalleryView { .. } => "DesignGallery",
            AppView::FooterGalleryView { .. } => "FooterGallery",
            AppView::NonListStatesView { .. } => "NonListStates",
            #[cfg(feature = "storybook")]
            AppView::DesignExplorerView { .. } => "DesignExplorer",
            AppView::ScratchPadView { .. } => "ScratchPad",
            AppView::QuickTerminalView { .. } => "QuickTerminal",
            AppView::FileSearchView { .. } => "FileSearch",
            AppView::ProfileSearchView { .. } => "ProfileSearch",
            AppView::ThemeChooserView { .. } => "ThemeChooser",
            AppView::EmojiPickerView { .. } => "EmojiPicker",
            AppView::ActionsDialog => "ActionsDialog",
            AppView::WebcamView { .. } => "Webcam",
            AppView::CreationFeedback { .. } => "CreationFeedback",
            AppView::NamingPrompt { .. } => "NamingPrompt",
            AppView::BrowseKitsView { .. } => "BrowseKits",
            AppView::InstalledKitsView { .. } => "InstalledKits",
            AppView::ProcessManagerView { .. } => "ProcessManager",
            AppView::CurrentAppCommandsView { .. } => "CurrentAppCommands",
            AppView::SearchAiPresetsView { .. } => "SearchAiPresets",
            AppView::CreateAiPresetView { .. } => "CreateAiPreset",
            AppView::SettingsView { .. } => "Settings",
            AppView::FavoritesBrowseView { .. } => "FavoritesBrowse",
            AppView::AgentChatHistoryView { .. } => "AgentChatHistory",
            AppView::BrowserHistoryView { .. } => "BrowserHistory",
            AppView::DictationHistoryView { .. } => "DictationHistory",
            AppView::NotesBrowseView { .. } => "NotesBrowse",
            AppView::AgentChatView { .. } => "AgentChat",
            AppView::ScriptIssuesView { .. } => "ScriptIssues",
            AppView::SdkReferenceView { .. } => "SdkReference",
            AppView::ScriptTemplateCatalogView { .. } => "ScriptTemplateCatalog",
            AppView::ConfirmPrompt { .. } => "ConfirmPrompt",
        };

        let main_view_has_context_zone = matches!(
            self.current_view,
            AppView::ScriptList
                | AppView::FileSearchView { .. }
                | AppView::ClipboardHistoryView { .. }
                | AppView::ProfileSearchView { .. }
                | AppView::AgentChatView { .. }
        );
        let main_view_context_zone_height = menu_def.header_info_bar.height_px;
        let main_view_header_content_height = if main_view_has_context_zone {
            menu_def.search.height + menu_def.shell.header_gap + main_view_context_zone_height
        } else {
            menu_def.search.height
        };
        let header_height =
            px(menu_def.shell.header_padding_y * 2.0 + main_view_header_content_height);
        let content_top = header_height;
        let content_height = height - header_height;

        // Header bounds (includes padding + context + input; no divider) - common to all views
        bounds.push(
            ComponentBounds::new(
                "MainViewHeader",
                gpui::Bounds {
                    origin: gpui::point(px(0.), px(0.)),
                    size: gpui::size(width, header_height),
                },
            )
            .with_type(ComponentType::Header)
            .with_padding(BoxModel::symmetric(
                menu_def.shell.header_padding_y,
                content_padding,
            )),
        );

        // Build view-specific bounds
        match &self.current_view {
            AppView::ScriptList => {
                let uses_split_preview = matches!(self.main_window_mode, MainWindowMode::Full);
                let list_width = if uses_split_preview {
                    width * 0.5
                } else {
                    width
                };
                let item_height = px(LIST_ITEM_HEIGHT);

                bounds.push(
                    ComponentBounds::new(
                        "ScriptList",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(list_width, content_height),
                        },
                    )
                    .with_type(ComponentType::List)
                    .with_padding(BoxModel::uniform(0.0)),
                );

                // Add sample list items
                for i in 0..5 {
                    let item_top = content_top + px(i as f32 * LIST_ITEM_HEIGHT);
                    if item_top + item_height > height {
                        break;
                    }
                    bounds.push(
                        ComponentBounds::new(
                            format!("ListItem[{}]", i),
                            gpui::Bounds {
                                origin: gpui::point(px(0.), item_top),
                                size: gpui::size(list_width, item_height),
                            },
                        )
                        .with_type(ComponentType::ListItem)
                        .with_padding(BoxModel::symmetric(12.0, content_padding))
                        .with_margin(BoxModel::uniform(0.0)),
                    );
                }

                if uses_split_preview {
                    // Preview panel (right side)
                    bounds.push(
                        ComponentBounds::new(
                            "PreviewPanel",
                            gpui::Bounds {
                                origin: gpui::point(list_width, content_top),
                                size: gpui::size(width - list_width, content_height),
                            },
                        )
                        .with_type(ComponentType::Container)
                        .with_padding(BoxModel::uniform(content_padding)),
                    );
                }
            }

            AppView::DivPrompt { .. } => {
                // DivPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "DivContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::EditorPrompt { .. } => {
                // EditorPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "EditorContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::TermPrompt { .. } => {
                // TermPrompt takes full width below header
                bounds.push(
                    ComponentBounds::new(
                        "TerminalContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::ArgPrompt { choices, .. } => {
                // ArgPrompt may have choices list
                if choices.is_empty() {
                    // No choices - just input area
                    bounds.push(
                        ComponentBounds::new(
                            "ArgInput",
                            gpui::Bounds {
                                origin: gpui::point(px(0.), content_top),
                                size: gpui::size(width, content_height),
                            },
                        )
                        .with_type(ComponentType::Prompt)
                        .with_padding(BoxModel::uniform(content_padding)),
                    );
                } else {
                    // Has choices - show list
                    let item_height = px(LIST_ITEM_HEIGHT);
                    bounds.push(
                        ComponentBounds::new(
                            "ChoicesList",
                            gpui::Bounds {
                                origin: gpui::point(px(0.), content_top),
                                size: gpui::size(width, content_height),
                            },
                        )
                        .with_type(ComponentType::List)
                        .with_padding(BoxModel::uniform(0.0)),
                    );

                    // Add choice items
                    for i in 0..choices.len().min(5) {
                        let item_top = content_top + px(i as f32 * LIST_ITEM_HEIGHT);
                        if item_top + item_height > height {
                            break;
                        }
                        bounds.push(
                            ComponentBounds::new(
                                format!("Choice[{}]", i),
                                gpui::Bounds {
                                    origin: gpui::point(px(0.), item_top),
                                    size: gpui::size(width, item_height),
                                },
                            )
                            .with_type(ComponentType::ListItem)
                            .with_padding(BoxModel::symmetric(12.0, content_padding)),
                        );
                    }
                }
            }

            AppView::FormPrompt { .. } => {
                bounds.push(
                    ComponentBounds::new(
                        "FormContent",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }

            AppView::SelectPrompt { .. } | AppView::PathPrompt { .. } => {
                // List-based prompts
                let item_height = px(LIST_ITEM_HEIGHT);
                bounds.push(
                    ComponentBounds::new(
                        view_name,
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::List)
                    .with_padding(BoxModel::uniform(0.0)),
                );

                for i in 0..5 {
                    let item_top = content_top + px(i as f32 * LIST_ITEM_HEIGHT);
                    if item_top + item_height > height {
                        break;
                    }
                    bounds.push(
                        ComponentBounds::new(
                            format!("Item[{}]", i),
                            gpui::Bounds {
                                origin: gpui::point(px(0.), item_top),
                                size: gpui::size(width, item_height),
                            },
                        )
                        .with_type(ComponentType::ListItem)
                        .with_padding(BoxModel::symmetric(12.0, content_padding)),
                    );
                }
            }

            AppView::AgentChatView { .. } => {
                let info_columns =
                    crate::components::main_view_chrome::main_view_content_columns(menu_def);
                let info_metrics = crate::components::info_state::info_metrics(
                    crate::components::info_state::InfoStateDensity::Comfortable,
                );
                let info_x = px(info_columns.text_column_x);
                let info_y = content_top + px(info_columns.top_inset_y);
                let info_width =
                    (width - info_x - px(info_columns.content_right_inset_x)).max(px(0.));
                let shortcut_slot_width = px(
                    crate::components::footer_chrome::FOOTER_KEYCAP_HEIGHT_PX * 2.0
                        + crate::components::footer_chrome::FOOTER_ACTION_CONTENT_GAP_PX,
                );
                let guidance_label_x = info_x
                    + shortcut_slot_width
                    + px(crate::components::info_state::INFO_SPACING.sm);
                let guidance_label_width = (info_width
                    - shortcut_slot_width
                    - px(crate::components::info_state::INFO_SPACING.sm))
                .max(px(0.));

                bounds.push(
                    ComponentBounds::new(
                        "MainViewMain",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Container)
                    .with_padding(BoxModel::uniform(0.0)),
                );
                bounds.push(
                    ComponentBounds::new(
                        "AgentChatConversation",
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::List)
                    .with_padding(BoxModel::uniform(0.0)),
                );
                bounds.push(
                    ComponentBounds::new(
                        "AgentChatEmptyGuidance",
                        gpui::Bounds {
                            origin: gpui::point(info_x, info_y),
                            size: gpui::size(
                                info_width,
                                (content_height
                                    - px(info_columns.top_inset_y)
                                    - px(menu_def.shell.content_inset_bottom))
                                .max(px(0.)),
                            ),
                        },
                    )
                    .with_type(ComponentType::Container)
                    .with_padding(BoxModel::uniform(0.0)),
                );
                bounds.push(
                    ComponentBounds::new(
                        "AgentChatEmptyGuidanceTitle",
                        gpui::Bounds {
                            origin: gpui::point(info_x, info_y),
                            size: gpui::size(
                                info_width,
                                px(crate::components::info_state::INFO_TYPE_SCALE.title.line),
                            ),
                        },
                    )
                    .with_type(ComponentType::Header)
                    .with_padding(BoxModel::uniform(0.0)),
                );
                bounds.push(
                    ComponentBounds::new(
                        "AgentChatEmptyGuidanceShortcutSlot",
                        gpui::Bounds {
                            origin: gpui::point(
                                info_x,
                                info_y
                                    + px(info_metrics.block_gap)
                                    + px(crate::components::info_state::INFO_TYPE_SCALE.title.line)
                                    + px(crate::components::info_state::INFO_SPACING.xs * 0.5)
                                    + px(crate::components::info_state::INFO_TYPE_SCALE.body.line),
                            ),
                            size: gpui::size(shortcut_slot_width, px(info_metrics.row_min_h)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::uniform(0.0)),
                );
                bounds.push(
                    ComponentBounds::new(
                        "AgentChatEmptyGuidanceLabelColumn",
                        gpui::Bounds {
                            origin: gpui::point(
                                guidance_label_x,
                                info_y
                                    + px(info_metrics.block_gap)
                                    + px(crate::components::info_state::INFO_TYPE_SCALE.title.line)
                                    + px(crate::components::info_state::INFO_SPACING.xs * 0.5)
                                    + px(crate::components::info_state::INFO_TYPE_SCALE.body.line),
                            ),
                            size: gpui::size(guidance_label_width, px(info_metrics.row_min_h)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::uniform(0.0)),
                );
            }

            // Other prompts - generic full-width content
            _ => {
                bounds.push(
                    ComponentBounds::new(
                        view_name,
                        gpui::Bounds {
                            origin: gpui::point(px(0.), content_top),
                            size: gpui::size(width, content_height),
                        },
                    )
                    .with_type(ComponentType::Prompt)
                    .with_padding(BoxModel::uniform(content_padding)),
                );
            }
        }

        if matches!(
            self.current_view,
            AppView::ScriptList
                | AppView::FileSearchView { .. }
                | AppView::ClipboardHistoryView { .. }
                | AppView::ProfileSearchView { .. }
                | AppView::AgentChatView { .. }
        ) {
            if main_view_has_context_zone {
                let context_outset_x = px(menu_def.header_info_bar.context_edge_outset_x);
                bounds.push(
                    ComponentBounds::new(
                        "MainViewContextZone",
                        gpui::Bounds {
                            origin: gpui::point(
                                px(menu_def.shell.header_padding_x) - context_outset_x,
                                px(menu_def.shell.header_padding_y),
                            ),
                            size: gpui::size(
                                (width - px(menu_def.shell.header_padding_x * 2.0)
                                    + context_outset_x * 2.0)
                                    .max(px(0.)),
                                px(main_view_context_zone_height),
                            ),
                        },
                    )
                    .with_type(ComponentType::Container),
                );
            }
            // Input field in header
            // Positioned from the same shared main-view theme inset used by
            // the rendered chrome, so the input aligns with the app shell.
            // The input is vertically centered in the header (which has 28px content height)
            // Input height is ~22px (CURSOR_HEIGHT_LG=18 + CURSOR_MARGIN_Y*2=4)
            let input_height = menu_def.search.height;
            let input_x = px(menu_def.shell.header_padding_x);
            let context_offset_y = if main_view_has_context_zone {
                main_view_context_zone_height + menu_def.shell.header_gap
            } else {
                0.0
            };
            let input_y = px(menu_def.shell.header_padding_y + context_offset_y);
            let input_width = (width - (input_x * 2.)).max(px(0.));

            bounds.push(
                ComponentBounds::new(
                    "MainViewInput",
                    gpui::Bounds {
                        origin: gpui::point(input_x, input_y),
                        size: gpui::size(input_width, px(input_height)),
                    },
                )
                .with_type(ComponentType::Input)
                .with_padding(BoxModel::symmetric(0.0, 0.0)),
            );
        }

        if matches!(self.current_view, AppView::ScriptList) {
            let uses_split_preview = matches!(self.main_window_mode, MainWindowMode::Full);
            let list_width = if uses_split_preview {
                width * 0.5
            } else {
                width
            };

            if uses_split_preview {
                // Preview panel contents (right 50% of window)
                // Preview has its own padding, content starts at list_width + padding
                let preview_padding = 16.0_f32;
                let preview_left = list_width + px(preview_padding);
                let preview_width = width * 0.5 - px(preview_padding * 2.0);

                // Script path label (small text at top of preview)
                bounds.push(
                    ComponentBounds::new(
                        "ScriptPath",
                        gpui::Bounds {
                            origin: gpui::point(preview_left, content_top + px(8.)),
                            size: gpui::size(preview_width, px(16.)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::symmetric(2.0, 0.0)),
                );

                // Script title (large heading)
                bounds.push(
                    ComponentBounds::new(
                        "ScriptTitle",
                        gpui::Bounds {
                            origin: gpui::point(preview_left, content_top + px(32.)),
                            size: gpui::size(preview_width, px(32.)),
                        },
                    )
                    .with_type(ComponentType::Header)
                    .with_padding(BoxModel::symmetric(4.0, 0.0)),
                );

                // Description label
                bounds.push(
                    ComponentBounds::new(
                        "DescLabel", // Shortened
                        gpui::Bounds {
                            origin: gpui::point(preview_left, content_top + px(72.)),
                            size: gpui::size(px(80.), px(16.)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::uniform(2.0)),
                );

                // Description value
                bounds.push(
                    ComponentBounds::new(
                        "DescValue", // Shortened
                        gpui::Bounds {
                            origin: gpui::point(preview_left, content_top + px(92.)),
                            size: gpui::size(preview_width, px(20.)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::symmetric(2.0, 0.0)),
                );

                // Code Preview label
                bounds.push(
                    ComponentBounds::new(
                        "CodeLabel", // Shortened from CodePreviewLabel
                        gpui::Bounds {
                            origin: gpui::point(preview_left, content_top + px(130.)),
                            size: gpui::size(px(100.), px(16.)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::uniform(2.0)),
                );

                // Code preview area
                bounds.push(
                    ComponentBounds::new(
                        "CodePreview",
                        gpui::Bounds {
                            origin: gpui::point(preview_left, content_top + px(150.)),
                            size: gpui::size(preview_width, height - content_top - px(170.)),
                        },
                    )
                    .with_type(ComponentType::Container)
                    .with_padding(BoxModel::uniform(12.0)),
                );
            }

            // List item icons (left side of each list item)
            // Icons are typically 24x24, positioned with some padding from left edge
            // Item height is LIST_ITEM_HEIGHT, icon vertically centered: (LIST_ITEM_HEIGHT - 24) / 2 from top
            let item_height = px(LIST_ITEM_HEIGHT);
            for i in 0..5 {
                let item_top = content_top + px(i as f32 * LIST_ITEM_HEIGHT);
                if item_top + item_height > height {
                    break;
                }
                bounds.push(
                    ComponentBounds::new(
                        format!("Icon[{}]", i),
                        gpui::Bounds {
                            origin: gpui::point(
                                px(content_padding),
                                item_top + px((LIST_ITEM_HEIGHT - 24.0) / 2.0),
                            ),
                            size: gpui::size(px(24.), px(24.)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::uniform(0.0)),
                );
            }
        } // End of ScriptList-specific bounds

        {
            let footer_height = px(menu_def.footer.metrics.height_px);
            bounds.push(
                ComponentBounds::new(
                    "MainViewFooter",
                    gpui::Bounds {
                        origin: gpui::point(px(0.), (height - footer_height).max(px(0.))),
                        size: gpui::size(width, footer_height),
                    },
                )
                .with_type(ComponentType::Container)
                .with_padding(BoxModel::symmetric(
                    menu_def.footer.metrics.button_padding_y,
                    menu_def.footer.metrics.side_inset_px,
                )),
            );
        }

        bounds
    }
}
