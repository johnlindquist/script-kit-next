impl ScriptListApp {
    fn build_component_bounds(
        &self,
        window_size: gpui::Size<gpui::Pixels>,
    ) -> Vec<debug_grid::ComponentBounds> {
        use debug_grid::{BoxModel, ComponentBounds, ComponentType};

        let mut bounds = Vec::new();
        let width = window_size.width;
        let height = window_size.height;

        // Layout constants from panel.rs and list_item.rs
        // Header: py(HEADER_PADDING_Y=8) + max(input=22px, buttons=28px) + py(8) + divider(1px)
        // The buttons are 28px tall, input is 22px, so header content height is 28px
        // Total: 8 + 28 + 8 + 1 = 45px
        const HEADER_PADDING_Y: f32 = 8.0;
        const HEADER_PADDING_X: f32 = 16.0;
        const BUTTON_HEIGHT: f32 = 28.0;
        const DIVIDER_HEIGHT: f32 = 1.0;
        let header_height = px(HEADER_PADDING_Y * 2.0 + BUTTON_HEIGHT + DIVIDER_HEIGHT); // 45px

        // Content padding matches HEADER_PADDING_X
        let content_padding = HEADER_PADDING_X;

        // Main content area (below header)
        let content_top = header_height;
        let content_height = height - header_height;

        // Determine the current view type and build appropriate bounds
        let view_name = match &self.current_view {
            AppView::ScriptList => "ScriptList",
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
            AppView::ChatPrompt { .. } => "ChatPrompt",
            AppView::ClipboardHistoryView { .. } => "ClipboardHistory",
            AppView::PasteSequentiallyView { .. } => "PasteSequentially",
            AppView::AppLauncherView { .. } => "AppLauncher",
            AppView::WindowSwitcherView { .. } => "WindowSwitcher",
            AppView::DesignGalleryView { .. } => "DesignGallery",
            AppView::ScratchPadView { .. } => "ScratchPad",
            AppView::QuickTerminalView { .. } => "QuickTerminal",
            AppView::FileSearchView { .. } => "FileSearch",
            AppView::ThemeChooserView { .. } => "ThemeChooser",
            AppView::EmojiPickerView { .. } => "EmojiPicker",
            AppView::ActionsDialog => "ActionsDialog",
            AppView::WebcamView { .. } => "Webcam",
            AppView::CreationFeedback { .. } => "CreationFeedback",
            AppView::NamingPrompt { .. } => "NamingPrompt",
        };

        // Header bounds (includes padding + input + divider) - common to all views
        bounds.push(
            ComponentBounds::new(
                "Header",
                gpui::Bounds {
                    origin: gpui::point(px(0.), px(0.)),
                    size: gpui::size(width, header_height),
                },
            )
            .with_type(ComponentType::Header)
            .with_padding(BoxModel::symmetric(HEADER_PADDING_Y, content_padding)),
        );

        // Build view-specific bounds
        match &self.current_view {
            AppView::ScriptList => {
                // ScriptList has left panel (50%) + right preview panel (50%)
                let list_width = width * 0.5;
                let item_height = px(48.0);

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
                    let item_top = content_top + px(i as f32 * 48.0);
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
                    let item_height = px(48.0);
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
                        let item_top = content_top + px(i as f32 * 48.0);
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
                let item_height = px(48.0);
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
                    let item_top = content_top + px(i as f32 * 48.0);
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

        // Only add header detail bounds for ScriptList view (the original behavior)
        if matches!(self.current_view, AppView::ScriptList) {
            let list_width = width * 0.5;

            // Input field in header
            // Positioned at: px(HEADER_PADDING_X) = 16, py(HEADER_PADDING_Y) = 8
            // The input is vertically centered in the header (which has 28px content height)
            // Input height is ~22px (CURSOR_HEIGHT_LG=18 + CURSOR_MARGIN_Y*2=4)
            const INPUT_HEIGHT: f32 = 22.0;
            let input_x = px(content_padding);
            let input_y = px(HEADER_PADDING_Y + (BUTTON_HEIGHT - INPUT_HEIGHT) / 2.0); // Vertically centered
                                                                                       // Input takes flex-1, estimate it takes most of the header width before buttons
                                                                                       // Buttons area is roughly: Run(50) + divider(20) + Actions(70) + divider(20) + Logo(16) + padding(16) = ~192px
            let buttons_area_width = px(200.);
            let input_width = width - px(content_padding) - buttons_area_width;

            bounds.push(
                ComponentBounds::new(
                    "SearchInput",
                    gpui::Bounds {
                        origin: gpui::point(input_x, input_y),
                        size: gpui::size(input_width, px(INPUT_HEIGHT)),
                    },
                )
                .with_type(ComponentType::Input)
                .with_padding(BoxModel::symmetric(0.0, 0.0)),
            );

            // Header buttons (right side)
            // Buttons are h(28px) positioned at top of content area (after top padding)
            let button_height = px(BUTTON_HEIGHT);
            let button_y = px(HEADER_PADDING_Y); // Buttons at top of content area

            // Buttons layout from right to left:
            // [SearchInput flex-1] [Run ~45px] [|] [Actions ~70px] [|] [Logo 16px] [padding 16px]
            // Spacing: gap=12, divider ~8px each side = ~20px between groups
            let logo_size = px(16.);
            let right_padding = px(content_padding);

            // Logo (Script Kit icon) - rightmost, 16x16 vertically centered in button area
            let logo_x = width - right_padding - logo_size;
            let logo_y = px(HEADER_PADDING_Y + (BUTTON_HEIGHT - 16.0) / 2.0); // Vertically centered
            bounds.push(
                ComponentBounds::new(
                    "Lg", // Short name for Logo to fit in small space
                    gpui::Bounds {
                        origin: gpui::point(logo_x, logo_y),
                        size: gpui::size(logo_size, logo_size),
                    },
                )
                .with_type(ComponentType::Other)
                .with_padding(BoxModel::uniform(0.0)),
            );

            // Actions button - left of divider, left of logo
            // Actual button text "Actions ⌘K" is roughly 80-90px wide
            let actions_width = px(85.);
            let actions_x = logo_x - px(24.) - actions_width; // ~24px for divider + spacing

            bounds.push(
                ComponentBounds::new(
                    "Actions", // Shortened from ActionsButton
                    gpui::Bounds {
                        origin: gpui::point(actions_x, button_y),
                        size: gpui::size(actions_width, button_height),
                    },
                )
                .with_type(ComponentType::Button)
                .with_padding(BoxModel::symmetric(4.0, 8.0)),
            );

            // Run button - left of divider, left of Actions
            // Actual button text "Run ↵" is roughly 50-60px wide
            let run_width = px(55.);
            let run_x = actions_x - px(24.) - run_width; // ~24px for divider + spacing

            bounds.push(
                ComponentBounds::new(
                    "Run", // Shortened from RunButton
                    gpui::Bounds {
                        origin: gpui::point(run_x, button_y),
                        size: gpui::size(run_width, button_height),
                    },
                )
                .with_type(ComponentType::Button)
                .with_padding(BoxModel::symmetric(4.0, 8.0)),
            );

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

            // List item icons (left side of each list item)
            // Icons are typically 24x24, positioned with some padding from left edge
            // Item height is 48px, icon vertically centered: (48 - 24) / 2 = 12px from top
            let item_height = px(48.0);
            for i in 0..5 {
                let item_top = content_top + px(i as f32 * 48.0);
                if item_top + item_height > height {
                    break;
                }
                bounds.push(
                    ComponentBounds::new(
                        format!("Icon[{}]", i),
                        gpui::Bounds {
                            origin: gpui::point(px(content_padding), item_top + px(12.)),
                            size: gpui::size(px(24.), px(24.)),
                        },
                    )
                    .with_type(ComponentType::Other)
                    .with_padding(BoxModel::uniform(0.0)),
                );
            }
        } // End of ScriptList-specific bounds

        bounds
    }
}
