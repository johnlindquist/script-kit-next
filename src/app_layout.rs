// Layout calculation methods for ScriptListApp
// This file is included via include!() macro in main.rs
// Contains: build_component_bounds, build_layout_info

impl ScriptListApp {
    /// Build component bounds for the debug grid overlay based on current view
    ///
    /// This creates approximate bounds for major UI components based on our
    /// known layout structure. Since GPUI doesn't expose layout bounds at runtime,
    /// we calculate them based on our layout constants.
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
            AppView::ClipboardHistoryView { .. } => "ClipboardHistory",
            AppView::AppLauncherView { .. } => "AppLauncher",
            AppView::WindowSwitcherView { .. } => "WindowSwitcher",
            AppView::DesignGalleryView { .. } => "DesignGallery",
            AppView::ScratchPadView { .. } => "ScratchPad",
            AppView::QuickTerminalView { .. } => "QuickTerminal",
            AppView::ActionsDialog => "ActionsDialog",
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

    /// Build complete layout info for the getLayoutInfo() SDK function.
    ///
    /// This provides AI agents with detailed component information including:
    /// - Bounds (position and size)
    /// - Box model (padding, margin, gap)
    /// - Flex properties (direction, grow, align)
    /// - Human-readable explanations of why components are sized as they are
    ///
    /// This is the "why" function - it explains the layout, not just shows it.
    pub fn build_layout_info(&self, _cx: &mut gpui::Context<Self>) -> protocol::LayoutInfo {
        use protocol::{LayoutComponentInfo, LayoutComponentType, LayoutInfo};

        // TODO: Get actual window size once we have access to window in this context
        // For now, use default values
        let window_width = 750.0_f32;
        let window_height = 500.0_f32;

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
            AppView::ClipboardHistoryView { .. } => "clipboardHistory",
            AppView::AppLauncherView { .. } => "appLauncher",
            AppView::WindowSwitcherView { .. } => "windowSwitcher",
            AppView::DesignGalleryView { .. } => "designGallery",
            AppView::ScratchPadView { .. } => "scratchPad",
            AppView::QuickTerminalView { .. } => "quickTerminal",
            AppView::ActionsDialog => "actionsDialog",
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
